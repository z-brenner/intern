//! Stable input and output types for the document-understanding engine.
//!
//! Everything a caller needs to drive Intern headlessly lives here: a
//! [`DocumentSource`] goes in, a [`DocumentAnalysis`] comes out. The desktop
//! app, the CLI, and any future watched-folder or connector host share this
//! boundary, so the engine can change without changing its callers.

use serde::{Deserialize, Serialize};

use crate::retrieve::Tier;
use crate::structure::{PageLayout, TextSource};

/// Where a page's text came from.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageOrigin {
    /// Text drawn directly from the PDF content stream.
    Native,
    /// Text recovered by optical character recognition.
    Ocr,
    /// Markdown produced from an Office container.
    Office,
    /// A plain text or Markdown source file.
    PlainText,
}

/// One extracted page of a source document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourcePage {
    pub page_number: usize,
    pub text: String,
    pub origin: PageOrigin,
    pub ocr_confidence: Option<u32>,
    /// The page as blocks in reading order, as the parser worker built it
    /// (see [`crate::structure`]). Absent from pages stored before layouts
    /// existed and from pages built from plain text;
    /// [`crate::structure::structured`] segments those from their text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<PageLayout>,
}

impl SourcePage {
    pub fn new(page_number: usize, text: impl Into<String>, origin: PageOrigin) -> Self {
        Self {
            page_number,
            text: text.into(),
            origin,
            ocr_confidence: None,
            layout: None,
        }
    }
}

/// A rendered page image, supplied only when text extraction was inadequate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PageImage {
    pub page_number: usize,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

/// A non-fatal problem the parser reported about the extracted text.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParserWarning {
    pub code: String,
    /// Whether the warning can plausibly corrupt the facts the model reads.
    pub field_affecting: bool,
}

impl ParserWarning {
    pub fn new(code: impl Into<String>, field_affecting: bool) -> Self {
        Self {
            code: code.into(),
            field_affecting,
        }
    }
}

/// Everything the extraction stage produced for one document.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentSource {
    pub pages: Vec<SourcePage>,
    pub parser_warnings: Vec<ParserWarning>,
    /// Present only when a page could not be read as text at all.
    pub page_image: Option<PageImage>,
}

impl DocumentSource {
    pub fn from_pages(pages: Vec<SourcePage>) -> Self {
        Self {
            pages,
            parser_warnings: Vec::new(),
            page_image: None,
        }
    }

    pub fn character_count(&self) -> usize {
        self.pages
            .iter()
            .map(|page| page.text.chars().count())
            .sum()
    }
}

/// What a document-defining date actually means.
///
/// The grammar deliberately has no "due", "deadline", or "renewal" member: a
/// future obligation date must never become the filename date, so the model is
/// not given the vocabulary to propose one.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateRole {
    Effective,
    Execution,
    Notice,
    Termination,
    Amendment,
    Invoice,
    Filing,
    Issuance,
    Other,
}

impl DateRole {
    pub const ALL: [Self; 9] = [
        Self::Effective,
        Self::Execution,
        Self::Notice,
        Self::Termination,
        Self::Amendment,
        Self::Invoice,
        Self::Filing,
        Self::Issuance,
        Self::Other,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Effective => "effective",
            Self::Execution => "execution",
            Self::Notice => "notice",
            Self::Termination => "termination",
            Self::Amendment => "amendment",
            Self::Invoice => "invoice",
            Self::Filing => "filing",
            Self::Issuance => "issuance",
            Self::Other => "other",
        }
    }
}

/// How the defining parties attach to the document type in a filename.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PartyRelation {
    Between,
    For,
    With,
    From,
    To,
    #[default]
    None,
}

impl PartyRelation {
    pub const ALL: [Self; 6] = [
        Self::Between,
        Self::For,
        Self::With,
        Self::From,
        Self::To,
        Self::None,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Between => "between",
            Self::For => "for",
            Self::With => "with",
            Self::From => "from",
            Self::To => "to",
            Self::None => "none",
        }
    }
}

/// Verbatim excerpts that must be found in the distilled document before the
/// corresponding fact is allowed to reach a filename.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Evidence {
    pub date: Option<String>,
    pub document_type: Option<String>,
    pub parties: Vec<String>,
}

