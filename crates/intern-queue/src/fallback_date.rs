//! A date for a reading whose document gave none.
//!
//! A name must start with a date (see [`crate::DATE_REQUIRED`]), and a
//! reading the engine could find no date for used to wait for a person to
//! type one, whatever else it said. When a missing date is the only doubt
//! about the date, the queue gives the reading one from the file itself
//! instead, in this order, and notes on the record which it was:
//!
//! 1. The one date the original filename states unambiguously
//!    ([`date_in_filename`]): "Scan 2024-03-08.pdf", "IMG_20240308_1422.jpg".
//!    A person or a scanner wrote it there about this document.
//! 2. The day the file was last modified, in this machine's calendar
//!    ([`modified_date`]). A document added through Intern is copied into
//!    the input folder with its times kept (`keep_times` in intern-core), so
//!    this is the original's date, not the day it was added.
//! 3. Neither: the reading waits for a person, as it always did.
//!
//! The engine never sees this date: it is the queue's, applied when a
//! reading is stored (`Pipeline::record_for`), and
//! [`intern_engine::review_level::supplied_date`] tells it from a date the
//! document gave.

use std::path::Path;

use chrono::{Datelike, Local, NaiveDate};

/// The note on a record whose date was taken from its original filename.
pub const DATE_FROM_FILENAME: &str = "DATE_FROM_FILENAME";

/// The note on a record whose date is the day its file was last modified.
pub const DATE_FROM_FILE_MODIFIED: &str = "DATE_FROM_FILE_MODIFIED";

/// Where a date the document did not give came from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateSource {
    Filename,
    FileModified,
}

impl DateSource {
    /// The note the record carries in place of `DATE_MISSING`.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Filename => DATE_FROM_FILENAME,
            Self::FileModified => DATE_FROM_FILE_MODIFIED,
        }
    }

    /// Whether `code` is the note of a date from the file.
    pub fn is_note(code: &str) -> bool {
        code == DATE_FROM_FILENAME || code == DATE_FROM_FILE_MODIFIED
    }
}

/// The date to give a reading of the file at `path` that found none in the
/// document, as `YYYY-MM-DD`, and where it came from: the date its name
/// states, else the day it was last modified. `None` when neither can be
/// had.
pub fn date_for(path: &Path) -> Option<(String, DateSource)> {
    let today = Local::now().date_naive();
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| date_in_filename(stem, today))
        .map(|date| (date, DateSource::Filename))
        .or_else(|| modified_date(path).map(|date| (date, DateSource::FileModified)))
}

/// The day `path` was last modified, in this machine's calendar - the day
/// a person sees in their file manager - or `None` when it cannot be read,
/// or reads as a day not yet come or before 1900, which no file's last edit
/// is: a clock set wrong, or a file system that keeps no times.
pub fn modified_date(path: &Path) -> Option<String> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let day = chrono::DateTime::<Local>::from(modified).date_naive();
    plausible(day, Local::now().date_naive()).then(|| iso(day))
}

/// The one date `name` states unambiguously, as `YYYY-MM-DD`, or `None`.
///
/// Read are a year first - `2024-03-08`, `2024_03_08`, `2024.03.08`, or
/// the eight digits `20240308` standing on their own - and a day and a
/// month either way round before a four-digit year, only where one reading
/// alone is a date (`31-01-2024`) or both are the same day (`05-05-2024`).
/// The separators of one date are a hyphen, an underscore or a period, the
/// same one twice. A run of digits is read whole: a date
/// is never cut out of a longer number, an invoice's or a timestamp's.
/// Two-digit years, month names and every date that reads two ways are
/// not read: a filename is no document, and nothing in it settles them.
///
/// A date not yet come (a renewal date in a name, say) or before 1900 is
/// no date of the document's. Two different dates in one name leave the
/// choice to a person; the same date twice is one date.
pub fn date_in_filename(name: &str, today: NaiveDate) -> Option<String> {
    let mut found: Option<NaiveDate> = None;
    let bytes = name.as_bytes();
    let mut start = 0;
    while start < bytes.len() {
        if !bytes[start].is_ascii_digit() || (start > 0 && bytes[start - 1].is_ascii_digit()) {
            start += 1;
            continue;
        }
        if let Some((date, end)) = date_at(bytes, start) {
            if plausible(date, today) {
                match found {
                    Some(earlier) if earlier != date => return None,
                    _ => found = Some(date),
                }
            }
            start = end;
        } else {
            start += 1;
        }
    }
    found.map(iso)
}

/// The date that starts at `start` in `bytes`, which begins a run of
/// digits, and where it ends.
fn date_at(bytes: &[u8], start: usize) -> Option<(NaiveDate, usize)> {
    let (first, after_first) = digits(bytes, start);
    // Eight digits on their own: a year first, no separators.
    if first.len() == 8 {
        let year = number(&first[..4]);
        let month = number(&first[4..6]);
        let day = number(&first[6..]);
        return NaiveDate::from_ymd_opt(year as i32, month, day).map(|date| (date, after_first));
    }
    let separator = *bytes.get(after_first)?;
    if !matches!(separator, b'-' | b'_' | b'.') {
        return None;
    }
    let (second, after_second) = digits(bytes, after_first + 1);
    if second.is_empty() || second.len() > 2 || bytes.get(after_second) != Some(&separator) {
        return None;
    }
    let (third, after_third) = digits(bytes, after_second + 1);
    let (year, month, day) = match (first.len(), third.len()) {
        (4, 1..=2) => (number(first), number(second), number(third)),
        (1..=2, 4) => {
            let (a, b, year) = (number(first), number(second), number(third));
            let day_first = NaiveDate::from_ymd_opt(year as i32, b, a);
            let month_first = NaiveDate::from_ymd_opt(year as i32, a, b);
            return match (day_first, month_first) {
                (Some(date), None) | (None, Some(date)) => Some((date, after_third)),
                (Some(one), Some(other)) if one == other => Some((one, after_third)),
                _ => None,
            };
        }
        _ => return None,
    };
    NaiveDate::from_ymd_opt(year as i32, month, day).map(|date| (date, after_third))
}

