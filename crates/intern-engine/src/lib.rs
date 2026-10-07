//! Intern's document-understanding engine.
//!
//! One local inference per document, over a distillation of the *whole*
//! document, producing a filename, a one-sentence description, and the verbatim
//! evidence behind every fact.
//!
//! By default nothing in this crate talks to anything but `127.0.0.1`, and
//! document text never leaves the machine. The one exception is deliberate and
//! off unless configured: [`hosted`], which sends the same distilled prompt to
//! a hosted model behind an API key the user supplied.
//!
//! ```no_run
//! use intern_engine::{Engine, ModelClient, DocumentSource, SourcePage, PageOrigin};
//!
//! let client = ModelClient::new("http://127.0.0.1:8080/v1/chat/completions", "key", "intern-local")?;
//! let engine = Engine::new(client);
//! let source = DocumentSource::from_pages(vec![SourcePage::new(1, "NOTICE OF TERMINATION", PageOrigin::Native)]);
//! let analysis = engine.analyze(&source, "pdf", &[])?;
//! println!("{} - {}", analysis.filename, analysis.description);
//! # Ok::<(), intern_engine::EngineError>(())
//! ```

#![deny(unsafe_code)]

pub mod client;
pub mod compose;
pub mod cues;
pub mod distill;
pub mod domain;
pub mod download;
pub mod engine;
pub mod error;
pub mod evidence;
pub mod facts;
pub mod fingerprint;
pub mod hosted;
pub mod house_style;
pub mod index;
pub mod infer;
pub mod legacy;
pub mod logs;
pub mod manifest;
pub mod naming;
pub mod own_names;
pub(crate) mod phrases;
mod process;
pub mod prompt;
pub mod retrieve;
pub mod server;
pub mod setup;
pub mod structure;
#[cfg(test)]
mod test_support;
pub mod text;
pub mod validate;
pub mod worker;

pub use client::{EvidenceHandles, ModelClient, ModelRequest, Proposer, ProposerReply};
pub use distill::{DigestBudget, DocumentDigest, distill, source_from_text};
pub use domain::*;
pub use engine::{Engine, Pipeline, PreparedEvidence, estimated_prompt_tokens};
pub use error::{EngineError, EngineErrorCode, EngineResult};
pub use hosted::{HostedClient, HostedModelConfig, HostedProvider};
pub use house_style::{HouseRule, HouseStyle, RuleKind, compose_styled_filename, lesson_from_edit};
pub use manifest::{ModelFile, ModelManifest, ModelRole};
pub use naming::{compose_filename, sanitize_folder_name};
pub use own_names::{counterparty_view, is_own_name};
pub use server::{LlamaServer, ServerOptions};
pub use structure::{StructuredDocument, StructuredPage, structured};
pub use validate::validate;
pub use worker::{
    DocumentExtractor, ExtractFailure, ExtractProgress, ExtractionTimings, SupervisedWorker,
    prepare_worker_temp_root,
};

/// Semantic version of the engine's input/output contract.
///
/// Callers that persist [`DocumentAnalysis`] should record this alongside it.
///
/// 2 added, without removing anything, the evidence pipeline's facts
/// ([`ModelProposal::facts`], [`DocumentAnalysis::facts`]) and the index and
/// retrieval figures in [`AnalysisTelemetry`]. Every addition reads as absent
/// from an analysis stored under 1, and none refuses a field it does not
/// know, so a version-1 reader still reads a version-2 analysis.
pub const ENGINE_CONTRACT_VERSION: u32 = 2;
