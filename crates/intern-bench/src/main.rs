//! `intern-bench`: run InternBench, compare two runs, or re-render a report.
//!
//! ```text
//! intern-bench run --corpus bench/generated --gold bench/gold.json
//!                  (--worker PATH --endpoint URL --api-key KEY [--model-id intern-local]
//!                   [--model-path GGUF] [--record OUT.json] [--note TEXT] [--server-pid PID]
//!                   [--no-warmup]
//!                  | --replay bench/recording.json [--allow-stale]
//!                  | --extract-only --worker PATH [--no-warmup])
//!                  [--only id,id] [--manifest bench/manifest.json]
//!                  [--output report.json] [--markdown report.md]
//!                  [--baseline bench/baseline.json] [--write-baseline bench/baseline.json]
//!                  [--latency-gate 1.5]
//!                  [--pipeline digest|evidence] [--id-style stable|ordinal]
//!                  [--retrieval-tier auto|whole|small|normal|dense] [--context-tokens N]
//!                  [--field-order fact-first|evidence-first] [--string-limits bounded|unbounded]
//! intern-bench compare --before a.json --after b.json [--markdown diff.md] [--output diff.json]
//! intern-bench report  --input report.json --markdown out.md
//! intern-bench merge-recordings --base bench/recording.json --add new.json --output merged.json
//!                  [--gold bench/gold.json] [--only id,id] [--note TEXT]
//! intern-bench retrieval --recording bench/recording.json[,more.json] --gold bench/gold.json
//!                  [--fixtures fixtures/corpus-recording.json --expected fixtures/expected.json]
//!                  [--corpus bench/generated --worker PATH]
//!                  [--config a.json,b.json] [--sweep] [--only id,id]
//!                  [--output sweep.json] [--markdown sweep.md] [--dump DIR]
//! ```
//!
//! Exit status: 0 when the run scored; 2 when it regressed against the
//! baseline or replay could not score a document; 1 for a usage or I/O
//! error.

use std::{collections::HashMap, env, path::PathBuf, process};

use intern_bench::{
    extract::ExtractOptions,
    live::LiveOptions,
    run::{Mode, RunOptions, compare_reports, render_report, run},
};

const USAGE: &str = "usage:
  intern-bench run --corpus DIR --gold GOLD.json
                   (--worker PATH --endpoint URL --api-key KEY [--model-id ID] [--model-path GGUF]
                    [--record OUT.json] [--note TEXT] [--server-pid PID] [--no-warmup]
                   | --replay RECORDING.json [--allow-stale]
                   | --extract-only --worker PATH [--no-warmup])
                   [--only id,id] [--manifest MANIFEST.json] [--output REPORT.json] [--markdown REPORT.md]
                   [--baseline BASELINE.json] [--write-baseline BASELINE.json] [--latency-gate RATIO]
                   [--pipeline digest|evidence] [--id-style stable|ordinal]
                   [--retrieval-tier auto|whole|small|normal|dense] [--context-tokens N]
                   [--field-order fact-first|evidence-first] [--string-limits bounded|unbounded]
  intern-bench compare --before A.json --after B.json [--markdown DIFF.md] [--output DIFF.json]
  intern-bench report --input REPORT.json --markdown OUT.md
  intern-bench merge-recordings --base A.json --add B.json --output C.json
                   [--gold GOLD.json (default bench/gold.json)] [--only id,id] [--note TEXT]
  intern-bench retrieval --recording A.json[,B.json] --gold GOLD.json
                   [--fixtures RECORDING.json --expected EXPECTED.json]
                   [--corpus DIR --worker PATH]
                   [--config C.json[,D.json]] [--sweep] [--only id,id]
                   [--output SWEEP.json] [--markdown SWEEP.md] [--dump DIR]";

/// Arguments that take no value.
const FLAGS: &[&str] = &["allow-stale", "no-warmup", "extract-only", "sweep"];

const RUN_KEYS: &[&str] = &[
    "corpus",
    "gold",
    "worker",
    "endpoint",
    "api-key",
    "model-id",
    "model-path",
    "record",
    "note",
    "server-pid",
    "no-warmup",
    "replay",
    "allow-stale",
    "extract-only",
    "only",
    "manifest",
    "output",
    "markdown",
    "baseline",
    "write-baseline",
    "latency-gate",
    "pipeline",
    "id-style",
    "retrieval-tier",
    "context-tokens",
    "field-order",
    "string-limits",
];

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match dispatch(&arguments) {
        Ok(exit) => process::exit(exit),
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    }
}

