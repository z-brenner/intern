mod common;

use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime},
};

use common::{MockClock, facts_for, identity, labelled_identity, wait_until};
use intern_intake::{
    CLAIM_LEASE_SECONDS, COURTESY_DELAY_SECONDS, ClaimInfo, ClaimState, ClaimStore, DoneOutcome,
    ENQUEUE_RETRY_CAP_SECONDS, ENQUEUE_RETRY_SECONDS, Hydration, IntakeAdmission, IntakeConfig,
    IntakeHost, IntakeStatus, IntakeWatcher, ItemState, MachineIdentity, scan::is_conflict_copy,
};
use tempfile::TempDir;

/// In-memory host: records what the watcher hands over and answers
/// `item_state` from a scriptable map, defaulting to `Unknown` like a queue
/// that has never seen the path.
///
/// It keeps the real queue's rules where the watcher depends on them: one row
/// per path and content, so handing over a document it already has returns
/// that row as it is - failed, canceled, finished - instead of starting it
/// again; and a row the watcher itself withdrew starts again when the same
/// document is handed over later.
#[derive(Default)]
struct FakeHost {
    enqueued: Mutex<Vec<PathBuf>>,
    abandoned: Mutex<Vec<PathBuf>>,
    retried: Mutex<Vec<PathBuf>>,
    states: Mutex<HashMap<PathBuf, ItemState>>,
    /// The content each path's row was made from.
    contents: Mutex<HashMap<PathBuf, Vec<u8>>>,
    /// Rows the watcher's own `abandon` withdrew.
    withdrawn: Mutex<HashSet<PathBuf>>,
    statuses: Mutex<Vec<IntakeStatus>>,
    fail_enqueue: AtomicBool,
    /// Every hand-over the watcher attempted, refused or not. Each one
    /// follows a claim acquired in the shared folder.
    enqueue_calls: AtomicUsize,
    admission: Mutex<Option<IntakeAdmission>>,
    admission_calls: AtomicUsize,
}

impl FakeHost {
    fn enqueued(&self) -> Vec<PathBuf> {
        self.enqueued.lock().unwrap().clone()
    }

    fn abandoned(&self) -> Vec<PathBuf> {
        self.abandoned.lock().unwrap().clone()
    }

    fn retried(&self) -> Vec<PathBuf> {
        self.retried.lock().unwrap().clone()
    }

    /// A person takes the item out of the queue: Remove on a review item, or
    /// Discard waiting. The row is gone.
    fn remove(&self, path: &Path) {
        self.states.lock().unwrap().remove(path);
        self.contents.lock().unwrap().remove(path);
    }

    /// A person cancels the item. The host reports a row they canceled as
    /// kept where it is.
    fn cancel(&self, path: &Path) {
        self.set_state(
            path,
            ItemState::Done {
                outcome: DoneOutcome::KeptOriginal,
                result_filename: None,
            },
        );
    }

    fn admission_calls(&self) -> usize {
        self.admission_calls.load(Ordering::SeqCst)
    }

    fn set_state(&self, path: &Path, state: ItemState) {
        self.states
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), state);
    }
}

impl IntakeHost for FakeHost {
    // The existing tests exercise the explicitly local-only protocol.
    fn admission(&self, _path: &Path) -> IntakeAdmission {
        self.admission_calls.fetch_add(1, Ordering::SeqCst);
        self.admission
            .lock()
            .unwrap()
            .unwrap_or(IntakeAdmission::LocalOnly)
    }
    fn enqueue(&self, paths: &[PathBuf]) -> Result<(), String> {
        self.enqueue_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_enqueue.load(Ordering::SeqCst) {
            return Err("the queue is unavailable".to_string());
        }
        let mut states = self.states.lock().unwrap();
        let mut contents = self.contents.lock().unwrap();
        let mut withdrawn = self.withdrawn.lock().unwrap();
        for path in paths {
            let content = fs::read(path).map_err(|_| "FILE_UNREADABLE".to_string())?;
            let same_row =
                states.contains_key(path) && contents.get(path).is_some_and(|row| *row == content);
            if same_row && !withdrawn.contains(path) {
                continue;
            }
            withdrawn.remove(path);
            states.insert(path.clone(), ItemState::Active);
            contents.insert(path.clone(), content);
        }
        self.enqueued.lock().unwrap().extend(paths.iter().cloned());
        Ok(())
    }

    fn item_state(&self, path: &Path) -> ItemState {
        self.states
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .unwrap_or(ItemState::Unknown)
    }

    /// Like the real host: only a pending item can be withdrawn.
    fn abandon(&self, path: &Path) {
        self.abandoned.lock().unwrap().push(path.to_path_buf());
        let mut states = self.states.lock().unwrap();
        if let Some(state @ (ItemState::Active | ItemState::NeedsReview)) = states.get_mut(path) {
            *state = ItemState::Unknown;
            self.withdrawn.lock().unwrap().insert(path.to_path_buf());
        }
    }

    fn retry(&self, path: &Path) -> bool {
        let mut states = self.states.lock().unwrap();
        match states.get_mut(path) {
            Some(state @ ItemState::Failed) => {
                *state = ItemState::Active;
                self.retried.lock().unwrap().push(path.to_path_buf());
                true
            }
            _ => false,
        }
    }

    fn status_changed(&self, status: &IntakeStatus) {
        self.statuses.lock().unwrap().push(status.clone());
    }
}

/// Deterministic harness: an hour-long scan interval means the loop only
/// moves when `step` wakes it, and the mock clock stamps every tick uniquely
/// so `step` can wait for exactly the scan it triggered.
struct Rig {
    temp: TempDir,
    clock: Arc<MockClock>,
    host: Arc<FakeHost>,
    hydration: Arc<FakeHydration>,
    config: IntakeConfig,
    identity: MachineIdentity,
    watcher: IntakeWatcher,
}

/// Stands in for Files On-Demand: no test can create a real placeholder, so
/// the set of paths whose bytes are "still in the cloud" is scripted.
#[derive(Default)]
struct FakeHydration {
    dehydrated: Mutex<HashSet<PathBuf>>,
    /// Whether a request for a placeholder's content would succeed.
    reachable: AtomicBool,
    /// How often the scan asked the sync client for a file's bytes.
    fetches: AtomicUsize,
}

impl FakeHydration {
    fn fetches(&self) -> usize {
        self.fetches.load(Ordering::SeqCst)
    }

    fn set_dehydrated(&self, path: &Path, dehydrated: bool) {
        let mut paths = self.dehydrated.lock().unwrap();
        if dehydrated {
            paths.insert(path.to_path_buf());
        } else {
            paths.remove(path);
        }
    }
}

impl Hydration for FakeHydration {
    /// A file that is gone is not a placeholder, which is what the real
    /// attribute probe reports too.
    fn is_dehydrated(&self, path: &Path) -> bool {
        path.exists() && self.dehydrated.lock().unwrap().contains(path)
    }

