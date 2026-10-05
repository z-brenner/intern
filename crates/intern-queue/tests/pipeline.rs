use std::{
    collections::{HashMap, VecDeque},
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc, Barrier, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use intern_core::{
    ErrorCode, FileSystem, LockedFile, OperationReceipt, OperationStage, PrivateSnapshotDirectory,
    QueueItem, QueueStatus, QueueStore, StdFileSystem,
};
use intern_engine::{
    AnalysisTelemetry, DateRole, DigestBudget, DocumentAnalysis, DocumentSource, Evidence,
    ExtractProgress, ModelProposal, PageOrigin, ParserWarning, PartyRelation, ProposalStatus,
    SourcePage, distill, engine::finish, fingerprint, validate,
};
use intern_intake::{
    SharePointDeployment,
    microsoft::{
        Account,
        hashing::QuickXor,
        proof::{FreshUploadMetadata, FreshUploadOutcome, verify_fresh_upload},
    },
};
use intern_queue::{
    paths::display_path,
    pipeline::{
        ALREADY_NAMED, AnalyzerBoundary, CoreFileActions, DuplicateOracle, FileActions,
        FiledDocument, FilingSink, KnownFiling, ModelFailure, NEAR_DUPLICATE, Pipeline,
        PipelineError, PipelineEventSink, PipelineProgress, SimilarFiling, UNDONE, UnfiledDocument,
        WorkerBoundary, WorkerFailure,
    },
    settings::{AppSettings, DestinationLayout, SettingsStore},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use tempfile::tempdir;
use url::Url;

/// Runs the real distillation, validation, and naming over a canned model
/// reply, so queue tests still exercise the production evidence rules instead
/// of a stub that always agrees with itself.
fn analyze_locally(
    source: &DocumentSource,
    proposal: ModelProposal,
    extension: &str,
    existing_names: &[&str],
) -> DocumentAnalysis {
    let digest = distill(source, DigestBudget::default());
    let outcome = validate(proposal, &digest);
    let mut analysis = finish(
        outcome,
        &digest,
        extension,
        existing_names,
        AnalysisTelemetry::default(),
    );
    analysis.text_fingerprint = fingerprint::source_fingerprint(source).map(fingerprint::encode);
    analysis
}

#[derive(Default)]
struct RecordingEvents {
    changed: AtomicUsize,
    progress: Mutex<Vec<PipelineProgress>>,
}

/// Remembers every filed-document report and every retraction, in order.
#[derive(Default)]
struct RecordingFiling {
    filed: Mutex<Vec<FiledDocument>>,
    unfiled: Mutex<Vec<UnfiledDocument>>,
}

impl FilingSink for RecordingFiling {
    fn filed(&self, document: &FiledDocument) {
        self.filed.lock().unwrap().push(document.clone());
    }
    fn unfiled(&self, document: &UnfiledDocument) {
        self.unfiled.lock().unwrap().push(document.clone());
    }
}

/// What teammates have filed, keyed by content hash - the shared filed index
/// as the queue sees it.
#[derive(Default)]
struct TeammateFilings {
    known: Mutex<HashMap<String, KnownFiling>>,
    asked: Mutex<Vec<(String, PathBuf)>>,
    /// What the shared index answers about a text fingerprint.
    similar: Mutex<Option<SimilarFiling>>,
}

impl DuplicateOracle for TeammateFilings {
    fn filed_elsewhere(&self, source_hash: &str, source_path: &Path) -> Option<KnownFiling> {
        self.asked
            .lock()
            .unwrap()
            .push((source_hash.to_owned(), source_path.to_path_buf()));
        self.known.lock().unwrap().get(source_hash).cloned()
    }

    fn similar_elsewhere(&self, _fingerprint: u64) -> Option<SimilarFiling> {
        self.similar.lock().unwrap().clone()
    }
}

impl PipelineEventSink for RecordingEvents {
    fn queue_changed(&self) {
        self.changed.fetch_add(1, Ordering::SeqCst);
    }
    fn progress(&self, progress: PipelineProgress) {
        self.progress.lock().unwrap().push(progress);
    }
}

struct FakeWorker {
    responses: Mutex<VecDeque<Result<DocumentSource, WorkerFailure>>>,
    calls: AtomicUsize,
    active: AtomicUsize,
    maximum_active: AtomicUsize,
    cancellations: AtomicUsize,
    restarts: AtomicUsize,
    gate: (Mutex<bool>, Condvar),
}

impl FakeWorker {
    fn new(responses: Vec<Result<DocumentSource, WorkerFailure>>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            calls: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            maximum_active: AtomicUsize::new(0),
            cancellations: AtomicUsize::new(0),
            restarts: AtomicUsize::new(0),
            gate: (Mutex::new(false), Condvar::new()),
        }
    }

    fn blocking(response: Result<DocumentSource, WorkerFailure>) -> Self {
        let worker = Self::new(vec![response]);
        *worker.gate.0.lock().unwrap() = true;
        worker
    }
}

impl WorkerBoundary for FakeWorker {
    fn extract(
        &self,
        _request_id: &str,
        _path: &Path,
        _progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, WorkerFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum_active.fetch_max(active, Ordering::SeqCst);
        let (lock, wake) = &self.gate;
        let mut blocked = lock.lock().unwrap();
        while *blocked && self.cancellations.load(Ordering::SeqCst) == 0 {
            blocked = wake.wait(blocked).unwrap();
        }
        drop(blocked);
        self.active.fetch_sub(1, Ordering::SeqCst);
        if self.cancellations.load(Ordering::SeqCst) > 0 {
            return Err(WorkerFailure::canceled());
        }
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Err(WorkerFailure::new("PARSE_FAILED", false, false)))
    }

    fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
        self.cancellations.fetch_add(1, Ordering::SeqCst);
        self.gate.1.notify_all();
        Ok(())
    }

    fn restart(&self) -> Result<(), WorkerFailure> {
        self.restarts.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn shutdown(&self) -> Result<(), WorkerFailure> {
        self.cancellations.fetch_add(1, Ordering::SeqCst);
        self.gate.1.notify_all();
        Ok(())
    }
}

/// A worker that holds its first request open until a test releases it, and
/// then reports a crash it cannot be restarted from - every time.
#[derive(Default)]
struct CrashingWorker {
    started: AtomicBool,
    gate: (Mutex<bool>, Condvar),
}

impl CrashingWorker {
    fn release(&self) {
        *self.gate.0.lock().unwrap() = true;
        self.gate.1.notify_all();
    }
}

impl WorkerBoundary for CrashingWorker {
    fn extract(
        &self,
        _request_id: &str,
        _path: &Path,
        _progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, WorkerFailure> {
        self.started.store(true, Ordering::SeqCst);
        let (lock, wake) = &self.gate;
        let mut released = lock.lock().unwrap();
        while !*released {
            released = wake.wait(released).unwrap();
        }
        Err(WorkerFailure::crashed())
    }

    fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
        Ok(())
    }

    fn restart(&self) -> Result<(), WorkerFailure> {
        Err(WorkerFailure::new("WORKER_RESTART_FAILED", false, false))
    }

    fn shutdown(&self) -> Result<(), WorkerFailure> {
        Ok(())
    }
}

struct FakeModel {
    responses: Mutex<VecDeque<Result<ModelProposal, ModelFailure>>>,
    calls: AtomicUsize,
}

impl FakeModel {
    fn new(responses: Vec<Result<ModelProposal, ModelFailure>>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            calls: AtomicUsize::new(0),
        }
    }
}

impl AnalyzerBoundary for FakeModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let proposal = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Err(ModelFailure::fatal("MODEL_RESPONSE_INVALID")))?;
        Ok(analyze_locally(source, proposal, extension, existing_names))
    }
}

struct GatedSuccessModel {
    calls: AtomicUsize,
    gate: (Mutex<bool>, Condvar),
}

struct RecoveringModel {
    responses: Mutex<VecDeque<Result<ModelProposal, ModelFailure>>>,
    calls: AtomicUsize,
    recoveries: AtomicUsize,
    recovery_succeeds: bool,
}

impl RecoveringModel {
    fn new(responses: Vec<Result<ModelProposal, ModelFailure>>, recovery_succeeds: bool) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            calls: AtomicUsize::new(0),
            recoveries: AtomicUsize::new(0),
            recovery_succeeds,
        }
    }
}

impl AnalyzerBoundary for RecoveringModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let proposal = self.responses.lock().unwrap().pop_front().unwrap()?;
        Ok(analyze_locally(source, proposal, extension, existing_names))
    }

    fn recover(&self, _failure: &ModelFailure) -> Result<(), ModelFailure> {
        self.recoveries.fetch_add(1, Ordering::SeqCst);
        if self.recovery_succeeds {
            Ok(())
        } else {
            Err(ModelFailure::fatal("MODEL_RECOVERY_FAILED"))
        }
    }
}

impl GatedSuccessModel {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            gate: (Mutex::new(true), Condvar::new()),
        }
    }

    fn release(&self) {
        *self.gate.0.lock().unwrap() = false;
        self.gate.1.notify_all();
    }
}

impl AnalyzerBoundary for GatedSuccessModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (lock, wake) = &self.gate;
        let mut blocked = lock.lock().unwrap();
        while *blocked {
            blocked = wake.wait(blocked).unwrap();
        }
        Ok(analyze_locally(
            source,
            proposal(0.94, false),
            extension,
            existing_names,
        ))
    }
}

struct BlockingModel {
    calls: AtomicUsize,
    cancel_started: AtomicBool,
    cancel_succeeds: bool,
    request_release: (Mutex<bool>, Condvar),
    cancel_release: (Mutex<bool>, Condvar),
}

impl BlockingModel {
    fn new(cancel_succeeds: bool) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            cancel_started: AtomicBool::new(false),
            cancel_succeeds,
            request_release: (Mutex::new(false), Condvar::new()),
            cancel_release: (Mutex::new(false), Condvar::new()),
        }
    }

    fn wait_for_cancel(&self) {
        // Thirty seconds, not two. The loop exits as soon as the flag is set, so a
        // healthy run is no slower; the deadline only decides how much scheduling
        // delay on a loaded runner counts as a failure, and two seconds of it is
        // ordinary rather than broken.
        let deadline = Instant::now() + Duration::from_secs(30);
        while !self.cancel_started.load(Ordering::SeqCst) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(self.cancel_started.load(Ordering::SeqCst));
    }

    fn release_request(&self) {
        *self.request_release.0.lock().unwrap() = true;
        self.request_release.1.notify_all();
    }

    fn release_cancel(&self) {
        *self.cancel_release.0.lock().unwrap() = true;
        self.cancel_release.1.notify_all();
    }
}

impl AnalyzerBoundary for BlockingModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if call == 1 {
            let (lock, wake) = &self.request_release;
            let mut released = lock.lock().unwrap();
            while !*released {
                released = wake.wait(released).unwrap();
            }
            return Err(ModelFailure::fatal("MODEL_REQUEST_CANCELED"));
        }
        Ok(analyze_locally(
            source,
            proposal(0.94, false),
            extension,
            existing_names,
        ))
    }

    fn cancel(&self) -> Result<(), ModelFailure> {
        self.cancel_started.store(true, Ordering::SeqCst);
        if self.cancel_succeeds {
            self.release_request();
        }
        let (lock, wake) = &self.cancel_release;
        let mut released = lock.lock().unwrap();
        while !*released {
            released = wake.wait(released).unwrap();
        }
        if self.cancel_succeeds {
            Ok(())
        } else {
            Err(ModelFailure::fatal("MODEL_CANCEL_FAILED"))
        }
    }
}

#[derive(Default)]
struct FakeFiles {
    hashes: Mutex<HashMap<PathBuf, String>>,
    fingerprint_calls: AtomicUsize,
    applies: Mutex<Vec<i64>>,
    reconciles: Mutex<Vec<i64>>,
    fail_next_apply: AtomicUsize,
}

impl FakeFiles {
    fn trust(&self, path: &Path, hash: &str) {
        self.hashes
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), hash.to_owned());
    }

    fn forget(&self, path: &Path) {
        self.hashes.lock().unwrap().remove(path);
    }
    fn fail_next_apply(&self) {
        self.fail_next_apply.store(1, Ordering::SeqCst);
    }
}

impl FileActions for FakeFiles {
    fn fingerprint(&self, path: &Path) -> Result<String, PipelineError> {
        self.fingerprint_calls.fetch_add(1, Ordering::SeqCst);
        self.hashes
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| PipelineError::new("FILE_MISSING", "source file is unavailable"))
    }

    fn apply(&self, item: &QueueItem, _destination: &Path) -> Result<(), PipelineError> {
        if self
            .fail_next_apply
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
        {
            return Err(PipelineError::new("MOVE_FAILED", "injected apply failure"));
        }
        self.applies.lock().unwrap().push(item.id);
        Ok(())
    }

    fn undo(&self, _item: &QueueItem, _receipt: &OperationReceipt) -> Result<(), PipelineError> {
        Ok(())
    }

    fn reconcile(&self, item: &QueueItem) -> Result<(), PipelineError> {
        self.reconciles.lock().unwrap().push(item.id);
        Ok(())
    }
}

fn parsed(text: &str) -> DocumentSource {
    DocumentSource::from_pages(vec![SourcePage::new(1, text, PageOrigin::Native)])
}

fn proposal(confidence: f32, needs_review: bool) -> ModelProposal {
    ModelProposal {
        document_type: Some("Employment Agreement".into()),
        document_date: Some("2024-04-12".into()),
        date_role: Some(DateRole::Execution),
        parties: vec!["John Smith".into(), "Acme Corporation".into()],
        party_relation: PartyRelation::Between,
        description:
            "Employment agreement between John Smith and Acme Corporation covering duties, salary, and term."
                .into(),
        confidence,
        needs_review,
        evidence: Evidence {
            date: Some("signed April 12, 2024".into()),
            document_type: Some("Employment Agreement".into()),
            parties: vec!["by John Smith and Acme Corporation".into()],
        },
    }
}

fn source(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, name.as_bytes()).unwrap();
    path.canonicalize().unwrap()
}

fn pipeline(
    root: &Path,
    worker: Arc<FakeWorker>,
    model: Arc<FakeModel>,
    files: Arc<FakeFiles>,
    settings: AppSettings,
) -> Pipeline {
    let settings_store = SettingsStore::new(root.join("settings.json"));
    settings_store.save(&settings).unwrap();
    Pipeline::open(
        root.join("queue.sqlite3"),
        worker,
        model,
        files,
        Arc::new(RecordingEvents::default()),
        settings_store,
    )
    .unwrap()
}

#[test]
fn mixed_queue_is_sequential_and_ready_review_are_evidence_gated() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "first.pdf");
    let second = source(temp.path(), "second.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement mentioning Acme Corporation only.",
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    let pipeline = pipeline(
        temp.path(),
        Arc::clone(&worker),
        model,
        Arc::clone(&files),
        AppSettings::default(),
    );

    pipeline.enqueue_files(&[first, second]).unwrap();
    pipeline.run_until_idle().unwrap();

    let items = pipeline.list().unwrap();
    assert_eq!(worker.maximum_active.load(Ordering::SeqCst), 1);
    assert_eq!(items[0].status, QueueStatus::Ready);
    assert_eq!(items[1].status, QueueStatus::NeedsReview);
    assert_eq!(
        items[0].proposal.as_ref().unwrap().status,
        ProposalStatus::Ready
    );
    assert_eq!(
        items[1].proposal.as_ref().unwrap().status,
        ProposalStatus::NeedsReview
    );
}

#[test]
fn retryable_malformed_model_output_retries_once_only() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "retry.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![
        Err(ModelFailure::retryable("MODEL_RESPONSE_INVALID")),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "retry-hash");
    let pipeline = pipeline(
        temp.path(),
        worker,
        model.clone(),
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&[path]).unwrap();
    pipeline.run_until_idle().unwrap();

    assert_eq!(model.calls.load(Ordering::SeqCst), 2);
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Ready);
}

#[test]
fn crashed_model_endpoint_is_recovered_once_before_the_only_retry() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "model-crash.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(RecoveringModel::new(
        vec![
            Err(ModelFailure::retryable("MODEL_REQUEST_FAILED")),
            Ok(proposal(0.94, false)),
        ],
        true,
    ));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "model-hash");
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings.save(&AppSettings::default()).unwrap();
    let pipeline = Pipeline::open(
        temp.path().join("queue.sqlite3"),
        worker,
        model.clone(),
        files,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    pipeline.enqueue_files(&[path]).unwrap();

    pipeline.run_until_idle().unwrap();

    assert_eq!(model.calls.load(Ordering::SeqCst), 2);
    assert_eq!(model.recoveries.load(Ordering::SeqCst), 1);
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Ready);
}

#[test]
fn failed_model_recovery_pauses_before_retrying_or_draining_next_item() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "model-crash.pdf");
    let second = source(temp.path(), "must-wait.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(RecoveringModel::new(
        vec![Err(ModelFailure::retryable("MODEL_REQUEST_FAILED"))],
        false,
    ));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings.save(&AppSettings::default()).unwrap();
    let pipeline = Pipeline::open(
        temp.path().join("queue.sqlite3"),
        worker,
        model.clone(),
        files,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    pipeline.enqueue_files(&[first, second]).unwrap();

    pipeline.run_until_idle().unwrap();

    assert!(pipeline.is_paused());
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
    assert_eq!(model.recoveries.load(Ordering::SeqCst), 1);
    assert_eq!(pipeline.list().unwrap()[1].status, QueueStatus::Queued);
}

#[test]
fn failed_item_does_not_block_the_next_and_worker_restarts_only_once() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "crashes.pdf");
    let second = source(temp.path(), "continues.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Err(WorkerFailure::crashed()),
        Err(WorkerFailure::crashed()),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "crash-hash");
    files.trust(&second, "continue-hash");
    let pipeline = pipeline(
        temp.path(),
        Arc::clone(&worker),
        model,
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&[first, second]).unwrap();
    pipeline.run_until_idle().unwrap();

    let items = pipeline.list().unwrap();
    assert_eq!(worker.restarts.load(Ordering::SeqCst), 1);
    assert_eq!(items[0].status, QueueStatus::Failed);
    assert_eq!(items[0].error_code, Some(ErrorCode::IoError));
    assert_eq!(items[1].status, QueueStatus::Ready);
}

const READABLE: &str =
    "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.";

/// Enqueues one trusted document per name, in order, and returns the paths.
fn documents(dir: &Path, files: &FakeFiles, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|name| {
            let path = source(dir, name);
            files.trust(&path, &format!("{name}-hash"));
            path
        })
        .collect()
}

