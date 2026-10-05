use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::{
    ErrorCode, InternError, InternResult, OperationDirection, OperationKind, OperationReceipt,
    OperationStage, QueueItem, QueueStatus,
};

const LEASE_SECONDS: i64 = 60;
static SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(1);
/// Whether an error code this build does not know has been reported yet. One
/// line says what happened; one per row per listing would bury everything
/// else on stderr.
static UNKNOWN_ERROR_CODE_REPORTED: AtomicBool = AtomicBool::new(false);

pub struct QueueStore {
    connection: Mutex<Connection>,
    session_id: String,
    /// Queue rows the latest listing left out because this build cannot read
    /// them, and receipts the latest history listing left out for the same
    /// reason. See [`QueueStore::hidden_rows`].
    hidden_items: AtomicUsize,
    hidden_receipts: AtomicUsize,
}

/// A completed item whose content matches a newly added file.
///
/// `filed_as` is the leaf name the content actually lives under: the
/// destination of the latest completed apply receipt. A keep-original
/// completion moved nothing and has no such receipt, so `filed_as` is `None`
/// and the caller falls back to the item's original filename.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DuplicateInfo {
    pub queue_item_id: i64,
    pub source_path: PathBuf,
    pub filed_as: Option<String>,
}

/// One finished journalled file operation, for the history view.
///
/// `at` is the receipt's `updated_at`: the moment the operation reached its
/// terminal stage, not the moment it was planned. `original_path` and
/// `new_path` are the receipt's source and destination as recorded — for an
/// undo the "original" is therefore the previously applied name and the "new"
/// path is the restored one, which is exactly what a history reader expects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryEntry {
    pub receipt_id: i64,
    pub queue_item_id: i64,
    pub at: i64,
    pub direction: OperationDirection,
    pub kind: OperationKind,
    pub stage: OperationStage,
    pub original_path: PathBuf,
    pub new_path: PathBuf,
}

/// The most receipts a history listing will ever return.
pub const HISTORY_LIMIT: usize = 500;

/// The newest receipt of an item that never reached a terminal stage.
const UNSETTLED_RECEIPT: &str =
    "WHERE queue_item_id = ?1 AND stage NOT IN ('complete', 'rolled_back')
     ORDER BY id DESC LIMIT 1";

