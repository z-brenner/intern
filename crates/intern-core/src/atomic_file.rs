use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const TEMP_NAME_ATTEMPTS: usize = 8;

/// Replaces `target` with fully durable `bytes`.
///
/// Publication starts with a same-directory, create-new temporary file. The
/// temporary file is flushed before its path is published. Windows uses
/// `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`; Unix-like platforms
/// flush the parent directory after rename so the new directory entry is
/// durable as well as its bytes.
pub fn replace_file_durable(target: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_file_with(target, bytes, || random_temp_sibling(target), publish_temp)
}

fn replace_file_with(
    target: &Path,
    bytes: &[u8],
    mut next_temporary: impl FnMut() -> PathBuf,
    publish: impl Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    for _ in 0..TEMP_NAME_ATTEMPTS {
        let temporary = next_temporary();
        match write_temp(&temporary, bytes) {
            Ok(()) => {
                let result = publish(&temporary, target);
                if result.is_err() {
                    let _ = fs::remove_file(&temporary);
                }
                return result;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique temporary state file",
    ))
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
fn publish_temp(temporary: &Path, target: &Path) -> io::Result<()> {
    windows::replace_write_through(temporary, target)
}

#[cfg(not(windows))]
fn publish_temp(temporary: &Path, target: &Path) -> io::Result<()> {
    fs::rename(temporary, target)?;
    sync_parent(target)
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

    use super::{publish_temp, replace_file_durable, replace_file_with};

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
            |_, _| Err(io::Error::other("injected publication failure")),
        )
        .expect_err("publication failure must be reported");

        assert_eq!(error.kind(), io::ErrorKind::Other);
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
}
