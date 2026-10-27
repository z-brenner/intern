//! Scores `ocr_eval read` outputs with intern-bench's own OCR measure
//! (exploratory; for docs/ocr.md's engine comparisons).
//!
//! ```text
//! ocr_score --gold bench/gold.json --out SCORES.json READ.json [READ.json ...]
//! ```
//!
//! Per reading and per document: [`intern_bench::ocr::measure`] exactly as
//! InternBench computes it (CER, WER, dates, names, identifiers), and two
//! figures of this harness's own:
//!
//! * amounts: every money amount printed on a truth page (`$1,234.56`,
//!   `1,234.56`, `$25,000`, `436.73`), found when the reading has it as a
//!   whole token, punctuation around it aside - so an amount glued to its
//!   neighbour (`13.14100,436.73`) is a miss;
//! * merged tokens: tokens of the reading that are not on the page but are
//!   two neighbouring printed tokens run together, the failure docs/ocr.md
//!   records for the newer recognizers on widely spaced table columns.
use std::collections::{BTreeMap, BTreeSet};

use intern_bench::gold::{GoldFile, OcrTruth};
use intern_bench::ocr::{OcrMeasure, comparable, measure};
use intern_engine::{DocumentSource, PageOrigin, SourcePage};
use serde_json::{Value, json};

/// The first 12 scanned documents, the set docs/ocr.md was measured on.
const ORIGINAL: &[&str] = &[
    "scan-clean-lease-2p",
    "scan-mixed-amendment",
    "scan-rotated-90-invoice",
    "scan-upside-down-po",
    "scan-skewed-notice",
    "scan-low-res-receipt",
    "scan-noisy-statement",
    "scan-faint-letter",
    "scan-agreement-10p",
    "scan-lease-25p",
    "scan-fax-two-frames",
    "scan-patient-intake-form",
];

fn trim_token(token: &str) -> &str {
    token
        .trim_start_matches(['(', '[', '"', '\''])
        .trim_end_matches(['.', ',', ';', ':', ')', ']', '"', '\''])
}

fn is_amount(token: &str) -> bool {
    let body = token.strip_prefix('$').unwrap_or(token);
    let dollar = body.len() != token.len();
    let (whole, cents) = match body.split_once('.') {
        Some((whole, cents)) => (whole, Some(cents)),
        None => (body, None),
    };
    if whole.is_empty() || !whole.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    let groups: Vec<&str> = whole.split(',').collect();
    let grouped = groups.len() > 1
        && groups[0].len() <= 3
        && groups[1..].iter().all(|group| group.len() == 3);
    if !groups
        .iter()
        .all(|group| group.chars().all(|c| c.is_ascii_digit()))
    {
        return false;
    }
    if groups.len() > 1 && !grouped {
        return false;
    }
    match cents {
        Some(cents) => cents.len() == 2 && cents.chars().all(|c| c.is_ascii_digit()),
        None => dollar,
    }
}

fn tokens(text: &str) -> Vec<String> {
    comparable(text)
        .split_whitespace()
        .map(|token| trim_token(token).to_owned())
        .filter(|token| !token.is_empty())
        .collect()
}

struct Extra {
    digits_found: usize,
    digits_total: usize,
    digits_missed: Vec<String>,
    amounts_found: usize,
    amounts_total: usize,
    amounts_missed: Vec<String>,
    merged: Vec<String>,
}

