use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use intern_worker::extract::{CancellationToken, ExtractionWarning, PageSource};
use intern_worker::limits::ResourceLimits;
use intern_worker::sheet::{MAX_CELL_CHARS, MAX_SHEET_COLS, MAX_SHEET_ROWS, extract_xlsx};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;

/// Writes a minimal calamine-readable workbook whose sheets carry the given
/// `<sheetData>` inner XML.
fn write_xlsx(directory: &TempDir, sheets: &[(&str, String)]) -> PathBuf {
    write_workbook(directory, "workbook.xlsx", sheets, None)
}

/// Cell styles for a workbook with dates in it: `s="1"` is a date
/// (`m/d/yyyy`), `s="2"` a date and time.
const DATE_STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <cellXfs count="3"><xf numFmtId="0"/><xf numFmtId="14"/><xf numFmtId="22"/></cellXfs>
</styleSheet>"#;

/// [`write_xlsx`] under a name of its own, with a `styles.xml` when one is
/// given.
fn write_workbook(
    directory: &TempDir,
    file_name: &str,
    sheets: &[(&str, String)],
    styles: Option<&str>,
) -> PathBuf {
    let path = directory.path().join(file_name);
    let file = File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    let mut content_types = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#,
    );
    for index in 1..=sheets.len() {
        content_types.push_str(&format!(
            "\n  <Override PartName=\"/xl/worksheets/sheet{index}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>"
        ));
    }
    content_types.push_str("\n</Types>");
    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#).unwrap();

    let mut workbook = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>"#,
    );
    let mut workbook_rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for (index, (name, _)) in sheets.iter().enumerate() {
        let id = index + 1;
        workbook.push_str(&format!(
            "\n    <sheet name=\"{name}\" sheetId=\"{id}\" r:id=\"rId{id}\"/>"
        ));
        workbook_rels.push_str(&format!(
            "\n  <Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{id}.xml\"/>"
        ));
    }
    workbook.push_str("\n  </sheets>\n</workbook>");
    workbook_rels.push_str("\n</Relationships>");
    zip.start_file("xl/workbook.xml", options).unwrap();
    zip.write_all(workbook.as_bytes()).unwrap();
    zip.start_file("xl/_rels/workbook.xml.rels", options)
        .unwrap();
    zip.write_all(workbook_rels.as_bytes()).unwrap();
    if let Some(styles) = styles {
        zip.start_file("xl/styles.xml", options).unwrap();
        zip.write_all(styles.as_bytes()).unwrap();
    }

    for (index, (_, sheet_data)) in sheets.iter().enumerate() {
        zip.start_file(format!("xl/worksheets/sheet{}.xml", index + 1), options)
            .unwrap();
        zip.write_all(
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>{sheet_data}</sheetData>
</worksheet>"#
            )
            .as_bytes(),
        )
        .unwrap();
    }
    zip.finish().unwrap();
    path
}

fn column_reference(mut index: usize) -> String {
    let mut reference = String::new();
    loop {
        reference.insert(0, (b'A' + (index % 26) as u8) as char);
        if index < 26 {
            return reference;
        }
        index = index / 26 - 1;
    }
}

/// `<sheetData>` rows of inline strings.
fn inline_rows(rows: &[Vec<String>]) -> String {
    let mut data = String::new();
    for (row_index, row) in rows.iter().enumerate() {
        data.push_str(&format!("<row r=\"{}\">", row_index + 1));
        for (col_index, value) in row.iter().enumerate() {
            data.push_str(&format!(
                "<c r=\"{}{}\" t=\"inlineStr\"><is><t>{value}</t></is></c>",
                column_reference(col_index),
                row_index + 1
            ));
        }
        data.push_str("</row>");
    }
    data
}

