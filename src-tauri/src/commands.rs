use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use intern_core::{
    HISTORY_LIMIT, HistoryEntry, OperationDirection, OperationKind, OperationStage, QueueStatus,
    QueueStore,
};
use intern_engine::HouseRule;
use intern_engine::{
    DocumentAnalysis, DocumentSource, Engine, EngineErrorCode, LlamaServer, ModelClient, ModelFile,
    ModelManifest, ServerOptions, SupervisedWorker, prepare_worker_temp_root,
};
use intern_queue::{LearnedRule, ModelSource};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use intern_engine::download::{
    CancellationToken, DiskSpace, Downloader, HttpTransport, ReqwestHttpTransport, SetupProgress,
    SystemDiskSpace, validate_selected_file,
};
use intern_engine::setup::{
    ExistingModelSelection, SetupOperationGate, install_existing_model_files, semantic_probes,
    validate_semantic_probe,
};
use intern_intake::{IntakeConfig, IntakeWatcher, MachineIdentity};
use intern_queue::{
    AnalyzerBoundary, AppSettings, FilingSink, FilingSinks, LoadedSettings, ModelFailure, Pipeline,
    PipelineError, PipelineEventSink, PipelineItem, PipelineProgress, SettingsStore,
    paths::{
        SUPPORTED_EXTENSIONS, canonical_file, canonical_folder, canonical_model_file,
        collect_supported_files, display_path, parse_item_id,
    },
};