fn extras(truth: &OcrTruth, read_pages: &BTreeMap<usize, String>) -> Extra {
    let drawn: Vec<String> = truth
        .pages
        .iter()
        .flat_map(|page| tokens(&page.text))
        .collect();
    let read: Vec<String> = truth
        .pages
        .iter()
        .flat_map(|page| tokens(read_pages.get(&page.page).map_or("", String::as_str)))
        .collect();
    let read_set: BTreeSet<&str> = read.iter().map(String::as_str).collect();
    let drawn_set: BTreeSet<&str> = drawn.iter().map(String::as_str).collect();
    let amounts: BTreeSet<&str> = drawn
        .iter()
        .map(String::as_str)
        .filter(|token| is_amount(token))
        .collect();
    let missed: Vec<String> = amounts
        .iter()
        .filter(|amount| !read_set.contains(*amount))
        .map(|amount| (*amount).to_owned())
        .collect();
    let pairs: BTreeSet<String> = drawn
        .windows(2)
        .map(|pair| format!("{}{}", pair[0], pair[1]))
        .collect();
    let raw_pairs: BTreeSet<String> = truth
        .pages
        .iter()
        .flat_map(|page| {
            let words: Vec<String> = comparable(&page.text)
                .split_whitespace()
                .map(str::to_owned)
                .collect();
            words
                .windows(2)
                .map(|pair| format!("{}{}", pair[0], pair[1]))
                .collect::<Vec<_>>()
        })
        .collect();
    let merged: Vec<String> = read
        .iter()
        .filter(|token| {
            !drawn_set.contains(token.as_str())
                && token.chars().any(|c| c.is_ascii_digit())
                && (pairs.contains(token.as_str()) || raw_pairs.contains(token.as_str()))
        })
        .cloned()
        .collect();
    // Every printed token holding a digit - dates, amounts, identifiers,
    // policy and account numbers, postcodes - counted with multiplicity,
    // found when the reading has as many of it.
    let mut wanted: BTreeMap<&str, usize> = BTreeMap::new();
    for token in drawn
        .iter()
        .filter(|token| token.chars().any(|c| c.is_ascii_digit()))
    {
        *wanted.entry(token.as_str()).or_default() += 1;
    }
    let mut have: BTreeMap<&str, usize> = BTreeMap::new();
    for token in &read {
        *have.entry(token.as_str()).or_default() += 1;
    }
    let digits_total: usize = wanted.values().sum();
    let mut digits_found = 0;
    let mut digits_missed = Vec::new();
    for (token, count) in &wanted {
        let got = have.get(token).copied().unwrap_or(0).min(*count);
        digits_found += got;
        if got < *count {
            digits_missed.push((*token).to_owned());
        }
    }
    Extra {
        digits_found,
        digits_total,
        digits_missed,
        amounts_found: amounts.len() - missed.len(),
        amounts_total: amounts.len(),
        amounts_missed: missed,
        merged,
    }
}

fn missed(values: &[String], drawn: &str, read: &str, fold: bool) -> Vec<String> {
    let form = |value: &str| {
        let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
        if fold {
            collapsed.to_lowercase()
        } else {
            collapsed
        }
    };
    let (drawn, read) = (form(drawn), form(read));
    values
        .iter()
        .filter(|value| drawn.contains(&form(value)) && !read.contains(&form(value)))
        .cloned()
        .collect()
}

fn summary(
    total: &OcrMeasure,
    amounts: (usize, usize),
    digits: (usize, usize),
    merged: usize,
    wall: f64,
    cpu: f64,
    pages: usize,
) -> Value {
    json!({
        "pages": pages,
        "cer": total.cer(),
        "wer": total.wer(),
        "dates": [total.dates_found, total.dates_total],
        "names": [total.names_found, total.names_total],
        "identifiers": [total.identifiers_found, total.identifiers_total],
        "amounts": [amounts.0, amounts.1],
        "digit_tokens": [digits.0, digits.1],
        "merged_tokens": merged,
        "pages_failed_or_missing": total.pages_failed + total.pages_missing,
        "wall_per_page_s": wall / pages.max(1) as f64,
        "cpu_per_page_s": cpu / pages.max(1) as f64,
    })
}

/// One subset's running totals.
#[derive(Default)]
struct Subset {
    measure: OcrMeasure,
    amounts: (usize, usize),
    digits: (usize, usize),
    merged: usize,
    wall: f64,
    cpu: f64,
    pages: usize,
}

