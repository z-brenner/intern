//! Spreadsheet extraction: Excel 2007 workbooks (`.xlsx`, `.xlsm`) and Excel
//! 97-2003 binary workbooks (`.xls`) via calamine, and OpenDocument
//! spreadsheets (`.ods`) via anydoc's document model.
//!
//! Each non-empty worksheet becomes one Markdown page: the sheet name as a
//! heading, then the used range as a pipe table. Output is capped at
//! [`MAX_SHEET_ROWS`] × [`MAX_SHEET_COLS`] per sheet with an explicit elision
//! marker, so a hundred-thousand-row workbook cannot flood distillation.
//! Formula cells surface as their cached values (calamine reads values, not
//! formulas), and empty cells collapse to empty table cells.
//!
//! anydoc's Markdown renders every cell of every sheet into a single page with
//! no row or column cap, so spreadsheets route through this capped renderer
//! instead - an OpenDocument one as anydoc parsed it, before it is rendered.

use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use anydoc::model::{Block, CellSlot, Table, inlines_to_plain_text};
use calamine::{Data, DataRef, Range, Reader, Xls, XlsError, XlsOptions, Xlsx};

use crate::extract::{
    CancellationToken, ExtractedDocument, ExtractedPage, ExtractionError, ExtractionWarning,
    OLE_MAGIC, PageSource, anydoc_document, enforce_office_decompressed_limit, link_sections,
    reject_encrypted_ole,
};
use crate::limits::ResourceLimits;

/// Rows rendered per sheet before elision.
pub const MAX_SHEET_ROWS: usize = 200;
/// Columns rendered per sheet before elision.
pub const MAX_SHEET_COLS: usize = 30;
/// Characters one cell may contribute before it is cut with an ellipsis.
///
/// A real cell is a label, a number or a sentence of notes. Excel lets one
/// hold 32,767 characters, though, and a crafted workbook that repeats one
/// such string across the whole window renders hundreds of megabytes of
/// table; the page and document caps would still cut it, but only after it
/// had been built.
pub const MAX_CELL_CHARS: usize = 1_000;

/// How many cells may pile up before the window is pruned again. Pruning on
/// every cell would be quadratic; pruning when the buffer reaches a few times
/// the rendered window keeps the cost amortised and the memory bounded.
const CELLS_BEFORE_PRUNE: usize = 4 * MAX_SHEET_ROWS * MAX_SHEET_COLS;

/// The most memory opening a binary workbook may have calamine hold at once.
///
/// calamine reads an `.xls` eagerly. Opening it builds every sheet as two
/// dense ranges spanning that sheet's corners - a value per cell, and a
/// formula per cell - and keeps every sheet's ranges until the workbook is
/// dropped; while it builds each one, it also holds a list of that sheet's
/// cells, reserved up front at the size the sheet's `Dimensions` record
/// claims. Those corners and that claim come from the file. Two cells at
/// `A1` and `IV65536`, or one forged `Dimensions` record, would have the
/// allocator asked for gigabytes, and an allocation failure aborts the
/// process rather than failing one document.
///
/// The bound is on those allocations themselves, in bytes: what the sheets
/// already built keep, plus what the one being built holds while it is
/// built. A cap on cells summed across sheets counted each sheet's list as
/// if it were kept, and refused a year of monthly ledgers that opens in
/// under a hundred megabytes. Half a gigabyte opens a full-height sheet a
/// hundred columns wide; one sheet filled out to both of the format's
/// corners, which no real workbook is, does not fit.
const MAX_LEGACY_WORKBOOK_BYTES: u64 = 512 * 1024 * 1024;

/// What calamine holds per cell: a value in a sheet's dense value range, a
/// formula in its dense formula range, and an entry in each of the lists
/// those ranges are built from.
const RANGE_VALUE_BYTES: u64 = size_of::<Data>() as u64;
const RANGE_FORMULA_BYTES: u64 = size_of::<String>() as u64;
const LISTED_VALUE_BYTES: u64 = size_of::<calamine::Cell<Data>>() as u64;
const LISTED_FORMULA_BYTES: u64 = size_of::<calamine::Cell<String>>() as u64;

/// Runs one calamine operation behind a panic barrier: calamine can panic on
/// crafted or corrupt containers, and a dependency panic must degrade to a
/// parse error that routes the document to review. `AssertUnwindSafe` is
/// sound because a caught panic always propagates as an error, so the
/// workbook is never used again.
fn contained<T>(operation: &str, run: impl FnOnce() -> T) -> Result<T, ExtractionError> {
    catch_unwind(AssertUnwindSafe(run)).map_err(|_| {
        ExtractionError::parse_failed(format!("workbook parser aborted during {operation}"))
    })
}

/// The part of a worksheet that will be rendered, plus the corners of the
/// used range the elision marker counts against.
pub(crate) struct CappedSheet {
    /// Absolute `(row, column, text)` of every cell inside the rendered window.
    cells: Vec<(u32, u32, String)>,
    start: (u32, u32),
    end: (u32, u32),
}

/// Collects the cells of one sheet, keeping only those the window can show
/// while tracking the corners of everything it was offered.
#[derive(Default)]
pub(crate) struct WindowBuilder {
    cells: Vec<(u32, u32, String)>,
    start: Option<(u32, u32)>,
    end: (u32, u32),
}

impl WindowBuilder {
    /// Offers one non-empty cell. Its text is only produced when the cell can
    /// still be shown: the top-left corner only ever moves up and to the
    /// left, so a cell outside the window now stays outside it, and a
    /// million-row sheet costs a million comparisons rather than a million
    /// strings.
    pub(crate) fn push(&mut self, row: u32, column: u32, text: impl FnOnce() -> String) {
        let corner = self.start.get_or_insert((row, column));
        corner.0 = corner.0.min(row);
        corner.1 = corner.1.min(column);
        let corner = *corner;
        self.end = (self.end.0.max(row), self.end.1.max(column));
        if in_window(row, column, corner) {
            self.cells.push((row, column, text()));
            if self.cells.len() >= CELLS_BEFORE_PRUNE {
                prune_to_window(&mut self.cells, corner);
            }
        }
    }

    /// The sheet as rendered, or `None` for a sheet with no cell values at
    /// all, which the caller skips.
    pub(crate) fn finish(mut self) -> Option<CappedSheet> {
        let start = self.start?;
        prune_to_window(&mut self.cells, start);
        Some(CappedSheet {
            cells: self.cells,
            start,
            end: self.end,
        })
    }
}

fn in_window(row: u32, column: u32, start: (u32, u32)) -> bool {
    (row - start.0) < MAX_SHEET_ROWS as u32 && (column - start.1) < MAX_SHEET_COLS as u32
}