/// The run of digits at `start`, and where it ends.
fn digits(bytes: &[u8], start: usize) -> (&[u8], usize) {
    let end = start
        + bytes[start.min(bytes.len())..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
    (&bytes[start.min(bytes.len())..end], end)
}

/// A run of at most eight ASCII digits as a number.
fn number(digits: &[u8]) -> u32 {
    digits
        .iter()
        .fold(0, |value, digit| value * 10 + u32::from(digit - b'0'))
}

/// Whether `date` can be a document's: from 1900, and not after `today`.
fn plausible(date: NaiveDate, today: NaiveDate) -> bool {
    date.year() >= 1900 && date <= today
}

fn iso(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, SystemTime};

    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 17).unwrap()
    }

    fn read(name: &str) -> Option<String> {
        date_in_filename(name, today())
    }

    #[test]
    fn a_year_first_date_is_read_whatever_joins_it() {
        for name in [
            "Scan 2024-03-08",
            "2024-03-08 Lease",
            "lease_2024_03_08_signed",
            "Minutes 2024.03.08",
            "IMG_20240308_142233",
            "Statement 2024-3-8",
            "Invoice 2024-03-08 copy of 2024-03-08",
        ] {
            assert_eq!(read(name).as_deref(), Some("2024-03-08"), "{name}");
        }
    }

    #[test]
    fn a_day_and_month_either_way_round_are_read_only_where_one_reading_is_a_date() {
        assert_eq!(read("Invoice 31-01-2024").as_deref(), Some("2024-01-31"));
        assert_eq!(read("Invoice 01-31-2024").as_deref(), Some("2024-01-31"));
        assert_eq!(read("Board 05.05.2024").as_deref(), Some("2024-05-05"));
        // 4 March or 3 April: nothing in a filename says which.
        assert_eq!(read("Invoice 03-04-2024"), None);
        assert_eq!(read("Invoice 3.4.2024"), None);
    }

    #[test]
    fn what_is_not_one_plain_date_is_no_date() {
        for name in [
            "Scan",
            "Invoice 10438",
            // A number a date would have to be cut out of.
            "PO 202403081",
            "Order 12024-03-08",
            "Ref 2024-03-0812",
            // Separators that differ, more than one character, or a space.
            "Scan 2024-03_08",
            "Scan 2024--03--08",
            "Scan 2024 03 08",
            // Not a calendar date, or not a document's.
            "Scan 2024-02-30",
            "Scan 20241340",
            "Renewal 2031-01-01",
            "Ledger 1850-06-01",
            // Two-digit years, and two different dates.
            "Scan 08-03-24",
            "Lease 2024-03-08 amended 2025-01-10",
            // A version, a time.
            "Policy v2.3.1",
            "Recording 10.22.33",
        ] {
            assert_eq!(read(name), None, "{name}");
        }
    }

    #[test]
    fn today_is_a_date_and_tomorrow_is_not() {
        assert_eq!(read("Scan 2026-11-17").as_deref(), Some("2026-11-17"));
        assert_eq!(read("Scan 2026-11-18"), None);
    }

    /// The date a file's name gives comes first; with none, the day it was
    /// last modified, in this machine's calendar.
    #[test]
    fn a_name_comes_before_the_day_the_file_was_modified() {
        let temp = tempfile::tempdir().unwrap();
        let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        let file = |name: &str| {
            let path = temp.path().join(name);
            fs::write(&path, b"letter").unwrap();
            fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(modified)
                .unwrap();
            path
        };
        let day = iso(chrono::DateTime::<Local>::from(modified).date_naive());

        let named = file("Scan 2024-03-08.pdf");
        assert_eq!(
            date_for(&named),
            Some(("2024-03-08".to_owned(), DateSource::Filename))
        );
        let unnamed = file("Scan.pdf");
        assert_eq!(modified_date(&unnamed), Some(day.clone()));
        assert_eq!(date_for(&unnamed), Some((day, DateSource::FileModified)));
        // The extension is no part of the name's date.
        let odd = file("Scan.20240308");
        assert_eq!(
            date_for(&odd).map(|(_, source)| source),
            Some(DateSource::FileModified)
        );

        assert_eq!(date_for(&temp.path().join("gone.pdf")), None);
        assert_eq!(DateSource::Filename.code(), DATE_FROM_FILENAME);
        assert_eq!(DateSource::FileModified.code(), DATE_FROM_FILE_MODIFIED);
        assert!(DateSource::is_note(DATE_FROM_FILENAME));
        assert!(!DateSource::is_note("DATE_MISSING"));
    }

    /// A file system that keeps no times, or a clock set wrong, gives a day
    /// no document was edited on.
    #[test]
    fn a_modified_day_not_yet_come_is_no_date() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("Scan.pdf");
        fs::write(&path, b"letter").unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(SystemTime::now() + Duration::from_secs(40 * 24 * 60 * 60))
            .unwrap();
        assert_eq!(modified_date(&path), None);
        assert_eq!(date_for(&path), None);
    }
}
