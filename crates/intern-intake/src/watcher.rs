//! The polling watcher thread: walks the intake folder, runs the claim
//! protocol for each stable file, and feeds newly claimed documents to the
//! host queue.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Instant,
};

/// The pause after a document's first failed hand-over to the queue, doubled
/// after each further failure.
pub const ENQUEUE_RETRY_SECONDS: i64 = 20;
/// The longest pause between hand-over attempts for one document.
pub const ENQUEUE_RETRY_CAP_SECONDS: i64 = 15 * 60;

use crate::{
    coordination::{
        AcquireOutcome, COURTESY_DELAY_SECONDS, ClaimState, ClaimStore, Clock, DocumentFacts,
        DoneOutcome, SystemClock,
    },
    identity::MachineIdentity,
    scan::{
        FileFacts, Hydration, IntakeAdmission, IntakeConfig, IntakeHost, IntakeStatus, ItemState,
        StabilityTracker, SystemHydration, is_conflict_copy, walk_intake,
    },
};

pub struct IntakeWatcher {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

struct Shared {
    control: Mutex<Control>,
    wake: Condvar,
    status: Mutex<IntakeStatus>,
    shutdown: AtomicBool,
}

struct Control {
    config: IntakeConfig,
    generation: u64,
    wake: bool,
    shutdown: bool,
}

impl IntakeWatcher {
    pub fn start(
        config: IntakeConfig,
        identity: MachineIdentity,
        host: Arc<dyn IntakeHost>,
    ) -> IntakeWatcher {
        Self::start_with_clock(config, identity, host, Arc::new(SystemClock))
    }

    pub fn start_with_clock(
        config: IntakeConfig,
        identity: MachineIdentity,
        host: Arc<dyn IntakeHost>,
        clock: Arc<dyn Clock>,
    ) -> IntakeWatcher {
        Self::start_with_seams(config, identity, host, clock, Arc::new(SystemHydration))
    }

    pub fn start_with_seams(
        config: IntakeConfig,
        identity: MachineIdentity,
        host: Arc<dyn IntakeHost>,
        clock: Arc<dyn Clock>,
        hydration: Arc<dyn Hydration>,
    ) -> IntakeWatcher {
        let shared = Arc::new(Shared {
            status: Mutex::new(IntakeStatus::idle(config.intake_root.clone())),
            shutdown: AtomicBool::new(false),
            control: Mutex::new(Control {
                config,
                generation: 0,
                wake: false,
                shutdown: false,
            }),
            wake: Condvar::new(),
        });
        let thread = thread::Builder::new()
            .name("intern-intake-watcher".to_string())
            .spawn({
                let shared = shared.clone();
                move || run(&shared, &identity, host, clock, hydration)
            })
            .expect("intake watcher thread could not be spawned");
        IntakeWatcher {
            shared,
            thread: Some(thread),
        }
    }

    pub fn status(&self) -> IntakeStatus {
        lock(&self.shared.status).clone()
    }

    /// Wakes the loop for an immediate scan.
    pub fn scan_now(&self) {
        lock(&self.shared.control).wake = true;
        self.shared.wake.notify_all();
    }

