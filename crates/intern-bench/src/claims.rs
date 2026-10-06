//! What a description asserts, and whether the document says it.
//!
//! A one-sentence description is the part of Intern's output a person reads
//! as fact, and the part no evidence check guards. This module pulls the
//! checkable claims out of it - amounts, percentages, other numbers, dates,
//! month-and-year mentions, identifiers, and capitalised multi-word names -
//! and asks whether each occurs in the document's own text.
//!
//! It is a heuristic and errs towards *supported*: a number counts as
//! stated if the document states that number anywhere, a name if its words
//! occur in that order with punctuation and case ignored. What it is built
//! to catch is the invented fact - an amount, a date, an identifier, or a
//! company the document never mentions - not a paraphrase. Single
//! capitalised words are not claims (every sentence starts with one), and a
//! leading article or preposition is dropped from a name ("The Halvorsen
//! Group" claims "Halvorsen Group").

use std::collections::BTreeSet;

use intern_engine::evidence::{date_match_positions, is_valid_iso_date, normalize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Amount,
    Percent,
    Number,
    Date,
    Month,
    Identifier,
    Name,
}

impl ClaimKind {
    /// Whether the claim is a concrete detail - something a reader can act
    /// on - rather than a name.
    pub fn is_concrete(self) -> bool {
        !matches!(self, Self::Name)
    }
}

/// One checkable statement in a description.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Claim {
    pub kind: ClaimKind,
    /// As the description wrote it.
    pub text: String,
    pub supported: bool,
}

/// The document's text, indexed once for every claim checked against it.
pub struct DocumentText {
    /// `evidence::normalize`d, for the engine's own date matching.
    normalized: String,
    /// Lower-case words, punctuation dropped, padded with spaces.
    words: String,
    /// Upper-case letters and digits only, for identifiers whose separators
    /// the description spelled differently.
    alnum: String,
    numbers: BTreeSet<String>,
    values: Vec<f64>,
    identifiers: BTreeSet<String>,
    months: BTreeSet<(u32, u32)>,
}

impl DocumentText {
    pub fn new<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let joined = texts.into_iter().collect::<Vec<_>>().join("\n");
        let normalized = normalize(&joined);
        let words = word_stream(&normalized);
        let alnum = normalized
            .chars()
            .filter(|character| character.is_alphanumeric())
            .flat_map(char::to_uppercase)
            .collect();
        let mut numbers = BTreeSet::new();
        let mut values = Vec::new();
        for run in number_runs(&normalized) {
            for canonical in run_numbers(run) {
                if let Ok(value) = canonical.parse::<f64>() {
                    values.push(value);
                }
                numbers.insert(canonical);
            }
        }
        let mut identifiers = BTreeSet::new();
        let mut months = BTreeSet::new();
        for token in normalized.split_whitespace() {
            let core = trim_token(token);
            if core.chars().any(|character| character.is_ascii_digit()) {
                identifiers.insert(canonical_identifier(core));
            }
            if let Some(dates) = numeric_date(core) {
                for (year, month, _) in dates {
                    months.insert((year, month));
                }
            }
        }
        let list = words.split_whitespace().collect::<Vec<_>>();
        for (index, word) in list.iter().enumerate() {
            let Some(month) = month_number(word) else {
                continue;
            };
            // "March 2026", "March 4 2026", "4th day of March 2026".
            for offset in 1..=2 {
                if let Some(year) = list.get(index + offset).and_then(|word| year_number(word)) {
                    months.insert((year, month));
                }
            }
        }
        Self {
            normalized,
            words,
            alnum,
            numbers,
            values,
            identifiers,
            months,
        }
    }

    /// Whether the document states the ISO date in any ordinary spelling.
    pub fn states_date(&self, iso_date: &str) -> bool {
        !date_match_positions(iso_date, &self.normalized).is_empty()
    }

    /// Whether `phrase` occurs as whole words, case and punctuation ignored.
    pub fn contains_words(&self, phrase: &str) -> bool {
        let phrase = word_stream(&normalize(phrase));
        !phrase.trim().is_empty() && self.words.contains(&phrase)
    }

    fn supports(&self, value: &ClaimValue) -> bool {
        match value {
            ClaimValue::Number {
                canonical,
                value,
                scale,
                decimals,
            } => {
                self.numbers.contains(canonical)
                    || (*scale > 1.0
                        && self
                            .values
                            .iter()
                            .any(|stated| crate::stats::round(stated / scale, *decimals) == *value))
            }
            ClaimValue::Dates(candidates) => candidates.iter().any(|date| self.states_date(date)),
            ClaimValue::Month { year, month } => self.months.contains(&(*year, *month)),
            ClaimValue::Identifier {
                canonical,
                digit_groups,
                has_letters,
            } => {
                self.identifiers.contains(canonical)
                    || (canonical.chars().count() >= 4 && self.alnum.contains(canonical.as_str()))
                    || (!has_letters
                        && digit_groups
                            .iter()
                            .all(|group| self.numbers.contains(group)))
            }
            ClaimValue::Name { words } => self.words.contains(words.as_str()),
        }
    }
}

