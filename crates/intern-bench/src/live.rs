//! A live run: the real worker, the real engine, the local model.
//!
//! Each document goes the way it goes in the app - the supervised parser
//! worker extracts it, [`Engine::analyze`] distills it, fits the prompt to
//! the model's context, asks the model once (or again, condensed, if the
//! server says the prompt did not fit), validates the reply, and names the
//! document - with a proposer in front of the model client that records
//! every request and its reply for replay.
//!
//! Before the corpus a synthetic document goes through the worker and the
//! engine and is thrown away, so the first scored document is not charged
//! for starting the worker or for the server's empty prompt cache, and every
//! scored document reuses the instructions as a restored app does
//! ([`WARM_UP_TEXT`]).

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use intern_engine::{
    DigestBudget, DocumentSource, Engine, EngineResult, ExtractFailure, ExtractionTimings,
    ModelClient, ModelProposal, ModelRequest, Proposer, ProposerReply, SupervisedWorker,
    TokenConfidence,
};

use crate::{
    gold::GoldDocument,
    machine::ModelInfo,
    memory::MemorySampler,
    pipeline::EngineSettings,
    record::{
        COMPLETED, DocumentRecord, EXTRACTION_FAILED, MODEL_FAILED, Observation, scored_record,
    },
    recording::{
        Exchange, ExchangeLog, RecordedDocument, RecordedExtraction, RecordedReply, sha256_hex,
        without_image_bytes,
    },
    timing::{Measured, flatten},
};

pub struct LiveOptions {
    pub worker: PathBuf,
    #[cfg(feature = "benchmark-transport")]
    pub benchmark_transport: Option<(PathBuf, PathBuf)>,
    pub endpoint: String,
    pub api_key: String,
    pub model_id: String,
    pub model_path: Option<PathBuf>,
    pub warm_up: bool,
    /// The model server's process id, for sampling its memory exactly.
    pub server_pid: Option<u32>,
}

/// What a live run produced: a record and a recording entry per document.
pub struct LiveRun {
    pub records: Vec<DocumentRecord>,
    pub recorded: Vec<RecordedDocument>,
    pub model: ModelInfo,
    pub context_tokens: Option<usize>,
}

/// A fictional notice, read as a document and thrown away.
///
/// Long enough that the server processes the instructions in one piece: a
/// first request shorter than the instructions plus 4 + `-ub` tokens is
/// broken inside them, and every later document reuses instructions
/// computed that way, which can change a reply
/// (`intern_engine::slots::SlotKeeper::save_once`). The app restores the
/// instructions as a long first document computes them, so this is the
/// state its replies are measured in. About 1,050 tokens with the
/// instructions on Qwen3.5's tokenizer (counted with `llama-tokenize`),
/// past the 737 the app's `-ub 256` needs and the 993 of llama.cpp's
/// default 512. Through alpha.16 it was a four-sentence notice of 637
/// tokens, and every live run recorded until then was made with the
/// instructions in pieces (docs/architecture.md).
const WARM_UP_TEXT: &str = "NOTICE OF OFFICE RELOCATION\n\n\
    Dated May 4, 2026\n\n\
    Brindlemoor Survey Partners LLP informs its clients that from June 1, 2026 its \
    office moves from 4 Quarry Lane to 12 Larkspur Row, Easthaven. Telephone numbers \
    and the client portal are unchanged. Correspondence sent to the old address will \
    be forwarded until August 31, 2026.\n\n\
    The new office is on the second floor of the Larkspur Row building, above the \
    public library. The entrance is on the east side of the building, facing the \
    river walk, and a lift serves every floor. Visitors should sign in at the ground \
    floor reception desk, which is staffed from 8:30 a.m. to 5:30 p.m. on weekdays.\n\n\
    Parking is available in the municipal car park on Weir Street, a short walk from \
    the office. The firm will validate parking for clients attending meetings of up \
    to three hours. Bicycle racks are provided at the rear of the building, and the \
    number 14 and number 22 buses stop on Larkspur Row.\n\n\
    The office will be closed on Friday, May 29, 2026 while files and equipment are \
    moved. Meetings already arranged for that day will be held by video call, and \
    each client affected will receive the details by email. Site visits and field \
    surveys continue as scheduled throughout the move.\n\n\
    Original documents, title deeds and survey records held for clients will be \
    moved under the supervision of the records manager and remain in locked storage \
    at all times. Clients who wish to collect original documents before the move are \
    asked to arrange a time with their surveyor by May 22, 2026.\n\n\
    Deliveries of plans, drawings and signed instructions may be left at the Quarry \
    Lane reception until May 28, 2026, and at the Larkspur Row reception from June \
    1, 2026. Courier accounts and invoice references are unchanged, and invoices \
    issued before the move remain payable in the usual way.\n\n\
    Questions about the relocation may be directed to the office manager on the \
    usual telephone number or through the client portal. The firm thanks its clients \
    for their patience during the move and looks forward to welcoming them at \
    Larkspur Row.\n\n\
    Orla Penhallow, Managing Partner";

