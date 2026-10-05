use std::{
    fs, io,
    path::Path,
    sync::{Arc, Barrier},
    thread,
};

use intern_core::{
    ErrorCode, FileApplier, FileSystem, HISTORY_LIMIT, LockedFile, OperationDirection,
    OperationKind, OperationStage, QueueStatus, QueueStore, StdFileSystem, source_path_key,
};

use tempfile::TempDir;

fn store(temp: &TempDir) -> QueueStore {
    QueueStore::open(temp.path().join("queue.sqlite3")).unwrap()
}

fn advance_to_ready(db: &QueueStore, id: i64) {
    assert_eq!(db.claim_next().unwrap().unwrap().id, id);
    db.transition(id, QueueStatus::Extracting, QueueStatus::Analyzing, None)
        .unwrap();
    db.transition(id, QueueStatus::Analyzing, QueueStatus::Ready, None)
        .unwrap();
}

/// Enqueues `source` and completes it through a real journalled apply so the
/// completed row carries an apply/complete receipt naming `destination`.
fn complete_via_apply(db: &Arc<QueueStore>, source: &Path, destination: &Path) -> i64 {
    let hash = StdFileSystem.hash(source).unwrap();
    let item = db.enqueue(source, &hash).unwrap();
    advance_to_ready(db, item.id);
    db.begin_applying(item.id, QueueStatus::Ready).unwrap();
    let receipt = FileApplier::local(Arc::clone(db))
        .apply(item.id, source, destination, &hash)
        .unwrap();
    db.complete_apply(item.id, receipt.id).unwrap();
    item.id
}

#[test]
fn duplicate_unchanged_path_focuses_existing_item_but_same_hash_at_new_path_enqueues() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let first = db.enqueue(Path::new("C:/docs/a.pdf"), "same-hash").unwrap();
    let duplicate = db.enqueue(Path::new("C:/docs/a.pdf"), "same-hash").unwrap();
    let case_variant = db
        .enqueue(Path::new("c:\\DOCS\\A.PDF"), "same-hash")
        .unwrap();
    let second_path = db.enqueue(Path::new("C:/docs/b.pdf"), "same-hash").unwrap();
    assert_eq!(first.id, duplicate.id);
    assert_eq!(first.id, case_variant.id);
    assert_ne!(first.id, second_path.id);
    assert_eq!(db.list().unwrap().len(), 2);
}

#[test]
fn exactly_one_concurrent_claimant_wins() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("queue.sqlite3");
    QueueStore::open(&path)
        .unwrap()
        .enqueue(Path::new("one.pdf"), "h1")
        .unwrap();
    QueueStore::open(&path)
        .unwrap()
        .enqueue(Path::new("two.pdf"), "h2")
        .unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let path = path.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let db = QueueStore::open(path).unwrap();
                barrier.wait();
                db.claim_next().unwrap()
            })
        })
        .collect();
    barrier.wait();
    let claims = handles
        .into_iter()
        .filter_map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].status, QueueStatus::Extracting);
}

#[test]
fn invalid_transition_has_stable_code_and_does_not_change_state() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let item = db.enqueue(Path::new("one.pdf"), "h1").unwrap();
    let err = db
        .transition(item.id, QueueStatus::Queued, QueueStatus::Completed, None)
        .unwrap_err();
    assert_eq!(err.code(), ErrorCode::InvalidTransition);
    assert_eq!(db.list().unwrap()[0].status, QueueStatus::Queued);
}

#[test]
fn recovery_requeues_processing_but_leaves_applying_for_reconciliation() {
    for interrupted_status in [QueueStatus::Extracting, QueueStatus::Analyzing] {
        let temp = TempDir::new().unwrap();
        let db = store(&temp);
        db.enqueue(Path::new("processing.pdf"), "hp").unwrap();
        let item = db.claim_next().unwrap().unwrap();
        if interrupted_status == QueueStatus::Analyzing {
            db.transition(
                item.id,
                QueueStatus::Extracting,
                QueueStatus::Analyzing,
                None,
            )
            .unwrap();
        }
        drop(db);
        let db = store(&temp);
        assert_eq!(db.recover_interrupted().unwrap(), 1);
        assert_eq!(db.list().unwrap()[0].status, QueueStatus::Queued);
    }

    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    db.enqueue(Path::new("applying.pdf"), "ha").unwrap();
    db.enqueue(Path::new("waiting.pdf"), "hw").unwrap();
    let applying = db.claim_next().unwrap().unwrap();
    db.transition(
        applying.id,
        QueueStatus::Extracting,
        QueueStatus::Analyzing,
        None,
    )
    .unwrap();
    db.transition(
        applying.id,
        QueueStatus::Analyzing,
        QueueStatus::Ready,
        None,
    )
    .unwrap();
    db.begin_applying(applying.id, QueueStatus::Ready).unwrap();
    drop(db);
    let db = store(&temp);
    assert_eq!(db.recover_interrupted().unwrap(), 0);
    assert_eq!(db.list().unwrap()[0].status, QueueStatus::Applying);
    assert!(db.claim_next().unwrap().is_none());
    db.claim_applying_reconciliation(applying.id).unwrap();
    let db = Arc::new(db);
    assert_eq!(
        FileApplier::local(db.clone())
            .reconcile(applying.id)
            .unwrap()
            .status,
        QueueStatus::Ready
    );
    assert_eq!(
        db.claim_next().unwrap().unwrap().source_path,
        Path::new("waiting.pdf")
    );
}

#[test]
fn live_owner_cannot_be_stolen_but_closed_owner_can_be_recovered() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("queue.sqlite3");
    let owner = QueueStore::open(&path).unwrap();
    owner.enqueue(Path::new("active.pdf"), "ha").unwrap();
    let active = owner.claim_next().unwrap().unwrap();
    let observer = QueueStore::open(&path).unwrap();
    assert_eq!(observer.recover_interrupted().unwrap(), 0);
    assert_eq!(observer.list().unwrap()[0].status, QueueStatus::Extracting);
    drop(owner);
    assert_eq!(observer.recover_interrupted().unwrap(), 1);
    assert_eq!(observer.list().unwrap()[0].id, active.id);
    assert_eq!(observer.list().unwrap()[0].status, QueueStatus::Queued);
}

#[test]
fn expired_item_lease_does_not_override_a_fresh_owner_heartbeat() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("queue.sqlite3");
    let owner = QueueStore::open(&path).unwrap();
    owner.enqueue(Path::new("active.pdf"), "ha").unwrap();
    let active = owner.claim_next().unwrap().unwrap();
    let observer = QueueStore::open(&path).unwrap();
    let inspector = rusqlite::Connection::open(&path).unwrap();
    inspector
        .execute(
            "UPDATE queue_items SET lease_expires_at = 0 WHERE id = ?1",
            [active.id],
        )
        .unwrap();

    assert_eq!(observer.recover_interrupted().unwrap(), 0);
    assert_eq!(observer.list().unwrap()[0].status, QueueStatus::Extracting);
    assert_eq!(
        owner.renew_lease(active.id).unwrap().status,
        QueueStatus::Extracting
    );
}

