//! Renames in place for documents Intern could not read: `NEEDS REVIEW - `
//! put in front of the name, the queue row following the file, and every
//! crash window between the rename and its record settled from the disk.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use intern_core::{
    ErrorCode, FileApplier, FileSystem, LockedFile, QueueStatus, QueueStore, ReviewMark,
    ReviewRenamePurpose, ReviewRenameStage, StdFileSystem, source_path_key,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn database(temp: &TempDir) -> PathBuf {
    temp.path().join("queue.sqlite3")
}

/// A document in `folder` that Intern read and could not: in review under
/// `code`, with nothing renamed yet.
fn unreadable(store: &QueueStore, folder: &Path, name: &str, code: ErrorCode) -> (i64, PathBuf) {
    let path = folder.join(name);
    fs::write(&path, format!("{name} bytes")).unwrap();
    let hash = StdFileSystem.hash(&path).unwrap();
    let item = store.enqueue(&path, &hash).unwrap();
    assert_eq!(store.claim_next().unwrap().unwrap().id, item.id);
    store
        .record_unreadable(item.id, code, ReviewMark::Now)
        .unwrap();
    (item.id, path)
}

fn inbox(temp: &TempDir) -> PathBuf {
    let folder = temp.path().join("Inbox");
    fs::create_dir_all(&folder).unwrap();
    folder
}

fn row(store: &QueueStore, id: i64) -> intern_core::QueueItem {
    store.get(id).unwrap().unwrap()
}

fn stages(temp: &TempDir) -> Vec<(String, String)> {
    let connection = Connection::open(database(temp)).unwrap();
    let mut statement = connection
        .prepare("SELECT purpose, stage FROM review_renames ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn an_unreadable_document_is_renamed_where_it_lies_and_its_row_follows() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::PasswordProtected);
    let applier = FileApplier::local(Arc::clone(&store));

    let renamed = applier.mark_for_review(id).unwrap().unwrap();

    assert_eq!(renamed, folder.join("NEEDS REVIEW - scan.pdf"));
    assert!(renamed.exists());
    assert!(!original.exists());
    let item = row(&store, id);
    assert_eq!(item.source_path, renamed);
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(item.error_code, Some(ErrorCode::PasswordProtected));
    assert_eq!(
        stages(&temp),
        vec![("mark".to_string(), "complete".to_string())]
    );
    // Asked again - a second failure after a retry - the name is not marked
    // twice.
    assert_eq!(applier.mark_for_review(id).unwrap(), None);
    assert!(renamed.exists());
}

#[test]
fn a_taken_review_name_gets_the_next_number_and_nothing_is_overwritten() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    fs::write(folder.join("NEEDS REVIEW - scan.pdf"), b"someone else's").unwrap();
    let (id, _) = unreadable(&store, &folder, "scan.pdf", ErrorCode::ExtractionFailed);

    let renamed = FileApplier::local(Arc::clone(&store))
        .mark_for_review(id)
        .unwrap()
        .unwrap();

    assert_eq!(renamed, folder.join("NEEDS REVIEW - scan (2).pdf"));
    assert_eq!(
        fs::read(folder.join("NEEDS REVIEW - scan.pdf")).unwrap(),
        b"someone else's"
    );
    assert_eq!(fs::read(&renamed).unwrap(), b"scan.pdf bytes");
}

#[test]
fn only_a_document_in_review_under_an_unreadable_code_is_marked() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let path = folder.join("fine.pdf");
    fs::write(&path, b"fine").unwrap();
    let item = store.enqueue(&path, "hash").unwrap();
    let applier = FileApplier::local(Arc::clone(&store));
    assert_eq!(applier.mark_for_review(item.id).unwrap(), None);
    assert!(path.exists());
    // Failed is not review, whatever the code.
    store.claim_next().unwrap();
    store
        .record_terminal_failure(item.id, ErrorCode::OcrUnavailable)
        .unwrap();
    assert_eq!(applier.mark_for_review(item.id).unwrap(), None);
    assert!(path.exists());
}

