//! Scoring the corpus again from a recording: no worker, no model.
//!
//! Each document's recorded text goes through [`Engine::analyze`] - the
//! real entry point, token-fitting included - with the model replaced by a
//! [`LookupProposer`] that answers each prompt with the reply recorded for
//! it. Everything after the model (validation, evidence checks, inference
//! of roles and types, naming) runs as the code now is, so a change to any
//! of it is scored as a live run would score it.
//!
//! A prompt the recording has no reply for means the engine now asks
//! something the model was never asked: the document is `stale_prompt`,
//! and the run fails unless staleness is allowed. A document whose bytes
//! differ from the ones recorded is `stale_fixture`; one the recording
//! lacks is `unrecorded`. Timings and memory are the recording's, flagged.

use std::{collections::BTreeMap, path::Path};

use intern_engine::{DigestBudget, Engine};
use serde_json::Value;

use crate::{
    gold::GoldDocument,
    record::{
        COMPLETED, DocumentRecord, EXTRACTION_FAILED, MODEL_FAILED, Observation, PENDING,
        STALE_FIXTURE, STALE_PROMPT, UNRECORDED, scored_record,
    },
    recording::{ExchangeLog, LookupProposer, RecordedExtraction, Recording, sha256_hex},
    timing,
};

/// The generator's manifest: each generated file's SHA-256.
#[derive(Clone, Debug, Default)]
pub struct Manifest {
    pub files: BTreeMap<String, String>,
}

impl Manifest {
    /// Reads `{"files": [{"file", "sha256", ...}]}`, the shape the fixture
    /// generator writes, and tolerates the same facts nested or keyed by
    /// file name: anything carrying a `sha256` beside a `file`, `name`, or
    /// `path`, or an object of such entries keyed by file.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|error| format!("cannot parse manifest: {error}"))?;
        let mut manifest = Self::default();
        collect(&value, None, &mut manifest.files);
        if manifest.files.is_empty() {
            return Err("manifest lists no file with a sha256".to_owned());
        }
        Ok(manifest)
    }

    pub fn sha256(&self, file: &str) -> Option<&str> {
        self.files.get(file).map(String::as_str)
    }
}

fn collect(value: &Value, key: Option<&str>, files: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(object) => {
            if let Some(sha) = object.get("sha256").and_then(Value::as_str) {
                let name = ["file", "name", "path"]
                    .iter()
                    .find_map(|field| object.get(*field).and_then(Value::as_str))
                    .or(key);
                if let Some(name) = name {
                    files.insert(name.to_owned(), sha.to_owned());
                }
            }
            for (child_key, child) in object {
                if child.is_object() || child.is_array() {
                    collect(child, Some(child_key), files);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect(item, None, files);
            }
        }
        _ => {}
    }
}

/// The budget a recording was made with: the engine's default, or a
/// passthrough-free one of the recorded size.
pub fn budget_of(recording: &Recording) -> DigestBudget {
    let default = DigestBudget::default();
    if recording.budget_characters == default.max_characters {
        default
    } else {
        DigestBudget {
            passthrough_characters: recording.budget_characters,
            max_characters: recording.budget_characters,
        }
    }
}