    /// The sync client fetches the bytes when something opens the file;
    /// offline, the open fails and the file stays a placeholder.
    fn hydrate(&self, path: &Path) -> bool {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        if !self.reachable.load(Ordering::SeqCst) {
            return false;
        }
        self.dehydrated.lock().unwrap().remove(path);
        true
    }
}

impl Rig {
    fn start(process_others_uploads: bool, backlog_files: &[&str]) -> Rig {
        Self::start_as(
            identity("here-machine", "here"),
            process_others_uploads,
            backlog_files,
        )
    }

    fn start_as(
        identity: MachineIdentity,
        process_others_uploads: bool,
        backlog_files: &[&str],
    ) -> Rig {
        let temp = TempDir::new().unwrap();
        for name in backlog_files {
            fs::write(temp.path().join(name), b"backlog content").unwrap();
        }
        let mut config = IntakeConfig::new(temp.path(), vec!["pdf".to_string(), "txt".to_string()]);
        config.process_others_uploads = process_others_uploads;
        config.scan_interval = Duration::from_secs(3600);
        Self::launch(
            temp,
            MockClock::at_real_now(),
            Arc::new(FakeHost::default()),
            Arc::new(FakeHydration::default()),
            config,
            identity,
        )
    }

    fn launch(
        temp: TempDir,
        clock: Arc<MockClock>,
        host: Arc<FakeHost>,
        hydration: Arc<FakeHydration>,
        config: IntakeConfig,
        identity: MachineIdentity,
    ) -> Rig {
        let watcher = IntakeWatcher::start_with_seams(
            config.clone(),
            identity.clone(),
            host.clone(),
            clock.clone(),
            hydration.clone(),
        );
        wait_until("the initial scan", || {
            watcher.status().last_scan_at.is_some()
        });
        Rig {
            temp,
            clock,
            host,
            hydration,
            config,
            identity,
            watcher,
        }
    }

    /// Stops the watcher and starts a new one, as the app does on every
    /// start, update restart, and intake settings save. The folder, the
    /// queue, and the clock carry over; nothing the old watcher held in
    /// memory does.
    fn restart(self) -> Rig {
        let Rig {
            temp,
            clock,
            host,
            hydration,
            config,
            identity,
            watcher,
        } = self;
        drop(watcher);
        clock.advance(1);
        Self::launch(temp, clock, host, hydration, config, identity)
    }

    /// Triggers exactly one scan and waits for it to complete.
    fn step(&self) {
        self.step_by(1);
    }

    /// Lets `seconds` pass on the clock, then triggers exactly one scan.
    fn step_by(&self, seconds: i64) {
        let target = self.clock.advance(seconds);
        self.watcher.scan_now();
        wait_until("a scan tick", || {
            self.watcher.status().last_scan_at == Some(target)
        });
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.temp.path().join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn claim_file(&self, key: &str) -> PathBuf {
        self.temp
            .path()
            .join(".intern")
            .join("claims")
            .join(format!("{key}.json"))
    }

    fn read_claim(&self, key: &str) -> ClaimInfo {
        serde_json::from_slice(&fs::read(self.claim_file(key)).unwrap()).unwrap()
    }
}

#[test]
fn a_new_stable_file_is_claimed_enqueued_and_marked_done_after_the_host_finishes() {
    let rig = Rig::start(false, &[]);
    let path = rig.write("contract.pdf", b"agreement text");
    let key = facts_for(rig.temp.path(), "contract.pdf").key();

    rig.step();
    assert!(
        rig.host.enqueued().is_empty(),
        "a first sighting is not yet stable"
    );
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    let claim = rig.read_claim(&key);
    assert_eq!(claim.machine_id, "here-machine");
    assert_eq!(claim.state, ClaimState::Claimed);
    let store = ClaimStore::new(rig.temp.path(), identity("other", "elsewhere")).unwrap();
    assert_eq!(
        store.read_origin(&key).unwrap().machine_id,
        "here-machine",
        "a file appearing after watch start is attributed to this machine"
    );

    rig.host.set_state(
        &path,
        ItemState::Done {
            outcome: DoneOutcome::Renamed,
            result_filename: Some("2024 Contract.pdf".to_string()),
        },
    );
    rig.step();
    let done = rig.read_claim(&key);
    assert_eq!(done.state, ClaimState::Done);
    assert_eq!(done.outcome, Some(DoneOutcome::Renamed));
    assert_eq!(done.result_filename.as_deref(), Some("2024 Contract.pdf"));

    rig.step();
    let status = rig.watcher.status();
    assert_eq!(status.processed_here, 1);
    assert!(status.watching);
    assert_eq!(status.folder, rig.temp.path());
    assert!(
        status
            .machines
            .iter()
            .any(|machine| machine.machine_id == "here-machine"),
        "presence must include this machine: {status:?}"
    );
    assert!(
        rig.host.statuses.lock().unwrap().iter().any(|s| s.watching),
        "status_changed must have been reported to the host"
    );
}

#[test]
fn files_that_predate_the_watcher_are_held_for_others_in_mine_scope() {
    let rig = Rig::start(false, &["old-report.pdf"]);
    // Advance far past the courtesy delay: scope, not age, is what holds here.
    rig.clock.advance(10 * COURTESY_DELAY_SECONDS);
    rig.step();
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    assert_eq!(rig.watcher.status().held_for_others, 1);
}

#[test]
fn anothers_upload_is_claimed_only_after_the_courtesy_delay_in_everyone_scope() {
    let rig = Rig::start(true, &[]);
    rig.step();
    let path = rig.write("their-scan.pdf", b"uploaded elsewhere");
    let other = ClaimStore::new(rig.temp.path(), identity("other-machine", "elsewhere")).unwrap();
    other
        .write_origin(&facts_for(rig.temp.path(), "their-scan.pdf"))
        .unwrap();

    rig.step();
    rig.step();
    assert!(
        rig.host.enqueued().is_empty(),
        "the uploader's machine gets first shot during the courtesy delay"
    );
    assert_eq!(rig.watcher.status().held_for_others, 1);

    rig.clock.advance(COURTESY_DELAY_SECONDS);
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path]);
    assert_eq!(rig.watcher.status().held_for_others, 0);
}

#[test]
fn anothers_upload_is_never_claimed_in_mine_scope_even_after_the_delay() {
    let rig = Rig::start(false, &[]);
    rig.step();
    rig.write("their-scan.pdf", b"uploaded elsewhere");
    let other = ClaimStore::new(rig.temp.path(), identity("other-machine", "elsewhere")).unwrap();
    other
        .write_origin(&facts_for(rig.temp.path(), "their-scan.pdf"))
        .unwrap();
    rig.clock.advance(10 * COURTESY_DELAY_SECONDS);
    rig.step();
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    assert_eq!(rig.watcher.status().held_for_others, 1);
}

