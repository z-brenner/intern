//! Extract-only: the parser worker alone, scored on what extraction
//! decides - no model, no endpoint, no recording.
//!
//! Each document goes through [`SupervisedWorker`] exactly as a live run
//! sends it (the same timeouts, the worker started on a throwaway document
//! first, the worker's own account of its stages), and is then scored on
//! the extraction alone: OCR against the drawn text, the structure the gold
//! gives (reading order, tables, labelled values, routes), and whether the
//! gold's evidence reaches the digest the engine would build. A document
//! is `completed` when the worker read it and `extraction_failed` when it
//! did not; a failure is a miss on every score its gold defines.
//!
//! Scoring is [`extraction_record`], a pure function of the gold, what the
//! worker returned and the timings, so it is tested on sources built in
//! memory; [`run`] only feeds it.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use intern_engine::{DigestBudget, DocumentSource, SupervisedWorker, retrieve::RetrievalConfig};
use serde_json::json;

use crate::{
    context::context_scores,
    gold::GoldDocument,
    live::{extract, micros_since, warm_up_worker},
    memory::{MemoryPeaks, MemorySampler},
    record::{COMPLETED, DocumentRecord, EXTRACTION_FAILED},
    score::{digest_recall, extraction_scores},
    stats::round,
    timing::{Measured, Timings, flatten},
};

pub struct ExtractOptions {
    pub worker: PathBuf,
    /// Start the worker on a throwaway document before the corpus.
    pub warm_up: bool,
}

/// What an extract-only run produced.
pub struct ExtractRun {
    pub records: Vec<DocumentRecord>,
    /// The whole run, warm-up included.
    pub wall_micros: u64,
}

/// The extract-only scores, in the order the report lists them.
pub const EXTRACTION_SCORES: &[&str] = &[
    "reading_order_accuracy",
    "table_row_accuracy",
    "table_cell_recall",
    "kv_accuracy",
    "route_correct",
    "ocr_cer",
    "ocr_cer_ci",
    "ocr_wer",
    "ocr_date_accuracy",
    "ocr_name_accuracy",
    "ocr_identifier_accuracy",
    "ocr_mean_confidence",
    "digest_recall",
    "context_type_recall",
    "context_date_recall",
    "context_party_recall",
    "context_recall",
    "context_fact_recall",
    "context_subject_recall",
];

/// One document's record from what its extraction produced: the source the
/// worker returned, or the code it failed with.
pub fn extraction_record(
    document: &GoldDocument,
    extracted: Result<&DocumentSource, &str>,
    timings: Timings,
    memory: MemoryPeaks,
) -> DocumentRecord {
    extraction_record_with(
        document,
        extracted,
        timings,
        memory,
        &RetrievalConfig::default(),
    )
}

/// [`extraction_record`], with the `context_*` scores measured for
/// `retrieval`.
pub fn extraction_record_with(
    document: &GoldDocument,
    extracted: Result<&DocumentSource, &str>,
    timings: Timings,
    memory: MemoryPeaks,
    retrieval: &RetrievalConfig,
) -> DocumentRecord {
    let (status, source, error) = match extracted {
        Ok(source) => (COMPLETED, Some(source), None),
        Err(code) => (EXTRACTION_FAILED, None, Some(code.to_owned())),
    };
    let mut record = DocumentRecord::for_document(document, status);
    record.error = error;
    record.timings = timings;
    record.memory = memory;
    let extraction = extraction_scores(document, source);
    record.scores = extraction.scores;
    if let Some(recall) = digest_recall(document, source, DigestBudget::default()) {
        record
            .scores
            .insert("digest_recall".into(), json!(round(recall, 4)));
    }
    let context = context_scores(document, source, retrieval);
    record.scores.extend(context.scores);
    record.timings.extend(context.timings);
    record.ocr = extraction.ocr;
    record.structure = extraction.structure;
    if let Some(source) = source {
        record.set_routes(source);
    }
    record
}