/// Every extraction failure used to be stored as IO_ERROR ("A file operation
/// failed.") and extracted a second time, whatever the worker said. A failure
/// the worker calls permanent now fails the document on the first attempt,
/// under a code that says what is wrong with it.
#[test]
fn non_retryable_extraction_failure_is_terminal_with_its_code() {
    let cases = [
        ("PASSWORD_PROTECTED", ErrorCode::PasswordProtected),
        ("UNSUPPORTED_FORMAT", ErrorCode::UnsupportedContent),
        ("RESOURCE_LIMIT_EXCEEDED", ErrorCode::DocumentTooLarge),
        // The host's own deadline on the worker.
        ("RESOURCE_LIMIT", ErrorCode::DocumentTooLarge),
        ("PARSE_FAILED", ErrorCode::ExtractionFailed),
    ];
    let temp = tempdir().unwrap();
    let files = Arc::new(FakeFiles::default());
    let names = ["a.docx", "b.pdf", "c.xlsx", "d.tif", "e.pdf"];
    let paths = documents(temp.path(), &files, &names);
    let worker = Arc::new(FakeWorker::new(
        cases
            .iter()
            .map(|(code, _)| Err(WorkerFailure::reported(*code, "the worker's words", false)))
            .collect(),
    ));
    let model = Arc::new(FakeModel::new(vec![]));
    let pipeline = pipeline(
        temp.path(),
        Arc::clone(&worker),
        Arc::clone(&model),
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&paths).unwrap();
    pipeline.run_until_idle().unwrap();

    assert_eq!(
        worker.calls.load(Ordering::SeqCst),
        cases.len(),
        "each document is read once"
    );
    assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    let items = pipeline.list().unwrap();
    for (item, (code, expected)) in items.iter().zip(cases) {
        assert_eq!(item.status, QueueStatus::Failed, "{code}");
        assert_eq!(item.error_code, Some(expected), "{code}");
    }
    assert!(!pipeline.is_paused());
}

/// A crash, or a failure the worker says may pass, keeps its second attempt.
#[test]
fn crashed_extraction_is_retried_once() {
    let temp = tempdir().unwrap();
    let files = Arc::new(FakeFiles::default());
    let paths = documents(temp.path(), &files, &["crashes-once.pdf", "busy-disk.pdf"]);
    let worker = Arc::new(FakeWorker::new(vec![
        Err(WorkerFailure::crashed()),
        Ok(parsed(READABLE)),
        Err(WorkerFailure::reported(
            "PARSE_FAILED",
            "the file could not be read",
            true,
        )),
        Err(WorkerFailure::reported(
            "PARSE_FAILED",
            "the file could not be read",
            true,
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let pipeline = pipeline(
        temp.path(),
        Arc::clone(&worker),
        model,
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&paths).unwrap();
    pipeline.run_until_idle().unwrap();

    assert_eq!(worker.calls.load(Ordering::SeqCst), 4);
    assert_eq!(worker.restarts.load(Ordering::SeqCst), 1);
    let items = pipeline.list().unwrap();
    assert_eq!(items[0].status, QueueStatus::Ready);
    assert_eq!(items[0].processing_failures, 1);
    assert_eq!(items[1].status, QueueStatus::Failed);
    assert_eq!(items[1].error_code, Some(ErrorCode::IoError));
    assert_eq!(items[1].processing_failures, 2);
    assert!(!pipeline.is_paused());
}

/// Without its text-recognition files the worker fails every document that
/// needs them, so the first such failure stops the queue and says why.
#[test]
fn native_assets_missing_pauses() {
    let temp = tempdir().unwrap();
    let files = Arc::new(FakeFiles::default());
    let paths = documents(temp.path(), &files, &["scan.pdf", "next-scan.pdf"]);
    let worker = Arc::new(FakeWorker::new(vec![Err(WorkerFailure::reported(
        "NATIVE_ASSETS_MISSING",
        "tessdata is missing",
        false,
    ))]));
    let pipeline = pipeline(
        temp.path(),
        Arc::clone(&worker),
        Arc::new(FakeModel::new(vec![])),
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&paths).unwrap();
    pipeline.run_until_idle().unwrap();

    assert_eq!(worker.calls.load(Ordering::SeqCst), 1);
    let items = pipeline.list().unwrap();
    assert_eq!(items[0].status, QueueStatus::Failed);
    assert_eq!(items[0].error_code, Some(ErrorCode::OcrUnavailable));
    assert_eq!(items[1].status, QueueStatus::Queued);
    assert!(pipeline.is_paused());
    assert_eq!(pipeline.pause_reason().as_deref(), Some("OCR_UNAVAILABLE"));

    pipeline.resume();
    assert_eq!(pipeline.pause_reason(), None);
}

/// A refusal, a document too long for the model, and an internal failure on
/// one document are that document's answer. Each used to be re-queued - and,
/// on a hosted model, re-sent and re-billed - before failing anyway.
#[test]
fn refused_and_too_large_are_terminal_and_do_not_pause() {
    let cases = [
        ("HOSTED_MODEL_REFUSED", ErrorCode::ModelDeclined),
        ("MODEL_INPUT_TOO_LARGE", ErrorCode::DocumentTooLarge),
        ("ANALYSIS_FAILED", ErrorCode::AnalysisFailed),
    ];
    let temp = tempdir().unwrap();
    let files = Arc::new(FakeFiles::default());
    let paths = documents(
        temp.path(),
        &files,
        &["refused.pdf", "too-long.pdf", "internal.pdf", "fine.pdf"],
    );
    let worker = Arc::new(FakeWorker::new(
        (0..paths.len()).map(|_| Ok(parsed(READABLE))).collect(),
    ));
    let mut replies = cases
        .iter()
        .map(|(code, _)| Err(ModelFailure::fatal(*code)))
        .collect::<Vec<_>>();
    replies.push(Ok(proposal(0.94, false)));
    let model = Arc::new(FakeModel::new(replies));
    let pipeline = pipeline(
        temp.path(),
        Arc::clone(&worker),
        Arc::clone(&model),
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&paths).unwrap();
    pipeline.run_until_idle().unwrap();

    assert_eq!(model.calls.load(Ordering::SeqCst), paths.len());
    assert_eq!(worker.calls.load(Ordering::SeqCst), paths.len());
    let items = pipeline.list().unwrap();
    for (item, (code, expected)) in items.iter().zip(cases) {
        assert_eq!(item.status, QueueStatus::Failed, "{code}");
        assert_eq!(item.error_code, Some(expected), "{code}");
    }
    assert_eq!(items[3].status, QueueStatus::Ready);
    assert!(!pipeline.is_paused());
    assert_eq!(pipeline.pause_reason(), None);
}

/// One reply that cannot be used fails that document. A model that keeps
/// answering that way is broken for every document, so the third in a row
/// stops the queue - but only in a row: a document read successfully in
/// between starts the count again.
#[test]
fn three_consecutive_invalid_replies_pause_but_a_success_resets() {
    let invalid = || Err(ModelFailure::fatal("MODEL_RESPONSE_INVALID"));
    let truncated = || Err(ModelFailure::fatal("MODEL_REPLY_TRUNCATED"));
    let run = |names: &[&str], replies: Vec<Result<ModelProposal, ModelFailure>>| {
        let temp = tempdir().unwrap();
        let files = Arc::new(FakeFiles::default());
        let paths = documents(temp.path(), &files, names);
        let worker = Arc::new(FakeWorker::new(
            (0..paths.len()).map(|_| Ok(parsed(READABLE))).collect(),
        ));
        let model = Arc::new(FakeModel::new(replies));
        let pipeline = pipeline(
            temp.path(),
            worker,
            Arc::clone(&model),
            files,
            AppSettings::default(),
        );
        pipeline.enqueue_files(&paths).unwrap();
        pipeline.run_until_idle().unwrap();
        let items = pipeline.list().unwrap();
        let calls = model.calls.load(Ordering::SeqCst);
        (
            pipeline.is_paused(),
            pipeline.pause_reason(),
            items,
            calls,
            temp,
        )
    };

    let (paused, _, items, calls, _temp) = run(
        &["a.pdf", "b.pdf", "c.pdf", "d.pdf", "e.pdf"],
        vec![
            invalid(),
            truncated(),
            Ok(proposal(0.94, false)),
            invalid(),
            truncated(),
        ],
    );
    assert!(
        !paused,
        "two, a success, and two more is never three in a row"
    );
    assert_eq!(calls, 5, "each document is asked once");
    let statuses = items.iter().map(|item| item.status).collect::<Vec<_>>();
    assert_eq!(
        statuses,
        [
            QueueStatus::Failed,
            QueueStatus::Failed,
            QueueStatus::Ready,
            QueueStatus::Failed,
            QueueStatus::Failed,
        ]
    );
    assert!(
        items
            .iter()
            .filter(|item| item.status == QueueStatus::Failed)
            .all(|item| item.error_code == Some(ErrorCode::ModelOutputInvalid))
    );

    let (paused, reason, items, calls, _temp) = run(
        &["a.pdf", "b.pdf", "c.pdf", "waits.pdf"],
        vec![invalid(), truncated(), invalid()],
    );
    assert!(paused);
    assert_eq!(reason.as_deref(), Some("MODEL_OUTPUT_INVALID"));
    assert_eq!(calls, 3);
    assert!(items[..3].iter().all(|item| {
        item.status == QueueStatus::Failed && item.error_code == Some(ErrorCode::ModelOutputInvalid)
    }));
    assert_eq!(items[3].status, QueueStatus::Queued);
}

/// An account out of credit fails every document behind it. The queue stops
/// at the first one and names the reason, and that document keeps its second
/// attempt for when the queue is resumed.
#[test]
fn billing_pauses() {
    let temp = tempdir().unwrap();
    let files = Arc::new(FakeFiles::default());
    let paths = documents(temp.path(), &files, &["first.pdf", "second.pdf"]);
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(READABLE))]));
    let model = Arc::new(FakeModel::new(vec![Err(ModelFailure::fatal(
        "HOSTED_MODEL_BILLING",
    ))]));
    let pipeline = pipeline(
        temp.path(),
        worker,
        Arc::clone(&model),
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&paths).unwrap();
    pipeline.run_until_idle().unwrap();

    assert!(pipeline.is_paused());
    assert_eq!(
        pipeline.pause_reason().as_deref(),
        Some("HOSTED_MODEL_BILLING")
    );
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
    let items = pipeline.list().unwrap();
    assert_eq!(items[0].status, QueueStatus::Queued);
    assert_eq!(items[0].processing_failures, 1);
    assert_eq!(items[0].error_code, Some(ErrorCode::HostedModelUnavailable));
    assert_eq!(items[1].status, QueueStatus::Queued);
    assert_eq!(items[1].processing_failures, 0);

    pipeline.resume();
    assert!(!pipeline.is_paused());
    assert_eq!(pipeline.pause_reason(), None);
}

/// A request that was called off says nothing about the next document. It is
/// counted like any failure and never stops the queue.
#[test]
fn canceled_never_pauses() {
    let temp = tempdir().unwrap();
    let files = Arc::new(FakeFiles::default());
    let paths = documents(temp.path(), &files, &["called-off.pdf", "next.pdf"]);
    let worker = Arc::new(FakeWorker::new(
        (0..3).map(|_| Ok(parsed(READABLE))).collect(),
    ));
    let model = Arc::new(FakeModel::new(vec![
        Err(ModelFailure::fatal("MODEL_CANCELED")),
        Err(ModelFailure::fatal("MODEL_CANCELED")),
        Ok(proposal(0.94, false)),
    ]));
    let pipeline = pipeline(
        temp.path(),
        worker,
        Arc::clone(&model),
        files,
        AppSettings::default(),
    );

    pipeline.enqueue_files(&paths).unwrap();
    pipeline.run_until_idle().unwrap();

    assert!(!pipeline.is_paused());
    assert_eq!(model.calls.load(Ordering::SeqCst), 3);
    let items = pipeline.list().unwrap();
    assert_eq!(items[0].status, QueueStatus::Failed);
    assert_eq!(items[0].error_code, Some(ErrorCode::ModelFailed));
    assert_eq!(items[1].status, QueueStatus::Ready);
}

/// Switching from the hosted model to the local one leaves a moment with no
/// model loaded. A document that meets that moment was never analysed: it
/// goes back to wait with nothing counted against it, however often it
/// happens, and the drain ends instead of claiming it again at once.
#[test]
fn model_not_ready_requeues_without_counting_a_failure() {
    let temp = tempdir().unwrap();
    let files = Arc::new(FakeFiles::default());
    let paths = documents(temp.path(), &files, &["first.pdf", "second.pdf"]);
    let worker = Arc::new(FakeWorker::new(
        (0..5).map(|_| Ok(parsed(READABLE))).collect(),
    ));
    let not_ready = || Err(ModelFailure::fatal("MODEL_NOT_READY"));
    let model = Arc::new(FakeModel::new(vec![
        not_ready(),
        not_ready(),
        not_ready(),
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let pipeline = pipeline(
        temp.path(),
        worker,
        Arc::clone(&model),
        files,
        AppSettings::default(),
    );
    pipeline.enqueue_files(&paths).unwrap();

    for attempt in 1..=3 {
        pipeline.run_until_idle().unwrap();
        assert_eq!(model.calls.load(Ordering::SeqCst), attempt);
        assert!(!pipeline.is_paused());
        let items = pipeline.list().unwrap();
        for item in &items {
            assert_eq!(item.status, QueueStatus::Queued);
            assert_eq!(item.processing_failures, 0);
            assert_eq!(item.error_code, None);
        }
    }

    pipeline.run_until_idle().unwrap();
    let items = pipeline.list().unwrap();
    assert!(
        items
            .iter()
            .all(|item| item.status == QueueStatus::Ready && item.processing_failures == 0)
    );
}

/// A crash the worker cannot be restarted from used to reclaim whatever the
/// queue offered next, which is not always the document that crashed. Another
/// waiting document was claimed instead and left extracting, under a lease
/// nothing would renew and recovery would not take back while the app kept
/// running: the document was simply gone.
#[test]
fn worker_crash_with_failed_restart_does_not_strand_another_queued_item() {
    let temp = tempdir().unwrap();
    let waiting = source(temp.path(), "waiting.pdf");
    let crashing = source(temp.path(), "crashing.pdf");
    let worker = Arc::new(CrashingWorker::default());
    let files = Arc::new(FakeFiles::default());
    files.trust(&waiting, "waiting-hash");
    files.trust(&crashing, "crashing-hash");
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings.save(&AppSettings::default()).unwrap();
    let pipeline = Arc::new(
        Pipeline::open(
            temp.path().join("queue.sqlite3"),
            Arc::clone(&worker) as Arc<dyn WorkerBoundary>,
            Arc::new(FakeModel::new(vec![])),
            files,
            Arc::new(RecordingEvents::default()),
            settings,
        )
        .unwrap(),
    );
    let enqueued = pipeline
        .enqueue_files(&[waiting.clone(), crashing.clone()])
        .unwrap();
    let waiting_id = enqueued[0].id;
    let crashing_id = enqueued[1].id;
    // The first document is out of the queue while the second one runs, and a
    // person puts it back - Retry - while that one is crashing.
    pipeline.cancel(waiting_id).unwrap();

    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_until_idle());
    while !worker.started.load(Ordering::SeqCst) {
        thread::yield_now();
    }
    pipeline.retry(waiting_id).unwrap();
    worker.release();
    join.join().unwrap().unwrap();

    let items = pipeline.list().unwrap();
    assert!(
        items
            .iter()
            .all(|item| !matches!(item.status, QueueStatus::Extracting)),
        "no document is left claimed by a worker that is gone: {items:?}"
    );
    let status = |id: i64| {
        items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.status)
            .unwrap()
    };
    assert_eq!(
        status(crashing_id),
        QueueStatus::Failed,
        "the document that crashed the worker is the one that fails"
    );
    assert_eq!(status(waiting_id), QueueStatus::Failed);
}

#[test]
fn pause_starts_no_item_and_cancel_interrupts_the_active_worker_request() {
    let temp = tempdir().unwrap();
    let paused_path = source(temp.path(), "paused.pdf");
    let cancel_path = source(temp.path(), "cancel.pdf");
    let worker = Arc::new(FakeWorker::blocking(Ok(parsed("unused"))));
    let model = Arc::new(FakeModel::new(vec![]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&paused_path, "paused-hash");
    files.trust(&cancel_path, "cancel-hash");
    let pipeline = Arc::new(pipeline(
        temp.path(),
        Arc::clone(&worker),
        model,
        files,
        AppSettings::default(),
    ));

    pipeline.enqueue_files(&[paused_path, cancel_path]).unwrap();
    pipeline.pause();
    pipeline.run_until_idle().unwrap();
    assert!(
        pipeline
            .list()
            .unwrap()
            .iter()
            .all(|item| item.status == QueueStatus::Queued)
    );

    pipeline.resume();
    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_next());
    while worker.active.load(Ordering::SeqCst) == 0 {
        thread::yield_now();
    }
    let active_id = pipeline.list().unwrap()[0].id;
    pipeline.cancel(active_id).unwrap();
    join.join().unwrap().unwrap();

    assert_eq!(worker.cancellations.load(Ordering::SeqCst), 1);
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Canceled);
}

#[test]
fn pause_during_extraction_returns_item_to_queue_before_analysis() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "pause-extracting.pdf");
    let worker = Arc::new(FakeWorker::blocking(Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "pause-hash");
    let pipeline = Arc::new(pipeline(
        temp.path(),
        Arc::clone(&worker),
        Arc::clone(&model),
        files,
        AppSettings::default(),
    ));
    pipeline.enqueue_files(&[path]).unwrap();

    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_until_idle());
    while worker.active.load(Ordering::SeqCst) == 0 {
        thread::yield_now();
    }
    pipeline.pause();
    *worker.gate.0.lock().unwrap() = false;
    worker.gate.1.notify_all();
    join.join().unwrap().unwrap();

    assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Queued);
}

#[test]
fn pause_during_analysis_prevents_automatic_apply_until_resume() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "pause-analysis.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(GatedSuccessModel::new());
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "pause-hash");
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Arc::new(
        Pipeline::open(
            temp.path().join("queue.sqlite3"),
            worker,
            model.clone(),
            files.clone(),
            Arc::new(RecordingEvents::default()),
            settings,
        )
        .unwrap(),
    );
    pipeline.enqueue_files(&[path]).unwrap();

    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_until_idle());
    while model.calls.load(Ordering::SeqCst) == 0 {
        thread::yield_now();
    }
    pipeline.pause();
    model.release();
    join.join().unwrap().unwrap();

    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Ready);
    assert!(files.applies.lock().unwrap().is_empty());

    pipeline.resume();
    pipeline.run_until_idle().unwrap();
    assert_eq!(files.applies.lock().unwrap().len(), 1);
}

#[test]
fn automatic_apply_targets_only_ready_items_and_rechecks_source_fingerprint() {
    let temp = tempdir().unwrap();
    let ready = source(temp.path(), "ready.pdf");
    let review = source(temp.path(), "review.pdf");
    let changed = source(temp.path(), "changed.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.70, true)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&ready, "ready-hash");
    files.trust(&review, "review-hash");
    files.trust(&changed, "changed-hash");
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        Arc::clone(&files),
        AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        },
    );
    pipeline
        .enqueue_files(&[ready.clone(), review, changed.clone()])
        .unwrap();
    files.trust(&changed, "mutated-after-ingest");

    pipeline.run_until_idle().unwrap();

    let items = pipeline.list().unwrap();
    let applies = files.applies.lock().unwrap();
    assert_eq!(applies.as_slice(), &[items[0].id]);
    assert_eq!(items[1].status, QueueStatus::NeedsReview);
    assert_eq!(items[2].status, QueueStatus::NeedsReview);
    assert!(
        items[2]
            .proposal
            .as_ref()
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == "FILE_CHANGED")
    );
}

