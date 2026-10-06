//! One document's line in the report: what Intern produced, what it should
//! have produced, every score, and where the time went.

use std::collections::BTreeMap;

use intern_engine::{DigestBudget, DocumentAnalysis, DocumentSource, domain::ProposalStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    claims::{Claim, DocumentText},
    gold::{GoldDocument, page_bucket},
    memory::MemoryPeaks,
    ocr::OcrMeasure,
    recording::Exchange,
    score::{Outcome, Texts, gold_filenames, score},
    timing::{self, Timings},
};

/// Analysed, named, and scored.
pub const COMPLETED: &str = "completed";
/// The worker could not read the document. Scored as a miss.
pub const EXTRACTION_FAILED: &str = "extraction_failed";
/// The model gave no usable answer. Scored as a miss.
pub const MODEL_FAILED: &str = "model_failed";
/// Replay: the engine now builds a prompt nobody recorded a reply to.
pub const STALE_PROMPT: &str = "stale_prompt";
/// Replay: the document's bytes are not the ones the recording was made
/// from.
pub const STALE_FIXTURE: &str = "stale_fixture";
/// Replay: the recording has no entry for the document.
pub const UNRECORDED: &str = "unrecorded";
/// Added to the corpus before anyone could record it; neither scored nor a
/// failure.
pub const PENDING: &str = "pending";

/// A status replay could not score, which fails the run unless staleness
/// is allowed.
pub fn is_unscorable(status: &str) -> bool {
    matches!(status, STALE_PROMPT | STALE_FIXTURE | UNRECORDED)
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DocumentRecord {
    pub id: String,
    pub file: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub text_layer: String,
    #[serde(default)]
    pub pages: u32,
    #[serde(default)]
    pub page_bucket: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub replayed: bool,
    /// Scored from a reply to a prompt the engine no longer builds, because
    /// staleness was allowed.
    #[serde(default)]
    pub stale: bool,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// `ready`, `needs_review`, or `failed`.
    #[serde(default)]
    pub readiness: Option<String>,
    #[serde(default)]
    pub review_reasons: Vec<String>,
    /// Every name the gold composes to; the first is the reviewed one.
    #[serde(default)]
    pub gold_filenames: Vec<String>,
    #[serde(default)]
    pub proposal: Option<Value>,
    #[serde(default)]
    pub model_proposal: Option<Value>,
    #[serde(default)]
    pub token_confidence: Option<Value>,
    #[serde(default)]
    pub scores: BTreeMap<String, Value>,
    /// The description's checkable claims and whether the document states
    /// each.
    #[serde(default)]
    pub claims: Vec<Claim>,
    /// Gold `description_forbidden` strings the description asserted.
    #[serde(default)]
    pub forbidden_description: Vec<String>,
    /// Every trap date or party the outcome chose, with the gold's reason.
    #[serde(default)]
    pub traps: Vec<String>,
    #[serde(default)]
    pub ocr: Option<OcrMeasure>,
    /// The SHA-256 of every prompt the engine sent, in order.
    #[serde(default)]
    pub prompt_sha256: Vec<String>,
    #[serde(default)]
    pub timings: Timings,
    /// Replay: the timings are the recording's, not measured now.
    #[serde(default)]
    pub timings_recorded: bool,
    #[serde(default)]
    pub memory: MemoryPeaks,
}

impl DocumentRecord {
    /// The record's identity and grouping keys, with nothing measured.
    pub fn for_document(document: &GoldDocument, status: &str) -> Self {
        Self {
            id: document.id.clone(),
            file: document.file.clone(),
            kind: document.kind.clone(),
            format: document.format.clone(),
            text_layer: document.text_layer.clone(),
            pages: document.pages,
            page_bucket: page_bucket(document.pages).to_owned(),
            categories: document.categories.clone(),
            status: status.to_owned(),
            gold_filenames: gold_filenames(document),
            timings: timing::empty(),
            ..Self::default()
        }
    }

    pub fn bool_score(&self, key: &str) -> Option<bool> {
        self.scores.get(key).and_then(Value::as_bool)
    }
}

/// What one run observed for one document.
pub struct Observation<'a> {
    pub status: &'a str,
    pub error: Option<String>,
    pub analysis: Option<&'a DocumentAnalysis>,
    pub source: Option<&'a DocumentSource>,
    pub budget: DigestBudget,
    pub exchanges: &'a [Exchange],
    /// The last prompt sent to the model, in full.
    pub last_prompt: Option<&'a str>,
    pub timings: Timings,
    pub timings_recorded: bool,
    pub memory: MemoryPeaks,
    pub replayed: bool,
    pub stale: bool,
}

