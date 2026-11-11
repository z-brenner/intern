//! Literal-evidence checking.
//!
//! Every fact that can reach a filename must be backed by a verbatim excerpt
//! the model quotes from the digest. Because distillation keeps text verbatim,
//! "the model quoted something that is actually in the document" is a check
//! Intern can make locally and cheaply, and it is the main reason a proposal
//! can be trusted without a human reading it.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};

use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::UnicodeNormalization;

use crate::distill::DocumentDigest;
use crate::domain::ParserWarning;

/// The text the literal checks read: the blocks a fact must be found inside,
/// the document's headings, the lines that carry its dates, and what
/// extraction warned about. A [`DocumentDigest`] is one; the evidence
/// pipeline's views of the context and of the whole document are the
/// others, so every check runs the same code
/// over whichever text it is asked about.
pub trait Segments {
    /// The verbatim blocks, in document order. A quote must lie inside one.
    fn segments(&self) -> &[String];
    /// Every heading, in document order.
    fn headings(&self) -> &[String];
    /// The lines that carry a date, in document order.
    fn date_lines(&self) -> &[String];
    fn parser_warnings(&self) -> &[ParserWarning];
}

impl Segments for DocumentDigest {
    fn segments(&self) -> &[String] {
        &self.segments
    }

    fn headings(&self) -> &[String] {
        &self.outline
    }

    fn date_lines(&self) -> &[String] {
        &self.date_lines
    }

    fn parser_warnings(&self) -> &[ParserWarning] {
        &self.parser_warnings
    }
}

/// Folds case, normalizes Unicode, unifies quote characters, and collapses
/// whitespace so that a quote survives PDF and OCR typography differences.
/// Invisible characters ([`is_invisible`]) are left out, so one neither
/// splits a word nor keeps a quote from matching it.
pub fn normalize(value: &str) -> String {
    if value.is_ascii() {
        return normalize_ascii(value);
    }
    normalize_unicode(value)
}

/// [`normalize`] for text that is all ASCII, which is most of what documents
/// write and all of a date's spellings. There it comes to very little - no
/// ASCII character is invisible, is changed by Unicode normalization, or is
/// one of the quotes, dashes and spaces that are unified, and folding case
/// is lowering "A" to "Z" - so the tables [`normalize_unicode`] walks for
/// every character are not consulted. The result is the same, character for
/// character.
fn normalize_ascii(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut pending_space = false;
    for byte in value.bytes() {
        let character = char::from(byte);
        if character.is_whitespace() {
            pending_space = !result.is_empty();
        } else {
            if pending_space {
                result.push(' ');
                pending_space = false;
            }
            result.push(character.to_ascii_lowercase());
        }
    }
    result
}

/// [`normalize`] for any text.
fn normalize_unicode(value: &str) -> String {
    let visible = value.chars().filter(|character| !is_invisible(*character));
    let normalized = visible.nfkc().case_fold().map(|character| match character {
        '\u{2018}' | '\u{2019}' | '\u{201a}' | '\u{201b}' | '\u{2032}' => '\'',
        '\u{201c}' | '\u{201d}' | '\u{201e}' | '\u{201f}' | '\u{2033}' => '"',
        '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
        '\u{00a0}' | '\u{2007}' | '\u{202f}' => ' ',
        other => other,
    });

    let mut result = String::with_capacity(value.len());
    let mut pending_space = false;
    for character in normalized {
        if character.is_whitespace() {
            pending_space = !result.is_empty();
        } else {
            if pending_space {
                result.push(' ');
                pending_space = false;
            }
            result.push(character);
        }
    }
    result
}

/// Whether a character is invisible: a soft hyphen, a zero-width space or
/// joiner, a direction mark, embedding or isolate (the Arabic letter mark
/// among them), a word joiner, a byte order mark. PDF and web text carry
/// them inside words and names. The invisible operators (U+2061 to U+2064:
/// invisible times, plus, separator, function application) are not among
/// them: they mean something, and "2", invisible times, "3" is not "23".
pub(crate) fn is_invisible(character: char) -> bool {
    matches!(
        character,
        '\u{ad}'
            | '\u{61c}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'
            | '\u{2066}'..='\u{2069}'
            | '\u{feff}'
    )
}

/// True when `excerpt` appears verbatim inside a single kept block.
///
/// Checking per block rather than against the joined digest means a quote can
/// never be "supported" by text that straddles an elision marker.
pub fn digest_contains(digest: &impl Segments, excerpt: &str) -> bool {
    let excerpt = normalize(excerpt);
    !excerpt.is_empty()
        && digest
            .segments()
            .iter()
            .any(|segment| contains_whole(&normalize(segment), &excerpt))
}

/// True when `needle` occurs in `haystack` without a letter or a digit
/// running straight into either end of it.
///
/// Plain substring matching made "John Smithson" evidence for "John Smith" -
/// a different person - and "in order to ... form" evidence for an "Order
/// Form". The boundary is only required where the needle itself ends in a
/// word character, so a quote that starts or ends on punctuation still
/// matches the way it reads.
pub(crate) fn contains_whole(haystack: &str, needle: &str) -> bool {
    let word_start = needle.chars().next().is_some_and(char::is_alphanumeric);
    let word_end = needle
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric);
    let mut from = 0;
    while let Some(found) = haystack.get(from..).and_then(|rest| rest.find(needle)) {
        let position = from + found;
        let end = position + needle.len();
        let before_is_word = word_start
            && haystack[..position]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
        let after_is_word = word_end
            && haystack[end..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric);
        if !before_is_word && !after_is_word {
            return true;
        }
        from = position
            + haystack[position..]
                .chars()
                .next()
                .map_or(1, char::len_utf8);
    }
    false
}

/// `normalize`, minus the punctuation that typography and typing scatter
/// through a name: commas, periods, apostrophes, and quotation marks.
///
/// "Contoso Worldwide Inc" and "Contoso Worldwide, Inc." are one company, and
/// a name a person would recognize as the same name should not be thrown out
/// of a filename over a comma. A standalone "&" reads as "and" for the same
/// reason - "Quill & Vane" and "Quill and Vane" are one firm - while the "&"
/// inside a name like "AT&T" is part of the name and stays.
pub fn normalize_loosely(value: &str) -> String {
    let stripped = normalize(value)
        .chars()
        .filter(|character| !matches!(character, '.' | ',' | '\'' | '"'))
        .collect::<String>();
    stripped
        .split_whitespace()
        .map(|word| if word == "&" { "and" } else { word })
        .collect::<Vec<_>>()
        .join(" ")
}

/// True when `excerpt` appears inside a single kept block once punctuation is
/// disregarded on both sides. Used only for names, where punctuation is
/// typography rather than meaning.
pub fn digest_contains_loosely(digest: &impl Segments, excerpt: &str) -> bool {
    let excerpt = normalize_loosely(excerpt);
    !excerpt.is_empty()
        && digest
            .segments()
            .iter()
            .any(|segment| contains_whole(&normalize_loosely(segment), &excerpt))
}

/// True when the quoted evidence both contains the claimed field value and is
/// itself present in the digest.
pub fn evidence_supports(digest: &impl Segments, excerpt: &str, field: &str) -> bool {
    let normalized_excerpt = normalize(excerpt);
    let normalized_field = normalize(field);
    !normalized_field.is_empty()
        && normalized_excerpt.contains(&normalized_field)
        && digest
            .segments()
            .iter()
            .any(|segment| normalize(segment).contains(&normalized_excerpt))
}

/// True when the ISO date is written, in some ordinary human form, somewhere in
/// the document itself.
///
/// This is the check that matters. A small model paraphrases its own quotes -
/// it will answer "This Agreement is effective as of February 14, 2025" for a
/// document whose line actually reads "Effective date: February 14, 2025" - so
/// gating on the exact wrapper wording throws away correct dates. Gating on
/// whether the *date* is really in the document does not.
pub fn digest_contains_date(digest: &impl Segments, iso_date: &str) -> bool {
    digest
        .segments()
        .iter()
        .any(|segment| date_matches_evidence(iso_date, segment))
}

/// True when the ISO date is written, in some ordinary human form, inside the
/// given text.
pub fn date_matches_evidence(iso_date: &str, excerpt: &str) -> bool {
    !date_match_positions(iso_date, &normalize(excerpt)).is_empty()
}

/// Byte offsets in `normalized` (already `normalize`d text) where a spelling
/// of `iso_date` begins. Every offset is a real statement of that date; a
/// caller judging context - what wording introduces the date - needs all of
/// them, because one date can be stated twice on a line in different roles.
pub fn date_match_positions(iso_date: &str, normalized: &str) -> Vec<usize> {
    let mut positions = date_statements(iso_date, normalized)
        .into_iter()
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    positions.sort_unstable();
    positions.dedup();
    positions
}

