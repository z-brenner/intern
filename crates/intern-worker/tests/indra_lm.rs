//! Indra's language model, beam search and vocabularies against the
//! reference package (`indra-ocr` 1.1.2, models `a1283c4b`).
//!
//! `fixtures/indra/lm/beam_cases.json` holds every beam-search case of the
//! reference's golden vectors (68 lines the pipeline searched on the
//! reference images) and 70 more replayed from the dumped recognizer
//! outputs at other settings, with their results; the 80 `logp_keys`
//! queries a model of the golden vectors; and, for each of the three
//! models, the table entries those cases and queries touch. A model built
//! from just those entries (and the full model's `max_logp`) answers every
//! one of those lookups as the full model does, so the cases run in CI
//! without the 15-19 MB models. `vocab_cases.json` holds `Vocab.decode`
//! goldens. Both are written by the scripts beside them.
//!
//! With `INDRA_MODEL_DIR` set to a directory holding the pinned `lm.npz`,
//! `lm_cjk.npz`, `lm_world.npz` and `meta.json`, the same cases also run
//! against the full models, and the load and search times are printed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use intern_worker::indra::beam::{BeamParams, beam_search};
use intern_worker::indra::lm::{LmTable, NgramLm};
use intern_worker::indra::meta::Meta;
use intern_worker::indra::vocab::Vocab;
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/indra/lm")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    STANDARD.decode(value.as_str().unwrap()).unwrap()
}