use crate::intake::{
    CloudLocationDto, CloudRootDto, DescriptionsStatusDto, IntakeStatusDto, LedgerSink,
    PipelineIntakeHost, SharedFiledIndex, classify_folder, filed_folder_for, list_cloud_roots,
    now_unix, status_dto,
};
use crate::model::{
    HostedModel, HostedModelStatusDto, HostedModelTestDto, SwitchingModel, suggested_date,
};
use crate::secrets::{KeyringStore, SecretStore};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileSelectionDto {
    pub path: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FolderSelectionDto {
    pub path: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExistingModelFilesDto {
    pub model_path: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<PipelineError> for CommandError {
    fn from(error: PipelineError) -> Self {
        Self {
            code: error.code,
            message: error.message,
        }
    }
}

impl From<intern_engine::EngineError> for CommandError {
    fn from(error: intern_engine::EngineError) -> Self {
        Self {
            code: error.code().as_str().into(),
            message: error.message().into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemDto {
    id: String,
    original_filename: String,
    status: QueueStatus,
    proposed_filename: Option<String>,
    confidence: Option<f32>,
    description: Option<String>,
    evidence: Option<EvidenceDto>,
    reason: Option<String>,
    error_code: Option<String>,
    undoable: bool,
    /// A ready item whose name a person has already approved. An approval
    /// that arrives while the queue is busy with another document is kept
    /// and filed between documents, so the item stays ready - decided, and
    /// waiting only for the queue, which the status alone cannot say.
    approved: bool,
    proposal_revision: Option<String>,
    reconciliation: Option<ReconciliationDto>,
    /// A date the model proposed that validation withheld from the filename,
    /// for the reviewer to accept with one click.
    #[serde(skip_serializing_if = "Option::is_none")]
    suggested_date: Option<String>,
    /// Every date the document states, for a reviewer who must give a
    /// document a date the model did not.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    dates_in_document: Vec<String>,
    /// The file's own last-modified date, in this machine's calendar - a
    /// last resort for a document that states no date, labelled as such.
    #[serde(skip_serializing_if = "Option::is_none")]
    file_modified_date: Option<String>,
    /// The reviewer's own spellings applied to the proposed name, so the
    /// inspector can say why the name differs from the evidence under it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    house_rules: Vec<HouseRuleDto>,
    /// The filing this document's text nearly repeats, when there is one:
    /// the name it was filed under, and the machine when it was not this one.
    #[serde(skip_serializing_if = "Option::is_none")]
    near_duplicate_of: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseRuleDto {
    kind: String,
    from: String,
    to: String,
}

impl From<&HouseRule> for HouseRuleDto {
    fn from(rule: &HouseRule) -> Self {
        Self {
            kind: rule.kind.as_str().to_owned(),
            from: rule.from.clone(),
            to: rule.to.clone(),
        }
    }
}

/// A spelling review has taught Intern, as Settings lists it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnedRuleDto {
    id: String,
    kind: String,
    from: String,
    to: String,
    seen: u32,
    active: bool,
    learned_at: i64,
}

impl From<LearnedRule> for LearnedRuleDto {
    fn from(rule: LearnedRule) -> Self {
        Self {
            id: rule.id.to_string(),
            kind: rule.kind.as_str().to_owned(),
            from: rule.from,
            to: rule.to,
            seen: rule.seen,
            active: rule.active,
            learned_at: rule.learned_at,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceDto {
    date: Option<String>,
    r#type: Option<String>,
    parties: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReconciliationDto {
    source_path: String,
    destination_path: String,
    error_code: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SetupStatus {
    Ready,
    Required,
    Downloading,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStateDto {
    state: SetupStatus,
    downloaded_bytes: u64,
    total_bytes: u64,
    error: Option<String>,
    /// Whether a hosted model is chosen and configured, which lets documents
    /// be processed whatever the local model's state.
    hosted_model_ready: bool,
}

struct TauriPipelineEvents {
    app: AppHandle,
    /// The queue this reports on. Weak because the queue owns the sink, and
    /// filled in afterwards because the sink has to exist before the queue
    /// that holds it does.
    pipeline: std::sync::OnceLock<std::sync::Weak<Pipeline>>,
}

impl TauriPipelineEvents {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            pipeline: std::sync::OnceLock::new(),
        }
    }

    fn watch(&self, pipeline: &Arc<Pipeline>) {
        let _ = self.pipeline.set(Arc::downgrade(pipeline));
    }

    fn paused(&self) -> bool {
        self.pipeline
            .get()
            .and_then(std::sync::Weak::upgrade)
            .is_some_and(|pipeline| pipeline.is_paused())
    }
}

/// What a queue-change event carries.
///
/// The queue pauses itself - a hosted model that refuses the key, a shared
/// lease that cannot be taken - and the window only heard "something
/// changed", so it went on offering to pause a queue that had already
/// stopped. Every change now says which it is.
fn queue_changed_payload(paused: bool) -> serde_json::Value {
    serde_json::json!({ "paused": paused })
}

impl PipelineEventSink for TauriPipelineEvents {
    fn queue_changed(&self) {
        let _ = self
            .app
            .emit("queue://changed", queue_changed_payload(self.paused()));
    }
    fn progress(&self, progress: PipelineProgress) {
        let _ = self.app.emit("queue://progress", progress);
    }
}

struct RuntimeModel {
    executable: PathBuf,
    model_directory: PathBuf,
    engine: RwLock<Option<Engine>>,
    server: Mutex<Option<LlamaServer>>,
}

impl RuntimeModel {
    fn new(executable: PathBuf, model_directory: PathBuf) -> Self {
        Self {
            executable,
            model_directory,
            engine: RwLock::new(None),
            server: Mutex::new(None),
        }
    }

    fn installed(&self, manifest: &ModelManifest) -> bool {
        manifest.files.iter().all(|file| {
            validate_selected_file(&self.model_directory.join(&file.name), file).is_ok()
        })
    }

    /// Starts the local server.
    ///
    /// Text only, and not as a mode: no vision projector is pinned, downloaded,
    /// or loaded. Essentially every business document carries usable text, and a
    /// projector for this model is 668,227,264 bytes - 637 MiB - which every
    /// user would download and hold resident for a path almost nothing takes.
    fn start(&self, manifest: &ModelManifest) -> Result<(), CommandError> {
        if !self.installed(manifest) {
            return Err(CommandError {
                code: "MODEL_NOT_READY".into(),
                message: "model files are not installed".into(),
            });
        }
        let model = manifest.model().ok_or_else(|| CommandError {
            code: "MODEL_MANIFEST_INVALID".into(),
            message: "model manifest names no text model".into(),
        })?;
        let server = LlamaServer::start(
            &self.executable,
            &self.model_directory.join(&model.name),
            None,
            &ServerOptions::default(),
        )?;
        let client = ModelClient::new(
            &server.completion_endpoint(),
            server.api_key().to_owned(),
            manifest.served_model_name.clone(),
        )?;
        let mut server_state = self.server.lock().map_err(|_| CommandError {
            code: "MODEL_NOT_READY".into(),
            message: "model process state is unavailable".into(),
        })?;
        let mut engine_state = self.engine.write().map_err(|_| CommandError {
            code: "MODEL_NOT_READY".into(),
            message: "model state is unavailable".into(),
        })?;
        if server_state.is_some() || engine_state.is_some() {
            return Err(CommandError {
                code: "MODEL_ALREADY_RUNNING".into(),
                message: "local model process is already running".into(),
            });
        }
        *server_state = Some(server);
        *engine_state = Some(Engine::new(client));
        Ok(())
    }

    fn start_verified(
        &self,
        manifest: &ModelManifest,
        cancellation: &CancellationToken,
    ) -> Result<(), CommandError> {
        self.stop_runtime().map_err(|error| CommandError {
            code: error.code,
            message: "existing local model process could not be stopped".into(),
        })?;
        if cancellation.is_canceled() {
            return Err(setup_canceled_error());
        }
        self.start(manifest)?;
        let result = self.semantic_self_test(cancellation);
        if result.is_err() {
            let _ = self.stop_runtime();
        }
        result
    }

    fn semantic_self_test(&self, cancellation: &CancellationToken) -> Result<(), CommandError> {
        for probe in semantic_probes()? {
            if cancellation.is_canceled() {
                return Err(setup_canceled_error());
            }
            let analysis = {
                let engine = self.engine.read().map_err(|_| CommandError {
                    code: "MODEL_SELF_TEST_FAILED".into(),
                    message: "local model state is unavailable during self-test".into(),
                })?;
                let engine = engine.as_ref().ok_or_else(|| CommandError {
                    code: "MODEL_SELF_TEST_FAILED".into(),
                    message: "local model is unavailable during self-test".into(),
                })?;
                engine.analyze(&probe.document, "pdf", &[])
            };
            if cancellation.is_canceled() {
                return Err(setup_canceled_error());
            }
            let analysis = analysis.map_err(|_| CommandError {
                code: "MODEL_SELF_TEST_FAILED".into(),
                message: "local model semantic self-test request failed".into(),
            })?;
            validate_semantic_probe(&probe, &analysis)?;
        }
        Ok(())
    }

    fn stop_runtime(&self) -> Result<(), ModelFailure> {
        let stop_result = {
            let mut server = self
                .server
                .lock()
                .map_err(|_| ModelFailure::fatal("MODEL_CANCEL_FAILED"))?;
            server
                .take()
                .map(|server| {
                    server
                        .stop()
                        .map_err(|_| ModelFailure::fatal("MODEL_CANCEL_FAILED"))
                })
                .unwrap_or(Ok(()))
        };
        *self
            .engine
            .write()
            .map_err(|_| ModelFailure::fatal("MODEL_CANCEL_FAILED"))? = None;
        stop_result
    }
}

impl AnalyzerBoundary for RuntimeModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        let engine = self
            .engine
            .read()
            .map_err(|_| ModelFailure::fatal("MODEL_NOT_READY"))?;
        let engine = engine
            .as_ref()
            .ok_or_else(|| ModelFailure::fatal("MODEL_NOT_READY"))?;
        engine
            .analyze(source, extension, existing_names)
            .map_err(|error| ModelFailure::retryable(error.code().as_str()))
    }

    fn recover(&self, failure: &ModelFailure) -> Result<(), ModelFailure> {
        if failure.code != "MODEL_REQUEST_FAILED" {
            return Ok(());
        }
        self.stop_runtime()
            .map_err(|_| ModelFailure::fatal("MODEL_RECOVERY_FAILED"))?;
        let manifest =
            ModelManifest::embedded().map_err(|_| ModelFailure::fatal("MODEL_RECOVERY_FAILED"))?;
        self.start(&manifest)
            .map_err(|_| ModelFailure::fatal("MODEL_RECOVERY_FAILED"))
    }

    fn cancel(&self) -> Result<(), ModelFailure> {
        self.stop_runtime()?;
        let manifest = ModelManifest::embedded()
            .map_err(|error| ModelFailure::fatal(error.code().as_str()))?;
        self.start(&manifest)
            .map_err(|error| ModelFailure::fatal(error.code))
    }

    fn shutdown(&self) -> Result<(), ModelFailure> {
        self.stop_runtime()
    }
}

struct SetupManager {
    app: AppHandle,
    runtime: Arc<RuntimeModel>,
    state: Mutex<SetupStateDto>,
    operation: SetupOperationGate,
    scheduler: Mutex<Option<std::sync::mpsc::Sender<SchedulerMessage>>>,
    /// Whether the queue may run: the local model is ready, or a hosted one
    /// is chosen and configured. The scheduler reads this.
    model_ready: Arc<AtomicBool>,
    local_ready: AtomicBool,
    hosted_active: AtomicBool,
}

impl SetupManager {
    fn new(app: AppHandle, runtime: Arc<RuntimeModel>, manifest: &ModelManifest) -> Self {
        let total_bytes = manifest.total_bytes();
        let installed = runtime.installed(manifest);
        let state = SetupStateDto {
            state: if installed {
                SetupStatus::Ready
            } else {
                SetupStatus::Required
            },
            downloaded_bytes: if installed { total_bytes } else { 0 },
            total_bytes,
            error: None,
            hosted_model_ready: false,
        };
        Self {
            app,
            runtime,
            state: Mutex::new(state),
            operation: SetupOperationGate::default(),
            scheduler: Mutex::new(None),
            model_ready: Arc::new(AtomicBool::new(installed)),
            local_ready: AtomicBool::new(installed),
            hosted_active: AtomicBool::new(false),
        }
    }

    /// Records whether a hosted model stands ready, and lets the queue run
    /// on it when the local model is not there.
    fn set_hosted_active(&self, active: bool) {
        self.hosted_active.store(active, Ordering::SeqCst);
        let ready = self.refresh_ready();
        if let Ok(mut current) = self.state.lock() {
            current.hosted_model_ready = active;
            let _ = self.app.emit("setup://progress", current.clone());
        }
        if ready {
            self.wake_scheduler();
        }
    }

    fn refresh_ready(&self) -> bool {
        let ready = model_ready(
            self.local_ready.load(Ordering::SeqCst),
            self.hosted_active.load(Ordering::SeqCst),
        );
        self.model_ready.store(ready, Ordering::SeqCst);
        ready
    }

    fn wake_scheduler(&self) {
        if let Ok(scheduler) = self.scheduler.lock()
            && let Some(sender) = scheduler.as_ref()
        {
            let _ = sender.send(SchedulerMessage::Wake);
        }
    }

    fn get(&self) -> Result<SetupStateDto, CommandError> {
        self.state
            .lock()
            .map(|state| state.clone())
            .map_err(|_| CommandError {
                code: "SETUP_UNAVAILABLE".into(),
                message: "setup state is unavailable".into(),
            })
    }

    fn start(self: &Arc<Self>) -> Result<(), CommandError> {
        if self.local_ready.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.start_operation(SetupSource::Download)
    }

    fn choose_existing(
        self: &Arc<Self>,
        selection: ExistingModelSelection,
    ) -> Result<(), CommandError> {
        if self.local_ready.load(Ordering::SeqCst) {
            return Err(CommandError {
                code: "SETUP_ALREADY_READY".into(),
                message: "local model setup is already complete".into(),
            });
        }
        self.start_operation(SetupSource::Existing(selection))
    }

    /// Starts and verifies a model that is already installed, on the setup
    /// thread.
    ///
    /// Verification loads 1.19 GiB into llama-server, waits for it to answer
    /// its health check, and runs a real inference through it. Launch used to
    /// do that inline in Tauri's setup hook, which runs before the window
    /// paints, so Intern opened as an unresponsive white rectangle for as long
    /// as the machine took - and for the full three minutes the health check
    /// allows when the server was slow to answer. It runs behind the window
    /// instead, and reports where it ended on `setup://progress` like any
    /// other setup operation.
    fn verify_installed(self: &Arc<Self>) {
        if let Err(error) = self.start_operation(SetupSource::Installed) {
            eprintln!(
                "intern: the installed local model could not be verified: {}",
                error.code
            );
        }
    }

    /// Holds the queue without changing what the interface shows. The model
    /// is installed and the window may open on the queue, but no document may
    /// meet a server that has not finished loading - `RuntimeModel::analyze`
    /// would fail it outright with MODEL_NOT_READY.
    fn hold_local_model(&self) {
        self.local_ready.store(false, Ordering::SeqCst);
        self.refresh_ready();
    }

    fn start_operation(self: &Arc<Self>, source: SetupSource) -> Result<(), CommandError> {
        let completed = self.get()?.downloaded_bytes;
        let cancellation = self.operation.begin()?;
        match setup_progress_status(&source) {
            Some(status) => self.set_state(status, completed, None),
            None => self.hold_local_model(),
        }
        let manager = Arc::clone(self);
        std::thread::Builder::new()
            .name("intern-model-setup".into())
            .spawn(move || {
                let result = manager.install_and_start(source, &cancellation);
                // The outcome is settled the moment the work returns, so a
                // cancel racing the last few instructions of a successful
                // setup cannot stop the model that was just started and
                // verified.
                manager.operation.settle();
                let final_state = match result {
                    Ok(total) => (SetupStatus::Ready, total, None),
                    Err(error) if error.code == "MODEL_DOWNLOAD_CANCELED" => {
                        let completed = manager
                            .get()
                            .map(|state| state.downloaded_bytes)
                            .unwrap_or(0);
                        (SetupStatus::Required, completed, Some(error.code))
                    }
                    Err(error) => {
                        let completed = manager
                            .get()
                            .map(|state| state.downloaded_bytes)
                            .unwrap_or(0);
                        (SetupStatus::Failed, completed, Some(error.code))
                    }
                };
                manager.set_state(final_state.0, final_state.1, final_state.2);
                manager.operation.finish();
            })
            .map_err(|_| {
                self.set_state(
                    SetupStatus::Failed,
                    completed,
                    Some("SETUP_UNAVAILABLE".into()),
                );
                self.operation.finish();
                CommandError {
                    code: "SETUP_UNAVAILABLE".into(),
                    message: "setup thread could not start".into(),
                }
            })?;
        Ok(())
    }

    fn cancel(&self) -> Result<(), CommandError> {
        if !self.operation.cancel() {
            return Ok(());
        }
        self.runtime.stop_runtime().map_err(|error| CommandError {
            code: error.code,
            message: "local model setup could not be canceled cleanly".into(),
        })
    }

    fn install_and_start(
        &self,
        source: SetupSource,
        cancellation: &CancellationToken,
    ) -> Result<u64, CommandError> {
        let manifest = ModelManifest::embedded()?;
        let total = manifest.total_bytes();
        match source {
            SetupSource::Installed => {}
            SetupSource::Download => {
                let downloader = Downloader::new(ReqwestHttpTransport::new()?, SystemDiskSpace);
                let mut completed_before = 0;
                for file in &manifest.files {
                    let offset = completed_before;
                    download_with_retry(
                        &downloader,
                        file,
                        &self.runtime.model_directory,
                        cancellation,
                        |progress: SetupProgress| {
                            self.set_state(
                                SetupStatus::Downloading,
                                offset + progress.completed_bytes,
                                None,
                            );
                        },
                    )?;
                    completed_before += file.size;
                }
            }
            SetupSource::Existing(selection) => {
                install_existing_model_files(
                    &manifest,
                    &selection,
                    &self.runtime.model_directory,
                    &SystemDiskSpace,
                    cancellation,
                    |progress| {
                        self.set_state(SetupStatus::Downloading, progress.completed_bytes, None);
                    },
                )?;
            }
        }
        if cancellation.is_canceled() {
            return Err(setup_canceled_error());
        }
        self.runtime.start_verified(&manifest, cancellation)?;
        Ok(total)
    }

    fn set_state(&self, state: SetupStatus, downloaded_bytes: u64, error: Option<String>) {
        let local_ready = matches!(state, SetupStatus::Ready);
        self.local_ready.store(local_ready, Ordering::SeqCst);
        let ready = self.refresh_ready();
        if let Ok(mut current) = self.state.lock() {
            current.state = state;
            current.downloaded_bytes = downloaded_bytes.min(current.total_bytes);
            current.error = error;
            current.hosted_model_ready = self.hosted_active.load(Ordering::SeqCst);
            let _ = self.app.emit("setup://progress", current.clone());
        }
        if ready {
            self.wake_scheduler();
        }
    }
}

/// The queue runs when either model can answer.
fn model_ready(local_ready: bool, hosted_active: bool) -> bool {
    local_ready || hosted_active
}

/// A dropped connection partway through 1.19 GiB is ordinary on an unreliable
/// network and says nothing about whether the download can succeed - only
/// that this attempt did not. Each retry calls the same resumable download
/// again, so it picks the partial file back up rather than starting over; a
/// person on a flaky connection no longer has to notice the failure and press
/// "Try download again" themselves. A refused request, a full disk, or a
/// person canceling are not retried: waiting does not fix them.
const DOWNLOAD_RETRY_LIMIT: u32 = 5;
const DOWNLOAD_RETRY_BACKOFF: Duration = Duration::from_secs(5);

fn download_with_retry<H, D, F>(
    downloader: &Downloader<H, D>,
    file: &ModelFile,
    destination_directory: &Path,
    cancellation: &CancellationToken,
    progress: F,
) -> Result<PathBuf, CommandError>
where
    H: HttpTransport,
    D: DiskSpace,
    F: FnMut(SetupProgress),
{
    download_with_retry_backoff(
        downloader,
        file,
        destination_directory,
        cancellation,
        progress,
        DOWNLOAD_RETRY_BACKOFF,
    )
}

/// `backoff` is the per-attempt unit in production; tests pass a much smaller
/// one so exercising every retry does not spend real seconds asleep.
fn download_with_retry_backoff<H, D, F>(
    downloader: &Downloader<H, D>,
    file: &ModelFile,
    destination_directory: &Path,
    cancellation: &CancellationToken,
    mut progress: F,
    backoff: Duration,
) -> Result<PathBuf, CommandError>
where
    H: HttpTransport,
    D: DiskSpace,
    F: FnMut(SetupProgress),
{
    let mut attempt = 0;
    loop {
        match downloader.download(file, destination_directory, cancellation, &mut progress) {
            Ok(path) => return Ok(path),
            Err(error)
                if attempt < DOWNLOAD_RETRY_LIMIT
                    && matches!(
                        error.code(),
                        EngineErrorCode::DownloadFailed | EngineErrorCode::DownloadInterrupted
                    ) =>
            {
                attempt += 1;
                sleep_respecting_cancellation(backoff * attempt, cancellation);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn sleep_respecting_cancellation(duration: Duration, cancellation: &CancellationToken) {
    let step = Duration::from_millis(200);
    let mut remaining = duration;
    while remaining > Duration::ZERO && !cancellation.is_canceled() {
        let slice = remaining.min(step);
        std::thread::sleep(slice);
        remaining -= slice;
    }
}

enum SetupSource {
    Download,
    Existing(ExistingModelSelection),
    /// A model that was already installed when Intern launched, which needs
    /// only to be started and verified.
    Installed,
}

/// The status to show while a setup operation runs, or `None` to leave the
/// interface saying what it already says.
///
/// A download or an install has bytes to move and no model to run, so the
/// setup screen takes the window. Verifying a model that is already installed
/// is not a download and must not look like one: the screen would offer to
/// fetch 1.19 GiB that is already on disk.
fn setup_progress_status(source: &SetupSource) -> Option<SetupStatus> {
    match source {
        SetupSource::Download | SetupSource::Existing(_) => Some(SetupStatus::Downloading),
        SetupSource::Installed => None,
    }
}

fn setup_canceled_error() -> CommandError {
    CommandError {
        code: "MODEL_DOWNLOAD_CANCELED".into(),
        message: "model setup was canceled".into(),
    }
}

pub(crate) enum SchedulerMessage {
    Wake,
    Shutdown,
}

fn scheduler_actions(timed_out: bool, model_ready: bool) -> (bool, bool) {
    (timed_out, model_ready)
}

struct PipelineScheduler {
    sender: std::sync::mpsc::Sender<SchedulerMessage>,
    join: Mutex<Option<std::thread::JoinHandle<()>>>,
    pipeline: Arc<Pipeline>,
}

impl PipelineScheduler {
    fn start(
        pipeline: Arc<Pipeline>,
        model_ready: Arc<AtomicBool>,
        app: AppHandle,
    ) -> Result<Self, CommandError> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let scheduled_pipeline = Arc::clone(&pipeline);
        let join = std::thread::Builder::new()
            .name("intern-pipeline-scheduler".into())
            .spawn(move || {
                loop {
                    let timed_out = match receiver.recv_timeout(Duration::from_secs(65)) {
                        Ok(SchedulerMessage::Shutdown) => return,
                        Ok(SchedulerMessage::Wake) => false,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => true,
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                    };
                    let (recover, drain) =
                        scheduler_actions(timed_out, model_ready.load(Ordering::SeqCst));
                    if recover && let Err(error) = scheduled_pipeline.recover() {
                        let _ = app.emit(
                            "queue://changed",
                            serde_json::json!({ "error": error.code }),
                        );
                    }
                    if drain && let Err(error) = scheduled_pipeline.run_until_idle() {
                        let _ = app.emit(
                            "queue://changed",
                            serde_json::json!({ "error": error.code }),
                        );
                    }
                }
            })
            .map_err(|_| CommandError {
                code: "STATE_CONFLICT".into(),
                message: "pipeline scheduler could not start".into(),
            })?;
        Ok(Self {
            sender,
            join: Mutex::new(Some(join)),
            pipeline,
        })
    }

    fn wake(&self) -> Result<(), CommandError> {
        self.sender
            .send(SchedulerMessage::Wake)
            .map_err(|_| CommandError {
                code: "STATE_CONFLICT".into(),
                message: "pipeline scheduler is unavailable".into(),
            })
    }
}

impl Drop for PipelineScheduler {
    fn drop(&mut self) {
        let _ = self.pipeline.shutdown();
        let _ = self.sender.send(SchedulerMessage::Shutdown);
        if let Ok(join) = self.join.get_mut()
            && let Some(join) = join.take()
        {
            let _ = join.join();
        }
    }
}

pub struct AppState {
    pipeline: Arc<Pipeline>,
    settings: SettingsStore,
    setup: Arc<SetupManager>,
    scheduler: PipelineScheduler,
    app: AppHandle,
    data_dir: PathBuf,
    identity: Mutex<MachineIdentity>,
    intake: Mutex<Option<IntakeWatcher>>,
    /// Why the watcher is not running even though intake is enabled — a stale
    /// intake folder must surface in `intake_status`, not block launch/save.
    intake_error: Mutex<Option<String>>,
    /// A dedicated read-only connection to the queue database for the history
    /// view. The pipeline owns its store privately; history listing and CSV
    /// export are pure reads, so they take their own SQLite session (WAL
    /// readers never block the pipeline's writes) instead of widening the
    /// pipeline's surface.
    history: QueueStore,
    /// Writes description records beside filed documents when the setting
    /// asks for them; the pipeline reports every completed rename to it.
    ledger: Arc<LedgerSink>,
    /// Leaves a marker in the watched intake folder for every document filed
    /// out of it, and reads the markers teammates left, so the same content
    /// is never filed twice across machines.
    filed_index: Arc<SharedFiledIndex>,
    /// The hosted model, when one is chosen: its key in the credential
    /// store, its engine, and its test.
    hosted: Arc<HostedModel>,
    settings_gate: Mutex<()>,
    sharepoint_activation: AtomicBool,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self, CommandError> {
        let data = app.path().app_local_data_dir().map_err(|_| CommandError {
            code: "APP_DATA_UNAVAILABLE".into(),
            message: "local application data directory is unavailable".into(),
        })?;
        std::fs::create_dir_all(&data).map_err(|_| CommandError {
            code: "APP_DATA_UNAVAILABLE".into(),
            message: "local application data directory could not be created".into(),
        })?;
        let executable_directory = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .ok_or_else(|| CommandError {
                code: "SIDECAR_UNAVAILABLE".into(),
                message: "application executable directory is unavailable".into(),
            })?;
        let worker_name = if cfg!(windows) {
            "intern-worker.exe"
        } else {
            "intern-worker"
        };
        let server_name = if cfg!(windows) {
            "llama-server.exe"
        } else {
            "llama-server"
        };
        let model_directory = data.join("models");
        let runtime = Arc::new(RuntimeModel::new(
            executable_directory.join(server_name),
            model_directory,
        ));
        let manifest = ModelManifest::embedded()?;
        let setup = Arc::new(SetupManager::new(
            app.clone(),
            Arc::clone(&runtime),
            &manifest,
        ));
        if runtime.installed(&manifest) {
            setup.verify_installed();
        }
        let settings = SettingsStore::new(data.join("settings.json"));
        let worker_temp_root = data.join("worker-temp");
        prepare_worker_temp_root(&worker_temp_root, 128).map_err(|_| CommandError {
            code: "APP_DATA_UNAVAILABLE".into(),
            message: "private worker temporary directory is unavailable".into(),
        })?;
        let microsoft = Arc::new(crate::microsoft_intake::MicrosoftIntake::new(
            settings.clone(),
            data.clone(),
        ));
        // Invalid identity configuration holds processing, but the UI must
        // still open so its diagnostic can be read and repaired.
        if let Ok(initial) = settings.load() {
            let _ = microsoft.protect_settings(&initial);
        }
        app.manage(microsoft.clone());
        let ledger = Arc::new(LedgerSink::new(settings.clone(), data.clone()));
        ledger.attach(app.clone());
        let filed_index = Arc::new(SharedFiledIndex::new(settings.clone(), data.clone()));
        let secrets: Arc<dyn SecretStore> = Arc::new(KeyringStore);
        let hosted = Arc::new(HostedModel::new(secrets));
        let model = Arc::new(SwitchingModel::new(
            Arc::clone(&runtime),
            Arc::clone(&hosted),
            settings.clone(),
        ));
        let events = Arc::new(TauriPipelineEvents::new(app.clone()));
        let pipeline = Arc::new(
            Pipeline::with_local_files(
                data.join("queue.sqlite3"),
                Arc::new(SupervisedWorker::with_temp_root(
                    executable_directory.join(worker_name),
                    worker_temp_root,
                )),
                model,
                events.clone(),
                settings.clone(),
            )?
            .with_filing_sink(Arc::new(FilingSinks(vec![
                ledger.clone() as Arc<dyn FilingSink>,
                filed_index.clone(),
                microsoft.clone(),
            ])))
            .with_duplicate_oracle(filed_index.clone())
            .with_admission_guard(microsoft),
        );
        events.watch(&pipeline);
        pipeline.recover()?;
        // Opened after the pipeline so the pipeline's own store has already
        // migrated the schema this connection reads.
        let history = QueueStore::open(data.join("queue.sqlite3")).map_err(|_| CommandError {
            code: "APP_DATA_UNAVAILABLE".into(),
            message: "the rename history database is unavailable".into(),
        })?;
        let scheduler = PipelineScheduler::start(
            Arc::clone(&pipeline),
            Arc::clone(&setup.model_ready),
            app.clone(),
        )?;
        *setup.scheduler.lock().map_err(|_| CommandError {
            code: "STATE_CONFLICT".into(),
            message: "setup scheduler state is unavailable".into(),
        })? = Some(scheduler.sender.clone());
        // Defaults are how the window opens on a settings file nothing can
        // read - Settings is where a person repairs it, and the defaults keep
        // automatic renaming off while they do. What must not happen is
        // Intern behaving as though the file said "no destination, no intake":
        // the file is left exactly as it is until somebody deliberately saves,
        // and `intake_status_dto` reports the trouble to the interface.
        let startup_settings = settings.load().unwrap_or_default();
        let identity = MachineIdentity::load_or_create(&data, &startup_settings.machine_label)
            .map_err(|_| CommandError {
                code: "APP_DATA_UNAVAILABLE".into(),
                message: "machine identity could not be created".into(),
            })?;
        let state = Self {
            pipeline,
            settings,
            setup,
            scheduler,
            app: app.clone(),
            data_dir: data,
            identity: Mutex::new(identity),
            intake: Mutex::new(None),
            intake_error: Mutex::new(None),
            history,
            ledger,
            filed_index,
            hosted,
            settings_gate: Mutex::new(()),
            sharepoint_activation: AtomicBool::new(false),
        };
        state.refresh_hosted_active(&startup_settings);
        if state.setup.model_ready.load(Ordering::SeqCst) {
            state.schedule()?;
        }
        if startup_settings.intake_enabled
            && let Err(error) = state.restart_intake(&startup_settings)
            && let Ok(mut slot) = state.intake_error.lock()
        {
            *slot = Some(format!("{}: {}", error.code, error.message));
        }
        Ok(state)
    }

    fn schedule(&self) -> Result<(), CommandError> {
        if self.setup.model_ready.load(Ordering::SeqCst) {
            self.scheduler.wake()?;
        }
        Ok(())
    }

    /// Tells setup whether a hosted model is chosen and able to answer, so
    /// the queue runs on it - or stops, when the key is gone.
    fn refresh_hosted_active(&self, settings: &AppSettings) {
        let active =
            settings.model_source == ModelSource::Hosted && self.hosted.configured(settings);
        self.setup.set_hosted_active(active);
    }

    /// Stops any running watcher and starts a fresh one when the settings
    /// call for it. The identity is reloaded so a changed machine label takes
    /// effect. An intake folder that no longer canonicalizes is recorded for
    /// `intake_status` instead of returned as an error.
    fn restart_intake(&self, settings: &AppSettings) -> Result<(), CommandError> {
        let identity = MachineIdentity::load_or_create(&self.data_dir, &settings.machine_label)
            .map_err(|_| CommandError {
                code: "APP_DATA_UNAVAILABLE".into(),
                message: "machine identity could not be loaded".into(),
            })?;
        *self.identity.lock().map_err(|_| intake_state_conflict())? = identity.clone();
        let previous = self
            .intake
            .lock()
            .map_err(|_| intake_state_conflict())?
            .take();
        // Dropping the watcher joins its scan thread; never do that while
        // holding the slot's lock.
        drop(previous);
        let mut watcher = None;
        let mut error = None;
        if settings.intake_enabled {
            match canonical_folder(Path::new(&settings.intake_folder)) {
                Ok(folder) => {
                    let mut config = IntakeConfig::new(
                        folder,
                        SUPPORTED_EXTENSIONS
                            .iter()
                            .map(|extension| (*extension).to_owned())
                            .collect(),
                    );
                    config.process_others_uploads = settings.process_others_uploads;
                    let host = Arc::new(PipelineIntakeHost::new(
                        Arc::clone(&self.pipeline),
                        self.scheduler.sender.clone(),
                        Arc::clone(&self.setup.model_ready),
                        self.app.clone(),
                        identity.clone(),
                        Arc::clone(&self.filed_index),
                    ));
                    watcher = Some(IntakeWatcher::start(config, identity, host));
                }
                Err(folder_error) => {
                    error = Some(format!("INTAKE_FOLDER_MISSING: {}", folder_error.message));
                }
            }
        }
        *self.intake.lock().map_err(|_| intake_state_conflict())? = watcher;
        *self
            .intake_error
            .lock()
            .map_err(|_| intake_state_conflict())? = error;
        Ok(())
    }

    fn intake_status_dto(&self) -> Result<IntakeStatusDto, CommandError> {
        let loaded = self.settings.load_with_report();
        let settings = loaded
            .as_ref()
            .map(|loaded| loaded.settings.clone())
            .unwrap_or_default();
        let identity = self
            .identity
            .lock()
            .map_err(|_| intake_state_conflict())?
            .clone();
        let status = self
            .intake
            .lock()
            .map_err(|_| intake_state_conflict())?
            .as_ref()
            .map(IntakeWatcher::status);
        let error = self
            .intake_error
            .lock()
            .map_err(|_| intake_state_conflict())?
            .clone()
            .or_else(|| unreadable_settings(&loaded))
            .or_else(|| {
                self.app
                    .state::<Arc<crate::microsoft_intake::MicrosoftIntake>>()
                    .local_only_contradiction()
            })
            .or_else(|| self.filed_index.last_error());
        Ok(status_dto(
            settings.intake_enabled,
            &identity,
            &settings.intake_folder,
            status.as_ref(),
            error,
            now_unix(),
        ))
    }

    fn emit_intake_changed(&self) -> Result<(), CommandError> {
        let dto = self.intake_status_dto()?;
        let _ = self.app.emit("intake://changed", dto);
        Ok(())
    }

    /// The settings as currently stored, for startup decisions (tray,
    /// start-hidden). A missing file is the defaults, same as `load`.
    pub(crate) fn settings_snapshot(&self) -> AppSettings {
        self.settings.load().unwrap_or_default()
    }

    /// Whether a main-window close should hide to the tray instead of running
    /// the normal exit path. Read fresh on every close so a settings save
    /// takes effect on the very next close, and erring toward `false`: a
    /// broken settings file must fall back to the ordinary exit, never to a
    /// window that hides with no tray to bring it back.
    pub(crate) fn hide_window_on_close(&self) -> bool {
        self.settings
            .load()
            .map(|settings| crate::tray::close_hides_to_tray(settings.run_in_background))
            .unwrap_or(false)
    }
}

/// What a second launch of Intern does.
///
/// Intern is one process per machine: a second one would start a second local
/// model, a second intake watcher, and a second tray against the same queue
/// database, and a sign-in autostart launch followed by a click on the
/// shortcut is an ordinary way to end up with both. The single-instance
/// plugin ends the second process and hands its command line here instead.
pub fn second_instance_launched(app: &AppHandle, arguments: Vec<String>, _directory: String) {
    if second_launch_shows_window(&arguments) {
        crate::tray::show_main_window(app);
    }
}

/// Whether a second launch means "show me the window". Someone who clicked
/// the icon, the shortcut, or a document wants it; a sign-in autostart launch
/// asked for the tray and must not take the window from whatever is using it.
fn second_launch_shows_window(arguments: &[String]) -> bool {
    !arguments.iter().any(|argument| argument == "--minimized")
}

/// The explicit quit path, used by the tray's "Quit Intern" item: shut the
/// pipeline (and with it the local model process) down deliberately, then
/// leave without starting window teardown - the same shape as the close-time
/// exit, which deliberately avoids wedging in WebView destruction.
pub(crate) fn shutdown_and_exit(app: &AppHandle) -> ! {
    shutdown_runtime(app);
    std::process::exit(0);
}

/// Stop the pipeline, and with it the local model process and the parser
/// worker, without leaving. Tauri's exit events and the before-exit guard both
/// call this; it is safe to call more than once, because stopping a runtime
/// that is already stopped does nothing.
pub(crate) fn shutdown_runtime(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let _ = state.pipeline.shutdown();
    }
}

/// A background task that panicked, or was dropped before it finished.
///
/// Nothing the user asked for was refused here, so this must not borrow
/// another failure's reason: reporting a panic as an unverified Microsoft
/// upload sent people to Settings to repair a connection that was working.
/// The panic message itself goes nowhere in a windowed build, so the task
/// that died is named on stderr as well as in the error.
fn background_task_failed(task: &str) -> CommandError {
    eprintln!("intern: the {task} background task did not finish");
    CommandError {
        code: "INTERNAL_ERROR".into(),
        message: format!("{task} could not finish because of an internal error"),
    }
}

fn intake_state_conflict() -> CommandError {
    CommandError {
        code: "STATE_CONFLICT".into(),
        message: "intake state is unavailable".into(),
    }
}

/// What to tell a person about a settings file that could not be read whole.
///
/// Intern opens on defaults so that Settings can be reached and the file
/// repaired, but defaults are not what anybody configured: with them there is
/// no destination and no watched folder, and saying nothing would leave a
/// product that looks healthy and files nothing. So the trouble travels on
/// the intake status, where a startup problem with the watched folder already
/// surfaces, and the file itself is left alone until somebody saves.
fn unreadable_settings(loaded: &Result<LoadedSettings, PipelineError>) -> Option<String> {
    match loaded {
        Ok(loaded) if loaded.unreadable.is_empty() => None,
        Ok(loaded) => Some(format!(
            "SETTINGS_PARTIAL: {} in your settings file could not be read and {} using the default instead. Everything else was kept.",
            loaded.unreadable.join(", "),
            if loaded.unreadable.len() == 1 {
                "is"
            } else {
                "are"
            }
        )),
        Err(error) => Some(format!(
            "{}: your settings file could not be read, so Intern started with defaults - no destination folder, no watched folder, and automatic renaming off. The file has not been written over: repair it, or save from this window to replace it.",
            error.code
        )),
    }
}

/// The queue as the window shows it.
///
/// Blocking: every reviewable item is stat'd for its last-modified date, and
/// the documents can be on a network share, so the listing runs off the
/// thread WebView2 delivers invokes on.
#[tauri::command]
pub async fn queue_list(state: State<'_, AppState>) -> Result<Vec<QueueItemDto>, CommandError> {
    let app = state.app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let items = state.pipeline.list()?;
        // The window asks for the list on every queue change, which makes this
        // the one place that always knows the current counts - so the tray's
        // tooltip is kept here rather than on a second event path.
        let (needs_review, ready) = attention_counts(&items);
        crate::tray::update_tooltip(&app, needs_review, ready);
        items.into_iter().map(queue_item_dto).collect()
    })
    .await
    .map_err(|_| background_task_failed("queue listing"))?
}

/// How many items wait on a person: those needing review, and those ready
/// to rename but not applied automatically.
fn attention_counts(items: &[PipelineItem]) -> (usize, usize) {
    let count = |status: QueueStatus| items.iter().filter(|item| item.status == status).count();
    (count(QueueStatus::NeedsReview), count(QueueStatus::Ready))
}

#[tauri::command]
pub async fn queue_add_files(
    files: Vec<FileSelectionDto>,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let pipeline = state.pipeline.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        let mut paths = Vec::new();
        for file in files {
            let input = Path::new(&file.path);
            match canonical_file(input) {
                Ok(path) => paths.push(path),
                Err(file_error) => match canonical_folder(input) {
                    Ok(folder) => paths.extend(collect_supported_files(&folder)?),
                    Err(_) => return Err(file_error.into()),
                },
            }
        }
        pipeline.enqueue_files(&paths)?;
        Ok(())
    })
    .await
    .map_err(|_| background_task_failed("file intake"))??;
    state.schedule()
}

#[tauri::command]
pub async fn queue_add_folder(
    folder: FolderSelectionDto,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let pipeline = state.pipeline.clone();
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        let folder = canonical_folder(Path::new(&folder.path))?;
        let paths = collect_supported_files(&folder)?;
        pipeline.enqueue_files(&paths)?;
        Ok(())
    })
    .await
    .map_err(|_| background_task_failed("folder intake"))??;
    state.schedule()
}

#[tauri::command]
pub fn queue_pause(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.pause();
    let _ = state
        .app
        .emit("queue://changed", serde_json::json!({ "paused": true }));
    Ok(())
}

#[tauri::command]
pub fn queue_resume(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.resume();
    let _ = state
        .app
        .emit("queue://changed", serde_json::json!({ "paused": false }));
    state.schedule()
}

/// Stops the document being analysed.
///
/// Blocking: cancelling the local model kills llama-server and starts it
/// again, which reloads 1.19 GiB and waits for the new process to answer, so
/// this cannot run on the thread WebView2 delivers invokes on.
#[tauri::command]
pub async fn queue_cancel(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let pipeline = Arc::clone(&state.pipeline);
    let id = parse_item_id(&id)?;
    tauri::async_runtime::spawn_blocking(move || pipeline.cancel(id))
        .await
        .map_err(|_| background_task_failed("cancel"))??;
    Ok(())
}

#[tauri::command]
pub fn queue_retry(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.retry(parse_item_id(&id)?)?;
    state.schedule()
}

#[tauri::command]
pub fn queue_remove(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.remove(parse_item_id(&id)?)?;
    Ok(())
}

#[tauri::command]
pub async fn proposal_approve(
    id: String,
    filename: String,
    description: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let pipeline = state.pipeline.clone();
    let id = parse_item_id(&id)?;
    tauri::async_runtime::spawn_blocking(move || pipeline.approve(id, &filename, &description))
        .await
        .map_err(|_| background_task_failed("rename"))??;
    Ok(())
}

/// The spellings review has taught Intern, newest first.
#[tauri::command]
pub fn house_rules_list(state: State<'_, AppState>) -> Result<Vec<LearnedRuleDto>, CommandError> {
    Ok(state
        .pipeline
        .learned_rules()?
        .into_iter()
        .map(LearnedRuleDto::from)
        .collect())
}

/// Stop applying a learned spelling; documents still waiting go back to
/// the document's own words. Takes effect at once.
#[tauri::command]
pub fn house_rule_forget(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.forget_rule(parse_item_id(&id)?)?;
    Ok(())
}

/// Apply a learned spelling from now on without waiting for a second edit.
#[tauri::command]
pub fn house_rule_use(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.use_rule(parse_item_id(&id)?)?;
    Ok(())
}

#[tauri::command]
pub fn proposal_keep_original(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.keep_original(parse_item_id(&id)?)?;
    Ok(())
}

#[tauri::command]
pub fn operation_undo(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.undo(parse_item_id(&id)?)?;
    Ok(())
}

/// Opens the document a queue item names in the program the system uses for
/// it, so a reviewer can read the page they are being asked to name.
///
/// The window sends an id and never a path: which file that is - the filed
/// copy or the one that arrived - is decided here, from the queue's own
/// record. Blocking: the document can be on a network share, and the shell
/// hand-off waits on it.
#[tauri::command]
pub async fn document_open(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let pipeline = Arc::clone(&state.pipeline);
    let id = parse_item_id(&id)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        let items = pipeline.list()?;
        let path = document_to_open(items.iter().find(|item| item.id == id))?;
        tauri_plugin_opener::open_path(shell_path(&path), None::<&str>).map_err(|_| CommandError {
            code: "OPEN_FAILED".into(),
            message: "the system could not open the document".into(),
        })
    })
    .await
    .map_err(|_| background_task_failed("open document"))?
}

/// Shows the document a queue item names, selected in its folder. Same id-only
/// contract as `document_open`; nothing is run, so any document the queue
/// holds can be shown.
#[tauri::command]
pub async fn document_reveal(id: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    let pipeline = Arc::clone(&state.pipeline);
    let id = parse_item_id(&id)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<(), CommandError> {
        let items = pipeline.list()?;
        let path = locate_document(items.iter().find(|item| item.id == id))?;
        tauri_plugin_opener::reveal_item_in_dir(&path).map_err(|_| CommandError {
            code: "OPEN_FAILED".into(),
            message: "the system could not show the document in its folder".into(),
        })
    })
    .await
    .map_err(|_| background_task_failed("show document"))?
}

/// Where a queue item's document is now: the filed copy once a rename has
/// finished, and the file as it arrived for everything else - including a
/// document kept under its own name, which never moved. `None` while an
/// operation is moving the file, when neither name can be trusted.
fn document_path(item: &PipelineItem) -> Option<PathBuf> {
    match item.status {
        QueueStatus::Applying => None,
        QueueStatus::Completed => Some(match item.receipt.as_ref() {
            Some(receipt)
                if receipt.direction == OperationDirection::Apply
                    && receipt.stage == OperationStage::Complete =>
            {
                receipt.destination.clone()
            }
            // An undo that was rolled back left the filed copy where it was;
            // an undo moves from the filed name back to the original.
            Some(receipt)
                if receipt.direction == OperationDirection::Undo
                    && receipt.stage == OperationStage::RolledBack =>
            {
                receipt.source.clone()
            }
            _ => item.source_path.clone(),
        }),
        _ => Some(item.source_path.clone()),
    }
}

/// The document an item names, provided it is on disk now. A file moved or
/// deleted behind Intern's back is reported as such rather than handed to the
/// shell, which would show its own, less helpful, error.
fn locate_document(item: Option<&PipelineItem>) -> Result<PathBuf, CommandError> {
    let item = item.ok_or_else(|| CommandError {
        code: "ITEM_NOT_FOUND".into(),
        message: "queue item does not exist".into(),
    })?;
    document_path(item)
        .filter(|path| path.is_file())
        .ok_or_else(|| CommandError {
            code: "PATH_UNAVAILABLE".into(),
            message: "the document is not where Intern last saw it".into(),
        })
}

/// The spelling of a queue path to hand to the shell. Queue paths are stored
/// canonical, which on Windows is the verbatim `\\?\C:\...` form: every file
/// API takes it, but `open_path` puts a file's path into ShellExecuteExW as it
/// is, the shell namespace does not reliably accept the prefix, and the
/// viewer it starts receives it through `"%1"` unchanged. Both libraries
/// simplify the path on their other shell routes (Show in folder, a folder's
/// Open); this does the same for a file, keeping the verbatim form only where
/// the plain one would not name it - too long, or a trailing dot or space.
fn shell_path(path: &Path) -> PathBuf {
    PathBuf::from(display_path(path))
}

/// The document Open hands to its program for an item. One held because
/// Intern could not verify who uploaded it is not opened from here: the hold
/// means nobody vouched for the file, and Open would start whatever program
/// handles it from Intern's own window. Show in folder still finds it.
fn document_to_open(item: Option<&PipelineItem>) -> Result<PathBuf, CommandError> {
    if item.is_some_and(|item| {
        item.status == QueueStatus::NeedsReview
            && item.error_code == Some(intern_core::ErrorCode::UploaderUnverified)
    }) {
        return Err(CommandError {
            code: "UPLOADER_UNVERIFIED".into(),
            message:
                "a document from an uploader Intern could not verify is not opened from Intern"
                    .into(),
        });
    }
    openable_document(locate_document(item)?)
}

/// Opening hands the file to whatever program the system associates with it,
/// so only the document formats Intern itself reads are handed over - never
/// an executable, a shortcut or a script. The queue only ever takes those, so
/// this refuses nothing in practice; it is here so that no future path into
/// the queue can make Open launch a program file. A format Intern reads can
/// still carry macros (a .pptm presentation): Office's own macro settings
/// govern those, as when the file is opened from Explorer, and a document
/// from an unverified uploader is not opened at all (`document_to_open`).
fn openable_document(path: PathBuf) -> Result<PathBuf, CommandError> {
    let supported = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            SUPPORTED_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        });
    if supported {
        Ok(path)
    } else {
        Err(CommandError {
            code: "UNSUPPORTED_FORMAT".into(),
            message: "Intern opens only the document formats it reads".into(),
        })
    }
}