#[test]
fn a_backlog_file_is_claimed_in_everyone_scope_once_the_courtesy_delay_passes() {
    let rig = Rig::start(true, &["unattributed.pdf"]);
    rig.step();
    assert!(
        rig.host.enqueued().is_empty(),
        "still inside the courtesy delay"
    );
    rig.clock.advance(COURTESY_DELAY_SECONDS);
    rig.step();
    assert_eq!(
        rig.host.enqueued(),
        vec![rig.temp.path().join("unattributed.pdf")]
    );
}

#[test]
fn a_claim_lost_to_a_sync_conflict_makes_the_watcher_abandon_the_item() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contested.pdf", b"contested content");
    let facts = facts_for(rig.temp.path(), "contested.pdf");
    let key = facts.key();
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    let mut stolen = rig.read_claim(&key);
    stolen.machine_id = "other-machine".to_string();
    stolen.machine_name = "elsewhere".to_string();
    fs::write(
        rig.claim_file(&key),
        serde_json::to_vec_pretty(&stolen).unwrap(),
    )
    .unwrap();

    rig.step();
    assert_eq!(rig.host.abandoned(), vec![path]);
    rig.step();
    assert_eq!(
        rig.watcher.status().claimed_by_others,
        1,
        "after abandoning, the foreign claim is counted like any other"
    );
}

#[test]
fn a_claimed_file_deleted_by_the_user_leaves_a_removed_tombstone() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("withdrawn.pdf", b"changed their mind");
    let key = facts_for(rig.temp.path(), "withdrawn.pdf").key();
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    fs::remove_file(&path).unwrap();
    rig.host.set_state(&path, ItemState::Unknown);
    rig.step();
    let claim = rig.read_claim(&key);
    assert_eq!(claim.state, ClaimState::Done);
    assert_eq!(claim.outcome, Some(DoneOutcome::Removed));
}

/// Remove on a review item deletes the queue row. Releasing the claim because
/// the row was gone put the document straight back into the queue on the next
/// scan, to be analysed again seconds after the person said "Item removed."
#[test]
fn removed_review_item_is_tombstoned_not_reenqueued() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("bad-proposal.pdf", b"a document the person gave up on");
    let key = facts_for(rig.temp.path(), "bad-proposal.pdf").key();
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);
    rig.host.set_state(&path, ItemState::NeedsReview);
    rig.step();

    rig.host.remove(&path);
    rig.step();
    let tombstone = rig.read_claim(&key);
    assert_eq!(tombstone.state, ClaimState::Done);
    assert_eq!(tombstone.outcome, Some(DoneOutcome::KeptOriginal));

    for _ in 0..3 {
        rig.step();
    }
    assert_eq!(
        rig.host.enqueued(),
        vec![path],
        "a removed document is never handed over again"
    );
    assert_eq!(
        rig.read_claim(&key),
        tombstone,
        "tombstoned exactly once, and left alone after"
    );
    assert_eq!(rig.watcher.status().processed_here, 1);
}

/// The queue keeps a canceled row, and handing the same document over again
/// returns it canceled, so the old mapping (canceled reads as no item) made the
/// claim file appear and vanish every two scans for ever - an upload to the
/// sync client each time - and re-hashed the document on every cycle.
#[test]
fn user_cancel_is_tombstoned() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("not-this-one.pdf", b"a document the person canceled");
    let key = facts_for(rig.temp.path(), "not-this-one.pdf").key();
    rig.step();
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    rig.host.cancel(&path);
    rig.step();
    let tombstone = rig.read_claim(&key);
    assert_eq!(tombstone.state, ClaimState::Done);
    assert_eq!(tombstone.outcome, Some(DoneOutcome::KeptOriginal));
    for _ in 0..4 {
        rig.step();
        assert_eq!(rig.read_claim(&key), tombstone, "no claim churn");
    }
    assert_eq!(rig.host.enqueued(), vec![path]);
}

/// A claim with no queue row is not always a person's decision: a crash
/// between taking the claim and handing the document over leaves exactly that,
/// and the next run must still hand the document over.
#[test]
fn restart_adopted_claim_without_row_is_released() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("interrupted.pdf", b"claimed, then the app died");
    let facts = facts_for(rig.temp.path(), "interrupted.pdf");
    let rig = {
        // The previous run got as far as the origin marker and the claim.
        let store = ClaimStore::new(rig.temp.path(), rig.identity.clone()).unwrap();
        store.write_origin(&facts).unwrap();
        assert!(matches!(
            store.acquire(&facts),
            intern_intake::AcquireOutcome::Acquired
        ));
        rig.restart()
    };
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    assert!(
        !rig.claim_file(&facts.key()).exists(),
        "an adopted claim with no item behind it is released"
    );

    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path]);
    assert_eq!(rig.read_claim(&facts.key()).state, ClaimState::Claimed);
}

/// The watcher's own withdrawal is not a person's decision. A document it
/// let go of while its uploader could not be vouched for is handed over again
/// once it can be, and starts again rather than being tombstoned.
#[test]
fn a_document_the_watcher_withdrew_is_handed_over_again_not_tombstoned() {
    let rig = Rig::start(false, &[]);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Verified);
    let path = rig.write("contract.pdf", b"a document being processed");
    let key = facts_for(rig.temp.path(), "contract.pdf").key();
    rig.step();
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Revoked);
    rig.step();
    assert_eq!(rig.host.abandoned(), vec![path.clone()]);
    assert_eq!(rig.host.item_state(&path), ItemState::Unknown);

    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Verified);
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone(), path.clone()]);
    assert_eq!(rig.host.item_state(&path), ItemState::Active);
    assert_eq!(rig.read_claim(&key).state, ClaimState::Claimed);
}

#[test]
fn a_file_changing_between_scans_is_not_claimed_until_it_settles() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("uploading.pdf", b"first chunk");
    rig.step();
    for chunk in 0..3 {
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        writeln!(file, "chunk {chunk}").unwrap();
        drop(file);
        rig.step();
        assert!(
            rig.host.enqueued().is_empty(),
            "a growing file must never be claimed (iteration {chunk})"
        );
    }
    rig.step();
    assert_eq!(
        rig.host.enqueued(),
        vec![path],
        "one quiet scan interval proves stability"
    );
}

#[test]
fn an_enqueue_failure_releases_the_claim_so_a_later_scan_can_retry() {
    let rig = Rig::start(false, &[]);
    rig.step();
    rig.host.fail_enqueue.store(true, Ordering::SeqCst);
    let path = rig.write("retry.pdf", b"try me twice");
    let key = facts_for(rig.temp.path(), "retry.pdf").key();
    rig.step();
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    assert!(
        !rig.claim_file(&key).exists(),
        "a claim without a queued item would deadlock the document"
    );
    assert!(
        rig.watcher
            .status()
            .error
            .as_deref()
            .is_some_and(|error| error.starts_with("ENQUEUE_FAILED")),
        "status: {:?}",
        rig.watcher.status()
    );

    assert_eq!(rig.watcher.status().unreadable_documents, 1);

    rig.host.fail_enqueue.store(false, Ordering::SeqCst);
    rig.step();
    assert!(
        rig.host.enqueued().is_empty(),
        "the next attempt waits out its pause"
    );
    rig.step_by(ENQUEUE_RETRY_SECONDS);
    assert_eq!(rig.host.enqueued(), vec![path]);
    assert!(rig.claim_file(&key).exists());
    let status = rig.watcher.status();
    assert_eq!(status.error, None, "a clean scan clears the error");
    assert_eq!(status.unreadable_documents, 0);
}