/// Raw, unvalidated model output.
///
/// A reply in the evidence pipeline ([`crate::engine::Pipeline::Evidence`])
/// carries its facts and the evidence ids behind them in `facts`, and fills
/// the fields above it from those facts so everything that reads a stored
/// proposal keeps working: the type, date, role and party names as replied,
/// the relation derived from the replied roles, no description (the engine
/// composes it), and as evidence the text of the first unit each field
/// cited.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ModelProposal {
    pub document_type: Option<String>,
    pub document_date: Option<String>,
    pub date_role: Option<DateRole>,
    pub parties: Vec<String>,
    pub party_relation: PartyRelation,
    pub description: String,
    pub confidence: f32,
    pub needs_review: bool,
    pub evidence: Evidence,
    /// The facts and evidence ids of an evidence-pipeline reply. Absent from
    /// a digest-pipeline reply, and from every proposal stored before the
    /// evidence pipeline existed.
    /// Boxed: a proposal is cloned and stored often, and the digest
    /// pipeline's never has facts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facts: Option<Box<ModelFacts>>,
}

/// A semantic role a party plays in a document.
///
/// The model is never made to choose one: a role the evidence does not
/// state is left out, and [`PartyRole::Other`] is the model saying the
/// party has none of these. Only a role validation found support for
/// decides how the parties read in a filename ([`crate::compose`]).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PartyRole {
    Client,
    Contractor,
    Employer,
    Employee,
    Buyer,
    Seller,
    Landlord,
    Tenant,
    Issuer,
    Recipient,
    Vendor,
    Customer,
    Borrower,
    Lender,
    Licensor,
    Licensee,
    Sender,
    Addressee,
    Other,
}

impl PartyRole {
    pub const ALL: [Self; 19] = [
        Self::Client,
        Self::Contractor,
        Self::Employer,
        Self::Employee,
        Self::Buyer,
        Self::Seller,
        Self::Landlord,
        Self::Tenant,
        Self::Issuer,
        Self::Recipient,
        Self::Vendor,
        Self::Customer,
        Self::Borrower,
        Self::Lender,
        Self::Licensor,
        Self::Licensee,
        Self::Sender,
        Self::Addressee,
        Self::Other,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Contractor => "contractor",
            Self::Employer => "employer",
            Self::Employee => "employee",
            Self::Buyer => "buyer",
            Self::Seller => "seller",
            Self::Landlord => "landlord",
            Self::Tenant => "tenant",
            Self::Issuer => "issuer",
            Self::Recipient => "recipient",
            Self::Vendor => "vendor",
            Self::Customer => "customer",
            Self::Borrower => "borrower",
            Self::Lender => "lender",
            Self::Licensor => "licensor",
            Self::Licensee => "licensee",
            Self::Sender => "sender",
            Self::Addressee => "addressee",
            Self::Other => "other",
        }
    }

    /// The role a reply named, or `None` for a word that is not one: an
    /// unrecognised role is no role, never [`PartyRole::Other`].
    pub fn parse(word: &str) -> Option<Self> {
        let word = word.trim();
        Self::ALL
            .into_iter()
            .find(|role| role.as_str().eq_ignore_ascii_case(word))
    }
}

/// One party a reply named, with its role and the evidence ids it cited.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PartyFact {
    pub name: String,
    #[serde(default)]
    pub role: Option<PartyRole>,
    #[serde(default)]
    pub evidence: Vec<String>,
}

/// One key fact a reply named - an amount due, a term - with its evidence.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct KeyFact {
    pub fact: String,
    #[serde(default)]
    pub evidence: Vec<String>,
}

/// What an evidence-pipeline reply says, each fact with the ids of the
/// evidence units it cited.
///
/// Ids are always the units' stable ids ([`crate::index::EvidenceUnit::id`]),
/// whatever the prompt showed: a prompt-local handle is mapped back before
/// anything is stored. An id the prompt did not show is never evidence; it
/// is kept in `unknown_evidence` for the record and counted.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModelFacts {
    #[serde(default)]
    pub document_type: Option<String>,
    #[serde(default)]
    pub type_evidence: Vec<String>,
    #[serde(default)]
    pub document_date: Option<String>,
    #[serde(default)]
    pub date_role: Option<DateRole>,
    #[serde(default)]
    pub date_evidence: Vec<String>,
    #[serde(default)]
    pub parties: Vec<PartyFact>,
    /// The subject or purpose: the work, goods, premises, loan, project or
    /// matter, or the transaction the document records.
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub subject_evidence: Vec<String>,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub identifier_evidence: Vec<String>,
    #[serde(default)]
    pub key_facts: Vec<KeyFact>,
    /// The line a compact reply says states the main amount; the amount
    /// itself is read from it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amount_evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_evidence: Vec<String>,
}

