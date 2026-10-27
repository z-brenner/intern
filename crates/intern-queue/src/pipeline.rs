use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicI64, Ordering},
    },
};

use intern_core::{
    ErrorCode, FileApplier, FileSystem, InternError, OperationDirection, OperationReceipt,
    OperationStage, QueueItem, QueueStatus, QueueStore, ReviewMark, StdFileSystem, copy_name,
    crash_doubt_kind, source_path_key,
};
use intern_engine::{
    DocumentAnalysis, DocumentSource, ExtractProgress, HouseRule, HouseStyle, ProposalStatus,
    RuleKind, ValidatedProposal, compose_styled_filename, counterparty_view,
    evidence::is_valid_iso_date,
    fingerprint::{self, NEAR_DUPLICATE_DISTANCE},
    lesson_from_edit, sanitize_folder_name,
    setup::{semantic_probes, validate_semantic_probe},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::admission::{AdmissionEvidence, AdmissionGuard, AdmissionStage, LocalAdmission};
use crate::paths::display_path;
use crate::settings::{AppSettings, DestinationLayout, SettingsStore, normalize_own_names};

const LEASE_RENEWAL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(20);
const LEASE_RENEWAL_ATTEMPTS: usize = 3;

pub struct LeaseKeeper {
    stop: Arc<(Mutex<bool>, Condvar)>,
    failure: Arc<Mutex<Option<PipelineError>>>,
    join: Option<std::thread::JoinHandle<()>>,
}

impl LeaseKeeper {
    pub fn start(
        store: Arc<QueueStore>,
        item_id: i64,
        interval: std::time::Duration,
    ) -> PipelineResult<Self> {
        Self::start_with_cancel(store, item_id, interval, Arc::new(|| {}))
    }

    fn start_with_cancel(
        store: Arc<QueueStore>,
        item_id: i64,
        interval: std::time::Duration,
        cancel: Arc<dyn Fn() + Send + Sync>,
    ) -> PipelineResult<Self> {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let failure = Arc::new(Mutex::new(None));
        let thread_stop = Arc::clone(&stop);
        let thread_failure = Arc::clone(&failure);
        let join = std::thread::Builder::new()
            .name(format!("intern-lease-{item_id}"))
            .spawn(move || {
                let (lock, wake) = &*thread_stop;
                let mut stopped = match lock.lock() {
                    Ok(value) => value,
                    Err(_) => return,
                };
                loop {
                    if *stopped {
                        return;
                    }
                    let waited = match wake.wait_timeout(stopped, interval) {
                        Ok(value) => value,
                        Err(_) => return,
                    };
                    stopped = waited.0;
                    if *stopped {
                        return;
                    }
                    drop(stopped);
                    let mut renewed = false;
                    for attempt in 0..LEASE_RENEWAL_ATTEMPTS {
                        match store.renew_lease(item_id) {
                            Ok(_) => {
                                renewed = true;
                                break;
                            }
                            Err(error) => {
                                let terminal = error.code() == ErrorCode::StateConflict
                                    || attempt + 1 == LEASE_RENEWAL_ATTEMPTS;
                                if terminal {
                                    if let Ok(mut failure) = thread_failure.lock() {
                                        *failure = Some(PipelineError::from(error));
                                    }
                                    cancel();
                                    return;
                                }
                                let retry_delay = interval.min(std::time::Duration::from_secs(2));
                                stopped = match lock.lock() {
                                    Ok(value) => value,
                                    Err(_) => return,
                                };
                                let waited = match wake.wait_timeout(stopped, retry_delay) {
                                    Ok(value) => value,
                                    Err(_) => return,
                                };
                                stopped = waited.0;
                                if *stopped {
                                    return;
                                }
                                drop(stopped);
                            }
                        }
                    }
                    if !renewed {
                        return;
                    }
                    stopped = match lock.lock() {
                        Ok(value) => value,
                        Err(_) => return,
                    };
                }
            })
            .map_err(|_| {
                PipelineError::new("STATE_CONFLICT", "lease renewal thread could not start")
            })?;
        Ok(Self {
            stop,
            failure,
            join: Some(join),
        })
    }

    pub fn check(&self) -> PipelineResult<()> {
        match self.failure.lock() {
            Ok(failure) => failure.clone().map_or(Ok(()), Err),
            Err(_) => Err(PipelineError::new(
                "STATE_CONFLICT",
                "lease renewal state is unavailable",
            )),
        }
    }

    pub fn stop_and_check(mut self) -> PipelineResult<()> {
        self.stop_thread()?;
        self.check()
    }

    fn stop_thread(&mut self) -> PipelineResult<()> {
        let (lock, wake) = &*self.stop;
        let mut stopped = lock.lock().map_err(|_| {
            PipelineError::new("STATE_CONFLICT", "lease renewal stop state is unavailable")
        })?;
        *stopped = true;
        wake.notify_all();
        drop(stopped);
        if self.join.take().is_some_and(|join| join.join().is_err()) {
            return Err(PipelineError::new(
                "STATE_CONFLICT",
                "lease renewal thread did not terminate cleanly",
            ));
        }
        Ok(())
    }
}

impl Drop for LeaseKeeper {
    fn drop(&mut self) {
        let _ = self.stop_thread();
    }
}

pub const PARSER_TIMEOUT_SECONDS: u64 = 30 * 60;
pub const MODEL_TIMEOUT_SECONDS: u64 = 15 * 60;

/// How long a request that has already missed its deadline is given to
/// notice its cancel before the queue stops waiting for it.
const MODEL_CANCEL_GRACE: std::time::Duration = std::time::Duration::from_secs(30);

/// The extraction boundary, as the queue sees it.
pub use intern_engine::DocumentExtractor as WorkerBoundary;
/// Extraction failures, as the queue sees them.
pub use intern_engine::ExtractFailure as WorkerFailure;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipelineError {
    pub code: String,
    pub message: String,
    retryable: bool,
}

impl PipelineError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable: false,
        }
    }

    pub fn retryable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable: true,
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.retryable
    }
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PipelineError {}
pub type PipelineResult<T> = Result<T, PipelineError>;

impl From<InternError> for PipelineError {
    fn from(error: InternError) -> Self {
        Self::new(error.code().as_str(), error.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelFailure {
    pub code: String,
    pub retryable: bool,
}

impl ModelFailure {
    pub fn retryable(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            retryable: true,
        }
    }
    pub fn fatal(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            retryable: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineProgress {
    pub item_id: i64,
    pub stage: String,
    pub current: usize,
    pub total: Option<usize>,
}

/// The document-understanding boundary, as the queue sees it.
///
/// The queue knows nothing about distillation, prompts, or models; it hands
/// over extracted pages and receives a finished proposal. Swapping the engine
/// out, or driving it from a CLI or a watched folder instead, changes nothing
/// on this side of the line.
pub trait AnalyzerBoundary: Send + Sync {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure>;
    fn recover(&self, _failure: &ModelFailure) -> Result<(), ModelFailure> {
        Ok(())
    }
    fn cancel(&self) -> Result<(), ModelFailure> {
        Err(ModelFailure::fatal("MODEL_CANCEL_UNAVAILABLE"))
    }
    fn shutdown(&self) -> Result<(), ModelFailure> {
        Ok(())
    }
}

pub trait FileActions: Send + Sync {
    fn fingerprint(&self, path: &Path) -> PipelineResult<String>;
    fn apply(&self, item: &QueueItem, destination: &Path) -> PipelineResult<()>;
    fn undo(&self, item: &QueueItem, receipt: &OperationReceipt) -> PipelineResult<()>;
    fn reconcile(&self, item: &QueueItem) -> PipelineResult<()>;
    /// Whether `path` exists, or the error that kept that from being known:
    /// what a failed undo asks before telling a person where their document
    /// is.
    fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
        path.try_exists()
    }
    /// Renames the file of an item in review because it could not be read
    /// to `NEEDS REVIEW - <its name>`, where it lies, and moves the row
    /// with it. Nothing, by default: a boundary that keeps no files has no
    /// name to change.
    fn mark_for_review(&self, _item_id: i64) -> PipelineResult<()> {
        Ok(())
    }
    /// Takes that prefix back off and completes the item as kept, when a
    /// person keeps the document's own name. `false` when the file carries
    /// no mark of Intern's, and keeping it renames nothing.
    fn restore_from_review(&self, _item_id: i64) -> PipelineResult<bool> {
        Ok(false)
    }
    /// Settles the renames in place a crash left half-recorded, and marks
    /// the documents whose mark never happened.
    fn reconcile_review_renames(&self) -> PipelineResult<()> {
        Ok(())
    }
    /// Gives a canceled or failed item's file its own name back, when it
    /// carries a NEEDS REVIEW mark Intern made.
    fn unmark_settled(&self, _item_id: i64) -> PipelineResult<()> {
        Ok(())
    }
}

pub struct CoreFileActions {
    store: Arc<QueueStore>,
    applier: FileApplier,
}

impl CoreFileActions {
    pub fn local(store: Arc<QueueStore>) -> Self {
        Self::with_file_system(store, Arc::new(StdFileSystem))
    }

    /// The journalled file operations over `filesystem` rather than the
    /// machine's own: what lets a held file, a refused rename or a slow copy
    /// be reproduced on any platform, against the queue's own store.
    pub fn with_file_system(store: Arc<QueueStore>, filesystem: Arc<dyn FileSystem>) -> Self {
        Self {
            applier: FileApplier::new(filesystem, Arc::clone(&store)),
            store,
        }
    }
}

impl FileActions for CoreFileActions {
    fn fingerprint(&self, path: &Path) -> PipelineResult<String> {
        self.applier.fingerprint(path).map_err(Into::into)
    }

    fn apply(&self, item: &QueueItem, destination: &Path) -> PipelineResult<()> {
        self.store
            .begin_applying(item.id, QueueStatus::Ready)
            .map_err(|error| match error.code() {
                // The queue is working on another document, so the rename is
                // early rather than wrong. Saying so distinctly is what lets
                // the caller wait instead of blaming the document.
                ErrorCode::StateConflict => PipelineError::new(
                    APPLY_DEFERRED,
                    "another document is being processed; the rename waits for the queue",
                ),
                _ => error.into(),
            })?;
        let lease = LeaseKeeper::start(Arc::clone(&self.store), item.id, LEASE_RENEWAL_INTERVAL)?;
        let result = self
            .applier
            .apply(item.id, &item.source_path, destination, &item.source_hash);
        lease.stop_and_check()?;
        match result {
            Ok(receipt) => {
                self.store.complete_apply(item.id, receipt.id)?;
                Ok(())
            }
            Err(error) => {
                if error.receipt().is_none() {
                    let _ = self.applier.reconcile(item.id);
                }
                Err(error.into())
            }
        }
    }

    fn undo(&self, item: &QueueItem, receipt: &OperationReceipt) -> PipelineResult<()> {
        self.store.begin_undo(item.id)?;
        let lease = LeaseKeeper::start(Arc::clone(&self.store), item.id, LEASE_RENEWAL_INTERVAL)?;
        let result = self.applier.undo(item.id, receipt);
        lease.stop_and_check()?;
        match result {
            Ok(undo_receipt) => {
                self.store.complete_undo(item.id, undo_receipt.id)?;
                Ok(())
            }
            Err(error) => {
                if error.receipt().is_none() {
                    let _ = self.applier.reconcile(item.id);
                }
                Err(error.into())
            }
        }
    }

    fn reconcile(&self, item: &QueueItem) -> PipelineResult<()> {
        self.applier
            .reconcile(item.id)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn try_exists(&self, path: &Path) -> std::io::Result<bool> {
        self.applier.try_exists(path)
    }

    fn mark_for_review(&self, item_id: i64) -> PipelineResult<()> {
        self.applier
            .mark_for_review(item_id)
            .map(drop)
            .map_err(Into::into)
    }

    fn restore_from_review(&self, item_id: i64) -> PipelineResult<bool> {
        self.applier
            .restore_from_review(item_id)
            .map(|kept| kept.is_some())
            .map_err(Into::into)
    }

    fn reconcile_review_renames(&self) -> PipelineResult<()> {
        self.applier
            .reconcile_review_renames()
            .map(drop)
            .map_err(Into::into)
    }

    fn unmark_settled(&self, item_id: i64) -> PipelineResult<()> {
        self.applier
            .unmark_settled(item_id)
            .map(drop)
            .map_err(Into::into)
    }
}

pub trait PipelineEventSink: Send + Sync {
    fn queue_changed(&self);
    fn progress(&self, progress: PipelineProgress);
}

/// A document the queue has just filed, as reported to whoever keeps records
/// beside filed documents (the description ledger and the shared filed index,
/// in the desktop app).
#[derive(Clone, Debug, PartialEq)]
pub struct FiledDocument {
    pub item_id: i64,
    /// Where the document was before the rename.
    pub source_path: PathBuf,
    /// The SHA-256 of the document's bytes, lowercase hex - the fingerprint
    /// the rename was verified against.
    pub source_hash: String,
    /// Where it is now - the receipt's destination, suffix and all.
    pub destination: PathBuf,
    /// The sentence that was applied: the model's, or the reviewer's edit.
    pub description: String,
    /// The validated facts behind the name.
    pub proposal: ValidatedProposal,
    /// Unix seconds when the apply completed.
    pub filed_at: i64,
    /// The text fingerprint the analysis carried, for a near-duplicate check
    /// on other machines. Absent for a text too short to fingerprint.
    pub text_fingerprint: Option<String>,
}

/// A filing the queue has just undone: the document is back at
/// `source_path`, and `destination` is empty again.
#[derive(Clone, Debug, PartialEq)]
pub struct UnfiledDocument {
    pub item_id: i64,
    pub source_path: PathBuf,
    pub source_hash: String,
    pub destination: PathBuf,
}

/// Where the queue reports filed documents to.
///
/// Filing has already succeeded when `filed` is called and cannot be undone
/// by it: an implementation that fails records its own failure and says so
/// elsewhere, and the rename stands. `unfiled` is the mirror image, called
/// after an undo has put the document back.
pub trait FilingSink: Send + Sync {
    fn filed(&self, document: &FiledDocument);
    fn unfiled(&self, document: &UnfiledDocument);
}

/// The default: nobody is listening.
struct NoFilingSink;

impl FilingSink for NoFilingSink {
    fn filed(&self, _document: &FiledDocument) {}
    fn unfiled(&self, _document: &UnfiledDocument) {}
}

/// Several listeners behind one sink, told in order. A sink cannot fail, so
/// none of them can keep the others from hearing.
pub struct FilingSinks(pub Vec<Arc<dyn FilingSink>>);

impl FilingSink for FilingSinks {
    fn filed(&self, document: &FiledDocument) {
        for sink in &self.0 {
            sink.filed(document);
        }
    }

    fn unfiled(&self, document: &UnfiledDocument) {
        for sink in &self.0 {
            sink.unfiled(document);
        }
    }
}

/// A filing the queue has no record of: one made by another machine sharing
/// an intake folder, or one this machine made before its history was cleared.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownFiling {
    /// The name the content was filed under.
    pub filename: String,
    /// The machine that filed it, when it was not this one.
    pub filed_by: Option<String>,
}

impl KnownFiling {
    /// How the queue names the filing to a person: the filename, and the
    /// machine when there is one to name.
    pub fn describe(&self) -> String {
        match &self.filed_by {
            Some(machine) => format!("{} (filed from {machine})", self.filename),
            None => self.filename.clone(),
        }
    }
}

/// Where the queue asks whether a document's content has already been filed
/// somewhere its own history cannot see. Asked once per enqueued document,
/// before any analysis; a positive answer files the document as a copy named
/// after that filing (`COPY - <filename>`), or, with no output folder to file
/// it into, routes it to review as a duplicate, where "process anyway" is one
/// click.
pub trait DuplicateOracle: Send + Sync {
    fn filed_elsewhere(&self, source_hash: &str, source_path: &Path) -> Option<KnownFiling>;
    /// A filing whose text fingerprint is within [`NEAR_DUPLICATE_DISTANCE`]
    /// of `fingerprint`: the closest one, when there is one.
    fn similar_elsewhere(&self, _fingerprint: u64) -> Option<SimilarFiling> {
        None
    }
}

/// A filing whose text is nearly the text of the document at hand.
#[derive(Clone, Debug, PartialEq)]
pub struct SimilarFiling {
    pub filing: KnownFiling,
    /// How many fingerprint bits apart the two texts are.
    pub distance: u32,
}

/// The default: the queue's own history is all there is.
struct NoDuplicateOracle;

impl DuplicateOracle for NoDuplicateOracle {
    fn filed_elsewhere(&self, _source_hash: &str, _source_path: &Path) -> Option<KnownFiling> {
        None
    }
}

/// What the queue stores about one proposal.
///
/// `analysis` is exactly what the engine produced and never changes; `filename`
/// and `description` are what will actually be applied, and a human edit
/// changes only those. Keeping them apart means the evidence a reviewer sees
/// still belongs to the model's own answer.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalRecord {
    pub analysis: DocumentAnalysis,
    pub status: ProposalStatus,
    pub filename: String,
    pub description: String,
    pub reasons: Vec<String>,
    pub revision: u64,
    /// The reviewer's spellings applied when `filename` was composed. The
    /// analysis keeps the document's own words; these say how the name
    /// differs from them, and why.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub house_rules: Vec<HouseRule>,
    /// The person's own organisation as Settings named it when `filename`
    /// was composed. A party it names is left out of the name, so the name
    /// and the Party folder carry the other side. Kept with the record so the
    /// name can be read back exactly as it was composed, whatever Settings
    /// says by then.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub own_names: Vec<String>,
    /// The name a document with nearly this text was already filed under -
    /// a second scan, a re-export, a copy saved again - when there is one.
    /// Such a document waits for a person rather than being filed twice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub near_duplicate_of: Option<String>,
    /// Whether a person has approved this name. An approval the queue was too
    /// busy to act on at once waits here, so the scheduler files the document
    /// when it is free even with automatic renaming switched off.
    #[serde(default)]
    pub approved: bool,
    /// A name a person typed for a document Intern could not read. There
    /// was no analysis to propose one, so `analysis` is empty: nothing here
    /// is the model's, and nothing recomposes the name from it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub manual: bool,
    /// Set on the record of an exact duplicate filed as a copy of the
    /// document it repeats. `filename` is the copy's name; there was no
    /// analysis, and the filing is no filing of its own (see
    /// [`CopyOf`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy_of: Option<CopyOf>,
}

/// What an exact duplicate is a copy of, and where its copy goes.
///
/// A document whose bytes are already filed is not read or named again: it
/// is moved out of the input folder into the output folder as
/// `COPY - <the original's filed name>`, beside the original when Intern
/// knows its folder. Such a filing leaves no record of its own - no
/// description, no shared filed marker, no text fingerprint - because the
/// document it repeats already has them, and no later duplicate is ever
/// named after a copy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyOf {
    /// The name the original is filed under.
    pub original: String,
    /// The folder the copy is filed into.
    pub folder: PathBuf,
}

/// The review reason for a document whose text is nearly the text of one
/// already filed. The record's `near_duplicate_of` names that filing.
pub const NEAR_DUPLICATE: &str = "NEAR_DUPLICATE";

/// The note on an exact duplicate filed as a copy of the document it
/// repeats: why it is named `COPY - ...` and was never read.
pub const EXACT_COPY: &str = "EXACT_COPY";

/// The review reason for a rename a person took back. The document is where
/// it started and waits for a decision; nothing files it again on its own.
pub const UNDONE: &str = "UNDONE";

/// Recorded when the document already carried exactly the name Intern would
/// give it, so filing it completed without touching the file.
pub const ALREADY_NAMED: &str = "ALREADY_NAMED";

impl ProposalRecord {
    /// A record with no analysis behind it, for `filename`: a name typed by
    /// hand, or a copy's.
    fn synthetic(filename: String, description: String) -> Self {
        Self {
            analysis: DocumentAnalysis {
                filename: String::new(),
                description: String::new(),
                status: ProposalStatus::NeedsReview,
                review_reasons: Vec::new(),
                proposal: ValidatedProposal::default(),
                telemetry: Default::default(),
                model_proposal: None,
                stated_dates: Vec::new(),
                text_fingerprint: None,
                token_confidence: None,
                facts: None,
            },
            status: ProposalStatus::NeedsReview,
            filename,
            description,
            reasons: Vec::new(),
            revision: 1,
            house_rules: Vec::new(),
            own_names: Vec::new(),
            near_duplicate_of: None,
            approved: false,
            manual: false,
            copy_of: None,
        }
    }

    /// Whether the record has no analysis behind it - a name typed for an
    /// unreadable document, or a copy's - so there is nothing to recompose
    /// its name from, and no confidence or evidence to show.
    pub fn is_synthetic(&self) -> bool {
        self.manual || self.copy_of.is_some()
    }

    /// The validated facts as the name carries them: the document's words,
    /// respelled the way the reviewer has taught Intern to, with the
    /// person's own organisation left out where the document names someone
    /// else too. The one view the filename, the layout folder, a lesson
    /// from an edit, and a filing report are all read from.
    pub fn styled_proposal(&self) -> ValidatedProposal {
        self.name_view().0
    }

    /// [`Self::styled_proposal`], and the parties it left out because they
    /// are the person's own organisation.
    pub fn name_view(&self) -> (ValidatedProposal, Vec<String>) {
        let styled = HouseStyle::new(self.house_rules.clone())
            .apply(&self.analysis.proposal)
            .0;
        counterparty_view(&styled, &self.own_names)
    }
}

/// How many times the same respelling must be made in review before Intern
/// applies it on its own. One edit is a decision about one document; the
/// second is a preference.
pub const EDITS_TO_LEARN: u32 = 2;

/// A spelling Intern has learned from review, and how settled it is.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnedRule {
    pub id: i64,
    pub kind: RuleKind,
    pub from: String,
    pub to: String,
    /// How many times a reviewer has made exactly this change.
    pub seen: u32,
    /// Whether Intern applies it: made often enough, or told to use it now.
    pub active: bool,
    /// Unix seconds of the latest edit that taught it.
    pub learned_at: i64,
}

