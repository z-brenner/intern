//! Delimited text (`.csv`): bank statements, ledger exports, client lists.
//!
//! A CSV file is a sheet without a workbook around it, so it is rendered
//! through the spreadsheet window, the first [`MAX_SHEET_ROWS`] rows by
//! [`MAX_SHEET_COLS`] columns as a pipe table with the same marked elision,
//! rather than handed on as raw text, where a hundred-thousand-row export
//! would flood distillation and its columns would be commas to count.
//!
//! [`MAX_SHEET_ROWS`]: crate::sheet::MAX_SHEET_ROWS
//! [`MAX_SHEET_COLS`]: crate::sheet::MAX_SHEET_COLS

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::extract::{
    CancellationToken, ExtractedDocument, ExtractedPage, ExtractionError, PageSource,
};
use crate::limits::ResourceLimits;
use crate::sheet::{WindowBuilder, elided_document, render_sheet, sanitize_cell};

/// How much of the file the delimiter is chosen from.
const SNIFF_BYTES: u64 = 64 * 1024;
/// How many records of that sample each candidate delimiter is tried on.
const SNIFF_RECORDS: usize = 50;
/// The delimiters exports actually use: comma, the semicolon of locales
/// whose decimal separator is a comma, tab, and pipe. Earlier wins ties.
const CANDIDATES: [u8; 4] = [b',', b';', b'\t', b'|'];

pub fn extract_delimited(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    cancel.check()?;
    let metadata = std::fs::metadata(path).map_err(ExtractionError::io)?;
    limits.validate_source_size(metadata.len())?;
    let mut head = Vec::new();
    File::open(path)
        .map_err(ExtractionError::io)?
        .take(SNIFF_BYTES)
        .read_to_end(&mut head)
        .map_err(ExtractionError::io)?;

    let utf16 = match head.get(..2) {
        Some([0xFF, 0xFE]) => Some(encoding_rs::UTF_16LE),
        Some([0xFE, 0xFF]) => Some(encoding_rs::UTF_16BE),
        _ => None,
    };
    let sheet = if let Some(encoding) = utf16 {
        // UTF-16 cannot be split into records byte by byte, so it is turned
        // into UTF-8 as it is read: a long export streams through the window
        // like any other, rather than being decoded whole first.
        let (sample, _) = encoding.decode_with_bom_removal(&head);
        let delimiter = sniff_delimiter(sample.as_bytes());
        let file = File::open(path).map_err(ExtractionError::io)?;
        read_window(Utf8Reader::new(file, encoding), delimiter, cancel)?
    } else {
        let delimiter = sniff_delimiter(head.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&head));
        let file = File::open(path).map_err(ExtractionError::io)?;
        read_window(BufReader::new(file), delimiter, cancel)?
    };
    let Some(sheet) = sheet else {
        return Err(ExtractionError::unsupported(
            "delimited file contains no readable fields",
        ));
    };
    let (text, elided) = render_sheet(None, &sheet);
    Ok(elided_document(
        vec![ExtractedPage::of_text(1, text, PageSource::AnyDoc)],
        elided,
    ))
}

/// Streams every record through the spreadsheet window, so only what can be
/// shown is kept however long the file is, while the row and column counts
/// the elision marker reports still cover all of it.
fn read_window<R: Read>(
    reader: R,
    delimiter: u8,
    cancel: &CancellationToken,
) -> Result<Option<crate::sheet::CappedSheet>, ExtractionError> {
    let mut records = csv_reader(reader, delimiter);
    let mut record = csv::ByteRecord::new();
    let mut window = WindowBuilder::default();
    let mut row = 0_u32;
    loop {
        if row % 1_024 == 0 {
            cancel.check()?;
        }
        match records.read_byte_record(&mut record) {
            Ok(true) => {}
            Ok(false) => break,
            Err(error) => {
                return Err(ExtractionError::parse_failed(format!(
                    "delimited record {} did not read: {error}",
                    u64::from(row) + 1
                )));
            }
        }
        for (column, field) in record.iter().enumerate() {
            // A byte-order mark is not part of the first field.
            let field = if row == 0 && column == 0 {
                field.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(field)
            } else {
                field
            };
            if field.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let Ok(column) = u32::try_from(column) else {
                break;
            };
            window.push(row, column, || sanitize_cell(&decode_field(field)));
        }
        row = row.checked_add(1).ok_or_else(|| {
            ExtractionError::resource_limit("delimited file has more rows than can be counted")
        })?;
    }
    Ok(window.finish())
}

/// A reader of UTF-16 bytes that yields the same text as UTF-8, its
/// byte-order mark removed.
struct Utf8Reader<R> {
    source: R,
    decoder: encoding_rs::Decoder,
    input: Vec<u8>,
    output: Vec<u8>,
    position: usize,
    finished: bool,
}

impl<R: Read> Utf8Reader<R> {
    fn new(source: R, encoding: &'static encoding_rs::Encoding) -> Self {
        Self {
            source,
            decoder: encoding.new_decoder_with_bom_removal(),
            input: vec![0; 64 * 1024],
            output: Vec::new(),
            position: 0,
            finished: false,
        }
    }
}