/// Scores an observed document. A failed document - no analysis - is
/// scored too, as a miss on everything the gold defines: the corpus is the
/// denominator, and a document Intern could not name is not one it got
/// right.
pub fn scored_record(document: &GoldDocument, observation: Observation<'_>) -> DocumentRecord {
    let mut record = DocumentRecord::for_document(document, observation.status);
    record.error = observation.error;
    record.replayed = observation.replayed;
    record.stale = observation.stale;
    record.timings = observation.timings;
    record.timings_recorded = observation.timings_recorded;
    record.memory = observation.memory;
    record.prompt_sha256 = observation
        .exchanges
        .iter()
        .map(|exchange| exchange.prompt_sha256.clone())
        .collect();

    let truth_pages = document
        .ocr_truth
        .iter()
        .flat_map(|truth| &truth.pages)
        .map(|page| page.text.as_str());
    let source_pages = observation
        .source
        .iter()
        .flat_map(|source| &source.pages)
        .map(|page| page.text.as_str());
    let text = DocumentText::new(
        source_pages
            .chain(truth_pages)
            .chain(document.clean_text.as_deref()),
    );
    let digest = observation
        .source
        .map(|source| intern_engine::distill(source, observation.budget).text);
    let texts = Texts {
        document: Some(&text),
        digest: digest.as_deref(),
        prompt: observation.last_prompt,
        source: observation.source,
    };

    let outcome = match observation.analysis {
        Some(analysis) => {
            let ready = analysis.status == ProposalStatus::Ready;
            record.filename = Some(analysis.filename.clone());
            record.description = Some(analysis.description.clone());
            record.readiness = Some(if ready { "ready" } else { "needs_review" }.to_owned());
            record.review_reasons = analysis
                .review_reasons
                .iter()
                .map(|reason| reason.as_str().to_owned())
                .collect();
            record.proposal = serde_json::to_value(&analysis.proposal).ok();
            record.model_proposal = analysis
                .model_proposal
                .as_ref()
                .and_then(|proposal| serde_json::to_value(proposal).ok());
            record.token_confidence = analysis
                .token_confidence
                .and_then(|confidence| serde_json::to_value(confidence).ok());
            let proposal = &analysis.proposal;
            Outcome {
                analysed: true,
                filename: Some(&analysis.filename),
                description: &analysis.description,
                document_type: proposal.document_type.as_deref(),
                document_date: proposal.document_date.as_deref(),
                date_role: proposal.date_role.map(|role| role.as_str()),
                parties: &proposal.parties,
                party_relation: Some(proposal.party_relation.as_str()),
                ready,
                evidence: Some(
                    analysis
                        .model_proposal
                        .as_ref()
                        .map_or(&proposal.evidence, |candidate| &candidate.evidence),
                ),
            }
        }
        None => {
            record.readiness = Some("failed".to_owned());
            Outcome::default()
        }
    };
    let scored = score(document, &outcome, &texts);
    record.scores = scored.scores;
    record.claims = scored.claims;
    record.forbidden_description = scored.forbidden_description;
    record.traps = scored.traps;
    record.ocr = scored.ocr;
    record
}