impl LearnedRule {
    pub fn rule(&self) -> HouseRule {
        HouseRule::new(self.kind, self.from.clone(), self.to.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PipelineItem {
    pub id: i64,
    pub source_path: PathBuf,
    pub source_hash: String,
    pub status: QueueStatus,
    pub processing_failures: u32,
    pub error_code: Option<ErrorCode>,
    pub proposal: Option<ProposalRecord>,
    /// The newest receipt of any stage: the operation reconciliation is
    /// about, whether or not it finished.
    pub receipt: Option<OperationReceipt>,
    /// The completed apply the document is filed by, when its newest finished
    /// operation is one. An undo that was refused and rolled back leaves this
    /// in place; an undo that finished takes it away. This, not `receipt`, is
    /// what says where the document is and whether it can be undone.
    pub filed_receipt: Option<OperationReceipt>,
    /// An operation that never finished, on an item that is not applying: one
    /// a reconciliation could not settle. Until it is checked again, no new
    /// rename of the document can be journalled.
    pub unsettled_receipt: Option<OperationReceipt>,
    /// For an item flagged DUPLICATE: the name its content is already filed
    /// under (the completed apply's destination leaf, or the completed item's
    /// original filename for keep-original completions). `None` once the
    /// completed row is gone, e.g. after the history was cleared.
    pub duplicate_of: Option<String>,
}

/// What adding a batch of documents did, document by document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnqueueReport {
    /// Rows this batch created, in the order given.
    pub added: Vec<QueueItem>,
    /// Documents whose path already held these exact bytes in the queue.
    pub already_queued: usize,
    /// Documents that could not be admitted, each with the code of the
    /// failure: `SOURCE_LOCKED` for a file another program holds open,
    /// `IO_ERROR` for one that cannot be read, and so on.
    pub skipped: Vec<(PathBuf, String)>,
}

pub struct Pipeline {
    store: Arc<QueueStore>,
    repository: PipelineRepository,
    worker: Arc<dyn WorkerBoundary>,
    model: Arc<dyn AnalyzerBoundary>,
    files: Arc<dyn FileActions>,
    events: Arc<dyn PipelineEventSink>,
    filing: Arc<dyn FilingSink>,
    duplicates: Arc<dyn DuplicateOracle>,
    admission: Arc<dyn AdmissionGuard>,
    settings: SettingsStore,
    paused: AtomicBool,
    /// Why the queue stopped itself, as a code the window has a sentence
    /// for; `None` when it is running or a person paused it.
    pause_reason: Mutex<Option<String>>,
    /// When a document first found no model loaded to ask, in the current
    /// run of such documents; `None` once a model has answered since.
    model_missing_since: Mutex<Option<std::time::Instant>>,
    /// How long no model may be loaded before the queue stops for it.
    model_missing_grace: std::time::Duration,
    active_item: AtomicI64,
    /// Set when the active item's model request is canceled - by a person or
    /// by its deadline - before the model itself is told. The request thread
    /// reads it to tell a failure the cancel caused from one worth a retry.
    active_cancel: Mutex<Option<Arc<AtomicBool>>>,
    shutting_down: AtomicBool,
    model_timeout: std::time::Duration,
    lease_renewal_interval: std::time::Duration,
    run_lock: Mutex<()>,
    /// Held around every file operation and the reconciliation that follows
    /// it. Approve, undo, retry and the recovery pass run on different
    /// threads with the same store session, so every ownership check of one
    /// passed against another still in flight: recovery could record a
    /// rename as rolled back just before it landed, or delete a temporary
    /// copy still being verified. Taken after `run_lock`, never before it,
    /// and never held twice.
    file_ops: Mutex<()>,
    /// Intern's own data folder - the queue database's - where the reader's
    /// self-test writes its file.
    data_folder: PathBuf,
}

/// Which waiting renames one scheduler pass applies.
#[derive(Clone, Copy, Eq, PartialEq)]
enum ReadyScope {
    /// Everything owed: every ready document when automatic renaming is on,
    /// and every approval still waiting. What opens a drain.
    Everything,
    /// Only the approvals the queue was too busy to act on when they were
    /// made. What runs between documents.
    ApprovalsOnly,
}

#[derive(Clone, Copy)]
enum LeasePhase {
    Extracting,
    Analyzing,
}

impl Pipeline {
    pub fn open(
        database: impl AsRef<Path>,
        worker: Arc<dyn WorkerBoundary>,
        model: Arc<dyn AnalyzerBoundary>,
        files: Arc<dyn FileActions>,
        events: Arc<dyn PipelineEventSink>,
        settings: SettingsStore,
    ) -> PipelineResult<Self> {
        let database = database.as_ref();
        let store = Arc::new(QueueStore::open(database)?);
        let repository = PipelineRepository::open(database)?;
        Ok(Self::assemble(
            store,
            repository,
            worker,
            model,
            files,
            events,
            settings,
            database.parent().unwrap_or(Path::new(".")).to_path_buf(),
        ))
    }

    pub fn with_local_files(
        database: impl AsRef<Path>,
        worker: Arc<dyn WorkerBoundary>,
        model: Arc<dyn AnalyzerBoundary>,
        events: Arc<dyn PipelineEventSink>,
        settings: SettingsStore,
    ) -> PipelineResult<Self> {
        Self::with_file_system(
            database,
            worker,
            model,
            events,
            settings,
            Arc::new(StdFileSystem),
        )
    }

    /// The queue with its real journalled file operations, over `filesystem`.
    ///
    /// The file operations share the queue's own store, as they do in the
    /// app: their ownership checks run against the same session as approve,
    /// undo and recovery, which is exactly where those calls used to race.
    pub fn with_file_system(
        database: impl AsRef<Path>,
        worker: Arc<dyn WorkerBoundary>,
        model: Arc<dyn AnalyzerBoundary>,
        events: Arc<dyn PipelineEventSink>,
        settings: SettingsStore,
        filesystem: Arc<dyn FileSystem>,
    ) -> PipelineResult<Self> {
        let database = database.as_ref();
        let store = Arc::new(QueueStore::open(database)?);
        let repository = PipelineRepository::open(database)?;
        let files = Arc::new(CoreFileActions::with_file_system(
            Arc::clone(&store),
            filesystem,
        ));
        Ok(Self::assemble(
            store,
            repository,
            worker,
            model,
            files,
            events,
            settings,
            database.parent().unwrap_or(Path::new(".")).to_path_buf(),
        ))
    }

    fn assemble(
        store: Arc<QueueStore>,
        repository: PipelineRepository,
        worker: Arc<dyn WorkerBoundary>,
        model: Arc<dyn AnalyzerBoundary>,
        files: Arc<dyn FileActions>,
        events: Arc<dyn PipelineEventSink>,
        settings: SettingsStore,
        data_folder: PathBuf,
    ) -> Self {
        Self {
            store,
            repository,
            worker,
            model,
            files,
            events,
            filing: Arc::new(NoFilingSink),
            duplicates: Arc::new(NoDuplicateOracle),
            admission: Arc::new(LocalAdmission),
            settings,
            paused: AtomicBool::new(false),
            pause_reason: Mutex::new(None),
            model_missing_since: Mutex::new(None),
            model_missing_grace: MODEL_MISSING_GRACE,
            active_item: AtomicI64::new(0),
            active_cancel: Mutex::new(None),
            shutting_down: AtomicBool::new(false),
            model_timeout: std::time::Duration::from_secs(MODEL_TIMEOUT_SECONDS),
            lease_renewal_interval: LEASE_RENEWAL_INTERVAL,
            run_lock: Mutex::new(()),
            file_ops: Mutex::new(()),
            data_folder,
        }
    }

    /// Reports every completed rename (and every undo of one) to `sink`.
    #[must_use]
    pub fn with_filing_sink(mut self, sink: Arc<dyn FilingSink>) -> Self {
        self.filing = sink;
        self
    }

    /// Asks `oracle` about every enqueued document whose content the queue's
    /// own history has not already filed.
    #[must_use]
    pub fn with_duplicate_oracle(mut self, oracle: Arc<dyn DuplicateOracle>) -> Self {
        self.duplicates = oracle;
        self
    }

    #[must_use]
    pub fn with_admission_guard(mut self, guard: Arc<dyn AdmissionGuard>) -> Self {
        self.admission = guard;
        self
    }

    #[doc(hidden)]
    pub fn with_model_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.model_timeout = timeout;
        self
    }

    #[doc(hidden)]
    pub fn with_lease_renewal_interval(mut self, interval: std::time::Duration) -> Self {
        self.lease_renewal_interval = interval;
        self
    }

    #[doc(hidden)]
    pub fn with_model_missing_grace(mut self, grace: std::time::Duration) -> Self {
        self.model_missing_grace = grace;
        self
    }

    pub fn enqueue_files(&self, paths: &[PathBuf]) -> PipelineResult<Vec<QueueItem>> {
        let mut queued = Vec::with_capacity(paths.len());
        for path in paths {
            let fingerprint = self.admit_for_enqueue(path)?;
            queued.push(self.enqueue_admitted(path, &fingerprint)?.0);
        }
        if !queued.is_empty() {
            self.events.queue_changed();
        }
        Ok(queued)
    }

    /// Adds documents a person chose, one at a time, and says what became of
    /// each.
    ///
    /// `enqueue_files` stops at the first document it cannot admit, after the
    /// ones before it are already in the queue - right for the intake
    /// watcher, which hands over one path at a time, and wrong for a person
    /// adding a folder of two hundred: one file a scanner is still writing, or
    /// one online-only placeholder that cannot be read offline, silently
    /// dropped the rest and announced nothing. Here a document that cannot be
    /// admitted or fingerprinted is set aside with its code and the rest go
    /// on.
    ///
    /// A failure of the queue database is not about one document, so it still
    /// ends the batch; what was added before it is announced first, so the
    /// window shows it either way.
    pub fn enqueue_files_report(&self, paths: &[PathBuf]) -> PipelineResult<EnqueueReport> {
        let mut report = EnqueueReport::default();
        let result = self.enqueue_reporting_into(paths, &mut report);
        if !report.added.is_empty() {
            self.events.queue_changed();
        }
        result.map(|()| report)
    }

    fn enqueue_reporting_into(
        &self,
        paths: &[PathBuf],
        report: &mut EnqueueReport,
    ) -> PipelineResult<()> {
        for path in paths {
            let fingerprint = match self.admit_for_enqueue(path) {
                Ok(fingerprint) => fingerprint,
                Err(error) => {
                    report.skipped.push((path.clone(), error.code));
                    continue;
                }
            };
            let (item, inserted) = self.enqueue_admitted(path, &fingerprint)?;
            if inserted {
                report.added.push(item);
            } else {
                report.already_queued += 1;
            }
        }
        Ok(())
    }

    /// Everything about one document that can refuse it before the queue is
    /// touched: who may add it, and whether its bytes can be read. Returns the
    /// fingerprint the row is keyed on.
    fn admit_for_enqueue(&self, path: &Path) -> PipelineResult<String> {
        let verified = self.admission.authorize(path, AdmissionStage::Enqueue)?;
        let fingerprint = self.files.fingerprint(path)?;
        if verified
            .verified_hash()
            .is_some_and(|hash| hash != fingerprint)
        {
            return Err(PipelineError::new(
                "FILE_CHANGED",
                "The file changed after Microsoft verified its uploader.",
            ));
        }
        Ok(fingerprint)
    }

    /// Writes an admitted document's row and flags it when its content is
    /// already filed. The flag is `true` when the row is new.
    fn enqueue_admitted(
        &self,
        path: &Path,
        fingerprint: &str,
    ) -> PipelineResult<(QueueItem, bool)> {
        let (mut item, inserted) = self.store.enqueue_with_outcome(path, fingerprint)?;
        if item.status == QueueStatus::Queued {
            item = self.flag_if_completed_duplicate(item)?;
        }
        if item.status == QueueStatus::Queued {
            item = self.flag_if_filed_elsewhere(item)?;
        }
        Ok((item, inserted))
    }

    /// Files a just-queued item whose content is already filed as completed
    /// as a copy of that filing, or flags it for review (see
    /// [`Self::file_duplicate`]).
    ///
    /// Enqueue holds no run lock, so either is a compare-and-swap on the
    /// Queued status: if the scheduler claimed the item between the lookup and
    /// the transition, the claim wins and the item analyzes normally.
    fn flag_if_completed_duplicate(&self, item: QueueItem) -> PipelineResult<QueueItem> {
        let Some(duplicate) = self
            .store
            .find_completed_duplicate(&item.source_hash, &source_path_key(&item.source_path))?
        else {
            return Ok(item);
        };
        let original = match (duplicate.filed_as, duplicate.filed_path) {
            (Some(name), Some(filed)) => DuplicateOriginal {
                name,
                folder: filed.parent().map(Path::to_path_buf),
                filed_path: Some(filed),
            },
            // Completed without a rename: the original kept its own name,
            // wherever it is, so that is the name the copy carries.
            _ => DuplicateOriginal {
                name: duplicate
                    .source_path
                    .file_name()
                    .unwrap_or(duplicate.source_path.as_os_str())
                    .to_string_lossy()
                    .into_owned(),
                folder: None,
                filed_path: Some(duplicate.source_path),
            },
        };
        self.file_duplicate(item, original)
    }

    /// The same for a just-queued item whose content the duplicate oracle
    /// knows to be filed already - by a teammate, typically. The name is the
    /// one the filing was made under, as the oracle has it; its folder is
    /// another machine's business, so the copy goes to the output folder's
    /// root.
    fn flag_if_filed_elsewhere(&self, item: QueueItem) -> PipelineResult<QueueItem> {
        let Some(known) = self
            .duplicates
            .filed_elsewhere(&item.source_hash, &item.source_path)
        else {
            return Ok(item);
        };
        self.file_duplicate(
            item,
            DuplicateOriginal {
                name: known.filename,
                folder: None,
                filed_path: None,
            },
        )
    }

    /// What becomes of an exact duplicate: with an output folder to file
    /// into, it is filed there as `COPY - <the original's name>` without
    /// being read; without one, it waits in review as a duplicate, untouched,
    /// as every duplicate used to.
    ///
    /// Staged, the copy is a ready item with an approved copy record, so it
    /// is filed through the same journaled move as any approval - now, unless
    /// the queue is paused or busy, and otherwise by the scheduler the moment
    /// it is free - whether automatic renaming is on or not. Undo puts it
    /// back where it came from like any filing.
    fn file_duplicate(
        &self,
        item: QueueItem,
        original: DuplicateOriginal,
    ) -> PipelineResult<QueueItem> {
        let Some((filename, copy)) = self.copy_plan(&item, original) else {
            return self.flag_duplicate(item);
        };
        let mut record = ProposalRecord::synthetic(filename, String::new());
        record.status = ProposalStatus::Ready;
        record.approved = true;
        record.reasons = vec![EXACT_COPY.to_owned()];
        record.copy_of = Some(copy);
        match self.repository.stage_copy(item.id, &record) {
            Ok(()) => {}
            Err(error) if error.code == "STATE_CONFLICT" => return Ok(item),
            Err(error) => return Err(error),
        }
        if !self.paused.load(Ordering::SeqCst) {
            let staged = self.find_item(item.id)?;
            // The scheduler files it if this cannot: busy is not a failure,
            // and a failure has already put it in review.
            let _ = self.apply_copy(&staged, &record);
        }
        self.find_item(item.id)
    }

    /// The name and folder a duplicate's copy is filed under, or `None` when
    /// it is not filed as a copy: there is no output folder, the file is
    /// already in it, or the original's name cannot be a file's name.
    ///
    /// A file inside the output folder is a filed document - very likely the
    /// original itself, added again from where it is filed - and moving it
    /// would take a filing out of its place; it waits in review instead. The
    /// copy goes beside the original when the original is known to be filed
    /// inside the output folder, and to its root otherwise.
    fn copy_plan(&self, item: &QueueItem, original: DuplicateOriginal) -> Option<(String, CopyOf)> {
        let settings = self.settings.load().ok()?;
        let root = settings.destination.trim();
        if root.is_empty() {
            return None;
        }
        let root = PathBuf::from(root);
        // Only what was put in the input folder is moved on Intern's own
        // initiative; a duplicate added from anywhere else waits in review.
        if !self.in_input_folder(&item.source_path)
            || is_within(&item.source_path, &root)
            || original
                .filed_path
                .as_deref()
                .is_some_and(|filed| same_file_path(filed, &item.source_path))
        {
            return None;
        }
        let folder = original
            .folder
            .filter(|folder| is_within(folder, &root))
            .unwrap_or(root);
        let extension = item
            .source_path
            .extension()
            .and_then(|extension| extension.to_str());
        // A teammate's filing is named from a file anyone with access to the
        // shared folder can write, so its name must be a plain file name
        // before it becomes part of a path.
        let filename = validate_leaf_filename(&copy_name(&original.name, extension)).ok()?;
        Some((
            filename,
            CopyOf {
                original: original.name,
                folder,
            },
        ))
    }

    /// Files a staged copy: the journaled move into its folder, under its
    /// name, with ` (2)` and on when a copy is there already. Nothing is
    /// reported as filed: the original's records stand for both.
    ///
    /// Busy is no failure - the copy stays staged and the scheduler files it
    /// between documents. Anything else puts the document in review as a
    /// duplicate, untouched, as it would have been with nowhere to file it.
    fn apply_copy(&self, item: &QueueItem, record: &ProposalRecord) -> PipelineResult<()> {
        let Some(copy) = record.copy_of.as_ref() else {
            return Ok(());
        };
        if let Err(error) = self.authorize_item(item, AdmissionStage::Apply) {
            if !error.is_retryable() {
                let _ = self.repository.abandon_copy(item.id);
                self.events.queue_changed();
            }
            return Err(error);
        }
        // The folder was chosen when the copy was staged, and the output
        // folder may have changed while it waited: a folder no longer inside
        // it is not recreated, and the copy goes to the root of the output
        // folder as it is now. With none set any more, there is nowhere to
        // file it, and it waits in review as a duplicate.
        let root = self.settings.load()?.destination.trim().to_owned();
        if root.is_empty() {
            let _ = self.repository.abandon_copy(item.id);
            self.events.queue_changed();
            return Ok(());
        }
        let folder = if is_within(&copy.folder, Path::new(&root)) {
            copy.folder.clone()
        } else {
            PathBuf::from(&root)
        };
        if let Err(error) = fs::create_dir_all(&folder) {
            let _ = self.repository.abandon_copy(item.id);
            self.events.queue_changed();
            return Err(PipelineError::new(
                "DESTINATION_UNAVAILABLE",
                format!("the destination folder could not be created ({error})"),
            ));
        }
        let destination = folder.join(&record.filename);
        let applied = {
            let _files = self.hold_file_ops();
            let applied = self.files.apply(item, &destination);
            if let Err(error) = &applied
                && error.code != APPLY_DEFERRED
            {
                let _ = self.files.reconcile(item);
            }
            applied
        };
        match applied {
            Ok(()) => {
                self.events.queue_changed();
                Ok(())
            }
            Err(error) if error.code == APPLY_DEFERRED => Ok(()),
            Err(error) => {
                if self
                    .store
                    .get(item.id)?
                    .is_some_and(|current| current.status == QueueStatus::Ready)
                {
                    let _ = self.repository.abandon_copy(item.id);
                } else {
                    self.report_settled(item.id);
                }
                self.events.queue_changed();
                Err(error)
            }
        }
    }

    /// Whether the item's record is a copy's.
    fn is_copy(&self, item_id: i64) -> bool {
        self.repository
            .load_proposal(item_id)
            .ok()
            .flatten()
            .is_some_and(|record| record.copy_of.is_some())
    }

    fn flag_duplicate(&self, item: QueueItem) -> PipelineResult<QueueItem> {
        match self.store.transition(
            item.id,
            QueueStatus::Queued,
            QueueStatus::NeedsReview,
            Some(ErrorCode::Duplicate),
        ) {
            Ok(flagged) => Ok(flagged),
            Err(error) if error.code() == ErrorCode::StateConflict => Ok(item),
            Err(error) => Err(error.into()),
        }
    }

    pub fn list(&self) -> PipelineResult<Vec<PipelineItem>> {
        self.store
            .list()?
            .into_iter()
            .map(|item| self.pipeline_item(item))
            .collect()
    }

    /// The newest item enqueued from `path` - as given, or as it
    /// canonicalizes now - with its proposal and receipt, or `None` when the
    /// queue has never seen the path.
    ///
    /// A path that no longer canonicalizes (the apply already renamed it away)
    /// still matches as given, which is what lets a finished intake document
    /// report its fate instead of `Unknown`.
    pub fn find_by_source_path(&self, path: &Path) -> PipelineResult<Option<PipelineItem>> {
        let canonical = fs::canonicalize(path).ok();
        let mut candidates = vec![path];
        if let Some(canonical) = canonical.as_deref()
            && canonical != path
        {
            candidates.push(canonical);
        }
        self.store
            .find_newest_by_source_path(&candidates)?
            .map(|item| self.pipeline_item(item))
            .transpose()
    }

    /// [`Self::find_by_source_path`] for the intake watcher, which knows a
    /// document by the name it arrived under: also the item whose file was
    /// renamed `NEEDS REVIEW - ...` from that name because it could not be
    /// read. Only the watcher's question about a document's fate may follow
    /// that rename. Withdrawing or retrying by the old name must not find the
    /// row: the document was taken out of the folder's reach on purpose.
    pub fn find_by_intake_path(&self, path: &Path) -> PipelineResult<Option<PipelineItem>> {
        let canonical = fs::canonicalize(path).ok();
        let mut candidates = vec![path];
        if let Some(canonical) = canonical.as_deref()
            && canonical != path
        {
            candidates.push(canonical);
        }
        self.store
            .find_newest_by_intake_path(&candidates)?
            .map(|item| self.pipeline_item(item))
            .transpose()
    }

    fn pipeline_item(&self, item: QueueItem) -> PipelineResult<PipelineItem> {
        let proposal = self.repository.load_proposal(item.id)?;
        // Read so that a receipt a newer build wrote shows as absent rather
        // than failing the listing - and with it the whole queue window.
        let receipts = self.store.listed_receipts(item.id)?;
        let receipt = receipts.newest;
        let filed_receipt = receipts
            .latest_complete
            .filter(|receipt| receipt.direction == OperationDirection::Apply);
        // An applying item's unfinished receipt is the operation in flight,
        // not one left behind.
        let unsettled_receipt = if item.status == QueueStatus::Applying {
            None
        } else {
            receipts.unsettled
        };
        let duplicate_of = if item.status == QueueStatus::NeedsReview
            && item.error_code == Some(ErrorCode::Duplicate)
        {
            self.store
                .find_completed_duplicate(&item.source_hash, &source_path_key(&item.source_path))?
                .map(|duplicate| {
                    duplicate.filed_as.unwrap_or_else(|| {
                        duplicate
                            .source_path
                            .file_name()
                            .unwrap_or(duplicate.source_path.as_os_str())
                            .to_string_lossy()
                            .into_owned()
                    })
                })
                .or_else(|| {
                    self.duplicates
                        .filed_elsewhere(&item.source_hash, &item.source_path)
                        .map(|known| known.describe())
                })
        } else {
            None
        };
        Ok(PipelineItem {
            id: item.id,
            source_path: item.source_path,
            source_hash: item.source_hash,
            status: item.status,
            processing_failures: item.processing_failures,
            error_code: item.error_code,
            proposal,
            receipt,
            filed_receipt,
            unsettled_receipt,
            duplicate_of,
        })
    }

    /// Every document the queue has filed and not undone: completed items
    /// whose latest finished operation is an apply, with the sentence and
    /// facts that were applied. What a records keeper replays when it is
    /// switched on after documents were already filed.
    ///
    /// The filing time is when the receipt finished. It used to be read from
    /// the filed document's modification time, but a rename does not change
    /// that, so a contract last saved years ago was recorded as filed then.
    pub fn filed_documents(&self) -> PipelineResult<Vec<FiledDocument>> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|item| item.status == QueueStatus::Completed)
            .filter_map(|item| {
                let receipt = item.filed_receipt?;
                // A copy is no filing of its own; see `report_filed`.
                let proposal = item.proposal.filter(|record| record.copy_of.is_none())?;
                let filed_at = self
                    .store
                    .receipt_updated_at(receipt.id)
                    .ok()
                    .flatten()
                    .unwrap_or_else(unix_now);
                filed_document(item.id, &item.source_hash, &receipt, &proposal, filed_at)
            })
            .collect())
    }

    pub fn run_until_idle(&self) -> PipelineResult<()> {
        let _run = self
            .run_lock
            .lock()
            .map_err(|_| PipelineError::new("STATE_CONFLICT", "pipeline lock is unavailable"))?;
        self.apply_pending_ready(ReadyScope::Everything)?;
        while !self.paused.load(Ordering::SeqCst) {
            if !self.run_next_inner()? {
                break;
            }
            // Between documents, not only before the first: an approval made
            // while the queue was working could not be applied then, and this
            // is the next moment the store will let it through.
            self.apply_pending_ready(ReadyScope::ApprovalsOnly)?;
        }
        Ok(())
    }

    pub fn run_next(&self) -> PipelineResult<()> {
        let _run = self
            .run_lock
            .lock()
            .map_err(|_| PipelineError::new("STATE_CONFLICT", "pipeline lock is unavailable"))?;
        if !self.paused.load(Ordering::SeqCst) {
            self.apply_pending_ready(ReadyScope::Everything)?;
            let _ = self.run_next_inner()?;
        }
        Ok(())
    }

    fn run_next_inner(&self) -> PipelineResult<bool> {
        let Some(item) = self.claim_next()? else {
            // Nothing left but documents in doubt, if anything: only proof
            // that the machine works lets them be tried again.
            return self.prove_the_machine();
        };
        // Whether this attempt is a doubted document's one proof attempt,
        // decided as it starts: the machine was shown to work for this kind
        // of failure after the document last failed (`QueueStore::doubt`).
        let proof = self.store.proven_doubt(item.id)?;
        let extraction_evidence = match self.authorize_item(&item, AdmissionStage::Extract) {
            Ok(evidence) => evidence,
            Err(error) if error.is_retryable() => {
                self.store.transition(
                    item.id,
                    QueueStatus::Extracting,
                    QueueStatus::Queued,
                    None,
                )?;
                self.events.queue_changed();
                return Ok(false);
            }
            Err(_) => {
                self.store.transition(
                    item.id,
                    QueueStatus::Extracting,
                    QueueStatus::NeedsReview,
                    Some(ErrorCode::UploaderUnverified),
                )?;
                self.events.queue_changed();
                return Ok(true);
            }
        };
        let Some(extraction_path) = extraction_evidence.extraction_path(&item.source_path) else {
            self.store.transition(
                item.id,
                QueueStatus::Extracting,
                QueueStatus::NeedsReview,
                Some(ErrorCode::UploaderUnverified),
            )?;
            self.events.queue_changed();
            return Ok(true);
        };
        self.active_item.store(item.id, Ordering::SeqCst);
        let request_id = format!("queue-{}-{}", item.id, item.processing_failures + 1);
        let phase = Arc::new(Mutex::new(LeasePhase::Extracting));
        let cancel_phase = Arc::clone(&phase);
        let cancel_worker = Arc::clone(&self.worker);
        let cancel_model = Arc::clone(&self.model);
        let cancel_request_id = request_id.clone();
        let cancel_store = Arc::clone(&self.store);
        let cancel_item = item.id;
        let lease = match LeaseKeeper::start_with_cancel(
            Arc::clone(&self.store),
            item.id,
            self.lease_renewal_interval,
            Arc::new(move || match cancel_phase.lock().map(|phase| *phase) {
                Ok(LeasePhase::Extracting) => {
                    let _ = cancel_worker.cancel(&cancel_request_id);
                }
                // A canceled item loses its lease by being canceled, and the
                // cancel already stopped its request. Stopping the model
                // again only restarted the server a second time.
                Ok(LeasePhase::Analyzing) => {
                    if !item_is_canceled(&cancel_store, cancel_item).unwrap_or(false) {
                        let _ = cancel_model.cancel();
                    }
                }
                Err(_) => {
                    let _ = cancel_worker.shutdown();
                    let _ = cancel_model.shutdown();
                }
            }),
        ) {
            Ok(lease) => lease,
            Err(error) => {
                self.active_item.store(0, Ordering::SeqCst);
                self.store
                    .record_processing_failure(item.id, ErrorCode::IoError)?;
                self.events.queue_changed();
                return Err(error);
            }
        };
        self.events.queue_changed();
        let mut forward_progress = |progress: ExtractProgress| {
            self.events.progress(PipelineProgress {
                item_id: item.id,
                stage: progress.stage,
                current: progress.current,
                total: progress.total,
            });
        };
        let source = match self
            .worker
            .extract(&request_id, extraction_path, &mut forward_progress)
        {
            Ok(source) => source,
            Err(error) => {
                self.active_item.store(0, Ordering::SeqCst);
                if self.shutting_down.load(Ordering::SeqCst) {
                    return Err(PipelineError::new(
                        "SHUTTING_DOWN",
                        "pipeline is shutting down",
                    ));
                }
                if let Err(lease_error) = lease.check() {
                    self.paused.store(true, Ordering::SeqCst);
                    self.events.queue_changed();
                    return Err(lease_error);
                }
                if error.canceled {
                    self.events.queue_changed();
                    return Ok(true);
                }
                let code = extraction_error_code(&error.code, error.retryable);
                let verified = extraction_evidence.verified_hash().is_some();
                // A failure the worker calls permanent - a password, a file
                // that is not what its name says, a document past the limits
                // - reads the same on every attempt, and a second attempt
                // only re-ran it (thirty minutes of it, for a document that
                // hit the time limit). It is settled now, with its own
                // reason. A crash, or a failure the worker says may pass,
                // still gets its second attempt.
                if !error.retryable && !error.crashed && code != ErrorCode::IoError {
                    if code.marks_unreadable() {
                        // The document's own problem, by its nature: it
                        // waits in review, renamed so the input folder says
                        // so too.
                        let mark = self.review_mark(&item, verified);
                        self.hold_unreadable(item.id, code, mark)?;
                    } else {
                        self.store.record_terminal_failure(item.id, code)?;
                        self.unmark_if_marked(item.id);
                    }
                    // Without the text-recognition files every document that
                    // needs them fails the same way, so the queue stops and
                    // says why rather than failing the backlog one by one.
                    if code == ErrorCode::OcrUnavailable {
                        self.pause_for(code.as_str());
                    }
                    self.events.queue_changed();
                    return Ok(true);
                }
                let restart_failed = error.crashed
                    && item.processing_failures == 0
                    && self.worker.restart().is_err();
                // A crash is as often the reader's fault as the
                // document's, and one document cannot tell which: the
                // document goes back to wait in doubt (see `doubt_or_review`)
                // unless the reader was shown to work since it last
                // crashed on it. A worker that will not come back, and any
                // other failure that may pass - the worker's own reading of
                // the file, a placeholder whose content is still in the
                // cloud, a busy worker - fails as before; the intake watcher
                // hands such a document over again once its content
                // arrives.
                if error.crashed && !restart_failed {
                    self.doubt_or_review(
                        &item,
                        &crash_doubt_kind(&item.source_path),
                        ErrorCode::ExtractionFailed,
                        proof.as_deref(),
                        verified,
                        READER_CRASHED,
                    )?;
                    self.events.queue_changed();
                    return Ok(true);
                }
                if self.store.record_processing_failure(item.id, code)? == QueueStatus::Failed {
                    self.unmark_if_marked(item.id);
                }
                // A worker that will not come back cannot read this
                // document on a second attempt either, so the failure is
                // counted twice and the document fails now rather than
                // stalling the queue again. Counted against this item by
                // id: reclaiming through the queue would claim whichever
                // document is next in line, which is not always this one,
                // and leave that one extracting under nobody's lease.
                if restart_failed {
                    self.repository.record_recovered_failure(item.id)?;
                    self.unmark_if_marked(item.id);
                }
                self.events.queue_changed();
                return Ok(true);
            }
        };
        // The reader read a document of this kind, so it is not crashing on
        // every one: a document of the same kind in doubt over a crash is
        // due its proof attempt. One of another kind is not - a reader that
        // crashes on every PDF still reads text - and this one is no longer
        // in doubt over a crash.
        let crash_kind = crash_doubt_kind(&item.source_path);
        let _ = self.store.end_failure_runs(Some(&crash_kind));
        let _ = self.store.prove_doubts(&[&crash_kind], Some(item.id));
        if self
            .store
            .doubt_kind(item.id)
            .ok()
            .flatten()
            .is_some_and(|kind| kind.starts_with("crash:"))
        {
            let _ = self.store.forget_doubt(item.id);
        }
        self.ensure_lease(&lease)?;
        if self.paused.load(Ordering::SeqCst) {
            lease.stop_and_check()?;
            self.store
                .transition(item.id, QueueStatus::Extracting, QueueStatus::Queued, None)?;
            self.active_item.store(0, Ordering::SeqCst);
            self.events.queue_changed();
            return Ok(true);
        }
        self.store.transition(
            item.id,
            QueueStatus::Extracting,
            QueueStatus::Analyzing,
            None,
        )?;
        *phase
            .lock()
            .map_err(|_| PipelineError::new("STATE_CONFLICT", "lease phase is unavailable"))? =
            LeasePhase::Analyzing;
        self.events.queue_changed();
        let extension = item
            .source_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned();
        let existing = names_beside(
            item.source_path.parent().unwrap_or_else(|| Path::new(".")),
            &item.source_path,
        );
        if let Err(error) = self.authorize_item(&item, AdmissionStage::Analyze) {
            lease.stop_and_check()?;
            let (next, code, keep_draining) = if error.is_retryable() {
                (QueueStatus::Queued, None, false)
            } else {
                (
                    QueueStatus::NeedsReview,
                    Some(ErrorCode::UploaderUnverified),
                    true,
                )
            };
            self.store
                .transition(item.id, QueueStatus::Analyzing, next, code)?;
            self.active_item.store(0, Ordering::SeqCst);
            self.events.queue_changed();
            return Ok(keep_draining);
        }
        let analysis = match self.analyze_with_deadline(&source, &extension, &existing) {
            Ok(analysis) => {
                self.forget_missing_model();
                // A document read successfully shows the model working now,
                // so every document in doubt over the model is due its proof
                // attempt - and this one is no longer in doubt.
                for kind in MODEL_DOUBTS {
                    let _ = self.store.end_failure_runs(Some(kind));
                }
                let _ = self.store.forget_doubt(item.id);
                let _ = self.store.prove_doubts(MODEL_DOUBTS, Some(item.id));
                analysis
            }
            Err(error) => {
                self.active_item.store(0, Ordering::SeqCst);
                if self.shutting_down.load(Ordering::SeqCst) {
                    return Err(PipelineError::new(
                        "SHUTTING_DOWN",
                        "pipeline is shutting down",
                    ));
                }
                // Before the lease: canceling an item is what takes its lease
                // away, so a canceled item always fails its lease check, and
                // checking that first paused the queue for every cancel.
                if item_is_canceled(&self.store, item.id)? {
                    self.events.queue_changed();
                    return Ok(true);
                }
                if let Err(lease_error) = lease.check() {
                    self.paused.store(true, Ordering::SeqCst);
                    self.events.queue_changed();
                    return Err(lease_error);
                }
                let code = model_error_code(&error);
                let action = model_failure_action(&error.code);
                if action != ModelFailureAction::Requeue {
                    // Anything but an empty slot means there was a model to
                    // ask this time, whatever it answered.
                    self.forget_missing_model();
                }
                match action {
                    ModelFailureAction::Requeue => {
                        // Nothing was asked: no model was loaded to ask, as
                        // happens for a moment while switching between the
                        // hosted and the local one, before the local server's
                        // start has begun. (A request that meets a start or a
                        // restart under way waits for it in the app's model,
                        // and comes back here only if no server came up.) The
                        // document goes back to wait with nothing counted
                        // against it - counted, it failed outright at the
                        // second such moment - and this drain ends rather
                        // than claiming it again at once; the scheduler's next
                        // pass finds the model in place.
                        lease.stop_and_check()?;
                        self.store.transition(
                            item.id,
                            QueueStatus::Analyzing,
                            QueueStatus::Queued,
                            None,
                        )?;
                        // Unless it never comes back. A restart that failed
                        // leaves the slot empty for as long as the app runs,
                        // and every pass then read the head document again in
                        // full only to put it back, with nothing on screen to
                        // say the queue had stopped moving. A model still
                        // missing once the moment has passed stops the queue
                        // and says so.
                        if self.model_missing_too_long() {
                            self.pause_for(ErrorCode::ModelFailed.as_str());
                        }
                        self.events.queue_changed();
                        return Ok(false);
                    }
                    ModelFailureAction::Terminal => {
                        // Asking again gets the same answer about this
                        // document, and from a hosted model bills for it
                        // again, so it is settled now and the queue moves
                        // on: in review, for a person to retry, name by
                        // hand, keep or remove. A refusal or a document too
                        // large is the document's own by its nature. A
                        // reply that could not be used, or an internal
                        // failure, can as well be the model's: the document
                        // waits in doubt until the model is shown to work.
                        let verified = extraction_evidence.verified_hash().is_some();
                        if is_unreadable_reply(&error.code) || code == ErrorCode::AnalysisFailed {
                            self.doubt_or_review(
                                &item,
                                STREAK_REPLY,
                                code,
                                proof.as_deref(),
                                verified,
                                pause_reason_code(&error.code, code),
                            )?;
                        } else {
                            let mark = self.review_mark(&item, verified);
                            self.hold_unreadable(item.id, code, mark)?;
                        }
                    }
                    ModelFailureAction::Pause => {
                        // Failures that would repeat for every document - a
                        // model that cannot be reached, a key that was
                        // refused, an account out of credit - pause the queue
                        // rather than fail the backlog one item at a time.
                        // The document keeps its second attempt for when the
                        // queue is resumed.
                        if self.store.record_processing_failure(item.id, code)?
                            == QueueStatus::Failed
                        {
                            self.unmark_if_marked(item.id);
                        }
                        self.pause_for(pause_reason_code(&error.code, code));
                    }
                    ModelFailureAction::Retry => {
                        // A model that ran out of time on a document is as
                        // often too slow for everything as stuck on that
                        // one: the document waits in doubt until the model
                        // is shown to work. Every other failure here - an
                        // unhealthy server, a rejected request, settings
                        // that could not be read - is this computer's, and
                        // fails as it always did once the second attempt is
                        // used.
                        if error.code == "MODEL_TIMEOUT" {
                            self.doubt_or_review(
                                &item,
                                STREAK_TIMEOUT,
                                ErrorCode::ModelFailed,
                                proof.as_deref(),
                                extraction_evidence.verified_hash().is_some(),
                                ErrorCode::ModelFailed.as_str(),
                            )?;
                        } else if self.store.record_processing_failure(item.id, code)?
                            == QueueStatus::Failed
                        {
                            self.unmark_if_marked(item.id);
                        }
                    }
                }
                self.events.queue_changed();
                return Ok(true);
            }
        };
        // An answer for a document canceled while it was being read is not
        // wanted, and its lost lease is not the queue's problem.
        if item_is_canceled(&self.store, item.id)? {
            self.active_item.store(0, Ordering::SeqCst);
            self.events.queue_changed();
            return Ok(true);
        }
        self.ensure_lease(&lease)?;
        // The document's words, respelled the way review has taught Intern
        // to. Applied here, after validation, so the evidence stayed the
        // document's and only the name is the reviewer's.
        let (styled, house_rules) = self.repository.active_style()?.apply(&analysis.proposal);
        // And named by the other side when it names the person's own
        // organisation too. Settings that cannot be read name nobody, so the
        // name carries every party, as it did before the setting existed; an
        // automatic rename below reports the unreadable file.
        let own_names = self
            .settings
            .load()
            .map(|settings| settings.own_names())
            .unwrap_or_default();
        let (named, _) = counterparty_view(&styled, &own_names);
        let filename = self.compose_for_target(
            &item.source_path,
            &named,
            &house_rules,
            &extension,
            &existing,
        );
        // The exact-bytes check ran before analysis. This one needs the text
        // and the date, so it runs after: a second scan, a re-export, or a
        // copy saved again with new metadata says what a filed document
        // says, and is not filed on its own.
        let near_duplicate_of = self.near_duplicate_of(item.id, &analysis);
        let mut reasons = analysis
            .review_reasons
            .iter()
            .map(|reason| reason.as_str().to_owned())
            .collect::<Vec<_>>();
        let mut status = analysis.status;
        if near_duplicate_of.is_some() {
            reasons.push(NEAR_DUPLICATE.to_owned());
            status = ProposalStatus::NeedsReview;
        }
        let record = ProposalRecord {
            status,
            filename,
            description: analysis.description.clone(),
            reasons,
            analysis,
            revision: 1,
            house_rules,
            own_names,
            near_duplicate_of,
            approved: false,
            manual: false,
            copy_of: None,
        };
        let next = match record.status {
            ProposalStatus::Ready => QueueStatus::Ready,
            ProposalStatus::NeedsReview => QueueStatus::NeedsReview,
        };
        lease.stop_and_check()?;
        self.repository.save_initial_and_transition(
            item.id,
            &record,
            next,
            self.store.session_id(),
        )?;
        self.admission
            .processed(&item.source_path, &item.source_hash);
        // The organisation's names were read before the name was composed,
        // and a Settings save since then renamed everything that was waiting
        // - which this document was not, yet. Read once more now that it
        // waits: a list that changed in between names it the way it names
        // every other waiting document, and a save after this point finds it
        // waiting.
        let settings = self.settings.load();
        if let Ok(current) = &settings {
            let names = current.own_names();
            if names != record.own_names {
                self.recompose(&self.repository.active_style()?, &item, Some(&names))?;
            }
        }
        let ready_item = self.store.get(item.id)?.ok_or_else(|| {
            PipelineError::new(
                "ITEM_NOT_FOUND",
                "queue item disappeared after proposal storage",
            )
        })?;
        self.active_item.store(0, Ordering::SeqCst);
        self.events.queue_changed();
        if next == QueueStatus::Ready {
            let settings = match settings {
                Ok(settings) => settings,
                Err(error) => {
                    self.repository.mark_needs_review(item.id, &error.code)?;
                    self.events.queue_changed();
                    return Ok(true);
                }
            };
            if settings.automatic_rename && !self.paused.load(Ordering::SeqCst) {
                let _ = self.apply_if_unchanged(&ready_item, &record.filename, &settings);
            }
        }
        Ok(true)
    }

    /// The next document to read, if there is one the store will hand out.
    ///
    /// Nothing is claimed while any document is being renamed, and an undo,
    /// an approval or a check of unsettled files can now be renaming one on
    /// another thread in the middle of a drain - an undo no longer waits for
    /// the document being read. Finding nothing then ended the drain, and
    /// nothing woke the scheduler again when the rename was done: the backlog
    /// sat until the next document arrived or the recovery pass came round,
    /// up to a minute later. So a drain that finds nothing waits for whatever
    /// file operation is in flight and looks once more. Holding the run lock
    /// while it waits is safe: those operations never take it.
    fn claim_next(&self) -> PipelineResult<Option<QueueItem>> {
        if let Some(item) = self.store.claim_next()? {
            return Ok(Some(item));
        }
        drop(self.hold_file_ops());
        if self.paused.load(Ordering::SeqCst) {
            return Ok(None);
        }
        Ok(self.store.claim_next()?)
    }

    /// Applies the renames the scheduler owes: every ready document when
    /// automatic renaming is on, and every document a person approved while
    /// the queue was busy elsewhere, whether it is on or not.
    fn apply_pending_ready(&self, scope: ReadyScope) -> PipelineResult<()> {
        if self.paused.load(Ordering::SeqCst) {
            return Ok(());
        }
        let ready = self
            .store
            .list()?
            .into_iter()
            .filter(|item| item.status == QueueStatus::Ready)
            .collect::<Vec<_>>();
        if ready.is_empty() {
            return Ok(());
        }
        let settings = self.settings.load()?;
        let automatic = settings.automatic_rename && scope == ReadyScope::Everything;
        for item in ready {
            if self.paused.load(Ordering::SeqCst) {
                break;
            }
            let Some(proposal) = self.repository.load_proposal(item.id)? else {
                if automatic {
                    self.repository
                        .mark_needs_review(item.id, "PROPOSAL_MISSING")?;
                }
                continue;
            };
            if !automatic && !proposal.approved {
                continue;
            }
            // A duplicate staged as a copy while the queue was busy, filed
            // whatever automatic renaming says: it was never going to be
            // named by anyone.
            if proposal.copy_of.is_some() {
                let _ = self.apply_copy(&item, &proposal);
                continue;
            }
            let _ = self.apply_if_unchanged(&item, &proposal.filename, &settings);
        }
        Ok(())
    }

    fn ensure_lease(&self, lease: &LeaseKeeper) -> PipelineResult<()> {
        if let Err(error) = lease.check() {
            self.active_item.store(0, Ordering::SeqCst);
            self.paused.store(true, Ordering::SeqCst);
            self.events.queue_changed();
            return Err(error);
        }
        Ok(())
    }

    /// The filing whose text this analysis nearly repeats, if any: first the
    /// queue's own history, then whatever the duplicate oracle knows from
    /// other machines.
    fn near_duplicate_of(&self, item_id: i64, analysis: &DocumentAnalysis) -> Option<String> {
        let fingerprint = fingerprint::decode(analysis.text_fingerprint.as_deref()?)?;
        let date = analysis.proposal.document_date.clone().or_else(|| {
            analysis
                .model_proposal
                .as_ref()
                .and_then(|reply| reply.document_date.clone())
        });
        let local = self
            .repository
            .find_similar(fingerprint, item_id)
            .unwrap_or_default()
            .into_iter()
            .find(|similar| {
                same_document(similar.distance, &similar.filing.filename, date.as_deref())
            })
            .map(|similar| similar.filing.describe());
        local.or_else(|| {
            self.duplicates
                .similar_elsewhere(fingerprint)
                .filter(|similar| {
                    same_document(similar.distance, &similar.filing.filename, date.as_deref())
                })
                .map(|similar| similar.filing.describe())
        })
    }

    /// The name a proposal will be applied under. The engine composed its
    /// name against the source folder, which is the only folder it knows;
    /// the name that is actually applied must not collide in the folder the
    /// document is going to. Without readable settings the source folder's
    /// names (`fallback`) stand in.
    ///
    /// `proposal` is the styled proposal and `house_rules` the rules that
    /// styled it: what a rule spelled is the reviewer's, and is written as
    /// they typed it.
    fn compose_for_target(
        &self,
        source_path: &Path,
        proposal: &ValidatedProposal,
        house_rules: &[HouseRule],
        extension: &str,
        fallback: &[String],
    ) -> String {
        let named = compose_styled_filename(proposal, house_rules, extension, &[]).value;
        let existing = match self.settings.load() {
            Ok(settings) => names_beside(
                &target_folder(
                    &settings,
                    source_path,
                    &proposal_as_applied(proposal, &named),
                ),
                source_path,
            ),
            Err(_) => fallback.to_vec(),
        };
        compose_styled_filename(
            proposal,
            house_rules,
            extension,
            &existing.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .value
    }

    /// The spellings review has taught Intern, newest first.
    pub fn learned_rules(&self) -> PipelineResult<Vec<LearnedRule>> {
        self.repository.list_rules()
    }

    /// Stop applying a learned spelling. Names already applied keep it;
    /// documents still waiting go back to the document's own words.
    pub fn forget_rule(&self, id: i64) -> PipelineResult<()> {
        if !self.repository.forget_rule(id)? {
            return Err(PipelineError::new(
                "RULE_NOT_FOUND",
                "learned spelling does not exist",
            ));
        }
        self.restyle_waiting()
    }

    /// Apply a learned spelling from now on without waiting for a second
    /// edit, including to documents still waiting.
    pub fn use_rule(&self, id: i64) -> PipelineResult<()> {
        if !self.repository.use_rule(id)? {
            return Err(PipelineError::new(
                "RULE_NOT_FOUND",
                "learned spelling does not exist",
            ));
        }
        self.restyle_waiting()
    }

    /// Recomposes the proposed name of every document still waiting under
    /// the spellings now in force, so a rule that just changed shows in the
    /// queue at once rather than only on the next document.
    fn restyle_waiting(&self) -> PipelineResult<()> {
        self.recompose_waiting(None)
    }

    /// Names every document still waiting by the counterparty when it names
    /// the person's own organisation, now that Settings names it `names`.
    /// Waiting, unapproved documents are renamed in the queue at once, the
    /// way a learned spelling renames them; a name a person approved is
    /// theirs and keeps everything it says. A document already named by
    /// `names` is not touched, so asking again with the same list - after
    /// every save, to finish a rename an earlier one never reached - writes
    /// nothing and announces nothing.
    pub fn refresh_own_names(&self, names: &[String]) -> PipelineResult<()> {
        self.recompose_waiting(Some(&normalize_own_names(names)))
    }

    /// Recomposes the proposed name of every waiting document whose name
    /// the spellings in force, or `own_names` when it is given, would change.
    ///
    /// A name a person approved is never recomposed: the one being approved
    /// right now, and one approved earlier that is still waiting for a busy
    /// queue to file it. It is the name they chose, and composing it again
    /// from the validated facts would throw away everything the facts do not
    /// carry, the date they typed in most of all. An earlier approval waiting
    /// for the queue counts too: a rule learned while it waited once rebuilt
    /// its name, and the scheduler then filed the rebuilt name under the
    /// approval - without the date the reviewer had typed.
    fn recompose_waiting(&self, own_names: Option<&[String]>) -> PipelineResult<()> {
        let style = self.repository.active_style()?;
        let mut changed = false;
        for item in self.store.list()? {
            if !matches!(item.status, QueueStatus::NeedsReview | QueueStatus::Ready) {
                continue;
            }
            changed |= self.recompose(&style, &item, own_names)?;
        }
        if changed {
            self.events.queue_changed();
        }
        Ok(())
    }

    /// Recomposes one waiting document's proposed name the way
    /// [`Self::recompose_waiting`] does, and says whether its record changed.
    ///
    /// The revision moves only when the name does. It is how the window
    /// knows the name it holds is no longer the proposal, and the reviewer's
    /// draft of the name and the description is replaced when it moves. A
    /// change that leaves the name as it was - an organisation this document
    /// never mentions, a respelling of a party the name leaves out - is
    /// stored without moving it, so whatever the reviewer has typed and not
    /// yet approved survives a Settings save.
    fn recompose(
        &self,
        style: &HouseStyle,
        item: &QueueItem,
        own_names: Option<&[String]>,
    ) -> PipelineResult<bool> {
        let Some(mut record) = self.repository.load_proposal(item.id)? else {
            return Ok(false);
        };
        // A copy's name, or one typed for a document that could not be
        // read, has no facts behind it to compose a name from.
        if record.approved || record.is_synthetic() {
            return Ok(false);
        }
        let (styled, house_rules) = style.apply(&record.analysis.proposal);
        let own_names = own_names.map_or_else(|| record.own_names.clone(), <[String]>::to_vec);
        if house_rules == record.house_rules && own_names == record.own_names {
            return Ok(false);
        }
        let (named, _) = counterparty_view(&styled, &own_names);
        let extension = item
            .source_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned();
        let existing = names_beside(
            item.source_path.parent().unwrap_or_else(|| Path::new(".")),
            &item.source_path,
        );
        let filename = self.compose_for_target(
            &item.source_path,
            &named,
            &house_rules,
            &extension,
            &existing,
        );
        if filename != record.filename {
            record.revision += 1;
        }
        record.filename = filename;
        record.house_rules = house_rules;
        record.own_names = own_names;
        self.repository.replace_proposal(item.id, &record)?;
        Ok(true)
    }

    /// What an approved edit teaches, if anything: a respelled party or
    /// type, remembered, and applied on its own once the same change has
    /// been made twice. A spelling Intern itself applied that the reviewer
    /// changed again is a change of mind about the document's word, not
    /// about Intern's; restoring the document's own spelling retracts the
    /// rule.
    fn learn_from_edit(
        &self,
        record: &ProposalRecord,
        extension: &str,
        approved: &str,
    ) -> PipelineResult<()> {
        let Some(lesson) = lesson_from_edit(
            &record.styled_proposal(),
            extension,
            &record.filename,
            approved,
        ) else {
            return Ok(());
        };
        let lesson = match record.house_rules.iter().find(|applied| {
            applied.kind == lesson.kind && HouseRule::key(&applied.to) == lesson.from_key()
        }) {
            Some(applied) => HouseRule::new(lesson.kind, applied.from.clone(), lesson.to),
            None => lesson,
        };
        if HouseRule::key(&lesson.to) == lesson.from_key() {
            self.repository.forget_rule_for(lesson.kind, &lesson.from)?;
        } else if lesson.is_meaningful() {
            self.repository.learn(&lesson)?;
        }
        self.restyle_waiting()
    }

    fn analyze_with_deadline(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[String],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        let model = Arc::clone(&self.model);
        let request_model = Arc::clone(&model);
        let source = source.clone();
        let extension = extension.to_owned();
        let existing_names = existing_names.to_vec();
        let canceled = Arc::new(AtomicBool::new(false));
        let request_canceled = Arc::clone(&canceled);
        self.set_active_cancel(Some(Arc::clone(&canceled)));
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let join = std::thread::Builder::new()
            .name("intern-model-request".into())
            .spawn(move || {
                let existing = existing_names
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let was_canceled = || {
                    request_canceled
                        .load(Ordering::SeqCst)
                        .then(|| ModelFailure::fatal("MODEL_CANCELED"))
                };
                // A request the cancel interrupted fails like a server that
                // died - the local model is restarted under it - and was
                // recovered and sent again: the canceled document read a
                // second time, on yet another restarted server.
                let result = match request_model.analyze(&source, &extension, &existing) {
                    Err(error) => match was_canceled() {
                        Some(canceled) => Err(canceled),
                        None if error.retryable => {
                            request_model
                                .recover(&error)
                                .and_then(|()| match was_canceled() {
                                    Some(canceled) => Err(canceled),
                                    None => request_model.analyze(&source, &extension, &existing),
                                })
                        }
                        None => Err(error),
                    },
                    result => result,
                };
                let _ = sender.send(result);
            })
            .map_err(|_| {
                self.set_active_cancel(None);
                ModelFailure::fatal("MODEL_REQUEST_FAILED")
            })?;
        let result = self.await_model_request(&model, &canceled, &receiver, join);
        self.set_active_cancel(None);
        result
    }

    fn set_active_cancel(&self, flag: Option<Arc<AtomicBool>>) {
        *self
            .active_cancel
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = flag;
    }

    /// Marks the active model request canceled, before the model is told.
    fn cancel_active_request(&self) {
        if let Some(flag) = self
            .active_cancel
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            flag.store(true, Ordering::SeqCst);
        }
    }

    fn await_model_request(
        &self,
        model: &Arc<dyn AnalyzerBoundary>,
        request_canceled: &AtomicBool,
        receiver: &std::sync::mpsc::Receiver<Result<DocumentAnalysis, ModelFailure>>,
        join: std::thread::JoinHandle<()>,
    ) -> Result<DocumentAnalysis, ModelFailure> {
        match receiver.recv_timeout(self.model_timeout) {
            Ok(result) => {
                let _ = join.join();
                result
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                request_canceled.store(true, Ordering::SeqCst);
                let canceled = model.cancel();
                // A request that has already missed its deadline is given a
                // little longer to notice the cancel, and then left to finish
                // on its own. Waiting on it without a deadline meant one model
                // that honoured neither its deadline nor its cancel stopped
                // the queue for as long as the process lived.
                if receiver
                    .recv_timeout(self.model_timeout.min(MODEL_CANCEL_GRACE))
                    .is_ok()
                {
                    let _ = join.join();
                }
                match canceled {
                    Ok(()) => Err(ModelFailure::fatal("MODEL_TIMEOUT")),
                    Err(_) => Err(ModelFailure::fatal("MODEL_CANCEL_FAILED")),
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let _ = join.join();
                Err(ModelFailure::fatal("MODEL_REQUEST_FAILED"))
            }
        }
    }

    fn authorize_item(
        &self,
        item: &QueueItem,
        stage: AdmissionStage,
    ) -> PipelineResult<AdmissionEvidence> {
        let verified = self.admission.authorize(&item.source_path, stage)?;
        if verified
            .verified_hash()
            .is_some_and(|hash| *hash != item.source_hash)
        {
            return Err(PipelineError::new(
                "UPLOADER_UNVERIFIED",
                "The current file is not the version whose uploader was verified. Retry after verification.",
            ));
        }
        Ok(verified)
    }

    fn apply_if_unchanged(
        &self,
        item: &QueueItem,
        filename: &str,
        settings: &AppSettings,
    ) -> PipelineResult<()> {
        if let Err(error) = self.authorize_item(item, AdmissionStage::Apply) {
            if !error.is_retryable()
                && self
                    .repository
                    .mark_needs_review(item.id, "UPLOADER_UNVERIFIED")
                    .is_err()
            {
                // The record could not be put in review - none to update, or
                // the database refused - and a ready row is what the
                // scheduler files from. The row alone goes to review, so the
                // denial is never retried as an approval on every pass.
                let _ = self
                    .repository
                    .hold_row_for_review(item.id, ErrorCode::UploaderUnverified);
            }
            if !error.is_retryable() {
                self.events.queue_changed();
            }
            return Err(error);
        }
        let fingerprint = match self.files.fingerprint(&item.source_path) {
            Ok(fingerprint) => fingerprint,
            Err(error) => {
                self.repository.mark_needs_review(item.id, &error.code)?;
                self.events.queue_changed();
                return Err(error);
            }
        };
        if fingerprint != item.source_hash {
            // Back to review, and said: approve reported "Rename applied" for a
            // document that had just gone back to review instead, and approving
            // again could only repeat it. Reading the file again is the way on.
            self.repository.mark_needs_review(item.id, "FILE_CHANGED")?;
            self.events.queue_changed();
            return Err(PipelineError::new(
                "FILE_CHANGED",
                "The file changed after it was analyzed. Re-analyze it.",
            ));
        }
        let proposal = self.repository.load_proposal(item.id)?;
        // The name to apply is the one the record holds now, not the one the
        // caller read a moment ago. Nothing holds the queue still between a
        // scheduler pass deciding what to file and the file operation itself,
        // and an edit approved in that moment is the name the person expects
        // to see on the document.
        let filename = proposal
            .as_ref()
            .map_or(filename, |record| record.filename.as_str());
        if leading_date(filename).is_none() {
            self.repository.mark_needs_review(item.id, DATE_REQUIRED)?;
            self.events.queue_changed();
            return Ok(());
        }
        let target = match proposal.as_ref() {
            Some(record) => target_folder(
                settings,
                &item.source_path,
                &proposal_as_applied(&record.styled_proposal(), filename),
            ),
            None => destination_root(settings, &item.source_path),
        };
        // A layout subfolder exists only once a document is filed into it; a
        // folder that cannot be created is the same failure a missing
        // destination would be, and is reported the same way.
        if let Err(error) = fs::create_dir_all(&target) {
            let failure = PipelineError::new(
                "DESTINATION_UNAVAILABLE",
                format!("the destination folder could not be created ({error})"),
            );
            self.repository.mark_needs_review(item.id, &failure.code)?;
            self.events.queue_changed();
            return Err(failure);
        }
        let destination = target.join(filename);
        if names_the_source(&destination, &item.source_path) {
            return self.complete_already_named(item);
        }
        let applied = {
            let _files = self.hold_file_ops();
            let applied = self.files.apply(item, &destination);
            if let Err(error) = &applied
                && error.code != APPLY_DEFERRED
            {
                // Core file operations journal ambiguous failures in Applying.
                // Settle them now, under the same hold, so no recovery pass
                // can take the operation over between its failure and this;
                // the scheduler also retries reconciliation periodically.
                let _ = self.files.reconcile(item);
            }
            applied
        };
        if let Err(error) = applied {
            if error.code == APPLY_DEFERRED {
                // Nothing is wrong with this document: the queue was busy with
                // another one. The name a person approved is durable in the
                // proposal, so the item stays ready and the scheduler applies
                // it between documents rather than sending it to review.
                self.events.queue_changed();
                return Ok(());
            }
            if self
                .store
                .get(item.id)?
                .is_some_and(|current| current.status == QueueStatus::Ready)
            {
                self.repository.mark_needs_review(item.id, &error.code)?;
            } else {
                self.report_settled(item.id);
            }
            self.events.queue_changed();
            return Err(error);
        }
        self.report_filed(item);
        self.events.queue_changed();
        Ok(())
    }

    /// Completes a document whose name is already the one it would be filed
    /// under, without touching the file.
    ///
    /// A rename onto the document's own name has nowhere to go: the name is
    /// taken, by the document itself, so the applier refused it or - through
    /// the collision suffix - filed it as "... (2)". A name that differs only
    /// in case is the same file to Windows, and a case-only rename is not one
    /// this release makes, so it completes the same way. The record says why
    /// nothing moved.
    fn complete_already_named(&self, item: &QueueItem) -> PipelineResult<()> {
        if let Err(error) = self
            .store
            .complete_keep_original(item.id, QueueStatus::Ready)
        {
            // An approval and a scheduler pass can reach the same document;
            // the one that finds it completed by the other has nothing to do.
            if self
                .store
                .get(item.id)?
                .is_some_and(|current| current.status == QueueStatus::Completed)
            {
                return Ok(());
            }
            return Err(error.into());
        }
        if let Some(mut record) = self.repository.load_proposal(item.id)?
            && !record.reasons.iter().any(|reason| reason == ALREADY_NAMED)
        {
            record.reasons.push(ALREADY_NAMED.to_owned());
            record.revision += 1;
            self.repository.replace_proposal(item.id, &record)?;
        }
        self.events.queue_changed();
        Ok(())
    }

    /// Tells the filing sink about a rename that just completed. Read back
    /// from the store rather than assumed: the receipt carries the destination
    /// the applier actually chose, suffix and all, and the proposal carries
    /// the sentence a reviewer may have edited.
    fn report_filed(&self, item: &QueueItem) {
        let Ok(Some(receipt)) = self.store.load_latest_complete_receipt(item.id) else {
            return;
        };
        let Ok(Some(proposal)) = self.repository.load_proposal(item.id) else {
            return;
        };
        // A copy is the original's duplicate, not a filing of its own: no
        // description record, no shared marker, no text fingerprint, so
        // nothing ever names another document after it.
        if proposal.copy_of.is_some() {
            return;
        }
        if let Some(document) =
            filed_document(item.id, &item.source_hash, &receipt, &proposal, unix_now())
        {
            if let Some(fingerprint) = document
                .text_fingerprint
                .as_deref()
                .and_then(fingerprint::decode)
            {
                let filed_name = document
                    .destination
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let _ = self
                    .repository
                    .remember_fingerprint(item.id, fingerprint, &filed_name);
            }
            self.filing.filed(&document);
        }
    }

    /// Settles a document Intern could not read: in review under `code`,
    /// and its file renamed `NEEDS REVIEW - ...` where it lies.
    ///
    /// The review comes first and on its own: it is the decision, and it is
    /// durable the moment it is made. The rename follows under its own
    /// journal, so a crash between the two leaves a document in review that
    /// recovery marks (see `FileApplier::reconcile_review_renames`), never a
    /// renamed file the queue has lost.
    fn hold_unreadable(
        &self,
        item_id: i64,
        code: ErrorCode,
        mark: ReviewMark,
    ) -> PipelineResult<()> {
        self.store.record_unreadable(item_id, code, mark)?;
        self.mark_unreadable(item_id);
        Ok(())
    }

    /// Whether the file of a document going to review because it could not
    /// be read is renamed `NEEDS REVIEW - ...` (see [`ReviewMark`]).
    ///
    /// Never for a document admitted on Microsoft's word about its uploader
    /// (`verified`): that proof names the file - its name in the library,
    /// and a creation time equal to its last change - and every later stage
    /// asks for it again, so a renamed file could never be retried, named or
    /// filed. Such a document waits in review under its own name. Never,
    /// either, for a file outside the input folder: Intern renames only
    /// what was put there for it. When Settings cannot be read to say,
    /// nothing is decided: the mark is made, or decided against, once they
    /// can be (see `mark_unreadable` and `recover`).
    fn review_mark(&self, item: &QueueItem, verified: bool) -> ReviewMark {
        if verified || self.input_folder_holds(&item.source_path) == Some(false) {
            ReviewMark::Never
        } else {
            ReviewMark::Now
        }
    }

    /// Whether `path` is inside the input folder Settings names: `None`
    /// when Settings cannot be read to say. Intern renames and moves on its
    /// own only what was put there for it.
    fn input_folder_holds(&self, path: &Path) -> Option<bool> {
        let settings = self.settings.load().ok()?;
        let folder = settings.intake_folder.trim();
        Some(!folder.is_empty() && is_within(path, Path::new(folder)))
    }

    /// [`Self::input_folder_holds`], with "cannot say" read as no.
    fn in_input_folder(&self, path: &Path) -> bool {
        self.input_folder_holds(path) == Some(true)
    }

    /// Settles a failure that may be the machine's rather than the
    /// document's: an unusable reply or an internal failure (`reply`), a
    /// time-out (`timeout`), a reader crash (`crash`).
    ///
    /// One document cannot tell those apart, and a later success says the
    /// machine works now, not that it worked when this document failed. So
    /// the document is judged only on an attempt that started after the
    /// machine was shown to work for this kind of failure - another document
    /// read, or the self-test passed (`proof`). Failing so, the failure is
    /// its own: it goes to review, renamed under the usual rules. Otherwise
    /// it goes back to wait in doubt, under its own name and with nothing
    /// counted against it, behind every document not in doubt; and it is
    /// one more document in a run of such failures, which stops the queue
    /// with `reason` once it is long enough.
    fn doubt_or_review(
        &self,
        item: &QueueItem,
        kind: &str,
        code: ErrorCode,
        proof: Option<&str>,
        verified: bool,
        reason: &str,
    ) -> PipelineResult<()> {
        if proof == Some(kind) {
            let mark = self.review_mark(item, verified);
            return self.hold_unreadable(item.id, code, mark);
        }
        self.store.doubt(item.id, kind, code)?;
        if self.store.join_failure_run(item.id, kind).unwrap_or(0) >= FAILURES_BEFORE_PAUSE {
            self.pause_for(reason);
        }
        Ok(())
    }

    /// Shows the machine working, when only documents in doubt are left to
    /// read and nothing else will: the model's self-test - the calibration
    /// document every model must read correctly before Intern uses it - for
    /// doubts over the model, and a small text file read for doubts over the
    /// reader on text files. Passing, the documents in doubt are due their
    /// proof attempt, and the queue goes on. Failing, the machine is what is
    /// wrong: the queue stops and says so, and the documents stay in doubt
    /// under their own names. Says whether there is more to do.
    ///
    /// A reader crash on any other kind of file has no self-test: a reader
    /// that crashes on every PDF reads a text file fine, so only a PDF read
    /// successfully proves it works for PDFs. Those documents wait, under
    /// their own names, for a document of their kind; several of one kind
    /// waiting with none to prove it stop the queue to say the reader may be
    /// what is wrong. Resuming gives each one more attempt, which proves
    /// nothing (`QueueStore::allow_unproven_crash_attempts`).
    fn prove_the_machine(&self) -> PipelineResult<bool> {
        let doubts = self.store.unproven_doubts()?;
        if doubts.is_empty() || self.paused.load(Ordering::SeqCst) {
            return Ok(false);
        }
        let model_kinds = doubts
            .iter()
            .map(|(kind, _)| kind.as_str())
            .filter(|kind| MODEL_DOUBTS.contains(kind))
            .collect::<Vec<_>>();
        if !model_kinds.is_empty() {
            match self.model_self_test() {
                Ok(()) => self.store.prove_doubts(&model_kinds, None)?,
                // No model loaded to ask: as for a document that meets
                // none, the moment of a switch passes, and a model still
                // missing once it has stops the queue and says so rather
                // than leaving the documents in doubt waiting in silence.
                Err(None) => {
                    if self.model_missing_too_long() {
                        self.pause_for(ErrorCode::ModelFailed.as_str());
                    }
                    return Ok(false);
                }
                Err(Some(reason)) => {
                    self.pause_for(&reason);
                    return Ok(false);
                }
            }
        }
        let text_crashes = crash_doubt_kind(Path::new("check.txt"));
        if doubts.iter().any(|(kind, _)| *kind == text_crashes) {
            match self.reader_self_test() {
                ReaderCheck::Read => self.store.prove_doubts(&[&text_crashes], None)?,
                ReaderCheck::Failed => {
                    self.pause_for(READER_CRASHED);
                    return Ok(false);
                }
                ReaderCheck::Unwritable => {
                    self.pause_for(READER_CHECK_UNWRITABLE);
                    return Ok(false);
                }
            }
        }
        let unprovable = doubts
            .iter()
            .filter(|(kind, _)| kind.starts_with("crash:") && *kind != text_crashes)
            .collect::<Vec<_>>();
        if unprovable.len() == doubts.len() {
            if unprovable
                .iter()
                .any(|(_, waiting)| *waiting >= SEVERAL_WAITING_ON_THE_READER)
            {
                self.pause_for(READER_CRASHED);
            }
            return Ok(false);
        }
        self.events.queue_changed();
        Ok(true)
    }

    /// The model's self-test, as setup runs it: the calibration document,
    /// read and checked. `Err(None)` when no model is loaded to ask yet,
    /// which says nothing about the model; `Err(Some(reason))` when it
    /// failed, with the reason the queue stops for.
    fn model_self_test(&self) -> Result<(), Option<String>> {
        let probes =
            semantic_probes().map_err(|_| Some(ErrorCode::ModelFailed.as_str().to_owned()))?;
        for probe in probes {
            let analysis = match self.analyze_with_deadline(&probe.document, "pdf", &[]) {
                Ok(analysis) => analysis,
                Err(error) => {
                    if model_failure_action(&error.code) == ModelFailureAction::Requeue {
                        return Err(None);
                    }
                    self.forget_missing_model();
                    let code = model_error_code(&error);
                    return Err(Some(pause_reason_code(&error.code, code).to_owned()));
                }
            };
            self.forget_missing_model();
            if validate_semantic_probe(&probe, &analysis).is_err() {
                return Err(Some(ErrorCode::ModelFailed.as_str().to_owned()));
            }
        }
        Ok(())
    }

    /// The reader's self-test for text files: a small one Intern writes in
    /// its own data folder, read without a crash. It proves the reader works
    /// for text files and nothing else.
    fn reader_self_test(&self) -> ReaderCheck {
        let path = self.data_folder.join(format!(
            "intern-reader-check-{}.txt",
            self.store.session_id()
        ));
        if fs::write(&path, READER_CHECK_TEXT).is_err() {
            let _ = fs::remove_file(&path);
            return ReaderCheck::Unwritable;
        }
        let request_id = format!("reader-check-{}", self.store.session_id());
        let read = self.worker.extract(&request_id, &path, &mut |_| {});
        let _ = fs::remove_file(&path);
        match read {
            Ok(_) => ReaderCheck::Read,
            Err(error) => {
                if error.crashed {
                    let _ = self.worker.restart();
                }
                ReaderCheck::Failed
            }
        }
    }

    /// Gives a canceled or failed item's file its own name back when Intern
    /// marked it, so the row can be cleared away with the history without
    /// leaving a file every scan skips. A rename that is refused stays for
    /// the recovery pass to try again.
    fn unmark_if_marked(&self, item_id: i64) {
        let _files = self.hold_file_ops();
        let _ = self.files.unmark_settled(item_id);
    }

    /// Renames an unreadable document's file, under the hold every file
    /// operation runs under. A rename that is refused - a viewer holding the
    /// file, a folder this account may not write - is recorded as refused,
    /// and the document still waits in review under its own name, which is
    /// no worse than before it was renamed: so the refusal is not the
    /// queue's to report.
    ///
    /// The input folder is asked again here, not only when the document went
    /// to review: a mark put off until a later success can come due after
    /// the input folder changed in Settings, and Intern renames only what is
    /// in the input folder now. One outside it is recorded as never to be
    /// marked.
    fn mark_unreadable(&self, item_id: i64) {
        let _files = self.hold_file_ops();
        let Ok(Some(item)) = self.store.get(item_id) else {
            return;
        };
        match self.input_folder_holds(&item.source_path) {
            Some(true) => {
                let _ = self.files.mark_for_review(item_id);
            }
            Some(false) => {
                let _ = self.store.withhold_review_mark(item_id);
            }
            // Settings cannot be read to say: the recovery pass decides
            // once they can.
            None => {}
        }
    }

    /// Puts an item an undo returned to ready back in review, as undone: a
    /// copy back where it came from waits as the duplicate it is, so
    /// "Process anyway" stays one click, and nothing files it again.
    fn mark_undone(&self, item_id: i64) -> PipelineResult<()> {
        if self.is_copy(item_id) {
            self.repository.hold_undone_copy(item_id)
        } else {
            self.repository.mark_needs_review(item_id, UNDONE)
        }
    }

    /// Sends a document an undo just put back to review, while the hold of
    /// the operation that did it is still held.
    ///
    /// A finished undo leaves the item ready - the state the scheduler files
    /// from - with the approval of the rename it took back still on its
    /// record. `report_settled` marks it for review, but only after the hold
    /// is released, and a drain that was waiting for that very undo to finish
    /// could pick the document up in between and file it again under the
    /// name just undone. Marked here, any rename of it starts only once it is
    /// already in review, and is refused.
    fn hold_undone_for_review(&self, item_id: i64) {
        let undone = self
            .store
            .load_receipt(item_id)
            .ok()
            .flatten()
            .is_some_and(|receipt| {
                receipt.direction == OperationDirection::Undo
                    && receipt.stage == OperationStage::Complete
            });
        if undone
            && self
                .store
                .get(item_id)
                .ok()
                .flatten()
                .is_some_and(|item| item.status == QueueStatus::Ready)
        {
            let _ = self.mark_undone(item_id);
        }
    }

    /// Reports an operation a reconciliation finished rather than the call
    /// that started it.
    ///
    /// The applier journals an ambiguous apply or undo and settles it
    /// afterwards - on the next retry, on the next recovery pass, or right
    /// here - and nobody used to be told: a document filed that way was never
    /// described and never remembered as a filing, so a second scan of it was
    /// filed all over again, and a document put back that way was still
    /// remembered as filed. What is reported is read from the store, so it is
    /// the same work whichever call finished the operation.
    fn report_settled(&self, item_id: i64) {
        let Ok(Some(item)) = self.store.get(item_id) else {
            return;
        };
        let Ok(Some(receipt)) = self.store.load_receipt(item_id) else {
            return;
        };
        if receipt.stage == OperationStage::RolledBack
            && receipt.direction == OperationDirection::Apply
            && item.status == QueueStatus::NeedsReview
            && let Some(code) = item.error_code
        {
            // A rename that moved nothing and left the document in review,
            // for a reason a person has to act on. A copy that could not be
            // filed waits as the duplicate it is, as it would have with
            // nowhere to file it.
            if self.is_copy(item_id) {
                let _ = self.repository.abandon_copy(item_id);
            } else {
                let _ = self.repository.hold_for_review(item_id, code.as_str());
            }
            return;
        }
        if receipt.stage != OperationStage::Complete {
            return;
        }
        match receipt.direction {
            OperationDirection::Apply if item.status == QueueStatus::Completed => {
                self.report_filed(&item);
            }
            OperationDirection::Undo if item.status != QueueStatus::Completed => {
                // An undo returns the item to ready, which is the state the
                // scheduler files from: with automatic renaming on it would
                // apply the same name again within the minute and undo the
                // person's undo. Taking a decision back is a decision, so the
                // document waits for the next one.
                let copy = self.is_copy(item_id);
                let _ = self.mark_undone(item_id);
                let _ = self.repository.forget_fingerprint(item_id);
                // An undo receipt reads the other way round: it moved the
                // document from the name it was filed under back to where it
                // started, and the vacated name is what a records keeper knows
                // it by. A copy was never reported filed, and its bytes are
                // the original's: retracting it would retract the original's
                // shared marker, which is keyed by those bytes.
                if !copy {
                    self.filing.unfiled(&UnfiledDocument {
                        item_id,
                        source_path: item.source_path.clone(),
                        source_hash: item.source_hash.clone(),
                        destination: receipt.source.clone(),
                    });
                }
                if let Ok(settings) = self.settings.load() {
                    prune_empty_layout_folders(
                        &destination_root(&settings, &item.source_path),
                        &receipt.source,
                        settings.destination_layout,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        self.events.queue_changed();
    }
    pub fn resume(&self) {
        *self
            .pause_reason
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        // A person resuming starts every run of failures over. Documents in
        // doubt stay so until the machine is shown to work; those in doubt
        // over a reader crash, which no self-test can clear, get one more
        // attempt that proves nothing (see `prove_the_machine`).
        let _ = self.store.end_failure_runs(None);
        let _ = self.store.allow_unproven_crash_attempts();
        self.paused.store(false, Ordering::SeqCst);
        self.events.queue_changed();
    }
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Why the queue stopped itself, while it is stopped: a code the window
    /// turns into a sentence. `None` while it runs, and for a pause a person
    /// asked for.
    pub fn pause_reason(&self) -> Option<String> {
        if !self.is_paused() {
            return None;
        }
        self.pause_reason
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Stops the queue for a failure every following document would share,
    /// and keeps the reason. The queue used to stop with nothing to say: the
    /// window heard only that it was paused, and a person resuming it had no
    /// way to know there was a key or an account to fix first.
    fn pause_for(&self, reason: &str) {
        *self
            .pause_reason
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(reason.to_owned());
        self.paused.store(true, Ordering::SeqCst);
    }

    /// Records that a document found no model loaded, and says whether one
    /// has now been missing for longer than a switch or a restart takes.
    ///
    /// The first such document never stops the queue: the slot is empty for
    /// a moment whenever Settings switches models or the local server is
    /// restarted after a cancel. A later one stops it once the first is more
    /// than the grace in the past. Resuming does not start the clock again -
    /// only a model answering does - so a model still missing after a resume
    /// stops the queue at the next document rather than minutes later.
    fn model_missing_too_long(&self) -> bool {
        let mut since = self
            .model_missing_since
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match *since {
            Some(first) => first.elapsed() >= self.model_missing_grace,
            None => {
                *since = Some(std::time::Instant::now());
                false
            }
        }
    }

    /// A model answered, so the slot is filled again.
    fn forget_missing_model(&self) {
        *self
            .model_missing_since
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    /// The hold every file operation and its reconciliation runs under. The
    /// guarded value is nothing, so a holder that panicked leaves nothing to
    /// distrust behind it.
    fn hold_file_ops(&self) -> std::sync::MutexGuard<'_, ()> {
        self.file_ops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn find_item(&self, id: i64) -> PipelineResult<QueueItem> {
        self.store
            .get(id)?
            .ok_or_else(|| PipelineError::new("ITEM_NOT_FOUND", "queue item does not exist"))
    }

    /// The operation an item was left with when a reconciliation could not
    /// settle it. An applying item's unfinished receipt is the operation in
    /// flight, so it never counts.
    fn unsettled_receipt(&self, item: &QueueItem) -> PipelineResult<Option<OperationReceipt>> {
        if item.status == QueueStatus::Applying {
            return Ok(None);
        }
        Ok(self.store.load_unsettled_receipt(item.id)?)
    }

    pub fn shutdown(&self) -> PipelineResult<()> {
        self.shutting_down.store(true, Ordering::SeqCst);
        self.paused.store(true, Ordering::SeqCst);
        let worker = self.worker.shutdown().map_err(worker_error);
        let model = self.model.shutdown().map_err(|error| {
            PipelineError::new(error.code, "local model process could not be stopped")
        });
        self.events.queue_changed();
        worker.and(model)
    }

    pub fn cancel(&self, id: i64) -> PipelineResult<()> {
        self.cancel_marked(id, None)
    }

    /// Cancels on the intake watcher's behalf: the claim on the document went
    /// to another computer, or its uploader could no longer be confirmed. The
    /// row is marked `INTAKE_WITHDRAWN`, on the row itself so the mark
    /// outlives a restart, because a person's cancel leaves the same status
    /// and means the opposite - leave the document alone - where this one
    /// may be handed over and run again. A row already canceled stays the
    /// person's.
    pub fn withdraw(&self, id: i64) -> PipelineResult<()> {
        self.cancel_marked(id, Some(ErrorCode::IntakeWithdrawn))
    }

    fn cancel_marked(&self, id: i64, mark: Option<ErrorCode>) -> PipelineResult<()> {
        let item = self.find_item(id)?;
        if item.status == QueueStatus::NeedsReview
            && let Some(error) = unsettled_files(&item, self.unsettled_receipt(&item)?.as_ref())
        {
            return Err(error);
        }
        match item.status {
            QueueStatus::Queued
            | QueueStatus::Extracting
            | QueueStatus::Analyzing
            | QueueStatus::Ready
            | QueueStatus::NeedsReview => {
                self.store
                    .transition(id, item.status, QueueStatus::Canceled, mark)?;
            }
            QueueStatus::Canceled => {}
            QueueStatus::Applying => {
                return Err(PipelineError::new(
                    "STATE_CONFLICT",
                    "an atomic file operation cannot be canceled",
                ));
            }
            QueueStatus::Failed | QueueStatus::Completed => {
                return Err(PipelineError::new(
                    "INVALID_TRANSITION",
                    "item cannot be canceled",
                ));
            }
        }
        if self.active_item.load(Ordering::SeqCst) == id {
            match item.status {
                QueueStatus::Extracting => {
                    let request_id = format!("queue-{}-{}", id, item.processing_failures + 1);
                    self.worker.cancel(&request_id).map_err(worker_error)?;
                }
                QueueStatus::Analyzing => {
                    self.cancel_active_request();
                    self.model.cancel().map_err(|error| {
                        PipelineError::new(error.code, "local model request could not be canceled")
                    })?
                }
                _ => {}
            }
        }
        // A canceled row can be cleared away with the history, so a file
        // Intern marked gets its own name back now.
        self.unmark_if_marked(id);
        self.events.queue_changed();
        Ok(())
    }

    pub fn retry(&self, id: i64) -> PipelineResult<()> {
        let item = self.find_item(id)?;
        // For a document whose files were left unsettled, the only thing
        // Retry can mean is "look at them again": the deletion a sync client
        // was blocking, the rename a program holding the file refused, the two
        // names a person has since sorted out by hand. Whatever stage the
        // operation reached.
        if item.status == QueueStatus::NeedsReview
            && (item.error_code == Some(ErrorCode::SourceDeleteFailed)
                || self.unsettled_receipt(&item)?.is_some())
        {
            return self.check_again(id);
        }
        if item.status == QueueStatus::NeedsReview
            && item.error_code == Some(ErrorCode::UploaderUnverified)
        {
            // The denial happened before any analysis, so there is no
            // proposal to review and nothing for a person to approve: the
            // only useful thing Retry can mean is "the file is verified now,
            // read it". Same compare-and-swap as the duplicate shortcut.
            self.repository.retry_unverified(id)?;
            self.events.queue_changed();
            return Ok(());
        }
        if item.status == QueueStatus::NeedsReview && item.error_code == Some(ErrorCode::Duplicate)
        {
            // "Process anyway": the duplicate flag was set before any
            // analysis, so clearing it simply returns the item to the queue
            // for a normal run.
            self.store.retry_duplicate(id)?;
            self.events.queue_changed();
            return Ok(());
        }
        if item.status == QueueStatus::NeedsReview
            && item.error_code.is_some_and(ErrorCode::marks_unreadable)
        {
            // A document Intern could not read is read again from the start:
            // the password removed, the file saved again, the model working
            // now. It is read from wherever its file is - under its
            // NEEDS REVIEW name, once it has one - and taken back to the
            // queue under the file hold, so a rename of it still in flight
            // finishes before it is read. It is fingerprinted again as it
            // is now: removing a password changes the file, and the rename
            // after the new reading is checked against that.
            {
                let _files = self.hold_file_ops();
                let current = self.find_item(id)?;
                let fingerprint = self.files.fingerprint(&current.source_path)?;
                let new_hash = (fingerprint != current.source_hash).then_some(fingerprint.as_str());
                self.store.retry_unreadable(id, new_hash)?;
            }
            self.events.queue_changed();
            return Ok(());
        }
        match item.status {
            QueueStatus::Failed => {
                self.store.manual_retry(id)?;
            }
            QueueStatus::Canceled => self.repository.retry_canceled(id)?,
            _ => {
                return Err(PipelineError::new(
                    "INVALID_TRANSITION",
                    "only failed or canceled items can be retried",
                ));
            }
        }
        self.repository.delete_proposal(id)?;
        self.events.queue_changed();
        Ok(())
    }

    /// Reads a document waiting on a person again, from the start:
    /// Re-analyze.
    ///
    /// What a person needs once the document itself has changed - signed
    /// after it was read, edited, scanned again over itself - and for any
    /// reading they would rather have done again than correct by hand. The
    /// file is fingerprinted now and the item takes the new fingerprint when
    /// it changed, so the new reading is of the file as it is, and the rename
    /// that follows checks against that. What the earlier reading left goes:
    /// the proposal and any approval in it, and the text fingerprint a
    /// filing of it left for near-duplicate checks.
    pub fn reanalyze(&self, id: i64) -> PipelineResult<()> {
        let item = self.find_item(id)?;
        if !matches!(item.status, QueueStatus::NeedsReview | QueueStatus::Ready) {
            return Err(PipelineError::new(
                "INVALID_TRANSITION",
                "only a document waiting for review can be analyzed again",
            ));
        }
        let fingerprint = self.files.fingerprint(&item.source_path)?;
        let new_hash = (fingerprint != item.source_hash).then_some(fingerprint.as_str());
        self.store.requeue_for_analysis(id, item.status, new_hash)?;
        self.repository.forget_fingerprint(id)?;
        self.events.queue_changed();
        Ok(())
    }

    /// Takes an item out of the queue, with its proposal and receipts.
    ///
    /// An item whose files were left unsettled is removed only once the
    /// person has `confirmed` they resolved the files themselves: removing it
    /// otherwise decides what the files are on their behalf, and with the
    /// receipts goes the only record of what is where. Without that there was
    /// no way out at all for an operation reconciliation could never prove -
    /// every action refused it, even after the files were sorted by hand.
    pub fn remove(&self, id: i64, confirmed: bool) -> PipelineResult<()> {
        if !confirmed {
            self.reject_deferred_reconciliation_mutation(id)?;
        }
        self.repository.remove_item(id)?;
        self.events.queue_changed();
        Ok(())
    }

    pub fn approve(&self, id: i64, filename: &str, description: &str) -> PipelineResult<()> {
        // Applying a staged copy under its own COPY name is filing it: it was
        // never going to be named by anyone, and its name carries no date.
        if let Some(record) = self
            .repository
            .load_proposal(id)?
            .filter(|record| record.copy_of.is_some() && record.filename == filename.trim())
        {
            let item = self.find_item(id)?;
            if item.status == QueueStatus::Ready && record.approved {
                return self.apply_copy(&item, &record);
            }
        }
        let filename = validate_leaf_filename(filename)?;
        if leading_date(&filename).is_none() {
            return Err(PipelineError::new(
                DATE_REQUIRED,
                "the filename must start with the document's date as YYYY-MM-DD",
            ));
        }
        let mut item = self.find_item(id)?;
        // A rename that never finished stops any new one being journalled,
        // and approving again - once the program holding the file has let go
        // - is exactly how a person asks for it to be settled. So it is
        // checked first, rather than left to fail the apply every time.
        if matches!(item.status, QueueStatus::NeedsReview | QueueStatus::Ready)
            && self.unsettled_receipt(&item)?.is_some()
        {
            self.check_again(id)?;
            item = self.find_item(id)?;
            if item.status == QueueStatus::Completed {
                // The earlier operation had in fact landed, or an undo of it
                // never did: the document is filed, under the name that
                // rename gave it.
                return self.already_filed(&item, &filename, description);
            }
        }
        if let Some(error) = unsettled_files(&item, self.unsettled_receipt(&item)?.as_ref()) {
            return Err(error);
        }
        let source_extension = item
            .source_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let approved_extension = Path::new(&filename)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if source_extension.is_empty() || !source_extension.eq_ignore_ascii_case(approved_extension)
        {
            return Err(PipelineError::new(
                "NAME_INVALID",
                "approved filename must preserve the source extension",
            ));
        }
        if !matches!(item.status, QueueStatus::NeedsReview | QueueStatus::Ready) {
            return Err(PipelineError::new(
                "INVALID_TRANSITION",
                "proposal is not reviewable",
            ));
        }
        if let Err(error) = self.authorize_item(&item, AdmissionStage::Apply) {
            if !error.is_retryable() {
                let _ = self
                    .repository
                    .mark_needs_review(item.id, "UPLOADER_UNVERIFIED");
                self.events.queue_changed();
            }
            return Err(error);
        }
        let proposed = self.repository.load_proposal(id)?;
        if proposed.is_none()
            && item.status == QueueStatus::NeedsReview
            && item.error_code.is_some_and(ErrorCode::marks_unreadable)
        {
            // A document Intern could not read has no proposal to approve,
            // so the name the person typed becomes one: approved like any
            // other, under the same date rule, and filed into the output
            // folder the same way - which leaves its NEEDS REVIEW name
            // behind with the input folder. Nothing is learned from it;
            // there was no proposal for it to correct.
            self.repository.create_manual_proposal(id)?;
        }
        if proposed
            .as_ref()
            .is_some_and(|record| record.copy_of.is_some())
        {
            // A person naming a duplicate - one staged as a copy, or a copy
            // they undid - is filing it as a document of its own, under
            // their name: it is a copy no longer, and is recorded like any
            // other filing.
            self.repository.release_copy(id)?;
        }
        self.repository
            .approve_user_edit(id, item.status, &filename, description)?;
        // A preference store, not a filing step: a lesson that cannot be
        // written must not stop the rename that was just approved. A name
        // with no reading behind it corrects nothing.
        if let Some(record) = proposed.as_ref().filter(|record| !record.is_synthetic()) {
            let _ = self.learn_from_edit(record, source_extension, &filename);
        }
        let ready = self.store.get(id)?.ok_or_else(|| {
            PipelineError::new("ITEM_NOT_FOUND", "queue item disappeared during approval")
        })?;
        let settings = match self.settings.load() {
            Ok(settings) => settings,
            Err(error) => {
                self.repository.mark_needs_review(id, &error.code)?;
                self.events.queue_changed();
                return Err(error);
            }
        };
        self.apply_if_unchanged(&ready, &filename, &settings)
    }

    /// What approving says when checking the files again finished the earlier
    /// operation instead, and the document is filed already.
    ///
    /// Approval of exactly what the earlier rename was filing - the name and
    /// sentence on the record, or the name the file actually took - is done.
    /// Anything the person changed was not applied, and reporting success
    /// told them it had been: the window said "Rename applied" over a
    /// document still filed under the old name, with the new name and
    /// sentence dropped and nothing learned from them. Renaming a filed
    /// document is an undo and a new approval, and the refusal says so.
    fn already_filed(
        &self,
        item: &QueueItem,
        filename: &str,
        description: &str,
    ) -> PipelineResult<()> {
        let filed_name = self
            .store
            .load_latest_complete_receipt(item.id)?
            .filter(|receipt| receipt.direction == OperationDirection::Apply)
            .and_then(|receipt| {
                receipt
                    .destination
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            });
        let unchanged = self
            .repository
            .load_proposal(item.id)?
            .is_some_and(|record| {
                (record.filename == filename || filed_name.as_deref() == Some(filename))
                    && record.description == description.trim()
            });
        if unchanged {
            return Ok(());
        }
        let filed_as = filed_name.unwrap_or_else(|| {
            item.source_path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
        Err(PipelineError::new(
            "ALREADY_FILED",
            format!(
                "The earlier rename had already finished, so the document is filed as {filed_as} and your changes were not applied. Undo it to rename it again."
            ),
        ))
    }

    /// Completes a document under the name it has, without filing it.
    ///
    /// A document Intern renamed `NEEDS REVIEW - ...` because it could not
    /// read it gets its own name back first - the name the mark took away,
    /// or the next ` (2)` when another file has taken it since - in the same
    /// journaled step that completes it, so a crash can never leave a kept
    /// document under the review prefix, or one in review without it. A
    /// rename that is refused (the file is open somewhere) completes
    /// nothing, and says why.
    pub fn keep_original(&self, id: i64) -> PipelineResult<()> {
        let item = self.find_item(id)?;
        if let Some(error) = unsettled_files(&item, self.unsettled_receipt(&item)?.as_ref()) {
            return Err(error);
        }
        if !matches!(item.status, QueueStatus::NeedsReview | QueueStatus::Ready) {
            self.store.complete_keep_original(id, item.status)?;
            return Ok(());
        }
        let restored = {
            let _files = self.hold_file_ops();
            self.files.restore_from_review(id)?
        };
        if !restored {
            self.store.complete_keep_original(id, item.status)?;
        }
        self.events.queue_changed();
        Ok(())
    }

    pub fn undo(&self, id: i64) -> PipelineResult<()> {
        let item = self.find_item(id)?;
        if item.status != QueueStatus::Completed {
            return Err(PipelineError::new(
                "INVALID_TRANSITION",
                "only completed operations can be undone",
            ));
        }
        // The apply that filed the document, not the newest receipt: an undo
        // refused and rolled back earlier sits on top of it having moved
        // nothing.
        let receipt = self
            .store
            .load_latest_complete_receipt(id)?
            .filter(|receipt| receipt.direction == OperationDirection::Apply)
            .ok_or_else(|| {
                PipelineError::new(
                    "STATE_CONFLICT",
                    "completed item has no durable operation receipt",
                )
            })?;
        let outcome = {
            let _files = self.hold_file_ops();
            let outcome = self.files.undo(&item, &receipt);
            if outcome.is_err() {
                // A journalled undo that failed leaves the item applying, and
                // one applying row stops every other document - until a
                // recovery pass that steady intake could put off for good.
                // Settle it now, as a failed apply is.
                let _ = self.files.reconcile(&item);
            }
            self.hold_undone_for_review(id);
            outcome
        };
        // An undo the applier journalled can be finished by a reconciliation
        // even when the call itself reports a failure, so what settles this is
        // the store rather than the return value.
        self.report_settled(id);
        self.events.queue_changed();
        match outcome {
            Ok(()) => Ok(()),
            Err(_) if self.is_undone(id) => Ok(()),
            Err(error) => {
                let filed = self.files.try_exists(&receipt.destination);
                Err(undo_failure(error, &receipt, filed))
            }
        }
    }

    /// Whether the item's newest finished operation is an undo: the document
    /// is back where it started, whichever call got it there.
    fn is_undone(&self, id: i64) -> bool {
        self.store
            .load_latest_complete_receipt(id)
            .ok()
            .flatten()
            .is_some_and(|receipt| receipt.direction == OperationDirection::Undo)
    }

    /// Looks again at a file operation that never finished - "Check again",
    /// which retry and approve run too.
    ///
    /// The operation is put back in `applying` bound to its receipt, and the
    /// same reconciliation recovery runs settles it: a rename that never
    /// happened is rolled back, one that did is finished, and one that still
    /// cannot be proven is parked again with the reason. Before this, nothing
    /// ever looked at a parked receipt again, and its unfinished stage made
    /// every later apply of the document fail to journal.
    pub fn check_again(&self, id: i64) -> PipelineResult<()> {
        let result = {
            let _files = self.hold_file_ops();
            let reattached =
                self.store
                    .reattach_unsettled_receipt(id)
                    .map_err(|error| match error.code() {
                        ErrorCode::StateConflict => PipelineError::new(
                            "STATE_CONFLICT",
                            "another file operation is in progress; check again in a moment",
                        ),
                        _ => error.into(),
                    })?;
            if reattached.is_none() {
                return Err(PipelineError::new(
                    "INVALID_TRANSITION",
                    "no file operation of this document is waiting to be checked",
                ));
            }
            let item = self.find_item(id)?;
            let result = self.files.reconcile(&item);
            self.hold_undone_for_review(id);
            // Under the same hold, so no rename of it can start in the
            // moment it reads as ready.
            let kept = if self
                .store
                .get(id)?
                .is_some_and(|settled| settled.status == QueueStatus::Ready)
            {
                self.repository.keep_in_review(id)
            } else {
                Ok(())
            };
            result.and(kept)
        };
        self.report_settled(id);
        self.events.queue_changed();
        result
    }

    pub fn clear_history(&self) -> PipelineResult<usize> {
        // Names first: a canceled or failed document whose file still
        // carries Intern's mark gets its name back, and one whose name is
        // refused back (a viewer holding the file) stays in the history
        // until it can be (see `QueueStore::clear_terminal`).
        if let Ok(settled) = self.store.marked_but_settled() {
            for item in settled {
                self.unmark_if_marked(item.id);
            }
        }
        let removed = self.store.clear_terminal()?;
        self.events.queue_changed();
        Ok(removed)
    }

    /// Abandons everything still waiting, for the folder that was chosen by
    /// mistake. Renames already applied keep their receipts, and anything
    /// awaiting a human decision stays put.
    pub fn discard_waiting(&self) -> PipelineResult<usize> {
        let removed = self.store.discard_queued()?;
        self.events.queue_changed();
        Ok(removed)
    }

    fn reject_deferred_reconciliation_mutation(&self, id: i64) -> PipelineResult<()> {
        let item = self.find_item(id)?;
        if item.status == QueueStatus::NeedsReview
            && let Some(error) = unsettled_files(&item, self.unsettled_receipt(&item)?.as_ref())
        {
            return Err(error);
        }
        Ok(())
    }

    pub fn recover(&self) -> PipelineResult<()> {
        {
            // Renames in place for documents that could not be read: one a
            // crash left between the rename and its record, and one that
            // never happened at all. Under the file hold, like every rename.
            // One that cannot be settled now stays journaled for the next
            // pass, and is no reason to leave the rest of recovery undone.
            let _files = self.hold_file_ops();
            // A mark recovery would make up for, on a file the input folder
            // no longer holds, is never made. With Settings unreadable
            // nobody can say which files those are, so this waits for a
            // pass that can.
            if self.settings.load().is_ok() {
                if let Ok(awaiting) = self.store.awaiting_review_mark() {
                    for item in awaiting {
                        if self.input_folder_holds(&item.source_path) == Some(false) {
                            let _ = self.store.withhold_review_mark(item.id);
                        }
                    }
                }
                let _ = self.files.reconcile_review_renames();
            }
        }
        let interrupted = self
            .store
            .list()?
            .into_iter()
            .filter(|item| {
                matches!(
                    item.status,
                    QueueStatus::Extracting | QueueStatus::Analyzing
                )
            })
            .map(|item| item.id)
            .collect::<Vec<_>>();
        self.store.recover_interrupted()?;
        for id in interrupted {
            if self
                .store
                .get(id)?
                .is_some_and(|item| item.status == QueueStatus::Queued)
            {
                self.repository.record_recovered_failure(id)?;
            }
        }
        let applying = self
            .store
            .list()?
            .into_iter()
            .filter(|item| item.status == QueueStatus::Applying)
            .map(|item| item.id)
            .collect::<Vec<_>>();
        for id in applying {
            {
                // An approve, undo or check on another thread may be in the
                // middle of this very operation, with the same session, so
                // nothing in the store tells the two apart. Wait for it, then
                // look at what it left: if it settled the item, there is
                // nothing to recover, and whoever settled it reported it.
                let _files = self.hold_file_ops();
                let Some(item) = self
                    .store
                    .get(id)?
                    .filter(|item| item.status == QueueStatus::Applying)
                else {
                    continue;
                };
                // A still-running app may own the ambiguous operation. Its file boundary can
                // reconcile under that lease without waiting for its own session to go stale.
                if self.files.reconcile(&item).is_err()
                    && let Ok(claimed) = self.store.claim_applying_reconciliation(item.id)
                {
                    let _ = self.files.reconcile(&claimed);
                }
                self.hold_undone_for_review(id);
            }
            self.report_settled(id);
        }
        // Rows this pass failed after their file was marked, or that a
        // crash left so: their files get their own names back.
        if let Ok(settled) = self.store.marked_but_settled() {
            for item in settled {
                self.unmark_if_marked(item.id);
            }
        }
        self.events.queue_changed();
        Ok(())
    }

    /// Records which item a claim of the intake watcher handed over (see
    /// `QueueStore::record_intake_claim`).
    pub fn record_intake_claim(&self, claim_key: &str, item_id: i64) -> PipelineResult<()> {
        Ok(self.store.record_intake_claim(claim_key, item_id)?)
    }

    /// The item a claim handed over, with its proposal and receipts:
    /// `None` for a claim never recorded, `Some(None)` for one whose item is
    /// gone.
    pub fn find_by_intake_claim(
        &self,
        claim_key: &str,
    ) -> PipelineResult<Option<Option<PipelineItem>>> {
        match self.store.find_by_intake_claim(claim_key)? {
            None => Ok(None),
            Some(None) => Ok(Some(None)),
            Some(Some(item)) => Ok(Some(Some(self.pipeline_item(item)?))),
        }
    }
}

/// What a failed undo tells the person who asked for it, given whether the
/// filed document is still where it was filed (`filed_present`).
///
/// The window shows the message as it is, and the applier's own words were
/// written for a log: a filed document someone moved read as an unavailable
/// destination volume, and a moment of contention as a compare-and-swap. Only
/// a document the file system says is not there was moved or deleted; one it
/// could not look for - an offline network share, a folder it may not read -
/// is still filed as far as anyone knows, and saying otherwise could send a
/// person off to clear the history of a filing that is intact.
fn undo_failure(
    error: PipelineError,
    filed: &OperationReceipt,
    filed_present: std::io::Result<bool>,
) -> PipelineError {
    match filed_present {
        Ok(false) => PipelineError::new(
            "FILE_CHANGED",
            format!(
                "The filed document is no longer at {}; it was moved or deleted.",
                display_path(&filed.destination)
            ),
        ),
        Err(_) => PipelineError::new(
            error.code,
            format!(
                "The filed document at {} could not be reached. Check that its folder is available, then try Undo again.",
                display_path(&filed.destination)
            ),
        ),
        Ok(true) if error.code == "STATE_CONFLICT" => PipelineError::new(
            "STATE_CONFLICT",
            "Another rename is in progress. Try Undo again in a moment.",
        ),
        Ok(true) => error,
    }
}

/// Why an item's files must be settled before anything else moves them, or
/// `None` when nothing is outstanding.
///
/// Both codes mean two files are on disk where one document should be: a
/// verified copy whose original could not be deleted, and a reconciliation
/// that could not tell which of the two the document is. Any other
/// unfinished receipt is an operation nobody has looked at since it failed,
/// and it stops the next rename from being journalled. Renaming, cancelling
/// or removing such an item would decide what the files are on the person's
/// behalf, so each of those refuses and says which question is open - by
/// name, rather than letting an apply fail on it later as a conflict.
fn unsettled_files(
    item: &QueueItem,
    unsettled: Option<&OperationReceipt>,
) -> Option<PipelineError> {
    match item.error_code {
        Some(ErrorCode::SourceDeleteFailed) => Some(PipelineError::new(
            "RECONCILIATION_REQUIRED",
            "retry the verified source deletion or resolve the files manually",
        )),
        Some(ErrorCode::ReconciliationRequired) => Some(PipelineError::new(
            "RECONCILIATION_REQUIRED",
            "reconciliation could not tell which file is the document; compare both names",
        )),
        _ => unsettled.map(|_| {
            PipelineError::new(
                "RECONCILIATION_REQUIRED",
                "an earlier rename of this document did not finish, so its files need checking: use Check again",
            )
        }),
    }
}

struct PipelineRepository {
    connection: Mutex<Connection>,
}

impl PipelineRepository {
    fn open(path: &Path) -> PipelineResult<Self> {
        let mut connection = Connection::open(path).map_err(database_error)?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(database_error)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON;
                 CREATE TABLE IF NOT EXISTS house_rules (
                   id INTEGER PRIMARY KEY AUTOINCREMENT,
                   kind TEXT NOT NULL,
                   from_key TEXT NOT NULL,
                   from_value TEXT NOT NULL,
                   to_value TEXT NOT NULL,
                   seen INTEGER NOT NULL DEFAULT 1,
                   created_at INTEGER NOT NULL,
                   updated_at INTEGER NOT NULL,
                   UNIQUE(kind, from_key)
                 );
                 CREATE TABLE IF NOT EXISTS fingerprints (
                   queue_item_id INTEGER PRIMARY KEY REFERENCES queue_items(id) ON DELETE CASCADE,
                   fingerprint INTEGER NOT NULL,
                   filed_name TEXT NOT NULL,
                   filed_at INTEGER NOT NULL
                 );",
            )
            .map_err(database_error)?;
        let legacy_exists = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'pipeline_proposals')",
            [], |row| row.get::<_, bool>(0),
        ).map_err(database_error)?;
        if legacy_exists {
            let transaction = connection.transaction().map_err(database_error)?;
            transaction
                .execute(
                    "INSERT OR REPLACE INTO proposals(queue_item_id, proposal_json, created_at)
                 SELECT queue_item_id, record_json, unixepoch() FROM pipeline_proposals",
                    [],
                )
                .map_err(database_error)?;
            transaction
                .execute("DROP TABLE pipeline_proposals", [])
                .map_err(database_error)?;
            transaction.commit().map_err(database_error)?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn save_initial_and_transition(
        &self,
        id: i64,
        record: &ProposalRecord,
        next: QueueStatus,
        session_id: &str,
    ) -> PipelineResult<()> {
        let json = serde_json::to_string(record)
            .map_err(|_| PipelineError::new("INVALID_DATA", "proposal could not be stored"))?;
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(database_error)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items SET status = ?1, owner_session = NULL, lease_expires_at = NULL,
             error_code = NULL, updated_at = unixepoch()
             WHERE id = ?2 AND status = 'analyzing' AND owner_session = ?3",
                params![queue_status_text(next), id, session_id],
            )
            .map_err(database_error)?;
        if changed != 1 {
            return Err(PipelineError::new(
                "STATE_CONFLICT",
                "analyzing proposal transition compare-and-swap failed",
            ));
        }
        transaction.execute(
            "INSERT INTO proposals(queue_item_id, proposal_json, created_at) VALUES (?1, ?2, unixepoch())
             ON CONFLICT(queue_item_id) DO UPDATE SET proposal_json = excluded.proposal_json, created_at = excluded.created_at",
            params![id, json],
        ).map_err(database_error)?;
        transaction.commit().map_err(database_error)
    }

    fn load_proposal(&self, id: i64) -> PipelineResult<Option<ProposalRecord>> {
        let json = self
            .lock()?
            .query_row(
                "SELECT proposal_json FROM proposals WHERE queue_item_id = ?1",
                params![id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(database_error)?;
        json.map(|value| {
            serde_json::from_str(&value)
                .map_err(|_| PipelineError::new("INVALID_DATA", "stored proposal is invalid"))
        })
        .transpose()
    }

    /// Stores a record over the one a queue item already has.
    fn replace_proposal(&self, id: i64, record: &ProposalRecord) -> PipelineResult<()> {
        let json = serde_json::to_string(record)
            .map_err(|_| PipelineError::new("INVALID_DATA", "proposal could not be stored"))?;
        let updated = self
            .lock()?
            .execute(
                "UPDATE proposals SET proposal_json = ?1 WHERE queue_item_id = ?2",
                params![json, id],
            )
            .map_err(database_error)?;
        if updated != 1 {
            return Err(PipelineError::new("INVALID_DATA", "proposal is missing"));
        }
        Ok(())
    }

    fn list_rules(&self) -> PipelineResult<Vec<LearnedRule>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, kind, from_value, to_value, seen, updated_at FROM house_rules
                 ORDER BY updated_at DESC, id DESC",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, u32>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(database_error)?;
        let mut rules = Vec::new();
        for row in rows {
            let (id, kind, from, to, seen, learned_at) = row.map_err(database_error)?;
            let Some(kind) = RuleKind::parse(&kind) else {
                continue;
            };
            rules.push(LearnedRule {
                id,
                kind,
                from,
                to,
                seen,
                active: seen >= EDITS_TO_LEARN,
                learned_at,
            });
        }
        Ok(rules)
    }

    /// The rules in force: learned often enough, or told to be used.
    fn active_style(&self) -> PipelineResult<HouseStyle> {
        Ok(HouseStyle::new(
            self.list_rules()?
                .into_iter()
                .filter(|rule| rule.active)
                .map(|rule| rule.rule())
                .collect(),
        ))
    }

    /// Records one respelling. The same change again counts it up; a
    /// different spelling for the same word starts the count over.
    fn learn(&self, rule: &HouseRule) -> PipelineResult<LearnedRule> {
        let connection = self.lock()?;
        connection
            .execute(
                "INSERT INTO house_rules(kind, from_key, from_value, to_value, seen, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 1, unixepoch(), unixepoch())
                 ON CONFLICT(kind, from_key) DO UPDATE SET
                   seen = CASE WHEN to_value = excluded.to_value THEN seen + 1 ELSE 1 END,
                   to_value = excluded.to_value,
                   from_value = excluded.from_value,
                   updated_at = unixepoch()",
                params![rule.kind.as_str(), rule.from_key(), rule.from, rule.to],
            )
            .map_err(database_error)?;
        connection
            .query_row(
                "SELECT id, from_value, to_value, seen, updated_at FROM house_rules
                 WHERE kind = ?1 AND from_key = ?2",
                params![rule.kind.as_str(), rule.from_key()],
                |row| {
                    Ok(LearnedRule {
                        id: row.get(0)?,
                        kind: rule.kind,
                        from: row.get(1)?,
                        to: row.get(2)?,
                        seen: row.get(3)?,
                        active: row.get::<_, u32>(3)? >= EDITS_TO_LEARN,
                        learned_at: row.get(4)?,
                    })
                },
            )
            .map_err(database_error)
    }

    /// Keeps the text fingerprint of a document just filed, under the name
    /// it was filed as, so a later document saying the same thing can be
    /// told so. Cleared with the item when the history is cleared.
    fn remember_fingerprint(
        &self,
        item_id: i64,
        fingerprint: u64,
        filed_name: &str,
    ) -> PipelineResult<()> {
        self.lock()?
            .execute(
                "INSERT INTO fingerprints(queue_item_id, fingerprint, filed_name, filed_at)
                 VALUES (?1, ?2, ?3, unixepoch())
                 ON CONFLICT(queue_item_id) DO UPDATE SET
                   fingerprint = excluded.fingerprint,
                   filed_name = excluded.filed_name,
                   filed_at = excluded.filed_at",
                params![item_id, fingerprint as i64, filed_name],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn forget_fingerprint(&self, item_id: i64) -> PipelineResult<()> {
        self.lock()?
            .execute(
                "DELETE FROM fingerprints WHERE queue_item_id = ?1",
                params![item_id],
            )
            .map_err(database_error)?;
        Ok(())
    }

    /// The filings within the near-duplicate distance of `fingerprint`,
    /// closest first, other than `except_item`'s own.
    ///
    /// All of them, not only the closest. Last year's renewal of an agreement
    /// can be nearer in text than this year's second scan of it is, and
    /// answering with that one alone hid the filing the document really
    /// repeats behind a date that says "another document".
    fn find_similar(
        &self,
        fingerprint: u64,
        except_item: i64,
    ) -> PipelineResult<Vec<SimilarFiling>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare("SELECT fingerprint, filed_name FROM fingerprints WHERE queue_item_id <> ?1")
            .map_err(database_error)?;
        let rows = statement
            .query_map(params![except_item], |row| {
                Ok((row.get::<_, i64>(0)? as u64, row.get::<_, String>(1)?))
            })
            .map_err(database_error)?;
        let mut similar = Vec::new();
        for row in rows {
            let (stored, filed_name) = row.map_err(database_error)?;
            let distance = fingerprint::hamming(fingerprint, stored);
            if distance <= NEAR_DUPLICATE_DISTANCE {
                similar.push(SimilarFiling {
                    filing: KnownFiling {
                        filename: filed_name,
                        filed_by: None,
                    },
                    distance,
                });
            }
        }
        similar.sort_by_key(|candidate| candidate.distance);
        Ok(similar)
    }

    fn forget_rule_for(&self, kind: RuleKind, from: &str) -> PipelineResult<()> {
        self.lock()?
            .execute(
                "DELETE FROM house_rules WHERE kind = ?1 AND from_key = ?2",
                params![kind.as_str(), HouseRule::key(from)],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn forget_rule(&self, id: i64) -> PipelineResult<bool> {
        let removed = self
            .lock()?
            .execute("DELETE FROM house_rules WHERE id = ?1", params![id])
            .map_err(database_error)?;
        Ok(removed == 1)
    }

    fn use_rule(&self, id: i64) -> PipelineResult<bool> {
        let changed = self
            .lock()?
            .execute(
                "UPDATE house_rules SET seen = MAX(seen, ?2), updated_at = unixepoch() WHERE id = ?1",
                params![id, EDITS_TO_LEARN],
            )
            .map_err(database_error)?;
        Ok(changed == 1)
    }

    fn delete_proposal(&self, id: i64) -> PipelineResult<()> {
        self.lock()?
            .execute(
                "DELETE FROM proposals WHERE queue_item_id = ?1",
                params![id],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn mark_needs_review(&self, id: i64, reason: &str) -> PipelineResult<()> {
        self.put_in_review(id, Some(reason), QueueStatus::Ready)
    }

    /// Makes the proposal of an item a file operation left in review say so:
    /// waiting for a person, no longer approved, and why.
    ///
    /// A rename that rolled back because the original changed, was held, or
    /// found someone else's file at its name moves the row to review in the
    /// same transaction that records the rollback - which knows nothing of
    /// proposals. The record went on saying "ready" and "approved", so the
    /// window showed whatever reasons it held instead of the one that
    /// mattered, and house-style changes passed over it as an approval
    /// waiting to be filed, leaving a name built under the old rules.
    fn hold_for_review(&self, id: i64, reason: &str) -> PipelineResult<()> {
        self.put_in_review(id, Some(reason), QueueStatus::NeedsReview)
    }

    /// Puts an item a check of its files left ready back in review, when its
    /// proposal was waiting on a person all along.
    ///
    /// Checking again puts a rename that never happened back the way it was
    /// before the rename - for every rename this build makes, ready, because
    /// only a ready document is renamed. A receipt older builds left behind
    /// can belong to a document read again since, whose new proposal was
    /// sent to review: low confidence, a near duplicate. Left ready, it would
    /// be filed under a name nobody approved.
    fn keep_in_review(&self, id: i64) -> PipelineResult<()> {
        if self
            .load_proposal(id)?
            .is_some_and(|record| record.status == ProposalStatus::NeedsReview)
        {
            self.put_in_review(id, None, QueueStatus::Ready)?;
        }
        Ok(())
    }

    /// Stages an exact duplicate to be filed as a copy: its copy record, the
    /// row ready, and the mark that keeps every later duplicate from being
    /// named after it - in one transaction, from queued, so a document the
    /// scheduler has claimed in the meantime is analyzed as usual instead.
    fn stage_copy(&self, id: i64, record: &ProposalRecord) -> PipelineResult<()> {
        let json = serde_json::to_string(record)
            .map_err(|_| PipelineError::new("INVALID_DATA", "proposal could not be stored"))?;
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(database_error)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items SET status = 'ready', error_code = NULL, copy_filing = 1,
                     updated_at = unixepoch()
                 WHERE id = ?1 AND status = 'queued'",
                params![id],
            )
            .map_err(database_error)?;
        if changed != 1 {
            return Err(PipelineError::new(
                "STATE_CONFLICT",
                "the duplicate was claimed before it could be filed as a copy",
            ));
        }
        transaction
            .execute(
                "INSERT INTO proposals(queue_item_id, proposal_json, created_at) VALUES (?1, ?2, unixepoch())
                 ON CONFLICT(queue_item_id) DO UPDATE SET proposal_json = excluded.proposal_json, created_at = excluded.created_at",
                params![id, json],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)
    }

    /// Gives up filing a staged copy: the document waits in review as a
    /// duplicate, untouched and with no record, exactly as a duplicate with
    /// nowhere to file it does - and "Process anyway" is one click.
    fn abandon_copy(&self, id: i64) -> PipelineResult<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(database_error)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items SET status = 'needs_review', error_code = 'DUPLICATE',
                     copy_filing = 0, updated_at = unixepoch()
                 WHERE id = ?1 AND copy_filing = 1 AND status IN ('ready', 'needs_review')",
                params![id],
            )
            .map_err(database_error)?;
        if changed != 1 {
            return Err(PipelineError::new(
                "STATE_CONFLICT",
                "the copy changed before it could be returned to review",
            ));
        }
        transaction
            .execute(
                "DELETE FROM proposals WHERE queue_item_id = ?1",
                params![id],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)
    }

    /// Makes a copy's record an ordinary hand-named one, and the row no copy:
    /// for a duplicate a person names and files as a document of its own.
    fn release_copy(&self, id: i64) -> PipelineResult<()> {
        let mut record = self
            .load_proposal(id)?
            .ok_or_else(|| PipelineError::new("INVALID_DATA", "proposal is missing"))?;
        record.copy_of = None;
        record.manual = true;
        record.reasons.retain(|reason| reason != EXACT_COPY);
        let json = serde_json::to_string(&record)
            .map_err(|_| PipelineError::new("INVALID_DATA", "proposal could not be stored"))?;
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(database_error)?;
        transaction
            .execute(
                "UPDATE queue_items SET copy_filing = 0, updated_at = unixepoch() WHERE id = ?1",
                params![id],
            )
            .map_err(database_error)?;
        transaction
            .execute(
                "UPDATE proposals SET proposal_json = ?1 WHERE queue_item_id = ?2",
                params![json, id],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)
    }

    /// Puts a copy an undo returned to ready in review, as the undone
    /// duplicate it is: flagged DUPLICATE so "Process anyway" applies, and
    /// with its record no longer approved, so nothing files it again.
    fn hold_undone_copy(&self, id: i64) -> PipelineResult<()> {
        self.put_in_review_as(id, Some(UNDONE), QueueStatus::Ready, Some("DUPLICATE"))
    }

    /// Puts a ready row in review under `code` without touching its record:
    /// the fallback when the record itself cannot be updated.
    fn hold_row_for_review(&self, id: i64, code: ErrorCode) -> PipelineResult<()> {
        self.lock()?
            .execute(
                "UPDATE queue_items SET status = 'needs_review', error_code = ?2,
                     updated_at = unixepoch()
                 WHERE id = ?1 AND status = 'ready'",
                params![id, code.as_str()],
            )
            .map_err(database_error)?;
        Ok(())
    }

    /// Gives a document Intern could not read the empty record a typed name
    /// is approved into. Nothing is replaced: a record already there - a
    /// name typed before - is the one approved.
    fn create_manual_proposal(&self, id: i64) -> PipelineResult<()> {
        let mut record = ProposalRecord::synthetic(String::new(), String::new());
        record.manual = true;
        let json = serde_json::to_string(&record)
            .map_err(|_| PipelineError::new("INVALID_DATA", "proposal could not be stored"))?;
        self.lock()?
            .execute(
                "INSERT INTO proposals(queue_item_id, proposal_json, created_at)
                 VALUES (?1, ?2, unixepoch())
                 ON CONFLICT(queue_item_id) DO NOTHING",
                params![id, json],
            )
            .map_err(database_error)?;
        Ok(())
    }

    /// Records the proposal as waiting for a person - with `reason` among its
    /// reasons, when there is one - and the row as in review, provided the
    /// row is still `row_status`.
    fn put_in_review(
        &self,
        id: i64,
        reason: Option<&str>,
        row_status: QueueStatus,
    ) -> PipelineResult<()> {
        self.put_in_review_as(id, reason, row_status, None)
    }

    /// [`Self::put_in_review`], with the row's code set to `error_code` when
    /// one is given and left as it is otherwise.
    fn put_in_review_as(
        &self,
        id: i64,
        reason: Option<&str>,
        row_status: QueueStatus,
        error_code: Option<&str>,
    ) -> PipelineResult<()> {
        let mut record = self
            .load_proposal(id)?
            .ok_or_else(|| PipelineError::new("INVALID_DATA", "proposal is missing"))?;
        record.status = ProposalStatus::NeedsReview;
        if let Some(reason) = reason
            && !record.reasons.iter().any(|entry| entry == reason)
        {
            record.reasons.push(reason.to_owned());
        }
        // A document waiting for a person is no longer a document waiting to
        // be filed, whatever was approved before.
        record.approved = false;
        record.revision += 1;
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(database_error)?;
        let changed = transaction.execute(
            "UPDATE queue_items SET status = 'needs_review', error_code = COALESCE(?3, error_code),
                 updated_at = unixepoch()
             WHERE id = ?1 AND status = ?2",
            params![id, queue_status_text(row_status), error_code],
        ).map_err(database_error)?;
        if changed != 1 {
            return Err(PipelineError::new(
                "STATE_CONFLICT",
                "item changed before review",
            ));
        }
        let json = serde_json::to_string(&record)
            .map_err(|_| PipelineError::new("INVALID_DATA", "proposal could not be stored"))?;
        transaction.execute("UPDATE proposals SET proposal_json = ?1, created_at = unixepoch() WHERE queue_item_id = ?2", params![json, id]).map_err(database_error)?;
        transaction.commit().map_err(database_error)
    }

    /// Returns a document denied at admission to the queue, for the person
    /// who has since had its uploader verified. The compare-and-swap covers
    /// the error code as well as the status, so only that denial takes the
    /// shortcut and a concurrent decision on the item makes it fail closed.
    fn retry_unverified(&self, id: i64) -> PipelineResult<()> {
        let changed = self.lock()?.execute(
            "UPDATE queue_items SET status = 'queued', processing_failures = 0, error_code = NULL,
             owner_session = NULL, lease_expires_at = NULL, updated_at = unixepoch()
             WHERE id = ?1 AND status = 'needs_review' AND error_code = 'UPLOADER_UNVERIFIED'",
            params![id],
        ).map_err(database_error)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(PipelineError::new(
                "STATE_CONFLICT",
                "unverified retry compare-and-swap failed",
            ))
        }
    }

    fn retry_canceled(&self, id: i64) -> PipelineResult<()> {
        let changed = self.lock()?.execute(
            "UPDATE queue_items SET status = 'queued', processing_failures = 0, error_code = NULL,
             owner_session = NULL, lease_expires_at = NULL, copy_filing = 0,
             updated_at = unixepoch()
             WHERE id = ?1 AND status = 'canceled'",
            params![id],
        ).map_err(database_error)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(PipelineError::new(
                "STATE_CONFLICT",
                "canceled retry compare-and-swap failed",
            ))
        }
    }

    /// Counts one processing failure against an item that is back in the
    /// queue and owned by nobody - interrupted by a crash, or given up on
    /// because the worker that was reading it cannot be restarted - and
    /// fails it once it has failed twice.
    fn record_recovered_failure(&self, id: i64) -> PipelineResult<()> {
        let changed = self
            .lock()?
            .execute(
                "UPDATE queue_items
             SET processing_failures = processing_failures + 1,
                 status = CASE WHEN processing_failures + 1 >= 2 THEN 'failed' ELSE 'queued' END,
                 error_code = 'IO_ERROR', updated_at = unixepoch()
             WHERE id = ?1 AND status = 'queued' AND owner_session IS NULL",
                params![id],
            )
            .map_err(database_error)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(PipelineError::new(
                "STATE_CONFLICT",
                "recovered item changed before failure accounting",
            ))
        }
    }

    fn remove_item(&self, id: i64) -> PipelineResult<()> {
        let changed = self.lock()?.execute(
            "DELETE FROM queue_items WHERE id = ?1 AND status NOT IN ('extracting', 'analyzing', 'applying')",
            params![id],
        ).map_err(database_error)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(PipelineError::new(
                "STATE_CONFLICT",
                "active or missing item cannot be removed",
            ))
        }
    }

    fn approve_user_edit(
        &self,
        id: i64,
        expected: QueueStatus,
        filename: &str,
        description: &str,
    ) -> PipelineResult<()> {
        let mut record = self
            .load_proposal(id)?
            .ok_or_else(|| PipelineError::new("INVALID_DATA", "proposal is missing"))?;
        record.filename = filename.to_owned();
        record.description = description.trim().to_owned();
        record.status = ProposalStatus::Ready;
        record.reasons.clear();
        record.approved = true;
        record.revision += 1;
        let json = serde_json::to_string(&record)
            .map_err(|_| PipelineError::new("INVALID_DATA", "proposal could not be stored"))?;
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(database_error)?;
        let changed = transaction.execute(
            "UPDATE queue_items SET status = 'ready', error_code = NULL, updated_at = unixepoch()
             WHERE id = ?1 AND status = ?2",
            params![id, queue_status_text(expected)],
        ).map_err(database_error)?;
        if changed != 1 {
            return Err(PipelineError::new(
                "STATE_CONFLICT",
                "review item changed before approval",
            ));
        }
        let updated = transaction.execute(
            "UPDATE proposals SET proposal_json = ?1, created_at = unixepoch() WHERE queue_item_id = ?2",
            params![json, id],
        ).map_err(database_error)?;
        if updated != 1 {
            return Err(PipelineError::new("INVALID_DATA", "proposal is missing"));
        }
        transaction.commit().map_err(database_error)
    }

    fn lock(&self) -> PipelineResult<std::sync::MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| {
            PipelineError::new(
                "DATABASE_UNAVAILABLE",
                "pipeline database lock is unavailable",
            )
        })
    }
}

/// The folder a document is filed into: the destination (or, with none set,
/// the document's own folder) plus the layout's subfolder for its facts.
pub fn target_folder(
    settings: &AppSettings,
    source_path: &Path,
    proposal: &ValidatedProposal,
) -> PathBuf {
    let root = destination_root(settings, source_path);
    match layout_subfolder(settings.destination_layout, proposal) {
        Some(subfolder) => root.join(subfolder),
        None => root,
    }
}

fn destination_root(settings: &AppSettings, source_path: &Path) -> PathBuf {
    if settings.destination.trim().is_empty() {
        source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        PathBuf::from(settings.destination.trim())
    }
}

/// The subfolder a layout puts a document in, relative to the destination.
///
/// A document missing the fact a layout keys on goes in a named catch-all
/// ("Undated", "Unsorted") rather than the root, so the root stays a set of
/// folders and a person can see what still needs a hand.
pub fn layout_subfolder(
    layout: DestinationLayout,
    proposal: &ValidatedProposal,
) -> Option<PathBuf> {
    let year = || {
        proposal
            .document_date
            .as_deref()
            .and_then(|date| date.get(..4))
            .filter(|year| year.bytes().all(|byte| byte.is_ascii_digit()))
            .map(str::to_owned)
            .unwrap_or_else(|| "Undated".to_owned())
    };
    let kind = || {
        proposal
            .document_type
            .as_deref()
            .and_then(sanitize_folder_name)
            .unwrap_or_else(|| "Unsorted".to_owned())
    };
    match layout {
        DestinationLayout::Flat => None,
        DestinationLayout::Year => Some(PathBuf::from(year())),
        DestinationLayout::YearType => Some(PathBuf::from(year()).join(kind())),
        DestinationLayout::Type => Some(PathBuf::from(kind())),
        DestinationLayout::Party => Some(PathBuf::from(
            proposal
                .parties
                .first()
                .and_then(|party| sanitize_folder_name(party))
                .unwrap_or_else(|| "Unsorted".to_owned()),
        )),
    }
}

/// After an undo, removes the layout folders the vacated document was alone
/// in, walking up from its folder to (but never including) the destination
/// root. A folder holding anything else is left where it is; so is a folder
/// outside the root.
///
/// Never more folders than `layout` itself creates, either. A destination
/// changed to a folder that contains the old one puts everything ever filed
/// "inside the root", and walking up until the root then emptied out the
/// previous destination - somebody's filing, not Intern's scaffolding.
fn prune_empty_layout_folders(root: &Path, vacated: &Path, layout: DestinationLayout) {
    let mut remaining = match layout {
        DestinationLayout::Flat => 0,
        DestinationLayout::Year | DestinationLayout::Type | DestinationLayout::Party => 1,
        DestinationLayout::YearType => 2,
    };
    let mut folder = vacated.parent();
    while let Some(current) = folder {
        if remaining == 0 || current == root || !current.starts_with(root) {
            break;
        }
        remaining -= 1;
        let empty = fs::read_dir(current).is_ok_and(|mut entries| entries.next().is_none());
        if !empty || fs::remove_dir(current).is_err() {
            break;
        }
        folder = current.parent();
    }
}

/// The filed-document report for a completed apply receipt, or `None` when
/// the receipt is not one (an undo, or an apply that never completed).
fn filed_document(
    item_id: i64,
    source_hash: &str,
    receipt: &OperationReceipt,
    proposal: &ProposalRecord,
    filed_at: i64,
) -> Option<FiledDocument> {
    (receipt.direction == OperationDirection::Apply && receipt.stage == OperationStage::Complete)
        .then(|| FiledDocument {
            item_id,
            source_path: receipt.source.clone(),
            source_hash: source_hash.to_owned(),
            destination: receipt.destination.clone(),
            description: proposal.description.clone(),
            proposal: proposal_as_applied(
                &proposal.styled_proposal(),
                &receipt
                    .destination
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            filed_at,
            text_fingerprint: proposal.analysis.text_fingerprint.clone(),
        })
}

/// Whether a fingerprint match is one document filed twice, or two
/// documents that share their words: this month's statement and last
/// month's differ in a date and a few figures, which a fingerprint barely
/// sees. Dates settle it when both sides have one; without a date on one
/// side only a near-identical text may say duplicate.
fn same_document(distance: u32, filed_name: &str, date: Option<&str>) -> bool {
    if distance > NEAR_DUPLICATE_DISTANCE {
        return false;
    }
    match (leading_date(filed_name), date) {
        (Some(filed), Some(this)) => filed == this,
        _ => distance <= 1,
    }
}

/// The validated facts as the applied name carries them. A reviewer who
/// types a date, or accepts the one the model read, puts it in the filename
/// and nowhere else; the layout folder and the description record must
/// follow that date, not the one validation withheld.
pub fn proposal_as_applied(proposal: &ValidatedProposal, filename: &str) -> ValidatedProposal {
    let mut applied = proposal.clone();
    if let Some(date) = leading_date(filename) {
        applied.document_date = Some(date.to_owned());
    }
    applied
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs() as i64)
}

/// Whether the item has been canceled. One indexed read: this is asked
/// whenever a model request ends, and from the lease's cancel callback.
fn item_is_canceled(store: &QueueStore, id: i64) -> PipelineResult<bool> {
    Ok(store
        .get(id)?
        .is_some_and(|item| item.status == QueueStatus::Canceled))
}

/// The queue code an extraction failure is stored under, from the worker's
/// code and its own word on whether trying again can help.
///
/// Anything this does not recognise stays IO_ERROR and keeps its second
/// attempt: crashes, a busy or misbehaving worker, and the worker's I/O
/// failures, which it reports as retryable PARSE_FAILED.
fn extraction_error_code(code: &str, retryable: bool) -> ErrorCode {
    match code {
        "PASSWORD_PROTECTED" => ErrorCode::PasswordProtected,
        "UNSUPPORTED_FORMAT" => ErrorCode::UnsupportedContent,
        // The worker's own limits, and the host's deadline on the worker.
        "RESOURCE_LIMIT_EXCEEDED" | "RESOURCE_LIMIT" => ErrorCode::DocumentTooLarge,
        "NATIVE_ASSETS_MISSING" => ErrorCode::OcrUnavailable,
        "PARSE_FAILED" if !retryable => ErrorCode::ExtractionFailed,
        _ => ErrorCode::IoError,
    }
}

/// The queue code a model failure is stored under. Nothing here is a file
/// operation, so nothing here is IO_ERROR: a code the table does not name
/// still failed at the model's step, and says so.
fn model_error_code(error: &ModelFailure) -> ErrorCode {
    match error.code.as_str() {
        "MODEL_RESPONSE_INVALID" | "MODEL_REPLY_TRUNCATED" => ErrorCode::ModelOutputInvalid,
        "HOSTED_MODEL_REFUSED" => ErrorCode::ModelDeclined,
        "MODEL_INPUT_TOO_LARGE" => ErrorCode::DocumentTooLarge,
        "ANALYSIS_FAILED" => ErrorCode::AnalysisFailed,
        code if code.starts_with("HOSTED_MODEL_") => ErrorCode::HostedModelUnavailable,
        _ => ErrorCode::ModelFailed,
    }
}

/// How many documents in a row may fail the same way that may be the
/// machine's - replies that cannot be used, time-outs, reader crashes -
/// before the queue stops to say the machine itself is the problem.
const FAILURES_BEFORE_PAUSE: u32 = 3;

/// What the reader's self-test reads (see `Pipeline::reader_self_test`).
const READER_CHECK_TEXT: &str = "Intern reader check: this file was written to test the reader.\n";

/// How the reader's self-test went.
enum ReaderCheck {
    Read,
    Failed,
    /// Its file could not be written, which says nothing about the reader.
    Unwritable,
}

/// How many documents of one kind waiting in doubt over a reader crash,
/// with no document of that kind to prove the reader, stop the queue.
const SEVERAL_WAITING_ON_THE_READER: u32 = 2;

/// The kinds of failure that may be the machine's rather than a
/// document's, each doubted and counted on its own (see
/// `QueueStore::doubt` and `QueueStore::join_failure_run`). A reader crash
/// is doubted by the family of file it crashed on (`crash_doubt_kind`).
const STREAK_REPLY: &str = "reply";
const STREAK_TIMEOUT: &str = "timeout";
/// The doubts a document the model read successfully clears.
const MODEL_DOUBTS: &[&str] = &[STREAK_REPLY, STREAK_TIMEOUT];

/// The reason the queue stops when the reader's self-test could not write
/// its file in Intern's data folder.
pub const READER_CHECK_UNWRITABLE: &str = "READER_CHECK_UNWRITABLE";

/// The reason the queue stopped for a run of reader crashes, as the window
/// names it.
pub const READER_CRASHED: &str = "READER_CRASHED";

/// How long documents may keep finding no model loaded before the queue
/// stops for it. A request that meets a start or a restart under way waits
/// for it in the app's model, so the only empty slot that fills on its own
/// is the moment a switch between the hosted and the local model takes to
/// begin its start. Far longer than that, and short enough that a start or
/// restart that failed is reported within minutes rather than never.
const MODEL_MISSING_GRACE: std::time::Duration = std::time::Duration::from_secs(4 * 60);

/// What the queue does with a document whose analysis failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModelFailureAction {
    /// Back to waiting, nothing counted: the document was never analysed.
    Requeue,
    /// Fail this document now; another attempt would answer the same.
    Terminal,
    /// Count the failure and stop the queue; the next document would fail
    /// the same way.
    Pause,
    /// Count the failure; the document gets its one more attempt.
    Retry,
}

fn model_failure_action(code: &str) -> ModelFailureAction {
    match code {
        "MODEL_NOT_READY" => ModelFailureAction::Requeue,
        "MODEL_INPUT_TOO_LARGE"
        | "HOSTED_MODEL_REFUSED"
        | "ANALYSIS_FAILED"
        | "MODEL_RESPONSE_INVALID"
        | "MODEL_REPLY_TRUNCATED" => ModelFailureAction::Terminal,
        "MODEL_CANCEL_FAILED"
        | "MODEL_RECOVERY_FAILED"
        | "MODEL_REQUEST_FAILED"
        | "MODEL_SERVER_START_FAILED"
        | "HOSTED_MODEL_MISCONFIGURED"
        | "HOSTED_MODEL_KEY_MISSING"
        | "HOSTED_MODEL_UNAUTHORIZED"
        | "HOSTED_MODEL_UNREACHABLE"
        | "HOSTED_MODEL_RATE_LIMITED"
        | "HOSTED_MODEL_BILLING" => ModelFailureAction::Pause,
        // MODEL_CANCELED among them: a request that was called off says
        // nothing about the next document, so it never stops the queue.
        _ => ModelFailureAction::Retry,
    }
}

/// A reply that came back but could not be used.
fn is_unreadable_reply(code: &str) -> bool {
    matches!(code, "MODEL_RESPONSE_INVALID" | "MODEL_REPLY_TRUNCATED")
}

/// The reason a pause reports. A hosted failure's own code names what to
/// fix - the key, the address, the account's credit - and the window has a
/// sentence for each; anything else reports the code the document was
/// stored under, which it also has a sentence for.
fn pause_reason_code(failure: &str, stored: ErrorCode) -> &str {
    if failure.starts_with("HOSTED_MODEL_") {
        failure
    } else {
        stored.as_str()
    }
}

/// The review reason and error code for a rename that carries no date.
pub const DATE_REQUIRED: &str = "DATE_REQUIRED";

/// What an apply reports when the queue is busy with another document. Not a
/// failure of the rename: the item stays ready and the scheduler applies it
/// as soon as it is free.
const APPLY_DEFERRED: &str = "APPLY_DEFERRED";

/// The date a filename begins with - `YYYY-MM-DD`, a real calendar date,
/// standing on its own before whatever follows - or `None`.
///
/// Every rename must carry one. A name without a date sorts nowhere and
/// says nothing about when, so the engine never proposes one as ready, and
/// a person approving a name types the date in (or takes the one the model
/// suggested) rather than filing an undated document.
pub fn leading_date(filename: &str) -> Option<&str> {
    let date = filename.get(..10)?;
    if !is_valid_iso_date(date) {
        return None;
    }
    let follows_cleanly = filename[10..]
        .chars()
        .next()
        .is_none_or(|next| !next.is_alphanumeric());
    follows_cleanly.then_some(date)
}

fn worker_error(error: WorkerFailure) -> PipelineError {
    PipelineError::new(error.code, "parser worker request failed")
}

fn database_error(_: rusqlite::Error) -> PipelineError {
    PipelineError::new("DATABASE_UNAVAILABLE", "pipeline database operation failed")
}

fn queue_status_text(status: QueueStatus) -> &'static str {
    match status {
        QueueStatus::Queued => "queued",
        QueueStatus::Extracting => "extracting",
        QueueStatus::Analyzing => "analyzing",
        QueueStatus::Ready => "ready",
        QueueStatus::NeedsReview => "needs_review",
        QueueStatus::Failed => "failed",
        QueueStatus::Canceled => "canceled",
        QueueStatus::Applying => "applying",
        QueueStatus::Completed => "completed",
    }
}

/// The names already in `folder` that a document from `source_path` must not
/// take. Its own name is not one of them: in its own folder the document is
/// the file with that name, and counting it made a document already named the
/// way Intern names documents collide with itself and be proposed - and with
/// automatic renaming, renamed - as "... (2)".
fn names_beside(folder: &Path, source_path: &Path) -> Vec<String> {
    let mut names = existing_names(folder);
    if let Some(own) = source_path
        .file_name()
        .filter(|_| is_own_folder(folder, source_path))
    {
        let own = name_key(&own.to_string_lossy());
        names.retain(|name| name_key(name) != own);
    }
    names
}

/// The document an exact duplicate repeats, as far as the queue knows it.
struct DuplicateOriginal {
    /// The name it is filed under, or kept under.
    name: String,
    /// The folder it is filed in, when this machine filed it.
    folder: Option<PathBuf>,
    /// Where it is, when this machine knows: never moved as a copy of itself.
    filed_path: Option<PathBuf>,
}

/// Whether `path` is `root` or inside it, as written or once both resolve.
fn is_within(path: &Path, root: &Path) -> bool {
    let (path_key, root_key) = (source_path_key(path), source_path_key(root));
    path.starts_with(root)
        || (!root_key.is_empty()
            && (path_key == root_key || path_key.starts_with(&format!("{root_key}\\"))))
        || matches!(
            (fs::canonicalize(path), fs::canonicalize(root)),
            (Ok(path), Ok(root)) if path.starts_with(&root)
        )
}

/// Whether two paths name the same file, as the queue keys paths or once
/// both resolve.
fn same_file_path(left: &Path, right: &Path) -> bool {
    source_path_key(left) == source_path_key(right)
        || matches!(
            (fs::canonicalize(left), fs::canonicalize(right)),
            (Ok(left), Ok(right)) if left == right
        )
}

/// Whether filing at `destination` would leave the document where it is,
/// under a name Windows takes for its own.
fn names_the_source(destination: &Path, source_path: &Path) -> bool {
    let (Some(folder), Some(name), Some(own)) = (
        destination.parent(),
        destination.file_name(),
        source_path.file_name(),
    ) else {
        return false;
    };
    is_own_folder(folder, source_path)
        && name_key(&name.to_string_lossy()) == name_key(&own.to_string_lossy())
}

/// Whether `folder` is the folder `source_path` is in: the same path, or the
/// same folder spelled another way.
fn is_own_folder(folder: &Path, source_path: &Path) -> bool {
    let Some(parent) = source_path.parent() else {
        return false;
    };
    folder == parent
        || matches!(
            (fs::canonicalize(folder), fs::canonicalize(parent)),
            (Ok(folder), Ok(parent)) if folder == parent
        )
}

/// A name as Windows compares two in one folder - without regard to case, and
/// without the trailing dots and spaces it drops - which is how the composer
/// decides that two names collide.
fn name_key(name: &str) -> String {
    name.trim_end_matches([' ', '.']).to_lowercase()
}

fn existing_names(directory: &Path) -> Vec<String> {
    fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

#[cfg(test)]
mod date_gate_tests {
    use intern_engine::{Evidence, PartyRelation, ValidatedProposal};

    use super::{leading_date, proposal_as_applied};

    #[test]
    fn the_applied_name_lends_its_date_to_the_facts_but_never_takes_one_away() {
        let withheld = ValidatedProposal {
            document_type: Some("Invoice".into()),
            document_date: None,
            date_role: None,
            parties: Vec::new(),
            party_relation: PartyRelation::None,
            description: "An invoice.".into(),
            confidence: 0.8,
            evidence: Evidence::default(),
        };
        assert_eq!(
            proposal_as_applied(&withheld, "2026-03-02 Invoice.pdf")
                .document_date
                .as_deref(),
            Some("2026-03-02")
        );
        let dated = ValidatedProposal {
            document_date: Some("2026-01-01".into()),
            ..withheld.clone()
        };
        assert_eq!(
            proposal_as_applied(&dated, "2026-03-02 Invoice.pdf")
                .document_date
                .as_deref(),
            Some("2026-03-02"),
            "a reviewer's date wins over the model's"
        );
        assert_eq!(
            proposal_as_applied(&dated, "Invoice.pdf")
                .document_date
                .as_deref(),
            Some("2026-01-01"),
            "a name without a date changes nothing"
        );
    }

    #[test]
    fn a_filename_carries_a_date_only_when_it_starts_with_a_real_one() {
        assert_eq!(
            leading_date("2026-03-02 Invoice from Acme.pdf"),
            Some("2026-03-02")
        );
        assert_eq!(leading_date("2026-03-02.pdf"), Some("2026-03-02"));
        assert_eq!(leading_date("2026-03-02-invoice.pdf"), Some("2026-03-02"));
        assert_eq!(leading_date("2026-03-02"), Some("2026-03-02"));
        assert_eq!(leading_date("Invoice from Acme.pdf"), None);
        assert_eq!(leading_date("Invoice 2026-03-02 from Acme.pdf"), None);
        assert_eq!(
            leading_date("2026-02-30 Invoice.pdf"),
            None,
            "not a real day"
        );
        assert_eq!(
            leading_date("2026-03-021 Invoice.pdf"),
            None,
            "digits run on"
        );
        assert_eq!(leading_date("2026-03-0"), None);
        assert_eq!(leading_date(""), None);
    }
}

#[cfg(test)]
mod sink_tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use intern_engine::{Evidence, PartyRelation, ValidatedProposal};

    use super::{FiledDocument, FilingSink, FilingSinks, KnownFiling, UnfiledDocument};

    #[derive(Default)]
    struct Heard(Mutex<Vec<String>>);

    impl FilingSink for Heard {
        fn filed(&self, document: &FiledDocument) {
            self.0
                .lock()
                .unwrap()
                .push(format!("filed {}", document.item_id));
        }
        fn unfiled(&self, document: &UnfiledDocument) {
            self.0
                .lock()
                .unwrap()
                .push(format!("unfiled {}", document.item_id));
        }
    }

    #[test]
    fn every_sink_behind_the_fan_out_hears_every_report_in_order() {
        let first = Arc::new(Heard::default());
        let second = Arc::new(Heard::default());
        let sinks = FilingSinks(vec![first.clone(), second.clone()]);
        sinks.filed(&FiledDocument {
            item_id: 4,
            source_path: PathBuf::from("/in/a.pdf"),
            source_hash: "hash".into(),
            destination: PathBuf::from("/out/a.pdf"),
            description: "A sentence.".into(),
            proposal: ValidatedProposal {
                document_type: None,
                document_date: None,
                date_role: None,
                parties: Vec::new(),
                party_relation: PartyRelation::Between,
                description: "A sentence.".into(),
                confidence: 0.5,
                evidence: Evidence::default(),
            },
            filed_at: 1,
            text_fingerprint: None,
        });
        sinks.unfiled(&UnfiledDocument {
            item_id: 4,
            source_path: PathBuf::from("/in/a.pdf"),
            source_hash: "hash".into(),
            destination: PathBuf::from("/out/a.pdf"),
        });
        for sink in [&first, &second] {
            assert_eq!(
                *sink.0.lock().unwrap(),
                vec!["filed 4".to_string(), "unfiled 4".to_string()]
            );
        }
    }

    #[test]
    fn a_known_filing_names_the_machine_only_when_there_is_one_to_name() {
        let teammate = KnownFiling {
            filename: "2026-03-02 Agreement.pdf".into(),
            filed_by: Some("Front desk".into()),
        };
        assert_eq!(
            teammate.describe(),
            "2026-03-02 Agreement.pdf (filed from Front desk)"
        );
        let own = KnownFiling {
            filename: "2026-03-02 Agreement.pdf".into(),
            filed_by: None,
        };
        assert_eq!(own.describe(), "2026-03-02 Agreement.pdf");
    }
}

#[cfg(test)]
mod layout_tests {
    use std::path::{Path, PathBuf};

    use intern_engine::{DateRole, Evidence, PartyRelation, ValidatedProposal};

    use super::{layout_subfolder, prune_empty_layout_folders, target_folder};
    use crate::settings::{AppSettings, DestinationLayout};

    fn proposal(date: Option<&str>, kind: Option<&str>, parties: &[&str]) -> ValidatedProposal {
        ValidatedProposal {
            document_type: kind.map(str::to_owned),
            document_date: date.map(str::to_owned),
            date_role: date.map(|_| DateRole::Effective),
            parties: parties.iter().map(|party| (*party).to_owned()).collect(),
            party_relation: PartyRelation::Between,
            description: "A description.".into(),
            confidence: 0.9,
            evidence: Evidence::default(),
        }
    }

    #[test]
    fn each_layout_derives_its_folder_from_the_documents_facts() {
        let full = proposal(
            Some("2026-04-01"),
            Some("Statement of Work"),
            &["Ridgeline Cartography LLC", "Contoso Worldwide, Inc."],
        );
        assert_eq!(layout_subfolder(DestinationLayout::Flat, &full), None);
        assert_eq!(
            layout_subfolder(DestinationLayout::Year, &full),
            Some(PathBuf::from("2026"))
        );
        assert_eq!(
            layout_subfolder(DestinationLayout::YearType, &full),
            Some(PathBuf::from("2026").join("Statement of Work"))
        );
        assert_eq!(
            layout_subfolder(DestinationLayout::Type, &full),
            Some(PathBuf::from("Statement of Work"))
        );
        assert_eq!(
            layout_subfolder(DestinationLayout::Party, &full),
            Some(PathBuf::from("Ridgeline Cartography LLC"))
        );
    }

    #[test]
    fn a_missing_fact_goes_to_a_named_catch_all_never_the_root() {
        let bare = proposal(None, None, &[]);
        assert_eq!(
            layout_subfolder(DestinationLayout::Year, &bare),
            Some(PathBuf::from("Undated"))
        );
        assert_eq!(
            layout_subfolder(DestinationLayout::YearType, &bare),
            Some(PathBuf::from("Undated").join("Unsorted"))
        );
        assert_eq!(
            layout_subfolder(DestinationLayout::Party, &bare),
            Some(PathBuf::from("Unsorted"))
        );
        // Hostile characters in a fact never reach a folder name.
        let hostile = proposal(Some("2026-04-01"), Some("Invoice: 3/4 <draft>"), &["CON"]);
        assert_eq!(
            layout_subfolder(DestinationLayout::Type, &hostile),
            Some(PathBuf::from("Invoice 34 draft"))
        );
        assert_eq!(
            layout_subfolder(DestinationLayout::Party, &hostile),
            Some(PathBuf::from("_CON"))
        );
    }

    #[test]
    fn the_target_folder_falls_back_to_the_documents_own_folder_without_a_destination() {
        let mut settings = AppSettings {
            destination_layout: DestinationLayout::Year,
            ..AppSettings::default()
        };
        let source = Path::new("/inbox/scan.pdf");
        let proposal = proposal(Some("2026-04-01"), None, &[]);
        assert_eq!(
            target_folder(&settings, source, &proposal),
            PathBuf::from("/inbox").join("2026")
        );
        settings.destination = "/filed".into();
        assert_eq!(
            target_folder(&settings, source, &proposal),
            PathBuf::from("/filed").join("2026")
        );
    }

    /// The destination can be changed to a folder that contains the old one.
    /// Everything under the old destination is then "inside the root", and
    /// walking up until the root emptied out the previous destination itself,
    /// which is somebody's filing rather than Intern's scaffolding. Only as
    /// many folders as the layout in force could have made are ever removed.
    #[test]
    fn pruning_never_reaches_above_the_folders_the_layout_makes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let previous_destination = root.join("filed");
        let vacated = previous_destination
            .join("2026")
            .join("Invoice")
            .join("a.pdf");
        std::fs::create_dir_all(vacated.parent().unwrap()).unwrap();
        prune_empty_layout_folders(&root, &vacated, DestinationLayout::YearType);
        assert!(!previous_destination.join("2026").exists());
        assert!(
            previous_destination.exists(),
            "the folder that used to be the destination is not Intern's to remove"
        );
    }

    #[test]
    fn pruning_stops_at_the_root_and_at_the_first_folder_with_contents() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("filed");
        let vacated = root.join("2026").join("Invoice").join("a.pdf");
        std::fs::create_dir_all(vacated.parent().unwrap()).unwrap();
        prune_empty_layout_folders(&root, &vacated, DestinationLayout::YearType);
        assert!(!root.join("2026").exists());
        assert!(root.exists());

        let sibling = root.join("2025").join("Invoice").join("b.pdf");
        std::fs::create_dir_all(sibling.parent().unwrap()).unwrap();
        std::fs::write(&sibling, b"x").unwrap();
        let vacated = root.join("2025").join("Notice").join("c.pdf");
        std::fs::create_dir_all(vacated.parent().unwrap()).unwrap();
        prune_empty_layout_folders(&root, &vacated, DestinationLayout::YearType);
        assert!(!root.join("2025").join("Notice").exists());
        assert!(
            root.join("2025").join("Invoice").exists(),
            "a year folder with another document in it stays"
        );

        // A path outside the root is never touched.
        let elsewhere = temp.path().join("elsewhere").join("d.pdf");
        std::fs::create_dir_all(elsewhere.parent().unwrap()).unwrap();
        prune_empty_layout_folders(&root, &elsewhere, DestinationLayout::YearType);
        assert!(elsewhere.parent().unwrap().exists());
    }
}

