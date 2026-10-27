//! Two things the queue now does to files on its own: a document it cannot
//! read is renamed `NEEDS REVIEW - ...` where it lies and waits in review,
//! and an exact duplicate of a filed document is filed into the output
//! folder as `COPY - <the original's name>`. Both run here against the real
//! journaled file operations and real files.

use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use intern_core::{ErrorCode, QueueStatus, QueueStore};
use intern_engine::{
    AnalysisTelemetry, DateRole, DigestBudget, DocumentAnalysis, DocumentSource, Evidence,
    ExtractProgress, ModelProposal, PageOrigin, PartyRelation, SourcePage, distill, engine::finish,
    fingerprint, validate,
};
use intern_queue::{
    AdmissionEvidence, AdmissionGuard, AdmissionStage,
    paths::canonical_file,
    pipeline::{
        AnalyzerBoundary, DuplicateOracle, EXACT_COPY, FiledDocument, FilingSink, KnownFiling,
        ModelFailure, Pipeline, PipelineEventSink, PipelineItem, PipelineProgress, UNDONE,
        UnfiledDocument, WorkerBoundary, WorkerFailure,
    },
    settings::{AppSettings, DestinationLayout, SettingsStore},
};
use tempfile::{TempDir, tempdir};

const READABLE: &str =
    "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.";

fn parsed(text: &str) -> DocumentSource {
    DocumentSource::from_pages(vec![SourcePage::new(1, text, PageOrigin::Native)])
}

fn proposal() -> ModelProposal {
    ModelProposal {
        document_type: Some("Employment Agreement".into()),
        document_date: Some("2024-04-12".into()),
        date_role: Some(DateRole::Execution),
        parties: vec!["John Smith".into(), "Acme Corporation".into()],
        party_relation: PartyRelation::Between,
        description:
            "Employment agreement between John Smith and Acme Corporation covering duties, salary, and term."
                .into(),
        confidence: 0.94,
        needs_review: false,
        evidence: Evidence {
            date: Some("signed April 12, 2024".into()),
            document_type: Some("Employment Agreement".into()),
            parties: vec!["by John Smith and Acme Corporation".into()],
        },
        facts: None,
    }
}

/// What a working model answers for the calibration document of its
/// self-test (`intern_engine::setup::semantic_probes`), which the queue runs
/// when only documents in doubt are left.
fn calibration_reply() -> ModelProposal {
    let party = "Northstar Calibration Holdings LLC";
    ModelProposal {
        document_type: Some("Notice of Calibration".into()),
        document_date: Some("2024-01-02".into()),
        date_role: Some(DateRole::Notice),
        parties: vec![party.into()],
        party_relation: PartyRelation::To,
        description: format!("Notice of calibration confirming the model path works for {party}."),
        confidence: 0.99,
        needs_review: false,
        evidence: Evidence {
            date: Some("Date of this Notice: January 2, 2024".into()),
            document_type: Some("NOTICE OF CALIBRATION".into()),
            parties: vec![format!("To: {party}")],
        },
        facts: None,
    }
}

fn is_calibration(source: &DocumentSource) -> bool {
    source
        .pages
        .iter()
        .any(|page| page.text.contains("NOTICE OF CALIBRATION"))
}

/// The small text file the queue writes for the reader's self-test.
fn is_reader_check(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with("intern-reader-check"))
}

fn analyzed(
    source: &DocumentSource,
    reply: ModelProposal,
    extension: &str,
    existing_names: &[&str],
) -> DocumentAnalysis {
    let digest = distill(source, DigestBudget::default());
    let mut analysis = finish(
        validate(reply, &digest),
        &digest,
        extension,
        existing_names,
        AnalysisTelemetry::default(),
    );
    analysis.text_fingerprint = fingerprint::source_fingerprint(source).map(fingerprint::encode);
    analysis
}

/// Reads what it is told to, in order, and the readable agreement once the
/// script runs out.
#[derive(Default)]
struct ScriptedWorker {
    script: Mutex<VecDeque<Result<DocumentSource, WorkerFailure>>>,
    read: Mutex<Vec<PathBuf>>,
}

impl ScriptedWorker {
    fn new(script: Vec<Result<DocumentSource, WorkerFailure>>) -> Self {
        Self {
            script: Mutex::new(script.into()),
            read: Mutex::new(Vec::new()),
        }
    }
}

impl WorkerBoundary for ScriptedWorker {
    fn extract(
        &self,
        _request_id: &str,
        path: &Path,
        _progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, WorkerFailure> {
        if is_reader_check(path) {
            return Ok(parsed("reader check"));
        }
        self.read.lock().unwrap().push(path.to_path_buf());
        self.script
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok(parsed(READABLE)))
    }
    fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
        Ok(())
    }
    fn restart(&self) -> Result<(), WorkerFailure> {
        Ok(())
    }
}

/// Answers as it is told to, in order, and with the agreement's proposal
/// once the script runs out.
#[derive(Default)]
struct ScriptedModel {
    script: Mutex<VecDeque<Result<ModelProposal, ModelFailure>>>,
    calls: AtomicUsize,
}

impl ScriptedModel {
    fn new(script: Vec<Result<ModelProposal, ModelFailure>>) -> Self {
        Self {
            script: Mutex::new(script.into()),
            calls: AtomicUsize::new(0),
        }
    }
}

impl AnalyzerBoundary for ScriptedModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        if is_calibration(source) {
            return Ok(analyzed(
                source,
                calibration_reply(),
                extension,
                existing_names,
            ));
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        let reply = self
            .script
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok(proposal()))?;
        let digest = distill(source, DigestBudget::default());
        let mut analysis = finish(
            validate(reply, &digest),
            &digest,
            extension,
            existing_names,
            AnalysisTelemetry::default(),
        );
        analysis.text_fingerprint =
            fingerprint::source_fingerprint(source).map(fingerprint::encode);
        Ok(analysis)
    }
}

struct Quiet;

impl PipelineEventSink for Quiet {
    fn queue_changed(&self) {}
    fn progress(&self, _progress: PipelineProgress) {}
}

#[derive(Default)]
struct Filings {
    filed: Mutex<Vec<FiledDocument>>,
    unfiled: Mutex<Vec<UnfiledDocument>>,
}

impl FilingSink for Filings {
    fn filed(&self, document: &FiledDocument) {
        self.filed.lock().unwrap().push(document.clone());
    }
    fn unfiled(&self, document: &UnfiledDocument) {
        self.unfiled.lock().unwrap().push(document.clone());
    }
}

/// An input folder, an output folder, and a queue over them with the real
/// file operations.
struct Rig {
    temp: TempDir,
    inbox: PathBuf,
    filed: PathBuf,
    worker: Arc<ScriptedWorker>,
    model: Arc<ScriptedModel>,
    filings: Arc<Filings>,
    pipeline: Pipeline,
}

impl Rig {
    fn new(
        worker: Vec<Result<DocumentSource, WorkerFailure>>,
        model: Vec<Result<ModelProposal, ModelFailure>>,
        configure: impl FnOnce(&mut AppSettings),
    ) -> Rig {
        Self::with_oracle(worker, model, configure, None)
    }

    fn with_oracle(
        worker: Vec<Result<DocumentSource, WorkerFailure>>,
        model: Vec<Result<ModelProposal, ModelFailure>>,
        configure: impl FnOnce(&mut AppSettings),
        oracle: Option<Arc<dyn DuplicateOracle>>,
    ) -> Rig {
        Self::build(worker, model, configure, oracle, None)
    }