/// Byte offsets in `normalized` where `iso_date` is written as
/// [`date_match_positions`] finds it, or in a looser form a scan or a stamp
/// leaves: a rubber stamp's day with its figures set apart ("SEP 1 8
/// 2015"), a comma that lost its space ("November 22,1923"), an invoice's
/// two-figure year after a month in words ("02-Jun-17"), a long month OCR
/// misread by one letter ("Decenber 18, 1907"). Validation reads the
/// model's date this way, and only validation: the index and the prompt
/// read dates as [`date_match_positions`] does, so a recorded reply is
/// asked the same question, and a two-figure year's century is the
/// reply's, never offered on its own.
pub(crate) fn read_date_positions(iso_date: &str, normalized: &str) -> Vec<usize> {
    let loose = loose_spellings(iso_date);
    let mut positions = date_match_positions(iso_date, normalized);
    positions.extend(spelled_at(&loose, normalized));
    let mended = with_months_mended(normalized);
    if mended != normalized {
        positions.extend(date_match_positions(iso_date, &mended));
        positions.extend(spelled_at(&loose, &mended));
    }
    positions.sort_unstable();
    positions.dedup();
    positions
}

/// Whether a digest writes `iso_date` as [`read_date_positions`] reads it.
pub(crate) fn digest_reads_date(digest: &impl Segments, iso_date: &str) -> bool {
    digest
        .segments()
        .iter()
        .any(|segment| !read_date_positions(iso_date, &normalize(segment)).is_empty())
}

/// The looser spellings of an ISO date [`read_date_positions`] reads.
fn loose_spellings(iso_date: &str) -> Vec<String> {
    let Some((year, month, day)) = iso_parts(iso_date) else {
        return Vec::new();
    };
    let short_year = &year[2..];
    let day_unpadded = day.trim_start_matches('0');
    let mut spellings = Vec::new();
    for month_word in month_words(month) {
        if let [first, second] = day.as_bytes() {
            let (first, second) = (char::from(*first), char::from(*second));
            spellings.push(format!("{month_word} {first} {second} {year}"));
            spellings.push(format!("{first} {second} {month_word} {year}"));
        }
        for day_form in [day_unpadded, day] {
            spellings.push(format!("{month_word} {day_form},{year}"));
            spellings.push(format!("{day_form}-{month_word}-{short_year}"));
        }
    }
    spellings
}

/// An ISO date's year, month number and two-figure day.
fn iso_parts(iso_date: &str) -> Option<(&str, usize, &str)> {
    if !is_valid_iso_date(iso_date) {
        return None;
    }
    let month = iso_date[5..7].parse::<usize>().ok()?;
    Some((&iso_date[0..4], month, &iso_date[8..10]))
}

/// A month's name as documents write it: whole, cut to three or four
/// letters, with or without a stop.
fn month_words(month: usize) -> Vec<String> {
    let Some(name) = MONTH_NAMES.get(month.wrapping_sub(1)) else {
        return Vec::new();
    };
    let mut words = vec![(*name).to_owned()];
    for length in [3, 4] {
        if length < name.len() {
            let cut = &name[..length];
            for word in [cut.to_owned(), format!("{cut}.")] {
                if !words.contains(&word) {
                    words.push(word);
                }
            }
        }
    }
    words
}

const MONTH_NAMES: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// Where any of `spellings` stands in `normalized` with no digit running
/// into either end.
fn spelled_at(spellings: &[String], normalized: &str) -> Vec<usize> {
    let bytes = normalized.as_bytes();
    let mut found = Vec::new();
    for spelling in spellings {
        let mut from = 0;
        while let Some(at) = normalized[from..].find(spelling.as_str()) {
            let position = from + at;
            let end = position + spelling.len();
            let digit_before = position > 0 && bytes[position - 1].is_ascii_digit();
            let digit_after = bytes.get(end).is_some_and(u8::is_ascii_digit);
            if !digit_before && !digit_after {
                found.push(position);
            }
            from = position + spelling.chars().next().map_or(1, char::len_utf8);
        }
    }
    found
}

/// How one statement of a date is written, which decides whether a reader
/// could take it for a different date.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DateSpelling {
    /// With the month in words: "April 1, 2026", "1st April 2026".
    Written,
    /// Numbers, year first - "2026-04-01", "2026.04.01" - which every
    /// convention reads the same way.
    YearFirst,
    /// Numbers, month first: "04/01/2026", "4-1-2026", "04.01.26".
    MonthFirst,
    /// Numbers, day first: "01/04/2026", "1.4.2026", "01-04-26".
    DayFirst,
}

/// Every statement of `iso_date` in `normalized`, with the spelling it is
/// written in. One position can carry two spellings - "04/04/2026" is the
/// same text month first and day first - and each is listed.
pub(crate) fn date_statements(iso_date: &str, normalized: &str) -> Vec<(usize, DateSpelling)> {
    let candidates = searched_spellings(iso_date);
    let bytes = normalized.as_bytes();
    let mut statements = Vec::new();
    for (candidate, spelling) in candidates.iter() {
        let mut from = 0;
        while let Some(found) = normalized[from..].find(candidate.as_str()) {
            let position = from + found;
            let end = position + candidate.len();
            // A spelling that runs straight into other digits is part of a
            // longer number, not this date: "12/1/2026" states December 1 and
            // must never support February 1 because "2/1/2026" sits inside it.
            let digit_before = position > 0 && bytes[position - 1].is_ascii_digit();
            let digit_after = bytes.get(end).is_some_and(u8::is_ascii_digit);
            if !digit_before && !digit_after && !statements.contains(&(position, *spelling)) {
                statements.push((position, *spelling));
            }
            from = position + candidate.chars().next().map_or(1, char::len_utf8);
        }
    }
    statements
}

/// The spellings of one date as [`date_statements`] searches for them.
type SearchedSpellings = Arc<[(String, DateSpelling)]>;

/// How many dates' spellings [`searched_spellings`] keeps. A document's
/// checks ask about few dates - the model's, and the ones stated beside it -
/// and ask about each on every line, so this is room to spare; a document
/// with more dates than this has the oldest worked out again when it is next
/// asked about. One date's spellings take about 10 KB, so the list stays
/// under a megabyte.
const SPELLINGS_KEPT: usize = 64;

/// The dates asked about most recently and their spellings, oldest first.
/// One list for the process, behind a lock that is held only to look a date
/// up or to add one - never while spellings are worked out or searched for.
static SPELLINGS: Mutex<VecDeque<(String, SearchedSpellings)>> = Mutex::new(VecDeque::new());

/// [`date_spellings`], each spelling `normalize`d as the text it is searched
/// in is, the empty ones dropped, in the same order.
///
/// Working these out is most of what a date check costs - up to 166
/// spellings built and normalized - and a document's checks ask about the
/// same date on every line, which made it close to half of all the engine's
/// time. The spellings depend on the date alone, so the last
/// [`SPELLINGS_KEPT`] dates' are kept: what comes back is exactly what
/// working it out again would give.
fn searched_spellings(iso_date: &str) -> SearchedSpellings {
    let kept = |spellings: &VecDeque<(String, SearchedSpellings)>| {
        spellings
            .iter()
            .find(|(date, _)| date == iso_date)
            .map(|(_, searched)| Arc::clone(searched))
    };
    // A panic cannot leave the list half-written - nothing is computed while
    // it is locked - so a poisoned lock still guards a sound list.
    let found = kept(&SPELLINGS.lock().unwrap_or_else(PoisonError::into_inner));
    if let Some(searched) = found {
        return searched;
    }
    let searched: SearchedSpellings = date_spellings(iso_date)
        .into_iter()
        .map(|(candidate, spelling)| (normalize(&candidate), spelling))
        .filter(|(candidate, _)| !candidate.is_empty())
        .collect();
    // Anything that is not a date's shape has no spellings, costs nothing to
    // ask about again, and must not push a real date out of the list.
    if searched.is_empty() {
        return searched;
    }
    let mut spellings = SPELLINGS.lock().unwrap_or_else(PoisonError::into_inner);
    // Another thread may have added this date while it was worked out here.
    if let Some(searched) = kept(&spellings) {
        return searched;
    }
    if spellings.len() >= SPELLINGS_KEPT {
        spellings.pop_front();
    }
    spellings.push_back((iso_date.to_owned(), Arc::clone(&searched)));
    searched
}

