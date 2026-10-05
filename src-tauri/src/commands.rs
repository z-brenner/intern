use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
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

/// What brings a local model runtime up: in production llama-server and a
/// client for it, under test a fake that counts its launches.
trait RuntimeLauncher: Send + Sync {
    fn launch(&self, manifest: &ModelManifest) -> Result<LaunchedRuntime, CommandError>;
}

/// A runtime that has started and answered its health check.
struct LaunchedRuntime {
    process: Box<dyn RuntimeProcess>,
    engine: Engine,
}

/// The process behind a running runtime.
trait RuntimeProcess: Send {
    fn stop(self: Box<Self>) -> Result<(), ModelFailure>;
}

impl RuntimeProcess for LlamaServer {
    fn stop(self: Box<Self>) -> Result<(), ModelFailure> {
        LlamaServer::stop(&self).map_err(|_| ModelFailure::fatal("MODEL_CANCEL_FAILED"))
    }
}

/// llama-server from beside Intern's own executable, over the model in the
/// app's data folder.
struct LlamaLauncher {
    executable: PathBuf,
    model_directory: PathBuf,
}

impl RuntimeLauncher for LlamaLauncher {
    /// Text only, and not as a mode: no vision projector is pinned, downloaded,
    /// or loaded. Essentially every business document carries usable text, and a
    /// projector for this model is 668,227,264 bytes - 637 MiB - which every
    /// user would download and hold resident for a path almost nothing takes.
    fn launch(&self, manifest: &ModelManifest) -> Result<LaunchedRuntime, CommandError> {
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
        Ok(LaunchedRuntime {
            process: Box::new(server),
            engine: Engine::new(client),
        })
    }
}

/// Reads a model file end to end and checks it against the digest its
/// manifest entry pins. `validate_selected_file` in production; a counter
/// under test, which is how the tests prove which paths never hash.
type FileValidator =
    dyn Fn(&Path, &ModelFile) -> Result<(), intern_engine::EngineError> + Send + Sync;

/// The model file as it was when its digest was last checked.
#[derive(Clone, Debug, Eq, PartialEq)]
struct VerifiedFile {
    path: PathBuf,
    len: u64,
    modified: std::time::SystemTime,
}

/// Where a model that passed its digest and its self-test says so, beside the
/// model itself.
const VERIFICATION_STAMP: &str = ".verified.json";
const VERIFICATION_STAMP_SCHEMA: u32 = 1;

/// What a launch needs to know to trust the installed model without reading
/// it again: the files as they were when their digest and the self-test both
/// passed, the server binary that ran the self-test, and the build that did
/// the checking. Any of them changing - a replaced file, an updated runtime,
/// a new Intern - sends the next launch through the full check again.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct VerificationStamp {
    schema_version: u32,
    app_version: String,
    files: Vec<StampedFile>,
    server_len: u64,
    server_modified_nanos: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StampedFile {
    name: String,
    len: u64,
    modified_nanos: u64,
    /// The digest the manifest pins, which the file was checked against. A
    /// new pin is a different model, whatever the file's size and date.
    sha256: String,
}

fn modified_nanos(metadata: &std::fs::Metadata) -> Option<u64> {
    let since = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    u64::try_from(since.as_nanos()).ok()
}

/// A manifest file name that stays inside the model folder. The embedded
/// manifest is validated when it is parsed; this repeats the check because
/// a path is built from the name before anything else looks at it.
fn safe_model_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && Path::new(name).file_name() == Some(std::ffi::OsStr::new(name))
        && Path::new(name).components().count() == 1
}

/// What a start did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Launch {
    Started,
    /// Nothing, because the local model is held: a hosted one is chosen, or
    /// Intern is shutting down.
    Held,
}

/// The engine in the slot, and the generation it was started in.
struct RunningEngine {
    engine: Engine,
    generation: u64,
}

/// The local model runtime: one llama-server, the engine that talks to it,
/// and everything that starts, stops, and restarts it.
///
/// Starts and stops are serialized by `lifecycle`. Cancel, recover, and the
/// setup thread's verified start used to run unserialized, so a cancel and a
/// recover that overlapped each launched a server - twice the memory while
/// both loaded - and the loser reported a failure for a cancel that had
/// worked. `generation` counts the deliberate stops (cancel, hold, shutdown):
/// a request that fails after one is reported as canceled - or, when the
/// local model was set aside, handed back - rather than as a server fault,
/// so nothing restarts the server a second time for it.
struct RuntimeModel {
    executable: PathBuf,
    model_directory: PathBuf,
    manifest: ModelManifest,
    app_version: String,
    launcher: Arc<dyn RuntimeLauncher>,
    validate: Box<FileValidator>,
    engine: RwLock<Option<RunningEngine>>,
    server: Mutex<Option<Box<dyn RuntimeProcess>>>,
    lifecycle: Mutex<()>,
    generation: AtomicU64,
    /// The generation the last retryable failure happened in, which tells
    /// `recover` whether anyone has restarted the server since.
    failed_generation: AtomicU64,
    /// Set while a hosted model is chosen and from shutdown on: nothing
    /// starts the local server, and a start already under way discards what
    /// it launched.
    held: AtomicBool,
    /// The model file as it was when this session last checked its digest.
    /// A start trusts the file while it still looks exactly like that.
    verified: Mutex<Option<VerifiedFile>>,
}

impl RuntimeModel {
    fn new(executable: PathBuf, model_directory: PathBuf, manifest: ModelManifest) -> Self {
        let launcher = Arc::new(LlamaLauncher {
            executable: executable.clone(),
            model_directory: model_directory.clone(),
        });
        Self::with_parts(
            executable,
            model_directory,
            manifest,
            launcher,
            Box::new(validate_selected_file),
            env!("CARGO_PKG_VERSION").to_owned(),
        )
    }

    fn with_parts(
        executable: PathBuf,
        model_directory: PathBuf,
        manifest: ModelManifest,
        launcher: Arc<dyn RuntimeLauncher>,
        validate: Box<FileValidator>,
        app_version: String,
    ) -> Self {
        Self {
            executable,
            model_directory,
            manifest,
            app_version,
            launcher,
            validate,
            engine: RwLock::new(None),
            server: Mutex::new(None),
            lifecycle: Mutex::new(()),
            generation: AtomicU64::new(0),
            failed_generation: AtomicU64::new(0),
            held: AtomicBool::new(false),
            verified: Mutex::new(None),
        }
    }

    fn manifest(&self) -> &ModelManifest {
        &self.manifest
    }

    /// Whether the manifest's files are in place at the size it pins.
    ///
    /// Metadata only, never a byte of the file. Launch asks this inside
    /// Tauri's setup hook, before the window paints, and used to answer it by
    /// hashing all 1.19 GiB - twice, so an office laptop without SHA
    /// instructions showed a white rectangle for five seconds or more at
    /// every sign-in. The digest is checked on the setup thread instead.
    fn installed_quick(&self, manifest: &ModelManifest) -> bool {
        manifest.files.iter().all(|file| {
            safe_model_name(&file.name)
                && std::fs::metadata(self.model_directory.join(&file.name))
                    .is_ok_and(|metadata| metadata.is_file() && metadata.len() == file.size)
        })
    }

    /// Whether a server is up and its engine in the slot.
    ///
    /// A stop takes the server out of its slot before it stops it, so this
    /// is false from the moment a stop begins: a request the stop interrupts
    /// always fails after it, never before.
    fn running(&self) -> bool {
        let server = self
            .server
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some();
        server
            && self
                .engine
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
    }

    fn is_held(&self) -> bool {
        self.held.load(Ordering::SeqCst)
    }

    fn model_file_state(&self) -> Option<VerifiedFile> {
        let model = self.manifest.model()?;
        let path = self.model_directory.join(&model.name);
        let metadata = std::fs::metadata(&path).ok()?;
        Some(VerifiedFile {
            path,
            len: metadata.len(),
            modified: metadata.modified().ok()?,
        })
    }

    fn verified_slot(&self) -> std::sync::MutexGuard<'_, Option<VerifiedFile>> {
        self.verified
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Records that the model file, as it is now, has just passed its digest
    /// somewhere else - the download and the install of a chosen file both
    /// check it - so the start that follows need not read it again.
    fn remember_verified(&self) {
        *self.verified_slot() = self.model_file_state();
    }

    /// Checks the model's digest, unless this session already has for the
    /// file exactly as it is now. A mismatch is MODEL_FILE_INVALID, which is
    /// what setup's existing repair - delete it and download it again - is for.
    fn verify_files(&self) -> Result<(), CommandError> {
        let current = self.model_file_state();
        if current.is_some() && *self.verified_slot() == current {
            return Ok(());
        }
        *self.verified_slot() = None;
        for file in &self.manifest.files {
            if !safe_model_name(&file.name) {
                return Err(CommandError {
                    code: "MODEL_FILE_INVALID".into(),
                    message: "model manifest names an unsafe file".into(),
                });
            }
            (self.validate)(&self.model_directory.join(&file.name), file)?;
        }
        // The state from before the digest: a file that changed while it
        // was being read no longer matches it, so nothing trusts the change.
        *self.verified_slot() = current;
        Ok(())
    }

    fn stamp_path(&self) -> PathBuf {
        self.model_directory.join(VERIFICATION_STAMP)
    }

