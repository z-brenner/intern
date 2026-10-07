//! End to end through the `run` command in replay mode, over a tiny gold
//! corpus and a recording built here - no worker, no model.

use std::path::{Path, PathBuf};

use intern_bench::{
    compare,
    machine::{MachineInfo, ModelInfo},
    markdown,
    recording::{
        Exchange, RECORDING_SCHEMA_VERSION, RecordedDocument, RecordedExtraction, RecordedReply,
        Recording,
    },
    report::Report,
    run::{EXIT_REGRESSED, Mode, RunOptions, run},
    timing,
};
use intern_engine::{
    DateRole, DigestBudget, DocumentSource, Evidence, ModelProposal, ModelRequest, ModelTimings,
    PageOrigin, PartyRelation, SourcePage, distill, source_from_text,
};
use serde_json::{Value, json};

const INVOICE_TEXT: &str = "HALVORSEN FIXTURE WORKS, LLC\n18 Kettle Lane, Brackwater\n\nINVOICE\n\n\
    Invoice No.: INV-20417\nInvoice Date: March 4, 2026\nDue Date: April 3, 2026\n\n\
    Bill To: Quillon Ridge Bakery, 7 Orchard Row\n\n\
    Display fixtures, supplied and installed: $4,812.50\n\nTotal due: $4,812.50";

const NOTICE_TEXT: &str = "NOTICE OF RENT INCREASE\n\nDate of notice: January 12, 2026\n\n\
    To: Pell Andersby, Unit 4, 22 Wren Street\n\nUnder the lease dated June 1, 2024, the monthly \
    rent rises to $1,450.00 effective March 1, 2026.\n\nTorvane Property Group";

fn gold() -> Value {
    json!({
        "schema_version": 1,
        "suite": "internbench",
        "documents": [
            {
                "id": "invoice-alpha", "file": "invoice-alpha.pdf", "title": "Invoice",
                "kind": "invoice", "format": "pdf", "text_layer": "native", "pages": 1,
                "categories": ["invoice", "competing_dates"],
                "gold": {
                    "document_type": "Invoice", "document_date": "2026-03-04", "date_role": "invoice",
                    "forbidden_dates": [{"date": "2026-04-03", "why": "due date"}],
                    "parties": ["Halvorsen Fixture Works LLC"], "party_relation": "from",
                    "party_roles": [{"name": "Halvorsen Fixture Works LLC", "role": "issuer"}],
                    "forbidden_parties": [{"name": "Quillon Ridge Bakery", "why": "bill-to"}],
                    "description_facts": [["$4,812.50", "4812.50"]],
                    "description_forbidden": ["$1,200.00"],
                    "subject_terms": ["display fixtures"],
                    "expected_readiness": "ready",
                    "evidence": {"date_text": ["March 4, 2026"], "party_text": {"Halvorsen Fixture Works LLC": ["HALVORSEN FIXTURE WORKS, LLC"]}}
                },
                "ocr_truth": null
            },
            {
                "id": "notice-beta", "file": "notice-beta.pdf", "kind": "notice", "format": "pdf",
                "text_layer": "native", "pages": 1, "categories": ["notice"],
                "gold": {
                    "document_type": "Notice of Rent Increase", "document_date": "2026-01-12",
                    "parties": ["Pell Andersby"], "party_relation": "for", "expected_readiness": "ready"
                }
            },
            {
                "id": "letter-gamma", "file": "letter-gamma.pdf", "kind": "letter", "format": "pdf",
                "text_layer": "native", "pages": 2, "categories": ["letter"],
                "gold": {"document_type": "Letter", "document_date": "2026-02-02", "parties": [], "party_relation": "none"}
            },
            {
                "id": "scan-delta", "file": "scan-delta.png", "kind": "notice", "format": "png",
                "text_layer": "scan", "pages": 1, "categories": ["image_only_scan", "png"],
                "gold": {
                    "document_type": "Notice of Rent Increase", "document_date": "2026-01-12",
                    "parties": ["Pell Andersby"], "party_relation": "for", "expected_readiness": "either"
                },
                "ocr_truth": {
                    "pages": [{"page": 1, "text": "NOTICE OF RENT INCREASE\nDate of notice: January 12, 2026\nTo: Pell Andersby, Unit 4, 22 Wren Street\nUnder the lease dated June 1, 2024, the monthly rent rises to $1,450.00 effective March 1, 2026.\nTorvane Property Group"}],
                    "dates": ["January 12, 2026"], "names": ["Pell Andersby"], "identifiers": ["INV-20417"]
                }
            },
            {
                "id": "broken-epsilon", "file": "broken-epsilon.pdf", "kind": "invoice", "format": "pdf",
                "text_layer": "native", "pages": 1, "categories": ["invoice"],
                "gold": {"document_type": "Invoice", "document_date": "2026-05-05", "parties": ["Halvorsen Fixture Works LLC"], "party_relation": "from"}
            }
        ]
    })
}

