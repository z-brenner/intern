//! The report: every record, and the corpus summarised several ways.
//!
//! Written as JSON with every object's keys sorted and every list in a
//! fixed order (records in gold order, groups and metrics by name), so two
//! reports of the same corpus diff line by line and `compare` can align
//! them without guessing.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    baseline::BaselineComparison,
    machine::{MachineInfo, ModelInfo},
    memory::{INTERVAL_MS, MemoryPeaks},
    ocr::OcrMeasure,
    record::{COMPLETED, DocumentRecord},
    stats::{Distribution, round},
    structure::StructureMeasure,
    timing::{self, METRICS},
};

pub const REPORT_SCHEMA_VERSION: u32 = 1;
pub const SUITE: &str = "internbench";
/// The mode of an extract-only run, and of the baselines written from one.
pub const EXTRACT: &str = "extract";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Report {
    pub schema_version: u32,
    #[serde(default)]
    pub suite: String,
    /// `live`, `replay`, or `extract` (the worker alone).
    pub mode: String,
    #[serde(default)]
    pub created_at: String,
    /// The machine the timings were taken on: this one live, the
    /// recording's in replay.
    #[serde(default)]
    pub machine: MachineInfo,
    #[serde(default)]
    pub model: ModelInfo,
    /// The code that scored the run.
    #[serde(default)]
    pub git_commit: Option<String>,
    #[serde(default)]
    pub worker: Option<String>,
    /// `measured`, or `recorded` when replay reports the recording's.
    #[serde(default)]
    pub timings_source: String,
    #[serde(default)]
    pub recording: Option<RecordingInfo>,
    #[serde(default)]
    pub corpus: CorpusInfo,
    /// How long the whole run took, warm-up included, in milliseconds; a
    /// replay measures none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<f64>,
    #[serde(default)]
    pub summary: Summary,
    /// Summaries sliced by `kind`, `text_layer`, `format`, `page_bucket`,
    /// `route_class`, `category` and `slice` (`long`, `complex`; see
    /// [`slices`]), each keyed by the group's value.
    #[serde(default)]
    pub groups: BTreeMap<String, BTreeMap<String, Summary>>,
    #[serde(default)]
    pub latency: Latency,
    #[serde(default)]
    pub ocr: OcrReport,
    /// The structure the gold gives, measured over the text read; absent
    /// when no document has a structure block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structure: Option<StructureReport>,
    /// Which route each page took, and how the pages the gold gives a route
    /// for were judged: present whenever a page came with a layout or the
    /// gold expects a route (so a run whose worker sends no layouts says
    /// its expected routes went unjudged), absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routes: Option<RouteReport>,
    /// The phase 3 comparison list, one entry per figure (see
    /// [`SCORECARD`]); empty for an extract-only run, which names nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scorecard: Vec<ScorecardEntry>,
    #[serde(default)]
    pub memory: MemoryReport,
    #[serde(default)]
    pub records: Vec<DocumentRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineComparison>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RecordingInfo {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub recorded_at: String,
    /// The code the recording was made with.
    #[serde(default)]
    pub git_commit: Option<String>,
    #[serde(default)]
    pub note: String,
    /// How the digest budget or context the recording was made with differs
    /// from the engine's now; replay uses today's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_change: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CorpusInfo {
    pub documents: usize,
    #[serde(default)]
    pub gold_sha256: String,
    #[serde(default)]
    pub manifest_sha256: Option<String>,
    /// The ids `--only` restricted the run to; empty for the whole corpus.
    #[serde(default)]
    pub only: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Rate {
    pub correct: usize,
    pub total: usize,
    pub rate: f64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Mean {
    pub mean: f64,
    pub total_docs: usize,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Sum {
    pub sum: u64,
    pub total_docs: usize,
}

/// The failures a person would regret: each a count of documents, except
/// the claim counts.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SafetyCounts {
    /// Filed without review under a wrong name.
    pub unsafe_ready: usize,
    /// A date the gold marks as a trap was chosen.
    pub trap_dates: usize,
    /// A party the gold marks as a trap was named.
    pub forbidden_parties: usize,
    /// Parties named that the gold does not list (a count of parties).
    pub spurious_parties: u64,
    /// Descriptions asserting a fact the gold marks as wrong.
    pub forbidden_descriptions: usize,
    pub claims: u64,
    pub unsupported_claims: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Summary {
    pub documents: usize,
    /// Documents with scores: completed, or failed and scored as misses.
    pub scored: usize,
    pub completed: usize,
    pub statuses: BTreeMap<String, usize>,
    /// Of the completed documents that were named, the fraction sent to
    /// review; none for an extract-only run, which names nothing.
    #[serde(default)]
    pub review_rate: Option<f64>,
    /// Unsupported description claims over all claims.
    #[serde(default)]
    pub unsupported_fact_rate: Option<f64>,
    #[serde(default)]
    pub counts: SafetyCounts,
    /// Every boolean score.
    #[serde(default)]
    pub rates: BTreeMap<String, Rate>,
    /// Every fractional score.
    #[serde(default)]
    pub means: BTreeMap<String, Mean>,
    /// Every integer score.
    #[serde(default)]
    pub sums: BTreeMap<String, Sum>,
}

pub fn summarize<'a>(records: impl IntoIterator<Item = &'a DocumentRecord>) -> Summary {
    let mut summary = Summary::default();
    let mut reviewed = 0;
    // Completed documents that were routed at all: an extract-only record
    // names nothing and is neither ready nor sent to review.
    let mut routed = 0;
    let mut fractions: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for record in records {
        summary.documents += 1;
        *summary.statuses.entry(record.status.clone()).or_insert(0) += 1;
        if record.status == COMPLETED {
            summary.completed += 1;
            routed += usize::from(record.readiness.is_some());
            if record.readiness.as_deref() == Some("needs_review") {
                reviewed += 1;
            }
        }
        if record.scores.is_empty() {
            continue;
        }
        summary.scored += 1;
        for (key, value) in &record.scores {
            match value {
                Value::Bool(flag) => {
                    let rate = summary.rates.entry(key.clone()).or_default();
                    rate.total += 1;
                    rate.correct += usize::from(*flag);
                }
                Value::Number(number) if number.is_f64() => {
                    let entry = fractions.entry(key.clone()).or_insert((0.0, 0));
                    entry.0 += number.as_f64().unwrap_or(0.0);
                    entry.1 += 1;
                }
                Value::Number(number) => {
                    let sum = summary.sums.entry(key.clone()).or_default();
                    sum.sum += number.as_u64().unwrap_or(0);
                    sum.total_docs += 1;
                }
                _ => {}
            }
        }
        let counts = &mut summary.counts;
        counts.unsafe_ready += usize::from(record.bool_score("unsafe_ready") == Some(true));
        counts.trap_dates += usize::from(record.bool_score("date_forbidden") == Some(true));
        counts.forbidden_parties += usize::from(record.bool_score("party_forbidden") == Some(true));
        counts.forbidden_descriptions += usize::from(!record.forbidden_description.is_empty());
        let integer = |key: &str| record.scores.get(key).and_then(Value::as_u64).unwrap_or(0);
        counts.spurious_parties += integer("parties_spurious");
        counts.claims += integer("description_claims");
        counts.unsupported_claims += integer("description_unsupported");
    }
    for rate in summary.rates.values_mut() {
        rate.rate = round(rate.correct as f64 / rate.total.max(1) as f64, 4);
    }
    summary.means = fractions
        .into_iter()
        .map(|(key, (total, count))| {
            (
                key,
                Mean {
                    mean: round(total / count.max(1) as f64, 4),
                    total_docs: count,
                },
            )
        })
        .collect();
    summary.review_rate = (routed > 0).then(|| round(reviewed as f64 / routed as f64, 4));
    summary.unsupported_fact_rate = (summary.counts.claims > 0).then(|| {
        round(
            summary.counts.unsupported_claims as f64 / summary.counts.claims as f64,
            4,
        )
    });
    summary
}

/// The page count from which a document is `long`.
pub const LONG_PAGES: u32 = 10;

/// The categories that make a document `complex`: what it says has to be
/// found or reasoned out - a fact in the middle, a referenced agreement's
/// date or parties, names that are not parties, columns or a stream order
/// that scrambles the reading, a date or labelled value inside a table,
/// parties placed only by the layout, a dense page. Left out:
/// `competing_dates`, which nearly every document has; `table`, whose
/// header tables are routine (`date_in_table` and `key_value` keep the
/// tables that matter for the name); and the scan conditions, which
/// measure OCR and are sliced by `text_layer`.
pub const COMPLEX_CATEGORIES: [&str; 10] = [
    "referenced_agreement",
    "middle_fact",
    "multi_column",
    "layout_parties",
    "irrelevant_names",
    "information_dense",
    "stream_order",
    "date_in_table",
    "key_value",
    "complex_pdf",
];

/// The slices a document belongs to: `long` (at least [`LONG_PAGES`]
/// pages) and `complex` (any of [`COMPLEX_CATEGORIES`]); either, both, or
/// neither.
pub fn slices(record: &DocumentRecord) -> Vec<&'static str> {
    let mut slices = Vec::new();
    if record.pages >= LONG_PAGES {
        slices.push("long");
    }
    if record
        .categories
        .iter()
        .any(|category| COMPLEX_CATEGORIES.contains(&category.as_str()))
    {
        slices.push("complex");
    }
    slices
}

/// The dimensions the corpus is sliced along, each with how a record
/// names its group or groups.
pub fn group_keys(record: &DocumentRecord) -> Vec<(&'static str, String)> {
    let mut keys = vec![
        ("kind", record.kind.clone()),
        ("text_layer", record.text_layer.clone()),
        ("format", record.format.clone()),
        ("page_bucket", record.page_bucket.clone()),
    ];
    keys.extend(
        record
            .route_class
            .iter()
            .map(|class| ("route_class", class.clone())),
    );
    keys.extend(
        record
            .categories
            .iter()
            .map(|category| ("category", category.clone())),
    );
    keys.extend(
        slices(record)
            .into_iter()
            .map(|slice| ("slice", slice.to_owned())),
    );
    keys
}

pub fn groups(records: &[DocumentRecord]) -> BTreeMap<String, BTreeMap<String, Summary>> {
    let mut members: BTreeMap<&str, BTreeMap<String, Vec<&DocumentRecord>>> = BTreeMap::new();
    for record in records {
        for (dimension, value) in group_keys(record) {
            members
                .entry(dimension)
                .or_default()
                .entry(value)
                .or_default()
                .push(record);
        }
    }
    members
        .into_iter()
        .map(|(dimension, groups)| {
            (
                dimension.to_owned(),
                groups
                    .into_iter()
                    .map(|(value, records)| (value, summarize(records)))
                    .collect(),
            )
        })
        .collect()
}

/// Distributions of every timing metric, overall and by slice, over the
/// documents that completed (see [`timed`]); each distribution's `count`
/// says how many had the metric.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Latency {
    #[serde(default)]
    pub overall: BTreeMap<String, Distribution>,
    #[serde(default)]
    pub by_page_bucket: BTreeMap<String, BTreeMap<String, Distribution>>,
    #[serde(default)]
    pub by_kind: BTreeMap<String, BTreeMap<String, Distribution>>,
    #[serde(default)]
    pub by_text_layer: BTreeMap<String, BTreeMap<String, Distribution>>,
    /// By the most expensive route a document's pages took (`ocr`,
    /// `ocr_regions`, `layout`, `fast`), or `unrouted`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_route_class: BTreeMap<String, BTreeMap<String, Distribution>>,
    /// By slice (`long`, `complex`); a document may be in both.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_slice: BTreeMap<String, BTreeMap<String, Distribution>>,
}

/// Records whose timings describe the whole pipeline: the documents that
/// completed. A failed document's time is how long it took to fail - an
/// extraction failure has no analysis time at all, a model failure no
/// validation or naming - so mixing it in would give each stage a
/// different set of documents, and a slow document that starts failing
/// would make p95 look faster. Failures are counted in the summary's
/// statuses instead.
pub fn timed(record: &DocumentRecord) -> bool {
    record.status == COMPLETED
}

pub fn distributions<'a>(
    records: impl IntoIterator<Item = &'a DocumentRecord> + Clone,
) -> BTreeMap<String, Distribution> {
    METRICS
        .iter()
        .filter_map(|(metric, _)| {
            let values = records
                .clone()
                .into_iter()
                .filter(|record| timed(record))
                .filter_map(|record| timing::get(&record.timings, metric))
                .collect::<Vec<_>>();
            Distribution::of(&values).map(|distribution| ((*metric).to_owned(), distribution))
        })
        .collect()
}