/// Every rename carries a date. A reviewer who strips it - or a document
/// whose date the model could not support - is told so at approval, and the
/// name is applied only once a date leads it.
#[test]
fn an_approved_name_without_a_leading_date_is_refused_until_one_is_added() {
    let temp = tempdir().unwrap();
    let review = source(temp.path(), "review.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.70, true))]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&review, "review-hash");
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        Arc::clone(&files),
        AppSettings::default(),
    );
    pipeline.enqueue_files(&[review]).unwrap();
    pipeline.run_until_idle().unwrap();
    let item = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(item.status, QueueStatus::NeedsReview);

    for undated in [
        "Employment Agreement with John Smith.pdf",
        "Employment Agreement 2024-04-12 with John Smith.pdf",
        "2024-04-1 Employment Agreement.pdf",
    ] {
        let refused = pipeline
            .approve(item.id, undated, "A sentence.")
            .unwrap_err();
        assert_eq!(refused.code, "DATE_REQUIRED", "{undated}");
    }
    let unchanged = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(unchanged.status, QueueStatus::NeedsReview);
    assert!(files.applies.lock().unwrap().is_empty());

    pipeline
        .approve(
            item.id,
            "2024-04-12 Employment Agreement with John Smith.pdf",
            "A sentence.",
        )
        .unwrap();
    assert_eq!(files.applies.lock().unwrap().as_slice(), &[item.id]);
}

#[test]
fn fingerprint_failure_is_durable_and_does_not_stop_the_queue() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "missing-after-ingest.pdf");
    let second = source(temp.path(), "still-processes.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        Arc::clone(&files),
        AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        },
    );
    pipeline.enqueue_files(&[first.clone(), second]).unwrap();
    files.forget(&first);

    pipeline.run_until_idle().unwrap();

    let items = pipeline.list().unwrap();
    assert_eq!(items[0].status, QueueStatus::NeedsReview);
    assert!(
        items[0]
            .proposal
            .as_ref()
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == "FILE_MISSING")
    );
    assert_eq!(files.applies.lock().unwrap().as_slice(), &[items[1].id]);
}

#[test]
fn apply_failure_is_durable_and_does_not_stop_the_queue() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "apply-fails.pdf");
    let second = source(temp.path(), "apply-continues.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    files.fail_next_apply();
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        Arc::clone(&files),
        AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        },
    );
    pipeline.enqueue_files(&[first, second]).unwrap();

    pipeline.run_until_idle().unwrap();

    let items = pipeline.list().unwrap();
    assert_eq!(items[0].status, QueueStatus::NeedsReview);
    assert!(
        items[0]
            .proposal
            .as_ref()
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == "MOVE_FAILED")
    );
    assert_eq!(files.applies.lock().unwrap().as_slice(), &[items[1].id]);
}

#[test]
fn settings_failure_is_recorded_for_each_item_while_the_queue_drains() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "settings-first.pdf");
    let second = source(temp.path(), "settings-second.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    let pipeline = pipeline(temp.path(), worker, model, files, AppSettings::default());
    pipeline.enqueue_files(&[first, second]).unwrap();
    std::fs::write(temp.path().join("settings.json"), b"not-json").unwrap();

    pipeline.run_until_idle().unwrap();

    let items = pipeline.list().unwrap();
    assert!(
        items
            .iter()
            .all(|item| item.status == QueueStatus::NeedsReview)
    );
    assert!(items.iter().all(|item| {
        item.proposal
            .as_ref()
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == "SETTINGS_INVALID")
    }));
}

#[test]
fn field_affecting_parser_warning_prevents_ready() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "warning.pdf");
    let mut document =
        parsed("Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.");
    document.parser_warnings.push(ParserWarning {
        code: "TEXT_TRUNCATED".into(),
        field_affecting: true,
    });
    let worker = Arc::new(FakeWorker::new(vec![Ok(document)]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "warning-hash");
    let pipeline = pipeline(temp.path(), worker, model, files, AppSettings::default());
    pipeline.enqueue_files(&[path]).unwrap();

    pipeline.run_until_idle().unwrap();

    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::NeedsReview);
}

#[test]
fn initial_proposal_and_ready_transition_commit_atomically() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "atomic.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "atomic-hash");
    let pipeline = pipeline(temp.path(), worker, model, files, AppSettings::default());
    pipeline.enqueue_files(&[path]).unwrap();
    let database = temp.path().join("queue.sqlite3");
    Connection::open(&database)
        .unwrap()
        .execute_batch(
            "CREATE TRIGGER reject_ready BEFORE UPDATE OF status ON queue_items
         WHEN NEW.status = 'ready' BEGIN SELECT RAISE(ABORT, 'reject ready'); END;",
        )
        .unwrap();

    assert!(pipeline.run_until_idle().is_err());

    let connection = Connection::open(database).unwrap();
    let proposals: i64 = connection
        .query_row("SELECT COUNT(*) FROM proposals", [], |row| row.get(0))
        .unwrap();
    assert_eq!(proposals, 0);
}

#[test]
fn real_sqlite_and_core_file_actions_apply_then_undo_the_operation_receipt() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "real-file.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let filing = Arc::new(RecordingFiling::default());
    let pipeline = Pipeline::with_local_files(
        temp.path().join("real-queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_filing_sink(filing.clone());
    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();
    assert!(
        pipeline.filed_documents().unwrap().is_empty(),
        "nothing is filed before the queue runs"
    );

    pipeline.run_until_idle().unwrap();

    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);
    let receipt = completed.receipt.clone().unwrap();
    assert!(!receipt.source.exists());
    assert!(receipt.destination.exists());

    // The filing sink hears about the rename once it is complete, with the
    // destination the applier actually chose and the sentence that was applied.
    let filed = filing.filed.lock().unwrap().clone();
    assert_eq!(filed.len(), 1, "{filed:?}");
    assert_eq!(filed[0].item_id, completed.id);
    assert_eq!(filed[0].source_path, path);
    assert_eq!(filed[0].source_hash, completed.source_hash);
    assert_eq!(filed[0].source_hash.len(), 64, "the content hash, as hex");
    assert_eq!(filed[0].destination, receipt.destination);
    assert_eq!(
        filed[0].description,
        completed.proposal.as_ref().unwrap().description
    );
    assert_eq!(
        filed[0].proposal.document_date.as_deref(),
        Some("2024-04-12")
    );
    assert!(filed[0].filed_at > 0);
    // The same report is available on demand, for a records keeper that is
    // switched on after the fact.
    let replay = pipeline.filed_documents().unwrap();
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].destination, receipt.destination);
    assert_eq!(replay[0].description, filed[0].description);

    // The original path no longer exists, and still finds its item; the
    // renamed path was never enqueued and finds nothing.
    let by_path = pipeline.find_by_source_path(&path).unwrap().unwrap();
    assert_eq!(by_path.id, completed.id);
    assert_eq!(by_path.status, QueueStatus::Completed);
    assert!(by_path.receipt.is_some());
    assert!(
        pipeline
            .find_by_source_path(&receipt.destination)
            .unwrap()
            .is_none()
    );

    pipeline.undo(completed.id).unwrap();

    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::NeedsReview);
    assert!(path.exists());
    assert!(!receipt.destination.exists());
    assert_eq!(
        filing.unfiled.lock().unwrap().clone(),
        vec![UnfiledDocument {
            item_id: completed.id,
            source_path: path.clone(),
            source_hash: completed.source_hash.clone(),
            destination: receipt.destination.clone(),
        }],
        "an undo retracts the report for the destination it vacated"
    );
    assert!(
        pipeline.filed_documents().unwrap().is_empty(),
        "an undone rename is no longer a filed document"
    );
    assert_eq!(
        pipeline.find_by_source_path(&path).unwrap().unwrap().status,
        QueueStatus::NeedsReview
    );
}

/// An undo is a decision, and the scheduler's next pass must respect it: with
/// automatic renaming on, the queue used to file the document again within
/// the minute, which undoes the person's undo.
#[test]
fn an_undone_rename_is_not_reapplied_automatically() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "undone.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("undone-queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();

    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();
    pipeline.run_until_idle().unwrap();
    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);
    let destination = completed.receipt.clone().unwrap().destination;

    pipeline.undo(completed.id).unwrap();
    assert!(path.exists());

    // The scheduler's next pass - the one that comes round every sixty-five
    // seconds whether or not anything else happened.
    pipeline.run_until_idle().unwrap();

    let after = pipeline.list().unwrap().pop().unwrap();
    let record = after.proposal.as_ref().unwrap();
    assert_eq!(
        after.status,
        QueueStatus::NeedsReview,
        "an undone rename waits for a person"
    );
    assert!(
        record.reasons.iter().any(|reason| reason == UNDONE),
        "{:?}",
        record.reasons
    );
    assert!(path.exists(), "the document stays where the undo put it");
    assert!(!destination.exists());
}

/// A person who clicks Approve while the queue is working on another document
/// must end up with the document filed under the name they typed, not with an
/// error they never asked about and a document pushed into review.
#[test]
fn approving_while_another_document_is_analyzing_files_it_when_the_queue_is_free() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    let reviewed = source(&inbox, "reviewed.pdf");
    let other = source(&inbox, "other.pdf");
    let database = temp.path().join("busy-queue.sqlite3");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings.save(&AppSettings::default()).unwrap();
    let pipeline = Pipeline::with_local_files(
        database.clone(),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();

    pipeline
        .enqueue_files(std::slice::from_ref(&reviewed))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let waiting = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(waiting.status, QueueStatus::Ready);

    // A second document, claimed the way a running queue claims one: the
    // store refuses any apply while it is being processed.
    pipeline
        .enqueue_files(std::slice::from_ref(&other))
        .unwrap();
    let busy = QueueStore::open(&database).unwrap();
    let claimed = busy.claim_next().unwrap().unwrap();
    assert_eq!(claimed.status, QueueStatus::Extracting);

    let approved = "2024-04-12 Employment Agreement between John Smith and Acme Corp.pdf";
    pipeline
        .approve(waiting.id, approved, "A sentence about the agreement.")
        .unwrap();
    let deferred = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == waiting.id)
        .unwrap();
    assert_eq!(
        deferred.status,
        QueueStatus::Ready,
        "a busy queue is not a reason to review the document again"
    );
    assert_eq!(deferred.proposal.as_ref().unwrap().filename, approved);

    // The other document leaves the queue, and the approval that was waiting
    // is applied under the name the reviewer typed.
    busy.transition(
        claimed.id,
        QueueStatus::Extracting,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();
    drop(busy);
    pipeline.run_until_idle().unwrap();

    let filed = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == waiting.id)
        .unwrap();
    assert_eq!(filed.status, QueueStatus::Completed);
    assert_eq!(
        filed
            .receipt
            .unwrap()
            .destination
            .file_name()
            .unwrap()
            .to_string_lossy(),
        approved
    );
    assert!(inbox.join(approved).exists());
}

/// A year of contracts should not become one folder of a thousand files. The
/// layout puts each document under its year and type, creates the folders as
/// documents arrive, and takes them away again when an undo empties them.
#[test]
fn a_layout_files_into_year_and_type_subfolders_and_undo_removes_the_empty_ones() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::create_dir_all(&filed).unwrap();
    let path = source(&inbox, "scan.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            destination: filed.to_string_lossy().into_owned(),
            destination_layout: DestinationLayout::YearType,
            ..AppSettings::default()
        })
        .unwrap();
    let filing = Arc::new(RecordingFiling::default());
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_filing_sink(filing.clone());
    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();

    pipeline.run_until_idle().unwrap();

    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);
    let receipt = completed.receipt.clone().unwrap();
    let expected_folder = filed.join("2024").join("Employment Agreement");
    assert_eq!(
        receipt.destination.parent(),
        Some(expected_folder.as_path())
    );
    assert_eq!(
        receipt
            .destination
            .file_name()
            .and_then(|name| name.to_str()),
        Some("2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf")
    );
    assert!(receipt.destination.exists());
    assert_eq!(
        filing.filed.lock().unwrap()[0].destination,
        receipt.destination,
        "the records keeper hears the path with its subfolders"
    );

    pipeline.undo(completed.id).unwrap();

    assert!(path.exists());
    assert!(!receipt.destination.exists());
    assert!(
        !expected_folder.exists() && !filed.join("2024").exists(),
        "an undo takes the folders it emptied away again"
    );
    assert!(filed.exists(), "the destination itself is never removed");
}

/// A reviewer's date lives in the filename and nowhere else. The layout
/// folder and the filing report must follow it, not the date validation
/// withheld, or a document dated by hand lands in "Undated" with a record
/// that says it has no date.
#[test]
fn a_date_given_in_review_is_the_date_the_document_is_filed_under() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::create_dir_all(&filed).unwrap();
    let path = source(&inbox, "scan.pdf");
    // The model's date is nowhere in the text, so validation withholds it.
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement between John Smith and Acme Corporation covering duties, salary, and term.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            destination_layout: DestinationLayout::Year,
            ..AppSettings::default()
        })
        .unwrap();
    let filing = Arc::new(RecordingFiling::default());
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_filing_sink(filing.clone());
    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();
    pipeline.run_until_idle().unwrap();

    let item = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(item.status, QueueStatus::NeedsReview);
    let record = item.proposal.as_ref().unwrap();
    assert_eq!(record.analysis.proposal.document_date, None);
    assert!(
        record
            .reasons
            .iter()
            .any(|reason| reason == "DATE_UNSUPPORTED")
    );
    assert_eq!(
        record
            .analysis
            .model_proposal
            .as_ref()
            .and_then(|reply| reply.document_date.as_deref()),
        Some("2024-04-12"),
        "the model's reading is kept for the reviewer to accept"
    );

    pipeline
        .approve(
            item.id,
            "2025-01-15 Employment Agreement with John Smith.pdf",
            "Employment agreement between John Smith and Acme Corporation.",
        )
        .unwrap();

    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);
    let receipt = completed.receipt.unwrap();
    assert_eq!(
        receipt.destination,
        filed
            .join("2025")
            .join("2025-01-15 Employment Agreement with John Smith.pdf"),
        "the year folder follows the date the reviewer gave"
    );
    let heard = filing.filed.lock().unwrap();
    assert_eq!(heard.len(), 1);
    assert_eq!(
        heard[0].proposal.document_date.as_deref(),
        Some("2025-01-15")
    );
    drop(heard);
    let replay = pipeline.filed_documents().unwrap();
    assert_eq!(
        replay[0].proposal.document_date.as_deref(),
        Some("2025-01-15")
    );
}

/// The name that will be applied must not collide in the folder the
/// document is going to; the engine only knows the source folder.
#[test]
fn proposed_names_avoid_collisions_in_the_destination_not_the_source() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::create_dir_all(&filed).unwrap();
    let path = source(&inbox, "scan.pdf");
    std::fs::write(
        filed.join("2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf"),
        b"already filed",
    )
    .unwrap();
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();

    pipeline.run_until_idle().unwrap();

    let ready = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(ready.status, QueueStatus::Ready);
    assert_eq!(
        ready.proposal.unwrap().filename,
        "2024-04-12 Employment Agreement between John Smith and Acme Corporation (2).pdf",
        "the reviewer is shown the name that will actually be used"
    );
}

#[test]
fn model_timeout_does_not_start_another_proposal_until_cancel_is_terminal() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "timeout-first.pdf");
    let second = source(temp.path(), "timeout-second.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
    ]));
    let model = Arc::new(BlockingModel::new(true));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    let pipeline = Arc::new(
        Pipeline::open(
            temp.path().join("queue.sqlite3"),
            worker,
            model.clone(),
            files,
            Arc::new(RecordingEvents::default()),
            SettingsStore::new(temp.path().join("settings.json")),
        )
        .unwrap()
        .with_model_timeout(Duration::from_millis(20)),
    );
    pipeline.enqueue_files(&[first, second]).unwrap();
    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_until_idle());

    model.wait_for_cancel();
    thread::sleep(Duration::from_millis(30));
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
    model.release_cancel();
    join.join().unwrap().unwrap();

    assert!(model.calls.load(Ordering::SeqCst) >= 3);
}

#[test]
fn failed_model_timeout_cancel_pauses_drain_until_request_is_terminal() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "cancel-fails.pdf");
    let second = source(temp.path(), "must-not-start.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(BlockingModel::new(false));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    let pipeline = Arc::new(
        Pipeline::open(
            temp.path().join("queue.sqlite3"),
            worker,
            model.clone(),
            files,
            Arc::new(RecordingEvents::default()),
            SettingsStore::new(temp.path().join("settings.json")),
        )
        .unwrap()
        .with_model_timeout(Duration::from_millis(20)),
    );
    pipeline.enqueue_files(&[first, second]).unwrap();
    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_until_idle());

    model.wait_for_cancel();
    model.release_cancel();
    thread::sleep(Duration::from_millis(30));
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
    model.release_request();
    join.join().unwrap().unwrap();

    assert!(pipeline.is_paused());
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
}

/// A cancel the model refuses is still an answer; the request itself may
/// never come back. The queue then waited on that request with no deadline
/// at all, so one model that ignored both its deadline and its cancel stopped
/// the queue for good.
#[test]
fn a_cancel_the_model_refuses_does_not_hold_the_queue_forever() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "ignores-cancel.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(BlockingModel::new(false));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "ignores-hash");
    let pipeline = Arc::new(
        Pipeline::open(
            temp.path().join("queue.sqlite3"),
            worker,
            model.clone(),
            files,
            Arc::new(RecordingEvents::default()),
            SettingsStore::new(temp.path().join("settings.json")),
        )
        .unwrap()
        .with_model_timeout(Duration::from_millis(20)),
    );
    pipeline.enqueue_files(&[path]).unwrap();
    let running = Arc::clone(&pipeline);
    let (done, finished) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let _ = done.send(running.run_until_idle());
    });

    model.wait_for_cancel();
    // The cancel comes back refused, and the request behind it never returns.
    model.release_cancel();

    finished
        .recv_timeout(Duration::from_secs(10))
        .expect("the queue gave up on a request that would not come back")
        .unwrap();
    assert!(pipeline.is_paused());
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Queued);
}

#[test]
fn shutdown_cancels_blocking_worker_before_waiting_for_pipeline_exit() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "shutdown.pdf");
    let worker = Arc::new(FakeWorker::blocking(Ok(parsed("unused"))));
    let model = Arc::new(FakeModel::new(vec![]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "shutdown-hash");
    let pipeline = Arc::new(pipeline(
        temp.path(),
        Arc::clone(&worker),
        model,
        files,
        AppSettings::default(),
    ));
    pipeline.enqueue_files(&[path]).unwrap();
    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_next());
    while worker.active.load(Ordering::SeqCst) == 0 {
        thread::yield_now();
    }

    let started = Instant::now();
    pipeline.shutdown().unwrap();
    join.join().unwrap().unwrap_err();

    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(worker.cancellations.load(Ordering::SeqCst), 1);
}