fn invoice_reply() -> ModelProposal {
    ModelProposal {
        document_type: Some("Invoice".into()),
        document_date: Some("2026-03-04".into()),
        date_role: Some(DateRole::Invoice),
        parties: vec!["Halvorsen Fixture Works LLC".into()],
        party_relation: PartyRelation::From,
        description: "Invoice INV-20417 from Halvorsen Fixture Works LLC to Quillon Ridge Bakery for display fixtures supplied and installed, totalling $4,812.50.".into(),
        confidence: 0.93,
        needs_review: false,
        evidence: Evidence {
            date: Some("Invoice Date: March 4, 2026".into()),
            document_type: Some("INVOICE".into()),
            parties: vec!["HALVORSEN FIXTURE WORKS, LLC".into()],
        },
    }
}

fn notice_reply() -> ModelProposal {
    ModelProposal {
        document_type: Some("Notice of Rent Increase".into()),
        document_date: Some("2026-01-12".into()),
        date_role: Some(DateRole::Notice),
        parties: vec!["Pell Andersby".into()],
        party_relation: PartyRelation::For,
        description:
            "Notice to Pell Andersby that the monthly rent rises to $1,450.00 from March 1, 2026."
                .into(),
        confidence: 0.9,
        needs_review: false,
        evidence: Evidence {
            date: Some("Date of notice: January 12, 2026".into()),
            document_type: Some("NOTICE OF RENT INCREASE".into()),
            parties: vec!["To: Pell Andersby, Unit 4, 22 Wren Street".into()],
        },
    }
}

fn prompt_sha(source: &DocumentSource) -> String {
    ModelRequest::from_digest(&distill(source, DigestBudget::default())).sha256()
}

fn exchange(sha: String, reply: RecordedReply) -> Exchange {
    Exchange {
        prompt_sha256: sha,
        prompt_characters: 1_000,
        reply,
        model_timings: Some(ModelTimings {
            prompt_tokens: 900,
            cached_tokens: 400,
            prefill_micros: 3_000_000,
            generated_tokens: 120,
            generation_micros: 6_000_000,
        }),
        wall_micros: 9_100_000,
    }
}

fn recorded_document(
    id: &str,
    file: &str,
    extraction: RecordedExtraction,
    exchanges: Vec<Exchange>,
    total_ms: f64,
) -> RecordedDocument {
    let mut timings = timing::empty();
    timings.insert("total_ms".into(), json!(total_ms));
    timings.insert("worker_ocr_ms".into(), json!(total_ms / 4.0));
    RecordedDocument {
        id: id.into(),
        file: file.into(),
        sha256: Some(format!("sha-of-{id}")),
        extraction,
        exchanges,
        timings,
        memory: Default::default(),
    }
}

fn scan_source() -> DocumentSource {
    let mut page = SourcePage::new(
        1,
        "NOTICE OF RENT lNCREASE\nDate of notice: January 12, 2026\n\nTo: Pell Andersby, Unit 4, 22 Wren Street\n\
         Under the lease dated June 1, 2024, the monthly rent rises to $1,450.00 effective March 1, 2026.\n\
         Torvane Property Group",
        PageOrigin::Ocr,
    );
    page.ocr_confidence = Some(88);
    DocumentSource::from_pages(vec![page])
}