/// Reads an Excel 2007 workbook - `.xlsx`, or `.xlsm`, whose macros calamine
/// ignores.
pub fn extract_xlsx(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    reject_encrypted_ole(path)?;
    cancel.check()?;
    let metadata = std::fs::metadata(path).map_err(ExtractionError::io)?;
    limits.validate_source_size(metadata.len())?;
    // A binary workbook saved under the newer extension is still a
    // workbook, and the binary reader carries the same window.
    if starts_with(path, &OLE_MAGIC)? {
        return extract_legacy_workbook(path, limits, cancel);
    }
    enforce_office_decompressed_limit(path, limits, cancel)?;
    cancel.check()?;

    let mut workbook: Xlsx<BufReader<File>> = contained("workbook open", || {
        calamine::open_workbook(path)
    })?
    .map_err(|error| ExtractionError::parse_failed(format!("workbook did not open: {error}")))?;
    let sheet_names = contained("sheet listing", || workbook.sheet_names())?;
    limits.validate_page_count(sheet_names.len())?;

    let mut sheets = Vec::new();
    for name in sheet_names {
        cancel.check()?;
        let sheet = contained("worksheet read", || {
            read_capped_sheet(&mut workbook, &name, cancel)
        })??;
        sheets.push((Some(name), sheet));
    }
    workbook_document(sheets)
}

/// Reads an OpenDocument spreadsheet (`.ods`) through the same window as
/// every other workbook.
///
/// anydoc parses it. Its OpenDocument reader charges every repeated row and
/// cell against a fixed expansion budget, which a format built on repeat
/// runs needs: calamine's would materialise a dense range of up to a hundred
/// million cells from a few kilobytes of `number-rows-repeated`. Rendered by
/// anydoc, though, every cell of every sheet went into one page, so a long
/// ledger was cut at the page cap as `TEXT_TRUNCATED` and could never be
/// Ready, where the same ledger saved as `.xlsx`, `.xls` or `.csv` is a
/// window marked `CONTENT_ELIDED`. Each sheet's table is cut to that window
/// here instead, and becomes a page of its own.
///
/// anydoc names the sheets of a workbook with more than one; a lone sheet's
/// page has no heading.
pub fn extract_ods(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    let document = anydoc_document(path, limits, cancel)?;
    let mut sheets = Vec::new();
    let mut name = None;
    for block in &document.blocks {
        match block {
            Block::Heading { content, .. } => name = Some(inlines_to_plain_text(content)),
            Block::Table(table) => {
                cancel.check()?;
                sheets.push((name.take(), table_window(table, cancel)?));
            }
            _ => {}
        }
    }
    limits.validate_page_count(sheets.len())?;
    workbook_document(sheets)
}

/// The window of a sheet anydoc has already read whole.
fn table_window(
    table: &Table,
    cancel: &CancellationToken,
) -> Result<Option<CappedSheet>, ExtractionError> {
    let mut window = WindowBuilder::default();
    for (row, slots) in table.grid.iter().enumerate() {
        if row % 1_024 == 0 {
            cancel.check()?;
        }
        let row = u32::try_from(row).unwrap_or(u32::MAX);
        for (column, slot) in slots.iter().enumerate() {
            // A covered slot is the shadow of a merged cell, whose text
            // belongs to the cell that covers it.
            if let CellSlot::Origin(cell) = slot
                && !cell.is_empty()
            {
                let column = u32::try_from(column).unwrap_or(u32::MAX);
                window.push(row, column, || sanitize_cell(&blocks_text(&cell.blocks)));
            }
        }
    }
    Ok(window.finish())
}

/// The text of an OpenDocument cell: its paragraphs, and anything nested in
/// them, one after another.
fn blocks_text(blocks: &[Block]) -> String {
    fn collect(blocks: &[Block], parts: &mut Vec<String>) {
        for block in blocks {
            match block {
                Block::Paragraph(inlines)
                | Block::Heading {
                    content: inlines, ..
                } => parts.push(inlines_to_plain_text(inlines)),
                Block::List(list) => {
                    for item in &list.items {
                        collect(&item.blocks, parts);
                    }
                }
                Block::BlockQuote(blocks) => collect(blocks, parts),
                Block::CodeBlock { text, .. } => parts.push(text.clone()),
                Block::Table(table) => {
                    for slot in table.grid.iter().flatten() {
                        if let CellSlot::Origin(cell) = slot {
                            collect(&cell.blocks, parts);
                        }
                    }
                }
                Block::Rule => {}
            }
        }
    }
    let mut parts = Vec::new();
    collect(blocks, &mut parts);
    parts.retain(|part| !part.trim().is_empty());
    parts.join(" ")
}

/// Reads an Excel 97-2003 binary workbook (`.xls`).
///
/// calamine's binary reader parses the compound-file container and every
/// sheet when the workbook is opened, sizing its allocations from what the
/// file claims. Neither is left to it: the `Workbook` stream is read out
/// with the `cfb` crate, which validates the container's chains, surveyed
/// for what opening it would allocate, and handed to calamine inside a fresh
/// container written here, so the only compound file calamine ever parses is
/// one with honest headers.
pub fn extract_xls(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    reject_encrypted_ole(path)?;
    cancel.check()?;
    let metadata = std::fs::metadata(path).map_err(ExtractionError::io)?;
    limits.validate_source_size(metadata.len())?;
    // Spreadsheet exports are routinely named `.xls` whatever they are, and
    // an Excel 2007 workbook among them is read as one.
    if starts_with(path, b"PK\x03\x04")? {
        return extract_xlsx(path, limits, cancel);
    }
    extract_legacy_workbook(path, limits, cancel)
}

fn extract_legacy_workbook(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    let mut stream = workbook_stream(path, limits)?;
    blank_formula_expressions(&mut stream)?;
    survey_legacy_workbook(&stream, limits)?;
    cancel.check()?;
    let container = trusted_container(&stream)?;
    drop(stream);

    let mut workbook = contained("workbook open", || {
        Xls::new_with_options(Cursor::new(container), XlsOptions::default())
    })?
    .map_err(legacy_workbook_error)?;
    let sheet_names = contained("sheet listing", || workbook.sheet_names())?;
    limits.validate_page_count(sheet_names.len())?;

    let mut sheets = Vec::new();
    for name in sheet_names {
        cancel.check()?;
        let range = contained("worksheet read", || workbook.worksheet_range(&name))?
            .map_err(legacy_workbook_error)?;
        let sheet = capped_range(&range, cancel)?;
        sheets.push((Some(name), sheet));
    }
    workbook_document(sheets)
}

/// What calamine's binary-workbook errors mean to the person filing it. A
/// workbook saved with a password to open is the one worth naming.
pub(crate) fn legacy_workbook_error(error: XlsError) -> ExtractionError {
    match error {
        XlsError::Password => ExtractionError::encrypted(),
        other => ExtractionError::parse_failed(format!("workbook did not open: {other}")),
    }
}