#[test]
fn lost_processing_lease_cancels_worker_and_pauses_before_more_work() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "lease-loss.pdf");
    let worker = Arc::new(FakeWorker::blocking(Ok(parsed("unused"))));
    let model = Arc::new(FakeModel::new(vec![]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "lease-hash");
    let pipeline = Arc::new(
        pipeline(
            temp.path(),
            Arc::clone(&worker),
            model,
            files,
            AppSettings::default(),
        )
        .with_lease_renewal_interval(Duration::from_millis(10)),
    );
    pipeline.enqueue_files(&[path]).unwrap();
    let running = Arc::clone(&pipeline);
    let join = thread::spawn(move || running.run_next());
    while worker.active.load(Ordering::SeqCst) == 0 {
        thread::yield_now();
    }
    Connection::open(temp.path().join("queue.sqlite3"))
        .unwrap()
        .execute("DELETE FROM queue_sessions", [])
        .unwrap();

    let error = join.join().unwrap().unwrap_err();

    assert_eq!(error.code, "STATE_CONFLICT");
    assert!(pipeline.is_paused());
    assert_eq!(worker.cancellations.load(Ordering::SeqCst), 1);
}

#[test]
fn refiled_content_at_a_new_path_is_flagged_duplicate_and_retry_analyzes_it() {
    let temp = tempdir().unwrap();
    let original = source(temp.path(), "agreement.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
        Ok(parsed(
            "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
        )),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        Arc::clone(&model) as Arc<dyn AnalyzerBoundary>,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    pipeline.enqueue_files(&[original]).unwrap();
    pipeline.run_until_idle().unwrap();
    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);
    let filed_name = completed
        .receipt
        .unwrap()
        .destination
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    // The same bytes arrive again under a different name in another folder.
    let copies = temp.path().join("copies");
    std::fs::create_dir(&copies).unwrap();
    std::fs::write(copies.join("copy-of-agreement.pdf"), "agreement.pdf").unwrap();
    let copy = copies.join("copy-of-agreement.pdf").canonicalize().unwrap();
    let added = pipeline.enqueue_files(&[copy]).unwrap().pop().unwrap();
    assert_eq!(added.status, QueueStatus::NeedsReview);
    assert_eq!(added.error_code, Some(ErrorCode::Duplicate));
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
    let flagged = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == added.id)
        .unwrap();
    assert_eq!(flagged.duplicate_of.as_deref(), Some(filed_name.as_str()));

    // "Process anyway": retry clears the flag and the item analyzes normally.
    pipeline.retry(added.id).unwrap();
    let requeued = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == added.id)
        .unwrap();
    assert_eq!(requeued.status, QueueStatus::Queued);
    assert_eq!(requeued.error_code, None);
    pipeline.run_until_idle().unwrap();
    let settled = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == added.id)
        .unwrap();
    assert_eq!(model.calls.load(Ordering::SeqCst), 2);
    assert_eq!(settled.status, QueueStatus::Completed);
}

/// A teammate filed this content last week from the shared intake folder.
/// This machine's history has never seen it, so only the shared index knows;
/// the document goes to review before any analysis, naming the filing and
/// the machine, and "process anyway" still works.
#[test]
fn content_a_teammate_already_filed_is_flagged_before_analysis_and_names_their_machine() {
    let temp = tempdir().unwrap();
    let again = source(temp.path(), "agreement-again.pdf");
    let fresh = source(temp.path(), "fresh.pdf");
    let worker = Arc::new(FakeWorker::new(vec![]));
    let model = Arc::new(FakeModel::new(vec![]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&again, "teammate-hash");
    files.trust(&fresh, "fresh-hash");
    let teammates = Arc::new(TeammateFilings::default());
    teammates.known.lock().unwrap().insert(
        "teammate-hash".into(),
        KnownFiling {
            filename: "2024-04-12 Employment Agreement.pdf".into(),
            filed_by: Some("Front desk".into()),
        },
    );
    let pipeline = pipeline(
        temp.path(),
        worker,
        model.clone(),
        files,
        AppSettings::default(),
    )
    .with_duplicate_oracle(teammates.clone());

    let added = pipeline
        .enqueue_files(&[again.clone(), fresh.clone()])
        .unwrap();

    assert_eq!(added[0].status, QueueStatus::NeedsReview);
    assert_eq!(added[0].error_code, Some(ErrorCode::Duplicate));
    assert_eq!(
        added[1].status,
        QueueStatus::Queued,
        "unknown content is queued"
    );
    assert_eq!(added[1].error_code, None);
    assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        teammates.asked.lock().unwrap().clone(),
        vec![
            ("teammate-hash".to_string(), again.clone()),
            ("fresh-hash".to_string(), fresh.clone()),
        ],
        "asked once per document, with the hash and the path"
    );
    let flagged = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == added[0].id)
        .unwrap();
    assert_eq!(
        flagged.duplicate_of.as_deref(),
        Some("2024-04-12 Employment Agreement.pdf (filed from Front desk)")
    );

    // "Process anyway" clears the flag; the oracle is not asked again.
    pipeline.retry(added[0].id).unwrap();
    let requeued = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == added[0].id)
        .unwrap();
    assert_eq!(requeued.status, QueueStatus::Queued);
    assert_eq!(requeued.error_code, None);
    assert_eq!(requeued.duplicate_of, None);
    // Two enqueues, plus the one listing that named the referent while the
    // item was still flagged; a retried item is never asked about again.
    assert_eq!(teammates.asked.lock().unwrap().len(), 3);
}

#[test]
fn same_content_still_pending_does_not_flag_a_duplicate() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "pending-a.pdf");
    let second = source(temp.path(), "pending-b.pdf");
    let worker = Arc::new(FakeWorker::new(vec![]));
    let model = Arc::new(FakeModel::new(vec![]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "shared-hash");
    files.trust(&second, "shared-hash");
    let pipeline = pipeline(temp.path(), worker, model, files, AppSettings::default());

    let added = pipeline.enqueue_files(&[first, second]).unwrap();

    assert_eq!(added.len(), 2);
    assert!(added.iter().all(|item| item.status == QueueStatus::Queued));
    assert!(added.iter().all(|item| item.error_code.is_none()));
}

#[test]
fn undone_completion_is_not_flagged_as_a_duplicate_on_re_add() {
    let temp = tempdir().unwrap();
    let original = source(temp.path(), "undone.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    pipeline
        .enqueue_files(std::slice::from_ref(&original))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);

    pipeline.undo(completed.id).unwrap();
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::NeedsReview);

    // The apply was undone, so the content is not filed anywhere: a new copy
    // must analyze normally instead of being flagged.
    std::fs::write(temp.path().join("undone-copy.pdf"), "undone.pdf").unwrap();
    let copy = temp.path().join("undone-copy.pdf").canonicalize().unwrap();
    let added = pipeline.enqueue_files(&[copy]).unwrap().pop().unwrap();
    assert_eq!(added.status, QueueStatus::Queued);
    assert_eq!(added.error_code, None);
}

#[test]
fn flagged_duplicates_support_keep_original_remove_and_cleared_history() {
    let temp = tempdir().unwrap();
    let original = source(temp.path(), "keeper.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    pipeline.enqueue_files(&[original]).unwrap();
    pipeline.run_until_idle().unwrap();
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Completed);

    // Keep original completes the duplicate without touching the disk.
    std::fs::write(temp.path().join("copy-two.pdf"), "keeper.pdf").unwrap();
    let second = temp.path().join("copy-two.pdf").canonicalize().unwrap();
    let kept = pipeline
        .enqueue_files(std::slice::from_ref(&second))
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(kept.status, QueueStatus::NeedsReview);
    pipeline.keep_original(kept.id).unwrap();
    assert!(second.exists());
    let kept_row = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == kept.id)
        .unwrap();
    assert_eq!(kept_row.status, QueueStatus::Completed);
    assert_eq!(kept_row.error_code, None);

    // The next copy points at the most recent completion; a keep-original one
    // has no filed name, so its own filename is used.
    std::fs::write(temp.path().join("copy-three.pdf"), "keeper.pdf").unwrap();
    let third = temp.path().join("copy-three.pdf").canonicalize().unwrap();
    let flagged = pipeline.enqueue_files(&[third]).unwrap().pop().unwrap();
    assert_eq!(flagged.status, QueueStatus::NeedsReview);
    assert_eq!(flagged.error_code, Some(ErrorCode::Duplicate));
    let listed = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == flagged.id)
        .unwrap();
    assert_eq!(listed.duplicate_of.as_deref(), Some("copy-two.pdf"));

    // Clearing the history removes the completed rows; the stale flag still
    // lists without a referent and remains removable.
    assert_eq!(pipeline.clear_history().unwrap(), 2);
    let stale = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == flagged.id)
        .unwrap();
    assert_eq!(stale.status, QueueStatus::NeedsReview);
    assert_eq!(stale.duplicate_of, None);
    pipeline.remove(flagged.id, false).unwrap();
    assert!(pipeline.list().unwrap().is_empty());

    // With no completed rows left a fresh copy simply queues for analysis.
    std::fs::write(temp.path().join("copy-four.pdf"), "keeper.pdf").unwrap();
    let fourth = temp.path().join("copy-four.pdf").canonicalize().unwrap();
    let requeued = pipeline.enqueue_files(&[fourth]).unwrap().pop().unwrap();
    assert_eq!(requeued.status, QueueStatus::Queued);
    assert_eq!(requeued.error_code, None);
}

/// A proposal whose date the text states, so the item comes out Ready.
fn dated_proposal(iso: &str, written: &str) -> ModelProposal {
    ModelProposal {
        document_date: Some(iso.into()),
        evidence: Evidence {
            date: Some(format!("signed {written}")),
            ..proposal(0.94, false).evidence
        },
        ..proposal(0.94, false)
    }
}

fn dated_text(written: &str) -> DocumentSource {
    parsed(&format!(
        "Employment Agreement signed {written} by John Smith and Acme Corporation."
    ))
}

fn learning_pipeline(temp: &Path, dates: &[(&str, &str)]) -> (Pipeline, Arc<RecordingFiling>) {
    let filed = temp.join("filed");
    std::fs::create_dir_all(&filed).unwrap();
    let worker = Arc::new(FakeWorker::new(
        dates
            .iter()
            .map(|(_, written)| Ok(dated_text(written)))
            .collect(),
    ));
    let model = Arc::new(FakeModel::new(
        dates
            .iter()
            .map(|(iso, written)| Ok(dated_proposal(iso, written)))
            .collect(),
    ));
    let settings = SettingsStore::new(temp.join("settings.json"));
    settings
        .save(&AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            ..AppSettings::default()
        })
        .unwrap();
    let filing = Arc::new(RecordingFiling::default());
    let pipeline = Pipeline::with_local_files(
        temp.join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_filing_sink(filing.clone());
    (pipeline, filing)
}

fn record_of(pipeline: &Pipeline, id: i64) -> intern_queue::ProposalRecord {
    pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap()
        .proposal
        .unwrap()
}

/// Four documents naming the same counterparty. The reviewer shortens it
/// once - a decision about one document - and the next proposal still uses
/// the document's words. Shortening it a second time makes it a spelling:
/// the document still waiting is renamed in the queue, the one processed
/// afterwards is proposed with it, and what is filed under it reports the
/// reviewer's word while the analysis keeps the document's.
#[test]
fn a_respelling_made_twice_in_review_becomes_interns_own_spelling() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    let (pipeline, filing) = learning_pipeline(
        temp.path(),
        &[
            ("2024-04-12", "April 12, 2024"),
            ("2024-05-03", "May 3, 2024"),
            ("2024-06-07", "June 7, 2024"),
            ("2024-07-01", "July 1, 2024"),
        ],
    );
    let first = source(&inbox, "a.pdf");
    let second = source(&inbox, "b.pdf");
    let third = source(&inbox, "c.pdf");
    let queued = pipeline
        .enqueue_files(&[first, second, third])
        .unwrap()
        .into_iter()
        .map(|item| item.id)
        .collect::<Vec<_>>();
    pipeline.run_until_idle().unwrap();
    assert_eq!(
        record_of(&pipeline, queued[1]).filename,
        "2024-05-03 Employment Agreement between John Smith and Acme Corporation.pdf"
    );

    pipeline
        .approve(
            queued[0],
            "2024-04-12 Employment Agreement between John Smith and Acme.pdf",
            "Employment agreement between John Smith and Acme Corporation.",
        )
        .unwrap();
    let rules = pipeline.learned_rules().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(
        (rules[0].from.as_str(), rules[0].to.as_str()),
        ("Acme Corporation", "Acme")
    );
    assert_eq!((rules[0].seen, rules[0].active), (1, false));
    assert_eq!(
        record_of(&pipeline, queued[1]).filename,
        "2024-05-03 Employment Agreement between John Smith and Acme Corporation.pdf",
        "one edit is a decision about one document"
    );

    pipeline
        .approve(
            queued[1],
            "2024-05-03 Employment Agreement between John Smith and Acme.pdf",
            "Employment agreement between John Smith and Acme Corporation.",
        )
        .unwrap();
    let rules = pipeline.learned_rules().unwrap();
    assert_eq!((rules[0].seen, rules[0].active), (2, true));
    let waiting = record_of(&pipeline, queued[2]);
    assert_eq!(
        waiting.filename, "2024-06-07 Employment Agreement between John Smith and Acme.pdf",
        "the document still waiting is renamed in the queue"
    );
    assert_eq!(waiting.revision, 2);
    assert_eq!(waiting.house_rules.len(), 1);
    assert_eq!(
        waiting.analysis.proposal.parties,
        vec!["John Smith", "Acme Corporation"],
        "the analysis keeps the document's own words"
    );

    let fourth = source(&inbox, "d.pdf");
    let later = pipeline.enqueue_files(&[fourth]).unwrap()[0].id;
    pipeline.run_until_idle().unwrap();
    let proposed = record_of(&pipeline, later);
    assert_eq!(
        proposed.filename,
        "2024-07-01 Employment Agreement between John Smith and Acme.pdf"
    );
    assert_eq!(proposed.house_rules[0].to, "Acme");

    pipeline
        .approve(
            queued[2],
            &waiting.filename,
            "Employment agreement between John Smith and Acme Corporation.",
        )
        .unwrap();
    let heard = filing.filed.lock().unwrap();
    assert_eq!(heard.len(), 3);
    assert_eq!(heard[2].proposal.parties, vec!["John Smith", "Acme"]);
    drop(heard);
    let replay = pipeline.filed_documents().unwrap();
    assert!(
        replay
            .iter()
            .any(|document| document.proposal.parties == vec!["John Smith", "Acme"])
    );
    assert_eq!(
        pipeline.learned_rules().unwrap()[0].seen,
        2,
        "approving the spelling Intern applied teaches nothing new"
    );
}

/// The reviewer's name is the reviewer's. A second identical respelling makes
/// the spelling Intern's own and every waiting document is recomposed under
/// it - but not the one being approved, whose name a person has just typed:
/// recomposing that one throws away the date they typed with it, and the loss
/// shows the moment the apply does not go through.
#[test]
fn a_second_edit_that_also_typed_a_date_keeps_the_date_when_the_apply_fails() {
    let temp = tempdir().unwrap();
    // No date anywhere in the document, so validation withholds the model's
    // and the reviewer types one in.
    let undated = "Employment Agreement between John Smith and Acme Corporation         covering duties, salary, and term.";
    let first = source(temp.path(), "first.pdf");
    let second = source(temp.path(), "second.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(undated)),
        Ok(parsed(undated)),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "first-hash");
    files.trust(&second, "second-hash");
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        Arc::clone(&files),
        AppSettings::default(),
    );
    let queued = pipeline.enqueue_files(&[first, second]).unwrap();
    let first_id = queued[0].id;
    let second_id = queued[1].id;
    pipeline.run_until_idle().unwrap();
    assert_eq!(
        record_of(&pipeline, second_id).filename,
        "Employment Agreement between John Smith and Acme Corporation.pdf",
        "no date the document supports"
    );

    let approved = "2024-04-12 Employment Agreement between John Smith and Acme.pdf";
    pipeline
        .approve(first_id, approved, "A sentence about the agreement.")
        .unwrap();
    // The same respelling a second time, and this apply does not go through.
    files.fail_next_apply();
    let _ = pipeline.approve(second_id, approved, "A sentence about the agreement.");

    assert_eq!(
        record_of(&pipeline, second_id).filename,
        approved,
        "the name the reviewer typed, date and all"
    );
}

#[test]
fn a_spelling_can_be_used_at_once_and_forgotten_again() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    let (pipeline, _) = learning_pipeline(
        temp.path(),
        &[
            ("2024-04-12", "April 12, 2024"),
            ("2024-05-03", "May 3, 2024"),
        ],
    );
    let first = source(&inbox, "a.pdf");
    let second = source(&inbox, "b.pdf");
    let queued = pipeline
        .enqueue_files(&[first, second])
        .unwrap()
        .into_iter()
        .map(|item| item.id)
        .collect::<Vec<_>>();
    pipeline.run_until_idle().unwrap();
    pipeline
        .approve(
            queued[0],
            "2024-04-12 Employment Agreement between John Smith and Acme.pdf",
            "A sentence about the agreement.",
        )
        .unwrap();
    let rule = pipeline.learned_rules().unwrap().remove(0);
    assert!(!rule.active);

    pipeline.use_rule(rule.id).unwrap();
    assert!(pipeline.learned_rules().unwrap()[0].active);
    assert_eq!(
        record_of(&pipeline, queued[1]).filename,
        "2024-05-03 Employment Agreement between John Smith and Acme.pdf"
    );

    pipeline.forget_rule(rule.id).unwrap();
    assert!(pipeline.learned_rules().unwrap().is_empty());
    let restored = record_of(&pipeline, queued[1]);
    assert_eq!(
        restored.filename,
        "2024-05-03 Employment Agreement between John Smith and Acme Corporation.pdf"
    );
    assert!(restored.house_rules.is_empty());
    assert_eq!(
        pipeline.forget_rule(rule.id).unwrap_err().code,
        "RULE_NOT_FOUND"
    );
}

/// A spelling Intern applied that the reviewer undoes is retracted; one the
/// reviewer changes to a third form starts over from the document's word.
#[test]
fn respelling_interns_own_spelling_in_review_changes_the_rule_not_the_document() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    let (pipeline, _) = learning_pipeline(
        temp.path(),
        &[
            ("2024-04-12", "April 12, 2024"),
            ("2024-05-03", "May 3, 2024"),
            ("2024-06-07", "June 7, 2024"),
        ],
    );
    let first = source(&inbox, "a.pdf");
    let second = source(&inbox, "b.pdf");
    let third = source(&inbox, "c.pdf");
    let queued = pipeline
        .enqueue_files(&[first, second, third])
        .unwrap()
        .into_iter()
        .map(|item| item.id)
        .collect::<Vec<_>>();
    pipeline.run_until_idle().unwrap();
    pipeline
        .approve(
            queued[0],
            "2024-04-12 Employment Agreement between John Smith and Acme.pdf",
            "A sentence about the agreement.",
        )
        .unwrap();
    let rule = pipeline.learned_rules().unwrap().remove(0);
    pipeline.use_rule(rule.id).unwrap();
    assert!(
        record_of(&pipeline, queued[1])
            .filename
            .ends_with("and Acme.pdf")
    );

    // A third spelling: the rule now maps the document's word to it, and
    // has to be earned again.
    pipeline
        .approve(
            queued[1],
            "2024-05-03 Employment Agreement between John Smith and Acme Corp.pdf",
            "A sentence about the agreement.",
        )
        .unwrap();
    let rules = pipeline.learned_rules().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(
        (rules[0].from.as_str(), rules[0].to.as_str()),
        ("Acme Corporation", "Acme Corp")
    );
    assert_eq!((rules[0].seen, rules[0].active), (1, false));
    assert!(
        record_of(&pipeline, queued[2])
            .filename
            .ends_with("and Acme Corporation.pdf"),
        "an unearned rule is not applied"
    );

    // The document's own word restored retracts the rule outright.
    pipeline.use_rule(rules[0].id).unwrap();
    assert!(
        record_of(&pipeline, queued[2])
            .filename
            .ends_with("and Acme Corp.pdf")
    );
    pipeline
        .approve(
            queued[2],
            "2024-06-07 Employment Agreement between John Smith and Acme Corporation.pdf",
            "A sentence about the agreement.",
        )
        .unwrap();
    assert!(pipeline.learned_rules().unwrap().is_empty());
}