    fn build(
        worker: Vec<Result<DocumentSource, WorkerFailure>>,
        model: Vec<Result<ModelProposal, ModelFailure>>,
        configure: impl FnOnce(&mut AppSettings),
        oracle: Option<Arc<dyn DuplicateOracle>>,
        guard: Option<Arc<dyn AdmissionGuard>>,
    ) -> Rig {
        let temp = tempdir().unwrap();
        let inbox = temp.path().join("Inbox");
        let filed = temp.path().join("Filed");
        fs::create_dir_all(&inbox).unwrap();
        fs::create_dir_all(&filed).unwrap();
        let inbox = inbox.canonicalize().unwrap();
        let filed = filed.canonicalize().unwrap();
        let mut settings = AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            intake_folder: inbox.to_string_lossy().into_owned(),
            intake_enabled: true,
            automatic_rename: true,
            ..AppSettings::default()
        };
        configure(&mut settings);
        let store = SettingsStore::new(temp.path().join("settings.json"));
        store.save(&settings).unwrap();
        let worker = Arc::new(ScriptedWorker::new(worker));
        let model = Arc::new(ScriptedModel::new(model));
        let filings = Arc::new(Filings::default());
        let mut pipeline = Pipeline::with_local_files(
            temp.path().join("queue.sqlite3"),
            worker.clone(),
            model.clone(),
            Arc::new(Quiet),
            store,
        )
        .unwrap()
        .with_filing_sink(filings.clone());
        if let Some(oracle) = oracle {
            pipeline = pipeline.with_duplicate_oracle(oracle);
        }
        if let Some(guard) = guard {
            pipeline = pipeline.with_admission_guard(guard);
        }
        Rig {
            temp,
            inbox,
            filed,
            worker,
            model,
            filings,
            pipeline,
        }
    }

    fn drop_in(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.inbox.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn add(&self, path: &Path) -> i64 {
        self.pipeline
            .enqueue_files(&[path.to_path_buf()])
            .unwrap()
            .pop()
            .unwrap()
            .id
    }

    fn item(&self, id: i64) -> PipelineItem {
        self.pipeline
            .list()
            .unwrap()
            .into_iter()
            .find(|item| item.id == id)
            .unwrap()
    }

    fn filed_name(&self, id: i64) -> String {
        self.item(id)
            .filed_receipt
            .unwrap()
            .destination
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    fn inbox_names(&self) -> Vec<String> {
        names_in(&self.inbox)
    }
}

fn names_in(folder: &Path) -> Vec<String> {
    let mut names = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect::<Vec<_>>();
    names.sort();
    names
}

// ---------------------------------------------------------------------------
// Documents Intern could not read
// ---------------------------------------------------------------------------

/// Every reason that is the document's own leaves it in the queue, in
/// review under that reason, its file renamed where it lies. A reason that
/// is the document's by its nature does so at once. One that could as well
/// be the model's (a reply that could not be used, an internal failure)
/// leaves it waiting in doubt until the model is shown to work - here, with
/// nothing else left, by the self-test - and does so only when it fails the
/// same way after that. The queue goes on to the next document either way.
#[test]
fn a_document_that_cannot_be_read_waits_in_review_under_a_needs_review_name() {
    let extraction = [
        ("PASSWORD_PROTECTED", ErrorCode::PasswordProtected),
        ("UNSUPPORTED_FORMAT", ErrorCode::UnsupportedContent),
        ("RESOURCE_LIMIT_EXCEEDED", ErrorCode::DocumentTooLarge),
        ("PARSE_FAILED", ErrorCode::ExtractionFailed),
    ];
    let model = [
        ("HOSTED_MODEL_REFUSED", ErrorCode::ModelDeclined),
        ("MODEL_RESPONSE_INVALID", ErrorCode::ModelOutputInvalid),
        ("MODEL_INPUT_TOO_LARGE", ErrorCode::DocumentTooLarge),
        ("ANALYSIS_FAILED", ErrorCode::AnalysisFailed),
    ];
    let reads = extraction
        .iter()
        .map(|(code, _)| Err(WorkerFailure::reported(*code, "the worker's words", false)))
        .collect::<Vec<_>>();
    let mut replies = model
        .iter()
        .map(|(code, _)| Err(ModelFailure::fatal(*code)))
        .collect::<Vec<_>>();
    // The two in doubt, asked again once the self-test proved the model.
    replies.push(Err(ModelFailure::fatal("MODEL_RESPONSE_INVALID")));
    replies.push(Err(ModelFailure::fatal("ANALYSIS_FAILED")));
    let rig = Rig::new(reads, replies, |settings| settings.automatic_rename = false);
    let ids = (0..extraction.len() + model.len())
        .map(|index| {
            rig.add(&rig.drop_in(
                &format!("doc-{index}.pdf"),
                format!("doc {index}").as_bytes(),
            ))
        })
        .collect::<Vec<_>>();

    rig.pipeline.run_until_idle().unwrap();

    assert!(!rig.pipeline.is_paused());
    for ((index, id), (code, expected)) in ids
        .iter()
        .enumerate()
        .zip(extraction.iter().chain(model.iter()))
    {
        let item = rig.item(*id);
        assert_eq!(item.status, QueueStatus::NeedsReview, "{code}");
        assert_eq!(item.error_code, Some(*expected), "{code}");
        assert!(item.proposal.is_none(), "{code}");
        let name = format!("NEEDS REVIEW - doc-{index}.pdf");
        assert_eq!(item.source_path, rig.inbox.join(&name), "{code}");
        assert!(rig.inbox.join(&name).exists(), "{code}");
    }
}

/// Problems with this computer are not the document's: they stop or fail
/// the queue as they always did, and rename nothing.
#[test]
fn machine_problems_fail_or_pause_and_rename_nothing() {
    let rig = Rig::new(
        vec![
            Err(WorkerFailure::reported(
                "NATIVE_ASSETS_MISSING",
                "tessdata is missing",
                false,
            )),
            Ok(parsed(READABLE)),
            Err(WorkerFailure::reported("PARSE_FAILED", "read failed", true)),
            Err(WorkerFailure::reported("PARSE_FAILED", "read failed", true)),
        ],
        vec![Err(ModelFailure::fatal("HOSTED_MODEL_UNREACHABLE"))],
        |_| {},
    );
    let scan = rig.add(&rig.drop_in("scan.pdf", b"a scan"));
    rig.pipeline.run_until_idle().unwrap();
    assert!(rig.pipeline.is_paused());
    assert_eq!(rig.item(scan).status, QueueStatus::Failed);
    assert_eq!(rig.item(scan).error_code, Some(ErrorCode::OcrUnavailable));

    rig.pipeline.resume();
    let unreachable = rig.add(&rig.drop_in("unreachable.pdf", b"model is down"));
    rig.pipeline.run_until_idle().unwrap();
    assert!(rig.pipeline.is_paused());
    assert_eq!(rig.item(unreachable).status, QueueStatus::Queued);

    rig.pipeline.resume();
    rig.pipeline.remove(unreachable, false).unwrap();
    let flaky = rig.add(&rig.drop_in("offline.pdf", b"still in the cloud"));
    rig.pipeline.run_until_idle().unwrap();
    let flaky = rig.item(flaky);
    assert_eq!(flaky.status, QueueStatus::Failed);
    assert_eq!(flaky.error_code, Some(ErrorCode::IoError));

    assert_eq!(
        rig.inbox_names(),
        vec!["offline.pdf", "scan.pdf", "unreachable.pdf"],
        "nothing was renamed"
    );
}

/// A parser crash, or a model running out of time, may be the machine's or
/// the document's. The document waits in doubt, under its own name, until
/// the machine is shown to work for that kind of failure - another document
/// read, or the self-test - and only failing the same way after that is it
/// the document's: in review, renamed.
#[test]
fn a_document_that_crashes_the_parser_or_times_out_after_the_machine_works_goes_to_review() {
    let rig = Rig::new(
        vec![
            Err(WorkerFailure::crashed()),
            // slow.pdf is read, which proves the reader...
            Ok(parsed(READABLE)),
            // ...and crashes.pdf crashes it again: its own.
            Err(WorkerFailure::crashed()),
        ],
        vec![
            Err(ModelFailure::fatal("MODEL_TIMEOUT")),
            // Asked again after the self-test proved the model.
            Err(ModelFailure::fatal("MODEL_TIMEOUT")),
        ],
        |_| {},
    );
    let crashing = rig.add(&rig.drop_in("crashes.pdf", b"crashes the parser"));
    let slow = rig.add(&rig.drop_in("slow.pdf", b"too slow to read"));

    rig.pipeline.run_until_idle().unwrap();

    assert!(!rig.pipeline.is_paused(), "two in a row is not a run");
    let item = rig.item(crashing);
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(item.error_code, Some(ErrorCode::ExtractionFailed));
    assert_eq!(
        item.source_path,
        rig.inbox.join("NEEDS REVIEW - crashes.pdf")
    );
    let item = rig.item(slow);
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(item.error_code, Some(ErrorCode::ModelFailed));
    assert_eq!(item.source_path, rig.inbox.join("NEEDS REVIEW - slow.pdf"));
}

/// Retry reads the renamed file again. Read successfully this time, it is
/// filed under the name that reading proposed, and the NEEDS REVIEW name
/// goes with it out of the input folder.
#[test]
fn a_retried_document_is_read_from_its_new_name_and_filed_under_the_proposed_one() {
    let rig = Rig::new(
        vec![Err(WorkerFailure::reported(
            "PASSWORD_PROTECTED",
            "encrypted",
            false,
        ))],
        vec![],
        |_| {},
    );
    let id = rig.add(&rig.drop_in("locked.pdf", b"was locked"));
    rig.pipeline.run_until_idle().unwrap();
    let marked = rig.inbox.join("NEEDS REVIEW - locked.pdf");
    assert_eq!(rig.item(id).source_path, marked);

    rig.pipeline.retry(id).unwrap();
    let retried = rig.item(id);
    assert_eq!(retried.status, QueueStatus::Queued);
    assert_eq!(retried.error_code, None);
    rig.pipeline.run_until_idle().unwrap();

    assert_eq!(rig.worker.read.lock().unwrap().last(), Some(&marked));
    let filed = rig.item(id);
    assert_eq!(filed.status, QueueStatus::Completed);
    let name = rig.filed_name(id);
    assert!(name.starts_with("2024-04-12 "), "{name}");
    assert!(rig.filed.join(&name).exists());
    assert!(rig.inbox_names().is_empty());
    assert_eq!(rig.filings.filed.lock().unwrap().len(), 1);
}

/// A person can name an unreadable document themselves. There is no
/// proposal, so the name they type becomes one - under the same date rule -
/// and the document is filed into the output folder like any other.
#[test]
fn a_document_that_cannot_be_read_can_be_named_by_hand_and_filed() {
    let rig = Rig::new(
        vec![Err(WorkerFailure::reported(
            "PARSE_FAILED",
            "damaged",
            false,
        ))],
        vec![],
        |settings| settings.destination_layout = DestinationLayout::Year,
    );
    let id = rig.add(&rig.drop_in("damaged.pdf", b"damaged bytes"));
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(rig.item(id).status, QueueStatus::NeedsReview);

    let refused = rig
        .pipeline
        .approve(id, "Lease from the landlord.pdf", "")
        .unwrap_err();
    assert_eq!(refused.code, "DATE_REQUIRED");
    assert_eq!(rig.item(id).status, QueueStatus::NeedsReview);
    let refused = rig
        .pipeline
        .approve(id, "2026-01-02 Lease.docx", "")
        .unwrap_err();
    assert_eq!(
        refused.code, "NAME_INVALID",
        "the extension stays the file's"
    );

    rig.pipeline
        .approve(id, "2026-01-02 Lease.pdf", "A lease I read myself.")
        .unwrap();

    let filed = rig.item(id);
    assert_eq!(filed.status, QueueStatus::Completed);
    assert_eq!(rig.filed_name(id), "2026-01-02 Lease.pdf");
    assert!(rig.filed.join("2026").join("2026-01-02 Lease.pdf").exists());
    assert!(rig.inbox_names().is_empty());
    let record = filed.proposal.unwrap();
    assert!(record.manual);
    assert_eq!(record.description, "A lease I read myself.");
    let reported = rig.filings.filed.lock().unwrap();
    assert_eq!(reported.len(), 1, "a name typed by hand is a real filing");
    assert_eq!(reported[0].description, "A lease I read myself.");
    assert_eq!(rig.model.calls.load(Ordering::SeqCst), 0);
}

/// Keep original leaves the document in the input folder under its own
/// name: the prefix Intern added comes off again.
#[test]
fn keeping_an_unreadable_documents_name_takes_the_prefix_off() {
    let rig = Rig::new(
        vec![Err(WorkerFailure::reported(
            "UNSUPPORTED_FORMAT",
            "not a PDF",
            false,
        ))],
        vec![],
        |_| {},
    );
    let id = rig.add(&rig.drop_in("photo.pdf", b"actually a JPEG"));
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(rig.inbox_names(), vec!["NEEDS REVIEW - photo.pdf"]);

    rig.pipeline.keep_original(id).unwrap();

    let kept = rig.item(id);
    assert_eq!(kept.status, QueueStatus::Completed);
    assert_eq!(kept.source_path, rig.inbox.join("photo.pdf"));
    assert_eq!(rig.inbox_names(), vec!["photo.pdf"]);
    assert!(rig.filings.filed.lock().unwrap().is_empty());
}

/// Removed from the queue, the document stays where it is under its
/// NEEDS REVIEW name, and nothing that adds files takes that name in again.
#[test]
fn a_removed_unreadable_document_keeps_its_name_and_is_never_taken_in_again() {
    let rig = Rig::new(
        vec![Err(WorkerFailure::reported(
            "PASSWORD_PROTECTED",
            "encrypted",
            false,
        ))],
        vec![],
        |_| {},
    );
    let id = rig.add(&rig.drop_in("locked.pdf", b"was locked"));
    rig.pipeline.run_until_idle().unwrap();
    rig.pipeline.remove(id, false).unwrap();

    let marked = rig.inbox.join("NEEDS REVIEW - locked.pdf");
    assert!(marked.exists());
    assert!(rig.pipeline.list().unwrap().is_empty());
    let refused = canonical_file(&marked).unwrap_err();
    assert_eq!(refused.code, "UNSUPPORTED_FORMAT");
}

/// The crash after a document reached review and before its file was
/// renamed: the next start finds it in review under its old name and
/// renames it then.
#[test]
fn recovery_renames_a_document_left_in_review_without_its_mark() {
    let rig = Rig::new(vec![], vec![], |_| {});
    let path = rig.drop_in("scan.pdf", b"a scan");
    {
        let store = QueueStore::open(rig.temp.path().join("queue.sqlite3")).unwrap();
        let item = store.enqueue(&path, "hash-of-the-scan").unwrap();
        store.claim_next().unwrap().unwrap();
        store
            .record_unreadable(
                item.id,
                ErrorCode::PasswordProtected,
                intern_core::ReviewMark::Now,
            )
            .unwrap();
    }

    rig.pipeline.recover().unwrap();

    let item = rig.pipeline.list().unwrap().pop().unwrap();
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(item.source_path, rig.inbox.join("NEEDS REVIEW - scan.pdf"));
    assert_eq!(rig.inbox_names(), vec!["NEEDS REVIEW - scan.pdf"]);
}

// ---------------------------------------------------------------------------
// Exact duplicates
// ---------------------------------------------------------------------------

/// Files the agreement once, and returns its item and the name it was
/// filed under.
fn file_the_original(rig: &Rig) -> (i64, String) {
    let id = rig.add(&rig.drop_in("agreement.pdf", b"the agreement's bytes"));
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(rig.item(id).status, QueueStatus::Completed);
    let name = rig.filed_name(id);
    (id, name)
}

/// An exact duplicate is moved out of the input folder into the output
/// folder under the original's filed name with COPY in front, and each
/// further copy takes the next number - every one named after the original,
/// never after a copy. Nothing is read, and a copy is no filing of its own.
#[test]
fn an_exact_duplicate_is_filed_as_a_copy_of_the_original() {
    let rig = Rig::new(vec![], vec![], |_| {});
    let (_, original) = file_the_original(&rig);
    let calls = rig.model.calls.load(Ordering::SeqCst);

    let first = rig.add(&rig.drop_in("scan of agreement.pdf", b"the agreement's bytes"));
    let second = rig.add(&rig.drop_in("agreement again.pdf", b"the agreement's bytes"));
    let third = rig.add(&rig.drop_in("and again.pdf", b"the agreement's bytes"));

    let copy = format!("COPY - {original}");
    let stem = original.trim_end_matches(".pdf");
    for (id, expected) in [
        (first, copy.clone()),
        (second, format!("COPY - {stem} (2).pdf")),
        (third, format!("COPY - {stem} (3).pdf")),
    ] {
        let item = rig.item(id);
        assert_eq!(item.status, QueueStatus::Completed, "{expected}");
        assert_eq!(rig.filed_name(id), expected);
        assert!(rig.filed.join(&expected).exists(), "{expected}");
        let record = item.proposal.unwrap();
        assert_eq!(record.copy_of.as_ref().unwrap().original, original);
        assert!(record.reasons.iter().any(|reason| reason == EXACT_COPY));
    }
    assert!(
        rig.inbox_names().is_empty(),
        "every copy left the input folder"
    );
    assert_eq!(
        rig.model.calls.load(Ordering::SeqCst),
        calls,
        "a copy is never read"
    );
    assert_eq!(
        rig.filings.filed.lock().unwrap().len(),
        1,
        "only the original was reported filed"
    );
    assert_eq!(
        rig.pipeline.filed_documents().unwrap().len(),
        1,
        "and only the original is replayed to a records keeper"
    );
}

/// Undo puts a copy back where it came from, where it waits in review as
/// the duplicate it is - and the scheduler never moves it again on its own.
/// "Process anyway" then reads it as a document like any other.
#[test]
fn an_undone_copy_waits_in_review_and_is_not_moved_again() {
    let rig = Rig::new(vec![], vec![], |_| {});
    let (_, original) = file_the_original(&rig);
    let dropped = rig.drop_in("scan.pdf", b"the agreement's bytes");
    let id = rig.add(&dropped);
    assert_eq!(rig.item(id).status, QueueStatus::Completed);

    rig.pipeline.undo(id).unwrap();

    let undone = rig.item(id);
    assert_eq!(undone.status, QueueStatus::NeedsReview);
    assert_eq!(undone.error_code, Some(ErrorCode::Duplicate));
    assert!(
        undone
            .proposal
            .as_ref()
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == UNDONE)
    );
    assert!(dropped.exists());
    assert!(!rig.filed.join(format!("COPY - {original}")).exists());
    assert!(
        rig.filings.unfiled.lock().unwrap().is_empty(),
        "the original's records are not retracted for its copy"
    );
    for _ in 0..2 {
        rig.pipeline.run_until_idle().unwrap();
        rig.pipeline.recover().unwrap();
    }
    assert_eq!(rig.item(id).status, QueueStatus::NeedsReview);
    assert!(dropped.exists());

    rig.pipeline.retry(id).unwrap();
    let requeued = rig.item(id);
    assert_eq!(requeued.status, QueueStatus::Queued);
    assert!(requeued.proposal.is_none());
    rig.pipeline.run_until_idle().unwrap();
    let processed = rig.item(id);
    assert_eq!(processed.status, QueueStatus::Completed);
    assert!(processed.proposal.unwrap().copy_of.is_none());
    assert!(!rig.filed_name(id).starts_with("COPY - "));
}