fn starts_with(path: &Path, magic: &[u8]) -> Result<bool, ExtractionError> {
    let mut head = Vec::with_capacity(magic.len());
    File::open(path)
        .map_err(ExtractionError::io)?
        .take(magic.len() as u64)
        .read_to_end(&mut head)
        .map_err(ExtractionError::io)?;
    Ok(head == magic)
}

/// The BIFF record stream of a binary workbook, read with the `cfb` crate.
fn workbook_stream(path: &Path, limits: &ResourceLimits) -> Result<Vec<u8>, ExtractionError> {
    let file = File::open(path).map_err(ExtractionError::io)?;
    let mut compound = cfb::CompoundFile::open(BufReader::new(file)).map_err(|error| {
        ExtractionError::parse_failed(format!("workbook container did not open: {error}"))
    })?;
    // BIFF8 names the stream `Workbook`, BIFF5 `Book`; compound-file names
    // compare without regard to case.
    let name = ["Workbook", "Book"]
        .into_iter()
        .find(|name| compound.is_stream(name))
        .ok_or_else(|| {
            ExtractionError::parse_failed("workbook container has no Workbook stream")
        })?;
    let mut stream = Vec::new();
    compound
        .open_stream(name)
        .map_err(ExtractionError::io)?
        .take(limits.max_source_bytes + 1)
        .read_to_end(&mut stream)
        .map_err(ExtractionError::io)?;
    limits.validate_source_size(stream.len() as u64)?;
    Ok(stream)
}

/// One BIFF record: its type, where its payload starts in the stream, and
/// the payload. Records whose stated length runs past the end of the stream
/// end the walk; calamine reports those as errors itself.
fn biff_records(stream: &[u8], from: usize) -> impl Iterator<Item = (u16, usize, &[u8])> {
    let mut at = from;
    std::iter::from_fn(move || {
        let header = stream.get(at..at.checked_add(4)?)?;
        let kind = u16::from_le_bytes([header[0], header[1]]);
        let length = usize::from(u16::from_le_bytes([header[2], header[3]]));
        let data = stream.get(at + 4..at + 4 + length)?;
        let record = (kind, at + 4, data);
        at += 4 + length;
        Some(record)
    })
}

const BIFF_EOF: u16 = 0x000A;
const BIFF_FILEPASS: u16 = 0x002F;
const BIFF_CONTINUE: u16 = 0x003C;
const BIFF_BOUNDSHEET: u16 = 0x0085;
const BIFF_SST: u16 = 0x00FC;
const BIFF_LABELSST: u16 = 0x00FD;
const BIFF_DIMENSIONS: u16 = 0x0200;
const BIFF_STRING: u16 = 0x0207;
const BIFF_MULRK: u16 = 0x00BD;
const BIFF_FORMULA: u16 = 0x0006;
/// Every record type calamine turns into a cell, each of which starts with
/// the cell's row and column as two 16-bit words.
const BIFF_CELLS: [u16; 7] = [
    0x0203,
    0x0204,
    0x00D6,
    0x0205,
    0x027E,
    BIFF_LABELSST,
    BIFF_FORMULA,
];

/// The most shared-string text a binary workbook's cells may make calamine
/// copy out, in characters.
///
/// calamine gives every cell that refers to a shared string its own copy of
/// it. A 14-byte cell record can name a string of tens of thousands of
/// characters, so a workbook a few megabytes long could otherwise have
/// calamine copy out tens of gigabytes as it opens. Thirty-two million
/// characters is a full-height sheet of long text in every column, at a
/// cost of a hundred or two megabytes.
const MAX_LEGACY_WORKBOOK_TEXT: u64 = 32_000_000;

fn word(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from(u16::from_le_bytes(
        data.get(at..at + 2)?.try_into().ok()?,
    )))
}

fn long(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// What the global records up to the first `EOF` say about the workbook:
/// where each sheet's records start, and how long each shared string is.
///
/// A `FilePass` record means everything after it is encrypted: positions
/// read from there on are noise, and the workbook needs its password.
struct Globals {
    sheet_offsets: Vec<usize>,
    shared_string_lengths: Vec<u16>,
}

fn read_globals(stream: &[u8]) -> Result<Globals, ExtractionError> {
    let mut globals = Globals {
        sheet_offsets: Vec::new(),
        shared_string_lengths: Vec::new(),
    };
    let mut records = biff_records(stream, 0).peekable();
    while let Some((kind, _, data)) = records.next() {
        match kind {
            BIFF_FILEPASS => return Err(ExtractionError::encrypted()),
            BIFF_BOUNDSHEET => {
                if let Some(offset) = long(data, 0) {
                    globals.sheet_offsets.push(offset as usize);
                }
            }
            BIFF_SST => {
                // The table runs on through the `Continue` records that
                // follow it, as calamine reads it.
                let mut segments = vec![data];
                while let Some((_, _, more)) =
                    records.next_if(|(kind, _, _)| *kind == BIFF_CONTINUE)
                {
                    segments.push(more);
                }
                globals.shared_string_lengths = shared_string_lengths(&segments);
            }
            BIFF_EOF => break,
            _ => {}
        }
    }
    Ok(globals)
}

/// The character count of every string in a shared string table, found by
/// walking it exactly as calamine does: a count and flags per string, its
/// characters one or two bytes wide, a fresh width flag wherever the
/// characters cross into a `Continue` record, then formatting runs and
/// phonetic data skipped. Where calamine would fail, the walk stops; the
/// workbook then does not open, and the lengths already found are enough.
fn shared_string_lengths<'a>(segments: &[&'a [u8]]) -> Vec<u16> {
    let mut lengths = Vec::new();
    let Some((first, mut rest)) = segments.split_first() else {
        return lengths;
    };
    let Some(mut data) = first.get(8..) else {
        return lengths;
    };
    let mut next_segment = |data: &mut &'a [u8]| match rest.split_first() {
        Some((segment, remaining)) => {
            *data = segment;
            rest = remaining;
            true
        }
        None => false,
    };
    loop {
        if data.is_empty() && !next_segment(&mut data) {
            return lengths;
        }
        // An empty `Continue` record is an empty string to calamine.
        if data.is_empty() {
            lengths.push(0);
            continue;
        }
        let Some(&[low, high, flags]) = data.get(..3) else {
            return lengths;
        };
        data = &data[3..];
        let characters = u16::from_le_bytes([low, high]);
        let mut skip = 0_usize;
        if flags & 0x8 != 0 {
            let Some(runs) = word(data, 0) else {
                return lengths;
            };
            skip += 4 * runs as usize;
            data = &data[2..];
        }
        if flags & 0x4 != 0 {
            let Some(phonetic) = long(data, 0) else {
                return lengths;
            };
            skip = skip.saturating_add(phonetic as usize);
            data = &data[4..];
        }
        let mut wide = flags & 0x1 != 0;
        let mut remaining = usize::from(characters);
        while remaining > 0 {
            let width = if wide { 2 } else { 1 };
            let taken = remaining.min(data.len() / width);
            data = &data[taken * width..];
            remaining -= taken;
            if remaining > 0 {
                if !next_segment(&mut data) || data.is_empty() {
                    lengths.push(characters);
                    return lengths;
                }
                wide = data[0] & 0x1 != 0;
                data = &data[1..];
            }
        }
        lengths.push(characters);
        while skip > 0 {
            if data.is_empty() && !next_segment(&mut data) {
                return lengths;
            }
            let skipped = skip.min(data.len());
            data = &data[skipped..];
            skip -= skipped;
        }
    }
}