#[test]
fn automatic_processing_retries_stop_after_two_failures() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let item = db.enqueue(Path::new("a.pdf"), "ha").unwrap();
    for expected_attempt in 1..=2 {
        let claimed = db.claim_next().unwrap().unwrap();
        assert_eq!(claimed.id, item.id);
        let status = db
            .record_processing_failure(item.id, ErrorCode::ModelOutputInvalid)
            .unwrap();
        assert_eq!(
            status,
            if expected_attempt == 1 {
                QueueStatus::Queued
            } else {
                QueueStatus::Failed
            }
        );
    }
    assert!(db.claim_next().unwrap().is_none());
}

/// A failure that a second attempt would only repeat fails the document on
/// the first one, with its own code, and only for the session doing the work.
#[test]
fn record_terminal_failure_fails_owned_work_once() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("queue.sqlite3");
    let db = QueueStore::open(&path).unwrap();

    let locked = db.enqueue(Path::new("locked.docx"), "hl").unwrap();
    assert_eq!(db.claim_next().unwrap().unwrap().id, locked.id);
    db.record_terminal_failure(locked.id, ErrorCode::PasswordProtected)
        .unwrap();
    let failed = &db.list().unwrap()[0];
    assert_eq!(failed.status, QueueStatus::Failed);
    assert_eq!(failed.error_code, Some(ErrorCode::PasswordProtected));
    assert_eq!(failed.processing_failures, 2);
    assert!(
        db.claim_next().unwrap().is_none(),
        "nothing re-queues a terminal failure"
    );
    // Once failed it is no longer processing work, so a second report is
    // refused rather than rewriting the reason.
    assert_eq!(
        db.record_terminal_failure(locked.id, ErrorCode::ExtractionFailed)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict
    );

    // Work that is waiting, or owned by another session, is not this
    // session's to fail.
    let waiting = db.enqueue(Path::new("waiting.pdf"), "hw").unwrap();
    assert_eq!(
        db.record_terminal_failure(waiting.id, ErrorCode::DocumentTooLarge)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict
    );
    let other = QueueStore::open(&path).unwrap();
    assert_eq!(other.claim_next().unwrap().unwrap().id, waiting.id);
    assert_eq!(
        db.record_terminal_failure(waiting.id, ErrorCode::DocumentTooLarge)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict
    );
    assert_eq!(other.list().unwrap()[1].status, QueueStatus::Extracting);

    // An item that already used attempts keeps counting up from there.
    let inspector = rusqlite::Connection::open(&path).unwrap();
    inspector
        .execute(
            "UPDATE queue_items SET processing_failures = 4 WHERE id = ?1",
            [waiting.id],
        )
        .unwrap();
    other
        .record_terminal_failure(waiting.id, ErrorCode::ModelDeclined)
        .unwrap();
    let declined = &other.list().unwrap()[1];
    assert_eq!(declined.status, QueueStatus::Failed);
    assert_eq!(declined.error_code, Some(ErrorCode::ModelDeclined));
    assert_eq!(declined.processing_failures, 5);
}

#[test]
fn clear_terminal_removes_only_terminal_rows() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let active = db.enqueue(Path::new("active.pdf"), "a").unwrap();
    let failed = db.enqueue(Path::new("failed.pdf"), "f").unwrap();
    db.transition(failed.id, QueueStatus::Queued, QueueStatus::Canceled, None)
        .unwrap();
    assert_eq!(db.clear_terminal().unwrap(), 1);
    assert_eq!(db.list().unwrap()[0].id, active.id);
}

/// Pointing the queue at the wrong folder must be recoverable, and recovering
/// must not cost a rename the user can no longer undo. Only rows that never
/// started are dropped.
#[test]
fn discard_queued_drops_waiting_work_and_nothing_else() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);

    // Enqueued in claim order, because claim_next takes the oldest and the queue
    // holds exactly one active claim - it processes one document at a time. The
    // review item therefore has to reach a resting state before the next claim.
    let awaiting_decision = db.enqueue(Path::new("review.pdf"), "r").unwrap();
    let in_flight = db.enqueue(Path::new("in-flight.pdf"), "f").unwrap();
    let canceled = db.enqueue(Path::new("canceled.pdf"), "c").unwrap();
    let waiting_one = db.enqueue(Path::new("waiting-one.pdf"), "w1").unwrap();
    let waiting_two = db.enqueue(Path::new("waiting-two.pdf"), "w2").unwrap();

    assert_eq!(db.claim_next().unwrap().unwrap().id, awaiting_decision.id);
    db.transition(
        awaiting_decision.id,
        QueueStatus::Extracting,
        QueueStatus::Analyzing,
        None,
    )
    .unwrap();
    db.transition(
        awaiting_decision.id,
        QueueStatus::Analyzing,
        QueueStatus::NeedsReview,
        None,
    )
    .unwrap();
    // Only now is the single active slot free for the next claim.
    assert_eq!(db.claim_next().unwrap().unwrap().id, in_flight.id);
    db.transition(
        canceled.id,
        QueueStatus::Queued,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();

    assert_eq!(db.discard_queued().unwrap(), 2);

    let remaining: Vec<i64> = db.list().unwrap().into_iter().map(|item| item.id).collect();
    assert!(!remaining.contains(&waiting_one.id));
    assert!(!remaining.contains(&waiting_two.id));
    // Mid-flight work belongs to a session and has to reach its own end; the
    // review is a human's to decide, and terminal rows carry rename receipts.
    assert!(remaining.contains(&in_flight.id));
    assert!(remaining.contains(&awaiting_decision.id));
    assert!(remaining.contains(&canceled.id));

    // Idempotent: nothing left to discard.
    assert_eq!(db.discard_queued().unwrap(), 0);
}

#[test]
fn explicit_manual_retry_and_keep_original_cas_paths_are_enforced() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);

    let retry = db.enqueue(Path::new("retry.pdf"), "hr").unwrap();
    for _ in 0..2 {
        db.claim_next().unwrap().unwrap();
        db.record_processing_failure(retry.id, ErrorCode::ModelOutputInvalid)
            .unwrap();
    }
    assert_eq!(
        db.manual_retry(retry.id).unwrap().status,
        QueueStatus::Queued
    );
    assert_eq!(
        db.manual_retry(retry.id).unwrap_err().code(),
        ErrorCode::StateConflict
    );

    let keep = db.enqueue(Path::new("keep.pdf"), "hk").unwrap();
    let claimed = db.claim_next().unwrap().unwrap();
    assert_eq!(claimed.id, retry.id);
    db.transition(
        retry.id,
        QueueStatus::Extracting,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();
    let claimed = db.claim_next().unwrap().unwrap();
    assert_eq!(claimed.id, keep.id);
    db.transition(
        keep.id,
        QueueStatus::Extracting,
        QueueStatus::Analyzing,
        None,
    )
    .unwrap();
    db.transition(keep.id, QueueStatus::Analyzing, QueueStatus::Ready, None)
        .unwrap();
    assert_eq!(
        db.complete_keep_original(keep.id, QueueStatus::Ready)
            .unwrap()
            .status,
        QueueStatus::Completed
    );
}