/// Every claim in `description`, each checked against `document`.
pub fn check_claims(description: &str, document: &DocumentText) -> Vec<Claim> {
    extract(description)
        .into_iter()
        .map(|found| Claim {
            kind: found.kind,
            supported: document.supports(&found.value),
            text: found.text,
        })
        .collect()
}

/// Every claim in `description`, unchecked.
pub fn extract_claims(description: &str) -> Vec<Claim> {
    extract(description)
        .into_iter()
        .map(|found| Claim {
            kind: found.kind,
            text: found.text,
            supported: false,
        })
        .collect()
}

/// The forbidden facts the description asserts. A forbidden string is hit
/// when the description contains it (case, spacing and punctuation
/// ignored), or states the same amount, date, or identifier in another
/// spelling: "$1,200" hits a forbidden "$1,200.00", "2026-04-03" a
/// forbidden "April 3, 2026".
pub fn forbidden_hits(description: &str, forbidden: &[String]) -> Vec<String> {
    let words = word_stream(&normalize(description));
    let stated = extract(description)
        .into_iter()
        .map(|found| found.value.key())
        .collect::<Vec<_>>();
    forbidden
        .iter()
        .filter(|value| {
            let phrase = word_stream(&normalize(value));
            (!phrase.trim().is_empty() && words.contains(&phrase))
                || extract(value).iter().any(|found| {
                    found.kind != ClaimKind::Name
                        && found
                            .value
                            .key()
                            .iter()
                            .any(|key| stated.iter().any(|keys| keys.contains(key)))
                })
        })
        .cloned()
        .collect()
}

struct Found {
    /// The index of the token the claim starts at, for listing claims in
    /// the order the description makes them.
    position: usize,
    kind: ClaimKind,
    text: String,
    value: ClaimValue,
}

enum ClaimValue {
    Number {
        canonical: String,
        value: f64,
        /// 1, or the multiplier a following "million" or "thousand" stood
        /// for, so "$4.8 million" can be held to a stated 4,812,500.00.
        scale: f64,
        decimals: i32,
    },
    /// Every ISO reading: one for a written date, up to two for a numeric
    /// date whose day and month could be either way round.
    Dates(Vec<String>),
    Month {
        year: u32,
        month: u32,
    },
    Identifier {
        canonical: String,
        digit_groups: Vec<String>,
        has_letters: bool,
    },
    Name {
        words: String,
    },
}

impl ClaimValue {
    /// What makes two claims the same fact, for spotting a forbidden one
    /// written differently.
    fn key(&self) -> Vec<String> {
        match self {
            Self::Number {
                canonical, scale, ..
            } => vec![format!("number:{canonical}:{scale}")],
            Self::Dates(dates) => dates.iter().map(|date| format!("date:{date}")).collect(),
            Self::Month { year, month } => vec![format!("month:{year}-{month:02}")],
            Self::Identifier { canonical, .. } => vec![format!("id:{canonical}")],
            Self::Name { words } => vec![format!("name:{}", words.trim())],
        }
    }
}

struct Token<'a> {
    raw: &'a str,
    core: &'a str,
}

impl Token<'_> {
    /// Whether a name cannot continue past this token: it closed a clause
    /// or a sentence. "Inc." and initials end in a full stop without ending
    /// anything.
    fn ends_phrase(&self) -> bool {
        let raw = self
            .raw
            .trim_end_matches(['"', '\'', '\u{201d}', '\u{2019}']);
        if raw.ends_with([',', ';', ':', '!', '?', ')', ']']) {
            return true;
        }
        raw.ends_with('.') && !is_abbreviation(self.core)
    }
}