/// The copy goes beside the original when the layout put the original in
/// a subfolder.
#[test]
fn a_copy_is_filed_in_the_originals_folder() {
    let rig = Rig::new(vec![], vec![], |settings| {
        settings.destination_layout = DestinationLayout::YearType;
    });
    let (original_id, original) = file_the_original(&rig);
    let folder = rig
        .item(original_id)
        .filed_receipt
        .unwrap()
        .destination
        .parent()
        .unwrap()
        .to_path_buf();
    assert_ne!(folder, rig.filed);

    let id = rig.add(&rig.drop_in("copy.pdf", b"the agreement's bytes"));

    assert_eq!(rig.item(id).status, QueueStatus::Completed);
    assert!(folder.join(format!("COPY - {original}")).exists());
}

/// With no output folder there is nowhere to file a copy, so a duplicate
/// waits in review untouched, as every duplicate used to; and a file that
/// is already in the output folder - the original itself, added again - is
/// never moved as a copy of itself.
#[test]
fn a_duplicate_with_nowhere_to_go_or_already_filed_stays_where_it_is() {
    let rig = Rig::new(vec![], vec![], |_| {});
    let (original_id, _) = file_the_original(&rig);
    let filed = rig.item(original_id).filed_receipt.unwrap().destination;

    let again = rig
        .pipeline
        .enqueue_files(std::slice::from_ref(&filed))
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(again.status, QueueStatus::NeedsReview);
    assert_eq!(again.error_code, Some(ErrorCode::Duplicate));
    assert!(filed.exists());

    let unset = Rig::new(vec![], vec![], |settings| {
        settings.destination = String::new();
    });
    let first = unset.add(&unset.drop_in("agreement.pdf", b"the agreement's bytes"));
    unset.pipeline.run_until_idle().unwrap();
    assert_eq!(unset.item(first).status, QueueStatus::Completed);
    let copy = unset.drop_in("copy.pdf", b"the agreement's bytes");
    let id = unset.add(&copy);
    assert_eq!(unset.item(id).status, QueueStatus::NeedsReview);
    assert_eq!(unset.item(id).error_code, Some(ErrorCode::Duplicate));
    assert!(copy.exists());
}

