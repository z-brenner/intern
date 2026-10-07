//! The `run`, `compare` and `report` subcommands, from parsed options to
//! files written and an exit status.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use intern_engine::DigestBudget;

use crate::{
    baseline::{self, Baseline},
    compare,
    extract::{self, ExtractOptions},
    gold::{GoldDocument, GoldFile},
    live::{self, LiveOptions},
    machine::{MachineInfo, git_commit, utc_now},
    markdown,
    pipeline::EngineSettings,
    record::{DocumentRecord, PENDING, STALE_PROMPT, is_unscorable},
    recording::{RECORDING_SCHEMA_VERSION, Recording, sha256_hex},
    replay::{self, Manifest},
    report::{self, CorpusInfo, EXTRACT, RecordingInfo, Report, RunInfo, SUITE},
    stats::round,
};

/// The run scored, but worse than the baseline, or with documents replay
/// could not score.
pub const EXIT_REGRESSED: i32 = 2;

pub enum Mode {
    Live {
        options: LiveOptions,
        record: Option<PathBuf>,
        note: String,
    },
    Replay {
        recording: PathBuf,
        allow_stale: bool,
    },
    /// The worker alone, scored on extraction.
    Extract { options: ExtractOptions },
}

pub struct RunOptions {
    pub corpus: PathBuf,
    pub gold: PathBuf,
    pub manifest: Option<PathBuf>,
    pub only: Vec<String>,
    pub output: Option<PathBuf>,
    pub markdown: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub write_baseline: Option<PathBuf>,
    pub latency_gate: Option<f64>,
    pub mode: Mode,
    /// The engine's pipeline and retrieval ([`crate::pipeline`]).
    pub engine: EngineSettings,
}

/// The documents a run covers, in gold order.
pub fn select<'a>(gold: &'a GoldFile, only: &[String]) -> Result<Vec<&'a GoldDocument>, String> {
    if only.is_empty() {
        return Ok(gold.documents.iter().collect());
    }
    let known = gold
        .documents
        .iter()
        .map(|document| document.id.as_str())
        .collect::<BTreeSet<_>>();
    let unknown = only
        .iter()
        .filter(|id| !known.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(format!(
            "--only names unknown documents: {}",
            unknown.join(", ")
        ));
    }
    Ok(gold
        .documents
        .iter()
        .filter(|document| only.contains(&document.id))
        .collect())
}