    /// Re-arms the running watcher on a new configuration. Per-folder scan
    /// state (backlog, stability, owned claims) is discarded, exactly as a
    /// stop-and-start would discard it.
    pub fn update_config(&self, config: IntakeConfig) {
        {
            let mut control = lock(&self.shared.control);
            control.config = config;
            control.generation += 1;
            control.wake = true;
        }
        self.shared.wake.notify_all();
    }
}

/// The thread must never be detached: a detached scanner would keep claiming
/// documents for a host that is shutting down.
impl Drop for IntakeWatcher {
    fn drop(&mut self) {
        self.shared.shutdown.store(true, Ordering::SeqCst);
        lock(&self.shared.control).shutdown = true;
        self.shared.wake.notify_all();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Default)]
struct ScanState {
    store: Option<ClaimStore>,
    stability: StabilityTracker,
    /// Relative paths already present when watching started. Files that
    /// predate the watcher have no known uploader, so "mine" scope leaves
    /// them alone rather than guessing.
    backlog: HashSet<String>,
    backlog_recorded: bool,
    /// Claims this machine believes it holds, so a takeover or sync conflict
    /// that rewrites a claim file is noticed and the local item abandoned.
    owned: HashMap<String, PathBuf>,
    /// Claims kept open because the document failed while its content was
    /// still in the cloud. Their fate is decided once the bytes arrive.
    awaiting_hydration: HashSet<String>,
    /// Owned keys whose queue item this run has seen working or waiting for
    /// review. When such an item disappears while its file is still here, a
    /// person removed or discarded it; an owned claim with no item that was
    /// never seen live is the crash between acquire and enqueue instead.
    seen_live: HashSet<String>,
    /// Documents the queue refused, by key: how many hand-overs failed in a
    /// row, and the clock time before which no claim is attempted again.
    enqueue_failures: HashMap<String, (u32, i64)>,
}

fn run(
    shared: &Shared,
    identity: &MachineIdentity,
    host: Arc<dyn IntakeHost>,
    clock: Arc<dyn Clock>,
    hydration: Arc<dyn Hydration>,
) {
    let mut state = ScanState::default();
    let mut state_generation = 0_u64;
    let mut last_reported: Option<IntakeStatus> = None;
    loop {
        let (config, generation) = {
            let mut control = lock(&shared.control);
            if control.shutdown {
                break;
            }
            control.wake = false;
            (control.config.clone(), control.generation)
        };
        if generation != state_generation {
            state = ScanState::default();
            state_generation = generation;
        }
        let status = scan_once(
            &config,
            identity,
            host.as_ref(),
            &clock,
            hydration.as_ref(),
            &mut state,
            &shared.shutdown,
        );
        *lock(&shared.status) = status.clone();
        if last_reported
            .as_ref()
            .is_none_or(|previous| previous.materially_differs(&status))
        {
            host.status_changed(&status);
            last_reported = Some(status);
        }
        let deadline = Instant::now() + config.scan_interval;
        let mut control = lock(&shared.control);
        while !control.wake && !control.shutdown {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            control = shared
                .wake
                .wait_timeout(control, deadline - now)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }
}

fn scan_once(
    config: &IntakeConfig,
    identity: &MachineIdentity,
    host: &dyn IntakeHost,
    clock: &Arc<dyn Clock>,
    hydration: &dyn Hydration,
    state: &mut ScanState,
    shutdown: &AtomicBool,
) -> IntakeStatus {
    // Stamped at the start of the walk: the timestamp then vouches that
    // everything on disk up to that instant has been observed.
    let scan_started = clock.now();
    let mut status = IntakeStatus::idle(config.intake_root.clone());
    status.last_scan_at = Some(scan_started);
    let ScanState {
        store,
        stability,
        backlog,
        backlog_recorded,
        owned,
        awaiting_hydration,
        seen_live,
        enqueue_failures,
    } = state;
    if store.is_none() {
        match ClaimStore::with_clock(&config.intake_root, identity.clone(), clock.clone()) {
            Ok(created) => *store = Some(created),
            Err(error) => {
                status.error = Some(format!("INTAKE_COORDINATION_UNAVAILABLE: {error}"));
                return status;
            }
        }
    }
    let store = store.as_ref().expect("claim store was just created");

    let walk = match walk_intake(&config.intake_root, &config.extensions) {
        Ok(walk) => walk,
        Err(error) => {
            status.error = Some(format!("INTAKE_FOLDER_UNAVAILABLE: {error}"));
            return status;
        }
    };
    status.unreadable_folders = walk.unreadable_folders;
    let files = walk.files;

    // Read before the walk is processed so a conflict copy is recognised on the
    // same scan it appears, and include this machine: OneDrive names the losing
    // side of a conflict after whichever machine wrote it, which is often us.
    // Hostnames only: a label typed into Settings ("Office", "Jane") is not
    // what a sync client writes, and matching it would silently skip an
    // ordinary document such as "Lease-Office.pdf".
    let mut machines: Vec<String> = store
        .list_machines()
        .iter()
        .map(|presence| presence.conflict_name().to_owned())
        .collect();
    machines.push(identity.host_name.clone());

    let mut scanner = Scanner {
        config,
        identity,
        host,
        hydration,
        machines,
        clock: clock.as_ref(),
        store,
        stability,
        backlog,
        record_backlog: !*backlog_recorded,
        owned,
        awaiting_hydration,
        seen_live,
        enqueue_failures,
        status: &mut status,
        visited: HashSet::new(),
        live: HashSet::new(),
    };
    for facts in &files {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        scanner.process_file(facts);
    }
    scanner.finish_unseen_owned();
    // A refused document that is gone, or changed into a new key, starts
    // with a clean slate if it comes back.
    let visited = &scanner.visited;
    scanner
        .enqueue_failures
        .retain(|key, _| visited.contains(key));
    let live = scanner.live;
    *backlog_recorded = true;
    stability.retain_live(&live);

    if let Err(error) = store.touch_presence() {
        status.error = Some(format!("PRESENCE_WRITE_FAILED: {error}"));
    }
    store.prune();
    status.machines = store.list_machines();
    status
}

struct Scanner<'a> {
    config: &'a IntakeConfig,
    identity: &'a MachineIdentity,
    host: &'a dyn IntakeHost,
    hydration: &'a dyn Hydration,
    machines: Vec<String>,
    clock: &'a dyn Clock,
    store: &'a ClaimStore,
    stability: &'a mut StabilityTracker,
    backlog: &'a mut HashSet<String>,
    record_backlog: bool,
    owned: &'a mut HashMap<String, PathBuf>,
    awaiting_hydration: &'a mut HashSet<String>,
    seen_live: &'a mut HashSet<String>,
    enqueue_failures: &'a mut HashMap<String, (u32, i64)>,
    status: &'a mut IntakeStatus,
    visited: HashSet<String>,
    live: HashSet<PathBuf>,
}

impl Scanner<'_> {
    fn process_file(&mut self, facts: &FileFacts) {
        self.live.insert(facts.path.clone());
        // A conflict copy is the sync client's bookkeeping, not a new document.
        // Naming it would file a second copy of something already filed, so it
        // is counted and left where it is for a person to resolve.
        if is_conflict_copy(&facts.path, &self.machines) {
            self.status.sync_conflicts += 1;
            return;
        }
        if self.record_backlog {
            self.backlog.insert(facts.relative_path.clone());
        }
        if !self.stability.observe(
            &facts.path,
            facts.size,
            facts.modified_secs,
            self.clock.now(),
        ) {
            return;
        }
        let doc = DocumentFacts {
            relative_path: facts.relative_path.clone(),
            size: facts.size,
            modified_secs: facts.modified_secs,
        };
        let key = doc.key();
        self.visited.insert(key.clone());

        if self.owned.contains_key(&key) {
            self.manage_admitted(&key, facts);
            return;
        }

        // Asking the host about a document only when this machine might act on
        // it: an uploader check can cost a Microsoft audit search, and only 32
        // of those can be pending at once. Spending them on documents already
        // tombstoned here, or claimed by another machine, starves the ones that
        // actually need a verdict.
        match self.store.read(&key) {
            Some(claim) if claim.machine_id == self.identity.id => match claim.state {
                ClaimState::Claimed => {
                    // A claim from a previous run of this machine: adopt it and
                    // let the host's item state drive it forward again.
                    self.owned.insert(key.clone(), facts.path.clone());
                    self.manage_admitted(&key, facts);
                }
                ClaimState::Done => self.status.processed_here += 1,
            },
            Some(claim) => {
                // A machine that dies mid-document leaves its claim behind and
                // nothing else ever clears it: the lease runs out, the
                // heartbeat stops, and on a shared folder that document would
                // sit unfiled for ever unless another machine took the claim
                // over. Whether it may be taken over is the store's two-factor
                // liveness rule; who may have it is the same uploader gate an
                // unclaimed file goes through.
                if claim.state == ClaimState::Claimed {
                    if self.store.is_takeable(&claim) {
                        self.consider_admission(&doc, &key, facts);
                    } else {
                        self.status.claimed_by_others += 1;
                    }
                }
            }
            None => self.consider_admission(&doc, &key, facts),
        }
    }

