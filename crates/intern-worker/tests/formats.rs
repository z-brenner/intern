//! The legacy and open formats law and accounting offices actually file,
//! read from small committed fixtures LibreOffice wrote (see
//! `fixtures/formats/README.md`). Each holds the same one-page engagement
//! letter: a date and two parties, which is what naming a document needs.

use std::path::{Path, PathBuf};

use intern_worker::delimited::extract_delimited;
use intern_worker::extract::{
    CancellationToken, ExtractedDocument, ExtractionError, ExtractionWarning, extract_anydoc,
};
use intern_worker::limits::ResourceLimits;
use intern_worker::sheet::{MAX_SHEET_ROWS, extract_xls, extract_xlsx};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/formats")
        .join(name)
}

fn text_of(document: &ExtractedDocument) -> String {
    document
        .pages
        .iter()
        .map(|page| page.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read(name: &str) -> Result<ExtractedDocument, ExtractionError> {
    read_path(&fixture(name))
}

/// The same choice of reader the worker's router makes.
fn read_path(path: &Path) -> Result<ExtractedDocument, ExtractionError> {
    let limits = ResourceLimits::default();
    let cancel = CancellationToken::new();
    match path.extension().and_then(|value| value.to_str()) {
        Some("xlsx" | "xlsm") => extract_xlsx(path, &limits, &cancel),
        Some("xls") => extract_xls(path, &limits, &cancel),
        Some("csv") => extract_delimited(path, &limits, &cancel),
        _ => extract_anydoc(path, &limits, &cancel),
    }
}

/// A copy of a fixture under another name, in a directory the test owns.
fn renamed(name: &str, as_name: &str) -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(as_name);
    std::fs::copy(fixture(name), &path).unwrap();
    (directory, path)
}

#[test]
fn each_legacy_and_open_format_extracts_date_and_parties() {
    for name in [
        "letter.doc",
        "letter.docm",
        "letter.rtf",
        "letter.odt",
        "deck.ppt",
        "deck.odp",
        "ledger.xls",
        "ledger.xlsm",
        "ledger.ods",
        "ledger.csv",
    ] {
        let document = read(name).unwrap_or_else(|error| panic!("{name}: {error}"));
        let text = text_of(&document);
        for fact in [
            "March 3, 2025",
            "Harbor Lantern Accounting LLP",
            "Juniper Ridge Holdings Inc.",
            "Lena M\u{fc}ller",
        ] {
            assert!(text.contains(fact), "{name} lacks {fact:?}:\n{text}");
        }
        assert!(
            document.warnings.is_empty(),
            "{name}: {:?}",
            document.warnings
        );
        assert!(!document.truncated, "{name}");
    }
}

/// A date the workbook stores as a date - not as text - comes out as the
/// ISO date the sheet means, through the binary reader as through the
/// Excel 2007 one, and a formula as the value it last calculated.
#[test]
fn workbook_dates_and_tables_survive_every_workbook_format() {
    for name in ["ledger.xls", "ledger.xlsm"] {
        let text = text_of(&read(name).unwrap());
        assert!(text.starts_with("## Engagement\n\n"), "{name}:\n{text}");
        assert!(
            text.contains("| Fieldwork starts | 2025-04-14 |"),
            "{name}:\n{text}"
        );
        assert!(text.contains("| Fee | 48500 |"), "{name}:\n{text}");
        assert!(
            text.contains("| Fee with expenses | 50000 |"),
            "{name}:\n{text}"
        );
    }
    let csv = text_of(&read("ledger.csv").unwrap());
    assert!(csv.contains("| Letter date | March 3, 2025 |"), "{csv}");
}

/// Word has saved RTF under `.doc` for as long as both have existed.
#[test]
fn rtf_under_doc_is_read() {
    let (_directory, path) = renamed("letter.rtf", "letter.doc");
    let text = text_of(&read_path(&path).unwrap());
    assert!(text.contains("Juniper Ridge Holdings Inc."), "{text}");
    assert!(text.contains("March 3, 2025"), "{text}");
}

/// A Word file named for the other generation of Word is the same kind of
/// document, and is read as what it is - in both directions, and likewise
/// for decks.
#[test]
fn a_word_or_powerpoint_file_under_the_other_generations_extension_is_read() {
    for (name, as_name) in [
        ("letter.docm", "letter.doc"),
        ("letter.doc", "letter.docx"),
        ("deck.ppt", "deck.pptx"),
        ("deck.odp", "deck.ppt"),
    ] {
        let (_directory, path) = renamed(name, as_name);
        let text = text_of(
            &read_path(&path).unwrap_or_else(|error| panic!("{name} as {as_name}: {error}")),
        );
        assert!(
            text.contains("Juniper Ridge Holdings Inc."),
            "{name} as {as_name}: {text}"
        );
    }
}

/// Spreadsheet exports are named `.xls` whatever they are. A workbook from
/// the other generation of Excel is still a workbook, read through the same
/// window.
#[test]
fn a_workbook_under_the_other_generations_extension_is_read() {
    for (name, as_name) in [("ledger.xlsm", "ledger.xls"), ("ledger.xls", "ledger.xlsx")] {
        let (_directory, path) = renamed(name, as_name);
        let text = text_of(
            &read_path(&path).unwrap_or_else(|error| panic!("{name} as {as_name}: {error}")),
        );
        assert!(
            text.contains("| Client | Juniper Ridge Holdings Inc. |"),
            "{text}"
        );
    }
}

/// Content of another kind is still a routing failure: a workbook under a
/// word-processing extension would reach anydoc's uncapped sheet renderer.
#[test]
fn a_workbook_under_a_word_extension_is_still_refused() {
    let (_directory, path) = renamed("ledger.xls", "ledger.doc");
    let error = read_path(&path).unwrap_err();
    assert_eq!(error.code(), "UNSUPPORTED_FORMAT");
}

/// Legacy Excel encryption is a `FilePass` record inside the workbook
/// stream, not the `EncryptionInfo` streams of an encrypted package, and it
/// is reported as what it is: a password to remove, not a damaged file.
#[test]
fn a_password_protected_binary_workbook_is_password_protected() {
    let error = read("ledger-encrypted.xls").unwrap_err();
    assert_eq!(error.code(), "PASSWORD_PROTECTED");
    assert!(!error.retryable());
}

/// A bank export runs to thousands of rows. It is read through the same
/// window as a workbook - the semicolons of a European export split it, the
/// one inside a quoted payee does not - and what lies past the window is
/// elided, not truncated.
#[test]
fn a_long_csv_is_windowed_and_marked_as_elided() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("statement.csv");
    let mut csv = String::from("Date;Payee;Amount\r\n");
    for day in 0..MAX_SHEET_ROWS + 99 {
        csv.push_str(&format!(
            "2025-03-{:02};\"Juniper Ridge; Holdings\";12,50\r\n",
            day % 28 + 1
        ));
    }
    std::fs::write(&path, csv).unwrap();

    let document = read_path(&path).unwrap();
    let text = text_of(&document);

    assert!(
        text.starts_with("| Date | Payee | Amount |\n| --- | --- | --- |\n"),
        "{text}"
    );
    assert!(
        text.contains("| 2025-03-01 | Juniper Ridge; Holdings | 12,50 |"),
        "{text}"
    );
    assert!(text.ends_with("[... 100 more rows not shown]\n"), "{text}");
    assert_eq!(document.warnings, vec![ExtractionWarning::ContentElided]);
    assert!(!document.truncated);
}