/// A duplicate that arrives while the queue is paused is staged and filed
/// once it runs again, automatic renaming or not.
#[test]
fn a_copy_staged_while_paused_is_filed_when_the_queue_runs_again() {
    let rig = Rig::new(vec![], vec![], |_| {});
    let (_, original) = file_the_original(&rig);
    let settings = SettingsStore::new(rig.temp.path().join("settings.json"));
    let mut saved = settings.load().unwrap();
    saved.automatic_rename = false;
    settings.save(&saved).unwrap();
    rig.pipeline.pause();

    let id = rig.add(&rig.drop_in("copy.pdf", b"the agreement's bytes"));
    let staged = rig.item(id);
    assert_eq!(staged.status, QueueStatus::Ready);
    assert!(staged.proposal.as_ref().unwrap().approved);
    assert_eq!(rig.inbox_names(), vec!["copy.pdf"]);

    rig.pipeline.resume();
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(rig.item(id).status, QueueStatus::Completed);
    assert!(rig.filed.join(format!("COPY - {original}")).exists());
}

/// What a teammate's machine filed, as the shared filed index knows it.
struct Teammate(Mutex<Option<KnownFiling>>);

impl DuplicateOracle for Teammate {
    fn filed_elsewhere(&self, _source_hash: &str, _source_path: &Path) -> Option<KnownFiling> {
        self.0.lock().unwrap().clone()
    }
}

/// Content a teammate filed is filed as a copy too, under the name their
/// machine filed it as, in the output folder's root - unless that name,
/// which comes from a file anyone can write, is no file name at all.
#[test]
fn a_teammates_filing_is_copied_under_its_raw_name_and_a_hostile_name_is_refused() {
    let teammate = Arc::new(Teammate(Mutex::new(Some(KnownFiling {
        filename: "2026-01-02 Lease.pdf".into(),
        filed_by: Some("Front desk".into()),
    }))));
    let rig = Rig::with_oracle(vec![], vec![], |_| {}, Some(teammate.clone()));

    let id = rig.add(&rig.drop_in("lease.PDF", b"the lease"));
    assert_eq!(rig.item(id).status, QueueStatus::Completed);
    assert_eq!(rig.filed_name(id), "COPY - 2026-01-02 Lease.PDF");
    assert_eq!(rig.model.calls.load(Ordering::SeqCst), 0);

    *teammate.0.lock().unwrap() = Some(KnownFiling {
        filename: "..\\..\\escape.pdf".into(),
        filed_by: Some("Somewhere".into()),
    });
    let hostile = rig.drop_in("other.pdf", b"other bytes");
    let id = rig.add(&hostile);
    assert_eq!(rig.item(id).status, QueueStatus::NeedsReview);
    assert_eq!(rig.item(id).error_code, Some(ErrorCode::Duplicate));
    assert!(hostile.exists());
}

/// A person who names an undone copy themselves files it as a document of
/// its own: it is reported like any filing, and is no copy any more.
#[test]
fn naming_an_undone_copy_files_it_as_a_document_of_its_own() {
    let rig = Rig::new(vec![], vec![], |_| {});
    file_the_original(&rig);
    let id = rig.add(&rig.drop_in("scan.pdf", b"the agreement's bytes"));
    rig.pipeline.undo(id).unwrap();
    assert_eq!(rig.filings.filed.lock().unwrap().len(), 1);

    rig.pipeline
        .approve(id, "2024-04-13 Signed copy.pdf", "The countersigned copy.")
        .unwrap();

    let filed = rig.item(id);
    assert_eq!(filed.status, QueueStatus::Completed);
    assert_eq!(rig.filed_name(id), "2024-04-13 Signed copy.pdf");
    let record = filed.proposal.unwrap();
    assert!(record.copy_of.is_none());
    assert!(record.manual);
    assert_eq!(rig.filings.filed.lock().unwrap().len(), 2);
}

// ---------------------------------------------------------------------------
// What a review rename must never do
// ---------------------------------------------------------------------------

/// Replies that cannot be used, several in a row, are the model's problem:
/// the queue stops, and the documents of that run go back to wait under
/// their own names - nothing renamed, nothing counted - and resuming the
/// queue reads them again. The model is still broken for one more document
/// after the resume, which waits in doubt too; once the model answers, every
/// document in doubt is read again, and filed.
#[test]
fn a_run_of_unusable_replies_stops_the_queue_and_renames_nothing() {
    let invalid = || Err(ModelFailure::fatal("MODEL_RESPONSE_INVALID"));
    let rig = Rig::new(
        vec![],
        // Three in a row, and one more after the first resume.
        vec![invalid(), invalid(), invalid(), invalid()],
        |_| {},
    );
    let ids = (0..5)
        .map(|index| {
            rig.add(&rig.drop_in(&format!("d{index}.pdf"), format!("doc {index}").as_bytes()))
        })
        .collect::<Vec<_>>();

    rig.pipeline.run_until_idle().unwrap();

    assert!(rig.pipeline.is_paused());
    assert_eq!(
        rig.pipeline.pause_reason().as_deref(),
        Some("MODEL_OUTPUT_INVALID")
    );
    assert_eq!(
        rig.inbox_names(),
        vec!["d0.pdf", "d1.pdf", "d2.pdf", "d3.pdf", "d4.pdf"]
    );
    for id in &ids {
        let item = rig.item(*id);
        assert_eq!(
            item.status,
            QueueStatus::Queued,
            "the run waits to be read again"
        );
        assert_eq!(item.processing_failures, 0);
    }

    rig.pipeline.resume();
    rig.pipeline.run_until_idle().unwrap();
    assert!(!rig.pipeline.is_paused());
    for id in &ids {
        assert_eq!(rig.item(*id).status, QueueStatus::Completed);
    }
    assert!(rig.inbox_names().is_empty(), "nothing was renamed");
}

/// A model too slow for anything times out on every document. It used to
/// send each to review renamed, one after another, through the whole
/// backlog; several time-outs in a row stop the queue instead.
#[test]
fn a_run_of_time_outs_stops_the_queue_and_renames_nothing() {
    let rig = Rig::new(
        vec![],
        (0..8)
            .map(|_| Err(ModelFailure::fatal("MODEL_TIMEOUT")))
            .collect(),
        |_| {},
    );
    let ids = (0..3)
        .map(|index| {
            rig.add(&rig.drop_in(&format!("t{index}.pdf"), format!("doc {index}").as_bytes()))
        })
        .collect::<Vec<_>>();

    rig.pipeline.run_until_idle().unwrap();

    assert!(rig.pipeline.is_paused());
    assert_eq!(rig.pipeline.pause_reason().as_deref(), Some("MODEL_FAILED"));
    assert_eq!(rig.inbox_names(), vec!["t0.pdf", "t1.pdf", "t2.pdf"]);
    for id in &ids {
        assert_eq!(rig.item(*id).status, QueueStatus::Queued);
    }
}