/// The settings as the interface should show them: folders in their readable
/// spelling. Storage keeps the canonical form (on Windows, the verbatim
/// `\\?\` prefix that long and oddly named paths need), and `settings_save`
/// canonicalizes whatever comes back, so the round trip is lossless.
#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    let mut settings = state.settings.load()?;
    settings.destination = display_folder(&settings.destination);
    settings.intake_folder = display_folder(&settings.intake_folder);
    Ok(settings)
}

fn display_folder(folder: &str) -> String {
    if folder.trim().is_empty() {
        String::new()
    } else {
        display_path(Path::new(folder))
    }
}

/// Saves the settings.
///
/// Blocking: the folders are canonicalized, which reaches whatever they live
/// on and can be an unreachable network share, and a changed intake folder
/// restarts the watcher, which joins its scan thread. None of that may happen
/// on the thread WebView2 delivers invokes on.
#[tauri::command]
pub async fn settings_save(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let app = state.app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        save_settings(app.state::<AppState>().inner(), settings)
    })
    .await
    .map_err(|_| background_task_failed("settings save"))?
}

/// Every live effect of applying settings. `AppState` is the production
/// implementation; tests drive the same application and rollback code over a
/// real settings file with recorded watcher, tray, and autostart effects,
/// because a Tauri `AppHandle` cannot be constructed outside the app.
pub(crate) trait SettingsRuntime {
    fn load_settings(&self) -> Result<AppSettings, CommandError>;
    fn persist_settings(&self, settings: &AppSettings) -> Result<(), CommandError>;
    fn canonical_folder(&self, path: &Path) -> Result<PathBuf, CommandError>;
    fn check_hosted_model(&self, settings: &AppSettings) -> Result<(), CommandError>;
    fn protect_microsoft(&self, settings: &AppSettings) -> Result<(), CommandError>;
    fn set_autostart(&self, enabled: bool) -> Result<(), CommandError>;
    fn refresh_hosted_active(&self, settings: &AppSettings);
    fn schedule(&self) -> Result<(), CommandError>;
    fn sync_tray(&self, run_in_background: bool);
    fn restart_intake(&self, settings: &AppSettings) -> Result<(), CommandError>;
    fn emit_intake_changed(&self) -> Result<(), CommandError>;
    /// The paths a completed SharePoint activation owns, if one is active.
    fn managed_sharepoint(
        &self,
        stored: Option<&AppSettings>,
    ) -> Result<Option<ManagedSharePoint>, CommandError>;
    /// Serializes whole settings applications so a Settings save cannot
    /// interleave with a SharePoint activation or its rollback.
    fn settings_gate(&self) -> &Mutex<()>;
    /// Set, under the settings gate, while a SharePoint activation is between
    /// reading the settings it may restore and finishing.
    fn sharepoint_activation(&self) -> &AtomicBool;
}

