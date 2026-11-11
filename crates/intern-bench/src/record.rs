//! One document's line in the report: what Intern produced, what it should
//! have produced, every score, and where the time went.

use std::collections::BTreeMap;

use intern_engine::{
    DigestBudget, DocumentAnalysis, DocumentSource, ValidatedFacts,
    domain::{PartyRole, ProposalStatus, Support},
    retrieve::RetrievalConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    claims::{Claim, DocumentText},
    gold::{GoldDocument, page_bucket},
    memory::MemoryPeaks,
    ocr::OcrMeasure,
    recording::Exchange,
    score::{Outcome, Texts, gold_filenames, score},
    structure::{self, StructureMeasure},
    timing::{self, Timings},
};

/// Analyzed, named, and scored.
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

/// A status replay could not score. Any of them fails the run (exit 2):
/// `--allow-stale` keeps a document from being `stale_prompt` at all, by
/// scoring it from the old reply and marking it `stale`, but nothing makes
/// `stale_fixture` or `unrecorded` pass - those need recording again.
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
    /// How the evidence pipeline came to the filename's parties (see
    /// [`RelationRecord`]); absent for the digest pipeline, which keeps no
    /// such facts, and for a document that was not analyzed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<RelationRecord>,
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
    /// The structure the gold gives, measured over the text read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structure: Option<StructureMeasure>,
    /// The most expensive route a page took (`ocr`, `ocr_regions`,
    /// `layout`, `fast`), or `unrouted` when the worker sent no layouts;
    /// none when nothing was read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_class: Option<String>,
    /// Each page's route, in page order (`none` for a page without a
    /// layout); empty when no page had one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub page_routes: Vec<String>,
    /// The SHA-256 of every prompt the engine sent, in order.
    #[serde(default)]
    pub prompt_sha256: Vec<String>,
    #[serde(default)]
    pub timings: Timings,
    /// Replay: the timings are the recording's, not measured now - all but
    /// `index_ms` and `retrieval_ms`, which the context measurements take.
    #[serde(default)]
    pub timings_recorded: bool,
    #[serde(default)]
    pub memory: MemoryPeaks,
}

/// Why a document's filename names the parties it does: the class its type
/// sorted into, the rule of the relation table that applied, and every
/// party validation kept with the role the reply gave it and the support
/// the document gave that role.
///
/// It is what a rule's firings are counted from - how many names a default
/// such as `record: the first party, by default` gave, and how many of them
/// were right - which a filename and its review reasons alone do not say.
/// Nothing here is a score: no baseline holds it and no comparison reads
/// it.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RelationRecord {
    /// `agreement`, `issued`, `notice`, `record`, ...
    pub document_class: String,
    /// The relation rule that applied, in the engine's own words.
    pub basis: String,
    /// The validated parties in the order the document first names them,
    /// those set aside (copied, or signing for a party) included.
    #[serde(default)]
    pub parties: Vec<PartyRecord>,
}

/// One validated party of a [`RelationRecord`].
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PartyRecord {
    pub name: String,
    /// The role the document supports, which alone decides the relation.
    #[serde(default)]
    pub role: Option<PartyRole>,
    /// The role the reply gave, supported or not.
    #[serde(default)]
    pub proposed_role: Option<PartyRole>,
    /// How the document supports the reply's role.
    #[serde(default)]
    pub role_support: Support,
    /// The role the document's own wording gives the party.
    #[serde(default)]
    pub document_role: Option<PartyRole>,
    /// Named only on a "cc:" line.
    #[serde(default)]
    pub copied: bool,
    /// A person acting for a party, and so no party of their own.
    #[serde(default)]
    pub signatory: bool,
}

impl RelationRecord {
    fn of(facts: &ValidatedFacts) -> Self {
        Self {
            document_class: facts.document_class.as_str().to_owned(),
            basis: facts.relation_basis.clone(),
            parties: facts
                .parties
                .iter()
                .map(|party| PartyRecord {
                    name: party.name.clone(),
                    role: party.role,
                    proposed_role: party.proposed_role,
                    role_support: party.role_support,
                    document_role: party.document_role,
                    copied: party.copied,
                    signatory: party.signatory,
                })
                .collect(),
        }
    }
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

