//! The installed model, started the way the app starts it and asked what
//! setup asks it.
//!
//! Runs only when `INTERN_LIVE_LLAMA_SERVER` names a llama-server binary (the
//! Prism llama.cpp build `runtime-assets.json` pins, or its Linux release) and
//! `INTERN_LIVE_MODEL` one of the GGUF files `model-manifest.json` pins;
//! otherwise it says so and passes. CI has neither. It is how the semantic
//! self-test - the probe every setup runs before Intern names a document - is
//! checked against each variant before a release:
//!
//! ```text
//! INTERN_LIVE_LLAMA_SERVER=/path/to/llama-server \
//! INTERN_LIVE_MODEL=/path/to/Ternary-Bonsai-2-4B-PQ2_0.gguf \
//!   cargo test --locked -p intern-engine --test live_model -- --nocapture
//! ```

use std::path::PathBuf;
use std::time::Instant;

use intern_engine::{
    Engine, LlamaServer, ModelClient, ModelManifest, ServerOptions,
    setup::{semantic_probes, validate_semantic_probe},
};

fn live() -> Option<(PathBuf, PathBuf)> {
    match (
        std::env::var_os("INTERN_LIVE_LLAMA_SERVER"),
        std::env::var_os("INTERN_LIVE_MODEL"),
    ) {
        (Some(server), Some(model)) => Some((server.into(), model.into())),
        _ => {
            eprintln!("skipped: INTERN_LIVE_LLAMA_SERVER and INTERN_LIVE_MODEL are not both set");
            None
        }
    }
}

#[test]
fn the_pinned_model_passes_the_setup_self_test() {
    let Some((server, model)) = live() else {
        return;
    };
    let manifest = ModelManifest::embedded().unwrap();
    let size = std::fs::metadata(&model).unwrap().len();
    let variant = manifest
        .variant_with_size(size)
        .expect("INTERN_LIVE_MODEL is not the size of any pinned variant");
    eprintln!(
        "variant {} ({}), {size} bytes",
        variant.variant, variant.model_id
    );

    let started = Instant::now();
    let server = LlamaServer::start(&server, &model, None, &ServerOptions::default()).unwrap();
    eprintln!("server healthy after {:.1} s", started.elapsed().as_secs_f64());
    let client = ModelClient::new(
        &server.completion_endpoint(),
        server.api_key().to_owned(),
        variant.served_model_name.clone(),
    )
    .unwrap();
    let engine = Engine::new(client);
    let mut failures = Vec::new();
    for probe in semantic_probes().unwrap() {
        let started = Instant::now();
        let analysis = engine.analyze(&probe.document, "pdf", &[]).unwrap();
        eprintln!(
            "probe answered in {:.1} s: {} ({:?})",
            started.elapsed().as_secs_f64(),
            analysis.proposal.filename,
            analysis.proposal.document_date,
        );
        if validate_semantic_probe(&probe, &analysis).is_err() {
            failures.push(analysis.proposal.filename.clone());
        }
    }
    server.stop().unwrap();
    assert!(
        failures.is_empty(),
        "self-test failed for {}: {failures:?}",
        variant.variant
    );
}