impl ModelFacts {
    /// Every evidence id the reply cited, field by field, in reply order.
    pub fn cited_ids(&self) -> impl Iterator<Item = &str> {
        self.type_evidence
            .iter()
            .chain(&self.date_evidence)
            .chain(self.parties.iter().flat_map(|party| &party.evidence))
            .chain(&self.subject_evidence)
            .chain(&self.identifier_evidence)
            .chain(self.key_facts.iter().flat_map(|fact| &fact.evidence))
            .chain(&self.amount_evidence)
            .map(String::as_str)
    }
}

/// Where validation found a fact.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Support {
    /// In a unit the reply cited for it.
    Cited,
    /// Not in a cited unit, but elsewhere in what the model was shown - the
    /// support today's digest pipeline accepts.
    Context,
    /// Nowhere the model was shown.
    Unsupported,
    /// The reply did not state it.
    #[default]
    Absent,
}

impl Support {
    pub fn is_absent(&self) -> bool {
        *self == Self::Absent
    }
}

/// A line of the document shown to a reviewer as the evidence for a fact:
/// the text of a unit, dereferenced by the engine, never written by the
/// model.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceRef {
    pub id: String,
    pub page: usize,
    pub text: String,
    #[serde(default)]
    pub source: TextSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
}

/// A party validation kept, with how its name and its role were supported.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatedParty {
    pub name: String,
    /// The role the document supports: the reply's when its `role_support`
    /// is [`Support::Cited`] or [`Support::Context`], else the
    /// `document_role`, else none - an unsupported role is never kept.
    /// Someone the document only copies in is [`PartyRole::Other`].
    #[serde(default)]
    pub role: Option<PartyRole>,
    /// The role the reply gave, supported or not, for the record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_role: Option<PartyRole>,
    /// How the document supports `proposed_role`.
    #[serde(default)]
    pub role_support: Support,
    /// The role the document's own wording gives the party - `Resident:`
    /// before the name, `("Tenant")` after it - when the reply gave it no
    /// role the document supports. It decides the relation in the reply's
    /// role's place; an unsupported role never does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_role: Option<PartyRole>,
    /// Named only on a "cc:" line: a bystander, never a filename's party.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub copied: bool,
    /// A person who signs for an organisation that is a party - "Harriet
    /// Voss, Vice President of People Operations, Northstar Lantern Works
    /// LLC" - and so is not a party of their own.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub signatory: bool,
    #[serde(default)]
    pub support: Support,
    /// The stable ids of the units that state the name.
    #[serde(default)]
    pub evidence: Vec<String>,
}

/// How well each fact of a reply was supported.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactSupport {
    #[serde(default)]
    pub document_type: Support,
    #[serde(default)]
    pub document_date: Support,
    #[serde(default)]
    pub parties: Vec<Support>,
    #[serde(default)]
    pub subject: Support,
    #[serde(default)]
    pub identifier: Support,
    #[serde(default)]
    pub key_facts: Vec<Support>,
    /// The amount a compact reply's cited line states.
    #[serde(default, skip_serializing_if = "Support::is_absent")]
    pub amount: Support,
    /// Cited ids the prompt never showed. Never evidence.
    #[serde(default)]
    pub unknown_ids: u32,
    /// Cited ids that were shown but do not state the fact they were cited
    /// for. Never evidence.
    #[serde(default)]
    pub miscited_ids: u32,
    /// Which view fired a date guard - `"context"`, `"document"` or
    /// `"both"` - when one did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard_scope: Option<String>,
}

impl FactSupport {
    /// Facts the reply stated, and how many of them nothing supported.
    pub fn proposed_and_unsupported(&self) -> (u32, u32) {
        let mut proposed = 0;
        let mut unsupported = 0;
        let scalars = [
            self.document_type,
            self.document_date,
            self.subject,
            self.identifier,
            self.amount,
        ];
        for support in scalars.iter().chain(&self.parties).chain(&self.key_facts) {
            if *support != Support::Absent {
                proposed += 1;
            }
            if *support == Support::Unsupported {
                unsupported += 1;
            }
        }
        (proposed, unsupported)
    }