fn recording() -> Recording {
    let invoice = source_from_text(INVOICE_TEXT);
    let notice = source_from_text(NOTICE_TEXT);
    let scan = scan_source();
    Recording {
        schema_version: RECORDING_SCHEMA_VERSION,
        suite: "internbench".into(),
        recorded_at: "2026-10-01T09:00:00Z".into(),
        model: ModelInfo {
            id: "intern-local".into(),
            ..ModelInfo::default()
        },
        context_tokens: Some(8_192),
        budget_characters: DigestBudget::default().max_characters,
        machine: MachineInfo {
            cpu: Some("Recorded CPU".into()),
            logical_cores: Some(8),
            total_ram_mb: Some(16_000),
            os: "linux".into(),
            os_version: None,
        },
        git_commit: Some("0123456789abcdef".into()),
        worker: Some("intern-worker".into()),
        note: "built by the test".into(),
        documents: vec![
            recorded_document(
                "invoice-alpha",
                "invoice-alpha.pdf",
                RecordedExtraction::Parsed {
                    source: invoice.clone(),
                },
                vec![exchange(
                    prompt_sha(&invoice),
                    RecordedReply::Proposed {
                        proposal: invoice_reply(),
                        token_confidence: None,
                    },
                )],
                12_000.0,
            ),
            // Recorded against a prompt the engine does not build.
            recorded_document(
                "notice-beta",
                "notice-beta.pdf",
                RecordedExtraction::Parsed { source: notice },
                vec![exchange(
                    "0".repeat(64),
                    RecordedReply::Proposed {
                        proposal: notice_reply(),
                        token_confidence: None,
                    },
                )],
                8_000.0,
            ),
            recorded_document(
                "scan-delta",
                "scan-delta.png",
                RecordedExtraction::Parsed {
                    source: scan.clone(),
                },
                vec![exchange(
                    prompt_sha(&scan),
                    RecordedReply::Proposed {
                        proposal: notice_reply(),
                        token_confidence: None,
                    },
                )],
                30_000.0,
            ),
            recorded_document(
                "broken-epsilon",
                "broken-epsilon.pdf",
                RecordedExtraction::Failed {
                    code: "PDF_ENCRYPTED".into(),
                },
                Vec::new(),
                50.0,
            ),
        ],
    }
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
        recording().save(&bench.path("recording.json")).unwrap();
        bench
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.path().join(name)
    }

    fn replay(&self, output: &str, configure: impl FnOnce(&mut RunOptions)) -> (i32, Report) {
        let mut options = self.options(output);
        configure(&mut options);
        let exit = run(options).unwrap();
        (exit, read_report(&self.path(output)))
    }

    fn options(&self, output: &str) -> RunOptions {
        RunOptions {
            // The corpus is never generated here: replay needs only the
            // recording.
            corpus: self.path("generated"),
            gold: self.path("gold.json"),
            manifest: None,
            only: Vec::new(),
            output: Some(self.path(output)),
            markdown: None,
            baseline: None,
            write_baseline: None,
            latency_gate: None,
            mode: Mode::Replay {
                recording: self.path("recording.json"),
                allow_stale: false,
            },
        }
    }
}

fn read_report(path: &Path) -> Report {
    Report::parse(&std::fs::read(path).unwrap()).unwrap()
}

fn scores(report: &Report, id: &str) -> Value {
    serde_json::to_value(&report.record(id).unwrap().scores).unwrap()
}