/// A reader that crashes on every document is broken, not the documents.
#[test]
fn a_run_of_reader_crashes_stops_the_queue_and_renames_nothing() {
    let rig = Rig::new(
        (0..8).map(|_| Err(WorkerFailure::crashed())).collect(),
        vec![],
        |_| {},
    );
    let ids = (0..3)
        .map(|index| {
            rig.add(&rig.drop_in(&format!("c{index}.pdf"), format!("doc {index}").as_bytes()))
        })
        .collect::<Vec<_>>();

    rig.pipeline.run_until_idle().unwrap();

    assert!(rig.pipeline.is_paused());
    assert_eq!(
        rig.pipeline.pause_reason().as_deref(),
        Some("READER_CRASHED")
    );
    assert_eq!(rig.inbox_names(), vec!["c0.pdf", "c1.pdf", "c2.pdf"]);
    for id in &ids {
        assert_eq!(rig.item(*id).status, QueueStatus::Queued);
    }
}

/// "Remove the password and retry": the file the person saved again is read
/// and filed as it is now, not refused as changed since it was read.
#[test]
fn a_document_fixed_and_retried_is_read_and_filed_as_it_is_now() {
    let rig = Rig::new(
        vec![Err(WorkerFailure::reported(
            "PASSWORD_PROTECTED",
            "encrypted",
            false,
        ))],
        vec![],
        |_| {},
    );
    let id = rig.add(&rig.drop_in("locked.pdf", b"was locked"));
    rig.pipeline.run_until_idle().unwrap();
    let marked = rig.inbox.join("NEEDS REVIEW - locked.pdf");
    assert_eq!(rig.item(id).source_path, marked);
    fs::write(&marked, b"password removed, saved again").unwrap();

    rig.pipeline.retry(id).unwrap();
    rig.pipeline.run_until_idle().unwrap();

    let item = rig.item(id);
    assert_eq!(
        item.status,
        QueueStatus::Completed,
        "{:?}",
        item.proposal.map(|p| p.reasons)
    );
    assert!(rig.filed_name(id).starts_with("2024-04-12 "));
    assert!(rig.inbox_names().is_empty());
}

/// A canceled document can be cleared away with the history, and so can one
/// that failed after a retry; a file left named for review behind either
/// would be skipped by every scan for good. Each gets its own name back.
#[test]
fn a_marked_document_canceled_or_failed_gets_its_name_back() {
    let rig = Rig::new(
        vec![
            Err(WorkerFailure::reported(
                "PASSWORD_PROTECTED",
                "encrypted",
                false,
            )),
            Err(WorkerFailure::reported(
                "PASSWORD_PROTECTED",
                "encrypted",
                false,
            )),
            Err(WorkerFailure::reported("PARSE_FAILED", "read failed", true)),
            Err(WorkerFailure::reported("PARSE_FAILED", "read failed", true)),
        ],
        vec![],
        |_| {},
    );
    let canceled = rig.add(&rig.drop_in("canceled.pdf", b"canceled"));
    let failed = rig.add(&rig.drop_in("failed.pdf", b"failed"));
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(
        rig.inbox_names(),
        vec!["NEEDS REVIEW - canceled.pdf", "NEEDS REVIEW - failed.pdf"]
    );

    rig.pipeline.cancel(canceled).unwrap();
    rig.pipeline.retry(failed).unwrap();
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(rig.item(failed).status, QueueStatus::Failed);
    assert_eq!(rig.pipeline.clear_history().unwrap(), 2);

    assert!(rig.pipeline.list().unwrap().is_empty());
    assert_eq!(rig.inbox_names(), vec!["canceled.pdf", "failed.pdf"]);
}

/// Admits a document only on a proof that names its file, the way
/// Microsoft's word on an uploader does: a file renamed since is unknown.
struct NamedProof {
    snapshots: intern_core::PrivateSnapshotDirectory,
}

impl AdmissionGuard for NamedProof {
    fn authorize(
        &self,
        path: &Path,
        _stage: AdmissionStage,
    ) -> Result<AdmissionEvidence, intern_queue::PipelineError> {
        let name = path.file_name().unwrap().to_string_lossy();
        if intern_core::is_needs_review_name(&name) {
            return Err(intern_queue::PipelineError::new(
                "UPLOADER_UNVERIFIED",
                "the library has no file by this name",
            ));
        }
        let bytes = fs::read(path)
            .map_err(|_| intern_queue::PipelineError::new("UPLOADER_UNVERIFIED", "unreadable"))?;
        let mut writer = self
            .snapshots
            .create_for_supported_extension("pdf")
            .unwrap();
        std::io::Write::write_all(&mut writer, &bytes).unwrap();
        let snapshot = writer.finish().unwrap();
        let hash = intern_core::FileSystem::hash(&intern_core::StdFileSystem, path).unwrap();
        Ok(AdmissionEvidence::verified_snapshot(hash, snapshot))
    }
}

/// A document admitted on such a proof waits in review under its own name:
/// renamed, it could never be proven again - not retried, not named by
/// hand, not filed.
#[test]
fn a_document_admitted_on_a_proof_of_its_name_is_never_renamed() {
    let temp = tempdir().unwrap();
    let guard = Arc::new(NamedProof {
        snapshots: intern_core::PrivateSnapshotDirectory::new(temp.path().join("snapshots"))
            .unwrap(),
    });
    let rig = Rig::build(
        vec![Err(WorkerFailure::reported(
            "PASSWORD_PROTECTED",
            "encrypted",
            false,
        ))],
        vec![],
        |_| {},
        None,
        Some(guard),
    );
    let path = rig.drop_in("locked.pdf", b"was locked");
    let id = rig.add(&path);
    rig.pipeline.run_until_idle().unwrap();

    let item = rig.item(id);
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(item.error_code, Some(ErrorCode::PasswordProtected));
    assert_eq!(item.source_path, path);
    rig.pipeline.recover().unwrap();
    assert_eq!(
        rig.inbox_names(),
        vec!["locked.pdf"],
        "not even on recovery"
    );

    rig.pipeline
        .approve(id, "2026-01-02 Lease.pdf", "Named by hand.")
        .unwrap();
    assert_eq!(rig.item(id).status, QueueStatus::Completed);
    assert_eq!(rig.filed_name(id), "2026-01-02 Lease.pdf");
}

/// Intern renames and moves on its own only what was put in the input
/// folder: a document added from anywhere else that cannot be read waits in
/// review under its own name, and a duplicate of a filed one waits in review
/// where it is.
#[test]
fn nothing_outside_the_input_folder_is_renamed_or_moved() {
    let rig = Rig::new(
        vec![Err(WorkerFailure::reported(
            "PASSWORD_PROTECTED",
            "encrypted",
            false,
        ))],
        vec![],
        |_| {},
    );
    let elsewhere = rig.temp.path().join("Desktop");
    fs::create_dir_all(&elsewhere).unwrap();
    let locked = elsewhere.join("locked.pdf");
    fs::write(&locked, b"was locked").unwrap();
    let id = rig.add(&locked.canonicalize().unwrap());
    rig.pipeline.run_until_idle().unwrap();
    assert_eq!(rig.item(id).status, QueueStatus::NeedsReview);
    assert!(locked.exists());
    rig.pipeline.recover().unwrap();
    assert!(locked.exists());

    file_the_original(&rig);
    let copy = elsewhere.join("agreement copy.pdf");
    fs::write(&copy, b"the agreement's bytes").unwrap();
    let id = rig.add(&copy.canonicalize().unwrap());
    assert_eq!(rig.item(id).status, QueueStatus::NeedsReview);
    assert_eq!(rig.item(id).error_code, Some(ErrorCode::Duplicate));
    assert!(copy.exists());
}

/// A staged copy applied under its own name is filed as the copy it is,
/// not refused for a name without a date.
#[test]
fn applying_a_staged_copy_under_its_name_files_it() {
    let rig = Rig::new(vec![], vec![], |_| {});
    let (_, original) = file_the_original(&rig);
    rig.pipeline.pause();
    let id = rig.add(&rig.drop_in("copy.pdf", b"the agreement's bytes"));
    let copy = format!("COPY - {original}");
    assert_eq!(rig.item(id).proposal.unwrap().filename, copy);

    rig.pipeline.approve(id, &copy, "").unwrap();

    assert_eq!(rig.item(id).status, QueueStatus::Completed);
    assert!(rig.filed.join(&copy).exists());
}