/// The crash after the rename was journaled and before it happened: the
/// document is still at its old name. Recovery records that nothing moved,
/// and then makes the mark that never happened.
#[test]
fn a_crash_before_the_rename_is_rolled_back_and_the_mark_made_on_recovery() {
    let temp = TempDir::new().unwrap();
    let folder = inbox(&temp);
    let (id, original) = {
        let store = QueueStore::open(database(&temp)).unwrap();
        let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::ModelDeclined);
        store
            .plan_review_rename(
                id,
                ReviewRenamePurpose::Mark,
                &original,
                &folder.join("NEEDS REVIEW - scan.pdf"),
            )
            .unwrap();
        (id, original)
    };
    assert!(original.exists());

    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let settled = FileApplier::local(Arc::clone(&store))
        .reconcile_review_renames()
        .unwrap();

    assert_eq!(settled, 1);
    let marked = folder.join("NEEDS REVIEW - scan.pdf");
    assert!(marked.exists());
    assert!(!original.exists());
    assert_eq!(row(&store, id).source_path, marked);
    assert_eq!(
        stages(&temp),
        vec![
            ("mark".to_string(), "rolled_back".to_string()),
            ("mark".to_string(), "complete".to_string()),
        ]
    );
}

/// The crash after the rename landed and before the row moved: the file is
/// at its new name and the row still at the old one. Recovery moves the row.
#[test]
fn a_crash_after_the_rename_moves_the_row_on_recovery() {
    let temp = TempDir::new().unwrap();
    let folder = inbox(&temp);
    let marked = folder.join("NEEDS REVIEW - scan.pdf");
    let id = {
        let store = QueueStore::open(database(&temp)).unwrap();
        let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::DocumentTooLarge);
        store
            .plan_review_rename(id, ReviewRenamePurpose::Mark, &original, &marked)
            .unwrap();
        fs::rename(&original, &marked).unwrap();
        id
    };

    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    FileApplier::local(Arc::clone(&store))
        .reconcile_review_renames()
        .unwrap();

    assert_eq!(row(&store, id).source_path, marked);
    assert_eq!(
        stages(&temp),
        vec![("mark".to_string(), "complete".to_string())],
        "the landed rename is completed, not tried a second time"
    );
}

/// The crash after the document reached review and before its rename was
/// even journaled: nothing records a mark at all, so recovery makes one.
#[test]
fn a_document_left_in_review_without_its_mark_is_marked_on_recovery() {
    let temp = TempDir::new().unwrap();
    let folder = inbox(&temp);
    let id = {
        let store = QueueStore::open(database(&temp)).unwrap();
        unreadable(&store, &folder, "scan.pdf", ErrorCode::ModelOutputInvalid).0
    };

    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let settled = FileApplier::local(Arc::clone(&store))
        .reconcile_review_renames()
        .unwrap();

    assert_eq!(settled, 0);
    assert_eq!(
        row(&store, id).source_path,
        folder.join("NEEDS REVIEW - scan.pdf")
    );
}