pub fn run(
    documents: &[GoldDocument],
    corpus: &Path,
    options: &ExtractOptions,
    retrieval: &RetrievalConfig,
) -> Result<ExtractRun, String> {
    let started = Instant::now();
    let worker = SupervisedWorker::new(&options.worker);
    if options.warm_up {
        warm_up_worker(&worker)?;
        eprintln!("worker started in {:.1} s", started.elapsed().as_secs_f64());
    }
    let mut records = Vec::new();
    for (index, document) in documents.iter().enumerate() {
        let path = corpus.join(&document.file);
        let sampler = MemorySampler::worker_only();
        let extraction_started = Instant::now();
        let (extracted, worker_timings) =
            extract(&worker, &format!("bench-{}", document.id), &path);
        let extraction_wall_micros = micros_since(extraction_started);
        let memory = sampler.finish();
        let timings = flatten(&Measured {
            extraction_wall_micros: Some(extraction_wall_micros),
            worker: worker_timings.as_ref(),
            ..Measured::default()
        });
        let record = extraction_record_with(
            document,
            extracted.as_ref().map_err(|failure| failure.code.as_str()),
            timings,
            memory,
            retrieval,
        );
        eprintln!(
            "[{}/{}] {} {} in {:.2} s{}",
            index + 1,
            documents.len(),
            document.id,
            record.status,
            extraction_wall_micros as f64 / 1e6,
            record
                .route_class
                .as_deref()
                .map(|class| format!(" ({class})"))
                .unwrap_or_default()
        );
        records.push(record);
    }
    worker.stop();
    Ok(ExtractRun {
        records,
        wall_micros: micros_since(started),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::gold::{
        GoldAnswer, GoldEvidence, KeyValueTruth, OcrTruth, OcrTruthPage, StructureTruth, TableTruth,
    };
    use crate::timing;
    use intern_engine::{
        PageOrigin, SourcePage,
        structure::{PageLayout, PageRoute, RouteSignals},
    };
    use serde_json::Value;

    fn notice() -> GoldDocument {
        GoldDocument {
            id: "notice".into(),
            file: "notice.pdf".into(),
            kind: "notice".into(),
            pages: 2,
            gold: GoldAnswer {
                document_type: Some("Notice of Annual Meeting".into()),
                document_date: Some("2026-05-04".into()),
                parties: Some(vec!["Saltmarsh Boat Club".into()]),
                evidence: GoldEvidence {
                    date_text: vec!["May 4, 2026".into()],
                    party_text: BTreeMap::from([(
                        "Saltmarsh Boat Club".to_owned(),
                        vec!["SALTMARSH BOAT CLUB".to_owned()],
                    )]),
                    ..GoldEvidence::default()
                },
                ..GoldAnswer::default()
            },
            ocr_truth: Some(OcrTruth {
                pages: vec![OcrTruthPage {
                    page: 2,
                    text: "Signed May 4, 2026 for Saltmarsh Boat Club".into(),
                }],
                dates: vec!["May 4, 2026".into()],
                names: vec!["Saltmarsh Boat Club".into()],
                identifiers: Vec::new(),
            }),
            structure: Some(StructureTruth {
                reading_order: vec![
                    "Annual meeting".into(),
                    "Dock repairs".into(),
                    "Slip fees".into(),
                ],
                tables: vec![TableTruth {
                    rows: vec![
                        vec!["Slip".into(), "Fee".into()],
                        vec!["A-12".into(), "$840.00".into()],
                    ],
                }],
                key_values: vec![KeyValueTruth {
                    key: "Dated".into(),
                    value: "May 4, 2026".into(),
                }],
                expected_routes: BTreeMap::from([(1, "layout".into()), (2, "ocr".into())]),
            }),
            ..GoldDocument::default()
        }
    }

    fn page(number: usize, text: &str, route: Option<PageRoute>) -> SourcePage {
        let mut page = SourcePage::new(
            number,
            text,
            if route == Some(PageRoute::Ocr) {
                PageOrigin::Ocr
            } else {
                PageOrigin::Native
            },
        );
        page.layout = route.map(|route| PageLayout {
            width: 6120,
            height: 7920,
            route,
            signals: RouteSignals::default(),
            blocks: Vec::new(),
        });
        page
    }

    fn fraction(record: &DocumentRecord, key: &str) -> Option<f64> {
        record.scores.get(key).and_then(Value::as_f64)
    }

    #[test]
    fn a_read_document_is_scored_on_extraction_alone() {
        let document = notice();
        let source = DocumentSource::from_pages(vec![
            page(
                1,
                "SALTMARSH BOAT CLUB\n\nDated: May 4, 2026\n\nAnnual meeting\n\nDock repairs\n\n| Slip | Fee |\n| A-12 | $840.00 |\n\nSlip fees",
                Some(PageRoute::Layout),
            ),
            page(
                2,
                "Signed May 4, 2026 for Saltmarsh Boat Club",
                Some(PageRoute::Ocr),
            ),
        ]);
        let mut timings = timing::empty();
        timings.insert("worker_total_ms".into(), json!(12.5));
        let record = extraction_record(&document, Ok(&source), timings, MemoryPeaks::default());
        assert_eq!(record.status, COMPLETED);
        for key in [
            "reading_order_accuracy",
            "table_row_accuracy",
            "table_cell_recall",
            "kv_accuracy",
            "route_correct",
            "ocr_date_accuracy",
            "ocr_name_accuracy",
            "digest_recall",
        ] {
            assert_eq!(fraction(&record, key), Some(1.0), "{key}");
        }
        assert_eq!(fraction(&record, "ocr_cer"), Some(0.0));
        assert_eq!(record.route_class.as_deref(), Some("ocr"));
        assert_eq!(record.page_routes, vec!["layout", "ocr"]);
        assert!(
            record.scores.values().all(|value| !value.is_boolean()),
            "extraction alone names nothing, so nothing is a rate"
        );
        assert!(record.readiness.is_none());
        assert_eq!(
            crate::report::summarize([&record]).review_rate,
            None,
            "nothing was named, so nothing was sent to review"
        );
        assert_eq!(timing::get(&record.timings, "worker_total_ms"), Some(12.5));

        // Today's worker: the same text, no layouts. Everything but the
        // route is scored the same way.
        let plain = DocumentSource::from_pages(
            source
                .pages
                .iter()
                .map(|page| {
                    let mut page = page.clone();
                    page.layout = None;
                    page
                })
                .collect(),
        );
        let unrouted = extraction_record(
            &document,
            Ok(&plain),
            timing::empty(),
            MemoryPeaks::default(),
        );
        assert_eq!(fraction(&unrouted, "route_correct"), None);
        assert_eq!(fraction(&unrouted, "kv_accuracy"), Some(1.0));
        assert_eq!(unrouted.route_class.as_deref(), Some("unrouted"));
        assert!(unrouted.page_routes.is_empty());
    }

    #[test]
    fn a_failed_extraction_misses_every_score_its_gold_defines() {
        let record = extraction_record(
            &notice(),
            Err("PDF_ENCRYPTED"),
            timing::empty(),
            MemoryPeaks::default(),
        );
        assert_eq!(record.status, EXTRACTION_FAILED);
        assert_eq!(record.error.as_deref(), Some("PDF_ENCRYPTED"));
        for key in [
            "reading_order_accuracy",
            "table_row_accuracy",
            "table_cell_recall",
            "kv_accuracy",
            "ocr_date_accuracy",
            "ocr_name_accuracy",
            "digest_recall",
        ] {
            assert_eq!(fraction(&record, key), Some(0.0), "{key}");
        }
        // Nothing was read, so nothing was routed.
        assert_eq!(fraction(&record, "route_correct"), None);
        assert_eq!(fraction(&record, "ocr_cer"), Some(1.0));
        assert_eq!(
            fraction(&record, "ocr_identifier_accuracy"),
            None,
            "no identifier drawn"
        );
        assert!(record.route_class.is_none());
        assert!(
            record
                .structure
                .as_ref()
                .is_some_and(|measure| measure.failed)
        );
    }
}