impl QueueStore {
    pub fn open(path: impl AsRef<Path>) -> InternResult<Self> {
        let mut connection = Connection::open(path).map_err(InternError::from)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(InternError::from)?;
        connection
            .execute_batch(
                "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS schema_migrations (
               version INTEGER PRIMARY KEY,
               applied_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS queue_sessions (
               session_id TEXT PRIMARY KEY,
               heartbeat_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS queue_items (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               source_path TEXT NOT NULL,
               source_path_key TEXT NOT NULL,
               source_hash TEXT NOT NULL,
               status TEXT NOT NULL,
               processing_failures INTEGER NOT NULL DEFAULT 0,
               error_code TEXT,
               owner_session TEXT REFERENCES queue_sessions(session_id) ON DELETE SET NULL,
               lease_expires_at INTEGER,
               previous_status TEXT,
               active_receipt_id INTEGER,
               reconciliation_receipt_id INTEGER,
               applying_epoch INTEGER NOT NULL DEFAULT 0,
               created_at INTEGER NOT NULL,
               updated_at INTEGER NOT NULL,
               UNIQUE(source_path_key, source_hash)
             );
             -- Every file added to the queue asks whether its content was filed
             -- before, and the UNIQUE index above is no use for that question
             -- because it leads with the path. Without this one the answer is a
             -- scan of the whole queue, once per file, on the intake path.
             CREATE INDEX IF NOT EXISTS queue_items_source_hash
               ON queue_items(source_hash);

             CREATE TABLE IF NOT EXISTS proposals (
               queue_item_id INTEGER PRIMARY KEY REFERENCES queue_items(id) ON DELETE CASCADE,
               proposal_json TEXT NOT NULL,
               created_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS operation_receipts (
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
             INSERT OR IGNORE INTO schema_migrations(version, applied_at)
               VALUES (1, unixepoch());",
            )
            .map_err(InternError::from)?;
        migrate_legacy_schema(&mut connection)?;
        // Listing the queue asks each item for its newest receipt, its newest
        // finished one and any unfinished one. Each question is the newest
        // receipt of one item, which without this index is a walk down every
        // receipt the queue has ever written - per item, per listing. Created
        // after the legacy migration, which may rebuild the receipts table.
        connection
            .execute_batch(
                "CREATE INDEX IF NOT EXISTS operation_receipts_by_item
                   ON operation_receipts(queue_item_id, id);",
            )
            .map_err(InternError::from)?;
        let session_id = new_session_id();
        connection
            .execute(
                "INSERT INTO queue_sessions(session_id, heartbeat_at) VALUES (?1, ?2)",
                params![session_id, now()],
            )
            .map_err(InternError::from)?;
        Ok(Self {
            connection: Mutex::new(connection),
            session_id,
            hidden_items: AtomicUsize::new(0),
            hidden_receipts: AtomicUsize::new(0),
        })
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn enqueue(&self, source_path: &Path, source_hash: &str) -> InternResult<QueueItem> {
        let path = source_path.to_string_lossy().into_owned();
        let path_key = windows_path_key(&path);
        let timestamp = now();
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        transaction.execute(
            "INSERT INTO queue_items(source_path, source_path_key, source_hash, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'queued', ?4, ?4)
             ON CONFLICT(source_path_key, source_hash) DO NOTHING",
            params![path, path_key, source_hash, timestamp],
        ).map_err(InternError::from)?;
        let item = query_one(
            &transaction,
            "WHERE source_path_key = ?1 AND source_hash = ?2",
            params![path_key, source_hash],
        )?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn claim_next(&self) -> InternResult<Option<QueueItem>> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let id = transaction
            .query_row(
                "SELECT id FROM queue_items
             WHERE status = 'queued'
               AND NOT EXISTS (
                 SELECT 1 FROM queue_items active
                 WHERE active.status IN ('extracting', 'analyzing', 'applying')
               )
             ORDER BY id LIMIT 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(InternError::from)?;
        let Some(id) = id else {
            transaction.commit().map_err(InternError::from)?;
            return Ok(None);
        };
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = 'extracting', error_code = NULL, owner_session = ?1,
                 lease_expires_at = ?2, updated_at = ?3
             WHERE id = ?4 AND status = 'queued'",
                params![self.session_id, lease_deadline(), now(), id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Ok(None);
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(Some(item))
    }

    pub fn renew_lease(&self, id: i64) -> InternResult<QueueItem> {
        let connection = self.lock()?;
        touch_session(&connection, &self.session_id)?;
        let changed = connection
            .execute(
                "UPDATE queue_items SET lease_expires_at = ?1, updated_at = ?2
             WHERE id = ?3 AND owner_session = ?4
               AND status IN ('extracting', 'analyzing', 'applying')",
                params![lease_deadline(), now(), id, self.session_id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "queue lease is not owned by this session",
            ));
        }
        query_one(&connection, "WHERE id = ?1", params![id])
    }

    pub(crate) fn renew_operation_lease(
        &self,
        id: i64,
        receipt_id: i64,
        expected_stage: OperationStage,
    ) -> InternResult<QueueItem> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items SET lease_expires_at = ?1, updated_at = ?2
             WHERE id = ?3 AND status = 'applying' AND owner_session = ?4
               AND active_receipt_id = ?5
               AND EXISTS (
                 SELECT 1 FROM operation_receipts receipts
                 WHERE receipts.id = ?5 AND receipts.queue_item_id = ?3
                   AND receipts.stage = ?6
               )",
                params![
                    lease_deadline(),
                    now(),
                    id,
                    self.session_id,
                    receipt_id,
                    expected_stage.as_db()
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "operation receipt lease renewal compare-and-swap failed",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn transition(
        &self,
        id: i64,
        expected: QueueStatus,
        next: QueueStatus,
        error: Option<ErrorCode>,
    ) -> InternResult<QueueItem> {
        if !expected.can_transition_to(next) {
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "queue transition is not permitted",
            ));
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        let expected_active = is_active(expected);
        let next_active = is_active(next);
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = ?1, error_code = ?2,
                 owner_session = CASE WHEN ?3 THEN ?4 ELSE NULL END,
                 lease_expires_at = CASE WHEN ?3 THEN ?5 ELSE NULL END,
                 updated_at = ?6
             WHERE id = ?7 AND status = ?8
               AND (NOT ?9 OR owner_session = ?4)",
                params![
                    next.as_db(),
                    error.map(ErrorCode::as_str),
                    next_active,
                    self.session_id,
                    lease_deadline(),
                    now(),
                    id,
                    expected.as_db(),
                    expected_active,
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "queue item changed or is owned by another session",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn manual_retry(&self, id: i64) -> InternResult<QueueItem> {
        self.cas_status(id, QueueStatus::Failed, QueueStatus::Queued, true)
    }

    /// Finds the most recently completed item holding the same content at a
    /// different path.
    ///
    /// Only `completed` rows count: an item whose apply was undone is back in
    /// `ready`, its content back at its original path, and a still-pending item
    /// has not been filed anywhere yet. Ties on the completion timestamp fall
    /// to the newest row.
    pub fn find_completed_duplicate(
        &self,
        source_hash: &str,
        excluding_path_key: &str,
    ) -> InternResult<Option<DuplicateInfo>> {
        let connection = self.lock()?;
        let Some((queue_item_id, source_path)) = connection
            .query_row(
                "SELECT id, source_path FROM queue_items
                 WHERE status = 'completed' AND source_hash = ?1 AND source_path_key <> ?2
                 ORDER BY updated_at DESC, id DESC LIMIT 1",
                params![source_hash, excluding_path_key],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(InternError::from)?
        else {
            return Ok(None);
        };
        // The newest *finished* receipt is where the content actually is now.
        // Asking for the newest receipt of any kind and then requiring it to be
        // a completed apply reported no filed name at all whenever something
        // later had been journalled and abandoned - an undo that rolled back
        // sits on top of the apply that filed the document without having moved
        // anything. An operation that rolled back moved nothing, so it cannot
        // answer the question, and the completed apply underneath it still can.
        // A completed undo does answer it: the file is back at its own name.
        let filed_as = connection
            .query_row(
                "SELECT direction, destination_path FROM operation_receipts
                 WHERE queue_item_id = ?1 AND stage = 'complete'
                 ORDER BY id DESC LIMIT 1",
                params![queue_item_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(InternError::from)?
            .filter(|(direction, _)| direction == OperationDirection::Apply.as_db())
            .and_then(|(_, destination)| {
                Path::new(&destination)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            });

        Ok(Some(DuplicateInfo {
            queue_item_id,
            source_path: PathBuf::from(source_path),
            filed_as,
        }))
    }

    /// Requeues a duplicate-flagged review item so it analyzes normally.
    ///
    /// The compare-and-swap covers the error code as well as the status: only
    /// the pre-processing DUPLICATE flag may take this shortcut back to the
    /// queue, and a concurrent decision on the item makes the retry fail
    /// closed.
    pub fn retry_duplicate(&self, id: i64) -> InternResult<QueueItem> {
        let connection = self.lock()?;
        let changed = connection
            .execute(
                "UPDATE queue_items
             SET status = 'queued', processing_failures = 0, error_code = NULL,
                 owner_session = NULL, lease_expires_at = NULL,
                 previous_status = NULL, active_receipt_id = NULL,
                 reconciliation_receipt_id = NULL, updated_at = ?1
             WHERE id = ?2 AND status = 'needs_review' AND error_code = 'DUPLICATE'",
                params![now(), id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "item is not a duplicate awaiting review",
            ));
        }
        query_one(&connection, "WHERE id = ?1", params![id])
    }

    /// Sends a document waiting on a person back to the queue to be read
    /// again from the start: Re-analyze.
    ///
    /// A document signed, edited or saved over after it was read had no way
    /// back. Approving it checked the old fingerprint and failed every time,
    /// and Retry was refused because nothing had failed. Everything the
    /// earlier reading decided goes in one transaction - the proposal, and
    /// with it any approval; the failure count; the error code; the receipt
    /// pointers - so no half of it can outlive the other. `new_hash` is the
    /// file's fingerprint now, when it is not the one stored: the item is
    /// whatever is at its path today, and the rename after the new reading
    /// is checked against that. A row already holding that path and that
    /// content is DUPLICATE: the same version of the file is in the queue
    /// once already.
    ///
    /// Refused while any operation of the item never finished. What is on
    /// disk is an open question until it is checked again, and a new
    /// reading would be of a file that may not be the document.
    pub fn requeue_for_analysis(
        &self,
        id: i64,
        expected: QueueStatus,
        new_hash: Option<&str>,
    ) -> InternResult<QueueItem> {
        if !matches!(expected, QueueStatus::Ready | QueueStatus::NeedsReview) {
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "only a document waiting for review can be analyzed again",
            ));
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        let unsettled = transaction
            .query_row(
                "SELECT EXISTS(
                   SELECT 1 FROM operation_receipts
                   WHERE queue_item_id = ?1 AND stage NOT IN ('complete', 'rolled_back')
                 )",
                params![id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(InternError::from)?;
        if unsettled {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "an earlier rename of this document did not finish; check it again first",
            ));
        }
        let changed = match transaction.execute(
            "UPDATE queue_items
             SET status = 'queued', source_hash = COALESCE(?1, source_hash),
                 processing_failures = 0, error_code = NULL, owner_session = NULL,
                 lease_expires_at = NULL, previous_status = NULL, active_receipt_id = NULL,
                 reconciliation_receipt_id = NULL, updated_at = ?2
             WHERE id = ?3 AND status = ?4",
            params![new_hash, now(), id, expected.as_db()],
        ) {
            Ok(changed) => changed,
            Err(rusqlite::Error::SqliteFailure(failure, _))
                if failure.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
            {
                transaction.rollback().map_err(InternError::from)?;
                return Err(InternError::new(
                    ErrorCode::Duplicate,
                    "this version of the file is already in the queue",
                ));
            }
            Err(error) => return Err(error.into()),
        };
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "the item changed before it could be analyzed again",
            ));
        }
        transaction
            .execute(
                "DELETE FROM proposals WHERE queue_item_id = ?1",
                params![id],
            )
            .map_err(InternError::from)?;
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn complete_keep_original(
        &self,
        id: i64,
        expected: QueueStatus,
    ) -> InternResult<QueueItem> {
        if !matches!(expected, QueueStatus::Ready | QueueStatus::NeedsReview) {
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "keep-original requires a reviewable item",
            ));
        }
        self.cas_status(id, expected, QueueStatus::Completed, false)
    }

    pub fn begin_applying(&self, id: i64, expected: QueueStatus) -> InternResult<QueueItem> {
        if !matches!(expected, QueueStatus::Ready | QueueStatus::Completed) {
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "apply requires ready or completed state",
            ));
        }
        self.enter_applying(id, expected, "('extracting', 'analyzing', 'applying')")
    }

    /// Starts undoing a completed rename, refusing only while another file
    /// operation is in flight.
    ///
    /// `begin_applying` also waits out a document being read or analyzed,
    /// which is right for a new rename - the analysis may be about to propose
    /// one - but with a local model taking minutes per document it made Undo
    /// fail for as long as a backlog drained. An undo moves only its own,
    /// already filed document, and the worker reads only its own item, so the
    /// two never touch the same file. The one-applying-row rule still holds:
    /// `claim_next` claims nothing while this row applies, and the analyzed
    /// document's own rename is deferred until the undo is done.
    pub fn begin_undo(&self, id: i64) -> InternResult<QueueItem> {
        self.enter_applying(id, QueueStatus::Completed, "('applying')")
    }

    fn enter_applying(
        &self,
        id: i64,
        expected: QueueStatus,
        blocking_statuses: &str,
    ) -> InternResult<QueueItem> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let changed = transaction
            .execute(
                &format!(
                    "UPDATE queue_items
             SET status = 'applying', previous_status = ?1, owner_session = ?2,
                 lease_expires_at = ?3, active_receipt_id = NULL,
                 reconciliation_receipt_id = NULL, applying_epoch = applying_epoch + 1,
                 error_code = NULL, updated_at = ?4
             WHERE id = ?5 AND status = ?1 AND active_receipt_id IS NULL
               AND NOT EXISTS (
                 SELECT 1 FROM queue_items active
                 WHERE active.id <> ?5 AND active.status IN {blocking_statuses}
               )"
                ),
                params![
                    expected.as_db(),
                    self.session_id,
                    lease_deadline(),
                    now(),
                    id
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "item cannot enter applying",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn complete_apply(&self, id: i64, receipt_id: i64) -> InternResult<QueueItem> {
        self.finish_applying(
            id,
            receipt_id,
            QueueStatus::Completed,
            QueueStatus::Ready,
            OperationDirection::Apply,
        )
    }

    pub fn complete_undo(&self, id: i64, receipt_id: i64) -> InternResult<QueueItem> {
        self.finish_applying(
            id,
            receipt_id,
            QueueStatus::Ready,
            QueueStatus::Completed,
            OperationDirection::Undo,
        )
    }

    pub fn claim_applying_reconciliation(&self, id: i64) -> InternResult<QueueItem> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let timestamp = now();
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET owner_session = ?1, lease_expires_at = ?2, updated_at = ?3
             WHERE id = ?4 AND status = 'applying'
               AND (
                 owner_session IS NULL
                 OR NOT EXISTS (
                   SELECT 1 FROM queue_sessions sessions
                   WHERE sessions.session_id = queue_items.owner_session
                 )
                 OR (
                   lease_expires_at <= ?3
                   AND NOT EXISTS (
                     SELECT 1 FROM queue_sessions sessions
                     WHERE sessions.session_id = queue_items.owner_session
                       AND sessions.heartbeat_at > ?5
                   )
                 )
               )",
                params![
                    self.session_id,
                    timestamp + LEASE_SECONDS,
                    timestamp,
                    id,
                    timestamp - LEASE_SECONDS
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "applying owner is still live",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    /// Records that an operation moved nothing and returns the item to where
    /// it was before the operation began.
    ///
    /// `review` is what the reconciliation found wrong with the files it left
    /// in place: an original that no longer matches its receipt, or a foreign
    /// file sitting at the name the rename wanted. The rollback is recorded
    /// either way, because nothing moved, and an apply then waits in review
    /// under that code rather than in ready, where the scheduler would file it
    /// again. An undo always goes back to completed: the document is still
    /// filed, and a filed document a person has since edited is theirs.
    pub(crate) fn resolve_reconciled_rollback(
        &self,
        id: i64,
        receipt_id: i64,
        expected_stage: OperationStage,
        review: Option<ErrorCode>,
    ) -> InternResult<QueueItem> {
        if !matches!(
            expected_stage,
            OperationStage::Planned
                | OperationStage::Copied
                | OperationStage::Verified
                | OperationStage::RollbackRequired
                | OperationStage::RolledBack
        ) {
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "receipt cannot resolve as rolled back",
            ));
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        if expected_stage != OperationStage::RolledBack {
            let changed = transaction
                .execute(
                    "UPDATE operation_receipts
                 SET stage = 'rolled_back', source_exists = 1, destination_exists = 0,
                     temporary_exists = 0,
                     updated_at = ?1
                 WHERE id = ?2 AND queue_item_id = ?3 AND stage = ?4",
                    params![now(), receipt_id, id, expected_stage.as_db()],
                )
                .map_err(InternError::from)?;
            if changed != 1 {
                transaction.rollback().map_err(InternError::from)?;
                return Err(InternError::new(
                    ErrorCode::StateConflict,
                    "receipt rollback stage compare-and-swap failed",
                ));
            }
        } else {
            let changed = transaction
                .execute(
                    "UPDATE operation_receipts SET temporary_exists = 0, updated_at = ?1
                 WHERE id = ?2 AND queue_item_id = ?3 AND stage = 'rolled_back'",
                    params![now(), receipt_id, id],
                )
                .map_err(InternError::from)?;
            if changed != 1 {
                transaction.rollback().map_err(InternError::from)?;
                return Err(InternError::new(
                    ErrorCode::StateConflict,
                    "rolled-back receipt cleanup compare-and-swap failed",
                ));
            }
        }
        // Every expression below reads the row as it was, so the review test
        // still sees the previous status the same statement clears.
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = CASE WHEN ?5 IS NOT NULL AND previous_status = 'ready'
                               THEN 'needs_review' ELSE previous_status END,
                 error_code = CASE WHEN previous_status = 'ready' THEN ?5 ELSE NULL END,
                 owner_session = NULL, lease_expires_at = NULL,
                 previous_status = NULL, active_receipt_id = NULL,
                 reconciliation_receipt_id = NULL, updated_at = ?1
             WHERE id = ?2 AND status = 'applying' AND owner_session = ?3
               AND active_receipt_id = ?4
               AND EXISTS (
                 SELECT 1 FROM operation_receipts receipts
                 WHERE receipts.id = ?4 AND receipts.queue_item_id = ?2
                   AND receipts.stage = 'rolled_back'
                   AND (
                     (receipts.direction = 'apply' AND queue_items.previous_status = 'ready')
                     OR (receipts.direction = 'undo' AND queue_items.previous_status = 'completed')
                   )
               )",
                params![
                    now(),
                    id,
                    self.session_id,
                    receipt_id,
                    review.map(ErrorCode::as_str)
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "rolled-back reconciliation compare-and-swap failed",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub(crate) fn resolve_empty_applying(&self, id: i64) -> InternResult<QueueItem> {
        // Only begin_applying() advances this epoch marker. Pre-v4 applying rows stay at
        // zero, so ambiguous legacy operations still fail closed instead of being
        // mistaken for a crash that happened before the receipt transaction.
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = previous_status, owner_session = NULL, lease_expires_at = NULL,
                 previous_status = NULL, reconciliation_receipt_id = NULL,
                 error_code = NULL, updated_at = ?1
             WHERE id = ?2 AND status = 'applying' AND owner_session = ?3
               AND active_receipt_id IS NULL
               AND applying_epoch > 0
               AND previous_status IN ('ready', 'completed')",
                params![now(), id, self.session_id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "empty applying reconciliation compare-and-swap failed",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub(crate) fn resolve_verified_operation(
        &self,
        id: i64,
        receipt_id: i64,
        expected_stage: OperationStage,
    ) -> InternResult<QueueItem> {
        if !matches!(
            expected_stage,
            OperationStage::Planned
                | OperationStage::Copied
                | OperationStage::Verified
                | OperationStage::Published
                | OperationStage::Complete
        ) {
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "receipt cannot resolve as a verified operation",
            ));
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let receipt = transaction
            .query_row(
                &receipt_select(
                    "WHERE id = ?1 AND queue_item_id = ?2 AND stage = ?3
                   AND EXISTS (
                     SELECT 1 FROM queue_items
                     WHERE id = ?2 AND status = 'applying' AND owner_session = ?4
                       AND active_receipt_id = ?1
                   )",
                ),
                params![receipt_id, id, expected_stage.as_db(), self.session_id],
                row_to_receipt,
            )
            .optional()
            .map_err(InternError::from)?
            .ok_or_else(|| {
                InternError::new(
                    ErrorCode::StateConflict,
                    "verified receipt reconciliation compare-and-swap failed",
                )
            })?;
        let (required_previous, next) = match receipt.direction {
            OperationDirection::Apply => (QueueStatus::Ready, QueueStatus::Completed),
            OperationDirection::Undo => (QueueStatus::Completed, QueueStatus::Ready),
        };
        if expected_stage != OperationStage::Complete {
            let changed = transaction
                .execute(
                    "UPDATE operation_receipts
                 SET stage = 'complete', source_exists = 0, destination_exists = 1,
                     temporary_exists = 0, post_hash = pre_hash, updated_at = ?1
                 WHERE id = ?2 AND queue_item_id = ?3 AND stage = ?4",
                    params![now(), receipt_id, id, expected_stage.as_db()],
                )
                .map_err(InternError::from)?;
            if changed != 1 {
                transaction.rollback().map_err(InternError::from)?;
                return Err(InternError::new(
                    ErrorCode::StateConflict,
                    "published receipt completion compare-and-swap failed",
                ));
            }
        }
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = ?1, owner_session = NULL, lease_expires_at = NULL,
                 previous_status = NULL, active_receipt_id = NULL,
                 reconciliation_receipt_id = NULL, error_code = NULL, updated_at = ?2
             WHERE id = ?3 AND status = 'applying' AND owner_session = ?4
               AND previous_status = ?5 AND active_receipt_id = ?6",
                params![
                    next.as_db(),
                    now(),
                    id,
                    self.session_id,
                    required_previous.as_db(),
                    receipt_id
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "verified operation queue reconciliation failed",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn record_applying_rollback(
        &self,
        id: i64,
        receipt_id: i64,
        error: ErrorCode,
    ) -> InternResult<QueueItem> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET reconciliation_receipt_id = ?1, error_code = ?2,
                 lease_expires_at = ?3, updated_at = ?4
             WHERE id = ?5 AND status = 'applying' AND owner_session = ?6
               AND active_receipt_id = ?1",
                params![
                    receipt_id,
                    error.as_str(),
                    lease_deadline(),
                    now(),
                    id,
                    self.session_id
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "applying item is not owned by this session",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn defer_published_reconciliation(
        &self,
        id: i64,
        receipt_id: i64,
        error: ErrorCode,
    ) -> InternResult<QueueItem> {
        if error != ErrorCode::SourceDeleteFailed {
            return Err(InternError::new(
                ErrorCode::InvalidData,
                "only source-delete uncertainty can be deferred for user review",
            ));
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = 'needs_review', reconciliation_receipt_id = ?1,
                 active_receipt_id = NULL, previous_status = NULL,
                 owner_session = NULL, lease_expires_at = NULL,
                 error_code = ?2, updated_at = ?3
             WHERE id = ?4 AND status = 'applying' AND owner_session = ?5
               AND active_receipt_id = ?1
               AND EXISTS (
                 SELECT 1 FROM operation_receipts receipts
                 WHERE receipts.id = ?1 AND receipts.queue_item_id = ?4
                   AND receipts.direction = 'apply' AND receipts.stage = 'published'
               )",
                params![receipt_id, error.as_str(), now(), id, self.session_id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "published source-delete uncertainty could not be deferred",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    /// Hands an `applying` item that reconciliation could not settle to a
    /// person.
    ///
    /// One `applying` row stops the whole queue: `claim_next` and
    /// `begin_applying` both refuse while any item is applying. An operation
    /// whose surviving paths cannot be proven used to stay applying forever, so
    /// a single half-applied rename froze every other document with no way out
    /// but editing the database by hand. Review is where a state only a person
    /// can judge belongs, and the receipt is kept so what is on disk can still
    /// be explained.
    ///
    /// The receipt itself is left in whatever stage it reached. It is not
    /// finished and pretending otherwise would lose the only record of what
    /// happened, and while it is unfinished no new operation can be journalled
    /// for the item. Keep original, cancel and an unconfirmed remove all
    /// refuse, because each would decide what the files are on the person's
    /// behalf. What moves a parked item on is `reattach_unsettled_receipt` -
    /// Check again, which the queue's retry and approve run - re-running the
    /// reconciliation once the person has dealt with whatever held the files,
    /// or a remove the person has confirmed, which deletes the row and its
    /// receipts.
    pub(crate) fn park_applying_for_review(
        &self,
        id: i64,
        receipt_id: i64,
        error: ErrorCode,
    ) -> InternResult<QueueItem> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = 'needs_review', reconciliation_receipt_id = ?1,
                 active_receipt_id = NULL, previous_status = NULL,
                 owner_session = NULL, lease_expires_at = NULL,
                 error_code = ?2, updated_at = ?3
             WHERE id = ?4 AND status = 'applying' AND owner_session = ?5
               AND active_receipt_id = ?1",
                params![receipt_id, error.as_str(), now(), id, self.session_id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "unsettled operation could not be parked for review",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    pub fn claim_deferred_reconciliation(&self, id: i64) -> InternResult<QueueItem> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = 'applying', previous_status = 'ready', owner_session = ?1,
                 lease_expires_at = ?2, active_receipt_id = reconciliation_receipt_id,
                 updated_at = ?3
             WHERE id = ?4 AND status = 'needs_review'
               AND error_code = 'SOURCE_DELETE_FAILED'
               AND active_receipt_id IS NULL AND reconciliation_receipt_id IS NOT NULL
               AND EXISTS (
                 SELECT 1 FROM operation_receipts receipts
                 WHERE receipts.id = queue_items.reconciliation_receipt_id
                   AND receipts.queue_item_id = queue_items.id
                   AND receipts.direction = 'apply' AND receipts.stage = 'published'
               )
               AND NOT EXISTS (
                 SELECT 1 FROM queue_items active
                 WHERE active.id <> ?4
                   AND active.status IN ('extracting', 'analyzing', 'applying')
               )",
                params![self.session_id, lease_deadline(), now(), id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "deferred source deletion is not available for explicit retry",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    /// Puts an item whose file operation never finished back into
    /// `applying`, bound to that operation's receipt, so a reconciliation
    /// can settle it. `None` when the item has no unfinished receipt.
    ///
    /// A parked item kept its receipt but nothing ever looked at it again:
    /// recovery only reconciles rows that are applying, and the receipt's
    /// live stage made every later apply fail to journal, so a document whose
    /// rename was refused once could never be renamed. The receipt is found by
    /// its stage rather than through `reconciliation_receipt_id`, which the
    /// first failed re-apply clears. Only a reviewable row can be reattached -
    /// a row being read or analyzed belongs to the worker - and, as for an
    /// undo, only while no other row is applying. The previous status follows
    /// the receipt's direction, which is what the reconciliation's
    /// compare-and-swaps return the row to.
    pub fn reattach_unsettled_receipt(&self, id: i64) -> InternResult<Option<OperationReceipt>> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let Some(receipt) = transaction
            .query_row(
                &receipt_select(UNSETTLED_RECEIPT),
                params![id],
                row_to_receipt,
            )
            .optional()
            .map_err(InternError::from)?
        else {
            transaction.commit().map_err(InternError::from)?;
            return Ok(None);
        };
        let previous_status = match receipt.direction {
            OperationDirection::Apply => QueueStatus::Ready,
            OperationDirection::Undo => QueueStatus::Completed,
        };
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = 'applying', previous_status = ?1, owner_session = ?2,
                 lease_expires_at = ?3, active_receipt_id = ?4, updated_at = ?5
             WHERE id = ?6 AND status IN ('needs_review', 'ready')
               AND active_receipt_id IS NULL
               AND NOT EXISTS (
                 SELECT 1 FROM queue_items active
                 WHERE active.id <> ?6 AND active.status = 'applying'
               )",
                params![
                    previous_status.as_db(),
                    self.session_id,
                    lease_deadline(),
                    receipt.id,
                    now(),
                    id
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "item cannot be reattached to its unfinished operation",
            ));
        }
        transaction.commit().map_err(InternError::from)?;
        Ok(Some(receipt))
    }

    pub fn recover_interrupted(&self) -> InternResult<usize> {
        let connection = self.lock()?;
        let timestamp = now();
        connection
            .execute(
                "UPDATE queue_items
             SET status = 'queued', owner_session = NULL, lease_expires_at = NULL, updated_at = ?1
             WHERE status IN ('extracting', 'analyzing')
               AND (
                 owner_session IS NULL
                 OR NOT EXISTS (
                   SELECT 1 FROM queue_sessions sessions
                   WHERE sessions.session_id = queue_items.owner_session
                 )
                 OR (
                   lease_expires_at <= ?1
                   AND NOT EXISTS (
                     SELECT 1 FROM queue_sessions sessions
                     WHERE sessions.session_id = queue_items.owner_session
                       AND sessions.heartbeat_at > ?2
                   )
                 )
               )",
                params![timestamp, timestamp - LEASE_SECONDS],
            )
            .map_err(InternError::from)
    }

    /// The newest item enqueued from any of `paths`, compared by the same
    /// normalized key `enqueue` stores.
    ///
    /// The intake watcher asks about one path per file per scan, and the
    /// answer used to come from listing the whole queue - and loading every
    /// proposal with it - for each question. One indexed lookup is what the
    /// question costs. Several spellings of the same path may be offered
    /// (as entered, and canonicalized) because a file the apply already
    /// renamed away no longer canonicalizes.
    pub fn find_newest_by_source_path(&self, paths: &[&Path]) -> InternResult<Option<QueueItem>> {
        if paths.is_empty() {
            return Ok(None);
        }
        let keys = paths
            .iter()
            .map(|path| source_path_key(path))
            .collect::<Vec<_>>();
        let placeholders = (1..=keys.len())
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let connection = self.lock()?;
        connection
            .query_row(
                &queue_select(&format!(
                    "WHERE source_path_key IN ({placeholders}) ORDER BY id DESC LIMIT 1"
                )),
                rusqlite::params_from_iter(keys.iter()),
                row_to_item,
            )
            .optional()
            .map_err(InternError::from)
    }

    /// Every item in the queue that this build can read, oldest first.
    ///
    /// A row with a status this build does not know is left out and counted
    /// in [`QueueStore::hidden_rows`]: a row a newer Intern wrote, in a
    /// database this one opened after the older release was reinstalled. It
    /// used to fail the whole query, and a window that cannot list the queue
    /// shows nothing at all.
    pub fn list(&self) -> InternResult<Vec<QueueItem>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(&queue_select("ORDER BY id"))
            .map_err(InternError::from)?;
        let rows = statement
            .query_map([], read_item)
            .map_err(InternError::from)?;
        let mut items = Vec::new();
        let mut hidden = 0;
        for row in rows {
            match row.map_err(InternError::from)? {
                Some(item) => items.push(item),
                None => hidden += 1,
            }
        }
        self.hidden_items.store(hidden, Ordering::Relaxed);
        Ok(items)
    }

    /// One item by id, or `None` when there is no such item - or none this
    /// build can read, which the queue listing leaves out as well.
    ///
    /// The one-item actions used to list the whole queue and search it for
    /// the id; this is the primary-key lookup that question needs.
    pub fn get(&self, id: i64) -> InternResult<Option<QueueItem>> {
        let connection = self.lock()?;
        Ok(connection
            .query_row(&queue_select("WHERE id = ?1"), params![id], read_item)
            .optional()
            .map_err(InternError::from)?
            .flatten())
    }

    /// How many rows the latest queue listing and the latest history listing
    /// left out because this build cannot read them: rows a newer Intern
    /// wrote, with a status, direction, kind or stage this one does not know.
    /// For diagnostics, and for saying so instead of leaving them unexplained.
    pub fn hidden_rows(&self) -> usize {
        self.hidden_items.load(Ordering::Relaxed) + self.hidden_receipts.load(Ordering::Relaxed)
    }

    /// Lists finished operations, newest first, for the history view.
    ///
    /// Only receipts in a terminal stage (`complete` or `rolled_back`) are
    /// reported: an in-flight receipt is bookkeeping for the applier and may
    /// still end either way. Receipts are joined to their queue items, so a
    /// cleared history (which cascades receipt deletion) lists nothing stale.
    /// `limit` is capped at [`HISTORY_LIMIT`]: this is what a window shows.
    pub fn list_operation_history(&self, limit: usize) -> InternResult<Vec<HistoryEntry>> {
        self.operation_history(i64::try_from(limit.min(HISTORY_LIMIT)).unwrap_or(0))
    }

    /// Every finished operation, newest first, with no cap: what an export
    /// writes. The history export used the window's listing and stopped at
    /// five hundred rows without a word, while the dialog promised every
    /// rename Intern had applied.
    pub fn list_all_operation_history(&self) -> InternResult<Vec<HistoryEntry>> {
        // SQLite reads a negative LIMIT as no limit at all.
        self.operation_history(-1)
    }

    /// How many finished operations there are in all, so a listing capped at
    /// [`HISTORY_LIMIT`] can say that it is.
    pub fn count_operation_history(&self) -> InternResult<usize> {
        let connection = self.lock()?;
        let count = connection
            .query_row(
                "SELECT COUNT(*)
                 FROM operation_receipts receipts
                 JOIN queue_items items ON items.id = receipts.queue_item_id
                 WHERE receipts.stage IN ('complete', 'rolled_back')",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(InternError::from)?;
        Ok(usize::try_from(count).unwrap_or(0))
    }

    /// Finished operations, newest first, up to `limit` (none when negative).
    /// A receipt this build cannot read is left out and counted, as a queue
    /// row is.
    fn operation_history(&self, limit: i64) -> InternResult<Vec<HistoryEntry>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT receipts.id, receipts.queue_item_id, receipts.updated_at,
                        receipts.direction, receipts.operation_kind, receipts.stage,
                        receipts.source_path, receipts.destination_path
                 FROM operation_receipts receipts
                 JOIN queue_items items ON items.id = receipts.queue_item_id
                 WHERE receipts.stage IN ('complete', 'rolled_back')
                 ORDER BY receipts.updated_at DESC, receipts.id DESC
                 LIMIT ?1",
            )
            .map_err(InternError::from)?;
        let rows = statement
            .query_map(params![limit], read_history_entry)
            .map_err(InternError::from)?;
        let mut entries = Vec::new();
        let mut hidden = 0;
        for row in rows {
            match row.map_err(InternError::from)? {
                Some(entry) => entries.push(entry),
                None => hidden += 1,
            }
        }
        self.hidden_receipts.store(hidden, Ordering::Relaxed);
        Ok(entries)
    }

    pub fn clear_terminal(&self) -> InternResult<usize> {
        let connection = self.lock()?;
        connection
            .execute(
                "DELETE FROM queue_items WHERE status IN ('failed', 'canceled', 'completed')",
                [],
            )
            .map_err(InternError::from)
    }

    /// Drops items that are still only waiting, leaving everything else alone.
    ///
    /// A user who points the queue at the wrong folder had no way out: the only
    /// bulk action was `clear_terminal`, which deletes finished work, and the
    /// only way to drop a waiting item was one at a time. Four hundred items
    /// meant four hundred clicks.
    ///
    /// Strictly `queued`. An item being extracted or analysed is owned by a
    /// session and must reach its own end; `ready` and `needs_review` are
    /// waiting on a human decision, not on the queue; and terminal rows hold the
    /// receipts that make a rename undoable.
    pub fn discard_queued(&self) -> InternResult<usize> {
        let connection = self.lock()?;
        connection
            .execute("DELETE FROM queue_items WHERE status = 'queued'", [])
            .map_err(InternError::from)
    }

    pub fn record_processing_failure(
        &self,
        id: i64,
        error: ErrorCode,
    ) -> InternResult<QueueStatus> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        let failures = transaction
            .query_row(
                "SELECT processing_failures FROM queue_items
             WHERE id = ?1 AND status IN ('extracting', 'analyzing') AND owner_session = ?2",
                params![id, self.session_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(InternError::from)?
            .ok_or_else(|| {
                InternError::new(
                    ErrorCode::StateConflict,
                    "item is not owned processing work",
                )
            })?
            + 1;
        let status = if failures >= 2 {
            QueueStatus::Failed
        } else {
            QueueStatus::Queued
        };
        transaction
            .execute(
                "UPDATE queue_items
             SET status = ?1, processing_failures = ?2, error_code = ?3,
                 owner_session = NULL, lease_expires_at = NULL, updated_at = ?4
             WHERE id = ?5 AND owner_session = ?6",
                params![
                    status.as_db(),
                    failures,
                    error.as_str(),
                    now(),
                    id,
                    self.session_id
                ],
            )
            .map_err(InternError::from)?;
        transaction.commit().map_err(InternError::from)?;
        Ok(status)
    }

    /// Fails owned processing work at once, for a failure that would come out
    /// the same on a second attempt: a password, a document past the limits,
    /// a hosted model that declined it.
    ///
    /// `record_processing_failure` gives every failure one more attempt, and
    /// for these that attempt only re-read the file, or re-sent and re-billed
    /// the request, before failing anyway. The failure count is raised to at
    /// least the automatic-retry limit so the row reads like any other
    /// document that has used its attempts.
    pub fn record_terminal_failure(&self, id: i64, error: ErrorCode) -> InternResult<()> {
        let connection = self.lock()?;
        let changed = connection
            .execute(
                "UPDATE queue_items
             SET status = 'failed', processing_failures = MAX(processing_failures + 1, 2),
                 error_code = ?1, owner_session = NULL, lease_expires_at = NULL,
                 updated_at = ?2
             WHERE id = ?3 AND status IN ('extracting', 'analyzing') AND owner_session = ?4",
                params![error.as_str(), now(), id, self.session_id],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "item is not owned processing work",
            ));
        }
        Ok(())
    }

    pub(crate) fn create_receipt(
        &self,
        queue_item_id: i64,
        mut receipt: OperationReceipt,
    ) -> InternResult<OperationReceipt> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let previous_status = transaction
            .query_row(
                "SELECT previous_status FROM queue_items
             WHERE id = ?1 AND status = 'applying' AND owner_session = ?2
               AND active_receipt_id IS NULL
               AND NOT EXISTS (
                 SELECT 1 FROM operation_receipts receipts
                 WHERE receipts.queue_item_id = ?1
                   AND receipts.stage NOT IN ('complete', 'rolled_back')
               )",
                params![queue_item_id, self.session_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(InternError::from)?;
        let direction_matches = matches!(
            (receipt.direction, previous_status.as_deref()),
            (OperationDirection::Apply, Some("ready"))
                | (OperationDirection::Undo, Some("completed"))
        );
        if !direction_matches || receipt.stage != OperationStage::Planned {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "planned receipt direction does not match the owned applying epoch",
            ));
        }
        receipt.queue_item_id = queue_item_id;
        let timestamp = now();
        let source_path = path_text(&receipt.source);
        let destination_path = path_text(&receipt.destination);
        let temporary_path = receipt.temporary_path.as_deref().map(path_text);
        transaction
            .execute(
                "INSERT INTO operation_receipts(
               queue_item_id, direction, source_path, destination_path, temporary_path,
               pre_hash, post_hash, operation_kind, stage, source_exists,
               destination_exists, temporary_exists, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
                params![
                    receipt.queue_item_id,
                    receipt.direction.as_db(),
                    source_path,
                    destination_path,
                    temporary_path,
                    receipt.pre_operation_hash,
                    receipt.post_operation_hash,
                    receipt.kind.as_db(),
                    receipt.stage.as_db(),
                    receipt.source_exists,
                    receipt.destination_exists,
                    receipt.temporary_exists,
                    timestamp,
                ],
            )
            .map_err(InternError::from)?;
        receipt.id = transaction.last_insert_rowid();
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET active_receipt_id = ?1, lease_expires_at = ?2, updated_at = ?3
             WHERE id = ?4 AND status = 'applying' AND owner_session = ?5
               AND active_receipt_id IS NULL",
                params![
                    receipt.id,
                    lease_deadline(),
                    timestamp,
                    queue_item_id,
                    self.session_id
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "applying epoch could not bind its receipt",
            ));
        }
        transaction.commit().map_err(InternError::from)?;
        Ok(receipt)
    }

    pub fn load_receipt(&self, queue_item_id: i64) -> InternResult<Option<OperationReceipt>> {
        let connection = self.lock()?;
        connection
            .query_row(
                &receipt_select("WHERE queue_item_id = ?1 ORDER BY id DESC LIMIT 1"),
                params![queue_item_id],
                row_to_receipt,
            )
            .optional()
            .map_err(InternError::from)
    }

    /// The item's newest finished operation: where its document is now.
    ///
    /// `load_receipt` answers with the newest receipt of any stage, and an
    /// undo that was refused and rolled back sits on top of the apply that
    /// filed the document without having moved anything. Comparing an undo
    /// against that, or deciding from it whether the item can be undone,
    /// refused every later undo for good. A completed undo does answer the
    /// question: the document is back at its own name.
    pub fn load_latest_complete_receipt(
        &self,
        queue_item_id: i64,
    ) -> InternResult<Option<OperationReceipt>> {
        let connection = self.lock()?;
        connection
            .query_row(
                &receipt_select(
                    "WHERE queue_item_id = ?1 AND stage = 'complete' ORDER BY id DESC LIMIT 1",
                ),
                params![queue_item_id],
                row_to_receipt,
            )
            .optional()
            .map_err(InternError::from)
    }

    /// The item's newest operation that never reached a terminal stage, if
    /// any. While the item is applying that is the operation in flight;
    /// otherwise it is one a reconciliation could not settle, and no new
    /// operation can be journalled for the item until it is.
    pub fn load_unsettled_receipt(
        &self,
        queue_item_id: i64,
    ) -> InternResult<Option<OperationReceipt>> {
        let connection = self.lock()?;
        connection
            .query_row(
                &receipt_select(UNSETTLED_RECEIPT),
                params![queue_item_id],
                row_to_receipt,
            )
            .optional()
            .map_err(InternError::from)
    }

    /// When a receipt last changed: for a finished operation, the moment it
    /// finished. `OperationReceipt` deliberately carries no timestamp - undo
    /// compares receipts for equality - so this is asked separately.
    pub fn receipt_updated_at(&self, receipt_id: i64) -> InternResult<Option<i64>> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT updated_at FROM operation_receipts WHERE id = ?1",
                params![receipt_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(InternError::from)
    }

    pub(crate) fn load_active_receipt(
        &self,
        queue_item_id: i64,
    ) -> InternResult<Option<OperationReceipt>> {
        let connection = self.lock()?;
        connection
            .query_row(
                &receipt_select(
                    "WHERE id = (
                   SELECT active_receipt_id FROM queue_items WHERE id = ?1
                 ) AND queue_item_id = ?1",
                ),
                params![queue_item_id],
                row_to_receipt,
            )
            .optional()
            .map_err(InternError::from)
    }

    pub(crate) fn update_receipt(
        &self,
        expected_stage: OperationStage,
        receipt: &OperationReceipt,
    ) -> InternResult<OperationReceipt> {
        if !expected_stage.can_advance_to(receipt.stage) {
            return Err(InternError::new(
                ErrorCode::InvalidTransition,
                "receipt stage transition is not permitted",
            ));
        }
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let timestamp = now();
        let renewed = transaction
            .execute(
                "UPDATE queue_items SET lease_expires_at = ?1, updated_at = ?2
             WHERE id = ?3 AND status = 'applying' AND owner_session = ?4
               AND active_receipt_id = ?5",
                params![
                    timestamp + LEASE_SECONDS,
                    timestamp,
                    receipt.queue_item_id,
                    self.session_id,
                    receipt.id
                ],
            )
            .map_err(InternError::from)?;
        if renewed != 1 {
            transaction.rollback().map_err(InternError::from)?;
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "receipt owner lease could not be renewed",
            ));
        }
        let changed = transaction
            .execute(
                "UPDATE operation_receipts
             SET temporary_path = ?1, post_hash = ?2, stage = ?3,
                 source_exists = ?4, destination_exists = ?5, temporary_exists = ?6,
                 updated_at = ?7
             WHERE id = ?8 AND queue_item_id = ?9 AND stage = ?10
               AND EXISTS(
                 SELECT 1 FROM queue_items
                 WHERE id = ?9 AND status = 'applying' AND owner_session = ?11
                   AND active_receipt_id = ?8
               )",
                params![
                    receipt.temporary_path.as_deref().map(path_text),
                    receipt.post_operation_hash,
                    receipt.stage.as_db(),
                    receipt.source_exists,
                    receipt.destination_exists,
                    receipt.temporary_exists,
                    timestamp,
                    receipt.id,
                    receipt.queue_item_id,
                    expected_stage.as_db(),
                    self.session_id,
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "receipt changed or applying ownership was lost",
            ));
        }
        let updated = transaction
            .query_row(
                &receipt_select("WHERE id = ?1"),
                params![receipt.id],
                row_to_receipt,
            )
            .map_err(InternError::from)?;
        transaction.commit().map_err(InternError::from)?;
        Ok(updated)
    }

    fn cas_status(
        &self,
        id: i64,
        expected: QueueStatus,
        next: QueueStatus,
        reset_failures: bool,
    ) -> InternResult<QueueItem> {
        let connection = self.lock()?;
        let changed = connection
            .execute(
                "UPDATE queue_items
             SET status = ?1,
                 processing_failures = CASE WHEN ?2 THEN 0 ELSE processing_failures END,
                 error_code = NULL, owner_session = NULL, lease_expires_at = NULL,
                 previous_status = NULL, active_receipt_id = NULL,
                 reconciliation_receipt_id = NULL, updated_at = ?3
             WHERE id = ?4 AND status = ?5",
                params![next.as_db(), reset_failures, now(), id, expected.as_db()],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "queue compare-and-swap failed",
            ));
        }
        query_one(&connection, "WHERE id = ?1", params![id])
    }

    fn finish_applying(
        &self,
        id: i64,
        receipt_id: i64,
        next: QueueStatus,
        required_previous: QueueStatus,
        direction: OperationDirection,
    ) -> InternResult<QueueItem> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(InternError::from)?;
        touch_session(&transaction, &self.session_id)?;
        let changed = transaction
            .execute(
                "UPDATE queue_items
             SET status = ?1, owner_session = NULL, lease_expires_at = NULL,
                 previous_status = NULL, active_receipt_id = NULL,
                 reconciliation_receipt_id = NULL,
                 error_code = NULL, updated_at = ?2
             WHERE id = ?3 AND status = 'applying' AND owner_session = ?4
               AND previous_status = ?5 AND active_receipt_id = ?6
               AND EXISTS (
                 SELECT 1 FROM operation_receipts receipts
                 WHERE receipts.id = ?6 AND receipts.queue_item_id = ?3
                   AND receipts.direction = ?7 AND receipts.stage = 'complete'
               )",
                params![
                    next.as_db(),
                    now(),
                    id,
                    self.session_id,
                    required_previous.as_db(),
                    receipt_id,
                    direction.as_db(),
                ],
            )
            .map_err(InternError::from)?;
        if changed != 1 {
            return Err(InternError::new(
                ErrorCode::StateConflict,
                "applying completion compare-and-swap failed",
            ));
        }
        let item = query_one(&transaction, "WHERE id = ?1", params![id])?;
        transaction.commit().map_err(InternError::from)?;
        Ok(item)
    }

    fn lock(&self) -> InternResult<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| InternError::new(ErrorCode::DatabaseUnavailable, "database lock poisoned"))
    }
}