impl SettingsRuntime for AppState {
    fn load_settings(&self) -> Result<AppSettings, CommandError> {
        self.settings.load().map_err(CommandError::from)
    }

    fn persist_settings(&self, settings: &AppSettings) -> Result<(), CommandError> {
        self.settings.save(settings).map_err(CommandError::from)
    }

    fn canonical_folder(&self, path: &Path) -> Result<PathBuf, CommandError> {
        canonical_folder(path).map_err(CommandError::from)
    }

    fn check_hosted_model(&self, settings: &AppSettings) -> Result<(), CommandError> {
        self.hosted.config(settings).map(|_| ())
    }

    fn protect_microsoft(&self, settings: &AppSettings) -> Result<(), CommandError> {
        self.app
            .state::<Arc<crate::microsoft_intake::MicrosoftIntake>>()
            .protect_settings(settings)
            .map_err(|message| CommandError {
                code: "UPLOADER_UNVERIFIED".into(),
                message,
            })
    }

    fn set_autostart(&self, enabled: bool) -> Result<(), CommandError> {
        apply_autostart(&self.app, enabled)
    }

    fn refresh_hosted_active(&self, settings: &AppSettings) {
        AppState::refresh_hosted_active(self, settings);
    }

    fn schedule(&self) -> Result<(), CommandError> {
        AppState::schedule(self)
    }

    fn sync_tray(&self, run_in_background: bool) {
        crate::tray::sync_tray(&self.app, run_in_background);
        // A tray that was just created starts with the bare tooltip; give it
        // the current counts rather than waiting for the next queue change.
        if run_in_background && let Ok(items) = self.pipeline.list() {
            let (needs_review, ready) = attention_counts(&items);
            crate::tray::update_tooltip(&self.app, needs_review, ready);
        }
    }

    fn restart_intake(&self, settings: &AppSettings) -> Result<(), CommandError> {
        AppState::restart_intake(self, settings)
    }

    fn emit_intake_changed(&self) -> Result<(), CommandError> {
        AppState::emit_intake_changed(self)
    }

    fn managed_sharepoint(
        &self,
        stored: Option<&AppSettings>,
    ) -> Result<Option<ManagedSharePoint>, CommandError> {
        managed_sharepoint_for(
            &self
                .app
                .state::<Arc<crate::microsoft_intake::MicrosoftIntake>>(),
            stored,
        )
    }

    fn settings_gate(&self) -> &Mutex<()> {
        &self.settings_gate
    }

    fn sharepoint_activation(&self) -> &AtomicBool {
        &self.sharepoint_activation
    }
}

pub(crate) fn save_settings(
    state: &impl SettingsRuntime,
    mut settings: AppSettings,
) -> Result<(), CommandError> {
    let _gate = state
        .settings_gate()
        .lock()
        .map_err(|_| settings_gate_unavailable())?;
    // An activation that fails puts back the settings it read when it began,
    // so a save accepted meanwhile would be reported saved and then undone.
    if state.sharepoint_activation().load(Ordering::SeqCst) {
        return Err(sharepoint_activation_in_progress());
    }
    let stored = state.load_settings();
    if let Some(managed) = state.managed_sharepoint(stored.as_ref().ok())? {
        managed.apply(&mut settings);
    }
    save_settings_with_microsoft_protection(state, settings, true)
}

/// The Inbox and Filed paths a completed fixed SharePoint activation owns.
pub(crate) struct ManagedSharePoint {
    pub inbox: String,
    pub destination: String,
}

impl ManagedSharePoint {
    /// Normalizes rather than rejects. The Settings dialog sends back every
    /// field it loaded, in display spelling, so comparing a payload against
    /// the managed values would refuse harmless saves, while overwriting holds
    /// the invariant whatever the webview sends. Hiding controls is not the
    /// enforcement; this is. Every unrelated field is saved as sent.
    fn apply(&self, settings: &mut AppSettings) {
        settings.intake_folder = self.inbox.clone();
        settings.destination = self.destination.clone();
        settings.intake_enabled = true;
        settings.process_others_uploads = false;
        settings.intake_local_only = false;
        settings.intake_my_folder = false;
        settings.run_in_background = true;
        settings.start_at_login = true;
        settings.start_minimized = true;
    }
}

fn managed_sharepoint_for(
    microsoft: &crate::microsoft_intake::MicrosoftIntake,
    stored: Option<&AppSettings>,
) -> Result<Option<ManagedSharePoint>, CommandError> {
    microsoft
        .managed_paths(stored)
        .map(|paths| paths.map(|(inbox, destination)| ManagedSharePoint { inbox, destination }))
        .map_err(|message| CommandError {
            code: "SHAREPOINT_MANAGED_SETTINGS_UNAVAILABLE".into(),
            message,
        })
}

fn save_settings_with_microsoft_protection(
    state: &impl SettingsRuntime,
    mut settings: AppSettings,
    protect_microsoft: bool,
) -> Result<(), CommandError> {
    let previous = state.load_settings().unwrap_or_default();
    validate_intake_settings(&mut settings, &previous.intake_folder, &|path| {
        state.canonical_folder(path).ok()
    })?;
    // With intake enabled the destination was already canonicalized (with the
    // intake-specific error code); otherwise keep the original behavior.
    if !settings.intake_enabled && !settings.destination.trim().is_empty() {
        settings.destination = state
            .canonical_folder(Path::new(&settings.destination))?
            .to_string_lossy()
            .into_owned();
    }
    validate_description_settings(&settings)?;
    // A hosted model that could not be sent to is refused at save time, like
    // every other configuration that could never do anything: the key must
    // be in the credential store and the address must be one a key may be
    // sent to.
    if settings.model_source == ModelSource::Hosted {
        state.check_hosted_model(&settings)?;
    }
    if protect_microsoft {
        state.protect_microsoft(&settings)?;
    }
    save_settings_and_autostart(
        &previous,
        &settings,
        |settings| state.persist_settings(settings),
        |enabled| state.set_autostart(enabled),
    )?;
    state.refresh_hosted_active(&settings);
    if previous.model_source != settings.model_source {
        state.schedule()?;
    }
    if previous.run_in_background != settings.run_in_background {
        state.sync_tray(settings.run_in_background);
    }
    if previous.intake_folder != settings.intake_folder
        || previous.intake_enabled != settings.intake_enabled
        || previous.intake_local_only != settings.intake_local_only
        || previous.intake_my_folder != settings.intake_my_folder
        || previous.process_others_uploads != settings.process_others_uploads
        || previous.machine_label != settings.machine_label
    {
        state.restart_intake(&settings)?;
        state.emit_intake_changed()?;
    }
    Ok(())
}

fn sharepoint_activation_in_progress() -> CommandError {
    CommandError {
        code: "SHAREPOINT_ACTIVATION_IN_PROGRESS".into(),
        message: "SharePoint setup is being activated, so settings were not changed. Try again when it finishes.".into(),
    }
}

/// Starts a SharePoint activation: under the settings gate, marks it running
/// (refusing a second one) and returns the settings stored at that moment,
/// which a failed activation restores. Until `end_sharepoint_activation`,
/// `save_settings` refuses with SHAREPOINT_ACTIVATION_IN_PROGRESS.
pub(crate) fn begin_sharepoint_activation(
    runtime: &impl SettingsRuntime,
) -> Result<AppSettings, CommandError> {
    let _gate = runtime
        .settings_gate()
        .lock()
        .map_err(|_| settings_gate_unavailable())?;
    if runtime.sharepoint_activation().swap(true, Ordering::SeqCst) {
        return Err(sharepoint_activation_in_progress());
    }
    runtime.load_settings().inspect_err(|_| {
        runtime
            .sharepoint_activation()
            .store(false, Ordering::SeqCst)
    })
}

pub(crate) fn end_sharepoint_activation(runtime: &impl SettingsRuntime) {
    runtime
        .sharepoint_activation()
        .store(false, Ordering::SeqCst);
}

fn settings_gate_unavailable() -> CommandError {
    CommandError {
        code: "STATE_CONFLICT".into(),
        message: "settings are being changed elsewhere and are unavailable".into(),
    }
}

/// Applies SharePoint's derived settings through the same persistence and
/// live-runtime path as the Settings UI. If any watcher, tray, hosted
/// runtime, intake event, or autostart step fails, the settings stored before
/// the attempt are restored exactly (see `restore_settings_unlocked`), and a
/// restore that also fails is reported as ACTIVATION_ROLLBACK_FAILED with both
/// errors. Microsoft protection is the caller's transaction, not this one.
pub(crate) fn activate_sharepoint_settings(
    runtime: &impl SettingsRuntime,
    settings: &AppSettings,
) -> Result<(), CommandError> {
    let _gate = runtime
        .settings_gate()
        .lock()
        .map_err(|_| settings_gate_unavailable())?;
    let previous = runtime.load_settings()?;
    if let Err(error) = save_settings_with_microsoft_protection(runtime, settings.clone(), false) {
        return match restore_settings_unlocked(runtime, &previous) {
            Ok(()) => Err(error),
            Err(restore) => Err(CommandError {
                code: "ACTIVATION_ROLLBACK_FAILED".into(),
                message: format!(
                    "{} Restoring the prior live settings also failed: {}. Activation remains incomplete.",
                    error.message, restore.message
                ),
            }),
        };
    }
    Ok(())
}

/// Restores settings that were stored before a SharePoint activation, and the
/// live runtime they describe. A failed restore is only ever reported; it
/// never falls back to re-applying the activated settings.
pub(crate) fn restore_sharepoint_settings(
    runtime: &impl SettingsRuntime,
    previous: &AppSettings,
) -> Result<(), CommandError> {
    let _gate = runtime
        .settings_gate()
        .lock()
        .map_err(|_| settings_gate_unavailable())?;
    restore_settings_unlocked(runtime, previous)
}

/// Deliberately not the validated save path: `previous` was already the stored
/// truth, and a folder or hosted-model key that has gone away since must not
/// refuse the restore and strand the activated settings. Every step is tried
/// even after one fails, and the tray and watcher are always re-applied because
/// the live runtime need not match what was persisted when the failure struck.
fn restore_settings_unlocked(
    runtime: &impl SettingsRuntime,
    previous: &AppSettings,
) -> Result<(), CommandError> {
    let current = runtime.load_settings().ok();
    let mut failures = Vec::new();
    if let Err(error) = runtime.persist_settings(previous) {
        failures.push(error);
    }
    if current
        .as_ref()
        .is_none_or(|current| current.start_at_login != previous.start_at_login)
        && let Err(error) = runtime.set_autostart(previous.start_at_login)
    {
        failures.push(error);
    }
    runtime.refresh_hosted_active(previous);
    if let Err(error) = runtime.schedule() {
        failures.push(error);
    }
    runtime.sync_tray(previous.run_in_background);
    if let Err(error) = runtime.restart_intake(previous) {
        failures.push(error);
    }
    if let Err(error) = runtime.emit_intake_changed() {
        failures.push(error);
    }
    let Some(first) = failures.first() else {
        return Ok(());
    };
    Err(CommandError {
        code: first.code.clone(),
        message: failures
            .iter()
            .map(|failure| format!("{}: {}", failure.code, failure.message))
            .collect::<Vec<_>>()
            .join("; "),
    })
}

/// Stores the settings and brings the operating system's login entry into
/// line with them.
///
/// The order matters in both directions. Toggling the entry first left it
/// changed when the save that followed failed, so the login entry and the
/// settings disagreed with nothing on screen to say so. Saving first and
/// putting the previous settings back when registration is refused keeps the
/// older promise as well - a refused login entry leaves the stored settings
/// unchanged - and the refusal is what the dialog reports, because it is the
/// thing that went wrong.
fn save_settings_and_autostart(
    previous: &AppSettings,
    settings: &AppSettings,
    save: impl Fn(&AppSettings) -> Result<(), CommandError>,
    autostart: impl Fn(bool) -> Result<(), CommandError>,
) -> Result<(), CommandError> {
    save(settings)?;
    if previous.start_at_login != settings.start_at_login
        && let Err(error) = autostart(settings.start_at_login)
    {
        let _ = save(previous);
        return Err(error);
    }
    Ok(())
}

/// Enables or disables the start-at-login entry to match the setting.
///
/// Registration lives with the operating system and can be refused (a locked
/// registry key, a read-only autostart directory); that refusal becomes an
/// ordinary save error the dialog can show, never a crash.
fn apply_autostart(app: &AppHandle, enabled: bool) -> Result<(), CommandError> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|_| CommandError {
        code: "AUTOSTART_FAILED".into(),
        message: if enabled {
            "starting Intern at sign-in could not be enabled".into()
        } else {
            "starting Intern at sign-in could not be disabled".into()
        },
    })
}