#[test]
fn legacy_schema_migrates_without_discarding_queue_or_receipt_rows() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("legacy.sqlite3");
    let legacy = rusqlite::Connection::open(&path).unwrap();
    legacy
        .execute_batch(
            "PRAGMA foreign_keys=ON;
         CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);
         CREATE TABLE queue_items(
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           source_path TEXT NOT NULL,
           source_path_key TEXT NOT NULL,
           source_hash TEXT NOT NULL,
           status TEXT NOT NULL,
           processing_failures INTEGER NOT NULL DEFAULT 0,
           error_code TEXT,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           UNIQUE(source_path_key, source_hash)
         );
         CREATE TABLE operation_receipts(
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           queue_item_id INTEGER NOT NULL REFERENCES queue_items(id) ON DELETE CASCADE,
           receipt_json TEXT NOT NULL,
           created_at INTEGER NOT NULL
         );
         INSERT INTO schema_migrations VALUES(1, 1);
         INSERT INTO queue_items(
           source_path, source_path_key, source_hash, status, created_at, updated_at
         ) VALUES('legacy.pdf', 'legacy.pdf', 'hash', 'queued', 1, 1);
         INSERT INTO operation_receipts(queue_item_id, receipt_json, created_at)
           VALUES(1, '{\"legacy\":true}', 1);",
        )
        .unwrap();
    drop(legacy);

    let migrated = QueueStore::open(&path).unwrap();
    assert_eq!(
        migrated.list().unwrap()[0].source_path,
        Path::new("legacy.pdf")
    );
    drop(migrated);
    let inspected = rusqlite::Connection::open(path).unwrap();
    assert_eq!(
        inspected
            .query_row(
                "SELECT receipt_json FROM operation_receipts_legacy_v1 WHERE id = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "{\"legacy\":true}",
    );
}

#[test]
fn v2_duplicate_nonterminal_receipts_open_unbound_and_fail_closed() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("v2-duplicates.sqlite3");
    let legacy = rusqlite::Connection::open(&path).unwrap();
    legacy.execute_batch(
        "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);
         CREATE TABLE queue_items(
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           source_path TEXT NOT NULL,
           source_path_key TEXT NOT NULL,
           source_hash TEXT NOT NULL,
           status TEXT NOT NULL,
           processing_failures INTEGER NOT NULL DEFAULT 0,
           error_code TEXT,
           owner_session TEXT,
           lease_expires_at INTEGER,
           previous_status TEXT,
           reconciliation_receipt_id INTEGER,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           UNIQUE(source_path_key, source_hash)
         );
         CREATE TABLE operation_receipts(
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           queue_item_id INTEGER NOT NULL REFERENCES queue_items(id) ON DELETE CASCADE,
           direction TEXT NOT NULL,
           source_path TEXT NOT NULL,
           destination_path TEXT NOT NULL,
           temporary_path TEXT,
           pre_hash TEXT NOT NULL,
           post_hash TEXT,
           operation_kind TEXT NOT NULL,
           stage TEXT NOT NULL,
           source_exists INTEGER NOT NULL,
           destination_exists INTEGER NOT NULL,
           temporary_exists INTEGER NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
         );
         INSERT INTO schema_migrations VALUES(2, 1);
         INSERT INTO queue_items(
           source_path, source_path_key, source_hash, status, previous_status, created_at, updated_at
         ) VALUES('source.pdf', 'source.pdf', 'hash', 'applying', 'ready', 1, 1);
         INSERT INTO operation_receipts(
           queue_item_id, direction, source_path, destination_path, pre_hash,
           operation_kind, stage, source_exists, destination_exists, temporary_exists,
           created_at, updated_at
         ) VALUES
           (1, 'apply', 'source.pdf', 'one.pdf', 'hash', 'rename', 'planned', 1, 0, 0, 1, 1),
           (1, 'apply', 'source.pdf', 'two.pdf', 'hash', 'rename', 'planned', 1, 0, 0, 1, 1);",
    ).unwrap();
    drop(legacy);

    let migrated = Arc::new(QueueStore::open(&path).unwrap());
    let items = migrated.list().unwrap();
    let item = &items[0];
    assert_eq!(item.status, QueueStatus::Applying);
    assert_eq!(item.active_receipt_id, None);
    migrated.claim_applying_reconciliation(item.id).unwrap();
    assert_eq!(
        FileApplier::local(migrated.clone())
            .reconcile(item.id)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict,
    );
    assert_eq!(migrated.list().unwrap()[0].status, QueueStatus::Applying);
    drop(migrated);
    let inspected = rusqlite::Connection::open(path).unwrap();
    assert_eq!(
        inspected
            .query_row("SELECT COUNT(*) FROM operation_receipts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2,
    );
}

fn create_v2_complete_epoch(
    database: &Path,
    queue_source: &Path,
    receipt_source: &Path,
    receipt_destination: &Path,
    direction: &str,
    previous_status: &str,
    hash: &str,
) {
    let connection = rusqlite::Connection::open(database).unwrap();
    connection.execute_batch(
        "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);
         CREATE TABLE queue_items(
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           source_path TEXT NOT NULL,
           source_path_key TEXT NOT NULL,
           source_hash TEXT NOT NULL,
           status TEXT NOT NULL,
           processing_failures INTEGER NOT NULL DEFAULT 0,
           error_code TEXT,
           owner_session TEXT,
           lease_expires_at INTEGER,
           previous_status TEXT,
           reconciliation_receipt_id INTEGER,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           UNIQUE(source_path_key, source_hash)
         );
         CREATE TABLE operation_receipts(
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           queue_item_id INTEGER NOT NULL REFERENCES queue_items(id) ON DELETE CASCADE,
           direction TEXT NOT NULL,
           source_path TEXT NOT NULL,
           destination_path TEXT NOT NULL,
           temporary_path TEXT,
           pre_hash TEXT NOT NULL,
           post_hash TEXT,
           operation_kind TEXT NOT NULL,
           stage TEXT NOT NULL,
           source_exists INTEGER NOT NULL,
           destination_exists INTEGER NOT NULL,
           temporary_exists INTEGER NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
         );
         INSERT INTO schema_migrations VALUES(2, 1);",
    ).unwrap();
    let queue_source = queue_source.to_string_lossy().into_owned();
    let receipt_source = receipt_source.to_string_lossy().into_owned();
    let receipt_destination = receipt_destination.to_string_lossy().into_owned();
    connection.execute(
        "INSERT INTO queue_items(
           source_path, source_path_key, source_hash, status, previous_status, created_at, updated_at
         ) VALUES(?1, ?1, ?2, 'applying', ?3, 1, 1)",
        rusqlite::params![queue_source, hash, previous_status],
    ).unwrap();
    connection
        .execute(
            "INSERT INTO operation_receipts(
           queue_item_id, direction, source_path, destination_path, pre_hash, post_hash,
           operation_kind, stage, source_exists, destination_exists, temporary_exists,
           created_at, updated_at
         ) VALUES(1, ?1, ?2, ?3, ?4, ?4, 'rename', 'complete', 0, 1, 0, 1, 1)",
            rusqlite::params![direction, receipt_source, receipt_destination, hash],
        )
        .unwrap();
}

fn insert_v2_complete_receipt(
    database: &Path,
    source: &Path,
    destination: &Path,
    direction: &str,
    hash: &str,
) {
    let connection = rusqlite::Connection::open(database).unwrap();
    connection
        .execute(
            "INSERT INTO operation_receipts(
           queue_item_id, direction, source_path, destination_path, pre_hash, post_hash,
           operation_kind, stage, source_exists, destination_exists, temporary_exists,
           created_at, updated_at
         ) VALUES(1, ?1, ?2, ?3, ?4, ?4, 'rename', 'complete', 0, 1, 0, 1, 1)",
            rusqlite::params![
                direction,
                source.to_string_lossy().into_owned(),
                destination.to_string_lossy().into_owned(),
                hash,
            ],
        )
        .unwrap();
}