    /// Accepted facts, and how many of them a cited unit supported.
    pub fn accepted_and_cited(&self) -> (u32, u32) {
        let mut accepted = 0;
        let mut cited = 0;
        let scalars = [
            self.document_type,
            self.document_date,
            self.subject,
            self.identifier,
            self.amount,
        ];
        for support in scalars.iter().chain(&self.parties).chain(&self.key_facts) {
            if matches!(support, Support::Cited | Support::Context) {
                accepted += 1;
            }
            if *support == Support::Cited {
                cited += 1;
            }
        }
        (accepted, cited)
    }
}

/// What a document is, for deciding how its parties read in a filename and
/// how its description is put together ([`crate::compose`]).
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentClass {
    Agreement,
    Amendment,
    /// Issued by one party to another: invoices, receipts, orders, quotes,
    /// statements.
    Issued,
    Notice,
    Letter,
    Email,
    Form,
    Record,
    #[default]
    Unknown,
}

impl DocumentClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Agreement => "agreement",
            Self::Amendment => "amendment",
            Self::Issued => "issued",
            Self::Notice => "notice",
            Self::Letter => "letter",
            Self::Email => "email",
            Self::Form => "form",
            Self::Record => "record",
            Self::Unknown => "unknown",
        }
    }
}

/// The facts of an evidence-pipeline reply after validation, and the
/// support each one had.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ValidatedFacts {
    /// Every party that was supported, an invoice's customer included;
    /// [`ValidatedProposal::parties`] holds only the filename's.
    #[serde(default)]
    pub parties: Vec<ValidatedParty>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub key_facts: Vec<String>,
    #[serde(default)]
    pub document_class: DocumentClass,
    /// Why the filename's parties read the way they do, for a reviewer
    /// and for tuning: the rule of the relation table that applied.
    #[serde(default)]
    pub relation_basis: String,
    #[serde(default)]
    pub support: FactSupport,
    /// The dereferenced lines behind the accepted facts.
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
}

/// How probable the model found its own date and party tokens.
///
/// Read from the token probabilities the local server reports alongside a
/// reply, over the characters of the `document_date` and `parties` values
/// only - never the JSON scaffolding the grammar forced. The probabilities
/// are the model's own, before the grammar masked anything, so a value the
/// model would rather not have written shows up here even though the grammar
/// made it well-formed. A signal next to the verbatim checks in
/// [`crate::validate`], never a replacement for them.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenConfidence {
    /// The least probable value token.
    pub min: f32,
    /// The mean probability over the value tokens.
    pub mean: f32,
    /// How many tokens the figures were taken over.
    pub tokens: u32,
}

/// Model output after evidence, format, and calibration checks.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ValidatedProposal {
    pub document_type: Option<String>,
    pub document_date: Option<String>,
    pub date_role: Option<DateRole>,
    pub parties: Vec<String>,
    pub party_relation: PartyRelation,
    pub description: String,
    pub confidence: f32,
    pub evidence: Evidence,
}

/// Whether a proposal can be applied without a human looking at it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Ready,
    NeedsReview,
}

/// Why a proposal was routed to review.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewReason {
    /// No usable document-defining date was found.
    DateMissing,
    /// The date was not literally present in the document.
    DateUnsupported,
    /// No specific document type was identified.
    TypeMissing,
    /// The document type was not literally present in the document.
    TypeUnsupported,
    /// The model gave no usable type, so the document's own title was used;
    /// a person should confirm it names the document.
    TypeInferred,
    /// A named party could not be found in the document.
    PartyUnsupported,
    /// The description asserted something the document does not contain.
    DescriptionUnsupported,
    /// The description was not a single usable sentence.
    DescriptionInvalid,
    /// The model reported low confidence.
    LowConfidence,
    /// The model asked for review itself.
    ModelRequestedReview,
    /// Extraction reported a problem that can corrupt the read facts.
    ParserWarning,
    /// The accepted date's year is implausible for a document (an OCR
    /// misread such as 2625, or a year before 1900).
    DateImplausible,
    /// The date the model chose is labelled in the document as a due,
    /// renewal, or expiry date rather than the document's own date.
    DateIsDeadline,
    /// The date is written only as numbers whose day and month could be
    /// read either way round, and nothing in the document settles which.
    DateAmbiguous,
}