/// The months a reading can misspell by one letter and still be read: the
/// long ones, where one wrong letter leaves no other word ("Decenber").
const LONG_MONTHS: [&str; 7] = [
    "january",
    "february",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// The long month a word of the same length is, but for one letter OCR
/// misread: "decenber" is "december". A month spelled right is not mended.
fn misread_month(word: &str) -> Option<&'static str> {
    LONG_MONTHS.iter().copied().find(|month| {
        word.len() == month.len()
            && word.is_ascii()
            && word
                .bytes()
                .zip(month.bytes())
                .filter(|(read, right)| read != right)
                .count()
                == 1
    })
}

/// `normalized` with each long month OCR misread by one letter spelled
/// right, when a day follows it: the same length, so every offset into it
/// is an offset into the text as read.
fn with_months_mended(normalized: &str) -> String {
    if !normalized.is_ascii() {
        return normalized.to_owned();
    }
    let mut mended = normalized.to_owned();
    let mut start = None;
    let bytes = normalized.as_bytes();
    for at in 0..=bytes.len() {
        let letter = bytes.get(at).is_some_and(u8::is_ascii_alphabetic);
        match (start, letter) {
            (None, true) => start = Some(at),
            (Some(from), false) => {
                start = None;
                let followed_by_day = normalized[at..]
                    .trim_start_matches([' ', '.'])
                    .starts_with(|character: char| character.is_ascii_digit());
                if followed_by_day && let Some(month) = misread_month(&normalized[from..at]) {
                    mended.replace_range(from..at, month);
                }
            }
            _ => {}
        }
    }
    mended
}

/// The ordinary human spellings of an ISO date, each with how it is written.
/// Empty for anything that is not an ISO date's shape.
fn date_spellings(iso_date: &str) -> Vec<(String, DateSpelling)> {
    use DateSpelling::{DayFirst, MonthFirst, Written, YearFirst};

    if iso_date.len() != 10 || !iso_date.is_ascii() {
        return Vec::new();
    }
    let year = &iso_date[0..4];
    let month = &iso_date[5..7];
    let day = &iso_date[8..10];
    let Ok(month_number) = month.parse::<usize>() else {
        return Vec::new();
    };
    if !(1..=12).contains(&month_number) {
        return Vec::new();
    }
    let month_name = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ][month_number - 1];
    let month_unpadded = month.trim_start_matches('0');
    let day_unpadded = day.trim_start_matches('0');
    let ordinal = ordinal_suffix(day_unpadded);
    let short_year = &year[2..];

    // Numeric shapes. Day-first forms are accepted alongside month-first ones
    // because the check asks whether the date is written in the document, and
    // a European invoice writes 1 April as 01/04/2026; the model has already
    // decided which reading the document supports, and validation asks
    // whether anything in the document settles it.
    let mut candidates = vec![
        (iso_date.to_owned(), YearFirst),
        (format!("{year}/{month}/{day}"), YearFirst),
        (format!("{year}.{month}.{day}"), YearFirst),
        (format!("{month}/{day}/{year}"), MonthFirst),
        (
            format!("{month_unpadded}/{day_unpadded}/{year}"),
            MonthFirst,
        ),
        (format!("{month}-{day}-{year}"), MonthFirst),
        (
            format!("{month_unpadded}-{day_unpadded}-{year}"),
            MonthFirst,
        ),
        (format!("{month}.{day}.{year}"), MonthFirst),
        (format!("{day}/{month}/{year}"), DayFirst),
        (format!("{day_unpadded}/{month_unpadded}/{year}"), DayFirst),
        (format!("{day}-{month}-{year}"), DayFirst),
        (format!("{day_unpadded}-{month_unpadded}-{year}"), DayFirst),
        (format!("{day}.{month}.{year}"), DayFirst),
        (format!("{day_unpadded}.{month_unpadded}.{year}"), DayFirst),
        // Two-digit years, the way forms and invoices abbreviate them, in
        // both orders: a UK invoice's "01/04/26" is 1 April, and accepting
        // only the US reading of it filed the document a quarter early. The
        // boundary check below keeps "4/1/26" from matching inside "14/1/26"
        // and "01-04-26" inside "2001-04-26". Dotted and dashed forms are
        // padded only, so a section or version number like "1.4.26" never
        // becomes a date.
        (
            format!("{month_unpadded}/{day_unpadded}/{short_year}"),
            MonthFirst,
        ),
        (format!("{month}/{day}/{short_year}"), MonthFirst),
        (format!("{month}-{day}-{short_year}"), MonthFirst),
        (format!("{month}.{day}.{short_year}"), MonthFirst),
        (
            format!("{day_unpadded}/{month_unpadded}/{short_year}"),
            DayFirst,
        ),
        (format!("{day}/{month}/{short_year}"), DayFirst),
        (format!("{day}-{month}-{short_year}"), DayFirst),
        (format!("{day}.{month}.{short_year}"), DayFirst),
    ];
    // Documents abbreviate months as "Sep", "Sept", "Sept.", or write them out;
    // all of those support the same ISO date.
    let mut spellings = vec![month_name.to_owned()];
    for length in [3, 4] {
        if month_name.len() > length {
            let abbreviation = month_name[..length].to_owned();
            if !spellings.contains(&abbreviation) {
                spellings.push(abbreviation);
            }
        }
    }
    for spelling in spellings {
        for suffix in ["", "."] {
            let month_word = format!("{spelling}{suffix}");
            let mut written = Vec::new();
            for day_form in [day_unpadded, day] {
                written.push(format!("{month_word} {day_form}, {year}"));
                written.push(format!("{month_word} {day_form} {year}"));
                written.push(format!("{month_word}-{day_form}-{year}"));
                written.push(format!("{day_form} {month_word} {year}"));
                written.push(format!("{day_form} {month_word}, {year}"));
                written.push(format!("{day_form}-{month_word}-{year}"));
                // "made this 22 day of June 2018": a contract's blank
                // filled in without its ordinal.
                written.push(format!("{day_form} day of {month_word}, {year}"));
                written.push(format!("{day_form} day of {month_word} {year}"));
            }
            written.push(format!("{month_word} {day_unpadded}{ordinal}, {year}"));
            written.push(format!("{month_word} {day_unpadded}{ordinal} {year}"));
            written.push(format!("{day_unpadded}{ordinal} {month_word} {year}"));
            written.push(format!("{day_unpadded}{ordinal} {month_word}, {year}"));
            written.push(format!(
                "{day_unpadded}{ordinal} day of {month_word}, {year}"
            ));
            written.push(format!(
                "{day_unpadded}{ordinal} day of {month_word} {year}"
            ));
            written.push(format!("{day_unpadded}{ordinal} of {month_word}, {year}"));
            written.push(format!("{day_unpadded}{ordinal} of {month_word} {year}"));
            candidates.extend(written.into_iter().map(|candidate| (candidate, Written)));
        }
    }
    candidates
}

fn ordinal_suffix(day: &str) -> &'static str {
    match day.parse::<u32>() {
        Ok(11..=13) => "th",
        Ok(value) if value % 10 == 1 => "st",
        Ok(value) if value % 10 == 2 => "nd",
        Ok(value) if value % 10 == 3 => "rd",
        _ => "th",
    }
}