impl Drop for QueueStore {
    fn drop(&mut self) {
        if let Ok(connection) = self.connection.lock() {
            // Failure is deliberately fail-closed: the item lease and stale
            // heartbeat still prevent another live session from stealing work.
            let _ = connection.execute(
                "DELETE FROM queue_sessions WHERE session_id = ?1",
                params![self.session_id],
            );
        }
    }
}

fn migrate_legacy_schema(connection: &mut Connection) -> InternResult<()> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(InternError::from)?;
    let has_v2_marker = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 2)",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(InternError::from)?;
    let migrating_to_v3 =
        has_v2_marker && !column_exists(&transaction, "queue_items", "active_receipt_id")?;
    for (column, definition) in [
        (
            "owner_session",
            "owner_session TEXT REFERENCES queue_sessions(session_id) ON DELETE SET NULL",
        ),
        ("lease_expires_at", "lease_expires_at INTEGER"),
        ("previous_status", "previous_status TEXT"),
        ("active_receipt_id", "active_receipt_id INTEGER"),
        (
            "reconciliation_receipt_id",
            "reconciliation_receipt_id INTEGER",
        ),
        (
            "applying_epoch",
            "applying_epoch INTEGER NOT NULL DEFAULT 0",
        ),
    ] {
        if !column_exists(&transaction, "queue_items", column)? {
            transaction
                .execute(
                    &format!("ALTER TABLE queue_items ADD COLUMN {definition}"),
                    [],
                )
                .map_err(InternError::from)?;
        }
    }
    if column_exists(&transaction, "operation_receipts", "receipt_json")? {
        transaction
            .execute(
                "ALTER TABLE operation_receipts RENAME TO operation_receipts_legacy_v1",
                [],
            )
            .map_err(InternError::from)?;
        transaction
            .execute_batch(
                "CREATE TABLE operation_receipts (
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
                 );",
            )
            .map_err(InternError::from)?;
    }
    if migrating_to_v3 {
        transaction
            .execute(
                "UPDATE queue_items
             SET active_receipt_id = (
               SELECT receipts.id FROM operation_receipts receipts
               WHERE receipts.queue_item_id = queue_items.id
                 AND receipts.id = (
                   SELECT MAX(latest.id) FROM operation_receipts latest
                   WHERE latest.queue_item_id = queue_items.id
                 )
                 AND receipts.stage <> 'rolled_back'
                 AND (
                   (queue_items.previous_status = 'ready' AND receipts.direction = 'apply'
                     AND receipts.source_path = queue_items.source_path)
                   OR (queue_items.previous_status = 'completed' AND receipts.direction = 'undo'
                     AND receipts.destination_path = queue_items.source_path)
                 )
             )
             WHERE status = 'applying' AND active_receipt_id IS NULL
               AND 1 = (
                 SELECT COUNT(*) FROM operation_receipts receipts
                 WHERE receipts.queue_item_id = queue_items.id
                   AND receipts.id = (
                     SELECT MAX(latest.id) FROM operation_receipts latest
                     WHERE latest.queue_item_id = queue_items.id
                   )
                   AND receipts.stage <> 'rolled_back'
                   AND (
                     (queue_items.previous_status = 'ready' AND receipts.direction = 'apply'
                       AND receipts.source_path = queue_items.source_path)
                     OR (queue_items.previous_status = 'completed' AND receipts.direction = 'undo'
                       AND receipts.destination_path = queue_items.source_path)
                   )
               )
               AND NOT EXISTS (
                 SELECT 1 FROM operation_receipts current_receipts
                 WHERE current_receipts.queue_item_id = queue_items.id
                   AND current_receipts.stage NOT IN ('complete', 'rolled_back')
                 GROUP BY current_receipts.queue_item_id
                 HAVING COUNT(*) > 1
               )",
                [],
            )
            .map_err(InternError::from)?;
    }
    let duplicate_nonterminal_receipts = transaction
        .query_row(
            "SELECT EXISTS(
           SELECT 1 FROM operation_receipts
           WHERE stage NOT IN ('complete', 'rolled_back')
           GROUP BY queue_item_id HAVING COUNT(*) > 1
         )",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(InternError::from)?;
    if !duplicate_nonterminal_receipts {
        transaction
            .execute_batch(
                "CREATE UNIQUE INDEX IF NOT EXISTS one_active_receipt_per_item
               ON operation_receipts(queue_item_id)
               WHERE stage NOT IN ('complete', 'rolled_back');",
            )
            .map_err(InternError::from)?;
    }
    transaction
        .execute(
            "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (3, ?1)",
            params![now()],
        )
        .map_err(InternError::from)?;
    transaction
        .execute(
            "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (4, ?1)",
            params![now()],
        )
        .map_err(InternError::from)?;
    transaction.commit().map_err(InternError::from)
}

