//! Two reports side by side: what a change did.
//!
//! Documents are aligned by id, and every aggregate is computed again over
//! the documents both reports scored (completed, or failed and scored as a
//! miss), never taken from either report's own summary: a run over a
//! subset, a document added since, or one that went stale in replay would
//! otherwise move the rates with no change to the code. Each score is
//! compared over the documents that have it in both runs. Rates are
//! compared in percentage points, fractions as differences, counts as
//! differences. A document whose boolean score went from good to bad is
//! *broken*, from bad to good *fixed* - trap scores being good when false,
//! as in the baseline gate.
//!
//! A value the extract-only gate holds ([`extract_values`]) - a structure
//! score, an OCR accuracy, digest recall, or OCR's character or word edit
//! distance - that moved further than the gate tolerates is listed per
//! document as *worse* or *better*.
//!
//! Latency is compared, as each stage's p50 and p95 change in per cent, over
//! the documents both runs completed, and only between two runs that
//! measured their timings. A replay reports its recording's timings, taken
//! on another run's machine; a difference involving one is not a measured
//! change, so it is not shown as one.

use std::{collections::BTreeSet, fmt::Write as _};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    baseline::{extract_values, value_moved},
    markdown::{HEADLINE_MEANS, HEADLINE_RATES, duration, metric_value, percent},
    record::{COMPLETED, DocumentRecord, PENDING, is_unscorable},
    report::{self, Rate, Report, Summary},
    score::{OCR_EDIT_COUNTS, bad_when_true, is_unit_fraction, lower_is_better},
    stats::{Distribution, round},
    timing::{self, METRICS},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Side {
    pub mode: String,
    pub created_at: String,
    pub git_commit: Option<String>,
    pub documents: usize,
    /// `measured`, or `recorded` for a replay.
    #[serde(default)]
    pub timings_source: String,
    /// The machine the timings were taken on.
    #[serde(default)]
    pub machine: String,
    /// A replay's recording.
    #[serde(default)]
    pub recording_sha256: Option<String>,
    #[serde(default)]
    pub recorded_at: Option<String>,
    #[serde(default)]
    pub only: Vec<String>,
    #[serde(default)]
    pub gold_sha256: String,
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
    /// The documents, completed in both runs with the metric measured, both
    /// percentiles are over.
    #[serde(default)]
    pub documents: usize,
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
    /// The documents both runs scored, which every rate, mean and count is
    /// computed over.
    #[serde(default)]
    pub aligned: usize,
    /// The documents both runs completed, which latency is computed over.
    #[serde(default)]
    pub completed_in_both: usize,
    /// How the two runs differ in what they cover, for the reader.
    #[serde(default)]
    pub notes: Vec<String>,
    /// Why latency is not compared, or what to bear in mind when it is.
    #[serde(default)]
    pub latency_note: Option<String>,
    pub rates: Vec<RateDelta>,
    pub means: Vec<ValueDelta>,
    pub counts: Vec<ValueDelta>,
    pub broken: Vec<Flip>,
    pub fixed: Vec<Flip>,
    /// Fractional scores that moved the wrong way by more than the
    /// extract-only gate's tolerance, and the right way.
    #[serde(default)]
    pub worse: Vec<Flip>,
    #[serde(default)]
    pub better: Vec<Flip>,
    pub status_changes: Vec<Change>,
    pub filename_changes: Vec<Change>,
    pub only_before: Vec<String>,
    pub only_after: Vec<String>,
    /// Empty when either run's timings were not measured by it.
    pub latency: Vec<LatencyDelta>,
}

fn side(report: &Report) -> Side {
    Side {
        mode: report.mode.clone(),
        created_at: report.created_at.clone(),
        git_commit: report.git_commit.clone(),
        documents: report.records.len(),
        timings_source: report.timings_source.clone(),
        machine: report.machine.summary(),
        recording_sha256: report
            .recording
            .as_ref()
            .map(|recording| recording.sha256.clone()),
        recorded_at: report
            .recording
            .as_ref()
            .map(|recording| recording.recorded_at.clone()),
        only: report.corpus.only.clone(),
        gold_sha256: report.corpus.gold_sha256.clone(),
    }
}