/// True when the value is a real calendar date in ISO `YYYY-MM-DD` form.
pub fn is_valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return false;
    }
    let parse = |range: std::ops::Range<usize>| value[range].parse::<u32>().ok();
    let (Some(year), Some(month), Some(day)) = (parse(0..4), parse(5..7), parse(8..10)) else {
        return false;
    };
    if !(1000..=2999).contains(&year) {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let maximum = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=maximum).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::{DigestBudget, distill, source_from_text};

    fn digest_of(text: &str) -> DocumentDigest {
        distill(&source_from_text(text), DigestBudget::default())
    }

    #[test]
    fn quotes_are_matched_through_typography_differences() {
        let digest = digest_of("The Company (\u{201c}Acme\u{201d}) shall\u{00a0}deliver.");
        assert!(digest_contains(
            &digest,
            "The Company (\"Acme\") shall deliver."
        ));
    }

    #[test]
    fn evidence_must_contain_the_claimed_value() {
        let digest = digest_of("This Statement of Work is between Acme and Contoso.");
        assert!(evidence_supports(
            &digest,
            "between Acme and Contoso",
            "Acme"
        ));
        assert!(!evidence_supports(
            &digest,
            "between Acme and Contoso",
            "Northwind"
        ));
    }

    /// Plain substring matching made a document that only ever writes "John
    /// Smithson" evidence for a filename that says "John Smith", which is a
    /// different person and the one thing evidence checking exists to stop.
    #[test]
    fn a_name_that_is_only_a_prefix_of_the_documents_name_is_rejected() {
        let digest = digest_of("This Notice is given to John Smithson of Acme Corporation.");
        assert!(!digest_contains(&digest, "John Smith"));
        assert!(!digest_contains_loosely(&digest, "John Smith"));
        assert!(digest_contains(&digest, "John Smithson"));

        // And a name the document really writes is still found beside
        // punctuation, which is not a word character.
        let digest = digest_of("by and between Acme Corporation (\"Acme\") and Contoso.");
        assert!(digest_contains(&digest, "Acme"));
        assert!(digest_contains_loosely(&digest, "Acme Corporation"));
    }

    #[test]
    fn a_quote_absent_from_the_document_is_rejected() {
        let digest = digest_of("This Statement of Work is between Acme and Contoso.");
        assert!(!digest_contains(&digest, "between Acme and Northwind"));
    }

    #[test]
    fn a_date_is_checked_against_the_document_not_the_models_wording() {
        let digest = digest_of("EMPLOYMENT AGREEMENT\n\nEffective date: February 14, 2025\n");
        // The model paraphrases its own quote; the date is still really there.
        assert!(digest_contains_date(&digest, "2025-02-14"));
        assert!(!digest_contains_date(&digest, "2025-02-15"));
    }

    #[test]
    fn iso_dates_are_matched_against_human_date_forms() {
        assert!(date_matches_evidence(
            "2026-04-01",
            "effective as of April 1, 2026"
        ));
        assert!(date_matches_evidence(
            "2026-04-01",
            "as of the 1st day of April, 2026"
        ));
        // CUAD's held-out cuad-08 and cuad-26: the day of a filled-in blank
        // without its ordinal.
        assert!(date_matches_evidence(
            "2018-06-22",
            "is made this 22 day of June 2018."
        ));
        assert!(date_matches_evidence(
            "2007-01-26",
            "made as of the 26 day of January, 2007 (the \u{201c}Effective Date\u{201d})"
        ));
        assert!(!date_matches_evidence(
            "2006-06-01",
            "executed this Agreement on the st day of JUNE , 2006"
        ));
        assert!(date_matches_evidence("2025-09-14", "dated 9/14/2025"));
        assert!(date_matches_evidence("2025-09-14", "Sept. 14, 2025"));
        assert!(date_matches_evidence("2026-01-05", "2026-01-05"));
        assert!(!date_matches_evidence(
            "2026-04-01",
            "effective as of April 2, 2026"
        ));
        assert!(!date_matches_evidence(
            "2026-04-01",
            "the term of this agreement"
        ));
    }

    /// A rubber stamp sets the day's figures apart ("SEP 1 8 2015"):
    /// validation reads its date, and the index's reading of dates, which
    /// the prompt is built from, is unchanged. A stray pair of figures in
    /// prose is no date.
    #[test]
    fn a_stamps_date_with_its_figures_apart_is_read() {
        let read = |iso: &str, text: &str| !read_date_positions(iso, &normalize(text)).is_empty();
        assert!(read("2015-09-18", "SEP 1 8 2015 OFFICE OF"));
        assert!(read("2007-04-24", "DATE APR 2 4 2007"));
        assert!(read("1974-12-28", "RECEIVED 2 8 DEC 1974"));
        assert!(read("2015-04-30", "APR. 3 0 2015"));
        assert!(read("2026-04-01", "April 1, 2026"));
        assert!(!read("2015-09-01", "SEP 1 8 2015"));
        assert!(!read("2015-09-18", "SEP 1 8 20150"));
        assert!(extract_stated_dates("SEP 1 8 2015 OFFICE OF").is_empty());
        assert!(!date_matches_evidence("2015-09-18", "SEP 1 8 2015"));
        assert!(stamped_dates("in March 1 2 copies were sent").is_empty());
        assert_eq!(
            stamped_dates("SEP 1 8 2015 OFFICE OF"),
            vec![("2015-09-18".to_owned(), 1)]
        );
    }

    /// OCR misreads a long month by a letter ("Decenber") or loses the
    /// space after a comma ("November 22,1923"); an invoice writes
    /// "02-Jun-17". Validation reads each date, and the index's reading of
    /// dates, which the prompt is built from, is unchanged. A misread month
    /// with no day after it is no date.
    #[test]
    fn a_misread_or_run_together_date_is_read() {
        let read = |iso: &str, text: &str| !read_date_positions(iso, &normalize(text)).is_empty();
        assert!(read("1907-12-18", "DAYTON, OHIO Decenber 18, 1907."));
        assert!(!read("1907-12-18", "a decenber of 18, 1907"));
        assert!(read("1923-11-22", "Los Angeles, Cal.,_November 22,1923"));
        assert!(read("2017-06-02", "Invoice Date: 02-JuN-17"));
        assert!(!read("2017-06-02", "Lot 02-Jun-175"));
        assert!(extract_stated_dates("DAYTON, OHIO Decenber 18, 1907.").is_empty());
        assert!(extract_stated_dates("Invoice Date: 02-JuN-17").is_empty());
        assert!(!date_matches_evidence("1923-11-22", "November 22,1923"));
    }

    /// Every spelling here came from a real document shape: forms that
    /// zero-pad the day, British and European orders, dotted numerics, and
    /// the two-digit years invoices abbreviate to.
    #[test]
    fn the_spellings_documents_actually_use_all_support_the_same_date() {
        for spelling in [
            "April 1st 2026",
            "April 01, 2026",
            "1st April 2026",
            "1st April, 2026",
            "1 April, 2026",
            "the 1st of April, 2026",
            "the 1st of April 2026",
            "01/04/2026",
            "01-04-2026",
            "01.04.2026",
            "1.4.2026",
            "2026.04.01",
            "Date: 4/1/26",
            "Date: 04/01/26",
            "APRIL 1, 2026",
        ] {
            assert!(
                date_matches_evidence("2026-04-01", spelling),
                "{spelling} should support 2026-04-01"
            );
        }
    }

    /// A date is a whole token. "12/1/2026" states December 1 and contains
    /// the characters "2/1/2026", which must not make it support February 1.
    #[test]
    fn a_date_inside_a_longer_number_is_not_a_statement_of_that_date() {
        assert!(!date_matches_evidence("2026-02-01", "dated 12/1/2026"));
        assert!(!date_matches_evidence("2026-04-01", "reference 14/1/2026"));
        assert!(!date_matches_evidence("2026-04-01", "order 4/1/2026001"));
        assert!(!date_matches_evidence("2026-04-01", "code 24/1/26"));
        assert!(date_matches_evidence("2026-12-01", "dated 12/1/2026"));
        assert!(date_matches_evidence("2026-04-01", "(4/1/2026)"));
    }

    /// A UK or Australian invoice prints 1 April 2026 as "01/04/26". Only
    /// the US reading of that used to match, so the right date was withheld
    /// and the wrong one - 4 January - filed the document as Ready.
    #[test]
    fn day_first_two_digit_years_padded_dotted_and_dashed_match() {
        for spelling in [
            "Invoice Date: 01/04/26",
            "Invoice Date: 01.04.26",
            "Invoice Date: 01-04-26",
            "Invoice Date: 1/4/26",
        ] {
            assert!(
                date_matches_evidence("2026-04-01", spelling),
                "{spelling} should support 2026-04-01 day first"
            );
        }
        for spelling in [
            "Invoice Date: 04/01/26",
            "Invoice Date: 04.01.26",
            "Invoice Date: 04-01-26",
            "Invoice Date: 4/1/26",
        ] {
            assert!(
                date_matches_evidence("2026-04-01", spelling),
                "{spelling} should support 2026-04-01 month first"
            );
        }
        // Each statement says which way round it was read.
        let statements = |date: &str, text: &str| {
            let mut found = date_statements(date, text);
            found.sort_by_key(|(position, _)| *position);
            found
        };
        assert_eq!(
            statements("2026-04-01", "date: 01.04.26"),
            vec![(6, DateSpelling::DayFirst)]
        );
        assert_eq!(
            statements("2026-04-01", "date: 04-01-26"),
            vec![(6, DateSpelling::MonthFirst)]
        );
        assert_eq!(
            statements("2026-04-04", "on 04/04/2026"),
            vec![(3, DateSpelling::MonthFirst), (3, DateSpelling::DayFirst)]
        );
        assert_eq!(
            statements("2026-04-01", "on april 1, 2026 and 2026-04-01"),
            vec![(3, DateSpelling::Written), (21, DateSpelling::YearFirst)]
        );
    }

    /// Unpadded dotted numbers are section and version numbers far more often
    /// than dates, and a two-digit date inside a longer number is no date.
    #[test]
    fn version_numbers_are_not_dates() {
        for text in [
            "Release v1.4.26 of the software",
            "Section 1.4.26 applies",
            "see clause 1-4-26",
            "filed 2001-04-26",
            "order 101/04/26",
            "code 01/04/265",
        ] {
            assert!(!date_matches_evidence("2026-04-01", text), "{text}");
            assert!(!date_matches_evidence("2026-01-04", text), "{text}");
        }
        assert!(
            date_matches_evidence("2001-04-26", "filed 2001-04-26"),
            "the ISO date itself still matches"
        );
    }

    /// A reviewer is offered the numeric dates that can only mean one thing,
    /// and the ones the document's own other dates show the order of.
    #[test]
    fn stated_dates_offers_unambiguous_numeric_dates_and_follows_the_documents_order() {
        let day_first = digest_of(
            "INVOICE INV-2048\nInvoice Date: 03/04/2026\nDelivered: 30/01/2026\nDue: 2026.05.03\n",
        );
        assert_eq!(numeric_date_order(&day_first), Some(NumericOrder::DayFirst));
        assert_eq!(
            stated_dates(&day_first),
            vec!["2026-04-03", "2026-01-30", "2026-05-03"]
        );

        let month_first = digest_of("INVOICE\nInvoice Date: 03/04/2026\nDue Date: 04/30/2026\n");
        assert_eq!(
            numeric_date_order(&month_first),
            Some(NumericOrder::MonthFirst)
        );
        assert_eq!(stated_dates(&month_first), vec!["2026-03-04", "2026-04-30"]);

        // Alone, "03/04/2026" could be either, and is not offered under a
        // guess; "05/05/2026" reads the same both ways.
        let unsettled = digest_of("INVOICE\nInvoice Date: 03/04/2026\nShipped: 05/05/2026\n");
        assert_eq!(numeric_date_order(&unsettled), None);
        assert_eq!(stated_dates(&unsettled), vec!["2026-05-05"]);
        // A two-digit year settles the order, but its century is a guess,
        // so it is not offered itself.
        let short_year = digest_of("INVOICE\nInvoice Date: 03/04/2026\nDue: 30/01/26\n");
        assert_eq!(
            numeric_date_order(&short_year),
            Some(NumericOrder::DayFirst)
        );
        assert_eq!(stated_dates(&short_year), vec!["2026-04-03"]);

        // A written date keeps its place among numeric ones on its line.
        let mixed = digest_of("NOTICE\nSent 30/01/2026, effective March 1, 2026.\n");
        assert_eq!(stated_dates(&mixed), vec!["2026-01-30", "2026-03-01"]);
    }

    /// The order is the document's only when the document is consistent
    /// about it: one numeric date that can only be read month first and one
    /// that can only be read day first settle nothing.
    #[test]
    fn a_document_that_writes_dates_both_ways_has_no_order() {
        assert_eq!(
            numeric_date_order(&digest_of("Dated 30/01/2026.\nDue 01/30/2026.\n")),
            None
        );
        assert_eq!(
            numeric_date_order(&digest_of("Dated 03/04/2026.\nTerm 12 months.\n")),
            None
        );
        // A two-digit year settles the order when it is padded or slashed;
        // an unpadded dotted section number does not.
        assert_eq!(
            numeric_date_order(&digest_of("Due 30/04/26.\n")),
            Some(NumericOrder::DayFirst)
        );
        assert_eq!(
            numeric_date_order(&digest_of("See section 13.4.26.\n")),
            None
        );
        assert_eq!(
            numeric_date_order(&digest_of("Due 04-30-26.\n")),
            Some(NumericOrder::MonthFirst)
        );
    }

    #[test]
    fn loose_matching_ignores_only_punctuation() {
        let digest = digest_of("by and between Contoso Worldwide, Inc. and Jane O'Brien");
        assert!(digest_contains_loosely(&digest, "Contoso Worldwide Inc"));
        assert!(digest_contains_loosely(&digest, "Contoso Worldwide, Inc."));
        assert!(digest_contains_loosely(&digest, "Jane OBrien"));
        assert!(!digest_contains_loosely(&digest, "Contoso Worldwide LLC"));
        assert!(!digest_contains_loosely(&digest, "Contoso Inc"));
        assert_eq!(normalize_loosely("  Acme,  Inc. "), "acme inc");
    }

    #[test]
    fn calendar_validity_is_enforced() {
        assert!(is_valid_iso_date("2024-02-29"));
        assert!(!is_valid_iso_date("2025-02-29"));
        assert!(!is_valid_iso_date("2026-13-01"));
        assert!(!is_valid_iso_date("2026-4-1"));
    }
}

