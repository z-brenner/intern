//! The report a person reads: what the run scored, what it got wrong, and
//! where the time went - concise, and stable enough to diff.

use std::fmt::Write as _;

use crate::{
    extract::EXTRACTION_SCORES,
    gold::{PAGE_BUCKETS, ROUTES},
    record::{COMPLETED, PENDING},
    report::{EXTRACT, OcrFigures, Rate, Report, StructureFigures, Summary},
    score::is_unit_fraction,
    stats::Distribution,
    timing::{METRICS, Unit},
};

/// The misses listed before the rest are left to the JSON.
const MAX_MISSES: usize = 40;

/// The headline boolean scores, in the order a reader asks about them. Three
/// are judged only when the answer gives them something to judge; a failed
/// document is a miss on all of them.
pub const HEADLINE_RATES: &[(&str, &str)] = &[
    ("filename_correct", "Filename (the whole name)"),
    ("type_correct", "Document type"),
    ("date_correct", "Date"),
    (
        "date_role_correct",
        "Date role (when the reviewed date was chosen)",
    ),
    ("parties_correct", "Parties"),
    (
        "relation_correct",
        "Relation word (when parties were named)",
    ),
    (
        "party_role_correct",
        "Party in the right role (when parties were named)",
    ),
    ("readiness_match", "Ready / review routing"),
    ("description_complete", "Description has every fact"),
    ("description_factual", "Description states nothing false"),
    ("description_specific", "Description is specific"),
];

/// The headline fractional scores.
pub const HEADLINE_MEANS: &[(&str, &str)] = &[
    ("description_completeness", "Description fact coverage"),
    ("description_specificity", "Description specificity"),
    ("evidence_recall", "Evidence recall (model's quotes)"),
    ("digest_recall", "Digest recall (facts reaching the digest)"),
    (
        "prompt_recall",
        "Prompt recall (facts reaching the prompt sent)",
    ),
];

pub fn render(report: &Report) -> String {
    if report.mode == EXTRACT {
        return render_extraction(report);
    }
    let mut out = String::new();
    header(&mut out, report);
    scorecard(&mut out, &report.summary);
    safety(&mut out, &report.summary);
    groups(&mut out, report);
    ocr(&mut out, report);
    structure(&mut out, report);
    routes(&mut out, report);
    stages(&mut out, report);
    if report.timings_source != "recorded" {
        extraction_time(&mut out, report);
    }
    memory(&mut out, report);
    misses(&mut out, report);
    baseline(&mut out, report);
    let _ = writeln!(out, "## How to compare");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Keep this run's JSON, make the change, run again, then:\n\n```text\nintern-bench compare --before before.json --after after.json --markdown diff.md\n```\n\n\
         It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, \
         and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. \
         `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON."
    );
    out
}