/// Both names taken after a crash: the bytes say which one is the document.
#[test]
fn both_names_present_after_a_crash_are_settled_by_the_documents_bytes() {
    let temp = TempDir::new().unwrap();
    let folder = inbox(&temp);
    let marked = folder.join("NEEDS REVIEW - landed.pdf");
    let foreign = folder.join("NEEDS REVIEW - stayed.pdf");
    let (landed, stayed, landed_original, stayed_original) = {
        let store = QueueStore::open(database(&temp)).unwrap();
        let (landed, landed_original) =
            unreadable(&store, &folder, "landed.pdf", ErrorCode::AnalysisFailed);
        let (stayed, stayed_original) =
            unreadable(&store, &folder, "stayed.pdf", ErrorCode::AnalysisFailed);
        store
            .plan_review_rename(landed, ReviewRenamePurpose::Mark, &landed_original, &marked)
            .unwrap();
        store
            .plan_review_rename(
                stayed,
                ReviewRenamePurpose::Mark,
                &stayed_original,
                &foreign,
            )
            .unwrap();
        // The first landed, and a new file arrived under its old name.
        fs::rename(&landed_original, &marked).unwrap();
        fs::write(&landed_original, b"a new upload").unwrap();
        // The second never moved, and something else took its new name.
        fs::write(&foreign, b"not the document").unwrap();
        (landed, stayed, landed_original, stayed_original)
    };

    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    FileApplier::local(Arc::clone(&store))
        .reconcile_review_renames()
        .unwrap();

    assert_eq!(row(&store, landed).source_path, marked);
    assert_eq!(fs::read(&landed_original).unwrap(), b"a new upload");
    // Rolled back, then marked again under the next free number.
    assert_eq!(
        row(&store, stayed).source_path,
        folder.join("NEEDS REVIEW - stayed (2).pdf")
    );
    assert!(!stayed_original.exists());
    assert_eq!(fs::read(&foreign).unwrap(), b"not the document");
}

/// A file system that refuses every rename, as a folder the account may not
/// write does.
struct RefusingRenames {
    refuse: AtomicBool,
}

impl FileSystem for RefusingRenames {
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
        if self.refuse.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected refusal",
            ));
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

#[test]
fn a_refused_rename_moves_nothing_and_is_not_retried_on_its_own() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::PasswordProtected);
    let filesystem = Arc::new(RefusingRenames {
        refuse: AtomicBool::new(true),
    });
    let applier = FileApplier::new(filesystem.clone(), Arc::clone(&store));

    let error = applier.mark_for_review(id).unwrap_err();

    assert_eq!(error.code(), ErrorCode::IoError);
    assert!(original.exists());
    let item = row(&store, id);
    assert_eq!(item.source_path, original);
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(
        stages(&temp),
        vec![("mark".to_string(), "failed".to_string())]
    );
    // Recovery does not try a refused mark again, even once it would work.
    filesystem.refuse.store(false, Ordering::SeqCst);
    applier.reconcile_review_renames().unwrap();
    assert!(original.exists());
    assert_eq!(row(&store, id).source_path, original);
}

#[test]
fn keeping_the_original_takes_the_mark_back_and_completes_the_item() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::UnsupportedContent);
    let applier = FileApplier::local(Arc::clone(&store));
    applier.mark_for_review(id).unwrap().unwrap();

    let kept = applier.restore_from_review(id).unwrap().unwrap();

    assert_eq!(kept.status, QueueStatus::Completed);
    assert_eq!(kept.error_code, None);
    assert_eq!(kept.source_path, original);
    assert!(original.exists());
    assert!(!folder.join("NEEDS REVIEW - scan.pdf").exists());
    // Taken back once, there is no mark left to take back.
    assert!(applier.restore_from_review(id).unwrap().is_none());
}

#[test]
fn a_restored_name_that_was_taken_since_gets_the_next_number() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::UnsupportedContent);
    let applier = FileApplier::local(Arc::clone(&store));
    applier.mark_for_review(id).unwrap().unwrap();
    fs::write(&original, b"a new upload under the old name").unwrap();

    let kept = applier.restore_from_review(id).unwrap().unwrap();

    assert_eq!(kept.source_path, folder.join("scan (2).pdf"));
    assert_eq!(
        fs::read(&original).unwrap(),
        b"a new upload under the old name"
    );
}

/// A file whose prefix Intern did not put there is the person's name, and
/// keeping it renames nothing.
#[test]
fn a_name_that_arrived_with_the_prefix_is_not_restored() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, path) = unreadable(
        &store,
        &folder,
        "NEEDS REVIEW - mine.pdf",
        ErrorCode::PasswordProtected,
    );
    let applier = FileApplier::local(Arc::clone(&store));
    assert_eq!(applier.mark_for_review(id).unwrap(), None);
    assert!(applier.restore_from_review(id).unwrap().is_none());
    assert!(path.exists());
}