/// Where each sheet's formulas keep a token stream, found by walking every
/// sheet as calamine will: the offset of each nonzero expression length.
fn formula_expressions(stream: &[u8]) -> Result<Vec<usize>, ExtractionError> {
    let mut found = Vec::new();
    for offset in read_globals(stream)?.sheet_offsets {
        for (kind, at, data) in biff_records(stream, offset) {
            match kind {
                BIFF_FORMULA if data.get(20..22).is_some_and(|length| length != [0, 0]) => {
                    found.push(at + 20);
                }
                BIFF_EOF => break,
                _ => {}
            }
        }
    }
    Ok(found)
}

/// Empties every formula's token stream, leaving its cached value.
///
/// Only cached values are rendered, but calamine turns every formula into
/// text as the workbook opens, and that text can be far longer than its
/// record: a five-byte reference to a defined name spells out the whole
/// name, so a few megabytes of formulas can become gigabytes of strings.
/// The expression length sits inside each record, so emptying it moves no
/// record and no sheet offset.
///
/// A second walk proves no expression is left. Real sheets never share
/// records, but a crafted workbook can point two sheets into the same bytes
/// at different record boundaries, where emptying one sheet's formulas
/// could reveal another's.
fn blank_formula_expressions(stream: &mut [u8]) -> Result<(), ExtractionError> {
    for at in formula_expressions(stream)? {
        stream[at..at + 2].fill(0);
    }
    if formula_expressions(stream)?.is_empty() {
        Ok(())
    } else {
        Err(ExtractionError::parse_failed(
            "workbook sheets overlap one another",
        ))
    }
}

/// Walks the workbook stream the way calamine's reader will, and refuses it
/// before calamine allocates anything the window could never show.
///
/// The walk mirrors calamine's own: the global records up to the first
/// `EOF`, then each sheet from the offset its `BoundSheet8` record gives,
/// to that sheet's `EOF`. Offsets are followed rather than the stream read
/// once, because nothing stops two sheet records pointing at the same
/// records, and calamine would build that sheet twice.
fn survey_legacy_workbook(stream: &[u8], limits: &ResourceLimits) -> Result<(), ExtractionError> {
    let globals = read_globals(stream)?;
    limits.validate_page_count(globals.sheet_offsets.len())?;

    let too_large = || {
        ExtractionError::resource_limit(format!(
            "workbook would take more than {} MiB to open",
            MAX_LEGACY_WORKBOOK_BYTES / (1024 * 1024)
        ))
    };
    // The ranges of the sheets built so far, which calamine keeps.
    let mut kept = 0_u64;
    // The largest sheet's value range: reading a sheet back copies it.
    let mut largest_values = 0_u64;
    let mut text = 0_u64;
    for offset in globals.sheet_offsets {
        // Values and formulas become two dense ranges, each over its own
        // corners.
        let mut values = Corners::default();
        let mut formulas = Corners::default();
        // The room a Dimensions record has the cell list reserve: its claim
        // on top of the cells already listed.
        let mut reserved = 0_u64;
        let mut cells = 0_u64;
        let mut formula_cells = 0_u64;
        // A formula's string result follows it in a record of its own, and
        // calamine places it wherever the last formula was - the top-left
        // cell, if none has been.
        let mut formula_at = (0, 0);
        for (kind, _, data) in biff_records(stream, offset) {
            match kind {
                BIFF_DIMENSIONS => {
                    reserved = reserved.max(cells.saturating_add(claimed_cells(data)));
                }
                BIFF_MULRK => {
                    if let (Some(row), Some(first)) = (word(data, 0), word(data, 2))
                        && let Some(last) = data.len().checked_sub(2).and_then(|at| word(data, at))
                    {
                        values.extend(row, first);
                        values.extend(row, last);
                        cells += u64::from(last.saturating_sub(first)) + 1;
                    }
                }
                BIFF_STRING => {
                    values.extend(formula_at.0, formula_at.1);
                    cells += 1;
                }
                kind if BIFF_CELLS.contains(&kind) => {
                    if let (Some(row), Some(column)) = (word(data, 0), word(data, 2)) {
                        values.extend(row, column);
                        if kind == BIFF_FORMULA {
                            formulas.extend(row, column);
                            formula_at = (row, column);
                            formula_cells += 1;
                        }
                        cells += 1;
                    }
                    if kind == BIFF_LABELSST
                        && let Some(index) = long(data, 6)
                        && let Some(length) = globals.shared_string_lengths.get(index as usize)
                    {
                        text += u64::from(*length);
                    }
                }
                BIFF_EOF => break,
                _ => {}
            }
        }
        // The lists start at whatever room was reserved and double whenever
        // the cells outgrow it; the formula list is never reserved at all.
        let listed_values = if cells > reserved {
            cells.saturating_mul(2)
        } else {
            reserved
        };
        let listed = listed_values
            .saturating_mul(LISTED_VALUE_BYTES)
            .saturating_add(
                formula_cells
                    .saturating_mul(2)
                    .saturating_mul(LISTED_FORMULA_BYTES),
            );
        let value_range = values.area().saturating_mul(RANGE_VALUE_BYTES);
        let ranges =
            value_range.saturating_add(formulas.area().saturating_mul(RANGE_FORMULA_BYTES));
        let building = kept.saturating_add(listed).saturating_add(ranges);
        kept = kept.saturating_add(ranges);
        largest_values = largest_values.max(value_range);
        if building > MAX_LEGACY_WORKBOOK_BYTES
            || kept.saturating_add(largest_values) > MAX_LEGACY_WORKBOOK_BYTES
        {
            return Err(too_large());
        }
        if text > MAX_LEGACY_WORKBOOK_TEXT {
            return Err(ExtractionError::resource_limit(format!(
                "workbook would copy out more than {MAX_LEGACY_WORKBOOK_TEXT} characters of text"
            )));
        }
    }
    Ok(())
}