fn is_abbreviation(core: &str) -> bool {
    const ABBREVIATIONS: &[&str] = &[
        "inc", "co", "corp", "ltd", "llc", "l.l.c", "lp", "l.p", "llp", "n.a", "jr", "sr", "st",
        "no", "dr", "mr", "mrs", "ms", "plc", "p.c", "pllc", "bros",
    ];
    let lower = core.to_lowercase();
    ABBREVIATIONS.contains(&lower.as_str())
        || (core.chars().count() == 1 && core.chars().all(char::is_uppercase))
}

fn tokens(text: &str) -> Vec<Token<'_>> {
    text.split_whitespace()
        .map(|raw| Token {
            raw,
            core: trim_token(raw),
        })
        .collect()
}

/// A token without the quotes, brackets and sentence punctuation around it.
fn trim_token(raw: &str) -> &str {
    raw.trim_start_matches(['(', '[', '"', '\'', '\u{201c}', '\u{2018}'])
        .trim_end_matches([
            ')', ']', '"', '\'', '\u{201d}', '\u{2019}', ',', ';', ':', '!', '?', '.',
        ])
}

fn extract(description: &str) -> Vec<Found> {
    let tokens = tokens(description);
    let mut used = vec![false; tokens.len()];
    let mut found = Vec::new();
    let text_of = |range: std::ops::Range<usize>| {
        let joined = tokens[range]
            .iter()
            .map(|token| token.raw)
            .collect::<Vec<_>>()
            .join(" ");
        trim_token(&joined).to_owned()
    };

    // Dates first, so their numbers are not claimed again as numbers.
    let mut index = 0;
    while index < tokens.len() {
        if let Some((length, value, kind)) = date_at(&tokens, index) {
            found.push(Found {
                position: index,
                kind,
                text: text_of(index..index + length),
                value,
            });
            used[index..index + length].fill(true);
            index += length;
        } else {
            index += 1;
        }
    }

    for index in 0..tokens.len() {
        if used[index] {
            continue;
        }
        let core = tokens[index].core;
        if let Some(number) = parse_number(core) {
            let next = tokens
                .get(index + 1)
                .filter(|_| !used.get(index + 1).copied().unwrap_or(true));
            let percent_word = next.is_some_and(|token| token.core.eq_ignore_ascii_case("percent"));
            let scale_word = next
                .and_then(|token| scale_word(token.core))
                .filter(|_| number.scale == 1.0);
            let (kind, length) = if percent_word {
                (ClaimKind::Percent, 2)
            } else if scale_word.is_some() {
                (number.kind, 2)
            } else {
                (number.kind, 1)
            };
            found.push(Found {
                position: index,
                kind,
                text: text_of(index..index + length),
                value: ClaimValue::Number {
                    canonical: number.canonical,
                    value: number.value,
                    scale: scale_word.unwrap_or(number.scale),
                    decimals: number.decimals,
                },
            });
            used[index..index + length].fill(true);
        } else if let Some(number) = counted_number(core) {
            // "30-day", "25th": the number is the claim, the word is not.
            found.push(Found {
                position: index,
                kind: ClaimKind::Number,
                text: text_of(index..index + 1),
                value: ClaimValue::Number {
                    value: number.parse().unwrap_or(0.0),
                    canonical: number,
                    scale: 1.0,
                    decimals: 0,
                },
            });
            used[index] = true;
        } else if is_identifier(core) {
            found.push(Found {
                position: index,
                kind: ClaimKind::Identifier,
                text: core.to_owned(),
                value: ClaimValue::Identifier {
                    canonical: canonical_identifier(core),
                    digit_groups: core
                        .split(|character: char| !character.is_ascii_digit())
                        .filter(|group| !group.is_empty())
                        .map(canonical_number)
                        .collect(),
                    has_letters: core.chars().any(char::is_alphabetic),
                },
            });
            used[index] = true;
        }
    }

    // Names: runs of two or more capitalised words among what is left.
    let mut run: Vec<usize> = Vec::new();
    let flush = |run: &mut Vec<usize>, found: &mut Vec<Found>| {
        let mut start = 0;
        while start < run.len() && is_leading_stopword(tokens[run[start]].core) {
            start += 1;
        }
        let mut end = run.len();
        while end > start && tokens[run[end - 1]].core == "&" {
            end -= 1;
        }
        let words = &run[start..end];
        if words
            .iter()
            .filter(|&&index| tokens[index].core != "&")
            .count()
            >= 2
        {
            let text = words
                .iter()
                .map(|&index| possessive_stripped(tokens[index].core))
                .collect::<Vec<_>>()
                .join(" ");
            let stream = word_stream(&normalize(&text));
            if !stream.trim().is_empty() {
                found.push(Found {
                    position: words[0],
                    kind: ClaimKind::Name,
                    text,
                    value: ClaimValue::Name { words: stream },
                });
            }
        }
        run.clear();
    };
    for (index, token) in tokens.iter().enumerate() {
        // An opening bracket starts a new phrase.
        if token.raw.starts_with(['(', '[']) {
            flush(&mut run, &mut found);
        }
        let joins =
            !used[index] && (is_name_word(token.core) || (token.core == "&" && !run.is_empty()));
        if !joins {
            flush(&mut run, &mut found);
            continue;
        }
        run.push(index);
        if token.ends_phrase() || possessive_stripped(token.core) != token.core {
            flush(&mut run, &mut found);
        }
    }
    flush(&mut run, &mut found);
    found.sort_by_key(|claim| claim.position);
    found
}