/// The crash after keep-original renamed the file back and before it was
/// recorded: recovery moves the row and completes the item as kept, so the
/// document is not marked all over again.
#[test]
fn a_crash_during_a_restore_completes_it_on_recovery() {
    let temp = TempDir::new().unwrap();
    let folder = inbox(&temp);
    let marked = folder.join("NEEDS REVIEW - scan.pdf");
    let (id, original) = {
        let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
        let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::ExtractionFailed);
        FileApplier::local(Arc::clone(&store))
            .mark_for_review(id)
            .unwrap();
        store
            .plan_review_rename(id, ReviewRenamePurpose::Restore, &marked, &original)
            .unwrap();
        fs::rename(&marked, &original).unwrap();
        (id, original)
    };

    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    FileApplier::local(Arc::clone(&store))
        .reconcile_review_renames()
        .unwrap();

    let item = row(&store, id);
    assert_eq!(item.status, QueueStatus::Completed);
    assert_eq!(item.source_path, original);
    assert!(original.exists());
    assert!(!marked.exists());
}

/// The watcher knows a document by the name it arrived under; the row it
/// asks about must still be found once the file carries the prefix.
#[test]
fn a_marked_document_is_still_found_by_the_name_it_arrived_under() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::PasswordProtected);
    let marked = FileApplier::local(Arc::clone(&store))
        .mark_for_review(id)
        .unwrap()
        .unwrap();

    assert_eq!(
        store
            .find_newest_by_intake_path(&[original.as_path()])
            .unwrap()
            .map(|item| item.id),
        Some(id)
    );
    assert_eq!(
        store
            .find_newest_by_intake_path(&[marked.as_path()])
            .unwrap()
            .map(|item| item.id),
        Some(id)
    );
    // The plain lookup follows the file only.
    assert!(
        store
            .find_newest_by_source_path(&[original.as_path()])
            .unwrap()
            .is_none()
    );
    // A new upload under the old name is the newer row for that name.
    fs::write(&original, b"a new upload").unwrap();
    let newer = store.enqueue(&original, "new-hash").unwrap();
    assert_eq!(
        store
            .find_newest_by_intake_path(&[original.as_path()])
            .unwrap()
            .map(|item| item.id),
        Some(newer.id)
    );
}

#[test]
fn a_plan_is_refused_once_the_document_moved_on() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::PasswordProtected);
    store.retry_unreadable(id, None).unwrap();
    let error = store
        .plan_review_rename(
            id,
            ReviewRenamePurpose::Mark,
            &original,
            &folder.join("NEEDS REVIEW - scan.pdf"),
        )
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::StateConflict);
    assert_eq!(row(&store, id).status, QueueStatus::Queued);
    assert!(stages(&temp).is_empty());
}

#[test]
fn a_settled_plan_cannot_be_committed_or_settled_again() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::PasswordProtected);
    let plan = store
        .plan_review_rename(
            id,
            ReviewRenamePurpose::Mark,
            &original,
            &folder.join("NEEDS REVIEW - scan.pdf"),
        )
        .unwrap();
    store
        .settle_review_rename(&plan, ReviewRenameStage::RolledBack)
        .unwrap();
    assert!(store.commit_review_rename(&plan).is_err());
    assert!(
        store
            .settle_review_rename(&plan, ReviewRenameStage::Failed)
            .is_err()
    );
    assert_eq!(row(&store, id).source_path, original);
    assert_eq!(
        source_path_key(&row(&store, id).source_path),
        source_path_key(&original)
    );
}