/// A copy staged before the output folder changed goes to the new output
/// folder, not to a folder of the old one, recreated.
#[test]
fn a_staged_copy_follows_a_changed_output_folder() {
    let rig = Rig::new(vec![], vec![], |settings| {
        settings.destination_layout = DestinationLayout::Year;
    });
    let (_, original) = file_the_original(&rig);
    rig.pipeline.pause();
    let id = rig.add(&rig.drop_in("copy.pdf", b"the agreement's bytes"));
    let moved = rig.temp.path().join("Archive");
    fs::create_dir_all(&moved).unwrap();
    let moved = moved.canonicalize().unwrap();
    let settings = SettingsStore::new(rig.temp.path().join("settings.json"));
    let mut saved = settings.load().unwrap();
    saved.destination = moved.to_string_lossy().into_owned();
    settings.save(&saved).unwrap();
    fs::remove_dir_all(rig.filed.join("2024")).unwrap();

    rig.pipeline.resume();
    rig.pipeline.run_until_idle().unwrap();

    assert_eq!(rig.item(id).status, QueueStatus::Completed);
    assert!(moved.join(format!("COPY - {original}")).exists());
    assert!(
        !rig.filed.join("2024").exists(),
        "the old folder is not recreated"
    );
}

// ---- doubt and proof, keyed on the documents' own words -------------------

/// Reads a file's bytes as its text: "CRASH" in it crashes the reader, and
/// "LOCKED" makes it password-protected. While `broken`, it crashes on
/// everything, its own self-test too; while `pdfs_broken`, on every PDF.
#[derive(Default)]
struct BytesWorker {
    broken: AtomicBool,
    pdfs_broken: AtomicBool,
}

impl WorkerBoundary for BytesWorker {
    fn extract(
        &self,
        _request_id: &str,
        path: &Path,
        _progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, WorkerFailure> {
        if self.broken.load(Ordering::SeqCst) {
            return Err(WorkerFailure::crashed());
        }
        if self.pdfs_broken.load(Ordering::SeqCst)
            && path.extension().is_some_and(|extension| extension == "pdf")
        {
            return Err(WorkerFailure::crashed());
        }
        let text = fs::read_to_string(path).unwrap_or_default();
        if text.contains("CRASH") {
            return Err(WorkerFailure::crashed());
        }
        if text.contains("LOCKED") {
            return Err(WorkerFailure::reported(
                "PASSWORD_PROTECTED",
                "encrypted",
                false,
            ));
        }
        Ok(parsed(&format!("{text} {READABLE}")))
    }
    fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
        Ok(())
    }
    fn restart(&self) -> Result<(), WorkerFailure> {
        Ok(())
    }
}

/// Answers from the document's text: "INVALID" gets a reply that cannot be
/// used, "SLOW" a time-out, the calibration document its right answer, and
/// anything else the agreement's proposal. During an `outage` every request
/// - the self-test's too - fails with that code.
#[derive(Default)]
struct TextModel {
    outage: Mutex<Option<&'static str>>,
    /// How often the self-test's calibration document was asked about.
    calibrations: AtomicUsize,
}

impl AnalyzerBoundary for TextModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        if is_calibration(source) {
            self.calibrations.fetch_add(1, Ordering::SeqCst);
        }
        if let Some(code) = *self.outage.lock().unwrap() {
            return Err(ModelFailure::fatal(code));
        }
        if is_calibration(source) {
            return Ok(analyzed(
                source,
                calibration_reply(),
                extension,
                existing_names,
            ));
        }
        let text = source
            .pages
            .iter()
            .map(|page| page.text.clone())
            .collect::<String>();
        if text.contains("INVALID") {
            return Err(ModelFailure::fatal("MODEL_RESPONSE_INVALID"));
        }
        if text.contains("SLOW") {
            return Err(ModelFailure::fatal("MODEL_TIMEOUT"));
        }
        Ok(analyzed(source, proposal(), extension, existing_names))
    }
}

/// The ways the machine can be broken for every document, with the reason
/// the queue stops for.
#[derive(Clone, Copy, Debug)]
enum Outage {
    TimesOut,
    AnswersGarbage,
    ReaderCrashes,
}

impl Outage {
    const ALL: [Outage; 3] = [
        Outage::TimesOut,
        Outage::AnswersGarbage,
        Outage::ReaderCrashes,
    ];

    fn reason(self) -> &'static str {
        match self {
            Outage::TimesOut => "MODEL_FAILED",
            Outage::AnswersGarbage => "MODEL_OUTPUT_INVALID",
            Outage::ReaderCrashes => "READER_CRASHED",
        }
    }

    /// What a document says to fail this way on its own.
    fn word(self) -> &'static str {
        match self {
            Outage::TimesOut => "SLOW",
            Outage::AnswersGarbage => "INVALID",
            Outage::ReaderCrashes => "CRASH",
        }
    }
}

/// An input folder and a queue whose reader and model answer from each
/// document's own words, and can be broken for everything.
struct TextRig {
    temp: TempDir,
    inbox: PathBuf,
    worker: Arc<BytesWorker>,
    model: Arc<TextModel>,
    pipeline: Pipeline,
}

impl TextRig {
    fn new() -> TextRig {
        let temp = tempdir().unwrap();
        let inbox = temp.path().join("Inbox");
        let filed = temp.path().join("Filed");
        fs::create_dir_all(&inbox).unwrap();
        fs::create_dir_all(&filed).unwrap();
        let inbox = inbox.canonicalize().unwrap();
        let settings = AppSettings {
            destination: filed.canonicalize().unwrap().to_string_lossy().into_owned(),
            intake_folder: inbox.to_string_lossy().into_owned(),
            intake_enabled: true,
            automatic_rename: true,
            ..AppSettings::default()
        };
        TextRig::settings(&temp).save(&settings).unwrap();
        let worker = Arc::new(BytesWorker::default());
        let model = Arc::new(TextModel::default());
        let pipeline = TextRig::open(&temp, &worker, &model);
        TextRig {
            temp,
            inbox,
            worker,
            model,
            pipeline,
        }
    }

    fn settings(temp: &TempDir) -> SettingsStore {
        SettingsStore::new(temp.path().join("settings.json"))
    }

    /// The queue over this rig's database, as the app opens it on start. A
    /// model missing at one moment is missing for good at the next: the
    /// grace a switch between models is given is the clock's, not a test's.
    fn open(temp: &TempDir, worker: &Arc<BytesWorker>, model: &Arc<TextModel>) -> Pipeline {
        Pipeline::with_local_files(
            temp.path().join("queue.sqlite3"),
            worker.clone(),
            model.clone(),
            Arc::new(Quiet),
            TextRig::settings(temp),
        )
        .unwrap()
        .with_model_missing_grace(std::time::Duration::ZERO)
    }

    /// Quits the app and starts it again over the same database, as after
    /// a crash or a restart.
    fn restart(self) -> TextRig {
        let TextRig {
            temp,
            inbox,
            worker,
            model,
            pipeline,
        } = self;
        drop(pipeline);
        let pipeline = TextRig::open(&temp, &worker, &model);
        pipeline.recover().unwrap();
        TextRig {
            temp,
            inbox,
            worker,
            model,
            pipeline,
        }
    }

    fn break_down(&self, outage: Outage) {
        match outage {
            Outage::TimesOut => *self.model.outage.lock().unwrap() = Some("MODEL_TIMEOUT"),
            Outage::AnswersGarbage => {
                *self.model.outage.lock().unwrap() = Some("MODEL_RESPONSE_INVALID")
            }
            Outage::ReaderCrashes => self.worker.broken.store(true, Ordering::SeqCst),
        }
    }

    fn fix(&self) {
        *self.model.outage.lock().unwrap() = None;
        self.worker.broken.store(false, Ordering::SeqCst);
    }

    fn add(&self, name: &str, body: &str) -> i64 {
        let path = self.inbox.join(name);
        fs::write(&path, body).unwrap();
        self.pipeline
            .enqueue_files(&[path])
            .unwrap()
            .pop()
            .unwrap()
            .id
    }

    fn run(&self) {
        self.pipeline.run_until_idle().unwrap();
    }

    fn resume(&self) {
        self.pipeline.resume();
        self.run();
    }

    fn status(&self, id: i64) -> QueueStatus {
        self.item(id).status
    }

    fn item(&self, id: i64) -> PipelineItem {
        self.pipeline
            .list()
            .unwrap()
            .into_iter()
            .find(|item| item.id == id)
            .unwrap()
    }

    fn renamed(&self) -> Vec<String> {
        names_in(&self.inbox)
            .into_iter()
            .filter(|name| name.starts_with("NEEDS REVIEW"))
            .collect()
    }

    fn journal_rows(&self, id: i64) -> i64 {
        rusqlite::Connection::open(self.temp.path().join("queue.sqlite3"))
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM review_renames WHERE queue_item_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap()
    }
}