#[test]
fn each_nonempty_sheet_becomes_one_markdown_page_with_its_name_as_heading() {
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(
        &directory,
        &[
            (
                "Invoices",
                inline_rows(&[
                    vec!["Invoice".to_owned(), "Amount".to_owned()],
                    vec!["INV-2048".to_owned(), "1200".to_owned()],
                ]),
            ),
            ("Blank", String::new()),
            (
                "Notes",
                inline_rows(&[vec!["Reviewed by finance".to_owned()]]),
            ),
        ],
    );

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();

    assert_eq!(document.pages.len(), 2);
    assert_eq!(document.pages[0].page_number, 1);
    assert_eq!(document.pages[1].page_number, 2);
    assert!(
        document
            .pages
            .iter()
            .all(|page| page.source == PageSource::AnyDoc)
    );
    assert_eq!(
        document.pages[0].text,
        "## Invoices\n\n\
         | Invoice | Amount |\n\
         | --- | --- |\n\
         | INV-2048 | 1200 |\n"
    );
    assert_eq!(
        document.pages[1].text,
        "## Notes\n\n| Reviewed by finance |\n| --- |\n"
    );
    assert!(!document.truncated);
    assert!(document.warnings.is_empty());
}

#[test]
fn rows_beyond_the_cap_are_elided_with_a_marker_instead_of_flooding_the_page() {
    let rows = (0..MAX_SHEET_ROWS + 25)
        .map(|index| vec![format!("row {}", index + 1)])
        .collect::<Vec<_>>();
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(&directory, &[("Ledger", inline_rows(&rows))]);

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains(&format!("| row {MAX_SHEET_ROWS} |")),
        "{text}"
    );
    assert!(
        !text.contains(&format!("| row {} |", MAX_SHEET_ROWS + 1)),
        "{text}"
    );
    assert!(text.ends_with("[... 25 more rows not shown]\n"), "{text}");
    assert!(!document.truncated);
    assert_eq!(document.warnings, vec![ExtractionWarning::ContentElided]);
}

#[test]
fn columns_beyond_the_cap_are_elided_with_a_marker() {
    let wide_row = (0..MAX_SHEET_COLS + 5)
        .map(|index| format!("col {}", index + 1))
        .collect::<Vec<_>>();
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(&directory, &[("Wide", inline_rows(&[wide_row]))]);

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains(&format!("| col {MAX_SHEET_COLS} |")),
        "{text}"
    );
    assert!(
        !text.contains(&format!("col {}", MAX_SHEET_COLS + 1)),
        "{text}"
    );
    assert!(text.ends_with("[... 5 more columns not shown]\n"), "{text}");
    assert!(!document.truncated);
    assert_eq!(document.warnings, vec![ExtractionWarning::ContentElided]);
}

#[test]
fn empty_cells_collapse_and_formulas_surface_their_cached_values() {
    let sheet_data = concat!(
        "<row r=\"1\">",
        "<c r=\"A1\" t=\"inlineStr\"><is><t>Total</t></is></c>",
        "<c r=\"C1\"><f>1+1</f><v>2</v></c>",
        "</row>",
    );
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(&directory, &[("Summary", sheet_data.to_owned())]);

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();

    assert_eq!(
        document.pages[0].text,
        "## Summary\n\n| Total |  | 2 |\n| --- | --- | --- |\n"
    );
}

#[test]
fn a_workbook_with_no_readable_cells_is_reported_as_unsupported() {
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(&directory, &[("Empty", String::new())]);

    let error =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap_err();

    assert_eq!(error.code(), "UNSUPPORTED_FORMAT");
}

#[test]
fn a_corrupt_workbook_reports_a_parse_error_instead_of_panicking() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("corrupt.xlsx");
    std::fs::write(&path, b"this is not a zip archive at all").unwrap();

    let error =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap_err();

    assert_eq!(error.code(), "PARSE_FAILED");
    assert!(!error.retryable());
}

/// Two cells at opposite corners of the sheet describe a bounding box of
/// seventeen billion cells. Reading the sheet as a dense range asks the
/// allocator for that box before any cap applies, and the process aborts;
/// only the window that is actually rendered may be materialized.
#[test]
fn a_sheet_with_a_far_away_cell_renders_the_cap_without_allocating_the_bounding_box() {
    let sheet_data = concat!(
        "<row r=\"1\"><c r=\"A1\" t=\"inlineStr\"><is><t>Statement of Account</t></is></c></row>",
        "<row r=\"1048576\"><c r=\"XFD1048576\" t=\"inlineStr\"><is><t>stray</t></is></c></row>",
    );
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(&directory, &[("Ledger", sheet_data.to_owned())]);

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.starts_with("## Ledger\n\n| Statement of Account |\n"),
        "{text}"
    );
    assert!(
        text.ends_with("[... 1048575 more rows and 16383 more columns not shown]\n"),
        "{text}"
    );
    assert!(!document.truncated);
    assert_eq!(document.warnings, vec![ExtractionWarning::ContentElided]);
}

