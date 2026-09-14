use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const TEMP_NAME_ATTEMPTS: usize = 8;

/// Whether a durable replacement failed before publication or after a new
/// directory entry became visible. Callers must not treat the latter as though
/// the old file still exists.
#[derive(Debug)]
pub enum DurableReplaceError {
    NotPublished(io::Error),
    PublishedButNotDurable(io::Error),
}

/// Replaces `target` with fully durable `bytes`.
///
/// Publication starts with a same-directory, create-new temporary file. The
/// temporary file is flushed before its path is published. Windows uses
/// `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`; Unix-like platforms
/// flush the parent directory after rename so the new directory entry is
/// durable as well as its bytes.
pub fn replace_file_durable(target: &Path, bytes: &[u8]) -> Result<(), DurableReplaceError> {
    replace_file_with(
        target,
        bytes,
        || random_temp_sibling(target),
        write_temp,
        publish_temp,
    )
}

fn replace_file_with(
    target: &Path,
    bytes: &[u8],
    mut next_temporary: impl FnMut() -> PathBuf,
    mut write: impl FnMut(&Path, &[u8]) -> io::Result<()>,
    publish: impl Fn(&Path, &Path) -> Result<(), DurableReplaceError>,
) -> Result<(), DurableReplaceError> {
    for _ in 0..TEMP_NAME_ATTEMPTS {
        let temporary = next_temporary();
        match write(&temporary, bytes) {
            Ok(()) => {
                let result = publish(&temporary, target);
                if matches!(&result, Err(DurableReplaceError::NotPublished(_))) {
                    let _ = fs::remove_file(&temporary);
                }
                return result;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(DurableReplaceError::NotPublished(error));
            }
        }
    }
    Err(DurableReplaceError::NotPublished(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique temporary state file",
    )))
}

fn random_temp_sibling(target: &Path) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let nonce = rand::random::<u64>();
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    target.with_file_name(format!(
        ".{name}.{}.{}.{nanos}.{sequence}.intern-tmp",
        std::process::id(),
        nonce,
    ))
}

fn write_temp(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(windows)]
fn publish_temp(temporary: &Path, target: &Path) -> Result<(), DurableReplaceError> {
    windows::replace_write_through(temporary, target).map_err(DurableReplaceError::NotPublished)
}

#[cfg(not(windows))]
fn publish_temp(temporary: &Path, target: &Path) -> Result<(), DurableReplaceError> {
    fs::rename(temporary, target).map_err(DurableReplaceError::NotPublished)?;
    sync_parent(target).map_err(DurableReplaceError::PublishedButNotDurable)
}

#[cfg(not(windows))]
fn sync_parent(target: &Path) -> io::Result<()> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(windows)]
mod windows {
    #![allow(unsafe_code)]