#[test]
fn a_replay_scores_what_it_can_and_fails_on_what_it_cannot() {
    let bench = Bench::new();
    let (exit, report) = bench.replay("report.json", |_| {});
    assert_eq!(
        exit, EXIT_REGRESSED,
        "a stale prompt and an unrecorded document"
    );
    assert_eq!(report.mode, "replay");
    assert_eq!(report.timings_source, "recorded");
    assert_eq!(report.machine.cpu.as_deref(), Some("Recorded CPU"));

    let status = |id: &str| report.record(id).unwrap().status.clone();
    assert_eq!(status("invoice-alpha"), "completed");
    assert_eq!(status("notice-beta"), "stale_prompt");
    assert_eq!(status("letter-gamma"), "unrecorded");
    assert_eq!(status("scan-delta"), "completed");
    assert_eq!(status("broken-epsilon"), "extraction_failed");

    let invoice = report.record("invoice-alpha").unwrap();
    assert_eq!(
        invoice.filename.as_deref(),
        Some("2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf")
    );
    assert!(invoice.replayed && invoice.timings_recorded && !invoice.stale);
    let invoice_scores = scores(&report, "invoice-alpha");
    for key in [
        "filename_correct",
        "type_correct",
        "date_correct",
        "date_role_correct",
        "parties_correct",
        "relation_correct",
        "party_role_correct",
        "readiness_match",
        "description_complete",
        "description_factual",
        "description_specific",
    ] {
        assert_eq!(invoice_scores[key], json!(true), "{key}: {invoice_scores}");
    }
    for key in [
        "date_forbidden",
        "party_forbidden",
        "unsafe_ready",
        "needless_review",
    ] {
        assert_eq!(invoice_scores[key], json!(false), "{key}");
    }
    assert_eq!(invoice_scores["evidence_recall"], json!(1.0));
    assert_eq!(invoice_scores["digest_recall"], json!(1.0));
    assert_eq!(invoice_scores["prompt_recall"], json!(1.0));
    assert_eq!(
        invoice_scores["description_unsupported"],
        json!(0),
        "{:?}",
        invoice.claims
    );
    assert_eq!(
        invoice.timings["total_ms"],
        json!(12_000.0),
        "the recording's timings"
    );
    assert_eq!(invoice.prompt_sha256.len(), 1);

    // Stale and unrecorded documents carry no scores at all.
    assert!(report.record("notice-beta").unwrap().scores.is_empty());
    assert!(
        report
            .record("notice-beta")
            .unwrap()
            .error
            .as_deref()
            .unwrap()
            .contains("never recorded")
    );
    assert!(report.record("letter-gamma").unwrap().scores.is_empty());

    // A document the worker could not read is a miss, not a gap.
    let broken = scores(&report, "broken-epsilon");
    assert_eq!(broken["filename_correct"], json!(false));
    assert!(broken.get("description_factual").is_none());
    assert_eq!(
        report
            .record("broken-epsilon")
            .unwrap()
            .readiness
            .as_deref(),
        Some("failed")
    );

    // The scan is measured against what was drawn on it.
    let scan = report.record("scan-delta").unwrap();
    let ocr = scan.ocr.as_ref().unwrap();
    assert_eq!(ocr.pages_compared, 1);
    assert_eq!(ocr.dates_found, 1);
    assert_eq!(ocr.identifiers_found, 0);
    let scan_scores = scores(&report, "scan-delta");
    // One letter misread: "lNCREASE".
    assert_eq!(ocr.char_distance, 1);
    assert!(scan_scores["ocr_cer"].as_f64().unwrap() < 0.01);
    assert_eq!(scan_scores["ocr_mean_confidence"], json!(88.0));
    assert!(
        scan_scores.get("readiness_match").is_none(),
        "either is fine"
    );

    let summary = &report.summary;
    assert_eq!(summary.documents, 5);
    assert_eq!(summary.scored, 3);
    assert_eq!(summary.rates["filename_correct"].total, 3);
    assert_eq!(summary.statuses["stale_prompt"], 1);
    assert_eq!(
        report.latency.overall["total_ms"].count, 2,
        "only completed documents: unscorable ones took no time, the failed one took time to fail"
    );
    assert_eq!(report.ocr.documents.len(), 1);
    assert_eq!(report.groups["category"]["png"].documents, 1);
}

#[test]
fn allowing_staleness_scores_the_stale_prompt_and_marks_it() {
    let bench = Bench::new();
    let (exit, report) = bench.replay("report.json", |options| {
        options.mode = Mode::Replay {
            recording: bench.path("recording.json"),
            allow_stale: true,
        };
        options.only = vec!["invoice-alpha".into(), "notice-beta".into()];
    });
    assert_eq!(exit, 0);
    let notice = report.record("notice-beta").unwrap();
    assert_eq!(notice.status, "completed");
    assert!(notice.stale);
    assert_eq!(
        notice.filename.as_deref(),
        Some("2026-01-12 Notice of Rent Increase for Pell Andersby.pdf")
    );
    assert_eq!(report.corpus.documents, 2);
    assert_eq!(report.corpus.only, vec!["invoice-alpha", "notice-beta"]);
}