#[test]
fn v2_complete_apply_receipt_binds_and_reconciles_after_migration() {
    let temp = TempDir::new().unwrap();
    let database = temp.path().join("v2-apply-complete.sqlite3");
    let original = temp.path().join("source.pdf");
    let published = temp.path().join("named.pdf");
    fs::write(&published, b"original").unwrap();
    let hash = StdFileSystem.hash(&published).unwrap();
    create_v2_complete_epoch(
        &database, &original, &original, &published, "apply", "ready", &hash,
    );

    let store = Arc::new(QueueStore::open(database).unwrap());
    let item = store.list().unwrap().into_iter().next().unwrap();
    assert_eq!(item.active_receipt_id, Some(1));
    store.claim_applying_reconciliation(item.id).unwrap();
    let resolved = FileApplier::local(store).reconcile(item.id).unwrap();
    assert_eq!(resolved.status, QueueStatus::Completed);
    assert!(!original.exists());
    assert!(published.exists());
}

#[test]
fn v2_complete_undo_receipt_binds_and_reconciles_after_migration() {
    let temp = TempDir::new().unwrap();
    let database = temp.path().join("v2-undo-complete.sqlite3");
    let original = temp.path().join("source.pdf");
    let published = temp.path().join("named.pdf");
    fs::write(&original, b"original").unwrap();
    let hash = StdFileSystem.hash(&original).unwrap();
    create_v2_complete_epoch(
        &database,
        &original,
        &published,
        &original,
        "undo",
        "completed",
        &hash,
    );

    let store = Arc::new(QueueStore::open(database).unwrap());
    let item = store.list().unwrap().into_iter().next().unwrap();
    assert_eq!(item.active_receipt_id, Some(1));
    store.claim_applying_reconciliation(item.id).unwrap();
    let resolved = FileApplier::local(store).reconcile(item.id).unwrap();
    assert_eq!(resolved.status, QueueStatus::Ready);
    assert!(original.exists());
    assert!(!published.exists());
}

#[test]
fn v2_empty_second_apply_does_not_bind_historical_first_apply_receipt() {
    let temp = TempDir::new().unwrap();
    let database = temp.path().join("v2-empty-apply-two.sqlite3");
    let original = temp.path().join("source.pdf");
    let first_published = temp.path().join("first.pdf");
    fs::write(&original, b"original").unwrap();
    let hash = StdFileSystem.hash(&original).unwrap();

    create_v2_complete_epoch(
        &database,
        &original,
        &original,
        &first_published,
        "apply",
        "ready",
        &hash,
    );
    insert_v2_complete_receipt(&database, &first_published, &original, "undo", &hash);

    let store = Arc::new(QueueStore::open(database).unwrap());
    let item = store.list().unwrap().into_iter().next().unwrap();
    assert_eq!(item.active_receipt_id, None);
    store.claim_applying_reconciliation(item.id).unwrap();
    assert_eq!(
        FileApplier::local(store.clone())
            .reconcile(item.id)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict,
    );
    assert_eq!(store.list().unwrap()[0].status, QueueStatus::Applying);
    assert!(original.exists());
    assert!(!first_published.exists());
}

#[test]
fn v2_empty_second_undo_does_not_bind_historical_first_undo_receipt() {
    let temp = TempDir::new().unwrap();
    let database = temp.path().join("v2-empty-undo-two.sqlite3");
    let original = temp.path().join("source.pdf");
    let first_published = temp.path().join("first.pdf");
    let second_published = temp.path().join("second.pdf");
    fs::write(&second_published, b"original").unwrap();
    let hash = StdFileSystem.hash(&second_published).unwrap();

    create_v2_complete_epoch(
        &database,
        &original,
        &original,
        &first_published,
        "apply",
        "completed",
        &hash,
    );
    insert_v2_complete_receipt(&database, &first_published, &original, "undo", &hash);
    insert_v2_complete_receipt(&database, &original, &second_published, "apply", &hash);

    let store = Arc::new(QueueStore::open(database).unwrap());
    let item = store.list().unwrap().into_iter().next().unwrap();
    assert_eq!(item.active_receipt_id, None);
    store.claim_applying_reconciliation(item.id).unwrap();
    assert_eq!(
        FileApplier::local(store.clone())
            .reconcile(item.id)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict,
    );
    assert_eq!(store.list().unwrap()[0].status, QueueStatus::Applying);
    assert!(!original.exists());
    assert!(second_published.exists());
}

#[test]
fn find_completed_duplicate_reports_the_filed_name_and_skips_pending_or_undone_items() {
    let temp = TempDir::new().unwrap();
    let db = Arc::new(store(&temp));
    let original = temp.path().join("original.pdf");
    fs::write(&original, b"same-content").unwrap();
    let hash = StdFileSystem.hash(&original).unwrap();
    let filed = temp.path().join("2024 - Filed Agreement.pdf");
    let completed_id = complete_via_apply(&db, &original, &filed);

    // A pending twin holding the same content has been filed nowhere yet.
    let pending = temp.path().join("pending.pdf");
    fs::write(&pending, b"same-content").unwrap();
    db.enqueue(&pending, &hash).unwrap();

    let incoming_key = source_path_key(&temp.path().join("incoming.pdf"));
    let found = db
        .find_completed_duplicate(&hash, &incoming_key)
        .unwrap()
        .unwrap();
    assert_eq!(found.queue_item_id, completed_id);
    assert_eq!(
        found.filed_as.as_deref(),
        Some("2024 - Filed Agreement.pdf")
    );
    assert_eq!(found.source_path, original);

    // The completed item's own path is excluded: re-adding the same file at
    // the same place is the existing same-item dedupe, not a duplicate flag.
    assert!(
        db.find_completed_duplicate(&hash, &source_path_key(&original))
            .unwrap()
            .is_none()
    );

    // Undo returns the content home and the item to Ready; it no longer
    // counts as a filed duplicate.
    db.begin_applying(completed_id, QueueStatus::Completed)
        .unwrap();
    let receipt = db.load_receipt(completed_id).unwrap().unwrap();
    let undo = FileApplier::local(Arc::clone(&db))
        .undo(completed_id, &receipt)
        .unwrap();
    db.complete_undo(completed_id, undo.id).unwrap();
    assert!(
        db.find_completed_duplicate(&hash, &incoming_key)
            .unwrap()
            .is_none()
    );
}

/// An operation whose rename cannot land. The undo it refuses is journalled,
/// fails, and reconciles as rolled back - which leaves the item's newest
/// receipt a rolled-back undo sitting on top of the apply that filed it.
struct RenameRefusedFileSystem;

impl FileSystem for RenameRefusedFileSystem {
    fn exists(&self, path: &Path) -> bool {
        StdFileSystem.exists(path)
    }
    fn hash(&self, path: &Path) -> io::Result<String> {
        StdFileSystem.hash(path)
    }
    fn same_volume(&self, source: &Path, destination: &Path) -> io::Result<bool> {
        StdFileSystem.same_volume(source, destination)
    }
    fn rename_no_replace(&self, _source: &Path, _destination: &Path) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "injected rename refusal",
        ))
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