pub fn latency(records: &[DocumentRecord]) -> Latency {
    let by = |key: fn(&DocumentRecord) -> Vec<&str>| {
        let mut groups: BTreeMap<String, Vec<&DocumentRecord>> = BTreeMap::new();
        for record in records {
            for value in key(record) {
                groups.entry(value.to_owned()).or_default().push(record);
            }
        }
        groups
            .into_iter()
            .map(|(value, members)| (value, distributions(members.iter().copied())))
            .filter(|(_, distributions)| !distributions.is_empty())
            .collect()
    };
    Latency {
        overall: distributions(records.iter()),
        by_page_bucket: by(|record| vec![record.page_bucket.as_str()]),
        by_kind: by(|record| vec![record.kind.as_str()]),
        by_text_layer: by(|record| vec![record.text_layer.as_str()]),
        by_route_class: by(|record| record.route_class.as_deref().into_iter().collect()),
        by_slice: by(slices),
    }
}

/// How a scorecard figure is measured, which says how it reads and which
/// way is better.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScorecardKind {
    /// A share of documents, higher better.
    Rate,
    /// A share of documents where lower is better.
    BadRate,
    /// A share of documents with no better direction (the review rate).
    NeutralRate,
    /// A mean of per-document fractions, higher better.
    Mean,
    /// A latency percentile in milliseconds, lower better.
    Milliseconds,
    /// A token-count percentile, lower better.
    Tokens,
}

