//! Literal-evidence checking.
//!
//! Every fact that can reach a filename must be backed by a verbatim excerpt
//! the model quotes from the digest. Because distillation keeps text verbatim,
//! "the model quoted something that is actually in the document" is a check
//! Intern can make locally and cheaply, and it is the main reason a proposal
//! can be trusted without a human reading it.

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
pub fn normalize(value: &str) -> String {
    let normalized = value.nfkc().case_fold().map(|character| match character {
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
/// a name a person would recognise as the same name should not be thrown out
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
    let candidates = date_spellings(iso_date);
    let bytes = normalized.as_bytes();
    let mut statements = Vec::new();
    for (candidate, spelling) in &candidates {
        let candidate = normalize(candidate);
        if candidate.is_empty() {
            continue;
        }
        let mut from = 0;
        while let Some(found) = normalized[from..].find(&candidate) {
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
