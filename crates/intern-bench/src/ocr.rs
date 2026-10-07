//! How close OCR came to what was drawn on a scanned page.
//!
//! Distances are Levenshtein edit distances, page by page, over the text
//! with whitespace collapsed - line breaks and spacing are layout, not
//! reading. A corpus figure is the sum of the distances over the sum of the
//! truth lengths, never a mean of per-page rates, so one short page cannot
//! outweigh a long one. Every comparison is per page with a two-row table:
//! a 100-page scan compared as one string would cost the square of its
//! whole length.

use serde::{Deserialize, Serialize};

use intern_engine::{DocumentSource, PageOrigin};

use crate::gold::OcrTruth;

/// Edit distance between two sequences: insertions, deletions, and
/// substitutions each cost one. O(n·m) time, O(min(n, m)) memory.
pub fn levenshtein<T: PartialEq>(left: &[T], right: &[T]) -> usize {
    let (long, short) = if left.len() >= right.len() {
        (left, right)
    } else {
        (right, left)
    };
    if short.is_empty() {
        return long.len();
    }
    let mut previous: Vec<usize> = (0..=short.len()).collect();
    let mut current = vec![0; short.len() + 1];
    for (row, long_item) in long.iter().enumerate() {
        current[0] = row + 1;
        for (column, short_item) in short.iter().enumerate() {
            let substitution = previous[column] + usize::from(long_item != short_item);
            let deletion = previous[column + 1] + 1;
            let insertion = current[column] + 1;
            current[column + 1] = substitution.min(deletion).min(insertion);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[short.len()]
}

/// The text with every run of whitespace made one space, trimmed.
pub fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Character edit distance and the truth's length in characters.
pub fn char_distance(truth: &str, read: &str) -> (usize, usize) {
    let truth = collapse_whitespace(truth).chars().collect::<Vec<_>>();
    let read = collapse_whitespace(read).chars().collect::<Vec<_>>();
    (levenshtein(&truth, &read), truth.len())
}

/// Word edit distance and the truth's length in words.
pub fn word_distance(truth: &str, read: &str) -> (usize, usize) {
    let truth = truth.split_whitespace().collect::<Vec<_>>();
    let read = read.split_whitespace().collect::<Vec<_>>();
    (levenshtein(&truth, &read), truth.len())
}

/// Distance over length, or nothing when there was nothing to read. A rate
/// over an empty truth is not zero errors; it is no measurement.
pub fn rate(distance: usize, length: usize) -> Option<f64> {
    (length > 0).then(|| distance as f64 / length as f64)
}

/// Every OCR figure for one document.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OcrMeasure {
    /// Truth pages compared with a page the extractor returned.
    pub pages_compared: usize,
    /// Truth pages of a document the extractor returned without them (a
    /// TIFF frame it does not read, say). Reported, not folded into the
    /// rates, and the dates, names and identifiers drawn only on them are
    /// left out of the targeted accuracies: no reading could have found
    /// them.
    pub pages_missing: usize,
    /// Truth pages of a scan whose extraction failed, so nothing was read
    /// at all. Each counts as read empty - every character, word, date,
    /// name and identifier on it missed - so a scan the reader fails on
    /// counts against the figures instead of leaving their denominator.
    #[serde(default)]
    pub pages_failed: usize,
    pub char_distance: usize,
    pub char_distance_ci: usize,
    pub truth_chars: usize,
    pub word_distance: usize,
    pub truth_words: usize,
    pub dates_found: usize,
    pub dates_total: usize,
    pub names_found: usize,
    pub names_total: usize,
    pub identifiers_found: usize,
    pub identifiers_total: usize,
    /// Mean of the worker's per-page OCR confidence over the compared
    /// pages that reported one.
    pub mean_confidence: Option<f64>,
}

impl OcrMeasure {
    pub fn cer(&self) -> Option<f64> {
        rate(self.char_distance, self.truth_chars)
    }

    pub fn cer_ci(&self) -> Option<f64> {
        rate(self.char_distance_ci, self.truth_chars)
    }

    pub fn wer(&self) -> Option<f64> {
        rate(self.word_distance, self.truth_words)
    }

    pub fn date_accuracy(&self) -> Option<f64> {
        fraction(self.dates_found, self.dates_total)
    }

    pub fn name_accuracy(&self) -> Option<f64> {
        fraction(self.names_found, self.names_total)
    }

    pub fn identifier_accuracy(&self) -> Option<f64> {
        fraction(self.identifiers_found, self.identifiers_total)
    }

    /// A scanned document nothing was read from because its extraction
    /// failed: every truth page read as empty, every targeted value missed.
    pub fn unread(truth: &OcrTruth) -> Self {
        let mut measure = Self {
            pages_failed: truth.pages.len(),
            dates_total: truth.dates.len(),
            names_total: truth.names.len(),
            identifiers_total: truth.identifiers.len(),
            ..Self::default()
        };
        for page in &truth.pages {
            let (distance, length) = char_distance(&page.text, "");
            measure.char_distance += distance;
            measure.char_distance_ci += distance;
            measure.truth_chars += length;
            let (words, word_count) = word_distance(&page.text, "");
            measure.word_distance += words;
            measure.truth_words += word_count;
        }
        measure
    }

    /// Adds another document's counts, for a corpus figure.
    pub fn accumulate(&mut self, other: &Self) {
        self.pages_compared += other.pages_compared;
        self.pages_missing += other.pages_missing;
        self.pages_failed += other.pages_failed;
        self.char_distance += other.char_distance;
        self.char_distance_ci += other.char_distance_ci;
        self.truth_chars += other.truth_chars;
        self.word_distance += other.word_distance;
        self.truth_words += other.truth_words;
        self.dates_found += other.dates_found;
        self.dates_total += other.dates_total;
        self.names_found += other.names_found;
        self.names_total += other.names_total;
        self.identifiers_found += other.identifiers_found;
        self.identifiers_total += other.identifiers_total;
    }
}

fn fraction(found: usize, total: usize) -> Option<f64> {
    (total > 0).then(|| found as f64 / total as f64)
}

/// Compares what the extractor returned for each scanned page with what was
/// drawn there.
///
/// The targeted checks ask whether each date, name, and identifier drawn on
/// the compared pages survives somewhere in the read text: dates and names
/// compared ignoring case (a capitalised month is the same date),
/// identifiers exactly, because `INV-2O417` is not `INV-20417`. A value
/// drawn only on a page the extractor did not return is not counted: no
/// reading of the returned pages could have found it.
pub fn measure(truth: &OcrTruth, source: &DocumentSource) -> OcrMeasure {
    let mut measure = OcrMeasure::default();
    let mut read_pages = Vec::new();
    let mut truth_pages = Vec::new();
    let mut confidences = Vec::new();
    for page in &truth.pages {
        let Some(read) = source
            .pages
            .iter()
            .find(|candidate| candidate.page_number == page.page)
        else {
            measure.pages_missing += 1;
            continue;
        };
        measure.pages_compared += 1;
        let (distance, length) = char_distance(&page.text, &read.text);
        measure.char_distance += distance;
        measure.truth_chars += length;
        let (distance_ci, _) = char_distance(&page.text.to_lowercase(), &read.text.to_lowercase());
        measure.char_distance_ci += distance_ci;
        let (words, word_count) = word_distance(&page.text, &read.text);
        measure.word_distance += words;
        measure.truth_words += word_count;
        if read.origin == PageOrigin::Ocr
            && let Some(confidence) = read.ocr_confidence
        {
            confidences.push(f64::from(confidence));
        }
        read_pages.push(read.text.as_str());
        truth_pages.push(page.text.as_str());
    }
    let read = collapse_whitespace(&read_pages.join(" "));
    let read_folded = read.to_lowercase();
    let drawn = collapse_whitespace(&truth_pages.join(" "));
    let drawn_folded = drawn.to_lowercase();
    // Each value counts when it is drawn on a compared page, and is found
    // when the reading has it too: dates and names ignoring case,
    // identifiers exactly.
    let count = |values: &[String], fold: bool| {
        let form = |value: &str| {
            let collapsed = collapse_whitespace(value);
            if fold {
                collapsed.to_lowercase()
            } else {
                collapsed
            }
        };
        let (drawn, read) = if fold {
            (&drawn_folded, &read_folded)
        } else {
            (&drawn, &read)
        };
        let on_compared = values
            .iter()
            .map(|value| form(value))
            .filter(|value| drawn.contains(value.as_str()))
            .collect::<Vec<_>>();
        let found = on_compared
            .iter()
            .filter(|value| read.contains(value.as_str()))
            .count();
        (found, on_compared.len())
    };
    (measure.dates_found, measure.dates_total) = count(&truth.dates, true);
    (measure.names_found, measure.names_total) = count(&truth.names, true);
    (measure.identifiers_found, measure.identifiers_total) = count(&truth.identifiers, false);
    measure.mean_confidence = (!confidences.is_empty())
        .then(|| confidences.iter().sum::<f64>() / confidences.len() as f64);
    measure
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gold::OcrTruthPage;
    use intern_engine::SourcePage;

    fn chars(value: &str) -> Vec<char> {
        value.chars().collect()
    }

    #[test]
    fn levenshtein_counts_single_edits_and_handles_empty_sides() {
        assert_eq!(levenshtein(&chars("kitten"), &chars("sitting")), 3);
        assert_eq!(levenshtein(&chars("sitting"), &chars("kitten")), 3);
        assert_eq!(levenshtein(&chars("flaw"), &chars("lawn")), 2);
        assert_eq!(levenshtein(&chars(""), &chars("abc")), 3);
        assert_eq!(levenshtein(&chars("abc"), &chars("")), 3);
        assert_eq!(levenshtein::<char>(&[], &[]), 0);
        assert_eq!(levenshtein(&chars("same"), &chars("same")), 0);
    }

    #[test]
    fn levenshtein_works_on_characters_not_bytes() {
        // One accented letter is one character, however many bytes it takes.
        assert_eq!(levenshtein(&chars("café"), &chars("cafe")), 1);
        assert_eq!(levenshtein(&chars("契約書"), &chars("契約")), 1);
        let (distance, length) = char_distance("Zoë Ångström", "Zoe Angstrom");
        assert_eq!((distance, length), (3, 12));
    }

    #[test]
    fn whitespace_is_layout_and_case_is_not() {
        let (distance, length) = char_distance("INVOICE  No.\n4417", "INVOICE No. 4417");
        assert_eq!((distance, length), (0, 16));
        let (distance, _) = char_distance("Invoice", "INVOICE");
        assert_eq!(distance, 6, "case-sensitive: six letters differ");
        assert_eq!(char_distance("", "").1, 0);
        assert_eq!(rate(0, 0), None, "no truth, no rate");
        assert_eq!(rate(3, 12), Some(0.25));
    }

    #[test]
    fn word_error_rate_counts_whole_words() {
        let (distance, length) = word_distance(
            "Remit to Halvorsen Fixture Works",
            "Rernit to Halvorsen Fixture",
        );
        assert_eq!((distance, length), (2, 5), "one misread word, one dropped");
        assert_eq!(word_distance("", "anything"), (1, 0));
    }

    fn scanned(pages: &[(usize, &str, Option<u32>)]) -> DocumentSource {
        DocumentSource::from_pages(
            pages
                .iter()
                .map(|(number, text, confidence)| {
                    let mut page = SourcePage::new(*number, *text, PageOrigin::Ocr);
                    page.ocr_confidence = *confidence;
                    page
                })
                .collect(),
        )
    }

    #[test]
    fn a_document_is_measured_page_by_page_with_targeted_checks() {
        let truth = OcrTruth {
            pages: vec![
                OcrTruthPage {
                    page: 1,
                    text: "INVOICE INV-20417\nDate: March 4, 2026".into(),
                },
                OcrTruthPage {
                    page: 2,
                    text: "Bill to Marta Quillon".into(),
                },
                OcrTruthPage {
                    page: 3,
                    text: "never read".into(),
                },
            ],
            dates: vec!["March 4, 2026".into()],
            names: vec!["Marta Quillon".into()],
            identifiers: vec!["INV-20417".into(), "PO-88213".into()],
        };
        let source = scanned(&[
            (1, "INVOICE INV-2O417 Date: MARCH 4, 2026", Some(80)),
            (2, "Bill to marta Quillon", Some(90)),
        ]);
        let measure = measure(&truth, &source);
        assert_eq!(measure.pages_compared, 2);
        assert_eq!(measure.pages_missing, 1, "page 3 was never returned");
        // O for 0, and four letters of MARCH; then one lower-case m.
        assert_eq!(measure.char_distance, 1 + 4 + 1);
        assert_eq!(measure.char_distance_ci, 1);
        assert_eq!(measure.truth_chars, 37 + 21);
        assert_eq!(measure.date_accuracy(), Some(1.0), "case-insensitive");
        assert_eq!(measure.name_accuracy(), Some(1.0));
        assert_eq!(
            measure.identifier_accuracy(),
            Some(0.0),
            "exact, and both missed"
        );
        assert_eq!(measure.mean_confidence, Some(85.0));
        assert!(measure.cer().unwrap() > measure.cer_ci().unwrap());

        let nothing = super::measure(&truth, &scanned(&[(9, "unrelated", None)]));
        assert_eq!((nothing.pages_compared, nothing.pages_missing), (0, 3));
        assert_eq!(nothing.cer(), None, "nothing compared, no rate");
        assert_eq!(nothing.dates_total, 0);
    }

    /// A fax whose second frame the reader never returns: what is drawn
    /// only there is reported as unread and left out of the accuracies, so
    /// a perfect reading of the first frame scores perfectly.
    #[test]
    fn values_drawn_only_on_an_unread_page_are_not_counted() {
        let truth = OcrTruth {
            pages: vec![
                OcrTruthPage {
                    page: 1,
                    text: "FAX Date: August 12, 2026 Re: N0612X".into(),
                },
                OcrTruthPage {
                    page: 2,
                    text: "QUOTATION No. KRA-Q-2611 Valid until: September 11, 2026".into(),
                },
            ],
            dates: vec!["August 12, 2026".into(), "September 11, 2026".into()],
            names: Vec::new(),
            identifiers: vec!["KRA-Q-2611".into(), "N0612X".into()],
        };
        let source = scanned(&[(1, "FAX Date: August 12, 2026 Re: N0612X", Some(90))]);
        let measure = measure(&truth, &source);
        assert_eq!(measure.pages_missing, 1);
        assert_eq!((measure.dates_found, measure.dates_total), (1, 1));
        assert_eq!(
            (measure.identifiers_found, measure.identifiers_total),
            (1, 1)
        );
        assert_eq!(measure.date_accuracy(), Some(1.0));
        assert_eq!(measure.cer(), Some(0.0));
    }

    /// A scan the extractor failed on reads as empty: a total miss that
    /// counts against the pooled figures.
    #[test]
    fn a_failed_scan_counts_as_read_empty() {
        let truth = OcrTruth {
            pages: vec![OcrTruthPage {
                page: 1,
                text: "Bill to Marta Quillon, INV-20417".into(),
            }],
            dates: Vec::new(),
            names: vec!["Marta Quillon".into()],
            identifiers: vec!["INV-20417".into()],
        };
        let unread = OcrMeasure::unread(&truth);
        assert_eq!(
            (
                unread.pages_failed,
                unread.pages_compared,
                unread.pages_missing
            ),
            (1, 0, 0)
        );
        assert_eq!(unread.cer(), Some(1.0));
        assert_eq!(unread.wer(), Some(1.0));
        assert_eq!(unread.name_accuracy(), Some(0.0));
        assert_eq!(unread.identifier_accuracy(), Some(0.0));
        assert_eq!(unread.date_accuracy(), None, "no date drawn");
        assert_eq!(unread.mean_confidence, None);

        // Pooled with a perfect reading of the same page, half is lost.
        let mut pooled = measure(
            &truth,
            &scanned(&[(1, "Bill to Marta Quillon, INV-20417", Some(90))]),
        );
        pooled.accumulate(&unread);
        assert_eq!(pooled.name_accuracy(), Some(0.5));
        assert_eq!(pooled.identifier_accuracy(), Some(0.5));
        assert_eq!(pooled.cer(), Some(0.5));
        assert_eq!(pooled.pages_failed, 1);
    }
}