fn header(out: &mut String, report: &Report) {
    let _ = writeln!(out, "# InternBench: {} run", report.mode);
    let _ = writeln!(out);
    let commit = report
        .git_commit
        .as_deref()
        .map(short)
        .unwrap_or_else(|| "unknown".to_owned());
    let _ = writeln!(out, "- **Run:** {} · commit `{commit}`", report.created_at);
    let _ = writeln!(out, "- **Machine:** {}", report.machine.summary());
    if report.mode == EXTRACT {
        if let Some(worker) = &report.worker {
            let _ = writeln!(out, "- **Worker:** `{worker}`");
        }
    } else {
        let mut model = format!("`{}`", report.model.id);
        if let (Some(size), Some(sha)) = (report.model.size_bytes, &report.model.sha256) {
            let _ = write!(
                model,
                " · {:.2} GB · sha256 `{}`",
                size as f64 / 1e9,
                &sha[..sha.len().min(12)]
            );
        }
        let _ = writeln!(out, "- **Model:** {model}");
    }
    let summary = &report.summary;
    let mut corpus = format!(
        "{} documents · {} completed",
        summary.documents, summary.completed
    );
    for (status, count) in &summary.statuses {
        if status != COMPLETED {
            let _ = write!(corpus, " · {count} {status}");
        }
    }
    let stale = report.records.iter().filter(|record| record.stale).count();
    if stale > 0 {
        let _ = write!(
            corpus,
            " · {stale} scored from stale replies (--allow-stale)"
        );
    }
    if !report.corpus.gold_sha256.is_empty() {
        let _ = write!(corpus, " · gold `{}`", short(&report.corpus.gold_sha256));
    }
    if !report.corpus.only.is_empty() {
        let _ = write!(corpus, " · only {}", report.corpus.only.join(", "));
    }
    let _ = writeln!(out, "- **Corpus:** {corpus}");
    if let Some(wall) = report.wall_ms {
        let _ = writeln!(
            out,
            "- **Wall time:** {} for the whole run, the worker's start included",
            duration(wall)
        );
    }
    if let Some(recording) = &report.recording {
        let _ = writeln!(
            out,
            "- **Recording:** made {} at commit `{}` · sha256 `{}`",
            recording.recorded_at,
            recording
                .git_commit
                .as_deref()
                .map(short)
                .unwrap_or_else(|| "unknown".into()),
            short(&recording.sha256)
        );
    }
    if report.timings_source == "recorded" {
        if stale == 0 {
            let _ = writeln!(
                out,
                "\n> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run."
            );
        } else {
            let _ = writeln!(
                out,
                "\n> Replay: every score is this code's except those of the {stale} document(s) scored from replies to prompts the engine no longer builds (`--allow-stale`, marked in Misses), which do not measure this code. Timings and memory are the recording's, taken on the machine above - not measured by this run."
            );
        }
    }
    if let Some(change) = report
        .recording
        .as_ref()
        .and_then(|recording| recording.configuration_change.as_deref())
    {
        let _ = writeln!(out, "\n> **Warning:** {change}.");
    }
    if report.mode == EXTRACT {
        let _ = writeln!(
            out,
            "\n> Extract-only: the parser worker alone, no model. Each document is scored on what extraction decides - OCR against the drawn text, the structure the gold gives, and whether the gold's evidence reaches the digest - and timed as the worker reports it."
        );
    }
    let _ = writeln!(out);
}

fn short(value: &str) -> String {
    let (hash, suffix) = value
        .split_once('-')
        .map_or((value, ""), |(hash, suffix)| (hash, suffix));
    let cut = &hash[..hash.len().min(10)];
    if suffix.is_empty() {
        cut.to_owned()
    } else {
        format!("{cut}-{suffix}")
    }
}

pub fn percent(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

fn rate_cell(rate: Option<&Rate>) -> String {
    rate.map_or_else(
        || "–".to_owned(),
        |rate| format!("{}/{} ({})", rate.correct, rate.total, percent(rate.rate)),
    )
}

fn rate_short(rate: Option<&Rate>) -> String {
    rate.map_or_else(
        || "–".to_owned(),
        |rate| format!("{}/{}", rate.correct, rate.total),
    )
}

/// A duration, in the unit that reads best at its size.
pub fn duration(milliseconds: f64) -> String {
    if milliseconds >= 1_000.0 {
        format!("{:.2} s", milliseconds / 1_000.0)
    } else if milliseconds >= 1.0 {
        format!("{milliseconds:.1} ms")
    } else {
        format!("{milliseconds:.2} ms")
    }
}

pub fn metric_value(metric: &str, value: f64) -> String {
    match crate::timing::unit(metric) {
        Some(Unit::Milliseconds) => duration(value),
        Some(Unit::TokensPerSecond) => format!("{value:.1} tok/s"),
        _ => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                format!("{value:.1}")
            }
        }
    }
}

fn table(out: &mut String, header: &[&str], rows: &[Vec<String>]) {
    let _ = writeln!(out, "| {} |", header.join(" | "));
    let alignments = header
        .iter()
        .enumerate()
        .map(|(index, _)| if index == 0 { "---" } else { "---:" })
        .collect::<Vec<_>>();
    let _ = writeln!(out, "| {} |", alignments.join(" | "));
    for row in rows {
        let _ = writeln!(out, "| {} |", row.join(" | "));
    }
    let _ = writeln!(out);
}

fn scorecard(out: &mut String, summary: &Summary) {
    let _ = writeln!(out, "## Scorecard");
    let _ = writeln!(out);
    let mut rows = HEADLINE_RATES
        .iter()
        .filter_map(|(key, label)| {
            summary
                .rates
                .get(*key)
                .map(|rate| vec![(*label).to_owned(), rate_cell(Some(rate))])
        })
        .collect::<Vec<_>>();
    rows.extend(HEADLINE_MEANS.iter().filter_map(|(key, label)| {
        summary.means.get(*key).map(|mean| {
            vec![
                (*label).to_owned(),
                format!("{} (mean of {})", percent(mean.mean), mean.total_docs),
            ]
        })
    }));
    if let Some(review) = summary.review_rate {
        rows.push(vec![
            "Sent to review".to_owned(),
            format!("{} of {} completed", percent(review), summary.completed),
        ]);
    }
    table(out, &["Score", "Result"], &rows);
}

