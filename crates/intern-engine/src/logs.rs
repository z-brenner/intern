//! Where the sidecars' own diagnostics go.
//!
//! llama-server and the parser worker say why they failed on standard error:
//! a CPU without the instructions the build needs, a runtime library an
//! antivirus product quarantined, a model file that will not load. That used
//! to go to nowhere, and a start failure reached support as a bare code. Once
//! the app names a directory, each sidecar's standard error goes to a file of
//! its own there instead.
//!
//! Only what the processes themselves print arrives here. llama-server at its
//! default verbosity logs neither prompts nor its `--api-key` (it is never
//! started with `-v`), and the worker's panic hook prints where a panic
//! happened, never its message, which can quote the document.

use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    process::Stdio,
    sync::OnceLock,
};

/// A log file at least this long is emptied when it is next opened, so a
/// machine that restarts its model every day keeps a few starts' worth.
const MAX_LOG_BYTES: u64 = 256 * 1024;

static LOG_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

/// Sends the sidecars' standard error to files in `dir` from the next launch
/// on. The first directory set is kept for the life of the process.
pub fn set_log_directory(dir: PathBuf) {
    let _ = LOG_DIRECTORY.set(dir);
}

/// The directory set with [`set_log_directory`], if any.
pub(crate) fn log_directory() -> Option<&'static Path> {
    LOG_DIRECTORY.get().map(PathBuf::as_path)
}

/// `name` under `directory`, opened to append, the directory created if need
/// be; a file over [`MAX_LOG_BYTES`] is emptied first. `None` when the file
/// cannot be opened.
pub(crate) fn log_file(directory: &Path, name: &str) -> Option<File> {
    std::fs::create_dir_all(directory).ok()?;
    let path = directory.join(name);
    // Emptied through a handle of its own, by path: on Windows a handle
    // opened to append has no right to shorten its file, and the log would
    // stop being written the first time it passed the cap.
    if std::fs::metadata(&path).is_ok_and(|metadata| metadata.len() > MAX_LOG_BYTES) {
        File::create(&path).ok()?;
    }
    OpenOptions::new().create(true).append(true).open(path).ok()
}

/// Standard error for a sidecar: its log file under `directory`, or nowhere
/// when there is no directory or the file cannot be opened. A log is never a
/// reason not to launch.
pub(crate) fn stderr_for(directory: Option<&Path>, name: &str) -> Stdio {
    directory
        .and_then(|directory| log_file(directory, name))
        .map_or_else(Stdio::null, Stdio::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_log_is_appended_to_until_it_passes_the_cap() {
        let directory = tempfile::tempdir().unwrap();
        let logs = directory.path().join("logs");
        {
            use std::io::Write as _;
            let mut first = log_file(&logs, "llama-server.log").expect("created with its folder");
            first.write_all(b"first start\n").unwrap();
            let mut second = log_file(&logs, "llama-server.log").unwrap();
            second.write_all(b"second start\n").unwrap();
        }
        assert_eq!(
            std::fs::read_to_string(logs.join("llama-server.log")).unwrap(),
            "first start\nsecond start\n"
        );

        // Sizes are read by path: a handle opened to append may not be
        // allowed to read its own file's attributes on Windows.
        let length = |name: &str| std::fs::metadata(logs.join(name)).unwrap().len();
        std::fs::write(logs.join("llama-server.log"), vec![b'x'; 256 * 1024 + 1]).unwrap();
        {
            use std::io::Write as _;
            let mut reopened = log_file(&logs, "llama-server.log").expect("emptied, not refused");
            assert_eq!(length("llama-server.log"), 0, "emptied past the cap");
            reopened.write_all(b"third start\n").unwrap();
        }
        assert_eq!(
            std::fs::read_to_string(logs.join("llama-server.log")).unwrap(),
            "third start\n"
        );

        std::fs::write(logs.join("worker.log"), vec![b'x'; 256 * 1024]).unwrap();
        let _kept = log_file(&logs, "worker.log").unwrap();
        assert_eq!(length("worker.log"), 256 * 1024, "kept at the cap");
    }

    #[test]
    fn an_unusable_directory_means_no_log_rather_than_no_launch() {
        let directory = tempfile::tempdir().unwrap();
        let occupied = directory.path().join("not-a-directory");
        std::fs::write(&occupied, b"").unwrap();
        assert!(log_file(&occupied, "worker.log").is_none());
        // Both still give the child somewhere to write.
        let _ = stderr_for(Some(&occupied), "worker.log");
        let _ = stderr_for(None, "worker.log");
    }
}
