//! The document-understanding engine: one call in, one structured result out.
//!
//! ```text
//! DocumentSource ─▶ distill ─▶ prompt ─▶ one inference ─▶ validate ─▶ name
//! ```
//!
//! Everything above this line (extraction, OCR) and everything below it (the
//! queue, the file operations, the UI) is somebody else's problem. That is what
//! makes a CLI, a watched folder, or a future connector able to reuse this
//! without touching how documents are understood.
//!
//! The inference is local by default. A hosted model behind an API key can
//! stand in the same place - see [`crate::hosted`] - and everything on either
//! side of it, the distillation the model reads and the validation its reply
//! must pass, is identical.

use std::time::Instant;

use crate::client::{ModelClient, ModelRequest, Proposer};
use crate::distill::{DigestBudget, DocumentDigest, distill};
use crate::domain::{
    AnalysisTelemetry, DocumentAnalysis, DocumentSource, ProposalStatus, ReviewReason,
    ValidationOutcome,
};
use crate::error::{EngineError, EngineErrorCode, EngineResult};
use crate::evidence::stated_dates;
use crate::fingerprint::{self, source_fingerprint};
use crate::naming::compose_filename;
use crate::validate::validate;

/// Below this much extracted text, a document is treated as unreadable rather
/// than analysed on the strength of a few stray characters.
///
/// Intern has no vision fallback: the model is text-only and the local server
/// runs without a projector, so a page nothing could read is a page for a human
/// to look at, not one to guess about.
pub const MIN_READABLE_CHARACTERS: usize = 200;

pub struct Engine {
    client: Box<dyn Proposer>,
    budget: DigestBudget,
    min_token_confidence: Option<f32>,
}

impl Engine {
    pub fn new(client: ModelClient) -> Self {
        Self::with_proposer(Box::new(client))
    }

    /// An engine over any model that can answer the prompt - the local server
    /// or a hosted one.
    pub fn with_proposer(client: Box<dyn Proposer>) -> Self {
        Self {
            client,
            budget: DigestBudget::default(),
            min_token_confidence: None,
        }
    }

    /// Routes a proposal to review when the model's least probable date or
    /// party token falls below `threshold` ([`TokenConfidence::min`]).
    ///
    /// Unset by default, and silent for a reply that carried no token
    /// probabilities: readiness is decided by validation alone until a
    /// threshold has been calibrated against the corpus.
    ///
    /// [`TokenConfidence::min`]: crate::domain::TokenConfidence::min
    pub fn with_min_token_confidence(mut self, threshold: f32) -> Self {
        self.min_token_confidence = Some(threshold);
        self
    }

    pub fn with_budget(mut self, budget: DigestBudget) -> Self {
        self.budget = budget;
        self
    }

    pub fn budget(&self) -> DigestBudget {
        self.budget
    }

    /// Reads one document and proposes a name, a description, and the evidence
    /// behind both.
    pub fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> EngineResult<DocumentAnalysis> {
        let distill_started = Instant::now();
        let digest = distill(source, self.budget);
        let distill_micros =
            u64::try_from(distill_started.elapsed().as_micros()).unwrap_or(u64::MAX);
        self.analyze_digest(source, &digest, distill_micros, extension, existing_names)
    }

    /// Runs inference, validation, and naming over an already-built digest.
    pub fn analyze_digest(
        &self,
        source: &DocumentSource,
        digest: &DocumentDigest,
        distill_micros: u64,
        extension: &str,
        existing_names: &[&str],
    ) -> EngineResult<DocumentAnalysis> {
        let request = ModelRequest::from_digest(digest);
        let inference_started = Instant::now();
        let (proposal, token_confidence) = self.client.propose_scored(&request)?;
        let inference_millis =
            u64::try_from(inference_started.elapsed().as_millis()).unwrap_or(u64::MAX);

        guard_analysis(|| {
            let mut outcome = validate(proposal, digest);
            if barely_readable(source) {
                // A page that yielded almost no text cannot support a
                // confident name, whatever the model returned about it.
                if !outcome.reasons.contains(&ReviewReason::ParserWarning) {
                    outcome.reasons.push(ReviewReason::ParserWarning);
                }
                outcome.status = ProposalStatus::NeedsReview;
            }
            if let (Some(threshold), Some(confidence)) =
                (self.min_token_confidence, token_confidence)
                && confidence.min < threshold
            {
                if !outcome.reasons.contains(&ReviewReason::LowConfidence) {
                    outcome.reasons.push(ReviewReason::LowConfidence);
                }
                outcome.status = ProposalStatus::NeedsReview;
            }
            let mut analysis = finish(
                outcome,
                digest,
                extension,
                existing_names,
                AnalysisTelemetry {
                    source_characters: digest.source_characters,
                    digest_characters: digest.digest_characters,
                    compression_ratio: digest.compression_ratio(),
                    distill_micros,
                    inference_millis,
                },
            );
            analysis.text_fingerprint = source_fingerprint(source).map(fingerprint::encode);
            analysis.token_confidence = token_confidence;
            analysis
        })
    }