#[test]
fn a_manifest_that_disagrees_with_the_recording_marks_the_document_stale() {
    let bench = Bench::new();
    std::fs::write(
        bench.path("manifest.json"),
        json!({"schema_version": 1, "files": [
            {"file": "invoice-alpha.pdf", "sha256": "sha-of-invoice-alpha"},
            {"file": "scan-delta.png", "sha256": "regenerated"}
        ]})
        .to_string(),
    )
    .unwrap();
    let (exit, report) = bench.replay("report.json", |options| {
        options.manifest = Some(bench.path("manifest.json"));
        options.only = vec!["invoice-alpha".into(), "scan-delta".into()];
    });
    assert_eq!(exit, EXIT_REGRESSED);
    assert_eq!(report.record("invoice-alpha").unwrap().status, "completed");
    assert_eq!(report.record("scan-delta").unwrap().status, "stale_fixture");
    assert!(report.corpus.manifest_sha256.is_some());
}

#[test]
fn a_document_the_manifest_does_not_vouch_for_is_stale_when_there_is_no_file() {
    let bench = Bench::new();
    // The manifest lists one document; the other is in the gold and the
    // recording but nowhere a replay could check its bytes.
    std::fs::write(
        bench.path("manifest.json"),
        json!({"schema_version": 1, "files": [
            {"file": "invoice-alpha.pdf", "sha256": "sha-of-invoice-alpha"}
        ]})
        .to_string(),
    )
    .unwrap();
    let (exit, report) = bench.replay("report.json", |options| {
        options.manifest = Some(bench.path("manifest.json"));
        options.only = vec!["invoice-alpha".into(), "notice-beta".into()];
    });
    assert_eq!(exit, EXIT_REGRESSED);
    assert_eq!(report.record("invoice-alpha").unwrap().status, "completed");
    assert_eq!(
        report.record("notice-beta").unwrap().status,
        "stale_fixture"
    );
}

#[test]
fn a_subset_run_never_overwrites_a_baseline_that_covers_more() {
    let bench = Bench::new();
    let (exit, _) = bench.replay("full.json", |options| {
        options.only = vec!["invoice-alpha".into(), "scan-delta".into()];
        options.write_baseline = Some(bench.path("baseline.json"));
    });
    assert_eq!(exit, 0);
    let before = std::fs::read(bench.path("baseline.json")).unwrap();

    let mut options = bench.options("subset.json");
    options.only = vec!["invoice-alpha".into()];
    options.write_baseline = Some(bench.path("baseline.json"));
    let error = run(options).unwrap_err();
    assert!(error.contains("would drop 1 document"), "{error}");
    assert_eq!(std::fs::read(bench.path("baseline.json")).unwrap(), before);

    // The same subset may rewrite a baseline of exactly that subset.
    let (exit, _) = bench.replay("again.json", |options| {
        options.only = vec!["invoice-alpha".into(), "scan-delta".into()];
        options.write_baseline = Some(bench.path("baseline.json"));
    });
    assert_eq!(exit, 0);
}

#[test]
fn a_baseline_holds_replay_to_every_document_and_compare_names_the_flip() {
    let bench = Bench::new();
    let scorable = |options: &mut RunOptions| {
        options.only = vec![
            "invoice-alpha".into(),
            "scan-delta".into(),
            "broken-epsilon".into(),
        ];
    };
    let (exit, _) = bench.replay("before.json", |options| {
        scorable(options);
        options.write_baseline = Some(bench.path("baseline.json"));
        options.markdown = Some(bench.path("before.md"));
    });
    assert_eq!(exit, 0);
    let (exit, report) = bench.replay("same.json", |options| {
        scorable(options);
        options.baseline = Some(bench.path("baseline.json"));
    });
    assert_eq!(exit, 0, "{:?}", report.baseline);
    assert!(report.baseline.as_ref().unwrap().passed);

    // The reviewed answer moves; the recorded reply no longer matches it.
    let mut moved = gold();
    moved["documents"][0]["gold"]["document_date"] = json!("2026-03-05");
    std::fs::write(bench.path("gold.json"), moved.to_string()).unwrap();
    let (exit, after) = bench.replay("after.json", |options| {
        scorable(options);
        options.baseline = Some(bench.path("baseline.json"));
    });
    assert_eq!(exit, EXIT_REGRESSED);
    let comparison = after.baseline.as_ref().unwrap();
    assert_eq!(comparison.gates, vec!["documents"]);
    assert!(
        comparison
            .failures
            .contains(&"invoice-alpha: filename_correct was true, now false".to_owned()),
        "{:?}",
        comparison.failures
    );

    let before = read_report(&bench.path("before.json"));
    let diff = compare::compare(&before, &after);
    assert!(
        diff.broken
            .iter()
            .any(|flip| flip.id == "invoice-alpha" && flip.score == "date_correct")
    );
    let rendered = compare::render(&diff);
    assert!(rendered.contains("## Broken"), "{rendered}");

    // The page a person reads leads with the scorecard and names the miss.
    let page = markdown::render(&after);
    assert!(page.starts_with("# InternBench: replay run\n"), "{page}");
    // The unreadable document counts as a miss: the corpus is the
    // denominator.
    assert!(
        page.contains("| Filename (the whole name) | 1/3 (33.3%) |"),
        "{page}"
    );
    assert!(page.contains("## Safety"));
    assert!(page.contains("- **invoice-alpha**: expected `2026-03-05 Invoice from Halvorsen Fixture Works LLC.pdf`, got `2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf` · **filed without review**"), "{page}");
    assert!(page.contains("## Baseline: FAILED (gates: documents)"));
    assert!(page.contains("timings and memory are the recording's"));
    let before_page = std::fs::read_to_string(bench.path("before.md")).unwrap();
    assert!(
        before_page.contains("| Filename (the whole name) | 2/3 (66.7%) |"),
        "{before_page}"
    );
    assert!(before_page.contains("## OCR"));
}