/// The phase 3 comparison list: key, label, kind. A `_p50` or `_p95`
/// suffix is that percentile of the timing metric before it, over the
/// documents that completed.
pub const SCORECARD: [(&str, &str, ScorecardKind); 13] = [
    (
        "long_filename_correct",
        "Long-document filename accuracy (10+ pages)",
        ScorecardKind::Rate,
    ),
    (
        "complex_filename_correct",
        "Complex-document filename accuracy",
        ScorecardKind::Rate,
    ),
    (
        "description_completeness",
        "Description completeness",
        ScorecardKind::Mean,
    ),
    (
        "unsupported_fact_doc",
        "Unsupported-fact rate (documents)",
        ScorecardKind::BadRate,
    ),
    ("review_rate", "Review rate", ScorecardKind::NeutralRate),
    ("evidence_recall", "Evidence recall", ScorecardKind::Mean),
    (
        "total_ms_p50",
        "Total latency p50",
        ScorecardKind::Milliseconds,
    ),
    (
        "total_ms_p95",
        "Total latency p95",
        ScorecardKind::Milliseconds,
    ),
    (
        "generation_ms_p50",
        "Generation latency p50",
        ScorecardKind::Milliseconds,
    ),
    (
        "generation_ms_p95",
        "Generation latency p95",
        ScorecardKind::Milliseconds,
    ),
    (
        "generated_tokens_p50",
        "Generated tokens p50",
        ScorecardKind::Tokens,
    ),
    (
        "generated_tokens_p95",
        "Generated tokens p95",
        ScorecardKind::Tokens,
    ),
    (
        "prompt_tokens_p50",
        "Prompt tokens p50",
        ScorecardKind::Tokens,
    ),
];