/// Intake-related validation for `settings_save`, before anything persists.
///
/// A watched intake with an in-place rename would loop (the renamed file
/// reappears as new), so intake enabled requires a real destination outside
/// the intake folder. Canonical forms are written back so later comparisons
/// and the containment check are component-wise, never string-prefix.
/// `canonicalize` is `canonical_folder` in production and a seam for tests;
/// `stored` is the intake folder as it is saved now.
fn validate_intake_settings(
    settings: &mut AppSettings,
    stored: &str,
    canonicalize: &dyn Fn(&Path) -> Option<PathBuf>,
) -> Result<(), CommandError> {
    if settings.intake_folder.trim().is_empty() {
        if settings.intake_enabled {
            return Err(CommandError {
                code: "INTAKE_FOLDER_MISSING".into(),
                message: "an intake folder must be chosen".into(),
            });
        }
    } else {
        settings.intake_folder = match canonicalize(Path::new(&settings.intake_folder)) {
            Some(folder) => folder.to_string_lossy().into_owned(),
            None if settings.intake_enabled => {
                return Err(CommandError {
                    code: "INTAKE_FOLDER_MISSING".into(),
                    message: "the intake folder does not exist".into(),
                });
            }
            // Intake is off, so nothing is read from this folder: an offline
            // network share must not refuse a save that has nothing to do
            // with it. The spelling already stored is kept, so it is still
            // canonical when intake is turned back on - and when nothing is
            // stored yet, what the person chose is kept rather than cleared.
            None if !stored.trim().is_empty() => stored.to_owned(),
            None => settings.intake_folder.clone(),
        };
    }
    if !settings.intake_enabled {
        return Ok(());
    }
    if settings.destination.trim().is_empty() {
        return Err(CommandError {
            code: "INTAKE_NEEDS_DESTINATION".into(),
            message: "watching an intake folder requires a destination folder".into(),
        });
    }
    let destination =
        canonicalize(Path::new(&settings.destination)).ok_or_else(|| CommandError {
            code: "INTAKE_NEEDS_DESTINATION".into(),
            message: "the destination folder does not exist".into(),
        })?;
    if destination.starts_with(Path::new(&settings.intake_folder)) {
        return Err(CommandError {
            code: "DESTINATION_INSIDE_INTAKE".into(),
            message: "the destination cannot be the intake folder or live inside it".into(),
        });
    }
    settings.destination = destination.to_string_lossy().into_owned();
    Ok(())
}

/// Description records live under the destination folder, so asking for them
/// without one is a configuration that could never do anything. Refused at
/// save time, like the intake rules, rather than silently ignored.
fn validate_description_settings(settings: &AppSettings) -> Result<(), CommandError> {
    if settings.record_descriptions && settings.destination.trim().is_empty() {
        return Err(CommandError {
            code: "DESCRIPTIONS_NEED_DESTINATION".into(),
            message: "description records need a destination folder to live in".into(),
        });
    }
    Ok(())
}

#[tauri::command]
pub fn intake_status(state: State<'_, AppState>) -> Result<IntakeStatusDto, CommandError> {
    state.intake_status_dto()
}

#[tauri::command]
pub fn hosted_model_status(
    state: State<'_, AppState>,
) -> Result<HostedModelStatusDto, CommandError> {
    let settings = state.settings.load().unwrap_or_default();
    Ok(state.hosted.status(&settings))
}

/// Stores the API key in the operating system's credential store. The key
/// travels from the dialog to here and no further; it is never written to
/// the settings file.
#[tauri::command]
pub fn hosted_model_set_key(key: String, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.hosted.set_key(&key)?;
    let settings = state.settings.load().unwrap_or_default();
    state.refresh_hosted_active(&settings);
    Ok(())
}

#[tauri::command]
pub fn hosted_model_clear_key(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.hosted.clear_key()?;
    let settings = state.settings.load().unwrap_or_default();
    state.refresh_hosted_active(&settings);
    Ok(())
}

/// Sends the calibration document to the hosted model described by
/// `settings` - the dialog's draft, so what is tested is what is on screen -
/// with the stored key.
#[tauri::command]
pub async fn hosted_model_test(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<HostedModelTestDto, CommandError> {
    let hosted = Arc::clone(&state.hosted);
    let saved = state.settings.load().unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || hosted.test(&settings, &saved))
        .await
        .map_err(|_| background_task_failed("hosted model test"))?
}

/// The OneDrive accounts and SharePoint libraries the sync client keeps on
/// this machine. A local lookup of the sync client's own configuration; no
/// network request is made.
#[tauri::command]
pub fn cloud_roots() -> Result<Vec<CloudRootDto>, CommandError> {
    Ok(list_cloud_roots())
}

/// How many documents a folder already holds, counted exactly as adding the
/// folder to the queue would collect them, so setup can ask once whether to
/// rename those too.
#[tauri::command]
pub async fn intake_folder_documents(path: String) -> Result<usize, CommandError> {
    tauri::async_runtime::spawn_blocking(move || -> Result<usize, CommandError> {
        let folder = canonical_folder(Path::new(&path))?;
        Ok(collect_supported_files(&folder)?.len())
    })
    .await
    .map_err(|_| background_task_failed("folder count"))?
}

/// Creates the "Filed" folder beside a chosen intake folder, or finds the one
/// already there, and returns where it is.
#[tauri::command]
pub fn filed_folder_create(intake_folder: String) -> Result<String, CommandError> {
    let intake = canonical_folder(Path::new(&intake_folder))?;
    let filed = filed_folder_for(&intake).ok_or_else(|| CommandError {
        code: "FILED_FOLDER_UNAVAILABLE".into(),
        message: "a drive's top folder has nothing beside it to file into".into(),
    })?;
    std::fs::create_dir_all(&filed).map_err(|error| CommandError {
        code: "FILED_FOLDER_UNAVAILABLE".into(),
        message: format!("the Filed folder could not be created: {error}"),
    })?;
    Ok(display_path(&filed))
}

/// Creates (or finds) an "Inbox" folder inside a OneDrive or SharePoint
/// folder's own top folder, for folder setup to watch: a Filed folder beside
/// the top folder would sit outside what OneDrive syncs. Refused for any
/// other folder, so the webview cannot make folders anywhere it likes.
#[tauri::command]
pub fn inbox_folder_create(root: String) -> Result<String, CommandError> {
    let unavailable = |message: String| CommandError {
        code: "INBOX_FOLDER_UNAVAILABLE".into(),
        message,
    };
    let root = canonical_folder(Path::new(&root))?;
    let synced = intern_intake::detect_cloud_roots()
        .iter()
        .any(|candidate| canonical_folder(&candidate.root).is_ok_and(|found| found == root));
    if !synced {
        return Err(unavailable(
            "only a OneDrive or SharePoint folder's top folder gets an Inbox".into(),
        ));
    }
    let inbox = root.join("Inbox");
    std::fs::create_dir_all(&inbox)
        .map_err(|error| unavailable(format!("the Inbox folder could not be created: {error}")))?;
    Ok(display_path(&inbox))
}

/// Starts OneDrive, or opens its folder when it is already running.
#[tauri::command]
pub fn onedrive_open() -> Result<(), CommandError> {
    crate::sharepoint_setup::open_one_drive().map_err(|error| CommandError {
        code: error.code,
        message: error.message,
    })
}

#[tauri::command]
pub fn descriptions_status(
    state: State<'_, AppState>,
) -> Result<DescriptionsStatusDto, CommandError> {
    Ok(state.ledger.status())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillResultDto {
    pub written: u32,
    pub failed: u32,
}

/// Writes a description record for every document Intern has already filed
/// and not undone, for a records folder switched on after the fact. Each
/// document's record is rewritten from the queue's own copy of its sentence
/// and facts, so running it twice changes nothing.
#[tauri::command]
pub fn descriptions_backfill(
    state: State<'_, AppState>,
) -> Result<BackfillResultDto, CommandError> {
    let settings = state.settings.load()?;
    if !settings.record_descriptions {
        return Err(CommandError {
            code: "DESCRIPTIONS_DISABLED".into(),
            message: "turn on description records and save before writing them".into(),
        });
    }
    let documents = state.pipeline.filed_documents()?;
    let (written, failed) = state.ledger.backfill(&documents);
    Ok(BackfillResultDto { written, failed })
}

#[tauri::command]
pub fn intake_scan_now(state: State<'_, AppState>) -> Result<(), CommandError> {
    let watcher = state.intake.lock().map_err(|_| intake_state_conflict())?;
    if let Some(watcher) = watcher.as_ref() {
        watcher.scan_now();
    }
    Ok(())
}

#[tauri::command]
pub fn folder_classify(path: String) -> Result<Option<CloudLocationDto>, CommandError> {
    Ok(classify_folder(&path))
}

#[tauri::command]
pub fn setup_get(state: State<'_, AppState>) -> Result<SetupStateDto, CommandError> {
    state.setup.get()
}

#[tauri::command]
pub fn setup_start(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.setup.start()
}

#[tauri::command]
pub fn setup_cancel(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.setup.cancel()
}

#[tauri::command]
pub fn setup_choose_existing(
    files: ExistingModelFilesDto,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let model_path = canonical_model_file(Path::new(&files.model_path))?;
    state
        .setup
        .choose_existing(ExistingModelSelection { model_path })
}

#[tauri::command]
pub fn history_clear(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.pipeline.clear_history()?;
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryDto {
    receipt_id: String,
    queue_item_id: String,
    /// Unix seconds; the frontend formats it locally, the CSV as ISO-8601 UTC.
    at: i64,
    /// "apply" | "undo" (serde snake_case of the core enum).
    direction: OperationDirection,
    /// "rename" | "verified_copy".
    kind: OperationKind,
    /// "complete" | "rolled_back" — only terminal stages are listed.
    stage: OperationStage,
    original_path: String,
    new_path: String,
    /// The one-sentence description that was applied with the rename, when
    /// the item still has its proposal.
    description: Option<String>,
}

fn history_entry_dto(entry: HistoryEntry, description: Option<String>) -> HistoryEntryDto {
    HistoryEntryDto {
        receipt_id: entry.receipt_id.to_string(),
        queue_item_id: entry.queue_item_id.to_string(),
        at: entry.at,
        direction: entry.direction,
        kind: entry.kind,
        stage: entry.stage,
        original_path: display_path(&entry.original_path),
        new_path: display_path(&entry.new_path),
        description,
    }
}

/// The applied description for every queue item that still has a proposal,
/// so history rows and the CSV export can carry the sentence beside the
/// rename it belongs to.
fn descriptions_by_item(
    state: &AppState,
) -> Result<std::collections::HashMap<i64, String>, CommandError> {
    Ok(state
        .pipeline
        .list()?
        .into_iter()
        .filter_map(|item| {
            item.proposal
                .map(|proposal| (item.id, proposal.description))
        })
        .collect())
}

fn history_read_error(error: intern_core::InternError) -> CommandError {
    CommandError {
        code: error.code().as_str().into(),
        message: "the rename history could not be read".into(),
    }
}

fn history_export_failed(message: impl Into<String>) -> CommandError {
    CommandError {
        code: "HISTORY_EXPORT_FAILED".into(),
        message: message.into(),
    }
}

#[tauri::command]
pub fn history_list(state: State<'_, AppState>) -> Result<Vec<HistoryEntryDto>, CommandError> {
    let descriptions = descriptions_by_item(&state)?;
    Ok(state
        .history
        .list_operation_history(HISTORY_LIMIT)
        .map_err(history_read_error)?
        .into_iter()
        .map(|entry| {
            let description = descriptions.get(&entry.queue_item_id).cloned();
            history_entry_dto(entry, description)
        })
        .collect())
}

/// Where the history CSV may be written.
///
/// The path comes from the native save dialog, so it is expected to be
/// absolute, in a folder that exists, and named the way the dialog's own
/// filter names it. Anything else is refused before a byte is written rather
/// than being resolved against whatever the process's working directory
/// happens to be - and the extension matters because the window is what
/// chooses the path, so this is the only thing standing between a page that
/// should not be there and a file elsewhere on the machine being overwritten.
fn history_export_destination(path: &str) -> Result<&Path, CommandError> {
    let destination = Path::new(path);
    if !destination.is_absolute() {
        return Err(history_export_failed("the export path must be absolute"));
    }
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| history_export_failed("the export path has no parent folder"))?;
    if !parent.is_dir() {
        return Err(history_export_failed("the export folder does not exist"));
    }
    if destination
        .extension()
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("csv"))
    {
        return Err(history_export_failed("the export must be a CSV file"));
    }
    Ok(destination)
}

/// Writes the rename history to `path` as RFC 4180 CSV and reports how many
/// operations were written.
#[tauri::command]
pub fn history_export(path: String, state: State<'_, AppState>) -> Result<usize, CommandError> {
    let destination = history_export_destination(&path)?;
    let entries = state
        .history
        .list_operation_history(HISTORY_LIMIT)
        .map_err(history_read_error)?;
    let descriptions = descriptions_by_item(&state)?;
    std::fs::write(destination, history_csv(&entries, &descriptions))
        .map_err(|_| history_export_failed("the history CSV could not be written"))?;
    Ok(entries.len())
}

/// Renders history entries as RFC 4180 CSV: CRLF row endings, and any field
/// containing a comma, quote, or line break is quoted with quotes doubled.
/// The description column is last, so a spreadsheet opened from this export
/// can be pasted straight into a SharePoint grid view beside the filenames.
fn history_csv(
    entries: &[HistoryEntry],
    descriptions: &std::collections::HashMap<i64, String>,
) -> String {
    let mut csv = String::from("at,direction,kind,stage,originalPath,newPath,description\r\n");
    for entry in entries {
        let fields = [
            iso8601_utc(entry.at),
            match entry.direction {
                OperationDirection::Apply => "apply".into(),
                OperationDirection::Undo => "undo".into(),
            },
            match entry.kind {
                OperationKind::Rename => "rename".into(),
                OperationKind::VerifiedCopy => "verified_copy".into(),
            },
            match entry.stage {
                OperationStage::Complete => "complete".to_owned(),
                OperationStage::RolledBack => "rolled_back".to_owned(),
                // Unreachable for listed history (terminal stages only), but a
                // receipt must never be silently mislabeled if that changes.
                other => format!("{other:?}").to_lowercase(),
            },
            display_path(&entry.original_path),
            display_path(&entry.new_path),
            descriptions
                .get(&entry.queue_item_id)
                .cloned()
                .unwrap_or_default(),
        ];
        let row = fields
            .iter()
            .map(|field| csv_field(field))
            .collect::<Vec<_>>()
            .join(",");
        csv.push_str(&row);
        csv.push_str("\r\n");
    }
    csv
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

/// Formats unix seconds as ISO-8601 UTC ("2024-04-12T09:30:00Z") without
/// pulling in a date-time dependency (days-from-civil inverse, Howard
/// Hinnant's algorithm).
fn iso8601_utc(unix_seconds: i64) -> String {
    let days = unix_seconds.div_euclid(86_400);
    let seconds = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds / 3600,
        (seconds / 60) % 60,
        seconds % 60
    )
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    (year, month as u32, day as u32)
}

/// Abandons every item still waiting, for a folder chosen by mistake.
///
/// Returns the number dropped so the interface can say what it did rather than
/// leaving the user to count rows.
#[tauri::command]
pub fn queue_discard_waiting(state: State<'_, AppState>) -> Result<usize, CommandError> {
    Ok(state.pipeline.discard_waiting()?)
}

