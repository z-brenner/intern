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
//! temporary name inside the input folder's own hidden `.intern/incoming`
//! folder, which the scanner never enters, checked against the original byte
//! for byte, and only then published onto its real name with a rename that
//! refuses to replace anything - so a crash leaves at most a temporary file out
//! of sight, never a half-written document for the scanner to find. A
//! name already taken, on disk or by a document the queue has seen at that
//! path before, gets " (2)", " (3)", and so on. A file that is already in the
//! input folder is not copied again; it is handed to the queue where it is.

use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use intern_core::{FileIdentity, FileSystem, LockRetry, is_transient_lock, set_hidden};
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

/// A document whose uploader belongs to someone else, by the admission check.
pub(crate) const UPLOADER_OTHER: &str = "UPLOADER_OTHER";
/// A document the admission check cannot place: an unknown uploader, or a
/// proof that no longer holds.
pub(crate) const UPLOADER_UNVERIFIED: &str = "UPLOADER_UNVERIFIED";
/// Held only until Microsoft can answer: OneDrive has not uploaded the copy
/// yet, or Microsoft could not be asked just now.
pub(crate) const AWAITING_MICROSOFT: &str = "AWAITING_MICROSOFT";
/// A copy the queue did not take that could not be taken back out - a sync
/// client or scanner still held it, or the name no longer held this copy. It
/// stays in the input folder, and the scan takes it from there.
pub(crate) const LEFT_IN_INPUT_FOLDER: &str = "LEFT_IN_INPUT_FOLDER";

/// The ending of a copy still being written into the input folder. Its own,
/// so a sweep of leftovers never touches the temporary file of a filing or an
/// undo that is part-way through (those end `.intern-tmp`).
const COPY_SUFFIX: &str = ".intern-copy";
/// How much of a document's name a temporary copy carries, so a long name
/// still fits in the 255 characters a folder entry allows.
const COPY_NAME_CHARS: usize = 40;
/// How old a leftover copy must be before a sweep removes it. A copy is
/// written in seconds; one an hour old was abandoned by a crash.
const STALE_COPY_AGE: Duration = Duration::from_secs(60 * 60);

static COPY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// How many names a clash may try before the copy gives up. A folder with ten
/// thousand copies of one name is not one a person is adding to by hand.
const NAME_ATTEMPTS: u32 = 10_000;

/// The two folders an add works between, both canonical, and the machine
/// copying into them.
#[derive(Clone, Debug)]
pub(crate) struct Folders {
    pub(crate) input: PathBuf,
    pub(crate) output: Option<PathBuf>,
    /// This machine's tag in the names of its temporary copies (see
    /// `machine_tag`), so it only ever sweeps its own.
    pub(crate) machine: String,
}

/// The part of a machine id a temporary copy's name carries. The id is
/// random hex; sixteen characters of it tell this machine's copies from a
/// teammate's in the same synced folder.
pub(crate) fn machine_tag(machine_id: &str) -> String {
    machine_id
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(16)
        .collect()
}

/// Where copies are written before they are published: inside the input
/// folder's `.intern` folder, which the scan never enters, on the same volume
/// so publishing is a rename. The folder is hidden on Windows, so a copy a
/// crash interrupts is out of sight from the moment it is created, and the
/// published document - which is a different directory entry - never
/// inherits the attribute.
fn incoming_folder(input: &Path) -> io::Result<PathBuf> {
    let incoming = input.join(".intern").join("incoming");
    std::fs::create_dir_all(&incoming)?;
    let _ = set_hidden(&incoming, true);
    Ok(incoming)
}