    /// What this machine may do with a document nobody is processing: ask the
    /// host who uploaded it, and claim it only if the answer allows.
    fn consider_admission(&mut self, doc: &DocumentFacts, key: &str, facts: &FileFacts) {
        match self.host.admission(&facts.path) {
            IntakeAdmission::Verified => self.attempt_claim(doc, key, &facts.path),
            IntakeAdmission::LocalOnly => self.consider_unclaimed(doc, key, facts),
            IntakeAdmission::Other => self.status.held_for_others += 1,
            IntakeAdmission::Retryable => {}
            IntakeAdmission::Unknown | IntakeAdmission::Revoked => {
                self.status.uploader_unknown += 1
            }
        }
    }

    /// Re-checks the uploader of a document this machine already holds, then
    /// drives the claim.
    ///
    /// Only a verdict ends the work. "We could not check right now" — an
    /// unreachable Microsoft, a throttled connection, an audit event that has
    /// not been delivered yet — leaves the claim and the queue item alone: the
    /// queue authorizes again at every stage of its own, so nothing is
    /// processed on stale evidence, and cancelling here would throw away work
    /// that was legitimately admitted a moment ago.
    fn manage_admitted(&mut self, key: &str, facts: &FileFacts) {
        let admission = self.host.admission(&facts.path);
        match admission {
            IntakeAdmission::Other | IntakeAdmission::Revoked => {
                if admission == IntakeAdmission::Other {
                    self.status.held_for_others += 1;
                } else {
                    self.status.uploader_unknown += 1;
                }
                self.abandon(key, &facts.path);
                let _ = self.store.release(key);
                return;
            }
            IntakeAdmission::Unknown => {
                self.status.uploader_unknown += 1;
                if self.store.verify(key) {
                    let _ = self.store.renew(key);
                } else {
                    self.abandon(key, &facts.path);
                }
                return;
            }
            IntakeAdmission::Retryable => {
                if self.store.verify(key) {
                    let _ = self.store.renew(key);
                } else {
                    self.abandon(key, &facts.path);
                }
                return;
            }
            IntakeAdmission::Verified | IntakeAdmission::LocalOnly => {}
        }
        if self.store.verify(key) {
            self.manage_owned(key, &facts.path, true);
        } else {
            self.abandon(key, &facts.path);
        }
    }

