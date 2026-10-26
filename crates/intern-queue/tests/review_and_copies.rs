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
        atomic::{AtomicUsize, Ordering},
    },
};

use intern_core::{ErrorCode, QueueStatus, QueueStore};
use intern_engine::{
    AnalysisTelemetry, DateRole, DigestBudget, DocumentAnalysis, DocumentSource, Evidence,
    ExtractProgress, ModelProposal, PageOrigin, PartyRelation, SourcePage, distill, engine::finish,
    fingerprint, validate,
};
use intern_queue::{
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
        let temp = tempdir().unwrap();
        let inbox = temp.path().join("Inbox");
        let filed = temp.path().join("Filed");
        fs::create_dir_all(&inbox).unwrap();
        fs::create_dir_all(&filed).unwrap();
        let inbox = inbox.canonicalize().unwrap();
        let filed = filed.canonicalize().unwrap();
        let mut settings = AppSettings {
            destination: filed.to_string_lossy().into_owned(),
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
/// review under that reason, with its file renamed where it lies - and the
/// queue goes on to the next document.
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
    let mut reads = extraction
        .iter()
        .map(|(code, _)| Err(WorkerFailure::reported(*code, "the worker's words", false)))
        .collect::<Vec<_>>();
    reads.extend(model.iter().map(|_| Ok(parsed(READABLE))));
    let mut replies = model
        .iter()
        .map(|(code, _)| Err(ModelFailure::fatal(*code)))
        .collect::<Vec<_>>();
    // A reply that could not be used, then one that could: the row of
    // unusable replies is broken before it can stop the queue.
    replies.insert(2, Ok(proposal()));
    let rig = Rig::new(reads, replies, |settings| settings.automatic_rename = false);
    let mut ids = Vec::new();
    for index in 0..extraction.len() + model.len() {
        ids.push(rig.add(&rig.drop_in(
            &format!("doc-{index}.pdf"),
            format!("doc {index}").as_bytes(),
        )));
        if index == extraction.len() + 1 {
            ids.push(rig.add(&rig.drop_in("fine.pdf", b"a fine document")));
        }
    }

    rig.pipeline.run_until_idle().unwrap();

    assert!(!rig.pipeline.is_paused());
    let fine = ids.remove(extraction.len() + 2);
    assert_eq!(rig.item(fine).status, QueueStatus::Ready);
    for ((index, id), (code, expected)) in ids
        .iter()
        .enumerate()
        .zip(extraction.iter().chain(model.iter()))
    {
        let item = rig.item(*id);
        assert_eq!(item.status, QueueStatus::NeedsReview, "{code}");
        assert_eq!(item.error_code, Some(*expected), "{code}");
        assert!(item.proposal.is_none(), "{code}");
        let marked = rig.inbox.join(format!("NEEDS REVIEW - doc-{index}.pdf"));
        assert_eq!(item.source_path, marked, "{code}");
        assert!(marked.exists(), "{code}");
        assert!(
            !rig.inbox.join(format!("doc-{index}.pdf")).exists(),
            "{code}"
        );
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

/// Two attempts that each fail in a way that may pass - a parser crash, a
/// model running out of time - are the document's problem once both are
/// used: it waits in review rather than failing.
#[test]
fn a_document_that_crashes_the_parser_or_times_out_twice_goes_to_review() {
    let rig = Rig::new(
        vec![
            Err(WorkerFailure::crashed()),
            Err(WorkerFailure::crashed()),
            Ok(parsed(READABLE)),
            Ok(parsed(READABLE)),
        ],
        vec![
            Err(ModelFailure::fatal("MODEL_TIMEOUT")),
            Err(ModelFailure::fatal("MODEL_TIMEOUT")),
        ],
        |_| {},
    );
    let crashing = rig.add(&rig.drop_in("crashes.pdf", b"crashes the parser"));
    let slow = rig.add(&rig.drop_in("slow.pdf", b"too slow to read"));

    rig.pipeline.run_until_idle().unwrap();

    let crashing = rig.item(crashing);
    assert_eq!(crashing.status, QueueStatus::NeedsReview);
    assert_eq!(crashing.error_code, Some(ErrorCode::ExtractionFailed));
    assert_eq!(
        crashing.source_path,
        rig.inbox.join("NEEDS REVIEW - crashes.pdf")
    );
    let slow = rig.item(slow);
    assert_eq!(slow.status, QueueStatus::NeedsReview);
    assert_eq!(slow.error_code, Some(ErrorCode::ModelFailed));
    assert_eq!(slow.source_path, rig.inbox.join("NEEDS REVIEW - slow.pdf"));
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
            .record_unreadable(item.id, ErrorCode::PasswordProtected)
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