/// How many cells a `Dimensions` record makes calamine reserve room for,
/// computed as calamine computes it - including the 32-bit subtraction that,
/// in a release build, wraps a last row before the first round to four
/// billion rows rather than failing.
fn claimed_cells(data: &[u8]) -> u64 {
    let (first_row, last_row, mut first_column, last_column) = match data.len() {
        10 => (
            word(data, 0).unwrap_or(0),
            word(data, 2).unwrap_or(0),
            word(data, 4).unwrap_or(0),
            word(data, 6).unwrap_or(0),
        ),
        14 => (
            long(data, 0).unwrap_or(0),
            long(data, 4).unwrap_or(0),
            word(data, 8).unwrap_or(0),
            word(data, 10).unwrap_or(0),
        ),
        _ => return 0,
    };
    if first_column > 0xFF || last_column < first_column {
        first_column = 0;
    }
    if last_row == 0 || last_column == 0 {
        return 1;
    }
    let span = |first: u32, last: u32| u64::from((last - 1).wrapping_sub(first).wrapping_add(1));
    span(first_row, last_row).saturating_mul(span(first_column, last_column))
}

/// The top-left and bottom-right corners of a set of cells.
#[derive(Default)]
struct Corners(Option<((u32, u32), (u32, u32))>);

impl Corners {
    fn extend(&mut self, row: u32, column: u32) {
        let (start, end) = self.0.get_or_insert(((row, column), (row, column)));
        *start = (start.0.min(row), start.1.min(column));
        *end = (end.0.max(row), end.1.max(column));
    }

    /// Cells in the dense range calamine builds over these corners.
    fn area(&self) -> u64 {
        self.0.map_or(0, |(start, end)| {
            u64::from(end.0 - start.0 + 1) * u64::from(end.1 - start.1 + 1)
        })
    }
}

/// A compound file holding nothing but the given `Workbook` stream, written
/// by the `cfb` crate - the container calamine actually parses.
fn trusted_container(stream: &[u8]) -> Result<Vec<u8>, ExtractionError> {
    let rewrite = |error: std::io::Error| {
        ExtractionError::parse_failed(format!("workbook could not be re-containered: {error}"))
    };
    let mut compound =
        cfb::CompoundFile::create_with_version(cfb::Version::V3, Cursor::new(Vec::new()))
            .map_err(rewrite)?;
    compound
        .create_stream("/Workbook")
        .and_then(|mut target| target.write_all(stream))
        .map_err(rewrite)?;
    compound.flush().map_err(rewrite)?;
    Ok(compound.into_inner().into_inner())
}

/// The window of a sheet calamine has already read whole.
fn capped_range(
    range: &Range<Data>,
    cancel: &CancellationToken,
) -> Result<Option<CappedSheet>, ExtractionError> {
    let (top, left) = range.start().unwrap_or((0, 0));
    let mut window = WindowBuilder::default();
    for (seen, (row, column, value)) in range.used_cells().enumerate() {
        if seen % 1_024 == 0 {
            cancel.check()?;
        }
        window.push(top + row as u32, left + column as u32, || cell_text(value));
    }
    Ok(window.finish())
}

/// Renders each sheet that has any cells as a page, numbering pages in the
/// order the workbook lists its sheets.
fn workbook_document(
    sheets: Vec<(Option<String>, Option<CappedSheet>)>,
) -> Result<ExtractedDocument, ExtractionError> {
    let mut pages = Vec::new();
    let mut elided = false;
    for (name, sheet) in sheets {
        let Some(sheet) = sheet else {
            continue;
        };
        let (text, sheet_elided) = render_sheet(name.as_deref(), &sheet);
        elided |= sheet_elided;
        pages.push(ExtractedPage::of_text(
            pages.len() + 1,
            text,
            PageSource::AnyDoc,
        ));
    }
    if pages.is_empty() {
        return Err(ExtractionError::unsupported(
            "workbook contains no readable cells",
        ));
    }
    Ok(elided_document(pages, elided))
}

/// A document whose only loss, if any, is the rows and columns past the
/// window. That is not truncation: the reader chose to leave them out and
/// marked where it did, so the document is still whole as far as anything
/// downstream can tell, and `truncated` stays false.
pub(crate) fn elided_document(mut pages: Vec<ExtractedPage>, elided: bool) -> ExtractedDocument {
    link_sections(&mut pages);
    ExtractedDocument {
        pages,
        warnings: if elided {
            vec![ExtractionWarning::ContentElided]
        } else {
            vec![]
        },
        truncated: false,
        optional_image: None,
        timings: None,
    }
}

/// Streams one worksheet's cells and keeps only the ones the cap can render.
///
/// `worksheet_range` materialises the used range densely — one slot per cell
/// of the bounding box — before any cap of ours applies. A three-kilobyte
/// workbook whose only two cells are `A1` and `XFD1048576` therefore asks the
/// allocator for half a terabyte and aborts the process, which no panic
/// barrier can turn back into a parse error. Reading cell by cell keeps at
/// most a few thousand of them, whatever the sheet claims its corners are.
///
/// `None` is a sheet with no cell values at all, which the caller skips.
fn read_capped_sheet(
    workbook: &mut Xlsx<BufReader<File>>,
    name: &str,
    cancel: &CancellationToken,
) -> Result<Option<CappedSheet>, ExtractionError> {
    let read_error = |error: calamine::XlsxError| {
        ExtractionError::parse_failed(format!("worksheet {name:?} did not read: {error}"))
    };
    let mut reader = workbook.worksheet_cells_reader(name).map_err(read_error)?;
    let mut window = WindowBuilder::default();
    let mut seen = 0_u64;
    while let Some(cell) = reader.next_cell().map_err(read_error)? {
        if seen % 1_024 == 0 {
            cancel.check()?;
        }
        seen += 1;
        if matches!(cell.get_value(), DataRef::Empty) {
            continue;
        }
        let (row, column) = cell.get_position();
        window.push(row, column, || cell_text(&cell.get_value().clone().into()));
    }
    Ok(window.finish())
}

/// Drops every cell outside the rows and columns the renderer can show. A
/// cell is unreachable once it sits more than a windowful past the top-left
/// corner, and that corner only ever moves up and to the left.
fn prune_to_window(cells: &mut Vec<(u32, u32, String)>, start: (u32, u32)) {
    cells.retain(|(row, column, _)| in_window(*row, *column, start));
}

