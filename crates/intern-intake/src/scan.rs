//! Scan-side types: watcher configuration, the host boundary, status, and the
//! filesystem walk with its skip and stability rules.

use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

use crate::{
    cloud::{self, CloudRoot},
    coordination::{DoneOutcome, MachinePresence},
};

pub const DEFAULT_SCAN_INTERVAL: Duration = Duration::from_secs(20);
/// How soon the loop looks again while a file is still arriving.
pub const SETTLING_SCAN_INTERVAL: Duration = Duration::from_secs(3);
/// How long a file's size and modification time must hold still before it is
/// claimed: half the default interval, so ordinary pickup still takes two
/// scans, but no longer two scans that happen to land a moment apart.
pub const DEFAULT_MIN_QUIET_SECONDS: i64 = 10;
/// How often, between scans, a folder on this computer's own disk is listed
/// for files that have just turned up ([`IntakeConfig::arrival_probe_interval`]).
pub const ARRIVAL_PROBE_INTERVAL: Duration = Duration::from_secs(3);
/// How long after a listing shows something new the folder is listed once
/// more before it is scanned - and again for as long as further files keep
/// turning up - so that files dropped together are first seen by one scan,
/// and so claimed by one, as they are when a scan happens upon them.
pub const ARRIVAL_BURST_WINDOW: Duration = Duration::from_secs(1);
/// The look between scans waits at least this many times as long as its
/// last listing of the folder took (`next_look_after`), so that listing
/// the folder never takes more than about one part in this many of the
/// time, however big or slow the folder is. A folder whose listing takes
/// more than 30 ms is looked at less often than [`ARRIVAL_PROBE_INTERVAL`],
/// and one whose listing takes a fifth of a second no more often than it is
/// scanned: not between scans at all.
pub const ARRIVAL_PROBE_LOAD: u32 = 100;

#[derive(Clone, Debug)]
pub struct IntakeConfig {
    pub intake_root: PathBuf,
    /// Lowercase extensions without the dot — the host passes intern-queue's
    /// supported list.
    pub extensions: Vec<String>,
    pub process_others_uploads: bool,
    /// Injectable so tests can drive the loop without real waits.
    pub scan_interval: Duration,
    /// The interval while any file is still arriving. Injectable for the same
    /// reason as `scan_interval`.
    pub settling_interval: Duration,
    /// Seconds, on the injected clock, that a file must go unchanged before it
    /// is stable. See `StabilityTracker`.
    pub min_quiet_seconds: i64,
    /// Where the record of what was already in the folder when this machine
    /// first watched it is kept, so a restart does not retake it. The host
    /// puts it in its own data directory, never in the shared `.intern`
    /// folder. `None` keeps it in memory, retaken by every new watcher.
    pub backlog_file: Option<PathBuf>,
    /// Takes that record again, whatever was kept: the person has just
    /// started watching the folder - turned watching on, or chose how its
    /// documents are admitted - so what is in it now is what they chose to
    /// leave alone. A restart or a changed label leaves this false.
    pub retake_backlog: bool,
    /// How often the folder's listings are read between scans, while nothing
    /// is arriving, to notice a file that has just turned up.
    ///
    /// A file's quiet time starts when a scan first sees it, and with
    /// nothing arriving the next scan is up to `scan_interval` away: a
    /// dropped document waited ten seconds on average, and up to twenty,
    /// before its ten seconds of quiet even began. This look only reads
    /// directory listings - names, and the size and time each listing gives -
    /// never a file, the `.intern` folder, or the host, and when it finds
    /// nothing new that is all that happens. When it finds a file it has not
    /// seen, or one whose size or time changed, the next scan runs now
    /// rather than at the end of the interval. That scan is an ordinary one:
    /// it starts the same quiet time, and nothing is claimed any sooner
    /// after it was first seen than before.
    ///
    /// `None`, the default, never looks. The host turns it on with
    /// [`Self::probe_arrivals_if_local`].
    pub arrival_probe_interval: Option<Duration>,
    /// See [`ARRIVAL_BURST_WINDOW`]. Injectable like the intervals.
    pub arrival_burst_window: Duration,
}

impl IntakeConfig {
    pub fn new(intake_root: impl Into<PathBuf>, extensions: Vec<String>) -> Self {
        Self {
            intake_root: intake_root.into(),
            extensions,
            process_others_uploads: false,
            scan_interval: DEFAULT_SCAN_INTERVAL,
            settling_interval: SETTLING_SCAN_INTERVAL,
            min_quiet_seconds: DEFAULT_MIN_QUIET_SECONDS,
            backlog_file: None,
            retake_backlog: false,
            arrival_probe_interval: None,
            arrival_burst_window: ARRIVAL_BURST_WINDOW,
        }
    }

    /// Turns the look for arrivals between scans on
    /// ([`Self::arrival_probe_interval`]) when the folder is on this
    /// computer's own disk, and off when it is inside one of `sync_roots` -
    /// a OneDrive or SharePoint folder - or on a network share
    /// ([`cloud::classify`]).
    ///
    /// Only there, because only there is seeing a document sooner known to
    /// cost nothing. In a shared library the first question a settled
    /// document raises is who uploaded it, which is a request to Microsoft
    /// that an upload's metadata may not have caught up with yet; asked
    /// sooner it is more often asked twice. And on a share every listing is
    /// a round trip to a server that other machines are listing too. Those
    /// folders are scanned exactly as before.
    pub fn probe_arrivals_if_local(&mut self, sync_roots: &[CloudRoot]) {
        self.arrival_probe_interval = cloud::classify(&self.intake_root, sync_roots)
            .is_none()
            .then_some(ARRIVAL_PROBE_INTERVAL);
    }