/// An agreement long enough to fingerprint, with the date and the names
/// the canned proposal quotes.
const AGREEMENT: &str = "EMPLOYMENT AGREEMENT\n\nThis Employment Agreement is signed April 12, 2024 \
    by John Smith and Acme Corporation. Acme Corporation employs John Smith as a senior \
    cartographer at its Fictional Harbor office. The position begins on the start date named \
    above and continues until terminated by either party on thirty days written notice. Salary, \
    benefits, and duties are described in the attached schedule, which forms part of this \
    agreement.";

/// A rename that really happens but is reported to the queue as a failure:
/// what the caller sees when the applier journalled an ambiguous operation
/// and a reconciliation finished it afterwards.
struct ReconciledApply {
    inner: CoreFileActions,
}

impl FileActions for ReconciledApply {
    fn fingerprint(&self, path: &Path) -> Result<String, PipelineError> {
        self.inner.fingerprint(path)
    }

    fn apply(&self, item: &QueueItem, destination: &Path) -> Result<(), PipelineError> {
        self.inner.apply(item, destination)?;
        Err(PipelineError::new(
            "MOVE_VERIFICATION_FAILED",
            "the rename could not be confirmed by the caller",
        ))
    }

    fn undo(&self, item: &QueueItem, receipt: &OperationReceipt) -> Result<(), PipelineError> {
        self.inner.undo(item, receipt)
    }

    fn reconcile(&self, item: &QueueItem) -> Result<(), PipelineError> {
        self.inner.reconcile(item)
    }
}

/// A rename finished by a reconciliation is still a rename: the records
/// keepers must hear about it, and it must be remembered as a filing, or the
/// description is never written and a second scan of the same document is
/// filed a second time.
#[test]
fn a_rename_finished_by_reconciliation_is_reported_and_fingerprinted() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    let database = temp.path().join("queue.sqlite3");
    let rescan_text = AGREEMENT
        .replace("employs John", "emplcys John")
        .replace("cartographer", "cartograpner")
        .replace("thirty", "thirly");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(AGREEMENT)),
        Ok(parsed(&rescan_text)),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let store = Arc::new(QueueStore::open(&database).unwrap());
    let filing = Arc::new(RecordingFiling::default());
    let pipeline = Pipeline::open(
        &database,
        worker,
        model,
        Arc::new(ReconciledApply {
            inner: CoreFileActions::local(store),
        }),
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_filing_sink(filing.clone());

    let original = source(&inbox, "agreement-scan-1.pdf");
    pipeline
        .enqueue_files(std::slice::from_ref(&original))
        .unwrap();
    pipeline.run_until_idle().unwrap();

    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);
    let destination = completed.receipt.clone().unwrap().destination;
    let filed = filing.filed.lock().unwrap().clone();
    assert_eq!(filed.len(), 1, "the filing sink is told once: {filed:?}");
    assert_eq!(filed[0].destination, destination);

    // And the filing is remembered, so a second scan of the same agreement is
    // recognised instead of being filed all over again.
    let rescan = source(&inbox, "agreement-scan-2.pdf");
    pipeline
        .enqueue_files(std::slice::from_ref(&rescan))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let second = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.source_path == rescan)
        .unwrap();
    assert_eq!(second.status, QueueStatus::NeedsReview);
    let record = second.proposal.as_ref().unwrap();
    assert!(
        record.reasons.iter().any(|reason| reason == NEAR_DUPLICATE),
        "{:?}",
        record.reasons
    );
    assert_eq!(
        record.near_duplicate_of.as_deref(),
        destination.file_name().and_then(|name| name.to_str())
    );
}

/// Two documents that say the same thing are one document filed twice. A
/// second scan of a filed agreement - different bytes, three misread words -
/// waits for a person and names the filing it repeats; the same template
/// signed a year later is a new document; and once the filing is undone the
/// scan is new again.
#[test]
fn a_second_scan_of_a_filed_document_waits_and_names_the_filing_it_repeats() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::create_dir_all(&filed).unwrap();
    let rescan_text = AGREEMENT
        .replace("employs John", "emplcys John")
        .replace("cartographer", "cartograpner")
        .replace("thirty", "thirly");
    let renewal_text = AGREEMENT
        .replace("April 12, 2024", "April 12, 2025")
        .replace("continues", "renews");
    let renewal = ModelProposal {
        document_date: Some("2025-04-12".into()),
        evidence: Evidence {
            date: Some("signed April 12, 2025".into()),
            ..proposal(0.94, false).evidence
        },
        ..proposal(0.94, false)
    };
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(AGREEMENT)),
        Ok(parsed(&rescan_text)),
        Ok(parsed(&renewal_text)),
        Ok(parsed(&rescan_text)),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
        Ok(renewal),
        Ok(proposal(0.94, false)),
    ]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();

    let original = source(&inbox, "agreement-scan-1.pdf");
    pipeline
        .enqueue_files(std::slice::from_ref(&original))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let first = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(first.status, QueueStatus::Ready);
    let filed_name = first.proposal.as_ref().unwrap().filename.clone();
    assert_eq!(
        filed_name,
        "2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf"
    );
    pipeline
        .approve(
            first.id,
            &filed_name,
            "Employment agreement between John Smith and Acme Corporation.",
        )
        .unwrap();

    let rescan = source(&inbox, "agreement-scan-2.pdf");
    pipeline
        .enqueue_files(std::slice::from_ref(&rescan))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let second = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.source_path == rescan)
        .unwrap();
    assert_eq!(
        second.status,
        QueueStatus::NeedsReview,
        "not filed on its own"
    );
    let record = second.proposal.as_ref().unwrap();
    assert!(
        record.reasons.iter().any(|reason| reason == NEAR_DUPLICATE),
        "{:?}",
        record.reasons
    );
    assert_eq!(
        record.near_duplicate_of.as_deref(),
        Some(filed_name.as_str())
    );
    assert_eq!(record.status, ProposalStatus::NeedsReview);

    let renewal_path = source(&inbox, "agreement-renewal.pdf");
    pipeline
        .enqueue_files(std::slice::from_ref(&renewal_path))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let third = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.source_path == renewal_path)
        .unwrap();
    assert_eq!(
        third.status,
        QueueStatus::Ready,
        "same words, another date: a new document"
    );
    assert_eq!(third.proposal.as_ref().unwrap().near_duplicate_of, None);

    pipeline.undo(first.id).unwrap();
    let again = source(&inbox, "agreement-scan-3.pdf");
    pipeline
        .enqueue_files(std::slice::from_ref(&again))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let fourth = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.source_path == again)
        .unwrap();
    assert_eq!(
        fourth.status,
        QueueStatus::Ready,
        "an undone filing is forgotten"
    );
    assert_eq!(fourth.proposal.as_ref().unwrap().near_duplicate_of, None);
}

/// Closeness alone does not decide, and neither does the closest row. A
/// renewal of the same agreement is nearer in text than a scan of the
/// original is, and used to be the only filing the duplicate check looked at:
/// its date said "another document", and the filing this scan really repeats
/// was never mentioned.
#[test]
fn the_closest_fingerprint_with_the_wrong_date_does_not_hide_the_true_duplicate() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::create_dir_all(&filed).unwrap();
    let rescan_text = AGREEMENT
        .replace("employs John", "emplcys John")
        .replace("cartographer", "cartograpner")
        .replace("thirly", "thirty");
    let renewal_text = AGREEMENT.replace("April 12, 2024", "April 12, 2025");
    let original = fingerprint::source_fingerprint(&parsed(AGREEMENT)).unwrap();
    let to_rescan = fingerprint::hamming(
        original,
        fingerprint::source_fingerprint(&parsed(&rescan_text)).unwrap(),
    );
    let to_renewal = fingerprint::hamming(
        original,
        fingerprint::source_fingerprint(&parsed(&renewal_text)).unwrap(),
    );
    assert!(
        to_renewal < to_rescan && to_rescan <= fingerprint::NEAR_DUPLICATE_DISTANCE,
        "the renewal must be the nearer filing and the rescan a near duplicate          ({to_renewal} then {to_rescan})"
    );
    let renewal = ModelProposal {
        document_date: Some("2025-04-12".into()),
        evidence: Evidence {
            date: Some("signed April 12, 2025".into()),
            ..proposal(0.94, false).evidence
        },
        ..proposal(0.94, false)
    };
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(&rescan_text)),
        Ok(parsed(&renewal_text)),
        Ok(parsed(AGREEMENT)),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(renewal),
        Ok(proposal(0.94, false)),
    ]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            automatic_rename: true,
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();

    for name in ["scan-1.pdf", "renewal.pdf"] {
        let path = source(&inbox, name);
        pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();
        pipeline.run_until_idle().unwrap();
    }
    let filings = pipeline.filed_documents().unwrap();
    assert_eq!(filings.len(), 2, "{filings:?}");

    let again = source(&inbox, "scan-2.pdf");
    pipeline
        .enqueue_files(std::slice::from_ref(&again))
        .unwrap();
    pipeline.run_until_idle().unwrap();

    let third = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.source_path == again)
        .unwrap();
    assert_eq!(third.status, QueueStatus::NeedsReview);
    let record = third.proposal.as_ref().unwrap();
    assert!(
        record.reasons.iter().any(|reason| reason == NEAR_DUPLICATE),
        "{:?}",
        record.reasons
    );
    assert_eq!(
        record.near_duplicate_of.as_deref(),
        Some("2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf"),
        "the filing that shares this document's date, not the nearer one"
    );
}

/// The shared index answers for teammates' machines. Its answer is held to
/// the same date test, and names the machine.
#[test]
fn a_teammates_filing_with_nearly_this_text_is_named_with_their_machine() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "agreement.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(AGREEMENT)),
        Ok(parsed(AGREEMENT)),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "agreement-hash");
    let teammates = Arc::new(TeammateFilings::default());
    *teammates.similar.lock().unwrap() = Some(SimilarFiling {
        filing: KnownFiling {
            filename: "2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf"
                .into(),
            filed_by: Some("Front desk".into()),
        },
        distance: 2,
    });
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        Arc::clone(&files),
        AppSettings::default(),
    )
    .with_duplicate_oracle(Arc::clone(&teammates) as Arc<dyn DuplicateOracle>);
    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();
    pipeline.run_until_idle().unwrap();
    let item = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(
        item.proposal.as_ref().unwrap().near_duplicate_of.as_deref(),
        Some(
            "2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf (filed from Front desk)"
        )
    );

    // The same words filed under another date, on another machine: a new
    // document here too.
    *teammates.similar.lock().unwrap() = Some(SimilarFiling {
        filing: KnownFiling {
            filename: "2023-04-12 Employment Agreement between John Smith and Acme Corporation.pdf"
                .into(),
            filed_by: Some("Front desk".into()),
        },
        distance: 0,
    });
    let other = source(temp.path(), "agreement-b.pdf");
    files.trust(&other, "agreement-b-hash");
    pipeline
        .enqueue_files(std::slice::from_ref(&other))
        .unwrap();
    pipeline.run_until_idle().unwrap();
    let item = pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.source_path == other)
        .unwrap();
    assert_eq!(item.status, QueueStatus::Ready);
}

struct UploaderGuard {
    allowed: AtomicBool,
    hash: Option<String>,
}

struct RetryableStageGuard {
    stage: intern_queue::AdmissionStage,
    retrying: AtomicBool,
}

impl RetryableStageGuard {
    fn unavailable_at(stage: intern_queue::AdmissionStage) -> Self {
        Self {
            stage,
            retrying: AtomicBool::new(true),
        }
    }

    fn allowed_until(stage: intern_queue::AdmissionStage) -> Self {
        Self {
            stage,
            retrying: AtomicBool::new(false),
        }
    }

    fn retry(&self) {
        self.retrying.store(true, Ordering::SeqCst);
    }

    fn allow(&self) {
        self.retrying.store(false, Ordering::SeqCst);
    }
}

impl intern_queue::AdmissionGuard for RetryableStageGuard {
    fn authorize(
        &self,
        _path: &Path,
        stage: intern_queue::AdmissionStage,
    ) -> Result<intern_queue::AdmissionEvidence, PipelineError> {
        if stage == self.stage && self.retrying.load(Ordering::SeqCst) {
            Err(PipelineError::retryable(
                "MICROSOFT_UNAVAILABLE",
                "Microsoft upload verification is temporarily unavailable.",
            ))
        } else {
            Ok(intern_queue::AdmissionEvidence::local())
        }
    }
}

fn automatic_settings() -> AppSettings {
    AppSettings {
        automatic_rename: true,
        ..AppSettings::default()
    }
}

const RETRY_DOCUMENT: &str =
    "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.";

#[test]
fn retryable_enqueue_stops_before_fingerprinting_or_creating_a_row() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "enqueue-retry.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(RETRY_DOCUMENT))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let files = Arc::new(FakeFiles::default());
    let guard = Arc::new(RetryableStageGuard::unavailable_at(
        intern_queue::AdmissionStage::Enqueue,
    ));
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        files.clone(),
        automatic_settings(),
    )
    .with_admission_guard(guard.clone());

    let error = pipeline
        .enqueue_files(std::slice::from_ref(&path))
        .unwrap_err();

    assert!(error.is_retryable());
    assert_eq!(error.code, "MICROSOFT_UNAVAILABLE");
    assert_eq!(files.fingerprint_calls.load(Ordering::SeqCst), 0);
    assert!(pipeline.list().unwrap().is_empty());

    files.trust(&path, "enqueue-hash");
    guard.allow();
    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();
    pipeline.run_until_idle().unwrap();
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Ready);
    assert_eq!(files.applies.lock().unwrap().len(), 1);
}

#[test]
fn retryable_extract_restores_queued_and_stops_the_current_drain() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "extract-retry-a.pdf");
    let second = source(temp.path(), "extract-retry-b.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(RETRY_DOCUMENT)),
        Ok(parsed(RETRY_DOCUMENT)),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "extract-a-hash");
    files.trust(&second, "extract-b-hash");
    let guard = Arc::new(RetryableStageGuard::unavailable_at(
        intern_queue::AdmissionStage::Extract,
    ));
    let pipeline = pipeline(
        temp.path(),
        worker.clone(),
        model.clone(),
        files.clone(),
        automatic_settings(),
    )
    .with_admission_guard(guard.clone());
    pipeline.enqueue_files(&[first, second]).unwrap();

    pipeline.run_until_idle().unwrap();

    let waiting = pipeline.list().unwrap();
    assert!(waiting.iter().all(|item| {
        item.status == QueueStatus::Queued && item.error_code.is_none() && item.proposal.is_none()
    }));
    assert_eq!(worker.calls.load(Ordering::SeqCst), 0);
    assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    assert!(files.applies.lock().unwrap().is_empty());

    guard.allow();
    pipeline.run_until_idle().unwrap();
    assert!(
        pipeline
            .list()
            .unwrap()
            .iter()
            .all(|item| item.status == QueueStatus::Ready)
    );
    assert_eq!(files.applies.lock().unwrap().len(), 2);
}

#[test]
fn retryable_analyze_restores_queued_and_stops_the_current_drain() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "analyze-retry-a.pdf");
    let second = source(temp.path(), "analyze-retry-b.pdf");
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(RETRY_DOCUMENT)),
        Ok(parsed(RETRY_DOCUMENT)),
        Ok(parsed(RETRY_DOCUMENT)),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(proposal(0.94, false)),
    ]));
    let files = Arc::new(FakeFiles::default());
    files.trust(&first, "analyze-a-hash");
    files.trust(&second, "analyze-b-hash");
    let guard = Arc::new(RetryableStageGuard::unavailable_at(
        intern_queue::AdmissionStage::Analyze,
    ));
    let pipeline = pipeline(
        temp.path(),
        worker.clone(),
        model.clone(),
        files.clone(),
        automatic_settings(),
    )
    .with_admission_guard(guard.clone());
    pipeline.enqueue_files(&[first, second]).unwrap();

    pipeline.run_until_idle().unwrap();

    let waiting = pipeline.list().unwrap();
    assert!(waiting.iter().all(|item| {
        item.status == QueueStatus::Queued && item.error_code.is_none() && item.proposal.is_none()
    }));
    assert_eq!(worker.calls.load(Ordering::SeqCst), 1);
    assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    assert!(files.applies.lock().unwrap().is_empty());

    guard.allow();
    pipeline.run_until_idle().unwrap();
    assert!(
        pipeline
            .list()
            .unwrap()
            .iter()
            .all(|item| item.status == QueueStatus::Ready)
    );
    assert_eq!(files.applies.lock().unwrap().len(), 2);
}

#[test]
fn retryable_automatic_apply_preserves_ready_proposal_without_file_actions() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    fs::create_dir_all(&inbox).unwrap();
    fs::create_dir_all(&filed).unwrap();
    let path = source(&inbox, "automatic-apply-retry.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(RETRY_DOCUMENT))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let guard = Arc::new(RetryableStageGuard::unavailable_at(
        intern_queue::AdmissionStage::Apply,
    ));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            automatic_rename: true,
            destination: filed.to_string_lossy().into_owned(),
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker.clone(),
        model.clone(),
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_admission_guard(guard.clone());
    pipeline.enqueue_files(&[path]).unwrap();

    pipeline.run_until_idle().unwrap();

    let waiting = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(waiting.status, QueueStatus::Ready);
    assert_eq!(waiting.error_code, None);
    let proposal_before = waiting
        .proposal
        .expect("analysis produced a ready proposal");
    assert!(waiting.receipt.is_none());
    assert!(waiting.source_path.exists());
    assert!(fs::read_dir(&filed).unwrap().next().is_none());

    guard.allow();
    pipeline.run_until_idle().unwrap();
    let filed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(filed.status, QueueStatus::Completed);
    assert_eq!(filed.proposal.as_ref(), Some(&proposal_before));
    assert!(filed.receipt.is_some());
    assert!(!filed.source_path.exists());
    assert_eq!(worker.calls.load(Ordering::SeqCst), 1);
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn retryable_apply_during_approval_preserves_ready_proposal() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    fs::create_dir_all(&inbox).unwrap();
    fs::create_dir_all(&filed).unwrap();
    let path = source(&inbox, "approved-apply-retry.pdf");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(RETRY_DOCUMENT))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let guard = Arc::new(RetryableStageGuard::allowed_until(
        intern_queue::AdmissionStage::Apply,
    ));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker.clone(),
        model.clone(),
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_admission_guard(guard.clone());
    let id = pipeline.enqueue_files(&[path]).unwrap()[0].id;
    pipeline.run_until_idle().unwrap();
    let ready = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(ready.status, QueueStatus::Ready);
    let proposal_before = ready.proposal.clone().unwrap();

    guard.retry();
    let error = pipeline
        .approve(id, "2024-04-12 Employment Agreement.pdf", "")
        .unwrap_err();

    assert!(error.is_retryable());
    let retained = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(retained.status, QueueStatus::Ready);
    assert_eq!(retained.error_code, None);
    assert_eq!(retained.proposal.as_ref(), Some(&proposal_before));
    assert!(retained.receipt.is_none());
    assert!(retained.source_path.exists());
    assert!(fs::read_dir(&filed).unwrap().next().is_none());
    assert_eq!(worker.calls.load(Ordering::SeqCst), 1);
    assert_eq!(model.calls.load(Ordering::SeqCst), 1);

    guard.allow();
    pipeline
        .approve(id, "2024-04-12 Employment Agreement.pdf", "")
        .unwrap();
    let completed = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(completed.status, QueueStatus::Completed);
    assert!(completed.receipt.is_some());
    assert!(!completed.source_path.exists());
}