fn safety(out: &mut String, summary: &Summary) {
    let _ = writeln!(out, "## Safety");
    let _ = writeln!(out);
    let counts = &summary.counts;
    let of = |key: &str| summary.rates.get(key).map_or(0, |rate| rate.total);
    let mut rows = vec![
        vec![
            "Filed without review under a wrong name".to_owned(),
            format!("{} of {}", counts.unsafe_ready, of("unsafe_ready")),
        ],
        vec![
            "Trap date chosen".to_owned(),
            format!("{} of {}", counts.trap_dates, of("date_forbidden")),
        ],
        vec![
            "Forbidden party named".to_owned(),
            format!("{} of {}", counts.forbidden_parties, of("party_forbidden")),
        ],
        vec![
            "Spurious parties named".to_owned(),
            counts.spurious_parties.to_string(),
        ],
        vec![
            "Description states a known-wrong fact".to_owned(),
            counts.forbidden_descriptions.to_string(),
        ],
        vec![
            "Unsupported description claims".to_owned(),
            format!(
                "{} of {}{}",
                counts.unsupported_claims,
                counts.claims,
                summary
                    .unsupported_fact_rate
                    .map(|rate| format!(" ({})", percent(rate)))
                    .unwrap_or_default()
            ),
        ],
    ];
    if let Some(rate) = summary.rates.get("needless_review") {
        rows.push(vec![
            "Right name sent to review anyway".to_owned(),
            format!("{} of {}", rate.correct, rate.total),
        ]);
    }
    table(out, &["Check", "Count"], &rows);
}

