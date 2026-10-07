//! The engine a run uses: the pipeline, and for the evidence pipeline how
//! its evidence is retrieved and named and what context its prompts are
//! fitted to - read from the command line, written into a recording's
//! header, and checked against it when the recording is replayed.
//!
//! ```text
//! --pipeline evidence|digest      the engine's pipeline (evidence by default)
//! --id-style stable|ordinal       how the evidence lines are named
//! --retrieval-tier auto|whole|small|normal|dense
//! --context-tokens N              the server's context, when not the app's
//! ```
//!
//! The evidence pipeline's own measurements are here too
//! ([`evidence_scores`]): how many facts a reply stated and how many
//! nothing supported, how many of the accepted ones a cited unit supported,
//! and how many cited ids the prompt never showed.

use std::collections::{BTreeMap, HashMap};

use intern_engine::{
    DocumentAnalysis, Engine, Pipeline,
    engine::min_evidence_context_tokens,
    prompt::evidence_prompt_version,
    retrieve::{IdStyle, RetrievalConfig},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::recording::Recording;

/// The command-line options [`EngineSettings::parse`] reads.
pub const KEYS: &[&str] = &["pipeline", "id-style", "retrieval-tier", "context-tokens"];

/// How a run's engine is configured.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineSettings {
    pub pipeline: Pipeline,
    /// The evidence pipeline's retrieval; also what the `context_*` scores
    /// measure, in either pipeline.
    pub retrieval: RetrievalConfig,
    /// The model server's context, when it is not the app's.
    pub context_tokens: Option<usize>,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            pipeline: Pipeline::Evidence,
            retrieval: RetrievalConfig::default(),
            context_tokens: None,
        }
    }
}

impl EngineSettings {
    pub fn parse(values: &HashMap<String, String>) -> Result<Self, String> {
        let mut settings = Self::default();
        if let Some(word) = values.get("pipeline") {
            settings.pipeline = Pipeline::parse(word)
                .ok_or_else(|| format!("--pipeline is digest or evidence, not {word}"))?;
        }
        if let Some(word) = values.get("id-style") {
            settings.retrieval.id_style = IdStyle::parse(word)
                .ok_or_else(|| format!("--id-style is stable or ordinal, not {word}"))?;
        }
        if let Some(word) = values.get("retrieval-tier") {
            settings.retrieval = settings.retrieval.clone().with_tier(word).ok_or_else(|| {
                format!("--retrieval-tier is auto, whole, small, normal or dense, not {word}")
            })?;
        }
        if let Some(value) = values.get("context-tokens") {
            // The instructions and the reply take most of a small context;
            // one with no room left for evidence would fail every document.
            let least = min_evidence_context_tokens();
            settings.context_tokens = Some(
                value
                    .parse::<usize>()
                    .ok()
                    .filter(|tokens| *tokens >= least)
                    .ok_or_else(|| {
                        format!("--context-tokens must be a number of at least {least}")
                    })?,
            );
        }
        Ok(settings)
    }

    /// An engine configured with these settings.
    pub fn configure(&self, engine: Engine) -> Engine {
        engine
            .with_pipeline(self.pipeline)
            .with_retrieval(self.retrieval.clone())
    }

    /// The context prompts are fitted to: the one given, or the app's.
    pub fn context(&self) -> Option<usize> {
        self.context_tokens
            .or(Some(intern_engine::server::CONTEXT_TOKENS as usize))
    }

    /// What a recording made with these settings says about them. The
    /// digest pipeline says nothing, so its recordings read as they always
    /// have.
    pub fn header(&self) -> PipelineHeader {
        match self.pipeline {
            Pipeline::Digest => PipelineHeader::default(),
            Pipeline::Evidence => PipelineHeader {
                pipeline: Some(Pipeline::Evidence),
                retrieval: Some(self.retrieval.fingerprint()),
                prompt_version: Some(evidence_prompt_version()),
            },
        }
    }
}

/// What a recording's header says about the engine that made it. Absent
/// fields are the digest pipeline.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PipelineHeader {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<Pipeline>,
    /// The retrieval configuration's fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<String>,
    /// The evidence prompt's fixed parts' fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_version: Option<String>,
}