#[test]
fn the_duplicate_lookup_is_indexed_on_a_fresh_and_on_a_migrated_database() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("queue.sqlite3");
    drop(QueueStore::open(&path).unwrap());
    // Reopening is what an upgraded install does, and the index has to arrive
    // for a database that already exists as well as for a new one.
    drop(QueueStore::open(&path).unwrap());

    let connection = rusqlite::Connection::open(&path).unwrap();
    let plan = connection
        .query_row(
            "EXPLAIN QUERY PLAN
             SELECT id FROM queue_items
             WHERE status = 'completed' AND source_hash = ?1 AND source_path_key <> ?2",
            ["hash", "key"],
            |row| row.get::<_, String>(3),
        )
        .unwrap();
    assert!(
        plan.contains("queue_items_source_hash"),
        "the duplicate lookup should read an index, not scan: {plan}"
    );
}

#[test]
fn duplicate_report_names_the_applied_file_after_a_rolled_back_undo() {
    let temp = TempDir::new().unwrap();
    let db = Arc::new(store(&temp));
    let original = temp.path().join("original.pdf");
    fs::write(&original, b"same-content").unwrap();
    let hash = StdFileSystem.hash(&original).unwrap();
    let filed = temp.path().join("2024 - Filed Agreement.pdf");
    let completed_id = complete_via_apply(&db, &original, &filed);

    // An undo that cannot rename is journalled, fails, and reconciles as rolled
    // back. The content never moved: it is still at the filed name.
    db.begin_applying(completed_id, QueueStatus::Completed)
        .unwrap();
    let applied = db.load_receipt(completed_id).unwrap().unwrap();
    let refused = FileApplier::new(Arc::new(RenameRefusedFileSystem), Arc::clone(&db));
    refused.undo(completed_id, &applied).unwrap_err();
    let restored = refused.reconcile(completed_id).unwrap();
    assert_eq!(restored.status, QueueStatus::Completed);
    assert_eq!(
        db.load_receipt(completed_id).unwrap().unwrap().stage,
        OperationStage::RolledBack
    );
    assert!(filed.exists());

    // The rolled-back undo is the newest receipt but it filed nothing, so the
    // name a duplicate is reported under is still the one the apply gave it.
    let found = db
        .find_completed_duplicate(&hash, &source_path_key(&temp.path().join("incoming.pdf")))
        .unwrap()
        .unwrap();
    assert_eq!(found.queue_item_id, completed_id);
    assert_eq!(
        found.filed_as.as_deref(),
        Some("2024 - Filed Agreement.pdf")
    );
}

#[test]
fn most_recent_completed_duplicate_wins_and_keep_original_reports_no_filed_name() {
    let temp = TempDir::new().unwrap();
    let db = Arc::new(store(&temp));
    let first = temp.path().join("first.pdf");
    fs::write(&first, b"shared-content").unwrap();
    let hash = StdFileSystem.hash(&first).unwrap();
    complete_via_apply(&db, &first, &temp.path().join("First Filed.pdf"));

    let second = temp.path().join("second.pdf");
    fs::write(&second, b"shared-content").unwrap();
    let kept = db.enqueue(&second, &hash).unwrap();
    advance_to_ready(&db, kept.id);
    db.complete_keep_original(kept.id, QueueStatus::Ready)
        .unwrap();

    let found = db
        .find_completed_duplicate(&hash, &source_path_key(&temp.path().join("third.pdf")))
        .unwrap()
        .unwrap();
    assert_eq!(found.queue_item_id, kept.id);
    assert_eq!(found.filed_as, None);
    assert_eq!(found.source_path, second);
}

#[test]
fn operation_history_lists_finished_work_newest_first_and_hides_in_flight_receipts() {
    let temp = TempDir::new().unwrap();
    let db = Arc::new(store(&temp));
    let original = temp.path().join("scan.pdf");
    fs::write(&original, b"content").unwrap();
    let filed = temp.path().join("2024-04-12 Employment Agreement.pdf");
    let item_id = complete_via_apply(&db, &original, &filed);

    // Undoing the rename records a second, newer terminal receipt.
    db.begin_applying(item_id, QueueStatus::Completed).unwrap();
    let applied = db.load_receipt(item_id).unwrap().unwrap();
    let undo = FileApplier::local(Arc::clone(&db))
        .undo(item_id, &applied)
        .unwrap();
    db.complete_undo(item_id, undo.id).unwrap();

    // A receipt still mid-operation is applier bookkeeping, not history.
    let pending = db
        .enqueue(Path::new("in-flight.pdf"), "in-flight-hash")
        .unwrap();
    let inspector = rusqlite::Connection::open(temp.path().join("queue.sqlite3")).unwrap();
    inspector
        .execute(
            "INSERT INTO operation_receipts(
               queue_item_id, direction, source_path, destination_path, pre_hash,
               operation_kind, stage, source_exists, destination_exists, temporary_exists,
               created_at, updated_at
             ) VALUES(?1, 'apply', 'in-flight.pdf', 'renamed.pdf', 'in-flight-hash',
                      'rename', 'published', 0, 1, 0, 9999999999, 9999999999)",
            [pending.id],
        )
        .unwrap();

    let history = db.list_operation_history(10).unwrap();
    assert_eq!(history.len(), 2);
    let newest = &history[0];
    assert_eq!(newest.receipt_id, undo.id);
    assert_eq!(newest.queue_item_id, item_id);
    assert_eq!(newest.direction, OperationDirection::Undo);
    assert_eq!(newest.stage, OperationStage::Complete);
    assert_eq!(newest.original_path, filed);
    assert_eq!(newest.new_path, original);
    let earlier = &history[1];
    assert_eq!(earlier.queue_item_id, item_id);
    assert_eq!(earlier.direction, OperationDirection::Apply);
    assert_eq!(earlier.kind, OperationKind::Rename);
    assert_eq!(earlier.stage, OperationStage::Complete);
    assert_eq!(earlier.original_path, original);
    assert_eq!(earlier.new_path, filed);
    assert!(newest.at >= earlier.at);
}

#[test]
fn operation_history_honors_the_requested_limit_and_caps_at_five_hundred() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let item = db.enqueue(Path::new("bulk.pdf"), "bulk-hash").unwrap();
    let inspector = rusqlite::Connection::open(temp.path().join("queue.sqlite3")).unwrap();
    let mut insert = inspector
        .prepare(
            "INSERT INTO operation_receipts(
               queue_item_id, direction, source_path, destination_path, pre_hash,
               operation_kind, stage, source_exists, destination_exists, temporary_exists,
               created_at, updated_at
             ) VALUES(?1, 'apply', 'bulk.pdf', 'renamed.pdf', 'bulk-hash',
                      'rename', 'complete', 0, 1, 0, ?2, ?2)",
        )
        .unwrap();
    let total = HISTORY_LIMIT + 5;
    for moment in 0..total {
        insert
            .execute(rusqlite::params![item.id, moment as i64])
            .unwrap();
    }

    assert_eq!(db.list_operation_history(3).unwrap().len(), 3);
    let capped = db.list_operation_history(total + 100).unwrap();
    assert_eq!(capped.len(), HISTORY_LIMIT);
    assert_eq!(capped[0].at, (total - 1) as i64);
    assert!(capped.windows(2).all(|pair| pair[0].at >= pair[1].at));
}