impl<R: Read> Read for Utf8Reader<R> {
    fn read(&mut self, target: &mut [u8]) -> std::io::Result<usize> {
        while self.position == self.output.len() {
            if self.finished {
                return Ok(0);
            }
            let read = self.source.read(&mut self.input)?;
            let last = read == 0;
            // Room for everything this input can decode to, so one call
            // consumes all of it.
            let room = self
                .decoder
                .max_utf8_buffer_length(read)
                .ok_or_else(|| std::io::Error::other("decoded text would not fit in memory"))?;
            self.output.resize(room, 0);
            let (_, _, written, _) =
                self.decoder
                    .decode_to_utf8(&self.input[..read], &mut self.output, last);
            self.output.truncate(written);
            self.position = 0;
            self.finished = last;
        }
        let copied = target.len().min(self.output.len() - self.position);
        target[..copied].copy_from_slice(&self.output[self.position..self.position + copied]);
        self.position += copied;
        Ok(copied)
    }
}

fn csv_reader<R: Read>(reader: R, delimiter: u8) -> csv::Reader<R> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(reader)
}

/// One field's text. Exports that are not UTF-8 are, on the Windows machines
/// they come from, Windows-1252; a field that is plain ASCII reads the same
/// either way, so deciding per field costs nothing and needs no look-ahead.
fn decode_field(field: &[u8]) -> String {
    match std::str::from_utf8(field) {
        Ok(text) => text.to_owned(),
        Err(_) => encoding_rs::WINDOWS_1252.decode(field).0.into_owned(),
    }
}

/// The delimiter that splits the sample most consistently.
///
/// Each candidate parses the leading records, and the one under which the
/// most records agree on a field count of two or more wins. Trial parsing
/// rather than counting characters keeps a comma inside a quoted field -
/// `"March 3, 2025"` - from voting for the comma.
fn sniff_delimiter(sample: &[u8]) -> u8 {
    let mut best = (b',', (0_usize, 0_usize));
    for delimiter in CANDIDATES {
        let mut counts = std::collections::HashMap::<usize, usize>::new();
        for record in csv_reader(sample, delimiter)
            .byte_records()
            .take(SNIFF_RECORDS)
            .flatten()
        {
            *counts.entry(record.len()).or_default() += 1;
        }
        let score = counts
            .into_iter()
            .filter(|(fields, _)| *fields > 1)
            .map(|(fields, records)| (records, fields))
            .max()
            .unwrap_or_default();
        if score > best.1 {
            best = (delimiter, score);
        }
    }
    best.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_delimiter_is_the_one_that_splits_records_consistently() {
        assert_eq!(sniff_delimiter(b"a,b,c\n1,2,3\n"), b',');
        assert_eq!(
            sniff_delimiter(b"Datum;Betrag\n03.03.2025;1.250,00\n"),
            b';'
        );
        assert_eq!(sniff_delimiter(b"a\tb\n1\t2\n"), b'\t');
        assert_eq!(sniff_delimiter(b"a|b\n1|2\n"), b'|');
        // The comma inside the quoted date is not a separator.
        assert_eq!(
            sniff_delimiter(b"Date;Payee\n\"March 3, 2025\";Juniper Ridge\n"),
            b';'
        );
        // One column, nothing to split: comma, the format's own default.
        assert_eq!(sniff_delimiter(b"just\none\ncolumn\n"), b',');
    }

    /// A UTF-16 export is read as it streams, a few kilobytes at a time,
    /// and comes out as the same text a UTF-8 one would - characters split
    /// across reads and all.
    #[test]
    fn utf16_streams_through_as_utf8() {
        let text =
            "Payee;Amount\r\nCaf\u{e9} M\u{fc}ller;\u{a3}420\r\n\u{1F4B7};1\r\n".repeat(5_000);
        for (encoding, mark) in [
            (encoding_rs::UTF_16LE, [0xFF, 0xFE]),
            (encoding_rs::UTF_16BE, [0xFE, 0xFF]),
        ] {
            let mut bytes = mark.to_vec();
            for unit in text.encode_utf16() {
                bytes.extend_from_slice(&if encoding == encoding_rs::UTF_16LE {
                    unit.to_le_bytes()
                } else {
                    unit.to_be_bytes()
                });
            }
            // An odd-sized source read splits characters between reads.
            struct Trickle<'a>(&'a [u8]);
            impl Read for Trickle<'_> {
                fn read(&mut self, target: &mut [u8]) -> std::io::Result<usize> {
                    let count = self.0.len().min(target.len()).min(4_097);
                    target[..count].copy_from_slice(&self.0[..count]);
                    self.0 = &self.0[count..];
                    Ok(count)
                }
            }
            let mut decoded = String::new();
            Utf8Reader::new(Trickle(&bytes), encoding)
                .read_to_string(&mut decoded)
                .unwrap();
            assert_eq!(decoded, text);
        }
    }

    #[test]
    fn fields_that_are_not_utf8_read_as_windows_1252() {
        assert_eq!(decode_field(b"M\xFCller \xA3"), "Müller £");
        assert_eq!(decode_field("Müller".as_bytes()), "Müller");
    }
}