#[test]
fn a_failure_that_uses_up_its_attempts_goes_to_review_only_when_asked() {
    let temp = TempDir::new().unwrap();
    let store = QueueStore::open(database(&temp)).unwrap();
    let crashed = store.enqueue(Path::new("/in/crashes.pdf"), "a").unwrap();
    let busy = store.enqueue(Path::new("/in/busy.pdf"), "b").unwrap();
    for _ in 0..2 {
        let claimed = store.claim_next().unwrap().unwrap();
        assert_eq!(claimed.id, crashed.id);
        store
            .record_retryable_failure(
                crashed.id,
                ErrorCode::IoError,
                Some((ErrorCode::ExtractionFailed, ReviewMark::Defer)),
            )
            .unwrap();
    }
    let item = store.get(crashed.id).unwrap().unwrap();
    assert_eq!(item.status, QueueStatus::NeedsReview);
    assert_eq!(item.error_code, Some(ErrorCode::ExtractionFailed));
    assert_eq!(item.processing_failures, 2);

    for _ in 0..2 {
        store.claim_next().unwrap().unwrap();
        store
            .record_processing_failure(busy.id, ErrorCode::IoError)
            .unwrap();
    }
    let item = store.get(busy.id).unwrap().unwrap();
    assert_eq!(item.status, QueueStatus::Failed);
    assert_eq!(item.error_code, Some(ErrorCode::IoError));

    // Retry takes an unreadable document back to the queue, and nothing else.
    assert!(store.retry_unreadable(busy.id, None).is_err());
    let retried = store.retry_unreadable(crashed.id, None).unwrap();
    assert_eq!(retried.status, QueueStatus::Queued);
    assert_eq!(retried.error_code, None);
    assert_eq!(retried.processing_failures, 0);
}

/// The crash after keep-original was journaled, with the marked file since
/// gone and a new upload under the document's own name. The name being
/// free of the marked file is no proof the restore landed: only the
/// document's bytes at its own name are, and a stranger's are not.
#[test]
fn a_restore_is_not_taken_as_landed_on_a_strangers_file() {
    let temp = TempDir::new().unwrap();
    let folder = inbox(&temp);
    let marked = folder.join("NEEDS REVIEW - scan.pdf");
    let (id, original) = {
        let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
        let (id, original) = unreadable(&store, &folder, "scan.pdf", ErrorCode::ExtractionFailed);
        FileApplier::local(Arc::clone(&store))
            .mark_for_review(id)
            .unwrap();
        store
            .plan_review_rename(id, ReviewRenamePurpose::Restore, &marked, &original)
            .unwrap();
        (id, original)
    };
    fs::remove_file(&marked).unwrap();
    fs::write(&original, b"a stranger's upload").unwrap();

    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    FileApplier::local(Arc::clone(&store))
        .reconcile_review_renames()
        .unwrap();

    let item = row(&store, id);
    assert_eq!(item.status, QueueStatus::NeedsReview, "nothing was kept");
    assert_eq!(item.source_path, marked);
}