/// An expense ledger runs to hundreds of rows, and what names it - the
/// title, the date, the vendor - sits in the first five. Leaving the rest
/// out is the window doing its job, marked where it happens, so it is
/// reported as elision: a note the host can pass over, not a truncation
/// that sends every long spreadsheet to review.
#[test]
fn deliberate_window_elision_is_content_elided_not_truncated() {
    let mut rows = vec![
        vec!["Expense Ledger".to_owned()],
        vec!["Period".to_owned(), "March 2025".to_owned()],
        vec![
            "Vendor".to_owned(),
            "Juniper Ridge Holdings Inc.".to_owned(),
        ],
        vec!["Prepared".to_owned(), "April 2, 2025".to_owned()],
        vec!["Date".to_owned(), "Item".to_owned(), "Amount".to_owned()],
    ];
    rows.extend((0..345).map(|index| {
        vec![
            "2025-03-01".to_owned(),
            format!("Line item {}", index + 1),
            "12.50".to_owned(),
        ]
    }));
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(&directory, &[("Ledger", inline_rows(&rows))]);

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains("| Vendor | Juniper Ridge Holdings Inc. |"),
        "{text}"
    );
    assert!(text.ends_with("[... 150 more rows not shown]\n"), "{text}");
    assert!(!document.truncated);
    assert_eq!(document.warnings, vec![ExtractionWarning::ContentElided]);
    // The wire name the host reads.
    assert_eq!(
        serde_json::to_value(&document.warnings).unwrap(),
        serde_json::json!(["CONTENT_ELIDED"])
    );
}

/// A cell may hold 32,767 characters. One repeated across the window would
/// render a table hundreds of megabytes long before any page cap saw it.
#[test]
fn an_oversized_cell_is_cut_before_it_is_rendered() {
    let long = "x".repeat(MAX_CELL_CHARS * 3);
    let directory = tempfile::tempdir().unwrap();
    let path = write_xlsx(
        &directory,
        &[("Notes", inline_rows(&[vec!["Memo".to_owned(), long]]))],
    );

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains(&format!("| Memo | {}… |", "x".repeat(MAX_CELL_CHARS))),
        "{}",
        text.len()
    );
    assert!(!text.contains(&"x".repeat(MAX_CELL_CHARS + 1)));
}

/// One cell of a dated sheet: its reference, its attributes, the `<f>`
/// element it holds if any, and its cached value.
struct DatedCell {
    at: String,
    attributes: &'static str,
    formula: Option<&'static str>,
    value: &'static str,
}

/// `<sheetData>` of the given cells, each on the row its reference names
/// (rows in order, cells of a row together) - with their formulas, or as
/// the same values typed in.
fn dated_rows(cells: &[DatedCell], with_formulas: bool) -> String {
    let mut data = String::new();
    let mut open: Option<&str> = None;
    for cell in cells {
        let row = cell.at.trim_start_matches(char::is_alphabetic);
        if open != Some(row) {
            if open.is_some() {
                data.push_str("</row>");
            }
            data.push_str(&format!("<row r=\"{row}\">"));
            open = Some(row);
        }
        data.push_str(&format!(
            "<c r=\"{}\"{}>{}<v>{}</v></c>",
            cell.at,
            cell.attributes,
            cell.formula.filter(|_| with_formulas).unwrap_or_default(),
            cell.value
        ));
    }
    if open.is_some() {
        data.push_str("</row>");
    }
    data
}

fn label(at: &str, text: &'static str) -> DatedCell {
    DatedCell {
        at: at.to_owned(),
        attributes: " t=\"str\"",
        formula: None,
        value: text,
    }
}

/// A cell formatted as a date (`s="1"`), holding a day's serial number.
fn date(at: &str, formula: Option<&'static str>, serial: &'static str) -> DatedCell {
    DatedCell {
        at: at.to_owned(),
        attributes: " s=\"1\"",
        formula,
        value: serial,
    }
}

