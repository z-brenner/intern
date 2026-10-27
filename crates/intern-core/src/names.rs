//! The names Intern itself gives files it could not file normally, and the
//! collision rule every name it chooses follows.
//!
//! Two prefixes mark a file as Intern's doing without moving it anywhere a
//! person would not look:
//!
//! - `NEEDS REVIEW - ` goes in front of a document Intern could not read - a
//!   password, a damaged file, a model that could not answer for it - renamed
//!   in its own folder, so whoever opens the input folder sees at once which
//!   files are waiting on them. Every scan of a folder skips such a name, so
//!   a file renamed this way is never taken in again as a new document.
//! - `COPY - ` goes in front of the filed name of a document whose bytes were
//!   already filed once: an exact duplicate, moved to the output folder beside
//!   the document it repeats rather than left in the input folder.

use std::path::{Path, PathBuf};

use crate::{ErrorCode, InternError, InternResult};

/// What goes in front of the name of a document Intern could not read.
pub const NEEDS_REVIEW_PREFIX: &str = "NEEDS REVIEW - ";

/// What goes in front of the filed name of an exact duplicate.
pub const COPY_PREFIX: &str = "COPY - ";

/// Whether `file_name` already carries the needs-review prefix, in any case:
/// a person who retypes it as "Needs review - " has still marked the file.
pub fn is_needs_review_name(file_name: &str) -> bool {
    starts_with_ignoring_case(file_name, NEEDS_REVIEW_PREFIX)
}

/// The needs-review name for a file called `file_name`, or `None` when it
/// already carries the prefix: a document that fails again is not marked
/// twice.
pub fn needs_review_name(file_name: &str) -> Option<String> {
    (!is_needs_review_name(file_name)).then(|| format!("{NEEDS_REVIEW_PREFIX}{file_name}"))
}

/// `file_name` without the needs-review prefix, when it carries one.
pub fn strip_needs_review_prefix(file_name: &str) -> Option<&str> {
    is_needs_review_name(file_name).then(|| &file_name[NEEDS_REVIEW_PREFIX.len()..])
}

/// The name an exact duplicate is filed under: the original's filed name
/// with the copy prefix in front, carrying the duplicate's own extension.
///
/// The bytes are the same, so the extension normally is too; a different
/// spelling of it (`.PDF` beside `.pdf`) is the duplicate's own, and it
/// keeps it. A name with no extension takes the duplicate's whole.
pub fn copy_name(original_name: &str, duplicate_extension: Option<&str>) -> String {
    let original = Path::new(original_name);
    let stem = match original.extension() {
        Some(_) => original
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| original_name.to_owned()),
        None => original_name.to_owned(),
    };
    match duplicate_extension.filter(|extension| !extension.is_empty()) {
        Some(extension) => format!("{COPY_PREFIX}{stem}.{extension}"),
        None => format!("{COPY_PREFIX}{original_name}"),
    }
}

/// The first of `requested`, then `name (2).ext`, `name (3).ext`, and so on,
/// that `taken` does not claim. Every name Intern chooses in a folder is
/// chosen this way, so a collision reads the same whichever path made it.
pub fn available_name(
    requested: &Path,
    mut taken: impl FnMut(&Path) -> bool,
) -> InternResult<PathBuf> {
    let parent = requested.parent().ok_or_else(|| {
        InternError::new(
            ErrorCode::DestinationUnavailable,
            "destination has no parent",
        )
    })?;
    let stem = requested
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            InternError::new(
                ErrorCode::DestinationUnavailable,
                "destination name is invalid",
            )
        })?;
    let extension = requested.extension().and_then(|value| value.to_str());
    for index in 1_u32.. {
        let name = if index == 1 {
            requested
                .file_name()
                .ok_or_else(|| {
                    InternError::new(
                        ErrorCode::DestinationUnavailable,
                        "destination name is missing",
                    )
                })?
                .to_owned()
        } else {
            match extension {
                Some(value) => format!("{stem} ({index}).{value}").into(),
                None => format!("{stem} ({index})").into(),
            }
        };
        let candidate = parent.join(name);
        if !taken(&candidate) {
            return Ok(candidate);
        }
    }
    unreachable!("u32 destination suffix space exhausted")
}

fn starts_with_ignoring_case(text: &str, prefix: &str) -> bool {
    text.get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        available_name, copy_name, is_needs_review_name, needs_review_name,
        strip_needs_review_prefix,
    };

    #[test]
    fn the_needs_review_prefix_is_added_once_and_recognized_in_any_case() {
        assert_eq!(
            needs_review_name("scan.pdf").as_deref(),
            Some("NEEDS REVIEW - scan.pdf")
        );
        assert_eq!(needs_review_name("NEEDS REVIEW - scan.pdf"), None);
        assert_eq!(needs_review_name("needs review - scan.pdf"), None);
        assert!(is_needs_review_name("Needs Review - scan.pdf"));
        assert!(!is_needs_review_name("NEEDS REVIEW scan.pdf"));
        assert!(!is_needs_review_name("NEEDS"));
        // A multibyte name shorter than the prefix in bytes is not a panic.
        assert!(!is_needs_review_name("é"));
        assert_eq!(
            strip_needs_review_prefix("needs review - scan.pdf"),
            Some("scan.pdf")
        );
        assert_eq!(strip_needs_review_prefix("scan.pdf"), None);
    }

    #[test]
    fn a_copy_takes_the_original_name_and_its_own_extension() {
        assert_eq!(
            copy_name("2026-01-02 Lease.pdf", Some("pdf")),
            "COPY - 2026-01-02 Lease.pdf"
        );
        assert_eq!(
            copy_name("2026-01-02 Lease.pdf", Some("PDF")),
            "COPY - 2026-01-02 Lease.PDF"
        );
        assert_eq!(copy_name("Lease", Some("pdf")), "COPY - Lease.pdf");
        assert_eq!(copy_name("Lease.pdf", None), "COPY - Lease.pdf");
        assert_eq!(
            copy_name("2026-01-02 A.B. Smith letter.docx", Some("docx")),
            "COPY - 2026-01-02 A.B. Smith letter.docx"
        );
    }

    #[test]
    fn a_taken_name_is_numbered_before_the_extension() {
        let taken = [
            PathBuf::from("/out/COPY - Lease.pdf"),
            PathBuf::from("/out/COPY - Lease (2).pdf"),
        ];
        assert_eq!(
            available_name(Path::new("/out/COPY - Lease.pdf"), |candidate| taken
                .iter()
                .any(|path| path == candidate))
            .unwrap(),
            PathBuf::from("/out/COPY - Lease (3).pdf")
        );
        assert_eq!(
            available_name(Path::new("/out/README"), |candidate| candidate
                == Path::new("/out/README"))
            .unwrap(),
            PathBuf::from("/out/README (2)")
        );
    }
}