    /// How long the loop waits before its next scan, given how many files the
    /// last one found still arriving.
    ///
    /// A file is only claimed once it has been quiet for `min_quiet_seconds`,
    /// and only a scan can notice that, so waiting out a whole `scan_interval`
    /// after a file goes quiet added up to 20 seconds to every pickup. While
    /// something is arriving the loop looks again every `settling_interval`
    /// instead - never less often than `scan_interval` - and a dropped document
    /// is claimed within a few seconds of settling.
    pub fn next_interval(&self, settling: usize) -> Duration {
        if settling == 0 {
            self.scan_interval
        } else {
            self.settling_interval.min(self.scan_interval)
        }
    }
}

/// What the host queue knows about a document it was handed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ItemState {
    /// The queue has no live item for the path: none was ever made, the
    /// person removed it, or the watcher's own `abandon` withdrew it.
    Unknown,
    Active,
    NeedsReview,
    Done {
        outcome: DoneOutcome,
        result_filename: Option<String>,
    },
    Failed,
    /// The queue could not be asked - its database refused the lookup. Not a
    /// verdict about the document, so the claim is only kept alive.
    Unavailable,
}

/// The boundary to whatever processes documents (the pipeline in the real
/// app, a fake in tests). The watcher only hands over paths and asks about
/// their fate; it never reads document content itself.
///
/// `Retryable`, `Unknown`, and `Revoked` are all holds and none admits a
/// document. `Retryable` means no verdict was available right now and leaves
/// owned work exactly where it is. `Unknown` is a durable unknown-uploader
/// verdict. `Revoked` means the connection, account, or file proof changed and
/// cancels work already in flight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntakeAdmission {
    LocalOnly,
    Verified,
    Other,
    Retryable,
    Unknown,
    Revoked,
}

pub trait IntakeHost: Send + Sync {
    /// The default is a hold. File arrival and origin markers are not proof of
    /// a Microsoft uploader. A host must explicitly authorize the source.
    fn admission(&self, _path: &Path) -> IntakeAdmission {
        IntakeAdmission::Unknown
    }
    /// Hands documents to the queue. Handing over a path and content the
    /// queue already has must leave that item as it is, whatever state it is
    /// in, rather than starting it again: the real queue keeps one row per
    /// path and content.
    fn enqueue(&self, paths: &[PathBuf]) -> Result<(), String>;
    fn item_state(&self, path: &Path) -> ItemState;
    /// The claim was lost to a takeover or sync conflict: cancel/remove the
    /// local item if it is still pending. A withdrawn item reports `Unknown`,
    /// and handing the same document over again later starts it again.
    fn abandon(&self, path: &Path);
    /// Runs a `Failed` item again in place. True when it was failed and is
    /// queued again; false when there was nothing to retry, or the host
    /// cannot, in which case the watcher releases the claim instead.
    fn retry(&self, _path: &Path) -> bool {
        false
    }
    fn status_changed(&self, status: &IntakeStatus);