/// Each marked range of a page's text with the line it stands on, to show
/// both what was marked and where.
fn marked(page: &intern_worker::extract::ExtractedPage) -> Vec<(&str, &str)> {
    page.volatile_spans
        .iter()
        .map(|&(start, end)| {
            let line_start = page.text[..start].rfind('\n').map_or(0, |at| at + 1);
            let line_end = page.text[end..]
                .find('\n')
                .map_or(page.text.len(), |at| end + at);
            (&page.text[start..end], &page.text[line_start..line_end])
        })
        .collect()
}

/// An invoice template dates itself with `=TODAY()`: the cell holds the
/// day the file was last saved, and reads as another day when it is opened
/// again. The cells such a formula worked out are marked where they stand
/// in the page's text - `TODAY` or `NOW`, in any case, with arithmetic or
/// another function around it, stated in the cell or shared with the cells
/// around it - and no other cell is: not a typed date, not a formula that
/// calls neither, not text that merely says TODAY. The text is the text
/// the same values typed in give, to the byte.
#[test]
fn cells_a_today_or_now_formula_worked_out_are_marked_and_the_text_is_unchanged() {
    let cells = [
        label("A1", "INVOICE"),
        label("A2", "Typed"),
        date("B2", None, "46084"),
        label("A3", "Today"),
        date("B3", Some("<f ca=\"1\">TODAY()</f>"), "46084"),
        label("A4", "Now"),
        DatedCell {
            at: "B4".to_owned(),
            attributes: " s=\"2\"",
            formula: Some("<f>NOW()</f>"),
            value: "46084.5",
        },
        label("A5", "Due"),
        date("B5", Some("<f>TODAY()+30</f>"), "46114"),
        label("A6", "Next day"),
        date("B6", Some("<f>B2+1</f>"), "46085"),
        label("A7", "Note"),
        label("B7", "Pay TODAY() or NOW()"),
        label("A8", "Reminder"),
        DatedCell {
            at: "B8".to_owned(),
            attributes: " t=\"str\"",
            formula: Some("<f>\"Due TODAY() for \"&amp;A1</f>"),
            value: "Due TODAY() for INVOICE",
        },
        label("A9", "Either"),
        date("B9", Some("<f>IF(B2=\"\",today(),B2)</f>"), "46084"),
        // A formula shared down a column is stated once, and the cells
        // below carry only its index, as Excel writes them.
        label("A10", "Shared"),
        date(
            "B10",
            Some("<f t=\"shared\" ref=\"B10:B11\" si=\"0\">TODAY()+ROW()</f>"),
            "46094",
        ),
        label("A11", "Shared too"),
        date("B11", Some("<f t=\"shared\" si=\"0\"/>"), "46095"),
        label("A12", "Shared sum"),
        date(
            "B12",
            Some("<f t=\"shared\" ref=\"B12:B13\" si=\"1\">B2+ROW()</f>"),
            "46096",
        ),
        label("A13", "Shared sum too"),
        date("B13", Some("<f t=\"shared\" si=\"1\"/>"), "46097"),
    ];
    let directory = tempfile::tempdir().unwrap();
    let read = |file_name: &str, with_formulas: bool| {
        let path = write_workbook(
            &directory,
            file_name,
            &[("Invoice", dated_rows(&cells, with_formulas))],
            Some(DATE_STYLES),
        );
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap()
    };
    let computed = read("computed.xlsx", true);
    let typed = read("typed.xlsx", false);

    let text = "## Invoice\n\n\
                | INVOICE |  |\n\
                | --- | --- |\n\
                | Typed | 2026-03-03 |\n\
                | Today | 2026-03-03 |\n\
                | Now | 2026-03-03 12:00:00 |\n\
                | Due | 2026-04-02 |\n\
                | Next day | 2026-03-04 |\n\
                | Note | Pay TODAY() or NOW() |\n\
                | Reminder | Due TODAY() for INVOICE |\n\
                | Either | 2026-03-03 |\n\
                | Shared | 2026-03-13 |\n\
                | Shared too | 2026-03-14 |\n\
                | Shared sum | 2026-03-15 |\n\
                | Shared sum too | 2026-03-16 |\n";
    assert_eq!(computed.pages[0].text, text);
    assert_eq!(typed.pages[0].text, text);
    assert_eq!(computed.pages[0].layout, typed.pages[0].layout);

    assert_eq!(
        marked(&computed.pages[0]),
        [
            ("2026-03-03", "| Today | 2026-03-03 |"),
            ("2026-03-03 12:00:00", "| Now | 2026-03-03 12:00:00 |"),
            ("2026-04-02", "| Due | 2026-04-02 |"),
            ("2026-03-03", "| Either | 2026-03-03 |"),
            ("2026-03-13", "| Shared | 2026-03-13 |"),
            ("2026-03-14", "| Shared too | 2026-03-14 |"),
        ]
    );
    assert!(typed.pages[0].volatile_spans.is_empty());

    // The page goes over the pipe with the ranges beside its text, and a
    // page without any goes exactly as it always did: the field is not
    // written.
    let sent = serde_json::to_value(&computed.pages[0]).unwrap();
    assert_eq!(sent["volatile_spans"][0], serde_json::json!([74, 84]));
    let plain = serde_json::to_value(&typed.pages[0]).unwrap();
    assert!(plain.get("volatile_spans").is_none(), "{plain}");
    let mut without = sent.clone();
    without.as_object_mut().unwrap().remove("volatile_spans");
    assert_eq!(without, plain);
}