// Synthetic identifiers shaped like real Microsoft Graph values; none are real.
const SNAPSHOT_TENANT: &str = "11111111-1111-1111-1111-111111111111";
const SNAPSHOT_DRIVE: &str = "b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO";
const SNAPSHOT_INBOX: &str = "01SYNTHETICINBOXFOLDERAAAAAAAAAAAA";
const SNAPSHOT_ME: &str = "99999999-9999-9999-9999-999999999999";
const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

struct ProofSnapshotGuard {
    source: PathBuf,
    inbox: PathBuf,
    snapshots: PrivateSnapshotDirectory,
    deployment: SharePointDeployment,
    metadata_calls: AtomicUsize,
}

impl FreshUploadMetadata for ProofSnapshotGuard {
    fn metadata(&self, _url: Url) -> Result<(Account, Value), String> {
        self.metadata_calls.fetch_add(1, Ordering::SeqCst);
        let mut quick = QuickXor::default();
        quick.update(b"hello");
        Ok((
            Account {
                tenant_id: SNAPSHOT_TENANT.into(),
                id: SNAPSHOT_ME.into(),
                display_name: "Pat Example".into(),
                email: "pat@example.test".into(),
                user_principal_name: "pat@example.test".into(),
            },
            json!({
                "id": "01SYNTHETICAGREEMENTFILEAAAAAAAAAA",
                "eTag": "\"fresh,1\"",
                "cTag": "\"content,1\"",
                "name": "swap.pdf",
                "size": 5,
                "webUrl": "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox/swap.pdf",
                "parentReference": { "driveId": SNAPSHOT_DRIVE, "id": SNAPSHOT_INBOX },
                "sharepointIds": {
                    "tenantId": SNAPSHOT_TENANT,
                    "siteId": "33333333-3333-3333-3333-333333333333",
                    "webId": "44444444-4444-4444-4444-444444444444",
                    "listId": "55555555-5555-5555-5555-555555555555",
                    "listItemUniqueId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
                },
                "createdBy": { "user": { "id": SNAPSHOT_ME, "userPrincipalName": "pat@example.test" } },
                "lastModifiedBy": { "user": { "id": SNAPSHOT_ME, "userPrincipalName": "pat@example.test" } },
                "createdDateTime": "2026-09-14T16:00:00Z",
                "lastModifiedDateTime": "2026-09-14T16:00:00Z",
                "file": {
                    "mimeType": "application/pdf",
                    "hashes": { "quickXorHash": STANDARD.encode(quick.finish()) }
                }
            }),
        ))
    }
}

impl intern_queue::AdmissionGuard for ProofSnapshotGuard {
    fn authorize(
        &self,
        path: &Path,
        stage: intern_queue::AdmissionStage,
    ) -> Result<intern_queue::AdmissionEvidence, PipelineError> {
        match verify_fresh_upload(
            &self.deployment,
            self,
            1_789_401_599_000,
            &self.snapshots,
            &self.inbox,
            path,
        ) {
            FreshUploadOutcome::Authorized {
                local_sha256,
                snapshot,
                ..
            } => {
                if stage == intern_queue::AdmissionStage::Extract {
                    fs::write(&self.source, b"replacement bytes").unwrap();
                }
                Ok(intern_queue::AdmissionEvidence::verified_snapshot(
                    local_sha256,
                    snapshot,
                ))
            }
            other => Err(PipelineError::new(
                "UPLOADER_UNVERIFIED",
                format!("unexpected proof outcome: {other:?}"),
            )),
        }
    }
}

struct SnapshotReadingWorker {
    public_source: PathBuf,
    read: Mutex<Vec<u8>>,
}

impl WorkerBoundary for SnapshotReadingWorker {
    fn extract(
        &self,
        _request_id: &str,
        path: &Path,
        _progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, WorkerFailure> {
        *self.read.lock().unwrap() = fs::read(path).unwrap();
        fs::write(&self.public_source, b"hello").unwrap();
        Err(WorkerFailure::new("TEST_STOP", false, false))
    }

    fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
        Ok(())
    }

    fn restart(&self) -> Result<(), WorkerFailure> {
        Ok(())
    }
}

#[test]
fn extraction_reads_the_owned_verified_snapshot_across_a_source_swap_and_restore() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    fs::create_dir(&inbox).unwrap();
    let source_path = source(&inbox, "swap.pdf");
    fs::write(&source_path, b"hello").unwrap();
    let private_root = temp.path().join("private-snapshots");
    let files = Arc::new(FakeFiles::default());
    files.trust(&source_path, HELLO_SHA256);
    let worker = Arc::new(SnapshotReadingWorker {
        public_source: source_path.clone(),
        read: Mutex::new(Vec::new()),
    });
    let deployment = SharePointDeployment::from_slice(
        format!(
            r#"{{
              "schema_version": 1,
              "enabled": true,
              "site_url": "https://teamcontoso.sharepoint.com/sites/InternTestSite",
              "library_name": "Files",
              "intake_folder_name": "Inbox",
              "destination_folder_name": "Filed",
              "tenant_id": "{SNAPSHOT_TENANT}",
              "client_id": "22222222-2222-2222-2222-222222222222",
              "site_id": "33333333-3333-3333-3333-333333333333",
              "web_id": "44444444-4444-4444-4444-444444444444",
              "list_id": "55555555-5555-5555-5555-555555555555",
              "drive_id": "{SNAPSHOT_DRIVE}",
              "intake_folder_id": "{SNAPSHOT_INBOX}",
              "destination_folder_id": "01SYNTHETICFILEDFOLDERAAAAAAAAAAAA"
            }}"#
        )
        .as_bytes(),
    )
    .unwrap();
    let guard = Arc::new(ProofSnapshotGuard {
        source: source_path.clone(),
        inbox: inbox.canonicalize().unwrap(),
        snapshots: PrivateSnapshotDirectory::new(&private_root).unwrap(),
        deployment,
        metadata_calls: AtomicUsize::new(0),
    });
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings.save(&AppSettings::default()).unwrap();
    let pipeline = Pipeline::open(
        temp.path().join("queue.sqlite3"),
        worker.clone(),
        Arc::new(FakeModel::new(vec![])),
        files,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap()
    .with_admission_guard(guard.clone());

    pipeline
        .enqueue_files(std::slice::from_ref(&source_path))
        .unwrap();
    pipeline.run_next().unwrap();

    assert_eq!(&*worker.read.lock().unwrap(), b"hello");
    assert_eq!(fs::read(&source_path).unwrap(), b"hello");
    assert_eq!(guard.metadata_calls.load(Ordering::SeqCst), 4);
    assert!(
        fs::read_dir(&private_root).unwrap().next().is_none(),
        "the queue drops and removes each proof-owned snapshot"
    );
}

struct HashOnlyGuard;

impl intern_queue::AdmissionGuard for HashOnlyGuard {
    fn authorize(
        &self,
        _path: &Path,
        _stage: intern_queue::AdmissionStage,
    ) -> Result<intern_queue::AdmissionEvidence, PipelineError> {
        Ok(intern_queue::AdmissionEvidence::verified(
            "same-bytes".into(),
        ))
    }
}

#[test]
fn protected_extraction_without_an_owned_snapshot_fails_closed_before_the_worker() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "missing-snapshot.pdf");
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "same-bytes");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed("private text"))]));
    let pipeline = pipeline(
        temp.path(),
        worker.clone(),
        Arc::new(FakeModel::new(vec![])),
        files,
        AppSettings::default(),
    )
    .with_admission_guard(Arc::new(HashOnlyGuard));

    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();
    pipeline.run_next().unwrap();

    assert_eq!(worker.maximum_active.load(Ordering::SeqCst), 0);
    let item = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(item.error_code, Some(ErrorCode::UploaderUnverified));
}

impl intern_queue::AdmissionGuard for UploaderGuard {
    fn authorize(
        &self,
        _path: &Path,
        _stage: intern_queue::AdmissionStage,
    ) -> Result<intern_queue::AdmissionEvidence, PipelineError> {
        if self.allowed.load(Ordering::SeqCst) {
            Ok(self.hash.clone().map_or_else(
                intern_queue::AdmissionEvidence::local,
                intern_queue::AdmissionEvidence::verified,
            ))
        } else {
            Err(PipelineError::new(
                "UPLOADER_UNVERIFIED",
                "Uploader could not be verified.",
            ))
        }
    }
}
#[test]
fn uploader_guard_denies_before_any_source_fingerprinting_or_enqueue() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "new.pdf");
    let guard = Arc::new(UploaderGuard {
        allowed: AtomicBool::new(false),
        hash: None,
    });
    let pipeline = pipeline(
        temp.path(),
        Arc::new(FakeWorker::new(vec![])),
        Arc::new(FakeModel::new(vec![])),
        Arc::new(FakeFiles::default()),
        AppSettings::default(),
    )
    .with_admission_guard(guard);
    // FakeFiles has no fingerprint for the file. FILE_MISSING proves a read was
    // attempted before identity authorization, rather than this explicit hold.
    let error = pipeline.enqueue_files(&[path]).unwrap_err();
    assert_eq!(error.code, "UPLOADER_UNVERIFIED");
    assert!(pipeline.list().unwrap().is_empty());
}
#[test]
fn uploader_guard_rechecks_a_previously_queued_document_before_extraction() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "new.pdf");
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "same-bytes");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed("private text"))]));
    let model = Arc::new(FakeModel::new(vec![]));
    let guard = Arc::new(UploaderGuard {
        allowed: AtomicBool::new(true),
        hash: Some("same-bytes".into()),
    });
    let pipeline = pipeline(
        temp.path(),
        worker.clone(),
        model.clone(),
        files,
        AppSettings::default(),
    )
    .with_admission_guard(guard.clone());
    pipeline.enqueue_files(&[path]).unwrap();
    guard.allowed.store(false, Ordering::SeqCst);
    pipeline.run_until_idle().unwrap();
    assert_eq!(worker.maximum_active.load(Ordering::SeqCst), 0);
    assert_eq!(model.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        pipeline.list().unwrap()[0].error_code,
        Some(ErrorCode::UploaderUnverified)
    );
}

/// A denial at admission happens before any analysis, so there is nothing to
/// review and nothing to approve: the only useful thing a person can do is
/// verify the file and ask for it again. Retry used to refuse.
#[test]
fn an_item_denied_at_extraction_can_be_retried_after_verification() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "denied.pdf");
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "denied-hash");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let guard = Arc::new(UploaderGuard {
        allowed: AtomicBool::new(true),
        hash: None,
    });
    let pipeline = pipeline(temp.path(), worker, model, files, AppSettings::default())
        .with_admission_guard(guard.clone());
    pipeline.enqueue_files(&[path]).unwrap();
    guard.allowed.store(false, Ordering::SeqCst);
    pipeline.run_until_idle().unwrap();
    let denied = pipeline.list().unwrap().pop().unwrap();
    assert_eq!(denied.status, QueueStatus::NeedsReview);
    assert_eq!(denied.error_code, Some(ErrorCode::UploaderUnverified));
    assert!(denied.proposal.is_none(), "nothing was analyzed");

    guard.allowed.store(true, Ordering::SeqCst);
    pipeline.retry(denied.id).unwrap();
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Queued);

    pipeline.run_until_idle().unwrap();
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::Ready);
}

#[test]
fn uploader_guard_binds_provider_verified_bytes_to_the_queue_fingerprint() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "new.pdf");
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "replacement");
    let guard = Arc::new(UploaderGuard {
        allowed: AtomicBool::new(true),
        hash: Some("verified-version".into()),
    });
    let pipeline = pipeline(
        temp.path(),
        Arc::new(FakeWorker::new(vec![])),
        Arc::new(FakeModel::new(vec![])),
        files,
        AppSettings::default(),
    )
    .with_admission_guard(guard);
    assert_eq!(
        pipeline.enqueue_files(&[path]).unwrap_err().code,
        "FILE_CHANGED"
    );
    assert!(pipeline.list().unwrap().is_empty());
}

#[test]
fn uploader_guard_rechecks_at_apply_and_never_renames_after_disconnect() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "new.pdf");
    let files = Arc::new(FakeFiles::default());
    files.trust(&path, "same-bytes");
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(
        "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.",
    ))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let guard = Arc::new(UploaderGuard {
        allowed: AtomicBool::new(true),
        hash: Some("same-bytes".into()),
    });
    let pipeline = pipeline(
        temp.path(),
        worker,
        model,
        files.clone(),
        AppSettings::default(),
    )
    .with_admission_guard(guard.clone());
    let id = pipeline.enqueue_files(&[path.clone()]).unwrap()[0].id;
    pipeline.run_until_idle().unwrap();
    guard.allowed.store(false, Ordering::SeqCst);
    assert_eq!(
        pipeline
            .approve(id, "2024-04-12 Employment Agreement.pdf", "")
            .unwrap_err()
            .code,
        "UPLOADER_UNVERIFIED"
    );
    assert!(files.applies.lock().unwrap().is_empty());
    assert!(path.exists());
    assert_eq!(pipeline.list().unwrap()[0].status, QueueStatus::NeedsReview);
}

/// What the connected Microsoft account and Graph report while a
/// [`ScriptedProofGuard`] runs the real fresh-upload proof.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ProofScenario {
    /// The connected account created the unchanged upload: a complete proof.
    Proven,
    /// Microsoft is disconnected, so no proof can be attempted.
    Disconnected,
    /// A different person is now the connected account.
    AccountChanged,
    /// The connected account changes between the proof's two metadata reads.
    AccountChangesDuringProof,
    /// The item's revision changes between the proof's two metadata reads.
    RevisionChanges,
}

const BYPASS_SCENARIOS: [ProofScenario; 4] = [
    ProofScenario::Disconnected,
    ProofScenario::AccountChanged,
    ProofScenario::AccountChangesDuringProof,
    ProofScenario::RevisionChanges,
];

const SNAPSHOT_OTHER: &str = "aaaaaaaa-9999-9999-9999-999999999999";

/// An admission guard that runs the real `verify_fresh_upload` against
/// scripted Graph metadata. Stages before `deny_from` see a complete proof;
/// `deny_from` and every later stage see `scenario`.
struct ScriptedProofGuard {
    inbox: PathBuf,
    snapshots: PrivateSnapshotDirectory,
    deployment: SharePointDeployment,
    deny_from: Mutex<Option<(intern_queue::AdmissionStage, ProofScenario)>>,
    active: Mutex<ProofScenario>,
    /// Metadata reads within the current proof.
    reads: AtomicUsize,
}

impl ScriptedProofGuard {
    fn new(root: &Path, inbox: &Path) -> Arc<Self> {
        Arc::new(Self {
            inbox: inbox.canonicalize().unwrap(),
            snapshots: PrivateSnapshotDirectory::new(root.join("private-snapshots")).unwrap(),
            deployment: snapshot_deployment(),
            deny_from: Mutex::new(None),
            active: Mutex::new(ProofScenario::Proven),
            reads: AtomicUsize::new(0),
        })
    }

    fn deny_from(&self, stage: intern_queue::AdmissionStage, scenario: ProofScenario) {
        *self.deny_from.lock().unwrap() = Some((stage, scenario));
    }

    fn prove(&self) {
        *self.deny_from.lock().unwrap() = None;
    }
}

fn stage_rank(stage: intern_queue::AdmissionStage) -> u8 {
    match stage {
        intern_queue::AdmissionStage::Enqueue => 0,
        intern_queue::AdmissionStage::Extract => 1,
        intern_queue::AdmissionStage::Analyze => 2,
        intern_queue::AdmissionStage::Apply => 3,
    }
}

fn snapshot_deployment() -> SharePointDeployment {
    SharePointDeployment::from_slice(
        format!(
            r#"{{
              "schema_version": 1,
              "enabled": true,
              "site_url": "https://teamcontoso.sharepoint.com/sites/InternTestSite",
              "library_name": "Files",
              "intake_folder_name": "Inbox",
              "destination_folder_name": "Filed",
              "tenant_id": "{SNAPSHOT_TENANT}",
              "client_id": "22222222-2222-2222-2222-222222222222",
              "site_id": "33333333-3333-3333-3333-333333333333",
              "web_id": "44444444-4444-4444-4444-444444444444",
              "list_id": "55555555-5555-5555-5555-555555555555",
              "drive_id": "{SNAPSHOT_DRIVE}",
              "intake_folder_id": "{SNAPSHOT_INBOX}",
              "destination_folder_id": "01SYNTHETICFILEDFOLDERAAAAAAAAAAAA"
            }}"#
        )
        .as_bytes(),
    )
    .unwrap()
}

impl FreshUploadMetadata for ScriptedProofGuard {
    fn metadata(&self, url: Url) -> Result<(Account, Value), String> {
        let second_read = self.reads.fetch_add(1, Ordering::SeqCst) > 0;
        let scenario = *self.active.lock().unwrap();
        let connected = match scenario {
            ProofScenario::AccountChanged => SNAPSHOT_OTHER,
            ProofScenario::AccountChangesDuringProof if second_read => SNAPSHOT_OTHER,
            _ => SNAPSHOT_ME,
        };
        let etag = if scenario == ProofScenario::RevisionChanges && second_read {
            "\"fresh,2\""
        } else {
            "\"fresh,1\""
        };
        let name = url.path_segments().unwrap().next_back().unwrap().to_owned();
        let mut quick = QuickXor::default();
        quick.update(b"hello");
        Ok((
            Account {
                tenant_id: SNAPSHOT_TENANT.into(),
                id: connected.into(),
                display_name: "Pat Example".into(),
                email: "pat@example.test".into(),
                user_principal_name: "pat@example.test".into(),
            },
            json!({
                "id": "01SYNTHETICAGREEMENTFILEAAAAAAAAAA",
                "eTag": etag,
                "cTag": "\"content,1\"",
                "name": name,
                "size": 5,
                "webUrl": format!("https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox/{name}"),
                "parentReference": { "driveId": SNAPSHOT_DRIVE, "id": SNAPSHOT_INBOX },
                "sharepointIds": {
                    "tenantId": SNAPSHOT_TENANT,
                    "siteId": "33333333-3333-3333-3333-333333333333",
                    "webId": "44444444-4444-4444-4444-444444444444",
                    "listId": "55555555-5555-5555-5555-555555555555",
                    "listItemUniqueId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
                },
                "createdBy": { "user": { "id": SNAPSHOT_ME, "userPrincipalName": "pat@example.test" } },
                "lastModifiedBy": { "user": { "id": SNAPSHOT_ME, "userPrincipalName": "pat@example.test" } },
                "createdDateTime": "2026-09-14T16:00:00Z",
                "lastModifiedDateTime": "2026-09-14T16:00:00Z",
                "file": {
                    "mimeType": "application/pdf",
                    "hashes": { "quickXorHash": STANDARD.encode(quick.finish()) }
                }
            }),
        ))
    }
}

