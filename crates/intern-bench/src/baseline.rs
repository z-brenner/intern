//! The baseline a run is held to, and how it is held.
//!
//! `bench/baseline.json` keeps every document's status and boolean scores,
//! the aggregate rates and trap counts, and the p50/p95 of every stage.
//!
//! * **Replay** is deterministic, so it is held document by document, as
//!   `intern-evaluate` holds the fixture corpus: a score that was good and
//!   is now bad, or a status that changed, fails the run (exit 2). A trap
//!   score (`date_forbidden`, `party_forbidden`, `unsafe_ready`,
//!   `needless_review`) is good when false. `ready` alone is not compared:
//!   it is a routing decision, and `readiness_match` judges it.
//! * **Live** inference moves a document now and then for reasons no change
//!   made, so per-document flips are reported but only the aggregate fails
//!   the run: a rate may not fall by more than one document's worth
//!   (1/total) below the baseline, and the trap-date, forbidden-party, and
//!   unsafe-ready counts may not rise at all. Both are computed over the
//!   documents the run and the baseline share, so a document added since
//!   is reported as new rather than moving the rates.
//! * **Latency** is gated only live and only on request (`--latency-gate
//!   RATIO`): the p95 of the total and of every stage may not exceed RATIO
//!   times the baseline's. A stage whose baseline p95 is under a
//!   millisecond is too small to time and is not gated.
//!
//! There are no absolute thresholds: a baseline records what the code
//! achieved, and accepting a new state of the world is writing a new one.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    record::{DocumentRecord, PENDING, is_unscorable},
    report::Report,
    score::bad_when_true,
    stats::round,
    timing::{Unit, unit},
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
    /// changed - gated in replay, reported live.
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
    for record in &report.records {
        if record.status == PENDING {
            comparison.pending.push(record.id.clone());
            continue;
        }
        let Some(expected) = baseline.documents.get(&record.id) else {
            comparison.new.push(record.id.clone());
            continue;
        };
        if record.status != expected.status {
            comparison.document_regressions.push(format!(
                "{}: was {}, now {}",
                record.id, expected.status, record.status
            ));
            continue;
        }
        shared.push((record, expected));
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
    if live && let Some(ratio) = latency_gate {
        comparison.latency = latency_checks(report, baseline, ratio);
    }

    if live {
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
                        "{} p95: {} ms against a baseline of {} ms (limit {} ms)",
                        check.metric, check.current_p95, check.baseline_p95, check.limit
                    )
                }),
        );
    } else {
        comparison.failures = comparison.document_regressions.clone();
    }
    comparison.passed = comparison.failures.is_empty();
    comparison
}

fn is_good(key: &str, value: bool) -> bool {
    if bad_when_true(key) { !value } else { value }
}

fn describe(value: Option<bool>) -> String {
    value.map_or_else(|| "absent".to_owned(), |flag| flag.to_string())
}

/// Rates and trap counts over the documents both sides scored.
fn aggregate_checks(shared: &[(&DocumentRecord, &BaselineDocument)]) -> Vec<AggregateCheck> {
    let mut rates: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    for (record, expected) in shared {
        for (key, was) in &expected.scores {
            if key == "ready" || is_unscorable(&record.status) {
                continue;
            }
            let Some(now) = record.scores.get(key).and_then(Value::as_bool) else {
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

fn latency_checks(report: &Report, baseline: &Baseline, ratio: f64) -> Vec<LatencyCheck> {
    baseline
        .latency
        .iter()
        .filter(|(_, percentiles)| percentiles.p95 >= 1.0)
        .filter_map(|(metric, percentiles)| {
            let current = report.latency.overall.get(metric)?;
            let limit = round(percentiles.p95 * ratio, 3);
            Some(LatencyCheck {
                metric: metric.clone(),
                baseline_p95: percentiles.p95,
                current_p95: current.p95,
                limit,
                regressed: current.p95 > limit,
            })
        })
        .collect()
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
            vec!["total_ms p95: 1600 ms against a baseline of 1000 ms (limit 1500 ms)"]
        );
        assert!(compare(&report("live", after(&[], 1_400.0)), &baseline, Some(1.5)).passed);
    }
}