impl ReviewReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DateMissing => "DATE_MISSING",
            Self::DateUnsupported => "DATE_UNSUPPORTED",
            Self::TypeMissing => "TYPE_MISSING",
            Self::TypeUnsupported => "TYPE_UNSUPPORTED",
            Self::TypeInferred => "TYPE_INFERRED",
            Self::PartyUnsupported => "PARTY_UNSUPPORTED",
            Self::DescriptionUnsupported => "DESCRIPTION_UNSUPPORTED",
            Self::DescriptionInvalid => "DESCRIPTION_INVALID",
            Self::LowConfidence => "LOW_CONFIDENCE",
            Self::ModelRequestedReview => "MODEL_REQUESTED_REVIEW",
            Self::ParserWarning => "PARSER_WARNING",
            Self::DateImplausible => "DATE_IMPLAUSIBLE",
            Self::DateIsDeadline => "DATE_IS_DEADLINE",
            Self::DateAmbiguous => "DATE_AMBIGUOUS",
        }
    }
}

/// The result of validating one model proposal.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ValidationOutcome {
    pub proposal: ValidatedProposal,
    pub status: ProposalStatus,
    pub reasons: Vec<ReviewReason>,
    /// The reply exactly as the model gave it, before any check. What
    /// validation withheld from `proposal` is still here for a reviewer to
    /// be offered - a date the document did not state verbatim, say.
    pub candidate: ModelProposal,
    /// The validated facts of an evidence-pipeline reply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facts: Option<ValidatedFacts>,
}

/// A composed filename and the collision suffix it needed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ComposedName {
    pub value: String,
    pub collision_index: u32,
}

/// Local-only measurements for one analysis. Never leaves the machine.
///
/// Every field after `inference_millis` arrived later than the analyses a
/// queue may already hold, so each reads as zero, or absent, from one stored
/// without it.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisTelemetry {
    pub source_characters: usize,
    pub digest_characters: usize,
    pub compression_ratio: f32,
    pub distill_micros: u64,
    /// Wall time of the model request, as the engine waited for it.
    pub inference_millis: u64,
    /// Building the prompt from the digest.
    #[serde(default)]
    pub prompt_micros: u64,
    /// Validating the reply, and the readability and token-confidence gates
    /// after it.
    #[serde(default)]
    pub validation_micros: u64,
    /// Composing the filename, the description and the stated dates, and
    /// fingerprinting the text.
    #[serde(default)]
    pub naming_micros: u64,
    /// Distillations made after the first, to fit the prompt to the model's
    /// context or after the model said it did not fit.
    #[serde(default)]
    pub redistillations: u32,
    /// Characters of the user turn sent to the model.
    #[serde(default)]
    pub prompt_characters: usize,
    /// The engine's own estimate of the tokens that user turn costs.
    #[serde(default)]
    pub estimated_prompt_tokens: usize,
    /// What the model server said about the request, when it said anything:
    /// the local server does, a hosted service does not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelTimings>,
    /// Building the evidence index (evidence pipeline only; `distill_micros`
    /// then holds index and retrieval together). Like the three after it,
    /// written only when it is not zero, so the digest pipeline's analyses
    /// are written exactly as before.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub index_micros: u64,
    /// Choosing the evidence the prompt carries (evidence pipeline only).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub retrieval_micros: u64,
    /// Units in the evidence index.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub index_units: u32,
    /// Units the prompt carried.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub context_units: u32,
    /// How much of the document the prompt carried. Absent for the digest
    /// pipeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_tier: Option<Tier>,
}

fn is_zero<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// How the model server spent one request, in its own words.
///
/// llama-server reports this with every reply. Its prefill counts only the
/// tokens it had to evaluate: a prompt that begins as the previous one did
/// reuses that prefix from the slot's cache, and the reused tokens cost
/// nothing. The system turn is always reused. The user turn's fixed
/// instructions are reused only when the previous document was likewise
/// condensed or complete: `build_prompt` puts the sentence saying which in
/// front of them, so a complete document after a condensed one, or the
/// reverse, evaluates the whole instruction block again and `cached_tokens`
/// falls (see docs/pipeline-bottlenecks.md, Latency item 2).
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTimings {
    /// Prompt tokens the server evaluated.
    pub prompt_tokens: u64,
    /// Prompt tokens reused from the slot's cache rather than evaluated.
    pub cached_tokens: u64,
    /// Time spent evaluating `prompt_tokens`.
    pub prefill_micros: u64,
    /// Tokens of reply generated.
    pub generated_tokens: u64,
    /// Time spent generating them.
    pub generation_micros: u64,
}