    /// Scope and ownership rules for a file nobody has claimed.
    ///
    /// The courtesy delay gives the uploader's own machine first shot at its
    /// files even in "everyone" scope: origin markers and fresh files travel
    /// through the sync layer with the same latency, so claiming a
    /// seconds-old file here would routinely beat the uploader's own claim
    /// and shuttle the result across machines for no reason.
    fn consider_unclaimed(&mut self, doc: &DocumentFacts, key: &str, facts: &FileFacts) {
        let mut new_local = false;
        let claimable = match self.store.read_origin(key) {
            Some(origin) if origin.machine_id == self.identity.id => true,
            Some(_) => self.others_claimable(facts),
            // No origin marker: a file already present when watching started
            // has an unknown uploader and is treated like someone else's;
            // one that appeared later must have been put here locally.
            None if self.backlog.contains(&facts.relative_path) => self.others_claimable(facts),
            None => {
                new_local = true;
                true
            }
        };
        if !claimable {
            self.status.held_for_others += 1;
            return;
        }
        if new_local && let Err(error) = self.store.write_origin(doc) {
            self.status.error = Some(format!("ORIGIN_WRITE_FAILED: {error}"));
        }
        self.attempt_claim(doc, key, &facts.path);
    }

    fn others_claimable(&self, facts: &FileFacts) -> bool {
        if !self.config.process_others_uploads {
            return false;
        }
        // The delay runs from when the file turned up here, not from the
        // timestamp it carries. A sync client preserves the uploader's
        // modification time, so a document that has only just landed already
        // looks hours old and there would be no delay at all.
        let arrived = self
            .stability
            .first_seen_at(&facts.path)
            .map_or(facts.modified_secs, |seen| seen.max(facts.modified_secs));
        self.clock.now() - arrived >= COURTESY_DELAY_SECONDS
    }