    /// [`Self::enqueue`] for one document a claim keyed `key` holds, so the
    /// host can tie the claim to the item it hands over. Everything the
    /// watcher later asks about that claim goes through the methods below,
    /// and a host that keeps the tie answers for that item, not for
    /// whatever is newest at the path: the path can come to hold another
    /// document, and the document can leave the path - renamed
    /// `NEEDS REVIEW - ...` where it lies - while its claim stays open.
    fn enqueue_claimed(&self, _key: &str, path: &Path) -> Result<(), String> {
        self.enqueue(&[path.to_path_buf()])
    }
    /// Ties a claim keyed `key` to the item the queue now holds for `path`,
    /// for a document handed over outside a scan - a copy of a file dropped
    /// on the window (`IntakeHandle::hand_over`) - whose enqueue the caller
    /// made itself. Afterwards the claim answers for that item exactly as a
    /// claim a scan handed over through [`Self::enqueue_claimed`] does. A
    /// host that keeps no ties does nothing.
    fn tie_claim(&self, _key: &str, _path: &Path) {}
    /// [`Self::item_state`] for the item a claim handed over.
    fn claim_state(&self, _key: &str, path: &Path) -> ItemState {
        self.item_state(path)
    }
    /// [`Self::abandon`] for the item a claim handed over.
    fn abandon_claim(&self, _key: &str, path: &Path) {
        self.abandon(path);
    }
    /// [`Self::retry`] for the item a claim handed over.
    fn retry_claim(&self, _key: &str, path: &Path) -> bool {
        self.retry(path)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntakeStatus {
    pub watching: bool,
    pub folder: PathBuf,
    pub last_scan_at: Option<i64>,
    /// Eligible-extension files we are NOT claiming (scope/ownership rules).
    pub held_for_others: u32,
    pub uploader_unknown: u32,
    /// Files skipped because their name is a sync client's conflict copy.
    pub sync_conflicts: u32,
    /// Documents waiting on their content to come down from the cloud: claims
    /// held open after a failure, and placeholders the queue could not read
    /// while offline.
    pub awaiting_hydration: u32,
    /// Settled documents the queue could not take - a file this account may
    /// not read, or one the queue refuses to open - tried again with growing
    /// pauses rather than on every scan.
    pub unreadable_documents: u32,
    /// Subfolders the last scan could not read - a permission the share does
    /// not grant, or a folder that vanished mid-scan. The rest of the folder
    /// is still scanned; these are counted so a person can see them.
    pub unreadable_folders: u32,
    /// Files that turned up or changed while being watched and have not yet
    /// held still long enough to be claimed.
    pub arriving: u32,
    pub claimed_by_others: u32,
    /// Done-by-us claims seen.
    pub processed_here: u32,
    pub machines: Vec<MachinePresence>,
    /// Last scan error code/message.
    pub error: Option<String>,
}

impl IntakeStatus {
    /// A watcher's status before its first scan has counted anything.
    pub fn idle(folder: PathBuf) -> Self {
        Self {
            watching: true,
            folder,
            last_scan_at: None,
            held_for_others: 0,
            uploader_unknown: 0,
            sync_conflicts: 0,
            awaiting_hydration: 0,
            unreadable_documents: 0,
            unreadable_folders: 0,
            arriving: 0,
            claimed_by_others: 0,
            processed_here: 0,
            machines: Vec::new(),
            error: None,
        }
    }

    /// `last_scan_at` advances on every tick; a change there alone is not
    /// worth waking the host for.
    pub(crate) fn materially_differs(&self, other: &Self) -> bool {
        self.watching != other.watching
            || self.folder != other.folder
            || self.held_for_others != other.held_for_others
            || self.uploader_unknown != other.uploader_unknown
            || self.sync_conflicts != other.sync_conflicts
            || self.awaiting_hydration != other.awaiting_hydration
            || self.unreadable_documents != other.unreadable_documents
            || self.unreadable_folders != other.unreadable_folders
            || self.arriving != other.arriving
            || self.claimed_by_others != other.claimed_by_others
            || self.processed_here != other.processed_here
            || self.machines != other.machines
            || self.error != other.error
    }
}

/// One eligible file as observed by a scan tick (stat only — a OneDrive
/// placeholder must be claimable without ever hydrating it).
#[derive(Clone, Debug)]
pub(crate) struct FileFacts {
    pub path: PathBuf,
    /// Relative to the intake root, `/`-separated.
    pub relative_path: String,
    pub size: u64,
    pub modified_secs: i64,
}

/// What one walk of the intake folder found.
#[derive(Debug, Default)]
pub(crate) struct Walk {
    pub files: Vec<FileFacts>,
    /// Subfolders that could not be listed and were skipped.
    pub unreadable_folders: u32,
    /// Where the walk could not look, relative to the root and
    /// `/`-separated: subfolders that could not be listed, a folder whose
    /// listing broke off (`""` for the root), and files whose type or
    /// attributes could not be read. A walk with any of these did not see
    /// every document in the folder.
    pub hidden: Vec<String>,
}

/// Recursive walk with the same skip rules as intern-queue's path handling:
/// dot-names (which covers `.intern` itself), `~$` office lock files,
/// Intern's own history export, unsupported extensions, zero-byte files,
/// symlinks, and a document Intern renamed `NEEDS REVIEW - ...` because it
/// could not read it - that file is already in the queue, waiting for a
/// person, and claimed again it would be read as a new upload every time
/// its name changed.
///
/// Only the root has to be readable. A subfolder that refuses to be listed -
/// a shared drive grants permissions per folder, and a sync client can remove
/// one between the listing and the descent - is counted and skipped rather
/// than failing the whole scan, so one locked folder never stops every other
/// document in the share from being filed.
pub(crate) fn walk_intake(root: &Path, extensions: &[String]) -> io::Result<Walk> {
    let mut pending = vec![root.to_path_buf()];
    let mut walk = Walk::default();
    while let Some(directory) = pending.pop() {
        let place = || relative_slash_path(root, &directory).unwrap_or_default();
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if directory == root => return Err(error),
            Err(_) => {
                walk.unreadable_folders += 1;
                walk.hidden.push(place());
                continue;
            }
        };
        for entry in entries {
            let Ok(entry) = entry else {
                // The listing broke off, and whatever came after is unknown.
                walk.hidden.push(place());
                continue;
            };
            let path = entry.path();
            if skipped_name(&path) {
                continue;
            }
            let Ok(file_type) = entry.file_type() else {
                walk.hidden.extend(relative_slash_path(root, &path));
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if !file_type.is_file() || !supported(&path, extensions) {
                continue;
            }
            // Not `entry.metadata()`: Windows updates a directory entry
            // lazily, so a listing reports the size a file had at its last
            // flush and a document the sync client is still writing looks
            // settled. Opening the file for its attributes costs a handle but
            // hydrates nothing, and a file held so exclusively that even this
            // is refused is one that is still being written, so skipping it is
            // the right answer too - though not proof the file is new, so it
            // is reported as unseen.
            let Ok(metadata) = fs::metadata(&path) else {
                walk.hidden.extend(relative_slash_path(root, &path));
                continue;
            };
            if metadata.len() == 0 {
                continue;
            }
            let Some(relative_path) = relative_slash_path(root, &path) else {
                continue;
            };
            walk.files.push(FileFacts {
                size: metadata.len(),
                modified_secs: modified_secs(&metadata),
                path,
                relative_path,
            });
        }
    }
    walk.files
        .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(walk)
}

/// What the folder's directory listings say of each file a scan would
/// consider: its size and modification time, as the listing has them.
pub(crate) type Listing = HashMap<PathBuf, (u64, i64)>;

/// Lists the files [`walk_intake`] would consider, by the same rules, from
/// the directory listings alone: no file is opened, so nothing is recalled
/// from the cloud and no handle is taken on a document someone is writing.
///
/// What a listing says can lag - Windows updates a directory entry lazily,
/// which is why a scan opens each file for its attributes - so this is only
/// fit for what it is used for: noticing that something is new or changed
/// and a scan is worth running now. An empty file is listed, though a scan
/// skips it, so that its filling up is noticed. A subfolder that cannot be
/// listed is passed over; the scan is what counts and reports those.
pub(crate) fn list_intake(root: &Path, extensions: &[String]) -> io::Result<Listing> {
    let mut pending = vec![root.to_path_buf()];
    let mut listing = Listing::new();
    while let Some(directory) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if directory == root => return Err(error),
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if skipped_name(&path) {
                continue;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                pending.push(path);
                continue;
            }
            if !file_type.is_file() || !supported(&path, extensions) {
                continue;
            }
            // The entry's own metadata: what the listing carries on Windows,
            // and one call that follows no link elsewhere.
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            listing.insert(path, (metadata.len(), modified_secs(&metadata)));
        }
    }
    Ok(listing)
}