fn dispatch(arguments: &[String]) -> Result<i32, String> {
    let Some((command, rest)) = arguments.split_first() else {
        return Err(USAGE.to_owned());
    };
    match command.as_str() {
        "run" => run(run_options(&parse(rest, RUN_KEYS)?)?),
        "compare" => {
            let values = parse(rest, &["before", "after", "markdown", "output"])?;
            compare_reports(
                &PathBuf::from(required(&values, "before")?),
                &PathBuf::from(required(&values, "after")?),
                values.get("markdown").map(PathBuf::from).as_deref(),
                values.get("output").map(PathBuf::from).as_deref(),
            )
        }
        "report" => {
            let values = parse(rest, &["input", "markdown"])?;
            render_report(
                &PathBuf::from(required(&values, "input")?),
                &PathBuf::from(required(&values, "markdown")?),
            )
        }
        "merge-recordings" => {
            let values = parse(rest, &["base", "add", "output", "gold", "only", "note"])?;
            intern_bench::merge::merge_command(
                &PathBuf::from(required(&values, "base")?),
                &PathBuf::from(required(&values, "add")?),
                &PathBuf::from(required(&values, "output")?),
                &values
                    .get("gold")
                    .map_or_else(|| PathBuf::from("bench/gold.json"), PathBuf::from),
                &id_list(values.get("only")),
                values.get("note").map(String::as_str),
            )
        }
        "retrieval" => {
            let values = parse(
                rest,
                &[
                    "recording",
                    "gold",
                    "fixtures",
                    "expected",
                    "corpus",
                    "worker",
                    "config",
                    "sweep",
                    "only",
                    "output",
                    "markdown",
                    "dump",
                ],
            )?;
            let paths = |key: &str| -> Vec<PathBuf> {
                id_list(values.get(key))
                    .into_iter()
                    .map(PathBuf::from)
                    .collect()
            };
            let fixtures = match (values.get("fixtures"), values.get("expected")) {
                (Some(recording), Some(expected)) => {
                    Some((PathBuf::from(recording), PathBuf::from(expected)))
                }
                (None, None) => None,
                _ => return Err("--fixtures and --expected go together".to_owned()),
            };
            let extract = match (values.get("corpus"), values.get("worker")) {
                (Some(corpus), Some(worker)) => {
                    Some((PathBuf::from(corpus), PathBuf::from(worker)))
                }
                (None, None) => None,
                _ => return Err("--corpus and --worker go together".to_owned()),
            };
            if !values.contains_key("recording") && fixtures.is_none() && extract.is_none() {
                return Err(format!(
                    "missing --recording, --fixtures or --corpus\n{USAGE}"
                ));
            }
            intern_bench::context::retrieval_command(&intern_bench::context::RetrievalOptions {
                recordings: paths("recording"),
                gold: values
                    .get("gold")
                    .map_or_else(|| PathBuf::from("bench/gold.json"), PathBuf::from),
                fixtures,
                extract,
                configs: paths("config"),
                sweep: values.contains_key("sweep"),
                only: id_list(values.get("only")),
                output: values.get("output").map(PathBuf::from),
                markdown: values.get("markdown").map(PathBuf::from),
                dump: values.get("dump").map(PathBuf::from),
            })
        }
        "--help" | "-h" | "help" => {
            println!("{USAGE}");
            Ok(0)
        }
        other => Err(format!("unknown command {other}\n{USAGE}")),
    }
}

