//! Two reports side by side: what a change did.
//!
//! Documents are aligned by id. Rates are compared in percentage points,
//! fractions as differences, counts as differences, and every stage's p50
//! and p95 as a percentage change. A document whose boolean score went
//! from good to bad is *broken*, from bad to good *fixed* - trap scores
//! being good when false, as in the baseline gate.

use std::{collections::BTreeSet, fmt::Write as _};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    markdown::{HEADLINE_MEANS, HEADLINE_RATES, duration, metric_value, percent},
    report::{Rate, Report},
    score::{bad_when_true, lower_is_better},
    stats::round,
    timing::METRICS,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Side {
    pub mode: String,
    pub created_at: String,
    pub git_commit: Option<String>,
    pub documents: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RateDelta {
    pub metric: String,
    pub before: Option<Rate>,
    pub after: Option<Rate>,
    /// After minus before, in percentage points.
    pub delta_points: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ValueDelta {
    pub metric: String,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub delta: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Flip {
    pub id: String,
    pub score: String,
    pub before: Value,
    pub after: Value,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Change {
    pub id: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LatencyDelta {
    pub metric: String,
    pub p50_before: f64,
    pub p50_after: f64,
    pub p50_change_percent: Option<f64>,
    pub p95_before: f64,
    pub p95_after: f64,
    pub p95_change_percent: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Comparison {
    pub before: Side,
    pub after: Side,
    pub rates: Vec<RateDelta>,
    pub means: Vec<ValueDelta>,
    pub counts: Vec<ValueDelta>,
    pub broken: Vec<Flip>,
    pub fixed: Vec<Flip>,
    pub status_changes: Vec<Change>,
    pub filename_changes: Vec<Change>,
    pub only_before: Vec<String>,
    pub only_after: Vec<String>,
    pub latency: Vec<LatencyDelta>,
}

fn side(report: &Report) -> Side {
    Side {
        mode: report.mode.clone(),
        created_at: report.created_at.clone(),
        git_commit: report.git_commit.clone(),
        documents: report.records.len(),
    }
}

fn change_percent(before: f64, after: f64) -> Option<f64> {
    (before > 0.0).then(|| round((after - before) / before * 100.0, 1))
}

fn counts(report: &Report) -> Vec<(&'static str, f64)> {
    let summary = &report.summary;
    let counts = &summary.counts;
    let mut values = vec![
        ("unsafe_ready", counts.unsafe_ready as f64),
        ("trap_dates", counts.trap_dates as f64),
        ("forbidden_parties", counts.forbidden_parties as f64),
        ("spurious_parties", counts.spurious_parties as f64),
        (
            "forbidden_descriptions",
            counts.forbidden_descriptions as f64,
        ),
        ("unsupported_claims", counts.unsupported_claims as f64),
        ("claims", counts.claims as f64),
    ];
    if let Some(rate) = summary.review_rate {
        values.push(("review_rate", rate));
    }
    if let Some(rate) = summary.unsupported_fact_rate {
        values.push(("unsupported_fact_rate", rate));
    }
    values
}

fn is_good(key: &str, value: bool) -> bool {
    if bad_when_true(key) { !value } else { value }
}

pub fn compare(before: &Report, after: &Report) -> Comparison {
    let mut comparison = Comparison {
        before: side(before),
        after: side(after),
        ..Comparison::default()
    };

    let rate_keys = before
        .summary
        .rates
        .keys()
        .chain(after.summary.rates.keys())
        .collect::<BTreeSet<_>>();
    comparison.rates = rate_keys
        .into_iter()
        .map(|key| {
            let was = before.summary.rates.get(key).copied();
            let now = after.summary.rates.get(key).copied();
            RateDelta {
                metric: key.clone(),
                delta_points: was
                    .zip(now)
                    .map(|(was, now)| round((now.rate - was.rate) * 100.0, 1)),
                before: was,
                after: now,
            }
        })
        .collect();
    let mean_keys = before
        .summary
        .means
        .keys()
        .chain(after.summary.means.keys())
        .collect::<BTreeSet<_>>();
    comparison.means = mean_keys
        .into_iter()
        .map(|key| {
            let was = before.summary.means.get(key).map(|mean| mean.mean);
            let now = after.summary.means.get(key).map(|mean| mean.mean);
            ValueDelta {
                metric: key.clone(),
                delta: was.zip(now).map(|(was, now)| round(now - was, 4)),
                before: was,
                after: now,
            }
        })
        .collect();
    let (was_counts, now_counts) = (counts(before), counts(after));
    comparison.counts = was_counts
        .iter()
        .map(|(metric, was)| {
            let now = now_counts
                .iter()
                .find(|(other, _)| other == metric)
                .map(|(_, value)| *value);
            ValueDelta {
                metric: (*metric).to_owned(),
                before: Some(*was),
                delta: now.map(|now| round(now - was, 4)),
                after: now,
            }
        })
        .collect();

    let after_ids = after
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<BTreeSet<_>>();
    for record in &before.records {
        let Some(now) = after.record(&record.id) else {
            comparison.only_before.push(record.id.clone());
            continue;
        };
        if record.status != now.status {
            comparison.status_changes.push(Change {
                id: record.id.clone(),
                before: record.status.clone(),
                after: now.status.clone(),
            });
        }
        if record.filename != now.filename
            && let (Some(was), Some(is)) = (&record.filename, &now.filename)
        {
            comparison.filename_changes.push(Change {
                id: record.id.clone(),
                before: was.clone(),
                after: is.clone(),
            });
        }
        let keys = record
            .scores
            .keys()
            .chain(now.scores.keys())
            .collect::<BTreeSet<_>>();
        for key in keys {
            if key == "ready" {
                continue;
            }
            let was = record.scores.get(key).and_then(Value::as_bool);
            let is = now.scores.get(key).and_then(Value::as_bool);
            let (Some(was), Some(is)) = (was, is) else {
                continue;
            };
            if was == is {
                continue;
            }
            let flip = Flip {
                id: record.id.clone(),
                score: key.clone(),
                before: Value::Bool(was),
                after: Value::Bool(is),
            };
            if is_good(key, is) {
                comparison.fixed.push(flip);
            } else {
                comparison.broken.push(flip);
            }
        }
    }
    let before_ids = before
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<BTreeSet<_>>();
    comparison.only_after = after_ids
        .iter()
        .filter(|id| !before_ids.contains(*id))
        .map(|id| (*id).to_owned())
        .collect();

    comparison.latency = METRICS
        .iter()
        .filter_map(|(metric, _)| {
            let was = before.latency.overall.get(*metric)?;
            let now = after.latency.overall.get(*metric)?;
            Some(LatencyDelta {
                metric: (*metric).to_owned(),
                p50_before: was.p50,
                p50_after: now.p50,
                p50_change_percent: change_percent(was.p50, now.p50),
                p95_before: was.p95,
                p95_after: now.p95,
                p95_change_percent: change_percent(was.p95, now.p95),
            })
        })
        .collect();
    comparison
}

impl Comparison {
    pub fn to_json(&self) -> String {
        serde_json::to_value(self)
            .and_then(|value| serde_json::to_string_pretty(&value))
            .unwrap_or_default()
            + "\n"
    }
}

fn signed(value: f64, unit: &str) -> String {
    if value > 0.0 {
        format!("+{value}{unit}")
    } else {
        format!("{value}{unit}")
    }
}

/// How a delta reads: better, worse, or no change.
fn verdict(metric: &str, delta: f64) -> &'static str {
    if delta == 0.0 {
        return "";
    }
    let better =
        if bad_when_true(metric) || lower_is_better(metric) || is_lower_better_count(metric) {
            delta < 0.0
        } else {
            delta > 0.0
        };
    if better { " better" } else { " worse" }
}

fn is_lower_better_count(metric: &str) -> bool {
    matches!(
        metric,
        "unsafe_ready"
            | "trap_dates"
            | "forbidden_parties"
            | "spurious_parties"
            | "forbidden_descriptions"
            | "unsupported_claims"
            | "review_rate"
            | "unsupported_fact_rate"
    )
}

pub fn render(comparison: &Comparison) -> String {
    let mut out = String::new();
    let describe = |side: &Side| {
        format!(
            "{} run {} at `{}` ({} documents)",
            side.mode,
            side.created_at,
            side.git_commit.as_deref().unwrap_or("unknown"),
            side.documents
        )
    };
    let _ = writeln!(out, "# InternBench comparison\n");
    let _ = writeln!(out, "- **Before:** {}", describe(&comparison.before));
    let _ = writeln!(out, "- **After:** {}\n", describe(&comparison.after));

    let _ = writeln!(out, "## Scores\n");
    let _ = writeln!(out, "| Score | Before | After | Change |");
    let _ = writeln!(out, "| --- | ---: | ---: | ---: |");
    let cell = |rate: &Option<Rate>| {
        rate.map_or_else(
            || "–".to_owned(),
            |rate| format!("{}/{} ({})", rate.correct, rate.total, percent(rate.rate)),
        )
    };
    let headline = HEADLINE_RATES.iter().map(|(key, _)| *key).chain([
        "unsafe_ready",
        "date_forbidden",
        "party_forbidden",
        "needless_review",
    ]);
    let mut listed = BTreeSet::new();
    let ordered = headline
        .filter_map(|key| comparison.rates.iter().find(|delta| delta.metric == key))
        .chain(comparison.rates.iter())
        .filter(|delta| delta.metric != "ready" && listed.insert(delta.metric.clone()))
        .collect::<Vec<_>>();
    for delta in ordered {
        let change = delta.delta_points.map_or_else(
            || "–".to_owned(),
            |points| {
                format!(
                    "{}{}",
                    signed(points, " pts"),
                    verdict(&delta.metric, points)
                )
            },
        );
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {change} |",
            delta.metric,
            cell(&delta.before),
            cell(&delta.after)
        );
    }
    let fraction = |value: Option<f64>| value.map_or_else(|| "–".to_owned(), percent);
    let mut listed = BTreeSet::new();
    let means = HEADLINE_MEANS
        .iter()
        .filter_map(|(key, _)| comparison.means.iter().find(|delta| delta.metric == *key))
        .chain(comparison.means.iter())
        .filter(|delta| listed.insert(delta.metric.clone()));
    for delta in means {
        let change = delta.delta.map_or_else(
            || "–".to_owned(),
            |value| {
                let points = round(value * 100.0, 1);
                format!(
                    "{}{}",
                    signed(points, " pts"),
                    verdict(&delta.metric, points)
                )
            },
        );
        let _ = writeln!(
            out,
            "| `{}` (mean) | {} | {} | {change} |",
            delta.metric,
            fraction(delta.before),
            fraction(delta.after)
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Safety counts\n");
    let _ = writeln!(out, "| Count | Before | After | Change |");
    let _ = writeln!(out, "| --- | ---: | ---: | ---: |");
    for delta in &comparison.counts {
        let show = |value: Option<f64>| {
            value.map_or_else(
                || "–".to_owned(),
                |value| {
                    if delta.metric.ends_with("_rate") {
                        percent(value)
                    } else {
                        format!("{value:.0}")
                    }
                },
            )
        };
        let change = delta.delta.map_or_else(
            || "–".to_owned(),
            |value| {
                let shown = if delta.metric.ends_with("_rate") {
                    signed(round(value * 100.0, 1), " pts")
                } else {
                    signed(value, "")
                };
                format!("{shown}{}", verdict(&delta.metric, value))
            },
        );
        let _ = writeln!(
            out,
            "| {} | {} | {} | {change} |",
            delta.metric,
            show(delta.before),
            show(delta.after)
        );
    }
    let _ = writeln!(out);

    let flips = |out: &mut String, title: &str, flips: &[Flip]| {
        let _ = writeln!(out, "## {title} ({})\n", flips.len());
        if flips.is_empty() {
            let _ = writeln!(out, "None.\n");
            return;
        }
        let mut by_document: Vec<(&str, Vec<&str>)> = Vec::new();
        for flip in flips {
            match by_document.iter_mut().find(|(id, _)| *id == flip.id) {
                Some((_, scores)) => scores.push(&flip.score),
                None => by_document.push((&flip.id, vec![&flip.score])),
            }
        }
        for (id, scores) in by_document {
            let _ = writeln!(out, "- **{id}**: {}", scores.join(", "));
        }
        let _ = writeln!(out);
    };
    flips(&mut out, "Broken", &comparison.broken);
    flips(&mut out, "Fixed", &comparison.fixed);

    if !comparison.status_changes.is_empty() {
        let _ = writeln!(out, "## Status changes\n");
        for change in &comparison.status_changes {
            let _ = writeln!(
                out,
                "- **{}**: {} → {}",
                change.id, change.before, change.after
            );
        }
        let _ = writeln!(out);
    }
    if !comparison.filename_changes.is_empty() {
        let _ = writeln!(
            out,
            "## Filenames that changed ({})\n",
            comparison.filename_changes.len()
        );
        for change in &comparison.filename_changes {
            let _ = writeln!(
                out,
                "- **{}**: `{}` → `{}`",
                change.id, change.before, change.after
            );
        }
        let _ = writeln!(out);
    }
    for (title, ids) in [
        ("Only in before", &comparison.only_before),
        ("Only in after", &comparison.only_after),
    ] {
        if !ids.is_empty() {
            let _ = writeln!(out, "{title}: {}\n", ids.join(", "));
        }
    }

    if !comparison.latency.is_empty() {
        let _ = writeln!(out, "## Latency\n");
        let _ = writeln!(
            out,
            "| Stage | p50 before | p50 after | Change | p95 before | p95 after | Change |"
        );
        let _ = writeln!(out, "| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
        for delta in &comparison.latency {
            let pct = |value: Option<f64>| {
                value.map_or_else(|| "–".to_owned(), |value| signed(value, "%"))
            };
            let show = |value: f64| {
                if delta.metric.ends_with("_ms") {
                    duration(value)
                } else {
                    metric_value(&delta.metric, value)
                }
            };
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {} | {} | {} |",
                delta.metric,
                show(delta.p50_before),
                show(delta.p50_after),
                pct(delta.p50_change_percent),
                show(delta.p95_before),
                show(delta.p95_after),
                pct(delta.p95_change_percent)
            );
        }
        let _ = writeln!(out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        record::DocumentRecord,
        report::{RunInfo, build},
        timing,
    };
    use serde_json::json;

    fn report(records: Vec<(&str, Value, &str, f64)>) -> Report {
        build(
            RunInfo {
                mode: "replay".into(),
                ..RunInfo::default()
            },
            records
                .into_iter()
                .map(|(id, scores, filename, total)| {
                    let mut record = DocumentRecord {
                        id: id.into(),
                        status: "completed".into(),
                        filename: Some(filename.into()),
                        scores: serde_json::from_value(scores).unwrap(),
                        timings: timing::empty(),
                        ..DocumentRecord::default()
                    };
                    record.timings.insert("total_ms".into(), json!(total));
                    record
                })
                .collect(),
        )
    }

    #[test]
    fn documents_are_aligned_by_id_and_flips_named() {
        let before = report(vec![
            (
                "a",
                json!({"filename_correct": true, "date_forbidden": false}),
                "a.pdf",
                100.0,
            ),
            (
                "b",
                json!({"filename_correct": false, "date_forbidden": true}),
                "b.pdf",
                200.0,
            ),
            ("gone", json!({"filename_correct": true}), "g.pdf", 100.0),
        ]);
        let after = report(vec![
            (
                "b",
                json!({"filename_correct": true, "date_forbidden": false}),
                "b2.pdf",
                300.0,
            ),
            (
                "a",
                json!({"filename_correct": false, "date_forbidden": false}),
                "a.pdf",
                100.0,
            ),
            ("added", json!({"filename_correct": true}), "n.pdf", 100.0),
        ]);
        let comparison = compare(&before, &after);
        let names = |flips: &[Flip]| {
            flips
                .iter()
                .map(|flip| format!("{}:{}", flip.id, flip.score))
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&comparison.broken), vec!["a:filename_correct"]);
        assert_eq!(
            names(&comparison.fixed),
            vec!["b:date_forbidden", "b:filename_correct"]
        );
        assert_eq!(comparison.only_before, vec!["gone"]);
        assert_eq!(comparison.only_after, vec!["added"]);
        assert_eq!(comparison.filename_changes[0].after, "b2.pdf");
        let filename = comparison
            .rates
            .iter()
            .find(|delta| delta.metric == "filename_correct")
            .unwrap();
        assert_eq!(
            filename.delta_points,
            Some(0.0),
            "2 of 3 before, 2 of 3 after"
        );
        let trap = comparison
            .counts
            .iter()
            .find(|delta| delta.metric == "trap_dates")
            .unwrap();
        assert_eq!(
            (trap.before, trap.after, trap.delta),
            (Some(1.0), Some(0.0), Some(-1.0))
        );
        let total = &comparison.latency[0];
        assert_eq!(total.metric, "total_ms");
        assert_eq!((total.p50_before, total.p50_after), (100.0, 100.0));
        assert_eq!(total.p95_change_percent, Some(50.0));

        let rendered = render(&comparison);
        assert!(
            rendered.contains("## Broken (1)\n\n- **a**: filename_correct"),
            "{rendered}"
        );
        assert!(
            rendered.contains("| trap_dates | 1 | 0 | -1 better |"),
            "{rendered}"
        );
        assert!(
            rendered
                .contains("| `total_ms` | 100.0 ms | 100.0 ms | 0% | 200.0 ms | 300.0 ms | +50% |"),
            "{rendered}"
        );
    }
}