#[test]
fn duplicate_flag_and_retry_are_compare_and_swap_guarded() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let item = db.enqueue(Path::new("dup.pdf"), "h1").unwrap();

    let flagged = db
        .transition(
            item.id,
            QueueStatus::Queued,
            QueueStatus::NeedsReview,
            Some(ErrorCode::Duplicate),
        )
        .unwrap();
    assert_eq!(flagged.status, QueueStatus::NeedsReview);
    assert_eq!(flagged.error_code, Some(ErrorCode::Duplicate));

    let retried = db.retry_duplicate(item.id).unwrap();
    assert_eq!(retried.status, QueueStatus::Queued);
    assert_eq!(retried.error_code, None);

    // Once a session claims the item, the flag loses the race and the claim
    // is left untouched.
    assert_eq!(db.claim_next().unwrap().unwrap().id, item.id);
    let conflict = db
        .transition(
            item.id,
            QueueStatus::Queued,
            QueueStatus::NeedsReview,
            Some(ErrorCode::Duplicate),
        )
        .unwrap_err();
    assert_eq!(conflict.code(), ErrorCode::StateConflict);
    assert_eq!(db.list().unwrap()[0].status, QueueStatus::Extracting);

    // A review item that is not a duplicate cannot take the requeue shortcut.
    db.transition(
        item.id,
        QueueStatus::Extracting,
        QueueStatus::Analyzing,
        None,
    )
    .unwrap();
    db.transition(
        item.id,
        QueueStatus::Analyzing,
        QueueStatus::NeedsReview,
        None,
    )
    .unwrap();
    assert_eq!(
        db.retry_duplicate(item.id).unwrap_err().code(),
        ErrorCode::StateConflict
    );
}

#[test]
fn the_newest_item_for_a_path_is_found_by_key_across_spellings() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let scan = temp.path().join("Scans").join("Contract.PDF");
    fs::create_dir_all(scan.parent().unwrap()).unwrap();
    let first = db.enqueue(&scan, "hash-one").unwrap();
    let second = db.enqueue(&scan, "hash-two").unwrap();
    assert!(second.id > first.id);

    let found = db
        .find_newest_by_source_path(&[scan.as_path()])
        .unwrap()
        .unwrap();
    assert_eq!(
        found.id, second.id,
        "the newest row is the path's current fate"
    );

    let respelled = temp.path().join("scans").join("contract.pdf");
    let found = db
        .find_newest_by_source_path(&[respelled.as_path()])
        .unwrap()
        .unwrap();
    assert_eq!(found.id, second.id, "keys fold case like enqueue does");

    let elsewhere = temp.path().join("elsewhere.pdf");
    assert!(
        db.find_newest_by_source_path(&[elsewhere.as_path()])
            .unwrap()
            .is_none()
    );
    assert!(
        db.find_newest_by_source_path(&[elsewhere.as_path(), scan.as_path()])
            .unwrap()
            .is_some(),
        "any offered spelling may match"
    );
    assert!(db.find_newest_by_source_path(&[]).unwrap().is_none());
}

#[test]
fn undo_after_rolled_back_undo_succeeds() {
    let temp = TempDir::new().unwrap();
    let db = Arc::new(store(&temp));
    let original = temp.path().join("original.pdf");
    fs::write(&original, b"same-content").unwrap();
    let filed = temp.path().join("2024 - Filed Agreement.pdf");
    let id = complete_via_apply(&db, &original, &filed);
    let applied = db.load_latest_complete_receipt(id).unwrap().unwrap();

    // The first undo is refused - the filed document is open somewhere - and
    // reconciles as rolled back, which makes it the item's newest receipt.
    db.begin_undo(id).unwrap();
    let refused = FileApplier::new(Arc::new(RenameRefusedFileSystem), Arc::clone(&db));
    refused.undo(id, &applied).unwrap_err();
    assert_eq!(
        refused.reconcile(id).unwrap().status,
        QueueStatus::Completed
    );
    assert_eq!(
        db.load_receipt(id).unwrap().unwrap().stage,
        OperationStage::RolledBack
    );
    // ... but the document is still filed by the apply underneath it.
    assert_eq!(
        db.load_latest_complete_receipt(id).unwrap().unwrap(),
        applied
    );

    // So the next undo, with the document closed, goes through.
    db.begin_undo(id).unwrap();
    let undone = FileApplier::local(Arc::clone(&db))
        .undo(id, &db.load_latest_complete_receipt(id).unwrap().unwrap())
        .unwrap();
    assert_eq!(
        db.complete_undo(id, undone.id).unwrap().status,
        QueueStatus::Ready
    );
    assert_eq!(fs::read(&original).unwrap(), b"same-content");
    assert!(!filed.exists());
    assert_eq!(
        db.load_latest_complete_receipt(id)
            .unwrap()
            .unwrap()
            .direction,
        OperationDirection::Undo
    );
}

/// What `park_applying_for_review` left behind before parks could be checked
/// again, after a later apply attempt cleared the reconciliation pointer: a
/// row in review, holding no receipt, and an unfinished receipt beside it.
fn park_with_live_receipt(database: &Path, id: i64) {
    rusqlite::Connection::open(database)
        .unwrap()
        .execute(
            "UPDATE queue_items
             SET status = 'needs_review', error_code = 'FILE_CHANGED',
                 active_receipt_id = NULL, reconciliation_receipt_id = NULL,
                 previous_status = NULL, owner_session = NULL, lease_expires_at = NULL
             WHERE id = ?1",
            [id],
        )
        .unwrap();
}

#[test]
fn reattach_unsettled_receipt_restores_previous_status_by_direction() {
    let temp = TempDir::new().unwrap();
    let database = temp.path().join("queue.sqlite3");
    let db = Arc::new(store(&temp));

    // An apply whose rename was refused, parked with its receipt planned.
    let waiting = temp.path().join("waiting.pdf");
    fs::write(&waiting, b"waiting").unwrap();
    let hash = StdFileSystem.hash(&waiting).unwrap();
    let apply_item = db.enqueue(&waiting, &hash).unwrap().id;
    advance_to_ready(&db, apply_item);
    db.begin_applying(apply_item, QueueStatus::Ready).unwrap();
    FileApplier::new(Arc::new(RenameRefusedFileSystem), Arc::clone(&db))
        .apply(apply_item, &waiting, &temp.path().join("named.pdf"), &hash)
        .unwrap_err();
    park_with_live_receipt(&database, apply_item);
    let parked = db.load_unsettled_receipt(apply_item).unwrap().unwrap();

    let reattached = db.reattach_unsettled_receipt(apply_item).unwrap().unwrap();

    assert_eq!(reattached, parked);
    let row = db.list().unwrap().remove(0);
    assert_eq!(row.status, QueueStatus::Applying);
    assert_eq!(row.previous_status, Some(QueueStatus::Ready));
    assert_eq!(row.active_receipt_id, Some(parked.id));
    assert_eq!(row.owner_session.as_deref(), Some(db.session_id()));
    // Bound like that, the ordinary reconciliation settles it.
    assert_eq!(
        FileApplier::local(Arc::clone(&db))
            .reconcile(apply_item)
            .unwrap()
            .status,
        QueueStatus::Ready
    );
    assert!(db.reattach_unsettled_receipt(apply_item).unwrap().is_none());

    // An undo whose rename was refused goes back to completed: the document
    // is still filed.
    let original = temp.path().join("original.pdf");
    fs::write(&original, b"filed").unwrap();
    let filed = temp.path().join("Filed.pdf");
    let undo_item = complete_via_apply(&db, &original, &filed);
    db.begin_undo(undo_item).unwrap();
    let applied = db.load_latest_complete_receipt(undo_item).unwrap().unwrap();
    FileApplier::new(Arc::new(RenameRefusedFileSystem), Arc::clone(&db))
        .undo(undo_item, &applied)
        .unwrap_err();
    park_with_live_receipt(&database, undo_item);

    // Not while another operation is applying...
    db.begin_applying(apply_item, QueueStatus::Ready).unwrap();
    assert_eq!(
        db.reattach_unsettled_receipt(undo_item).unwrap_err().code(),
        ErrorCode::StateConflict
    );
    FileApplier::local(Arc::clone(&db))
        .reconcile(apply_item)
        .unwrap();

    // ... and then by the undo's own direction.
    let reattached = db.reattach_unsettled_receipt(undo_item).unwrap().unwrap();
    assert_eq!(reattached.direction, OperationDirection::Undo);
    let row = db
        .list()
        .unwrap()
        .into_iter()
        .find(|item| item.id == undo_item)
        .unwrap();
    assert_eq!(row.status, QueueStatus::Applying);
    assert_eq!(row.previous_status, Some(QueueStatus::Completed));
    assert_eq!(
        FileApplier::local(Arc::clone(&db))
            .reconcile(undo_item)
            .unwrap()
            .status,
        QueueStatus::Completed
    );
    assert!(filed.exists());
}