/// An extract-only run's page: what was read, how well, and how fast.
fn render_extraction(report: &Report) -> String {
    let mut out = String::new();
    header(&mut out, report);
    extraction_scorecard(&mut out, report);
    structure(&mut out, report);
    routes(&mut out, report);
    ocr(&mut out, report);
    extraction_time(&mut out, report);
    stages(&mut out, report);
    memory(&mut out, report);
    extraction_misses(&mut out, report);
    baseline(&mut out, report);
    let _ = writeln!(out, "## How to compare");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Keep this run's JSON, change the worker, run again with the same corpus, then:\n\n```text\nintern-bench compare --before before.json --after after.json --markdown diff.md\n```\n\n\
         It recomputes every score over the documents both runs read, lists each document whose score moved past the extract-only gate's tolerance, \
         and gives the change in p50/p95 of every worker stage over the documents both completed. Compare runs made on the same machine. \
         `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON."
    );
    out
}

fn share(found: usize, total: usize) -> String {
    if total == 0 {
        "–".to_owned()
    } else {
        format!("{found}/{total} ({})", percent(found as f64 / total as f64))
    }
}

/// Every extraction score: pooled where it pools by item, and as the mean
/// of the documents that have it.
fn extraction_scorecard(out: &mut String, report: &Report) {
    let _ = writeln!(out, "## Extraction scorecard");
    let _ = writeln!(out);
    let pooled = report
        .structure
        .as_ref()
        .map(|structure| &structure.aggregate);
    let ocr = report.ocr.aggregate.as_ref();
    let pooled_cell = |key: &str| -> String {
        let structure = |pick: fn(&StructureFigures) -> (usize, usize)| {
            pooled.map_or_else(
                || "–".to_owned(),
                |figures| {
                    let (found, total) = pick(figures);
                    share(found, total)
                },
            )
        };
        let rate = |value: Option<f64>| value.map_or_else(|| "–".to_owned(), percent);
        match key {
            "reading_order_accuracy" => structure(|f| (f.pairs_in_order, f.pairs)),
            "table_row_accuracy" => structure(|f| (f.rows_found, f.rows)),
            "table_cell_recall" => structure(|f| (f.cells_found, f.cells)),
            "kv_accuracy" => structure(|f| (f.key_values_found, f.key_values)),
            "route_correct" => structure(|f| (f.routes_correct, f.route_pages)),
            "ocr_cer" => rate(ocr.and_then(|figures| figures.cer)),
            "ocr_cer_ci" => rate(ocr.and_then(|figures| figures.cer_ci)),
            "ocr_wer" => rate(ocr.and_then(|figures| figures.wer)),
            "ocr_date_accuracy" => rate(ocr.and_then(|figures| figures.date_accuracy)),
            "ocr_name_accuracy" => rate(ocr.and_then(|figures| figures.name_accuracy)),
            "ocr_identifier_accuracy" => rate(ocr.and_then(|figures| figures.identifier_accuracy)),
            "ocr_mean_confidence" => ocr
                .and_then(|figures| figures.mean_confidence)
                .map_or_else(|| "–".to_owned(), |value| format!("{value:.1}")),
            _ => "–".to_owned(),
        }
    };
    let rows = EXTRACTION_SCORES
        .iter()
        .filter_map(|key| {
            let mean = report.summary.means.get(*key)?;
            let shown = if is_unit_fraction(key) {
                percent(mean.mean)
            } else {
                format!("{:.1}", mean.mean)
            };
            Some(vec![
                format!("`{key}`"),
                pooled_cell(key),
                format!("{shown} ({} docs)", mean.total_docs),
            ])
        })
        .collect::<Vec<_>>();
    table(out, &["Score", "Pooled", "Mean per document"], &rows);
    let _ = writeln!(
        out,
        "Pooled figures count items over every document (snippet pairs, rows, cells, labelled values, pages; OCR characters, words and values); the mean gives each document one vote. A document whose extraction failed counts with every item missed. `digest_recall` is the share of the gold's date and party evidence in the digest the engine would build from this text.\n"
    );
}

fn structure(out: &mut String, report: &Report) {
    let Some(structure) = &report.structure else {
        return;
    };
    let _ = writeln!(out, "## Structure");
    let _ = writeln!(out);
    let row = |name: String, class: &str, figures: &StructureFigures| {
        vec![
            name,
            class.to_owned(),
            share(figures.pairs_in_order, figures.pairs),
            share(figures.rows_found, figures.rows),
            share(figures.cells_found, figures.cells),
            share(figures.key_values_found, figures.key_values),
            share(figures.routes_correct, figures.route_pages),
        ]
    };
    let mut rows = structure
        .documents
        .iter()
        .map(|document| {
            let class = if document.status == COMPLETED {
                document.route_class.as_deref().unwrap_or("–")
            } else {
                document.status.as_str()
            };
            row(document.id.clone(), class, &document.figures)
        })
        .collect::<Vec<_>>();
    rows.push(row("**All**".into(), "", &structure.aggregate));
    table(
        out,
        &[
            "Document",
            "Route class",
            "Reading order (pairs)",
            "Table rows",
            "Table cells",
            "Key-values",
            "Routes (pages)",
        ],
        &rows,
    );
    let _ = writeln!(
        out,
        "Measured over the page text the engine receives. Reading order: consecutive gold snippets found in order. Table rows: rows whose cells are all on one line, in order, an empty check box left empty. Table cells: cells found in their table's lines. Key-values: values after their label on its line, alone on the next line, or in the cell under it in a linearised table. Routes: pages whose layout took the expected route, judged only when the worker sends layouts. See `docs/internbench.md` for the exact rules.\n"
    );
}

fn routes(out: &mut String, report: &Report) {
    let Some(routes) = &report.routes else {
        return;
    };
    let _ = writeln!(out, "## Routes");
    let _ = writeln!(out);
    let mut names = ROUTES
        .iter()
        .map(|route| (*route).to_owned())
        .collect::<Vec<_>>();
    for name in routes.pages.keys() {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    let pages = names
        .iter()
        .filter_map(|name| {
            routes
                .pages
                .get(name)
                .map(|count| format!("{name} {count}"))
        })
        .collect::<Vec<_>>();
    if !pages.is_empty() {
        let _ = writeln!(out, "- **Pages:** {}", pages.join(" · "));
    }
    let classes = routes
        .classes
        .iter()
        .map(|(class, count)| format!("{class} {count}"))
        .collect::<Vec<_>>();
    if !classes.is_empty() {
        let _ = writeln!(
            out,
            "- **Documents by route class:** {}",
            classes.join(" · ")
        );
    }
    let _ = writeln!(out);
    if routes.confusion.is_empty() {
        return;
    }
    let mut taken = routes
        .confusion
        .values()
        .flat_map(|row| row.keys().cloned())
        .collect::<Vec<_>>();
    taken.sort_by_key(|name| {
        ROUTES
            .iter()
            .position(|route| route == name)
            .unwrap_or(ROUTES.len())
    });
    taken.dedup();
    let mut header = vec!["Expected, then taken".to_owned()];
    header.extend(taken.iter().cloned());
    let rows = ROUTES
        .iter()
        .filter_map(|expected| {
            let row = routes.confusion.get(*expected)?;
            let mut cells = vec![(*expected).to_owned()];
            cells.extend(taken.iter().map(|name| {
                row.get(name)
                    .map_or_else(|| "·".to_owned(), usize::to_string)
            }));
            Some(cells)
        })
        .collect::<Vec<_>>();
    let header = header.iter().map(String::as_str).collect::<Vec<_>>();
    table(out, &header, &rows);
    let _ = writeln!(
        out,
        "Pages the gold gives a route for, by the route they should take and the one they took (`none`: the page came without a layout).\n"
    );
}

/// Distributions of every metric, keyed by a group's value.
type LatencySlices =
    std::collections::BTreeMap<String, std::collections::BTreeMap<String, Distribution>>;

/// Worker time per document by route class, page count and kind, with the
/// stages it is made of.
fn extraction_time(out: &mut String, report: &Report) {
    let latency = &report.latency;
    if !latency.overall.contains_key("worker_total_ms") {
        return;
    }
    let _ = writeln!(out, "## Extraction time");
    let _ = writeln!(out);
    let p = |metrics: &std::collections::BTreeMap<String, Distribution>,
             metric: &str,
             pick: fn(&Distribution) -> f64| {
        metrics.get(metric).map_or_else(
            || "–".to_owned(),
            |distribution| duration(pick(distribution)),
        )
    };
    let p50 = |distribution: &Distribution| distribution.p50;
    let p95 = |distribution: &Distribution| distribution.p95;
    let sections: [(&str, &LatencySlices); 3] = [
        ("Route class", &latency.by_route_class),
        ("Pages", &latency.by_page_bucket),
        ("Kind", &latency.by_kind),
    ];
    for (column, groups) in sections {
        if groups.is_empty() {
            continue;
        }
        let mut values = groups.keys().cloned().collect::<Vec<_>>();
        if column == "Pages" {
            values.sort_by_key(|value| PAGE_BUCKETS.iter().position(|bucket| bucket == value));
        } else if column == "Route class" {
            values.sort_by_key(|value| {
                ["fast", "layout", "ocr_regions", "ocr", "unrouted"]
                    .iter()
                    .position(|class| class == value)
            });
        }
        let rows = values
            .iter()
            .map(|value| {
                let metrics = &groups[value];
                vec![
                    value.clone(),
                    metrics.get("worker_total_ms").map_or_else(
                        || "0".to_owned(),
                        |distribution| distribution.count.to_string(),
                    ),
                    p(metrics, "extraction_wall_ms", p50),
                    p(metrics, "extraction_wall_ms", p95),
                    p(metrics, "worker_total_ms", p50),
                    p(metrics, "worker_total_ms", p95),
                    p(metrics, "worker_parse_ms", p95),
                    p(metrics, "worker_analysis_ms", p95),
                    p(metrics, "worker_render_ms", p95),
                    p(metrics, "worker_ocr_ms", p95),
                ]
            })
            .collect::<Vec<_>>();
        table(
            out,
            &[
                column,
                "Docs",
                "p50 wall",
                "p95 wall",
                "p50 worker",
                "p95 worker",
                "p95 parse",
                "p95 analysis",
                "p95 render",
                "p95 OCR",
            ],
            &rows,
        );
    }
    let _ = writeln!(
        out,
        "Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.\n"
    );
}

/// What extraction failed on, and the structure items it missed.
fn extraction_misses(out: &mut String, report: &Report) {
    let mut lines = Vec::new();
    for record in &report.records {
        if record.status != COMPLETED {
            lines.push(format!(
                "- **{}**: {}{}",
                record.id,
                record.status,
                record
                    .error
                    .as_deref()
                    .map(|error| format!(" ({error})"))
                    .unwrap_or_default()
            ));
            continue;
        }
        if let Some(structure) = &record.structure
            && !structure.misses.is_empty()
        {
            lines.push(format!(
                "- **{}**: {}",
                record.id,
                structure.misses.join("; ")
            ));
        }
    }
    let _ = writeln!(out, "## Misses ({})", lines.len());
    let _ = writeln!(out);
    if lines.is_empty() {
        let _ = writeln!(
            out,
            "Every document was read, and every structure item found.\n"
        );
        return;
    }
    for line in lines.iter().take(MAX_MISSES) {
        let _ = writeln!(out, "{line}");
    }
    if lines.len() > MAX_MISSES {
        let _ = writeln!(
            out,
            "- … and {} more in the JSON report",
            lines.len() - MAX_MISSES
        );
    }
    let _ = writeln!(out);
}

fn latency_cells(report: &Report, by: &str, value: &str) -> [String; 2] {
    let slice = match by {
        "kind" => report.latency.by_kind.get(value),
        "text_layer" => report.latency.by_text_layer.get(value),
        _ => report.latency.by_page_bucket.get(value),
    };
    let total = slice.and_then(|metrics| metrics.get("total_ms"));
    [
        total.map_or_else(|| "–".into(), |distribution| duration(distribution.p50)),
        total.map_or_else(|| "–".into(), |distribution| duration(distribution.p95)),
    ]
}

fn groups(out: &mut String, report: &Report) {
    let sections: [(&str, &str, &str); 3] = [
        ("kind", "By kind", "Kind"),
        ("text_layer", "By text layer", "Text layer"),
        ("page_bucket", "By page count", "Pages"),
    ];
    for (dimension, title, column) in sections {
        let Some(groups) = report.groups.get(dimension) else {
            continue;
        };
        let mut values = groups.keys().cloned().collect::<Vec<_>>();
        if dimension == "page_bucket" {
            values.sort_by_key(|value| PAGE_BUCKETS.iter().position(|bucket| bucket == value));
        }
        let rows = values
            .iter()
            .map(|value| {
                let summary = &groups[value];
                let [p50, p95] = latency_cells(report, dimension, value);
                vec![
                    value.clone(),
                    summary.documents.to_string(),
                    rate_short(summary.rates.get("filename_correct")),
                    rate_short(summary.rates.get("date_correct")),
                    rate_short(summary.rates.get("parties_correct")),
                    rate_short(summary.rates.get("readiness_match")),
                    summary.review_rate.map_or_else(|| "–".into(), percent),
                    p50,
                    p95,
                ]
            })
            .collect::<Vec<_>>();
        let _ = writeln!(out, "## {title}");
        let _ = writeln!(out);
        table(
            out,
            &[
                column,
                "Docs",
                "Filename",
                "Date",
                "Parties",
                "Routing",
                "Review",
                "p50 total",
                "p95 total",
            ],
            &rows,
        );
    }
    if let Some(categories) = report.groups.get("category") {
        let rows = categories
            .iter()
            .map(|(category, summary)| {
                vec![
                    category.clone(),
                    summary.documents.to_string(),
                    rate_short(summary.rates.get("filename_correct")),
                    rate_short(summary.rates.get("date_correct")),
                    rate_short(summary.rates.get("parties_correct")),
                    (summary.counts.trap_dates + summary.counts.forbidden_parties).to_string(),
                ]
            })
            .collect::<Vec<_>>();
        let _ = writeln!(out, "## By challenge");
        let _ = writeln!(out);
        table(
            out,
            &[
                "Category",
                "Docs",
                "Filename",
                "Date",
                "Parties",
                "Traps sprung",
            ],
            &rows,
        );
    }
}

fn figure(value: Option<f64>) -> String {
    value.map_or_else(|| "–".to_owned(), percent)
}

fn ocr_row(name: String, layer: &str, figures: &OcrFigures, time: Option<f64>) -> Vec<String> {
    let mut pages = figures.pages_compared.to_string();
    if figures.pages_missing > 0 {
        let _ = write!(pages, " (+{} unread)", figures.pages_missing);
    }
    if figures.pages_failed > 0 {
        let _ = write!(pages, " (+{} failed, read as empty)", figures.pages_failed);
    }
    vec![
        name,
        layer.to_owned(),
        pages,
        figure(figures.cer),
        figure(figures.cer_ci),
        figure(figures.wer),
        figure(figures.date_accuracy),
        figure(figures.name_accuracy),
        figure(figures.identifier_accuracy),
        figures
            .mean_confidence
            .map_or_else(|| "–".into(), |confidence| format!("{confidence:.0}")),
        time.map_or_else(|| "–".into(), duration),
    ]
}

fn ocr(out: &mut String, report: &Report) {
    let Some(aggregate) = &report.ocr.aggregate else {
        return;
    };
    let _ = writeln!(out, "## OCR");
    let _ = writeln!(out);
    let mut rows = report
        .ocr
        .documents
        .iter()
        .map(|row| ocr_row(row.id.clone(), &row.text_layer, &row.figures, row.ocr_ms))
        .collect::<Vec<_>>();
    rows.push(ocr_row("**All scanned pages**".into(), "", aggregate, None));
    table(
        out,
        &[
            "Document",
            "Layer",
            "Pages",
            "CER",
            "CER (any case)",
            "WER",
            "Dates",
            "Names",
            "IDs",
            "Confidence",
            "OCR time",
        ],
        &rows,
    );
    let _ = writeln!(
        out,
        "Error rates are edit distances over the drawn text's length, pooled over pages; Dates, Names and IDs are the fraction of those drawn on the read pages that survive OCR. \
         A page the reader does not return (a TIFF frame it does not read, shown as unread) counts as read empty, \
         as does every page of a scan whose extraction failed: every character and value on it missed.\n"
    );
}

fn stages(out: &mut String, report: &Report) {
    let overall = &report.latency.overall;
    if overall.is_empty() {
        return;
    }
    let _ = writeln!(out, "## Where the time goes");
    let _ = writeln!(out);
    let row = |metric: &str, distribution: &Distribution| {
        vec![
            format!("`{metric}`"),
            distribution.count.to_string(),
            metric_value(metric, distribution.p50),
            metric_value(metric, distribution.p95),
            metric_value(metric, distribution.max),
        ]
    };
    let timed = METRICS
        .iter()
        .filter(|(_, unit)| *unit == Unit::Milliseconds)
        .filter_map(|(metric, _)| {
            overall
                .get(*metric)
                .map(|distribution| row(metric, distribution))
        })
        .collect::<Vec<_>>();
    table(out, &["Stage", "Docs", "p50", "p95", "Max"], &timed);
    let left_out = report.summary.documents
        - report.summary.completed
        - report.summary.statuses.get(PENDING).copied().unwrap_or(0);
    if left_out > 0 {
        let _ = writeln!(
            out,
            "Over the {} documents that completed. The other {left_out} (failed, or not scored) are left out: a failed document's time is how long it took to fail.\n",
            report.summary.completed
        );
    }
    let counted = METRICS
        .iter()
        .filter(|(_, unit)| *unit != Unit::Milliseconds)
        .filter_map(|(metric, _)| {
            overall
                .get(*metric)
                .map(|distribution| row(metric, distribution))
        })
        .collect::<Vec<_>>();
    if !counted.is_empty() {
        table(out, &["Measure", "Docs", "p50", "p95", "Max"], &counted);
    }
}

fn memory(out: &mut String, report: &Report) {
    let memory = &report.memory;
    let _ = writeln!(out, "## Memory");
    let _ = writeln!(out);
    if !memory.sampled {
        let reason = if report.timings_source == "recorded" {
            "The recording carries no memory samples."
        } else {
            "Not sampled: memory is read from /proc, on Linux only."
        };
        let _ = writeln!(out, "{reason}\n");
        return;
    }
    let rows = [
        ("worker_mb", "Parser worker"),
        ("server_mb", "Model server"),
        ("bench_mb", "Benchmark (engine)"),
    ]
    .iter()
    .filter_map(|(key, label)| {
        let distribution = memory.per_document.get(*key)?;
        Some(vec![
            (*label).to_owned(),
            format!("{:.0} MB", distribution.p50),
            format!("{:.0} MB", distribution.max),
            memory.peak_documents.get(*key).cloned().unwrap_or_default(),
        ])
    })
    .collect::<Vec<_>>();
    table(
        out,
        &["Process", "Typical peak (p50)", "Highest peak", "During"],
        &rows,
    );
    let _ = writeln!(
        out,
        "Resident memory sampled every {} ms per document; a spike shorter than that can be missed.\n",
        memory.interval_ms
    );
}

fn misses(out: &mut String, report: &Report) {
    let lines = report
        .records
        .iter()
        .filter(|record| record.status != PENDING)
        .filter_map(|record| {
            if record.status != COMPLETED {
                let error = record
                    .error
                    .as_deref()
                    .map(|error| format!(" ({error})"))
                    .unwrap_or_default();
                let expected = record
                    .gold_filenames
                    .first()
                    .map(|name| format!(": expected `{name}`"))
                    .unwrap_or_default();
                return Some(format!(
                    "- **{}**: {}{error}{expected}",
                    record.id, record.status
                ));
            }
            if record.bool_score("filename_correct") != Some(false) {
                return None;
            }
            let reasons = if record.readiness.as_deref() == Some("ready") {
                " · **filed without review**".to_owned()
            } else if record.review_reasons.is_empty() {
                " · sent to review".to_owned()
            } else {
                format!(" · review: {}", record.review_reasons.join(", "))
            };
            let traps = if record.traps.is_empty() {
                String::new()
            } else {
                format!(" · trap: {}", record.traps.join("; "))
            };
            let stale = if record.stale { " · stale reply" } else { "" };
            Some(format!(
                "- **{}**: expected `{}`, got `{}`{traps}{reasons}{stale}",
                record.id,
                record
                    .gold_filenames
                    .first()
                    .map(String::as_str)
                    .unwrap_or("?"),
                record.filename.as_deref().unwrap_or("")
            ))
        })
        .collect::<Vec<_>>();
    let _ = writeln!(out, "## Misses ({})", lines.len());
    let _ = writeln!(out);
    let stale = report
        .records
        .iter()
        .filter(|record| record.stale)
        .map(|record| record.id.as_str())
        .collect::<Vec<_>>();
    if !stale.is_empty() {
        let _ = writeln!(
            out,
            "Scored from replies to prompts the engine no longer builds (`--allow-stale`): {}.\n",
            stale.join(", ")
        );
    }
    if lines.is_empty() {
        let _ = writeln!(out, "Every scored filename is right.\n");
        return;
    }
    for line in lines.iter().take(MAX_MISSES) {
        let _ = writeln!(out, "{line}");
    }
    if lines.len() > MAX_MISSES {
        let _ = writeln!(
            out,
            "- … and {} more in the JSON report",
            lines.len() - MAX_MISSES
        );
    }
    let _ = writeln!(out);
}

fn baseline(out: &mut String, report: &Report) {
    let Some(comparison) = &report.baseline else {
        return;
    };
    let verdict = if comparison.passed {
        "passed"
    } else {
        "FAILED"
    };
    let _ = writeln!(
        out,
        "## Baseline: {verdict} (gates: {})",
        comparison.gates.join(", ")
    );
    let _ = writeln!(out);
    let list = |out: &mut String, title: &str, lines: &[String]| {
        if lines.is_empty() {
            return;
        }
        let _ = writeln!(out, "{title} ({}):\n", lines.len());
        for line in lines.iter().take(MAX_MISSES) {
            let _ = writeln!(out, "- {line}");
        }
        if lines.len() > MAX_MISSES {
            let _ = writeln!(out, "- … and {} more", lines.len() - MAX_MISSES);
        }
        let _ = writeln!(out);
    };
    list(out, "Failures", &comparison.failures);
    if report.mode == "live" {
        let ungated = comparison
            .ungated_regressions()
            .cloned()
            .collect::<Vec<_>>();
        list(
            out,
            "Per-document flips to worse (reported, not gated live)",
            &ungated,
        );
    }
    list(out, "Improvements", &comparison.document_improvements);
    list(out, "New documents", &comparison.new);
    list(out, "Missing documents", &comparison.missing);
    if comparison.failures.is_empty()
        && comparison.document_improvements.is_empty()
        && comparison.document_regressions.is_empty()
    {
        let _ = writeln!(out, "No document changed.\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_and_values_read_at_their_size() {
        assert_eq!(duration(0.25), "0.25 ms");
        assert_eq!(duration(12.345), "12.3 ms");
        assert_eq!(duration(1_234.5), "1.23 s");
        assert_eq!(metric_value("prompt_tokens", 4_000.0), "4000");
        assert_eq!(metric_value("prefill_tok_per_s", 101.26), "101.3 tok/s");
        assert_eq!(percent(0.6889), "68.9%");
        assert_eq!(short("0123456789abcdef-dirty"), "0123456789-dirty");
    }
}
