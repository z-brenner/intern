//! Which lines are worth reading a second time, and which reading to keep.
//!
//! A misread costs most where Intern's answer comes from: a date, an
//! amount, an invoice number, the name of a company. The recognizer says
//! how sure it was of every character, so a line holding one of those with
//! a character it was unsure of is cut out again and re-read with
//! different preprocessing. Re-reading a whole page would double the cost
//! of OCR for the few characters that matter; this re-reads a handful of
//! lines.

/// A line whose least certain character is below this is a candidate.
pub const DEFAULT_MIN_CHAR_PROBABILITY: f32 = 0.9;

/// At most this many lines of one page are re-read.
pub const DEFAULT_MAX_LINES: usize = 12;

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

/// The month a word names, 1-12, by full name or three-letter
/// abbreviation ("Sept" too), ignoring case and a trailing full stop or
/// comma.
fn month_number(word: &str) -> Option<u32> {
    let word = word.trim_end_matches(['.', ',']).to_ascii_lowercase();
    if word == "sept" {
        return Some(9);
    }
    if word.len() < 3 {
        return None;
    }
    MONTHS
        .iter()
        .position(|month| *month == word || (word.len() == 3 && month.starts_with(&word)))
        .map(|index| index as u32 + 1)
}

fn has_digit(word: &str) -> bool {
    word.chars().any(|c| c.is_ascii_digit())
}

/// Whether the line holds something a misread would cost: a date, an
/// amount, an identifier, or a capitalised name of two or more words.
pub fn holds_critical_field(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    for (index, word) in words.iter().enumerate() {
        if month_number(word).is_some()
            && words
                .get(index + 1)
                .or_else(|| index.checked_sub(1).and_then(|before| words.get(before)))
                .is_some_and(|next| has_digit(next))
        {
            return true;
        }
        if numeric_date(word).is_some() || looks_like_amount(word) || looks_like_identifier(word) {
            return true;
        }
    }
    capitalised_run(&words) >= 2
}

/// The longest run of consecutive words that start with a capital letter
/// and continue in letters - "Harbourline Freight Partners".
fn capitalised_run(words: &[&str]) -> usize {
    let mut best = 0;
    let mut run = 0;
    for word in words {
        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
        let mut chars = word.chars();
        let capitalised = chars.next().is_some_and(char::is_uppercase)
            && word.chars().count() >= 2
            && word
                .chars()
                .all(|c| c.is_alphabetic() || c == '\'' || c == '-');
        run = if capitalised { run + 1 } else { 0 };
        best = best.max(run);
    }
    best
}

fn looks_like_amount(word: &str) -> bool {
    let body = word
        .trim_start_matches(['$', '€', '£', '('])
        .trim_end_matches([')', ',', '.']);
    if body.len() == word.len() && !body.contains(',') && !body.contains('.') {
        return false;
    }
    let digits = body.chars().filter(char::is_ascii_digit).count();
    digits >= 2
        && body
            .chars()
            .all(|c| c.is_ascii_digit() || c == ',' || c == '.')
        && (word.starts_with(['$', '€', '£'])
            || body.contains(',')
            || body
                .rsplit('.')
                .next()
                .is_some_and(|cents| cents.len() == 2 && body.contains('.')))
}

/// A token of four or more characters, at least two of them digits, that
/// mixes in letters or separators: INV-20417, PO 4471-B, A12-345678, #883104.
/// A plain number of five or more digits also counts: account numbers are
/// often printed bare.
fn looks_like_identifier(word: &str) -> bool {
    let word = word.trim_matches(|c: char| matches!(c, ',' | '.' | ';' | ':' | '(' | ')'));
    let digits = word.chars().filter(char::is_ascii_digit).count();
    let letters = word.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let separators = word
        .chars()
        .filter(|c| matches!(c, '-' | '/' | '#' | '_'))
        .count();
    if word.chars().count() < 4 || digits < 2 {
        return false;
    }
    (letters > 0 || separators > 0) || digits >= 5
}

/// Whether the parts of a `/`, `-` or `.` separated word have the lengths
/// a date's do: day and month of one or two characters and a year of two
/// or four, or a four-character year first.
fn date_shaped(parts: &[&str]) -> bool {
    let lengths: Vec<usize> = parts.iter().map(|part| part.chars().count()).collect();
    matches!(
        lengths.as_slice(),
        [4, 1..=2, 1..=2] | [1..=2, 1..=2, 2 | 4]
    )
}

fn date_parts(word: &str) -> Vec<&str> {
    word.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '(' | ')'))
        .split(['/', '-', '.'])
        .collect()
}