    use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, path::Path};

    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    pub(super) fn replace_write_through(temporary: &Path, target: &Path) -> io::Result<()> {
        let temporary = wide(temporary.as_os_str());
        let target = wide(target.as_os_str());
        // SAFETY: both path buffers are NUL-terminated UTF-16 strings that
        // remain alive for the whole call. The files are siblings, so the
        // replace is same-volume. WRITE_THROUGH makes Windows flush the move
        // before returning rather than leaving its directory metadata queued.
        let success = unsafe {
            MoveFileExW(
                temporary.as_ptr(),
                target.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if success == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn wide(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(Some(0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, fs, io};

    use super::{
        DurableReplaceError, publish_temp, replace_file_durable, replace_file_with, write_temp,
    };

    #[test]
    fn durable_replace_overwrites_an_existing_file() {
        let directory = tempfile::TempDir::new().expect("create temporary directory");
        let target = directory.path().join("ui-state.json");
        fs::write(&target, b"old").expect("seed old state");

        replace_file_durable(&target, b"new").expect("replace existing state");

        assert_eq!(fs::read(&target).expect("read replacement"), b"new");
    }

    #[test]
    fn a_colliding_temp_name_is_retried_with_a_fresh_candidate() {
        let directory = tempfile::TempDir::new().expect("create temporary directory");
        let target = directory.path().join("ui-state.json");
        fs::write(&target, b"old").expect("seed old state");
        let collision = directory.path().join("collision.tmp");
        let replacement = directory.path().join("replacement.tmp");
        fs::write(&collision, b"stale").expect("seed stale temporary file");
        let candidates = RefCell::new(vec![collision, replacement]);

        replace_file_with(
            &target,
            b"new",
            || candidates.borrow_mut().remove(0),
            write_temp,
            publish_temp,
        )
        .expect("retry a colliding temporary name");

        assert_eq!(fs::read(&target).expect("read replacement"), b"new");
        assert_eq!(
            fs::read(directory.path().join("collision.tmp")).expect("read stale file"),
            b"stale"
        );
    }

    #[test]
    fn an_injected_publication_failure_preserves_the_old_file() {
        let directory = tempfile::TempDir::new().expect("create temporary directory");
        let target = directory.path().join("ui-state.json");
        fs::write(&target, b"old").expect("seed old state");
        let temporary = directory.path().join("temporary.tmp");

        let error = replace_file_with(
            &target,
            b"new",
            || temporary.clone(),
            write_temp,
            |_, _| {
                Err(DurableReplaceError::NotPublished(io::Error::other(
                    "injected publication failure",
                )))
            },
        )
        .expect_err("publication failure must be reported");

        assert!(
            matches!(error, DurableReplaceError::NotPublished(error) if error.kind() == io::ErrorKind::Other)
        );
        assert_eq!(fs::read(&target).expect("read old state"), b"old");
        assert!(
            !temporary.exists(),
            "the unpublished temporary file is cleaned up"
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_replaces_an_existing_file_without_delete_then_create() {
        let directory = tempfile::TempDir::new().expect("create temporary directory");
        let target = directory.path().join("ui-state.json");
        fs::write(&target, b"old").expect("seed old state");

        replace_file_durable(&target, b"new").expect("replace existing state on Windows");

        assert_eq!(fs::read(target).expect("read replacement"), b"new");
    }

    #[test]
    fn post_rename_parent_sync_failure_reports_that_the_new_file_was_published() {
        let directory = tempfile::TempDir::new().expect("create temporary directory");
        let target = directory.path().join("ui-state.json");
        let temporary = directory.path().join("temporary.tmp");

        let error = replace_file_with(
            &target,
            b"new",
            || temporary.clone(),
            write_temp,
            |from, to| {
                fs::rename(from, to).map_err(DurableReplaceError::NotPublished)?;
                Err(DurableReplaceError::PublishedButNotDurable(
                    io::Error::other("injected parent sync failure"),
                ))
            },
        )
        .expect_err("post-rename sync failure must be reported distinctly");

        assert!(matches!(
            error,
            DurableReplaceError::PublishedButNotDurable(_)
        ));
        assert_eq!(fs::read(&target).expect("read published state"), b"new");
        assert!(
            !temporary.exists(),
            "the renamed temporary path no longer exists"
        );
    }

    #[test]
    fn partial_temporary_file_is_removed_when_writing_fails() {
        let directory = tempfile::TempDir::new().expect("create temporary directory");
        let target = directory.path().join("ui-state.json");
        let temporary = directory.path().join("partial.tmp");
        fs::write(&target, b"old").expect("seed old state");

        let error = replace_file_with(
            &target,
            b"new",
            || temporary.clone(),
            |path, _| {
                fs::write(path, b"partial")?;
                Err(io::Error::other("injected write failure"))
            },
            publish_temp,
        )
        .expect_err("write failure must be reported");

        assert!(
            matches!(error, DurableReplaceError::NotPublished(error) if error.kind() == io::ErrorKind::Other)
        );
        assert_eq!(fs::read(&target).expect("read old state"), b"old");
        assert!(!temporary.exists(), "partial temporary file is removed");
    }
}
