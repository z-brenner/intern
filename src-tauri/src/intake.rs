//! Shared-intake wiring for the Tauri host: the wire DTOs for the intake
//! commands and the `intake://changed` event, and the pipeline-backed
//! implementation of the intake crate's host boundary.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use intern_core::{OperationDirection, OperationReceipt, OperationStage, QueueItem, QueueStatus};
use intern_engine::fingerprint::NEAR_DUPLICATE_DISTANCE;
use intern_intake::{
    CloudLocation, CloudProviderKind, CloudRoot, DescriptionLedger, DoneOutcome, FiledIndex,
    FiledMarker, IntakeHost, IntakeStatus, ItemState, MachineIdentity, MachinePresence,
    PRESENCE_ACTIVE_WINDOW_SECONDS, classify, detect_cloud_roots,
};
use intern_queue::{
    DuplicateOracle, FiledDocument, FilingSink, KnownFiling, Pipeline, PipelineItem,
    PipelineResult, SettingsStore, SimilarFiling, UnfiledDocument,
    paths::{canonical_file, display_path},
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::SchedulerMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CloudProviderDto {
    #[serde(rename = "onedrive_personal")]
    OneDrivePersonal,
    #[serde(rename = "onedrive_business")]
    OneDriveBusiness,
    #[serde(rename = "sharepoint")]
    SharePoint,
    #[serde(rename = "network_share")]
    NetworkShare,
}

impl From<CloudProviderKind> for CloudProviderDto {
    fn from(kind: CloudProviderKind) -> Self {
        match kind {
            CloudProviderKind::OneDrivePersonal => Self::OneDrivePersonal,
            CloudProviderKind::OneDriveBusiness => Self::OneDriveBusiness,
            CloudProviderKind::SharePoint => Self::SharePoint,
            CloudProviderKind::NetworkShare => Self::NetworkShare,
        }
    }
}

/// One sync root the sync client keeps on this machine, for Settings to
/// offer as a folder to watch or file into.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudRootDto {
    pub provider: CloudProviderDto,
    pub display_name: String,
    pub path: String,
}

impl From<CloudRoot> for CloudRootDto {
    fn from(root: CloudRoot) -> Self {
        Self {
            provider: root.kind.into(),
            display_name: root.display_name,
            path: display_path(&root.root),
        }
    }
}

/// The sync roots detected on this machine, SharePoint libraries first,
/// then OneDrive accounts, each group in path order, so the list reads the
/// same on every open.
pub(crate) fn list_cloud_roots() -> Vec<CloudRootDto> {
    let mut roots = detect_cloud_roots();
    roots.sort_by(|left, right| {
        rank(left.kind)
            .cmp(&rank(right.kind))
            .then_with(|| left.root.cmp(&right.root))
    });
    roots.into_iter().map(Into::into).collect()
}