/// A document the queue can never take - a file this account may not read -
/// was claimed and released on every scan, a claim file created and deleted
/// in the shared folder every 20 seconds for ever.
#[test]
fn enqueue_failures_back_off() {
    let rig = Rig::start(false, &[]);
    rig.step();
    rig.host.fail_enqueue.store(true, Ordering::SeqCst);
    let path = rig.write("locked.pdf", b"a file this account may not read");
    let key = facts_for(rig.temp.path(), "locked.pdf").key();
    rig.step();
    // A minute of scans a second apart: attempts at 0, 20, and 60 seconds.
    for _ in 0..61 {
        rig.step();
        let status = rig.watcher.status();
        assert_eq!(status.unreadable_documents, 1, "{status:?}");
        assert!(!rig.claim_file(&key).exists());
    }
    let attempts = rig.host.enqueue_calls.load(Ordering::SeqCst);
    assert_eq!(attempts, 3, "61 scans, 3 attempts");

    // The pause stops growing at its cap; the document is still tried.
    for _ in 0..12 {
        rig.step_by(ENQUEUE_RETRY_CAP_SECONDS);
    }
    assert_eq!(rig.host.enqueue_calls.load(Ordering::SeqCst), attempts + 12);

    // Settled at last: handed over, and nothing more is counted.
    rig.host.fail_enqueue.store(false, Ordering::SeqCst);
    rig.step_by(ENQUEUE_RETRY_CAP_SECONDS);
    assert_eq!(rig.host.enqueued(), vec![path]);
    assert_eq!(rig.watcher.status().unreadable_documents, 0);
}

/// Offline, the queue cannot read a placeholder it was handed, and the health
/// notice said "Up to date" while the document went nowhere.
#[test]
fn dehydrated_enqueue_failure_counts_as_awaiting_hydration() {
    let rig = Rig::start(false, &[]);
    rig.step();
    rig.host.fail_enqueue.store(true, Ordering::SeqCst);
    let path = rig.write("online-only.pdf", b"bytes that live in the cloud");
    rig.hydration.set_dehydrated(&path, true);
    rig.step();
    rig.step();
    for _ in 0..3 {
        let status = rig.watcher.status();
        assert_eq!(status.awaiting_hydration, 1, "{status:?}");
        assert_eq!(status.unreadable_documents, 0);
        assert_eq!(
            status.error, None,
            "waiting for OneDrive is not an error to show"
        );
        rig.step();
    }

    // Back online, the next attempt hands it over.
    rig.host.fail_enqueue.store(false, Ordering::SeqCst);
    rig.step_by(ENQUEUE_RETRY_SECONDS);
    assert_eq!(rig.host.enqueued(), vec![path]);
    assert_eq!(rig.watcher.status().awaiting_hydration, 0);
}

#[test]
fn a_file_claimed_by_another_machine_is_counted_and_left_alone() {
    let rig = Rig::start(true, &[]);
    rig.step();
    rig.write("busy-elsewhere.pdf", b"already being processed");
    let facts = facts_for(rig.temp.path(), "busy-elsewhere.pdf");
    let other = ClaimStore::new(rig.temp.path(), identity("other-machine", "elsewhere")).unwrap();
    other.write_origin(&facts).unwrap();
    assert!(matches!(
        other.acquire(&facts),
        intern_intake::AcquireOutcome::Acquired
    ));
    rig.clock.advance(10 * COURTESY_DELAY_SECONDS);
    rig.step();
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    let status = rig.watcher.status();
    assert_eq!(status.claimed_by_others, 1);
    assert_eq!(status.held_for_others, 0);
    let claim = rig.read_claim(&facts.key());
    assert_eq!(claim.machine_id, "other-machine");
}

/// A teammate's machine that crashes mid-document leaves its claim behind: the
/// lease expires and the heartbeat never moves again. The store knows how to
/// take such a claim over, but the watcher only ever asked it for a claim when
/// there was no claim file at all, so on a shared SharePoint folder a crash
/// left a document that no machine would ever process.
#[test]
fn a_claim_left_behind_by_a_crashed_machine_is_taken_over() {
    let rig = Rig::start(true, &[]);
    rig.step();
    let path = rig.write("stranded.pdf", b"a teammate's document");
    let facts = facts_for(rig.temp.path(), "stranded.pdf");
    let crashed =
        ClaimStore::new(rig.temp.path(), identity("crashed-machine", "elsewhere")).unwrap();
    crashed.write_origin(&facts).unwrap();
    assert!(matches!(
        crashed.acquire(&facts),
        intern_intake::AcquireOutcome::Acquired
    ));

    // That machine is now gone; nothing renews the claim again. Its lease is
    // still live here, so the document is somebody else's business.
    rig.clock.advance(COURTESY_DELAY_SECONDS + 1);
    rig.step();
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    assert_eq!(rig.watcher.status().claimed_by_others, 1);

    // The heartbeat has now stood still for a full lease as observed here,
    // and the lease deadline is long past: the takeover rules are satisfied.
    rig.clock.advance(2 * CLAIM_LEASE_SECONDS);
    rig.step();
    rig.step();
    assert_eq!(
        rig.host.enqueued(),
        vec![path],
        "the stranded document must be picked up: {:?}",
        rig.watcher.status()
    );
    let claim = rig.read_claim(&facts.key());
    assert_eq!(claim.machine_id, "here-machine");
    assert_eq!(claim.state, ClaimState::Claimed);
    assert_eq!(rig.watcher.status().claimed_by_others, 0);
}

/// Taking a stranded claim over does not widen whose documents this machine
/// works on. A teammate's upload is still theirs in "mine" scope, however long
/// the claim on it has been dead; a document uploaded here is ours to rescue.
#[test]
fn a_stranded_claim_is_taken_over_only_within_the_configured_scope() {
    let rig = Rig::start(false, &[]);
    rig.step();
    rig.write("theirs.pdf", b"a teammate's document");
    let mine = rig.write("mine.pdf", b"uploaded on this machine");
    let theirs_facts = facts_for(rig.temp.path(), "theirs.pdf");
    let mine_facts = facts_for(rig.temp.path(), "mine.pdf");
    ClaimStore::new(rig.temp.path(), identity("here-machine", "here"))
        .unwrap()
        .write_origin(&mine_facts)
        .unwrap();
    let crashed =
        ClaimStore::new(rig.temp.path(), identity("crashed-machine", "elsewhere")).unwrap();
    crashed.write_origin(&theirs_facts).unwrap();
    for facts in [&theirs_facts, &mine_facts] {
        assert!(matches!(
            crashed.acquire(facts),
            intern_intake::AcquireOutcome::Acquired
        ));
    }
    rig.step();
    rig.step();
    assert_eq!(rig.watcher.status().claimed_by_others, 2);

    rig.clock.advance(2 * CLAIM_LEASE_SECONDS);
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![mine]);
    let status = rig.watcher.status();
    assert_eq!(
        status.held_for_others, 1,
        "the teammate's document is still theirs: {status:?}"
    );
    assert_eq!(
        rig.read_claim(&theirs_facts.key()).machine_id,
        "crashed-machine"
    );
    assert_eq!(rig.read_claim(&mine_facts.key()).machine_id, "here-machine");
}