/// What a look at the folder's listings calls for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Glimpse {
    /// Nothing new: look again after the usual interval.
    Nothing,
    /// Something turned up, and more may be on its way: look again after
    /// the burst window.
    Wait,
    /// What turned up has stopped turning up: scan now.
    Scan,
}

/// Notices arrivals between scans from one listing of the folder to the
/// next ([`IntakeConfig::arrival_probe_interval`]).
///
/// It decides only when a scan runs, never what a scan does. A file that is
/// new since the last listing, or whose size or time has changed, is worth a
/// scan now; and so that files dropped together are first seen by one scan,
/// the scan waits until a listing a burst window later shows no further new
/// file.
#[derive(Debug, Default)]
pub(crate) struct ArrivalProbe {
    /// The last listing looked at; `None` before the first, and after a
    /// folder that could not be listed.
    last: Option<Listing>,
    /// Whether something has turned up that no scan has been run for yet.
    waiting: bool,
}

impl ArrivalProbe {
    /// Takes `listing` as what is already known, with nothing waiting: a
    /// scan is about to walk the folder, and only what turns up after this
    /// listing is news to the next look. `None`, for a folder that could
    /// not be listed, makes the next look the first.
    pub(crate) fn rebase(&mut self, listing: Option<Listing>) {
        self.last = listing;
        self.waiting = false;
    }

    /// Compares `listing` with the one before it and keeps it for the next
    /// look.
    pub(crate) fn look(&mut self, listing: Listing) -> Glimpse {
        let Some(last) = &self.last else {
            // Nothing to compare with: whatever is here, the scans see it.
            self.last = Some(listing);
            return Glimpse::Nothing;
        };
        let mut appeared = false;
        let mut changed = false;
        for (path, facts) in &listing {
            match last.get(path) {
                None => appeared = true,
                Some(before) if before != facts => changed = true,
                Some(_) => {}
            }
        }
        self.last = Some(listing);
        if self.waiting {
            if appeared {
                return Glimpse::Wait;
            }
            self.waiting = false;
            return Glimpse::Scan;
        }
        if appeared || changed {
            self.waiting = true;
            return Glimpse::Wait;
        }
        Glimpse::Nothing
    }
}

/// How long the look between scans waits before it lists the folder again:
/// `every`, or [`ARRIVAL_PROBE_LOAD`] times as long as the last listing
/// took, whichever is longer.
pub(crate) fn next_look_after(every: Duration, listing_took: Duration) -> Duration {
    every.max(listing_took.saturating_mul(ARRIVAL_PROBE_LOAD))
}

/// A file only becomes claimable once it has been observed with an
/// identical `(size, mtime)` for at least `min_quiet_seconds` on the clock.
///
/// The sync client (or a user copy) writes intake files incrementally, and a
/// key computed from a half-written file would never match the settled file —
/// the claim would orphan and the document would be processed from a torn
/// snapshot. A stretch of quiet is the cheapest proof of stability available
/// from stat alone. It is measured in time, not in scans: "Scan now" wakes the
/// loop at once, and two scans a moment apart proved nothing about a writer
/// that had merely paused.
#[derive(Debug)]
pub struct StabilityTracker {
    observed: HashMap<PathBuf, Observation>,
    min_quiet_seconds: i64,
    /// Set once a whole pass over the folder has been observed. A file first
    /// seen before then was already there when watching started.
    primed: bool,
}

#[derive(Debug)]
struct Observation {
    facts: (u64, i64),
    /// When this machine first saw the file at all, on its own clock.
    first_seen_at: i64,
    /// When the facts were last seen to change, or first seen.
    last_change_at: i64,
    /// Whether the file turned up or changed while being watched, rather than
    /// sitting unchanged since watching started.
    arriving: bool,
}

impl Default for StabilityTracker {
    fn default() -> Self {
        Self::new(DEFAULT_MIN_QUIET_SECONDS)
    }
}

impl StabilityTracker {
    pub fn new(min_quiet_seconds: i64) -> Self {
        Self {
            observed: HashMap::new(),
            min_quiet_seconds,
            primed: false,
        }
    }

    /// Records the observation; true when the facts are unchanged since the
    /// previous one and have been for at least the minimum quiet time. A first
    /// sighting is always unstable. A clock that moved backwards restarts the
    /// quiet time rather than holding the file until it catches up.
    pub fn observe(&mut self, path: &Path, size: u64, modified_secs: i64, now: i64) -> bool {
        match self.observed.get_mut(path) {
            Some(observation) => {
                if observation.facts != (size, modified_secs) {
                    observation.facts = (size, modified_secs);
                    observation.last_change_at = now;
                    observation.arriving = true;
                    return false;
                }
                if now < observation.last_change_at {
                    observation.last_change_at = now;
                }
                now - observation.last_change_at >= self.min_quiet_seconds
            }
            None => {
                self.observed.insert(
                    path.to_path_buf(),
                    Observation {
                        facts: (size, modified_secs),
                        first_seen_at: now,
                        last_change_at: now,
                        arriving: self.primed,
                    },
                );
                false
            }
        }
    }

    /// Whether a file turned up or changed while being watched. A file that
    /// has sat unchanged since watching started is not arriving, even before
    /// its quiet time has passed.
    pub fn is_arriving(&self, path: &Path) -> bool {
        self.observed
            .get(path)
            .is_some_and(|observation| observation.arriving)
    }

    /// When this machine first saw the file, which is not the same as the
    /// timestamp the file carries.
    pub fn first_seen_at(&self, path: &Path) -> Option<i64> {
        self.observed
            .get(path)
            .map(|observation| observation.first_seen_at)
    }

    /// Ends a pass over the folder: drops observations for files that
    /// vanished so the map cannot grow without bound, and from now on counts
    /// any new file as arriving.
    pub fn retain_live(&mut self, live: &HashSet<PathBuf>) {
        self.observed.retain(|path, _| live.contains(path));
        self.primed = true;
    }
}