pub fn run(options: RunOptions) -> Result<i32, String> {
    // A baseline written from a subset over one that covers more would hold
    // later full runs to the subset alone: every document left out would
    // read as new, and new documents gate nothing. Re-record a subset with
    // `merge-recordings` instead, then write the baseline from a replay of
    // the whole corpus.
    if let Some(path) = &options.write_baseline
        && !options.only.is_empty()
        && let Ok(bytes) = std::fs::read(path)
    {
        let existing = Baseline::parse(&bytes)?;
        let dropped = existing
            .documents
            .keys()
            .filter(|id| !options.only.contains(id))
            .count();
        if dropped > 0 {
            return Err(format!(
                "--write-baseline with --only would drop {dropped} document(s) from {}; \
                 merge the subset's recording with `intern-bench merge-recordings`, \
                 then write the baseline from a replay of the whole corpus",
                path.display()
            ));
        }
    }
    let (gold, gold_bytes) = GoldFile::load(&options.gold)?;
    let documents = select(&gold, &options.only)?;
    let manifest = match &options.manifest {
        Some(path) => {
            let bytes = std::fs::read(path)
                .map_err(|error| format!("cannot read manifest {}: {error}", path.display()))?;
            Some((Manifest::parse(&bytes)?, sha256_hex(&bytes)))
        }
        None => None,
    };
    let corpus = CorpusInfo {
        documents: documents.len(),
        gold_sha256: sha256_hex(&gold_bytes),
        manifest_sha256: manifest.as_ref().map(|(_, sha)| sha.clone()),
        only: options.only.clone(),
    };
    let created_at = utc_now();
    let commit = git_commit();

    let (records, info, allow_stale) = match options.mode {
        Mode::Replay {
            recording,
            allow_stale,
        } => {
            let (recorded, bytes) = Recording::load(&recording)?;
            let configuration_change =
                crate::pipeline::configuration_change(&recorded, &options.engine)?;
            if let Some(change) = &configuration_change {
                eprintln!("WARNING: {change}");
            }
            if manifest.is_none() && !options.corpus.is_dir() {
                eprintln!(
                    "WARNING: no --manifest and no corpus at {}: nothing checks that the documents are the ones recorded",
                    options.corpus.display()
                );
            }
            let records = documents
                .iter()
                .map(|document| {
                    replay::replay_document(
                        document,
                        &recorded,
                        manifest.as_ref().map(|(manifest, _)| manifest),
                        &options.corpus,
                        allow_stale,
                        &options.engine,
                    )
                })
                .collect::<Vec<_>>();
            let info = RunInfo {
                mode: "replay".into(),
                created_at,
                machine: recorded.machine.clone(),
                model: recorded.model.clone(),
                git_commit: commit,
                worker: recorded.worker.clone(),
                timings_source: "recorded".into(),
                recording: Some(RecordingInfo {
                    path: recording.display().to_string(),
                    sha256: sha256_hex(&bytes),
                    recorded_at: recorded.recorded_at.clone(),
                    git_commit: recorded.git_commit.clone(),
                    note: recorded.note.clone(),
                    configuration_change,
                }),
                corpus,
                wall_ms: None,
            };
            (records, info, allow_stale)
        }
        Mode::Extract {
            options: extract_options,
        } => {
            // Nothing is recorded, but scores against the gold mean nothing
            // for bytes the gold was not written for.
            check_corpus(
                &documents,
                &options.corpus,
                manifest.as_ref().map(|(manifest, _)| manifest),
            )?;
            let machine = MachineInfo::current();
            let owned = documents
                .iter()
                .map(|document| (*document).clone())
                .collect::<Vec<_>>();
            let outcome = extract::run(
                &owned,
                &options.corpus,
                &extract_options,
                &options.engine.retrieval,
            )?;
            let info = RunInfo {
                mode: EXTRACT.into(),
                created_at,
                machine,
                model: Default::default(),
                git_commit: commit,
                worker: Some(extract_options.worker.display().to_string()),
                timings_source: "measured".into(),
                recording: None,
                corpus,
                wall_ms: Some(round(outcome.wall_micros as f64 / 1000.0, 1)),
            };
            (outcome.records, info, false)
        }
        Mode::Live {
            options: live_options,
            record,
            note,
        } => {
            // A live run is expensive and its recording is only worth keeping
            // if it read the documents the manifest vouches for, so a
            // mismatch stops it before the first document.
            check_corpus(
                &documents,
                &options.corpus,
                manifest.as_ref().map(|(manifest, _)| manifest),
            )?;
            let machine = MachineInfo::current();
            let owned = documents
                .iter()
                .map(|document| (*document).clone())
                .collect::<Vec<_>>();
            let started = std::time::Instant::now();
            let outcome = live::run(&owned, &options.corpus, &live_options, &options.engine)?;
            let wall_ms = round(started.elapsed().as_secs_f64() * 1000.0, 1);
            if let Some(path) = &record {
                Recording {
                    schema_version: RECORDING_SCHEMA_VERSION,
                    suite: SUITE.to_owned(),
                    recorded_at: created_at.clone(),
                    model: outcome.model.clone(),
                    context_tokens: outcome.context_tokens,
                    budget_characters: DigestBudget::default().max_characters,
                    engine: options.engine.header(),
                    machine: machine.clone(),
                    git_commit: commit.clone(),
                    worker: Some(live_options.worker.display().to_string()),
                    note,
                    documents: outcome.recorded,
                }
                .save(path)?;
                eprintln!(
                    "recorded {} documents to {}",
                    documents.len(),
                    path.display()
                );
            }
            let info = RunInfo {
                mode: "live".into(),
                created_at,
                machine,
                model: outcome.model,
                git_commit: commit,
                worker: Some(live_options.worker.display().to_string()),
                timings_source: "measured".into(),
                recording: None,
                corpus,
                wall_ms: Some(wall_ms),
            };
            (outcome.records, info, false)
        }
    };

    let mut exit = 0;
    let unscored = records
        .iter()
        .filter(|record| is_unscorable(&record.status))
        .map(|record| format!("{} ({})", record.id, record.status))
        .collect::<Vec<_>>();
    if !unscored.is_empty() {
        eprintln!(
            "{} document(s) could not be scored from the recording: {}",
            unscored.len(),
            unscored.join(", ")
        );
        eprintln!(
            "re-record (scripts/run-internbench.sh, then `intern-bench merge-recordings` to replace only these documents); --allow-stale scores stale_prompt documents anyway, never stale_fixture or unrecorded ones"
        );
        exit = EXIT_REGRESSED;
    }
    let stale = records.iter().filter(|record| record.stale).count();
    if stale > 0 && allow_stale {
        eprintln!(
            "{stale} document(s) scored from replies to prompts the engine no longer builds ({STALE_PROMPT}, allowed)"
        );
    }
    let pending = records
        .iter()
        .filter(|record| record.status == PENDING)
        .map(|record| record.id.as_str())
        .collect::<Vec<_>>();
    if !pending.is_empty() {
        eprintln!(
            "{} document(s) pending a recording: {}",
            pending.len(),
            pending.join(", ")
        );
    }
    for line in miss_lines(&records) {
        eprintln!("{line}");
    }

    let mut report = report::build(info, records);
    if let Some(path) = &options.baseline {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("cannot read baseline {}: {error}", path.display()))?;
        let comparison =
            baseline::compare(&report, &Baseline::parse(&bytes)?, options.latency_gate);
        for line in &comparison.failures {
            eprintln!("regressed: {line}");
        }
        if report.mode == "live" {
            for line in comparison.ungated_regressions() {
                eprintln!("flipped (not gated live): {line}");
            }
        }
        for line in &comparison.document_improvements {
            eprintln!("improved: {line}");
        }
        eprintln!(
            "baseline: {} ({} gated regression(s), {} improvement(s), {} new, {} pending)",
            if comparison.passed {
                "passed"
            } else {
                "failed"
            },
            comparison.failures.len(),
            comparison.document_improvements.len(),
            comparison.new.len(),
            comparison.pending.len()
        );
        if !comparison.passed {
            exit = EXIT_REGRESSED;
        }
        report.baseline = Some(comparison);
    }
    // A baseline written from a replay that could not score every document,
    // or that scored some from stale replies, would replace good coverage
    // with nothing - or with scores that do not measure this code.
    let unfit = report
        .records
        .iter()
        .filter(|record| is_unscorable(&record.status) || record.stale)
        .map(|record| record.id.as_str())
        .collect::<Vec<_>>();
    if options.write_baseline.is_some() && !unfit.is_empty() {
        eprintln!(
            "not writing the baseline: {} document(s) are stale or could not be scored ({})",
            unfit.len(),
            unfit.join(", ")
        );
        exit = EXIT_REGRESSED;
    } else if let Some(path) = &options.write_baseline {
        write(path, &Baseline::from_report(&report).to_json(), "baseline")?;
        eprintln!(
            "wrote the baseline for {} documents to {}",
            report.records.len(),
            path.display()
        );
    }
    headline(&report);
    let json = report.to_json()?;
    match &options.output {
        Some(path) => write(path, &json, "report")?,
        None => print!("{json}"),
    }
    if let Some(path) = &options.markdown {
        write(path, &markdown::render(&report), "markdown report")?;
    }
    Ok(exit)
}

