//! The `run` command in extract-only mode, end to end over a tiny gold
//! corpus, with a worker that cannot start: every document fails, and
//! every failure is scored, reported and gated - but never written as the
//! baseline, which would hold later runs to having read nothing.

use std::path::PathBuf;

use intern_bench::{
    baseline::Baseline,
    extract::ExtractOptions,
    markdown,
    report::Report,
    run::{EXIT_REGRESSED, Mode, RunOptions, run},
};
use serde_json::{Value, json};

fn gold() -> Value {
    json!({
        "schema_version": 1,
        "suite": "internbench",
        "documents": [
            {
                "id": "meeting-notice", "file": "meeting-notice.pdf", "kind": "notice",
                "format": "pdf", "text_layer": "native", "pages": 1,
                "categories": ["multi_column"],
                "recording": "pending",
                "gold": {
                    "document_type": "Notice of Annual Meeting", "document_date": "2026-05-04",
                    "date_role": "notice", "parties": ["Saltmarsh Boat Club"], "party_relation": "from",
                    "evidence": {"date_text": ["May 4, 2026"], "party_text": {"Saltmarsh Boat Club": ["SALTMARSH BOAT CLUB"]}}
                },
                "ocr_truth": null,
                "structure": {
                    "reading_order": ["Annual meeting", "Dock repairs", "Slip fees"],
                    "tables": [{"rows": [["Slip", "Fee"], ["A-12", "$840.00"]]}],
                    "key_values": [{"key": "Dated", "value": "May 4, 2026"}],
                    "expected_routes": {"1": "layout"}
                }
            },
            {
                "id": "scan-receipt", "file": "scan-receipt.png", "kind": "receipt",
                "format": "png", "text_layer": "scan", "pages": 1,
                "categories": ["low_resolution_scan"],
                "gold": {"document_type": "Receipt", "document_date": "2026-05-09", "date_role": "issuance"},
                "ocr_truth": {"pages": [{"page": 1, "text": "DATE 05/09/2026 AUTH 08812C"}],
                              "dates": ["05/09/2026"], "names": [], "identifiers": ["08812C"]}
            }
        ]
    })
}

struct Bench {
    directory: tempfile::TempDir,
}

impl Bench {
    fn new() -> Self {
        let bench = Self {
            directory: tempfile::tempdir().unwrap(),
        };
        std::fs::write(bench.path("gold.json"), gold().to_string()).unwrap();
        std::fs::create_dir_all(bench.path("generated")).unwrap();
        for file in ["meeting-notice.pdf", "scan-receipt.png"] {
            std::fs::write(bench.path(&format!("generated/{file}")), file).unwrap();
        }
        bench
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.path().join(name)
    }

    fn options(&self, output: &str, warm_up: bool) -> RunOptions {
        RunOptions {
            corpus: self.path("generated"),
            gold: self.path("gold.json"),
            manifest: None,
            only: Vec::new(),
            output: Some(self.path(output)),
            markdown: Some(self.path(&format!("{output}.md"))),
            baseline: None,
            write_baseline: None,
            latency_gate: None,
            mode: Mode::Extract {
                options: ExtractOptions {
                    worker: self.path("no-such-worker"),
                    warm_up,
                },
            },
            engine: Default::default(),
        }
    }

    fn report(&self, output: &str) -> Report {
        Report::parse(&std::fs::read(self.path(output)).unwrap()).unwrap()
    }
}

#[test]
fn a_worker_that_cannot_start_fails_the_warm_up() {
    let bench = Bench::new();
    let error = run(bench.options("report.json", true)).unwrap_err();
    assert!(
        error.contains("could not read the warm-up document"),
        "{error}"
    );
    assert!(!bench.path("report.json").exists());
}

