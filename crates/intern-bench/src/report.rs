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
    timing::{self, METRICS},
};

pub const REPORT_SCHEMA_VERSION: u32 = 1;
pub const SUITE: &str = "internbench";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Report {
    pub schema_version: u32,
    #[serde(default)]
    pub suite: String,
    /// `live` or `replay`.
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
    #[serde(default)]
    pub summary: Summary,
    /// Summaries sliced by `kind`, `text_layer`, `format`, `page_bucket`
    /// and `category`, each keyed by the group's value.
    #[serde(default)]
    pub groups: BTreeMap<String, BTreeMap<String, Summary>>,
    #[serde(default)]
    pub latency: Latency,
    #[serde(default)]
    pub ocr: OcrReport,
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
    /// Of the completed documents, the fraction sent to review.
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
    let mut fractions: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for record in records {
        summary.documents += 1;
        *summary.statuses.entry(record.status.clone()).or_insert(0) += 1;
        if record.status == COMPLETED {
            summary.completed += 1;
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
    summary.review_rate =
        (summary.completed > 0).then(|| round(reviewed as f64 / summary.completed as f64, 4));
    summary.unsupported_fact_rate = (summary.counts.claims > 0).then(|| {
        round(
            summary.counts.unsupported_claims as f64 / summary.counts.claims as f64,
            4,
        )
    });
    summary
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
            .categories
            .iter()
            .map(|category| ("category", category.clone())),
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
    let by = |key: fn(&DocumentRecord) -> &str| {
        let mut groups: BTreeMap<String, Vec<&DocumentRecord>> = BTreeMap::new();
        for record in records {
            groups
                .entry(key(record).to_owned())
                .or_default()
                .push(record);
        }
        groups
            .into_iter()
            .map(|(value, members)| (value, distributions(members.iter().copied())))
            .filter(|(_, distributions)| !distributions.is_empty())
            .collect()
    };
    Latency {
        overall: distributions(records.iter()),
        by_page_bucket: by(|record| &record.page_bucket),
        by_kind: by(|record| &record.kind),
        by_text_layer: by(|record| &record.text_layer),
    }
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
    let mut confidences = Vec::new();
    let mut rows = Vec::new();
    for record in records {
        let Some(measure) = &record.ocr else {
            continue;
        };
        pooled.accumulate(measure);
        confidences.extend(measure.mean_confidence);
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
    pooled.mean_confidence = (!confidences.is_empty())
        .then(|| confidences.iter().sum::<f64>() / confidences.len() as f64);
    OcrReport {
        aggregate: Some(OcrFigures::of(&pooled)),
        documents: rows,
    }
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
}

pub fn build(info: RunInfo, records: Vec<DocumentRecord>) -> Report {
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
        summary: summarize(&records),
        groups: groups(&records),
        latency: latency(&records),
        ocr: ocr_report(&records),
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