/// A document that was marked and then canceled, or failed after a retry,
/// can be cleared away with the history; its file gets its own name back
/// first, or it would be skipped by every scan with no row behind it.
#[test]
fn a_marked_document_that_is_canceled_or_fails_gets_its_name_back() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let applier = FileApplier::local(Arc::clone(&store));
    let (canceled, canceled_path) = unreadable(
        &store,
        &folder,
        "canceled.pdf",
        ErrorCode::PasswordProtected,
    );
    applier.mark_for_review(canceled).unwrap().unwrap();
    store
        .transition(
            canceled,
            QueueStatus::NeedsReview,
            QueueStatus::Canceled,
            None,
        )
        .unwrap();
    // Asked about a document still in review, nothing is taken back.
    let (waiting, _) = unreadable(&store, &folder, "waiting.pdf", ErrorCode::PasswordProtected);
    let waiting_marked = applier.mark_for_review(waiting).unwrap().unwrap();
    assert!(applier.unmark_settled(waiting).unwrap().is_none());

    let unmarked = applier.unmark_settled(canceled).unwrap().unwrap();
    assert_eq!(unmarked.status, QueueStatus::Canceled);
    assert_eq!(unmarked.source_path, canceled_path);
    assert!(canceled_path.exists());
    assert!(waiting_marked.exists());

    // A failure the crash left marked is taken back by recovery.
    let (failed, failed_path) =
        unreadable(&store, &folder, "failed.pdf", ErrorCode::PasswordProtected);
    applier.mark_for_review(failed).unwrap().unwrap();
    store.retry_unreadable(failed, None).unwrap();
    for _ in 0..2 {
        assert_eq!(store.claim_next().unwrap().unwrap().id, failed);
        store
            .record_processing_failure(failed, ErrorCode::IoError)
            .unwrap();
    }
    assert_eq!(row(&store, failed).status, QueueStatus::Failed);
    applier.reconcile_review_renames().unwrap();
    assert_eq!(row(&store, failed).source_path, failed_path);
    assert!(failed_path.exists());
    assert_eq!(row(&store, waiting).source_path, waiting_marked);
}

/// A mark decided against, or put off, is recorded with the review, so
/// recovery - which marks any document in review that has no mark at all -
/// never makes it.
#[test]
fn a_mark_put_off_or_decided_against_is_never_made_by_recovery() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let mut ids = Vec::new();
    for (name, mark) in [
        ("never.pdf", ReviewMark::Never),
        ("later.pdf", ReviewMark::Defer),
    ] {
        let path = folder.join(name);
        fs::write(&path, name).unwrap();
        let item = store.enqueue(&path, name).unwrap();
        assert_eq!(store.claim_next().unwrap().unwrap().id, item.id);
        store
            .record_unreadable(item.id, ErrorCode::ModelOutputInvalid, mark)
            .unwrap();
        ids.push((item.id, path));
    }
    let applier = FileApplier::local(Arc::clone(&store));
    applier.reconcile_review_renames().unwrap();
    for (id, path) in &ids {
        assert_eq!(&row(&store, *id).source_path, path);
        assert!(path.exists());
    }

    // A later success proves the put-off one was the document's problem.
    assert_eq!(store.promote_deferred_marks().unwrap(), vec![ids[1].0]);
    assert!(store.promote_deferred_marks().unwrap().is_empty());
    applier.reconcile_review_renames().unwrap();
    assert_eq!(
        row(&store, ids[1].0).source_path,
        folder.join("NEEDS REVIEW - later.pdf")
    );
    assert_eq!(row(&store, ids[0].0).source_path, ids[0].1);
}