impl intern_queue::AdmissionGuard for ScriptedProofGuard {
    fn authorize(
        &self,
        path: &Path,
        stage: intern_queue::AdmissionStage,
    ) -> Result<intern_queue::AdmissionEvidence, PipelineError> {
        let scenario = match *self.deny_from.lock().unwrap() {
            Some((from, scenario)) if stage_rank(stage) >= stage_rank(from) => scenario,
            _ => ProofScenario::Proven,
        };
        if scenario == ProofScenario::Disconnected {
            return Err(PipelineError::new(
                "UPLOADER_UNVERIFIED",
                "Microsoft is disconnected. Unverified uploads are never processed.",
            ));
        }
        *self.active.lock().unwrap() = scenario;
        self.reads.store(0, Ordering::SeqCst);
        // The same outcome mapping the desktop Microsoft intake manager uses.
        match verify_fresh_upload(
            &self.deployment,
            self,
            1_789_401_599_000,
            &self.snapshots,
            &self.inbox,
            path,
        ) {
            FreshUploadOutcome::Authorized {
                local_sha256,
                snapshot,
                ..
            } => Ok(intern_queue::AdmissionEvidence::verified_snapshot(
                local_sha256,
                snapshot,
            )),
            FreshUploadOutcome::HeldOther { reason, .. } => Err(PipelineError::new(
                "UPLOADER_OTHER",
                format!("UPLOADER_OTHER: {reason}"),
            )),
            FreshUploadOutcome::HeldUnknown { reason } => {
                Err(PipelineError::new("UPLOADER_UNVERIFIED", reason))
            }
            FreshUploadOutcome::RetryableUnavailable { reason } => {
                Err(PipelineError::retryable("UPLOADER_UNVERIFIED", reason))
            }
        }
    }
}

struct ProofRig {
    _temp: tempfile::TempDir,
    path: PathBuf,
    guard: Arc<ScriptedProofGuard>,
    worker: Arc<FakeWorker>,
    model: Arc<FakeModel>,
    files: Arc<FakeFiles>,
    pipeline: Pipeline,
}

impl ProofRig {
    fn new(settings: AppSettings) -> Self {
        let temp = tempdir().unwrap();
        let inbox = temp.path().join("Inbox");
        fs::create_dir(&inbox).unwrap();
        let path = inbox.join("agreement.pdf");
        fs::write(&path, b"hello").unwrap();
        let path = path.canonicalize().unwrap();
        let files = Arc::new(FakeFiles::default());
        files.trust(&path, HELLO_SHA256);
        let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(RETRY_DOCUMENT))]));
        let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
        let guard = ScriptedProofGuard::new(temp.path(), &inbox);
        let pipeline = pipeline(
            temp.path(),
            worker.clone(),
            model.clone(),
            files.clone(),
            settings,
        )
        .with_admission_guard(guard.clone());
        Self {
            _temp: temp,
            path,
            guard,
            worker,
            model,
            files,
            pipeline,
        }
    }

    /// Extractor calls, model calls, and completed file actions.
    fn counters(&self) -> (usize, usize, usize) {
        (
            self.worker.calls.load(Ordering::SeqCst),
            self.model.calls.load(Ordering::SeqCst),
            self.files.applies.lock().unwrap().len(),
        )
    }
}

#[test]
fn microsoft_proof_gates_watcher_and_manual_admission_before_any_file_read() {
    // The watcher's admission check and a manual add of a file in a protected
    // folder both reach the queue through `enqueue_files` at the Enqueue stage.
    for scenario in BYPASS_SCENARIOS {
        let rig = ProofRig::new(automatic_settings());
        rig.guard
            .deny_from(intern_queue::AdmissionStage::Enqueue, scenario);

        let error = rig
            .pipeline
            .enqueue_files(std::slice::from_ref(&rig.path))
            .unwrap_err();

        assert!(
            error.code.starts_with("UPLOADER_"),
            "{scenario:?}: {error:?}"
        );
        assert!(rig.pipeline.list().unwrap().is_empty(), "{scenario:?}");
        assert_eq!(
            rig.files.fingerprint_calls.load(Ordering::SeqCst),
            0,
            "{scenario:?}"
        );
        rig.pipeline.run_until_idle().unwrap();
        assert_eq!(rig.counters(), (0, 0, 0), "{scenario:?}");
    }
}

#[test]
fn microsoft_proof_gates_extraction_and_retry_until_the_same_account_proves_the_upload() {
    for scenario in BYPASS_SCENARIOS {
        let rig = ProofRig::new(automatic_settings());
        let id = rig
            .pipeline
            .enqueue_files(std::slice::from_ref(&rig.path))
            .unwrap()[0]
            .id;
        rig.guard
            .deny_from(intern_queue::AdmissionStage::Extract, scenario);

        rig.pipeline.run_until_idle().unwrap();
        assert_eq!(rig.counters(), (0, 0, 0), "{scenario:?}");
        let held = rig.pipeline.list().unwrap().pop().unwrap();
        assert_eq!(held.status, QueueStatus::NeedsReview, "{scenario:?}");
        assert_eq!(
            held.error_code,
            Some(ErrorCode::UploaderUnverified),
            "{scenario:?}"
        );

        // Retrying while the proof is still incomplete reads nothing.
        rig.pipeline.retry(id).unwrap();
        rig.pipeline.run_until_idle().unwrap();
        assert_eq!(rig.counters(), (0, 0, 0), "{scenario:?}");

        rig.guard.prove();
        rig.pipeline.retry(id).unwrap();
        rig.pipeline.run_until_idle().unwrap();
        assert_eq!(rig.counters(), (1, 1, 1), "{scenario:?}");
    }
}

#[test]
fn microsoft_proof_gates_inference_after_a_proven_extraction() {
    for scenario in BYPASS_SCENARIOS {
        let rig = ProofRig::new(automatic_settings());
        rig.pipeline
            .enqueue_files(std::slice::from_ref(&rig.path))
            .unwrap();
        rig.guard
            .deny_from(intern_queue::AdmissionStage::Analyze, scenario);

        rig.pipeline.run_until_idle().unwrap();

        assert_eq!(rig.counters(), (1, 0, 0), "{scenario:?}");
        let held = rig.pipeline.list().unwrap().pop().unwrap();
        assert_eq!(held.status, QueueStatus::NeedsReview, "{scenario:?}");
        assert!(held.proposal.is_none(), "{scenario:?}");
    }
}

#[test]
fn microsoft_proof_gates_automatic_and_approved_apply() {
    for scenario in BYPASS_SCENARIOS {
        let automatic = ProofRig::new(automatic_settings());
        automatic
            .pipeline
            .enqueue_files(std::slice::from_ref(&automatic.path))
            .unwrap();
        automatic
            .guard
            .deny_from(intern_queue::AdmissionStage::Apply, scenario);
        automatic.pipeline.run_until_idle().unwrap();
        assert_eq!(automatic.counters(), (1, 1, 0), "{scenario:?}");
        assert_ne!(
            automatic.pipeline.list().unwrap()[0].status,
            QueueStatus::Completed,
            "{scenario:?}"
        );

        let reviewed = ProofRig::new(AppSettings::default());
        let id = reviewed
            .pipeline
            .enqueue_files(std::slice::from_ref(&reviewed.path))
            .unwrap()[0]
            .id;
        reviewed.pipeline.run_until_idle().unwrap();
        assert_eq!(
            reviewed.pipeline.list().unwrap()[0].status,
            QueueStatus::Ready,
            "{scenario:?}"
        );
        reviewed
            .guard
            .deny_from(intern_queue::AdmissionStage::Apply, scenario);
        let error = reviewed
            .pipeline
            .approve(id, "2024-04-12 Employment Agreement.pdf", "")
            .unwrap_err();
        assert!(
            error.code.starts_with("UPLOADER_"),
            "{scenario:?}: {error:?}"
        );
        assert_eq!(reviewed.counters(), (1, 1, 0), "{scenario:?}");
    }
}

const SIGNED_AGREEMENT: &str =
    "Employment Agreement signed April 12, 2024 by John Smith and Acme Corporation.";
const AGREEMENT_NAME: &str =
    "2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf";

/// A queue over `filesystem`, sharing the queue's own store, with
/// `documents` agreements to read and automatic renaming off - so every
/// rename is an approval, made when the test says.
fn reviewed_queue(root: &Path, filesystem: Arc<dyn FileSystem>, documents: usize) -> Pipeline {
    let worker = Arc::new(FakeWorker::new(
        (0..documents)
            .map(|_| Ok(parsed(SIGNED_AGREEMENT)))
            .collect(),
    ));
    let model = Arc::new(FakeModel::new(
        (0..documents).map(|_| Ok(proposal(0.94, false))).collect(),
    ));
    let settings = SettingsStore::new(root.join("settings.json"));
    settings.save(&AppSettings::default()).unwrap();
    Pipeline::with_file_system(
        root.join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
        filesystem,
    )
    .unwrap()
}

fn item_of(pipeline: &Pipeline, id: i64) -> intern_queue::PipelineItem {
    pipeline
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap()
}

/// Reads one document to ready and returns its id.
fn ready_document(pipeline: &Pipeline, path: &Path) -> i64 {
    let id = pipeline
        .enqueue_files(&[path.to_path_buf()])
        .unwrap()
        .remove(0)
        .id;
    pipeline.run_until_idle().unwrap();
    assert_eq!(item_of(pipeline, id).status, QueueStatus::Ready);
    id
}

/// A document open in a program that shares it for reading only - a PDF in
/// Acrobat, a document in Word. While `held`, every rename is refused and so
/// is every open for deletion; plain reads still work.
#[derive(Default)]
struct HeldOpenFileSystem {
    held: AtomicBool,
}

impl HeldOpenFileSystem {
    fn refusal(&self) -> io::Result<()> {
        if self.held.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected hold: shared for reading only",
            ));
        }
        Ok(())
    }
}

impl FileSystem for HeldOpenFileSystem {
    fn exists(&self, path: &Path) -> bool {
        StdFileSystem.exists(path)
    }
    fn hash(&self, path: &Path) -> io::Result<String> {
        StdFileSystem.hash(path)
    }
    fn same_volume(&self, source: &Path, destination: &Path) -> io::Result<bool> {
        StdFileSystem.same_volume(source, destination)
    }
    fn rename_no_replace(&self, source: &Path, destination: &Path) -> io::Result<()> {
        self.refusal()?;
        StdFileSystem.rename_no_replace(source, destination)
    }
    fn copy_new_locked(
        &self,
        source: &Path,
        destination: &Path,
    ) -> io::Result<Box<dyn LockedFile>> {
        StdFileSystem.copy_new_locked(source, destination)
    }
    fn lock_for_delete(&self, path: &Path) -> io::Result<Box<dyn LockedFile>> {
        self.refusal()?;
        StdFileSystem.lock_for_delete(path)
    }
}

/// Approve & rename with the PDF still open in Acrobat, then again once it
/// is closed. The first attempt used to park the item with its receipt still
/// planned, and every later approve then failed for good.
#[test]
fn open_file_rename_failure_is_recoverable_by_approve_again() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "scan.pdf");
    let filesystem = Arc::new(HeldOpenFileSystem::default());
    let pipeline = reviewed_queue(temp.path(), filesystem.clone(), 1);
    let id = ready_document(&pipeline, &path);

    filesystem.held.store(true, Ordering::SeqCst);
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap_err();

    let refused = item_of(&pipeline, id);
    assert_eq!(refused.status, QueueStatus::NeedsReview);
    assert_eq!(
        refused.receipt.as_ref().unwrap().stage,
        OperationStage::RolledBack,
        "a rename that never happened is finished as rolled back, never left planned"
    );
    assert!(refused.unsettled_receipt.is_none());
    // The open document was read, not locked, to check it: it is unchanged,
    // and the reason is the refused rename.
    assert_ne!(refused.error_code, Some(ErrorCode::FileChanged));
    assert!(path.exists());

    filesystem.held.store(false, Ordering::SeqCst);
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();

    let filed = item_of(&pipeline, id);
    assert_eq!(filed.status, QueueStatus::Completed);
    let destination = filed.filed_receipt.unwrap().destination;
    assert_eq!(
        destination.file_name().and_then(|name| name.to_str()),
        Some(AGREEMENT_NAME)
    );
    assert_eq!(fs::read(&destination).unwrap(), b"scan.pdf");
    assert!(!path.exists());
}

/// What a parked rename left in databases written before parks could be
/// checked again: the row in review holding no receipt, and beside it a
/// receipt still planned that blocks every new one.
fn seed_orphaned_planned_receipt(database: &Path, item: &intern_queue::PipelineItem) {
    let connection = Connection::open(database).unwrap();
    connection
        .execute(
            "INSERT INTO operation_receipts(
               queue_item_id, direction, source_path, destination_path, pre_hash,
               operation_kind, stage, source_exists, destination_exists, temporary_exists,
               created_at, updated_at
             ) VALUES (?1, 'apply', ?2, ?3, ?4, 'rename', 'planned', 1, 0, 0,
                       unixepoch(), unixepoch())",
            rusqlite::params![
                item.id,
                item.source_path.to_string_lossy(),
                item.source_path
                    .with_file_name(AGREEMENT_NAME)
                    .to_string_lossy(),
                item.source_hash,
            ],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE queue_items SET status = 'needs_review', error_code = 'FILE_CHANGED'
             WHERE id = ?1",
            [item.id],
        )
        .unwrap();
}

#[test]
fn orphaned_planned_receipt_heals_on_check_again() {
    let temp = tempdir().unwrap();
    let database = temp.path().join("queue.sqlite3");
    let first = source(temp.path(), "first.pdf");
    let second = source(temp.path(), "second.pdf");
    let pipeline = reviewed_queue(temp.path(), Arc::new(StdFileSystem), 2);
    let ids = pipeline
        .enqueue_files(&[first.clone(), second.clone()])
        .unwrap();
    let (first_id, second_id) = (ids[0].id, ids[1].id);
    pipeline.run_until_idle().unwrap();
    for id in [first_id, second_id] {
        seed_orphaned_planned_receipt(&database, &item_of(&pipeline, id));
    }

    let parked = item_of(&pipeline, first_id);
    assert_eq!(parked.status, QueueStatus::NeedsReview);
    assert!(parked.unsettled_receipt.is_some());
    // Every action that would decide what the files are refuses, and names
    // the open question instead of failing on it later as a conflict.
    let refused = pipeline.keep_original(first_id).unwrap_err();
    assert_eq!(refused.code, "RECONCILIATION_REQUIRED");
    assert!(
        refused.message.contains("Check again"),
        "{}",
        refused.message
    );
    assert!(pipeline.remove(first_id, false).is_err());
    assert!(pipeline.cancel(first_id).is_err());

    // Retry on a parked item is Check again: the rename never happened.
    pipeline.retry(first_id).unwrap();
    let checked = item_of(&pipeline, first_id);
    assert_eq!(checked.status, QueueStatus::Ready);
    assert!(checked.unsettled_receipt.is_none());
    assert_eq!(checked.receipt.unwrap().stage, OperationStage::RolledBack);
    pipeline
        .approve(first_id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();
    let filed = item_of(&pipeline, first_id);
    assert_eq!(filed.status, QueueStatus::Completed);
    assert!(filed.filed_receipt.unwrap().destination.exists());
    assert!(!first.exists());

    // Approve checks the files first on its own.
    pipeline
        .approve(second_id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();
    let filed = item_of(&pipeline, second_id);
    assert_eq!(filed.status, QueueStatus::Completed);
    let destination = filed.filed_receipt.unwrap().destination;
    assert_eq!(fs::read(&destination).unwrap(), b"second.pdf");
    assert!(!second.exists());
}

/// A cross-volume copy that stops, once, with the temporary written and the
/// receipt still planned - the moment a recovery pass running beside the
/// apply used to roll the operation back and delete the copy under it.
struct BlockingCopyFileSystem {
    blocked: AtomicBool,
    entered: Barrier,
    release: Barrier,
}

impl FileSystem for BlockingCopyFileSystem {
    fn exists(&self, path: &Path) -> bool {
        StdFileSystem.exists(path)
    }
    fn hash(&self, path: &Path) -> io::Result<String> {
        StdFileSystem.hash(path)
    }
    fn same_volume(&self, _source: &Path, _destination: &Path) -> io::Result<bool> {
        Ok(false)
    }
    fn rename_no_replace(&self, source: &Path, destination: &Path) -> io::Result<()> {
        StdFileSystem.rename_no_replace(source, destination)
    }
    fn copy_new_locked(
        &self,
        source: &Path,
        destination: &Path,
    ) -> io::Result<Box<dyn LockedFile>> {
        let copied = StdFileSystem.copy_new_locked(source, destination)?;
        if !self.blocked.swap(true, Ordering::SeqCst) {
            self.entered.wait();
            self.release.wait();
        }
        Ok(copied)
    }
    fn lock_for_delete(&self, path: &Path) -> io::Result<Box<dyn LockedFile>> {
        StdFileSystem.lock_for_delete(path)
    }
}

#[test]
fn recover_waits_for_in_flight_apply() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "scan.pdf");
    let filesystem = Arc::new(BlockingCopyFileSystem {
        blocked: AtomicBool::new(false),
        entered: Barrier::new(2),
        release: Barrier::new(2),
    });
    let pipeline = Arc::new(reviewed_queue(temp.path(), filesystem.clone(), 1));
    let id = ready_document(&pipeline, &path);

    let approving = {
        let pipeline = Arc::clone(&pipeline);
        thread::spawn(move || pipeline.approve(id, AGREEMENT_NAME, "An employment agreement."))
    };
    filesystem.entered.wait();
    // The scheduler's recovery pass comes round while the copy is in flight.
    let recovering = {
        let pipeline = Arc::clone(&pipeline);
        thread::spawn(move || pipeline.recover())
    };
    thread::sleep(Duration::from_millis(100));
    assert!(
        !recovering.is_finished(),
        "recovery must wait for the operation in flight, not reconcile it"
    );
    filesystem.release.wait();
    approving.join().unwrap().unwrap();
    recovering.join().unwrap().unwrap();

    let filed = item_of(&pipeline, id);
    assert_eq!(filed.status, QueueStatus::Completed);
    assert_eq!(filed.receipt.unwrap().stage, OperationStage::Complete);
    assert!(!path.exists());
    let leftovers = fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".intern-tmp"))
        .collect::<Vec<_>>();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