    /// The routes the extracted document's pages took; each page's only
    /// when the worker sent layouts.
    pub fn set_routes(&mut self, source: &DocumentSource) {
        let class = structure::route_class(source);
        self.route_class = Some(class.to_owned());
        if class != "unrouted" {
            self.page_routes = structure::page_routes(source);
        }
    }
}

/// What one run observed for one document.
pub struct Observation<'a> {
    pub status: &'a str,
    pub error: Option<String>,
    pub analysis: Option<&'a DocumentAnalysis>,
    pub source: Option<&'a DocumentSource>,
    pub budget: DigestBudget,
    /// The retrieval the `context_*` scores measure.
    pub retrieval: &'a RetrievalConfig,
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
            record.relation = analysis.facts.as_ref().map(RelationRecord::of);
            let proposal = &analysis.proposal;
            Outcome {
                analyzed: true,
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
                unsupported_reason: analysis
                    .review_reasons
                    .iter()
                    .any(|reason| crate::score::is_unsupported_reason(reason.as_str())),
            }
        }
        None => {
            record.readiness = Some("failed".to_owned());
            Outcome::default()
        }
    };
    let scored = score(document, &outcome, &texts);
    record.scores = scored.scores;
    // Beside `digest_recall`: what the evidence context would carry, and
    // what it costs, measured from the extracted text as an extract run
    // measures it. A document that could not be read carries nothing, and
    // still counts.
    let context =
        crate::context::context_scores(document, observation.source, observation.retrieval);
    record.scores.extend(context.scores);
    record.timings.extend(context.timings);
    // What only the evidence pipeline measures: nothing for the digest's.
    if let Some(analysis) = observation.analysis {
        record
            .scores
            .extend(crate::pipeline::evidence_scores(analysis));
    }
    record.claims = scored.claims;
    record.forbidden_description = scored.forbidden_description;
    record.traps = scored.traps;
    record.ocr = scored.ocr;
    record.structure = scored.structure;
    if let Some(source) = observation.source {
        record.set_routes(source);
    }
    record
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gold::GoldAnswer;

    /// A document that could not be read is a miss on the evidence context
    /// too: its zeroes stay in every `context_*` denominator, so a
    /// configuration that fails to read documents never looks better for it.
    #[test]
    fn a_failed_extraction_scores_zero_context_recall() {
        let document = GoldDocument {
            id: "notice".into(),
            file: "notice.pdf".into(),
            pages: 1,
            gold: GoldAnswer {
                document_type: Some("Notice of Default".into()),
                document_date: Some("2025-10-14".into()),
                parties: Some(vec!["Glasswing Ceramics LLC".into()]),
                ..GoldAnswer::default()
            },
            ..GoldDocument::default()
        };
        let retrieval = RetrievalConfig::default();
        let record = scored_record(
            &document,
            Observation {
                status: "extraction_failed",
                error: Some("PARSER_FAILED".into()),
                analysis: None,
                source: None,
                budget: DigestBudget::default(),
                retrieval: &retrieval,
                exchanges: &[],
                last_prompt: None,
                timings: Timings::default(),
                timings_recorded: false,
                memory: MemoryPeaks::default(),
                replayed: true,
                stale: false,
            },
        );
        for key in [
            "context_type_recall",
            "context_date_recall",
            "context_recall",
        ] {
            assert_eq!(
                record.scores.get(key).and_then(Value::as_f64),
                Some(0.0),
                "{key}"
            );
        }
    }

    /// An evidence-pipeline record says which relation rule named its
    /// parties and what the reply had called them, so a default's firings
    /// can be counted against the names they gave. It is no score: the
    /// scores, which a baseline holds, are the same with it or without, and
    /// a record with no validated facts writes no such key.
    #[test]
    fn a_record_says_which_relation_rule_named_its_parties() {
        let document = GoldDocument {
            id: "minutes".into(),
            file: "minutes.docx".into(),
            pages: 1,
            ..GoldDocument::default()
        };
        let mut analysis: DocumentAnalysis = serde_json::from_value(serde_json::json!({
            "filename": "2018-06-14 Minutes for Odalys Brandt.docx", "description": "",
            "status": "needs_review", "reviewReasons": ["party_unsupported"],
            "proposal": {"document_type": "Minutes", "document_date": "2018-06-14",
                "date_role": null, "parties": ["Odalys Brandt"], "party_relation": "for",
                "description": "", "confidence": 0.9,
                "evidence": {"date": null, "document_type": null, "parties": []}},
            "telemetry": {"sourceCharacters": 1, "digestCharacters": 1, "compressionRatio": 1.0,
                "distillMicros": 1, "inferenceMillis": 1}
        }))
        .unwrap();
        let retrieval = RetrievalConfig::default();
        let observe = |analysis: &DocumentAnalysis| {
            scored_record(
                &document,
                Observation {
                    status: COMPLETED,
                    error: None,
                    analysis: Some(analysis),
                    source: None,
                    budget: DigestBudget::default(),
                    retrieval: &retrieval,
                    exchanges: &[],
                    last_prompt: None,
                    timings: timing::empty(),
                    timings_recorded: true,
                    memory: MemoryPeaks::default(),
                    replayed: true,
                    stale: false,
                },
            )
        };
        let bare = observe(&analysis);
        assert_eq!(bare.relation, None);
        let written = serde_json::to_value(&bare).unwrap();
        assert!(written.get("relation").is_none(), "{written}");

        analysis.facts = Some(ValidatedFacts {
            parties: vec![
                intern_engine::ValidatedParty {
                    name: "Odalys Brandt".into(),
                    proposed_role: Some(PartyRole::Client),
                    ..Default::default()
                },
                intern_engine::ValidatedParty {
                    name: "Teodor Lindqvist".into(),
                    role: Some(PartyRole::Other),
                    proposed_role: Some(PartyRole::Client),
                    signatory: true,
                    ..Default::default()
                },
            ],
            document_class: intern_engine::DocumentClass::Record,
            relation_basis: "record: the first party, by default".into(),
            ..ValidatedFacts::default()
        });
        let record = observe(&analysis);
        let relation = record.relation.as_ref().expect("the relation is recorded");
        assert_eq!(relation.document_class, "record");
        assert_eq!(relation.basis, "record: the first party, by default");
        assert_eq!(relation.parties.len(), 2);
        assert_eq!(relation.parties[0].proposed_role, Some(PartyRole::Client));
        assert_eq!(relation.parties[0].role, None);
        assert!(relation.parties[1].signatory);
        // Every score both records hold is the same; the facts add only the
        // evidence pipeline's own support counts.
        for (key, value) in &bare.scores {
            assert_eq!(record.scores.get(key), Some(value), "{key}");
        }
        assert!(record.scores.keys().all(|key| !key.contains("relation")));
        let read: DocumentRecord =
            serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
        assert_eq!(read.relation, record.relation);
    }

    /// A document that was read carries what its evidence context costs
    /// beside what it holds, as an extract run's record does.
    #[test]
    fn a_read_document_carries_its_context_measurements() {
        let document = GoldDocument {
            id: "notice".into(),
            file: "notice.pdf".into(),
            pages: 1,
            ..GoldDocument::default()
        };
        let source = intern_engine::source_from_text(
            "NOTICE OF DEFAULT\n\nDate: October 14, 2025\n\nTo: Glasswing Ceramics LLC",
        );
        let retrieval = RetrievalConfig::default();
        let record = scored_record(
            &document,
            Observation {
                status: "model_failed",
                error: None,
                analysis: None,
                source: Some(&source),
                budget: DigestBudget::default(),
                retrieval: &retrieval,
                exchanges: &[],
                last_prompt: None,
                timings: timing::empty(),
                timings_recorded: true,
                memory: MemoryPeaks::default(),
                replayed: true,
                stale: false,
            },
        );
        for key in [
            "context_tokens",
            "context_units",
            "index_units",
            "index_ms",
            "retrieval_ms",
        ] {
            assert!(
                timing::get(&record.timings, key).is_some(),
                "{key}: {:?}",
                record.timings.get(key)
            );
        }
        assert!(timing::get(&record.timings, "context_units").unwrap() > 0.0);
    }
}