/// (a) The machine breaks for everything - a model that times out or
/// answers garbage, a reader that crashes - and a person resumes the queue
/// again and again before it is fixed. Every stop is the machine's, and
/// nothing is ever renamed. Once it is fixed, every document is filed.
/// Through a restart in the middle, too (e).
#[test]
fn an_outage_renames_nothing_however_often_the_queue_is_resumed() {
    for outage in Outage::ALL {
        for restart in [false, true] {
            let mut rig = TextRig::new();
            let ids = (0..9)
                .map(|index| rig.add(&format!("d{index}.pdf"), &format!("good {index}")))
                .collect::<Vec<_>>();
            rig.break_down(outage);
            rig.run();
            assert_eq!(
                rig.pipeline.pause_reason().as_deref(),
                Some(outage.reason()),
                "{outage:?}"
            );
            if restart {
                rig = rig.restart();
            }
            for _ in 0..4 {
                rig.resume();
                assert_eq!(
                    rig.pipeline.pause_reason().as_deref(),
                    Some(outage.reason()),
                    "{outage:?}: still broken"
                );
                assert!(rig.renamed().is_empty(), "{outage:?}: {:?}", rig.renamed());
            }
            for id in &ids {
                let item = rig.item(*id);
                assert_eq!(item.status, QueueStatus::Queued, "{outage:?}");
                assert_eq!(item.processing_failures, 0, "{outage:?}");
            }

            rig.fix();
            rig.resume();
            assert!(!rig.pipeline.is_paused(), "{outage:?}");
            for id in &ids {
                assert_eq!(rig.status(*id), QueueStatus::Completed, "{outage:?}");
            }
            assert!(names_in(&rig.inbox).is_empty(), "{outage:?}: all filed");
        }
    }
}

/// The reviewer's sequence: one resume while still broken, then fixed with
/// documents not yet tried - one of those, read, proves the machine, and
/// every document in doubt is read and filed.
#[test]
fn an_outage_fixed_mid_backlog_files_everything() {
    for outage in Outage::ALL {
        let rig = TextRig::new();
        let ids = (0..9)
            .map(|index| rig.add(&format!("d{index}.pdf"), &format!("good {index}")))
            .collect::<Vec<_>>();
        rig.break_down(outage);
        rig.run();
        rig.resume();
        assert!(rig.pipeline.is_paused(), "{outage:?}");
        rig.fix();
        rig.resume();
        assert!(!rig.pipeline.is_paused(), "{outage:?}");
        for id in &ids {
            assert_eq!(rig.status(*id), QueueStatus::Completed, "{outage:?}");
        }
        assert!(rig.renamed().is_empty(), "{outage:?}");
    }
}

/// (b) One document that really is bad among good ones: the good ones
/// prove the machine works, so its second failure is its own - in review,
/// renamed. Through a restart in the middle, too (e).
#[test]
fn one_bad_document_among_good_ones_goes_to_review_renamed() {
    for outage in Outage::ALL {
        for restart in [false, true] {
            let mut rig = TextRig::new();
            let first = rig.add("a.pdf", "good a");
            let bad = rig.add("b.pdf", &format!("{} b", outage.word()));
            if restart {
                // Stop after the bad document's first failure.
                rig.pipeline.run_next().unwrap();
                rig.pipeline.run_next().unwrap();
                assert_eq!(rig.status(bad), QueueStatus::Queued, "{outage:?}");
                rig = rig.restart();
            }
            let last = rig.add("c.pdf", "good c");
            rig.run();

            assert!(!rig.pipeline.is_paused(), "{outage:?}");
            assert_eq!(rig.status(first), QueueStatus::Completed, "{outage:?}");
            assert_eq!(rig.status(last), QueueStatus::Completed, "{outage:?}");
            assert_eq!(rig.status(bad), QueueStatus::NeedsReview, "{outage:?}");
            assert_eq!(rig.renamed(), vec!["NEEDS REVIEW - b.pdf"], "{outage:?}");
        }
    }
}

/// (c) A lone document that really is bad, with nothing else to show the
/// machine works: the self-test does (the model's calibration document, or
/// for a text file the reader crashed on, a small text file of Intern's
/// own), and its second failure is its own. Through a restart in the
/// middle, too (e).
#[test]
fn a_lone_bad_document_is_proven_its_own_by_the_self_test() {
    for outage in Outage::ALL {
        for restart in [false, true] {
            let mut rig = TextRig::new();
            let name = match outage {
                Outage::ReaderCrashes => "a.txt",
                _ => "a.pdf",
            };
            let bad = rig.add(name, &format!("{} a", outage.word()));
            if restart {
                rig.pipeline.run_next().unwrap();
                assert_eq!(rig.status(bad), QueueStatus::Queued, "{outage:?}");
                assert!(rig.renamed().is_empty(), "{outage:?}");
                rig = rig.restart();
            }
            rig.run();

            assert!(!rig.pipeline.is_paused(), "{outage:?}");
            assert_eq!(rig.status(bad), QueueStatus::NeedsReview, "{outage:?}");
            assert_eq!(
                rig.renamed(),
                vec![format!("NEEDS REVIEW - {name}")],
                "{outage:?}"
            );
            let checks = names_in(rig.temp.path())
                .into_iter()
                .filter(|name| name.starts_with("intern-reader-check"))
                .count();
            assert_eq!(checks, 0, "the reader's check file is cleaned up");
        }
    }
}

/// A reader crash on a PDF has no self-test: a reader that crashes on every
/// PDF reads a text file fine. A lone PDF that crashes it waits, under its
/// own name, for another PDF to show the reader works - and only then is a
/// second crash its own.
#[test]
fn a_lone_pdf_that_crashes_the_reader_waits_for_another_pdf() {
    let rig = TextRig::new();
    let bad = rig.add("a.pdf", "CRASH a");
    rig.run();
    assert!(!rig.pipeline.is_paused());
    assert_eq!(rig.status(bad), QueueStatus::Queued);
    assert!(rig.renamed().is_empty());

    // A text file read proves nothing about PDFs.
    let text = rig.add("notes.txt", "good notes");
    rig.run();
    assert_eq!(rig.status(text), QueueStatus::Completed);
    assert_eq!(rig.status(bad), QueueStatus::Queued);
    assert!(rig.renamed().is_empty());

    let good = rig.add("b.pdf", "good b");
    rig.run();
    assert_eq!(rig.status(good), QueueStatus::Completed);
    assert_eq!(rig.status(bad), QueueStatus::NeedsReview);
    assert_eq!(rig.renamed(), vec!["NEEDS REVIEW - a.pdf"]);
}

/// A reader that crashes on every PDF but reads everything else: the PDFs
/// wait in doubt and are never renamed, a text file read in between proves
/// nothing about them, and several PDFs waiting with none to prove the
/// reader stop the queue for it. Resuming while it is still broken tries
/// them once more and renames nothing; fixed, they are filed.
#[test]
fn a_reader_broken_for_one_kind_of_file_renames_none_of_them() {
    let rig = TextRig::new();
    let pdfs = ["a.pdf", "b.pdf"].map(|name| rig.add(name, &format!("good {name}")));
    let text = rig.add("c.txt", "good c");
    rig.worker.pdfs_broken.store(true, Ordering::SeqCst);
    rig.run();

    assert_eq!(rig.status(text), QueueStatus::Completed);
    assert_eq!(
        rig.pipeline.pause_reason().as_deref(),
        Some("READER_CRASHED")
    );
    for _ in 0..3 {
        rig.resume();
        assert_eq!(
            rig.pipeline.pause_reason().as_deref(),
            Some("READER_CRASHED")
        );
    }
    for id in pdfs {
        assert_eq!(rig.status(id), QueueStatus::Queued);
    }
    assert!(rig.renamed().is_empty());

    rig.worker.pdfs_broken.store(false, Ordering::SeqCst);
    rig.resume();
    for id in pdfs {
        assert_eq!(rig.status(id), QueueStatus::Completed);
    }
}

/// The reader's check file is written in Intern's own data folder. When it
/// cannot be, that says nothing about the reader: the queue stops for that
/// reason, not as if the reader had crashed.
#[test]
fn a_reader_check_that_cannot_be_written_stops_the_queue_for_that() {
    let rig = TextRig::new();
    let bad = rig.add("a.txt", "CRASH a");
    rig.pipeline.run_next().unwrap();
    assert_eq!(rig.status(bad), QueueStatus::Queued);
    // The data folder goes away under the running app (the database stays
    // open), as a removed drive or a cleaned-up folder would.
    fs::remove_dir_all(rig.temp.path()).unwrap();
    rig.run();

    assert_eq!(
        rig.pipeline.pause_reason().as_deref(),
        Some("READER_CHECK_UNWRITABLE")
    );
    assert_eq!(rig.status(bad), QueueStatus::Queued);
}

/// (d) A lone document while the machine is broken: the self-test fails
/// too, so the queue stops for the machine and the document keeps its name.
/// A reader crash on a PDF has no self-test, so the document just waits;
/// resuming tries it once more, proving nothing. Fixed, it is filed.
/// Through a restart in the middle, too (e).
#[test]
fn a_lone_document_with_a_broken_machine_stops_the_queue_and_keeps_its_name() {
    for outage in Outage::ALL {
        for restart in [false, true] {
            let mut rig = TextRig::new();
            let id = rig.add("a.pdf", "good a");
            let waits_unpaused = matches!(outage, Outage::ReaderCrashes);
            rig.break_down(outage);
            rig.run();
            let expected = (!waits_unpaused).then_some(outage.reason());
            assert_eq!(
                rig.pipeline.pause_reason().as_deref(),
                expected,
                "{outage:?}"
            );
            assert_eq!(rig.status(id), QueueStatus::Queued, "{outage:?}");
            assert!(rig.renamed().is_empty(), "{outage:?}");
            if restart {
                rig = rig.restart();
            }
            rig.resume();
            assert_eq!(rig.pipeline.is_paused(), !waits_unpaused, "{outage:?}");
            assert_eq!(rig.status(id), QueueStatus::Queued, "{outage:?}");
            assert!(rig.renamed().is_empty(), "{outage:?}");

            rig.fix();
            rig.resume();
            assert_eq!(rig.status(id), QueueStatus::Completed, "{outage:?}");
        }
    }
}