/// The ranges are places in one page's text: a sheet whose cells begin
/// away from A1, a cell after one whose text is escaped, and a second
/// sheet each get the ranges their own text has. A cell that renders as
/// nothing has none, and neither has a cell past the window, which is not
/// rendered.
#[test]
fn marked_cells_are_placed_in_their_own_pages_text() {
    let offset = [
        label("C5", "Quote"),
        label("C6", "Valid until"),
        date("E6", Some("<f>TODAY()+30</f>"), "46114"),
        // A formula with no cached value shows nothing.
        date("F6", Some("<f>TODAY()</f>"), ""),
        label("G6", "a | b"),
        date("H6", Some("<f>NOW()</f>"), "46084"),
    ];
    let mut long = vec![label("A1", "Ledger")];
    for row in 2..=MAX_SHEET_ROWS + 5 {
        long.push(date(&format!("A{row}"), Some("<f>TODAY()</f>"), "46084"));
    }
    let directory = tempfile::tempdir().unwrap();
    let path = write_workbook(
        &directory,
        "two.xlsx",
        &[
            ("Quote", dated_rows(&offset, true)),
            ("Ledger", dated_rows(&long, true)),
        ],
        Some(DATE_STYLES),
    );

    let document =
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();

    let dated = "| Valid until |  | 2026-04-02 |  | a \\| b | 2026-03-03 |";
    assert_eq!(
        document.pages[0].text,
        format!(
            "## Quote\n\n\
             | Quote |  |  |  |  |  |\n\
             | --- | --- | --- | --- | --- | --- |\n\
             {dated}\n"
        )
    );
    assert_eq!(
        marked(&document.pages[0]),
        [("2026-04-02", dated), ("2026-03-03", dated)]
    );

    // Every row of the window but the title, and none past it.
    let ledger = &document.pages[1];
    assert!(
        ledger.text.ends_with("[... 5 more rows not shown]\n"),
        "{}",
        ledger.text
    );
    assert_eq!(ledger.volatile_spans.len(), MAX_SHEET_ROWS - 1);
    assert!(
        marked(ledger)
            .iter()
            .all(|marked| *marked == ("2026-03-03", "| 2026-03-03 |"))
    );
}