fn queue_item_dto(item: PipelineItem) -> Result<QueueItemDto, CommandError> {
    let proposal = item.proposal.as_ref();
    let evidence = proposal.map(|record| EvidenceDto {
        date: record.analysis.proposal.evidence.date.clone(),
        r#type: record.analysis.proposal.evidence.document_type.clone(),
        parties: (!record.analysis.proposal.parties.is_empty())
            .then(|| record.analysis.proposal.parties.join("; ")),
    });
    let reconciliation = item
        .receipt
        .as_ref()
        .filter(|receipt| {
            item.status == QueueStatus::NeedsReview
                && item.error_code == Some(intern_core::ErrorCode::SourceDeleteFailed)
                && receipt.direction == OperationDirection::Apply
                && receipt.stage == OperationStage::Published
        })
        .map(|receipt| ReconciliationDto {
            source_path: display_path(&receipt.source),
            destination_path: display_path(&receipt.destination),
            error_code: "SOURCE_DELETE_FAILED".into(),
        });
    Ok(QueueItemDto {
        id: item.id.to_string(),
        original_filename: item
            .source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Document")
            .to_owned(),
        status: item.status,
        proposed_filename: proposal.map(|record| record.filename.clone()),
        confidence: proposal.map(|record| record.analysis.proposal.confidence),
        description: proposal.map(|record| record.description.clone()),
        evidence,
        reason: proposal
            .filter(|record| !record.reasons.is_empty())
            .map(|record| record.reasons.join(", "))
            // A DUPLICATE flag is raised before analysis, so no proposal
            // carries its reason; name the file the content is already
            // filed under.
            .or_else(|| {
                item.duplicate_of
                    .as_deref()
                    .map(|name| format!("Duplicate of {name}"))
            }),
        error_code: item.error_code.map(|code| code.as_str().to_owned()),
        undoable: item.status == QueueStatus::Completed
            && item.receipt.as_ref().is_some_and(|receipt| {
                receipt.direction == OperationDirection::Apply
                    && receipt.stage == OperationStage::Complete
            }),
        approved: item.status == QueueStatus::Ready
            && proposal.is_some_and(|record| record.approved),
        proposal_revision: proposal.map(|record| record.revision.to_string()),
        reconciliation,
        suggested_date: proposal.and_then(|record| suggested_date(&record.analysis)),
        dates_in_document: proposal
            .map(|record| record.analysis.stated_dates.clone())
            .unwrap_or_default(),
        // One stat per reviewable item per listing; a settled item's source
        // is gone or no longer of interest.
        file_modified_date: matches!(item.status, QueueStatus::NeedsReview | QueueStatus::Ready)
            .then(|| file_modified_date(&item.source_path))
            .flatten(),
        house_rules: proposal
            .map(|record| record.house_rules.iter().map(HouseRuleDto::from).collect())
            .unwrap_or_default(),
        near_duplicate_of: proposal.and_then(|record| record.near_duplicate_of.clone()),
    })
}

/// The calendar date, in this machine's time zone, that a file was last
/// modified - or `None` when the file cannot be read.
fn file_modified_date(path: &Path) -> Option<String> {
    let modified = std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()?;
    Some(
        chrono::DateTime::<chrono::Local>::from(modified)
            .format("%Y-%m-%d")
            .to_string(),
    )
}

#[cfg(test)]
mod intake_tests {
    use std::path::{Path, PathBuf};

    use intern_core::{
        OperationDirection, OperationKind, OperationReceipt, OperationStage, QueueStatus,
    };
    use intern_intake::{CloudProviderKind, DoneOutcome, ItemState, MachineIdentity};
    use intern_queue::AppSettings;

    use super::{
        save_settings_and_autostart, validate_description_settings, validate_intake_settings,
    };
    use crate::intake::{
        CloudProviderDto, filed_folder_for, item_fate, lists_one_drive, presence_active, status_dto,
    };

    /// A fake folder canonicalizer: pairs of (as-entered, canonical form).
    fn canonicalizer(
        known: &'static [(&'static str, &'static str)],
    ) -> impl Fn(&Path) -> Option<PathBuf> {
        move |path| {
            known
                .iter()
                .find(|(entered, _)| Path::new(entered) == path)
                .map(|(_, canonical)| PathBuf::from(canonical))
        }
    }

    fn settings(intake_enabled: bool, intake_folder: &str, destination: &str) -> AppSettings {
        AppSettings {
            destination: destination.into(),
            intake_folder: intake_folder.into(),
            intake_enabled,
            ..AppSettings::default()
        }
    }

    fn error_code(result: Result<(), super::CommandError>) -> String {
        result.expect_err("validation should fail").code
    }

    #[test]
    fn enabling_intake_requires_an_existing_intake_folder() {
        let fs = canonicalizer(&[("/out", "/out")]);
        let mut blank = settings(true, "  ", "/out");
        assert_eq!(
            error_code(validate_intake_settings(&mut blank, "", &fs)),
            "INTAKE_FOLDER_MISSING"
        );
        let mut missing = settings(true, "/gone", "/out");
        assert_eq!(
            error_code(validate_intake_settings(&mut missing, "", &fs)),
            "INTAKE_FOLDER_MISSING"
        );
    }

    #[test]
    fn the_filed_folder_is_offered_beside_the_watched_folder() {
        assert_eq!(
            filed_folder_for(Path::new(r"C:\Users\pat\Contoso\Legal - Documents\Inbox")),
            Some(PathBuf::from(
                r"C:\Users\pat\Contoso\Legal - Documents\Filed"
            ))
        );
    }

    #[test]
    fn onedrive_is_running_only_when_tasklist_names_it() {
        assert!(lists_one_drive(
            "\"OneDrive.exe\",\"10436\",\"Console\",\"1\",\"87,212 K\"\r\n"
        ));
        assert!(!lists_one_drive(
            "INFO: No tasks are running which match the specified criteria.\r\n"
        ));
    }

    #[test]
    fn description_records_require_a_destination_to_live_in() {
        let mut wanted = settings(false, "", "");
        wanted.record_descriptions = true;
        assert_eq!(
            error_code(validate_description_settings(&wanted)),
            "DESCRIPTIONS_NEED_DESTINATION"
        );
        wanted.destination = "/out".into();
        assert!(validate_description_settings(&wanted).is_ok());
        let unwanted = settings(false, "", "");
        assert!(validate_description_settings(&unwanted).is_ok());
    }

    #[test]
    fn enabling_intake_requires_a_real_destination() {
        let fs = canonicalizer(&[("/in", "/in")]);
        let mut blank = settings(true, "/in", "");
        assert_eq!(
            error_code(validate_intake_settings(&mut blank, "", &fs)),
            "INTAKE_NEEDS_DESTINATION"
        );
        let mut missing = settings(true, "/in", "/gone");
        assert_eq!(
            error_code(validate_intake_settings(&mut missing, "", &fs)),
            "INTAKE_NEEDS_DESTINATION"
        );
    }

    #[test]
    fn destination_containment_is_component_wise_not_string_prefix() {
        let fs = canonicalizer(&[
            ("/a/b", "/a/b"),
            ("/a/b/c", "/a/b/c"),
            ("/a/bc", "/a/bc"),
            ("/a/other", "/a/other"),
        ]);
        let mut equal = settings(true, "/a/b", "/a/b");
        assert_eq!(
            error_code(validate_intake_settings(&mut equal, "", &fs)),
            "DESTINATION_INSIDE_INTAKE"
        );
        let mut inside = settings(true, "/a/b", "/a/b/c");
        assert_eq!(
            error_code(validate_intake_settings(&mut inside, "", &fs)),
            "DESTINATION_INSIDE_INTAKE"
        );
        // `/a/bc` shares the string prefix `/a/b` but is a sibling, not a child.
        let mut sibling = settings(true, "/a/b", "/a/bc");
        assert!(validate_intake_settings(&mut sibling, "", &fs).is_ok());
        let mut outside = settings(true, "/a/b", "/a/other");
        assert!(validate_intake_settings(&mut outside, "", &fs).is_ok());
    }

    #[test]
    fn folders_are_written_back_in_canonical_form() {
        let fs = canonicalizer(&[
            ("/in-entered", "/in/canonical"),
            ("/out-entered", "/out/canonical"),
        ]);
        let mut enabled = settings(true, "/in-entered", "/out-entered");
        validate_intake_settings(&mut enabled, "", &fs).expect("valid settings");
        assert_eq!(enabled.intake_folder, "/in/canonical");
        assert_eq!(enabled.destination, "/out/canonical");
        // A non-blank intake folder is canonicalized even while disabled, so
        // the stored form stays the one every later comparison uses.
        let mut disabled = settings(false, "/in-entered", "");
        validate_intake_settings(&mut disabled, "", &fs).expect("valid settings");
        assert_eq!(disabled.intake_folder, "/in/canonical");
    }

    #[test]
    fn an_unreachable_intake_folder_does_not_refuse_a_save_that_leaves_intake_off() {
        let fs = canonicalizer(&[("/out-entered", "/out/canonical")]);
        // An offline network share cannot be canonicalized. Nothing is being
        // read from it while intake is off, so a save that has nothing to do
        // with intake must go through, and the folder keeps the spelling
        // already stored - still canonical for when intake is turned back on.
        let mut disabled = settings(false, "//server/share", "");
        validate_intake_settings(&mut disabled, "/in/canonical", &fs).expect("valid settings");
        assert_eq!(disabled.intake_folder, "/in/canonical");
        // With nothing stored yet, what the person chose is kept rather than
        // silently cleared.
        let mut fresh = settings(false, "//server/share", "");
        validate_intake_settings(&mut fresh, "", &fs).expect("valid settings");
        assert_eq!(fresh.intake_folder, "//server/share");
        // Turning intake on is still refused: that folder would be watched.
        let mut enabled = settings(true, "//server/share", "/out-entered");
        assert_eq!(
            error_code(validate_intake_settings(&mut enabled, "/in/canonical", &fs)),
            "INTAKE_FOLDER_MISSING"
        );
    }

    fn login(previous: bool, next: bool) -> (AppSettings, AppSettings) {
        (
            AppSettings {
                start_at_login: previous,
                ..AppSettings::default()
            },
            AppSettings {
                start_at_login: next,
                ..AppSettings::default()
            },
        )
    }

    fn failure(code: &str) -> super::CommandError {
        super::CommandError {
            code: code.into(),
            message: "no".into(),
        }
    }

    #[test]
    fn autostart_is_untouched_when_the_save_fails() {
        let (previous, next) = login(false, true);
        let toggles = std::cell::Cell::new(0);
        let error = save_settings_and_autostart(
            &previous,
            &next,
            |_| Err(failure("SETTINGS_WRITE_FAILED")),
            |_| {
                toggles.set(toggles.get() + 1);
                Ok(())
            },
        )
        .expect_err("a save that fails is an error");
        assert_eq!(error.code, "SETTINGS_WRITE_FAILED");
        assert_eq!(
            toggles.get(),
            0,
            "the login entry must not be changed for settings that were never stored"
        );
    }

    #[test]
    fn a_refused_login_entry_leaves_the_stored_settings_unchanged() {
        let (previous, next) = login(false, true);
        let stored = std::cell::RefCell::new(Vec::new());
        let error = save_settings_and_autostart(
            &previous,
            &next,
            |settings: &AppSettings| {
                stored.borrow_mut().push(settings.start_at_login);
                Ok(())
            },
            |_| Err(failure("AUTOSTART_FAILED")),
        )
        .expect_err("a refused login entry is an error");
        assert_eq!(error.code, "AUTOSTART_FAILED");
        assert_eq!(
            *stored.borrow(),
            vec![true, false],
            "the saved settings are rolled back to what is actually true"
        );
    }

    #[test]
    fn a_save_that_does_not_change_the_login_entry_never_touches_it() {
        let (previous, next) = login(true, true);
        let toggles = std::cell::Cell::new(0);
        save_settings_and_autostart(
            &previous,
            &next,
            |_| Ok(()),
            |_| {
                toggles.set(toggles.get() + 1);
                Ok(())
            },
        )
        .expect("a save with no login change succeeds");
        assert_eq!(toggles.get(), 0);
    }

    fn receipt(
        direction: OperationDirection,
        stage: OperationStage,
        destination: &str,
    ) -> OperationReceipt {
        OperationReceipt {
            id: 1,
            queue_item_id: 1,
            direction,
            source: PathBuf::from("/intake/original.pdf"),
            destination: PathBuf::from(destination),
            temporary_path: None,
            pre_operation_hash: "hash".into(),
            post_operation_hash: None,
            kind: OperationKind::Rename,
            stage,
            source_exists: false,
            destination_exists: true,
            temporary_exists: false,
        }
    }

    #[test]
    fn queue_statuses_map_onto_claim_item_states() {
        for status in [
            QueueStatus::Queued,
            QueueStatus::Extracting,
            QueueStatus::Analyzing,
            QueueStatus::Ready,
            QueueStatus::Applying,
        ] {
            assert_eq!(item_fate(status, None, None), ItemState::Active);
        }
        assert_eq!(
            item_fate(QueueStatus::NeedsReview, None, None),
            ItemState::NeedsReview
        );
        assert_eq!(
            item_fate(QueueStatus::Failed, None, None),
            ItemState::Failed
        );
        assert_eq!(
            item_fate(QueueStatus::Canceled, None, None),
            ItemState::Unknown
        );
    }

    #[test]
    fn completed_apply_reports_renamed_with_the_applied_filename() {
        let applied = receipt(
            OperationDirection::Apply,
            OperationStage::Complete,
            "/dest/2024-03-01 Contract.pdf",
        );
        assert_eq!(
            item_fate(QueueStatus::Completed, Some(&applied), Some("proposal.pdf")),
            ItemState::Done {
                outcome: DoneOutcome::Renamed,
                result_filename: Some("2024-03-01 Contract.pdf".into()),
            }
        );
        // No usable destination leaf: fall back to the proposal filename.
        let rootward = receipt(OperationDirection::Apply, OperationStage::Complete, "/");
        assert_eq!(
            item_fate(
                QueueStatus::Completed,
                Some(&rootward),
                Some("proposal.pdf")
            ),
            ItemState::Done {
                outcome: DoneOutcome::Renamed,
                result_filename: Some("proposal.pdf".into()),
            }
        );
    }

    #[test]
    fn completed_without_finished_apply_reports_kept_original() {
        let kept = ItemState::Done {
            outcome: DoneOutcome::KeptOriginal,
            result_filename: None,
        };
        assert_eq!(item_fate(QueueStatus::Completed, None, Some("x.pdf")), kept);
        let unfinished = receipt(
            OperationDirection::Apply,
            OperationStage::Published,
            "/dest/renamed.pdf",
        );
        assert_eq!(
            item_fate(QueueStatus::Completed, Some(&unfinished), None),
            kept
        );
        let undone = receipt(
            OperationDirection::Undo,
            OperationStage::Complete,
            "/intake/original.pdf",
        );
        assert_eq!(item_fate(QueueStatus::Completed, Some(&undone), None), kept);
    }

    #[test]
    fn provider_strings_match_the_wire_contract() {
        for (kind, expected) in [
            (CloudProviderKind::OneDrivePersonal, "onedrive_personal"),
            (CloudProviderKind::OneDriveBusiness, "onedrive_business"),
            (CloudProviderKind::SharePoint, "sharepoint"),
            (CloudProviderKind::NetworkShare, "network_share"),
        ] {
            assert_eq!(
                serde_json::to_value(CloudProviderDto::from(kind)).unwrap(),
                serde_json::Value::String(expected.into())
            );
            assert_eq!(
                kind.as_str(),
                expected,
                "the record format spells it the same way"
            );
        }
    }

    #[test]
    fn status_dto_serializes_camel_case_with_zeros_when_disabled() {
        let identity = MachineIdentity {
            id: "0123456789abcdef0123456789abcdef".into(),
            name: "Front desk".into(),
            host_name: "DESKTOP-A1B2C3".into(),
            user: "pat".into(),
        };
        let dto = status_dto(false, &identity, "", None, None, 1_755_850_000);
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["enabled"], false);
        assert_eq!(json["folder"], "");
        let verbatim = status_dto(
            true,
            &identity,
            r"\\?\C:\Users\pat\Scans",
            None,
            None,
            1_755_850_000,
        );
        assert_eq!(verbatim.folder, r"C:\Users\pat\Scans");
        assert_eq!(json["watching"], false);
        assert_eq!(json["machineId"], "0123456789abcdef0123456789abcdef");
        assert_eq!(json["machineName"], "Front desk");
        assert_eq!(json["cloud"], serde_json::Value::Null);
        assert_eq!(json["machines"], serde_json::json!([]));
        assert_eq!(json["heldForOthers"], 0);
        assert_eq!(json["unreadableFolders"], 0);
        assert_eq!(json["claimedByOthers"], 0);
        assert_eq!(json["processedHere"], 0);
        assert_eq!(json["lastScanAt"], serde_json::Value::Null);
        assert_eq!(json["error"], serde_json::Value::Null);
    }

    #[test]
    fn presence_is_active_within_the_window_of_now() {
        let now = 10_000;
        assert!(presence_active(now, now));
        assert!(presence_active(
            now - intern_intake::PRESENCE_ACTIVE_WINDOW_SECONDS,
            now
        ));
        assert!(!presence_active(
            now - intern_intake::PRESENCE_ACTIVE_WINDOW_SECONDS - 1,
            now
        ));
        // Clock skew across machines: a future stamp still counts as active.
        assert!(presence_active(now + 60, now));
    }
}