#[test]
fn update_config_rearms_on_a_new_folder_and_rebuilds_the_backlog() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let old_path = rig.write("first.pdf", b"first folder");
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![old_path]);

    let second = TempDir::new().unwrap();
    fs::write(second.path().join("pre-existing.pdf"), b"was already here").unwrap();
    let mut config = IntakeConfig::new(second.path(), vec!["pdf".to_string()]);
    config.scan_interval = Duration::from_secs(3600);
    rig.watcher.update_config(config);
    wait_until("the watcher to adopt the new folder", || {
        rig.watcher.status().folder == second.path()
    });
    rig.step();
    rig.step();
    let status = rig.watcher.status();
    assert_eq!(status.folder, second.path());
    assert_eq!(
        status.held_for_others, 1,
        "the new folder's pre-existing file is backlog again: {status:?}"
    );
    assert_eq!(rig.host.enqueued().len(), 1, "nothing new was enqueued");
}

#[test]
fn status_changed_fires_only_on_material_changes_not_every_tick() {
    let rig = Rig::start(false, &["held.pdf"]);
    rig.step();
    rig.step();
    let reported = rig.host.statuses.lock().unwrap().len();
    rig.step();
    rig.step();
    rig.step();
    assert_eq!(
        rig.host.statuses.lock().unwrap().len(),
        reported,
        "ticks that only advance last_scan_at must not wake the host"
    );
}

#[test]
fn skip_rules_ignore_dotfiles_office_locks_unsupported_and_empty_files() {
    let rig = Rig::start(false, &[]);
    rig.step();
    rig.write(".hidden.pdf", b"dotfile");
    rig.write("~$lock.pdf", b"office lock");
    rig.write("notes.xyz", b"unsupported extension");
    rig.write("empty.pdf", b"");
    fs::create_dir(rig.temp.path().join("nested")).unwrap();
    let nested = rig.temp.path().join("nested").join("deep.txt");
    fs::write(&nested, b"nested but supported").unwrap();
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![nested]);
    let status = rig.watcher.status();
    assert_eq!(status.held_for_others, 0);
    assert_eq!(status.claimed_by_others, 0);
}

#[test]
fn dropping_the_watcher_joins_the_scan_thread() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let Rig {
        temp,
        clock,
        host,
        hydration,
        watcher,
        ..
    } = rig;
    // A hang here (a detached or stuck thread) fails the test by timeout.
    drop(watcher);
    drop(clock);
    drop(host);
    drop(hydration);
    drop(temp);
}

#[test]
fn a_sync_conflict_copy_is_counted_and_left_alone() {
    let rig = Rig::start(false, &[]);
    rig.step();
    // OneDrive names the losing side of a conflict after the machine that
    // wrote it; SharePoint and Dropbox spell it out instead.
    rig.write("report-here.pdf", b"onedrive conflict copy");
    rig.write(
        "report (Jane's conflicted copy 2026-08-31).pdf",
        b"spelled-out conflict copy",
    );
    // A hyphen and a capitalised trailing word do not make a conflict copy.
    let genuine = rig.write("Invoice-ACME.pdf", b"an ordinary document");
    rig.step();
    rig.step();

    assert_eq!(rig.host.enqueued(), vec![genuine]);
    let status = rig.watcher.status();
    assert_eq!(status.sync_conflicts, 2);
    assert_eq!(status.held_for_others, 0);
}

#[test]
fn a_machine_suffix_is_only_a_conflict_copy_when_the_folder_knows_that_machine() {
    let machines = vec!["DESKTOP-A1B2C3".to_string(), "  ".to_string()];

    assert!(is_conflict_copy(
        Path::new("report-desktop-a1b2c3.pdf"),
        &machines
    ));
    assert!(is_conflict_copy(
        Path::new("report (Jane's conflicted copy 2026-08-31).pdf"),
        &[]
    ));

    // A machine this folder has never seen proves nothing about the name.
    assert!(!is_conflict_copy(
        Path::new("report-LAPTOP-ZZZ.pdf"),
        &machines
    ));
    assert!(!is_conflict_copy(Path::new("Invoice-ACME.pdf"), &machines));
    // A blank presence record must not match every file in the folder.
    assert!(!is_conflict_copy(Path::new("Invoice-.pdf"), &machines));
    assert!(!is_conflict_copy(Path::new("Invoice.pdf"), &machines));
}

/// A sync client that finds its conflict name already taken decorates it
/// further: OneDrive numbers the repeat the way Explorer does, and some
/// clients stamp the day the conflict happened. The document underneath is
/// still the losing side of a conflict, and filing it would put a second copy
/// of an already filed document into the destination.
#[test]
fn a_numbered_or_dated_conflict_copy_is_still_a_conflict_copy() {
    let machines = vec!["DESKTOP-A1B2C3".to_string()];

    assert!(is_conflict_copy(
        Path::new("report-DESKTOP-A1B2C3 (2).pdf"),
        &machines
    ));
    assert!(is_conflict_copy(
        Path::new("report-DESKTOP-A1B2C3 2026-08-31.pdf"),
        &machines
    ));
    assert!(is_conflict_copy(
        Path::new("report-DESKTOP-A1B2C3 2026-08-31 (3).pdf"),
        &machines
    ));
    assert!(is_conflict_copy(
        Path::new("report (Jane's conflicted copy 2026-08-31) (2).pdf"),
        &[]
    ));

    // The decoration is never the evidence: a name has to end in a machine
    // this folder has actually seen, counter or no counter.
    assert!(!is_conflict_copy(
        Path::new("Invoice-ACME (2).pdf"),
        &machines
    ));
    assert!(!is_conflict_copy(Path::new("report (2).pdf"), &machines));
    assert!(!is_conflict_copy(
        Path::new("report 2026-08-31.pdf"),
        &machines
    ));
}