/// Every date the document states, in the order it first states them, with
/// duplicates removed and the list capped so a long agreement's schedule of
/// dates does not become a wall of buttons. Drawn from the digest's date
/// lines, so each is a date a person could find on the page - which is what
/// makes it fit to offer a reviewer who has to give a document a date the
/// model did not.
///
/// A date printed only in numbers is offered when it can be read one way:
/// "30/01/2026" can only be 30 January, and once the document has shown its
/// order that way, its "03/04/2026" is 3 April. A numeric date that could be
/// either, in a document that never says which, is left out rather than
/// offered under a guess - and so is a two-digit year, whose century is one.
pub fn stated_dates(digest: &impl Segments) -> Vec<String> {
    const MOST: usize = 8;
    let order = numeric_date_order(digest);
    let mut found: Vec<String> = Vec::new();
    for line in digest.date_lines() {
        let normalized = normalize(line);
        let mut on_line = extract_stated_dates(line)
            .into_iter()
            .map(|date| {
                let position = date_match_positions(&date, &normalized)
                    .first()
                    .copied()
                    .unwrap_or(usize::MAX);
                (position, date)
            })
            .collect::<Vec<_>>();
        on_line.extend(
            numeric_dates(&normalized)
                .iter()
                .filter_map(|token| numeric_reading(token, order).map(|date| (token.start, date))),
        );
        // In the order the line states them, so a reviewer reads the chips
        // the way the page reads.
        on_line.sort_by_key(|(position, _)| *position);
        for (_, date) in on_line {
            if !found.contains(&date) {
                found.push(date);
                if found.len() == MOST {
                    return found;
                }
            }
        }
    }
    found
}

/// Every ISO date a line states with a written month, a hyphenated written
/// month, or ISO/slash notation. Purely numeric forms like `3/4/2026` are
/// deliberately not extracted: without the document's locale they are
/// ambiguous, and this feeds a substitution that must never guess.
pub fn extract_stated_dates(line: &str) -> Vec<String> {
    const MONTHS: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    fn month_number(token: &str) -> Option<usize> {
        let token = token.trim_end_matches('.');
        MONTHS.iter().position(|month| {
            *month == token
                || (token.len() >= 3 && month.len() > token.len() && month.starts_with(token))
        })
    }
    fn day_number(token: &str) -> Option<u32> {
        let digits = token.trim_end_matches(|c: char| c.is_ascii_alphabetic());
        let day = digits.parse::<u32>().ok()?;
        ((1..=31).contains(&day) && digits.len() <= 2).then_some(day)
    }
    fn year_number(token: &str) -> Option<u32> {
        let token = token.trim_end_matches('.');
        let year = token.parse::<u32>().ok()?;
        ((1000..=2999).contains(&year) && token.len() == 4).then_some(year)
    }
    fn push(found: &mut Vec<String>, year: u32, month: usize, day: u32) {
        let iso = format!("{year:04}-{:02}-{day:02}", month + 1);
        if is_valid_iso_date(&iso) && !found.contains(&iso) {
            found.push(iso);
        }
    }

    let normalized = normalize(line);
    let tokens: Vec<&str> = normalized
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '(' | ')' | '"' | '\''))
        })
        .collect();
    let mut found = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        // ISO 2026-04-01 and slashed 2026/04/01, possibly ending a sentence.
        let bare = token.trim_end_matches('.');
        if bare.len() == 10 && (bare.as_bytes()[4] == b'-' || bare.as_bytes()[4] == b'/') {
            let iso = bare.replace('/', "-");
            if is_valid_iso_date(&iso) && !found.contains(&iso) {
                found.push(iso);
            }
            continue;
        }
        // Hyphenated written month: 2-june-2023 or june-2-2023.
        let parts: Vec<&str> = bare.split('-').collect();
        if parts.len() == 3 {
            if let (Some(month), Some(day), Some(year)) = (
                month_number(parts[1]),
                day_number(parts[0]),
                year_number(parts[2]),
            ) {
                push(&mut found, year, month, day);
                continue;
            }
            if let (Some(month), Some(day), Some(year)) = (
                month_number(parts[0]),
                day_number(parts[1]),
                year_number(parts[2]),
            ) {
                push(&mut found, year, month, day);
                continue;
            }
        }
        let Some(month) = month_number(bare) else {
            continue;
        };
        // "june 2, 2023" / "june 2 2023" / "june 2nd, 2023"
        if let (Some(Some(day)), Some(Some(year))) = (
            tokens.get(index + 1).map(|t| day_number(t)),
            tokens.get(index + 2).map(|t| year_number(t)),
        ) {
            push(&mut found, year, month, day);
            continue;
        }
        // "2 june 2023" and "2nd day of june, 2023"
        let day_before = index
            .checked_sub(1)
            .and_then(|i| day_number(tokens[i]))
            .or_else(|| {
                index.checked_sub(3).and_then(|i| {
                    (tokens[i + 1] == "day" && tokens[i + 2] == "of")
                        .then(|| day_number(tokens[i]))
                        .flatten()
                })
            });
        if let (Some(day), Some(Some(year))) =
            (day_before, tokens.get(index + 1).map(|t| year_number(t)))
        {
            push(&mut found, year, month, day);
        }
    }
    found
}