/// What became of one document once it was in the input folder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Arrival {
    /// A new item in the queue.
    Added,
    /// The queue already had it.
    AlreadyQueued,
    /// In the input folder, but not this machine's to take yet, for the
    /// reason given (`AWAITING_MICROSOFT`, `UPLOADER_OTHER`): the verified
    /// team Inbox admits a document only once Microsoft confirms who uploaded
    /// it, and the watcher keeps asking. A held copy is never taken back.
    Held(String),
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
/// queue, so the same file dropped twice - "did that work?" - is copied once;
/// two files with the same content in one drop are copied once too.
/// `excluded` says why a file already in the input folder is not one the
/// watcher would take (a sync client's conflict copy), if it is not.
pub(crate) fn copy_into_input(
    inputs: &[PathBuf],
    folders: &Folders,
    filesystem: &dyn FileSystem,
    known: &dyn Fn(&Path) -> bool,
    waiting: &dyn Fn(&str) -> bool,
    excluded: &dyn Fn(&Path) -> Option<String>,
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
    let mut copied_now = HashSet::new();
    for source in files {
        // A file dropped together with the folder it is in arrives twice.
        if !seen.insert(source.clone()) {
            continue;
        }
        let name = document_name(&source);
        // The input folder first: a file in it is queued where it is, whatever
        // else contains it.
        if source.starts_with(&folders.input) {
            if let Some(code) = excluded(&source) {
                report.skip(name, code);
                continue;
            }
            let arrival = hand_off(&source);
            if arrival == Arrival::Added {
                report.in_place = report.in_place.saturating_add(1);
            }
            report.tally(name, arrival);
            continue;
        }
        if folders
            .output
            .as_deref()
            .is_some_and(|output| source.starts_with(output))
        {
            report.skip(name, IN_OUTPUT_FOLDER.into());
            continue;
        }
        let already = |fingerprint: &str| waiting(fingerprint) || copied_now.contains(fingerprint);
        match copy_one(&source, folders, filesystem, known, &already) {
            Ok(Copied::Already) => report.tally(name, Arrival::AlreadyQueued),
            Ok(Copied::To(copy, fingerprint, identity)) => {
                let arrival = hand_off(&copy);
                // A copy the queue refused, or one another computer took in the
                // moment between, is taken back out, so the report is the
                // truth: it is not counted as copied, the input folder is as it
                // was, and adding the document again later does not leave a
                // second copy beside a forgotten first. Only what is provably
                // still this copy is taken: in a synced folder the name can
                // meanwhile hold a teammate's document, and deleting that would
                // delete it for everyone.
                let refused = matches!(arrival, Arrival::Refused(_) | Arrival::ClaimedElsewhere);
                let taken_back = refused && take_back(filesystem, &copy, &fingerprint, &identity);
                if !taken_back {
                    report.copied = report.copied.map(|count| count.saturating_add(1));
                }
                if !refused {
                    copied_now.insert(fingerprint);
                }
                // A refused copy still in the folder is reported as what it
                // is - copied, and waiting there - not also as left out: the
                // scan will find it, and the report must not say otherwise.
                let arrival = match arrival {
                    Arrival::Refused(code) if !taken_back && code == UPLOADER_UNVERIFIED => {
                        Arrival::Held(code)
                    }
                    Arrival::Refused(_) | Arrival::ClaimedElsewhere if !taken_back => {
                        Arrival::Held(LEFT_IN_INPUT_FOLDER.into())
                    }
                    arrival => arrival,
                };
                report.tally(name, arrival);
            }
            Err(code) => report.skip(name, code),
        }
    }
    report
}

/// What `copy_one` did.
enum Copied {
    /// Copied to this path, with this content fingerprint and file identity.
    To(PathBuf, String, FileIdentity),
    /// A copy with exactly this content is already there.
    Already,
}

/// Hands every document already in the input folder to the queue: setup's
/// "Process them too", for a folder that had documents in it before Intern
/// watched it. Nothing is copied; the report has no copy count.
///
/// The scan's own rules apply: what it would never take - a sync client's
/// conflict copy, by `excluded` - is not queued either, and is named.
pub(crate) fn queue_existing(
    input: &Path,
    excluded: &dyn Fn(&Path) -> Option<String>,
    hand_off: &dyn Fn(&Path) -> Arrival,
) -> Result<AddReportDto, CommandError> {
    let mut report = AddReportDto::default();
    for document in collect_supported_files(input)? {
        let name = document_name(&document);
        match excluded(&document) {
            Some(code) => report.skip(name, code),
            None => report.tally(name, hand_off(&document)),
        }
    }
    Ok(report)
}