/// Year, month and day of a numeric date written with `/`, `-` or `.`, in
/// either day-month or month-day order, or year first. `None` when the
/// word is not shaped like one.
fn numeric_date(word: &str) -> Option<(u32, u32, u32)> {
    let parts = date_parts(word);
    if !date_shaped(&parts)
        || !parts
            .iter()
            .all(|part| part.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    let numbers: Vec<u32> = parts.iter().map(|part| part.parse().unwrap_or(0)).collect();
    if parts[0].len() == 4 {
        return Some((numbers[0], numbers[1], numbers[2]));
    }
    let year = if parts[2].len() == 2 {
        2000 + numbers[2]
    } else {
        numbers[2]
    };
    // Month first if it can be; otherwise day first.
    if numbers[0] <= 12 {
        Some((year, numbers[0], numbers[1]))
    } else {
        Some((year, numbers[1], numbers[0]))
    }
}

/// A word shaped like a numeric date with a letter or two where digits
/// belong: 03/1O/2026, 2O26-01-31. An identifier such as INV-2041-7 has too
/// many letters to be one.
fn broken_numeric_date(word: &str) -> bool {
    let parts = date_parts(word);
    let letters = parts
        .iter()
        .flat_map(|part| part.chars())
        .filter(char::is_ascii_alphabetic)
        .count();
    let digits = parts
        .iter()
        .flat_map(|part| part.chars())
        .filter(char::is_ascii_digit)
        .count();
    let misread_glyph = date_shaped(&parts)
        && (1..=2).contains(&letters)
        && digits >= 3
        && parts
            .iter()
            .flat_map(|part| part.chars())
            .all(|c| c.is_ascii_alphanumeric());
    // A speck read as a separator: 0.7/01/2026. Slashes and a four-digit
    // year say date; the parts no longer do.
    let extra_separator = word.contains('/')
        && parts.len() > 3
        && parts.iter().any(|part| part.len() == 4)
        && parts
            .iter()
            .all(|part| part.chars().all(|c| c.is_ascii_digit()));
    misread_glyph || extra_separator
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_calendar_date(year: u32, month: u32, day: u32) -> bool {
    (1900..=2199).contains(&year) && day >= 1 && day <= days_in_month(year, month)
}

/// Whether every date the line seems to hold is a real calendar date. A
/// line with no date in it passes.
///
/// A month name followed by a day and a year - "March 4, 2026", "4 March
/// 2026" - must have a numeric day and a four-digit numeric year that fit
/// the month: "March 4, 2O26" and "June 31, 2026" fail. A numeric date must
/// name a real day in some order.
pub fn dates_parse(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    let trim = |word: &str| word.trim_end_matches([',', '.', ';']).to_owned();
    let day_like = |word: &str| {
        let word = trim(word);
        let word = strip_ordinal(&word);
        (1..=2).contains(&word.chars().count()) && has_digit(word)
    };
    let year_like = |word: &str| trim(word).chars().count() >= 3 && has_digit(word);
    for (index, word) in words.iter().enumerate() {
        if let Some(month) = month_number(word) {
            let before = index.checked_sub(1).map(|before| words[before]);
            let after = words.get(index + 1).copied();
            let fits = match (before, after) {
                // "4 March 2026"
                (Some(day), Some(year)) if day_like(day) && year_like(year) => {
                    day_and_year_fit(month, &trim(day), Some(&trim(year)))
                }
                // "March 4, 2026", "March 4"
                (_, Some(day)) if day_like(day) => day_and_year_fit(
                    month,
                    &trim(day),
                    words
                        .get(index + 2)
                        .filter(|year| year_like(year))
                        .map(|year| trim(year))
                        .as_deref(),
                ),
                // "March 2026"
                (_, Some(year)) if year_like(year) => trim(year)
                    .parse::<u32>()
                    .is_ok_and(|year| (1900..=2199).contains(&year)),
                _ => true,
            };
            if !fits {
                return false;
            }
        }
        if let Some((year, month, day)) = numeric_date(word) {
            let swapped = month <= 12 && is_calendar_date(year, day, month);
            if !is_calendar_date(year, month, day) && !swapped {
                return false;
            }
        } else if broken_numeric_date(word) {
            return false;
        }
    }
    true
}

fn strip_ordinal(word: &str) -> &str {
    for suffix in ["st", "nd", "rd", "th"] {
        if let Some(stripped) = word.strip_suffix(suffix)
            && stripped.chars().all(|c| c.is_ascii_digit())
            && !stripped.is_empty()
        {
            return stripped;
        }
    }
    word
}

fn day_and_year_fit(month: u32, day: &str, year: Option<&str>) -> bool {
    let Ok(day) = strip_ordinal(day).parse::<u32>() else {
        return false;
    };
    match year {
        Some(year) => match year.parse::<u32>() {
            Ok(year) => is_calendar_date(year, month, day),
            Err(_) => false,
        },
        // "March 4" with no year: only the day can be checked.
        None => day >= 1 && day <= days_in_month(2024, month),
    }
}

/// One reading of a line and how sure the recognizer was of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub text: String,
    pub mean_probability: f32,
}

/// Of the readings of one line, the index of the one to keep: a reading
/// whose dates are all real dates beats one whose are not, and then the
/// more confident one wins. A tie keeps the earlier reading, so the first
/// pass stands unless something is better than it.
pub fn best_reading(readings: &[Reading]) -> usize {
    let mut best = 0;
    for (index, reading) in readings.iter().enumerate().skip(1) {
        let incumbent = &readings[best];
        let (parses, incumbent_parses) = (dates_parse(&reading.text), dates_parse(&incumbent.text));
        let better = match (parses, incumbent_parses) {
            (true, false) => !reading.text.trim().is_empty(),
            (false, true) => false,
            _ => reading.mean_probability > incumbent.mean_probability,
        };
        if better {
            best = index;
        }
    }
    best
}

/// The lines of a page to re-read, least certain first, at most `limit`,
/// from each line's text and the probability of its least certain
/// character: a critical field read with an uncertain character, or a
/// date that is not a date however sure the recognizer was of it.
pub fn lines_to_reread(
    lines: &[(&str, f32)],
    min_char_probability: f32,
    limit: usize,
) -> Vec<usize> {
    let mut chosen: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, (text, least))| {
            (*least < min_char_probability && holds_critical_field(text)) || !dates_parse(text)
        })
        .map(|(index, _)| index)
        .collect();
    chosen.sort_by(|&a, &b| lines[a].1.total_cmp(&lines[b].1).then(a.cmp(&b)));
    chosen.truncate(limit);
    chosen.sort_unstable();
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_fields_are_recognised() {
        for line in [
            "Invoice Date: March 4, 2026",
            "Dated 4 March 2026",
            "Due 03/14/2026",
            "Period 2026-01-31",
            "Total due $4,417.20",
            "Amount 12,300.00",
            "Invoice No. INV-20417",
            "Account 4471093",
            "PO #883104",
            "Harbourline Freight Partners LLC",
            "Remit to Quillon Fixture Works",
        ] {
            assert!(holds_critical_field(line), "{line}");
        }
        for line in [
            "the tenant shall keep the premises in good repair",
            "Page 3",
            "Signature",
            "of 12",
        ] {
            assert!(!holds_critical_field(line), "{line}");
        }
    }

    #[test]
    fn real_dates_parse_and_misreads_do_not() {
        for line in [
            "March 4, 2026",
            "Effective 29 February 2024",
            "Due 03/14/2026",
            "14.03.2026",
            "2026-01-31",
            "Sept. 30, 2025",
            "no date here",
            "Invoice INV-2041-7",
            "Call 555-123-4567",
            "Staples 1-3/4 in.",
        ] {
            assert!(dates_parse(line), "{line}");
        }
        for line in [
            "March 4, 2O26",
            "June 31, 2026",
            "Effective 29 February 2025",
            "Due 13/32/2026",
            "03/1O/2026",
            "March 4, 202",
            "Paid 0.7/01/2026",
        ] {
            assert!(!dates_parse(line), "{line}");
        }
    }

    #[test]
    fn a_reading_with_a_real_date_beats_a_more_confident_one_without() {
        let readings = [
            Reading {
                text: "Date: March 4, 2O26".into(),
                mean_probability: 0.93,
            },
            Reading {
                text: "Date: March 4, 2026".into(),
                mean_probability: 0.90,
            },
        ];
        assert_eq!(best_reading(&readings), 1);
    }

    #[test]
    fn otherwise_the_more_confident_reading_wins_and_ties_keep_the_first() {
        let reading = |text: &str, probability| Reading {
            text: text.into(),
            mean_probability: probability,
        };
        assert_eq!(
            best_reading(&[reading("INV-2O417", 0.8), reading("INV-20417", 0.95)]),
            1
        );
        assert_eq!(
            best_reading(&[reading("INV-20417", 0.95), reading("INV-2O417", 0.95)]),
            0
        );
        // An empty reading never wins on parsing alone.
        assert_eq!(
            best_reading(&[reading("June 31, 2026", 0.9), reading(" ", 0.99)]),
            0
        );
    }

    #[test]
    fn rereading_is_bounded_and_least_certain_first() {
        let lines = [
            ("Invoice INV-20417", 0.5),
            ("plain prose here", 0.2),
            ("Total $1,200.00", 0.95),
            ("Due March 4, 2026", 0.7),
            ("Bill to Marta Quillon", 0.6),
            // Read with certainty, and still not a date.
            ("Paid June 31, 2026", 0.99),
        ];
        assert_eq!(lines_to_reread(&lines, 0.9, 10), vec![0, 3, 4, 5]);
        assert_eq!(lines_to_reread(&lines, 0.9, 2), vec![0, 4]);
        assert_eq!(lines_to_reread(&lines, 0.4, 10), vec![5]);
    }
}