    pub fn distill(&self, source: &DocumentSource) -> DocumentDigest {
        distill(source, self.budget)
    }
}

/// Builds the final analysis from an already-validated proposal.
///
/// Split out so evaluation harnesses can drive validation and naming without a
/// live model.
pub fn finish(
    outcome: ValidationOutcome,
    digest: &DocumentDigest,
    extension: &str,
    existing_names: &[&str],
    telemetry: AnalysisTelemetry,
) -> DocumentAnalysis {
    let filename = compose_filename(&outcome.proposal, extension, existing_names).value;
    DocumentAnalysis {
        filename,
        description: outcome.proposal.description.clone(),
        status: outcome.status,
        review_reasons: outcome.reasons,
        proposal: outcome.proposal,
        telemetry,
        model_proposal: Some(outcome.candidate),
        stated_dates: stated_dates(digest),
        text_fingerprint: None,
        token_confidence: None,
    }
}

/// Runs the work that follows the model's reply - validation, naming,
/// fingerprinting - so that a panic in it fails this one document instead of
/// taking the model thread down with it.
///
/// That work is plain string handling over text nobody controls, and a slip
/// in it once panicked on every French invoice: the panic ended the thread
/// that runs the model request, the queue read the lost reply as the model
/// failing, and paused everything. Caught here, it is
/// [`EngineErrorCode::AnalysisFailed`], a failure of this document alone.
///
/// The panic's payload is dropped unread and the error carries a fixed
/// message: a failed slice reports the string it was slicing, which is
/// document text, and document text never goes into an error. Retrying the
/// same document would only panic the same way again. `AssertUnwindSafe` is
/// sound because nothing the closure touched is used again after a panic:
/// what it owned is dropped, and what it borrowed it only read.
fn guard_analysis<F: FnOnce() -> DocumentAnalysis>(f: F) -> EngineResult<DocumentAnalysis> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .map_err(|_| EngineError::new(EngineErrorCode::AnalysisFailed, "document analysis failed"))
}