/// A bank export written by an older Windows program is Windows-1252, and
/// the names in it read correctly.
#[test]
fn a_windows_1252_csv_reads_its_names_correctly() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("payees.csv");
    std::fs::write(&path, b"Payee,Amount\r\nCaf\xE9 M\xFCller GmbH,\xA3420\r\n").unwrap();

    let text = text_of(&read_path(&path).unwrap());

    assert!(
        text.contains("| Caf\u{e9} M\u{fc}ller GmbH | \u{a3}420 |"),
        "{text}"
    );
}

/// Excel's "Unicode Text" export is UTF-16 with a byte-order mark, and is
/// routinely renamed `.csv`. It is read like any other export.
#[test]
fn a_utf16_export_reads_like_any_other() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("payees.csv");
    let mut bytes = vec![0xFF, 0xFE];
    for unit in "Payee\tAmount\r\nCaf\u{e9} M\u{fc}ller GmbH\t\u{a3}420\r\n".encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    std::fs::write(&path, bytes).unwrap();

    let text = text_of(&read_path(&path).unwrap());

    assert!(
        text.starts_with(
            "| Payee | Amount |\n| --- | --- |\n| Caf\u{e9} M\u{fc}ller GmbH | \u{a3}420 |"
        ),
        "{text}"
    );
}
