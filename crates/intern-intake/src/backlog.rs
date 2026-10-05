//! What was already in the intake folder when this machine first watched it.
//!
//! A file with no origin marker has no known uploader, and "mine" scope
//! leaves the ones that predate watching alone rather than guess. Kept only
//! in memory, that snapshot was retaken by every new watcher - each app
//! start, update restart and intake settings save - so a document that
//! arrived while Intern was off counted as already there and was held for
//! ever. The snapshot is therefore taken once per watch and kept in the
//! app's own data, beside the queue, never in the shared `.intern` folder:
//! what this machine found on its first look is nobody else's business. A
//! new watch - another folder, or the host saying the person has just started
//! watching this one - takes it again.
//!
//! It is keyed by document key - relative path, size and modification time -
//! not by path, so a file replaced with new content, or a new scan that
//! reuses an old name, is new.
//!
//! A first look can miss things that were there all along: a subfolder that
//! refused to be listed, a file whose attributes could not be read, a type of
//! document this version does not read but a later one will. Each of those
//! would be admitted as new the moment it could be seen. So the snapshot also
//! remembers where it could not look and which types it covered, and a
//! document first seen in such a place, or of a type added since, joins it.

use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{fsatomic::replace_file, scan::IntakeConfig};

/// How long, on the injected clock, passes that saw the whole folder must
/// miss a document before the snapshot forgets it.
pub const BACKLOG_FORGET_SECONDS: i64 = 24 * 60 * 60;

/// The file's shape.
#[derive(Deserialize, Serialize)]
struct Record {
    /// The canonical intake root the keys belong to.
    folder: PathBuf,
    keys: Vec<String>,
    /// The extensions the snapshot covers. A record without them covers
    /// none, so every document it could have missed joins it.
    #[serde(default)]
    extensions: Vec<String>,
    /// Places under the folder - relative, lowercase, `/`-separated - that no
    /// look has managed to see into since the snapshot was taken. `""` is the
    /// whole folder.
    #[serde(default)]
    unseen: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Backlog {
    /// Where the snapshot is kept; `None` keeps it in memory only.
    file: Option<PathBuf>,
    /// The intake root the snapshot is about, canonicalized when possible.
    folder: PathBuf,
    keys: HashSet<String>,
    recorded: bool,
    /// The extensions the snapshot covers, sorted.
    extensions: Vec<String>,
    /// Watched extensions the snapshot does not cover yet. Files of these
    /// types join it until the next pass over the whole folder ends.
    uncovered: Vec<String>,
    /// See `Record::unseen`.
    unseen: Vec<String>,
    /// Keys whole passes have stopped seeing, with the clock time the first
    /// of them missed each one. In memory only: a restart starts the wait
    /// again, which only keeps a key longer.
    missing: HashMap<String, i64>,
    /// The snapshot changed since the file was last written successfully.
    unsaved: bool,
}

impl Backlog {
    /// The snapshot kept for the watch `config` describes, or an empty one
    /// still to be taken when there is none, it cannot be read, it was taken
    /// of another folder, or the host asked for a fresh one. A restart or a
    /// changed machine label watches the same documents as before.
    pub(crate) fn load(config: &IntakeConfig) -> Self {
        let file = config.backlog_file.as_deref();
        let root = &config.intake_root;
        let folder = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        let mut watched: Vec<String> = config
            .extensions
            .iter()
            .map(|extension| extension.to_ascii_lowercase())
            .collect();
        watched.sort_unstable();
        watched.dedup();
        if config.retake_backlog {
            // The stale snapshot goes now, not when the new one is written:
            // a restart before the first pass ends must not bring it back.
            // Should the removal fail, that first pass overwrites it anyway.
            if let Some(file) = file {
                let _ = fs::remove_file(file);
            }
        }
        let record = file
            .filter(|_| !config.retake_backlog)
            .and_then(|file| fs::read(file).ok())
            .and_then(|bytes| serde_json::from_slice::<Record>(&bytes).ok())
            .filter(|record| record.folder == folder);
        let Some(record) = record else {
            return Self {
                file: file.map(Path::to_path_buf),
                folder,
                extensions: watched,
                ..Self::default()
            };
        };
        let uncovered = watched
            .iter()
            .filter(|extension| !record.extensions.contains(extension))
            .cloned()
            .collect();
        Self {
            file: file.map(Path::to_path_buf),
            folder,
            keys: record.keys.into_iter().collect(),
            recorded: true,
            extensions: record.extensions,
            uncovered,
            unseen: record.unseen,
            missing: HashMap::new(),
            unsaved: false,
        }
    }

