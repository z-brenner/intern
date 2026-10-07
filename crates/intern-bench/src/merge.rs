//! Re-recording some documents without re-recording the corpus.
//!
//! A change that touches a few documents - a generator fix that changes
//! one file's bytes, a worker change to how one kind of scan is read -
//! makes only those documents stale. Recording the whole corpus again
//! would also move every other document's reply, because live inference
//! is not bit-identical from run to run. So the few are recorded live on
//! their own (`run --only ... --record`) and merged into the recording of
//! record here, the way `scripts/splice-recording.mjs` does for the
//! fixture corpus.
//!
//! Each document the second recording holds replaces the base's entry of
//! the same id, or is added. Every other entry of the base is kept as it
//! was. The merged documents are in the gold's order. Two recordings made
//! with different models, digest budgets or contexts are refused: their
//! replies answer different prompts, or the same prompt from a different
//! model, and a recording that mixes them measures nothing.

use std::{collections::BTreeSet, path::Path};

use crate::{
    gold::GoldFile,
    machine::utc_now,
    recording::{RecordedDocument, Recording},
};

/// What a merge did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MergeSummary {
    pub replaced: Vec<String>,
    pub added: Vec<String>,
    pub kept: usize,
    /// Why the merged entries' timings may not compare with the rest.
    pub warnings: Vec<String>,
}

/// Merges `add`'s documents (all of them, or the ids in `only`) into
/// `base`, in the gold's document order.
pub fn merge(
    base: &Recording,
    add: &Recording,
    gold: &GoldFile,
    only: &[String],
    note: Option<&str>,
) -> Result<(Recording, MergeSummary), String> {
    let base_sha = base.model.sha256.as_deref();
    let add_sha = add.model.sha256.as_deref();
    match (base_sha, add_sha) {
        (Some(base_sha), Some(add_sha)) if base_sha == add_sha => {}
        (Some(base_sha), Some(add_sha)) => {
            return Err(format!(
                "the recordings were made with different models (sha256 {} and {}); record again with the base's model",
                short(base_sha),
                short(add_sha)
            ));
        }
        _ => {
            return Err(
                "a recording does not say which model file made it (record with --model-path), so a merge cannot tell whether the replies are the same model's"
                    .to_owned(),
            );
        }
    }
    if base.budget_characters != add.budget_characters {
        return Err(format!(
            "the recordings were made with different digest budgets ({} and {} characters)",
            base.budget_characters, add.budget_characters
        ));
    }
    if base.engine != add.engine {
        return Err(
            "the recordings were made with different pipelines, retrieval or prompts".to_owned(),
        );
    }
    if base.context_tokens != add.context_tokens {
        let describe = |tokens: Option<usize>| {
            tokens.map_or_else(|| "unstated".to_owned(), |tokens| tokens.to_string())
        };
        return Err(format!(
            "the recordings were made with different contexts ({} and {} tokens)",
            describe(base.context_tokens),
            describe(add.context_tokens)
        ));
    }

    let order = gold
        .documents
        .iter()
        .map(|document| document.id.as_str())
        .collect::<Vec<_>>();
    let known = order.iter().copied().collect::<BTreeSet<_>>();
    if let Some(unknown) = only.iter().find(|id| !known.contains(id.as_str())) {
        return Err(format!(
            "--only names {unknown}, which the gold does not have"
        ));
    }
    if let Some(missing) = only.iter().find(|id| add.document(id).is_none()) {
        return Err(format!(
            "--only names {missing}, which the added recording does not have"
        ));
    }
    let incoming = add
        .documents
        .iter()
        .filter(|document| only.is_empty() || only.contains(&document.id))
        .collect::<Vec<_>>();
    if incoming.is_empty() {
        return Err("the added recording has no document to merge".to_owned());
    }
    for document in incoming.iter().copied().chain(&base.documents) {
        if !known.contains(document.id.as_str()) {
            return Err(format!(
                "{} is in a recording but not in the gold; merge against the gold the recordings were made for",
                document.id
            ));
        }
    }

    let mut summary = MergeSummary::default();
    for document in &incoming {
        if base.document(&document.id).is_some() {
            summary.replaced.push(document.id.clone());
        } else {
            summary.added.push(document.id.clone());
        }
    }
    let incoming_ids = incoming
        .iter()
        .map(|document| document.id.as_str())
        .collect::<BTreeSet<_>>();
    summary.kept = base
        .documents
        .iter()
        .filter(|document| !incoming_ids.contains(document.id.as_str()))
        .count();
    if base.machine != add.machine {
        summary.warnings.push(format!(
            "the merged documents were recorded on another machine ({}; the base on {}): their timings do not compare with the rest",
            add.machine.summary(),
            base.machine.summary()
        ));
    }

    let documents = order
        .iter()
        .filter_map(|id| -> Option<RecordedDocument> {
            incoming
                .iter()
                .find(|document| document.id == *id)
                .copied()
                .or_else(|| base.document(id))
                .cloned()
        })
        .collect::<Vec<_>>();
    let merged_ids = summary
        .replaced
        .iter()
        .chain(&summary.added)
        .cloned()
        .collect::<Vec<_>>();
    let mut line = format!(
        "{} recorded again {}{}",
        merged_ids.join(", "),
        if add.recorded_at.is_empty() {
            "at an unstated time".to_owned()
        } else {
            format!("at {}", add.recorded_at)
        },
        add.git_commit
            .as_deref()
            .map(|commit| format!(" from commit {}", short(commit)))
            .unwrap_or_default()
    );
    if let Some(note) = note.filter(|note| !note.trim().is_empty()) {
        line.push_str(": ");
        line.push_str(note.trim());
    } else if !add.note.is_empty() {
        line.push_str(&format!(" ({})", add.note));
    }
    let note = if base.note.is_empty() {
        line
    } else {
        format!("{}; {line}", base.note)
    };
    Ok((
        Recording {
            note,
            documents,
            ..base.clone()
        },
        summary,
    ))
}