fn supported(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extensions.contains(&extension.to_ascii_lowercase()))
}

/// Whether a document's bytes are still in the cloud rather than on this disk.
///
/// A seam, because no test can conjure a real Files On-Demand placeholder.
pub trait Hydration: Send + Sync {
    fn is_dehydrated(&self, path: &Path) -> bool;

    /// Asks the sync client for a placeholder's content, and reports whether
    /// the bytes are on this disk afterwards.
    ///
    /// A placeholder is only recalled when something opens it, so a claim held
    /// waiting for content waits for ever unless the scan asks. The default is
    /// to ask for nothing and simply report what is already there, which is
    /// the honest answer for anything that is not a sync client.
    fn hydrate(&self, path: &Path) -> bool {
        !self.is_dehydrated(path)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemHydration;

impl Hydration for SystemHydration {
    /// OneDrive and SharePoint mark an online-only file with the recall
    /// attributes; Windows sets them without the content ever being fetched, so
    /// reading them costs nothing and hydrates nothing. `symlink_metadata` is
    /// deliberate - following the placeholder is the one thing that would pull
    /// the file down.
    #[cfg(windows)]
    fn is_dehydrated(&self, path: &Path) -> bool {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_OFFLINE: u32 = 0x0000_1000;
        const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x0004_0000;
        const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;

        fs::symlink_metadata(path).is_ok_and(|metadata| {
            metadata.file_attributes()
                & (FILE_ATTRIBUTE_OFFLINE
                    | FILE_ATTRIBUTE_RECALL_ON_OPEN
                    | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
                != 0
        })
    }

    /// No sync client outside Windows presents placeholders to Intern, so
    /// everything readable here is already local.
    #[cfg(not(windows))]
    fn is_dehydrated(&self, _path: &Path) -> bool {
        false
    }

    /// Opening a placeholder is what makes Files On-Demand fetch it; one byte
    /// is enough to start it and the sync client brings down the whole file.
    /// That download happens on the scan thread, which is why it is asked for
    /// only for a document this machine has already tried and failed to read.
    /// Offline the open fails quickly and the attributes still say the
    /// content is in the cloud, which is the answer the caller wants.
    #[cfg(windows)]
    fn hydrate(&self, path: &Path) -> bool {
        use std::io::Read;

        let _ = fs::File::open(path).and_then(|mut file| file.read(&mut [0_u8; 1]));
        !self.is_dehydrated(path)
    }
}

/// A name a sync client produced by resolving an edit conflict, rather than a
/// document a person put in the folder.
///
/// OneDrive and SharePoint resolve a conflict by keeping both versions and
/// suffixing the losing one with the machine that wrote it -
/// `report-DESKTOP-A1B2C3.pdf`. That shape is indistinguishable from an
/// ordinary hyphenated filename, and `Invoice-ACME.pdf` is a real document, so
/// the suffix is believed only when it names a machine this shared folder has
/// actually seen. Silently skipping a document someone meant to file would be
/// the worse failure of the two, so the guess is never made on shape alone.
///
/// The other wording sync clients use - `(Jane's conflicted copy 2026-08-31)` -
/// carries no such ambiguity and stands on its own.
pub fn is_conflict_copy(path: &Path, machines: &[String]) -> bool {
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return false;
    };
    let stem = stem.to_lowercase();
    if stem.contains("conflicted copy") {
        return true;
    }
    let stem = strip_conflict_decorations(&stem);
    machines.iter().any(|machine| {
        let machine = machine.trim().to_lowercase();
        !machine.is_empty() && stem.ends_with(&format!("-{machine}"))
    })
}

/// The decorations a sync client adds when the conflict name it wants is
/// already taken: OneDrive numbers the repeat the way Explorer does -
/// `report-DESKTOP-A1B2C3 (2).pdf` - and some clients stamp the day the
/// conflict happened. Peeling them off leaves the machine name where the
/// suffix check can see it. They are never the evidence themselves: the name
/// underneath still has to end in a machine this folder has seen, so
/// `Invoice-ACME (2).pdf` remains an ordinary document.
fn strip_conflict_decorations(stem: &str) -> &str {
    let mut rest = stem.trim_end();
    loop {
        let peeled = strip_counter(rest).unwrap_or(rest);
        let peeled = strip_iso_date(peeled).unwrap_or(peeled);
        if peeled.len() == rest.len() {
            return rest;
        }
        rest = peeled;
    }
}

/// A trailing ` (2)`.
fn strip_counter(text: &str) -> Option<&str> {
    let (head, digits) = text.strip_suffix(')')?.rsplit_once('(')?;
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| head.trim_end())
}

/// A trailing ` 2026-08-31`.
fn strip_iso_date(text: &str) -> Option<&str> {
    let (head, last) = text.rsplit_once(' ')?;
    let bytes = last.as_bytes();
    (last.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit()))
    .then(|| head.trim_end())
}

fn skipped_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_none_or(|name| {
            name.starts_with('.')
                || name.starts_with("~$")
                || intern_core::is_history_export(name)
                || intern_core::is_needs_review_name(name)
        })
}

pub(crate) fn relative_slash_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