/// The formulas are read a second time, after every sheet's values, and
/// only to mark cells: a formula calamine cannot read - a shared one with
/// no index - ends the marking of its sheet where it stands, and the text
/// and the cells already marked are kept. Before formulas were read at all
/// such a workbook was read whole, and it still is.
#[test]
fn a_formula_that_does_not_read_ends_the_marking_and_changes_no_text() {
    let cells = [
        label("A1", "Today"),
        date("B1", Some("<f>TODAY()</f>"), "46084"),
        label("A2", "Broken"),
        date("B2", Some("<f t=\"shared\">TODAY()</f>"), "46084"),
        label("A3", "After"),
        date("B3", Some("<f>TODAY()</f>"), "46084"),
    ];
    let later = [
        label("A1", "Now"),
        date("B1", Some("<f>NOW()</f>"), "46084"),
    ];
    let directory = tempfile::tempdir().unwrap();
    let read = |file_name: &str, with_formulas: bool| {
        let path = write_workbook(
            &directory,
            file_name,
            &[
                ("First", dated_rows(&cells, with_formulas)),
                ("Second", dated_rows(&later, with_formulas)),
            ],
            Some(DATE_STYLES),
        );
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap()
    };
    let computed = read("computed.xlsx", true);
    let typed = read("typed.xlsx", false);

    assert_eq!(
        computed.pages[0].text,
        "## First\n\n\
         | Today | 2026-03-03 |\n\
         | --- | --- |\n\
         | Broken | 2026-03-03 |\n\
         | After | 2026-03-03 |\n"
    );
    for (computed, typed) in computed.pages.iter().zip(&typed.pages) {
        assert_eq!(computed.text, typed.text);
    }
    assert_eq!(
        marked(&computed.pages[0]),
        [("2026-03-03", "| Today | 2026-03-03 |")]
    );
    // The next sheet is read for its formulas all the same.
    assert_eq!(
        marked(&computed.pages[1]),
        [("2026-03-03", "| Now | 2026-03-03 |")]
    );
}

/// A file that states one cell twice shows the last statement that has a
/// value, and that statement's formula is the one asked: a typed value
/// over a `=TODAY()` is typed, a `=TODAY()` over a typed value is the
/// formula's, and an empty statement changes neither. A shared formula's
/// index stated again stands for the formula stated last.
#[test]
fn a_cell_stated_twice_is_marked_by_the_statement_that_is_shown() {
    let cells = [
        label("A1", "Typed last"),
        date("B1", Some("<f>TODAY()</f>"), "46084"),
        date("B1", None, "46054"),
        label("A2", "Formula last"),
        date("B2", None, "46054"),
        date("B2", Some("<f>TODAY()</f>"), "46084"),
        label("A3", "Emptied"),
        date("B3", Some("<f>NOW()</f>"), "46084"),
        date("B3", None, ""),
        label("A4", "Shared"),
        date(
            "B4",
            Some("<f t=\"shared\" ref=\"B4:B5\" si=\"0\">TODAY()</f>"),
            "46084",
        ),
        label("A5", "Shared too"),
        date("B5", Some("<f t=\"shared\" si=\"0\"/>"), "46084"),
        label("A6", "Index used again"),
        date(
            "B6",
            Some("<f t=\"shared\" ref=\"B6:B7\" si=\"0\">B1+1</f>"),
            "46055",
        ),
        label("A7", "And again"),
        date("B7", Some("<f t=\"shared\" si=\"0\"/>"), "46056"),
    ];
    let directory = tempfile::tempdir().unwrap();
    let read = |file_name: &str, with_formulas: bool| {
        let path = write_workbook(
            &directory,
            file_name,
            &[("Twice", dated_rows(&cells, with_formulas))],
            Some(DATE_STYLES),
        );
        extract_xlsx(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap()
    };
    let computed = read("computed.xlsx", true);

    assert_eq!(
        computed.pages[0].text,
        "## Twice\n\n\
         | Typed last | 2026-02-01 |\n\
         | --- | --- |\n\
         | Formula last | 2026-03-03 |\n\
         | Emptied | 2026-03-03 |\n\
         | Shared | 2026-03-03 |\n\
         | Shared too | 2026-03-03 |\n\
         | Index used again | 2026-02-02 |\n\
         | And again | 2026-02-03 |\n"
    );
    assert_eq!(
        computed.pages[0].text,
        read("typed.xlsx", false).pages[0].text
    );
    assert_eq!(
        marked(&computed.pages[0]),
        [
            ("2026-03-03", "| Formula last | 2026-03-03 |"),
            ("2026-03-03", "| Emptied | 2026-03-03 |"),
            ("2026-03-03", "| Shared | 2026-03-03 |"),
            ("2026-03-03", "| Shared too | 2026-03-03 |"),
        ]
    );
}