#[test]
fn a_document_that_failed_while_still_in_the_cloud_is_held_not_tombstoned() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contract.pdf", b"an online-only document");
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);
    let key = facts_for(rig.temp.path(), "contract.pdf").key();

    // Offline: the placeholder never hydrated, so extraction failed without
    // anything having read the document.
    rig.hydration.set_dehydrated(&path, true);
    rig.host.set_state(&path, ItemState::Failed);
    rig.step();

    assert_eq!(rig.watcher.status().awaiting_hydration, 1);
    let claim = rig.read_claim(&key);
    assert_eq!(
        claim.state,
        ClaimState::Claimed,
        "a document nothing could read must not be tombstoned"
    );

    // Back online: the bytes arrive, so the queue retries the document in
    // place to give it a real attempt, and the claim stays this machine's.
    rig.hydration.set_dehydrated(&path, false);
    rig.step();
    assert_eq!(rig.watcher.status().awaiting_hydration, 0);
    assert_eq!(rig.host.retried(), vec![path]);
    let claim = rig.read_claim(&key);
    assert_eq!(claim.state, ClaimState::Claimed);
    assert_eq!(claim.machine_id, "here-machine");
}

#[test]
fn a_failure_with_the_content_local_still_tombstones() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contract.pdf", b"a document that is simply broken");
    rig.step();
    rig.step();
    let key = facts_for(rig.temp.path(), "contract.pdf").key();

    rig.host.set_state(&path, ItemState::Failed);
    rig.step();

    assert_eq!(rig.watcher.status().awaiting_hydration, 0);
    let claim = rig.read_claim(&key);
    assert_eq!(claim.state, ClaimState::Done);
    assert_eq!(claim.outcome, Some(DoneOutcome::Failed));
}

/// The queue keeps one row per path and content, so re-handing a failed
/// document over returns the failed row unchanged. Forgiving a document that
/// failed without its content therefore has to retry it through the host, or
/// nothing is retried at all and the next scan tombstones it as failed.
#[test]
fn hydration_forgiveness_retries_through_host() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contract.pdf", b"an online-only document");
    rig.step();
    rig.step();
    let key = facts_for(rig.temp.path(), "contract.pdf").key();

    rig.hydration.set_dehydrated(&path, true);
    rig.host.set_state(&path, ItemState::Failed);
    rig.step();
    assert_eq!(rig.watcher.status().awaiting_hydration, 1);

    rig.hydration.reachable.store(true, Ordering::SeqCst);
    rig.step();
    assert_eq!(rig.host.retried(), vec![path.clone()]);
    assert_eq!(rig.host.item_state(&path), ItemState::Active);
    for _ in 0..2 {
        rig.step();
        let claim = rig.read_claim(&key);
        assert_eq!(claim.state, ClaimState::Claimed, "the claim stays held");
        assert_eq!(claim.machine_id, "here-machine");
    }
    assert_eq!(rig.host.enqueued(), vec![path]);
}

/// Forgiveness is once per trip through the cloud: a document that fails
/// again with its content on this disk failed for real.
#[test]
fn second_failure_tombstones() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contract.pdf", b"an online-only document that is broken");
    rig.step();
    rig.step();
    let key = facts_for(rig.temp.path(), "contract.pdf").key();

    rig.hydration.set_dehydrated(&path, true);
    rig.host.set_state(&path, ItemState::Failed);
    rig.step();
    rig.hydration.set_dehydrated(&path, false);
    rig.step();
    assert_eq!(rig.host.retried(), vec![path.clone()]);
    rig.step();

    rig.host.set_state(&path, ItemState::Failed);
    rig.step();
    let claim = rig.read_claim(&key);
    assert_eq!(claim.state, ClaimState::Done);
    assert_eq!(claim.outcome, Some(DoneOutcome::Failed));
    assert_eq!(rig.host.retried(), vec![path], "retried exactly once");
}

#[test]
fn a_document_deleted_while_awaiting_hydration_stops_being_waited_on() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contract.pdf", b"an online-only document");
    rig.step();
    rig.step();
    let key = facts_for(rig.temp.path(), "contract.pdf").key();

    rig.hydration.set_dehydrated(&path, true);
    rig.host.set_state(&path, ItemState::Failed);
    rig.step();
    assert_eq!(rig.watcher.status().awaiting_hydration, 1);

    // The user gave up and deleted it. Nothing is owed to the cloud any more.
    fs::remove_file(&path).unwrap();
    rig.step();

    assert_eq!(rig.watcher.status().awaiting_hydration, 0);
    assert!(!rig.claim_file(&key).exists());
}

/// A shared drive grants permissions per folder. One folder this machine may
/// not list must not stop every other document in the share from being filed.
#[cfg(unix)]
#[test]
fn an_unreadable_subfolder_is_counted_and_skipped_rather_than_failing_the_scan() {
    use std::os::unix::fs::PermissionsExt;

    let rig = Rig::start(false, &[]);
    rig.step();
    let locked = rig.temp.path().join("locked");
    fs::create_dir(&locked).unwrap();
    fs::write(locked.join("hidden.pdf"), b"cannot be listed").unwrap();
    let open = rig.write("open.pdf", b"can be listed");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    // Root runs skip permission checks, and the restoration below must happen
    // even when the assertions fail, so the check is a guarded closure.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rig.step();
        rig.step();
        let status = rig.watcher.status();
        if fs::read_dir(&locked).is_ok() {
            return;
        }
        assert_eq!(status.unreadable_folders, 1, "{status:?}");
        assert_eq!(status.error, None, "{status:?}");
        assert_eq!(rig.host.enqueued(), vec![open.clone()]);
    }));
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    result.unwrap();
}

/// Rescuing a claim a crashed machine left behind is still subject to the
/// uploader gate. A document whose uploader cannot be established stays held,
/// whoever's dead claim happens to be sitting on it.
#[test]
fn a_stranded_claim_over_an_unverified_upload_stays_held() {
    let rig = Rig::start(true, &[]);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Unknown);
    rig.step();
    rig.write("unknown.pdf", b"nobody can vouch for this");
    let facts = facts_for(rig.temp.path(), "unknown.pdf");
    let crashed =
        ClaimStore::new(rig.temp.path(), identity("crashed-machine", "elsewhere")).unwrap();
    assert!(matches!(
        crashed.acquire(&facts),
        intern_intake::AcquireOutcome::Acquired
    ));
    rig.step();
    rig.step();
    rig.clock.advance(2 * CLAIM_LEASE_SECONDS);
    rig.step();
    rig.step();

    assert!(rig.host.enqueued().is_empty());
    let status = rig.watcher.status();
    assert_eq!(status.uploader_unknown, 1, "{status:?}");
    assert_eq!(
        rig.read_claim(&facts.key()).machine_id,
        "crashed-machine",
        "an unverifiable document is not claimed, dead lease or not"
    );
}