fn run_options(values: &HashMap<String, String>) -> Result<RunOptions, String> {
    let path = |key: &str| values.get(key).map(PathBuf::from);
    let replay = values.get("replay");
    let extract_only = values.contains_key("extract-only");
    if replay.is_some() && extract_only {
        return Err("--replay and --extract-only are different runs; give one".to_owned());
    }
    let mode = if let Some(recording) = replay {
        refuse(
            values,
            &[
                "worker",
                "endpoint",
                "api-key",
                "record",
                "model-path",
                "no-warmup",
                "server-pid",
            ],
            "--replay",
        )?;
        Mode::Replay {
            recording: PathBuf::from(recording),
            allow_stale: values.contains_key("allow-stale"),
        }
    } else if extract_only {
        refuse(
            values,
            &[
                "endpoint",
                "api-key",
                "model-id",
                "model-path",
                "record",
                "note",
                "server-pid",
                "allow-stale",
                "pipeline",
                "context-tokens",
                "field-order",
                "string-limits",
            ],
            "--extract-only",
        )?;
        Mode::Extract {
            options: ExtractOptions {
                worker: PathBuf::from(required(values, "worker")?),
                warm_up: !values.contains_key("no-warmup"),
            },
        }
    } else {
        if values.contains_key("allow-stale") {
            return Err("--allow-stale is for --replay".to_owned());
        }
        Mode::Live {
            options: LiveOptions {
                worker: PathBuf::from(required(values, "worker")?),
                endpoint: required(values, "endpoint")?.to_owned(),
                api_key: required(values, "api-key")?.to_owned(),
                model_id: values
                    .get("model-id")
                    .cloned()
                    .unwrap_or_else(|| "intern-local".to_owned()),
                model_path: path("model-path"),
                warm_up: !values.contains_key("no-warmup"),
                server_pid: values
                    .get("server-pid")
                    .map(|value| {
                        value
                            .parse()
                            .map_err(|_| "--server-pid must be a process id".to_owned())
                    })
                    .transpose()?,
            },
            record: path("record"),
            note: values.get("note").cloned().unwrap_or_default(),
        }
    };
    Ok(RunOptions {
        corpus: PathBuf::from(required(values, "corpus")?),
        gold: PathBuf::from(required(values, "gold")?),
        manifest: path("manifest"),
        only: id_list(values.get("only")),
        output: path("output"),
        markdown: path("markdown"),
        baseline: path("baseline"),
        write_baseline: path("write-baseline"),
        latency_gate: values
            .get("latency-gate")
            .map(|value| {
                value
                    .parse::<f64>()
                    .ok()
                    .filter(|ratio| *ratio >= 1.0)
                    .ok_or_else(|| "--latency-gate must be a ratio of at least 1".to_owned())
            })
            .transpose()?,
        mode,
        engine: intern_bench::pipeline::EngineSettings::parse(values)?,
    })
}

/// Refuses options that belong to another kind of run.
fn refuse(values: &HashMap<String, String>, keys: &[&str], mode: &str) -> Result<(), String> {
    match keys.iter().find(|key| values.contains_key(**key)) {
        Some(key) => Err(format!("--{key} is not for {mode} runs")),
        None => Ok(()),
    }
}

/// `--only a,b`: the ids, blanks dropped.
fn id_list(list: Option<&String>) -> Vec<String> {
    list.map(|list| {
        list.split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
            .collect()
    })
    .unwrap_or_default()
}

fn parse(arguments: &[String], allowed: &[&str]) -> Result<HashMap<String, String>, String> {
    let mut values = HashMap::new();
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        let key = argument
            .strip_prefix("--")
            .ok_or_else(|| format!("unexpected argument: {argument}\n{USAGE}"))?;
        if !allowed.contains(&key) {
            return Err(format!("unknown option --{key}\n{USAGE}"));
        }
        if FLAGS.contains(&key) {
            values.insert(key.to_owned(), "true".to_owned());
            continue;
        }
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for --{key}"))?;
        values.insert(key.to_owned(), value.clone());
    }
    Ok(values)
}

fn required<'a>(values: &'a HashMap<String, String>, key: &str) -> Result<&'a str, String> {
    values
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing --{key}\n{USAGE}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(arguments: &[&str]) -> Result<RunOptions, String> {
        let arguments = arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect::<Vec<_>>();
        run_options(&parse(&arguments, RUN_KEYS)?)
    }

    #[test]
    fn extract_only_takes_a_worker_and_nothing_a_model_needs() {
        let base = ["--corpus", "c", "--gold", "g.json", "--extract-only"];
        let parsed = options(&[&base[..], &["--worker", "w", "--no-warmup"]].concat()).unwrap();
        let Mode::Extract { options } = parsed.mode else {
            panic!("not an extract-only run");
        };
        assert_eq!(options.worker, PathBuf::from("w"));
        assert!(!options.warm_up);

        assert!(options_error(&base).contains("missing --worker"));
        for (extra, refused) in [
            (
                &["--endpoint", "http://x"][..],
                "--endpoint is not for --extract-only runs",
            ),
            (
                &["--record", "r.json"][..],
                "--record is not for --extract-only runs",
            ),
            (
                &["--allow-stale"][..],
                "--allow-stale is not for --extract-only runs",
            ),
            (
                &["--replay", "r.json"][..],
                "--replay and --extract-only are different runs",
            ),
        ] {
            let error = options_error(&[&base[..], &["--worker", "w"], extra].concat());
            assert!(error.contains(refused), "{error}");
        }
    }

    fn options_error(arguments: &[&str]) -> String {
        match options(arguments) {
            Ok(_) => panic!("accepted {arguments:?}"),
            Err(error) => error,
        }
    }
}