/// Two documents in a row that fail are two, not one per attempt, and never
/// a run that stops the queue. The good one behind them is filed, and shows
/// the machine works, so their failures on the next attempt are their own.
#[test]
fn two_failing_documents_are_not_a_run() {
    for outage in [Outage::TimesOut, Outage::ReaderCrashes] {
        let rig = TextRig::new();
        let word = outage.word();
        let a = rig.add("a.pdf", &format!("{word} a"));
        let b = rig.add("b.pdf", &format!("{word} b"));
        let c = rig.add("c.pdf", "good c");

        rig.run();

        assert!(!rig.pipeline.is_paused(), "{outage:?}");
        assert_eq!(rig.status(c), QueueStatus::Completed, "{outage:?}");
        assert_eq!(rig.status(a), QueueStatus::NeedsReview, "{outage:?}");
        assert_eq!(rig.status(b), QueueStatus::NeedsReview, "{outage:?}");
        assert_eq!(
            rig.renamed(),
            vec!["NEEDS REVIEW - a.pdf", "NEEDS REVIEW - b.pdf"],
            "{outage:?}"
        );
    }
}

/// Three documents in a row that fail the same way stop the queue and wait
/// in doubt. Resumed, the good document behind them is read, which shows
/// the machine works: failing again, theirs is their own. However often the
/// queue is resumed, it does not stop on them again.
#[test]
fn a_resumed_queue_moves_past_the_documents_that_stopped_it() {
    for outage in Outage::ALL {
        let rig = TextRig::new();
        let failing = ["a.pdf", "b.pdf", "c.pdf"]
            .into_iter()
            .map(|name| rig.add(name, &format!("{} {name}", outage.word())))
            .collect::<Vec<_>>();
        let good = rig.add("d.pdf", "good d");

        rig.run();
        assert_eq!(
            rig.pipeline.pause_reason().as_deref(),
            Some(outage.reason()),
            "{outage:?}"
        );
        for id in &failing {
            let item = rig.item(*id);
            assert_eq!(item.status, QueueStatus::Queued, "{outage:?}");
            assert_eq!(item.processing_failures, 0, "{outage:?}");
        }
        assert!(rig.renamed().is_empty(), "{outage:?}");

        for _ in 0..3 {
            rig.resume();
            assert!(!rig.pipeline.is_paused(), "{outage:?}: stopped again");
        }
        assert_eq!(rig.status(good), QueueStatus::Completed, "{outage:?}");
        for id in &failing {
            assert_eq!(rig.status(*id), QueueStatus::NeedsReview, "{outage:?}");
        }
        assert_eq!(
            rig.renamed(),
            vec![
                "NEEDS REVIEW - a.pdf",
                "NEEDS REVIEW - b.pdf",
                "NEEDS REVIEW - c.pdf"
            ],
            "{outage:?}"
        );
    }
}

/// A document found to be its own problem after the input folder changed
/// in Settings is not renamed: Intern renames only what the input folder
/// holds now.
#[test]
fn a_document_outside_the_input_folder_by_the_time_it_is_judged_keeps_its_name() {
    let rig = TextRig::new();
    let slow = rig.add("a.pdf", "SLOW a");
    rig.break_down(Outage::TimesOut);
    rig.run();
    assert!(rig.pipeline.is_paused());

    let elsewhere = rig.temp.path().join("Elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let settings = TextRig::settings(&rig.temp);
    let mut changed = settings.load().unwrap();
    changed.intake_folder = elsewhere.to_string_lossy().into_owned();
    settings.save(&changed).unwrap();
    rig.fix();
    rig.resume();
    rig.pipeline.recover().unwrap();

    assert_eq!(rig.status(slow), QueueStatus::NeedsReview);
    assert_eq!(names_in(&rig.inbox), vec!["a.pdf"]);
}

/// A canceled document whose marked file was deleted has no name to get
/// back: recovery gives up on it rather than journaling a new try on every
/// pass for as long as the app runs.
#[test]
fn a_canceled_document_whose_file_is_gone_is_not_unmarked_for_ever() {
    let rig = TextRig::new();
    let id = rig.add("locked.pdf", "LOCKED");
    rig.run();
    let marked = rig.inbox.join("NEEDS REVIEW - locked.pdf");
    assert!(marked.exists());
    let before = rig.journal_rows(id);
    fs::remove_file(&marked).unwrap();
    rig.pipeline.cancel(id).unwrap();

    for _ in 0..10 {
        rig.pipeline.recover().unwrap();
    }

    assert_eq!(rig.journal_rows(id), before);
    assert_eq!(rig.status(id), QueueStatus::Canceled);
    assert_eq!(rig.pipeline.clear_history().unwrap(), 1);
}

/// A claim of the intake watcher waits for the watcher to see its item
/// gone; with the watcher turned off, clearing the history drops it.
#[test]
fn clearing_the_history_with_intake_off_drops_claims_on_cleared_items() {
    let rig = TextRig::new();
    let id = rig.add("locked.pdf", "LOCKED");
    rig.pipeline.record_intake_claim("claim", id).unwrap();
    rig.pipeline.remove(id, false).unwrap();
    let claims = || -> i64 {
        rusqlite::Connection::open(rig.temp.path().join("queue.sqlite3"))
            .unwrap()
            .query_row("SELECT COUNT(*) FROM intake_claims", [], |row| row.get(0))
            .unwrap()
    };
    rig.pipeline.clear_history().unwrap();
    rig.pipeline.clear_history().unwrap();
    assert_eq!(
        claims(),
        1,
        "kept while the watcher is on and has not looked"
    );

    let settings = TextRig::settings(&rig.temp);
    let mut off = settings.load().unwrap();
    off.intake_enabled = false;
    settings.save(&off).unwrap();
    rig.pipeline.clear_history().unwrap();
    assert_eq!(claims(), 0);
}

/// Canceling a marked document and clearing the history leaves its file
/// under its own name, so the next scan takes it in again.
#[test]
fn a_canceled_and_cleared_document_has_its_own_name() {
    let rig = TextRig::new();
    let id = rig.add("locked.pdf", "LOCKED");
    rig.run();
    assert_eq!(names_in(&rig.inbox), vec!["NEEDS REVIEW - locked.pdf"]);
    rig.pipeline.cancel(id).unwrap();
    rig.pipeline.clear_history().unwrap();

    assert_eq!(names_in(&rig.inbox), vec!["locked.pdf"]);
    assert!(rig.pipeline.list().unwrap().is_empty());
}

/// Only documents in doubt are left, and no model is loaded for the
/// self-test to ask. As for a document that meets no model, the moment of a
/// switch between models passes; a model still missing after it stops the
/// queue and says so, rather than leaving the documents waiting in silence.
#[test]
fn doubted_documents_with_no_model_loaded_stop_the_queue_after_the_grace() {
    let rig = TextRig::new();
    let slow = rig.add("a.pdf", "SLOW a");
    rig.pipeline.run_next().unwrap();
    assert_eq!(rig.status(slow), QueueStatus::Queued);
    *rig.model.outage.lock().unwrap() = Some("MODEL_NOT_READY");

    rig.run();
    assert!(!rig.pipeline.is_paused(), "the moment of a switch");
    rig.run();
    assert_eq!(rig.pipeline.pause_reason().as_deref(), Some("MODEL_FAILED"));
    assert_eq!(rig.status(slow), QueueStatus::Queued);
    assert!(rig.renamed().is_empty());
}

/// A document in doubt that a person cancels and then retries starts afresh:
/// it is read in its turn like any new document, needing no proof that the
/// machine works.
#[test]
fn a_doubted_document_canceled_and_retried_is_read_afresh() {
    let rig = TextRig::new();
    let id = rig.add("a.pdf", "good a");
    rig.break_down(Outage::TimesOut);
    rig.pipeline.run_next().unwrap();
    assert_eq!(rig.status(id), QueueStatus::Queued);
    assert_eq!(rig.item(id).doubt.as_deref(), Some("timeout"));
    rig.pipeline.cancel(id).unwrap();
    assert_eq!(rig.item(id).doubt, None);
    rig.fix();
    rig.pipeline.retry(id).unwrap();
    rig.run();

    assert_eq!(rig.status(id), QueueStatus::Completed);
    assert!(!rig.pipeline.is_paused());
    assert_eq!(
        rig.model.calibrations.load(Ordering::SeqCst),
        0,
        "no proof was asked for"
    );
}