/// Refuses a corpus that lacks a selected document, or - given a manifest -
/// holds one whose bytes it does not vouch for: a measurement of other bytes
/// than the gold describes would be scored against the wrong answers.
fn check_corpus(
    documents: &[&GoldDocument],
    corpus: &Path,
    manifest: Option<&Manifest>,
) -> Result<(), String> {
    let missing = documents
        .iter()
        .filter(|document| !corpus.join(&document.file).is_file())
        .map(|document| document.file.as_str())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "{} document(s) missing from {} (generate the corpus with `node bench/generate.mjs`): {}",
            missing.len(),
            corpus.display(),
            missing.join(", ")
        ));
    }
    if let Some(manifest) = manifest {
        let differ = documents
            .iter()
            .filter(|document| {
                let bytes = std::fs::read(corpus.join(&document.file)).unwrap_or_default();
                // A document the manifest does not list is not vouched for
                // either.
                manifest
                    .sha256(&document.file)
                    .is_none_or(|expected| expected != sha256_hex(&bytes))
            })
            .map(|document| document.file.as_str())
            .collect::<Vec<_>>();
        if !differ.is_empty() {
            return Err(format!(
                "{} document(s) in {} differ from the manifest or are not in it (regenerate them with `node bench/generate.mjs` on the pinned Node, or leave out --manifest to measure these bytes anyway): {}",
                differ.len(),
                corpus.display(),
                differ.join(", ")
            ));
        }
    }
    Ok(())
}

/// "id: expected X, got Y" for every scored name that missed, as
/// `intern-evaluate` prints them.
fn miss_lines(records: &[DocumentRecord]) -> Vec<String> {
    records
        .iter()
        .filter(|record| record.bool_score("filename_correct") == Some(false))
        .map(|record| {
            format!(
                "{}: filename: expected {}, got {}",
                record.id,
                record
                    .gold_filenames
                    .first()
                    .map(String::as_str)
                    .unwrap_or("?"),
                record.filename.as_deref().unwrap_or("nothing")
            )
        })
        .collect()
}