    fn attempt_claim(&mut self, doc: &DocumentFacts, key: &str, path: &Path) {
        if self
            .enqueue_failures
            .get(key)
            .is_some_and(|&(_, next_try_at)| self.clock.now() < next_try_at)
        {
            self.count_refused(path);
            return;
        }
        match self.store.acquire(doc) {
            AcquireOutcome::Acquired => {
                if !self.store.verify(key) {
                    // Replaced under us before anything was enqueued; whoever
                    // owns the surviving claim keeps the document.
                    return;
                }
                match self.host.enqueue(&[path.to_path_buf()]) {
                    Ok(()) => {
                        self.enqueue_failures.remove(key);
                        self.owned.insert(key.to_string(), path.to_path_buf());
                    }
                    Err(message) => {
                        let _ = self.store.release(key);
                        self.note_refused(key, path, &message);
                    }
                }
            }
            AcquireOutcome::HeldByOther(_) => self.status.claimed_by_others += 1,
            AcquireOutcome::Done(_) => {}
            AcquireOutcome::Failed(error) => {
                self.status.error = Some(format!("CLAIM_IO_FAILED: {error}"));
            }
        }
    }

    /// The queue would not take a document this machine had just claimed.
    ///
    /// Some refusals never clear - a file this account may not read, a
    /// reparse point the queue will not open - and an offline placeholder
    /// fails until the connection returns. Claiming again on every scan
    /// created and deleted a claim file in the shared folder every 20 seconds
    /// for ever, so the next attempt waits, doubling from
    /// `ENQUEUE_RETRY_SECONDS` up to `ENQUEUE_RETRY_CAP_SECONDS`.
    fn note_refused(&mut self, key: &str, path: &Path, message: &str) {
        let attempts = self
            .enqueue_failures
            .get(key)
            .map_or(0, |&(attempts, _)| attempts)
            .saturating_add(1);
        let pause = ENQUEUE_RETRY_SECONDS
            .saturating_mul(1 << (attempts - 1).min(16))
            .min(ENQUEUE_RETRY_CAP_SECONDS);
        self.enqueue_failures
            .insert(key.to_owned(), (attempts, self.clock.now() + pause));
        // Waiting for OneDrive is what the health notice already explains;
        // only a refusal that is not about missing content is an error.
        if !self.count_refused(path) {
            self.status.error = Some(format!("ENQUEUE_FAILED: {message}"));
        }
    }

    /// Counts a document the queue refused, for as long as it is refused: as
    /// waiting for OneDrive when its content is still in the cloud, otherwise
    /// as unreadable. True for the first.
    fn count_refused(&mut self, path: &Path) -> bool {
        if self.hydration.is_dehydrated(path) {
            self.status.awaiting_hydration += 1;
            true
        } else {
            self.status.unreadable_documents += 1;
            false
        }
    }

    /// Drives an owned claim according to what the host reports about the
    /// item. `file_present` distinguishes a document still on disk from one
    /// the user deleted (tombstoned as `Removed` so the claim does not linger
    /// as a claimed lease forever).
    ///
    /// An item that vanishes while its file stays put is one of two things. If
    /// this run saw it working or in review, a person took it out of the
    /// queue - Remove, or Discard waiting - and that decision is recorded as
    /// `KeptOriginal`: releasing the claim instead put the document straight
    /// back into the queue on the next scan, to be analysed again. If it was
    /// never seen live, the claim outlived a crash between acquire and
    /// enqueue, and is released so the next scan hands the document over.
    fn manage_owned(&mut self, key: &str, path: &Path, file_present: bool) {
        match self.host.item_state(path) {
            ItemState::Active | ItemState::NeedsReview => {
                self.seen_live.insert(key.to_owned());
                self.keep(key, path);
            }
            // Nothing was learned about the document, so nothing is decided.
            ItemState::Unavailable => self.keep(key, path),
            ItemState::Done {
                outcome,
                result_filename,
            } => self.finish_owned(key, outcome, result_filename.as_deref()),
            ItemState::Failed => self.finish_failed(key, path),
            ItemState::Unknown if !file_present => {
                let _ = self.store.mark_done(key, DoneOutcome::Removed, None);
                self.owned.remove(key);
            }
            ItemState::Unknown if self.seen_live.contains(key) => {
                self.finish_owned(key, DoneOutcome::KeptOriginal, None);
            }
            ItemState::Unknown => {
                let _ = self.store.release(key);
                self.owned.remove(key);
            }
        }
    }

    /// Renews the lease on a document still in hand; a claim that is no
    /// longer this machine's lets the item go.
    fn keep(&mut self, key: &str, path: &Path) {
        if self.store.renew(key).is_err() {
            self.abandon(key, path);
        }
    }