fn column_exists(connection: &Connection, table: &str, column: &str) -> InternResult<bool> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(InternError::from)?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(InternError::from)?;
    for name in names {
        if name.map_err(InternError::from)? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn touch_session(connection: &Connection, session_id: &str) -> InternResult<()> {
    let changed = connection
        .execute(
            "UPDATE queue_sessions SET heartbeat_at = ?1 WHERE session_id = ?2",
            params![now(), session_id],
        )
        .map_err(InternError::from)?;
    if changed != 1 {
        return Err(InternError::new(
            ErrorCode::StateConflict,
            "queue session is no longer live",
        ));
    }
    Ok(())
}

fn query_one<P>(connection: &Connection, suffix: &str, parameters: P) -> InternResult<QueueItem>
where
    P: rusqlite::Params,
{
    connection
        .query_row(&queue_select(suffix), parameters, row_to_item)
        .map_err(InternError::from)
}

fn queue_select(suffix: &str) -> String {
    format!(
        "SELECT id, source_path, source_hash, status, processing_failures, error_code,
                owner_session, lease_expires_at, previous_status, active_receipt_id,
                reconciliation_receipt_id, created_at, updated_at
         FROM queue_items {suffix}"
    )
}

/// A queue row that was just written, or that must be read whole: one this
/// build cannot read is an error.
fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<QueueItem> {
    read_item(row)?.ok_or_else(|| invalid_column(3, "unknown queue status"))
}

/// A queue row as this build reads it, or `None` for a row it cannot: one
/// whose status, or status before applying, a newer build wrote. Nothing can
/// be decided about an item in a state this build has never heard of, so it
/// is left out rather than guessed at.
///
/// An error code this build does not know reads as no code. It is what the
/// window explains, not what the queue decides on - the decisions are made by
/// compare-and-swaps in SQL, against the stored text - and every release adds
/// codes, so it is the unknown value a database written by a newer release
/// is most likely to hold.
fn read_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Option<QueueItem>> {
    let Some(status) = QueueStatus::from_db(&row.get::<_, String>(3)?) else {
        return Ok(None);
    };
    let previous_status = match row.get::<_, Option<String>>(8)? {
        Some(value) => match QueueStatus::from_db(&value) {
            Some(previous) => Some(previous),
            None => return Ok(None),
        },
        None => None,
    };
    let error = row.get::<_, Option<String>>(5)?.and_then(|value| {
        let code = ErrorCode::from_str(&value);
        if code.is_none() && !UNKNOWN_ERROR_CODE_REPORTED.swap(true, Ordering::Relaxed) {
            eprintln!(
                "intern: a queue item carries an error code this version does not know; it is shown without one"
            );
        }
        code
    });
    Ok(Some(QueueItem {
        id: row.get(0)?,
        source_path: PathBuf::from(row.get::<_, String>(1)?),
        source_hash: row.get(2)?,
        status,
        processing_failures: u32::try_from(row.get::<_, i64>(4)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Integer,
                Box::new(error),
            )
        })?,
        error_code: error,
        owner_session: row.get(6)?,
        lease_expires_at: row.get(7)?,
        previous_status,
        active_receipt_id: row.get(9)?,
        reconciliation_receipt_id: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    }))
}