#[cfg(test)]
mod second_instance_tests {
    use super::second_launch_shows_window;

    #[test]
    fn a_second_launch_opens_the_window_unless_it_asked_for_the_tray() {
        assert!(second_launch_shows_window(&["intern.exe".to_owned()]));
        assert!(second_launch_shows_window(&[
            "intern.exe".to_owned(),
            "C:/drop/scan.pdf".to_owned()
        ]));
        // A sign-in autostart launch asked for the tray, so it must not take
        // the window from whatever is already using it.
        assert!(!second_launch_shows_window(&[
            "intern.exe".to_owned(),
            "--minimized".to_owned()
        ]));
    }
}

#[cfg(test)]
mod queue_event_tests {
    use super::queue_changed_payload;

    #[test]
    fn a_queue_change_says_whether_the_queue_is_paused() {
        assert_eq!(
            queue_changed_payload(true),
            serde_json::json!({ "paused": true })
        );
        assert_eq!(
            queue_changed_payload(false),
            serde_json::json!({ "paused": false })
        );
    }
}

#[cfg(test)]
mod ipc_thread_tests {
    use super::{
        document_open, document_reveal, hosted_model_test, queue_cancel, queue_list, settings_save,
    };

    /// Accepts a command only if calling it returns a future. WebView2
    /// delivers every invoke on one thread, so a command whose body blocks
    /// there freezes the whole window while it runs.
    fn leaves_the_ipc_thread<A, R: std::future::Future, F: FnOnce(A) -> R>(_: F) {}
    fn leaves_the_ipc_thread_2<A, B, R: std::future::Future, F: FnOnce(A, B) -> R>(_: F) {}

    #[test]
    fn commands_that_can_block_for_seconds_do_not_run_on_the_ipc_thread() {
        // Reloads the 1.2 GiB model and waits for it to answer.
        leaves_the_ipc_thread_2(queue_cancel);
        // Sends the calibration document to a hosted service, which can take
        // three minutes to decide it cannot be reached.
        leaves_the_ipc_thread_2(hosted_model_test);
        // Canonicalizes folders, which reaches a network share, and restarts
        // the intake watcher, which joins its scan thread.
        leaves_the_ipc_thread_2(settings_save);
        // One file stat per reviewable item, on whatever the documents live on.
        leaves_the_ipc_thread(queue_list);
        // Check a document that can be on a network share, then wait on the
        // shell to take it.
        leaves_the_ipc_thread_2(document_open);
        leaves_the_ipc_thread_2(document_reveal);
    }
}

#[cfg(test)]
mod document_path_tests {
    use std::path::{Path, PathBuf};

    use intern_core::{
        ErrorCode, OperationDirection, OperationKind, OperationReceipt, OperationStage, QueueStatus,
    };
    use intern_queue::PipelineItem;

    use super::{document_path, document_to_open, locate_document, openable_document, shell_path};

    fn item(status: QueueStatus, receipt: Option<OperationReceipt>) -> PipelineItem {
        PipelineItem {
            id: 4,
            source_path: PathBuf::from("/intake/scan.pdf"),
            source_hash: "hash".into(),
            status,
            processing_failures: 0,
            error_code: None,
            proposal: None,
            receipt,
            duplicate_of: None,
        }
    }

    fn receipt(
        direction: OperationDirection,
        stage: OperationStage,
        source: &str,
        destination: &str,
    ) -> OperationReceipt {
        OperationReceipt {
            id: 9,
            queue_item_id: 4,
            direction,
            source: PathBuf::from(source),
            destination: PathBuf::from(destination),
            temporary_path: None,
            pre_operation_hash: "hash".into(),
            post_operation_hash: Some("hash".into()),
            kind: OperationKind::Rename,
            stage,
            source_exists: false,
            destination_exists: true,
            temporary_exists: false,
        }
    }

    #[test]
    fn document_path_prefers_filed_destination_for_completed() {
        let filed = receipt(
            OperationDirection::Apply,
            OperationStage::Complete,
            "/intake/scan.pdf",
            "/filed/2024-03-01 Lease.pdf",
        );
        assert_eq!(
            document_path(&item(QueueStatus::Completed, Some(filed.clone()))),
            Some(PathBuf::from("/filed/2024-03-01 Lease.pdf"))
        );

        // Kept under its own name: nothing moved, so the document is where
        // it arrived.
        assert_eq!(
            document_path(&item(QueueStatus::Completed, None)),
            Some(PathBuf::from("/intake/scan.pdf"))
        );
        // An undo that rolled back left the filed copy in place.
        let rolled_back_undo = receipt(
            OperationDirection::Undo,
            OperationStage::RolledBack,
            "/filed/2024-03-01 Lease.pdf",
            "/intake/scan.pdf",
        );
        assert_eq!(
            document_path(&item(QueueStatus::Completed, Some(rolled_back_undo))),
            Some(PathBuf::from("/filed/2024-03-01 Lease.pdf"))
        );
        // A finished undo put it back.
        let undone = receipt(
            OperationDirection::Undo,
            OperationStage::Complete,
            "/filed/2024-03-01 Lease.pdf",
            "/intake/scan.pdf",
        );
        assert_eq!(
            document_path(&item(QueueStatus::Completed, Some(undone))),
            Some(PathBuf::from("/intake/scan.pdf"))
        );

        // Anything not yet filed is the source, whatever receipt it carries:
        // a review item whose renamed copy could not replace the original
        // still has the original where it was.
        for status in [
            QueueStatus::Queued,
            QueueStatus::Extracting,
            QueueStatus::Analyzing,
            QueueStatus::Ready,
            QueueStatus::NeedsReview,
            QueueStatus::Failed,
            QueueStatus::Canceled,
        ] {
            assert_eq!(
                document_path(&item(status, Some(filed.clone()))),
                Some(PathBuf::from("/intake/scan.pdf")),
                "{status:?}"
            );
        }
        // Mid-operation the file is between its two names.
        assert_eq!(document_path(&item(QueueStatus::Applying, None)), None);
    }

    // Held because nobody could vouch for who uploaded it: Open would start
    // its program from Intern's window. Show in folder still finds it.
    #[test]
    fn a_document_from_an_unverified_uploader_is_not_opened() {
        let temp = std::env::temp_dir().join(format!("intern-unverified-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let present = temp.join("Invoice.pptm");
        std::fs::write(&present, b"pptm").unwrap();
        let mut held = item(QueueStatus::NeedsReview, None);
        held.source_path = present.clone();
        held.error_code = Some(ErrorCode::UploaderUnverified);
        assert_eq!(
            document_to_open(Some(&held)).unwrap_err().code,
            "UPLOADER_UNVERIFIED"
        );
        assert_eq!(locate_document(Some(&held)).unwrap(), present);

        // Any other review item, once verified, opens as before.
        held.error_code = Some(ErrorCode::Duplicate);
        assert_eq!(document_to_open(Some(&held)).unwrap(), present);
        assert_eq!(document_to_open(None).unwrap_err().code, "ITEM_NOT_FOUND");
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn the_shell_is_given_the_plain_spelling_of_a_verbatim_path() {
        let shell = |path: &str| shell_path(Path::new(path));
        assert_eq!(
            shell(r"\\?\C:\Drop\scan.pdf"),
            PathBuf::from(r"C:\Drop\scan.pdf")
        );
        assert_eq!(
            shell(r"\\?\UNC\server\share\Filed\2024-03-01 Lease.pdf"),
            PathBuf::from(r"\\server\share\Filed\2024-03-01 Lease.pdf")
        );
        // Without the prefix a trailing space or dot is dropped, which would
        // name another file; such a path keeps it.
        assert_eq!(
            shell(r"\\?\C:\Drop\Old \scan.pdf"),
            PathBuf::from(r"\\?\C:\Drop\Old \scan.pdf")
        );
        // A path that was never verbatim - every one outside Windows - is left alone.
        assert_eq!(shell("/intake/scan.pdf"), PathBuf::from("/intake/scan.pdf"));
    }

    #[test]
    fn a_document_is_located_only_when_it_is_on_disk() {
        let error = locate_document(None).unwrap_err();
        assert_eq!(error.code, "ITEM_NOT_FOUND");

        let temp = std::env::temp_dir().join(format!("intern-document-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let present = temp.join("scan.pdf");
        std::fs::write(&present, b"%PDF").unwrap();
        let mut found = item(QueueStatus::NeedsReview, None);
        found.source_path = present.clone();
        found.error_code = Some(ErrorCode::Duplicate);
        assert_eq!(locate_document(Some(&found)).unwrap(), present);

        // Moved or deleted behind Intern's back, a directory where the file
        // was, or mid-operation: none of them is handed to the shell.
        found.source_path = temp.join("gone.pdf");
        assert_eq!(
            locate_document(Some(&found)).unwrap_err().code,
            "PATH_UNAVAILABLE"
        );
        found.source_path = temp.clone();
        assert_eq!(
            locate_document(Some(&found)).unwrap_err().code,
            "PATH_UNAVAILABLE"
        );
        found.source_path = present;
        found.status = QueueStatus::Applying;
        assert_eq!(
            locate_document(Some(&found)).unwrap_err().code,
            "PATH_UNAVAILABLE"
        );
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn only_document_formats_are_handed_to_their_program() {
        assert!(openable_document(PathBuf::from("/filed/2024-03-01 Lease.PDF")).is_ok());
        assert!(openable_document(PathBuf::from("/filed/notes.md")).is_ok());
        for refused in [
            "/intake/setup.exe",
            "/intake/link.lnk",
            "/intake/script",
            "/intake/.pdf",
        ] {
            assert_eq!(
                openable_document(PathBuf::from(refused)).unwrap_err().code,
                "UNSUPPORTED_FORMAT",
                "{refused}"
            );
        }
    }
}

#[cfg(test)]
mod setup_source_tests {
    use std::path::PathBuf;

    use intern_engine::setup::ExistingModelSelection;

    use super::{SetupSource, SetupStatus, setup_progress_status};

    #[test]
    fn verifying_an_installed_model_does_not_take_over_the_window() {
        // A download or an install has files to fetch and a model that cannot
        // answer yet, so the setup screen takes the window.
        assert_eq!(
            setup_progress_status(&SetupSource::Download),
            Some(SetupStatus::Downloading)
        );
        assert_eq!(
            setup_progress_status(&SetupSource::Existing(ExistingModelSelection {
                model_path: PathBuf::from("C:/models/model.gguf"),
            })),
            Some(SetupStatus::Downloading)
        );
        // A model that was already installed when Intern launched is only
        // being verified. The interface goes on saying what is true - it is
        // installed - and the window opens on the queue instead of on a
        // download screen for a download that is not happening.
        assert_eq!(setup_progress_status(&SetupSource::Installed), None);
    }
}

#[cfg(test)]
mod download_retry_tests {
    use std::{
        io::Cursor,
        sync::{
            Arc,
            atomic::{AtomicU32, Ordering},
        },
        time::Duration,
    };

    use intern_engine::download::{Downloader, HttpResponse, HttpTransport, SystemDiskSpace};
    use intern_engine::{EngineError, EngineErrorCode, EngineResult, ModelFile, ModelRole};

    use super::{CancellationToken, download_with_retry_backoff};

    /// Real production backoff would leave these tests asleep for most of a
    /// minute; the retry logic under test does not care about the unit's
    /// size, only that it is used, so a near-zero one keeps the suite fast.
    const TEST_BACKOFF: Duration = Duration::from_millis(1);

    /// Bytes chosen once and hashed offline; the digest below is theirs.
    const BODY: &[u8] = b"the pinned model bytes";
    const BODY_SHA256: &str = "b1f81de2183585b9793c38b27cdd46842ab24247c628b6fbbda1200b9a25ce99";

    #[derive(Clone)]
    struct FlakyTransport {
        failures_left: Arc<AtomicU32>,
    }

    impl HttpTransport for FlakyTransport {
        fn get(
            &self,
            _url: &str,
            _range_start: Option<u64>,
            _cancellation: &CancellationToken,
        ) -> EngineResult<HttpResponse> {
            let due_to_fail = self
                .failures_left
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                    (remaining > 0).then(|| remaining - 1)
                })
                .is_ok();
            if due_to_fail {
                return Err(EngineError::new(
                    EngineErrorCode::DownloadInterrupted,
                    "simulated drop",
                ));
            }
            Ok(HttpResponse {
                status: 200,
                content_range: None,
                body: Box::new(Cursor::new(BODY.to_vec())),
            })
        }
    }

    fn model_file() -> ModelFile {
        ModelFile {
            name: "model.gguf".into(),
            role: ModelRole::Model,
            url: "https://example.invalid/model.gguf".into(),
            size: BODY.len() as u64,
            sha256: BODY_SHA256.into(),
        }
    }

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("intern-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_connection_dropped_twice_is_retried_until_the_file_completes() {
        let destination = scratch_dir("download-retry-ok");
        let transport = FlakyTransport {
            failures_left: Arc::new(AtomicU32::new(2)),
        };
        let downloader = Downloader::new(transport, SystemDiskSpace);
        let cancellation = CancellationToken::new();

        let result = download_with_retry_backoff(
            &downloader,
            &model_file(),
            &destination,
            &cancellation,
            |_| {},
            TEST_BACKOFF,
        );

        let path = result.expect("two transient drops should be retried, not fatal");
        assert_eq!(std::fs::read(path).unwrap(), BODY);
        let _ = std::fs::remove_dir_all(&destination);
    }

    #[test]
    fn a_canceled_download_is_never_retried() {
        let destination = scratch_dir("download-retry-canceled");
        // Enough scheduled failures that a retry loop bug would spin on them
        // instead of stopping for the cancellation this test actually checks.
        let transport = FlakyTransport {
            failures_left: Arc::new(AtomicU32::new(99)),
        };
        let downloader = Downloader::new(transport, SystemDiskSpace);
        let cancellation = CancellationToken::new();
        cancellation.cancel();

        let result = download_with_retry_backoff(
            &downloader,
            &model_file(),
            &destination,
            &cancellation,
            |_| {},
            TEST_BACKOFF,
        );

        assert_eq!(result.unwrap_err().code, "MODEL_DOWNLOAD_CANCELED");
        let _ = std::fs::remove_dir_all(&destination);
    }

    #[test]
    fn failures_past_the_retry_limit_are_reported_instead_of_retried_forever() {
        let destination = scratch_dir("download-retry-exhausted");
        let transport = FlakyTransport {
            failures_left: Arc::new(AtomicU32::new(super::DOWNLOAD_RETRY_LIMIT + 1)),
        };
        let downloader = Downloader::new(transport, SystemDiskSpace);
        let cancellation = CancellationToken::new();

        let result = download_with_retry_backoff(
            &downloader,
            &model_file(),
            &destination,
            &cancellation,
            |_| {},
            TEST_BACKOFF,
        );

        assert_eq!(result.unwrap_err().code, "MODEL_DOWNLOAD_INTERRUPTED");
        let _ = std::fs::remove_dir_all(&destination);
    }
}

#[cfg(test)]
mod background_task_tests {
    use super::background_task_failed;

    #[test]
    fn a_panic_during_intake_is_not_reported_as_an_uploader_failure() {
        let error = tauri::async_runtime::block_on(async {
            tauri::async_runtime::spawn_blocking(|| panic!("the background task died"))
                .await
                .map_err(|_| background_task_failed("file intake"))
                .expect_err("a panicking task must not report success")
        });
        assert_eq!(error.code, "INTERNAL_ERROR");
        assert!(
            error.message.contains("file intake"),
            "the message must name the task that failed: {}",
            error.message
        );
    }
}

#[cfg(test)]
mod scheduler_tests {
    use super::{ExistingModelFilesDto, scheduler_actions};

    #[test]
    fn timer_recovers_but_does_not_drain_until_model_is_ready() {
        assert_eq!(scheduler_actions(true, false), (true, false));
        assert_eq!(scheduler_actions(false, false), (false, false));
        assert_eq!(scheduler_actions(false, true), (false, true));
        assert_eq!(scheduler_actions(true, true), (true, true));
    }

    #[test]
    fn existing_model_dto_is_exactly_one_backend_path() {
        let files: ExistingModelFilesDto = serde_json::from_value(serde_json::json!({
            "modelPath": "C:\\Models\\model.gguf"
        }))
        .unwrap();
        assert!(files.model_path.ends_with("model.gguf"));
        assert!(
            serde_json::from_value::<ExistingModelFilesDto>(serde_json::json!({
                "modelPath": { "name": "model.gguf" }
            }))
            .is_err()
        );
        // A projector path is not merely unused now; offering one must be an
        // error, so a stale caller cannot quietly ask for a file Intern will
        // never load.
        assert!(
            serde_json::from_value::<ExistingModelFilesDto>(serde_json::json!({
                "modelPath": "model.gguf",
                "projectorPath": "projector.gguf"
            }))
            .is_err()
        );
    }
}

#[cfg(test)]
mod history_tests {
    use std::path::PathBuf;

    use intern_core::{HistoryEntry, OperationDirection, OperationKind, OperationStage};

    use std::collections::HashMap;

    use super::{history_csv, history_entry_dto, iso8601_utc};

    fn entry(at: i64, original: &str, new: &str) -> HistoryEntry {
        HistoryEntry {
            receipt_id: 3,
            queue_item_id: 7,
            at,
            direction: OperationDirection::Apply,
            kind: OperationKind::Rename,
            stage: OperationStage::Complete,
            original_path: PathBuf::from(original),
            new_path: PathBuf::from(new),
        }
    }

    #[test]
    fn the_history_export_refuses_a_path_the_save_dialog_would_not_produce() {
        use super::history_export_destination;
        let message = |path: &std::path::Path| {
            history_export_destination(&path.to_string_lossy())
                .expect_err("refused")
                .message
        };
        let folder = std::env::temp_dir();
        assert!(
            history_export_destination(&folder.join("intern-history.csv").to_string_lossy())
                .is_ok()
        );
        assert!(
            history_export_destination(&folder.join("INTERN-HISTORY.CSV").to_string_lossy())
                .is_ok()
        );
        // The dialog offers one extension. Anything else is the window
        // steering Intern into overwriting something it has no business
        // writing to.
        assert!(message(&folder.join("hosts")).contains("CSV"));
        assert!(message(&folder.join("intern-history.csv.exe")).contains("CSV"));
        assert!(
            history_export_destination("intern-history.csv")
                .expect_err("refused")
                .message
                .contains("absolute")
        );
        assert!(
            message(&folder.join("no-such-folder").join("intern-history.csv"))
                .contains("folder does not exist")
        );
    }

    #[test]
    fn timestamps_render_as_iso_8601_utc_from_unix_seconds() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso8601_utc(1_713_173_696), "2024-04-15T09:34:56Z");
        assert_eq!(iso8601_utc(1_755_849_600), "2025-08-22T08:00:00Z");
        // Before the epoch still renders a real calendar date, not garbage.
        assert_eq!(iso8601_utc(-1), "1969-12-31T23:59:59Z");
    }

    #[test]
    fn history_csv_is_rfc_4180_with_quoting_only_where_needed() {
        let plain = entry(0, "C:\\drop\\scan.pdf", "C:\\filed\\2024 Agreement.pdf");
        let awkward = HistoryEntry {
            direction: OperationDirection::Undo,
            kind: OperationKind::VerifiedCopy,
            stage: OperationStage::RolledBack,
            ..entry(
                1_713_173_696,
                "C:\\drop\\comma, quote \" and\nnewline.pdf",
                "C:\\filed\\plain.pdf",
            )
        };

        let descriptions = HashMap::from([(
            7,
            "Lease agreement for a twelve-month term, beginning January 22, 2024.".to_owned(),
        )]);
        let csv = history_csv(&[awkward, plain], &descriptions);

        let mut lines = csv.split("\r\n");
        assert_eq!(
            lines.next(),
            Some("at,direction,kind,stage,originalPath,newPath,description")
        );
        assert_eq!(
            lines.next(),
            Some(
                "2024-04-15T09:34:56Z,undo,verified_copy,rolled_back,\
                 \"C:\\drop\\comma, quote \"\" and\nnewline.pdf\",C:\\filed\\plain.pdf,\
                 \"Lease agreement for a twelve-month term, beginning January 22, 2024.\""
            )
        );
        assert_eq!(
            lines.next(),
            Some(
                "1970-01-01T00:00:00Z,apply,rename,complete,C:\\drop\\scan.pdf,C:\\filed\\2024 Agreement.pdf,\
                 \"Lease agreement for a twelve-month term, beginning January 22, 2024.\""
            )
        );
        assert_eq!(lines.next(), Some(""));
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn history_dto_serializes_the_camel_case_wire_contract() {
        let json = serde_json::to_value(history_entry_dto(
            entry(
                1_713_173_696,
                "C:/drop/scan.pdf",
                "C:/filed/2024 Agreement.pdf",
            ),
            Some("A sentence.".to_owned()),
        ))
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "receiptId": "3",
                "queueItemId": "7",
                "at": 1_713_173_696i64,
                "direction": "apply",
                "kind": "rename",
                "stage": "complete",
                "originalPath": "C:/drop/scan.pdf",
                "newPath": "C:/filed/2024 Agreement.pdf",
                "description": "A sentence.",
            })
        );
        // Verbatim Windows paths are shown the way a person reads them.
        let json = serde_json::to_value(history_entry_dto(
            entry(
                0,
                r"\\?\C:\drop\scan.pdf",
                r"\\?\UNC\server\share\filed\a.pdf",
            ),
            None,
        ))
        .unwrap();
        assert_eq!(json["originalPath"], r"C:\drop\scan.pdf");
        assert_eq!(json["newPath"], r"\\server\share\filed\a.pdf");
        assert_eq!(json["description"], serde_json::Value::Null);
    }
}