pub fn run(
    documents: &[GoldDocument],
    corpus: &Path,
    options: &LiveOptions,
    settings: &EngineSettings,
) -> Result<LiveRun, String> {
    let model = ModelInfo::with_file(&options.model_id, options.model_path.as_deref())?;
    let client = ModelClient::new(
        &options.endpoint,
        options.api_key.clone(),
        &options.model_id,
    )
    .map_err(|error| format!("model client: {error}"))?;
    let client = match settings.context_tokens {
        Some(tokens) => client.with_context_tokens(tokens),
        None => client,
    };
    // Ask the server for each chosen token's probability (no generated token
    // changes) so the record carries `token_confidence`: always when the run
    // is scored with a token-confidence gate, as the app asks for the
    // accurate model; otherwise as a laboratory opt-in.
    let client = client.with_token_confidence(
        settings.min_token_confidence.is_some()
            || std::env::var_os("INTERN_BENCH_TOKEN_CONFIDENCE").is_some_and(|value| value == "1"),
    );
    // A laboratory opt-in: keep every generated token with its probability
    // and the two tokens the model found most probable, so a recording can
    // be studied and replayed token by token. No generated token changes.
    let client = client.with_token_trace(
        std::env::var_os("INTERN_BENCH_TOKEN_TRACE").is_some_and(|value| value == "1"),
    );
    #[cfg(feature = "benchmark-transport")]
    let client = match &options.benchmark_transport {
        Some((profile, output)) => client
            .with_benchmark_transport(
                intern_engine::benchmark_transport::BenchmarkTransport::load(profile, output)?,
            )
            .map_err(|e| format!("benchmark transport: {e}"))?,
        None => client,
    };
    let log = ExchangeLog::default();
    let proposer = RecordingProposer {
        inner: client,
        log: log.clone(),
    };
    let context_tokens = proposer.context_tokens();
    let budget = DigestBudget::default();
    let engine = settings.configure(Engine::with_proposer(Box::new(proposer)).with_budget(budget));
    let worker = SupervisedWorker::new(&options.worker);

    if options.warm_up {
        warm_up(&worker, &engine)?;
        log.take();
    }

    let mut records = Vec::new();
    let mut recorded = Vec::new();
    for (index, document) in documents.iter().enumerate() {
        eprintln!("[{}/{}] {}", index + 1, documents.len(), document.id);
        let path = corpus.join(&document.file);
        let bytes = std::fs::read(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let sha256 = sha256_hex(&bytes);
        drop(bytes);

        let sampler = MemorySampler::start(options.server_pid);
        let extraction_started = Instant::now();
        let (extracted, worker_timings) =
            extract(&worker, &format!("bench-{}", document.id), &path);
        let extraction_wall_micros = micros_since(extraction_started);

        let (record, entry) = match extracted {
            Err(failure) => {
                let memory = sampler.finish();
                let timings = flatten(&Measured {
                    extraction_wall_micros: Some(extraction_wall_micros),
                    worker: worker_timings.as_ref(),
                    ..Measured::default()
                });
                let record = scored_record(
                    document,
                    Observation {
                        status: EXTRACTION_FAILED,
                        error: Some(failure.code.clone()),
                        analysis: None,
                        source: None,
                        budget,
                        retrieval: &settings.retrieval,
                        exchanges: &[],
                        last_prompt: None,
                        timings: timings.clone(),
                        timings_recorded: false,
                        memory,
                        replayed: false,
                        stale: false,
                    },
                );
                let entry = RecordedDocument {
                    id: document.id.clone(),
                    file: document.file.clone(),
                    sha256: Some(sha256),
                    extraction: RecordedExtraction::Failed { code: failure.code },
                    exchanges: Vec::new(),
                    timings,
                    memory,
                };
                (record, entry)
            }
            Ok(source) => {
                let analyze_started = Instant::now();
                let result = engine.analyze(&source, document.extension(), &[]);
                let analyze_wall_micros = micros_since(analyze_started);
                let (exchanges, last_prompt) = log.take();
                let timings = flatten(&Measured {
                    extraction_wall_micros: Some(extraction_wall_micros),
                    worker: worker_timings.as_ref(),
                    telemetry: result.as_ref().ok().map(|analysis| &analysis.telemetry),
                    analyze_wall_micros: Some(analyze_wall_micros),
                    exchanges: &exchanges,
                });
                let (status, error, analysis) = match &result {
                    Ok(analysis) => (COMPLETED, None, Some(analysis)),
                    Err(error) => (MODEL_FAILED, Some(error.code().as_str().to_owned()), None),
                };
                // Scoring is part of the document's work for the sampler:
                // the claim checks over a hundred pages are not free.
                let mut record = scored_record(
                    document,
                    Observation {
                        status,
                        error,
                        analysis,
                        source: Some(&source),
                        budget,
                        retrieval: &settings.retrieval,
                        exchanges: &exchanges,
                        last_prompt: last_prompt.as_deref(),
                        timings: timings.clone(),
                        timings_recorded: false,
                        memory: Default::default(),
                        replayed: false,
                        stale: false,
                    },
                );
                let memory = sampler.finish();
                record.memory = memory;
                let entry = RecordedDocument {
                    id: document.id.clone(),
                    file: document.file.clone(),
                    sha256: Some(sha256),
                    extraction: RecordedExtraction::Parsed {
                        source: without_image_bytes(source),
                    },
                    exchanges,
                    timings,
                    memory,
                };
                (record, entry)
            }
        };
        if let Some(name) = &record.filename {
            eprintln!("    {} -> {name}", record.status);
        } else {
            eprintln!(
                "    {} {}",
                record.status,
                record.error.as_deref().unwrap_or_default()
            );
        }
        records.push(record);
        recorded.push(entry);
    }
    worker.stop();
    Ok(LiveRun {
        records,
        recorded,
        model,
        context_tokens,
    })
}

/// Starts the worker and fills the server's prompt cache with the system
/// instruction, on a document nobody scores, and says how long its prompt
/// was: the instructions are processed in one piece from 4 + `-ub` tokens
/// past them ([`WARM_UP_TEXT`]).
fn warm_up(worker: &SupervisedWorker, engine: &Engine) -> Result<(), String> {
    let started = Instant::now();
    let source = warm_up_worker(worker)?;
    let analysis = engine
        .analyze(&source, "txt", &[])
        .map_err(|error| format!("the model did not answer the warm-up document: {error}"))?;
    match analysis.telemetry.model {
        Some(timings) => eprintln!(
            "warmed up in {:.1} s: {} prompt tokens processed, {} cached",
            started.elapsed().as_secs_f64(),
            timings.prompt_tokens,
            timings.cached_tokens
        ),
        None => eprintln!("warmed up in {:.1} s", started.elapsed().as_secs_f64()),
    }
    Ok(())
}

/// Starts the worker on a text document nobody scores, so the first scored
/// document is not charged for the start.
pub(crate) fn warm_up_worker(worker: &SupervisedWorker) -> Result<DocumentSource, String> {
    let directory =
        std::env::temp_dir().join(format!("intern-bench-warm-up-{}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    let path = directory.join("warm-up.txt");
    std::fs::write(&path, WARM_UP_TEXT)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    let (extracted, _) = extract(worker, "bench-warm-up", &path);
    let _ = std::fs::remove_dir_all(&directory);
    extracted.map_err(|failure| {
        format!(
            "the worker could not read the warm-up document: {}",
            failure.code
        )
    })
}

/// Extracts a document, with the worker's account of its stages when the
/// worker gives one. A failed extraction reports none.
pub(crate) fn extract(
    worker: &SupervisedWorker,
    request_id: &str,
    path: &Path,
) -> (
    Result<DocumentSource, ExtractFailure>,
    Option<ExtractionTimings>,
) {
    match worker.extract_timed(request_id, path, &mut |_| {}) {
        Ok((source, timings)) => (Ok(source), timings),
        Err(failure) => (Err(failure), None),
    }
}

pub(crate) fn micros_since(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

/// The model client, with every request, its reply, and the server's
/// account of it written to a log. Every way the engine can ask goes
/// through [`Proposer::propose_measured`], so the client's own retry of a
/// malformed reply is kept, and the timings are those of the attempt that
/// answered.
struct RecordingProposer {
    inner: ModelClient,
    log: ExchangeLog,
}

impl Proposer for RecordingProposer {
    fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal> {
        self.propose_measured(request).map(|reply| reply.proposal)
    }

    fn propose_scored(
        &self,
        request: &ModelRequest,
    ) -> EngineResult<(ModelProposal, Option<TokenConfidence>)> {
        self.propose_measured(request)
            .map(|reply| (reply.proposal, reply.token_confidence))
    }

    fn propose_measured(&self, request: &ModelRequest) -> EngineResult<ProposerReply> {
        let started = Instant::now();
        let result = self.inner.propose_measured(request);
        self.log.push(
            &request.prompt,
            Exchange {
                prompt_sha256: request.sha256(),
                prompt_characters: request.prompt.chars().count(),
                reply: RecordedReply::from_result(&result),
                model_timings: result.as_ref().ok().and_then(|reply| reply.timings),
                wall_micros: micros_since(started),
            },
        );
        result
    }

    fn context_tokens(&self) -> Option<usize> {
        self.inner.context_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::WARM_UP_TEXT;

    /// The warm-up's prompt stays past the instructions plus 4 + 512
    /// tokens: its 2,095 characters make about 1,050 tokens with the
    /// instructions on Qwen3.5's tokenizer, and the 2,000 asked here about
    /// 1,025.
    #[test]
    fn the_warm_up_processes_the_instructions_in_one_piece() {
        assert!(
            WARM_UP_TEXT.chars().count() >= 2_000,
            "{}",
            WARM_UP_TEXT.len()
        );
        assert_eq!(WARM_UP_TEXT.split("\n\n").count(), 10);
    }
}
