//! Adding a document is copying it into the input folder.
//!
//! Intern has one way in: a document is processed because it is in the input
//! folder (the watched intake folder), and it is filed into the output folder
//! (the destination). Add files, Add folder, a drop on the window, and "Send
//! to > Intern" used to process a file wherever it sat; now each copies what
//! it is given into the input folder and hands the copy to the queue, so the
//! queue only ever works on documents in one place and the original is never
//! touched - not renamed, not moved, not deleted.
//!
//! Folders are flattened into the input folder, with the same rules an add
//! always had for what counts as a document. A copy is written under a
//! dot-prefixed temporary name the scanner never looks at, checked against the
//! original byte for byte, and only then published onto its real name with a
//! rename that refuses to replace anything - so a crash leaves at most a hidden
//! temporary file, never a half-written document for the scanner to find. A
//! name already taken, on disk or by a document the queue has seen at that
//! path before, gets " (2)", " (3)", and so on. A file that is already in the
//! input folder is not copied again; it is handed to the queue where it is.

use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
};

use intern_core::{FileSystem, LockRetry, is_transient_lock, temporary_path};
use intern_queue::paths::{collect_supported_files, display_path};

use crate::commands::{AddReportDto, CommandError, SkippedDocumentDto, document_name};

/// A file inside the output folder: filed already, and copying it into the
/// input folder would file it a second time.
pub(crate) const IN_OUTPUT_FOLDER: &str = "IN_OUTPUT_FOLDER";
/// The copy into the input folder could not be written or published.
pub(crate) const COPY_FAILED: &str = "COPY_FAILED";
/// Another computer watching the same shared folder is already working on it.
pub(crate) const CLAIMED_ELSEWHERE: &str = "CLAIMED_ELSEWHERE";
/// No input folder is set up, so there is nowhere to copy to.
pub(crate) const INPUT_FOLDER_NOT_SET: &str = "INPUT_FOLDER_NOT_SET";
/// The input folder is set up but cannot be reached.
pub(crate) const INPUT_FOLDER_UNAVAILABLE: &str = "INPUT_FOLDER_UNAVAILABLE";

/// How many names a clash may try before the copy gives up. A folder with ten
/// thousand copies of one name is not one a person is adding to by hand.
const NAME_ATTEMPTS: u32 = 10_000;

/// The two folders an add works between, both canonical.
#[derive(Clone, Debug)]
pub(crate) struct Folders {
    pub(crate) input: PathBuf,
    pub(crate) output: Option<PathBuf>,
}

/// What became of one document once it was in the input folder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Arrival {
    /// A new item in the queue.
    Added,
    /// The queue already had it.
    AlreadyQueued,
    /// In the input folder, but not this machine's to take yet: the verified
    /// team Inbox admits a document only once Microsoft confirms who uploaded
    /// it, and the watcher keeps asking.
    Held,
    /// Another computer watching the same folder is working on it.
    ClaimedElsewhere,
    /// The queue would not take it, for the code given.
    Refused(String),
}

/// Copies `inputs` - files, folders, or a mixture, as a drop carries them -
/// into the input folder and hands each copy to the queue with `hand_off`.
///
/// `known` says whether the queue has ever held a document at a path, so a
/// copy is never published onto a path whose old row would answer for it: a
/// document dropped again after the first copy was filed would otherwise land
/// on the same name with the same content and be reported as already queued
/// while it sat unprocessed. `waiting` says whether a copy with exactly this
/// content (by fingerprint) is already in the input folder waiting in the
/// queue, so the same file dropped twice - "did that work?" - is copied once.
pub(crate) fn copy_into_input(
    inputs: &[PathBuf],
    folders: &Folders,
    filesystem: &dyn FileSystem,
    known: &dyn Fn(&Path) -> bool,
    waiting: &dyn Fn(&str) -> bool,
    hand_off: &dyn Fn(&Path) -> Arrival,
) -> AddReportDto {
    let (files, rejected) = crate::commands::partition_inputs(inputs);
    let mut report = AddReportDto {
        copied: Some(0),
        input_folder: Some(display_path(&folders.input)),
        ..AddReportDto::default()
    };
    for (name, code) in rejected {
        report.skip(name, code);
    }
    let mut seen = HashSet::new();
    for source in files {
        // A file dropped together with the folder it is in arrives twice.
        if !seen.insert(source.clone()) {
            continue;
        }
        let name = document_name(&source);
        if folders
            .output
            .as_deref()
            .is_some_and(|output| source.starts_with(output))
        {
            report.skip(name, IN_OUTPUT_FOLDER.into());
            continue;
        }
        if source.starts_with(&folders.input) {
            let arrival = hand_off(&source);
            if arrival == Arrival::Added {
                report.in_place = report.in_place.saturating_add(1);
            }
            report.tally(name, arrival);
            continue;
        }
        match copy_one(&source, &folders.input, filesystem, known, waiting) {
            Ok(None) => report.tally(name, Arrival::AlreadyQueued),
            Ok(Some(copy)) => {
                report.copied = report.copied.map(|count| count.saturating_add(1));
                let arrival = hand_off(&copy);
                // A copy the queue refused is taken back out, so the report's
                // "skipped" is the truth: the input folder is as it was, and
                // adding the document again later does not leave a second copy
                // beside a forgotten first.
                if matches!(arrival, Arrival::Refused(_)) {
                    let _ = std::fs::remove_file(&copy);
                    report.copied = report.copied.map(|count| count.saturating_sub(1));
                }
                report.tally(name, arrival);
            }
            Err(code) => report.skip(name, code),
        }
    }
    report
}

