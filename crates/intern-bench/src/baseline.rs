//! The baseline a run is held to, and how it is held.
//!
//! `bench/baseline.json` keeps every document's status, boolean scores and
//! (for a document that completed) stage timings, the aggregate rates and
//! trap counts, and the p50/p95 of every stage.
//!
//! * **Replay** is deterministic, so it is held document by document, as
//!   `intern-evaluate` holds the fixture corpus: a score that was good and
//!   is now bad, or a status that changed, fails the run (exit 2). A trap
//!   score (`date_forbidden`, `party_forbidden`, `unsafe_ready`,
//!   `needless_review`) is good when false. `ready` alone is not compared:
//!   it is a routing decision, and `readiness_match` judges it.
//! * **Live** inference moves a document now and then for reasons no change
//!   made, so per-document score flips are reported but only the aggregate
//!   fails the run: a rate may not fall by more than one document's worth
//!   (1/total) below the baseline, and the trap-date, forbidden-party, and
//!   unsafe-ready counts may not rise at all. Both are computed over the
//!   documents the run and the baseline share - a document added since is
//!   reported as new rather than moving the rates - and a shared document
//!   that failed counts in them as the miss it is: every score the
//!   baseline holds for it that the failed record lacks counts as wrong
//!   (good-when-true) or as no trap (good-when-false). A document that
//!   completed in the baseline and now failed (or could not be scored)
//!   also fails a live run on its own: a document Intern stopped naming is
//!   never noise.
//! * **Latency** is gated only live and only on request (`--latency-gate
//!   RATIO`), over the documents that completed in both the run and the
//!   baseline, each side's p95 computed from those documents' own timings:
//!   the p95 of the total and of every stage may not exceed RATIO times the
//!   baseline's. A subset run (`--only`) is compared with the same subset
//!   of the baseline, and a document that failed on either side is in
//!   neither p95, so a slow document that starts failing cannot make the
//!   run look faster (it fails the run instead). A stage whose baseline p95
//!   is under a millisecond is too small to time and is not gated.
//!
//! There are no absolute thresholds: a baseline records what the code
//! achieved, and accepting a new state of the world is writing a new one.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    record::{COMPLETED, DocumentRecord, EXTRACTION_FAILED, MODEL_FAILED, PENDING, is_unscorable},
    report::Report,
    score::bad_when_true,
    stats::{Distribution, round},
    timing::{self, Unit, unit},
};

pub const BASELINE_SCHEMA_VERSION: u32 = 1;