/// The day and the year of a rubber stamp's date whose day is set as two
/// figures apart, starting at `tokens[at]`: "1 8 2015" after "SEP" is the
/// 18th. Each figure is one digit, and a four-digit year follows them.
fn stamped_day(tokens: &[&str], at: usize) -> Option<(u32, u32)> {
    let day = stamp_day(tokens.get(at)?, tokens.get(at + 1)?)?;
    let year_token = tokens.get(at + 2)?.trim_end_matches('.');
    let year = year_token.parse::<u32>().ok()?;
    (year_token.len() == 4 && (1000..=2999).contains(&year)).then_some((day, year))
}

/// The day two single figures set apart make: "1" and "8" are the 18th.
fn stamp_day(first: &str, second: &str) -> Option<u32> {
    let digit = |token: &str| {
        (token.len() == 1)
            .then(|| token.chars().next()?.to_digit(10))
            .flatten()
    };
    let day = digit(first)? * 10 + digit(second)?;
    (1..=31).contains(&day).then_some(day)
}

/// The dates a line states as a rubber stamp does, with the day's figures
/// set apart ("SEP 1 8 2015"), each with the first figure of its day: what
/// a reader that took the figures apart reads as the day.
pub(crate) fn stamped_dates(line: &str) -> Vec<(String, u32)> {
    let normalized = normalize(line);
    let tokens: Vec<&str> = normalized
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '(' | ')' | '"' | '\''))
        })
        .collect();
    let mut found = Vec::new();
    for index in 0..tokens.len() {
        let Some((day, year)) = stamped_day(&tokens, index + 1) else {
            continue;
        };
        let Some(month) = written_month(tokens[index]) else {
            continue;
        };
        let iso = format!("{year:04}-{month:02}-{day:02}");
        if is_valid_iso_date(&iso) {
            found.push((iso, day / 10));
        }
    }
    found
}

/// The number of a month written out or abbreviated: "sep", "sept.",
/// "september".
fn written_month(token: &str) -> Option<usize> {
    let token = token.trim_end_matches('.');
    MONTH_NAMES
        .iter()
        .position(|month| {
            *month == token
                || (token.len() >= 3 && month.len() > token.len() && month.starts_with(token))
        })
        .map(|month| month + 1)
}

/// A token shaped like a numeric date - "04/30/2025", "30.04.2025",
/// "2025-04-30", "4/30/25" - as byte offsets into the text it was found in,
/// with its three runs of digits and the separator between them.
pub(crate) struct NumericDate<'a> {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) parts: [&'a str; 3],
    pub(crate) separator: u8,
}

/// Every numeric-date-shaped token in `text`: one to four digits, a
/// separator out of `/ . -`, one or two digits, the same separator, two to
/// four digits, with no digit running into either end. Whether the token is
/// a real calendar date is not asked here.
pub(crate) fn numeric_dates(text: &str) -> Vec<NumericDate<'_>> {
    let bytes = text.as_bytes();
    let digits_from = |from: usize| -> usize {
        bytes[from..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count()
    };
    let mut found = Vec::new();
    for start in 0..bytes.len() {
        if !bytes[start].is_ascii_digit() || (start > 0 && bytes[start - 1].is_ascii_digit()) {
            continue;
        }
        // Each run is taken whole, so a run longer than its shape allows
        // is a longer number and not a date.
        let first = digits_from(start);
        let Some(&separator) = bytes.get(start + first) else {
            continue;
        };
        if !(1..=4).contains(&first) || !matches!(separator, b'/' | b'.' | b'-') {
            continue;
        }
        let second_start = start + first + 1;
        let second = digits_from(second_start);
        if !(1..=2).contains(&second) || bytes.get(second_start + second) != Some(&separator) {
            continue;
        }
        let third_start = second_start + second + 1;
        let third = digits_from(third_start);
        if !(2..=4).contains(&third) {
            continue;
        }
        let end = third_start + third;
        found.push(NumericDate {
            start,
            end,
            parts: [
                &text[start..start + first],
                &text[second_start..second_start + second],
                &text[third_start..end],
            ],
            separator,
        });
    }
    found
}

/// Which way round a document writes the day and the month of a numeric
/// date.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericOrder {
    /// "30/01/2026": day, month, year.
    DayFirst,
    /// "01/30/2026": month, day, year.
    MonthFirst,
}

/// The order the document writes its numeric dates in, when the document
/// itself settles it: at least one numeric date can only be read one way -
/// "30/01/2026" has no thirtieth month - and no other numeric date can only
/// be read the other way. `None` when nothing settles it, or when the
/// document contradicts itself, because then any one reading is a guess.
pub fn numeric_date_order(digest: &impl Segments) -> Option<NumericOrder> {
    numeric_date_order_of(digest.segments().iter().map(String::as_str))
}

/// [`numeric_date_order`] over any texts: the evidence index settles the
/// order over every unit of the document the same way.
pub(crate) fn numeric_date_order_of<'a>(
    texts: impl IntoIterator<Item = &'a str>,
) -> Option<NumericOrder> {
    let mut order = None;
    for segment in texts {
        let normalized = normalize(segment);
        for token in numeric_dates(&normalized) {
            let Some(settled) = order_of(&token) else {
                continue;
            };
            match order {
                None => order = Some(settled),
                Some(existing) if existing != settled => return None,
                Some(_) => {}
            }
        }
    }
    order
}

/// The order a year-last numeric date can only be read in, if there is
/// one: exactly one of its two readings is a calendar date. A two-digit
/// year counts - "30/01/26" is day first whatever its century - but, as the
/// evidence check does, only padded when dotted or dashed, so a section
/// number like "13.4.26" settles nothing.
fn order_of(token: &NumericDate<'_>) -> Option<NumericOrder> {
    let [first, second, year] = token.parts;
    if first.len() > 2 {
        return None;
    }
    let year = match year.len() {
        4 => year.to_owned(),
        2 if token.separator == b'/' || (first.len() == 2 && second.len() == 2) => {
            format!("20{year}")
        }
        _ => return None,
    };
    let month_first = is_valid_iso_date(&format!("{year}-{first:0>2}-{second:0>2}"));
    let day_first = is_valid_iso_date(&format!("{year}-{second:0>2}-{first:0>2}"));
    match (month_first, day_first) {
        (true, false) => Some(NumericOrder::MonthFirst),
        (false, true) => Some(NumericOrder::DayFirst),
        _ => None,
    }
}

/// The one date a numeric token states, read in the document's order where
/// the token alone could be either. A year-first token is read one way; a
/// two-digit year is not read, because its century is a guess.
fn numeric_reading(token: &NumericDate<'_>, order: Option<NumericOrder>) -> Option<String> {
    let [first, second, third] = token.parts;
    let valid = |year: &str, month: &str, day: &str| {
        let iso = format!("{year}-{month:0>2}-{day:0>2}");
        is_valid_iso_date(&iso).then_some(iso)
    };
    if first.len() == 4 {
        return (third.len() <= 2)
            .then(|| valid(first, second, third))
            .flatten();
    }
    if third.len() != 4 || first.len() > 2 {
        return None;
    }
    match (valid(third, first, second), valid(third, second, first)) {
        (Some(month_first), Some(day_first)) if month_first == day_first => Some(month_first),
        (Some(month_first), Some(day_first)) => match order? {
            NumericOrder::MonthFirst => Some(month_first),
            NumericOrder::DayFirst => Some(day_first),
        },
        (Some(only), None) | (None, Some(only)) => Some(only),
        (None, None) => None,
    }
}

#[cfg(test)]
mod invisible_tests {
    use super::normalize;