#[cfg(test)]
mod file_date_tests {
    use super::file_modified_date;

    #[test]
    fn a_files_date_is_todays_local_date_for_a_file_just_written_and_none_when_missing() {
        let temp = std::env::temp_dir().join(format!("intern-file-date-{}", std::process::id()));
        std::fs::write(&temp, b"x").unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let date = file_modified_date(&temp).unwrap();
        // Written a moment ago; a midnight between the write and the check is
        // the one legitimate reason for "yesterday".
        assert!(date == today || date == yesterday, "{date}");
        let _ = std::fs::remove_file(&temp);
        assert_eq!(file_modified_date(&temp), None);
    }
}

#[cfg(test)]
mod approval_dto_tests {
    use std::path::PathBuf;

    use intern_core::QueueStatus;
    use intern_queue::{PipelineItem, ProposalRecord};

    use super::queue_item_dto;

    /// An item whose proposal is stored as the queue stores it - JSON, which
    /// every later field must be able to read with its serde default.
    fn item(status: QueueStatus, approved: bool) -> PipelineItem {
        let proposal: ProposalRecord = serde_json::from_value(serde_json::json!({
            "analysis": {
                "filename": "2024-02-09 Invoice from Fabrikam Inc.pdf",
                "description": "An invoice from Fabrikam Inc.",
                "status": "ready",
                "reviewReasons": [],
                "proposal": {
                    "document_type": "Invoice",
                    "document_date": "2024-02-09",
                    "date_role": null,
                    "parties": ["Fabrikam Inc"],
                    "party_relation": "from",
                    "description": "An invoice from Fabrikam Inc.",
                    "confidence": 0.9,
                    "evidence": { "date": null, "document_type": null, "parties": [] }
                },
                "telemetry": {
                    "sourceCharacters": 0, "digestCharacters": 0, "compressionRatio": 0.0,
                    "distillMicros": 0, "inferenceMillis": 0
                }
            },
            "status": "ready",
            "filename": "2024-02-09 Invoice from Fabrikam Inc.pdf",
            "description": "An invoice from Fabrikam Inc.",
            "reasons": [],
            "revision": 2,
            "approved": approved
        }))
        .unwrap();
        PipelineItem {
            id: 12,
            source_path: PathBuf::from("/intake/scan.pdf"),
            source_hash: "hash".into(),
            status,
            processing_failures: 0,
            error_code: None,
            proposal: Some(proposal),
            receipt: None,
            duplicate_of: None,
        }
    }

    // The queue was busy with another document, so the approved name waits
    // in the proposal and the item stays ready. Without the flag the window
    // counted it as still to decide and review came back round to it.
    #[test]
    fn an_approval_waiting_for_the_queue_is_reported_as_decided() {
        let dto = |status, approved| queue_item_dto(item(status, approved)).unwrap();
        assert!(dto(QueueStatus::Ready, true).approved);
        // Ready alone is a proposal still waiting for a person.
        assert!(!dto(QueueStatus::Ready, false).approved);
        // Once filed, or sent back, there is nothing left waiting on it.
        for status in [QueueStatus::Completed, QueueStatus::NeedsReview] {
            assert!(!dto(status, true).approved, "{status:?}");
        }
        let json = serde_json::to_value(dto(QueueStatus::Ready, true)).unwrap();
        assert_eq!(json["approved"], serde_json::json!(true));
    }
}

#[cfg(test)]
mod duplicate_reason_tests {
    use std::path::PathBuf;

    use intern_core::{ErrorCode, QueueStatus};
    use intern_queue::PipelineItem;

    use super::queue_item_dto;

    fn duplicate_item(duplicate_of: Option<&str>) -> PipelineItem {
        PipelineItem {
            id: 7,
            source_path: PathBuf::from("C:/drop/copy.pdf"),
            source_hash: "hash".into(),
            status: QueueStatus::NeedsReview,
            processing_failures: 0,
            error_code: Some(ErrorCode::Duplicate),
            proposal: None,
            receipt: None,
            duplicate_of: duplicate_of.map(str::to_owned),
        }
    }

    #[test]
    fn duplicate_review_items_surface_the_filed_name_as_their_reason() {
        let dto = queue_item_dto(duplicate_item(Some("2024 - Filed Agreement.pdf"))).unwrap();
        assert_eq!(
            dto.reason.as_deref(),
            Some("Duplicate of 2024 - Filed Agreement.pdf")
        );
        assert_eq!(dto.error_code.as_deref(), Some("DUPLICATE"));

        // A cleared history leaves the flag without a referent: no fabricated
        // reason, and the item stays actionable through its error code.
        let stale = queue_item_dto(duplicate_item(None)).unwrap();
        assert_eq!(stale.reason, None);
        assert_eq!(stale.error_code.as_deref(), Some("DUPLICATE"));
    }
}

#[cfg(test)]
mod settings_report_tests {
    use super::*;

    #[test]
    fn a_settings_file_that_could_not_be_read_is_announced_rather_than_assumed() {
        assert_eq!(unreadable_settings(&Ok(LoadedSettings::default())), None);

        let partial = unreadable_settings(&Ok(LoadedSettings {
            settings: AppSettings::default(),
            unreadable: vec!["destinationLayout".into()],
        }))
        .expect("a defaulted field is reported");
        assert!(
            partial.starts_with("SETTINGS_PARTIAL: destinationLayout"),
            "{partial}"
        );
        assert!(partial.contains("Everything else was kept"), "{partial}");

        let refused = unreadable_settings(&Err(PipelineError::new(
            "SETTINGS_INVALID",
            "settings are not valid",
        )))
        .expect("a file nothing could read is reported");
        assert!(refused.starts_with("SETTINGS_INVALID:"), "{refused}");
        // Defaults are what Intern is running on, and the person is told the
        // file is still theirs to repair.
        assert!(refused.contains("automatic renaming off"), "{refused}");
        assert!(refused.contains("has not been written over"), "{refused}");
    }
}

/// A `SettingsRuntime` over a real settings file whose live effects are
/// recorded instead of reaching Tauri. Shared by the settings and SharePoint
/// activation tests so both drive the production application code.
#[cfg(test)]
pub(crate) mod test_runtime {
    use std::{
        path::{Path, PathBuf},
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
    };

    use intern_queue::{AppSettings, SettingsStore, paths::canonical_folder};

    use super::{CommandError, ManagedSharePoint, SettingsRuntime, managed_sharepoint_for};
    use crate::microsoft_intake::MicrosoftIntake;

    /// What is running, as opposed to what is persisted.
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub(crate) struct Live {
        pub tray: bool,
        pub watcher: Option<String>,
        pub autostart: bool,
        pub intake_events: usize,
    }

    type Predicate = Box<dyn Fn(&AppSettings) -> bool + Send + Sync>;
    type Hook = Box<dyn Fn(&AppSettings) + Send + Sync>;

    pub(crate) struct RecordingRuntime {
        pub store: SettingsStore,
        pub live: Mutex<Live>,
        pub microsoft: Option<Arc<MicrosoftIntake>>,
        /// Watcher restarts for these settings fail the way an unreadable
        /// machine identity does: before the running watcher is stopped.
        pub fail_intake_restart: Mutex<Option<Predicate>>,
        pub fail_persist: Mutex<Option<Predicate>>,
        pub fail_hosted_model: bool,
        pub fail_intake_events: AtomicBool,
        pub after_persist: Mutex<Option<Hook>>,
        gate: Mutex<()>,
        activation: AtomicBool,
    }

    impl RecordingRuntime {
        pub(crate) fn new(settings_path: PathBuf) -> Self {
            Self {
                store: SettingsStore::new(settings_path),
                live: Mutex::new(Live::default()),
                microsoft: None,
                fail_intake_restart: Mutex::new(None),
                fail_persist: Mutex::new(None),
                fail_hosted_model: false,
                fail_intake_events: AtomicBool::new(false),
                after_persist: Mutex::new(None),
                gate: Mutex::new(()),
                activation: AtomicBool::new(false),
            }
        }

        pub(crate) fn live(&self) -> Live {
            self.live.lock().unwrap().clone()
        }

        fn injected(code: &str, message: &str) -> CommandError {
            CommandError {
                code: code.into(),
                message: message.into(),
            }
        }
    }

    impl SettingsRuntime for RecordingRuntime {
        fn load_settings(&self) -> Result<AppSettings, CommandError> {
            self.store.load().map_err(CommandError::from)
        }

        fn persist_settings(&self, settings: &AppSettings) -> Result<(), CommandError> {
            if self
                .fail_persist
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|fail| fail(settings))
            {
                return Err(Self::injected(
                    "SETTINGS_WRITE_FAILED",
                    "injected settings write failure",
                ));
            }
            self.store.save(settings).map_err(CommandError::from)?;
            if let Some(hook) = self.after_persist.lock().unwrap().as_ref() {
                hook(settings);
            }
            Ok(())
        }

        fn canonical_folder(&self, path: &Path) -> Result<PathBuf, CommandError> {
            canonical_folder(path).map_err(CommandError::from)
        }

        fn check_hosted_model(&self, _settings: &AppSettings) -> Result<(), CommandError> {
            if self.fail_hosted_model {
                return Err(Self::injected(
                    "HOSTED_MODEL_KEY_MISSING",
                    "no API key is stored for the hosted model",
                ));
            }
            Ok(())
        }

        fn protect_microsoft(&self, settings: &AppSettings) -> Result<(), CommandError> {
            match &self.microsoft {
                Some(microsoft) => {
                    microsoft
                        .protect_settings(settings)
                        .map_err(|message| CommandError {
                            code: "UPLOADER_UNVERIFIED".into(),
                            message,
                        })
                }
                None => Ok(()),
            }
        }

        fn set_autostart(&self, enabled: bool) -> Result<(), CommandError> {
            self.live.lock().unwrap().autostart = enabled;
            Ok(())
        }

        fn refresh_hosted_active(&self, _settings: &AppSettings) {}

        fn schedule(&self) -> Result<(), CommandError> {
            Ok(())
        }

        fn sync_tray(&self, run_in_background: bool) {
            self.live.lock().unwrap().tray = run_in_background;
        }

        fn restart_intake(&self, settings: &AppSettings) -> Result<(), CommandError> {
            if self
                .fail_intake_restart
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|fail| fail(settings))
            {
                return Err(Self::injected(
                    "APP_DATA_UNAVAILABLE",
                    "injected watcher restart failure",
                ));
            }
            self.live.lock().unwrap().watcher = settings
                .intake_enabled
                .then(|| settings.intake_folder.clone());
            Ok(())
        }

        fn emit_intake_changed(&self) -> Result<(), CommandError> {
            if self.fail_intake_events.load(Ordering::SeqCst) {
                return Err(Self::injected(
                    "STATE_CONFLICT",
                    "injected intake event failure",
                ));
            }
            self.live.lock().unwrap().intake_events += 1;
            Ok(())
        }

        fn managed_sharepoint(
            &self,
            stored: Option<&AppSettings>,
        ) -> Result<Option<ManagedSharePoint>, CommandError> {
            match &self.microsoft {
                Some(microsoft) => managed_sharepoint_for(microsoft, stored),
                None => Ok(None),
            }
        }

        fn settings_gate(&self) -> &Mutex<()> {
            &self.gate
        }

        fn sharepoint_activation(&self) -> &AtomicBool {
            &self.activation
        }
    }
}