fn change_percent(before: f64, after: f64) -> Option<f64> {
    (before > 0.0).then(|| round((after - before) / before * 100.0, 1))
}

fn counts(summary: &Summary) -> Vec<(&'static str, f64)> {
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

/// Scored: completed, or failed and scored as a miss.
fn scored(record: &DocumentRecord) -> bool {
    record.status != PENDING && !is_unscorable(&record.status)
}

/// One aligned document on each side, holding only the scores both sides
/// have, so every score is compared over the same documents.
fn aligned_pair(
    before: &DocumentRecord,
    after: &DocumentRecord,
) -> (DocumentRecord, DocumentRecord) {
    let shared = before
        .scores
        .keys()
        .filter(|key| after.scores.contains_key(*key))
        .cloned()
        .collect::<BTreeSet<_>>();
    let keep = |record: &DocumentRecord| {
        let mut kept = DocumentRecord {
            id: record.id.clone(),
            status: record.status.clone(),
            readiness: record.readiness.clone(),
            scores: record
                .scores
                .iter()
                .filter(|(key, _)| shared.contains(*key))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            forbidden_description: record.forbidden_description.clone(),
            ..DocumentRecord::default()
        };
        // A forbidden fact asserted counts only where both descriptions
        // were checked.
        if !shared.contains("description_factual") {
            kept.forbidden_description.clear();
        }
        kept
    };
    (keep(before), keep(after))
}

pub fn compare(before: &Report, after: &Report) -> Comparison {
    let mut comparison = Comparison {
        before: side(before),
        after: side(after),
        ..Comparison::default()
    };

    let pairs = before
        .records
        .iter()
        .filter_map(|record| Some((record, after.record(&record.id)?)))
        .collect::<Vec<_>>();
    let aligned = pairs
        .iter()
        .filter(|(was, now)| scored(was) && scored(now))
        .map(|(was, now)| aligned_pair(was, now))
        .collect::<Vec<_>>();
    comparison.aligned = aligned.len();
    let mut was_summary = report::summarize(aligned.iter().map(|(was, _)| was));
    let mut now_summary = report::summarize(aligned.iter().map(|(_, now)| now));
    // The share sent to review, over the documents both runs completed.
    let completed = aligned
        .iter()
        .filter(|(was, now)| was.status == COMPLETED && now.status == COMPLETED)
        .filter(|(was, now)| was.readiness.is_some() && now.readiness.is_some())
        .collect::<Vec<_>>();
    let review_rate = |pick: fn(&(DocumentRecord, DocumentRecord)) -> &DocumentRecord| {
        (!completed.is_empty()).then(|| {
            let reviewed = completed
                .iter()
                .filter(|pair| pick(pair).readiness.as_deref() == Some("needs_review"))
                .count();
            round(reviewed as f64 / completed.len() as f64, 4)
        })
    };
    was_summary.review_rate = review_rate(|(was, _)| was);
    now_summary.review_rate = review_rate(|(_, now)| now);

    let rate_keys = was_summary
        .rates
        .keys()
        .chain(now_summary.rates.keys())
        .collect::<BTreeSet<_>>();
    comparison.rates = rate_keys
        .into_iter()
        .map(|key| {
            let was = was_summary.rates.get(key).copied();
            let now = now_summary.rates.get(key).copied();
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
    let mean_keys = was_summary
        .means
        .keys()
        .chain(now_summary.means.keys())
        .collect::<BTreeSet<_>>();
    comparison.means = mean_keys
        .into_iter()
        .map(|key| {
            let was = was_summary.means.get(key).map(|mean| mean.mean);
            let now = now_summary.means.get(key).map(|mean| mean.mean);
            ValueDelta {
                metric: key.clone(),
                delta: was.zip(now).map(|(was, now)| round(now - was, 4)),
                before: was,
                after: now,
            }
        })
        .collect();
    // Every count either side has: an optional one - a rate with nothing
    // to be a rate of yet - can appear only after a change, or only before.
    let (was_counts, now_counts) = (counts(&was_summary), counts(&now_summary));
    let value = |counts: &[(&str, f64)], metric: &str| {
        counts
            .iter()
            .find(|(other, _)| *other == metric)
            .map(|(_, value)| *value)
    };
    let mut count_names = was_counts
        .iter()
        .map(|(metric, _)| *metric)
        .collect::<Vec<_>>();
    count_names.extend(
        now_counts
            .iter()
            .map(|(metric, _)| *metric)
            .filter(|metric| !was_counts.iter().any(|(other, _)| other == metric)),
    );
    comparison.counts = count_names
        .into_iter()
        .map(|metric| {
            let was = value(&was_counts, metric);
            let now = value(&now_counts, metric);
            ValueDelta {
                metric: metric.to_owned(),
                before: was,
                delta: was.zip(now).map(|(was, now)| round(now - was, 4)),
                after: now,
            }
        })
        .collect();

    for (record, now) in &pairs {
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
        if scored(record) && scored(now) {
            let after_values = extract_values(now);
            for (key, was) in extract_values(record) {
                let Some(is) = after_values.get(&key).copied() else {
                    continue;
                };
                let Some(better) = value_moved(&key, was, is) else {
                    continue;
                };
                let flip = Flip {
                    id: record.id.clone(),
                    score: key,
                    before: json!(was),
                    after: json!(is),
                };
                if better {
                    comparison.better.push(flip);
                } else {
                    comparison.worse.push(flip);
                }
            }
        }
    }
    let before_ids = before
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<BTreeSet<_>>();
    let after_ids = after
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<BTreeSet<_>>();
    comparison.only_before = before_ids
        .difference(&after_ids)
        .map(|id| (*id).to_owned())
        .collect();
    comparison.only_after = after_ids
        .difference(&before_ids)
        .map(|id| (*id).to_owned())
        .collect();

    if comparison.before.only != comparison.after.only {
        comparison.notes.push(format!(
            "The runs cover different documents ({} and {}); only the {} both scored are compared.",
            describe_only(&comparison.before.only),
            describe_only(&comparison.after.only),
            comparison.aligned
        ));
    }
    if !comparison.before.gold_sha256.is_empty()
        && !comparison.after.gold_sha256.is_empty()
        && comparison.before.gold_sha256 != comparison.after.gold_sha256
    {
        comparison.notes.push(
            "The gold differs between the runs: a score can move because the reviewed answer changed, not the code."
                .to_owned(),
        );
    }
    if comparison.before.mode != comparison.after.mode
        && (comparison.before.mode == "extract" || comparison.after.mode == "extract")
    {
        comparison.notes.push(format!(
            "A {} run is compared with a {} run: only the scores both have - extraction's - are comparable.",
            comparison.before.mode, comparison.after.mode
        ));
    }
    let unscored = pairs.len() - comparison.aligned;
    if unscored > 0 {
        comparison.notes.push(format!(
            "{unscored} document(s) in both runs were not scored in one of them (pending, stale or unrecorded) and are left out of every figure."
        ));
    }

    comparison.completed_in_both = pairs
        .iter()
        .filter(|(was, now)| was.status == COMPLETED && now.status == COMPLETED)
        .count();
    let recorded = |side: &Side| side.timings_source == "recorded";
    if recorded(&comparison.before) || recorded(&comparison.after) {
        comparison.latency_note = Some(
            if recorded(&comparison.before)
                && recorded(&comparison.after)
                && comparison.before.recording_sha256 == comparison.after.recording_sha256
            {
                "Both runs replay the same recording: their timings are the recording's, identical by construction, and say nothing about this change's latency. Latency is not compared.".to_owned()
            } else {
                "A replayed run reports its recording's timings, taken when and where the recording was made, not measured by that run; a difference involving one is not a measured change. Latency is not compared: compare two live runs made on the same machine.".to_owned()
            },
        );
    } else {
        comparison.latency = latency(&pairs);
        if comparison.before.machine != comparison.after.machine {
            comparison.latency_note = Some(format!(
                "The runs were made on different machines ({} and {}): the changes compare the machines as much as the code.",
                comparison.before.machine, comparison.after.machine
            ));
        }
    }
    comparison
}

fn describe_only(only: &[String]) -> String {
    if only.is_empty() {
        "the whole corpus".to_owned()
    } else {
        format!("only {}", only.join(", "))
    }
}

/// p50 and p95 of every metric over the documents both runs completed and
/// measured it on.
fn latency(pairs: &[(&DocumentRecord, &DocumentRecord)]) -> Vec<LatencyDelta> {
    METRICS
        .iter()
        .filter_map(|(metric, _)| {
            let (was, now): (Vec<f64>, Vec<f64>) = pairs
                .iter()
                .filter(|(was, now)| report::timed(was) && report::timed(now))
                .filter_map(|(was, now)| {
                    Some((
                        timing::get(&was.timings, metric)?,
                        timing::get(&now.timings, metric)?,
                    ))
                })
                .unzip();
            let (was_distribution, now_distribution) =
                (Distribution::of(&was)?, Distribution::of(&now)?);
            Some(LatencyDelta {
                metric: (*metric).to_owned(),
                documents: was.len(),
                p50_before: was_distribution.p50,
                p50_after: now_distribution.p50,
                p50_change_percent: change_percent(was_distribution.p50, now_distribution.p50),
                p95_before: was_distribution.p95,
                p95_after: now_distribution.p95,
                p95_change_percent: change_percent(was_distribution.p95, now_distribution.p95),
            })
        })
        .collect()
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

/// How a delta reads: better, worse, or no change. Some counts have no
/// direction: more description claims is neither better nor worse (it is
/// the denominator of the unsupported-fact rate), and fewer documents sent
/// to review is better only if the names were right, which the rates say.
fn verdict(metric: &str, delta: f64) -> &'static str {
    if delta == 0.0 || is_neutral(metric) {
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
            | "unsupported_fact_rate"
    )
}

fn is_neutral(metric: &str) -> bool {
    matches!(metric, "claims" | "review_rate")
}

pub fn render(comparison: &Comparison) -> String {
    let mut out = String::new();
    let describe = |side: &Side| {
        let mut line = format!(
            "{} run {} at `{}` ({} documents) on {}",
            side.mode,
            side.created_at,
            side.git_commit.as_deref().unwrap_or("unknown"),
            side.documents,
            if side.machine.is_empty() {
                "an unknown machine"
            } else {
                &side.machine
            }
        );
        if side.timings_source == "recorded" {
            let _ = write!(
                line,
                " · timings recorded{}{}, not measured",
                side.recording_sha256
                    .as_deref()
                    .map(|sha| format!(" (recording `{}`", &sha[..sha.len().min(12)]))
                    .unwrap_or_default(),
                match (&side.recording_sha256, &side.recorded_at) {
                    (Some(_), Some(at)) if !at.is_empty() => format!(", made {at})"),
                    (Some(_), _) => ")".to_owned(),
                    _ => String::new(),
                }
            );
        }
        line
    };
    let _ = writeln!(out, "# InternBench comparison\n");
    let _ = writeln!(out, "- **Before:** {}", describe(&comparison.before));
    let _ = writeln!(out, "- **After:** {}", describe(&comparison.after));
    let _ = writeln!(
        out,
        "- **Compared:** every score, rate and count over the {} documents both runs scored (completed, or failed and scored as a miss), each score over the documents that have it in both runs; latency over the {} both completed.\n",
        comparison.aligned, comparison.completed_in_both
    );
    for note in &comparison.notes {
        let _ = writeln!(out, "> {note}\n");
    }

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
    let mut listed = BTreeSet::new();
    let means = HEADLINE_MEANS
        .iter()
        .filter_map(|(key, _)| comparison.means.iter().find(|delta| delta.metric == *key))
        .chain(comparison.means.iter())
        .filter(|delta| listed.insert(delta.metric.clone()));
    for delta in means {
        // A share of one reads as a percentage and moves in points; a
        // figure on its own scale (OCR confidence, 0-100) as itself.
        let unit = is_unit_fraction(&delta.metric);
        let shown = |value: Option<f64>| {
            value.map_or_else(
                || "–".to_owned(),
                |value| {
                    if unit {
                        percent(value)
                    } else {
                        format!("{value:.1}")
                    }
                },
            )
        };
        let change = delta.delta.map_or_else(
            || "–".to_owned(),
            |value| {
                let (moved, suffix) = if unit {
                    (round(value * 100.0, 1), " pts")
                } else {
                    (round(value, 1), "")
                };
                format!("{}{}", signed(moved, suffix), verdict(&delta.metric, moved))
            },
        );
        let _ = writeln!(
            out,
            "| `{}` (mean) | {} | {} | {change} |",
            delta.metric,
            shown(delta.before),
            shown(delta.after)
        );
    }
    let _ = writeln!(out);

    // Two extract-only runs name nothing: there are no safety counts and
    // no flips of a name's scores, only the extraction scores above and
    // their movements below.
    let extraction = comparison.before.mode == "extract" && comparison.after.mode == "extract";
    if !extraction {
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
            let mut by_document: Vec<(&str, Vec<&str>)> = Vec::new();
            for flip in flips {
                match by_document.iter_mut().find(|(id, _)| *id == flip.id) {
                    Some((_, scores)) => scores.push(&flip.score),
                    None => by_document.push((&flip.id, vec![&flip.score])),
                }
            }
            let _ = writeln!(
                out,
                "## {title}: {} score(s) in {} document(s)\n",
                flips.len(),
                by_document.len()
            );
            if flips.is_empty() {
                let _ = writeln!(out, "None.\n");
                return;
            }
            for (id, scores) in by_document {
                let _ = writeln!(out, "- **{id}**: {}", scores.join(", "));
            }
            let _ = writeln!(out);
        };
        flips(&mut out, "Broken", &comparison.broken);
        flips(&mut out, "Fixed", &comparison.fixed);
    }
    let moved = |out: &mut String, title: &str, flips: &[Flip]| {
        if flips.is_empty() {
            return;
        }
        let _ = writeln!(out, "## {title}: {} score(s)\n", flips.len());
        let _ = writeln!(out, "| Document | Score | Before | After |");
        let _ = writeln!(out, "| --- | --- | ---: | ---: |");
        for flip in flips {
            let show = |value: &Value| {
                value.as_f64().map_or_else(
                    || "–".to_owned(),
                    |value| {
                        if is_unit_fraction(&flip.score) {
                            percent(value)
                        } else if OCR_EDIT_COUNTS.contains(&flip.score.as_str()) {
                            format!("{value:.0}")
                        } else {
                            format!("{value:.1}")
                        }
                    },
                )
            };
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} |",
                flip.id,
                flip.score,
                show(&flip.before),
                show(&flip.after)
            );
        }
        let _ = writeln!(out);
    };
    moved(
        &mut out,
        "Worse past the extract-only tolerance",
        &comparison.worse,
    );
    moved(
        &mut out,
        "Better past the extract-only tolerance",
        &comparison.better,
    );

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

    if comparison.latency.is_empty() {
        if let Some(note) = &comparison.latency_note {
            let _ = writeln!(out, "## Latency\n\n{note}\n");
        }
    } else {
        let _ = writeln!(out, "## Latency\n");
        let _ = writeln!(
            out,
            "Measured by both runs, over the documents both completed.\n"
        );
        if let Some(note) = &comparison.latency_note {
            let _ = writeln!(out, "> {note}\n");
        }
        let _ = writeln!(
            out,
            "| Stage | Docs | p50 before | p50 after | Change | p95 before | p95 after | Change |"
        );
        let _ = writeln!(
            out,
            "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
        );
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
                "| `{}` | {} | {} | {} | {} | {} | {} | {} |",
                delta.metric,
                delta.documents,
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
        ocr::OcrMeasure,
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
            rendered
                .contains("## Broken: 1 score(s) in 1 document(s)\n\n- **a**: filename_correct"),
            "{rendered}"
        );
        assert!(
            rendered.contains("| trap_dates | 1 | 0 | -1 better |"),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "| `total_ms` | 2 | 100.0 ms | 100.0 ms | 0% | 200.0 ms | 300.0 ms | +50% |"
            ),
            "{rendered}"
        );
        assert!(
            rendered.contains("over the 2 documents both runs scored"),
            "{rendered}"
        );
    }

    fn rate(comparison: &Comparison, metric: &str) -> RateDelta {
        comparison
            .rates
            .iter()
            .find(|delta| delta.metric == metric)
            .cloned()
            .unwrap()
    }

    /// A subset run, or a document only one run has, does not move a rate.
    #[test]
    fn rates_are_computed_over_the_documents_both_runs_scored() {
        let before = report(vec![
            ("a", json!({"filename_correct": true}), "a.pdf", 100.0),
            ("b", json!({"filename_correct": true}), "b.pdf", 100.0),
            (
                "wrong",
                json!({"filename_correct": false, "unsafe_ready": true}),
                "w.pdf",
                100.0,
            ),
        ]);
        let mut after = report(vec![
            ("a", json!({"filename_correct": true}), "a.pdf", 100.0),
            ("b", json!({"filename_correct": true}), "b.pdf", 100.0),
        ]);
        after.corpus.only = vec!["a".into(), "b".into()];
        let comparison = compare(&before, &after);
        assert_eq!(comparison.aligned, 2);
        let filename = rate(&comparison, "filename_correct");
        assert_eq!(filename.delta_points, Some(0.0));
        assert_eq!(filename.before.unwrap().total, 2);
        assert!(
            !comparison
                .rates
                .iter()
                .any(|delta| delta.metric == "unsafe_ready"),
            "the document only one run scored is in no figure"
        );
        let unsafe_ready = comparison
            .counts
            .iter()
            .find(|delta| delta.metric == "unsafe_ready")
            .unwrap();
        assert_eq!(unsafe_ready.delta, Some(0.0));
        assert!(comparison.notes[0].starts_with("The runs cover different documents"));

        // A document stale in one run is left out of both.
        let mut stale = report(vec![
            ("a", json!({"filename_correct": true}), "a.pdf", 100.0),
            ("b", json!({}), "b.pdf", 100.0),
        ]);
        stale.records[1].status = "stale_prompt".into();
        let before = report(vec![
            ("a", json!({"filename_correct": true}), "a.pdf", 100.0),
            ("b", json!({"filename_correct": false}), "b.pdf", 100.0),
        ]);
        let comparison = compare(&before, &stale);
        assert_eq!(comparison.aligned, 1);
        assert_eq!(
            rate(&comparison, "filename_correct").delta_points,
            Some(0.0)
        );
        assert!(
            comparison
                .notes
                .iter()
                .any(|note| note.starts_with("1 document(s) in both runs were not scored")),
            "{:?}",
            comparison.notes
        );
    }

    #[test]
    fn recorded_timings_are_never_shown_as_a_measured_change() {
        let recorded = |sha: &str| {
            let mut report = report(vec![(
                "a",
                json!({"filename_correct": true}),
                "a.pdf",
                100.0,
            )]);
            report.timings_source = "recorded".into();
            report.recording = Some(crate::report::RecordingInfo {
                sha256: sha.into(),
                recorded_at: "2026-10-01T09:00:00Z".into(),
                ..Default::default()
            });
            report
        };
        let same = compare(&recorded("abc"), &recorded("abc"));
        assert!(same.latency.is_empty());
        assert!(
            same.latency_note
                .as_deref()
                .unwrap()
                .starts_with("Both runs replay the same recording")
        );
        let rendered = render(&same);
        assert!(
            rendered.contains("## Latency\n\nBoth runs replay"),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "timings recorded (recording `abc`, made 2026-10-01T09:00:00Z), not measured"
            ),
            "{rendered}"
        );
        assert!(!rendered.contains("| Stage |"));

        let mut live = report(vec![(
            "a",
            json!({"filename_correct": true}),
            "a.pdf",
            90.0,
        )]);
        live.timings_source = "measured".into();
        let mixed = compare(&recorded("abc"), &live);
        assert!(mixed.latency.is_empty());
        assert!(
            mixed
                .latency_note
                .as_deref()
                .unwrap()
                .contains("not measured by that run")
        );
    }

    /// A count only one side has - a rate of claims when the run before made
    /// none - is still compared, with a dash for the side that lacks it.
    #[test]
    fn a_count_only_the_after_run_has_is_shown() {
        let with = |claims: u64| {
            let mut report = report(vec![(
                "a",
                json!({"filename_correct": true, "description_claims": claims, "description_unsupported": 0}),
                "a.pdf",
                100.0,
            )]);
            report.summary = report::summarize(&report.records);
            report
        };
        assert!(with(0).summary.unsupported_fact_rate.is_none());
        let comparison = compare(&with(0), &with(4));
        let rate = comparison
            .counts
            .iter()
            .find(|delta| delta.metric == "unsupported_fact_rate")
            .expect("the after run's rate is compared");
        assert_eq!(
            (rate.before, rate.after, rate.delta),
            (None, Some(0.0), None)
        );
        let rendered = render(&comparison);
        assert!(
            rendered.contains("| unsupported_fact_rate | – | 0.0% | – |"),
            "{rendered}"
        );
        // And the other way round.
        let comparison = compare(&with(4), &with(0));
        assert!(
            comparison
                .counts
                .iter()
                .any(|delta| delta.metric == "unsupported_fact_rate"
                    && delta.before == Some(0.0)
                    && delta.after.is_none())
        );
    }

    #[test]
    fn extraction_scores_that_moved_past_tolerance_are_named() {
        let run = |chars: usize, rows: f64, completeness: f64| {
            let mut report = report(vec![(
                "a",
                json!({"ocr_cer": chars as f64 / 600.0, "table_row_accuracy": rows, "description_completeness": completeness, "ocr_mean_confidence": 80.0}),
                "a.pdf",
                100.0,
            )]);
            report.mode = "extract".into();
            report.records[0].ocr = Some(OcrMeasure {
                char_distance: chars,
                char_distance_ci: chars,
                truth_chars: 600,
                word_distance: 10,
                truth_words: 100,
                ..OcrMeasure::default()
            });
            report
        };
        let comparison = compare(&run(30, 0.5, 0.5), &run(18, 0.25, 1.0));
        let named = |flips: &[Flip]| {
            flips
                .iter()
                .map(|flip| format!("{}:{}", flip.id, flip.score))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            named(&comparison.better),
            vec!["a:ocr_char_distance", "a:ocr_char_distance_ci"],
            "OCR is compared by its edit distances, not its rates"
        );
        assert_eq!(
            named(&comparison.worse),
            vec!["a:table_row_accuracy"],
            "a score extraction does not decide is not listed"
        );
        let rendered = render(&comparison);
        assert!(
            rendered.contains("| a | `table_row_accuracy` | 50.0% | 25.0% |"),
            "{rendered}"
        );
        assert!(
            rendered.contains("| a | `ocr_char_distance` | 30 | 18 |"),
            "{rendered}"
        );
        assert!(
            !rendered.contains("## Safety counts") && !rendered.contains("review_rate"),
            "two extract-only runs name nothing: {rendered}"
        );
        // Within the edit distance's tolerance: not listed.
        assert!(
            compare(&run(30, 0.5, 0.5), &run(33, 0.5, 0.5))
                .worse
                .is_empty()
        );
    }

    #[test]
    fn counts_without_a_direction_get_no_verdict_and_confidence_keeps_its_scale() {
        let with = |claims: u64, confidence: f64, ready: &str| {
            let mut report = report(vec![(
                "a",
                json!({"filename_correct": true, "description_claims": claims, "description_unsupported": 0, "ocr_mean_confidence": confidence}),
                "a.pdf",
                100.0,
            )]);
            report.records[0].readiness = Some(ready.into());
            report.summary = report::summarize(&report.records);
            report
        };
        let rendered = render(&compare(
            &with(4, 85.0, "needs_review"),
            &with(9, 87.0, "ready"),
        ));
        assert!(rendered.contains("| claims | 4 | 9 | +5 |"), "{rendered}");
        assert!(
            rendered.contains("| review_rate | 100.0% | 0.0% | -100 pts |"),
            "{rendered}"
        );
        assert!(
            rendered.contains("| `ocr_mean_confidence` (mean) | 85.0 | 87.0 | +2 better |"),
            "{rendered}"
        );
    }
}