/// A date starting at `index`: how many tokens it spans, its value, and
/// whether it is a whole date or a month and year.
fn date_at(tokens: &[Token<'_>], index: usize) -> Option<(usize, ClaimValue, ClaimKind)> {
    let core = |offset: usize| tokens.get(index + offset).map(|token| token.core);
    let whole = |year: u32, month: u32, day: u32| {
        let iso = format!("{year:04}-{month:02}-{day:02}");
        is_valid_iso_date(&iso).then_some(iso)
    };
    // "March 4, 2026" / "Mar. 4th 2026"
    if let (Some(month), Some(day), Some(year)) = (
        core(0).and_then(month_number),
        core(1).and_then(day_number),
        core(2).and_then(year_number),
    ) && let Some(iso) = whole(year, month, day)
    {
        return Some((3, ClaimValue::Dates(vec![iso]), ClaimKind::Date));
    }
    // "4 March 2026"
    if let (Some(day), Some(month), Some(year)) = (
        core(0).and_then(day_number),
        core(1).and_then(month_number),
        core(2).and_then(year_number),
    ) && let Some(iso) = whole(year, month, day)
    {
        return Some((3, ClaimValue::Dates(vec![iso]), ClaimKind::Date));
    }
    // "March 2026"
    if let (Some(month), Some(year)) = (
        core(0).and_then(month_number),
        core(1).and_then(year_number),
    ) {
        return Some((2, ClaimValue::Month { year, month }, ClaimKind::Month));
    }
    // "2026-03-04", "03/04/2026"
    let readings = core(0).and_then(numeric_date)?;
    let dates = readings
        .into_iter()
        .filter_map(|(year, month, day)| whole(year, month, day))
        .collect::<Vec<_>>();
    (!dates.is_empty()).then_some((1, ClaimValue::Dates(dates), ClaimKind::Date))
}

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

fn month_number(word: &str) -> Option<u32> {
    let word = word.trim_end_matches('.').to_lowercase();
    if word == "sept" {
        return Some(9);
    }
    MONTHS
        .iter()
        .position(|month| *month == word || (word.len() == 3 && month.starts_with(&word)))
        .map(|position| position as u32 + 1)
}

fn day_number(word: &str) -> Option<u32> {
    let digits = word
        .strip_suffix("st")
        .or_else(|| word.strip_suffix("nd"))
        .or_else(|| word.strip_suffix("rd"))
        .or_else(|| word.strip_suffix("th"))
        .unwrap_or(word);
    if digits.is_empty() || digits.len() > 2 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok().filter(|day| (1..=31).contains(day))
}

fn year_number(word: &str) -> Option<u32> {
    if word.len() != 4 || !word.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    word.parse()
        .ok()
        .filter(|year| (1000..=2999).contains(year))
}

/// Every (year, month, day) reading of a numeric date: year first, or day
/// and month either way round when the numbers allow both.
fn numeric_date(word: &str) -> Option<Vec<(u32, u32, u32)>> {
    let separator = word.chars().find(|character| !character.is_ascii_digit())?;
    if !matches!(separator, '/' | '-' | '.') {
        return None;
    }
    let parts = word.split(separator).collect::<Vec<_>>();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty() || part.len() > 4 || !part.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        return None;
    }
    let numbers = parts
        .iter()
        .map(|part| part.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    if parts[0].len() == 4 {
        return Some(vec![(numbers[0], numbers[1], numbers[2])]);
    }
    let year = match parts[2].len() {
        4 => numbers[2],
        2 => 2000 + numbers[2],
        _ => return None,
    };
    let mut readings = Vec::new();
    if numbers[0] <= 12 {
        readings.push((year, numbers[0], numbers[1]));
    }
    if numbers[1] <= 12 && numbers[0] != numbers[1] {
        readings.push((year, numbers[1], numbers[0]));
    }
    (!readings.is_empty()).then_some(readings)
}

struct ParsedNumber {
    kind: ClaimKind,
    canonical: String,
    value: f64,
    decimals: i32,
    scale: f64,
}

/// An amount, a percentage, or a plain number. An amount may carry a
/// magnitude suffix: "$4.8M", "$2.5bn", "$900K".
fn parse_number(core: &str) -> Option<ParsedNumber> {
    let mut kind = ClaimKind::Number;
    let mut text = core;
    let mut scale = 1.0;
    for symbol in ["US$", "$", "\u{20ac}", "\u{a3}", "\u{a5}"] {
        if let Some(rest) = text.strip_prefix(symbol) {
            text = rest;
            kind = ClaimKind::Amount;
            break;
        }
    }
    if kind == ClaimKind::Amount {
        for (suffix, multiplier) in [
            ("bn", 1e9),
            ("B", 1e9),
            ("M", 1e6),
            ("m", 1e6),
            ("K", 1e3),
            ("k", 1e3),
        ] {
            if let Some(rest) = text.strip_suffix(suffix) {
                text = rest;
                scale = multiplier;
                break;
            }
        }
    }
    if let Some(rest) = text.strip_suffix('%') {
        text = rest;
        kind = ClaimKind::Percent;
    }
    let bytes = text.as_bytes();
    if bytes.is_empty()
        || !bytes[0].is_ascii_digit()
        || !bytes[bytes.len() - 1].is_ascii_digit()
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b',' | b'.'))
        || text.matches('.').count() > 1
    {
        return None;
    }
    let canonical = canonical_number(text);
    let decimals = canonical
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len() as i32);
    Some(ParsedNumber {
        kind,
        value: canonical.parse().ok()?,
        canonical,
        decimals,
        scale,
    })
}