fn rank(kind: CloudProviderKind) -> u8 {
    match kind {
        CloudProviderKind::SharePoint => 0,
        CloudProviderKind::OneDriveBusiness => 1,
        CloudProviderKind::OneDrivePersonal => 2,
        CloudProviderKind::NetworkShare => 3,
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudLocationDto {
    pub provider: CloudProviderDto,
    pub display_name: String,
}

impl From<CloudLocation> for CloudLocationDto {
    fn from(location: CloudLocation) -> Self {
        Self {
            provider: location.kind.into(),
            display_name: location.display_name,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeMachineDto {
    pub machine_id: String,
    pub machine_name: String,
    pub user_name: String,
    pub last_seen_at: i64,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeStatusDto {
    pub enabled: bool,
    pub watching: bool,
    pub folder: String,
    pub machine_id: String,
    pub machine_name: String,
    pub cloud: Option<CloudLocationDto>,
    pub machines: Vec<IntakeMachineDto>,
    pub held_for_others: u32,
    pub uploader_unknown: u32,
    pub sync_conflicts: u32,
    pub awaiting_hydration: u32,
    pub unreadable_folders: u32,
    pub claimed_by_others: u32,
    pub processed_here: u32,
    pub last_scan_at: Option<i64>,
    pub error: Option<String>,
    /// For a OneDrive or SharePoint folder, whether the OneDrive program is
    /// running, since nothing new arrives while it is not. `None` for any
    /// other folder, or where it cannot be told.
    pub one_drive_running: Option<bool>,
}

pub(crate) fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// A presence stamped in the future (sync-layer clock skew) still counts as
/// active rather than flickering off until the clocks agree.
pub(crate) fn presence_active(last_seen_at: i64, now: i64) -> bool {
    now - last_seen_at <= PRESENCE_ACTIVE_WINDOW_SECONDS
}

/// Roots are detected fresh on every call; the probe is a handful of env and
/// directory reads.
pub(crate) fn classify_folder(path: &str) -> Option<CloudLocationDto> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    classify(Path::new(trimmed), &detect_cloud_roots()).map(Into::into)
}

/// Whether OneDrive is running, asked only about a folder OneDrive keeps.
fn one_drive_running(cloud: Option<&CloudLocationDto>) -> Option<bool> {
    match cloud?.provider {
        CloudProviderDto::NetworkShare => None,
        _ => one_drive_process_listed(),
    }
}

/// Asks Windows' own process list. A paused OneDrive is still running and is
/// not told apart here: its documents simply stay waiting to download, which
/// the awaiting-hydration count already reports.
#[cfg(windows)]
fn one_drive_process_listed() -> Option<bool> {
    use std::os::windows::process::CommandExt;

    /// Keeps a console window from flashing up on every status read.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq OneDrive.exe", "/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| lists_one_drive(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(not(windows))]
fn one_drive_process_listed() -> Option<bool> {
    None
}

/// `tasklist /FO CSV` names each match in quotes first; with no match it
/// prints an informational line instead.
pub(crate) fn lists_one_drive(tasklist: &str) -> bool {
    tasklist.lines().any(|line| {
        line.trim_start()
            .to_ascii_lowercase()
            .starts_with("\"onedrive.exe\"")
    })
}

/// Where setup offers to file documents from `intake`: a "Filed" folder
/// beside it, because the destination may not be inside the watched folder.
pub(crate) fn filed_folder_for(intake: &Path) -> Option<PathBuf> {
    intake.parent().map(|parent| parent.join("Filed"))
}

fn machine_dto(machine: &MachinePresence, now: i64) -> IntakeMachineDto {
    IntakeMachineDto {
        machine_id: machine.machine_id.clone(),
        machine_name: machine.machine_name.clone(),
        user_name: machine.user_name.clone(),
        last_seen_at: machine.last_seen_at,
        active: presence_active(machine.last_seen_at, now),
    }
}

/// Builds the wire status. With no live `watcher` status (intake disabled, or
/// the watcher failed to start) everything scan-derived stays zeroed and
/// `error` carries the recorded startup failure; a live status supplies the
/// folder and counters, and its own error when it has one - otherwise
/// `error` still shows, which is how a filed-index write failure reaches
/// Settings beside a healthy scan.
pub(crate) fn status_dto(
    enabled: bool,
    identity: &MachineIdentity,
    folder: &str,
    watcher: Option<&IntakeStatus>,
    error: Option<String>,
    now: i64,
) -> IntakeStatusDto {
    let base = IntakeStatusDto {
        enabled,
        watching: false,
        folder: display_path(Path::new(folder)),
        machine_id: identity.id.clone(),
        machine_name: identity.name.clone(),
        cloud: classify_folder(folder),
        machines: Vec::new(),
        held_for_others: 0,
        uploader_unknown: 0,
        sync_conflicts: 0,
        awaiting_hydration: 0,
        unreadable_folders: 0,
        claimed_by_others: 0,
        processed_here: 0,
        last_scan_at: None,
        error,
        one_drive_running: None,
    };
    let Some(status) = watcher else {
        return IntakeStatusDto {
            one_drive_running: one_drive_running(base.cloud.as_ref()),
            ..base
        };
    };
    let folder = status.folder.to_string_lossy().into_owned();
    let cloud = classify_folder(&folder);
    IntakeStatusDto {
        watching: status.watching,
        one_drive_running: one_drive_running(cloud.as_ref()),
        cloud,
        folder: display_path(&status.folder),
        machines: status
            .machines
            .iter()
            .map(|machine| machine_dto(machine, now))
            .collect(),
        held_for_others: status.held_for_others,
        uploader_unknown: status.uploader_unknown,
        sync_conflicts: status.sync_conflicts,
        awaiting_hydration: status.awaiting_hydration,
        unreadable_folders: status.unreadable_folders,
        claimed_by_others: status.claimed_by_others,
        processed_here: status.processed_here,
        last_scan_at: status.last_scan_at,
        error: status.error.clone().or(base.error),
        ..base
    }
}

/// Maps a queue item's fate onto the intake claim protocol.
///
/// Completed rule: an apply receipt that reached `Complete` — the same test
/// the queue DTO uses for "undoable" — means the source file was physically
/// renamed out of the intake folder, so it reports `Renamed` with the
/// receipt's destination leaf (falling back to the proposal filename). Any
/// other completed item (keep-original, or an apply that was later undone)
/// left the source in place and reports `KeptOriginal`.
///
/// A canceled item is a person's decision to leave the document alone, so it
/// reports `KeptOriginal` too. One the watcher withdrew itself never reaches
/// here; see `screen_lookup`.
pub(crate) fn item_fate(
    status: QueueStatus,
    receipt: Option<&OperationReceipt>,
    proposal_filename: Option<&str>,
) -> ItemState {
    match status {
        QueueStatus::Queued
        | QueueStatus::Extracting
        | QueueStatus::Analyzing
        | QueueStatus::Ready
        | QueueStatus::Applying => ItemState::Active,
        QueueStatus::NeedsReview => ItemState::NeedsReview,
        QueueStatus::Failed => ItemState::Failed,
        QueueStatus::Canceled => ItemState::Done {
            outcome: DoneOutcome::KeptOriginal,
            result_filename: None,
        },
        QueueStatus::Completed => {
            let applied = receipt.filter(|receipt| {
                receipt.direction == OperationDirection::Apply
                    && receipt.stage == OperationStage::Complete
            });
            match applied {
                Some(receipt) => ItemState::Done {
                    outcome: DoneOutcome::Renamed,
                    result_filename: receipt
                        .destination
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(str::to_owned)
                        .or_else(|| proposal_filename.map(str::to_owned)),
                },
                None => ItemState::Done {
                    outcome: DoneOutcome::KeptOriginal,
                    result_filename: None,
                },
            }
        }
    }
}

/// What a lookup in the queue tells the watcher before the row itself is
/// read. A lookup that failed is `Unavailable`: it says nothing about the
/// document, and reading it as "no item" let the watcher release a claim on
/// a document the queue may still be working on. A row the watcher withdrew
/// itself is `Unknown`, so the claim is released and the document can be
/// handed over again later. Any other row goes on to `item_fate`.
pub(crate) fn screen_lookup(
    lookup: PipelineResult<Option<PipelineItem>>,
    abandoned: &AbandonedItems,
) -> Result<Option<PipelineItem>, ItemState> {
    match lookup {
        Ok(Some(item)) if abandoned.holds(&item) => Err(ItemState::Unknown),
        Ok(found) => Ok(found),
        Err(_) => Err(ItemState::Unavailable),
    }
}

/// The queue rows the watcher canceled itself, by stored path and content.
///
/// A person's cancel and the watcher's `abandon` leave the same canceled row,
/// and they mean opposite things: the first is a decision to leave the
/// document alone, the second only gives up a claim this machine lost, and the
/// document may be handed over again. Kept in memory, so after a restart every
/// canceled row reads as the person's.
#[derive(Default)]
pub(crate) struct AbandonedItems(Mutex<HashSet<(PathBuf, String)>>);

impl AbandonedItems {
    fn holds(&self, item: &PipelineItem) -> bool {
        item.status == QueueStatus::Canceled && self.contains(&item.source_path, &item.source_hash)
    }

    fn contains(&self, source_path: &Path, source_hash: &str) -> bool {
        self.0
            .lock()
            .is_ok_and(|rows| rows.contains(&(source_path.to_path_buf(), source_hash.to_owned())))
    }

    /// Cancels the pending item at `path` and remembers that the watcher did
    /// it. Best effort: a terminal or mid-apply item is not cancelable and the
    /// pipeline says so, and the claim protocol only needs pending work
    /// withdrawn. A row already canceled was canceled by a person.
    fn abandon(&self, pipeline: &Pipeline, path: &Path) {
        let Ok(Some(item)) = pipeline.find_by_source_path(path) else {
            return;
        };
        if item.status != QueueStatus::Canceled
            && pipeline.cancel(item.id).is_ok()
            && let Ok(mut rows) = self.0.lock()
        {
            rows.insert((item.source_path, item.source_hash));
        }
    }

    /// Queues again every row in `items` the watcher abandoned. The queue
    /// keeps one row per path and content, so handing an abandoned document
    /// over again returns its canceled row, and without this re-admission
    /// would never run it again. A row is forgotten only once it is queued, so
    /// a refusal here leaves it the watcher's to try again.
    fn requeue(&self, pipeline: &Pipeline, items: &[QueueItem]) -> PipelineResult<()> {
        for item in items {
            if item.status != QueueStatus::Canceled
                || !self.contains(&item.source_path, &item.source_hash)
            {
                continue;
            }
            pipeline.retry(item.id)?;
            if let Ok(mut rows) = self.0.lock() {
                rows.remove(&(item.source_path.clone(), item.source_hash.clone()));
            }
        }
        Ok(())
    }
}

/// Runs the failed item at `path` again in place. False when there is no
/// failed item there, or the queue refused.
fn retry_failed(pipeline: &Pipeline, path: &Path) -> bool {
    match pipeline.find_by_source_path(path) {
        Ok(Some(item)) if item.status == QueueStatus::Failed => pipeline.retry(item.id).is_ok(),
        _ => false,
    }
}

/// The intake crate's view of the app: documents go into the existing
/// pipeline, and status changes become `intake://changed` events.
pub(crate) struct PipelineIntakeHost {
    pipeline: Arc<Pipeline>,
    scheduler: Sender<SchedulerMessage>,
    model_ready: Arc<AtomicBool>,
    app: AppHandle,
    identity: MachineIdentity,
    filed_index: Arc<SharedFiledIndex>,
    abandoned: AbandonedItems,
}

impl PipelineIntakeHost {
    pub(crate) fn new(
        pipeline: Arc<Pipeline>,
        scheduler: Sender<SchedulerMessage>,
        model_ready: Arc<AtomicBool>,
        app: AppHandle,
        identity: MachineIdentity,
        filed_index: Arc<SharedFiledIndex>,
    ) -> Self {
        Self {
            pipeline,
            scheduler,
            model_ready,
            app,
            identity,
            filed_index,
            abandoned: AbandonedItems::default(),
        }
    }

    /// The queue stores the canonical path handed to `enqueue`, and the
    /// watcher's paths derive from the canonical intake root, so a live file
    /// matches either literally or after canonicalization. A path that no
    /// longer canonicalizes (the apply already renamed it away) still matches
    /// literally — that is what lets a finished item report `Done` instead of
    /// `Unknown`. The newest matching item is the source's current fate;
    /// older completed rows for the same path are history. One indexed
    /// lookup per file: this is asked once per document on every scan.
    fn find_item(&self, path: &Path) -> Result<Option<PipelineItem>, ItemState> {
        screen_lookup(self.pipeline.find_by_source_path(path), &self.abandoned)
    }

    /// The scheduler ignores wakes until the model is ready, and setup
    /// sends its own wake on becoming ready, so gating here only avoids a
    /// pointless message — mirroring AppState::schedule.
    fn wake_scheduler(&self) {
        if self.model_ready.load(Ordering::SeqCst) {
            let _ = self.scheduler.send(SchedulerMessage::Wake);
        }
    }
}

impl IntakeHost for PipelineIntakeHost {
    fn admission(&self, path: &Path) -> intern_intake::IntakeAdmission {
        use intern_queue::{AdmissionGuard, AdmissionStage};
        let manager = self
            .app
            .state::<Arc<crate::microsoft_intake::MicrosoftIntake>>();
        intake_admission(manager.authorize(path, AdmissionStage::Enqueue))
    }
    fn enqueue(&self, paths: &[PathBuf]) -> Result<(), String> {
        let mut canonical = Vec::with_capacity(paths.len());
        for path in paths {
            canonical.push(canonical_file(path).map_err(|error| error.code)?);
        }
        let items = self
            .pipeline
            .enqueue_files(&canonical)
            .map_err(|error| error.code)?;
        self.abandoned
            .requeue(&self.pipeline, &items)
            .map_err(|error| error.code)?;
        self.wake_scheduler();
        Ok(())
    }

    fn item_state(&self, path: &Path) -> ItemState {
        match self.find_item(path) {
            Ok(Some(item)) => item_fate(
                item.status,
                item.receipt.as_ref(),
                item.proposal
                    .as_ref()
                    .map(|record| record.filename.as_str()),
            ),
            Ok(None) => ItemState::Unknown,
            Err(state) => state,
        }
    }

    fn abandon(&self, path: &Path) {
        self.abandoned.abandon(&self.pipeline, path);
    }

    fn retry(&self, path: &Path) -> bool {
        let retried = retry_failed(&self.pipeline, path);
        if retried {
            self.wake_scheduler();
        }
        retried
    }

    fn status_changed(&self, status: &IntakeStatus) {
        let dto = status_dto(
            true,
            &self.identity,
            &status.folder.to_string_lossy(),
            Some(status),
            self.filed_index.last_error(),
            now_unix(),
        );
        let _ = self.app.emit("intake://changed", dto);
    }
}

fn intake_admission(
    result: intern_queue::PipelineResult<intern_queue::AdmissionEvidence>,
) -> intern_intake::IntakeAdmission {
    match result {
        Ok(evidence) if evidence.verified_hash().is_some() => {
            intern_intake::IntakeAdmission::Verified
        }
        Ok(_) => intern_intake::IntakeAdmission::LocalOnly,
        Err(error) if error.is_retryable() => intern_intake::IntakeAdmission::Retryable,
        Err(error) if error.code == "UPLOADER_OTHER" => intern_intake::IntakeAdmission::Other,
        Err(_) => intern_intake::IntakeAdmission::Unknown,
    }
}

/// The shared filed index, as the queue sees it: the filing sink that leaves
/// a marker for every document filed out of the watched intake folder, and
/// the duplicate oracle that reads the markers teammates left there.
///
/// Reads the settings on every call, like the ledger, so a newly watched
/// folder is indexed from its first filing. Only documents that came from
/// the intake folder are recorded: a document filed from anywhere else is
/// nobody else's business, and the shared folder should not learn its name.
/// Failures never reach the rename that caused them; the last one is kept
/// for the intake status.
pub(crate) struct SharedFiledIndex {
    settings: SettingsStore,
    data_dir: PathBuf,
    last_error: Mutex<Option<String>>,
}

impl SharedFiledIndex {
    pub(crate) fn new(settings: SettingsStore, data_dir: PathBuf) -> Self {
        Self {
            settings,
            data_dir,
            last_error: Mutex::new(None),
        }
    }

    /// The index for the current settings, or `None` when no intake folder
    /// is being watched.
    fn index(&self) -> Option<FiledIndex> {
        let settings = self.settings.load().ok()?;
        let folder = settings.intake_folder.trim();
        if !settings.intake_enabled || folder.is_empty() {
            return None;
        }
        let identity =
            MachineIdentity::load_or_create(&self.data_dir, &settings.machine_label).ok()?;
        Some(FiledIndex::new(PathBuf::from(folder), identity))
    }

    /// The last marker write or retraction that failed, as `CODE: detail`,
    /// until the next success.
    pub(crate) fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|error| error.clone())
    }

    fn note(&self, outcome: Result<(), String>) {
        if let Ok(mut slot) = self.last_error.lock() {
            *slot = outcome.err();
        }
    }
}

impl FilingSink for SharedFiledIndex {
    fn filed(&self, document: &FiledDocument) {
        let Some(index) = self.index() else {
            return;
        };
        if !index.covers(&document.source_path) {
            return;
        }
        let filename = document
            .destination
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.note(
            index
                .record(
                    &document.source_hash,
                    &document.source_path,
                    &filename,
                    document.filed_at,
                    document.text_fingerprint.as_deref(),
                )
                .map(drop)
                .map_err(|error| format!("FILED_INDEX_WRITE_FAILED: {error}")),
        );
    }

    fn unfiled(&self, document: &UnfiledDocument) {
        let Some(index) = self.index() else {
            return;
        };
        if !index.covers(&document.source_path) {
            return;
        }
        self.note(
            index
                .retract(&document.source_hash)
                .map(drop)
                .map_err(|error| format!("FILED_INDEX_RETRACT_FAILED: {error}")),
        );
    }
}

impl DuplicateOracle for SharedFiledIndex {
    fn filed_elsewhere(&self, source_hash: &str, source_path: &Path) -> Option<KnownFiling> {
        let index = self.index()?;
        let marker = index.lookup(source_hash)?;
        known_filing(
            &marker,
            &index.identity().id,
            index.relative_path(source_path).as_deref(),
        )
    }

    /// The closest marker by text fingerprint from any machine. Whether the
    /// closeness means one document is the queue's call, made with the
    /// dates; the index only says how close.
    fn similar_elsewhere(&self, fingerprint: u64) -> Option<SimilarFiling> {
        let index = self.index()?;
        let (marker, distance) = index.lookup_similar(fingerprint, NEAR_DUPLICATE_DISTANCE)?;
        let filing = known_filing(&marker, &index.identity().id, None)?;
        Some(SimilarFiling { filing, distance })
    }
}

/// What a marker means for a document being enqueued at `relative_path`
/// (relative to the intake folder; `None` when it comes from elsewhere).
///
/// A marker is a duplicate's referent unless it is this machine's own record
/// of the very document being enqueued again - an undone filing whose
/// retraction has not reached the folder, which is the document itself, not
/// a copy of it. A filing this machine made from another path is still a
/// duplicate, but there is no other machine to name.
///
/// The names come from a file anyone who can write the shared folder can
/// write, and they end up in a review reason, so they are clamped to the
/// length of a real filename.
pub(crate) fn known_filing(
    marker: &FiledMarker,
    own_machine_id: &str,
    relative_path: Option<&str>,
) -> Option<KnownFiling> {
    let own = marker.machine_id == own_machine_id;
    if own
        && relative_path
            .is_some_and(|path| path.to_lowercase() == marker.relative_path.to_lowercase())
    {
        return None;
    }
    Some(KnownFiling {
        filename: clamp_marker_text(&marker.filename),
        filed_by: (!own).then(|| clamp_marker_text(&marker.machine_name)),
    })
}

/// The longest marker string shown to a person: a filename's limit.
const MARKER_TEXT_LIMIT: usize = 255;

fn clamp_marker_text(text: &str) -> String {
    text.chars().take(MARKER_TEXT_LIMIT).collect()
}

/// What the description records are doing, for Settings.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptionsStatusDto {
    /// The setting, as saved.
    pub enabled: bool,
    /// Where records go: `<destination>/.intern/descriptions`, or empty when
    /// no destination is configured.
    pub folder: String,
    /// Records written since Intern started.
    pub recorded_this_session: u32,
    pub last_recorded_at: Option<i64>,
    /// The last write that failed, as `CODE: detail`, until the next success.
    pub last_error: Option<String>,
}

#[derive(Debug, Default)]
struct LedgerCounters {
    recorded: u32,
    last_recorded_at: Option<i64>,
    last_error: Option<String>,
}

/// The queue's filing sink: writes a description record for every completed
/// rename into the destination folder's ledger, and removes it again when the
/// rename is undone. Reads the settings on every call, so switching the
/// feature on or changing the destination takes effect at the next rename
/// without restarting anything. Failures never reach the rename that caused
/// them; they are kept here for Settings to show.
pub(crate) struct LedgerSink {
    settings: SettingsStore,
    data_dir: PathBuf,
    app: Mutex<Option<AppHandle>>,
    counters: Mutex<LedgerCounters>,
}

impl LedgerSink {
    pub(crate) fn new(settings: SettingsStore, data_dir: PathBuf) -> Self {
        Self {
            settings,
            data_dir,
            app: Mutex::new(None),
            counters: Mutex::new(LedgerCounters::default()),
        }
    }

    /// Attaches the app so record writes can announce themselves as
    /// `descriptions://changed` events; before this, they are silent.
    pub(crate) fn attach(&self, app: AppHandle) {
        if let Ok(mut slot) = self.app.lock() {
            *slot = Some(app);
        }
    }

    /// The ledger for the current settings, or `None` when records are
    /// switched off or there is no destination to keep them in.
    fn ledger(&self) -> Option<DescriptionLedger> {
        let settings = self.settings.load().ok()?;
        let destination = settings.destination.trim();
        if !settings.record_descriptions || destination.is_empty() {
            return None;
        }
        let identity =
            MachineIdentity::load_or_create(&self.data_dir, &settings.machine_label).ok()?;
        Some(DescriptionLedger::new(
            PathBuf::from(destination),
            identity,
            detect_cloud_roots(),
        ))
    }

    pub(crate) fn status(&self) -> DescriptionsStatusDto {
        let settings = self.settings.load().unwrap_or_default();
        let destination = settings.destination.trim();
        let folder = if destination.is_empty() {
            String::new()
        } else {
            display_path(&DescriptionLedger::directory_under(Path::new(destination)))
        };
        let counters = self
            .counters
            .lock()
            .map(|counters| LedgerCounters {
                recorded: counters.recorded,
                last_recorded_at: counters.last_recorded_at,
                last_error: counters.last_error.clone(),
            })
            .unwrap_or_default();
        DescriptionsStatusDto {
            enabled: settings.record_descriptions,
            folder,
            recorded_this_session: counters.recorded,
            last_recorded_at: counters.last_recorded_at,
            last_error: counters.last_error,
        }
    }

    /// Writes records for every document the queue has filed and not undone.
    /// Returns how many were written and how many failed; the last failure
    /// is kept for Settings.
    pub(crate) fn backfill(&self, documents: &[FiledDocument]) -> (u32, u32) {
        let Some(ledger) = self.ledger() else {
            return (0, 0);
        };
        let mut written = 0;
        let mut failed = 0;
        for document in documents {
            match ledger.record(&record_of(document)) {
                Ok(_) => {
                    written += 1;
                    self.note_success();
                }
                Err(error) => {
                    failed += 1;
                    self.note_failure(format!("DESCRIPTION_WRITE_FAILED: {error}"));
                }
            }
        }
        self.announce();
        (written, failed)
    }

    fn note_success(&self) {
        if let Ok(mut counters) = self.counters.lock() {
            counters.recorded += 1;
            counters.last_recorded_at = Some(now_unix());
            counters.last_error = None;
        }
    }

    fn note_failure(&self, error: String) {
        if let Ok(mut counters) = self.counters.lock() {
            counters.last_error = Some(error);
        }
    }

    fn announce(&self) {
        if let Ok(app) = self.app.lock()
            && let Some(app) = app.as_ref()
        {
            let _ = app.emit("descriptions://changed", self.status());
        }
    }
}

fn record_of(document: &FiledDocument) -> intern_intake::FiledDocument {
    intern_intake::FiledDocument {
        path: document.destination.clone(),
        original_filename: document
            .source_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        description: document.description.clone(),
        document_date: document.proposal.document_date.clone(),
        document_type: document.proposal.document_type.clone(),
        parties: document.proposal.parties.clone(),
        confidence: Some(document.proposal.confidence),
        filed_at: document.filed_at,
    }
}

impl FilingSink for LedgerSink {
    fn filed(&self, document: &FiledDocument) {
        let Some(ledger) = self.ledger() else {
            return;
        };
        match ledger.record(&record_of(document)) {
            Ok(_) => self.note_success(),
            Err(error) => self.note_failure(format!("DESCRIPTION_WRITE_FAILED: {error}")),
        }
        self.announce();
    }

    fn unfiled(&self, document: &UnfiledDocument) {
        let Some(ledger) = self.ledger() else {
            return;
        };
        if let Err(error) = ledger.retract(&document.destination) {
            self.note_failure(format!("DESCRIPTION_RETRACT_FAILED: {error}"));
        }
        self.announce();
    }
}

#[cfg(test)]
mod filed_index_tests {
    use intern_intake::FiledMarker;

    use super::known_filing;

    #[test]
    fn retryable_pipeline_admission_reaches_the_watcher_without_collapsing() {
        let result = Err(intern_queue::PipelineError::retryable(
            "UPLOADER_UNVERIFIED",
            "Microsoft Graph is temporarily unavailable.",
        ));

        assert_eq!(
            super::intake_admission(result),
            intern_intake::IntakeAdmission::Retryable
        );
    }

    fn marker(machine_id: &str) -> FiledMarker {
        FiledMarker {
            version: 1,
            content_hash: "0".repeat(64),
            filename: "2026-03-02 Agreement.pdf".into(),
            relative_path: "Scans/scan0012.pdf".into(),
            machine_id: machine_id.into(),
            machine_name: "Front desk".into(),
            user_name: "pat".into(),
            filed_at: 1,
            text_fingerprint: None,
        }
    }

    #[test]
    fn a_teammates_marker_names_the_filing_and_their_machine() {
        let known = known_filing(&marker("aaa"), "bbb", Some("Scans/scan0012.pdf")).unwrap();
        assert_eq!(known.filename, "2026-03-02 Agreement.pdf");
        assert_eq!(known.filed_by.as_deref(), Some("Front desk"));
        // From outside the intake folder, the same answer.
        assert_eq!(known_filing(&marker("aaa"), "bbb", None), Some(known));
    }

    /// A marker is a file anyone who can write the shared folder can write,
    /// and its names go straight into a review reason.
    #[test]
    fn a_teammates_marker_names_are_clamped_to_a_filenames_length() {
        let mut planted = marker("aaa");
        planted.filename = "x".repeat(10_000);
        planted.machine_name = "é".repeat(10_000);
        let known = known_filing(&planted, "bbb", None).unwrap();
        assert_eq!(known.filename.chars().count(), 255);
        assert_eq!(known.filed_by.unwrap().chars().count(), 255);
        // An ordinary marker is untouched.
        let ordinary = known_filing(&marker("aaa"), "bbb", None).unwrap();
        assert_eq!(ordinary.filename, "2026-03-02 Agreement.pdf");
    }

    #[test]
    fn this_machines_own_marker_for_the_same_document_is_the_document_itself() {
        assert_eq!(
            known_filing(&marker("aaa"), "aaa", Some("scans/SCAN0012.pdf")),
            None,
            "an undone filing whose retraction did not stick, spelled however"
        );
        // The same content from another path is a duplicate, with no other
        // machine to name.
        let elsewhere = known_filing(&marker("aaa"), "aaa", Some("Inbox/copy.pdf")).unwrap();
        assert_eq!(elsewhere.filed_by, None);
        assert_eq!(elsewhere.filename, "2026-03-02 Agreement.pdf");
        assert!(known_filing(&marker("aaa"), "aaa", None).is_some());
    }
}

#[cfg(test)]
mod watcher_host_tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::Arc,
    };

    use intern_core::QueueStatus;
    use intern_intake::{DoneOutcome, ItemState};
    use intern_queue::{
        AnalyzerBoundary, ModelFailure, Pipeline, PipelineError, PipelineEventSink,
        PipelineProgress, SettingsStore, WorkerBoundary, WorkerFailure,
    };

    use super::{AbandonedItems, item_fate, retry_failed, screen_lookup};

    /// An extractor that can read nothing, so every document fails.
    struct Unreadable;

    impl WorkerBoundary for Unreadable {
        fn extract(
            &self,
            _request_id: &str,
            _path: &Path,
            _progress: &mut dyn FnMut(intern_engine::ExtractProgress),
        ) -> Result<intern_engine::DocumentSource, WorkerFailure> {
            Err(WorkerFailure::new("TEST_UNREADABLE", false, false))
        }

        fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
            Ok(())
        }

        fn restart(&self) -> Result<(), WorkerFailure> {
            Ok(())
        }
    }

    struct NeverAnalyze;

    impl AnalyzerBoundary for NeverAnalyze {
        fn analyze(
            &self,
            _source: &intern_engine::DocumentSource,
            _extension: &str,
            _existing_names: &[&str],
        ) -> Result<intern_engine::DocumentAnalysis, ModelFailure> {
            panic!("no document gets past extraction here")
        }
    }

    struct NoEvents;

    impl PipelineEventSink for NoEvents {
        fn queue_changed(&self) {}

        fn progress(&self, _progress: PipelineProgress) {}
    }

    /// A real queue in a directory of its own, with the documents beside it.
    fn queue(name: &str) -> (PathBuf, Pipeline) {
        let dir =
            std::env::temp_dir().join(format!("intern-watcher-host-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dir = fs::canonicalize(dir).unwrap();
        let pipeline = Pipeline::with_local_files(
            dir.join("queue.sqlite3"),
            Arc::new(Unreadable),
            Arc::new(NeverAnalyze),
            Arc::new(NoEvents),
            SettingsStore::new(dir.join("settings.json")),
        )
        .unwrap();
        (dir, pipeline)
    }

    /// What the host tells the watcher about `path`, read the way
    /// `PipelineIntakeHost::item_state` reads it.
    fn fate(pipeline: &Pipeline, abandoned: &AbandonedItems, path: &Path) -> ItemState {
        match screen_lookup(pipeline.find_by_source_path(path), abandoned) {
            Ok(Some(item)) => item_fate(item.status, item.receipt.as_ref(), None),
            Ok(None) => ItemState::Unknown,
            Err(state) => state,
        }
    }

    /// A person's Cancel and the watcher's own withdrawal leave the same
    /// canceled row. Read as "no item", a person's cancel had the watcher
    /// release and re-claim the document every two scans for ever; read as a
    /// decision, the watcher's withdrawal would strand a document it only
    /// let go of.
    #[test]
    fn user_canceled_maps_to_kept_original_abandoned_maps_to_unknown() {
        let (dir, pipeline) = queue("cancel");
        let abandoned = AbandonedItems::default();
        let canceled = dir.join("canceled.pdf");
        let withdrawn = dir.join("withdrawn.pdf");
        fs::write(&canceled, b"a document the person canceled").unwrap();
        fs::write(&withdrawn, b"a document the watcher let go of").unwrap();
        let items = pipeline
            .enqueue_files(&[canceled.clone(), withdrawn.clone()])
            .unwrap();
        let kept = ItemState::Done {
            outcome: DoneOutcome::KeptOriginal,
            result_filename: None,
        };

        pipeline.cancel(items[0].id).unwrap();
        assert_eq!(fate(&pipeline, &abandoned, &canceled), kept);

        abandoned.abandon(&pipeline, &withdrawn);
        let row = pipeline.find_by_source_path(&withdrawn).unwrap().unwrap();
        assert_eq!(row.status, QueueStatus::Canceled);
        assert_eq!(fate(&pipeline, &abandoned, &withdrawn), ItemState::Unknown);
        // A row a person already canceled is never taken for the watcher's.
        abandoned.abandon(&pipeline, &canceled);
        assert_eq!(fate(&pipeline, &abandoned, &canceled), kept);

        // The queue hands back both canceled rows; only the one the watcher
        // let go of starts again.
        let again = pipeline
            .enqueue_files(&[canceled.clone(), withdrawn.clone()])
            .unwrap();
        assert!(
            again
                .iter()
                .all(|item| item.status == QueueStatus::Canceled)
        );
        abandoned.requeue(&pipeline, &again).unwrap();
        assert_eq!(fate(&pipeline, &abandoned, &withdrawn), ItemState::Active);
        assert_eq!(fate(&pipeline, &abandoned, &canceled), kept);

        drop(pipeline);
        let _ = fs::remove_dir_all(dir);
    }

    /// A lookup the database refused says nothing about the document. Read as
    /// "no item", it had the watcher release a claim on work in progress.
    #[test]
    fn db_error_maps_to_unavailable() {
        let abandoned = AbandonedItems::default();
        let refused = Err(PipelineError::new(
            "DATABASE_UNAVAILABLE",
            "database is locked",
        ));
        assert_eq!(
            screen_lookup(refused, &abandoned),
            Err(ItemState::Unavailable)
        );
        assert_eq!(screen_lookup(Ok(None), &abandoned), Ok(None));
    }

    /// The forgive-once retry for a document that failed without its content
    /// has to run the failed row again: handing the same document over again
    /// only returns that row, still failed.
    #[test]
    fn a_failed_item_is_retried_in_place() {
        let (dir, pipeline) = queue("retry");
        let document = dir.join("contract.pdf");
        fs::write(&document, b"a document that could not be read").unwrap();
        pipeline
            .enqueue_files(std::slice::from_ref(&document))
            .unwrap();
        pipeline.run_next().unwrap();
        pipeline.run_next().unwrap();
        let status = |path: &Path| pipeline.find_by_source_path(path).unwrap().unwrap().status;
        assert_eq!(status(&document), QueueStatus::Failed);
        let again = pipeline
            .enqueue_files(std::slice::from_ref(&document))
            .unwrap();
        assert_eq!(
            again[0].status,
            QueueStatus::Failed,
            "the row comes back as it was"
        );

        assert!(retry_failed(&pipeline, &document));
        assert_eq!(status(&document), QueueStatus::Queued);
        assert!(
            !retry_failed(&pipeline, &document),
            "nothing failed to retry"
        );
        assert!(!retry_failed(&pipeline, &dir.join("never-enqueued.pdf")));

        drop(pipeline);
        let _ = fs::remove_dir_all(dir);
    }
}