fn receipt_select(suffix: &str) -> String {
    format!(
        "SELECT id, queue_item_id, direction, source_path, destination_path, temporary_path,
                pre_hash, post_hash, operation_kind, stage, source_exists,
                destination_exists, temporary_exists
         FROM operation_receipts {suffix}"
    )
}

fn row_to_receipt(row: &rusqlite::Row<'_>) -> rusqlite::Result<OperationReceipt> {
    let direction_text: String = row.get(2)?;
    let kind_text: String = row.get(8)?;
    let stage_text: String = row.get(9)?;
    Ok(OperationReceipt {
        id: row.get(0)?,
        queue_item_id: row.get(1)?,
        direction: OperationDirection::from_db(&direction_text)
            .ok_or_else(|| invalid_column(2, "unknown receipt direction"))?,
        source: PathBuf::from(row.get::<_, String>(3)?),
        destination: PathBuf::from(row.get::<_, String>(4)?),
        temporary_path: row.get::<_, Option<String>>(5)?.map(PathBuf::from),
        pre_operation_hash: row.get(6)?,
        post_operation_hash: row.get(7)?,
        kind: OperationKind::from_db(&kind_text)
            .ok_or_else(|| invalid_column(8, "unknown operation kind"))?,
        stage: OperationStage::from_db(&stage_text)
            .ok_or_else(|| invalid_column(9, "unknown operation stage"))?,
        source_exists: row.get(10)?,
        destination_exists: row.get(11)?,
        temporary_exists: row.get(12)?,
    })
}

