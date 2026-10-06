//! A live run: the real worker, the real engine, the local model.
//!
//! Each document goes the way it goes in the app - the supervised parser
//! worker extracts it, [`Engine::analyze`] distils it, fits the prompt to
//! the model's context, asks the model once (or again, condensed, if the
//! server says the prompt did not fit), validates the reply, and names the
//! document - with a proposer in front of the model client that records
//! every request and its reply for replay.
//!
//! Before the corpus a short synthetic document goes through the worker
//! and the engine and is thrown away, so the first scored document is not
//! charged for starting the worker or for the server's empty prompt cache.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use intern_engine::{
    DigestBudget, DocumentExtractor, DocumentSource, Engine, EngineResult, ExtractFailure,
    ModelClient, ModelProposal, ModelRequest, Proposer, SupervisedWorker, TokenConfidence,
};
use serde_json::Value;

use crate::{
    gold::GoldDocument,
    machine::ModelInfo,
    memory::MemorySampler,
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

/// A fictional notice long enough to be read as a document, short enough
/// to cost a second.
const WARM_UP_TEXT: &str = "NOTICE OF OFFICE RELOCATION\n\nDated May 4, 2026\n\n\
    Brindlemoor Survey Partners LLP informs its clients that from June 1, 2026 its \
    office moves from 4 Quarry Lane to 12 Larkspur Row, Easthaven. Telephone numbers \
    and the client portal are unchanged. Correspondence sent to the old address will \
    be forwarded until August 31, 2026.\n\nOrla Penhallow, Managing Partner";

pub fn run(
    documents: &[GoldDocument],
    corpus: &Path,
    options: &LiveOptions,
) -> Result<LiveRun, String> {
    let model = ModelInfo::with_file(&options.model_id, options.model_path.as_deref())?;
    let client = ModelClient::new(
        &options.endpoint,
        options.api_key.clone(),
        &options.model_id,
    )
    .map_err(|error| format!("model client: {error}"))?;
    let log = ExchangeLog::default();
    let proposer = RecordingProposer {
        inner: client,
        log: log.clone(),
    };
    let context_tokens = proposer.context_tokens();
    let budget = DigestBudget::default();
    let engine = Engine::with_proposer(Box::new(proposer)).with_budget(budget);
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
                let telemetry = result
                    .as_ref()
                    .ok()
                    .and_then(|analysis| serde_json::to_value(analysis.telemetry).ok());
                let timings = flatten(&Measured {
                    extraction_wall_micros: Some(extraction_wall_micros),
                    worker: worker_timings.as_ref(),
                    telemetry: telemetry.as_ref(),
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
/// instruction, on a document nobody scores.
fn warm_up(worker: &SupervisedWorker, engine: &Engine) -> Result<(), String> {
    let directory =
        std::env::temp_dir().join(format!("intern-bench-warm-up-{}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    let path = directory.join("warm-up.txt");
    std::fs::write(&path, WARM_UP_TEXT)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    let started = Instant::now();
    let (extracted, _) = extract(worker, "bench-warm-up", &path);
    let _ = std::fs::remove_dir_all(&directory);
    let source = extracted.map_err(|failure| {
        format!(
            "the worker could not read the warm-up document: {}",
            failure.code
        )
    })?;
    engine
        .analyze(&source, "txt", &[])
        .map_err(|error| format!("the model did not answer the warm-up document: {error}"))?;
    eprintln!("warmed up in {:.1} s", started.elapsed().as_secs_f64());
    Ok(())
}

/// Extracts a document, with the worker's account of its stages when the
/// worker gives one.
fn extract(
    worker: &SupervisedWorker,
    request_id: &str,
    path: &Path,
) -> (Result<DocumentSource, ExtractFailure>, Option<Value>) {
    (worker.extract(request_id, path, &mut |_| {}), None)
}

fn micros_since(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

/// The model client, with every request and its reply written to a log.
struct RecordingProposer {
    inner: ModelClient,
    log: ExchangeLog,
}

impl Proposer for RecordingProposer {
    fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal> {
        self.propose_scored(request).map(|(proposal, _)| proposal)
    }

    fn propose_scored(
        &self,
        request: &ModelRequest,
    ) -> EngineResult<(ModelProposal, Option<TokenConfidence>)> {
        let started = Instant::now();
        let result = Proposer::propose_scored(&self.inner, request);
        self.log.push(
            &request.prompt,
            Exchange {
                prompt_sha256: request.sha256(),
                prompt_characters: request.prompt.chars().count(),
                reply: RecordedReply::from_result(&result),
                model_timings: None,
                wall_micros: micros_since(started),
            },
        );
        result
    }

    fn context_tokens(&self) -> Option<usize> {
        self.inner.context_tokens()
    }
}