#[test]
fn unknown_uploader_is_held_even_when_all_uploads_are_enabled() {
    let rig = Rig::start(true, &[]);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Unknown);
    rig.write("new.pdf", b"new upload");
    rig.step();
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    assert_eq!(rig.watcher.status().uploader_unknown, 1);
}
#[test]
fn verified_uploads_do_not_depend_on_which_machine_first_observed_a_file() {
    let rig = Rig::start(false, &["mine.pdf"]);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Verified);
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued().len(), 1);
}
#[test]
fn other_uploads_are_held_and_a_later_revocation_abandons_owned_work() {
    let rig = Rig::start(false, &[]);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Other);
    let path = rig.write("other.pdf", b"other upload");
    rig.step();
    rig.step();
    assert!(rig.host.enqueued().is_empty());
    assert_eq!(rig.watcher.status().held_for_others, 1);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Verified);
    rig.step();
    assert_eq!(rig.host.enqueued().len(), 1);
    // A revocation is a verdict; an unverifiable moment is not, and is covered
    // by a_transient_verification_error_does_not_cancel_owned_work.
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Revoked);
    rig.step();
    assert!(rig.host.abandoned().contains(&path));
}

/// Microsoft being briefly unreachable is not an uploader verdict. Collapsing
/// that answer into Unknown makes the watcher report an unknown uploader even
/// though it learned nothing, while cancelling would throw away work that was
/// legitimately admitted.
#[test]
fn a_retryable_recheck_keeps_the_owned_claim_without_recording_a_verdict() {
    let rig = Rig::start(false, &[]);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Verified);
    let path = rig.write("contract.pdf", b"a document being processed");
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);
    let key = facts_for(rig.temp.path(), "contract.pdf").key();

    let owned = rig.read_claim(&key);
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Retryable);
    rig.step();
    assert!(
        rig.host.abandoned().is_empty(),
        "work in flight must survive an unverifiable moment"
    );
    let retained = rig.read_claim(&key);
    assert_eq!(retained.state, ClaimState::Claimed);
    assert_eq!(retained.machine_id, owned.machine_id);
    assert!(retained.lease_expires_at >= owned.lease_expires_at);
    assert_eq!(rig.watcher.status().uploader_unknown, 0);
    assert_eq!(rig.watcher.status().held_for_others, 0);
    assert_eq!(rig.watcher.status().error, None);

    // A verdict, rather than a blip, still stops the work.
    *rig.host.admission.lock().unwrap() = Some(IntakeAdmission::Revoked);
    rig.step();
    assert_eq!(rig.host.abandoned(), vec![path]);
    assert!(!rig.claim_file(&key).exists());
}

/// Verifying an uploader costs a Microsoft audit search, and only 32 can be
/// pending at once. Spending them on documents this machine has already
/// finished, or that another machine is processing, starves the documents that
/// actually need a verdict.
#[test]
fn held_and_done_files_are_not_reverified_each_scan() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let mine = rig.write("mine.pdf", b"already filed here");
    rig.step();
    rig.step();
    rig.host.set_state(
        &mine,
        ItemState::Done {
            outcome: DoneOutcome::Renamed,
            result_filename: Some("2026 Contract.pdf".to_string()),
        },
    );
    rig.step();

    rig.write("theirs.pdf", b"another machine is on it");
    let facts = facts_for(rig.temp.path(), "theirs.pdf");
    let other = ClaimStore::new(rig.temp.path(), identity("other-machine", "elsewhere")).unwrap();
    other.write_origin(&facts).unwrap();
    assert!(matches!(
        other.acquire(&facts),
        intern_intake::AcquireOutcome::Acquired
    ));
    rig.step();
    rig.step();

    let before = rig.host.admission_calls();
    rig.step();
    rig.step();
    assert_eq!(
        rig.host.admission_calls(),
        before,
        "neither a tombstoned document nor one another machine holds is ours to verify"
    );
    let status = rig.watcher.status();
    assert_eq!(status.processed_here, 1);
    assert_eq!(status.claimed_by_others, 1);
}

/// OneDrive names the losing side of a conflict after the machine that wrote
/// it, and that is the hostname — not the friendly label someone typed into
/// Settings. A labelled machine that only knows its label never recognises its
/// own conflict copies, and files the losing copy as a second document.
#[test]
fn a_labelled_machine_still_recognises_its_own_hostname_conflict_copies() {
    let rig = Rig::start_as(
        labelled_identity("here-machine", "Front desk", "DESKTOP-A1B2C3"),
        false,
        &[],
    );
    rig.step();
    rig.write(
        "report-DESKTOP-A1B2C3.pdf",
        b"the losing side of a conflict",
    );
    let genuine = rig.write("Invoice-ACME.pdf", b"an ordinary document");
    rig.step();
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![genuine]);
    assert_eq!(rig.watcher.status().sync_conflicts, 1);

    // The same is true of a teammate's labelled machine, whose presence record
    // is all this machine knows about it.
    let other = ClaimStore::new(
        rig.temp.path(),
        labelled_identity("other-machine", "Reception", "LAPTOP-Z9"),
    )
    .unwrap();
    other.touch_presence().unwrap();
    rig.write("memo-LAPTOP-Z9.pdf", b"their conflict copy");
    rig.step();
    rig.step();
    assert_eq!(rig.watcher.status().sync_conflicts, 2);
}

/// A label someone typed into Settings is not a name any sync client writes.
/// Treating it as a conflict suffix silently skipped ordinary documents that
/// happen to end in it, with no per-file explanation.
#[test]
fn machine_label_is_not_a_conflict_suffix() {
    let rig = Rig::start_as(
        labelled_identity("here-machine", "Office", "DESKTOP-A1"),
        false,
        &[],
    );
    rig.step();
    // A teammate's labelled machine, and one whose presence record predates
    // hostnames, when its display name was the hostname.
    ClaimStore::new(
        rig.temp.path(),
        labelled_identity("other-machine", "Reception", "LAPTOP-Z9"),
    )
    .unwrap()
    .touch_presence()
    .unwrap();
    fs::write(
        rig.temp
            .path()
            .join(".intern")
            .join("machines")
            .join("legacy-machine.json"),
        format!(
            r#"{{"version":1,"machineId":"legacy-machine","machineName":"OLD-PC","userName":"pat","lastSeenAt":{}}}"#,
            common::real_now()
        ),
    )
    .unwrap();
    let invoice = rig.write("Invoice-Office.pdf", b"an ordinary document");
    let memo = rig.write("Memo-Reception.pdf", b"another ordinary document");
    rig.write("report-DESKTOP-A1.pdf", b"our own conflict copy");
    rig.write("minutes-LAPTOP-Z9.pdf", b"a teammate's conflict copy");
    rig.write("notes-OLD-PC.pdf", b"a legacy machine's conflict copy");
    rig.step();
    rig.step();

    assert_eq!(rig.host.enqueued(), vec![invoice, memo]);
    assert_eq!(
        rig.watcher.status().sync_conflicts,
        3,
        "{:?}",
        rig.watcher.status()
    );
}