fn scale_word(word: &str) -> Option<f64> {
    match word.to_lowercase().as_str() {
        "thousand" => Some(1e3),
        "million" => Some(1e6),
        "billion" => Some(1e9),
        _ => None,
    }
}

/// "30-day" -> 30, "25th" -> 25: a number counting something, written
/// into a word.
fn counted_number(core: &str) -> Option<String> {
    if let Some(day) = day_number(core).filter(|_| core.ends_with(|c: char| c.is_alphabetic())) {
        return Some(day.to_string());
    }
    let (number, rest) = core.split_once('-')?;
    (!number.is_empty()
        && number.bytes().all(|byte| byte.is_ascii_digit())
        && !rest.is_empty()
        && rest
            .split('-')
            .all(|word| !word.is_empty() && word.chars().all(|c| c.is_lowercase())))
    .then(|| canonical_number(number))
}

/// Digits with the thousands separators gone, no leading zeros, and no
/// trailing zeros after the decimal point: "4,812.50" and "4812.5" are the
/// same amount.
fn canonical_number(text: &str) -> String {
    let plain = text.replace(',', "");
    let (whole, fraction) = plain.split_once('.').unwrap_or((&plain, ""));
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let fraction = fraction.trim_end_matches('0');
    if fraction.is_empty() {
        whole.to_owned()
    } else {
        format!("{whole}.{fraction}")
    }
}