/// Undo with the filed document open in another program, while more work
/// waits. The failed undo used to leave its row applying, and one applying
/// row stops the whole queue.
#[test]
fn failed_undo_reconciles_and_frees_the_queue() {
    let temp = tempdir().unwrap();
    let database = temp.path().join("queue.sqlite3");
    let path = source(temp.path(), "scan.pdf");
    let later = source(temp.path(), "later.pdf");
    let filesystem = Arc::new(HeldOpenFileSystem::default());
    let pipeline = reviewed_queue(temp.path(), filesystem.clone(), 1);
    let id = ready_document(&pipeline, &path);
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();
    let filed = item_of(&pipeline, id).filed_receipt.unwrap().destination;
    pipeline
        .enqueue_files(std::slice::from_ref(&later))
        .unwrap();

    filesystem.held.store(true, Ordering::SeqCst);
    pipeline.undo(id).unwrap_err();

    let after = item_of(&pipeline, id);
    assert_eq!(after.status, QueueStatus::Completed);
    assert_eq!(
        after.receipt.as_ref().unwrap().stage,
        OperationStage::RolledBack
    );
    assert!(
        after.filed_receipt.is_some(),
        "still filed, so still undoable"
    );
    assert!(filed.exists());
    assert!(!path.exists());
    // Nothing is left applying: the next document can be claimed at once.
    let busy = QueueStore::open(&database).unwrap();
    let claimed = busy.claim_next().unwrap().unwrap();
    assert_eq!(claimed.source_path, later);
    busy.transition(
        claimed.id,
        QueueStatus::Extracting,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();
    drop(busy);

    // And with the document closed, the undo the rollback left available
    // goes through.
    filesystem.held.store(false, Ordering::SeqCst);
    pipeline.undo(id).unwrap();

    let undone = item_of(&pipeline, id);
    assert_eq!(undone.status, QueueStatus::NeedsReview);
    assert!(
        undone
            .proposal
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == UNDONE)
    );
    assert!(path.exists());
    assert!(!filed.exists());
}

#[test]
fn undo_while_another_item_is_extracting() {
    let temp = tempdir().unwrap();
    let database = temp.path().join("queue.sqlite3");
    let path = source(temp.path(), "scan.pdf");
    let other = source(temp.path(), "other.pdf");
    let pipeline = reviewed_queue(temp.path(), Arc::new(StdFileSystem), 1);
    let id = ready_document(&pipeline, &path);
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();
    let filed = item_of(&pipeline, id).filed_receipt.unwrap().destination;

    // A backlog is draining: another document is being read.
    pipeline
        .enqueue_files(std::slice::from_ref(&other))
        .unwrap();
    let busy = QueueStore::open(&database).unwrap();
    let claimed = busy.claim_next().unwrap().unwrap();
    assert_eq!(claimed.status, QueueStatus::Extracting);

    pipeline.undo(id).unwrap();

    assert!(path.exists());
    assert!(!filed.exists());
    assert_eq!(item_of(&pipeline, id).status, QueueStatus::NeedsReview);
    assert_eq!(
        item_of(&pipeline, claimed.id).status,
        QueueStatus::Extracting
    );
    busy.transition(
        claimed.id,
        QueueStatus::Extracting,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();
}

#[test]
fn an_undo_of_a_filed_document_someone_moved_says_where_it_was() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "scan.pdf");
    let pipeline = reviewed_queue(temp.path(), Arc::new(StdFileSystem), 1);
    let id = ready_document(&pipeline, &path);
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();
    let filed = item_of(&pipeline, id).filed_receipt.unwrap().destination;
    fs::rename(&filed, temp.path().join("moved by hand.pdf")).unwrap();

    let error = pipeline.undo(id).unwrap_err();

    assert_eq!(
        error.message,
        format!(
            "The filed document is no longer at {}; it was moved or deleted.",
            display_path(&filed)
        )
    );
    assert_eq!(item_of(&pipeline, id).status, QueueStatus::Completed);
}

/// The first rename finds a file already at the destination name: a
/// teammate's document syncing in (`foreign`), or a copy of this very
/// document (not `foreign`). Later renames go through.
struct ArrivingFileSystem {
    foreign: bool,
    arrived: AtomicBool,
}

impl FileSystem for ArrivingFileSystem {
    fn exists(&self, path: &Path) -> bool {
        StdFileSystem.exists(path)
    }
    fn hash(&self, path: &Path) -> io::Result<String> {
        StdFileSystem.hash(path)
    }
    fn same_volume(&self, source: &Path, destination: &Path) -> io::Result<bool> {
        StdFileSystem.same_volume(source, destination)
    }
    fn rename_no_replace(&self, source: &Path, destination: &Path) -> io::Result<()> {
        if !self.arrived.swap(true, Ordering::SeqCst) {
            if self.foreign {
                fs::write(destination, b"a teammate's invoice")?;
            } else {
                fs::copy(source, destination)?;
            }
            return Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        StdFileSystem.rename_no_replace(source, destination)
    }
    fn copy_new_locked(
        &self,
        source: &Path,
        destination: &Path,
    ) -> io::Result<Box<dyn LockedFile>> {
        StdFileSystem.copy_new_locked(source, destination)
    }
    fn lock_for_delete(&self, path: &Path) -> io::Result<Box<dyn LockedFile>> {
        StdFileSystem.lock_for_delete(path)
    }
}

fn arriving(foreign: bool) -> Arc<ArrivingFileSystem> {
    Arc::new(ArrivingFileSystem {
        foreign,
        arrived: AtomicBool::new(false),
    })
}

#[test]
fn a_foreign_file_at_the_destination_waits_in_review_and_files_once_it_is_gone() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "scan.pdf");
    let pipeline = reviewed_queue(temp.path(), arriving(true), 1);
    let id = ready_document(&pipeline, &path);

    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap_err();

    let waiting = item_of(&pipeline, id);
    assert_eq!(waiting.status, QueueStatus::NeedsReview);
    assert_eq!(waiting.error_code, Some(ErrorCode::DestinationUnavailable));
    assert_eq!(waiting.receipt.unwrap().stage, OperationStage::RolledBack);
    assert!(waiting.unsettled_receipt.is_none());
    let occupied = temp.path().join(AGREEMENT_NAME);
    assert_eq!(fs::read(&occupied).unwrap(), b"a teammate's invoice");

    fs::remove_file(&occupied).unwrap();
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();

    let filed = item_of(&pipeline, id);
    assert_eq!(filed.status, QueueStatus::Completed);
    assert_eq!(filed.filed_receipt.unwrap().destination, occupied);
    assert_eq!(fs::read(&occupied).unwrap(), b"scan.pdf");
}

#[test]
fn confirmed_remove_of_parked_item() {
    let temp = tempdir().unwrap();
    let database = temp.path().join("queue.sqlite3");
    let path = source(temp.path(), "scan.pdf");
    let pipeline = reviewed_queue(temp.path(), arriving(false), 1);
    let id = ready_document(&pipeline, &path);
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap_err();
    let parked = item_of(&pipeline, id);
    assert_eq!(parked.status, QueueStatus::NeedsReview);
    assert_eq!(parked.error_code, Some(ErrorCode::ReconciliationRequired));
    assert!(parked.unsettled_receipt.is_some());

    // Without the person saying they resolved the files, it stays.
    assert_eq!(
        pipeline.remove(id, false).unwrap_err().code,
        "RECONCILIATION_REQUIRED"
    );
    assert_eq!(pipeline.list().unwrap().len(), 1);

    pipeline.remove(id, true).unwrap();

    assert!(pipeline.list().unwrap().is_empty());
    let receipts: i64 = Connection::open(&database)
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM operation_receipts WHERE queue_item_id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(receipts, 0, "receipts go with the row");
    // Removing the row decides nothing about the files.
    assert!(path.exists());
    assert!(temp.path().join(AGREEMENT_NAME).exists());
}

#[test]
fn a_backfilled_filing_is_dated_when_it_was_filed_not_when_it_was_last_saved() {
    let temp = tempdir().unwrap();
    let started = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let path = source(temp.path(), "contract.pdf");
    // Last saved on 1 January 2001. A rename does not change that.
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH + Duration::from_secs(978_307_200))
        .unwrap();
    let pipeline = reviewed_queue(temp.path(), Arc::new(StdFileSystem), 1);
    let id = ready_document(&pipeline, &path);
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();

    let filings = pipeline.filed_documents().unwrap();

    assert_eq!(filings.len(), 1);
    assert!(
        filings[0].filed_at >= started,
        "filed at {} - before the test began at {started}",
        filings[0].filed_at
    );
}

/// Two documents analyzed before either was filed both propose the same
/// name; the second is filed beside the first, and its item says so.
#[test]
fn two_documents_proposing_one_name_are_filed_apart() {
    let temp = tempdir().unwrap();
    let first = source(temp.path(), "first.pdf");
    let second = source(temp.path(), "second.pdf");
    let pipeline = reviewed_queue(temp.path(), Arc::new(StdFileSystem), 2);
    let ids = pipeline.enqueue_files(&[first, second]).unwrap();
    pipeline.run_until_idle().unwrap();
    for item in &ids {
        let proposed = item_of(&pipeline, item.id).proposal.unwrap().filename;
        assert_eq!(proposed, AGREEMENT_NAME);
        pipeline
            .approve(item.id, &proposed, "An employment agreement.")
            .unwrap();
    }

    let second = item_of(&pipeline, ids[1].id).filed_receipt.unwrap();
    assert_eq!(
        second
            .destination
            .file_name()
            .and_then(|name| name.to_str()),
        Some("2024-04-12 Employment Agreement between John Smith and Acme Corporation (2).pdf")
    );
}

/// A queue over fake files - fingerprints the test sets - reading `documents`
/// signed agreements, with automatic renaming off.
fn edited_queue(root: &Path, documents: usize) -> (Pipeline, Arc<FakeWorker>, Arc<FakeFiles>) {
    let worker = Arc::new(FakeWorker::new(
        (0..documents)
            .map(|_| Ok(parsed(SIGNED_AGREEMENT)))
            .collect(),
    ));
    let model = Arc::new(FakeModel::new(
        (0..documents).map(|_| Ok(proposal(0.94, false))).collect(),
    ));
    let files = Arc::new(FakeFiles::default());
    let pipeline = pipeline(
        root,
        Arc::clone(&worker),
        model,
        Arc::clone(&files),
        AppSettings::default(),
    );
    (pipeline, worker, files)
}

/// The document was signed after it was read. Approving it used to report
/// "Rename applied" while the item went back to review, and every approval
/// after that did the same.
#[test]
fn approve_after_edit_reports_file_changed() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "scan.pdf");
    let (pipeline, _worker, files) = edited_queue(temp.path(), 1);
    files.trust(&path, "unsigned");
    let id = ready_document(&pipeline, &path);
    files.trust(&path, "signed");

    let refused = pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap_err();

    assert_eq!(refused.code, "FILE_CHANGED");
    assert!(
        refused.message.contains("Re-analyze"),
        "{}",
        refused.message
    );
    let item = item_of(&pipeline, id);
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert!(
        item.proposal
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == "FILE_CHANGED")
    );
    assert!(files.applies.lock().unwrap().is_empty());
}

/// Re-analyze is the way on from FILE_CHANGED: the file is read again, as it
/// is now, and the rename that follows checks against the new fingerprint.
#[test]
fn reanalyze_file_changed_item_gives_fresh_proposal() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "scan.pdf");
    let (pipeline, worker, files) = edited_queue(temp.path(), 2);
    files.trust(&path, "unsigned");
    let id = ready_document(&pipeline, &path);
    files.trust(&path, "signed");
    pipeline
        .approve(id, AGREEMENT_NAME, "A reviewer's sentence.")
        .unwrap_err();
    assert_eq!(item_of(&pipeline, id).status, QueueStatus::NeedsReview);

    pipeline.reanalyze(id).unwrap();

    let queued = item_of(&pipeline, id);
    assert_eq!(queued.status, QueueStatus::Queued);
    assert_eq!(queued.source_hash, "signed");
    assert!(queued.proposal.is_none(), "the earlier reading is gone");

    pipeline.run_until_idle().unwrap();

    let fresh = item_of(&pipeline, id);
    assert_eq!(fresh.status, QueueStatus::Ready);
    assert_eq!(fresh.source_hash, "signed");
    assert_eq!(worker.calls.load(Ordering::SeqCst), 2);
    let record = fresh.proposal.unwrap();
    assert_eq!(record.revision, 1);
    assert!(!record.approved);
    assert!(record.reasons.is_empty(), "{:?}", record.reasons);
    assert_eq!(record.filename, AGREEMENT_NAME);
    assert_ne!(record.description, "A reviewer's sentence.");

    // And the rename now goes through against the new fingerprint.
    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();
    assert_eq!(files.applies.lock().unwrap().as_slice(), &[id]);
}

#[test]
fn reanalyze_is_refused_for_work_not_waiting_on_a_person() {
    let temp = tempdir().unwrap();
    let path = source(temp.path(), "scan.pdf");
    let (pipeline, _worker, files) = edited_queue(temp.path(), 1);
    files.trust(&path, "unsigned");
    let id = pipeline
        .enqueue_files(std::slice::from_ref(&path))
        .unwrap()
        .remove(0)
        .id;

    assert_eq!(
        pipeline.reanalyze(id).unwrap_err().code,
        "INVALID_TRANSITION",
        "still queued: there is nothing to read again yet"
    );
    assert_eq!(
        pipeline.reanalyze(id + 100).unwrap_err().code,
        "ITEM_NOT_FOUND"
    );
}

/// An archive already named the way Intern names documents, with no
/// destination: every file used to be proposed, and with automatic renaming
/// renamed, as "... (2)".
#[test]
fn already_named_source_is_completed_without_rename() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    let path = source(&inbox, AGREEMENT_NAME);
    let worker = Arc::new(FakeWorker::new(vec![Ok(parsed(SIGNED_AGREEMENT))]));
    let model = Arc::new(FakeModel::new(vec![Ok(proposal(0.94, false))]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings.save(&automatic_settings()).unwrap();
    let pipeline = Pipeline::with_local_files(
        temp.path().join("queue.sqlite3"),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    pipeline.enqueue_files(std::slice::from_ref(&path)).unwrap();

    pipeline.run_until_idle().unwrap();

    let item = pipeline.list().unwrap().pop().unwrap();
    let record = item.proposal.as_ref().unwrap();
    assert_eq!(record.filename, AGREEMENT_NAME, "no ' (2)'");
    assert_eq!(item.status, QueueStatus::Completed);
    assert!(
        record.reasons.iter().any(|reason| reason == ALREADY_NAMED),
        "{:?}",
        record.reasons
    );
    assert!(item.receipt.is_none(), "no file operation was journalled");
    assert!(path.exists());
    let names = std::fs::read_dir(&inbox)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(names, vec![AGREEMENT_NAME.to_owned()]);
}

/// A name that differs from the proposal only in case is the document's own
/// name to Windows: proposed without a suffix, and approved without a rename,
/// since this release makes no case-only renames.
#[test]
fn lowercase_named_source_is_not_suffixed() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    let lowercase = AGREEMENT_NAME.to_lowercase();
    let path = source(&inbox, &lowercase);
    let pipeline = reviewed_queue(temp.path(), Arc::new(StdFileSystem), 1);
    let id = ready_document(&pipeline, &path);

    assert_eq!(
        item_of(&pipeline, id).proposal.unwrap().filename,
        AGREEMENT_NAME
    );

    pipeline
        .approve(id, AGREEMENT_NAME, "An employment agreement.")
        .unwrap();

    let item = item_of(&pipeline, id);
    assert_eq!(item.status, QueueStatus::Completed);
    assert!(
        item.proposal
            .unwrap()
            .reasons
            .iter()
            .any(|reason| reason == ALREADY_NAMED)
    );
    assert!(path.exists(), "the file keeps the name it had");
    assert_eq!(std::fs::read_dir(&inbox).unwrap().count(), 1);
}

/// A name approved while the queue was busy waits, approved, to be filed. A
/// spelling rule learned in the meantime used to rebuild it from the facts,
/// which do not carry the date the reviewer typed, and the rebuilt name was
/// what the approval then filed - or, with no date left, sent back to review.
#[test]
fn approved_deferred_name_survives_rule_change() {
    let temp = tempdir().unwrap();
    let inbox = temp.path().join("inbox");
    let filed = temp.path().join("filed");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::create_dir_all(&filed).unwrap();
    let database = temp.path().join("queue.sqlite3");
    // A says no date at all, so the reviewer types one.
    let undated = "Employment Agreement between John Smith and Acme Corporation         covering duties, salary, and term.";
    let worker = Arc::new(FakeWorker::new(vec![
        Ok(parsed(undated)),
        Ok(dated_text("May 3, 2024")),
        Ok(dated_text("April 12, 2024")),
    ]));
    let model = Arc::new(FakeModel::new(vec![
        Ok(proposal(0.94, false)),
        Ok(dated_proposal("2024-05-03", "May 3, 2024")),
        Ok(dated_proposal("2024-04-12", "April 12, 2024")),
    ]));
    let settings = SettingsStore::new(temp.path().join("settings.json"));
    settings
        .save(&AppSettings {
            destination: filed.to_string_lossy().into_owned(),
            ..AppSettings::default()
        })
        .unwrap();
    let pipeline = Pipeline::with_local_files(
        database.clone(),
        worker,
        model,
        Arc::new(RecordingEvents::default()),
        settings,
    )
    .unwrap();
    let ids = pipeline
        .enqueue_files(&[
            source(&inbox, "a.pdf"),
            source(&inbox, "b.pdf"),
            source(&inbox, "c.pdf"),
        ])
        .unwrap()
        .into_iter()
        .map(|item| item.id)
        .collect::<Vec<_>>();
    let (a, b, c) = (ids[0], ids[1], ids[2]);
    pipeline.run_until_idle().unwrap();
    assert_eq!(
        record_of(&pipeline, a).filename,
        "Employment Agreement between John Smith and Acme Corporation.pdf"
    );
    // The respelling once, while the queue is free: a decision, not a rule.
    pipeline
        .approve(
            c,
            "2024-04-12 Employment Agreement between John Smith and Acme.pdf",
            "An employment agreement.",
        )
        .unwrap();
    assert!(!pipeline.learned_rules().unwrap()[0].active);

    // Another document is being read: approvals wait to be filed.
    pipeline.enqueue_files(&[source(&inbox, "d.pdf")]).unwrap();
    let busy = QueueStore::open(&database).unwrap();
    let claimed = busy.claim_next().unwrap().unwrap();
    let typed = "2024-03-01 Employment Agreement between John Smith and Acme Corporation.pdf";
    pipeline
        .approve(a, typed, "An employment agreement.")
        .unwrap();
    assert_eq!(item_of(&pipeline, a).status, QueueStatus::Ready);
    // The same respelling a second time makes it a rule.
    pipeline
        .approve(
            b,
            "2024-05-03 Employment Agreement between John Smith and Acme.pdf",
            "An employment agreement.",
        )
        .unwrap();
    assert!(pipeline.learned_rules().unwrap()[0].active);
    assert_eq!(record_of(&pipeline, a).filename, typed);

    busy.transition(
        claimed.id,
        QueueStatus::Extracting,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();
    drop(busy);
    pipeline.run_until_idle().unwrap();

    let filed_a = item_of(&pipeline, a);
    assert_eq!(filed_a.status, QueueStatus::Completed);
    assert_eq!(
        filed_a.filed_receipt.unwrap().destination,
        filed.join(typed)
    );
    assert_eq!(item_of(&pipeline, b).status, QueueStatus::Completed);
}