    /// The stamp the files and server on disk would earn now, or `None` when
    /// something it records cannot be read.
    fn current_stamp(&self) -> Option<VerificationStamp> {
        let files = self
            .manifest
            .files
            .iter()
            .map(|file| {
                let metadata = std::fs::metadata(self.model_directory.join(&file.name)).ok()?;
                Some(StampedFile {
                    name: file.name.clone(),
                    len: metadata.len(),
                    modified_nanos: modified_nanos(&metadata)?,
                    sha256: file.sha256.clone(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let server = std::fs::metadata(&self.executable).ok()?;
        Some(VerificationStamp {
            schema_version: VERIFICATION_STAMP_SCHEMA,
            app_version: self.app_version.clone(),
            files,
            server_len: server.len(),
            server_modified_nanos: modified_nanos(&server)?,
        })
    }

    fn stored_stamp(&self) -> Option<VerificationStamp> {
        let bytes = std::fs::read(self.stamp_path()).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn write_stamp(&self, stamp: &VerificationStamp) {
        if let Ok(bytes) = serde_json::to_vec_pretty(stamp) {
            let _ = std::fs::write(self.stamp_path(), bytes);
        }
    }

    fn delete_stamp(&self) {
        let _ = std::fs::remove_file(self.stamp_path());
    }

    /// Launches the runtime into the empty slots.
    ///
    /// Callers hold `lifecycle`, and the guard is asked for to say so. The
    /// slots are checked before anything is launched, so no start can load a
    /// second server only to find the first one there. Never reads the model:
    /// the file must still look exactly as it did when this session checked
    /// its digest, or nothing starts.
    fn start(&self, _lifecycle: &std::sync::MutexGuard<'_, ()>) -> Result<Launch, CommandError> {
        if self.is_held() {
            return Ok(Launch::Held);
        }
        let current = self.model_file_state();
        if !self.installed_quick(&self.manifest)
            || current.is_none()
            || *self.verified_slot() != current
        {
            return Err(CommandError {
                code: "MODEL_NOT_READY".into(),
                message: "model files are not installed, or changed after they were verified"
                    .into(),
            });
        }
        let occupied = self
            .server
            .lock()
            .map_err(|_| process_state_unavailable())?
            .is_some()
            || self
                .engine
                .read()
                .map_err(|_| model_state_unavailable())?
                .is_some();
        if occupied {
            return Err(CommandError {
                code: "MODEL_ALREADY_RUNNING".into(),
                message: "local model process is already running".into(),
            });
        }
        let launched = self.launcher.launch(&self.manifest)?;
        let mut server = self
            .server
            .lock()
            .map_err(|_| process_state_unavailable())?;
        let mut engine = self.engine.write().map_err(|_| model_state_unavailable())?;
        // Checked under the slot lock that a hold or shutdown takes after
        // setting the flag: either this sees the hold, or the hold's stop
        // sees what this installs.
        if self.is_held() {
            drop(engine);
            drop(server);
            let _ = launched.process.stop();
            return Ok(Launch::Held);
        }
        *server = Some(launched.process);
        *engine = Some(RunningEngine {
            engine: launched.engine,
            generation: self.generation.load(Ordering::SeqCst),
        });
        Ok(Launch::Started)
    }

    /// Starts the installed model and makes sure it works, on the setup
    /// thread.
    ///
    /// A model that passed its digest and the semantic self-test under this
    /// build, this server binary, and its current size and date is started
    /// without either: both cost seconds of CPU at every sign-in, and
    /// nothing they check has changed. Anything else is hashed - unless this
    /// session already has - started, and self-tested, and only a model that
    /// passes is stamped.
    fn start_verified(&self, cancellation: &CancellationToken) -> Result<(), CommandError> {
        let lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| process_state_unavailable())?;
        self.stop_runtime().map_err(|error| CommandError {
            code: error.code,
            message: "existing local model process could not be stopped".into(),
        })?;
        if cancellation.is_canceled() {
            return Err(setup_canceled_error());
        }
        if self.is_held() {
            return Ok(());
        }
        let stamp = self.current_stamp();
        let stamped =
            stamp.is_some() && self.installed_quick(&self.manifest) && self.stored_stamp() == stamp;
        if stamped {
            self.remember_verified();
        } else {
            self.verify_files()?;
        }
        if cancellation.is_canceled() {
            return Err(setup_canceled_error());
        }
        // A model the stamp vouched for that will not start is checked in
        // full next time, in case the file is what is wrong with it.
        if self
            .start(&lifecycle)
            .inspect_err(|_| self.delete_stamp())?
            == Launch::Held
        {
            return Ok(());
        }
        if stamped {
            if cancellation.is_canceled() {
                let _ = self.stop_runtime();
                return Err(setup_canceled_error());
            }
            return Ok(());
        }
        let result = self.semantic_self_test(cancellation);
        match (&result, stamp) {
            (Ok(()), Some(stamp)) if self.current_stamp().as_ref() == Some(&stamp) => {
                self.write_stamp(&stamp);
            }
            (Ok(()), _) => {}
            // While this holds `lifecycle`, only a setup cancel - which its
            // token tells apart - a hold, or shutdown stops the server. A
            // server stopped under the self-test on purpose says nothing
            // about the model, and reporting it as a failed self-test told
            // someone who had just chosen a hosted model that their local one
            // was broken.
            (Err(_), _) if !cancellation.is_canceled() && !self.running() => return Ok(()),
            (Err(_), _) => {
                self.delete_stamp();
                let _ = self.stop_runtime();
            }
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
                engine.engine.analyze(&probe.document, "pdf", &[])
            };
            if cancellation.is_canceled() {
                return Err(setup_canceled_error());
            }
            // The cause travels in the message: "the self-test failed" alone
            // cannot tell a server that died from one that answered wrongly.
            let analysis = analysis.map_err(|error| CommandError {
                code: "MODEL_SELF_TEST_FAILED".into(),
                message: format!(
                    "local model semantic self-test request failed: {}",
                    error.code().as_str()
                ),
            })?;
            validate_semantic_probe(&probe, &analysis)?;
        }
        Ok(())
    }

    /// Stops the server, if one is running, and empties the engine slot.
    /// Says whether there was one to stop.
    fn stop_runtime(&self) -> Result<bool, ModelFailure> {
        let (was_running, stop_result) = {
            let mut server = self
                .server
                .lock()
                .map_err(|_| ModelFailure::fatal("MODEL_CANCEL_FAILED"))?;
            match server.take() {
                Some(process) => (true, process.stop()),
                None => (false, Ok(())),
            }
        };
        *self
            .engine
            .write()
            .map_err(|_| ModelFailure::fatal("MODEL_CANCEL_FAILED"))? = None;
        stop_result.map(|()| was_running)
    }

    /// Stops the local model and keeps it stopped: while a hosted model is
    /// chosen, and from shutdown on.
    ///
    /// Does not wait for `lifecycle`. A start holds it for as long as
    /// llama-server takes to load - minutes on a slow laptop - and the
    /// verified start for its self-test as well; Settings saves and Tauri's
    /// exit both call this, and neither may hang behind that. The hold does
    /// not need the lock either: a start checks it under the slot lock this
    /// stop takes, so either the start sees the hold and discards what it
    /// launched, or this stop finds what the start installed.
    fn hold(&self) -> Result<(), ModelFailure> {
        self.held.store(true, Ordering::SeqCst);
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.stop_runtime().map(|_| ())
    }

    /// Lets the local model start again, once the local model is chosen.
    fn release(&self) {
        self.held.store(false, Ordering::SeqCst);
    }

    /// The failure an engine error amounts to, for a request that read the
    /// generation as `asked` and ran on an engine started in `started`.
    fn failure_for(&self, code: &str, asked: u64, started: u64) -> ModelFailure {
        let now = self.generation.load(Ordering::SeqCst);
        if now != asked || now != started {
            return self.overtaken();
        }
        // These say something about this document, not about the server: it
        // does not fit, its reply was cut off or unreadable (the client has
        // already asked twice), or the engine failed on it. A restart and a
        // second request would only do the same again.
        if matches!(
            code,
            "MODEL_INPUT_TOO_LARGE"
                | "MODEL_RESPONSE_INVALID"
                | "MODEL_REPLY_TRUNCATED"
                | "ANALYSIS_FAILED"
        ) {
            return ModelFailure::fatal(code);
        }
        self.failed_generation.store(now, Ordering::SeqCst);
        ModelFailure::retryable(code)
    }

    /// What a request amounts to when a deliberate stop came after it began.
    fn overtaken(&self) -> ModelFailure {
        // A hosted model was chosen, or Intern is exiting: nothing is wrong
        // with the document, which goes back to be read again - by the
        // hosted model, now.
        if self.is_held() {
            return ModelFailure::retryable("MODEL_NOT_READY");
        }
        // A cancel came between: the server was stopped under this request
        // on purpose, and restarting it again for the request would re-read
        // a document someone has just canceled.
        ModelFailure::fatal("MODEL_CANCELED")
    }
}

fn process_state_unavailable() -> CommandError {
    CommandError {
        code: "MODEL_NOT_READY".into(),
        message: "model process state is unavailable".into(),
    }
}

fn model_state_unavailable() -> CommandError {
    CommandError {
        code: "MODEL_NOT_READY".into(),
        message: "model state is unavailable".into(),
    }
}

impl AnalyzerBoundary for RuntimeModel {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        let asked = self.generation.load(Ordering::SeqCst);
        let mut waited = false;
        loop {
            {
                let slot = self
                    .engine
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                // Only an engine started since the last deliberate stop. One
                // from before it is the one that stop is about to take away,
                // and a request sent to it would fail under the stop as if
                // something were wrong with the document.
                if let Some(running) = slot.as_ref().filter(|running| running.generation == asked) {
                    return running
                        .engine
                        .analyze(source, extension, existing_names)
                        .map_err(|error| {
                            self.failure_for(error.code().as_str(), asked, running.generation)
                        });
                }
            }
            // No engine to ask. Nothing is wrong with the document either
            // way, so none of this is a failure to count against it.
            if self.is_held() || self.generation.load(Ordering::SeqCst) != asked {
                return Err(self.overtaken());
            }
            if waited {
                // Nothing under way: never started, a start that failed, or
                // a verified start whose thread has yet to begin. The
                // queue's to put back until setup has a server running.
                return Err(ModelFailure::retryable("MODEL_NOT_READY"));
            }
            // A restart under way - a cancel's, or the verified start that
            // choosing the local model again begins - holds `lifecycle` from
            // before it empties the slot until the new engine is in it. A
            // cancel no longer holds the queue while it restarts the server,
            // so the next document arrives in the middle of that, and handing
            // it back failed it as a file error twice over in the seconds the
            // server took to load. Waiting costs it those seconds instead.
            // Taken with the engine guard released: a stop needs the slot.
            drop(
                self.lifecycle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
            waited = true;
        }
    }

    /// Restarts a server that failed a request. Only once per failure: a
    /// cancel or hold since then has already restarted or stopped it.
    fn recover(&self, failure: &ModelFailure) -> Result<(), ModelFailure> {
        if failure.code != "MODEL_REQUEST_FAILED" {
            return Ok(());
        }
        let lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ModelFailure::fatal("MODEL_RECOVERY_FAILED"))?;
        if self.generation.load(Ordering::SeqCst) != self.failed_generation.load(Ordering::SeqCst) {
            return Ok(());
        }
        self.stop_runtime()
            .map_err(|_| ModelFailure::fatal("MODEL_RECOVERY_FAILED"))?;
        self.start(&lifecycle)
            .map(|_| ())
            .map_err(|_| ModelFailure::fatal("MODEL_RECOVERY_FAILED"))
    }

    /// Interrupts the request in flight by restarting the server under it.
    /// A server that was not running is left that way.
    fn cancel(&self) -> Result<(), ModelFailure> {
        // Before anything stops: the request the stop interrupts must see a
        // cancel, not a server that failed.
        self.generation.fetch_add(1, Ordering::SeqCst);
        let lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| ModelFailure::fatal("MODEL_CANCEL_FAILED"))?;
        if !self.stop_runtime()? {
            return Ok(());
        }
        self.start(&lifecycle)
            .map(|_| ())
            .map_err(|error| ModelFailure::fatal(error.code))
    }

    /// Stops the server for good. Does not wait for a start under way: this
    /// runs on the thread Tauri exits on, and a start can take three minutes
    /// to give up. The hold makes that start stop what it launched instead.
    fn shutdown(&self) -> Result<(), ModelFailure> {
        self.hold()
    }
}

/// How often a download may tell the window how far it has got.
///
/// Every network chunk used to reach the webview as its own event - tens of
/// thousands over 1.19 GiB, each a React state update - on exactly the slow
/// laptops where the download already takes longest.
struct ProgressThrottle {
    last: Option<std::time::Instant>,
    last_fraction: f64,
}

const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);
const PROGRESS_STEP: f64 = 0.005;

impl ProgressThrottle {
    fn new() -> Self {
        Self {
            last: None,
            last_fraction: 0.0,
        }
    }