/// Renders one sheet as a pipe table - under `## name` when it has one -
/// returning the text and whether any rows or columns were elided.
///
/// Rows and columns past the last cell inside the window count as elided
/// rather than being drawn as empty table cells: a sheet whose used range
/// runs out to one stray far-away cell would otherwise render two hundred
/// blank rows before admitting it left anything out.
pub(crate) fn render_sheet(name: Option<&str>, sheet: &CappedSheet) -> (String, bool) {
    let height = u64::from(sheet.end.0 - sheet.start.0) + 1;
    let width = u64::from(sheet.end.1 - sheet.start.1) + 1;
    let shown_rows = sheet
        .cells
        .iter()
        .map(|(row, _, _)| (row - sheet.start.0) as usize + 1)
        .max()
        .unwrap_or(0);
    let shown_cols = sheet
        .cells
        .iter()
        .map(|(_, column, _)| (column - sheet.start.1) as usize + 1)
        .max()
        .unwrap_or(0);
    let mut grid = vec![vec![String::new(); shown_cols]; shown_rows];
    for (row, column, value) in &sheet.cells {
        grid[(row - sheet.start.0) as usize][(column - sheet.start.1) as usize] = value.clone();
    }

    let mut text = match name {
        Some(name) => format!("## {}\n\n", sanitize_cell(name)),
        None => String::new(),
    };
    for (index, row) in grid.iter().enumerate() {
        text.push_str(&format!("| {} |\n", row.join(" | ")));
        if index == 0 {
            text.push_str(&format!("| {} |\n", vec!["---"; shown_cols].join(" | ")));
        }
    }

    let hidden_rows = height - shown_rows as u64;
    let hidden_cols = width - shown_cols as u64;
    let elided = hidden_rows > 0 || hidden_cols > 0;
    if elided {
        let marker = match (hidden_rows, hidden_cols) {
            (rows, 0) => format!("[... {rows} more rows not shown]"),
            (0, cols) => format!("[... {cols} more columns not shown]"),
            (rows, cols) => format!("[... {rows} more rows and {cols} more columns not shown]"),
        };
        text.push('\n');
        text.push_str(&marker);
        text.push('\n');
    }
    (text, elided)
}

fn cell_text(data: &Data) -> String {
    match data {
        Data::Empty => String::new(),
        Data::String(value) | Data::DateTimeIso(value) | Data::DurationIso(value) => {
            sanitize_cell(value)
        }
        Data::Int(value) => value.to_string(),
        Data::Float(value) => value.to_string(),
        Data::Bool(value) => if *value { "TRUE" } else { "FALSE" }.to_owned(),
        Data::DateTime(value) => {
            if value.is_datetime() {
                let (year, month, day, hour, minute, second, _) = value.to_ymd_hms_milli();
                if hour == 0 && minute == 0 && second == 0 {
                    format!("{year:04}-{month:02}-{day:02}")
                } else {
                    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
                }
            } else {
                let total_seconds = (value.as_f64() * 86_400.0).round() as i64;
                format!(
                    "{}:{:02}:{:02}",
                    total_seconds / 3_600,
                    total_seconds % 3_600 / 60,
                    total_seconds % 60
                )
            }
        }
        Data::Error(error) => error.to_string(),
    }
}