    /// A direction mark, isolate or zero-width character between or inside
    /// words is no text: a plain copy of the words matches.
    #[test]
    fn invisible_characters_are_not_read() {
        for written in [
            "Quillon Ridge\u{2069} Bakery",
            "\u{2068}Quillon Ridge Bakery\u{2069}",
            "Quillon\u{61c} Ridge Bakery",
            "Quillon Ri\u{200b}dge Ba\u{ad}kery",
            "Quillon Ridge\u{200e} Bakery",
        ] {
            assert_eq!(normalize(written), "quillon ridge bakery", "{written:?}");
        }
        // An invisible operator means something: two times three is no 23.
        assert_ne!(normalize("2\u{2062}3"), "23");
    }
}

#[cfg(test)]
mod normalize_tests {
    use super::{normalize, normalize_ascii, normalize_unicode};

    /// ASCII text takes a shorter way through [`normalize`], and it must be
    /// the same text that comes out: every one- and two-character ASCII
    /// string, and every three-character one over each kind of whitespace
    /// and of character beside it, which is every step the whitespace
    /// collapsing can take.
    #[test]
    fn ascii_text_normalizes_as_all_text_does() {
        let ascii: Vec<char> = (0..=0x7f_u8).map(char::from).collect();
        for first in &ascii {
            let one = first.to_string();
            assert_eq!(normalize_ascii(&one), normalize_unicode(&one), "{one:?}");
            for second in &ascii {
                let two = format!("{first}{second}");
                assert_eq!(normalize_ascii(&two), normalize_unicode(&two), "{two:?}");
            }
        }
        // Every ASCII character Unicode calls whitespace, the controls beside
        // them that it does not, and a letter of each case, a digit, a mark.
        let kinds = [
            '\t', '\n', '\u{b}', '\u{c}', '\r', ' ', '\0', '\u{8}', '\u{e}', '\u{1c}', '\u{1f}',
            '\u{7f}', 'A', 'z', '0', '-',
        ];
        for first in kinds {
            for second in kinds {
                for third in kinds {
                    let three = format!("{first}{second}{third}");
                    assert_eq!(
                        normalize_ascii(&three),
                        normalize_unicode(&three),
                        "{three:?}"
                    );
                }
            }
        }
        assert_eq!(normalize_ascii(""), "");
        assert_eq!(
            normalize_ascii("\t INVOICE\u{b}Date:\r\n  March 3,\u{c}2024 \n"),
            "invoice date: march 3, 2024"
        );
    }

    /// Text with anything but ASCII in it goes the long way, whatever else it
    /// holds: a no-break or ideographic space is a space, a ligature and a
    /// full-width figure are their letters, a sharp s folds to two letters,
    /// and the whitespace around them collapses as it always did.
    #[test]
    fn other_text_still_goes_the_long_way() {
        for (written, read) in [
            ("Total\u{a0}Due", "total due"),
            (" \u{3000}Total \u{2003}\t Due\u{85}", "total due"),
            (
                "O\u{fb03}ce  \u{ff12}\u{ff10}\u{ff12}\u{ff16}",
                "office 2026",
            ),
            ("STRA\u{df}E\t1", "strasse 1"),
            ("\u{212a}elvin\u{2014}2026", "kelvin-2026"),
            ("Cafe\u{301}  \u{c9}t\u{e9}", "caf\u{e9} \u{e9}t\u{e9}"),
            ("Qu\u{200b}ill\u{ad}on  Ridge", "quillon ridge"),
        ] {
            assert_eq!(normalize(written), read, "{written:?}");
            assert_eq!(
                normalize(written),
                normalize_unicode(written),
                "{written:?}"
            );
        }
    }
}

#[cfg(test)]
mod searched_spelling_tests {
    use std::sync::PoisonError;

    use super::{
        DateSpelling, SPELLINGS, SPELLINGS_KEPT, date_spellings, date_statements, normalize,
        normalize_ascii, normalize_unicode, searched_spellings,
    };

    /// A date's spellings as every call used to work them out: each built,
    /// and normalized the long way.
    fn spellings_afresh(iso_date: &str) -> Vec<(String, DateSpelling)> {
        date_spellings(iso_date)
            .into_iter()
            .map(|(candidate, spelling)| (normalize_unicode(&candidate), spelling))
            .collect()
    }

    /// [`date_statements`] as it was before a date's spellings were kept,
    /// over the spellings [`spellings_afresh`] works out.
    fn statements_afresh(
        candidates: &[(String, DateSpelling)],
        normalized: &str,
    ) -> Vec<(usize, DateSpelling)> {
        let bytes = normalized.as_bytes();
        let mut statements = Vec::new();
        for (candidate, spelling) in candidates {
            if candidate.is_empty() {
                continue;
            }
            let mut from = 0;
            while let Some(found) = normalized[from..].find(candidate) {
                let position = from + found;
                let end = position + candidate.len();
                let digit_before = position > 0 && bytes[position - 1].is_ascii_digit();
                let digit_after = bytes.get(end).is_some_and(u8::is_ascii_digit);
                if !digit_before && !digit_after && !statements.contains(&(position, *spelling)) {
                    statements.push((position, *spelling));
                }
                from = position + candidate.chars().next().map_or(1, char::len_utf8);
            }
        }
        statements
    }

    /// xorshift64, as the fuzz tests scatter text: no dependency, and each
    /// case reproduces from its number.
    struct Rng(u64);

    impl Rng {
        fn seeded(case: u64) -> Self {
            let seed = (case + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
            Self(seed.max(1))
        }

        fn below(&mut self, bound: usize) -> usize {
            let mut value = self.0;
            value ^= value << 13;
            value ^= value >> 7;
            value ^= value << 17;
            self.0 = value;
            usize::try_from(value % bound as u64).expect("below a usize bound")
        }
    }

    /// Every day of four years, a leap year among them.
    fn generated_dates() -> Vec<String> {
        let mut dates = Vec::new();
        for year in 2023..=2026 {
            for month in 1..=12 {
                let days = match month {
                    2 if year % 4 == 0 => 29,
                    2 => 28,
                    4 | 6 | 9 | 11 => 30,
                    _ => 31,
                };
                for day in 1..=days {
                    dates.push(format!("{year:04}-{month:02}-{day:02}"));
                }
            }
        }
        dates
    }

    /// What stands between and against the spellings in a generated line:
    /// the digits and letters the boundary check reads, punctuation, text
    /// that is not ASCII, invisible characters, and every odd whitespace.
    const BETWEEN: &[&str] = &[
        " ", "  ", "", "1", "9", "a", ", ", "/", "-", ".", "\t", "\r\n", "\u{b}", "\u{c}",
        "\u{a0}", "\u{3000}", "\u{2003}", "\u{200b}", "\u{e9}", "\u{2014}", "\u{fb01}", "\u{ff11}",
        " dated ", " DUE: ",
    ];

    /// Things asked about that are not a date's shape, or are one only in
    /// length: capitals, spaces and tabs where figures belong, text that is
    /// not ASCII, months there are none of. The spellings of some of these
    /// are not what `normalize` leaves them as, which is why the kept ones
    /// are the normalized ones.
    const NOT_DATES: &[&str] = &[
        "",
        "2026-4-1",
        "2026-04-01T",
        "2026-13-01",
        "2026-00-10",
        "2026-1\u{e9}-0",
        "20\u{e9}6-01-2",
        "\u{ff12}\u{ff10}\u{ff12}",
        "ABCD-01-XY",
        "JUNE-06-TH",
        " 026-01-02",
        "2026-01- 2",
        "2026-01-2 ",
        "2026\t01\t02",
        "20\t\t-01-\u{b}2",
        "    -01-  ",
        "2026/01/02",
        "0000-01-00",
    ];

    /// A line that writes `iso_date` and the dates beside it some of the ways
    /// [`date_spellings`] lists, as a document might: in capitals, run into
    /// digits and letters, and set apart by whatever [`BETWEEN`] holds.
    fn generated_line(rng: &mut Rng, iso_dates: &[&str]) -> String {
        let mut line = String::new();
        for _ in 0..(2 + rng.below(5)) {
            line.push_str(BETWEEN[rng.below(BETWEEN.len())]);
            let spellings = date_spellings(iso_dates[rng.below(iso_dates.len())]);
            if spellings.is_empty() {
                line.push_str("no date");
                continue;
            }
            let (spelling, _) = &spellings[rng.below(spellings.len())];
            if rng.below(3) == 0 {
                line.push_str(&spelling.to_uppercase());
            } else {
                line.push_str(spelling);
            }
        }
        line.push_str(BETWEEN[rng.below(BETWEEN.len())]);
        line
    }