/// How the configuration a recording was made with differs from the one
/// replay uses, or `None` when they agree; an error when they cannot be
/// compared at all - a recording of one pipeline replayed through the
/// other, whose every prompt would be one the model was never asked.
///
/// The rest is said, not refused, as a changed digest budget is: replay
/// builds every prompt as the engine does now, and each document whose
/// prompt the difference changes is `stale_prompt`.
pub fn configuration_change(
    recording: &Recording,
    settings: &EngineSettings,
) -> Result<Option<String>, String> {
    let recorded = recording.engine.pipeline.unwrap_or(Pipeline::Digest);
    if recorded != settings.pipeline {
        return Err(format!(
            "the recording was made with the {} pipeline; replay it with --pipeline {}",
            recorded.as_str(),
            recorded.as_str()
        ));
    }
    let mut changes = Vec::new();
    let budget = crate::replay::current_budget().max_characters;
    if recording.budget_characters != budget {
        changes.push(format!(
            "a digest budget of {} characters (now {budget})",
            recording.budget_characters
        ));
    }
    let context = settings.context();
    if recording.context_tokens != context {
        let describe = |tokens: Option<usize>| {
            tokens.map_or_else(|| "no stated".to_owned(), |tokens| tokens.to_string())
        };
        changes.push(format!(
            "{} context tokens (now {})",
            describe(recording.context_tokens),
            describe(context)
        ));
    }
    let now = settings.header();
    if recording.engine.retrieval != now.retrieval {
        changes.push(format!(
            "retrieval configuration {} (now {})",
            recording.engine.retrieval.as_deref().unwrap_or("unstated"),
            now.retrieval.as_deref().unwrap_or("unstated")
        ));
    }
    if recording.engine.prompt_version != now.prompt_version {
        changes.push(format!(
            "evidence prompt {} (now {})",
            recording
                .engine
                .prompt_version
                .as_deref()
                .unwrap_or("unstated"),
            now.prompt_version.as_deref().unwrap_or("unstated")
        ));
    }
    Ok((!changes.is_empty()).then(|| {
        format!(
            "the recording was made with {}; replay builds every prompt as the engine does now, so each document whose prompt that changes is stale_prompt - record again",
            changes.join(" and ")
        )
    }))
}

/// The scores [`evidence_scores`] adds to a record.
pub const EVIDENCE_SCORES: &[&str] = &[
    "facts_proposed",
    "facts_unsupported",
    "cited_support_rate",
    "unknown_ids",
    "miscited_ids",
];