/// Hands every document already in the input folder to the queue: setup's
/// "Process them too", for a folder that had documents in it before Intern
/// watched it. Nothing is copied; the report has no copy count.
pub(crate) fn queue_existing(
    input: &Path,
    hand_off: &dyn Fn(&Path) -> Arrival,
) -> Result<AddReportDto, CommandError> {
    let mut report = AddReportDto::default();
    for document in collect_supported_files(input)? {
        report.tally(document_name(&document), hand_off(&document));
    }
    Ok(report)
}

impl AddReportDto {
    fn skip(&mut self, name: String, code: String) {
        self.skipped.push(SkippedDocumentDto { name, code });
    }

    fn tally(&mut self, name: String, arrival: Arrival) {
        match arrival {
            Arrival::Added => self.added = self.added.saturating_add(1),
            Arrival::AlreadyQueued => self.already_queued = self.already_queued.saturating_add(1),
            Arrival::Held => self.held = self.held.saturating_add(1),
            Arrival::ClaimedElsewhere => self.skip(name, CLAIMED_ELSEWHERE.into()),
            Arrival::Refused(code) => self.skip(name, code),
        }
    }
}

/// Copies one document into `input` under its own name, or the first
/// numbered one free, and returns where the copy is - or `None` when a copy
/// of exactly this content is already waiting in the queue.
fn copy_one(
    source: &Path,
    input: &Path,
    filesystem: &dyn FileSystem,
    known: &dyn Fn(&Path) -> bool,
    waiting: &dyn Fn(&str) -> bool,
) -> Result<Option<PathBuf>, String> {
    let name = source.file_name().ok_or_else(|| COPY_FAILED.to_owned())?;
    let retry = LockRetry::default();
    let original = retry
        .run(|| filesystem.hash(source))
        .map_err(|error| read_failure(&error))?;
    if waiting(&original) {
        return Ok(None);
    }
    let temporary = temporary_path(&input.join(name));
    let mut copy = retry
        .run(|| filesystem.copy_new_locked(source, &temporary))
        .map_err(|error| {
            if is_transient_lock(&error) {
                "SOURCE_LOCKED".to_owned()
            } else {
                COPY_FAILED.to_owned()
            }
        })?;
    match copy.hash() {
        Ok(copied) if copied == original => {}
        // Changed while it was being copied - a scanner still writing it, a
        // download still arriving. The copy is not the document the person
        // picked, so it goes.
        Ok(_) => {
            let _ = copy.delete();
            return Err("FILE_CHANGED".into());
        }
        Err(_) => {
            let _ = copy.delete();
            return Err(COPY_FAILED.into());
        }
    }
    // The handle shares reading only, and the rename below needs it gone.
    drop(copy);
    for index in 1..=NAME_ATTEMPTS {
        let candidate = input.join(numbered(name, index));
        if filesystem.exists(&candidate) || known(&candidate) {
            continue;
        }
        match retry.run(|| filesystem.rename_no_replace(&temporary, &candidate)) {
            Ok(()) => return Ok(Some(candidate)),
            // Taken between the look and the rename, or by a name that only
            // differs in case: the next number.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(_) => break,
        }
    }
    let _ = std::fs::remove_file(&temporary);
    Err(COPY_FAILED.into())
}