#[test]
fn begin_undo_refuses_only_while_another_row_applies() {
    let temp = TempDir::new().unwrap();
    let db = Arc::new(store(&temp));
    let original = temp.path().join("original.pdf");
    fs::write(&original, b"filed").unwrap();
    let filed_id = complete_via_apply(&db, &original, &temp.path().join("Filed.pdf"));

    // Another document is being read.
    let reading = db
        .enqueue(Path::new("reading.pdf"), "reading-hash")
        .unwrap();
    assert_eq!(db.claim_next().unwrap().unwrap().id, reading.id);

    // A new rename still waits for it; an undo does not.
    assert_eq!(
        db.begin_applying(filed_id, QueueStatus::Completed)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict
    );
    assert_eq!(
        db.begin_undo(filed_id).unwrap().status,
        QueueStatus::Applying
    );
    // While the undo applies, nothing new is claimed.
    db.transition(
        reading.id,
        QueueStatus::Extracting,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();
    db.enqueue(Path::new("next.pdf"), "next-hash").unwrap();
    assert!(db.claim_next().unwrap().is_none());
    assert_eq!(
        FileApplier::local(Arc::clone(&db))
            .reconcile(filed_id)
            .unwrap()
            .status,
        QueueStatus::Completed
    );

    // Another row applying is the one thing an undo waits for.
    let other = temp.path().join("other.pdf");
    fs::write(&other, b"other").unwrap();
    let other_id = db
        .enqueue(&other, &StdFileSystem.hash(&other).unwrap())
        .unwrap()
        .id;
    let next = db.claim_next().unwrap().unwrap();
    db.transition(
        next.id,
        QueueStatus::Extracting,
        QueueStatus::Canceled,
        None,
    )
    .unwrap();
    advance_to_ready(&db, other_id);
    db.begin_applying(other_id, QueueStatus::Ready).unwrap();
    assert_eq!(
        db.begin_undo(filed_id).unwrap_err().code(),
        ErrorCode::StateConflict
    );
}

#[test]
fn receipt_updated_at_reports_filing_time() {
    let temp = TempDir::new().unwrap();
    let db = Arc::new(store(&temp));
    let started = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let original = temp.path().join("original.pdf");
    fs::write(&original, b"content").unwrap();
    let id = complete_via_apply(&db, &original, &temp.path().join("Filed.pdf"));
    let receipt = db.load_latest_complete_receipt(id).unwrap().unwrap();

    let filed_at = db.receipt_updated_at(receipt.id).unwrap().unwrap();

    assert!(filed_at >= started, "{filed_at} < {started}");
    assert_eq!(db.receipt_updated_at(receipt.id + 1000).unwrap(), None);
}

fn proposal_rows(database: &Path, id: i64) -> i64 {
    rusqlite::Connection::open(database)
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM proposals WHERE queue_item_id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap()
}

/// Re-analyze takes a reviewed document back to the queue with nothing left
/// of the earlier reading, and rekeys it to the file as it is now.
#[test]
fn requeue_for_analysis_resets_and_rekeys() {
    let temp = TempDir::new().unwrap();
    let database = temp.path().join("queue.sqlite3");
    let db = store(&temp);
    let path = Path::new("C:/scans/signed.pdf");
    let item = db.enqueue(path, "before-signing").unwrap();
    // One failed attempt, then a reading that went to review.
    assert_eq!(db.claim_next().unwrap().unwrap().id, item.id);
    db.record_processing_failure(item.id, ErrorCode::ModelFailed)
        .unwrap();
    assert_eq!(db.claim_next().unwrap().unwrap().id, item.id);
    db.transition(
        item.id,
        QueueStatus::Extracting,
        QueueStatus::Analyzing,
        None,
    )
    .unwrap();
    db.transition(
        item.id,
        QueueStatus::Analyzing,
        QueueStatus::NeedsReview,
        Some(ErrorCode::FileChanged),
    )
    .unwrap();
    let inspector = rusqlite::Connection::open(&database).unwrap();
    inspector
        .execute(
            "INSERT INTO proposals(queue_item_id, proposal_json, created_at)
             VALUES (?1, '{\"approved\":true}', 0)",
            [item.id],
        )
        .unwrap();
    inspector
        .execute(
            "UPDATE queue_items SET reconciliation_receipt_id = 41, previous_status = 'ready'
             WHERE id = ?1",
            [item.id],
        )
        .unwrap();

    // Only from review or ready, and only from the state the caller saw.
    assert_eq!(
        db.requeue_for_analysis(item.id, QueueStatus::Completed, None)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        db.requeue_for_analysis(item.id, QueueStatus::Ready, None)
            .unwrap_err()
            .code(),
        ErrorCode::StateConflict
    );
    assert_eq!(
        proposal_rows(&database, item.id),
        1,
        "a refusal changes nothing"
    );

    let requeued = db
        .requeue_for_analysis(item.id, QueueStatus::NeedsReview, Some("after-signing"))
        .unwrap();

    assert_eq!(requeued.id, item.id);
    assert_eq!(requeued.status, QueueStatus::Queued);
    assert_eq!(requeued.source_hash, "after-signing");
    assert_eq!(requeued.processing_failures, 0);
    assert_eq!(requeued.error_code, None);
    assert_eq!(requeued.previous_status, None);
    assert_eq!(requeued.active_receipt_id, None);
    assert_eq!(requeued.reconciliation_receipt_id, None);
    assert_eq!(requeued.owner_session, None);
    assert_eq!(
        proposal_rows(&database, item.id),
        0,
        "the proposal, and the approval in it, went with the reading"
    );
    // The row is the file as it is now: the new version finds it, and the
    // old version would be a new document.
    assert_eq!(db.enqueue(path, "after-signing").unwrap().id, item.id);
    assert_ne!(db.enqueue(path, "before-signing").unwrap().id, item.id);

    // Without a new fingerprint the stored one stays.
    assert_eq!(db.claim_next().unwrap().unwrap().id, item.id);
    db.transition(
        item.id,
        QueueStatus::Extracting,
        QueueStatus::Analyzing,
        None,
    )
    .unwrap();
    db.transition(item.id, QueueStatus::Analyzing, QueueStatus::Ready, None)
        .unwrap();
    let again = db
        .requeue_for_analysis(item.id, QueueStatus::Ready, None)
        .unwrap();
    assert_eq!(again.status, QueueStatus::Queued);
    assert_eq!(again.source_hash, "after-signing");
}

#[test]
fn requeue_refused_with_unsettled_receipt() {
    let temp = TempDir::new().unwrap();
    let database = temp.path().join("queue.sqlite3");
    let db = Arc::new(store(&temp));
    let waiting = temp.path().join("waiting.pdf");
    fs::write(&waiting, b"waiting").unwrap();
    let hash = StdFileSystem.hash(&waiting).unwrap();
    let id = db.enqueue(&waiting, &hash).unwrap().id;
    advance_to_ready(&db, id);
    db.begin_applying(id, QueueStatus::Ready).unwrap();
    FileApplier::new(Arc::new(RenameRefusedFileSystem), Arc::clone(&db))
        .apply(id, &waiting, &temp.path().join("named.pdf"), &hash)
        .unwrap_err();
    park_with_live_receipt(&database, id);
    assert!(db.load_unsettled_receipt(id).unwrap().is_some());

    let refused = db
        .requeue_for_analysis(id, QueueStatus::NeedsReview, Some("edited"))
        .unwrap_err();

    assert_eq!(refused.code(), ErrorCode::InvalidTransition);
    let row = db.get(id).unwrap().unwrap();
    assert_eq!(row.status, QueueStatus::NeedsReview);
    assert_eq!(row.source_hash, hash);
    assert!(db.load_unsettled_receipt(id).unwrap().is_some());
}

#[test]
fn requeue_hash_collision_is_duplicate() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let path = Path::new("C:/scans/contract.pdf");
    let reviewed = db.enqueue(path, "first-version").unwrap();
    advance_to_ready(&db, reviewed.id);
    // The edited file was picked up again on its own, as a new row.
    let edited = db.enqueue(path, "second-version").unwrap();
    assert_ne!(edited.id, reviewed.id);

    let refused = db
        .requeue_for_analysis(reviewed.id, QueueStatus::Ready, Some("second-version"))
        .unwrap_err();

    assert_eq!(refused.code(), ErrorCode::Duplicate);
    let row = db.get(reviewed.id).unwrap().unwrap();
    assert_eq!(row.status, QueueStatus::Ready, "nothing changed");
    assert_eq!(row.source_hash, "first-version");
}