/// The evidence pipeline's measurements of one analysis; nothing for an
/// analysis the digest pipeline made.
///
/// * `facts_proposed`, `facts_unsupported`: the facts the reply stated -
///   type, date, each party, subject, identifier, each key fact - and those
///   nothing the model was shown supports.
/// * `cited_support_rate`: of the accepted facts, the share a unit the
///   reply cited for them supports (the rest the context supports).
/// * `unknown_ids`: cited ids the prompt never showed; `miscited_ids`:
///   shown ids whose unit does not state the fact cited.
pub fn evidence_scores(analysis: &DocumentAnalysis) -> BTreeMap<String, Value> {
    let mut scores = BTreeMap::new();
    let Some(facts) = &analysis.facts else {
        return scores;
    };
    let support = &facts.support;
    let (proposed, unsupported) = support.proposed_and_unsupported();
    let (accepted, cited) = support.accepted_and_cited();
    scores.insert("facts_proposed".to_owned(), Value::from(proposed));
    scores.insert("facts_unsupported".to_owned(), Value::from(unsupported));
    if accepted > 0 {
        scores.insert(
            "cited_support_rate".to_owned(),
            Value::from(f64::from(cited) / f64::from(accepted)),
        );
    }
    scores.insert("unknown_ids".to_owned(), Value::from(support.unknown_ids));
    scores.insert("miscited_ids".to_owned(), Value::from(support.miscited_ids));
    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use intern_engine::{FactSupport, Support, ValidatedFacts};

    fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn the_settings_read_every_flag_and_refuse_unknown_words() {
        let settings = EngineSettings::parse(&values(&[
            ("pipeline", "evidence"),
            ("id-style", "ordinal"),
            ("retrieval-tier", "dense"),
            ("context-tokens", "16384"),
        ]))
        .unwrap();
        assert_eq!(settings.pipeline, Pipeline::Evidence);
        assert_eq!(settings.retrieval.id_style, IdStyle::Ordinal);
        assert_eq!(
            settings.retrieval.tier,
            intern_engine::retrieve::TierPolicy::Dense
        );
        assert_eq!(settings.context(), Some(16_384));
        let whole = EngineSettings::parse(&values(&[("retrieval-tier", "whole")])).unwrap();
        assert_eq!(whole.retrieval.whole_document_tokens, u32::MAX);
        assert_eq!(
            EngineSettings::parse(&values(&[])).unwrap(),
            EngineSettings::default()
        );
        for (key, value) in [
            ("pipeline", "legacy"),
            ("id-style", "handles"),
            ("retrieval-tier", "huge"),
            ("context-tokens", "12"),
            // The reply budget alone is 1,024 tokens.
            ("context-tokens", "1024"),
        ] {
            assert!(
                EngineSettings::parse(&values(&[(key, value)])).is_err(),
                "{key} {value}"
            );
        }
    }

    /// A digest recording's header says nothing new, so it is written and
    /// read exactly as before.
    #[test]
    fn only_the_evidence_pipeline_writes_a_header() {
        let digest = EngineSettings {
            pipeline: Pipeline::Digest,
            ..EngineSettings::default()
        };
        assert_eq!(digest.header(), PipelineHeader::default());
        assert_eq!(EngineSettings::default().pipeline, Pipeline::Evidence);
        assert_eq!(
            serde_json::to_string(&PipelineHeader::default()).unwrap(),
            "{}"
        );
        let evidence = EngineSettings {
            pipeline: Pipeline::Evidence,
            ..EngineSettings::default()
        };
        let header = evidence.header();
        assert_eq!(header.pipeline, Some(Pipeline::Evidence));
        assert_eq!(
            header.retrieval.as_deref(),
            Some(RetrievalConfig::default().fingerprint().as_str())
        );
        assert!(header.prompt_version.is_some());
        let ordinal = EngineSettings {
            retrieval: RetrievalConfig {
                id_style: IdStyle::Ordinal,
                ..RetrievalConfig::default()
            },
            ..evidence
        };
        assert_ne!(ordinal.header().retrieval, header.retrieval);
    }

    #[test]
    fn the_evidence_scores_count_support() {
        let mut analysis: DocumentAnalysis = serde_json::from_value(serde_json::json!({
            "filename": "x.pdf", "description": "", "status": "ready", "reviewReasons": [],
            "proposal": {"document_type": null, "document_date": null, "date_role": null,
                "parties": [], "party_relation": "none", "description": "", "confidence": 0.9,
                "evidence": {"date": null, "document_type": null, "parties": []}},
            "telemetry": {"sourceCharacters": 1, "digestCharacters": 1, "compressionRatio": 1.0,
                "distillMicros": 1, "inferenceMillis": 1}
        }))
        .unwrap();
        assert!(evidence_scores(&analysis).is_empty());
        analysis.facts = Some(ValidatedFacts {
            support: FactSupport {
                document_type: Support::Cited,
                document_date: Support::Context,
                parties: vec![Support::Cited, Support::Unsupported],
                subject: Support::Absent,
                unknown_ids: 2,
                ..FactSupport::default()
            },
            ..ValidatedFacts::default()
        });
        let scores = evidence_scores(&analysis);
        assert_eq!(scores["facts_proposed"], Value::from(4));
        assert_eq!(scores["facts_unsupported"], Value::from(1));
        assert_eq!(scores["cited_support_rate"], Value::from(2.0 / 3.0));
        assert_eq!(scores["unknown_ids"], Value::from(2));
        for key in scores.keys() {
            assert!(EVIDENCE_SCORES.contains(&key.as_str()));
        }
    }

    fn recording(header: &PipelineHeader) -> Recording {
        let mut value = serde_json::json!({
            "schema_version": 1,
            "context_tokens": intern_engine::server::CONTEXT_TOKENS,
            "budget_characters": crate::replay::current_budget().max_characters,
            "documents": [],
        });
        let header = serde_json::to_value(header).unwrap();
        for (key, field) in header.as_object().unwrap() {
            value[key] = field.clone();
        }
        serde_json::from_value(value).unwrap()
    }

    /// A recording is replayed through the pipeline that made it; a changed
    /// retrieval or prompt is said, as a changed budget is.
    #[test]
    fn replay_refuses_the_other_pipeline_and_names_a_changed_configuration() {
        let evidence = EngineSettings::default();
        let digest = EngineSettings {
            pipeline: Pipeline::Digest,
            ..EngineSettings::default()
        };
        let digest_recording = recording(&PipelineHeader::default());
        assert_eq!(configuration_change(&digest_recording, &digest), Ok(None));
        let refused = configuration_change(&digest_recording, &evidence).unwrap_err();
        assert!(refused.contains("--pipeline digest"), "{refused}");

        let evidence_recording = recording(&evidence.header());
        assert_eq!(
            configuration_change(&evidence_recording, &evidence),
            Ok(None)
        );
        let refused = configuration_change(&evidence_recording, &digest).unwrap_err();
        assert!(refused.contains("--pipeline evidence"), "{refused}");
        let ordinal = EngineSettings {
            retrieval: RetrievalConfig {
                id_style: IdStyle::Ordinal,
                ..RetrievalConfig::default()
            },
            ..evidence.clone()
        };
        let change = configuration_change(&evidence_recording, &ordinal)
            .unwrap()
            .unwrap();
        assert!(change.contains("retrieval configuration"), "{change}");
        let mut stale_prompt = evidence.header();
        stale_prompt.prompt_version = Some("0123456789ab".into());
        let change = configuration_change(&recording(&stale_prompt), &evidence)
            .unwrap()
            .unwrap();
        assert!(change.contains("evidence prompt 0123456789ab"), "{change}");
        // The header is written only when there is something to say.
        let written = serde_json::to_value(&digest_recording).unwrap();
        for key in ["pipeline", "retrieval", "prompt_version"] {
            assert!(written.get(key).is_none(), "{key}");
        }
        let written = serde_json::to_value(&evidence_recording).unwrap();
        assert_eq!(written["pipeline"], "evidence");
    }
}