    /// Withdraws the local item for a claim this machine is giving up. The
    /// host reports a withdrawn item as `Unknown`, and that is the watcher's
    /// own doing, so it must not read as a person's removal later.
    fn abandon(&mut self, key: &str, path: &Path) {
        self.owned.remove(key);
        self.seen_live.remove(key);
        self.host.abandon(path);
    }

    /// A document that failed while its bytes were still in the cloud has not
    /// been judged - nothing ever read it. Tombstoning it there would strand a
    /// perfectly good document behind a laptop that happened to be offline, and
    /// the tombstone outlives the trip. So the claim is held, not closed, and
    /// the verdict waits for the content.
    ///
    /// Once the bytes arrive the document is given the attempt it never had
    /// (`forgive`). A second failure with the content local is a real failure
    /// and tombstones normally, so this can forgive a document exactly once
    /// per trip through the cloud.
    fn finish_failed(&mut self, key: &str, path: &Path) {
        if self.hydration.is_dehydrated(path) {
            // Nothing else will ever open a placeholder, and a placeholder is
            // only recalled when something opens it, so a claim held waiting
            // for content would wait for ever unless the scan asks for the
            // bytes itself.
            if self.hydration.hydrate(path) {
                self.forgive(key, path);
                return;
            }
            // Counted only once the lease is actually held: a document we just
            // abandoned is not one we are waiting on.
            if self.store.renew(key).is_err() {
                self.awaiting_hydration.remove(key);
                self.abandon(key, path);
                return;
            }
            self.awaiting_hydration.insert(key.to_owned());
            self.status.awaiting_hydration += 1;
            return;
        }
        if self.awaiting_hydration.contains(key) {
            // The bytes arrived some other way.
            self.forgive(key, path);
            return;
        }
        self.finish_owned(key, DoneOutcome::Failed, None);
    }

    /// Gives a document that failed without its content the attempt it never
    /// had, now that the content is here.
    ///
    /// The host retries the failed item in place and the claim stays held.
    /// Releasing it instead, for the next scan to hand the document over
    /// again, never retried anything against the real queue: it keeps one row
    /// per path and content and hands back the failed row unchanged, which
    /// the following scan then tombstoned. Only the document the claim names
    /// is retried in place; a file that changed or vanished since, or a host
    /// that cannot retry, gets the claim released as before.
    fn forgive(&mut self, key: &str, path: &Path) {
        self.awaiting_hydration.remove(key);
        if self.visited.contains(key) && self.host.retry(path) {
            self.keep(key, path);
            return;
        }
        let _ = self.store.release(key);
        self.owned.remove(key);
    }

    fn finish_owned(&mut self, key: &str, outcome: DoneOutcome, result_filename: Option<&str>) {
        match self.store.mark_done(key, outcome, result_filename) {
            Ok(()) => {
                self.owned.remove(key);
                self.status.processed_here += 1;
            }
            Err(error) => self.status.error = Some(format!("CLAIM_UPDATE_FAILED: {error}")),
        }
    }

    /// Owned claims whose file was not seen at the same key this scan: the
    /// file was deleted, or changed and now lives under a new key. The claim
    /// still needs shepherding or it would sit as a claimed lease until some
    /// other machine's takeover math has to deal with it.
    fn finish_unseen_owned(&mut self) {
        let unseen: Vec<(String, PathBuf)> = self
            .owned
            .iter()
            .filter(|(key, _)| !self.visited.contains(*key))
            .map(|(key, path)| (key.clone(), path.clone()))
            .collect();
        for (key, path) in unseen {
            if !self.store.verify(&key) {
                self.abandon(&key, &path);
                continue;
            }
            self.manage_owned(&key, &path, path.exists());
        }
        // A claim we no longer hold - deleted, taken over, abandoned - is not
        // one we are waiting on the cloud for, nor one whose item we watched.
        // Without this the sets grow for the life of the process and a later
        // file landing on the same key would inherit a stale history.
        let owned = &*self.owned;
        self.awaiting_hydration
            .retain(|key| owned.contains_key(key));
        self.seen_live.retain(|key| owned.contains_key(key));
    }
}