/// Everything Intern knows about one document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentAnalysis {
    pub filename: String,
    pub description: String,
    pub status: ProposalStatus,
    pub review_reasons: Vec<ReviewReason>,
    pub proposal: ValidatedProposal,
    pub telemetry: AnalysisTelemetry,
    /// The model's reply before validation. Absent for analyses stored by
    /// versions that did not keep it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_proposal: Option<ModelProposal>,
    /// Every date the document states, in first-mention order, for a
    /// reviewer who must give the document a date the model did not.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stated_dates: Vec<String>,
    /// A fingerprint of everything the extractor read (see
    /// [`crate::fingerprint`]), for telling a second scan or a re-export of
    /// a filed document from a new one. Absent for a text too short to
    /// fingerprint, and for analyses stored by versions that kept none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_fingerprint: Option<String>,
    /// How probable the model found its own date and party tokens. Absent
    /// when the model reported no token probabilities: a hosted model, a
    /// reply replayed from a recording made without them, or a client that
    /// did not ask for them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_confidence: Option<TokenConfidence>,
    /// The validated facts and their support, for an analysis made by the
    /// evidence pipeline. Absent otherwise, and from analyses stored before
    /// it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facts: Option<ValidatedFacts>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A queue holds analyses stored before the stage timings existed. They
    /// still read, the new figures as zero and no model timings, and an
    /// analysis without model timings is written without the key.
    #[test]
    fn telemetry_stored_before_stage_timings_still_reads() {
        let stored: AnalysisTelemetry = serde_json::from_str(
            r#"{"sourceCharacters":1200,"digestCharacters":900,"compressionRatio":0.75,
                "distillMicros":310,"inferenceMillis":14000}"#,
        )
        .unwrap();

        assert_eq!(
            stored,
            AnalysisTelemetry {
                source_characters: 1_200,
                digest_characters: 900,
                compression_ratio: 0.75,
                distill_micros: 310,
                inference_millis: 14_000,
                ..AnalysisTelemetry::default()
            }
        );
        assert_eq!(stored.model, None);
        let written = serde_json::to_value(stored).unwrap();
        assert!(written.get("model").is_none(), "{written}");
        assert_eq!(written["promptMicros"], 0);
        assert_eq!(written["estimatedPromptTokens"], 0);

        let measured = AnalysisTelemetry {
            redistillations: 1,
            prompt_characters: 5_400,
            model: Some(ModelTimings {
                prompt_tokens: 4,
                cached_tokens: 1_210,
                prefill_micros: 206_870,
                generated_tokens: 115,
                generation_micros: 16_131_292,
            }),
            ..stored
        };
        let written = serde_json::to_value(measured).unwrap();
        assert_eq!(written["model"]["cachedTokens"], 1_210);
        assert_eq!(written["model"]["generationMicros"], 16_131_292);
        assert_eq!(written["redistillations"], 1);
        assert_eq!(
            serde_json::from_value::<AnalysisTelemetry>(written).unwrap(),
            measured
        );
    }

    /// Everything the evidence pipeline added is optional: a proposal, an
    /// outcome and an analysis stored before it read as they did, and a
    /// digest-pipeline proposal is written exactly as before.
    #[test]
    fn analyses_stored_before_the_evidence_pipeline_still_read() {
        let proposal: ModelProposal = serde_json::from_str(
            r#"{"document_type":"Invoice","document_date":"2025-05-01","date_role":"invoice",
                "parties":["Acme"],"party_relation":"from","description":"An invoice from Acme.",
                "confidence":0.9,"needs_review":false,
                "evidence":{"date":"Invoice Date: May 1, 2025","document_type":"INVOICE","parties":["Acme"]}}"#,
        )
        .unwrap();
        assert_eq!(proposal.facts, None);
        let written = serde_json::to_value(&proposal).unwrap();
        assert!(written.get("facts").is_none(), "{written}");

        let analysis: DocumentAnalysis = serde_json::from_value(serde_json::json!({
            "filename": "2025-05-01 Invoice from Acme.pdf",
            "description": "An invoice from Acme.",
            "status": "ready",
            "reviewReasons": [],
            "proposal": {"document_type": "Invoice", "document_date": "2025-05-01",
                "date_role": "invoice", "parties": ["Acme"], "party_relation": "from",
                "description": "An invoice from Acme.", "confidence": 0.9,
                "evidence": {"date": null, "document_type": null, "parties": []}},
            "telemetry": {"sourceCharacters": 1200, "digestCharacters": 900,
                "compressionRatio": 0.75, "distillMicros": 310, "inferenceMillis": 14000},
            "modelProposal": written,
        }))
        .unwrap();
        assert_eq!(analysis.facts, None);
        assert_eq!(analysis.telemetry.context_tier, None);
        assert_eq!(analysis.telemetry.index_units, 0);
        let rewritten = serde_json::to_value(&analysis).unwrap();
        assert!(rewritten.get("facts").is_none());
        for key in [
            "contextTier",
            "indexMicros",
            "retrievalMicros",
            "indexUnits",
            "contextUnits",
        ] {
            assert!(rewritten["telemetry"].get(key).is_none(), "{key}");
        }

        let outcome: ValidationOutcome = serde_json::from_value(serde_json::json!({
            "proposal": analysis.proposal,
            "status": "ready",
            "reasons": [],
            "candidate": proposal,
        }))
        .unwrap();
        assert_eq!(outcome.facts, None);
    }

    /// An evidence-pipeline analysis round-trips, its facts and support
    /// included, and a reader that knows only version 1 still reads it.
    #[test]
    fn an_evidence_analysis_round_trips() {
        let facts = ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec!["p1.b2".into()],
            parties: vec![PartyFact {
                name: "Acme".into(),
                role: Some(PartyRole::Issuer),
                evidence: vec!["p1.b1".into()],
            }],
            unknown_evidence: vec!["p9.b9".into()],
            ..ModelFacts::default()
        };
        let proposal = ModelProposal {
            facts: Some(Box::new(facts.clone())),
            ..ModelProposal::default()
        };
        let read: ModelProposal =
            serde_json::from_value(serde_json::to_value(&proposal).unwrap()).unwrap();
        assert_eq!(read, proposal);
        let validated = ValidatedFacts {
            parties: vec![ValidatedParty {
                name: "Acme".into(),
                role: Some(PartyRole::Issuer),
                proposed_role: None,
                role_support: Support::Cited,
                support: Support::Cited,
                evidence: vec!["p1.b1".into()],
                document_role: None,
                copied: false,
                signatory: false,
            }],
            document_class: DocumentClass::Issued,
            support: FactSupport {
                document_type: Support::Context,
                unknown_ids: 1,
                ..FactSupport::default()
            },
            ..ValidatedFacts::default()
        };
        let telemetry = AnalysisTelemetry {
            index_units: 40,
            context_units: 12,
            context_tier: Some(Tier::Normal),
            ..AnalysisTelemetry::default()
        };
        let written = serde_json::to_value(telemetry).unwrap();
        assert_eq!(written["contextTier"], "normal");
        assert_eq!(
            serde_json::from_value::<AnalysisTelemetry>(written).unwrap(),
            telemetry
        );
        let written = serde_json::to_value(&validated).unwrap();
        assert_eq!(written["support"]["document_type"], "context");
        assert_eq!(written["parties"][0]["role"], "issuer");
        assert_eq!(
            serde_json::from_value::<ValidatedFacts>(written).unwrap(),
            validated
        );
        /// A version-1 reader: the fields it knows, nothing refused.
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct OldProposal {
            document_type: Option<String>,
            parties: Vec<String>,
            confidence: f32,
        }
        let old: OldProposal =
            serde_json::from_value(serde_json::to_value(&proposal).unwrap()).unwrap();
        assert_eq!(old.document_type, None);
    }

    #[test]
    fn a_role_is_read_only_from_its_own_words() {
        assert_eq!(PartyRole::parse("Tenant"), Some(PartyRole::Tenant));
        assert_eq!(PartyRole::parse(" other "), Some(PartyRole::Other));
        assert_eq!(PartyRole::parse("resident"), None);
        for role in PartyRole::ALL {
            assert_eq!(PartyRole::parse(role.as_str()), Some(role));
            assert_eq!(
                serde_json::to_value(role).unwrap(),
                serde_json::json!(role.as_str())
            );
        }
    }
}
