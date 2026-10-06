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

use crate::client::{MAX_REPLY_TOKENS, ModelClient, ModelRequest, Proposer, ProposerReply};
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

/// What the system turn and the chat template take from the context: the
/// local server's 8,192 tokens leave 8,000 for the prompt and the reply.
const TEMPLATE_TOKENS: usize = 192;
/// How far below that ceiling a prompt that did not fit is condensed to - to
/// 6,500 tokens on the local server - so one re-distillation usually lands
/// inside it despite the fixed instructions.
const CONDENSING_MARGIN_TOKENS: usize = 1_500;
/// A condensed document smaller than this has lost what it is.
const MIN_REDISTILLED_CHARACTERS: usize = 2_000;
/// How many times a prompt that will not fit is condensed further before it
/// is sent anyway, for the server to have the last word.
const MAX_REDISTILLATIONS: usize = 2;

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
    ///
    /// The budget is in characters, the model's context in tokens, and the
    /// two part ways on exactly the documents a firm files: Qwen reads every
    /// digit, and every CJK character, as a token of its own, so a bank
    /// statement or a Chinese contract inside the character budget can still
    /// overflow the local server's 8,192 tokens. For a model with a context
    /// that small the prompt is estimated first and condensed further until
    /// it fits; a hosted model, whose context is many times larger, is sent
    /// it whole - condensing it there only dropped the blocks that named the
    /// date and the parties from a request that was paid for anyway. Either
    /// way, a model that still finds the prompt too large gets it condensed
    /// to half once more. Only a document that does not fit changes, so every
    /// prompt that fitted before is sent byte for byte as it was.
    pub fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> EngineResult<DocumentAnalysis> {
        let distill_started = Instant::now();
        let mut budget = self.budget;
        let mut digest = distill(source, budget);
        let mut redistillations = 0_u32;
        if let Some(context) = self.client.context_tokens() {
            let ceiling = context.saturating_sub(TEMPLATE_TOKENS);
            let target = ceiling.saturating_sub(CONDENSING_MARGIN_TOKENS).max(1);
            for _ in 0..MAX_REDISTILLATIONS {
                let estimate = estimated_tokens(&ModelRequest::from_digest(&digest).prompt);
                if estimate + MAX_REPLY_TOKENS as usize <= ceiling {
                    break;
                }
                let scaled = sent_characters(&digest, budget) * target / estimate.max(1);
                budget = condensed(scaled.max(MIN_REDISTILLED_CHARACTERS));
                digest = distill(source, budget);
                redistillations += 1;
            }
        }
        let distill_micros = micros_since(distill_started);
        let result =
            match self.analyze_digest(source, &digest, distill_micros, extension, existing_names) {
                // The estimate is an estimate. The server counts exactly, and
                // when it says the prompt did not fit, half as much document
                // is sent once more rather than the same prompt again.
                Err(error) if error.code() == EngineErrorCode::ModelInputTooLarge => {
                    let redistill_started = Instant::now();
                    let digest = distill(source, condensed(sent_characters(&digest, budget) / 2));
                    let distill_micros =
                        distill_micros.saturating_add(micros_since(redistill_started));
                    redistillations += 1;
                    self.analyze_digest(source, &digest, distill_micros, extension, existing_names)
                }
                result => result,
            };
        result.map(|mut analysis| {
            analysis.telemetry.redistillations = redistillations;
            analysis
        })
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
        let prompt_started = Instant::now();
        let request = ModelRequest::from_digest(digest);
        let prompt_micros = micros_since(prompt_started);
        let prompt_characters = request.prompt.chars().count();
        let estimated_prompt_tokens = estimated_tokens(&request.prompt);
        let inference_started = Instant::now();
        let ProposerReply {
            proposal,
            token_confidence,
            timings: model,
        } = self.client.propose_measured(&request)?;
        let inference_millis =
            u64::try_from(inference_started.elapsed().as_millis()).unwrap_or(u64::MAX);

        guard_analysis(|| {
            let validation_started = Instant::now();
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
            let validation_micros = micros_since(validation_started);
            let naming_started = Instant::now();
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
                    prompt_micros,
                    validation_micros,
                    naming_micros: 0,
                    redistillations: 0,
                    prompt_characters,
                    estimated_prompt_tokens,
                    model,
                },
            );
            analysis.text_fingerprint = source_fingerprint(source).map(fingerprint::encode);
            analysis.token_confidence = token_confidence;
            analysis.telemetry.naming_micros = micros_since(naming_started);
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

/// Roughly how many tokens `text` costs Qwen's tokenizer: one per digit and
/// per CJK, Hangul, or Kana character, which it splits singly, and one per
/// three and a half characters of anything else. Deliberately on the high
/// side; underestimating is what costs a request.
pub(crate) fn estimated_tokens(text: &str) -> usize {
    let mut single = 0_usize;
    let mut other = 0_usize;
    for character in text.chars() {
        if character.is_numeric() || is_wide_script(character) {
            single += 1;
        } else {
            other += 1;
        }
    }
    single + (other * 2).div_ceil(7)
}

/// Roughly how many tokens a prompt costs the local model, by the estimate
/// the engine fits prompts to its context with.
pub fn estimated_prompt_tokens(prompt: &str) -> usize {
    estimated_tokens(prompt)
}

/// CJK ideographs, Hangul, and Kana - the scripts a BPE vocabulary built
/// mostly from English spends about a token per character on.
fn is_wide_script(character: char) -> bool {
    matches!(
        u32::from(character),
        0x1100..=0x11FF       // Hangul Jamo
            | 0x3040..=0x30FF // Hiragana, Katakana
            | 0x3130..=0x318F // Hangul Compatibility Jamo
            | 0x31F0..=0x31FF // Katakana Phonetic Extensions
            | 0x3400..=0x4DBF // CJK Extension A
            | 0x4E00..=0x9FFF // CJK Unified Ideographs
            | 0xAC00..=0xD7AF // Hangul Syllables
            | 0xF900..=0xFAFF // CJK Compatibility Ideographs
            | 0xFF66..=0xFF9F // Halfwidth Katakana
            | 0x20000..=0x3134F // CJK Extensions B-G
    )
}

fn micros_since(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

/// How many characters of document the digest actually carried: the budget,
/// or the whole text when it was small enough to pass through.
fn sent_characters(digest: &DocumentDigest, budget: DigestBudget) -> usize {
    budget.max_characters.min(digest.digest_characters)
}

/// A budget that keeps `max_characters` of the document and always condenses,
/// even a document small enough to have been passed through whole.
fn condensed(max_characters: usize) -> DigestBudget {
    DigestBudget {
        passthrough_characters: 0,
        max_characters,
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

    /// A proposer that records every prompt it is sent and answers the `n`th
    /// with `answers[n]` (the last again once they run out). It has the local
    /// server's context unless built as a hosted one.
    struct Recording {
        prompts: std::sync::Mutex<Vec<String>>,
        answers: Vec<Result<(), crate::error::EngineErrorCode>>,
        context: Option<usize>,
    }

    impl Recording {
        fn new(answers: Vec<Result<(), crate::error::EngineErrorCode>>) -> Self {
            Self {
                prompts: std::sync::Mutex::new(Vec::new()),
                answers,
                context: Some(crate::server::CONTEXT_TOKENS as usize),
            }
        }

        fn hosted(answers: Vec<Result<(), crate::error::EngineErrorCode>>) -> Self {
            Self {
                context: None,
                ..Self::new(answers)
            }
        }

        fn prompts(&self) -> Vec<String> {
            self.prompts.lock().unwrap().clone()
        }
    }

    impl Proposer for std::sync::Arc<Recording> {
        fn propose(&self, request: &ModelRequest) -> EngineResult<crate::domain::ModelProposal> {
            let mut prompts = self.prompts.lock().unwrap();
            prompts.push(request.prompt.clone());
            let answer = self.answers[(prompts.len() - 1).min(self.answers.len() - 1)];
            match answer {
                Ok(()) => Scored(None)
                    .propose_scored(request)
                    .map(|(proposal, _)| proposal),
                Err(code) => Err(crate::error::EngineError::new(code, "scripted")),
            }
        }

        fn context_tokens(&self) -> Option<usize> {
            self.context
        }
    }

    /// A bank statement: a short heading, then line after line of dates,
    /// references, and amounts. Under the 12,000-character passthrough limit,
    /// so it used to go to the model whole - about 8,000 tokens of digits.
    fn digit_dense_statement() -> DocumentSource {
        let mut text = String::from("ACCOUNT STATEMENT\n\nStatement date: January 31, 2026\n\n");
        let mut line = 0_u32;
        while text.chars().count() < 11_500 {
            text.push_str(&format!(
                "{:02}/01/2026 {:010} {:>9}.{:02} {:>10}.{:02}\n",
                line % 28 + 1,
                7_340_000_000_u64 + u64::from(line) * 7_919,
                (line * 7_573) % 100_000,
                line % 100,
                (line * 15_377) % 1_000_000,
                (line * 31) % 100
            ));
            line += 1;
        }
        source_from_text(text)
    }

    #[test]
    fn digits_and_cjk_are_counted_a_token_each() {
        assert_eq!(estimated_tokens("2026"), 4);
        assert_eq!(estimated_tokens("契約書"), 3);
        assert_eq!(estimated_tokens("계약서"), 3);
        assert_eq!(estimated_tokens("けいやく"), 4);
        assert_eq!(estimated_tokens("agreement"), 3, "nine letters, rounded up");
        assert_eq!(estimated_tokens(""), 0);
    }

    /// Nothing that fitted before changes: an ordinary document's prompt is
    /// exactly the one its plain digest makes.
    #[test]
    fn a_prompt_that_fits_is_sent_unchanged() {
        let recording = std::sync::Arc::new(Recording::new(vec![Ok(())]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&recording)));
        let source = source_from_text(
            "CONSULTING AGREEMENT\n\nThis Agreement is effective as of April 1, 2026.\n\
             It is made between Acme Corporation and the consultant for advisory services.",
        );
        engine.analyze(&source, "pdf", &[]).unwrap();
        let expected = ModelRequest::from_digest(&distill(&source, DigestBudget::default()));
        assert_eq!(recording.prompts(), vec![expected.prompt]);
    }

    /// A digit-dense statement inside the character budget overflowed the
    /// token context: the server refused it, was restarted, refused it again,
    /// and the queue paused. It is now condensed before it is ever sent.
    #[test]
    fn oversized_digest_is_redistilled_before_sending() {
        let source = digit_dense_statement();
        let whole = ModelRequest::from_digest(&distill(&source, DigestBudget::default()));
        assert!(
            estimated_tokens(&whole.prompt) + 1_024 > 8_000,
            "the fixture must overflow as sent whole: {}",
            estimated_tokens(&whole.prompt)
        );

        let recording = std::sync::Arc::new(Recording::new(vec![Ok(())]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&recording)));
        let analysis = engine.analyze(&source, "pdf", &[]).unwrap();

        let prompts = recording.prompts();
        assert_eq!(prompts.len(), 1, "condensed before sending, not retried");
        let estimate = estimated_tokens(&prompts[0]);
        assert!(estimate + 1_024 <= 8_000, "{estimate}");
        assert!(
            prompts[0].contains("faithful condensation"),
            "a passthrough-sized document is condensed when it does not fit"
        );
        assert!(prompts[0].contains("Statement date: January 31, 2026"));
        assert!(analysis.telemetry.digest_characters < whole.prompt.chars().count());
    }

    /// The 8,192-token ceiling is the local server's. A hosted model's
    /// context is many times larger, and condensing the statement for it
    /// dropped the blocks that named the date and the parties from a request
    /// it was paid for anyway: it is sent the digest whole.
    #[test]
    fn a_model_without_the_local_context_is_sent_the_whole_digest() {
        let source = digit_dense_statement();
        let whole = ModelRequest::from_digest(&distill(&source, DigestBudget::default()));
        assert!(estimated_tokens(&whole.prompt) + 1_024 > 8_000);

        let hosted = std::sync::Arc::new(Recording::hosted(vec![Ok(())]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&hosted)));
        engine.analyze(&source, "pdf", &[]).unwrap();
        assert_eq!(hosted.prompts(), vec![whole.prompt]);

        // Which models those are: the local client knows its server's
        // context, and anything else is not condensed to it.
        let local = ModelClient::new("http://127.0.0.1:9/v1/chat/completions", "k", "m").unwrap();
        assert_eq!(local.context_tokens(), Some(8_192));
        assert_eq!(Scored(None).context_tokens(), None);
    }

    /// When the server counts more tokens than the estimate did, the document
    /// is condensed to half once and sent again - once, and never reported as
    /// a failed request that restarts the server and pauses the queue.
    #[test]
    fn server_too_large_triggers_one_half_budget_retry() {
        use crate::error::EngineErrorCode::ModelInputTooLarge;

        let mut long = String::from(
            "MASTER SERVICES AGREEMENT\n\nThis Master Services Agreement is effective as of March 1, 2025 by and between Acme Corporation and Contoso Ltd.\n\n",
        );
        for clause in 0..120 {
            long.push_str(&format!(
                "{clause}. The Supplier shall perform the services described in statement of work {clause} with due care, and the Customer shall pay each undisputed invoice within thirty days.\n\n"
            ));
        }
        let source = source_from_text(long);

        let recovering = std::sync::Arc::new(Recording::new(vec![Err(ModelInputTooLarge), Ok(())]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&recovering)));
        engine.analyze(&source, "pdf", &[]).unwrap();
        let prompts = recovering.prompts();
        assert_eq!(prompts.len(), 2);
        let (first, second) = (prompts[0].chars().count(), prompts[1].chars().count());
        let instructions =
            ModelRequest::from_digest(&distill(&source_from_text(""), DigestBudget::default()))
                .prompt
                .chars()
                .count();
        // Half the document, give or take a block and the date index.
        assert!(
            second - instructions < (first - instructions) * 6 / 10,
            "{first} -> {second}"
        );

        let refusing = std::sync::Arc::new(Recording::new(vec![Err(ModelInputTooLarge)]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&refusing)));
        let error = engine.analyze(&source, "pdf", &[]).unwrap_err();
        assert_eq!(error.code(), ModelInputTooLarge);
        assert_eq!(refusing.prompts().len(), 2, "one smaller retry, no more");

        // Any other failure is reported as it was, with no second attempt.
        let failing = std::sync::Arc::new(Recording::new(vec![Err(
            crate::error::EngineErrorCode::ModelRequestFailed,
        )]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&failing)));
        let error = engine.analyze(&source, "pdf", &[]).unwrap_err();
        assert_eq!(
            error.code(),
            crate::error::EngineErrorCode::ModelRequestFailed
        );
        assert_eq!(failing.prompts().len(), 1);
    }

    /// The same, end to end against llama-server's own 400: one smaller
    /// retry, then either an answer or MODEL_INPUT_TOO_LARGE - never the
    /// MODEL_REQUEST_FAILED that restarted the server and paused the queue.
    #[test]
    fn a_context_overflow_from_the_local_server_is_retried_smaller_once() {
        use crate::test_support::{VALID_REPLY, completion_reply, http_reply, scripted_server};

        let overflow = http_reply(
            "400 Bad Request",
            &[("Content-Type", "application/json")],
            r#"{"error":{"code":400,"message":"request (9214 tokens) exceeds the available context size (8192 tokens), try increasing it","type":"exceed_context_size_error","n_prompt_tokens":9214,"n_ctx":8192}}"#,
        );
        let source = digit_dense_statement();
        let endpoint = |server: &crate::test_support::ScriptedServer| {
            format!("http://{}/v1/chat/completions", server.address)
        };

        let server = scripted_server(vec![
            overflow.clone(),
            completion_reply("stop", VALID_REPLY),
        ]);
        let engine = Engine::new(ModelClient::new(&endpoint(&server), "k", "m").unwrap());
        engine
            .analyze(&source, "pdf", &[])
            .expect("the smaller retry is answered");
        assert_eq!(server.attempts(), 2);
        let bodies = server.bodies();
        assert!(bodies[1].len() < bodies[0].len());

        let server = scripted_server(vec![overflow]);
        let engine = Engine::new(ModelClient::new(&endpoint(&server), "k", "m").unwrap());
        let error = engine.analyze(&source, "pdf", &[]).unwrap_err();
        assert_eq!(
            error.code(),
            crate::error::EngineErrorCode::ModelInputTooLarge
        );
        assert_eq!(server.attempts(), 2);
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

    const SERVER_TIMINGS: crate::domain::ModelTimings = crate::domain::ModelTimings {
        prompt_tokens: 312,
        cached_tokens: 1_180,
        prefill_micros: 4_210_000,
        generated_tokens: 96,
        generation_micros: 13_400_000,
    };

    /// A proposer that reports server timings, as the local client does.
    struct Measured;

    impl Proposer for Measured {
        fn propose(&self, request: &ModelRequest) -> EngineResult<crate::domain::ModelProposal> {
            Scored(None).propose(request)
        }

        fn propose_measured(&self, request: &ModelRequest) -> EngineResult<ProposerReply> {
            Ok(ProposerReply {
                proposal: self.propose(request)?,
                token_confidence: None,
                timings: Some(SERVER_TIMINGS),
            })
        }
    }

    /// The analysis says what each stage after distillation cost, what was
    /// sent, and what the server reported - and none of it changes the
    /// answer.
    #[test]
    fn the_analysis_reports_what_each_stage_cost() {
        let measured = analyze_with(Engine::with_proposer(Box::new(Measured)));
        let unmeasured = analyze_with(Engine::with_proposer(Box::new(Scored(None))));

        let source = source_from_text(
            "CONSULTING AGREEMENT\n\nThis Agreement is effective as of April 1, 2026.\n\
             It is made between Acme Corporation and the consultant for advisory services.",
        );
        let prompt = ModelRequest::from_digest(&distill(&source, DigestBudget::default())).prompt;
        let telemetry = measured.telemetry;
        assert_eq!(telemetry.model, Some(SERVER_TIMINGS));
        assert_eq!(telemetry.prompt_characters, prompt.chars().count());
        assert_eq!(telemetry.estimated_prompt_tokens, estimated_tokens(&prompt));
        assert_eq!(
            telemetry.estimated_prompt_tokens,
            estimated_prompt_tokens(&prompt)
        );
        assert_eq!(telemetry.redistillations, 0);
        assert!(
            telemetry.prompt_micros + telemetry.validation_micros + telemetry.naming_micros > 0,
            "{telemetry:?}"
        );

        assert_eq!(unmeasured.telemetry.model, None);
        assert_eq!(measured.filename, unmeasured.filename);
        assert_eq!(measured.status, unmeasured.status);
        assert_eq!(measured.review_reasons, unmeasured.review_reasons);
        assert_eq!(measured.proposal, unmeasured.proposal);
    }

    /// Every distillation after the first is counted: the ones that fit the
    /// prompt to the local context before it is sent, and the one after the
    /// model says it did not fit.
    #[test]
    fn redistillations_are_counted() {
        use crate::error::EngineErrorCode::ModelInputTooLarge;

        let fitted = std::sync::Arc::new(Recording::new(vec![Ok(())]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&fitted)));
        let analysis = engine
            .analyze(&digit_dense_statement(), "pdf", &[])
            .unwrap();
        assert!(
            (1..=MAX_REDISTILLATIONS as u32).contains(&analysis.telemetry.redistillations),
            "{:?}",
            analysis.telemetry
        );

        // A hosted model is never fitted beforehand, so its one condensing
        // is the retry after it said the prompt was too large.
        let refitted =
            std::sync::Arc::new(Recording::hosted(vec![Err(ModelInputTooLarge), Ok(())]));
        let engine = Engine::with_proposer(Box::new(std::sync::Arc::clone(&refitted)));
        let analysis = engine
            .analyze(&digit_dense_statement(), "pdf", &[])
            .unwrap();
        assert_eq!(refitted.prompts().len(), 2);
        assert_eq!(analysis.telemetry.redistillations, 1);

        let plain = analyze_with(Engine::with_proposer(Box::new(Scored(None))));
        assert_eq!(plain.telemetry.redistillations, 0);
    }

    #[test]
    fn a_short_document_with_no_image_is_not_treated_as_unreadable() {
        // A one-line note is short but perfectly legible; only a page the parser
        // gave up on arrives with an image attached.
        assert!(!barely_readable(&source_from_text("Paid in full.")));
    }
}