/// Takes back a copy that was not queued - but only while it is provably
/// still the copy: the same file (by its identity on disk) with the same
/// content. A sync client or antivirus holding the new copy for a moment is
/// waited out. True when it was removed.
fn take_back(
    filesystem: &dyn FileSystem,
    copy: &Path,
    fingerprint: &str,
    identity: &FileIdentity,
) -> bool {
    let Ok(mut locked) = LockRetry::default().run(|| filesystem.lock_for_delete(copy)) else {
        return false;
    };
    let ours = locked.identity().is_ok_and(|found| found == *identity)
        && locked.hash().is_ok_and(|found| found == fingerprint);
    ours && locked.delete().is_ok()
}

/// Removes copies a crash left part-written in the input folder: files in
/// its incoming folder named exactly as `copy_temp_path` names them, written
/// by this machine (`machine`), stamped more than `STALE_COPY_AGE` before
/// `now`, by a process that is no longer running. The incoming folder is
/// synced like the rest, so a teammate's copies are there too; their clocks
/// and their copies are theirs, and are never judged here. Anything else,
/// however it is named, stays.
pub(crate) fn sweep_stale_copies(input: &Path, machine: &str, now: SystemTime) {
    let Ok(entries) = std::fs::read_dir(input.join(".intern").join("incoming")) else {
        return;
    };
    let now = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(stamp) = name.to_str().and_then(copy_stamp) else {
            continue;
        };
        if stamp.machine == machine
            && !process_running(stamp.process)
            && now.saturating_sub(stamp.started) >= STALE_COPY_AGE
            && entry.file_type().is_ok_and(|kind| kind.is_file())
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Whether the process that wrote a copy may still be writing it. Intern
/// runs once per user, and a machine tag belongs to one user's data, so the
/// only Intern that can be writing this machine's copies is this one; where
/// the system says which processes are alive, that is asked as well.
fn process_running(process: u32) -> bool {
    if process == std::process::id() {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        Path::new(&format!("/proc/{process}")).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// What a temporary copy's name says about it.
#[derive(Debug, Eq, PartialEq)]
struct CopyStamp {
    machine: String,
    process: u32,
    started: Duration,
}

/// What a temporary copy named `name` says, or `None` for any name
/// `copy_temp_path` did not make:
/// `.<name>.<machine>.<process>.<nanoseconds>.<sequence>.intern-copy`.
fn copy_stamp(name: &str) -> Option<CopyStamp> {
    let inner = name.strip_prefix('.')?.strip_suffix(COPY_SUFFIX)?;
    let mut parts = inner.rsplitn(5, '.');
    let sequence = parts.next()?;
    let nanos = parts.next()?;
    let process = parts.next()?;
    let machine = parts.next()?;
    let original = parts.next()?;
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    if original.is_empty()
        || machine.is_empty()
        || !machine.bytes().all(|byte| byte.is_ascii_alphanumeric())
        || !digits(sequence)
        || !digits(process)
        || !digits(nanos)
    {
        return None;
    }
    Some(CopyStamp {
        machine: machine.to_owned(),
        process: process.parse().ok()?,
        started: Duration::from_nanos(nanos.parse::<u64>().ok()?),
    })
}

/// A unique, dot-prefixed name in `incoming` to write a copy of `name` under
/// before it is published, carrying at most `COPY_NAME_CHARS` of the name so
/// a long one still fits, and the copying machine's tag.
fn copy_temp_path(incoming: &Path, name: &OsStr, machine: &str, now: SystemTime) -> PathBuf {
    let short: String = name
        .to_string_lossy()
        .chars()
        .take(COPY_NAME_CHARS)
        .collect();
    let nanos = now
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = COPY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    incoming.join(format!(
        ".{short}.{machine}.{}.{nanos}.{sequence}{COPY_SUFFIX}",
        std::process::id()
    ))
}

impl AddReportDto {
    fn skip(&mut self, name: String, code: String) {
        self.skipped.push(SkippedDocumentDto { name, code });
    }

    fn tally(&mut self, name: String, arrival: Arrival) {
        match arrival {
            Arrival::Added => self.added = self.added.saturating_add(1),
            Arrival::AlreadyQueued => self.already_queued = self.already_queued.saturating_add(1),
            Arrival::Held(reason) => {
                self.held = self.held.saturating_add(1);
                // One reason for the batch: waiting for Microsoft is the
                // usual one, and any other outranks it.
                if matches!(self.held_reason.as_deref(), None | Some(AWAITING_MICROSOFT)) {
                    self.held_reason = Some(reason);
                }
            }
            Arrival::ClaimedElsewhere => self.skip(name, CLAIMED_ELSEWHERE.into()),
            Arrival::Refused(code) => self.skip(name, code),
        }
    }
}

/// Copies one document into `input` under its own name, or the first
/// numbered one free, and returns where the copy is - or `Already` when a
/// copy of exactly this content is already there (`already`).
///
/// The copy is written in the incoming folder (see `incoming_folder`) and
/// published into the input folder with a rename that replaces nothing.
fn copy_one(
    source: &Path,
    folders: &Folders,
    filesystem: &dyn FileSystem,
    known: &dyn Fn(&Path) -> bool,
    already: &dyn Fn(&str) -> bool,
) -> Result<Copied, String> {
    let input = folders.input.as_path();
    let name = source.file_name().ok_or_else(|| COPY_FAILED.to_owned())?;
    let retry = LockRetry::default();
    let original = retry
        .run(|| filesystem.hash(source))
        .map_err(|error| read_failure(&error))?;
    if already(&original) {
        return Ok(Copied::Already);
    }
    let incoming = incoming_folder(input).map_err(|_| COPY_FAILED.to_owned())?;
    let temporary = copy_temp_path(&incoming, name, &folders.machine, SystemTime::now());
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
    let Ok(identity) = copy.identity() else {
        let _ = copy.delete();
        return Err(COPY_FAILED.into());
    };
    // The handle shares reading only, and the rename below needs it gone.
    drop(copy);
    for index in 1..=NAME_ATTEMPTS {
        let candidate = input.join(numbered(name, index));
        if filesystem.exists(&candidate) || known(&candidate) {
            continue;
        }
        match retry.run(|| filesystem.rename_no_replace(&temporary, &candidate)) {
            Ok(()) => return Ok(Copied::To(candidate, original, identity)),
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
            machine: "here0123456789ab".into(),
        }
    }

    /// The documents in a folder: what a person sees, without Intern's own
    /// `.intern` folder.
    fn names(folder: &Path) -> Vec<String> {
        let mut names = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with('.'))
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
            &|_| None,
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
            &|_| None,
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
                held_reason: None,
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
            &|_| None,
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
            &|_| None,
            &|path| {
                if path.ends_with("refused.pdf") {
                    Arrival::Refused("SOURCE_LOCKED".into())
                } else {
                    Arrival::Held(super::AWAITING_MICROSOFT.into())
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

    /// An input folder inside the output folder - D:\Docs\Inbox inside
    /// D:\Docs - is refused on save, but a file in it is still the input
    /// folder's: it is queued where it is, not refused as filed already.
    #[test]
    fn a_file_in_the_input_folder_is_queued_even_when_the_output_folder_holds_it() {
        let root = folder("copy-nested");
        let input = root.join("Inbox");
        std::fs::create_dir_all(&input).unwrap();
        let inside = write(&input, "scan.pdf", b"%PDF-1.7 scan");
        let folders = Folders {
            input,
            output: Some(root.clone()),
            machine: "here0123456789ab".into(),
        };
        let report = copy_into_input(
            &[inside],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &|_| Arrival::Added,
        );
        assert_eq!(report.added, 1);
        assert_eq!(report.in_place, 1);
        assert!(report.skipped.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Two files with the same content in one drop are one document.
    #[test]
    fn identical_files_in_one_drop_are_copied_once() {
        let root = folder("copy-identical");
        let folders = folders(&root);
        let downloads = root.join("Downloads");
        std::fs::create_dir_all(&downloads).unwrap();
        let first = write(&downloads, "Lease.pdf", b"%PDF-1.7 the same lease");
        let second = write(&downloads, "Lease copy.pdf", b"%PDF-1.7 the same lease");
        let report = copy_into_input(
            &[first, second],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &|_| Arrival::Added,
        );
        assert_eq!(report.copied, Some(1));
        assert_eq!(report.added, 1);
        assert_eq!(report.already_queued, 1);
        assert_eq!(names(&folders.input), vec!["Lease.pdf"]);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A copy another computer claims in the moment between is not counted as
    /// copied while also named as left out, and is not left behind.
    #[test]
    fn a_copy_claimed_elsewhere_is_taken_back_and_not_counted() {
        let root = folder("copy-claimed");
        let folders = folders(&root);
        let source = write(&root, "claimed.pdf", b"%PDF-1.7 claimed");
        let report = copy_into_input(
            &[source],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &|_| Arrival::ClaimedElsewhere,
        );
        assert_eq!(report.copied, Some(0));
        assert_eq!(
            report.skipped,
            vec![SkippedDocumentDto {
                name: "claimed.pdf".into(),
                code: "CLAIMED_ELSEWHERE".into(),
            }]
        );
        assert!(names(&folders.input).is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    /// The temporary copy carries a shortened name, so a document whose own
    /// name nearly fills a folder entry still copies.
    #[test]
    fn a_long_name_still_copies() {
        let root = folder("copy-long");
        let folders = folders(&root);
        let long = format!("{}.pdf", "a".repeat(240));
        let source = write(&root, &long, b"%PDF-1.7 long");
        let report = copy_into_input(
            &[source],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &|_| Arrival::Added,
        );
        assert_eq!(report.copied, Some(1), "{report:?}");
        assert_eq!(names(&folders.input), vec![long]);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A copy a crash left behind on this machine is swept once it is stale.
    /// A teammate's copy in the same synced folder - whatever its stamp says
    /// against this machine's clock - a copy this process is writing, a
    /// filing's own temporary file, and anything else that merely starts with
    /// a dot all stay.
    #[test]
    fn only_this_machine_s_stale_copies_are_swept() {
        let root = folder("copy-sweep");
        let incoming = root.join(".intern").join("incoming");
        std::fs::create_dir_all(&incoming).unwrap();
        let now = std::time::SystemTime::now();
        let hours_ago = (now - std::time::Duration::from_secs(2 * 60 * 60))
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // 3999999999 is no running process.
        let stale = write(
            &incoming,
            &format!(".Lease.pdf.here0123456789ab.3999999999.{hours_ago}.0.intern-copy"),
            b"half",
        );
        let teammate = write(
            &incoming,
            &format!(".Lease.pdf.elsewhere0000000.3999999999.{hours_ago}.0.intern-copy"),
            b"theirs",
        );
        let running = write(
            &incoming,
            &format!(
                ".Lease.pdf.here0123456789ab.{}.{hours_ago}.1.intern-copy",
                std::process::id()
            ),
            b"being written",
        );
        let fresh = super::copy_temp_path(&incoming, "Lease.pdf".as_ref(), "here0123456789ab", now);
        std::fs::write(&fresh, b"half").unwrap();
        let filing = write(
            &incoming,
            ".Lease.pdf.12.34.0.intern-tmp",
            b"a filing's temp",
        );
        let dotted = write(
            &incoming,
            ".Lease.pdf.here0123456789ab.1.2.0.intern-copy.bak",
            b"not ours",
        );
        let odd = write(&incoming, ".x.y.z.w.intern-copy", b"not numeric");
        super::sweep_stale_copies(&root, "here0123456789ab", now);
        assert!(!stale.exists());
        assert!(teammate.exists(), "a teammate's copy is theirs");
        assert!(running.exists(), "this process may still be writing it");
        assert!(fresh.exists());
        assert!(filing.exists());
        assert!(dotted.exists());
        assert!(odd.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A refused copy is taken back only while it is provably the copy. In a
    /// synced folder the name can meanwhile hold a teammate's document - here,
    /// written in its place while the hand-over ran - and deleting that would
    /// delete it for everyone.
    #[test]
    fn a_refused_copy_is_taken_back_only_while_it_is_still_the_copy() {
        let root = folder("copy-replaced");
        let folders = folders(&root);
        let source = write(&root, "lease.pdf", b"%PDF-1.7 ours");
        let report = copy_into_input(
            &[source],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &|path| {
                std::fs::remove_file(path).unwrap();
                std::fs::write(path, b"%PDF-1.7 a teammate's document").unwrap();
                Arrival::Refused("UPLOADER_UNVERIFIED".into())
            },
        );
        assert_eq!(
            std::fs::read(folders.input.join("lease.pdf")).unwrap(),
            b"%PDF-1.7 a teammate's document"
        );
        assert_eq!(
            report.copied,
            Some(1),
            "it is still there, so it is not taken back"
        );

        // The same bytes, but another file: a teammate saved the same scan
        // under the same name. Still not this copy.
        let again = write(&root, "deed.pdf", b"%PDF-1.7 the same deed");
        let report = copy_into_input(
            &[again],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &|path| {
                let theirs = path.with_file_name("theirs.tmp");
                std::fs::write(&theirs, b"%PDF-1.7 the same deed").unwrap();
                std::fs::rename(&theirs, path).unwrap();
                Arrival::Refused("UPLOADER_UNVERIFIED".into())
            },
        );
        assert!(folders.input.join("deed.pdf").exists());
        assert_eq!(report.copied, Some(1));
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A file system whose files can never be opened for deletion: a sync
    /// client or antivirus that will not let go of a new copy.
    struct Held;

    impl FileSystem for Held {
        fn exists(&self, path: &Path) -> bool {
            StdFileSystem.exists(path)
        }
        fn hash(&self, path: &Path) -> std::io::Result<String> {
            StdFileSystem.hash(path)
        }
        fn same_volume(&self, source: &Path, destination: &Path) -> std::io::Result<bool> {
            StdFileSystem.same_volume(source, destination)
        }
        fn rename_no_replace(&self, source: &Path, destination: &Path) -> std::io::Result<()> {
            StdFileSystem.rename_no_replace(source, destination)
        }
        fn copy_new_locked(
            &self,
            source: &Path,
            destination: &Path,
        ) -> std::io::Result<Box<dyn intern_core::LockedFile>> {
            StdFileSystem.copy_new_locked(source, destination)
        }
        fn lock_for_delete(
            &self,
            _path: &Path,
        ) -> std::io::Result<Box<dyn intern_core::LockedFile>> {
            Err(std::io::ErrorKind::PermissionDenied.into())
        }
    }

    /// A refused copy that cannot be taken back is reported once, as what it
    /// is: copied and waiting in the input folder - not counted as copied and
    /// named as left out as well, when the scan will process it from there.
    #[test]
    fn a_refused_copy_that_cannot_be_taken_back_is_reported_as_left_in_the_folder() {
        let root = folder("copy-stuck");
        let folders = folders(&root);
        let stuck = write(&root, "stuck.pdf", b"%PDF-1.7 stuck");
        let unverified = write(&root, "unverified.pdf", b"%PDF-1.7 unverified");
        let report = copy_into_input(
            &[stuck, unverified],
            &folders,
            &Held,
            &|_| false,
            &|_| false,
            &|_| None,
            &|path| {
                if path.ends_with("stuck.pdf") {
                    Arrival::Refused("CLAIM_IO_FAILED".into())
                } else {
                    Arrival::Refused(super::UPLOADER_UNVERIFIED.into())
                }
            },
        );
        assert_eq!(report.copied, Some(2));
        assert_eq!(report.held, 2);
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);
        assert_eq!(report.held_reason.as_deref(), Some("LEFT_IN_INPUT_FOLDER"));
        assert_eq!(names(&folders.input), vec!["stuck.pdf", "unverified.pdf"]);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// The copy is written inside the input folder's own `.intern` folder and
    /// published from there, so nothing half-written ever sits among the
    /// documents, and the published document is an ordinary file.
    #[test]
    fn a_copy_is_written_out_of_sight_and_published_among_the_documents() {
        let root = folder("copy-incoming");
        let folders = folders(&root);
        let source = write(&root, "a.pdf", b"%PDF-1.7 a");
        let seen = RefCell::new(Vec::new());
        copy_into_input(
            &[source],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &|path| {
                seen.borrow_mut().push(path.to_path_buf());
                Arrival::Added
            },
        );
        assert_eq!(*seen.borrow(), vec![folders.input.join("a.pdf")]);
        let incoming = folders.input.join(".intern").join("incoming");
        assert!(incoming.is_dir());
        assert_eq!(std::fs::read_dir(incoming).unwrap().count(), 0);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// "Process them too" follows the scan's rules: a conflict copy is named,
    /// never queued.
    #[test]
    fn queueing_what_is_there_leaves_out_what_the_scan_would() {
        let root = folder("copy-existing");
        let folders = folders(&root);
        write(&folders.input, "a.pdf", b"%PDF-1.7 a");
        write(
            &folders.input,
            "a (Pat's conflicted copy).pdf",
            b"%PDF-1.7 a2",
        );
        let report = queue_existing(
            &folders.input,
            &|path| {
                path.to_string_lossy()
                    .contains("conflicted copy")
                    .then(|| "SYNC_CONFLICT_COPY".to_owned())
            },
            &|_| Arrival::Added,
        )
        .unwrap();
        assert_eq!(report.added, 1);
        assert_eq!(
            report.skipped,
            vec![SkippedDocumentDto {
                name: "a (Pat's conflicted copy).pdf".into(),
                code: "SYNC_CONFLICT_COPY".into(),
            }]
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A `NEEDS REVIEW - ` file is one Intern renamed because it could not
    /// read it, and it waits in the queue already: "Process them too" passes
    /// over it, and a drop names it rather than copying it in a second time.
    #[test]
    fn a_needs_review_file_is_never_queued_or_copied_again() {
        let root = folder("copy-needs-review");
        let folders = folders(&root);
        write(&folders.input, "a.pdf", b"%PDF-1.7 a");
        write(
            &folders.input,
            "NEEDS REVIEW - locked.pdf",
            b"%PDF-1.7 locked",
        );
        let handed = std::cell::RefCell::new(Vec::new());
        let hand_off = |path: &Path| {
            handed.borrow_mut().push(crate::commands::document_name(path));
            Arrival::Added
        };
        let report = queue_existing(&folders.input, &|_| None, &hand_off).unwrap();
        assert_eq!(report.added, 1);
        assert!(report.skipped.is_empty());
        assert_eq!(*handed.borrow(), vec!["a.pdf".to_owned()]);

        let elsewhere = root.join("Downloads");
        std::fs::create_dir_all(&elsewhere).unwrap();
        let marked = write(&elsewhere, "NEEDS REVIEW - scan.pdf", b"%PDF-1.7 scan");
        let copied = copy_into_input(
            &[marked],
            &folders,
            &StdFileSystem,
            &|_| false,
            &|_| false,
            &|_| None,
            &hand_off,
        );
        assert_eq!(copied.copied, Some(0));
        assert_eq!(
            copied.skipped,
            vec![SkippedDocumentDto {
                name: "NEEDS REVIEW - scan.pdf".into(),
                code: "NEEDS_REVIEW_FILE".into(),
            }]
        );
        assert!(!folders.input.join("NEEDS REVIEW - scan.pdf").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    /// The wire shape: copy reports carry the copy count and the folder;
    /// setup's report of what was already there does not.
    #[test]
    fn reports_serialize_for_the_window() {
        let root = folder("copy-wire");
        let folders = folders(&root);
        write(&folders.input, "a.pdf", b"%PDF-1.7 a");
        let existing = queue_existing(&folders.input, &|_| None, &|_| Arrival::Added).unwrap();
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
            &|_| None,
            &|_| Arrival::Held(super::AWAITING_MICROSOFT.into()),
        );
        assert_eq!(
            serde_json::to_value(&copied).unwrap(),
            serde_json::json!({
                "added": 0,
                "alreadyQueued": 0,
                "skipped": [],
                "copied": 1,
                "held": 1,
                "heldReason": "AWAITING_MICROSOFT",
                "inputFolder": intern_queue::paths::display_path(&folders.input),
            })
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