/// The `merge-recordings` subcommand: reads, merges, writes, and says what
/// it did on standard error.
pub fn merge_command(
    base: &Path,
    add: &Path,
    output: &Path,
    gold: &Path,
    only: &[String],
    note: Option<&str>,
) -> Result<i32, String> {
    let (base, _) = Recording::load(base)?;
    let (add, _) = Recording::load(add)?;
    let (gold, _) = GoldFile::load(gold)?;
    let (merged, summary) = merge(&base, &add, &gold, only, note)?;
    for warning in &summary.warnings {
        eprintln!("warning: {warning}");
    }
    merged.save(output)?;
    eprintln!(
        "merged at {}: {} replaced ({}), {} added ({}), {} kept; wrote {}",
        utc_now(),
        summary.replaced.len(),
        summary.replaced.join(", "),
        summary.added.len(),
        summary.added.join(", "),
        summary.kept,
        output.display()
    );
    Ok(0)
}

fn short(value: &str) -> String {
    value.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gold::GoldDocument,
        machine::{MachineInfo, ModelInfo},
        recording::{RECORDING_SCHEMA_VERSION, RecordedExtraction},
    };

    fn gold(ids: &[&str]) -> GoldFile {
        GoldFile {
            schema_version: 1,
            suite: "internbench".into(),
            documents: ids
                .iter()
                .map(|id| GoldDocument {
                    id: (*id).into(),
                    file: format!("{id}.pdf"),
                    ..GoldDocument::default()
                })
                .collect(),
        }
    }

    fn document(id: &str, sha: &str) -> RecordedDocument {
        RecordedDocument {
            id: id.into(),
            file: format!("{id}.pdf"),
            sha256: Some(sha.into()),
            extraction: RecordedExtraction::Failed {
                code: "PDF_ENCRYPTED".into(),
            },
            exchanges: Vec::new(),
            timings: Default::default(),
            memory: Default::default(),
        }
    }

    fn recording(documents: Vec<RecordedDocument>) -> Recording {
        Recording {
            schema_version: RECORDING_SCHEMA_VERSION,
            suite: "internbench".into(),
            recorded_at: "2026-10-01T09:00:00Z".into(),
            model: ModelInfo {
                id: "intern-local".into(),
                sha256: Some("a".repeat(64)),
                ..ModelInfo::default()
            },
            context_tokens: Some(8_192),
            budget_characters: 12_000,
            engine: Default::default(),
            machine: MachineInfo::default(),
            git_commit: Some("0123456789abcdef".into()),
            worker: None,
            note: "the corpus".into(),
            documents,
        }
    }

    #[test]
    fn documents_are_replaced_or_added_by_id_in_the_golds_order() {
        let gold = gold(&["a", "b", "c", "d"]);
        let base = recording(vec![document("a", "a1"), document("c", "c1")]);
        let mut add = recording(vec![document("d", "d2"), document("a", "a2")]);
        add.recorded_at = "2026-10-07T10:00:00Z".into();
        add.git_commit = Some("fedcba9876543210".into());
        let (merged, summary) =
            merge(&base, &add, &gold, &[], Some("the PO scan's bytes changed")).unwrap();
        assert_eq!(
            merged
                .documents
                .iter()
                .map(|document| (document.id.as_str(), document.sha256.as_deref().unwrap()))
                .collect::<Vec<_>>(),
            vec![("a", "a2"), ("c", "c1"), ("d", "d2")]
        );
        assert_eq!(summary.replaced, vec!["a"]);
        assert_eq!(summary.added, vec!["d"]);
        assert_eq!(summary.kept, 1);
        assert_eq!(
            merged.note,
            "the corpus; a, d recorded again at 2026-10-07T10:00:00Z from commit fedcba987654: the PO scan's bytes changed"
        );
        assert_eq!(
            merged.recorded_at, base.recorded_at,
            "the base's own fields"
        );

        // Only the named documents of a fuller recording.
        let (merged, summary) = merge(&base, &add, &gold, &["d".into()], None).unwrap();
        assert_eq!(summary.added, vec!["d"]);
        assert!(summary.replaced.is_empty());
        assert_eq!(merged.document("a").unwrap().sha256.as_deref(), Some("a1"));
    }

    #[test]
    fn recordings_of_another_model_budget_or_context_are_refused() {
        let gold = gold(&["a"]);
        let base = recording(vec![document("a", "a1")]);
        let refused = |change: &dyn Fn(&mut Recording)| {
            let mut add = recording(vec![document("a", "a2")]);
            change(&mut add);
            merge(&base, &add, &gold, &[], None).unwrap_err()
        };
        assert!(
            refused(&|add| add.model.sha256 = Some("b".repeat(64))).contains("different models")
        );
        assert!(refused(&|add| add.model.sha256 = None).contains("--model-path"));
        assert!(refused(&|add| add.budget_characters = 9_000).contains("digest budgets"));
        assert!(refused(&|add| add.context_tokens = Some(16_384)).contains("contexts"));
    }

    #[test]
    fn ids_outside_the_gold_or_the_added_recording_are_refused() {
        let base = recording(vec![document("a", "a1")]);
        let add = recording(vec![document("z", "z1")]);
        assert!(
            merge(&base, &add, &gold(&["a"]), &[], None)
                .unwrap_err()
                .contains("z is in a recording but not in the gold")
        );
        let add = recording(vec![document("a", "a2")]);
        assert!(
            merge(&base, &add, &gold(&["a", "b"]), &["b".into()], None)
                .unwrap_err()
                .contains("which the added recording does not have")
        );
        assert!(
            merge(&base, &add, &gold(&["a"]), &["q".into()], None)
                .unwrap_err()
                .contains("which the gold does not have")
        );
    }

    #[test]
    fn a_merge_from_another_machine_warns_about_timings() {
        let base = recording(vec![document("a", "a1")]);
        let mut add = recording(vec![document("a", "a2")]);
        add.machine.cpu = Some("Another CPU".into());
        let (_, summary) = merge(&base, &add, &gold(&["a"]), &[], None).unwrap();
        assert_eq!(summary.warnings.len(), 1);
        assert!(summary.warnings[0].contains("Another CPU"));
    }
}
