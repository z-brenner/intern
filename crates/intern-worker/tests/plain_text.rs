//! Plain text arrives in whatever encoding the program that wrote it used.
//! Notepad and PowerShell's redirection still write UTF-16 with a byte-order
//! mark, and a mark on a UTF-8 file is ordinary; none of that is a document
//! this worker may refuse or hand on with a stray glyph in front of it.

use std::path::PathBuf;

use intern_worker::extract::{
    CancellationToken, ExtractedDocument, ExtractionWarning, MAX_TEXT_FILE_BYTES, PageSource,
    extract_text,
};
use intern_worker::limits::ResourceLimits;
use tempfile::TempDir;

fn write(directory: &TempDir, bytes: &[u8]) -> PathBuf {
    let path = directory.path().join("notes.txt");
    std::fs::write(&path, bytes).unwrap();
    path
}

fn extract(bytes: &[u8]) -> ExtractedDocument {
    let directory = tempfile::tempdir().unwrap();
    let path = write(&directory, bytes);
    extract_text(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap()
}

fn utf16_le(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    bytes
}

fn utf16_be(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFE, 0xFF];
    bytes.extend(text.encode_utf16().flat_map(u16::to_be_bytes));
    bytes
}

#[test]
fn utf16_and_bom_text_files_are_read() {
    let expected = "Retainer notes\r\nBalance due: $4,200.00\r\n";

    let little_endian = extract(&utf16_le(expected));
    assert_eq!(little_endian.pages[0].text, expected);
    assert_eq!(little_endian.pages[0].source, PageSource::Text);
    assert!(little_endian.warnings.is_empty());

    let big_endian = extract(&utf16_be(expected));
    assert_eq!(big_endian.pages[0].text, expected);

    let mut with_utf8_mark = vec![0xEF, 0xBB, 0xBF];
    with_utf8_mark.extend_from_slice(expected.as_bytes());
    let marked = extract(&with_utf8_mark);
    assert_eq!(marked.pages[0].text, expected);
}

#[test]
fn plain_utf8_is_unchanged() {
    let document = extract("Retainer notes\nBalance due: $4,200.00\n".as_bytes());

    assert_eq!(
        document.pages[0].text,
        "Retainer notes\nBalance due: $4,200.00\n"
    );
    assert!(document.warnings.is_empty());
}

/// Text that is not UTF-8 and carries no mark is, on the Windows machines
/// these files come from, Windows-1252: an accounting export, a note saved
/// by an older editor. It reads correctly as that, so there is nothing to
/// warn about.
#[test]
fn windows_1252_text_decodes_without_warning() {
    let document = extract(b"Invoice for Caf\xE9 M\xFCller GmbH \xA3420 \x96 paid\r\n");

    assert_eq!(
        document.pages[0].text,
        "Invoice for Caf\u{e9} M\u{fc}ller GmbH \u{a3}420 \u{2013} paid\r\n"
    );
    assert!(document.warnings.is_empty(), "{:?}", document.warnings);
    assert!(!document.truncated);
}

/// Five byte values mean nothing in Windows-1252 and decode as C1 control
/// characters. A file that has them is in some other encoding, and what was
/// read from it is not to be trusted.
#[test]
fn bytes_windows_1252_leaves_undefined_are_flagged() {
    let document = extract(b"Payee \x81\x8D\x90 ref 1182\r\n");

    assert!(document.pages[0].text.starts_with("Payee "));
    assert_eq!(
        document.warnings,
        vec![ExtractionWarning::NativeTextCorrupt]
    );
}

/// A UTF-8 file with a damaged byte is still a UTF-8 file. Reading all of
/// it as Windows-1252 would turn every accented letter into two wrong ones,
/// so it is read as UTF-8, with the damage replaced and flagged.
#[test]
fn damaged_utf8_is_read_as_utf8_and_flagged() {
    let mut bytes = "Caf\u{e9} M\u{fc}ller ".as_bytes().to_vec();
    bytes.push(0xFF);
    bytes.extend_from_slice(" GmbH".as_bytes());
    let document = extract(&bytes);
    let text = &document.pages[0].text;

    assert!(text.starts_with("Caf\u{e9} M\u{fc}ller "), "{text}");
    assert!(text.ends_with(" GmbH"), "{text}");
    assert_eq!(
        document.warnings,
        vec![ExtractionWarning::NativeTextCorrupt]
    );
}

/// No page holds more than two million characters, so a text file is never
/// read past the bytes that could fill one. What is left unread is reported,
/// and a character cut in half where reading stopped is dropped rather than
/// condemning the whole file as not UTF-8.
#[test]
fn reading_stops_past_what_one_page_can_hold_and_says_so() {
    let mut bytes = "a".repeat(MAX_TEXT_FILE_BYTES as usize - 1).into_bytes();
    // A two-byte character straddling the read limit, then more beyond it.
    bytes.extend_from_slice("\u{e9}tail".as_bytes());
    let document = extract(&bytes);
    let text = &document.pages[0].text;

    assert_eq!(text.len(), MAX_TEXT_FILE_BYTES as usize - 1);
    assert!(text.bytes().all(|byte| byte == b'a'));
    assert!(document.truncated);
    assert_eq!(document.warnings, vec![ExtractionWarning::TextTruncated]);
}