/// The kind of a scorecard key.
pub fn scorecard_kind(key: &str) -> Option<ScorecardKind> {
    SCORECARD
        .iter()
        .find(|(entry, _, _)| *entry == key)
        .map(|(_, _, kind)| *kind)
}

/// One figure of the phase 3 comparison list.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ScorecardEntry {
    pub metric: String,
    /// None when no document gives it: no long document scored, or no
    /// timing measured.
    pub value: Option<f64>,
    /// The documents it is over.
    pub documents: usize,
}

/// A percentile scorecard key's timing metric and percentile.
pub fn scorecard_percentile(key: &str) -> Option<(&str, &str)> {
    key.strip_suffix("_p50")
        .map(|metric| (metric, "p50"))
        .or_else(|| key.strip_suffix("_p95").map(|metric| (metric, "p95")))
}

/// The phase 3 comparison list over `scored` (rates and means: every
/// document that has the score), with `review_rate` as given and the
/// percentiles from `latency` (each over the documents that completed and
/// measured the metric).
pub fn scorecard<'a>(
    scored: impl IntoIterator<Item = &'a DocumentRecord>,
    review_rate: Option<f64>,
    reviewed_over: usize,
    latency: &BTreeMap<String, Distribution>,
) -> Vec<ScorecardEntry> {
    let scored = scored.into_iter().collect::<Vec<_>>();
    let rate_of = |records: &[&DocumentRecord], key: &str| {
        let flags = records
            .iter()
            .filter_map(|record| record.bool_score(key))
            .collect::<Vec<_>>();
        let value = (!flags.is_empty()).then(|| {
            round(
                flags.iter().filter(|flag| **flag).count() as f64 / flags.len() as f64,
                4,
            )
        });
        (value, flags.len())
    };
    let in_slice = |slice: &str| {
        scored
            .iter()
            .copied()
            .filter(|record| slices(record).contains(&slice))
            .collect::<Vec<_>>()
    };
    SCORECARD
        .iter()
        .map(|(key, _, _)| {
            let (value, documents) = match *key {
                "long_filename_correct" => rate_of(&in_slice("long"), "filename_correct"),
                "complex_filename_correct" => rate_of(&in_slice("complex"), "filename_correct"),
                "unsupported_fact_doc" => rate_of(&scored, key),
                "review_rate" => (review_rate, reviewed_over),
                "description_completeness" | "evidence_recall" => {
                    let values = scored
                        .iter()
                        .filter_map(|record| record.scores.get(*key).and_then(Value::as_f64))
                        .collect::<Vec<_>>();
                    let mean = (!values.is_empty())
                        .then(|| round(values.iter().sum::<f64>() / values.len() as f64, 4));
                    (mean, values.len())
                }
                _ => scorecard_percentile(key)
                    .and_then(|(metric, percentile)| {
                        let distribution = latency.get(metric)?;
                        let value = if percentile == "p50" {
                            distribution.p50
                        } else {
                            distribution.p95
                        };
                        Some((Some(value), distribution.count))
                    })
                    .unwrap_or((None, 0)),
            };
            ScorecardEntry {
                metric: (*key).to_owned(),
                value,
                documents,
            }
        })
        .collect()
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OcrFigures {
    pub pages_compared: usize,
    pub pages_missing: usize,
    /// Pages of scans whose extraction failed, counted as read empty.
    #[serde(default)]
    pub pages_failed: usize,
    pub cer: Option<f64>,
    pub cer_ci: Option<f64>,
    pub wer: Option<f64>,
    pub date_accuracy: Option<f64>,
    pub name_accuracy: Option<f64>,
    pub identifier_accuracy: Option<f64>,
    pub mean_confidence: Option<f64>,
}

impl OcrFigures {
    fn of(measure: &OcrMeasure) -> Self {
        let rounded = |value: Option<f64>| value.map(|value| round(value, 4));
        Self {
            pages_compared: measure.pages_compared,
            pages_missing: measure.pages_missing,
            pages_failed: measure.pages_failed,
            cer: rounded(measure.cer()),
            cer_ci: rounded(measure.cer_ci()),
            wer: rounded(measure.wer()),
            date_accuracy: rounded(measure.date_accuracy()),
            name_accuracy: rounded(measure.name_accuracy()),
            identifier_accuracy: rounded(measure.identifier_accuracy()),
            mean_confidence: rounded(measure.mean_confidence),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OcrRow {
    pub id: String,
    pub text_layer: String,
    #[serde(flatten)]
    pub figures: OcrFigures,
    /// The worker's OCR time for the document.
    #[serde(default)]
    pub ocr_ms: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OcrReport {
    /// Distances summed over every compared page of every document, over
    /// the truth's total length; the targeted accuracies likewise pooled.
    /// A scan whose extraction failed is in it, read as empty.
    #[serde(default)]
    pub aggregate: Option<OcrFigures>,
    #[serde(default)]
    pub documents: Vec<OcrRow>,
}

pub fn ocr_report(records: &[DocumentRecord]) -> OcrReport {
    let mut pooled = OcrMeasure::default();
    let mut rows = Vec::new();
    for record in records {
        let Some(measure) = &record.ocr else {
            continue;
        };
        pooled.accumulate(measure);
        rows.push(OcrRow {
            id: record.id.clone(),
            text_layer: record.text_layer.clone(),
            figures: OcrFigures::of(measure),
            ocr_ms: timing::get(&record.timings, "worker_ocr_ms"),
        });
    }
    if rows.is_empty() {
        return OcrReport::default();
    }
    OcrReport {
        aggregate: Some(OcrFigures::of(&pooled)),
        documents: rows,
    }
}

/// A structure measure's scores and the counts they are made of.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StructureFigures {
    pub reading_order_accuracy: Option<f64>,
    pub table_row_accuracy: Option<f64>,
    pub table_cell_recall: Option<f64>,
    pub kv_accuracy: Option<f64>,
    pub route_correct: Option<f64>,
    #[serde(default)]
    pub snippets: usize,
    #[serde(default)]
    pub snippets_in_order: usize,
    pub rows: usize,
    pub rows_found: usize,
    pub cells: usize,
    pub cells_found: usize,
    pub key_values: usize,
    pub key_values_found: usize,
    pub route_pages: usize,
    pub routes_correct: usize,
}

impl StructureFigures {
    pub fn of(measure: &StructureMeasure) -> Self {
        let rounded = |value: Option<f64>| value.map(|value| round(value, 4));
        Self {
            reading_order_accuracy: rounded(measure.reading_order_accuracy()),
            table_row_accuracy: rounded(measure.table_row_accuracy()),
            table_cell_recall: rounded(measure.table_cell_recall()),
            kv_accuracy: rounded(measure.kv_accuracy()),
            route_correct: rounded(measure.route_correct()),
            snippets: measure.snippets,
            snippets_in_order: measure.snippets_in_order,
            rows: measure.rows,
            rows_found: measure.rows_found,
            cells: measure.cells,
            cells_found: measure.cells_found,
            key_values: measure.key_values,
            key_values_found: measure.key_values_found,
            route_pages: measure.route_pages,
            routes_correct: measure.routes_correct,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct StructureRow {
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub route_class: Option<String>,
    #[serde(flatten)]
    pub figures: StructureFigures,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct StructureReport {
    /// Every count summed over the documents, each score the share of the
    /// pooled items (snippets, rows, cells, labelled values, pages): a document with
    /// forty table rows weighs forty times one with one. A document whose
    /// extraction failed is in it, every item missed.
    pub aggregate: StructureFigures,
    pub documents: Vec<StructureRow>,
}

pub fn structure_report(records: &[DocumentRecord]) -> Option<StructureReport> {
    let mut pooled = StructureMeasure::default();
    let mut rows = Vec::new();
    for record in records {
        let Some(measure) = &record.structure else {
            continue;
        };
        pooled.accumulate(measure);
        rows.push(StructureRow {
            id: record.id.clone(),
            status: record.status.clone(),
            route_class: record.route_class.clone(),
            figures: StructureFigures::of(measure),
        });
    }
    (!rows.is_empty()).then(|| StructureReport {
        aggregate: StructureFigures::of(&pooled),
        documents: rows,
    })
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RouteReport {
    /// Pages per route over every document read: `fast`, `layout`, `ocr`,
    /// `ocr_regions`, or `none` for a page sent without a layout.
    pub pages: BTreeMap<String, usize>,
    /// Documents per route class (`unrouted` when the worker sent no
    /// layouts).
    pub classes: BTreeMap<String, usize>,
    /// Expected route, then the route taken, then pages: over every page
    /// the gold gives a route for, in documents whose pages came with
    /// layouts.
    pub confusion: BTreeMap<String, BTreeMap<String, usize>>,
    /// Pages the gold gives a route for, over every document, judged or
    /// not: the confusion holds those that were.
    #[serde(default)]
    pub expected_pages: usize,
}

pub fn route_report(records: &[DocumentRecord]) -> Option<RouteReport> {
    let mut report = RouteReport::default();
    for record in records {
        for route in &record.page_routes {
            *report.pages.entry(route.clone()).or_default() += 1;
        }
        if let Some(class) = &record.route_class {
            *report.classes.entry(class.clone()).or_default() += 1;
        }
        if let Some(measure) = &record.structure {
            report.expected_pages += measure.route_expected;
        }
        for check in record
            .structure
            .iter()
            .filter(|measure| !measure.failed)
            .flat_map(|measure| &measure.routes)
        {
            *report
                .confusion
                .entry(check.expected.clone())
                .or_default()
                .entry(check.actual.clone())
                .or_default() += 1;
        }
    }
    let routed =
        !report.pages.is_empty() || !report.confusion.is_empty() || report.expected_pages > 0;
    routed.then_some(report)
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MemoryReport {
    /// False when no document had a sample (not Linux, or nothing ran).
    pub sampled: bool,
    pub interval_ms: u64,
    /// The highest sampled peak of each process over the run.
    pub peak: MemoryPeaks,
    /// The document each highest peak was taken during.
    pub peak_documents: BTreeMap<String, String>,
    /// The per-document peaks of each process.
    pub per_document: BTreeMap<String, Distribution>,
}

/// Reads one process's peak out of a document's samples.
type PeakOf = fn(&MemoryPeaks) -> Option<f64>;

pub fn memory_report(records: &[DocumentRecord]) -> MemoryReport {
    let mut report = MemoryReport {
        interval_ms: INTERVAL_MS,
        ..MemoryReport::default()
    };
    let processes: [(&str, PeakOf); 3] = [
        ("worker_mb", |peaks| peaks.worker_mb),
        ("server_mb", |peaks| peaks.server_mb),
        ("bench_mb", |peaks| peaks.bench_mb),
    ];
    for (name, read) in processes {
        let samples = records
            .iter()
            .filter_map(|record| read(&record.memory).map(|value| (value, &record.id)))
            .collect::<Vec<_>>();
        let Some((highest, id)) = samples
            .iter()
            .max_by(|left, right| left.0.total_cmp(&right.0))
        else {
            continue;
        };
        report.sampled = true;
        report.peak_documents.insert(name.to_owned(), (*id).clone());
        match name {
            "worker_mb" => report.peak.worker_mb = Some(*highest),
            "server_mb" => report.peak.server_mb = Some(*highest),
            _ => report.peak.bench_mb = Some(*highest),
        }
        let values = samples.iter().map(|(value, _)| *value).collect::<Vec<_>>();
        if let Some(distribution) = Distribution::of(&values) {
            report.per_document.insert(name.to_owned(), distribution);
        }
    }
    report
}

/// Everything about a run that is not one of its records.
#[derive(Clone, Debug, Default)]
pub struct RunInfo {
    pub mode: String,
    pub created_at: String,
    pub machine: MachineInfo,
    pub model: ModelInfo,
    pub git_commit: Option<String>,
    pub worker: Option<String>,
    pub timings_source: String,
    pub recording: Option<RecordingInfo>,
    pub corpus: CorpusInfo,
    pub wall_ms: Option<f64>,
}

pub fn build(info: RunInfo, records: Vec<DocumentRecord>) -> Report {
    let summary = summarize(&records);
    let latency = latency(&records);
    // An extract-only run names nothing: none of the list applies.
    let scorecard = if info.mode == EXTRACT {
        Vec::new()
    } else {
        let routed = records
            .iter()
            .filter(|record| record.status == COMPLETED && record.readiness.is_some())
            .count();
        scorecard(&records, summary.review_rate, routed, &latency.overall)
    };
    Report {
        schema_version: REPORT_SCHEMA_VERSION,
        suite: SUITE.to_owned(),
        mode: info.mode,
        created_at: info.created_at,
        machine: info.machine,
        model: info.model,
        git_commit: info.git_commit,
        worker: info.worker,
        timings_source: info.timings_source,
        recording: info.recording,
        corpus: info.corpus,
        wall_ms: info.wall_ms,
        summary,
        groups: groups(&records),
        latency,
        ocr: ocr_report(&records),
        structure: structure_report(&records),
        routes: route_report(&records),
        scorecard,
        memory: memory_report(&records),
        records,
        baseline: None,
    }
}

impl Report {
    /// The report as JSON, every object's keys sorted.
    pub fn to_json(&self) -> Result<String, String> {
        // Through a `Value`, whose maps are sorted, rather than straight
        // from the structs, whose fields come out in declaration order.
        let value = serde_json::to_value(self).map_err(|error| error.to_string())?;
        serde_json::to_string_pretty(&value)
            .map(|text| text + "\n")
            .map_err(|error| error.to_string())
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let report: Self = serde_json::from_slice(bytes)
            .map_err(|error| format!("cannot parse report: {error}"))?;
        if report.schema_version != REPORT_SCHEMA_VERSION {
            return Err(format!(
                "report schema {} is not the supported {REPORT_SCHEMA_VERSION}",
                report.schema_version
            ));
        }
        Ok(report)
    }

    pub fn record(&self, id: &str) -> Option<&DocumentRecord> {
        self.records.iter().find(|record| record.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record(id: &str, kind: &str, pages: u32, scores: Value, total_ms: f64) -> DocumentRecord {
        let mut record = DocumentRecord {
            id: id.into(),
            kind: kind.into(),
            text_layer: "native".into(),
            format: "pdf".into(),
            pages,
            page_bucket: crate::gold::page_bucket(pages).into(),
            categories: vec![kind.into(), "table".into()],
            status: "completed".into(),
            readiness: Some("ready".into()),
            scores: serde_json::from_value(scores).unwrap(),
            timings: timing::empty(),
            ..DocumentRecord::default()
        };
        record.timings.insert("total_ms".into(), json!(total_ms));
        record
    }

    #[test]
    fn booleans_become_rates_fractions_means_and_integers_sums() {
        let records = vec![
            record(
                "a",
                "invoice",
                1,
                json!({"filename_correct": true, "date_forbidden": false, "evidence_recall": 1.0, "description_claims": 4, "description_unsupported": 1, "parties_spurious": 0}),
                100.0,
            ),
            record(
                "b",
                "invoice",
                12,
                json!({"filename_correct": false, "date_forbidden": true, "unsafe_ready": true, "evidence_recall": 0.5, "description_claims": 6, "description_unsupported": 0, "parties_spurious": 2}),
                300.0,
            ),
            DocumentRecord {
                id: "c".into(),
                status: "stale_prompt".into(),
                ..DocumentRecord::default()
            },
        ];
        let summary = summarize(&records);
        assert_eq!(
            (summary.documents, summary.scored, summary.completed),
            (3, 2, 2)
        );
        assert_eq!(
            summary.rates["filename_correct"],
            Rate {
                correct: 1,
                total: 2,
                rate: 0.5
            }
        );
        assert_eq!(summary.means["evidence_recall"].mean, 0.75);
        assert_eq!(summary.sums["description_claims"].sum, 10);
        assert_eq!(summary.counts.trap_dates, 1);
        assert_eq!(summary.counts.unsafe_ready, 1);
        assert_eq!(summary.counts.spurious_parties, 2);
        assert_eq!(summary.unsupported_fact_rate, Some(0.1));
        assert_eq!(summary.review_rate, Some(0.0));
        assert_eq!(summary.statuses["stale_prompt"], 1);

        let groups = groups(&records);
        assert_eq!(groups["kind"]["invoice"].documents, 2);
        assert_eq!(groups["page_bucket"]["10-24"].documents, 1);
        assert_eq!(groups["category"]["table"].documents, 2);

        let mut failed = records[1].clone();
        failed.id = "d".into();
        failed.status = "model_failed".into();
        let mut timed = records.clone();
        timed.push(failed);
        let latency = latency(&timed);
        assert_eq!(
            latency.overall["total_ms"].count, 2,
            "only completed documents: the stale one took no time, the failed one took time to fail"
        );
        assert_eq!(latency.overall["total_ms"].p95, 300.0);
        assert_eq!(latency.by_page_bucket["1"]["total_ms"].p50, 100.0);
        assert!(
            !latency.overall.contains_key("worker_ocr_ms"),
            "never measured"
        );
    }

    /// A long document and a complex one are sliced out of the corpus, a
    /// document may be both, and the phase 3 list reads each figure off
    /// the documents that give it.
    #[test]
    fn slices_and_the_phase3_scorecard() {
        let mut long_complex = record(
            "a",
            "contract",
            12,
            json!({"filename_correct": true, "unsupported_fact_doc": false, "description_completeness": 1.0, "evidence_recall": 0.5}),
            1_000.0,
        );
        long_complex.categories = vec![
            "contract".into(),
            "middle_fact".into(),
            "competing_dates".into(),
        ];
        let mut short = record(
            "b",
            "invoice",
            1,
            json!({"filename_correct": true, "unsupported_fact_doc": true, "description_completeness": 0.5, "evidence_recall": 1.0}),
            3_000.0,
        );
        // Competing dates alone, and a routine table, do not make it complex.
        short.categories = vec!["invoice".into(), "competing_dates".into(), "table".into()];
        short.readiness = Some("needs_review".into());
        let mut long = record(
            "c",
            "lease",
            25,
            json!({"filename_correct": false, "unsupported_fact_doc": false}),
            2_000.0,
        );
        long.categories = vec!["contract".into()];
        let mut records = vec![long_complex, short, long];
        for (record, (generation, generated, prompt)) in records.iter_mut().zip([
            (500.0, 120.0, 2_400.0),
            (900.0, 80.0, 900.0),
            (700.0, 100.0, 3_100.0),
        ]) {
            record
                .timings
                .insert("generation_ms".into(), json!(generation));
            record
                .timings
                .insert("generated_tokens".into(), json!(generated));
            record.timings.insert("prompt_tokens".into(), json!(prompt));
        }
        assert_eq!(slices(&records[0]), vec!["long", "complex"]);
        assert!(slices(&records[1]).is_empty());
        assert_eq!(slices(&records[2]), vec!["long"]);

        let report = build(
            RunInfo {
                mode: "live".into(),
                ..RunInfo::default()
            },
            records,
        );
        let sliced = &report.groups["slice"];
        assert_eq!(
            (sliced["long"].documents, sliced["complex"].documents),
            (2, 1)
        );
        assert_eq!(sliced["long"].rates["filename_correct"].correct, 1);
        assert_eq!(report.latency.by_slice["long"]["total_ms"].count, 2);
        assert!(!report.latency.by_slice.contains_key("short"));

        let figure = |key: &str| {
            report
                .scorecard
                .iter()
                .find(|entry| entry.metric == key)
                .map(|entry| (entry.value, entry.documents))
                .unwrap()
        };
        assert_eq!(report.scorecard.len(), SCORECARD.len());
        assert_eq!(figure("long_filename_correct"), (Some(0.5), 2));
        assert_eq!(figure("complex_filename_correct"), (Some(1.0), 1));
        assert_eq!(figure("unsupported_fact_doc"), (Some(0.3333), 3));
        assert_eq!(figure("review_rate"), (Some(0.3333), 3));
        assert_eq!(figure("description_completeness"), (Some(0.75), 2));
        assert_eq!(figure("evidence_recall"), (Some(0.75), 2));
        let overall = &report.latency.overall;
        assert_eq!(figure("total_ms_p50"), (Some(overall["total_ms"].p50), 3));
        assert_eq!(
            figure("generation_ms_p95"),
            (Some(overall["generation_ms"].p95), 3)
        );
        assert_eq!(
            figure("prompt_tokens_p50"),
            (Some(overall["prompt_tokens"].p50), 3)
        );

        let page = crate::markdown::render(&report);
        assert!(page.contains("## Phase 3 scorecard"), "{page}");
        assert!(
            page.contains("| Long-document filename accuracy (10+ pages) | 50.0% | 2 |"),
            "{page}"
        );
        assert!(page.contains("## By slice"), "{page}");
        assert!(page.contains("| long | 2 | 1/2 |"), "{page}");

        // An extract-only run names nothing, so there is no list.
        let extract = build(
            RunInfo {
                mode: EXTRACT.into(),
                ..RunInfo::default()
            },
            Vec::new(),
        );
        assert!(extract.scorecard.is_empty());
    }

    /// A worker that sends no layouts judges no route, but the report still
    /// says the gold expected some.
    #[test]
    fn routes_are_reported_whenever_the_gold_expects_one() {
        let mut plain = record("a", "invoice", 2, json!({}), 1.0);
        plain.route_class = Some("unrouted".into());
        assert!(
            route_report(std::slice::from_ref(&plain)).is_none(),
            "no layout, and no route expected"
        );
        plain.structure = Some(StructureMeasure {
            route_expected: 2,
            ..StructureMeasure::default()
        });
        let routes = route_report(std::slice::from_ref(&plain)).unwrap();
        assert_eq!(routes.expected_pages, 2);
        assert!(routes.pages.is_empty() && routes.confusion.is_empty());
        assert_eq!(routes.classes["unrouted"], 1);
        let page = crate::markdown::render(&build(
            RunInfo {
                mode: "replay".into(),
                ..RunInfo::default()
            },
            vec![plain],
        ));
        assert!(
            page.contains("## Routes")
                && page.contains("- **Pages the gold gives a route for:** 2, 0 judged")
                && page.contains("None was judged"),
            "{page}"
        );
    }

    #[test]
    fn the_json_has_sorted_keys_and_reads_back() {
        let report = build(
            RunInfo {
                mode: "replay".into(),
                ..RunInfo::default()
            },
            vec![record(
                "a",
                "invoice",
                1,
                json!({"filename_correct": true}),
                1.0,
            )],
        );
        let text = report.to_json().unwrap();
        let baseline = text.find("\"baseline\"");
        assert!(baseline.is_none(), "absent without a baseline");
        let corpus = text.find("\"corpus\"").unwrap();
        let summary = text.find("\"summary\"").unwrap();
        let records = text.find("\"records\"").unwrap();
        assert!(
            corpus < records && records < summary,
            "keys in sorted order"
        );
        let parsed = Report::parse(text.as_bytes()).unwrap();
        assert_eq!(parsed.records[0].id, "a");
        assert_eq!(
            parsed.to_json().unwrap(),
            text,
            "stable through a round trip"
        );
    }
}
