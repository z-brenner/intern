//! Where one document's time went, as a flat map of metrics.
//!
//! The map has every metric in [`METRICS`] for every document - `null`
//! where the run could not measure it - so two reports always have the same
//! keys and a diff shows only the numbers that moved. Times are
//! milliseconds with microsecond precision; counts are integers.
//!
//! The model's figures (prefill, generation, token counts) are the
//! server's own account of each request it answered, summed over the
//! document's requests. A request it did not answer reports nothing: a
//! prompt refused as too large counts toward `model_requests` and adds
//! nothing else, and the first attempt of a malformed reply the client
//! retried is not seen at all. Their time shows only in `analyze_wall_ms`
//! (and, for the retry, in `inference_ms`). The engine's telemetry, which
//! keeps only the request that answered, stands in when no request reported
//! (a recording made against a server that does not say).

use std::collections::BTreeMap;

use intern_engine::{AnalysisTelemetry, ExtractionTimings, ModelTimings};
use serde_json::{Value, json};

use crate::{recording::Exchange, stats::round};

/// How a metric is measured, which decides how it is printed and whether
/// the latency gate applies to it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unit {
    Milliseconds,
    Count,
    TokensPerSecond,
}

/// Every metric, in the order the report's stage table lists them.
pub const METRICS: &[(&str, Unit)] = &[
    ("total_ms", Unit::Milliseconds),
    ("extraction_wall_ms", Unit::Milliseconds),
    ("worker_total_ms", Unit::Milliseconds),
    ("worker_snapshot_ms", Unit::Milliseconds),
    ("worker_parse_ms", Unit::Milliseconds),
    ("worker_analysis_ms", Unit::Milliseconds),
    ("worker_render_ms", Unit::Milliseconds),
    ("worker_image_decode_ms", Unit::Milliseconds),
    ("worker_ocr_ms", Unit::Milliseconds),
    ("worker_ocr_encode_ms", Unit::Milliseconds),
    ("worker_ocr_engine_ms", Unit::Milliseconds),
    ("worker_vision_ms", Unit::Milliseconds),
    ("analyze_wall_ms", Unit::Milliseconds),
    ("distill_ms", Unit::Milliseconds),
    ("prompt_ms", Unit::Milliseconds),
    ("inference_ms", Unit::Milliseconds),
    ("prefill_ms", Unit::Milliseconds),
    ("generation_ms", Unit::Milliseconds),
    ("validation_ms", Unit::Milliseconds),
    ("naming_ms", Unit::Milliseconds),
    ("prefill_tok_per_s", Unit::TokensPerSecond),
    ("generation_tok_per_s", Unit::TokensPerSecond),
    ("prompt_tokens", Unit::Count),
    ("cached_tokens", Unit::Count),
    ("generated_tokens", Unit::Count),
    ("estimated_prompt_tokens", Unit::Count),
    ("model_requests", Unit::Count),
    ("redistillations", Unit::Count),
    ("source_characters", Unit::Count),
    ("digest_characters", Unit::Count),
    ("prompt_characters", Unit::Count),
    ("worker_ocr_pages", Unit::Count),
    ("worker_ocr_passes", Unit::Count),
    ("worker_orientation_passes", Unit::Count),
    ("worker_rendered_pixels", Unit::Count),
    ("index_ms", Unit::Milliseconds),
    ("retrieval_ms", Unit::Milliseconds),
    ("index_units", Unit::Count),
    ("context_units", Unit::Count),
    ("context_tokens", Unit::Count),
];

pub fn unit(metric: &str) -> Option<Unit> {
    METRICS
        .iter()
        .find(|(name, _)| *name == metric)
        .map(|(_, unit)| *unit)
}

/// One document's metrics.
pub type Timings = BTreeMap<String, Value>;

/// Every metric, unmeasured.
pub fn empty() -> Timings {
    METRICS
        .iter()
        .map(|(name, _)| ((*name).to_owned(), Value::Null))
        .collect()
}

/// The metric as a number, when it was measured.
pub fn get(timings: &Timings, metric: &str) -> Option<f64> {
    timings.get(metric).and_then(Value::as_f64)
}

/// What a run measured about one document, before it is flattened.
#[derive(Clone, Copy, Default)]
pub struct Measured<'a> {
    pub extraction_wall_micros: Option<u64>,
    /// What the worker said about its stages, when it said.
    pub worker: Option<&'a ExtractionTimings>,
    /// The engine's account, for a document it analysed.
    pub telemetry: Option<&'a AnalysisTelemetry>,
    pub analyze_wall_micros: Option<u64>,
    /// Every model request the engine made for the document.
    pub exchanges: &'a [Exchange],
}

