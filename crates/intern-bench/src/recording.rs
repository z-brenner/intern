//! What a live run saw, kept so the corpus can be scored again without a
//! worker or a model.
//!
//! The format, `bench/recording.json` (schema 1):
//!
//! ```text
//! {
//!   "schema_version": 1, "suite": "internbench",
//!   "recorded_at": "2026-10-06T12:00:00Z",
//!   "model": {"id", "path", "size_bytes", "sha256"},   // what --model-path named
//!   "context_tokens": 8192,                              // the proposer's, so replay fits prompts alike
//!   "budget_characters": 12000,
//!   "machine": {...}, "git_commit": "...", "worker": "...", "note": "...",
//!   "documents": [{
//!     "id", "file",
//!     "sha256": "...",                                   // the document bytes recorded from
//!     "extraction": {"outcome": "parsed", "source": DocumentSource}
//!                 | {"outcome": "failed", "code": "..."},
//!     "exchanges": [{                                    // every model request, in order
//!       "prompt_sha256", "prompt_characters", "wall_micros",
//!       "reply": {"outcome": "proposed", "proposal", "token_confidence"?}
//!              | {"outcome": "failed", "code": "MODEL_INPUT_TOO_LARGE"},
//!       "model_timings": {"promptTokens", ...}?          // the server's own account
//!     }],
//!     "timings": {...}, "memory": {...}                  // as measured live
//!   }]
//! }
//! ```
//!
//! Replies are keyed by the SHA-256 of the exact prompt they answer, as in
//! `intern-evaluate`'s recording, but a document keeps a list of them:
//! the engine may ask more than once - a prompt the server refused as too
//! large is condensed and asked again - and replay must hand back the
//! refusal for the first prompt to walk the same path. A rendered page
//! image is kept as its signal only, with the pixels dropped, because the
//! engine reads the image's presence, never its bytes.

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

use intern_engine::{
    DocumentSource, EngineError, EngineErrorCode, EngineResult, ModelProposal, ModelRequest,
    ModelTimings, PageImage, Proposer, ProposerReply, TokenConfidence,
};
use serde::{Deserialize, Serialize};

use crate::{
    machine::{MachineInfo, ModelInfo},
    memory::MemoryPeaks,
    timing::Timings,
};

pub const RECORDING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Recording {
    pub schema_version: u32,
    #[serde(default)]
    pub suite: String,
    #[serde(default)]
    pub recorded_at: String,
    #[serde(default)]
    pub model: ModelInfo,
    /// What the recording proposer reported as its context: the engine
    /// condenses prompts to fit it, so replay must report the same.
    #[serde(default)]
    pub context_tokens: Option<usize>,
    pub budget_characters: usize,
    #[serde(default)]
    pub machine: MachineInfo,
    #[serde(default)]
    pub git_commit: Option<String>,
    #[serde(default)]
    pub worker: Option<String>,
    /// Free text about the machine and runtime the recording was made on.
    #[serde(default)]
    pub note: String,
    pub documents: Vec<RecordedDocument>,
}