    #[cfg(test)]
    fn is_recorded(&self) -> bool {
        self.recorded
    }

    pub(crate) fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    /// Counts a document as already here if it is: everything is while the
    /// snapshot is being taken, and afterwards a document the snapshot could
    /// not have seen - in a place it could not look into, or of a type it did
    /// not cover - is too.
    pub(crate) fn note(&mut self, key: &str, relative_path: &str) {
        if self.recorded && !self.missed(relative_path) {
            return;
        }
        if self.keys.insert(key.to_owned()) {
            self.unsaved = true;
        }
    }

    fn missed(&self, relative_path: &str) -> bool {
        let path = normalized(relative_path);
        self.unseen.iter().any(|place| within(place, &path))
            || extension_of(&path).is_some_and(|extension| {
                self.uncovered
                    .iter()
                    .any(|uncovered| uncovered == extension)
            })
    }

    /// Ends a pass over the folder at clock time `now`. `seen` holds the key
    /// of every file the pass walked past; `hidden` the places it could not
    /// look into, as the walk reports them.
    ///
    /// A pass cut short by shutdown says nothing about the files after the
    /// cut, so it changes nothing. A whole pass ends the first look - with
    /// what it could not see remembered rather than waited on, since a
    /// folder this machine may never list must not keep every later document
    /// counted as already here - and lets the snapshot forget documents that
    /// are gone. Only a pass that saw everything can say what is gone: a
    /// place it could not look into hid its documents rather than removed
    /// them.
    pub(crate) fn end_pass(
        &mut self,
        seen: &HashSet<String>,
        hidden: &[String],
        interrupted: bool,
        now: i64,
    ) {
        if interrupted {
            return;
        }
        self.missing.retain(|key, _| !seen.contains(key));
        let mut hidden: Vec<String> = hidden.iter().map(|place| normalized(place)).collect();
        hidden.sort_unstable();
        hidden.dedup();
        if !self.recorded {
            self.recorded = true;
            self.unseen = hidden;
            self.unsaved = true;
            return;
        }
        if !self.uncovered.is_empty() {
            self.extensions.append(&mut self.uncovered);
            self.extensions.sort_unstable();
            self.extensions.dedup();
            self.unsaved = true;
        }
        let unseen = still_unseen(&self.unseen, &hidden);
        if unseen != self.unseen {
            self.unseen = unseen;
            self.unsaved = true;
        }
        if hidden.is_empty() {
            self.forget_gone(seen, now);
        }
    }

    /// Forgets documents that are no longer in the folder as they were: a
    /// file that was deleted, moved away or rewritten has a key nothing will
    /// see again, and keeping it would only grow the file.
    ///
    /// Not the moment one pass misses it, though. Even a walk that could look
    /// everywhere can miss a file for a moment - a sync client swapping it
    /// out and back, a listing taken mid-rename - and a document forgotten
    /// then is admitted as new when it is seen again, renamed and filed
    /// against the person's choice to leave it alone. So a key goes only once
    /// whole passes have missed it for `BACKLOG_FORGET_SECONDS`. A clock that
    /// moved backwards starts the wait again.
    fn forget_gone(&mut self, seen: &HashSet<String>, now: i64) {
        let missing = &mut self.missing;
        let before = self.keys.len();
        self.keys.retain(|key| {
            if seen.contains(key) {
                return true;
            }
            let since = missing.entry(key.clone()).or_insert(now);
            if now < *since {
                *since = now;
            }
            now - *since < BACKLOG_FORGET_SECONDS
        });
        let keys = &self.keys;
        self.missing.retain(|key, _| keys.contains(key));
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
            extensions: self.extensions.clone(),
            unseen: self.unseen.clone(),
        })
        .map_err(io::Error::other)?;
        replace_file(file, &bytes)?;
        self.unsaved = false;
        Ok(())
    }
}