/// A long, information-dense memo: every paragraph distinct, so the
/// digest budget decides what reaches the prompt.
fn long_text() -> String {
    let mut text = String::from("MEMORANDUM OF ANNUAL PLANT REVIEW\n\nDate: May 6, 2026\n\n");
    for index in 0..60 {
        text.push_str(&format!(
            "Section {index}. Line {index} at the Orrin Vale plant ran {} shifts in week {}, \
             producing {} cases of preserves with a reject rate of {}.{} per cent; the crew lead \
             for bay {} logged {} maintenance tickets and {} safety observations.\n\n",
            3 + index % 4,
            index + 1,
            1_200 + index * 37,
            index % 7,
            index % 10,
            index % 9 + 1,
            index % 5,
            index % 3
        ));
    }
    text
}

/// A digest budget or context changed in code since the recording: replay
/// builds prompts as the engine does now, so a document whose prompt that
/// changes is stale - and the run says why, loudly.
#[test]
fn a_budget_changed_since_the_recording_makes_long_documents_stale() {
    let bench = Bench::new();
    let mut gold = gold();
    gold["documents"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "id": "memo-long", "file": "memo-long.txt", "kind": "memo", "format": "txt",
            "text_layer": "native", "pages": 1, "categories": ["information_dense"],
            "gold": {"document_type": "Memorandum", "document_date": "2026-05-06", "parties": [], "party_relation": "none"}
        }));
    std::fs::write(bench.path("gold.json"), gold.to_string()).unwrap();
    let long = source_from_text(long_text());
    // Recorded when the budget let a document this long through whole.
    let old_budget = DigestBudget {
        passthrough_characters: 40_000,
        max_characters: 40_000,
    };
    assert_ne!(
        distill(&long, old_budget).text,
        distill(&long, DigestBudget::default()).text,
        "the test needs a document the budget changes"
    );
    let mut recorded = recording();
    recorded.budget_characters = old_budget.max_characters;
    recorded.documents.push(recorded_document(
        "memo-long",
        "memo-long.txt",
        RecordedExtraction::Parsed {
            source: long.clone(),
        },
        vec![exchange(
            ModelRequest::from_digest(&distill(&long, old_budget)).sha256(),
            RecordedReply::Proposed {
                proposal: notice_reply(),
                token_confidence: None,
            },
        )],
        20_000.0,
    ));
    recorded.save(&bench.path("recording.json")).unwrap();

    let (exit, report) = bench.replay("report.json", |options| {
        options.only = vec!["invoice-alpha".into(), "memo-long".into()];
        options.markdown = Some(bench.path("report.md"));
    });
    assert_eq!(exit, EXIT_REGRESSED);
    assert_eq!(report.record("memo-long").unwrap().status, "stale_prompt");
    assert_eq!(
        report.record("invoice-alpha").unwrap().status,
        "completed",
        "a short document's prompt does not depend on the budget"
    );
    let change = report
        .recording
        .as_ref()
        .unwrap()
        .configuration_change
        .clone()
        .unwrap();
    assert!(change.contains("40000 characters"), "{change}");
    let page = std::fs::read_to_string(bench.path("report.md")).unwrap();
    assert!(
        page.contains("> **Warning:** the recording was made with"),
        "{page}"
    );
}