/// Every run of digits, commas and points in `text` that starts and ends
/// with a digit.
fn number_runs(text: &str) -> Vec<&str> {
    let mut runs = Vec::new();
    let mut start = None;
    for (offset, character) in text.char_indices() {
        let part = character.is_ascii_digit() || matches!(character, ',' | '.');
        match (start, part) {
            (None, true) if character.is_ascii_digit() => start = Some(offset),
            (Some(begin), false) => {
                runs.push(text[begin..offset].trim_end_matches([',', '.']));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(begin) = start {
        runs.push(text[begin..].trim_end_matches([',', '.']));
    }
    runs
}

/// The numbers a run of digits states: the run itself, read as one
/// number, and - when it cannot be one ordinary number, like a dotted date
/// or a comma-separated list - each of its groups as well.
fn run_numbers(run: &str) -> Vec<String> {
    let mut numbers = Vec::new();
    let ordinary = run.matches('.').count() <= 1
        && run
            .split('.')
            .next()
            .unwrap_or_default()
            .split(',')
            .skip(1)
            .all(|group| group.len() == 3);
    if ordinary {
        numbers.push(canonical_number(run));
    }
    if !ordinary || run.contains(',') {
        for group in run.split([',', '.']).filter(|group| !group.is_empty()) {
            numbers.push(canonical_number(group));
        }
    }
    numbers
}

fn is_identifier(core: &str) -> bool {
    core.chars().any(|character| character.is_ascii_digit())
        && core.chars().count() >= 2
        && core.chars().all(|character| {
            character.is_alphanumeric() || matches!(character, '-' | '/' | '#' | '.' | '_')
        })
}

fn canonical_identifier(core: &str) -> String {
    core.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect()
}

fn is_name_word(core: &str) -> bool {
    let mut characters = core.chars();
    characters.next().is_some_and(char::is_uppercase)
        && core.chars().all(|character| {
            character.is_alphabetic() || matches!(character, '\'' | '\u{2019}' | '-' | '.' | '&')
        })
        // A month a date did not claim ("March rent") is not a name.
        && month_number(core).is_none()
}

fn possessive_stripped(core: &str) -> &str {
    core.strip_suffix("'s")
        .or_else(|| core.strip_suffix("\u{2019}s"))
        .unwrap_or(core)
}

/// Words that open a sentence or a phrase in capitals without being part
/// of a name.
fn is_leading_stopword(core: &str) -> bool {
    const STOPWORDS: &[&str] = &[
        "a", "an", "the", "this", "that", "these", "those", "its", "their", "our", "his", "her",
        "on", "in", "for", "from", "to", "by", "between", "with", "and", "of", "as", "at", "per",
        "under", "dated", "signed", "issued", "sent", "re",
    ];
    STOPWORDS.contains(&core.to_lowercase().as_str())
}

/// Lower-case words separated by single spaces, punctuation dropped, with a
/// space at either end so a phrase only matches whole words.
fn word_stream(normalized: &str) -> String {
    let mut stream = String::from(" ");
    for word in normalized
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        stream.extend(word.chars().flat_map(char::to_lowercase));
        stream.push(' ');
    }
    stream
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(description: &str) -> Vec<(ClaimKind, String)> {
        extract_claims(description)
            .into_iter()
            .map(|claim| (claim.kind, claim.text))
            .collect()
    }

    #[test]
    fn money_with_commas_percentages_and_plain_numbers_are_claims() {
        assert_eq!(
            kinds("Invoice for $4,812.50, a 12.5% surcharge, 3 crates and 7 percent tax."),
            vec![
                (ClaimKind::Amount, "$4,812.50".to_owned()),
                (ClaimKind::Percent, "12.5%".to_owned()),
                (ClaimKind::Number, "3".to_owned()),
                (ClaimKind::Percent, "7 percent".to_owned()),
            ]
        );
        assert_eq!(canonical_number("4,812.50"), "4812.5");
        assert_eq!(canonical_number("0093"), "93");
        assert_eq!(canonical_number("1,200.00"), "1200");
        assert_eq!(canonical_number("0.50"), "0.5");
    }

    #[test]
    fn identifiers_keep_their_letters_and_counted_numbers_lose_their_words() {
        assert_eq!(
            kinds(
                "Invoice INV-20417 against PO-88213 for account 4471-0093, net 30-day terms, 2nd notice."
            ),
            vec![
                (ClaimKind::Identifier, "INV-20417".to_owned()),
                (ClaimKind::Identifier, "PO-88213".to_owned()),
                (ClaimKind::Identifier, "4471-0093".to_owned()),
                (ClaimKind::Number, "30-day".to_owned()),
                (ClaimKind::Number, "2nd".to_owned()),
            ]
        );
    }

    #[test]
    fn dates_in_words_and_numbers_are_one_claim_each() {
        assert_eq!(
            kinds(
                "Notice dated March 4, 2026, effective 1 April 2026, replacing the 2025-12-31 schedule, for March 2026 and 03/04/2026."
            ),
            vec![
                (ClaimKind::Date, "March 4, 2026".to_owned()),
                (ClaimKind::Date, "1 April 2026".to_owned()),
                (ClaimKind::Date, "2025-12-31".to_owned()),
                (ClaimKind::Month, "March 2026".to_owned()),
                (ClaimKind::Date, "03/04/2026".to_owned()),
            ]
        );
    }

    #[test]
    fn names_are_runs_of_capitalised_words_without_their_leading_article() {
        assert_eq!(
            kinds(
                "The Halvorsen Fixture Works LLC invoice to Quillon Ridge Bakery for Display Fixtures, approved by Marta Quillon."
            ),
            vec![
                (ClaimKind::Name, "Halvorsen Fixture Works LLC".to_owned()),
                (ClaimKind::Name, "Quillon Ridge Bakery".to_owned()),
                (ClaimKind::Name, "Display Fixtures".to_owned()),
                (ClaimKind::Name, "Marta Quillon".to_owned()),
            ]
        );
        // One capitalised word is not a name claim; "Inc." does not end one;
        // a comma does.
        assert_eq!(
            kinds("Invoice from Brightwater Tooling, Inc. and Ostrander & Vey."),
            vec![
                (ClaimKind::Name, "Brightwater Tooling".to_owned()),
                (ClaimKind::Name, "Ostrander & Vey".to_owned()),
            ]
        );
    }

    fn document() -> DocumentText {
        DocumentText::new([
            "HALVORSEN FIXTURE WORKS, LLC\nRemit to: 18 Kettle Lane\nInvoice No. INV 20417\n\
             Invoice Date: March 4, 2026   Due Date: 04/03/2026\nBill To: Quillon Ridge Bakery\n\
             Display fixtures, installed: 4,812.50\nSurcharge 12.5%\nAccount 4471-0093",
            "Total contract value 4,812,500.00 over 3 years",
        ])
    }

    fn unsupported(description: &str) -> Vec<String> {
        check_claims(description, &document())
            .into_iter()
            .filter(|claim| !claim.supported)
            .map(|claim| claim.text)
            .collect()
    }

    #[test]
    fn a_claim_is_supported_by_the_same_fact_in_any_spelling() {
        assert!(
            unsupported(
                "Invoice INV-20417 from Halvorsen Fixture Works LLC to Quillon Ridge Bakery dated \
                 2026-03-04 for $4,812.5 with a 12.5% surcharge on account 4471-0093, a $4.8 million \
                 contract over 3 years, March 2026."
            )
            .is_empty()
        );
    }

    #[test]
    fn an_invented_fact_is_unsupported() {
        assert_eq!(
            unsupported(
                "Invoice INV-20418 from Halvorsen Fixture Works to Brightwater Tooling for $1,200.00 \
                 dated March 5, 2026, covering April 2027."
            ),
            vec![
                "INV-20418",
                "Brightwater Tooling",
                "$1,200.00",
                "March 5, 2026",
                "April 2027"
            ]
        );
    }

    #[test]
    fn forbidden_facts_are_caught_in_another_spelling() {
        let forbidden = vec![
            "$1,200.00".to_owned(),
            "April 3, 2026".to_owned(),
            "Quillon Ridge Bakery".to_owned(),
        ];
        assert_eq!(
            forbidden_hits("Invoice for $1,200 due 2026-04-03.", &forbidden),
            vec!["$1,200.00", "April 3, 2026"]
        );
        assert_eq!(
            forbidden_hits("Invoice billed to QUILLON RIDGE BAKERY.", &forbidden),
            vec!["Quillon Ridge Bakery"]
        );
        assert!(forbidden_hits("Invoice for $1,250.", &forbidden).is_empty());
    }
}