/// Three lines on standard error, so a terminal run ends with the answer.
fn headline(report: &Report) {
    if report.mode == EXTRACT {
        extraction_headline(report);
        return;
    }
    let summary = &report.summary;
    let rate = |key: &str| {
        summary.rates.get(key).map_or_else(
            || "–".to_owned(),
            |rate| {
                format!(
                    "{}/{} ({})",
                    rate.correct,
                    rate.total,
                    markdown::percent(rate.rate)
                )
            },
        )
    };
    eprintln!(
        "filename {} · type {} · date {} · parties {} · routing {}",
        rate("filename_correct"),
        rate("type_correct"),
        rate("date_correct"),
        rate("parties_correct"),
        rate("readiness_match")
    );
    eprintln!(
        "unsafe ready {} · trap dates {} · forbidden parties {} · unsupported claims {}/{}",
        summary.counts.unsafe_ready,
        summary.counts.trap_dates,
        summary.counts.forbidden_parties,
        summary.counts.unsupported_claims,
        summary.counts.claims
    );
    if let Some(total) = report.latency.overall.get("total_ms") {
        eprintln!(
            "total per document: p50 {} · p95 {} · max {}{}",
            markdown::duration(total.p50),
            markdown::duration(total.p95),
            markdown::duration(total.max),
            if report.timings_source == "recorded" {
                " (recorded)"
            } else {
                ""
            }
        );
    }
}

/// An extract-only run's answer: what was read, how well, and how fast.
fn extraction_headline(report: &Report) {
    let summary = &report.summary;
    let figure = |value: Option<f64>| value.map_or_else(|| "–".to_owned(), markdown::percent);
    eprintln!(
        "extracted {} of {} documents{}",
        summary.completed,
        summary.documents,
        report
            .wall_ms
            .map(|wall| format!(" in {}", markdown::duration(wall)))
            .unwrap_or_default()
    );
    if let Some(structure) = &report.structure {
        let figures = &structure.aggregate;
        eprintln!(
            "reading order {} · table rows {} · table cells {} · key-values {} · routes {}",
            figure(figures.reading_order_accuracy),
            figure(figures.table_row_accuracy),
            figure(figures.table_cell_recall),
            figure(figures.kv_accuracy),
            figure(figures.route_correct)
        );
    }
    if let Some(ocr) = &report.ocr.aggregate {
        eprintln!(
            "OCR: CER {} · WER {} · dates {} · names {} · identifiers {}",
            figure(ocr.cer),
            figure(ocr.wer),
            figure(ocr.date_accuracy),
            figure(ocr.name_accuracy),
            figure(ocr.identifier_accuracy)
        );
    }
    if let Some(total) = report.latency.overall.get("worker_total_ms") {
        eprintln!(
            "worker per document: p50 {} · p95 {} · max {}",
            markdown::duration(total.p50),
            markdown::duration(total.p95),
            markdown::duration(total.max)
        );
    }
}

pub fn compare_reports(
    before: &Path,
    after: &Path,
    markdown: Option<&Path>,
    output: Option<&Path>,
) -> Result<i32, String> {
    let read = |path: &Path| {
        std::fs::read(path)
            .map_err(|error| format!("cannot read report {}: {error}", path.display()))
            .and_then(|bytes| Report::parse(&bytes))
    };
    let comparison = compare::compare(&read(before)?, &read(after)?);
    let rendered = compare::render(&comparison);
    match output {
        Some(path) => write(path, &comparison.to_json(), "comparison")?,
        None if markdown.is_none() => print!("{rendered}"),
        None => {}
    }
    if let Some(path) = markdown {
        write(path, &rendered, "comparison")?;
    }
    Ok(0)
}

pub fn render_report(input: &Path, markdown_path: &Path) -> Result<i32, String> {
    let bytes = std::fs::read(input)
        .map_err(|error| format!("cannot read report {}: {error}", input.display()))?;
    let report = Report::parse(&bytes)?;
    write(markdown_path, &markdown::render(&report), "markdown report")?;
    Ok(0)
}

fn write(path: &Path, contents: &str, what: &str) -> Result<(), String> {
    std::fs::write(path, contents)
        .map_err(|error| format!("cannot write {what} {}: {error}", path.display()))
}