/// `--allow-stale` scores a stale prompt; it cannot rescue a document whose
/// bytes changed, and the page says which scores came from stale replies.
#[test]
fn allowing_staleness_never_passes_a_changed_fixture_and_is_shown() {
    let bench = Bench::new();
    std::fs::write(
        bench.path("manifest.json"),
        json!({"files": [
            {"file": "invoice-alpha.pdf", "sha256": "sha-of-invoice-alpha"},
            {"file": "notice-beta.pdf", "sha256": "sha-of-notice-beta"},
            {"file": "scan-delta.png", "sha256": "regenerated"}
        ]})
        .to_string(),
    )
    .unwrap();
    let (exit, report) = bench.replay("report.json", |options| {
        options.mode = Mode::Replay {
            recording: bench.path("recording.json"),
            allow_stale: true,
        };
        options.manifest = Some(bench.path("manifest.json"));
        options.only = vec![
            "invoice-alpha".into(),
            "notice-beta".into(),
            "scan-delta".into(),
        ];
        options.markdown = Some(bench.path("report.md"));
    });
    assert_eq!(
        exit, EXIT_REGRESSED,
        "stale_fixture fails whatever is allowed"
    );
    assert_eq!(report.record("scan-delta").unwrap().status, "stale_fixture");
    assert!(report.record("notice-beta").unwrap().stale);
    let page = std::fs::read_to_string(bench.path("report.md")).unwrap();
    assert!(
        page.contains("1 scored from stale replies (--allow-stale)"),
        "{page}"
    );
    assert!(
        page.contains("except those of the 1 document(s) scored from replies to prompts the engine no longer builds"),
        "{page}"
    );
    assert!(
        page.contains("Scored from replies to prompts the engine no longer builds (`--allow-stale`): notice-beta."),
        "{page}"
    );
}

/// One document's bytes changed: it is recorded again on its own and merged
/// into the recording of record, which then replays clean.
#[test]
fn a_document_recorded_again_on_its_own_is_merged_in() {
    let bench = Bench::new();
    let with_model = |mut recording: Recording| {
        recording.model.sha256 = Some("ab".repeat(32));
        recording
    };
    with_model(recording())
        .save(&bench.path("recording.json"))
        .unwrap();
    let mut again = with_model(recording());
    again.recorded_at = "2026-10-07T10:00:00Z".into();
    again
        .documents
        .retain(|document| document.id == "scan-delta");
    again.documents[0].sha256 = Some("regenerated".into());
    again.save(&bench.path("again.json")).unwrap();
    std::fs::write(
        bench.path("manifest.json"),
        json!({"files": [
            {"file": "invoice-alpha.pdf", "sha256": "sha-of-invoice-alpha"},
            {"file": "scan-delta.png", "sha256": "regenerated"}
        ]})
        .to_string(),
    )
    .unwrap();
    let only = |options: &mut RunOptions| {
        options.manifest = Some(bench.path("manifest.json"));
        options.only = vec!["invoice-alpha".into(), "scan-delta".into()];
    };
    let (exit, report) = bench.replay("stale.json", only);
    assert_eq!(exit, EXIT_REGRESSED);
    assert_eq!(report.record("scan-delta").unwrap().status, "stale_fixture");

    intern_bench::merge::merge_command(
        &bench.path("recording.json"),
        &bench.path("again.json"),
        &bench.path("recording.json"),
        &bench.path("gold.json"),
        &[],
        Some("scan-delta regenerated"),
    )
    .unwrap();
    let (exit, report) = bench.replay("merged.json", only);
    assert_eq!(exit, 0);
    assert_eq!(report.record("scan-delta").unwrap().status, "completed");
    let merged = Recording::load(&bench.path("recording.json")).unwrap().0;
    assert_eq!(
        merged
            .documents
            .iter()
            .map(|document| document.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "invoice-alpha",
            "notice-beta",
            "scan-delta",
            "broken-epsilon"
        ],
        "the gold's order"
    );
    assert!(
        merged.note.ends_with("scan-delta regenerated"),
        "{}",
        merged.note
    );
}