fn validate_leaf_filename(value: &str) -> PipelineResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 512 || Path::new(trimmed).components().count() != 1
        || trimmed.contains('/') || trimmed.contains('\\') || matches!(trimmed, "." | "..")
        || trimmed.ends_with(' ') || trimmed.ends_with('.')
        || trimmed.chars().any(|character| {
            character.is_control()
                || matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
                || matches!(character as u32, 0x061c | 0x200e..=0x200f | 0x202a..=0x202e | 0x2066..=0x2069)
        })
    {
        return Err(PipelineError::new("NAME_INVALID", "filename must be one nonblank path component"));
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod failure_policy_tests {
    use intern_core::ErrorCode;

    use super::{
        ModelFailure, ModelFailureAction, extraction_error_code, model_error_code,
        model_failure_action, pause_reason_code,
    };

    #[test]
    fn extraction_failures_the_worker_may_get_past_keep_the_generic_code() {
        assert_eq!(
            extraction_error_code("PARSE_FAILED", false),
            ErrorCode::ExtractionFailed
        );
        // The worker reports its own I/O failures as retryable PARSE_FAILED.
        assert_eq!(
            extraction_error_code("PARSE_FAILED", true),
            ErrorCode::IoError
        );
        for code in [
            "WORKER_CRASHED",
            "WORKER_BUSY",
            "WORKER_PROTOCOL_INVALID",
            "SOMETHING_NEW",
        ] {
            assert_eq!(
                extraction_error_code(code, false),
                ErrorCode::IoError,
                "{code}"
            );
        }
    }

    /// Every model failure the queue can meet, with where it is stored and
    /// what the queue does about it. Nothing on the model's side of the line
    /// is a file operation, so none of it is IO_ERROR any more.
    #[test]
    fn every_model_failure_has_a_code_and_a_policy() {
        use ModelFailureAction::{Pause, Requeue, Retry, Terminal};
        let table = [
            (
                "MODEL_RESPONSE_INVALID",
                ErrorCode::ModelOutputInvalid,
                Terminal,
            ),
            (
                "MODEL_REPLY_TRUNCATED",
                ErrorCode::ModelOutputInvalid,
                Terminal,
            ),
            ("HOSTED_MODEL_REFUSED", ErrorCode::ModelDeclined, Terminal),
            (
                "MODEL_INPUT_TOO_LARGE",
                ErrorCode::DocumentTooLarge,
                Terminal,
            ),
            ("ANALYSIS_FAILED", ErrorCode::AnalysisFailed, Terminal),
            ("MODEL_CANCEL_FAILED", ErrorCode::ModelFailed, Pause),
            ("MODEL_RECOVERY_FAILED", ErrorCode::ModelFailed, Pause),
            ("MODEL_REQUEST_FAILED", ErrorCode::ModelFailed, Pause),
            ("MODEL_SERVER_START_FAILED", ErrorCode::ModelFailed, Pause),
            (
                "HOSTED_MODEL_MISCONFIGURED",
                ErrorCode::HostedModelUnavailable,
                Pause,
            ),
            (
                "HOSTED_MODEL_KEY_MISSING",
                ErrorCode::HostedModelUnavailable,
                Pause,
            ),
            (
                "HOSTED_MODEL_UNAUTHORIZED",
                ErrorCode::HostedModelUnavailable,
                Pause,
            ),
            (
                "HOSTED_MODEL_UNREACHABLE",
                ErrorCode::HostedModelUnavailable,
                Pause,
            ),
            (
                "HOSTED_MODEL_RATE_LIMITED",
                ErrorCode::HostedModelUnavailable,
                Pause,
            ),
            (
                "HOSTED_MODEL_BILLING",
                ErrorCode::HostedModelUnavailable,
                Pause,
            ),
            (
                "HOSTED_MODEL_REJECTED",
                ErrorCode::HostedModelUnavailable,
                Retry,
            ),
            ("MODEL_TIMEOUT", ErrorCode::ModelFailed, Retry),
            ("MODEL_CANCELED", ErrorCode::ModelFailed, Retry),
            ("MODEL_SERVER_UNHEALTHY", ErrorCode::ModelFailed, Retry),
            ("SETTINGS_UNAVAILABLE", ErrorCode::ModelFailed, Retry),
            ("MODEL_NOT_READY", ErrorCode::ModelFailed, Requeue),
        ];
        for (code, stored, action) in table {
            assert_eq!(
                model_error_code(&ModelFailure::fatal(code)),
                stored,
                "{code}"
            );
            assert_eq!(model_failure_action(code), action, "{code}");
        }
    }

    #[test]
    fn a_pause_names_the_hosted_failure_or_the_stored_code() {
        assert_eq!(
            pause_reason_code("HOSTED_MODEL_BILLING", ErrorCode::HostedModelUnavailable),
            "HOSTED_MODEL_BILLING"
        );
        assert_eq!(
            pause_reason_code("MODEL_REQUEST_FAILED", ErrorCode::ModelFailed),
            "MODEL_FAILED"
        );
        assert_eq!(
            pause_reason_code("MODEL_REPLY_TRUNCATED", ErrorCode::ModelOutputInvalid),
            "MODEL_OUTPUT_INVALID"
        );
    }
}