    /// The spellings kept for a date are the ones it has: for every day of
    /// four years - some 230,000 spellings - each is what the long way
    /// through `normalize` makes of it, in the same order, and the short way
    /// makes the same of it.
    #[test]
    fn a_dates_kept_spellings_are_its_spellings_normalized() {
        let dates = generated_dates();
        assert!(dates.len() > 1_400);
        let mut spellings = 0_usize;
        for date in &dates {
            let afresh = spellings_afresh(date);
            // May has the fewest, with no abbreviation.
            assert!(afresh.len() >= 70, "{date}");
            assert!(afresh.iter().all(|(candidate, _)| !candidate.is_empty()));
            assert_eq!(&*searched_spellings(date), afresh.as_slice(), "{date}");
            // Asked again, the kept ones come back.
            assert_eq!(&*searched_spellings(date), afresh.as_slice(), "{date}");
            for ((written, _), (read, _)) in date_spellings(date).iter().zip(&afresh) {
                assert_eq!(normalize_ascii(written), *read, "{written:?}");
            }
            spellings += afresh.len();
        }
        assert!(spellings > 200_000, "{spellings}");
    }

    /// Keeping a date's spellings changes nothing a check can see. Over a
    /// few thousand generated lines - each date's own spellings and its
    /// neighbours', against digits, letters, text that is not ASCII and
    /// every odd whitespace, normalized and raw - the statements found are
    /// the ones found by working every spelling out afresh. There are far
    /// more dates here than are kept, so this is also every date dropped
    /// from the list and worked out again.
    #[test]
    fn statements_are_found_as_they_were_when_every_spelling_was_worked_out_afresh() {
        // Every fifth day and the day after it: several times as many dates
        // as are kept, each one's spellings worked out afresh once.
        let dates = generated_dates();
        let asked_about: Vec<(&str, Vec<(String, DateSpelling)>)> = dates
            .iter()
            .enumerate()
            .filter(|(at, _)| at % 5 < 2)
            .map(|(_, date)| (date.as_str(), spellings_afresh(date)))
            .collect();
        assert!(asked_about.len() > 8 * SPELLINGS_KEPT);
        let not_dates: Vec<(&str, Vec<(String, DateSpelling)>)> = NOT_DATES
            .iter()
            .map(|asked| (*asked, spellings_afresh(asked)))
            .collect();
        let mut stated = 0_usize;
        for case in 0..2_000_u64 {
            let mut rng = Rng::seeded(case);
            let at = rng.below(asked_about.len() - 1) & !1;
            let (date, date_afresh) = &asked_about[at];
            let (next_day, next_day_afresh) = &asked_about[at + 1];
            let (not_a_date, not_a_date_afresh) = &not_dates[rng.below(not_dates.len())];
            let written = [
                date,
                date,
                next_day,
                dates[rng.below(dates.len())].as_str(),
                dates[rng.below(dates.len())].as_str(),
                not_a_date,
            ];
            let line = generated_line(&mut rng, &written);
            let normalized = normalize(&line);
            for (asked, afresh) in [
                (*date, date_afresh),
                (*next_day, next_day_afresh),
                (*not_a_date, not_a_date_afresh),
            ] {
                for text in [&normalized, &line] {
                    let found = date_statements(asked, text);
                    assert_eq!(
                        found,
                        statements_afresh(afresh, text),
                        "case {case}: {asked:?} in {text:?}"
                    );
                    stated += found.len();
                }
            }
        }
        // The lines do state their dates: this compares findings, not blanks.
        assert!(stated > 2_000, "{stated}");
    }

    /// What is asked about need not be a date. Anything else is answered as
    /// it was - mostly with nothing, and a ten-character string with a month
    /// in the middle with whatever its "spellings" find - and is kept only
    /// if it has spellings.
    #[test]
    fn what_is_not_a_date_is_answered_as_it_was() {
        for asked in NOT_DATES {
            let afresh = spellings_afresh(asked);
            for case in 0..40_u64 {
                let mut rng = Rng::seeded(case);
                let line = generated_line(&mut rng, &[asked, asked, "2026-01-02"]);
                for text in [normalize(&line), line] {
                    assert_eq!(
                        date_statements(asked, &text),
                        statements_afresh(&afresh, &text),
                        "case {case}: {asked:?} in {text:?}"
                    );
                }
            }
        }
        // A capital or a space in what was asked is folded in its spellings
        // as it is in the text, so the two still meet.
        assert!(!date_statements("ABCD-01-XY", &normalize("January XY, ABCD")).is_empty());
        assert!(!date_statements("2026-01- 2", &normalize("January 2, 2026")).is_empty());
        assert!(date_statements("2026-13-01", "2026-13-01").is_empty());
        let kept = SPELLINGS.lock().unwrap_or_else(PoisonError::into_inner);
        for (date, spellings) in kept.iter() {
            assert!(!spellings.is_empty(), "{date:?}");
            assert!(!["", "2026-4-1", "2026-13-01"].contains(&date.as_str()));
        }
    }

    /// The list is bounded, and one list serves every thread: eight threads
    /// asking about overlapping dates, far more of them than are kept, each
    /// get the statements working it out afresh gives, and the list never
    /// holds more than [`SPELLINGS_KEPT`] dates, or one date twice.
    #[test]
    fn the_kept_spellings_are_bounded_and_shared_between_threads() {
        let dates = generated_dates();
        let asked_about: Vec<(&str, Vec<(String, DateSpelling)>)> = dates
            .iter()
            .take(3 * SPELLINGS_KEPT)
            .map(|date| (date.as_str(), spellings_afresh(date)))
            .collect();
        let asked_about = &asked_about;
        std::thread::scope(|scope| {
            for thread in 0..8_usize {
                scope.spawn(move || {
                    for round in 0..600_usize {
                        // Threads walk the same dates in step, so one date is
                        // worked out and added from several at once.
                        let (date, afresh) =
                            &asked_about[(round * 7 + (thread % 3) * 11) % asked_about.len()];
                        let (spelling, _) = &afresh[(round + thread) % afresh.len()];
                        let line = format!("dated {spelling}; due 1{spelling}");
                        let found = date_statements(date, &line);
                        assert_eq!(found, statements_afresh(afresh, &line), "{date}: {line:?}");
                        assert!(!found.is_empty(), "{date}: {line:?}");
                        let kept = SPELLINGS.lock().unwrap_or_else(PoisonError::into_inner);
                        assert!(kept.len() <= SPELLINGS_KEPT, "{}", kept.len());
                        let mut seen: Vec<&str> = kept.iter().map(|(date, _)| &**date).collect();
                        seen.sort_unstable();
                        seen.dedup();
                        assert_eq!(seen.len(), kept.len(), "a date is kept once");
                    }
                });
            }
        });
    }

    /// A panic while the list is locked - the engine catches panics and
    /// carries on with the next document - does not stop dates being read.
    #[test]
    fn a_panic_holding_the_list_does_not_stop_dates_being_read() {
        let panicked = std::thread::spawn(|| {
            let _kept = SPELLINGS.lock().unwrap_or_else(PoisonError::into_inner);
            panic!("a panic with the kept spellings locked");
        })
        .join();
        assert!(panicked.is_err());
        assert!(SPELLINGS.is_poisoned());
        let line = normalize("Effective as of April 1, 2026");
        assert_eq!(date_statements("2026-04-01", &line).len(), 1);
        assert_eq!(
            date_statements("2031-07-09", "due 9 july 2031"),
            statements_afresh(&spellings_afresh("2031-07-09"), "due 9 july 2031")
        );
    }
}

#[cfg(test)]
mod stated_date_tests {
    use super::extract_stated_dates;

    #[test]
    fn extracts_the_written_and_iso_shapes_documents_use() {
        assert_eq!(
            extract_stated_dates("Issued under the Master Services Agreement dated June 2, 2023"),
            vec!["2023-06-02".to_owned()]
        );
        assert_eq!(
            extract_stated_dates(
                "This Statement of Work is effective as of April 1, 2026 and continues"
            ),
            vec!["2026-04-01".to_owned()]
        );
        assert_eq!(
            extract_stated_dates("Delivered 3 March 2026."),
            vec!["2026-03-03".to_owned()]
        );
        assert_eq!(
            extract_stated_dates("signed this 2nd day of June, 2023"),
            vec!["2023-06-02".to_owned()]
        );
        assert_eq!(
            extract_stated_dates("Due on 2026-04-01."),
            vec!["2026-04-01".to_owned()]
        );
        assert_eq!(
            extract_stated_dates("filed 2-June-2023"),
            vec!["2023-06-02".to_owned()]
        );
    }

    #[test]
    fn never_guesses_at_ambiguous_or_broken_shapes() {
        assert!(extract_stated_dates("due 3/4/2026").is_empty());
        assert!(extract_stated_dates("Invoice 2026 covers May and June").is_empty());
        assert!(extract_stated_dates("February 30, 2026 is not a date").is_empty());
        assert!(extract_stated_dates("see section 4, page 2023").is_empty());
    }
}