/// A run of failures that stops the queue puts its own documents back to
/// wait - and only those - under their own names, with nothing counted. A
/// document is counted once however often it failed, and one a run put
/// back never counts again, so a resumed queue cannot stop on it again.
#[test]
fn a_run_that_stops_the_queue_requeues_its_own_documents_once() {
    let temp = TempDir::new().unwrap();
    let store = QueueStore::open(database(&temp)).unwrap();
    let days_ago = store.enqueue(Path::new("/in/old.pdf"), "old").unwrap();
    let put_off = store.enqueue(Path::new("/in/a.pdf"), "a").unwrap();
    let decided = store.enqueue(Path::new("/in/b.pdf"), "b").unwrap();
    let retrying = store.enqueue(Path::new("/in/c.pdf"), "c").unwrap();

    // A put-off document from an earlier run, which a resume ended.
    assert_eq!(store.claim_next().unwrap().unwrap().id, days_ago.id);
    store
        .record_unreadable(
            days_ago.id,
            ErrorCode::ModelOutputInvalid,
            ReviewMark::Defer,
        )
        .unwrap();
    assert_eq!(
        store.join_failure_streak(days_ago.id, "reply").unwrap(),
        Some(1)
    );
    store.end_failure_streaks(None, None).unwrap();

    for (item, mark) in [(&put_off, ReviewMark::Defer), (&decided, ReviewMark::Now)] {
        assert_eq!(store.claim_next().unwrap().unwrap().id, item.id);
        store
            .record_unreadable(item.id, ErrorCode::ModelOutputInvalid, mark)
            .unwrap();
    }
    assert_eq!(
        store.join_failure_streak(put_off.id, "reply").unwrap(),
        Some(1)
    );
    // Twice is still one document; another kind is another run.
    assert_eq!(
        store.join_failure_streak(put_off.id, "reply").unwrap(),
        Some(1)
    );
    assert_eq!(store.claim_next().unwrap().unwrap().id, retrying.id);
    store
        .record_processing_failure(retrying.id, ErrorCode::ModelFailed)
        .unwrap();
    assert_eq!(
        store.join_failure_streak(retrying.id, "timeout").unwrap(),
        Some(1)
    );

    assert_eq!(store.requeue_failure_streak().unwrap(), 2);
    let requeued = row(&store, put_off.id);
    assert_eq!(requeued.status, QueueStatus::Queued);
    assert_eq!(requeued.error_code, None);
    assert_eq!(requeued.processing_failures, 0);
    assert_eq!(row(&store, retrying.id).processing_failures, 0);
    assert_eq!(row(&store, decided.id).status, QueueStatus::NeedsReview);
    // The earlier run's document still waits in review for its person.
    assert_eq!(row(&store, days_ago.id).status, QueueStatus::NeedsReview);
    assert_eq!(store.promote_deferred_marks().unwrap(), vec![days_ago.id]);

    // Put back once, they never count towards a run again, until read.
    assert_eq!(
        store.join_failure_streak(put_off.id, "reply").unwrap(),
        None
    );
    assert_eq!(
        store.join_failure_streak(retrying.id, "reply").unwrap(),
        None
    );
    store.end_failure_streaks(None, Some(put_off.id)).unwrap();
    assert_eq!(
        store.join_failure_streak(put_off.id, "reply").unwrap(),
        Some(1)
    );
}

/// The mark of a canceled document whose file is gone is given up rather
/// than tried on every recovery pass; one whose name is refused back is
/// tried again only now and then, and the journal keeps one refusal.
#[test]
fn a_mark_that_cannot_be_taken_back_is_not_tried_for_ever() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let filesystem = Arc::new(RefusingRenames {
        refuse: AtomicBool::new(false),
    });
    let applier = FileApplier::new(filesystem.clone(), Arc::clone(&store));
    let cancel = |id| {
        store
            .transition(id, QueueStatus::NeedsReview, QueueStatus::Canceled, None)
            .unwrap();
    };

    let (gone, _) = unreadable(&store, &folder, "gone.pdf", ErrorCode::PasswordProtected);
    let gone_marked = applier.mark_for_review(gone).unwrap().unwrap();
    fs::remove_file(&gone_marked).unwrap();
    cancel(gone);
    for _ in 0..10 {
        applier.reconcile_review_renames().unwrap();
    }
    assert!(store.marked_but_settled().unwrap().is_empty());

    let (held, _) = unreadable(&store, &folder, "held.pdf", ErrorCode::PasswordProtected);
    let held_marked = applier.mark_for_review(held).unwrap().unwrap();
    cancel(held);
    filesystem.refuse.store(true, Ordering::SeqCst);
    for _ in 0..10 {
        applier.reconcile_review_renames().unwrap();
    }
    assert!(held_marked.exists());
    assert_eq!(
        stages(&temp),
        vec![
            ("mark".to_string(), "abandoned".to_string()),
            ("mark".to_string(), "complete".to_string()),
            ("unmark".to_string(), "failed".to_string()),
        ]
    );
    // Not offered again until the wait is over; then tried, and the
    // journal still holds only the latest refusal.
    assert!(store.marked_but_settled().unwrap().is_empty());
    let connection = Connection::open(database(&temp)).unwrap();
    connection
        .execute(
            "UPDATE review_renames SET updated_at = updated_at - 3600",
            [],
        )
        .unwrap();
    assert_eq!(store.marked_but_settled().unwrap().len(), 1);
    applier.reconcile_review_renames().unwrap();
    assert_eq!(stages(&temp).len(), 3);
}