#[test]
fn list_all_operation_history_is_uncapped() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let item = db.enqueue(Path::new("bulk.pdf"), "bulk-hash").unwrap();
    let inspector = rusqlite::Connection::open(temp.path().join("queue.sqlite3")).unwrap();
    let mut insert = inspector
        .prepare(
            "INSERT INTO operation_receipts(
               queue_item_id, direction, source_path, destination_path, pre_hash,
               operation_kind, stage, source_exists, destination_exists, temporary_exists,
               created_at, updated_at
             ) VALUES(?1, 'apply', 'bulk.pdf', 'renamed.pdf', 'bulk-hash',
                      'rename', 'complete', 0, 1, 0, ?2, ?2)",
        )
        .unwrap();
    let total = HISTORY_LIMIT * 2 + 3;
    for moment in 0..total {
        insert
            .execute(rusqlite::params![item.id, moment as i64])
            .unwrap();
    }

    let everything = db.list_all_operation_history().unwrap();

    assert_eq!(everything.len(), total);
    assert_eq!(everything[0].at, (total - 1) as i64);
    assert!(everything.windows(2).all(|pair| pair[0].at >= pair[1].at));
    assert_eq!(db.count_operation_history().unwrap(), total);
    assert_eq!(
        db.list_operation_history(total).unwrap().len(),
        HISTORY_LIMIT,
        "the window's listing keeps its cap"
    );
}

/// A database a newer Intern wrote, opened by this one after a reinstall of
/// the older release: what it cannot read is left out, and the rest lists.
#[test]
fn unknown_codes_and_statuses_are_tolerated_on_read() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let readable = db.enqueue(Path::new("readable.pdf"), "h1").unwrap();
    let future_code = db.enqueue(Path::new("future-code.pdf"), "h2").unwrap();
    let future_status = db.enqueue(Path::new("future-status.pdf"), "h3").unwrap();
    let inspector = rusqlite::Connection::open(temp.path().join("queue.sqlite3")).unwrap();
    inspector
        .execute(
            "UPDATE queue_items SET status = 'needs_review', error_code = 'FUTURE_CODE'
             WHERE id = ?1",
            [future_code.id],
        )
        .unwrap();
    inspector
        .execute(
            "UPDATE queue_items SET status = 'future_status', error_code = 'FUTURE_CODE'
             WHERE id = ?1",
            [future_status.id],
        )
        .unwrap();
    let insert_receipt = |direction: &str, kind: &str, stage: &str, at: i64| {
        inspector
            .execute(
                "INSERT INTO operation_receipts(
                   queue_item_id, direction, source_path, destination_path, pre_hash,
                   operation_kind, stage, source_exists, destination_exists, temporary_exists,
                   created_at, updated_at
                 ) VALUES(?1, ?2, 'a.pdf', 'b.pdf', 'h1', ?3, ?4, 0, 1, 0, ?5, ?5)",
                rusqlite::params![readable.id, direction, kind, stage, at],
            )
            .unwrap();
    };
    insert_receipt("apply", "rename", "complete", 1);
    insert_receipt("sideways", "rename", "complete", 2);
    insert_receipt("apply", "teleport", "rolled_back", 3);

    let items = db.list().unwrap();

    assert_eq!(
        items.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![readable.id, future_code.id]
    );
    assert_eq!(
        items[1].error_code, None,
        "an unknown code reads as no code"
    );
    assert_eq!(items[1].status, QueueStatus::NeedsReview);
    assert_eq!(db.hidden_rows(), 1);
    assert_eq!(db.get(future_status.id).unwrap(), None);
    assert_eq!(db.get(future_code.id).unwrap().unwrap().error_code, None);

    let history = db.list_all_operation_history().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].at, 1);
    assert_eq!(db.hidden_rows(), 3, "one item and two receipts");

    // Writes stay strict: nothing moves the hidden row.
    assert_eq!(
        db.transition(
            future_status.id,
            QueueStatus::Queued,
            QueueStatus::Canceled,
            None
        )
        .unwrap_err()
        .code(),
        ErrorCode::StateConflict
    );
}

#[test]
fn get_returns_one_item() {
    let temp = TempDir::new().unwrap();
    let db = store(&temp);
    let first = db.enqueue(Path::new("first.pdf"), "h1").unwrap();
    let second = db.enqueue(Path::new("second.pdf"), "h2").unwrap();
    db.transition(second.id, QueueStatus::Queued, QueueStatus::Canceled, None)
        .unwrap();

    assert_eq!(db.get(first.id).unwrap(), Some(first));
    let found = db.get(second.id).unwrap().unwrap();
    assert_eq!(found.id, second.id);
    assert_eq!(found.status, QueueStatus::Canceled);
    assert_eq!(db.get(second.id + 100).unwrap(), None);
}