#[test]
fn every_failed_extraction_is_scored_as_a_miss_and_gated_but_never_the_baseline() {
    let bench = Bench::new();
    let mut options = bench.options("report.json", false);
    options.write_baseline = Some(bench.path("baseline.json"));
    // Nothing completed: the report is written, the baseline refused.
    assert_eq!(run(options).unwrap(), EXIT_REGRESSED);
    assert!(!bench.path("baseline.json").exists());
    let report = bench.report("report.json");
    assert_eq!(report.mode, "extract");
    assert_eq!(report.timings_source, "measured");
    assert!(report.wall_ms.is_some());
    assert_eq!(report.summary.statuses["extraction_failed"], 2);
    // A pending recording is a model's business: extraction reads it.
    let notice = report.record("meeting-notice").unwrap();
    assert_eq!(notice.status, "extraction_failed");
    for key in [
        "reading_order_accuracy",
        "table_row_accuracy",
        "table_cell_recall",
        "kv_accuracy",
        "digest_recall",
    ] {
        assert_eq!(notice.scores[key], json!(0.0), "{key}");
    }
    // Nothing was read, so nothing was routed: as for a worker that sends
    // no layouts, the route is not judged.
    assert!(!notice.scores.contains_key("route_correct"));
    // The routes section still says the gold expected one.
    let routes = report.routes.as_ref().unwrap();
    assert_eq!((routes.expected_pages, routes.confusion.len()), (1, 0));
    let receipt = report.record("scan-receipt").unwrap();
    assert_eq!(receipt.scores["ocr_cer"], json!(1.0));
    assert_eq!(receipt.scores["ocr_identifier_accuracy"], json!(0.0));
    let structure = report.structure.as_ref().unwrap();
    assert_eq!(structure.aggregate.rows, 2);
    assert_eq!(structure.aggregate.reading_order_accuracy, Some(0.0));

    // Held to a baseline of the same failures, it passes.
    std::fs::write(
        bench.path("baseline.json"),
        Baseline::from_report(&report).to_json(),
    )
    .unwrap();
    let baseline = Baseline::parse(&std::fs::read(bench.path("baseline.json")).unwrap()).unwrap();
    assert_eq!(baseline.mode, "extract");
    assert_eq!(
        baseline.documents["meeting-notice"].values["kv_accuracy"],
        0.0
    );

    let page = std::fs::read_to_string(bench.path("report.json.md")).unwrap();
    assert_eq!(page, markdown::render(&report));
    assert!(page.starts_with("# InternBench: extract run"), "{page}");
    assert!(page.contains("## Extraction scorecard"), "{page}");
    assert!(
        page.contains("- **meeting-notice**: extraction_failed"),
        "{page}"
    );

    // Held to its own failures it passes; a replay baseline it is never
    // held to.
    let mut again = bench.options("again.json", false);
    again.baseline = Some(bench.path("baseline.json"));
    assert_eq!(run(again).unwrap(), 0);
    std::fs::write(
        bench.path("replay-baseline.json"),
        json!({"schema_version": 1, "mode": "replay", "documents": {}, "aggregate": {}, "latency": {}})
            .to_string(),
    )
    .unwrap();
    let mut mismatched = bench.options("mismatched.json", false);
    mismatched.baseline = Some(bench.path("replay-baseline.json"));
    assert_eq!(run(mismatched).unwrap(), EXIT_REGRESSED);
    let comparison = bench.report("mismatched.json").baseline.unwrap();
    assert!(
        comparison.failures[0]
            .contains("an extract-only run is held only to an extract-only baseline"),
        "{:?}",
        comparison.failures
    );
}

#[test]
fn bytes_the_manifest_does_not_vouch_for_are_never_extracted() {
    let bench = Bench::new();
    std::fs::write(
        bench.path("manifest.json"),
        json!({"schema_version": 1, "files": [
            {"file": "meeting-notice.pdf", "sha256": "not-these-bytes"}
        ]})
        .to_string(),
    )
    .unwrap();
    let mut options = bench.options("report.json", false);
    options.manifest = Some(bench.path("manifest.json"));
    let error = run(options).unwrap_err();
    assert!(
        error.contains("differ from the manifest or are not in it")
            && error.contains("meeting-notice.pdf")
            && error.contains("scan-receipt.png"),
        "{error}"
    );
    assert!(!bench.path("report.json").exists());
}