impl Recording {
    pub fn load(path: &Path) -> Result<(Self, Vec<u8>), String> {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("cannot read recording {}: {error}", path.display()))?;
        let recording: Self = serde_json::from_slice(&bytes)
            .map_err(|error| format!("cannot parse recording: {error}"))?;
        if recording.schema_version != RECORDING_SCHEMA_VERSION {
            return Err(format!(
                "recording schema {} is not the supported {RECORDING_SCHEMA_VERSION}",
                recording.schema_version
            ));
        }
        Ok((recording, bytes))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let rendered = serde_json::to_string_pretty(self)
            .map_err(|error| format!("cannot render recording: {error}"))?;
        std::fs::write(path, rendered + "\n")
            .map_err(|error| format!("cannot write recording {}: {error}", path.display()))
    }

    pub fn document(&self, id: &str) -> Option<&RecordedDocument> {
        self.documents.iter().find(|document| document.id == id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordedDocument {
    pub id: String,
    pub file: String,
    /// SHA-256 of the document bytes the recording was made from.
    #[serde(default)]
    pub sha256: Option<String>,
    pub extraction: RecordedExtraction,
    #[serde(default)]
    pub exchanges: Vec<Exchange>,
    #[serde(default)]
    pub timings: Timings,
    #[serde(default)]
    pub memory: MemoryPeaks,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RecordedExtraction {
    Parsed { source: DocumentSource },
    Failed { code: String },
}

/// One request the engine made and what came back.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Exchange {
    pub prompt_sha256: String,
    pub prompt_characters: usize,
    pub reply: RecordedReply,
    /// The server's account of the request, when it gave one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_timings: Option<ModelTimings>,
    pub wall_micros: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RecordedReply {
    Proposed {
        proposal: ModelProposal,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token_confidence: Option<TokenConfidence>,
    },
    Failed {
        code: EngineErrorCode,
    },
}

impl RecordedReply {
    pub fn from_result(result: &EngineResult<ProposerReply>) -> Self {
        match result {
            Ok(reply) => Self::Proposed {
                proposal: reply.proposal.clone(),
                token_confidence: reply.token_confidence,
            },
            Err(error) => Self::Failed { code: error.code() },
        }
    }

    /// The reply as the model gave it, minus the server's timings, which
    /// are the recording's to report and not the engine's to measure.
    fn answer(&self) -> EngineResult<ProposerReply> {
        match self {
            Self::Proposed {
                proposal,
                token_confidence,
            } => Ok(ProposerReply {
                proposal: proposal.clone(),
                token_confidence: *token_confidence,
                timings: None,
            }),
            Self::Failed { code } => Err(EngineError::new(*code, "recorded failure")),
        }
    }
}

/// A rendered page image is a signal that a page could not be read, never an
/// input, so the recording keeps the signal and drops the pixels.
pub fn without_image_bytes(mut source: DocumentSource) -> DocumentSource {
    source.page_image = source.page_image.map(|image| PageImage {
        bytes: Vec::new(),
        ..image
    });
    source
}

/// The SHA-256 of `bytes`, as lower-case hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// The requests the engine made for the current document, shared between
/// the proposer the engine owns and the runner that reads them back.
#[derive(Clone, Default)]
pub struct ExchangeLog {
    exchanges: Arc<Mutex<Vec<Exchange>>>,
    prompts: Arc<Mutex<Vec<String>>>,
}

impl ExchangeLog {
    pub fn push(&self, prompt: &str, exchange: Exchange) {
        if let Ok(mut exchanges) = self.exchanges.lock() {
            exchanges.push(exchange);
        }
        if let Ok(mut prompts) = self.prompts.lock() {
            prompts.push(prompt.to_owned());
        }
    }

    /// Everything logged since the last take: the exchanges, and the last
    /// prompt sent in full.
    pub fn take(&self) -> (Vec<Exchange>, Option<String>) {
        let exchanges = self
            .exchanges
            .lock()
            .map(|mut exchanges| std::mem::take(&mut *exchanges))
            .unwrap_or_default();
        let prompt = self
            .prompts
            .lock()
            .map(|mut prompts| std::mem::take(&mut *prompts).pop())
            .unwrap_or_default();
        (exchanges, prompt)
    }
}

/// Stands in for the model in replay: answers each prompt with the reply
/// recorded for it, and notes every prompt it was never recorded answering.
pub struct LookupProposer {
    replies: HashMap<String, RecordedReply>,
    /// What to answer a prompt nobody recorded when staleness is allowed:
    /// the reply the recorded run ended on.
    fallback: Option<RecordedReply>,
    context_tokens: Option<usize>,
    log: ExchangeLog,
    misses: Arc<Mutex<Vec<String>>>,
}

impl LookupProposer {
    pub fn new(
        exchanges: &[Exchange],
        context_tokens: Option<usize>,
        allow_stale: bool,
        log: ExchangeLog,
    ) -> Self {
        Self {
            replies: exchanges
                .iter()
                .map(|exchange| (exchange.prompt_sha256.clone(), exchange.reply.clone()))
                .collect(),
            fallback: allow_stale
                .then(|| exchanges.last().map(|exchange| exchange.reply.clone()))
                .flatten(),
            context_tokens,
            log,
            misses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// A handle on the prompts that missed, readable after the engine has
    /// taken ownership of the proposer.
    pub fn misses(&self) -> Arc<Mutex<Vec<String>>> {
        Arc::clone(&self.misses)
    }
}

impl Proposer for LookupProposer {
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
        let sha = request.sha256();
        let reply = match self.replies.get(&sha) {
            Some(reply) => reply.clone(),
            None => {
                if let Ok(mut misses) = self.misses.lock() {
                    misses.push(sha.clone());
                }
                match &self.fallback {
                    Some(reply) => reply.clone(),
                    // Not a failure the engine retries: it ends the
                    // document, which the runner reports as stale.
                    None => RecordedReply::Failed {
                        code: EngineErrorCode::ModelRequestFailed,
                    },
                }
            }
        };
        self.log.push(
            &request.prompt,
            Exchange {
                prompt_sha256: sha,
                prompt_characters: request.prompt.chars().count(),
                reply: reply.clone(),
                model_timings: None,
                wall_micros: u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
            },
        );
        reply.answer()
    }

    fn context_tokens(&self) -> Option<usize> {
        self.context_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recorded_failure_round_trips_under_its_engine_code() {
        let exchange = Exchange {
            prompt_sha256: "ab".repeat(32),
            prompt_characters: 10,
            reply: RecordedReply::Failed {
                code: EngineErrorCode::ModelInputTooLarge,
            },
            model_timings: None,
            wall_micros: 5,
        };
        let text = serde_json::to_string(&exchange).unwrap();
        assert!(
            text.contains(r#""outcome":"failed","code":"MODEL_INPUT_TOO_LARGE""#),
            "{text}"
        );
        assert!(!text.contains("model_timings"));
        assert_eq!(serde_json::from_str::<Exchange>(&text).unwrap(), exchange);
    }

    #[test]
    fn image_pixels_are_dropped_but_the_signal_is_kept() {
        let source = DocumentSource {
            pages: Vec::new(),
            parser_warnings: Vec::new(),
            page_image: Some(PageImage {
                page_number: 1,
                media_type: "image/png".into(),
                bytes: vec![1, 2, 3],
            }),
        };
        let kept = without_image_bytes(source);
        let image = kept.page_image.unwrap();
        assert!(image.bytes.is_empty());
        assert_eq!(image.page_number, 1);
    }

    #[test]
    fn the_lookup_answers_recorded_prompts_and_counts_the_rest() {
        let request = ModelRequest::new("a prompt");
        let recorded = Exchange {
            prompt_sha256: request.sha256(),
            prompt_characters: 8,
            reply: RecordedReply::Failed {
                code: EngineErrorCode::ModelReplyTruncated,
            },
            model_timings: None,
            wall_micros: 1,
        };
        let log = ExchangeLog::default();
        let lookup = LookupProposer::new(&[recorded.clone()], Some(8_192), false, log.clone());
        let misses = lookup.misses();
        assert_eq!(
            lookup.propose_measured(&request).unwrap_err().code(),
            EngineErrorCode::ModelReplyTruncated
        );
        assert!(misses.lock().unwrap().is_empty());
        let other = ModelRequest::new("another prompt");
        assert!(lookup.propose_measured(&other).is_err());
        assert_eq!(*misses.lock().unwrap(), vec![other.sha256()]);
        assert_eq!(lookup.context_tokens(), Some(8_192));
        let (exchanges, last) = log.take();
        assert_eq!(exchanges.len(), 2);
        assert_eq!(last.as_deref(), Some("another prompt"));

        // Allowed to be stale, a miss gets the reply the run ended on.
        let tolerant = LookupProposer::new(&[recorded], None, true, ExchangeLog::default());
        assert_eq!(
            tolerant.propose_measured(&other).unwrap_err().code(),
            EngineErrorCode::ModelReplyTruncated
        );
    }
}