pub fn flatten(measured: &Measured<'_>) -> Timings {
    let mut timings = empty();
    let mut millis = |key: &str, micros: Option<u64>| {
        if let Some(micros) = micros {
            timings.insert(key.to_owned(), json!(round(micros as f64 / 1000.0, 3)));
        }
    };
    millis("extraction_wall_ms", measured.extraction_wall_micros);
    millis("analyze_wall_ms", measured.analyze_wall_micros);
    if let (Some(extraction), Some(analysis)) = (
        measured.extraction_wall_micros,
        measured.analyze_wall_micros,
    ) {
        millis("total_ms", Some(extraction + analysis));
    }
    let worker = |read: fn(&ExtractionTimings) -> u64| measured.worker.map(read);
    millis("worker_total_ms", worker(|worker| worker.total_micros));
    millis(
        "worker_snapshot_ms",
        worker(|worker| worker.snapshot_micros),
    );
    millis("worker_parse_ms", worker(|worker| worker.parse_micros));
    millis(
        "worker_analysis_ms",
        worker(|worker| worker.analysis_micros),
    );
    millis("worker_render_ms", worker(|worker| worker.render_micros));
    millis(
        "worker_image_decode_ms",
        worker(|worker| worker.image_decode_micros),
    );
    millis("worker_ocr_ms", worker(|worker| worker.ocr_micros));
    millis(
        "worker_ocr_encode_ms",
        worker(|worker| worker.ocr_encode_micros),
    );
    millis(
        "worker_ocr_engine_ms",
        worker(|worker| worker.ocr_engine_micros),
    );
    millis("worker_vision_ms", worker(|worker| worker.vision_micros));
    let telemetry = |read: fn(&AnalysisTelemetry) -> u64| measured.telemetry.map(read);
    millis(
        "distill_ms",
        telemetry(|telemetry| telemetry.distill_micros),
    );
    millis("prompt_ms", telemetry(|telemetry| telemetry.prompt_micros));
    millis(
        "validation_ms",
        telemetry(|telemetry| telemetry.validation_micros),
    );
    millis("naming_ms", telemetry(|telemetry| telemetry.naming_micros));
    millis(
        "inference_ms",
        telemetry(|telemetry| telemetry.inference_millis.saturating_mul(1000)),
    );

    let per_request = measured
        .exchanges
        .iter()
        .filter_map(|exchange| exchange.model_timings.as_ref())
        .collect::<Vec<_>>();
    let reported = if per_request.is_empty() {
        measured
            .telemetry
            .and_then(|telemetry| telemetry.model.as_ref())
            .into_iter()
            .collect()
    } else {
        per_request
    };
    let sum = |read: fn(&ModelTimings) -> u64| {
        (!reported.is_empty()).then(|| reported.iter().map(|timings| read(timings)).sum::<u64>())
    };
    let prefill = sum(|timings| timings.prefill_micros);
    let generation = sum(|timings| timings.generation_micros);
    millis("prefill_ms", prefill);
    millis("generation_ms", generation);

    let mut count = |key: &str, value: Option<u64>| {
        if let Some(value) = value {
            timings.insert(key.to_owned(), json!(value));
        }
    };
    let prompt_tokens = sum(|timings| timings.prompt_tokens);
    let generated_tokens = sum(|timings| timings.generated_tokens);
    count("prompt_tokens", prompt_tokens);
    count("cached_tokens", sum(|timings| timings.cached_tokens));
    count("generated_tokens", generated_tokens);
    count(
        "worker_ocr_pages",
        worker(|worker| u64::from(worker.ocr_pages)),
    );
    count(
        "worker_ocr_passes",
        worker(|worker| u64::from(worker.ocr_passes)),
    );
    count(
        "worker_orientation_passes",
        worker(|worker| u64::from(worker.orientation_passes)),
    );
    count(
        "worker_rendered_pixels",
        worker(|worker| worker.rendered_pixels),
    );
    count(
        "redistillations",
        telemetry(|telemetry| u64::from(telemetry.redistillations)),
    );
    count(
        "source_characters",
        telemetry(|telemetry| telemetry.source_characters as u64),
    );
    count(
        "digest_characters",
        telemetry(|telemetry| telemetry.digest_characters as u64),
    );
    count(
        "prompt_characters",
        telemetry(|telemetry| telemetry.prompt_characters as u64),
    );
    count(
        "estimated_prompt_tokens",
        telemetry(|telemetry| telemetry.estimated_prompt_tokens as u64),
    );
    if !measured.exchanges.is_empty() {
        count("model_requests", Some(measured.exchanges.len() as u64));
    }

    let mut rate = |key: &str, tokens: Option<u64>, micros: Option<u64>| {
        if let (Some(tokens), Some(micros)) = (tokens, micros)
            && micros > 0
        {
            timings.insert(
                key.to_owned(),
                json!(round(tokens as f64 / (micros as f64 / 1e6), 1)),
            );
        }
    };
    rate("prefill_tok_per_s", prompt_tokens, prefill);
    rate("generation_tok_per_s", generated_tokens, generation);
    timings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recording::RecordedReply;
    use intern_engine::EngineErrorCode;

    /// A prompt the server refused as too large: no reply, so no timings.
    fn refused() -> Exchange {
        Exchange {
            prompt_sha256: "0".repeat(64),
            prompt_characters: 9_000,
            reply: RecordedReply::Failed {
                code: EngineErrorCode::ModelInputTooLarge,
            },
            model_timings: None,
            wall_micros: 1_000,
        }
    }

    /// A request the server answered, with its account of it.
    fn answered(timings: ModelTimings) -> Exchange {
        Exchange {
            prompt_sha256: "1".repeat(64),
            prompt_characters: 4_000,
            reply: RecordedReply::Proposed {
                proposal: intern_engine::ModelProposal {
                    document_type: None,
                    document_date: None,
                    date_role: None,
                    parties: Vec::new(),
                    party_relation: intern_engine::PartyRelation::None,
                    description: String::new(),
                    confidence: 0.5,
                    needs_review: false,
                    evidence: intern_engine::Evidence::default(),
                },
                token_confidence: None,
            },
            model_timings: Some(timings),
            wall_micros: 3_000_000,
        }
    }

    #[test]
    fn every_metric_is_present_and_unmeasured_ones_are_null() {
        let timings = flatten(&Measured::default());
        assert_eq!(timings.len(), METRICS.len());
        assert!(timings.values().all(Value::is_null));
    }

    #[test]
    fn worker_and_engine_figures_are_flattened_and_model_work_is_summed() {
        let worker = ExtractionTimings {
            total_micros: 1_500_000,
            snapshot_micros: 2_000,
            parse_micros: 300_000,
            ocr_micros: 1_100_000,
            ocr_pages: 2,
            ocr_passes: 3,
            orientation_passes: 2,
            rendered_pixels: 16_000_000,
            ..ExtractionTimings::default()
        };
        let telemetry = AnalysisTelemetry {
            source_characters: 9_000,
            digest_characters: 7_000,
            distill_micros: 1_234,
            inference_millis: 2_100,
            prompt_micros: 80,
            redistillations: 1,
            prompt_characters: 7_400,
            model: Some(ModelTimings {
                prompt_tokens: 999,
                ..ModelTimings::default()
            }),
            ..AnalysisTelemetry::default()
        };
        // Refused as too large, then condensed and answered: only the
        // answer carries the server's figures.
        let exchanges = [
            refused(),
            answered(ModelTimings {
                prompt_tokens: 1_000,
                cached_tokens: 600,
                prefill_micros: 1_000_000,
                generated_tokens: 100,
                generation_micros: 2_000_000,
            }),
        ];
        let timings = flatten(&Measured {
            extraction_wall_micros: Some(1_600_000),
            worker: Some(&worker),
            telemetry: Some(&telemetry),
            analyze_wall_micros: Some(2_500_000),
            exchanges: &exchanges,
        });
        assert_eq!(timings["total_ms"], json!(4_100.0));
        assert_eq!(timings["worker_ocr_ms"], json!(1_100.0));
        assert_eq!(timings["worker_render_ms"], json!(0.0), "reported as none");
        assert_eq!(timings["worker_ocr_passes"], json!(3));
        assert_eq!(timings["distill_ms"], json!(1.234));
        assert_eq!(timings["inference_ms"], json!(2_100.0));
        assert_eq!(timings["prompt_ms"], json!(0.08));
        assert_eq!(
            timings["prompt_tokens"],
            json!(1_000),
            "the answering request's, not the telemetry's"
        );
        assert_eq!(timings["cached_tokens"], json!(600));
        assert_eq!(timings["prefill_ms"], json!(1_000.0));
        assert_eq!(timings["prefill_tok_per_s"], json!(1_000.0));
        assert_eq!(timings["generation_tok_per_s"], json!(50.0));
        assert_eq!(timings["model_requests"], json!(2), "the refusal counts");

        // Every answered request is summed.
        let both = [
            answered(ModelTimings {
                prompt_tokens: 3_000,
                cached_tokens: 0,
                prefill_micros: 1_000_000,
                generated_tokens: 50,
                generation_micros: 1_000_000,
            }),
            exchanges[1].clone(),
        ];
        let summed = flatten(&Measured {
            exchanges: &both,
            ..Measured::default()
        });
        assert_eq!(summed["prompt_tokens"], json!(4_000));
        assert_eq!(summed["generated_tokens"], json!(150));
        assert_eq!(timings["prompt_characters"], json!(7_400));
        assert_eq!(timings["redistillations"], json!(1));

        // Without per-request timings the telemetry's answer stands in; a
        // worker that reported nothing leaves its stages unmeasured.
        let timings = flatten(&Measured {
            telemetry: Some(&telemetry),
            ..Measured::default()
        });
        assert_eq!(timings["prompt_tokens"], json!(999));
        assert_eq!(timings["worker_total_ms"], Value::Null);
        assert_eq!(timings["total_ms"], Value::Null);
    }
}