fn main() {
    let mut gold_path = String::from("bench/gold.json");
    let mut out = None;
    let mut readings = Vec::new();
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--gold" => gold_path = arguments.next().unwrap(),
            "--out" => out = arguments.next(),
            other => readings.push(other.to_owned()),
        }
    }
    let (gold, _) = GoldFile::load(std::path::Path::new(&gold_path)).unwrap();
    let mut results = serde_json::Map::new();
    for path in &readings {
        let reading: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let mut documents = serde_json::Map::new();
        let mut subsets: BTreeMap<&str, Subset> = BTreeMap::new();
        for document in &gold.documents {
            let Some(truth) = &document.ocr_truth else {
                continue;
            };
            let Some(pages) = reading["documents"][&document.id].as_array() else {
                continue;
            };
            let mut read_pages = BTreeMap::new();
            let mut source_pages = Vec::new();
            let (mut wall, mut cpu) = (0.0, 0.0);
            for page in pages {
                let number = page["page"].as_u64().unwrap() as usize;
                wall += page["wall_s"].as_f64().unwrap_or(0.0);
                cpu += page["cpu_s"].as_f64().unwrap_or(0.0);
                let Some(text) = page["text"].as_str() else {
                    continue;
                };
                read_pages.insert(number, text.to_owned());
                let mut source = SourcePage::new(number, text, PageOrigin::Ocr);
                source.ocr_confidence = page["confidence"].as_f64().map(|c| c.round() as u32);
                source_pages.push(source);
            }
            let source = DocumentSource::from_pages(source_pages);
            let scored = measure(truth, &source);
            let extra = extras(truth, &read_pages);
            let drawn = truth
                .pages
                .iter()
                .map(|page| page.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let read = truth
                .pages
                .iter()
                .map(|page| read_pages.get(&page.page).map_or("", String::as_str))
                .collect::<Vec<_>>()
                .join(" ");
            documents.insert(
                document.id.clone(),
                json!({
                    "cer": scored.cer(),
                    "wer": scored.wer(),
                    "dates": [scored.dates_found, scored.dates_total],
                    "names": [scored.names_found, scored.names_total],
                    "identifiers": [scored.identifiers_found, scored.identifiers_total],
                    "amounts": [extra.amounts_found, extra.amounts_total],
                    "digit_tokens": [extra.digits_found, extra.digits_total],
                    "missed_digit_tokens": extra.digits_missed,
                    "missed_dates": missed(&truth.dates, &drawn, &read, true),
                    "missed_names": missed(&truth.names, &drawn, &read, true),
                    "missed_identifiers": missed(&truth.identifiers, &drawn, &read, false),
                    "missed_amounts": extra.amounts_missed,
                    "merged_tokens": extra.merged,
                    "wall_s": wall,
                    "cpu_s": cpu,
                }),
            );
            let subset = if ORIGINAL.contains(&document.id.as_str()) {
                "original"
            } else {
                "added"
            };
            for name in [subset, "all"] {
                let entry = subsets.entry(name).or_default();
                entry.measure.accumulate(&scored);
                entry.amounts.0 += extra.amounts_found;
                entry.amounts.1 += extra.amounts_total;
                entry.digits.0 += extra.digits_found;
                entry.digits.1 += extra.digits_total;
                entry.merged += extra.merged.len();
                entry.wall += wall;
                entry.cpu += cpu;
                entry.pages += pages.len();
            }
        }
        let totals: serde_json::Map<String, Value> = subsets
            .iter()
            .map(|(name, subset)| {
                let value = summary(
                    &subset.measure,
                    subset.amounts,
                    subset.digits,
                    subset.merged,
                    subset.wall,
                    subset.cpu,
                    subset.pages,
                );
                ((*name).to_owned(), value)
            })
            .collect();
        eprintln!("{path}: {}", serde_json::to_string(&totals["all"]).unwrap());
        results.insert(
            path.clone(),
            json!({
                "options": reading["options"],
                "harness_peak_rss_mib": reading["harness_peak_rss_mib"],
                "totals": totals,
                "documents": documents,
            }),
        );
    }
    if let Some(out) = out {
        std::fs::write(
            out,
            serde_json::to_vec_pretty(&Value::Object(results)).unwrap(),
        )
        .unwrap();
    }
}