pub(crate) fn modified_secs(metadata: &fs::Metadata) -> i64 {
    let Ok(modified) = metadata.modified() else {
        return 0;
    };
    match modified.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(error) => -(error.duration().as_secs() as i64),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use std::time::Duration;

    use super::{
        ARRIVAL_BURST_WINDOW, ARRIVAL_PROBE_INTERVAL, ARRIVAL_PROBE_LOAD, ArrivalProbe,
        DEFAULT_SCAN_INTERVAL, Glimpse, IntakeConfig, Listing, list_intake, next_look_after,
        walk_intake,
    };
    use crate::cloud::{CloudProviderKind, CloudRoot};

    fn extensions() -> Vec<String> {
        vec!["pdf".to_string()]
    }

    /// A listing of `files`: name, size, modification time.
    fn listing(files: &[(&str, u64, i64)]) -> Listing {
        files
            .iter()
            .map(|(name, size, modified)| (PathBuf::from(name), (*size, *modified)))
            .collect()
    }

    /// The look between scans lists what a scan would consider - the same
    /// names skipped, subfolders followed, `.intern` never entered - and an
    /// empty file besides, so that its filling up is noticed. What it says
    /// of a settled file is what a scan reads from the file itself (checked
    /// where a listing is read from the file's own record; a Windows
    /// listing may lag, which is why it only ever decides when to scan).
    #[test]
    fn the_listing_between_scans_follows_a_scans_rules() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir(root.join("batch")).unwrap();
        fs::create_dir(root.join(".intern")).unwrap();
        for (name, bytes) in [
            ("keep.pdf", &b"settled"[..]),
            ("batch/inner.pdf", b"in a subfolder"),
            ("empty.pdf", b""),
            ("~$lock.pdf", b"x"),
            (".hidden.pdf", b"x"),
            ("notes.txt", b"x"),
            (".intern/claim.pdf", b"x"),
            ("NEEDS REVIEW - unread.pdf", b"x"),
        ] {
            fs::write(root.join(name), bytes).unwrap();
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("keep.pdf"), root.join("link.pdf")).unwrap();

        let listed = list_intake(root, &extensions()).unwrap();
        let walk = walk_intake(root, &extensions()).unwrap();

        let mut names: Vec<PathBuf> = listed.keys().cloned().collect();
        names.sort();
        assert_eq!(
            names,
            vec![
                root.join("batch").join("inner.pdf"),
                root.join("empty.pdf"),
                root.join("keep.pdf"),
            ]
        );
        assert_eq!(walk.files.len(), 2, "a scan skips the empty file");
        for facts in &walk.files {
            assert!(listed.contains_key(&facts.path), "{}", facts.relative_path);
            #[cfg(unix)]
            assert_eq!(
                listed[&facts.path],
                (facts.size, facts.modified_secs),
                "{}",
                facts.relative_path
            );
        }
        assert_eq!(listed[&root.join("empty.pdf")].0, 0);
        assert!(list_intake(&root.join("gone"), &extensions()).is_err());
    }

    /// On a folder where nothing happens, a look calls for nothing, however
    /// many there are: no scan runs that would not have run anyway.
    #[test]
    fn a_look_at_an_unchanged_folder_calls_for_nothing() {
        let files = [("a.pdf", 10, 100), ("empty.pdf", 0, 100)];
        let mut probe = ArrivalProbe::default();
        // The first listing is only what later ones are compared with.
        assert_eq!(probe.look(listing(&files)), Glimpse::Nothing);
        for _ in 0..50 {
            assert_eq!(probe.look(listing(&files)), Glimpse::Nothing);
        }
        // A file that left is no arrival either.
        assert_eq!(
            probe.look(listing(&[("empty.pdf", 0, 100)])),
            Glimpse::Nothing
        );
        assert_eq!(
            probe.look(listing(&[("empty.pdf", 0, 100)])),
            Glimpse::Nothing
        );
    }

    /// A file that turns up is worth a scan now - once a second listing, a
    /// burst window later, shows nothing further turning up - and then the
    /// folder is quiet again.
    #[test]
    fn a_new_file_calls_for_one_scan_after_the_burst_window() {
        let mut probe = ArrivalProbe::default();
        probe.rebase(Some(listing(&[("old.pdf", 10, 100)])));
        let dropped = [("old.pdf", 10, 100), ("dropped.pdf", 20, 200)];
        assert_eq!(probe.look(listing(&dropped)), Glimpse::Wait);
        assert_eq!(probe.look(listing(&dropped)), Glimpse::Scan);
        assert_eq!(probe.look(listing(&dropped)), Glimpse::Nothing);
    }

    /// Files dropped together are seen by one scan, as they are when a scan
    /// happens upon them: while each listing shows another new file the scan
    /// waits, and it runs once, for all of them, when one shows none.
    #[test]
    fn a_burst_of_files_calls_for_one_scan_when_it_has_all_turned_up() {
        let mut probe = ArrivalProbe::default();
        probe.rebase(Some(listing(&[])));
        let one = [("a.pdf", 10, 200)];
        let three = [("a.pdf", 10, 200), ("b.pdf", 10, 200), ("c.pdf", 10, 201)];
        let five = [
            ("a.pdf", 10, 200),
            ("b.pdf", 10, 200),
            ("c.pdf", 10, 201),
            ("d.pdf", 10, 201),
            ("e.pdf", 10, 202),
        ];
        assert_eq!(probe.look(listing(&one)), Glimpse::Wait);
        assert_eq!(probe.look(listing(&three)), Glimpse::Wait);
        assert_eq!(probe.look(listing(&five)), Glimpse::Wait);
        assert_eq!(probe.look(listing(&five)), Glimpse::Scan);
        assert_eq!(probe.look(listing(&five)), Glimpse::Nothing);
    }

    /// A file that changed - an empty one filling up, a document saved over -
    /// is worth a scan too. One still being written does not hold the scan
    /// off: how long it must then keep still is the scan's rule, and only
    /// the scan's.
    #[test]
    fn a_changed_file_calls_for_a_scan_and_a_growing_one_does_not_hold_it_off() {
        let mut probe = ArrivalProbe::default();
        probe.rebase(Some(listing(&[
            ("empty.pdf", 0, 100),
            ("saved.pdf", 10, 100),
        ])));
        assert_eq!(
            probe.look(listing(&[("empty.pdf", 512, 130), ("saved.pdf", 10, 100)])),
            Glimpse::Wait
        );
        // Still growing a burst window later: the scan runs all the same.
        assert_eq!(
            probe.look(listing(&[("empty.pdf", 2048, 131), ("saved.pdf", 10, 100)])),
            Glimpse::Scan
        );
        // And is called for again while it goes on changing.
        assert_eq!(
            probe.look(listing(&[("empty.pdf", 4096, 134), ("saved.pdf", 10, 100)])),
            Glimpse::Wait
        );
        // The same size saved at a later time is a change as well.
        probe.rebase(Some(listing(&[("saved.pdf", 10, 100)])));
        assert_eq!(
            probe.look(listing(&[("saved.pdf", 10, 160)])),
            Glimpse::Wait
        );
    }

    /// A scan is about to walk the folder: what the listing showed just
    /// before it is known, and nothing is waiting any more. After a folder
    /// that could not be listed, the next listing is the first again.
    #[test]
    fn a_scan_takes_what_was_listed_before_it_as_known() {
        let mut probe = ArrivalProbe::default();
        probe.rebase(Some(listing(&[])));
        let dropped = [("dropped.pdf", 20, 200)];
        assert_eq!(probe.look(listing(&dropped)), Glimpse::Wait);
        // A scan ran for another reason - its interval, Scan now.
        probe.rebase(Some(listing(&dropped)));
        assert_eq!(probe.look(listing(&dropped)), Glimpse::Nothing);

        probe.rebase(None);
        assert_eq!(
            probe.look(listing(&[("dropped.pdf", 20, 200), ("more.pdf", 5, 300)])),
            Glimpse::Nothing
        );
    }

    /// Listing a folder costs more the more it holds, and the look's cost is
    /// bounded all the same: it waits at least [`ARRIVAL_PROBE_LOAD`] times
    /// as long as its last listing took. A folder of tens of thousands of
    /// files that takes 47 ms to list is looked at every 4.7 s rather than
    /// every 3, and one that takes a fifth of a second is not looked at
    /// between its scans at all.
    #[test]
    fn a_folder_slow_to_list_is_looked_at_less_often() {
        let every = ARRIVAL_PROBE_INTERVAL;
        assert_eq!(next_look_after(every, Duration::ZERO), every);
        assert_eq!(next_look_after(every, Duration::from_millis(2)), every);
        assert_eq!(next_look_after(every, every / ARRIVAL_PROBE_LOAD), every);
        assert_eq!(
            next_look_after(every, Duration::from_millis(47)),
            Duration::from_millis(4_700)
        );
        assert!(next_look_after(every, Duration::from_millis(200)) >= DEFAULT_SCAN_INTERVAL);
        // The burst window stretches the same way.
        assert_eq!(
            next_look_after(ARRIVAL_BURST_WINDOW, Duration::from_millis(5)),
            ARRIVAL_BURST_WINDOW
        );
        assert_eq!(
            next_look_after(ARRIVAL_BURST_WINDOW, Duration::from_millis(20)),
            Duration::from_secs(2)
        );
        assert_eq!(
            next_look_after(every, Duration::MAX),
            Duration::MAX,
            "no overflow"
        );
    }

    /// Only a folder on this computer's own disk is looked at between
    /// scans. One inside a OneDrive or SharePoint folder, or on a network
    /// share, is scanned as it always was - and so is every folder until the
    /// host says otherwise.
    #[test]
    fn only_a_folder_on_this_computer_is_looked_at_between_scans() {
        let sync_roots = [CloudRoot {
            kind: CloudProviderKind::SharePoint,
            display_name: "Contoso - Legal".into(),
            root: PathBuf::from("/home/pat/Contoso/Legal - Documents"),
        }];
        let probed = |folder: &str| {
            let mut config = IntakeConfig::new(folder, extensions());
            assert_eq!(config.arrival_probe_interval, None, "off until asked for");
            config.probe_arrivals_if_local(&sync_roots);
            config.arrival_probe_interval
        };
        assert_eq!(probed("/home/pat/Scans"), Some(ARRIVAL_PROBE_INTERVAL));
        assert_eq!(probed("/home/pat/Contoso/Legal - Documents/Inbox"), None);
        assert_eq!(probed(r"\\fileserver\legal\intake"), None);
        assert_eq!(probed(r"\\?\UNC\fileserver\legal\intake"), None);

        // Asked again once the folder is no longer a plain one, it goes off.
        let mut config = IntakeConfig::new("/home/pat/Scans", extensions());
        config.probe_arrivals_if_local(&[]);
        assert_eq!(config.arrival_probe_interval, Some(ARRIVAL_PROBE_INTERVAL));
        config.intake_root = PathBuf::from("/home/pat/Contoso/Legal - Documents/Inbox");
        config.probe_arrivals_if_local(&sync_roots);
        assert_eq!(config.arrival_probe_interval, None);
    }

    #[test]
    fn a_missing_root_is_an_error_not_an_empty_folder() {
        let temp = tempfile::TempDir::new().unwrap();
        let gone = temp.path().join("gone");
        assert!(walk_intake(&gone, &extensions()).is_err());
    }

    #[test]
    fn dot_names_lock_files_and_empty_files_are_skipped() {
        let temp = tempfile::TempDir::new().unwrap();
        fs::write(temp.path().join("keep.pdf"), b"x").unwrap();
        fs::write(temp.path().join("~$lock.pdf"), b"x").unwrap();
        fs::write(temp.path().join(".hidden.pdf"), b"x").unwrap();
        fs::write(temp.path().join("empty.pdf"), b"").unwrap();
        fs::write(temp.path().join("notes.txt"), b"x").unwrap();
        fs::create_dir(temp.path().join(".intern")).unwrap();
        fs::write(temp.path().join(".intern").join("claim.pdf"), b"x").unwrap();
        let walk = walk_intake(temp.path(), &extensions()).unwrap();
        let names: Vec<String> = walk
            .files
            .iter()
            .map(|facts| facts.relative_path.clone())
            .collect();
        assert_eq!(names, vec!["keep.pdf".to_string()]);
        assert_eq!(walk.unreadable_folders, 0);
        assert!(walk.hidden.is_empty(), "skipped on purpose is not unseen");
    }

    /// The host hands intake the one admission list, so a legacy Word or
    /// Excel file is claimed like any other - and the `~$` lock file Office
    /// keeps beside an open one still is not.
    #[test]
    fn the_shared_list_admits_legacy_formats_and_still_skips_their_lock_files() {
        let temp = tempfile::TempDir::new().unwrap();
        for name in ["letter.doc", "ledger.xls", "~$letter.doc", "~$ledger.xls"] {
            fs::write(temp.path().join(name), b"x").unwrap();
        }
        let extensions = intern_core::SUPPORTED_EXTENSIONS
            .iter()
            .map(|extension| (*extension).to_owned())
            .collect::<Vec<_>>();
        let walk = walk_intake(temp.path(), &extensions).unwrap();
        let names: Vec<String> = walk
            .files
            .iter()
            .map(|facts| facts.relative_path.clone())
            .collect();
        assert_eq!(
            names,
            vec!["ledger.xls".to_string(), "letter.doc".to_string()]
        );
    }

    /// Intern's history export is a CSV, and CSV is admitted. Saved into the
    /// watched folder - the save dialog opens wherever the person last was -
    /// it was claimed as a new upload, its every filed path and description
    /// went to the model, and it was filed as a client document. A bank
    /// export beside it is still claimed.
    #[test]
    fn interns_own_history_export_is_never_claimed() {
        let temp = tempfile::TempDir::new().unwrap();
        for name in [
            "intern-history.csv",
            "Intern-History (2).CSV",
            "intern-history-march.csv",
            "statement.csv",
        ] {
            fs::write(temp.path().join(name), b"at,direction\r\n").unwrap();
        }
        let extensions = intern_core::SUPPORTED_EXTENSIONS
            .iter()
            .map(|extension| (*extension).to_owned())
            .collect::<Vec<_>>();
        let walk = walk_intake(temp.path(), &extensions).unwrap();
        let names: Vec<String> = walk
            .files
            .iter()
            .map(|facts| facts.relative_path.clone())
            .collect();
        assert_eq!(names, vec!["statement.csv".to_string()]);
    }

    /// Windows updates a file's directory entry lazily, so the size a listing
    /// reports for a file that is open for writing is the size it had at the
    /// last flush. Believing it lets the stability check pass a document the
    /// sync client is still writing, and the claim key is then computed from a
    /// torn snapshot that the settled file will never match.
    #[test]
    fn a_file_held_open_for_writing_is_not_stable() {
        use std::{fs::OpenOptions, io::Write};

        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("uploading.pdf");
        fs::write(&path, b"first chunk").unwrap();
        let settled = walk_intake(temp.path(), &extensions()).unwrap().files[0].size;

        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b" and a great deal more, not yet flushed")
            .unwrap();
        let growing = walk_intake(temp.path(), &extensions()).unwrap().files[0].size;
        drop(file);

        assert_ne!(
            growing, settled,
            "a file still being written must not look unchanged"
        );
        assert_eq!(growing, fs::metadata(&path).unwrap().len());
    }

    /// A shared drive grants permissions per folder. One folder this machine
    /// may not list is counted and skipped, and every readable document is
    /// still found.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_subfolder_is_counted_and_skipped() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::TempDir::new().unwrap();
        let locked = temp.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::write(locked.join("hidden.pdf"), b"cannot be listed").unwrap();
        fs::write(temp.path().join("open.pdf"), b"can be listed").unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        let outcome = walk_intake(temp.path(), &extensions());
        let listable = fs::read_dir(&locked).is_ok();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        let walk = outcome.unwrap();
        if listable {
            eprintln!("skipped: permission bits do not stop this user listing a folder");
            return;
        }
        assert_eq!(walk.unreadable_folders, 1);
        assert_eq!(walk.hidden, vec!["locked".to_string()]);
        let names: Vec<String> = walk
            .files
            .iter()
            .map(|facts| facts.relative_path.clone())
            .collect();
        assert_eq!(names, vec!["open.pdf".to_string()]);
    }

    /// Everything the walk could not reach is reported where it is, so the
    /// backlog can tell a hidden document from a gone one: a subfolder it
    /// could not list, and a file it could not read the attributes of. Made
    /// by spelling the root so long that both names take it past the path
    /// limit, which stops every user alike - permission bits do not stop
    /// root.
    #[cfg(target_os = "linux")]
    #[test]
    fn what_the_walk_could_not_reach_is_reported_as_hidden() {
        let temp = tempfile::TempDir::new().unwrap();
        let long = "d".repeat(250);
        fs::create_dir(temp.path().join(&long)).unwrap();
        fs::write(temp.path().join(&long).join("inside.pdf"), b"x").unwrap();
        fs::write(temp.path().join(format!("{long}.pdf")), b"x").unwrap();
        fs::write(temp.path().join("open.pdf"), b"x").unwrap();
        let mut root = temp.path().as_os_str().to_owned();
        while root.len() < 3900 {
            root.push("/.");
        }

        let walk = walk_intake(std::path::Path::new(&root), &extensions()).unwrap();
        let names: Vec<&str> = walk
            .files
            .iter()
            .map(|facts| facts.relative_path.as_str())
            .collect();
        assert_eq!(names, vec!["open.pdf"]);
        assert_eq!(walk.unreadable_folders, 1);
        let mut hidden = walk.hidden.clone();
        hidden.sort();
        assert_eq!(hidden, vec![long.clone(), format!("{long}.pdf")]);

        let plain = walk_intake(temp.path(), &extensions()).unwrap();
        assert_eq!(plain.files.len(), 3);
        assert!(plain.hidden.is_empty());
    }
}