    /// Whether this update is worth an event: the first one, a quarter of a
    /// second since the last, half a percent of movement, a change of
    /// status, or the final byte.
    fn should_emit(
        &mut self,
        now: std::time::Instant,
        downloaded: u64,
        total: u64,
        status_changed: bool,
    ) -> bool {
        let fraction = if total == 0 {
            1.0
        } else {
            downloaded as f64 / total as f64
        };
        let emit = status_changed
            || downloaded == total
            || (fraction - self.last_fraction).abs() >= PROGRESS_STEP
            || self
                .last
                .is_none_or(|last| now.saturating_duration_since(last) >= PROGRESS_INTERVAL);
        if emit {
            self.last = Some(now);
            self.last_fraction = fraction;
        }
        emit
    }
}

/// Where the setup state goes when it changes: the window, in production.
type SetupPublisher = Box<dyn Fn(&SetupStateDto) + Send + Sync>;

fn setup_publisher(app: AppHandle) -> SetupPublisher {
    Box::new(move |state| {
        let _ = app.emit("setup://progress", state.clone());
    })
}

struct SetupManager {
    publish: SetupPublisher,
    runtime: Arc<RuntimeModel>,
    state: Mutex<SetupStateDto>,
    throttle: Mutex<ProgressThrottle>,
    operation: SetupOperationGate,
    scheduler: Mutex<Option<std::sync::mpsc::Sender<SchedulerMessage>>>,
    /// Whether the queue may run: the local model is ready, or a hosted one
    /// is chosen and configured. The scheduler reads this.
    model_ready: Arc<AtomicBool>,
    local_ready: AtomicBool,
    hosted_active: AtomicBool,
}

impl SetupManager {
    /// `installed` is `RuntimeModel::installed_quick`, asked once by launch:
    /// whether the files are there, not whether they are right.
    fn new(publish: SetupPublisher, runtime: Arc<RuntimeModel>, installed: bool) -> Self {
        let total_bytes = runtime.manifest().total_bytes();
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
            publish,
            runtime,
            state: Mutex::new(state),
            throttle: Mutex::new(ProgressThrottle::new()),
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
            (self.publish)(&current);
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

    /// Whether the local model needs no setup: it is running, or it is
    /// installed and held while a hosted model is chosen. A held model is not
    /// ready for the queue, but it is installed, and setting it up again
    /// would fetch or copy 1.19 GiB that is already on disk.
    fn setup_complete(&self) -> bool {
        self.local_ready.load(Ordering::SeqCst)
            || (self.runtime.is_held()
                && self
                    .get()
                    .is_ok_and(|current| current.state == SetupStatus::Ready))
    }

    fn start(self: &Arc<Self>) -> Result<(), CommandError> {
        if self.setup_complete() {
            return Ok(());
        }
        self.start_operation(SetupSource::Download)
    }

    fn choose_existing(
        self: &Arc<Self>,
        selection: ExistingModelSelection,
    ) -> Result<(), CommandError> {
        if self.setup_complete() {
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
    /// its health check, and - unless the verification stamp vouches for the
    /// model - hashes it and runs a real inference through it. Launch used to
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
    /// would only hand it back with MODEL_NOT_READY.
    fn hold_local_model(&self) {
        self.local_ready.store(false, Ordering::SeqCst);
        self.refresh_ready();
    }

    /// Follows a change of model in Settings.
    ///
    /// Choosing a hosted model stops the local one: it holds 1.3-2.6 GB for
    /// as long as it runs, and nothing would ask it anything. Choosing the
    /// local model again starts and verifies it like a launch does, and the
    /// queue waits until that finishes. A setup operation already running is
    /// reported as SETUP_BUSY.
    fn model_source_changed(
        self: &Arc<Self>,
        from: ModelSource,
        to: ModelSource,
    ) -> Result<(), CommandError> {
        match (from, to) {
            (ModelSource::Local, ModelSource::Hosted) => {
                // Held even when the stop fails: the queue must not send the
                // local model anything once the hosted one is chosen.
                let stopped = self.runtime.hold();
                self.hold_local_model();
                stopped.map_err(|error| CommandError {
                    code: error.code,
                    message: "the local model could not be stopped".into(),
                })
            }
            (ModelSource::Hosted, ModelSource::Local) => {
                self.runtime.release();
                if self.runtime.installed_quick(self.runtime.manifest()) {
                    self.start_operation(SetupSource::Installed)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
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
                let installed = final_state.0 == SetupStatus::Ready;
                manager.set_state(final_state.0, final_state.1, final_state.2);
                manager.operation.finish();
                // A model this operation left installed but not started -
                // it was held for a hosted model, and the local one has been
                // chosen again while it ran - is started now, rather than
                // leaving the queue waiting on a server nothing will start.
                if installed && !manager.runtime.is_held() && !manager.runtime.running() {
                    manager.verify_installed();
                }
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
        self.runtime
            .stop_runtime()
            .map(|_| ())
            .map_err(|error| CommandError {
                code: error.code,
                message: "local model setup could not be canceled cleanly".into(),
            })
    }

    fn install_and_start(
        &self,
        source: SetupSource,
        cancellation: &CancellationToken,
    ) -> Result<u64, CommandError> {
        let manifest = self.runtime.manifest();
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
                self.runtime.remember_verified();
            }
            SetupSource::Existing(selection) => {
                install_existing_model_files(
                    manifest,
                    &selection,
                    &self.runtime.model_directory,
                    &SystemDiskSpace,
                    cancellation,
                    |progress| {
                        self.set_state(SetupStatus::Downloading, progress.completed_bytes, None);
                    },
                )?;
                self.runtime.remember_verified();
            }
        }
        if cancellation.is_canceled() {
            return Err(setup_canceled_error());
        }
        self.runtime.start_verified(cancellation)?;
        Ok(total)
    }

    fn set_state(&self, state: SetupStatus, downloaded_bytes: u64, error: Option<String>) {
        // Ready means installed; the queue may use the model only while a
        // server is actually running, which it is not while a hosted model
        // is chosen.
        let local_ready = state == SetupStatus::Ready && self.runtime.running();
        self.local_ready.store(local_ready, Ordering::SeqCst);
        let ready = self.refresh_ready();
        let mut emitted = false;
        if let Ok(mut current) = self.state.lock() {
            let status_changed = current.state != state;
            current.state = state;
            current.downloaded_bytes = downloaded_bytes.min(current.total_bytes);
            current.error = error;
            current.hosted_model_ready = self.hosted_active.load(Ordering::SeqCst);
            // The state itself is always current - the window polls it while
            // a download runs - but only some updates are worth an event.
            emitted = state != SetupStatus::Downloading
                || self
                    .throttle
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .should_emit(
                        std::time::Instant::now(),
                        current.downloaded_bytes,
                        current.total_bytes,
                        status_changed,
                    );
            if emitted {
                (self.publish)(&current);
            }
        }
        if ready && emitted {
            self.wake_scheduler();
        }
    }
}

/// What launch does about the local model, decided by the model the settings
/// name rather than by whether a model file happens to be installed.
///
/// A person who had chosen a hosted model still got the local one loaded and
/// self-tested at every launch, and resident for the rest of the day: 1.3 to
/// 2.6 GB on an 8 GB laptop for a model nothing would ask anything. Nothing
/// here reads the model file; the setup thread does that.
fn launch_local_model(
    publish: SetupPublisher,
    runtime: Arc<RuntimeModel>,
    source: ModelSource,
) -> Arc<SetupManager> {
    let installed = runtime.installed_quick(runtime.manifest());
    let setup = Arc::new(SetupManager::new(publish, Arc::clone(&runtime), installed));
    match source {
        ModelSource::Hosted => {
            let _ = runtime.hold();
            setup.hold_local_model();
        }
        ModelSource::Local if installed => setup.verify_installed(),
        ModelSource::Local => {}
    }
    setup
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

/// The watcher's configuration for the canonical intake `folder`.
///
/// What was already in the folder when this machine began watching it is kept
/// in the app's own data, so a restart, an update, or a changed label does not
/// retake it and hold everything that arrived in between. Never in the shared
/// `.intern` folder, which every machine reads. `first_look` takes it again,
/// for a watch that starts now (see `starts_a_new_watch`).
fn intake_config(
    folder: PathBuf,
    settings: &AppSettings,
    data_dir: &Path,
    first_look: bool,
) -> IntakeConfig {
    let mut config = IntakeConfig::new(
        folder,
        SUPPORTED_EXTENSIONS
            .iter()
            .map(|extension| (*extension).to_owned())
            .collect(),
    );
    config.process_others_uploads = settings.process_others_uploads;
    config.backlog_file = Some(data_dir.join("intake-backlog.json"));
    config.retake_backlog = first_look;
    config
}

/// Whether a save starts a new watch of the intake folder: watching was just
/// turned on, or pointed at another folder, or told differently whose
/// documents it admits. "Only new documents" means new from that moment, so a
/// new watch takes the first look at what is already there again. The look
/// kept from an earlier watch of the same folder knew nothing of what arrived
/// while watching was off, and those documents were taken for this machine's
/// own new uploads - renamed and filed although the person had just said to
/// leave them alone. A changed label, or whether teammates' documents are
/// processed too, carries the same watch on.
fn starts_a_new_watch(previous: &AppSettings, settings: &AppSettings) -> bool {
    settings.intake_enabled
        && (!previous.intake_enabled
            || previous.intake_folder != settings.intake_folder
            || previous.intake_local_only != settings.intake_local_only
            || previous.intake_my_folder != settings.intake_my_folder)
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self, CommandError> {
        let data = app.path().app_local_data_dir().map_err(|_| CommandError {
            code: "APP_DATA_UNAVAILABLE".into(),
            message: "local application data directory is unavailable".into(),
        })?;
        intern_engine::logs::set_log_directory(data.join("logs"));
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
            ModelManifest::embedded()?,
        ));
        let settings = SettingsStore::new(data.join("settings.json"));
        // Defaults are how the window opens on a settings file nothing can
        // read - Settings is where a person repairs it, and the defaults keep
        // automatic renaming off while they do. What must not happen is
        // Intern behaving as though the file said "no destination, no intake":
        // the file is left exactly as it is until somebody deliberately saves,
        // and `intake_status_dto` reports the trouble to the interface.
        let startup_settings = settings.load().unwrap_or_default();
        let setup = launch_local_model(
            setup_publisher(app.clone()),
            Arc::clone(&runtime),
            startup_settings.model_source,
        );
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
            && let Err(error) = state.restart_intake(&startup_settings, false)
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
    /// `intake_status` instead of returned as an error. `first_look` is set
    /// only by a save that starts a new watch.
    fn restart_intake(&self, settings: &AppSettings, first_look: bool) -> Result<(), CommandError> {
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
                    let config = intake_config(folder, settings, &self.data_dir, first_look);
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
    /// Stops the local model when a hosted one is chosen, and starts and
    /// verifies it again when it is chosen back.
    fn model_source_changed(&self, from: ModelSource, to: ModelSource) -> Result<(), CommandError>;
    fn schedule(&self) -> Result<(), CommandError>;
    fn sync_tray(&self, run_in_background: bool);
    /// `first_look` asks the new watcher to take the first look at what is
    /// already in the folder again; see `starts_a_new_watch`.
    fn restart_intake(&self, settings: &AppSettings, first_look: bool) -> Result<(), CommandError>;
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

    fn model_source_changed(&self, from: ModelSource, to: ModelSource) -> Result<(), CommandError> {
        self.setup.model_source_changed(from, to)
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

    fn restart_intake(&self, settings: &AppSettings, first_look: bool) -> Result<(), CommandError> {
        AppState::restart_intake(self, settings, first_look)
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
    // The settings are saved whatever the local model makes of the change,
    // so its answer - SETUP_BUSY, say - is reported once the rest of the
    // save has been applied rather than in place of it.
    let source_changed = if previous.model_source != settings.model_source {
        let changed = state.model_source_changed(previous.model_source, settings.model_source);
        state.schedule()?;
        changed
    } else {
        Ok(())
    };
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
        state.restart_intake(&settings, starts_a_new_watch(&previous, &settings))?;
        state.emit_intake_changed()?;
    }
    source_changed
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
    if let Some(current) = current.as_ref()
        && current.model_source != previous.model_source
        && let Err(error) =
            runtime.model_source_changed(current.model_source, previous.model_source)
    {
        failures.push(error);
    }
    if let Err(error) = runtime.schedule() {
        failures.push(error);
    }
    runtime.sync_tray(previous.run_in_background);
    // Back to the watch that was running before, not a new one.
    if let Err(error) = runtime.restart_intake(previous, false) {
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
    use intern_intake::{CloudProviderKind, DoneOutcome, IntakeStatus, ItemState, MachineIdentity};
    use intern_queue::AppSettings;

    use super::{
        intake_config, save_settings, save_settings_and_autostart, test_runtime::RecordingRuntime,
        validate_description_settings, validate_intake_settings,
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
        // Built with the platform's own separator: on Windows this is exactly
        // C:\Users\pat\Contoso\Legal - Documents\Inbox, and elsewhere a
        // backslash is not a separator at all, so a literal Windows path would
        // have no parent to put Filed beside.
        let library = Path::new(if cfg!(windows) {
            r"C:\Users\pat\Contoso"
        } else {
            "/home/pat/Contoso"
        })
        .join("Legal - Documents");
        assert_eq!(
            filed_folder_for(&library.join("Inbox")),
            Some(library.join("Filed"))
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
            ItemState::Done {
                outcome: DoneOutcome::KeptOriginal,
                result_filename: None,
            }
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
        assert_eq!(json["unreadableDocuments"], 0);
        assert_eq!(json["claimedByOthers"], 0);
        assert_eq!(json["processedHere"], 0);
        assert_eq!(json["lastScanAt"], serde_json::Value::Null);
        assert_eq!(json["error"], serde_json::Value::Null);
        assert_eq!(json["arriving"], 0);
    }

    /// The backlog has to outlive the watcher, and it is this machine's own
    /// record: kept with the app's data, not in the folder teammates share.
    #[test]
    fn the_intake_backlog_is_kept_in_app_data_not_the_shared_folder() {
        let settings = AppSettings {
            process_others_uploads: true,
            ..AppSettings::default()
        };
        let config = intake_config(
            PathBuf::from("/srv/scans"),
            &settings,
            Path::new("/home/pat/.local/share/intern"),
            false,
        );
        assert_eq!(config.intake_root, PathBuf::from("/srv/scans"));
        assert!(config.process_others_uploads);
        assert_eq!(
            config.backlog_file,
            Some(PathBuf::from(
                "/home/pat/.local/share/intern/intake-backlog.json"
            ))
        );
        assert!(!config.retake_backlog);
        let first_look = intake_config(
            PathBuf::from("/srv/scans"),
            &settings,
            Path::new("/home/pat/.local/share/intern"),
            true,
        );
        assert!(first_look.retake_backlog);
    }

    /// The watcher claims by the same single list the queue admits by, so a
    /// format the worker learns to read - a legacy Word or Excel file, a bank
    /// statement's CSV - is picked up from a watched folder too, rather than
    /// left there unseen while a dropped copy of it would be read.
    #[test]
    fn the_watcher_claims_every_format_the_queue_admits() {
        let config = intake_config(
            PathBuf::from("/srv/scans"),
            &AppSettings::default(),
            Path::new("/home/pat/.local/share/intern"),
            false,
        );
        assert_eq!(config.extensions, intern_queue::paths::SUPPORTED_EXTENSIONS);
        for extension in ["doc", "xls", "csv", "odt", "docm"] {
            assert!(
                config.extensions.iter().any(|watched| watched == extension),
                "{extension} is admitted, so it is watched"
            );
        }
    }

    /// "Only new documents" means new from when watching starts. Turning
    /// watching on, choosing another folder, or changing whose documents it
    /// admits starts a new watch, which takes the first look again; the look
    /// kept from an earlier watch of the same folder knew nothing of what
    /// arrived while watching was off, and those documents were renamed and
    /// filed as this machine's own. A restart, a changed label, or the
    /// everyone/mine choice carries the same watch on.
    #[test]
    fn only_a_new_watch_takes_the_first_look_again() {
        let dir = std::env::temp_dir().join(format!("intern-first-look-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Scans")).unwrap();
        std::fs::create_dir_all(dir.join("Other scans")).unwrap();
        std::fs::create_dir_all(dir.join("Filed")).unwrap();
        let dir = std::fs::canonicalize(dir).unwrap();
        let text = |name: &str| dir.join(name).to_string_lossy().into_owned();
        let watching = AppSettings {
            intake_enabled: true,
            intake_folder: text("Scans"),
            destination: text("Filed"),
            intake_my_folder: true,
            ..AppSettings::default()
        };
        let runtime = RecordingRuntime::new(dir.join("settings.json"));
        runtime
            .store
            .save(&AppSettings {
                intake_enabled: false,
                ..watching.clone()
            })
            .unwrap();

        let saves = [
            ("watching turned on", watching.clone(), true),
            (
                "a new label",
                AppSettings {
                    machine_label: "Front desk".into(),
                    ..watching.clone()
                },
                false,
            ),
            (
                "teammates' documents too",
                AppSettings {
                    machine_label: "Front desk".into(),
                    process_others_uploads: true,
                    ..watching.clone()
                },
                false,
            ),
            ("admitted differently", watching_local(&watching), true),
            (
                "another folder",
                AppSettings {
                    intake_folder: text("Other scans"),
                    ..watching_local(&watching)
                },
                true,
            ),
            (
                "watching turned off",
                AppSettings {
                    intake_enabled: false,
                    intake_folder: text("Other scans"),
                    ..watching_local(&watching)
                },
                false,
            ),
            (
                "and on again",
                AppSettings {
                    intake_folder: text("Other scans"),
                    ..watching_local(&watching)
                },
                true,
            ),
        ];
        for (what, settings, first_look) in saves {
            let before = runtime.intake_restarts.lock().unwrap().len();
            save_settings(&runtime, settings).unwrap();
            let restarts = runtime.intake_restarts.lock().unwrap().clone();
            assert_eq!(restarts.len(), before + 1, "{what} restarts the watcher");
            assert_eq!(restarts[before], first_look, "{what}");
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    fn watching_local(watching: &AppSettings) -> AppSettings {
        AppSettings {
            intake_my_folder: false,
            intake_local_only: true,
            ..watching.clone()
        }
    }

    /// Settings says what is on its way and what the queue could not take,
    /// so both counts have to reach the wire from a live watcher.
    #[test]
    fn status_dto_carries_the_arriving_and_unreadable_counts() {
        let identity = MachineIdentity {
            id: "0123456789abcdef0123456789abcdef".into(),
            name: "Front desk".into(),
            host_name: "DESKTOP-A1B2C3".into(),
            user: "pat".into(),
        };
        let live = IntakeStatus {
            arriving: 2,
            unreadable_documents: 3,
            last_scan_at: Some(1_755_850_000),
            ..IntakeStatus::idle(PathBuf::from("/srv/scans"))
        };
        let dto = status_dto(
            true,
            &identity,
            "/srv/scans",
            Some(&live),
            None,
            1_755_850_000,
        );
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["watching"], true);
        assert_eq!(json["arriving"], 2);
        assert_eq!(json["unreadableDocuments"], 3);
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
    use super::{hosted_model_test, queue_cancel, queue_list, settings_save};

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
mod runtime_tests {
    use std::{
        path::{Path, PathBuf},
        sync::{
            Arc, Condvar, Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc::{Receiver, channel},
        },
        time::{Duration, Instant},
    };

    use intern_engine::{
        DateRole, DocumentSource, Engine, EngineError, EngineErrorCode, EngineResult, Evidence,
        ModelFile, ModelManifest, ModelProposal, ModelRequest, ModelRole, PartyRelation, Proposer,
        download::CancellationToken, setup::ExistingModelSelection,
    };
    use intern_queue::{AnalyzerBoundary, ModelFailure, ModelSource};

    use super::{
        CommandError, LaunchedRuntime, ProgressThrottle, RuntimeLauncher, RuntimeModel,
        RuntimeProcess, SetupManager, SetupStateDto, SetupStatus, VERIFICATION_STAMP,
        launch_local_model,
    };

    /// Bytes chosen once and hashed offline; the digest below is theirs, so
    /// installing a chosen file checks a real digest.
    const MODEL_BYTES: &[u8] = b"the pinned model bytes";
    const MODEL_SHA256: &str = "b1f81de2183585b9793c38b27cdd46842ab24247c628b6fbbda1200b9a25ce99";
    const SETUP_THREAD: &str = "intern-model-setup";

    /// What the fake server answers.
    #[derive(Clone, Copy, Debug)]
    enum Reply {
        /// The calibration document read correctly.
        Calibration,
        /// A reading that names nothing the calibration document says.
        Wrong,
        Fail(EngineErrorCode),
        /// Holds the request open until the server is stopped under it, then
        /// fails it the way a killed server does.
        HangUntilStopped,
    }

    /// Everything the fakes count, shared by every server one test launches.
    struct Rig {
        launches: AtomicUsize,
        stops: AtomicUsize,
        alive: AtomicUsize,
        most_alive: AtomicUsize,
        proposals: AtomicUsize,
        /// The thread each full digest check ran on, by name.
        hashes: Mutex<Vec<Option<String>>>,
        reply: Mutex<Reply>,
        /// Set while a hanging request waits for its server to stop.
        in_flight: (Mutex<bool>, Condvar),
        /// When set, a stop waits here until the test opens it.
        stop_gate: Mutex<Option<Arc<Gate>>>,
        /// When set, a launch waits here until the test opens it.
        launch_gate: Mutex<Option<Arc<Gate>>>,
    }

    impl Rig {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                launches: AtomicUsize::new(0),
                stops: AtomicUsize::new(0),
                alive: AtomicUsize::new(0),
                most_alive: AtomicUsize::new(0),
                proposals: AtomicUsize::new(0),
                hashes: Mutex::new(Vec::new()),
                reply: Mutex::new(Reply::Calibration),
                in_flight: (Mutex::new(false), Condvar::new()),
                stop_gate: Mutex::new(None),
                launch_gate: Mutex::new(None),
            })
        }

        fn launches(&self) -> usize {
            self.launches.load(Ordering::SeqCst)
        }

        fn hash_count(&self) -> usize {
            self.hashes.lock().unwrap().len()
        }

        fn reply(&self, reply: Reply) {
            *self.reply.lock().unwrap() = reply;
        }

        fn wait_in_flight(&self) {
            let (lock, wake) = &self.in_flight;
            let mut in_flight = lock.lock().unwrap();
            let deadline = Instant::now() + Duration::from_secs(30);
            while !*in_flight {
                let left = deadline.saturating_duration_since(Instant::now());
                assert!(!left.is_zero(), "the request never started");
                in_flight = wake.wait_timeout(in_flight, left).unwrap().0;
            }
        }
    }

    /// A door a fake waits at: `reached` once something is waiting, `open`
    /// to let it through.
    #[derive(Default)]
    struct Gate {
        state: Mutex<(bool, bool)>,
        wake: Condvar,
    }

    impl Gate {
        fn pass(&self) {
            let mut state = self.state.lock().unwrap();
            state.0 = true;
            self.wake.notify_all();
            while !state.1 {
                state = self.wake.wait(state).unwrap();
            }
        }

        fn wait_reached(&self) {
            let mut state = self.state.lock().unwrap();
            let deadline = Instant::now() + Duration::from_secs(30);
            while !state.0 {
                let left = deadline.saturating_duration_since(Instant::now());
                assert!(!left.is_zero(), "nothing reached the gate");
                state = self.wake.wait_timeout(state, left).unwrap().0;
            }
        }

        fn open(&self) {
            self.state.lock().unwrap().1 = true;
            self.wake.notify_all();
        }
    }

    #[derive(Default)]
    struct FakeServer {
        stopped: Mutex<bool>,
        wake: Condvar,
    }

    struct FakeProcess {
        server: Arc<FakeServer>,
        rig: Arc<Rig>,
    }

    impl RuntimeProcess for FakeProcess {
        fn stop(self: Box<Self>) -> Result<(), ModelFailure> {
            let gate = self.rig.stop_gate.lock().unwrap().clone();
            if let Some(gate) = gate {
                gate.pass();
            }
            *self.server.stopped.lock().unwrap() = true;
            self.server.wake.notify_all();
            self.rig.alive.fetch_sub(1, Ordering::SeqCst);
            self.rig.stops.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct FakeProposer {
        server: Arc<FakeServer>,
        rig: Arc<Rig>,
    }

    impl Proposer for FakeProposer {
        fn propose(&self, _request: &ModelRequest) -> EngineResult<ModelProposal> {
            self.rig.proposals.fetch_add(1, Ordering::SeqCst);
            let reply = *self.rig.reply.lock().unwrap();
            match reply {
                Reply::Calibration => Ok(calibration_reply("Northstar Calibration Holdings LLC")),
                Reply::Wrong => Ok(calibration_reply("Somebody Else Entirely")),
                Reply::Fail(code) => Err(EngineError::new(code, "scripted failure")),
                Reply::HangUntilStopped => {
                    {
                        let (lock, wake) = &self.rig.in_flight;
                        *lock.lock().unwrap() = true;
                        wake.notify_all();
                    }
                    let mut stopped = self.server.stopped.lock().unwrap();
                    while !*stopped {
                        stopped = self.server.wake.wait(stopped).unwrap();
                    }
                    Err(EngineError::new(
                        EngineErrorCode::ModelRequestFailed,
                        "connection reset",
                    ))
                }
            }
        }
    }

    struct FakeLauncher {
        rig: Arc<Rig>,
    }

    impl RuntimeLauncher for FakeLauncher {
        fn launch(&self, _manifest: &ModelManifest) -> Result<LaunchedRuntime, CommandError> {
            let gate = self.rig.launch_gate.lock().unwrap().clone();
            if let Some(gate) = gate {
                gate.pass();
            }
            self.rig.launches.fetch_add(1, Ordering::SeqCst);
            let alive = self.rig.alive.fetch_add(1, Ordering::SeqCst) + 1;
            self.rig.most_alive.fetch_max(alive, Ordering::SeqCst);
            let server = Arc::new(FakeServer::default());
            Ok(LaunchedRuntime {
                process: Box::new(FakeProcess {
                    server: Arc::clone(&server),
                    rig: Arc::clone(&self.rig),
                }),
                engine: Engine::with_proposer(Box::new(FakeProposer {
                    server,
                    rig: Arc::clone(&self.rig),
                })),
            })
        }
    }

    fn calibration_reply(party: &str) -> ModelProposal {
        ModelProposal {
            document_type: Some("Notice of Calibration".into()),
            document_date: Some("2024-01-02".into()),
            date_role: Some(DateRole::Notice),
            parties: vec![party.to_owned()],
            party_relation: PartyRelation::To,
            description: format!(
                "Notice of calibration confirming that the local text model path is working for {party}."
            ),
            confidence: 0.99,
            needs_review: false,
            evidence: Evidence {
                date: Some("Date of this Notice: January 2, 2024".into()),
                document_type: Some("NOTICE OF CALIBRATION".into()),
                parties: vec![format!("To: {party}")],
            },
        }
    }

    fn manifest() -> ModelManifest {
        ModelManifest {
            schema_version: 2,
            model_id: "test-model".into(),
            served_model_name: "intern-local".into(),
            files: vec![ModelFile {
                name: "model.gguf".into(),
                role: ModelRole::Model,
                url: "https://example.invalid/model.gguf".into(),
                size: MODEL_BYTES.len() as u64,
                sha256: MODEL_SHA256.into(),
            }],
        }
    }

    /// A scratch data folder with the model and a server binary installed,
    /// removed when the test ends.
    struct Install {
        root: PathBuf,
    }

    impl Install {
        fn new(name: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("intern-runtime-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(root.join("models")).unwrap();
            std::fs::write(root.join("models").join("model.gguf"), MODEL_BYTES).unwrap();
            std::fs::write(root.join("llama-server"), b"server binary").unwrap();
            Self { root }
        }

        fn model(&self) -> PathBuf {
            self.root.join("models").join("model.gguf")
        }

        fn stamp(&self) -> PathBuf {
            self.root.join("models").join(VERIFICATION_STAMP)
        }

        /// Moves the model's last-modified time, as a replaced file would.
        fn touch_model(&self, seconds_ago: u64) {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(self.model())
                .unwrap();
            file.set_modified(std::time::SystemTime::now() - Duration::from_secs(seconds_ago))
                .unwrap();
        }

        /// A launch of this install: a fresh runtime, as a new process has.
        fn runtime(&self, rig: &Arc<Rig>, app_version: &str) -> Arc<RuntimeModel> {
            let counted = Arc::clone(rig);
            Arc::new(RuntimeModel::with_parts(
                self.root.join("llama-server"),
                self.root.join("models"),
                manifest(),
                Arc::new(FakeLauncher {
                    rig: Arc::clone(rig),
                }),
                Box::new(
                    move |_path: &Path, _file: &ModelFile| -> Result<(), EngineError> {
                        counted
                            .hashes
                            .lock()
                            .unwrap()
                            .push(std::thread::current().name().map(str::to_owned));
                        Ok(())
                    },
                ),
                app_version.to_owned(),
            ))
        }
    }

    impl Drop for Install {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    /// A setup manager whose published states arrive on the receiver.
    fn manager(
        runtime: &Arc<RuntimeModel>,
        source: ModelSource,
    ) -> (Arc<SetupManager>, Receiver<SetupStateDto>) {
        let (sender, receiver) = channel();
        let setup = launch_local_model(
            Box::new(move |state: &SetupStateDto| {
                let _ = sender.send(state.clone());
            }),
            Arc::clone(runtime),
            source,
        );
        (setup, receiver)
    }

    /// Waits for the setup operation in flight, if any, to finish entirely.
    fn settle(setup: &SetupManager) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if setup.operation.begin().is_ok() {
                setup.operation.finish();
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("the setup operation never finished");
    }

    fn verified(runtime: &RuntimeModel) {
        runtime
            .start_verified(&CancellationToken::new())
            .expect("the installed model starts and passes its self-test");
    }

    fn probe() -> DocumentSource {
        intern_engine::setup::semantic_probes()
            .unwrap()
            .remove(0)
            .document
    }

    #[test]
    fn initialize_path_never_hashes_the_model() {
        let install = Install::new("initialize");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");

        let (setup, _states) = manager(&runtime, ModelSource::Local);
        // What launch did on the thread the window waits for: nothing read
        // the model. Whatever hashing there is happens behind the window.
        let on_this_thread = std::thread::current().name().map(str::to_owned);
        settle(&setup);
        let hashes = rig.hashes.lock().unwrap().clone();
        assert!(
            !hashes.contains(&on_this_thread),
            "launch hashed the model on its own thread: {hashes:?}"
        );
        assert_eq!(hashes, vec![Some(SETUP_THREAD.to_owned())]);
        assert!(runtime.running());
        assert!(setup.model_ready.load(Ordering::SeqCst));

        // The quick check and the manager itself never read the file, even
        // for a launch whose verification has not happened yet.
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        assert!(runtime.installed_quick(runtime.manifest()));
        let _setup =
            SetupManager::new(Box::new(|_: &SetupStateDto| {}), Arc::clone(&runtime), true);
        assert_eq!(rig.hash_count(), 0);

        // A file of the wrong size is not installed, and finding that out
        // reads nothing either.
        std::fs::write(install.model(), b"short").unwrap();
        assert!(!runtime.installed_quick(runtime.manifest()));
        assert_eq!(rig.hash_count(), 0);
    }

    #[test]
    fn verification_stamp_skips_hash_and_self_test_when_unchanged() {
        let install = Install::new("stamp-unchanged");
        let rig = Rig::new();
        verified(&install.runtime(&rig, "1.0.0"));
        assert_eq!(rig.hash_count(), 1);
        assert_eq!(rig.proposals.load(Ordering::SeqCst), 1, "one self-test");
        assert!(install.stamp().is_file());

        // The next launch: same file, same server, same build.
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);
        assert_eq!(rig.hash_count(), 0, "no digest");
        assert_eq!(rig.proposals.load(Ordering::SeqCst), 0, "no self-test");
        assert_eq!(rig.launches(), 1);
        assert!(runtime.running());

        // And through the manager, as a real launch goes.
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        let (setup, states) = manager(&runtime, ModelSource::Local);
        settle(&setup);
        assert_eq!(rig.hash_count(), 0);
        assert_eq!(rig.proposals.load(Ordering::SeqCst), 0);
        assert_eq!(states.try_recv().unwrap().state, SetupStatus::Ready);
        assert!(setup.model_ready.load(Ordering::SeqCst));
    }

    #[test]
    fn changed_metadata_forces_one_hash_and_self_test() {
        let install = Install::new("stamp-changed");
        verified(&install.runtime(&Rig::new(), "1.0.0"));
        let original = std::fs::read(install.stamp()).unwrap();

        let relaunch = |app_version: &str| {
            let rig = Rig::new();
            verified(&install.runtime(&rig, app_version));
            (
                rig.hash_count(),
                rig.proposals.load(Ordering::SeqCst),
                std::fs::read(install.stamp()).unwrap(),
            )
        };

        // A file with a new date, as a replaced one has.
        install.touch_model(3_600);
        let (hashes, self_tests, rewritten) = relaunch("1.0.0");
        assert_eq!((hashes, self_tests), (1, 1));
        assert_ne!(rewritten, original, "the stamp records the new date");
        assert_eq!(relaunch("1.0.0").0, 0, "and is trusted again after");

        // A new build of Intern.
        let (hashes, self_tests, _) = relaunch("1.1.0");
        assert_eq!((hashes, self_tests), (1, 1));
        assert_eq!(relaunch("1.1.0").0, 0);

        // A new server binary, which is what the self-test exercised.
        std::fs::write(install.root.join("llama-server"), b"a newer server binary").unwrap();
        let (hashes, self_tests, _) = relaunch("1.1.0");
        assert_eq!((hashes, self_tests), (1, 1));

        // A stamp for a file of another size.
        let mut stamp: serde_json::Value =
            serde_json::from_slice(&std::fs::read(install.stamp()).unwrap()).unwrap();
        stamp["files"][0]["len"] = serde_json::json!(MODEL_BYTES.len() + 1);
        std::fs::write(install.stamp(), serde_json::to_vec(&stamp).unwrap()).unwrap();
        let (hashes, self_tests, _) = relaunch("1.1.0");
        assert_eq!((hashes, self_tests), (1, 1));

        // Within one launch the digest is not read twice for an unchanged
        // file: a second verified start trusts this session's own check.
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "2.0.0");
        verified(&runtime);
        std::fs::remove_file(install.stamp()).unwrap();
        verified(&runtime);
        assert_eq!(rig.hash_count(), 1);
        assert_eq!(rig.proposals.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn failed_self_test_deletes_stamp() {
        let install = Install::new("stamp-self-test");
        verified(&install.runtime(&Rig::new(), "1.0.0"));
        assert!(install.stamp().is_file());

        // A new build, whose self-test the model then fails.
        let rig = Rig::new();
        rig.reply(Reply::Wrong);
        let runtime = install.runtime(&rig, "1.1.0");
        let error = runtime
            .start_verified(&CancellationToken::new())
            .unwrap_err();
        assert_eq!(error.code, "MODEL_SELF_TEST_FAILED");
        assert!(!install.stamp().exists());
        assert!(
            !runtime.running(),
            "a model that failed is not left running"
        );

        // A request that fails names why, so a dead server and a wrong
        // answer can be told apart.
        let rig = Rig::new();
        rig.reply(Reply::Fail(EngineErrorCode::ModelServerUnhealthy));
        let error = install
            .runtime(&rig, "1.1.0")
            .start_verified(&CancellationToken::new())
            .unwrap_err();
        assert_eq!(error.code, "MODEL_SELF_TEST_FAILED");
        assert_eq!(
            error.message,
            "local model semantic self-test request failed: MODEL_SERVER_UNHEALTHY"
        );
        assert!(!install.stamp().exists());

        // Nothing is stamped until a self-test passes.
        let rig = Rig::new();
        verified(&install.runtime(&rig, "1.1.0"));
        assert!(install.stamp().is_file());
    }

    #[test]
    fn concurrent_cancel_and_recover_launch_once() {
        let install = Install::new("cancel-recover");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);
        let hashes = rig.hash_count();

        // A request fails on its own: a server fault, worth a restart.
        rig.reply(Reply::Fail(EngineErrorCode::ModelRequestFailed));
        let failure = runtime.analyze(&probe(), "pdf", &[]).unwrap_err();
        assert_eq!(failure, ModelFailure::retryable("MODEL_REQUEST_FAILED"));
        rig.reply(Reply::Calibration);

        // Meanwhile someone cancels. The cancel's stop is held open until the
        // recover has been sent after it.
        let gate = Arc::new(Gate::default());
        *rig.stop_gate.lock().unwrap() = Some(Arc::clone(&gate));
        let canceling = Arc::clone(&runtime);
        let cancel = std::thread::spawn(move || canceling.cancel());
        gate.wait_reached();
        let recovering = Arc::clone(&runtime);
        let recover = std::thread::spawn(move || recovering.recover(&failure));
        std::thread::sleep(Duration::from_millis(50));
        *rig.stop_gate.lock().unwrap() = None;
        gate.open();

        assert_eq!(cancel.join().unwrap(), Ok(()));
        assert_eq!(recover.join().unwrap(), Ok(()));
        assert_eq!(
            rig.launches(),
            2,
            "the first start and the cancel's restart"
        );
        assert_eq!(
            rig.most_alive.load(Ordering::SeqCst),
            1,
            "never two servers"
        );
        assert_eq!(rig.alive.load(Ordering::SeqCst), 1);
        assert_eq!(rig.hash_count(), hashes, "neither restart read the model");
        assert!(runtime.analyze(&probe(), "pdf", &[]).is_ok());
    }

    #[test]
    fn recover_after_newer_generation_does_not_restart() {
        let install = Install::new("recover-generation");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);

        rig.reply(Reply::Fail(EngineErrorCode::ModelRequestFailed));
        let failure = runtime.analyze(&probe(), "pdf", &[]).unwrap_err();
        assert!(failure.retryable);
        runtime.cancel().unwrap();
        assert_eq!(rig.launches(), 2);

        // The cancel has already put a fresh server under it.
        runtime.recover(&failure).unwrap();
        assert_eq!(rig.launches(), 2);

        // A failure with nothing in between is still recovered, once - and
        // without reading the model.
        let failure = runtime.analyze(&probe(), "pdf", &[]).unwrap_err();
        runtime.recover(&failure).unwrap();
        assert_eq!(rig.launches(), 3);
        assert_eq!(rig.hash_count(), 1, "only the verified start hashed");
    }

    #[test]
    fn analyze_after_cancel_reports_model_canceled_not_retryable() {
        let install = Install::new("analyze-cancel");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);

        rig.reply(Reply::HangUntilStopped);
        let analyzing = Arc::clone(&runtime);
        let request = std::thread::spawn(move || analyzing.analyze(&probe(), "pdf", &[]));
        rig.wait_in_flight();
        rig.reply(Reply::Calibration);
        runtime.cancel().unwrap();

        // The server stopped under the request on purpose. Reporting that as
        // a server fault had the queue restart it again and re-read the
        // document someone had just canceled.
        assert_eq!(
            request.join().unwrap().unwrap_err(),
            ModelFailure::fatal("MODEL_CANCELED")
        );
        assert_eq!(rig.launches(), 2);
        assert!(runtime.analyze(&probe(), "pdf", &[]).is_ok());
    }

    /// A cancel no longer holds the queue while it restarts the server, so
    /// the next document reaches the model while the new server is still
    /// loading - and so does one that arrives as the local model is chosen
    /// again. Each waits for the server instead of being handed back, which
    /// the queue counted as a failure and, the second time, failed it for.
    #[test]
    fn the_next_document_waits_out_a_restart_under_way() {
        let install = Install::new("analyze-restart");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);
        let asked = || rig.proposals.load(Ordering::SeqCst);
        let before = asked();

        let gate = Arc::new(Gate::default());
        *rig.launch_gate.lock().unwrap() = Some(Arc::clone(&gate));
        let canceling = Arc::clone(&runtime);
        let cancel = std::thread::spawn(move || canceling.cancel());
        gate.wait_reached();
        *rig.launch_gate.lock().unwrap() = None;
        let analyzing = Arc::clone(&runtime);
        let next = std::thread::spawn(move || analyzing.analyze(&probe(), "pdf", &[]));
        std::thread::sleep(Duration::from_millis(50));
        assert!(!next.is_finished(), "it waits for the new server");
        gate.open();
        assert_eq!(cancel.join().unwrap(), Ok(()));
        assert!(next.join().unwrap().is_ok());
        assert_eq!(asked(), before + 1, "asked once, of the new server");

        // A hosted model chosen and then the local one again: the verified
        // start holds the slot empty while it loads.
        runtime.hold().unwrap();
        runtime.release();
        let gate = Arc::new(Gate::default());
        *rig.launch_gate.lock().unwrap() = Some(Arc::clone(&gate));
        let starting = Arc::clone(&runtime);
        let start = std::thread::spawn(move || {
            starting
                .start_verified(&CancellationToken::new())
                .map_err(|error| error.code)
        });
        gate.wait_reached();
        *rig.launch_gate.lock().unwrap() = None;
        let analyzing = Arc::clone(&runtime);
        let next = std::thread::spawn(move || analyzing.analyze(&probe(), "pdf", &[]));
        std::thread::sleep(Duration::from_millis(50));
        assert!(!next.is_finished(), "it waits for the verified start");
        gate.open();
        assert_eq!(start.join().unwrap(), Ok(()));
        assert!(next.join().unwrap().is_ok());
        assert_eq!(rig.launches(), 3);
    }

    /// A cancel moves the generation before it stops anything, and for a
    /// moment the engine it is stopping is still in the slot. A document
    /// that arrives then is not sent to it - its request would fail under
    /// the stop and read as a canceled request - but waits for the new one.
    #[test]
    fn an_engine_a_cancel_is_stopping_is_not_asked() {
        let install = Install::new("analyze-stale");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);
        let before = rig.proposals.load(Ordering::SeqCst);

        let gate = Arc::new(Gate::default());
        *rig.stop_gate.lock().unwrap() = Some(Arc::clone(&gate));
        let canceling = Arc::clone(&runtime);
        let cancel = std::thread::spawn(move || canceling.cancel());
        gate.wait_reached();
        *rig.stop_gate.lock().unwrap() = None;
        let analyzing = Arc::clone(&runtime);
        let next = std::thread::spawn(move || analyzing.analyze(&probe(), "pdf", &[]));
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(
            rig.proposals.load(Ordering::SeqCst),
            before,
            "nothing was sent to the server being stopped"
        );
        assert!(!next.is_finished());
        gate.open();

        assert_eq!(cancel.join().unwrap(), Ok(()));
        assert!(next.join().unwrap().is_ok());
        assert_eq!(rig.proposals.load(Ordering::SeqCst), before + 1);
        assert_eq!(rig.launches(), 2);
    }

    /// The lock itself, which the generation check cannot stand in for: a
    /// cancel that arrives while a recovery is already loading a new server
    /// waits for it and then restarts that one. Unserialized, the cancel
    /// found the slot empty, launched nothing, and left running the server a
    /// re-sent request for the canceled document would be read on - or,
    /// before the slot check, loaded a second server beside it.
    #[test]
    fn a_cancel_during_a_recovery_launch_waits_for_it_and_restarts_once() {
        let install = Install::new("recover-launch-cancel");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);

        rig.reply(Reply::Fail(EngineErrorCode::ModelRequestFailed));
        let failure = runtime.analyze(&probe(), "pdf", &[]).unwrap_err();
        rig.reply(Reply::Calibration);

        let gate = Arc::new(Gate::default());
        *rig.launch_gate.lock().unwrap() = Some(Arc::clone(&gate));
        let recovering = Arc::clone(&runtime);
        let recover = std::thread::spawn(move || recovering.recover(&failure));
        // The recovery has stopped the failed server and is loading another.
        gate.wait_reached();
        *rig.launch_gate.lock().unwrap() = None;
        let canceling = Arc::clone(&runtime);
        let cancel = std::thread::spawn(move || canceling.cancel());
        std::thread::sleep(Duration::from_millis(50));
        assert!(!cancel.is_finished(), "the cancel waits for the recovery");
        gate.open();

        assert_eq!(recover.join().unwrap(), Ok(()));
        assert_eq!(cancel.join().unwrap(), Ok(()));
        assert_eq!(
            rig.launches(),
            3,
            "the first start, the recovery's, and the cancel's restart"
        );
        assert_eq!(
            rig.stops.load(Ordering::SeqCst),
            2,
            "the failed server, then the recovered one"
        );
        assert_eq!(
            rig.most_alive.load(Ordering::SeqCst),
            1,
            "never two servers"
        );
        assert_eq!(rig.alive.load(Ordering::SeqCst), 1);
        assert!(runtime.analyze(&probe(), "pdf", &[]).is_ok());
    }

    #[test]
    fn input_too_large_and_invalid_reply_are_fatal() {
        let install = Install::new("fatal-codes");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);

        for code in [
            EngineErrorCode::ModelInputTooLarge,
            EngineErrorCode::ModelResponseInvalid,
            EngineErrorCode::ModelReplyTruncated,
            EngineErrorCode::AnalysisFailed,
        ] {
            rig.reply(Reply::Fail(code));
            assert_eq!(
                runtime.analyze(&probe(), "pdf", &[]).unwrap_err(),
                ModelFailure::fatal(code.as_str()),
            );
        }
        // A server that failed the request is still worth a restart.
        rig.reply(Reply::Fail(EngineErrorCode::ModelRequestFailed));
        assert_eq!(
            runtime.analyze(&probe(), "pdf", &[]).unwrap_err(),
            ModelFailure::retryable("MODEL_REQUEST_FAILED")
        );
    }

    #[test]
    fn a_runtime_with_no_engine_hands_the_document_back() {
        let install = Install::new("not-ready");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        // Never started, or between a stop and a start: nothing is wrong
        // with the document, so it is not a failure to count against it.
        assert_eq!(
            runtime.analyze(&probe(), "pdf", &[]).unwrap_err(),
            ModelFailure::retryable("MODEL_NOT_READY")
        );
        verified(&runtime);
        runtime.shutdown().unwrap();
        assert_eq!(
            runtime.analyze(&probe(), "pdf", &[]).unwrap_err(),
            ModelFailure::retryable("MODEL_NOT_READY")
        );
        // Restarting, for that matter, is not what recovering from it means.
        runtime
            .recover(&ModelFailure::retryable("MODEL_NOT_READY"))
            .unwrap();
        assert_eq!(rig.launches(), 1);
    }

    #[test]
    fn restarts_never_start_a_model_file_that_changed_after_its_check() {
        let install = Install::new("changed-file");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);

        // Same size, new contents: a restart would have to hash it to trust
        // it, and restarts do not hash.
        std::fs::write(install.model(), b"the pinned model bytez").unwrap();
        install.touch_model(60);
        assert_eq!(
            runtime.cancel().unwrap_err(),
            ModelFailure::fatal("MODEL_NOT_READY")
        );
        assert_eq!(rig.launches(), 1);
        assert_eq!(rig.hash_count(), 1);
        assert!(!runtime.running());
    }

    #[test]
    fn a_start_under_way_at_shutdown_stops_what_it_launched() {
        let install = Install::new("shutdown-start");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        verified(&runtime);

        // A cancel's restart is still loading the new server when Intern
        // exits. Shutdown cannot wait for it: it runs on the thread Tauri
        // exits on.
        let gate = Arc::new(Gate::default());
        *rig.launch_gate.lock().unwrap() = Some(Arc::clone(&gate));
        let canceling = Arc::clone(&runtime);
        let cancel = std::thread::spawn(move || canceling.cancel());
        gate.wait_reached();
        runtime.shutdown().unwrap();
        gate.open();

        assert_eq!(cancel.join().unwrap(), Ok(()));
        assert_eq!(rig.launches(), 2);
        assert_eq!(rig.alive.load(Ordering::SeqCst), 0, "nothing left running");
        assert!(!runtime.running());
    }

    #[test]
    fn hosted_source_at_launch_starts_nothing() {
        let install = Install::new("hosted-launch");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");

        let (setup, _states) = manager(&runtime, ModelSource::Hosted);
        settle(&setup);
        assert_eq!(rig.launches(), 0);
        assert_eq!(rig.hash_count(), 0);
        assert!(!setup.local_ready.load(Ordering::SeqCst));
        // The window still says the model is installed.
        assert_eq!(setup.get().unwrap().state, SetupStatus::Ready);
        // Queue runs on the hosted model alone, once it is configured.
        assert!(!setup.model_ready.load(Ordering::SeqCst));
        setup.set_hosted_active(true);
        assert!(setup.model_ready.load(Ordering::SeqCst));

        // A cancel or recover meanwhile starts nothing either.
        runtime.cancel().unwrap();
        runtime
            .recover(&ModelFailure::retryable("MODEL_REQUEST_FAILED"))
            .unwrap();
        assert_eq!(rig.launches(), 0);
    }

    #[test]
    fn switching_sources_stops_and_starts() {
        let install = Install::new("switching");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        let (setup, states) = manager(&runtime, ModelSource::Local);
        settle(&setup);
        assert_eq!(rig.launches(), 1);
        assert!(setup.model_ready.load(Ordering::SeqCst));
        while states.try_recv().is_ok() {}

        // To hosted: the local server stops and stays stopped.
        setup
            .model_source_changed(ModelSource::Local, ModelSource::Hosted)
            .unwrap();
        assert_eq!(rig.stops.load(Ordering::SeqCst), 1);
        assert_eq!(rig.alive.load(Ordering::SeqCst), 0);
        assert!(!setup.model_ready.load(Ordering::SeqCst));

        // Back to local: one verified start, and the queue waits for it.
        let gate = Arc::new(Gate::default());
        *rig.launch_gate.lock().unwrap() = Some(Arc::clone(&gate));
        setup
            .model_source_changed(ModelSource::Hosted, ModelSource::Local)
            .unwrap();
        gate.wait_reached();
        assert!(!setup.model_ready.load(Ordering::SeqCst));
        gate.open();
        settle(&setup);
        assert_eq!(rig.launches(), 2);
        assert!(setup.model_ready.load(Ordering::SeqCst));
        assert_eq!(states.try_recv().unwrap().state, SetupStatus::Ready);
        // The stamp vouched for it: no second digest, no second self-test.
        assert_eq!(rig.hash_count(), 1);
        assert_eq!(rig.proposals.load(Ordering::SeqCst), 1);

        // A change while another setup operation runs is refused, and says so.
        *rig.launch_gate.lock().unwrap() = Some(Arc::new(Gate::default()));
        let held = rig.launch_gate.lock().unwrap().clone().unwrap();
        setup
            .model_source_changed(ModelSource::Local, ModelSource::Hosted)
            .unwrap();
        setup
            .model_source_changed(ModelSource::Hosted, ModelSource::Local)
            .unwrap();
        held.wait_reached();
        assert_eq!(
            setup
                .model_source_changed(ModelSource::Hosted, ModelSource::Local)
                .unwrap_err()
                .code,
            "SETUP_BUSY"
        );
        held.open();
        settle(&setup);
    }

    #[test]
    fn an_install_finished_for_a_held_model_starts_once_local_is_chosen_again() {
        let install = Install::new("held-install");
        std::fs::remove_file(install.model()).unwrap();
        let chosen = install.root.join("chosen.gguf");
        std::fs::write(&chosen, MODEL_BYTES).unwrap();
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");

        // Hosted is chosen while the local model is installed from a file.
        // The person switches back to the local model just as the install
        // finishes, too late for the install to start it and too early to
        // start another operation themselves.
        let switched = Arc::new(AtomicBool::new(false));
        let releasing = Arc::clone(&runtime);
        let seen = Arc::clone(&switched);
        let setup = launch_local_model(
            Box::new(move |state: &SetupStateDto| {
                if state.state == SetupStatus::Ready && !seen.swap(true, Ordering::SeqCst) {
                    releasing.release();
                }
            }),
            Arc::clone(&runtime),
            ModelSource::Hosted,
        );
        setup
            .choose_existing(ExistingModelSelection {
                model_path: chosen.clone(),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        while !(switched.load(Ordering::SeqCst) && runtime.running()) {
            assert!(Instant::now() < deadline, "the model was never started");
            std::thread::sleep(Duration::from_millis(5));
        }
        settle(&setup);
        assert_eq!(rig.launches(), 1);
        assert!(setup.model_ready.load(Ordering::SeqCst));
        // The install checked the digest itself; nothing read it again.
        assert_eq!(rig.hash_count(), 0);
    }

    #[test]
    fn an_install_while_hosted_is_chosen_starts_no_server() {
        let install = Install::new("hosted-install");
        std::fs::remove_file(install.model()).unwrap();
        let chosen = install.root.join("chosen.gguf");
        std::fs::write(&chosen, MODEL_BYTES).unwrap();
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        let (setup, _states) = manager(&runtime, ModelSource::Hosted);
        setup
            .choose_existing(ExistingModelSelection { model_path: chosen })
            .unwrap();
        settle(&setup);
        assert_eq!(setup.get().unwrap().state, SetupStatus::Ready);
        assert_eq!(rig.launches(), 0);
        assert!(!setup.local_ready.load(Ordering::SeqCst));
    }

    #[test]
    fn a_held_installed_model_is_not_set_up_again() {
        let install = Install::new("held-setup");
        let chosen = install.root.join("chosen.gguf");
        std::fs::write(&chosen, MODEL_BYTES).unwrap();
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        let (setup, _states) = manager(&runtime, ModelSource::Hosted);
        settle(&setup);

        // Held is not ready for the queue, but it is installed: neither a
        // download nor a chosen file may start over it.
        setup.start().unwrap();
        assert!(setup.operation.begin().is_ok(), "no setup operation began");
        setup.operation.finish();
        assert_eq!(
            setup
                .choose_existing(ExistingModelSelection { model_path: chosen })
                .unwrap_err()
                .code,
            "SETUP_ALREADY_READY"
        );
        assert_eq!(setup.get().unwrap().state, SetupStatus::Ready);
        assert_eq!(rig.launches(), 0);
    }

    #[test]
    fn choosing_hosted_mid_request_hands_the_document_back() {
        let install = Install::new("hosted-mid-request");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        let (setup, _states) = manager(&runtime, ModelSource::Local);
        settle(&setup);

        rig.reply(Reply::HangUntilStopped);
        let analyzing = Arc::clone(&runtime);
        let request = std::thread::spawn(move || analyzing.analyze(&probe(), "pdf", &[]));
        rig.wait_in_flight();
        setup
            .model_source_changed(ModelSource::Local, ModelSource::Hosted)
            .unwrap();

        // Nobody canceled the document, so it is not failed as canceled: it
        // goes back to be read again, by the hosted model now chosen.
        assert_eq!(
            request.join().unwrap().unwrap_err(),
            ModelFailure::retryable("MODEL_NOT_READY")
        );
        assert_eq!(rig.alive.load(Ordering::SeqCst), 0);
        // And handing it back restarts nothing.
        runtime
            .recover(&ModelFailure::retryable("MODEL_NOT_READY"))
            .unwrap();
        assert_eq!(rig.launches(), 1);
    }

    #[test]
    fn choosing_hosted_during_verification_neither_waits_nor_reports_a_broken_model() {
        let install = Install::new("hosted-self-test");
        let rig = Rig::new();
        let runtime = install.runtime(&rig, "1.0.0");
        // A first launch: no stamp, so the model is self-tested, and the
        // self-test is still waiting on the server when hosted is chosen.
        rig.reply(Reply::HangUntilStopped);
        let (setup, _states) = manager(&runtime, ModelSource::Local);
        rig.wait_in_flight();

        // The save does not wait for the self-test to finish: it would wait
        // on a server that only the save itself is going to stop.
        let switching = Arc::clone(&setup);
        let switch = std::thread::spawn(move || {
            switching.model_source_changed(ModelSource::Local, ModelSource::Hosted)
        });
        let deadline = Instant::now() + Duration::from_secs(30);
        while !switch.is_finished() {
            assert!(
                Instant::now() < deadline,
                "choosing hosted waited on the self-test"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        switch.join().unwrap().unwrap();
        settle(&setup);

        // Stopping the server under its self-test says nothing about the
        // model, which is still installed, and not running.
        let state = setup.get().unwrap();
        assert_eq!((state.state, state.error), (SetupStatus::Ready, None));
        assert_eq!(rig.alive.load(Ordering::SeqCst), 0);
        assert!(!setup.model_ready.load(Ordering::SeqCst));
        assert!(
            !install.stamp().exists(),
            "an unfinished self-test is not a pass"
        );

        // Chosen again, it is self-tested again - but this session already
        // checked its digest, so it is not read again.
        rig.reply(Reply::Calibration);
        setup
            .model_source_changed(ModelSource::Hosted, ModelSource::Local)
            .unwrap();
        settle(&setup);
        assert!(runtime.running());
        assert!(setup.model_ready.load(Ordering::SeqCst));
        assert!(install.stamp().is_file());
        assert_eq!(rig.hash_count(), 1);
    }

    #[test]
    fn progress_throttle_emits_on_time_fraction_status_and_final() {
        let total = 1_000_000;
        let start = Instant::now();
        let at = |millis: u64| start + Duration::from_millis(millis);
        let mut throttle = ProgressThrottle::new();

        assert!(throttle.should_emit(at(0), 0, total, false), "the first");
        assert!(!throttle.should_emit(at(10), 100, total, false));
        assert!(!throttle.should_emit(at(249), 4_999, total, false));
        assert!(throttle.should_emit(at(250), 5_000, total, false), "time");
        assert!(
            throttle.should_emit(at(260), 10_100, total, false),
            "half a percent"
        );
        assert!(!throttle.should_emit(at(270), 10_101, total, false));
        assert!(throttle.should_emit(at(271), 10_102, total, true), "status");
        assert!(throttle.should_emit(at(272), total, total, false), "final");
        // Backwards counts too: verifying starts its count again from zero.
        assert!(throttle.should_emit(at(273), 0, total, false));

        // A download sending a chunk every millisecond for ten seconds at a
        // steady crawl tells the window about it at most four times a second.
        let mut throttle = ProgressThrottle::new();
        let emitted = (0..10_000_u64)
            .filter(|millis| throttle.should_emit(at(*millis), millis * 4, 400_000_000, false))
            .count();
        assert!((40..=41).contains(&emitted), "{emitted}");
    }

    #[test]
    fn download_progress_reaches_the_window_throttled_but_the_state_stays_current() {
        let install = Install::new("throttled-state");
        let runtime = install.runtime(&Rig::new(), "1.0.0");
        let published = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&published);
        let setup = SetupManager::new(
            Box::new(move |state: &SetupStateDto| {
                recorded.lock().unwrap().push(state.downloaded_bytes);
            }),
            runtime,
            false,
        );
        for byte in 0..=MODEL_BYTES.len() as u64 {
            setup.set_state(SetupStatus::Downloading, byte, None);
        }
        // The first update, half a percent at a time on so small a file, and
        // the last: every one here moves 4.5%, so each is sent.
        assert_eq!(published.lock().unwrap().len(), MODEL_BYTES.len() + 1);

        let published = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&published);
        let big = SetupManager::new(
            Box::new(move |state: &SetupStateDto| {
                recorded.lock().unwrap().push(state.downloaded_bytes);
            }),
            install.runtime(&Rig::new(), "1.0.0"),
            false,
        );
        big.state.lock().unwrap().total_bytes = 1_000_000_000;
        for chunk in 0..1_000_u64 {
            big.set_state(SetupStatus::Downloading, chunk * 16_384, None);
        }
        assert_eq!(
            big.get().unwrap().downloaded_bytes,
            999 * 16_384,
            "what the window polls is never behind"
        );
        assert!(
            published.lock().unwrap().len() < 10,
            "{:?}",
            published.lock().unwrap()
        );
        big.set_state(
            SetupStatus::Failed,
            999 * 16_384,
            Some("MODEL_DOWNLOAD_FAILED".into()),
        );
        assert_eq!(published.lock().unwrap().last(), Some(&(999 * 16_384)));
    }

    /// The real llama-server through a whole lifecycle: start and verify,
    /// a document, a cancel in the middle of a request, another document,
    /// and shutdown - with exactly one server process at every step and
    /// none after. Runs only when INTERN_LIVE_LLAMA_SERVER and
    /// INTERN_LIVE_MODEL name a server binary and the pinned model.
    #[cfg(target_os = "linux")]
    #[test]
    fn live_runtime_keeps_exactly_one_server_through_cancel_and_shutdown() {
        let (Some(server), Some(model)) = (
            std::env::var_os("INTERN_LIVE_LLAMA_SERVER"),
            std::env::var_os("INTERN_LIVE_MODEL"),
        ) else {
            eprintln!("skipped: INTERN_LIVE_LLAMA_SERVER and INTERN_LIVE_MODEL are not both set");
            return;
        };
        let manifest = ModelManifest::embedded().unwrap();
        let name = manifest.model().unwrap().name.clone();
        // The model is linked into a scratch folder of its own, so the
        // verification stamp is written there and nowhere shared.
        let directory = std::env::temp_dir().join(format!("intern-live-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::os::unix::fs::symlink(PathBuf::from(&model), directory.join(&name)).unwrap();
        let runtime = Arc::new(RuntimeModel::new(
            PathBuf::from(server),
            directory.clone(),
            manifest,
        ));
        assert!(runtime.installed_quick(runtime.manifest()));
        assert_eq!(servers(), Vec::<u32>::new(), "a server was already running");

        runtime
            .start_verified(&CancellationToken::new())
            .expect("the pinned model starts and passes its self-test");
        let first = servers();
        assert_eq!(first.len(), 1, "after start: {first:?}");

        let memo = intern_engine::distill::source_from_text(
            "MEMORANDUM\n\nDate: March 2, 2026\n\nTo: All staff of Harbor Point Logistics\n\n\
             The office moves to the fourth floor on March 2, 2026.",
        );
        runtime
            .analyze(&memo, "pdf", &[])
            .expect("a short document is read");
        assert_eq!(servers(), first, "after a document");

        let long = || {
            intern_engine::distill::source_from_text(format!(
                "SERVICES AGREEMENT\n\nThis agreement is made on April 12, 2024 between Harbor \
                 Point Logistics LLC and Northwind Freight Partners.\n\n{}",
                "The provider shall perform the services described in each statement of work, \
                 and the customer shall pay the fees set out there within thirty days. "
                    .repeat(60)
            ))
        };
        let analyzing = Arc::clone(&runtime);
        let request = std::thread::spawn(move || analyzing.analyze(&long(), "pdf", &[]));
        std::thread::sleep(Duration::from_millis(1_500));
        assert!(
            !request.is_finished(),
            "the request ended before it could be canceled"
        );
        runtime.cancel().expect("the cancel restarts the server");
        assert_eq!(
            request.join().unwrap().unwrap_err(),
            ModelFailure::fatal("MODEL_CANCELED")
        );
        let restarted = servers();
        assert_eq!(restarted.len(), 1, "after cancel: {restarted:?}");
        assert_ne!(restarted, first, "the cancel replaced the server");

        // The next document arrives while a cancel is still loading the new
        // server, as it does in the queue: it waits for that server rather
        // than being handed back, and is read by it.
        let analyzing = Arc::clone(&runtime);
        let request = std::thread::spawn(move || analyzing.analyze(&long(), "pdf", &[]));
        std::thread::sleep(Duration::from_millis(1_500));
        assert!(!request.is_finished());
        let canceling = Arc::clone(&runtime);
        let cancel = std::thread::spawn(move || canceling.cancel());
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while runtime.running() {
            assert!(
                std::time::Instant::now() < deadline,
                "the cancel never began"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        runtime
            .analyze(&memo, "pdf", &[])
            .expect("the next document waits out the restart");
        assert_eq!(cancel.join().unwrap(), Ok(()));
        assert_eq!(
            request.join().unwrap().unwrap_err(),
            ModelFailure::fatal("MODEL_CANCELED")
        );
        let restarted_again = servers();
        assert_eq!(restarted_again.len(), 1, "after the second cancel");
        assert_ne!(restarted_again, restarted);
        let restarted = restarted_again;

        runtime
            .analyze(&memo, "pdf", &[])
            .expect("the restarted server reads a document");
        assert_eq!(servers(), restarted, "after the second document");

        runtime.shutdown().unwrap();
        assert_eq!(servers(), Vec::<u32>::new(), "after shutdown");
        let _ = std::fs::remove_dir_all(directory);
    }

    /// The process ids of this process's llama-server children.
    #[cfg(target_os = "linux")]
    fn servers() -> Vec<u32> {
        let me = std::process::id();
        let mut found = std::fs::read_dir("/proc")
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let pid = entry.file_name().to_str()?.parse::<u32>().ok()?;
                let stat = std::fs::read_to_string(entry.path().join("stat")).ok()?;
                // "pid (comm) state ppid ...": comm may hold spaces, so the
                // fields after it are read from its closing parenthesis.
                let (head, rest) = stat.rsplit_once(')')?;
                let comm = head.split_once('(')?.1;
                let ppid = rest.split_whitespace().nth(1)?.parse::<u32>().ok()?;
                (ppid == me && comm == "llama-server").then_some(pid)
            })
            .collect::<Vec<_>>();
        found.sort_unstable();
        found
    }
}

#[cfg(test)]
mod model_source_settings_tests {
    use std::{path::PathBuf, sync::atomic::Ordering};

    use intern_queue::{AppSettings, ModelSource};

    use super::{restore_sharepoint_settings, save_settings, test_runtime::RecordingRuntime};

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("intern-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn hosted() -> AppSettings {
        AppSettings {
            model_source: ModelSource::Hosted,
            ..AppSettings::default()
        }
    }

    #[test]
    fn saving_another_model_tells_the_local_model_and_nothing_else_does() {
        let dir = scratch("source-change");
        let runtime = RecordingRuntime::new(dir.join("settings.json"));

        save_settings(&runtime, AppSettings::default()).unwrap();
        save_settings(&runtime, hosted()).unwrap();
        save_settings(&runtime, hosted()).unwrap();
        save_settings(&runtime, AppSettings::default()).unwrap();

        assert_eq!(
            *runtime.source_changes.lock().unwrap(),
            vec![
                (ModelSource::Local, ModelSource::Hosted),
                (ModelSource::Hosted, ModelSource::Local),
            ]
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_busy_local_model_is_reported_once_the_rest_of_the_save_is_applied() {
        let dir = scratch("source-busy");
        let runtime = RecordingRuntime::new(dir.join("settings.json"));
        runtime.store.save(&hosted()).unwrap();
        runtime.setup_busy.store(true, Ordering::SeqCst);

        let error = save_settings(
            &runtime,
            AppSettings {
                run_in_background: true,
                ..AppSettings::default()
            },
        )
        .unwrap_err();

        // The local model could not start another setup operation now, and
        // the person is told; what they saved is saved and in effect.
        assert_eq!(error.code, "SETUP_BUSY");
        assert_eq!(
            runtime.store.load().unwrap().model_source,
            ModelSource::Local
        );
        assert!(runtime.live().tray);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn restoring_settings_puts_the_model_they_name_back() {
        let dir = scratch("source-restore");
        let runtime = RecordingRuntime::new(dir.join("settings.json"));
        runtime.store.save(&hosted()).unwrap();

        restore_sharepoint_settings(&runtime, &AppSettings::default()).unwrap();

        assert_eq!(
            *runtime.source_changes.lock().unwrap(),
            vec![(ModelSource::Hosted, ModelSource::Local)]
        );
        let _ = std::fs::remove_dir_all(dir);
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

    use intern_queue::{AppSettings, ModelSource, SettingsStore, paths::canonical_folder};

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
        /// Every watcher restart, by whether it asked for a first look.
        pub intake_restarts: Mutex<Vec<bool>>,
        /// Every change of model the runtime was told about, in order.
        pub source_changes: Mutex<Vec<(ModelSource, ModelSource)>>,
        /// The local model answers a change of model the way it does while
        /// another setup operation runs.
        pub setup_busy: AtomicBool,
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
                intake_restarts: Mutex::new(Vec::new()),
                source_changes: Mutex::new(Vec::new()),
                setup_busy: AtomicBool::new(false),
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

        fn model_source_changed(
            &self,
            from: ModelSource,
            to: ModelSource,
        ) -> Result<(), CommandError> {
            self.source_changes.lock().unwrap().push((from, to));
            if self.setup_busy.load(Ordering::SeqCst) {
                return Err(Self::injected(
                    "SETUP_BUSY",
                    "a model setup operation is already active",
                ));
            }
            Ok(())
        }

        fn schedule(&self) -> Result<(), CommandError> {
            Ok(())
        }

        fn sync_tray(&self, run_in_background: bool) {
            self.live.lock().unwrap().tray = run_in_background;
        }

        fn restart_intake(
            &self,
            settings: &AppSettings,
            first_look: bool,
        ) -> Result<(), CommandError> {
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
            self.intake_restarts.lock().unwrap().push(first_look);
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