/// Nothing else ever opens a placeholder, so a claim held waiting for content
/// waits for ever unless the scan asks the sync client for the bytes.
#[test]
fn a_held_placeholder_is_hydrated_when_the_host_can_reach_the_cloud() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contract.pdf", b"an online-only document");
    rig.step();
    rig.step();
    let key = facts_for(rig.temp.path(), "contract.pdf").key();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    // Offline: the content cannot be fetched, so the claim is held open.
    rig.hydration.set_dehydrated(&path, true);
    rig.host.set_state(&path, ItemState::Failed);
    rig.step();
    assert_eq!(rig.watcher.status().awaiting_hydration, 1);

    rig.hydration.reachable.store(true, Ordering::SeqCst);
    rig.step();
    assert!(
        !rig.hydration.is_dehydrated(&path),
        "the scan must ask for the content it is waiting on"
    );
    assert_eq!(rig.watcher.status().awaiting_hydration, 0);
    assert_eq!(
        rig.host.retried(),
        vec![path],
        "the document gets a real attempt now the content is here"
    );
    assert_eq!(rig.read_claim(&key).state, ClaimState::Claimed);
}

/// A sync client can settle a file's size before it has finished with it and
/// stamp the uploader's modification time at the end. From that moment the
/// document has a different claim key, so the claim taken under the old key
/// must not be left behind as a live lease and the document must still be
/// filed exactly once.
#[test]
fn a_modification_time_stamped_after_the_claim_still_ends_in_one_filed_document() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("scan.pdf", b"a document the sync client is still finishing");
    rig.step();
    rig.step();
    let first = facts_for(rig.temp.path(), "scan.pdf").key();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    // The sync client finishes and puts the uploader's own timestamp on it.
    let file = OpenOptions::new().write(true).open(&path).unwrap();
    file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000))
        .unwrap();
    drop(file);
    let second = facts_for(rig.temp.path(), "scan.pdf").key();
    assert_ne!(
        first, second,
        "a new modification time is a new document key"
    );

    rig.step();
    rig.step();
    assert_eq!(
        rig.host.enqueued(),
        vec![path.clone(), path.clone()],
        "the same path handed over again is what the queue's own (path, hash) \
         uniqueness absorbs; a torn key is never processed on its own"
    );

    // The queue files it. Neither claim may be left behind as a live lease for
    // another machine's takeover math to deal with.
    rig.host.set_state(
        &path,
        ItemState::Done {
            outcome: DoneOutcome::Renamed,
            result_filename: Some("2020-09-13 Scan.pdf".to_string()),
        },
    );
    fs::remove_file(&path).unwrap();
    rig.step();
    assert_eq!(rig.read_claim(&first).state, ClaimState::Done);
    assert_eq!(rig.read_claim(&second).state, ClaimState::Done);
}

/// A Files On-Demand placeholder is a real path with real metadata and no
/// content, and claiming one must cost exactly the stat the walk already did.
/// The sync client is asked for the bytes only once a document has actually
/// failed to read, because that fetch happens on the scan thread and pulls the
/// whole file down.
#[test]
fn a_placeholder_is_claimed_from_its_metadata_without_the_scan_fetching_it() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("online-only.pdf", b"bytes that live in the cloud");
    rig.hydration.set_dehydrated(&path, true);
    rig.hydration.reachable.store(true, Ordering::SeqCst);
    rig.step();
    rig.step();

    assert_eq!(rig.host.enqueued(), vec![path.clone()]);
    assert_eq!(rig.hydration.fetches(), 0);
    assert!(
        rig.hydration.is_dehydrated(&path),
        "claiming a placeholder must not pull it down"
    );
    // And it stays that way while the queue works on it.
    rig.step();
    assert_eq!(rig.hydration.fetches(), 0);
    assert_eq!(rig.watcher.status().awaiting_hydration, 0);
}

/// The whole round trip a placeholder takes: claimed from metadata, failed
/// because there was nothing to read, held while the sync client fetches the
/// content, and then given a real attempt with the bytes on disk. A document
/// that was only ever waiting for its content must never end up tombstoned as
/// a document that failed.
#[test]
fn a_placeholder_that_hydrates_is_given_a_real_attempt_rather_than_a_failure() {
    let rig = Rig::start(false, &[]);
    rig.step();
    let path = rig.write("contract.pdf", b"an online-only document");
    rig.step();
    rig.step();
    let key = facts_for(rig.temp.path(), "contract.pdf").key();
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    // Offline: extraction failed without anything having read the document,
    // and every further scan asks again rather than giving up on it.
    rig.hydration.set_dehydrated(&path, true);
    rig.host.set_state(&path, ItemState::Failed);
    rig.step();
    rig.step();
    assert_eq!(rig.hydration.fetches(), 2);
    assert_eq!(rig.watcher.status().awaiting_hydration, 1);
    assert_eq!(rig.read_claim(&key).state, ClaimState::Claimed);
    assert_eq!(rig.read_claim(&key).machine_id, "here-machine");

    // The bytes arrive and the queue runs the document again, with
    // something to read, while the claim stays held.
    rig.hydration.reachable.store(true, Ordering::SeqCst);
    rig.step();
    assert_eq!(rig.watcher.status().awaiting_hydration, 0);
    assert_eq!(
        rig.host.retried(),
        vec![path.clone()],
        "the document deserves a second attempt now the content is here"
    );
    rig.step();
    assert_eq!(rig.read_claim(&key).state, ClaimState::Claimed);
    assert_eq!(rig.host.enqueued(), vec![path.clone()]);

    rig.host.set_state(
        &path,
        ItemState::Done {
            outcome: DoneOutcome::Renamed,
            result_filename: Some("2026-04-01 Contract.pdf".to_string()),
        },
    );
    rig.step();
    let claim = rig.read_claim(&key);
    assert_eq!(claim.state, ClaimState::Done);
    assert_eq!(claim.outcome, Some(DoneOutcome::Renamed));
    assert_eq!(
        claim.result_filename.as_deref(),
        Some("2026-04-01 Contract.pdf")
    );
}

/// A sync client preserves the uploader's modification time, so a document
/// that has only just landed here already carries an old timestamp. Measuring
/// the courtesy delay from that timestamp means there is no delay at all, and
/// this machine races the uploader's own machine for every file it syncs down.
#[test]
fn a_file_that_arrives_with_an_old_timestamp_still_waits_out_the_courtesy_delay() {
    let rig = Rig::start(true, &[]);
    rig.clock.advance(10 * COURTESY_DELAY_SECONDS);
    rig.step();
    let path = rig.write("their-scan.pdf", b"synced down with its original timestamp");
    let other = ClaimStore::new(rig.temp.path(), identity("other-machine", "elsewhere")).unwrap();
    other
        .write_origin(&facts_for(rig.temp.path(), "their-scan.pdf"))
        .unwrap();

    rig.step();
    rig.step();
    assert!(
        rig.host.enqueued().is_empty(),
        "the uploader's own machine still gets first shot"
    );
    assert_eq!(rig.watcher.status().held_for_others, 1);

    rig.clock.advance(COURTESY_DELAY_SECONDS);
    rig.step();
    assert_eq!(rig.host.enqueued(), vec![path]);
}