fn u64s(value: &Value) -> Vec<u64> {
    bytes(value)
        .chunks_exact(8)
        .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn f32s(value: &Value) -> Vec<f32> {
    bytes(value)
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn f64s(value: &Value) -> Vec<f64> {
    bytes(value)
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn classes(value: &Value) -> Vec<i64> {
    bytes(value)
        .chunks_exact(2)
        .map(|c| i64::from(u16::from_le_bytes(c.try_into().unwrap())))
        .collect()
}

fn f64_bits(value: &Value) -> f64 {
    f64::from_bits(value.as_u64().unwrap())
}

fn usize_of(value: &Value) -> usize {
    value.as_u64().unwrap() as usize
}

/// The fixture's reduced models, by name (`lm`, `lm_cjk`, `lm_world`).
fn reduced_models(doc: &Value) -> HashMap<String, NgramLm> {
    doc["lms"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, m)| {
            let tables = m["tables"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| LmTable {
                    keys: u64s(&t["k"]),
                    logp: f32s(&t["v"]),
                    histories: u64s(&t["hk"]),
                    backoff: f32s(&t["hb"]),
                })
                .collect();
            let mut lm = NgramLm::from_tables(
                usize_of(&m["order"]),
                usize_of(&m["vocab_size"]),
                m["bits"].as_u64().unwrap() as u32,
                tables,
            )
            .unwrap();
            lm.max_logp = f64_bits(&m["max_logp_bits"]);
            (name.clone(), lm)
        })
        .collect()
}

/// The full models of `INDRA_MODEL_DIR`, or nothing (with a note) when it
/// is not set.
fn model_dir() -> Option<PathBuf> {
    match std::env::var_os("INDRA_MODEL_DIR") {
        Some(dir) => Some(PathBuf::from(dir)),
        None => {
            eprintln!("INDRA_MODEL_DIR is not set: skipping the full-model checks");
            None
        }
    }
}

fn full_models(dir: &Path) -> HashMap<String, NgramLm> {
    ["lm", "lm_cjk", "lm_world"]
        .iter()
        .map(|name| {
            let path = dir.join(format!("{name}.npz"));
            let started = Instant::now();
            let lm = NgramLm::load(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            eprintln!(
                "{name}.npz: order {}, {} bits, vocab {}, loaded in {:.1} ms",
                lm.order,
                lm.bits,
                lm.vocab_size,
                started.elapsed().as_secs_f64() * 1e3
            );
            (name.to_string(), lm)
        })
        .collect()
}

/// Runs every case under `models`; returns the number of cases and the
/// total search time in seconds.
fn check_cases(doc: &Value, models: &HashMap<String, NgramLm>, label: &str) -> (usize, f64) {
    let cases = doc["cases"].as_array().unwrap();
    let mut seconds = 0.0;
    let mut failures = Vec::new();
    for case in cases {
        let lm = &models[case["lm"].as_str().unwrap()];
        let params = BeamParams {
            alpha: f64_bits(&case["alpha_bits"]),
            beta: f64_bits(&case["beta_bits"]),
            beam: usize_of(&case["beam"]),
            cand_floor: f64_bits(&case["cand_floor_bits"]),
            ..BeamParams::default()
        };
        let blank = f32s(&case["blank"]);
        let topv = f32s(&case["topv"]);
        let topi = classes(&case["topi"]);
        let started = Instant::now();
        let (ids, steps) = beam_search(&blank, &topv, &topi, usize_of(&case["k"]), lm, params);
        seconds += started.elapsed().as_secs_f64();
        let want_ids: Vec<u32> = case["ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        let want_steps: Vec<usize> = case["steps"]
            .as_array()
            .unwrap()
            .iter()
            .map(usize_of)
            .collect();
        if ids != want_ids || steps != want_steps {
            failures.push(case["source"].as_str().unwrap().to_owned());
        }
    }
    assert!(failures.is_empty(), "{label}: cases differ: {failures:?}");
    (cases.len(), seconds)
}

/// Checks every golden `logp_keys` query; returns the largest difference.
fn check_logp(doc: &Value, models: &HashMap<String, NgramLm>) -> (usize, f64) {
    let mut count = 0;
    let mut worst = 0.0f64;
    for (name, queries) in doc["logp"].as_object().unwrap() {
        let lm = &models[name.as_str()];
        let keys = u64s(&queries["keys"]);
        let want = f64s(&queries["logp"]);
        assert_eq!(keys.len(), want.len());
        for (&key, &want) in keys.iter().zip(&want) {
            let got = lm.logp(key);
            assert!(
                (got - want).abs() <= 1e-9,
                "{name} key {key}: {got} != {want}"
            );
            worst = worst.max((got - want).abs());
            count += 1;
        }
    }
    (count, worst)
}

#[test]
fn beam_search_and_back_off_match_the_reference() {
    let doc = fixture("beam_cases.json");
    let models = reduced_models(&doc);
    let (cases, _) = check_cases(&doc, &models, "reduced models");
    assert_eq!(cases, 138);
    let golden = doc["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["source"].as_str().unwrap().starts_with("golden#"))
        .count();
    assert_eq!(golden, 68);
    let (queries, worst) = check_logp(&doc, &models);
    assert_eq!(queries, 240);
    eprintln!(
        "{cases} beam cases and {queries} logp queries match (largest logp difference {worst:e})"
    );
    // The model maximum the reference reports, from the full tables.
    assert_eq!(models["lm"].max_logp, -3.488574984444881e-10);
}

#[test]
fn full_models_match_the_reference() {
    let Some(dir) = model_dir() else {
        return;
    };
    let doc = fixture("beam_cases.json");
    let models = full_models(&dir);
    for (name, m) in doc["lms"].as_object().unwrap() {
        let lm = &models[name.as_str()];
        assert_eq!(lm.order, usize_of(&m["order"]), "{name}");
        assert_eq!(lm.bits as u64, m["bits"].as_u64().unwrap(), "{name}");
        assert_eq!(lm.vocab_size, usize_of(&m["vocab_size"]), "{name}");
        assert_eq!(lm.max_logp, f64_bits(&m["max_logp_bits"]), "{name}");
    }
    let (queries, worst) = check_logp(&doc, &models);
    let (cases, seconds) = check_cases(&doc, &models, "full models");
    let steps: usize = doc["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| f32s(&c["blank"]).len())
        .sum();
    eprintln!(
        "full models: {queries} logp queries (largest difference {worst:e}) and {cases} beam cases \
         match; search {:.3} ms a line, {:.1} us a step",
        seconds * 1e3 / cases as f64,
        seconds * 1e6 / steps as f64
    );
}

fn vocab_from(chars: &Value) -> Vocab {
    Vocab::new(
        chars
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_owned())
            .collect(),
    )
}

fn ids_of(case: &Value) -> Vec<u32> {
    case["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect()
}

#[test]
fn decode_matches_the_reference() {
    let doc = fixture("vocab_cases.json");
    let vocabs: HashMap<&str, Vocab> = doc["vocabs"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, chars)| (name.as_str(), vocab_from(chars)))
        .collect();
    for (name, flags) in doc["flags"].as_object().unwrap() {
        let v = &vocabs[name.as_str()];
        assert_eq!(v.jamo, flags["jamo"].as_bool().unwrap(), "{name}");
        assert_eq!(
            v.has_commands,
            flags["has_commands"].as_bool().unwrap(),
            "{name}"
        );
    }
    let cases = doc["cases"].as_array().unwrap();
    for case in cases {
        let v = &vocabs[case["vocab"].as_str().unwrap()];
        assert_eq!(
            v.decode(&ids_of(case)),
            case["text"].as_str().unwrap(),
            "{case}"
        );
    }
    // Python's isalpha on every base token's first character, and on every
    // token's tail: a command before the token, and the token before "x".
    let base = &vocabs["vocab"];
    let id = |token: &str| base.chars.iter().position(|c| c == token).unwrap() as u32 + 1;
    let (alpha, x) = (id("\\alpha"), id("x"));
    let first = doc["alpha_first"].as_str().unwrap().as_bytes();
    let tail = doc["alpha_tail"].as_str().unwrap().as_bytes();
    assert_eq!(first.len(), base.chars.len());
    for (index, token) in base.chars.iter().enumerate() {
        let token_id = index as u32 + 1;
        let spaced = base.decode(&[alpha, token_id]) == format!("\\alpha {token}");
        assert_eq!(spaced, first[index] == b'1', "first character of {token:?}");
        if token.starts_with('\\') && token.chars().count() > 2 {
            let spaced = base.decode(&[token_id, x]) == format!("{token} x");
            assert_eq!(spaced, tail[index] == b'1', "tail of {token:?}");
        }
    }
    eprintln!(
        "{} decode cases and {} tokens match",
        cases.len(),
        base.chars.len()
    );
}

#[test]
fn meta_and_full_vocabularies_match_the_reference() {
    let Some(dir) = model_dir() else {
        return;
    };
    let meta = Meta::load(&dir.join("meta.json")).unwrap();
    assert_eq!(meta.det_long_side, 1024);
    assert_eq!(meta.rec_height, 32);
    assert_eq!(meta.rec_window, 3);
    assert_eq!(meta.vocab.chars.len(), 1697);
    assert!(meta.vocab.has_commands && !meta.vocab.jamo);
    assert_eq!(meta.groups, ["world", "rtl", "cjk"]);
    assert_eq!(
        meta.script_groups.as_deref().unwrap(),
        ["base", "cjk", "rtl", "world"]
    );
    assert_eq!(meta.lm_settings("base"), (0.5, 1.5, 0.9));
    assert_eq!(meta.lm_settings("cjk"), (0.2, 1.5, 0.95));
    assert_eq!(meta.lm_settings("world"), (0.2, 1.5, 0.95));
    let world = meta.vocab_world.as_ref().unwrap();
    let rtl = meta.vocab_rtl.as_ref().unwrap();
    let cjk = meta.vocab_cjk.as_ref().unwrap();
    assert_eq!(
        (world.chars.len(), rtl.chars.len(), cjk.chars.len()),
        (2935, 1355, 11749)
    );
    assert!(cjk.jamo && !world.jamo && !rtl.jamo);
    let doc = fixture("vocab_cases.json");
    let cases = doc["full_cases"].as_array().unwrap();
    for case in cases {
        let v = match case["vocab"].as_str().unwrap() {
            "vocab" => &meta.vocab,
            "vocab_world" => world,
            "vocab_rtl" => rtl,
            "vocab_cjk" => cjk,
            other => panic!("{other}"),
        };
        assert_eq!(
            v.decode(&ids_of(case)),
            case["text"].as_str().unwrap(),
            "{case}"
        );
    }
    eprintln!(
        "meta.json and {} full-vocabulary decode cases match",
        cases.len()
    );
}