/// A finished operation as this build reads it, or `None` for one whose
/// direction, kind or stage a newer build wrote. The history listing leaves
/// such a receipt out and counts it, as the queue listing does a row.
fn read_history_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Option<HistoryEntry>> {
    let direction = OperationDirection::from_db(&row.get::<_, String>(3)?);
    let kind = OperationKind::from_db(&row.get::<_, String>(4)?);
    let stage = OperationStage::from_db(&row.get::<_, String>(5)?);
    let (Some(direction), Some(kind), Some(stage)) = (direction, kind, stage) else {
        return Ok(None);
    };
    Ok(Some(HistoryEntry {
        receipt_id: row.get(0)?,
        queue_item_id: row.get(1)?,
        at: row.get(2)?,
        direction,
        kind,
        stage,
        original_path: PathBuf::from(row.get::<_, String>(6)?),
        new_path: PathBuf::from(row.get::<_, String>(7)?),
    }))
}

fn invalid_column(index: usize, message: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        )),
    )
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn is_active(status: QueueStatus) -> bool {
    matches!(
        status,
        QueueStatus::Extracting | QueueStatus::Analyzing | QueueStatus::Applying
    )
}

fn lease_deadline() -> i64 {
    now() + LEASE_SECONDS
}

fn new_session_id() -> String {
    let sequence = SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{nanos}-{sequence}", std::process::id())
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn windows_path_key(path: &str) -> String {
    path.replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

/// The normalized key `enqueue` stores for a source path, for callers that
/// need to compare against `source_path_key` (e.g. duplicate lookups).
pub fn source_path_key(path: &Path) -> String {
    windows_path_key(&path.to_string_lossy())
}