/// Clearing the history never clears away a canceled document whose file
/// still carries Intern's mark: nothing would be left to give its name
/// back, and every scan would skip the file for good.
#[test]
fn clearing_the_history_keeps_a_document_whose_name_is_not_back_yet() {
    let temp = TempDir::new().unwrap();
    let store = Arc::new(QueueStore::open(database(&temp)).unwrap());
    let folder = inbox(&temp);
    let applier = FileApplier::local(Arc::clone(&store));
    let (marked, _) = unreadable(&store, &folder, "held.pdf", ErrorCode::PasswordProtected);
    applier.mark_for_review(marked).unwrap().unwrap();
    store
        .transition(
            marked,
            QueueStatus::NeedsReview,
            QueueStatus::Canceled,
            None,
        )
        .unwrap();
    let plain = store.enqueue(Path::new("/in/plain.pdf"), "plain").unwrap();
    store.record_intake_claim("plain", plain.id).unwrap();
    store
        .transition(plain.id, QueueStatus::Queued, QueueStatus::Canceled, None)
        .unwrap();

    assert_eq!(store.clear_terminal().unwrap(), 1);
    assert!(store.get(marked).unwrap().is_some());
    // The cleared item's claim reads as gone for one round, then goes.
    assert_eq!(store.find_by_intake_claim("plain").unwrap(), Some(None));
    applier.unmark_settled(marked).unwrap().unwrap();
    assert_eq!(store.clear_terminal().unwrap(), 1);
    assert_eq!(store.find_by_intake_claim("plain").unwrap(), None);
}

/// Retry after the person fixed the file - removed the password, saved it
/// again - reads the file as it is, under its new fingerprint.
#[test]
fn a_retry_takes_the_files_fingerprint_now() {
    let temp = TempDir::new().unwrap();
    let store = QueueStore::open(database(&temp)).unwrap();
    let folder = inbox(&temp);
    let (id, _) = unreadable(&store, &folder, "scan.pdf", ErrorCode::PasswordProtected);
    let retried = store.retry_unreadable(id, Some("after-the-fix")).unwrap();
    assert_eq!(retried.source_hash, "after-the-fix");
}

/// A claim follows the item it handed over, not the path: a second document
/// arriving under the same name never answers for the first, and a removed
/// item reads as gone.
#[test]
fn an_intake_claim_follows_its_own_item() {
    let temp = TempDir::new().unwrap();
    let store = QueueStore::open(database(&temp)).unwrap();
    assert_eq!(store.find_by_intake_claim("first").unwrap(), None);
    let first = store.enqueue(Path::new("/in/a.pdf"), "a").unwrap();
    store.record_intake_claim("first", first.id).unwrap();
    let second = store.enqueue(Path::new("/in/a.pdf"), "a-again").unwrap();
    store.record_intake_claim("second", second.id).unwrap();

    assert_eq!(
        store
            .find_by_intake_claim("first")
            .unwrap()
            .unwrap()
            .map(|item| item.id),
        Some(first.id)
    );
    let connection = Connection::open(database(&temp)).unwrap();
    connection
        .execute("DELETE FROM queue_items WHERE id = ?1", [first.id])
        .unwrap();
    assert_eq!(store.find_by_intake_claim("first").unwrap(), Some(None));
    assert_eq!(
        store
            .find_by_intake_claim("second")
            .unwrap()
            .unwrap()
            .map(|item| item.id),
        Some(second.id)
    );
}