/// True when extraction produced too little text to name a document from.
///
/// The parser hands back a rendered page image when it could not read a page as
/// text. Intern cannot show that image to the model, so the image is a signal
/// rather than an input: it means a human should look at this one.
pub fn barely_readable(source: &DocumentSource) -> bool {
    source.page_image.is_some() && source.character_count() < MIN_READABLE_CHARACTERS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::source_from_text;
    use crate::domain::{PageImage, PageOrigin, SourcePage};

    fn source_with_image(text: &str) -> DocumentSource {
        DocumentSource {
            pages: vec![SourcePage::new(1, text, PageOrigin::Ocr)],
            parser_warnings: Vec::new(),
            page_image: Some(PageImage {
                page_number: 1,
                media_type: "image/png".into(),
                bytes: vec![1, 2, 3],
            }),
        }
    }

    #[test]
    fn a_text_bearing_page_is_readable_even_when_an_image_came_with_it() {
        let source = source_with_image(
            "STATEMENT OF WORK\n\nThis Statement of Work is effective as of April 1, 2026 by and \
             between Acme Corporation and Contoso Worldwide, Inc. and covers the 2026 CRM \
             implementation, its deliverables, its fees, and its project term.",
        );
        assert!(!barely_readable(&source));
    }

    #[test]
    fn a_scan_that_yielded_nothing_is_flagged_rather_than_guessed_at() {
        assert!(barely_readable(&source_with_image("l1 ll  I")));
    }

    /// The dates a reviewer is offered are the document's own, each once, in
    /// the order the document states them, and never more than a handful.
    #[test]
    fn the_analysis_lists_the_dates_the_document_states_once_each_in_order() {
        use crate::domain::{ModelProposal, PartyRelation};

        let source = source_from_text(
            "CONSULTING AGREEMENT\n\nThis Agreement is effective as of April 1, 2026.\n\
             Signed on March 28, 2026 by both parties.\n\
             The initial term ends on 2027-03-31. Effective as of April 1, 2026 again.\n\
             Invoices are due 30 days after 15 May 2026.",
        );
        let digest = crate::distill::distill(&source, crate::distill::DigestBudget::default());
        let outcome = validate(
            ModelProposal {
                document_type: Some("Consulting Agreement".into()),
                document_date: Some("2026-04-01".into()),
                date_role: Some(crate::domain::DateRole::Effective),
                parties: Vec::new(),
                party_relation: PartyRelation::None,
                description: "Consulting agreement effective April 1, 2026 for an initial term."
                    .into(),
                confidence: 0.9,
                needs_review: false,
                evidence: crate::domain::Evidence::default(),
            },
            &digest,
        );
        let analysis = finish(outcome, &digest, "pdf", &[], AnalysisTelemetry::default());
        assert_eq!(
            analysis.stated_dates,
            vec!["2026-04-01", "2026-03-28", "2027-03-31", "2026-05-15"]
        );
    }

    struct Scored(Option<crate::domain::TokenConfidence>);

    impl Proposer for Scored {
        fn propose(&self, request: &ModelRequest) -> EngineResult<crate::domain::ModelProposal> {
            self.propose_scored(request).map(|(proposal, _)| proposal)
        }

        fn propose_scored(
            &self,
            _request: &ModelRequest,
        ) -> EngineResult<(
            crate::domain::ModelProposal,
            Option<crate::domain::TokenConfidence>,
        )> {
            Ok((
                crate::domain::ModelProposal {
                    document_type: Some("Consulting Agreement".into()),
                    document_date: Some("2026-04-01".into()),
                    date_role: Some(crate::domain::DateRole::Effective),
                    parties: vec!["Acme Corporation".into()],
                    party_relation: crate::domain::PartyRelation::With,
                    description: "Consulting agreement with Acme Corporation effective April 1, \
                                  2026."
                        .into(),
                    confidence: 0.9,
                    needs_review: false,
                    evidence: crate::domain::Evidence {
                        date: Some("This Agreement is effective as of April 1, 2026.".into()),
                        document_type: Some("CONSULTING AGREEMENT".into()),
                        parties: vec!["Acme Corporation".into()],
                    },
                },
                self.0,
            ))
        }
    }

    fn analyze_with(engine: Engine) -> DocumentAnalysis {
        let source = source_from_text(
            "CONSULTING AGREEMENT\n\nThis Agreement is effective as of April 1, 2026.\n\
             It is made between Acme Corporation and the consultant for advisory services.",
        );
        engine.analyze(&source, "pdf", &[]).unwrap()
    }

    const LOW: crate::domain::TokenConfidence = crate::domain::TokenConfidence {
        min: 0.2,
        mean: 0.7,
        tokens: 12,
    };

    /// Token confidence is data first: it rides along on the analysis and
    /// changes nothing about readiness unless a threshold was set.
    #[test]
    fn token_confidence_is_reported_without_changing_readiness_by_default() {
        let unscored = analyze_with(Engine::with_proposer(Box::new(Scored(None))));
        let scored = analyze_with(Engine::with_proposer(Box::new(Scored(Some(LOW)))));
        assert_eq!(unscored.status, ProposalStatus::Ready, "{unscored:?}");
        assert_eq!(scored.status, unscored.status);
        assert_eq!(scored.review_reasons, unscored.review_reasons);
        assert_eq!(scored.token_confidence, Some(LOW));
        assert_eq!(unscored.token_confidence, None);
    }

    #[test]
    fn a_set_threshold_routes_a_low_token_confidence_to_review() {
        let gated = analyze_with(
            Engine::with_proposer(Box::new(Scored(Some(LOW)))).with_min_token_confidence(0.5),
        );
        assert_eq!(gated.status, ProposalStatus::NeedsReview);
        assert!(gated.review_reasons.contains(&ReviewReason::LowConfidence));

        // Above the threshold, or with nothing to measure, the gate is silent.
        let passed = analyze_with(
            Engine::with_proposer(Box::new(Scored(Some(LOW)))).with_min_token_confidence(0.1),
        );
        assert_eq!(passed.status, ProposalStatus::Ready);
        let unmeasured = analyze_with(
            Engine::with_proposer(Box::new(Scored(None))).with_min_token_confidence(0.5),
        );
        assert_eq!(unmeasured.status, ProposalStatus::Ready);
    }

    /// A panic after the model answered is this document's failure, under
    /// its own code and a fixed message - never the panic's text, which for
    /// a bad slice is the document's own words.
    #[test]
    fn guard_analysis_turns_a_panic_into_analysis_failed() {
        let error = guard_analysis(|| {
            // What a bad slice reports: the text it was slicing.
            panic!(
                "byte index 4 is not a char boundary; it is inside 'é' (bytes 3..5) of \
                 `La présente convention`"
            )
        })
        .expect_err("a panic must come back as an error");
        assert_eq!(error.code(), EngineErrorCode::AnalysisFailed);
        assert_eq!(error.code().as_str(), "ANALYSIS_FAILED");
        assert_eq!(error.message(), "document analysis failed");
        assert!(!error.to_string().contains("convention"));

        // Work that does not panic passes straight through.
        let analysis =
            guard_analysis(|| analyze_with(Engine::with_proposer(Box::new(Scored(None)))))
                .expect("no panic, no error");
        assert_eq!(analysis.status, ProposalStatus::Ready);
    }

    #[test]
    fn a_short_document_with_no_image_is_not_treated_as_unreadable() {
        // A one-line note is short but perfectly legible; only a page the parser
        // gave up on arrives with an image attached.
        assert!(!barely_readable(&source_from_text("Paid in full.")));
    }
}