/// A relative path the way document keys spell it.
fn normalized(relative_path: &str) -> String {
    relative_path
        .to_lowercase()
        .replace('\\', "/")
        .trim_matches('/')
        .to_owned()
}

/// Whether `path` is `place` or lies under it. Both are normalized; `""` is
/// the whole folder.
fn within(place: &str, path: &str) -> bool {
    place.is_empty()
        || path == place
        || path
            .strip_prefix(place)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn extension_of(path: &str) -> Option<&str> {
    let name = path.rsplit('/').next()?;
    name.rsplit_once('.').map(|(_, extension)| extension)
}

/// The places no look has seen into: those the snapshot missed that this pass
/// missed too, as narrowly as either one puts it. A place this pass did see
/// into is settled - its documents were noted on the way past.
fn still_unseen(unseen: &[String], hidden: &[String]) -> Vec<String> {
    let mut still: Vec<String> = unseen
        .iter()
        .filter(|place| hidden.iter().any(|missed| within(missed, place)))
        .chain(
            hidden
                .iter()
                .filter(|missed| unseen.iter().any(|place| within(place, missed))),
        )
        .cloned()
        .collect();
    still.sort_unstable();
    still.dedup();
    still
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs, path::Path};

    use super::{BACKLOG_FORGET_SECONDS, Backlog, still_unseen, within};
    use crate::scan::IntakeConfig;

    /// Any clock time; the tests only care about time passing from here.
    const NOW: i64 = 1_800_000_000;

    fn keys(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn places(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn config(folder: &Path, file: &Path, extensions: &[&str]) -> IntakeConfig {
        let mut config = IntakeConfig::new(
            folder,
            extensions.iter().map(|value| (*value).to_owned()).collect(),
        );
        config.backlog_file = Some(file.to_path_buf());
        config
    }

    /// A snapshot taken and saved, then loaded again as a restart would.
    fn recorded(folder: &Path, file: &Path, noted: &[(&str, &str)], hidden: &[&str]) -> Backlog {
        let mut backlog = Backlog::load(&config(folder, file, &["pdf"]));
        for (key, path) in noted {
            backlog.note(key, path);
        }
        backlog.end_pass(&HashSet::new(), &places(hidden), false, NOW);
        backlog.save().unwrap();
        Backlog::load(&config(folder, file, &["pdf"]))
    }

    #[test]
    fn a_snapshot_is_kept_for_its_own_folder_only() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let other = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");

        let reloaded = recorded(folder.path(), &file, &[("old", "old.pdf")], &[]);
        assert!(reloaded.is_recorded());
        assert!(reloaded.contains("old"));
        assert!(!Backlog::load(&config(other.path(), &file, &["pdf"])).is_recorded());
    }

    /// The host asks for a fresh look when the person has just started
    /// watching: what is there now is what they chose to leave alone.
    #[test]
    fn a_retake_ignores_and_removes_the_kept_snapshot() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        recorded(folder.path(), &file, &[("old", "old.pdf")], &[]);

        let mut retake = config(folder.path(), &file, &["pdf"]);
        retake.retake_backlog = true;
        let fresh = Backlog::load(&retake);
        assert!(!fresh.is_recorded());
        assert!(!fresh.contains("old"));
        assert!(
            !file.exists(),
            "a restart before the new snapshot is written must not bring the old one back"
        );
    }

    #[test]
    fn a_snapshot_that_cannot_be_read_is_taken_again() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        fs::write(&file, b"{ not json").unwrap();
        assert!(!Backlog::load(&config(folder.path(), &file, &["pdf"])).is_recorded());
    }

    /// The file is rewritten only when the snapshot changed: a scan that
    /// finds every old document where it was writes nothing, and neither does
    /// one that has only just stopped seeing a document.
    #[test]
    fn the_file_is_rewritten_only_when_the_snapshot_shrinks() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        let mut backlog = recorded(
            folder.path(),
            &file,
            &[("kept", "kept.pdf"), ("gone", "gone.pdf")],
            &[],
        );

        fs::remove_file(&file).unwrap();
        backlog.note("new", "new.pdf");
        backlog.end_pass(&keys(&["kept", "gone", "new"]), &[], false, NOW);
        backlog.save().unwrap();
        assert!(!file.exists(), "nothing changed, so nothing was written");
        assert!(!backlog.contains("new"), "a later file is not backlog");

        backlog.end_pass(&keys(&["kept"]), &[], false, NOW);
        backlog.save().unwrap();
        assert!(!file.exists(), "missed once is not gone");

        backlog.end_pass(&keys(&["kept"]), &[], false, NOW + BACKLOG_FORGET_SECONDS);
        backlog.save().unwrap();
        let reloaded = Backlog::load(&config(folder.path(), &file, &["pdf"]));
        assert!(reloaded.contains("kept"));
        assert!(!reloaded.contains("gone"));
    }

    /// A walk that could look everywhere can still miss a file for a moment:
    /// a sync client swapping it out and back, a listing taken mid-rename.
    /// Forgotten on that one pass, the document was admitted as new when it
    /// was seen again, and renamed and filed although the person had chosen
    /// to leave it alone.
    #[test]
    fn a_document_missed_for_a_moment_stays_in_the_snapshot() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        let mut backlog = recorded(folder.path(), &file, &[("old", "old.pdf")], &[]);

        backlog.end_pass(&keys(&[]), &[], false, NOW);
        backlog.end_pass(&keys(&[]), &[], false, NOW + BACKLOG_FORGET_SECONDS - 1);
        assert!(backlog.contains("old"), "missed for less than the grace");

        // Seen again: the wait starts over the next time it is missed.
        backlog.end_pass(&keys(&["old"]), &[], false, NOW + BACKLOG_FORGET_SECONDS);
        backlog.end_pass(&keys(&[]), &[], false, NOW + 2 * BACKLOG_FORGET_SECONDS - 1);
        assert!(backlog.contains("old"));

        // A pass that could not look everywhere cannot end it, however late.
        let late = NOW + 3 * BACKLOG_FORGET_SECONDS;
        backlog.end_pass(&keys(&[]), &places(&["locked"]), false, late);
        assert!(backlog.contains("old"));

        backlog.end_pass(&keys(&[]), &[], false, late + 1);
        assert!(
            !backlog.contains("old"),
            "missed by whole passes for the grace"
        );
    }

    /// Shutdown can cut a pass short. The files after the cut were never
    /// walked, so the pass can neither end the first look nor say what is
    /// gone.
    #[test]
    fn an_interrupted_pass_changes_nothing() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        let mut first = Backlog::load(&config(folder.path(), &file, &["pdf"]));
        first.note("a", "a.pdf");
        first.end_pass(&keys(&["a"]), &[], true, NOW);
        assert!(!first.is_recorded(), "the first look goes on");
        first.note("b", "b.pdf");
        first.end_pass(&keys(&["a", "b"]), &[], false, NOW);
        assert!(first.contains("a") && first.contains("b"));

        first.end_pass(&keys(&[]), &[], true, NOW + 2 * BACKLOG_FORGET_SECONDS);
        assert!(
            first.contains("a"),
            "nothing is forgotten on a cut-short pass"
        );
    }

    /// A subfolder the first look could not list held documents that were
    /// there all along. Once it can be read they join the snapshot, rather
    /// than being taken for new uploads and filed; documents that arrive in
    /// it after that are new.
    #[test]
    fn documents_in_a_place_the_first_look_missed_join_the_snapshot_when_seen() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        let mut backlog = recorded(
            folder.path(),
            &file,
            &[("top", "top.pdf")],
            &["Locked", "held.pdf"],
        );

        // Still unreadable: nothing in them was seen, and nothing is
        // forgotten.
        backlog.note("later", "later.pdf");
        backlog.end_pass(
            &keys(&["later"]),
            &places(&["Locked", "held.pdf"]),
            false,
            NOW + 2 * BACKLOG_FORGET_SECONDS,
        );
        assert!(!backlog.contains("later"));
        assert!(
            backlog.contains("top"),
            "a pass that missed somewhere cannot say what is gone"
        );

        // Readable at last: what is in it, and the file that could not be
        // read before, were there all along.
        backlog.note("old-inside", "locked/2019/old.pdf");
        backlog.note("held", "held.pdf");
        backlog.end_pass(
            &keys(&["top", "later", "old-inside", "held"]),
            &[],
            false,
            NOW + 2 * BACKLOG_FORGET_SECONDS,
        );
        backlog.save().unwrap();
        let mut reloaded = Backlog::load(&config(folder.path(), &file, &["pdf"]));
        assert!(reloaded.contains("old-inside"));
        assert!(reloaded.contains("held"));

        // Seen once, it is settled: a document that lands there now is new.
        reloaded.note("new-inside", "locked/new.pdf");
        assert!(!reloaded.contains("new-inside"));
    }

    /// The first look completes even when a folder never becomes readable:
    /// waiting for it would count every later document as already here.
    #[test]
    fn a_place_that_stays_unreadable_only_holds_back_what_is_in_it() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        let mut backlog = recorded(folder.path(), &file, &[], &["private"]);
        assert!(backlog.is_recorded());
        backlog.note("fresh", "fresh.pdf");
        backlog.note("privately", "private/fresh.pdf");
        assert!(!backlog.contains("fresh"));
        assert!(backlog.contains("privately"));
    }

    /// A version that learns to read a new type of document finds files of
    /// that type that were there all along. They are not new uploads.
    #[test]
    fn documents_of_a_type_added_since_join_the_snapshot_on_the_next_pass() {
        let data = tempfile::TempDir::new().unwrap();
        let folder = tempfile::TempDir::new().unwrap();
        let file = data.path().join("intake-backlog.json");
        recorded(folder.path(), &file, &[("pdf", "old.pdf")], &[]);

        let mut wider = Backlog::load(&config(folder.path(), &file, &["pdf", "txt"]));
        wider.note("old-txt", "notes/OLD.TXT");
        wider.note("new-pdf", "new.pdf");
        assert!(wider.contains("old-txt"));
        assert!(!wider.contains("new-pdf"));
        wider.end_pass(&keys(&["pdf", "old-txt", "new-pdf"]), &[], false, NOW);
        wider.save().unwrap();

        let mut later = Backlog::load(&config(folder.path(), &file, &["pdf", "txt"]));
        assert!(later.contains("old-txt"));
        later.note("new-txt", "new.txt");
        assert!(
            !later.contains("new-txt"),
            "covered now, so a new one is new"
        );
    }

    #[test]
    fn a_place_is_settled_as_far_as_a_pass_saw_into_it() {
        assert!(within("", "anything.pdf"));
        assert!(within("a", "a"));
        assert!(within("a", "a/b.pdf"));
        assert!(!within("a", "ab.pdf"));
        assert!(!within("a/b", "a"));

        assert_eq!(still_unseen(&places(&["a"]), &[]), Vec::<String>::new());
        assert_eq!(
            still_unseen(&places(&["a"]), &places(&["a"])),
            places(&["a"])
        );
        assert_eq!(
            still_unseen(&places(&["a"]), &places(&["a/b"])),
            places(&["a/b"])
        );
        assert_eq!(
            still_unseen(&places(&["a/b"]), &places(&["a"])),
            places(&["a/b"])
        );
        assert_eq!(
            still_unseen(&places(&["a", "c"]), &places(&["b"])),
            Vec::<String>::new()
        );
        assert_eq!(
            still_unseen(&places(&[""]), &places(&["x"])),
            places(&["x"])
        );
    }
}