/// Keeps a value on one table line: newlines become spaces and pipes are
/// escaped so a cell cannot break the row it sits in, and a value past
/// [`MAX_CELL_CHARS`] is cut with an ellipsis.
pub(crate) fn sanitize_cell(value: &str) -> String {
    let value = match value.char_indices().nth(MAX_CELL_CHARS) {
        Some((end, _)) => format!("{}…", &value[..end]),
        None => value.to_owned(),
    };
    value
        .replace(['\r', '\n'], " ")
        .replace('|', "\\|")
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{ExcelDateTime, ExcelDateTimeType};

    #[test]
    fn cell_values_render_deterministically_for_every_data_variant() {
        assert_eq!(cell_text(&Data::Empty), "");
        assert_eq!(cell_text(&Data::String("a | b\nc".to_owned())), "a \\| b c");
        assert_eq!(cell_text(&Data::Int(-3)), "-3");
        assert_eq!(cell_text(&Data::Float(1.5)), "1.5");
        assert_eq!(cell_text(&Data::Bool(true)), "TRUE");
    }

    #[test]
    fn excel_serial_datetimes_render_as_iso_dates_and_times() {
        let date = ExcelDateTime::new(45_943.0, ExcelDateTimeType::DateTime, false);
        assert_eq!(cell_text(&Data::DateTime(date)), "2025-10-13");
        let datetime = ExcelDateTime::new(45_943.5, ExcelDateTimeType::DateTime, false);
        assert_eq!(cell_text(&Data::DateTime(datetime)), "2025-10-13 12:00:00");
        let duration = ExcelDateTime::new(1.25, ExcelDateTimeType::TimeDelta, false);
        assert_eq!(cell_text(&Data::DateTime(duration)), "30:00:00");
    }

    /// A cell may hold 32,767 characters, and a crafted sheet can repeat one
    /// across its whole window. Each cell is cut where a real one never
    /// reaches, and says so.
    #[test]
    fn a_cell_longer_than_the_cap_is_cut_with_an_ellipsis() {
        let long = "é".repeat(MAX_CELL_CHARS + 500);
        let rendered = cell_text(&Data::String(long));
        assert_eq!(rendered.chars().count(), MAX_CELL_CHARS + 1);
        assert!(rendered.ends_with('…'), "{rendered}");

        let exact = "a".repeat(MAX_CELL_CHARS);
        assert_eq!(cell_text(&Data::String(exact.clone())), exact);
    }

    #[test]
    fn a_password_on_a_binary_workbook_is_reported_as_one() {
        let error = legacy_workbook_error(XlsError::Password);
        assert_eq!(error.code(), "PASSWORD_PROTECTED");
        assert!(!error.retryable());

        let other = legacy_workbook_error(XlsError::StackLen);
        assert_eq!(other.code(), "PARSE_FAILED");
    }

    fn record(kind: u16, data: &[u8]) -> Vec<u8> {
        let mut bytes = kind.to_le_bytes().to_vec();
        bytes.extend_from_slice(&(data.len() as u16).to_le_bytes());
        bytes.extend_from_slice(data);
        bytes
    }

    fn number_cell(row: u16, column: u16) -> Vec<u8> {
        let mut data = row.to_le_bytes().to_vec();
        data.extend_from_slice(&column.to_le_bytes());
        data.extend_from_slice(&0_u16.to_le_bytes());
        data.extend_from_slice(&1.0_f64.to_le_bytes());
        record(0x0203, &data)
    }

    /// A global substream whose one sheet starts right after it, followed by
    /// that sheet's records.
    fn workbook(sheet: &[Vec<u8>]) -> Vec<u8> {
        workbook_with(&[], sheet)
    }

    /// [`workbook`], with further global records - a shared string table -
    /// after the sheet's `BoundSheet8`.
    fn workbook_with(globals: &[Vec<u8>], sheet: &[Vec<u8>]) -> Vec<u8> {
        let bof = record(0x0809, &[0; 16]);
        let eof = record(BIFF_EOF, &[]);
        let bound_sheet_length = 4 + 8 + 2;
        let globals_length = globals.iter().map(Vec::len).sum::<usize>();
        let sheet_offset = (bof.len() + bound_sheet_length + globals_length + eof.len()) as u32;
        let mut bound_sheet = sheet_offset.to_le_bytes().to_vec();
        bound_sheet.extend_from_slice(&[0, 0, 1, 0, b'S', 0]);
        let mut stream = bof.clone();
        stream.extend(record(BIFF_BOUNDSHEET, &bound_sheet));
        for record in globals {
            stream.extend(record);
        }
        stream.extend(&eof);
        stream.extend(&bof);
        for record in sheet {
            stream.extend(record);
        }
        stream.extend(eof);
        stream
    }

    /// A global substream listing every sheet, followed by each sheet's
    /// records in turn.
    fn workbook_of(sheets: &[Vec<Vec<u8>>]) -> Vec<u8> {
        let bof = record(0x0809, &[0; 16]);
        let eof = record(BIFF_EOF, &[]);
        let bound_sheet_length = 4 + 8 + 2;
        let mut offset = bof.len() + sheets.len() * bound_sheet_length + eof.len();
        let mut stream = bof.clone();
        let mut bodies = Vec::new();
        for sheet in sheets {
            let mut bound_sheet = (offset as u32).to_le_bytes().to_vec();
            bound_sheet.extend_from_slice(&[0, 0, 1, 0, b'S', 0]);
            stream.extend(record(BIFF_BOUNDSHEET, &bound_sheet));
            let mut body = bof.clone();
            for record in sheet {
                body.extend(record);
            }
            body.extend(&eof);
            offset += body.len();
            bodies.push(body);
        }
        stream.extend(&eof);
        for body in bodies {
            stream.extend(body);
        }
        stream
    }

    /// A `MulRk` record: a number in each of the first `columns` cells of
    /// `row`, which is how Excel writes a row of numbers.
    fn number_run(row: u16, columns: u16) -> Vec<u8> {
        let mut data = row.to_le_bytes().to_vec();
        data.extend_from_slice(&0_u16.to_le_bytes());
        for _ in 0..columns {
            data.extend_from_slice(&0_u16.to_le_bytes());
            data.extend_from_slice(&2_u32.to_le_bytes());
        }
        data.extend_from_slice(&(columns - 1).to_le_bytes());
        record(BIFF_MULRK, &data)
    }

    /// An honest BIFF8 `Dimensions` record for `rows` by `columns` cells.
    fn dimensions(rows: u32, columns: u16) -> Vec<u8> {
        let mut data = 0_u32.to_le_bytes().to_vec();
        data.extend_from_slice(&rows.to_le_bytes());
        data.extend_from_slice(&0_u16.to_le_bytes());
        data.extend_from_slice(&columns.to_le_bytes());
        data.extend_from_slice(&0_u16.to_le_bytes());
        record(BIFF_DIMENSIONS, &data)
    }

    /// A formula cell at `row`, `column` with a cached number and the given
    /// token stream.
    fn formula(row: u16, column: u16, expression: &[u8]) -> Vec<u8> {
        let mut data = row.to_le_bytes().to_vec();
        data.extend_from_slice(&column.to_le_bytes());
        data.extend_from_slice(&[0; 2]);
        data.extend_from_slice(&1.0_f64.to_le_bytes());
        data.extend_from_slice(&[0; 6]);
        data.extend_from_slice(&(expression.len() as u16).to_le_bytes());
        data.extend_from_slice(expression);
        record(BIFF_FORMULA, &data)
    }

    /// A string result arrives in a record of its own after its formula,
    /// and calamine puts it wherever the last formula was: the top-left
    /// cell when there was none. One such record and one number in the far
    /// corner make the same sixteen-million-cell range two numbers would.
    #[test]
    fn a_string_result_lands_where_calamine_puts_it() {
        let string = record(BIFF_STRING, &[1, 0, 0, b'x']);
        let stray = workbook(&[number_cell(65_535, 255), string.clone()]);
        let error = survey_legacy_workbook(&stray, &ResourceLimits::default()).unwrap_err();
        assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");

        let after_its_formula =
            workbook(&[number_cell(65_535, 255), formula(65_535, 254, &[]), string]);
        survey_legacy_workbook(&after_its_formula, &ResourceLimits::default()).unwrap();
    }

    /// One shared string, `length` single-byte characters long.
    fn shared_string_table(length: u16) -> Vec<u8> {
        let mut data = vec![0; 8];
        data.extend_from_slice(&length.to_le_bytes());
        data.push(0);
        data.extend(std::iter::repeat_n(b'a', usize::from(length)));
        record(BIFF_SST, &data)
    }

    fn shared_string_cell(row: u16, index: u32) -> Vec<u8> {
        let mut data = row.to_le_bytes().to_vec();
        data.extend_from_slice(&[0; 4]);
        data.extend_from_slice(&index.to_le_bytes());
        record(BIFF_LABELSST, &data)
    }

    /// calamine gives every cell its own copy of the shared string it names.
    /// Four thousand fourteen-byte records naming one eight-thousand
    /// character string are thirty-two million characters to copy.
    #[test]
    fn shared_string_text_copied_into_cells_is_bounded() {
        let cells = |count: u16| {
            (0..count)
                .map(|row| shared_string_cell(row, 0))
                .collect::<Vec<_>>()
        };
        let table = shared_string_table(8_000);

        let amplified = workbook_with(std::slice::from_ref(&table), &cells(4_001));
        let error = survey_legacy_workbook(&amplified, &ResourceLimits::default()).unwrap_err();
        assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");

        let ordinary = workbook_with(&[table], &cells(3_999));
        survey_legacy_workbook(&ordinary, &ResourceLimits::default()).unwrap();
    }

    /// The table is walked as calamine walks it, or an index would name the
    /// wrong string: characters that change width where they cross into a
    /// `Continue` record, formatting runs and phonetic data skipped, and an
    /// empty `Continue` record read as an empty string.
    #[test]
    fn shared_string_lengths_follow_calamines_walk() {
        let mut first = vec![0; 8];
        first.extend_from_slice(&[3, 0, 0]);
        first.extend_from_slice(b"abc");
        // Four wide characters, two of them in this record.
        first.extend_from_slice(&[4, 0, 1]);
        first.extend_from_slice(&[b'a', 0, b'b', 0]);
        // The rest narrow, then a string with one formatting run.
        let mut second = vec![0];
        second.extend_from_slice(b"cd");
        second.extend_from_slice(&[1, 0, 0x8, 1, 0, b'z', 0, 0, 0, 0]);
        // A string with three bytes of phonetic data.
        let mut fourth = vec![2, 0, 0x4];
        fourth.extend_from_slice(&3_u32.to_le_bytes());
        fourth.extend_from_slice(b"ok");
        fourth.extend_from_slice(&[9, 9, 9]);

        assert_eq!(
            shared_string_lengths(&[&first, &second, &[], &fourth]),
            vec![3, 4, 1, 0, 2]
        );
    }

    /// Only cached values are rendered, so a formula's tokens are emptied
    /// before calamine spells them out - and nothing else in the stream
    /// changes, so every record and sheet offset stays where it was.
    #[test]
    fn formula_expressions_are_emptied_and_nothing_else_moves() {
        let original = workbook(&[formula(1, 1, &[0x1E, 1, 0]), number_cell(2, 2)]);
        let found = formula_expressions(&original).unwrap();
        assert_eq!(found.len(), 1);
        let at = found[0];

        let mut stream = original.clone();
        blank_formula_expressions(&mut stream).unwrap();

        assert_eq!(&stream[at..at + 2], &[0, 0]);
        assert_eq!(stream[..at], original[..at]);
        assert_eq!(stream[at + 2..], original[at + 2..]);
        assert!(formula_expressions(&stream).unwrap().is_empty());
    }

    /// The committed binary workbook carries a real formula, so the fixture
    /// tests show its cached value survives the blanking.
    #[test]
    fn the_committed_binary_workbook_has_a_formula_to_blank() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/formats/ledger.xls");
        let stream = workbook_stream(&path, &ResourceLimits::default()).unwrap();
        assert_eq!(formula_expressions(&stream).unwrap().len(), 1);
    }

    /// Sheet B starts two bytes before sheet A's expression length, so that
    /// length is B's first record's length. Emptying A's expression shortens
    /// that record to nothing, and B then reads a formula hidden inside A's
    /// record that neither walk saw before. No real workbook shares records
    /// between sheets; this one is refused.
    #[test]
    fn sheets_sharing_bytes_at_different_boundaries_are_refused() {
        let mut stream = record(0x0809, &[0; 16]);
        let sheet_a = 52_u32;
        let sheet_b = sheet_a + 4 + 18;
        for offset in [sheet_a, sheet_b] {
            let mut bound_sheet = offset.to_le_bytes().to_vec();
            bound_sheet.extend_from_slice(&[0, 0, 1, 0, b'S', 0]);
            stream.extend(record(BIFF_BOUNDSHEET, &bound_sheet));
        }
        stream.extend(record(BIFF_EOF, &[]));
        assert_eq!(stream.len(), sheet_a as usize);
        let mut hidden = vec![0; 22];
        hidden[20] = 5;
        let mut payload = vec![0; 18];
        payload.extend_from_slice(&0x0099_u16.to_le_bytes());
        payload.extend_from_slice(&2_u16.to_le_bytes());
        payload.extend(record(BIFF_FORMULA, &hidden));
        payload.resize(60, 0);
        stream.extend(record(BIFF_FORMULA, &payload));
        stream.extend(record(BIFF_EOF, &[]));

        assert_eq!(
            formula_expressions(&stream).unwrap(),
            vec![sheet_a as usize + 4 + 20]
        );
        let error = blank_formula_expressions(&mut stream).unwrap_err();
        assert_eq!(error.code(), "PARSE_FAILED");
    }

    /// Two numbers at opposite corners of a binary sheet make calamine build
    /// a dense range of four billion cells as the workbook opens, which no
    /// panic barrier survives. The survey refuses the file first.
    #[test]
    fn opposite_corner_cells_in_a_binary_sheet_are_refused_before_calamine_opens_it() {
        let stream = workbook(&[number_cell(0, 0), number_cell(65_535, 65_535)]);
        let error = survey_legacy_workbook(&stream, &ResourceLimits::default()).unwrap_err();
        assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");

        let ordinary = workbook(&[number_cell(0, 0), number_cell(5_000, 20)]);
        survey_legacy_workbook(&ordinary, &ResourceLimits::default()).unwrap();
    }

    /// Ordinary workbooks far larger than the window open: a year of monthly
    /// ledgers, a few wide sheets with a total column of formulas, and one
    /// full-height ledger. The budget is what calamine holds at once, and
    /// summing every sheet's cell list as if it were kept refused all three.
    /// A full-height sheet a hundred columns wide, near the budget, still
    /// fits.
    #[test]
    fn large_ordinary_workbooks_open() {
        for (sheets, rows, columns) in [
            (12, 5_000, 40),
            (6, 10_000, 35),
            (1, 65_536, 30),
            (1, 65_536, 99),
        ] {
            let mut sheet = vec![dimensions(rows, columns + 1)];
            for row in 0..rows {
                let row = row as u16;
                sheet.push(number_run(row, columns));
                sheet.push(formula(row, columns, &[]));
            }
            let stream = workbook_of(&vec![sheet; sheets]);
            survey_legacy_workbook(&stream, &ResourceLimits::default())
                .unwrap_or_else(|error| panic!("{sheets} sheets of {rows} x {columns}: {error}"));
        }
    }

    /// calamine keeps every sheet it has built until the workbook is
    /// dropped, so sheets that each fit can still not fit together.
    #[test]
    fn the_sheets_calamine_keeps_add_up() {
        // Two numbers at opposite corners of 65,536 x 100 cells: a dense
        // range of a little over 200 MB.
        let sparse = vec![number_cell(0, 0), number_cell(65_535, 99)];
        survey_legacy_workbook(
            &workbook_of(std::slice::from_ref(&sparse)),
            &ResourceLimits::default(),
        )
        .unwrap();

        let error =
            survey_legacy_workbook(&workbook_of(&vec![sparse; 3]), &ResourceLimits::default())
                .unwrap_err();
        assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
    }

    /// A `Dimensions` record is a claim, and calamine reserves room for it
    /// before reading a single cell.
    #[test]
    fn a_forged_dimensions_claim_is_refused() {
        let mut claim = 0_u32.to_le_bytes().to_vec();
        claim.extend_from_slice(&u32::MAX.to_le_bytes());
        claim.extend_from_slice(&0_u16.to_le_bytes());
        claim.extend_from_slice(&200_u16.to_le_bytes());
        claim.extend_from_slice(&0_u16.to_le_bytes());
        let stream = workbook(&[record(BIFF_DIMENSIONS, &claim), number_cell(0, 0)]);
        let error = survey_legacy_workbook(&stream, &ResourceLimits::default()).unwrap_err();
        assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
    }

    #[test]
    fn a_filepass_record_is_a_password_not_a_parse_failure() {
        let mut stream = record(0x0809, &[0; 16]);
        stream.extend(record(BIFF_FILEPASS, &[1, 0, 1, 0]));
        stream.extend(record(BIFF_EOF, &[]));
        let error = survey_legacy_workbook(&stream, &ResourceLimits::default()).unwrap_err();
        assert_eq!(error.code(), "PASSWORD_PROTECTED");
    }

    /// The container calamine parses is one written here, and it holds the
    /// workbook stream unchanged.
    #[test]
    fn the_rewritten_container_carries_the_workbook_stream_unchanged() {
        let stream = workbook(&[number_cell(2, 3)]);
        let container = trusted_container(&stream).unwrap();
        let mut compound = cfb::CompoundFile::open(Cursor::new(container)).unwrap();
        let mut read_back = Vec::new();
        compound
            .open_stream("/Workbook")
            .unwrap()
            .read_to_end(&mut read_back)
            .unwrap();
        assert_eq!(read_back, stream);
    }
}
