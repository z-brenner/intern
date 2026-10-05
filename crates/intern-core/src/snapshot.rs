//! Private, process-owned file snapshots used to close validation-to-use gaps.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt as _;

#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

static SNAPSHOT_SEQUENCE: AtomicU64 = AtomicU64::new(1);
const SNAPSHOT_NAME_ATTEMPTS: usize = 8;
const CONTENT_NAME: &str = "content";

/// Every extension Intern admits, lowercase and without the dot.
///
/// This is the one list: the queue admits a dropped file by it, the intake
/// watcher claims a file by it, a snapshot is taken only for it, and the
/// worker's router has a test that every entry here reaches a reader and
/// every reader is reachable from here. Two copies of this list drifted apart
/// once - `.docm` was routed by the worker and refused by both admission
/// lists - which is why there is now only one.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "pdf", "docx", "docm", "doc", "rtf", "odt", "pptx", "pptm", "ppsx", "ppt", "odp", "xlsx",
    "xlsm", "xls", "ods", "csv", "eml", "msg", "txt", "md", "markdown", "png", "jpg", "jpeg",
    "tif", "tiff",
];

/// Whether a file name is one Intern's own rename-history export goes by:
/// `intern-history.csv`, the name its save dialog suggests, and the
/// `intern-history (2).csv` or `intern-history-march.csv` a person or the
/// dialog makes of it.
///
/// The export is Intern's own record of what it renamed - every filed
/// document's old and new path and its description - and since `.csv` is
/// admitted, one saved into a watched folder looked like any new upload:
/// it was claimed, read by the model as a client's ledger, and renamed and
/// filed like one. Nothing Intern admits by this name is a document to file.
pub fn is_history_export(file_name: &str) -> bool {
    let name = file_name.to_ascii_lowercase();
    name.starts_with("intern-history") && name.ends_with(".csv")
}

/// A directory controlled by Intern and kept outside watched or synced roots.
///
/// Each snapshot receives a create-new child directory. The child name is not
/// accepted from a caller, and cleanup removes only the fixed content file and
/// that one child. Unexpected entries make cleanup stop rather than broadening
/// into recursive deletion.
#[derive(Clone, Debug)]
pub struct PrivateSnapshotDirectory {
    root: Arc<SnapshotRoot>,
}

#[derive(Debug)]
struct SnapshotRoot {
    path: PathBuf,
}

impl PrivateSnapshotDirectory {
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref();
        if !root.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot directory must be absolute",
            ));
        }
        fs::create_dir_all(root)?;
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot path must be a real directory",
            ));
        }
        make_directory_private(root)?;
        Ok(Self {
            root: Arc::new(SnapshotRoot {
                path: root.to_path_buf(),
            }),
        })
    }

    pub fn create_for_supported_extension(&self, extension: &str) -> io::Result<SnapshotWriter> {
        let extension = extension.to_ascii_lowercase();
        if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot source extension is unsupported",
            ));
        }
        self.create_named(&format!("{CONTENT_NAME}.{extension}"))
    }

    fn create_named(&self, content_name: &str) -> io::Result<SnapshotWriter> {
        for _ in 0..SNAPSHOT_NAME_ATTEMPTS {
            let sequence = SNAPSHOT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let nonce = rand::random::<u64>();
            let directory = self.root.path.join(format!(
                "snapshot-{}-{nonce:016x}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&directory) {
                Ok(()) => {
                    if let Err(error) = make_directory_private(&directory) {
                        let _ = fs::remove_dir(&directory);
                        return Err(error);
                    }
                    let path = directory.join(content_name);
                    match open_private_file(&path) {
                        Ok(file) => {
                            return Ok(SnapshotWriter {
                                root: Arc::clone(&self.root),
                                directory: Some(directory),
                                path: Some(path),
                                file: Some(file),
                            });
                        }
                        Err(error) => {
                            let _ = fs::remove_dir(&directory);
                            return Err(error);
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a private snapshot directory",
        ))
    }
}

impl Drop for SnapshotRoot {
    fn drop(&mut self) {
        // Only succeeds when no snapshot or unexpected file is present.
        let _ = fs::remove_dir(&self.path);
    }
}

/// A create-new snapshot being populated. Dropping it cleans partial bytes.
#[derive(Debug)]
pub struct SnapshotWriter {
    root: Arc<SnapshotRoot>,
    directory: Option<PathBuf>,
    path: Option<PathBuf>,
    file: Option<File>,
}

impl SnapshotWriter {
    pub fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("snapshot writer path is present")
    }

    pub fn finish(mut self) -> io::Result<OwnedFileSnapshot> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("snapshot writer is unavailable"))?;
        file.flush()?;
        file.sync_all()?;
        make_file_read_only(file)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(OwnedFileSnapshot {
            root: Arc::clone(&self.root),
            directory: self.directory.take(),
            path: self.path.take(),
            file: self.file.take(),
        })
    }
}

impl Write for SnapshotWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("snapshot writer is unavailable"))?
            .write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("snapshot writer is unavailable"))?
            .flush()
    }
}

impl Drop for SnapshotWriter {
    fn drop(&mut self) {
        cleanup(
            &self.root,
            &mut self.directory,
            &mut self.path,
            &mut self.file,
        );
    }
}

/// An immutable-by-contract snapshot whose lifetime owns its cleanup.
///
/// The file handle remains open while a worker reads the path. On Windows it
/// was opened sharing reads only, so another process cannot replace, delete,
/// or open the snapshot for writing before the evidence is dropped.
#[derive(Debug)]
pub struct OwnedFileSnapshot {
    root: Arc<SnapshotRoot>,
    directory: Option<PathBuf>,
    path: Option<PathBuf>,
    file: Option<File>,
}

impl OwnedFileSnapshot {
    pub fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("owned snapshot path is present")
    }
}

impl Drop for OwnedFileSnapshot {
    fn drop(&mut self) {
        cleanup(
            &self.root,
            &mut self.directory,
            &mut self.path,
            &mut self.file,
        );
    }
}

fn cleanup(
    _root: &Arc<SnapshotRoot>,
    directory: &mut Option<PathBuf>,
    path: &mut Option<PathBuf>,
    file: &mut Option<File>,
) {
    drop(file.take());
    if let Some(path) = path.take() {
        make_file_writable_for_cleanup(&path);
        let _ = fs::remove_file(path);
    }
    if let Some(directory) = directory.take() {
        let _ = fs::remove_dir(directory);
    }
}

#[cfg(unix)]
fn make_file_read_only(file: &File) -> io::Result<()> {
    file.set_permissions(fs::Permissions::from_mode(0o400))
}

#[cfg(windows)]
fn make_file_read_only(file: &File) -> io::Result<()> {
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions)
}

#[cfg(not(any(unix, windows)))]
fn make_file_read_only(file: &File) -> io::Result<()> {
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(true);
    file.set_permissions(permissions)
}

// Windows-only: clearing the read-only attribute cannot make the file world
// writable there, which is the Unix hazard this lint guards against.
#[cfg(windows)]
#[allow(clippy::permissions_set_readonly_false)]
fn make_file_writable_for_cleanup(path: &Path) {
    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(not(windows))]
fn make_file_writable_for_cleanup(_path: &Path) {}

#[cfg(unix)]
fn make_directory_private(path: &Path) -> io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn make_directory_private(_path: &Path) -> io::Result<()> {
    // Windows inherits the per-user ACL from the app-local data directory.
    Ok(())
}

#[cfg(unix)]
fn open_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(windows)]
fn open_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
}

#[cfg(not(any(unix, windows)))]
fn open_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
}
