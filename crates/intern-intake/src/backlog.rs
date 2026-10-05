//! What was already in the intake folder when this machine first watched it.
//!
//! A file with no origin marker has no known uploader, and "mine" scope
//! leaves the ones that predate watching alone rather than guess. Kept only
//! in memory, that snapshot was retaken by every new watcher - each app
//! start, update restart and intake settings save - so a document that
//! arrived while Intern was off counted as already there and was held for
//! ever. The snapshot is therefore taken once per folder and kept in the
//! app's own data, beside the queue, never in the shared `.intern` folder:
//! what this machine found on its first look is nobody else's business.
//!
//! It is keyed by document key - relative path, size and modification time -
//! not by path, so a file replaced with new content, or a new scan that
//! reuses an old name, is new.

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::fsatomic::replace_file;

/// The file's shape: the canonical intake root the keys belong to, and the
/// keys.
#[derive(Deserialize, Serialize)]
struct Record {
    folder: PathBuf,
    keys: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Backlog {
    /// Where the snapshot is kept; `None` keeps it in memory only.
    file: Option<PathBuf>,
    /// The intake root the snapshot is about, canonicalized when possible.
    folder: PathBuf,
    keys: HashSet<String>,
    recorded: bool,
    /// The keys changed since the file was last written successfully.
    unsaved: bool,
}

impl Backlog {
    /// The snapshot kept for `root`, or an empty one still to be taken when
    /// there is none, it cannot be read, or it was taken of another folder.
    /// A changed folder is the one thing that retakes it: a restart or a
    /// changed machine label watches the same documents as before.
    pub(crate) fn load(file: Option<&Path>, root: &Path) -> Self {
        let folder = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        let record = file
            .and_then(|file| fs::read(file).ok())
            .and_then(|bytes| serde_json::from_slice::<Record>(&bytes).ok())
            .filter(|record| record.folder == folder);
        Self {
            file: file.map(Path::to_path_buf),
            folder,
            recorded: record.is_some(),
            keys: record
                .map(|record| record.keys.into_iter().collect())
                .unwrap_or_default(),
            unsaved: false,
        }
    }

    pub(crate) fn is_recorded(&self) -> bool {
        self.recorded
    }

    pub(crate) fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    /// Counts a document as already here while the snapshot is being taken.
    pub(crate) fn note(&mut self, key: &str) {
        if !self.recorded && self.keys.insert(key.to_owned()) {
            self.unsaved = true;
        }
    }

    /// Ends the snapshot after a whole pass over the folder.
    pub(crate) fn finish_recording(&mut self) {
        if !self.recorded {
            self.recorded = true;
            self.unsaved = true;
        }
    }

    /// Forgets documents that are no longer in the folder as they were: a
    /// file that was deleted, moved away or rewritten has a key nothing will
    /// ever see again, and keeping it would only grow the file.
    pub(crate) fn retain_seen(&mut self, seen: &HashSet<String>) {
        let before = self.keys.len();
        self.keys.retain(|key| seen.contains(key));
        if self.keys.len() != before {
            self.unsaved = true;
        }
    }

    /// Writes the snapshot when it changed, through a temp file and a rename
    /// so a crash never leaves half a snapshot - which would read as "nothing
    /// was here" and admit every document it left out. A failed write is
    /// tried again on the next call.
    pub(crate) fn save(&mut self) -> io::Result<()> {
        if !self.recorded || !self.unsaved {
            return Ok(());
        }
        let Some(file) = self.file.as_deref() else {
            self.unsaved = false;
            return Ok(());
        };
        let mut keys: Vec<String> = self.keys.iter().cloned().collect();
        keys.sort_unstable();
        let bytes = serde_json::to_vec(&Record {
            folder: self.folder.clone(),
            keys,
        })
        .map_err(io::Error::other)?;
        replace_file(file, &bytes)?;
        self.unsaved = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs};

    use super::Backlog;

    fn keys(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn a_snapshot_is_kept_for_its_own_folder_only() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let other = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");

        let mut backlog = Backlog::load(Some(&file), folder.path());
        assert!(!backlog.is_recorded());
        backlog.note("old");
        backlog.finish_recording();
        backlog.save().unwrap();

        let reloaded = Backlog::load(Some(&file), folder.path());
        assert!(reloaded.is_recorded());
        assert!(reloaded.contains("old"));
        assert!(!Backlog::load(Some(&file), other.path()).is_recorded());
    }

    #[test]
    fn a_snapshot_that_cannot_be_read_is_taken_again() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        fs::write(&file, b"{ not json").unwrap();
        assert!(!Backlog::load(Some(&file), folder.path()).is_recorded());
    }

    /// The file is rewritten only when the snapshot changed: a scan that
    /// finds every old document where it was writes nothing.
    #[test]
    fn the_file_is_rewritten_only_when_the_snapshot_shrinks() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        let mut backlog = Backlog::load(Some(&file), folder.path());
        backlog.note("kept");
        backlog.note("gone");
        backlog.finish_recording();
        backlog.save().unwrap();

        fs::remove_file(&file).unwrap();
        backlog.retain_seen(&keys(&["kept", "gone", "new"]));
        backlog.save().unwrap();
        assert!(!file.exists(), "nothing changed, so nothing was written");
        assert!(!backlog.contains("new"), "a later file is not backlog");

        backlog.retain_seen(&keys(&["kept"]));
        backlog.save().unwrap();
        let reloaded = Backlog::load(Some(&file), folder.path());
        assert!(reloaded.contains("kept"));
        assert!(!reloaded.contains("gone"));
    }
}