pub fn replay_document(
    document: &GoldDocument,
    recording: &Recording,
    manifest: Option<&Manifest>,
    corpus: &Path,
    allow_stale: bool,
) -> DocumentRecord {
    let unscored = |status: &str, error: Option<String>| {
        let mut record = DocumentRecord::for_document(document, status);
        record.replayed = true;
        record.error = error;
        record
    };
    // Added or regenerated before anyone could record it: neither scored
    // nor counted against the run.
    if document.is_pending() {
        return unscored(PENDING, None);
    }
    let Some(recorded) = recording.document(&document.id) else {
        return unscored(UNRECORDED, None);
    };
    // The document must be the one the recording was made from. The
    // manifest says what the generator makes now, without the corpus
    // having to be generated; failing that, the file itself.
    let current = manifest
        .and_then(|manifest| manifest.sha256(&document.file).map(str::to_owned))
        .or_else(|| {
            std::fs::read(corpus.join(&document.file))
                .ok()
                .map(|bytes| sha256_hex(&bytes))
        });
    if let (Some(current), Some(was)) = (&current, &recorded.sha256)
        && current != was
    {
        return unscored(
            STALE_FIXTURE,
            Some(format!(
                "recorded from {}, now {}",
                short(was),
                short(current)
            )),
        );
    }

    let source = match &recorded.extraction {
        RecordedExtraction::Failed { code } => {
            return scored_record(
                document,
                Observation {
                    status: EXTRACTION_FAILED,
                    error: Some(code.clone()),
                    analysis: None,
                    source: None,
                    budget: budget_of(recording),
                    exchanges: &[],
                    last_prompt: None,
                    timings: recorded_timings(&recorded.timings),
                    timings_recorded: true,
                    memory: recorded.memory,
                    replayed: true,
                    stale: false,
                },
            );
        }
        RecordedExtraction::Parsed { source } => source,
    };

    let log = ExchangeLog::default();
    let lookup = LookupProposer::new(
        &recorded.exchanges,
        recording.context_tokens,
        allow_stale,
        log.clone(),
    );
    let misses = lookup.misses();
    let engine = Engine::with_proposer(Box::new(lookup)).with_budget(budget_of(recording));
    let result = engine.analyze(source, document.extension(), &[]);
    let (exchanges, last_prompt) = log.take();
    let missed = misses
        .lock()
        .map(|misses| misses.clone())
        .unwrap_or_default();
    let stale = !missed.is_empty();
    if stale && !allow_stale {
        let mut record = unscored(
            STALE_PROMPT,
            Some(format!(
                "prompt {} was never recorded; the recording answered {}",
                short(&missed[0]),
                recorded
                    .exchanges
                    .iter()
                    .map(|exchange| short(&exchange.prompt_sha256))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        );
        record.prompt_sha256 = exchanges
            .iter()
            .map(|exchange| exchange.prompt_sha256.clone())
            .collect();
        return record;
    }
    let (status, error, analysis) = match &result {
        Ok(analysis) => (COMPLETED, None, Some(analysis)),
        Err(error) => (MODEL_FAILED, Some(error.code().as_str().to_owned()), None),
    };
    scored_record(
        document,
        Observation {
            status,
            error,
            analysis,
            source: Some(source),
            budget: budget_of(recording),
            exchanges: &exchanges,
            last_prompt: last_prompt.as_deref(),
            timings: recorded_timings(&recorded.timings),
            timings_recorded: true,
            memory: recorded.memory,
            replayed: true,
            stale,
        },
    )
}

/// The recording's timings, with every metric this build knows present.
fn recorded_timings(recorded: &timing::Timings) -> timing::Timings {
    let mut timings = timing::empty();
    timings.extend(
        recorded
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    timings
}

fn short(sha: &str) -> String {
    sha.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_manifest_is_read_in_the_generators_shape_and_tolerated_in_others() {
        let generator = br#"{"schema_version": 1, "generator": {"node": "24"}, "files": [{"file": "a.pdf", "size": 3, "sha256": "aaa"}, {"file": "b.docx", "sha256": "bbb"}]}"#;
        let manifest = Manifest::parse(generator).unwrap();
        assert_eq!(manifest.sha256("a.pdf"), Some("aaa"));
        assert_eq!(manifest.sha256("b.docx"), Some("bbb"));

        let keyed = br#"{"files": {"c.png": {"sha256": "ccc", "size": 1}}}"#;
        assert_eq!(Manifest::parse(keyed).unwrap().sha256("c.png"), Some("ccc"));
        let nested =
            br#"{"documents": [{"id": "d", "files": [{"name": "d.eml", "sha256": "ddd"}]}]}"#;
        assert_eq!(
            Manifest::parse(nested).unwrap().sha256("d.eml"),
            Some("ddd")
        );
        assert!(Manifest::parse(br#"{"files": []}"#).is_err());
    }
}