/// Trap counts: the number of documents with the score true.
const COUNTS: [(&str, &str); 3] = [
    ("trap_date_count", "date_forbidden"),
    ("forbidden_party_count", "party_forbidden"),
    ("unsafe_ready_count", "unsafe_ready"),
];

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Baseline {
    pub schema_version: u32,
    pub mode: String,
    pub documents: BTreeMap<String, BaselineDocument>,
    /// Every boolean rate, and the trap counts (`*_count`).
    pub aggregate: BTreeMap<String, f64>,
    /// p50 and p95 of every timed stage, in milliseconds.
    pub latency: BTreeMap<String, Percentiles>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BaselineDocument {
    pub status: String,
    pub scores: BTreeMap<String, bool>,
    /// Every stage time of a document that completed, in milliseconds, so
    /// the latency gate compares the same documents on both sides. Empty in
    /// a baseline written before it was kept.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub timings: BTreeMap<String, f64>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Percentiles {
    pub p50: f64,
    pub p95: f64,
}

impl Baseline {
    /// A pending document has nothing to hold a later run to, so it is
    /// left out: once recorded it arrives as new, not as a regression from
    /// "pending".
    pub fn from_report(report: &Report) -> Self {
        let documents = report
            .records
            .iter()
            .filter(|record| record.status != PENDING)
            .map(|record| {
                (
                    record.id.clone(),
                    BaselineDocument {
                        status: record.status.clone(),
                        scores: bool_scores(record),
                        timings: stage_times(record),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut aggregate = report
            .summary
            .rates
            .iter()
            .filter(|(key, _)| key.as_str() != "ready")
            .map(|(key, rate)| (key.clone(), rate.rate))
            .collect::<BTreeMap<_, _>>();
        for (name, key) in COUNTS {
            let count = report
                .records
                .iter()
                .filter(|record| record.bool_score(key) == Some(true))
                .count();
            aggregate.insert(name.to_owned(), count as f64);
        }
        let latency = report
            .latency
            .overall
            .iter()
            .filter(|(metric, _)| unit(metric) == Some(Unit::Milliseconds))
            .map(|(metric, distribution)| {
                (
                    metric.clone(),
                    Percentiles {
                        p50: distribution.p50,
                        p95: distribution.p95,
                    },
                )
            })
            .collect();
        Self {
            schema_version: BASELINE_SCHEMA_VERSION,
            mode: report.mode.clone(),
            documents,
            aggregate,
            latency,
        }
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let baseline: Self = serde_json::from_slice(bytes)
            .map_err(|error| format!("cannot parse baseline: {error}"))?;
        if baseline.schema_version != BASELINE_SCHEMA_VERSION {
            return Err(format!(
                "baseline schema {} is not the supported {BASELINE_SCHEMA_VERSION}",
                baseline.schema_version
            ));
        }
        Ok(baseline)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_value(self)
            .and_then(|value| serde_json::to_string_pretty(&value))
            .unwrap_or_default()
            + "\n"
    }
}

fn bool_scores(record: &DocumentRecord) -> BTreeMap<String, bool> {
    record
        .scores
        .iter()
        .filter_map(|(key, value)| value.as_bool().map(|flag| (key.clone(), flag)))
        .collect()
}

/// The millisecond timings of a document that completed; none for any
/// other, whose time measures how long it took to fail.
fn stage_times(record: &DocumentRecord) -> BTreeMap<String, f64> {
    if record.status != COMPLETED {
        return BTreeMap::new();
    }
    timing::METRICS
        .iter()
        .filter(|(_, unit)| *unit == Unit::Milliseconds)
        .filter_map(|(metric, _)| {
            timing::get(&record.timings, metric).map(|value| ((*metric).to_owned(), value))
        })
        .collect()
}

/// A status that means Intern gave the document no name.
fn is_failure(status: &str) -> bool {
    matches!(status, EXTRACTION_FAILED | MODEL_FAILED) || is_unscorable(status)
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AggregateCheck {
    pub metric: String,
    pub baseline: f64,
    pub current: f64,
    /// The worst value the gate accepts.
    pub limit: f64,
    pub regressed: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LatencyCheck {
    pub metric: String,
    /// How many documents, completed in both runs, both p95s are over.
    #[serde(default)]
    pub documents: usize,
    pub baseline_p95: f64,
    pub current_p95: f64,
    pub limit: f64,
    pub regressed: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BaselineComparison {
    pub baseline_mode: String,
    /// What can fail this run: `documents` (replay), `aggregate` and,
    /// with `--latency-gate`, `latency` (live).
    pub gates: Vec<String>,
    pub passed: bool,
    /// Every gated regression, one line each.
    pub failures: Vec<String>,
    /// Per-document scores that went from good to bad, or statuses that
    /// changed - gated in replay; reported live, where only a document that
    /// stopped completing is gated on its own.
    pub document_regressions: Vec<String>,
    pub document_improvements: Vec<String>,
    pub aggregate: Vec<AggregateCheck>,
    pub latency: Vec<LatencyCheck>,
    /// Documents scored now that the baseline does not have.
    pub new: Vec<String>,
    /// Documents the baseline has that this run did not score.
    pub missing: Vec<String>,
    pub pending: Vec<String>,
}

impl BaselineComparison {
    /// The per-document regressions no gate failed the run on: in a live
    /// run, the score flips and status changes other than a document that
    /// stopped completing, which is a failure of its own.
    pub fn ungated_regressions(&self) -> impl Iterator<Item = &String> {
        self.document_regressions.iter().filter(|line| {
            !self
                .failures
                .iter()
                .any(|failure| failure.starts_with(line.as_str()))
        })
    }
}

pub fn compare(
    report: &Report,
    baseline: &Baseline,
    latency_gate: Option<f64>,
) -> BaselineComparison {
    let live = report.mode == "live";
    let mut comparison = BaselineComparison {
        baseline_mode: baseline.mode.clone(),
        ..BaselineComparison::default()
    };
    comparison
        .gates
        .push(if live { "aggregate" } else { "documents" }.to_owned());
    if live && latency_gate.is_some() {
        comparison.gates.push("latency".to_owned());
    }

    let mut shared = Vec::new();
    let mut stopped_completing = Vec::new();
    let mut gone_pending = Vec::new();
    for record in &report.records {
        if record.status == PENDING {
            comparison.pending.push(record.id.clone());
            // Pending is for a document nobody has recorded yet. One the
            // baseline already scores would lose its coverage unnoticed.
            if baseline.documents.contains_key(&record.id) {
                gone_pending.push(record.id.clone());
            }
            continue;
        }
        let Some(expected) = baseline.documents.get(&record.id) else {
            comparison.new.push(record.id.clone());
            continue;
        };
        // A document whose status changed still belongs to the aggregate:
        // a failed record is scored as a miss, so it pulls the rates down.
        shared.push((record, expected));
        if record.status != expected.status {
            let line = format!(
                "{}: was {}, now {}",
                record.id, expected.status, record.status
            );
            // A document that failed in the baseline and completes now has
            // recovered. Its scores are still held to the baseline's below:
            // a failed record's misses cannot regress, but a trap it did not
            // spring then can be sprung now.
            if record.status == COMPLETED
                && matches!(expected.status.as_str(), EXTRACTION_FAILED | MODEL_FAILED)
            {
                comparison.document_improvements.push(line);
            } else {
                if expected.status == COMPLETED && is_failure(&record.status) {
                    stopped_completing.push(line.clone());
                }
                comparison.document_regressions.push(line);
                continue;
            }
        }
        for (key, was) in &expected.scores {
            if key == "ready" {
                continue;
            }
            let now = record.bool_score(key);
            let was_good = is_good(key, *was);
            let now_good = now.is_some_and(|now| is_good(key, now));
            let line = format!("{}: {key} was {was}, now {}", record.id, describe(now));
            if was_good && !now_good {
                comparison.document_regressions.push(line);
            } else if !was_good && now_good {
                comparison.document_improvements.push(line);
            }
        }
    }
    let run = report
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    comparison.missing = baseline
        .documents
        .keys()
        .filter(|id| !run.contains(id.as_str()))
        .filter(|_| report.corpus.only.is_empty())
        .cloned()
        .collect();

    comparison.aggregate = aggregate_checks(&shared);
    let mut latency_refused = None;
    if live && let Some(ratio) = latency_gate {
        match latency_checks(report, baseline, ratio) {
            Ok(checks) => comparison.latency = checks,
            Err(reason) => latency_refused = Some(format!("latency: {reason}")),
        }
    }

    if live {
        comparison.failures.extend(
            stopped_completing.into_iter().map(|line| {
                format!("{line} (a document that stopped completing fails a live run)")
            }),
        );
        comparison.failures.extend(
            comparison
                .aggregate
                .iter()
                .filter(|check| check.regressed)
                .map(|check| {
                    format!(
                        "{}: {} against a baseline of {} (limit {})",
                        check.metric, check.current, check.baseline, check.limit
                    )
                }),
        );
        comparison.failures.extend(
            comparison
                .latency
                .iter()
                .filter(|check| check.regressed)
                .map(|check| {
                    format!(
                        "{} p95: {} ms against a baseline of {} ms (limit {} ms, {} documents)",
                        check.metric,
                        check.current_p95,
                        check.baseline_p95,
                        check.limit,
                        check.documents
                    )
                }),
        );
        comparison.failures.extend(latency_refused);
    } else {
        comparison.failures = comparison.document_regressions.clone();
    }
    // A full run that no longer scores a document the baseline holds has
    // dropped its coverage - a gold entry deleted by accident reads exactly
    // like this. Dropping one on purpose means writing a new baseline.
    comparison
        .failures
        .extend(comparison.missing.iter().map(|id| {
            format!(
                "{id}: in the baseline but not in this run (its regression coverage would be lost; write a new baseline to drop it on purpose)"
            )
        }));
    comparison
        .failures
        .extend(gone_pending.iter().map(|id| {
            format!(
                "{id}: in the baseline but pending in the gold, so nothing scores it (record it again, or write a new baseline to drop it on purpose)"
            )
        }));
    comparison.passed = comparison.failures.is_empty();
    comparison
}

fn is_good(key: &str, value: bool) -> bool {
    if bad_when_true(key) { !value } else { value }
}

fn describe(value: Option<bool>) -> String {
    value.map_or_else(|| "absent".to_owned(), |flag| flag.to_string())
}

/// What a record says about a score the baseline holds. A record that
/// failed is a miss on every score it lacks: wrong where true is good, no
/// trap where true is bad. Any other record that lacks the score says
/// nothing about it.
fn current_score(record: &DocumentRecord, key: &str) -> Option<bool> {
    record.scores.get(key).and_then(Value::as_bool).or_else(|| {
        matches!(record.status.as_str(), EXTRACTION_FAILED | MODEL_FAILED).then_some(false)
    })
}

/// Rates and trap counts over the documents both sides have, a failed one
/// counted as a miss.
fn aggregate_checks(shared: &[(&DocumentRecord, &BaselineDocument)]) -> Vec<AggregateCheck> {
    let mut rates: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    for (record, expected) in shared {
        for (key, was) in &expected.scores {
            if key == "ready" || is_unscorable(&record.status) {
                continue;
            }
            let Some(now) = current_score(record, key) else {
                continue;
            };
            let entry = rates.entry(key.as_str()).or_insert((0, 0, 0));
            entry.0 += usize::from(*was);
            entry.1 += usize::from(now);
            entry.2 += 1;
        }
    }
    let mut checks = Vec::new();
    for (key, (was, now, total)) in rates {
        let baseline = was as f64 / total as f64;
        let current = now as f64 / total as f64;
        let tolerance = 1.0 / total as f64;
        let (limit, regressed) = if bad_when_true(key) {
            let limit = baseline + tolerance;
            (limit, current > limit + 1e-9)
        } else {
            let limit = baseline - tolerance;
            (limit, current < limit - 1e-9)
        };
        checks.push(AggregateCheck {
            metric: key.to_owned(),
            baseline: round(baseline, 4),
            current: round(current, 4),
            limit: round(limit, 4),
            regressed,
        });
    }
    for (name, key) in COUNTS {
        let count = |pick: &dyn Fn(&(&DocumentRecord, &BaselineDocument)) -> bool| {
            shared.iter().filter(|pair| pick(pair)).count() as f64
        };
        let baseline = count(&|(_, expected)| expected.scores.get(key) == Some(&true));
        let current = count(&|(record, _)| record.bool_score(key) == Some(true));
        checks.push(AggregateCheck {
            metric: name.to_owned(),
            baseline,
            current,
            limit: baseline,
            regressed: current > baseline,
        });
    }
    checks
}

/// p95 of every stage over the documents that completed in both the run
/// and the baseline. A baseline written before it kept per-document
/// timings can only be compared whole: then the run must cover exactly its
/// documents, every one completed on both sides.
fn latency_checks(
    report: &Report,
    baseline: &Baseline,
    ratio: f64,
) -> Result<Vec<LatencyCheck>, String> {
    let pairs = report
        .records
        .iter()
        .filter(|record| record.status == COMPLETED)
        .filter_map(|record| {
            baseline
                .documents
                .get(&record.id)
                .filter(|expected| expected.status == COMPLETED)
                .map(|expected| (record, expected))
        })
        .collect::<Vec<_>>();
    if pairs.is_empty() {
        return Err("no document completed in both this run and the baseline".to_owned());
    }
    if pairs
        .iter()
        .all(|(_, expected)| expected.timings.is_empty())
    {
        return whole_corpus_latency(report, baseline, ratio, pairs.len());
    }
    // A stage the baseline measured for a document and this run did not -
    // a worker that reports no timings, say - would shrink the sample, or
    // drop the stage, and the gate would pass on less than it claims.
    let mut unmeasured = Vec::new();
    let checks = timing::METRICS
        .iter()
        .filter(|(_, unit)| *unit == Unit::Milliseconds)
        .filter_map(|(metric, _)| {
            let mut was = Vec::new();
            let mut now = Vec::new();
            for (record, expected) in &pairs {
                let Some(before) = expected.timings.get(*metric) else {
                    continue;
                };
                match timing::get(&record.timings, metric) {
                    Some(current) => {
                        was.push(*before);
                        now.push(current);
                    }
                    None => unmeasured.push(format!("{} {metric}", record.id)),
                }
            }
            let baseline_p95 = Distribution::of(&was)?.p95;
            let current_p95 = Distribution::of(&now)?.p95;
            (baseline_p95 >= 1.0)
                .then(|| latency_check(metric, was.len(), baseline_p95, current_p95, ratio))
        })
        .collect();
    if !unmeasured.is_empty() {
        return Err(unmeasured_error(&unmeasured));
    }
    Ok(checks)
}

fn unmeasured_error(unmeasured: &[String]) -> String {
    const SHOWN: usize = 5;
    let mut listed = unmeasured.iter().take(SHOWN).cloned().collect::<Vec<_>>();
    if unmeasured.len() > SHOWN {
        listed.push(format!("and {} more", unmeasured.len() - SHOWN));
    }
    format!(
        "this run did not measure {} stage time(s) the baseline holds, so the gate would compare less than it says: {}",
        unmeasured.len(),
        listed.join(", ")
    )
}

fn whole_corpus_latency(
    report: &Report,
    baseline: &Baseline,
    ratio: f64,
    completed_in_both: usize,
) -> Result<Vec<LatencyCheck>, String> {
    let same_documents = report.corpus.only.is_empty()
        && report.records.len() == baseline.documents.len()
        && completed_in_both == baseline.documents.len();
    if !same_documents {
        return Err(
            "the baseline keeps no per-document timings, so it can only be compared whole, and this run does not cover exactly its documents, all completed; write the baseline again"
                .to_owned(),
        );
    }
    // Every document of the run must have measured every stage the
    // baseline has: a stage measured on fewer would compare a smaller
    // sample, and the document it lacks may be the slow one.
    let unmeasured = baseline
        .latency
        .keys()
        .filter_map(|metric| {
            let measured = report
                .latency
                .overall
                .get(metric)
                .map_or(0, |distribution| distribution.count);
            (measured != completed_in_both).then(|| {
                format!("{metric} (measured on {measured} of {completed_in_both} documents)")
            })
        })
        .collect::<Vec<_>>();
    if !unmeasured.is_empty() {
        return Err(unmeasured_error(&unmeasured));
    }
    Ok(baseline
        .latency
        .iter()
        .filter(|(_, percentiles)| percentiles.p95 >= 1.0)
        .filter_map(|(metric, percentiles)| {
            let current = report.latency.overall.get(metric)?;
            Some(latency_check(
                metric,
                current.count,
                percentiles.p95,
                current.p95,
                ratio,
            ))
        })
        .collect())
}

fn latency_check(
    metric: &str,
    documents: usize,
    baseline_p95: f64,
    current_p95: f64,
    ratio: f64,
) -> LatencyCheck {
    let limit = round(baseline_p95 * ratio, 3);
    LatencyCheck {
        metric: metric.to_owned(),
        documents,
        baseline_p95,
        current_p95,
        limit,
        regressed: current_p95 > limit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        report::{RunInfo, build},
        timing,
    };
    use serde_json::json;

    fn record(id: &str, status: &str, scores: Value, total_ms: f64) -> DocumentRecord {
        let mut record = DocumentRecord {
            id: id.into(),
            status: status.into(),
            scores: serde_json::from_value(scores).unwrap(),
            timings: timing::empty(),
            ..DocumentRecord::default()
        };
        record.timings.insert("total_ms".into(), json!(total_ms));
        record
    }

    fn report(mode: &str, records: Vec<DocumentRecord>) -> Report {
        build(
            RunInfo {
                mode: mode.into(),
                ..RunInfo::default()
            },
            records,
        )
    }

    fn before() -> Baseline {
        let mut records = (0..9)
            .map(|index| {
                record(
                    &format!("doc-{index}"),
                    "completed",
                    json!({"filename_correct": true, "date_forbidden": false, "ready": true}),
                    1_000.0,
                )
            })
            .collect::<Vec<_>>();
        records.push(record(
            "doc-9",
            "completed",
            json!({"filename_correct": false, "date_forbidden": false, "ready": false}),
            1_000.0,
        ));
        Baseline::from_report(&report("replay", records))
    }

    /// A full run that no longer has a document the baseline holds fails,
    /// in replay and live alike: its coverage would otherwise vanish. A run
    /// limited with `--only` is not held to the documents it left out.
    #[test]
    fn a_full_run_that_drops_a_baseline_document_fails() {
        let baseline = before();
        let nine = |mode: &str| {
            let records = (0..9)
                .map(|index| {
                    record(
                        &format!("doc-{index}"),
                        "completed",
                        json!({"filename_correct": true, "date_forbidden": false, "ready": true}),
                        1_000.0,
                    )
                })
                .collect::<Vec<_>>();
            report(mode, records)
        };
        for mode in ["replay", "live"] {
            let comparison = compare(&nine(mode), &baseline, None);
            assert_eq!(comparison.missing, vec!["doc-9".to_owned()], "{mode}");
            assert!(!comparison.passed, "{mode}");
            assert!(
                comparison
                    .failures
                    .iter()
                    .any(|line| line.starts_with("doc-9:")),
                "{mode}: {:?}",
                comparison.failures
            );
        }
        let mut subset = nine("replay");
        subset.corpus.only = (0..9).map(|index| format!("doc-{index}")).collect();
        let comparison = compare(&subset, &baseline, None);
        assert!(comparison.missing.is_empty());
        assert!(comparison.passed, "{:?}", comparison.failures);
    }

    /// A document that failed in the baseline and completes now is an
    /// improvement, not a status regression; a trap it springs now is
    /// still a regression.
    #[test]
    fn a_failed_document_that_recovers_is_an_improvement() {
        let mut records = (0..9)
            .map(|index| {
                record(
                    &format!("doc-{index}"),
                    "completed",
                    json!({"filename_correct": true, "date_forbidden": false, "ready": true}),
                    1_000.0,
                )
            })
            .collect::<Vec<_>>();
        records.push(record(
            "doc-9",
            MODEL_FAILED,
            json!({"filename_correct": false, "date_forbidden": false}),
            1_000.0,
        ));
        let baseline = Baseline::from_report(&report("replay", records.clone()));

        records[9] = record(
            "doc-9",
            COMPLETED,
            json!({"filename_correct": true, "date_forbidden": false, "ready": true}),
            1_000.0,
        );
        for mode in ["replay", "live"] {
            let comparison = compare(&report(mode, records.clone()), &baseline, None);
            assert!(comparison.passed, "{mode}: {:?}", comparison.failures);
            assert!(comparison.document_regressions.is_empty(), "{mode}");
            assert!(
                comparison
                    .document_improvements
                    .contains(&"doc-9: was model_failed, now completed".to_owned()),
                "{mode}: {:?}",
                comparison.document_improvements
            );
        }

        records[9] = record(
            "doc-9",
            COMPLETED,
            json!({"filename_correct": false, "date_forbidden": true, "ready": true}),
            1_000.0,
        );
        let comparison = compare(&report("replay", records), &baseline, None);
        assert!(!comparison.passed);
        assert_eq!(
            comparison.failures,
            vec!["doc-9: date_forbidden was false, now true".to_owned()]
        );
    }

    /// Turning a baselined document back to pending would take it out of
    /// every gate; only a document the baseline never held may be pending.
    #[test]
    fn a_baselined_document_that_turns_pending_fails() {
        let baseline = before();
        let mut records = (0..9)
            .map(|index| {
                record(
                    &format!("doc-{index}"),
                    "completed",
                    json!({"filename_correct": true, "date_forbidden": false, "ready": true}),
                    1_000.0,
                )
            })
            .collect::<Vec<_>>();
        records.push(record("doc-9", PENDING, json!({}), 0.0));
        records.push(record("doc-new", PENDING, json!({}), 0.0));
        let mut run = report("replay", records);
        let comparison = compare(&run, &baseline, None);
        assert!(comparison.missing.is_empty());
        assert_eq!(comparison.pending, vec!["doc-9", "doc-new"]);
        assert!(!comparison.passed);
        assert_eq!(comparison.failures.len(), 1, "{:?}", comparison.failures);
        assert!(comparison.failures[0].starts_with("doc-9: in the baseline but pending"));

        // Selecting it with --only does not excuse it either.
        run.corpus.only = vec!["doc-9".into()];
        run.records.retain(|record| record.id == "doc-9");
        assert!(!compare(&run, &baseline, None).passed);
    }

    #[test]
    fn a_written_baseline_keeps_scores_rates_counts_and_stage_percentiles() {
        let baseline = before();
        assert_eq!(baseline.documents.len(), 10);
        assert!(baseline.documents["doc-0"].scores["filename_correct"]);
        assert_eq!(baseline.aggregate["filename_correct"], 0.9);
        assert_eq!(baseline.aggregate["trap_date_count"], 0.0);
        assert!(!baseline.aggregate.contains_key("ready"));
        assert_eq!(
            baseline.latency["total_ms"],
            Percentiles {
                p50: 1_000.0,
                p95: 1_000.0
            }
        );
        let reparsed = Baseline::parse(baseline.to_json().as_bytes()).unwrap();
        assert_eq!(reparsed.documents.len(), 10);
    }

    fn after(flips: &[(&str, Value)], total_ms: f64) -> Vec<DocumentRecord> {
        let baseline = before();
        baseline
            .documents
            .iter()
            .map(|(id, document)| {
                let scores = flips
                    .iter()
                    .find(|(flipped, _)| flipped == id)
                    .map(|(_, scores)| scores.clone())
                    .unwrap_or_else(|| serde_json::to_value(&document.scores).unwrap());
                record(id, "completed", scores, total_ms)
            })
            .collect()
    }

    #[test]
    fn replay_is_held_document_by_document() {
        let baseline = before();
        let lost = json!({"filename_correct": false, "date_forbidden": false, "ready": true});
        let gained = json!({"filename_correct": true, "date_forbidden": false, "ready": true});
        let mut records = after(&[("doc-0", lost), ("doc-9", gained)], 1_000.0);
        records.push(record(
            "doc-new",
            "completed",
            json!({"filename_correct": false}),
            1.0,
        ));
        let comparison = compare(&report("replay", records), &baseline, Some(1.5));
        assert_eq!(comparison.gates, vec!["documents"]);
        assert!(!comparison.passed);
        assert_eq!(
            comparison.failures,
            vec!["doc-0: filename_correct was true, now false"]
        );
        assert_eq!(
            comparison.document_improvements,
            vec!["doc-9: filename_correct was false, now true"]
        );
        assert_eq!(comparison.new, vec!["doc-new"]);
        assert!(comparison.latency.is_empty(), "never gated in replay");

        // A status change is a regression; a sprung trap is too.
        let mut records = after(
            &[(
                "doc-1",
                json!({"filename_correct": true, "date_forbidden": true}),
            )],
            1_000.0,
        );
        records[2].status = "stale_prompt".into();
        let comparison = compare(&report("replay", records), &baseline, None);
        assert_eq!(
            comparison.failures,
            vec![
                "doc-1: date_forbidden was false, now true",
                "doc-2: was completed, now stale_prompt"
            ]
        );
    }

    #[test]
    fn live_reports_flips_but_fails_only_on_the_aggregate() {
        let baseline = before();
        // One document's worth of drop is noise; flips are listed, not failed.
        let lost = json!({"filename_correct": false, "date_forbidden": false, "ready": true});
        let comparison = compare(
            &report("live", after(&[("doc-0", lost.clone())], 1_000.0)),
            &baseline,
            None,
        );
        assert_eq!(comparison.gates, vec!["aggregate"]);
        assert!(comparison.passed, "{:?}", comparison.failures);
        assert_eq!(comparison.document_regressions.len(), 1);

        // Two documents' worth is a regression.
        let comparison = compare(
            &report(
                "live",
                after(&[("doc-0", lost.clone()), ("doc-1", lost)], 1_000.0),
            ),
            &baseline,
            None,
        );
        assert!(!comparison.passed);
        assert_eq!(comparison.failures.len(), 1);
        assert!(
            comparison.failures[0].starts_with("filename_correct: 0.7"),
            "{:?}",
            comparison.failures
        );

        // A single sprung trap is a regression however small.
        let trapped = json!({"filename_correct": true, "date_forbidden": true, "ready": true});
        let comparison = compare(
            &report("live", after(&[("doc-3", trapped)], 1_000.0)),
            &baseline,
            None,
        );
        assert!(!comparison.passed);
        assert!(
            comparison
                .failures
                .iter()
                .any(|line| line.starts_with("trap_date_count")),
            "{:?}",
            comparison.failures
        );
    }

    #[test]
    fn the_latency_gate_applies_live_and_only_when_asked() {
        let baseline = before();
        let slow = report("live", after(&[], 1_600.0));
        assert!(
            compare(&slow, &baseline, None).passed,
            "no gate without a ratio"
        );
        let gated = compare(&slow, &baseline, Some(1.5));
        assert_eq!(gated.gates, vec!["aggregate", "latency"]);
        assert!(!gated.passed);
        assert_eq!(
            gated.failures,
            vec![
                "total_ms p95: 1600 ms against a baseline of 1000 ms (limit 1500 ms, 10 documents)"
            ]
        );
        assert!(compare(&report("live", after(&[], 1_400.0)), &baseline, Some(1.5)).passed);
    }

    /// Ten documents the baseline has completed, in a live run where some
    /// of them now fail: each a miss on every score the baseline holds.
    fn failing(failed: &[(&str, &str)]) -> Report {
        let mut records = after(&[], 1_000.0);
        for record in &mut records {
            if let Some((_, status)) = failed.iter().find(|(id, _)| *id == record.id) {
                record.status = (*status).to_owned();
                record.scores = serde_json::from_value(
                    json!({"filename_correct": false, "date_forbidden": false, "ready": false}),
                )
                .unwrap();
                record.timings = timing::empty();
            }
        }
        report("live", records)
    }

    #[test]
    fn live_documents_that_stop_completing_count_and_fail_the_run() {
        let baseline = before();
        // Two of ten: the rate falls by two documents' worth, and each
        // document that stopped completing fails the run on its own.
        let comparison = compare(
            &failing(&[("doc-0", "model_failed"), ("doc-1", "extraction_failed")]),
            &baseline,
            None,
        );
        assert!(!comparison.passed);
        assert!(
            comparison
                .failures
                .contains(&"doc-0: was completed, now model_failed (a document that stopped completing fails a live run)".to_owned()),
            "{:?}",
            comparison.failures
        );
        let filename = comparison
            .aggregate
            .iter()
            .find(|check| check.metric == "filename_correct")
            .unwrap();
        assert_eq!(
            (filename.baseline, filename.current, filename.regressed),
            (0.9, 0.7, true),
            "the failed documents are in the rate, as misses"
        );

        // One document is within the aggregate's tolerance, but a
        // document Intern stopped naming is never noise.
        let comparison = compare(&failing(&[("doc-3", "model_failed")]), &baseline, None);
        assert!(!comparison.passed);
        assert!(comparison.aggregate.iter().all(|check| !check.regressed));
        assert_eq!(comparison.failures.len(), 1, "{:?}", comparison.failures);
        assert_eq!(
            comparison.ungated_regressions().count(),
            0,
            "gated, so not also listed as an ungated flip"
        );

        // Every document failing - the worker broke - fails, and the rates
        // say how badly instead of having nothing to compare.
        let all = (0..10)
            .map(|index| format!("doc-{index}"))
            .collect::<Vec<_>>();
        let every = all
            .iter()
            .map(|id| (id.as_str(), "extraction_failed"))
            .collect::<Vec<_>>();
        let comparison = compare(&failing(&every), &baseline, Some(1.5));
        assert!(!comparison.passed);
        let filename = comparison
            .aggregate
            .iter()
            .find(|check| check.metric == "filename_correct")
            .unwrap();
        assert_eq!((filename.current, filename.regressed), (0.0, true));
        assert!(
            comparison
                .failures
                .iter()
                .any(|line| line.starts_with("latency: no document completed")),
            "{:?}",
            comparison.failures
        );
    }

    #[test]
    fn a_failed_record_misses_every_score_it_lacks() {
        let baseline = before();
        let mut records = after(&[], 1_000.0);
        for record in records.iter_mut().take(2) {
            record.status = "model_failed".into();
            record.scores.clear();
        }
        let comparison = compare(&report("live", records), &baseline, None);
        let filename = comparison
            .aggregate
            .iter()
            .find(|check| check.metric == "filename_correct")
            .unwrap();
        assert_eq!(filename.current, 0.7, "absent is a miss on a failure");
        let traps = comparison
            .aggregate
            .iter()
            .find(|check| check.metric == "trap_date_count")
            .unwrap();
        assert!(!traps.regressed, "a failure springs no trap");
    }

    #[test]
    fn live_counts_a_document_that_began_completing_with_its_traps() {
        // The baseline failed on doc-9; now it completes, on a trap date.
        let mut records = after(&[], 1_000.0);
        records[9].status = "model_failed".into();
        records[9].scores = serde_json::from_value(
            json!({"filename_correct": false, "date_forbidden": false, "ready": false}),
        )
        .unwrap();
        let baseline = Baseline::from_report(&report("live", records));
        let now = after(
            &[(
                "doc-9",
                json!({"filename_correct": false, "date_forbidden": true, "ready": true}),
            )],
            1_000.0,
        );
        let comparison = compare(&report("live", now), &baseline, None);
        assert!(!comparison.passed);
        assert!(
            comparison
                .failures
                .iter()
                .any(|line| line.starts_with("trap_date_count")),
            "{:?}",
            comparison.failures
        );
        // Completing is the improvement; the trap it springs is the
        // regression, reported per document and gated in aggregate.
        assert!(
            comparison
                .document_improvements
                .contains(&"doc-9: was model_failed, now completed".to_owned())
        );
        assert!(
            comparison
                .document_regressions
                .contains(&"doc-9: date_forbidden was false, now true".to_owned()),
            "{:?}",
            comparison.document_regressions
        );
        assert!(
            !comparison
                .failures
                .iter()
                .any(|line| line.contains("stopped completing"))
        );
    }

    #[test]
    fn latency_is_compared_over_the_documents_that_completed_in_both() {
        // A slow document in the baseline that now fails: without
        // alignment the run's p95 would drop and look faster.
        let mut records = after(&[], 1_000.0);
        records[9]
            .timings
            .insert("total_ms".into(), json!(10_000.0));
        let baseline = Baseline::from_report(&report("live", records));
        assert_eq!(baseline.documents["doc-9"].timings["total_ms"], 10_000.0);
        let comparison = compare(&failing(&[("doc-9", "model_failed")]), &baseline, Some(1.5));
        assert_eq!(comparison.latency.len(), 1);
        let total = &comparison.latency[0];
        assert_eq!(
            (total.documents, total.baseline_p95, total.current_p95),
            (9, 1_000.0, 1_000.0),
            "the failed document is in neither p95"
        );
        assert!(!total.regressed);
        assert!(!comparison.passed, "the failure itself still fails the run");

        // A subset is held to the same subset of the baseline.
        let mut subset = report(
            "live",
            vec![record(
                "doc-9",
                "completed",
                json!({"filename_correct": false, "date_forbidden": false, "ready": false}),
                14_000.0,
            )],
        );
        subset.corpus.only = vec!["doc-9".into()];
        let comparison = compare(&subset, &baseline, Some(1.5));
        assert!(comparison.passed, "{:?}", comparison.failures);
        assert_eq!(comparison.latency[0].documents, 1);
        assert_eq!(comparison.latency[0].baseline_p95, 10_000.0);
        subset.records[0]
            .timings
            .insert("total_ms".into(), json!(16_000.0));
        assert!(!compare(&subset, &baseline, Some(1.5)).passed);
    }

    /// A stage the baseline measured and this run did not - a worker that
    /// reports no timings - refuses the latency gate instead of comparing a
    /// smaller sample.
    #[test]
    fn the_latency_gate_refuses_a_run_missing_a_measured_stage() {
        let with_worker = |records: &mut Vec<DocumentRecord>| {
            for record in records {
                record
                    .timings
                    .insert("worker_total_ms".into(), json!(200.0));
            }
        };
        let mut records = after(&[], 1_000.0);
        with_worker(&mut records);
        let baseline = Baseline::from_report(&report("live", records));

        let mut measured = after(&[], 1_000.0);
        with_worker(&mut measured);
        assert!(compare(&report("live", measured.clone()), &baseline, Some(1.5)).passed);

        measured[3]
            .timings
            .insert("worker_total_ms".into(), Value::Null);
        let comparison = compare(&report("live", measured), &baseline, Some(1.5));
        assert!(!comparison.passed);
        assert!(
            comparison.failures.iter().any(|line| line.starts_with(
                "latency: this run did not measure 1 stage time(s) the baseline holds"
            ) && line.contains("doc-3 worker_total_ms")),
            "{:?}",
            comparison.failures
        );
    }

    #[test]
    fn a_baseline_without_document_timings_gates_latency_only_whole() {
        let mut baseline = before();
        for document in baseline.documents.values_mut() {
            document.timings.clear();
        }
        let whole = compare(&report("live", after(&[], 1_600.0)), &baseline, Some(1.5));
        assert!(!whole.passed);
        assert_eq!(whole.latency[0].documents, 10);

        // A stage one document did not measure would shrink the sample.
        let mut short = after(&[], 1_000.0);
        short[4].timings.insert("total_ms".into(), Value::Null);
        let short = compare(&report("live", short), &baseline, Some(1.5));
        assert!(!short.passed);
        assert!(
            short
                .failures
                .iter()
                .any(|line| line.starts_with("latency:")
                    && line.contains("total_ms (measured on 9 of 10 documents)")),
            "{:?}",
            short.failures
        );

        let mut subset = report("live", after(&[], 1_000.0));
        subset.records.truncate(2);
        subset.corpus.only = vec!["doc-0".into(), "doc-1".into()];
        let refused = compare(&subset, &baseline, Some(1.5));
        assert!(!refused.passed);
        assert!(
            refused.failures[0].starts_with("latency: the baseline keeps no per-document timings"),
            "{:?}",
            refused.failures
        );
    }
}