/// Why the original could not be read for its fingerprint.
fn read_failure(error: &io::Error) -> String {
    if is_transient_lock(error) {
        "SOURCE_LOCKED".into()
    } else if error.kind() == io::ErrorKind::NotFound {
        "FILE_MISSING".into()
    } else {
        "IO_ERROR".into()
    }
}

/// `name` itself for the first attempt, then "Lease (2).pdf", "Lease (3).pdf"
/// - the numbering filing uses for a name already taken in the output folder.
fn numbered(name: &OsStr, index: u32) -> OsString {
    if index == 1 {
        return name.to_owned();
    }
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    match path.extension() {
        Some(extension) => format!("{stem} ({index}).{}", extension.to_string_lossy()).into(),
        None => format!("{stem} ({index})").into(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        path::{Path, PathBuf},
    };

    use intern_core::{FileSystem, StdFileSystem};

    use super::{Arrival, Folders, copy_into_input, numbered, queue_existing};
    use crate::commands::{
        AddReportDto, SkippedDocumentDto,
        scratch_queue::{folder, write},
    };

    fn folders(root: &Path) -> Folders {
        let input = root.join("Input");
        let output = root.join("Output");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::create_dir_all(&output).unwrap();
        Folders {
            input,
            output: Some(output),
        }
    }

    fn names(folder: &Path) -> Vec<String> {
        let mut names = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn numbered_names_follow_the_filing_convention() {
        assert_eq!(numbered("Lease.pdf".as_ref(), 1), "Lease.pdf");
        assert_eq!(numbered("Lease.pdf".as_ref(), 2), "Lease (2).pdf");
        assert_eq!(numbered("notes".as_ref(), 3), "notes (3)");
    }

    /// Files and a folder dropped together: the documents are copied into the
    /// input folder, the folder flattened, a clash numbered, what is not a
    /// document reported as before, and every original left exactly where
    /// and as it was.
    #[test]
    fn a_drop_is_copied_flat_into_the_input_folder_and_the_originals_are_untouched() {
        let root = folder("copy-drop");
        let folders = folders(&root);
        let desktop = root.join("Desktop");
        std::fs::create_dir_all(desktop.join("Scans").join("March")).unwrap();
        let lease = write(&desktop, "Lease.pdf", b"%PDF-1.7 a lease");
        let archive = write(&desktop, "notes.zip", b"PK an archive");
        write(
            &desktop.join("Scans"),
            "Lease.pdf",
            b"%PDF-1.7 another lease",
        );
        write(
            &desktop.join("Scans").join("March"),
            "scan.txt",
            b"a letter",
        );
        write(&desktop.join("Scans"), "~$draft.docx", b"lock");
        write(
            &folders.input,
            "scan.txt",
            b"already in the input folder by name",
        );
        let handed = RefCell::new(Vec::new());

        let report = copy_into_input(
            &[lease.clone(), archive, desktop.join("Scans")],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|path| {
                handed.borrow_mut().push(path.to_path_buf());
                Arrival::Added
            },
        );

        assert_eq!(report.copied, Some(3));
        assert_eq!(report.added, 3);
        assert_eq!(
            report.skipped,
            vec![SkippedDocumentDto {
                name: "notes.zip".into(),
                code: "UNSUPPORTED_FORMAT".into(),
            }]
        );
        assert_eq!(
            names(&folders.input),
            vec!["Lease (2).pdf", "Lease.pdf", "scan (2).txt", "scan.txt"]
        );
        assert_eq!(
            std::fs::read(folders.input.join("Lease.pdf")).unwrap(),
            b"%PDF-1.7 a lease"
        );
        assert_eq!(handed.borrow().len(), 3);
        assert!(
            handed
                .borrow()
                .iter()
                .all(|path| path.starts_with(&folders.input))
        );
        // The originals: still there, still the same.
        assert_eq!(std::fs::read(&lease).unwrap(), b"%PDF-1.7 a lease");
        assert!(
            desktop
                .join("Scans")
                .join("March")
                .join("scan.txt")
                .exists()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A document already in the input folder is handed over where it is, a
    /// filed one in the output folder is not filed again, and a name the
    /// queue has seen before is not reused.
    #[test]
    fn files_already_in_place_are_not_copied_and_known_names_are_skipped() {
        let root = folder("copy-in-place");
        let folders = folders(&root);
        let inside = write(&folders.input, "inside.pdf", b"%PDF-1.7 inside");
        let filed = write(
            folders.output.as_deref().unwrap(),
            "2024-01-02 Filed.pdf",
            b"%PDF-1.7 filed",
        );
        let elsewhere = root.join("Downloads");
        std::fs::create_dir_all(&elsewhere).unwrap();
        let again = write(&elsewhere, "Invoice.pdf", b"%PDF-1.7 invoice");
        let known_path = folders.input.join("Invoice.pdf");
        let handed = RefCell::new(Vec::<PathBuf>::new());

        let report = copy_into_input(
            &[inside.clone(), filed, again],
            &folders,
            &StdFileSystem,
            &|path| path == known_path,
            &|_| false,
            &|path| {
                handed.borrow_mut().push(path.to_path_buf());
                if path.ends_with("inside.pdf") {
                    Arrival::AlreadyQueued
                } else {
                    Arrival::Added
                }
            },
        );

        assert_eq!(
            report,
            AddReportDto {
                added: 1,
                already_queued: 1,
                held: 0,
                in_place: 0,
                copied: Some(1),
                input_folder: Some(intern_queue::paths::display_path(&folders.input)),
                skipped: vec![SkippedDocumentDto {
                    name: "2024-01-02 Filed.pdf".into(),
                    code: "IN_OUTPUT_FOLDER".into(),
                }],
            }
        );
        assert_eq!(
            *handed.borrow(),
            vec![inside, folders.input.join("Invoice (2).pdf")]
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    /// The same file dropped twice while its first copy still waits in the
    /// queue is not copied a second time.
    #[test]
    fn a_file_whose_copy_is_still_waiting_is_not_copied_again() {
        let root = folder("copy-twice");
        let folders = folders(&root);
        let source = write(&root, "twice.pdf", b"%PDF-1.7 twice");
        let fingerprint = StdFileSystem.hash(&source).unwrap();
        let report = copy_into_input(
            &[source],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|hash| hash == fingerprint,
            &|_| panic!("nothing new to hand over"),
        );
        assert_eq!(report.copied, Some(0));
        assert_eq!(report.already_queued, 1);
        assert!(names(&folders.input).is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A copy the queue will not take is removed again, and one held for
    /// Microsoft's word stays for the watcher.
    #[test]
    fn refused_copies_are_taken_back_and_held_ones_stay() {
        let root = folder("copy-refused");
        let folders = folders(&root);
        let source = root.join("Source");
        std::fs::create_dir_all(&source).unwrap();
        let refused = write(&source, "refused.pdf", b"%PDF-1.7 refused");
        let held = write(&source, "held.pdf", b"%PDF-1.7 held");

        let report = copy_into_input(
            &[refused, held],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|path| {
                if path.ends_with("refused.pdf") {
                    Arrival::Refused("SOURCE_LOCKED".into())
                } else {
                    Arrival::Held
                }
            },
        );

        assert_eq!(report.copied, Some(1));
        assert_eq!(report.held, 1);
        assert_eq!(report.added, 0);
        assert_eq!(
            report.skipped,
            vec![SkippedDocumentDto {
                name: "refused.pdf".into(),
                code: "SOURCE_LOCKED".into(),
            }]
        );
        assert_eq!(names(&folders.input), vec!["held.pdf"]);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// The wire shape: copy reports carry the copy count and the folder;
    /// setup's report of what was already there does not.
    #[test]
    fn reports_serialize_for_the_window() {
        let root = folder("copy-wire");
        let folders = folders(&root);
        write(&folders.input, "a.pdf", b"%PDF-1.7 a");
        let existing = queue_existing(&folders.input, &|_| Arrival::Added).unwrap();
        assert_eq!(
            serde_json::to_value(&existing).unwrap(),
            serde_json::json!({ "added": 1, "alreadyQueued": 0, "skipped": [] })
        );
        let source = write(&root, "b.pdf", b"%PDF-1.7 b");
        let copied = copy_into_input(
            &[source],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| Arrival::Held,
        );
        assert_eq!(
            serde_json::to_value(&copied).unwrap(),
            serde_json::json!({
                "added": 0,
                "alreadyQueued": 0,
                "skipped": [],
                "copied": 1,
                "held": 1,
                "inputFolder": intern_queue::paths::display_path(&folders.input),
            })
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
