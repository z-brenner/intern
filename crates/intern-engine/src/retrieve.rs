//! Evidence for each field the filename and description need, retrieved
//! from an [`EvidenceIndex`] field by field, each within its own budget.
//!
//! Six fields are retrieved independently: the document's type, its
//! defining date, its parties and their roles, its subject, its own
//! number, and the key facts a description states. Each field scores every
//! unit by a weighted sum of six components - the cue logic distillation
//! has always used, BM25 against a lexicon for the field, structure (a
//! heading, a labelled value, a letterhead, a signature block, a table's
//! header), heading proximity (a section about definitions, the parties,
//! the scope), position in the document, and the dates and names read off
//! the unit - and takes units greedily by score until its budget of
//! estimated tokens is spent. The context is the union, in document order,
//! one `[id] text` line per unit. A field can therefore never crowd another
//! out: a dozen candidate dates cannot push the parties out of the prompt,
//! and parties cannot push out the subject.
//!
//! Each unit taken may bring the context it needs to be understood - its
//! table's header row, the heading over it, the line after a label that
//! stands alone - paid from the field's budget and capped. Repeats are
//! dropped by the keys distillation drops them by, and a date is carried by
//! at most a couple of units, so a statement with twenty-five dated rows
//! cannot fill the date budget with one kind of date.
//!
//! A document short enough to send whole is sent whole, still as `[id]
//! text` lines. A long document can be retrieved hierarchically: sections
//! are scored per field first, and units only within the best sections, the
//! first page and the signature section.
//!
//! Every weight and budget is an integer, BM25 included, so the same index
//! and configuration always give the same context, byte for byte.

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{AssertUnwindSafe, catch_unwind};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cues::{PARTY_LABELS, TYPE_NOUNS};
use crate::distill::duplicate_keys;
use crate::domain::DocumentSource;
use crate::engine::estimated_tokens;
use crate::error::{EngineError, EngineErrorCode, EngineResult};
use crate::evidence::normalize;
use crate::index::{
    EvidenceIndex, EvidenceUnit, IndexOptions, UnitKind, collapse_whitespace, tag, terms_of,
};
use crate::structure::TextSource;

/// A field evidence is retrieved for.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    DocumentType,
    Date,
    Parties,
    Subject,
    Identifier,
    KeyFacts,
}

impl Field {
    pub const ALL: [Self; 6] = [
        Self::DocumentType,
        Self::Date,
        Self::Parties,
        Self::Subject,
        Self::Identifier,
        Self::KeyFacts,
    ];

    /// The field's bit in [`EvidenceContext::selected_by`].
    pub const fn bit(self) -> u8 {
        1 << (self as u8)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DocumentType => "document_type",
            Self::Date => "date",
            Self::Parties => "parties",
            Self::Subject => "subject",
            Self::Identifier => "identifier",
            Self::KeyFacts => "key_facts",
        }
    }

    /// The words BM25 looks for, per field.
    fn lexicon(self) -> &'static [&'static str] {
        match self {
            Self::DocumentType => TYPE_NOUNS,
            Self::Date => &[
                "effective",
                "dated",
                "date",
                "commencement",
                "invoice",
                "notice",
                "issued",
                "executed",
                "signed",
                "closing",
                "means",
                "made",
                "entered",
            ],
            Self::Parties => &[
                "between",
                "among",
                "inc",
                "llc",
                "ltd",
                "corporation",
                "company",
                "bill",
                "sold",
                "remit",
                "attn",
                "dear",
                "landlord",
                "tenant",
                "employer",
                "employee",
                "borrower",
                "lender",
                "client",
                "customer",
                "vendor",
                "contractor",
                "buyer",
                "seller",
                "licensor",
                "licensee",
                "party",
                "parties",
            ],
            Self::Subject => &[
                "services",
                "scope",
                "project",
                "purpose",
                "deliverables",
                "description",
                "premises",
                "property",
                "goods",
                "work",
                "loan",
                "facility",
                "regarding",
                "subject",
            ],
            Self::Identifier => &[
                "no",
                "number",
                "invoice",
                "order",
                "policy",
                "account",
                "reference",
                "case",
                "loan",
                "ref",
                "id",
            ],
            Self::KeyFacts => &[
                "total",
                "amount",
                "due",
                "balance",
                "principal",
                "commitment",
                "rent",
                "price",
                "fee",
                "term",
                "period",
            ],
        }
    }
}

/// A unit in the context for another unit's sake: its header row, its
/// heading, the line after it.
pub const EXPANSION_BIT: u8 = 1 << 6;
/// A unit in the context because the whole document is.
pub const WHOLE_BIT: u8 = 1 << 7;

/// How much of a document the prompt carries.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// Every unit: the document fits.
    Whole,
    Small,
    Normal,
    /// Larger budgets, for a document whose evidence is crowded.
    Dense,
}

impl Tier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Whole => "whole",
            Self::Small => "small",
            Self::Normal => "normal",
            Self::Dense => "dense",
        }
    }
}

/// Which tier a document that does not fit whole gets.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TierPolicy {
    /// Dense when a trigger fires (see [`RetrievalConfig::dense_triggers`]),
    /// normal otherwise.
    #[default]
    Auto,
    Small,
    Normal,
    Dense,
}

/// How units are named in the prompt.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdStyle {
    /// The unit's own id: `[p3.b7.r2]`.
    #[default]
    Stable,
    /// A number counting from 1 in the prompt, with `--- page N ---` lines.
    Ordinal,
}

/// Whether units are scored across the document or within its best
/// sections.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Flat,
    Hierarchical,
    /// Hierarchical from [`RetrievalConfig::hierarchical_min_units`] units.
    #[default]
    Auto,
}

/// Each field's budget, in estimated tokens, at the normal tier.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct FieldBudgets {
    pub document_type: u32,
    pub date: u32,
    pub parties: u32,
    pub subject: u32,
    pub identifier: u32,
    pub key_facts: u32,
}

impl Default for FieldBudgets {
    fn default() -> Self {
        Self {
            document_type: 120,
            date: 450,
            parties: 450,
            subject: 300,
            identifier: 120,
            key_facts: 200,
        }
    }
}

impl FieldBudgets {
    pub fn of(&self, field: Field) -> u32 {
        match field {
            Field::DocumentType => self.document_type,
            Field::Date => self.date,
            Field::Parties => self.parties,
            Field::Subject => self.subject,
            Field::Identifier => self.identifier,
            Field::KeyFacts => self.key_facts,
        }
    }

    pub fn total(&self) -> u32 {
        Field::ALL.iter().map(|field| self.of(*field)).sum()
    }
}

/// How much each scoring component counts, in percent: 100 as designed, 0
/// off. The benchmark's ablations turn them off one at a time.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Weights {
    /// The cue lists distillation scores blocks by.
    pub cues: u32,
    /// BM25 against each field's lexicon.
    pub bm25: u32,
    /// Headings, labelled values, letterheads, signature blocks, tables.
    pub structure: u32,
    /// What the headings above a unit say its section is about.
    pub heading: u32,
    /// The opening and closing of the document, its first and last page.
    pub position: u32,
    /// Dates with their roles, organisations, people, identifiers, money.
    pub features: u32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            cues: 100,
            bm25: 100,
            structure: 100,
            heading: 100,
            position: 100,
            features: 100,
        }
    }
}

/// Everything retrieval is tuned by. Integers only, so a configuration is
/// one fingerprint and one behaviour.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct RetrievalConfig {
    /// A document whose units cost at most this many estimated tokens goes
    /// whole.
    pub whole_document_tokens: u32,
    pub budgets: FieldBudgets,
    /// The small and dense tiers' budgets, in percent of the normal tier's.
    pub small_pct: u32,
    pub dense_pct: u32,
    /// How much of a field's budget may go on context for its units.
    pub expansion_pct: u32,
    /// Paragraphs longer than this are split into sentence chunks when the
    /// index is built ([`RetrievalConfig::index_options`]).
    pub max_unit_chars: u32,
    pub strategy: Strategy,
    pub hierarchical_min_units: u32,
    pub sections_per_field: u8,
    pub dense_sections_per_field: u8,
    /// At most this many units carry any one date.
    pub units_per_date: u8,
    pub weights: Weights,
    pub id_style: IdStyle,
    pub tier: TierPolicy,
    /// Whether [`TierPolicy::Auto`] goes dense for crowded evidence: many
    /// candidate dates scored alike, many organisations, poor OCR.
    pub dense_triggers: bool,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            whole_document_tokens: 1_800,
            budgets: FieldBudgets::default(),
            small_pct: 60,
            dense_pct: 175,
            expansion_pct: 25,
            max_unit_chars: 400,
            strategy: Strategy::Auto,
            hierarchical_min_units: 300,
            sections_per_field: 3,
            dense_sections_per_field: 5,
            units_per_date: 2,
            weights: Weights::default(),
            id_style: IdStyle::Stable,
            tier: TierPolicy::Auto,
            dense_triggers: true,
        }
    }
}

impl RetrievalConfig {
    /// SHA-256 of the configuration's canonical JSON: what a recording
    /// stores so that a change of configuration is a change it can see.
    pub fn fingerprint(&self) -> String {
        let canonical = serde_json::to_vec(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&canonical);
        format!("{:x}", hasher.finalize())
    }

    /// The index this configuration retrieves from is built with.
    pub fn index_options(&self) -> IndexOptions {
        IndexOptions {
            max_unit_characters: self.max_unit_chars.max(50) as usize,
        }
    }
}

/// What retrieval chose.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceContext {
    pub tier: Tier,
    /// Whether units were scored within their best sections.
    pub hierarchical: bool,
    /// The units sent, in document order.
    pub units: Vec<u32>,
    /// The handle each unit carries in the prompt.
    pub handles: Vec<(String, u32)>,
    /// Which fields chose each unit ([`Field::bit`]), and whether it came
    /// as context ([`EXPANSION_BIT`]) or with the whole document
    /// ([`WHOLE_BIT`]).
    pub selected_by: BTreeMap<u32, u8>,
    /// The lines the prompt carries.
    pub text: String,
    pub estimated_tokens: usize,
    pub characters: usize,
}

impl EvidenceContext {
    /// The units a field chose for itself, in document order.
    pub fn chosen_for(&self, field: Field) -> Vec<u32> {
        self.selected_by
            .iter()
            .filter(|(_, bits)| *bits & field.bit() != 0)
            .map(|(unit, _)| *unit)
            .collect()
    }
}

/// The index and the context for a document, built where a panic in
/// either (a slice inside a character the tests did not foresee) is an
/// analysis that failed rather than a model thread that died.
pub fn prepare_evidence(
    source: &DocumentSource,
    config: &RetrievalConfig,
    scale_pct: u32,
) -> EngineResult<(EvidenceIndex, EvidenceContext)> {
    catch_unwind(AssertUnwindSafe(|| {
        let index = EvidenceIndex::build_with(source, config.index_options());
        let context = retrieve(&index, config, scale_pct);
        (index, context)
    }))
    .map_err(|_| EngineError::new(EngineErrorCode::AnalysisFailed, "document analysis failed"))
}

/// The evidence a model is shown for a document: every field's units,
/// within budgets scaled by `scale_pct` (100 as configured; the fit loop
/// passes less when a prompt must shrink).
pub fn retrieve(
    index: &EvidenceIndex,
    config: &RetrievalConfig,
    scale_pct: u32,
) -> EvidenceContext {
    let scale_pct = scale_pct.max(1);
    let candidates = index
        .units()
        .iter()
        .filter(|unit| !unit.running)
        .map(|unit| unit.ordinal)
        .collect::<Vec<_>>();
    let cost = |ordinal: u32| -> u32 { line_cost(&index.units()[ordinal as usize], config) };

    let whole_cost: u64 = candidates
        .iter()
        .map(|ordinal| u64::from(cost(*ordinal)))
        .sum();
    let whole_limit = u64::from(config.whole_document_tokens) * u64::from(scale_pct) / 100;
    if whole_cost <= whole_limit {
        let selected_by = candidates
            .iter()
            .map(|ordinal| (*ordinal, WHOLE_BIT))
            .collect();
        return render(index, config, Tier::Whole, false, selected_by);
    }

    let scored = Field::ALL.map(|field| score_units(index, config, field, &candidates));
    let tier = match config.tier {
        TierPolicy::Small => Tier::Small,
        TierPolicy::Normal => Tier::Normal,
        TierPolicy::Dense => Tier::Dense,
        TierPolicy::Auto => {
            if config.dense_triggers && crowded(index, &candidates, &scored[1]) {
                Tier::Dense
            } else {
                Tier::Normal
            }
        }
    };
    let tier_pct = match tier {
        Tier::Small => config.small_pct,
        Tier::Dense => config.dense_pct,
        Tier::Whole | Tier::Normal => 100,
    };
    let hierarchical = match config.strategy {
        Strategy::Flat => false,
        Strategy::Hierarchical => true,
        Strategy::Auto => candidates.len() as u32 >= config.hierarchical_min_units,
    };
    let sections_per_field = if tier == Tier::Dense {
        config.dense_sections_per_field
    } else {
        config.sections_per_field
    };

    let mut selected_by: BTreeMap<u32, u8> = BTreeMap::new();
    for (field, scores) in Field::ALL.iter().zip(&scored) {
        let budget = u64::from(config.budgets.of(*field)) * u64::from(tier_pct) / 100
            * u64::from(scale_pct)
            / 100;
        let allowed = hierarchical.then(|| {
            allowed_units(
                index,
                *field,
                scores,
                usize::from(sections_per_field.max(1)),
            )
        });
        let chosen = select(
            index,
            config,
            *field,
            scores,
            allowed.as_ref(),
            budget as u32,
        );
        for (ordinal, expansion) in chosen {
            let bits = selected_by.entry(ordinal).or_insert(0);
            *bits |= if expansion {
                EXPANSION_BIT
            } else {
                field.bit()
            };
        }
    }
    // The same text twice in the union is said once: the first, in
    // document order, keeps every field that chose either.
    let mut seen: BTreeMap<String, u32> = BTreeMap::new();
    let mut deduplicated: BTreeMap<u32, u8> = BTreeMap::new();
    for (ordinal, bits) in selected_by {
        let unit = &index.units()[ordinal as usize];
        let (exact, _, _) = duplicate_keys(&unit.text, is_body(unit));
        match seen.get(&exact) {
            Some(first) => {
                if let Some(first_bits) = deduplicated.get_mut(first) {
                    *first_bits |= bits;
                }
            }
            None => {
                seen.insert(exact, ordinal);
                deduplicated.insert(ordinal, bits);
            }
        }
    }
    render(index, config, tier, hierarchical, deduplicated)
}

/// What a unit costs as a prompt line: its handle, its text, a newline.
fn line_cost(unit: &EvidenceUnit, config: &RetrievalConfig) -> u32 {
    let handle = match config.id_style {
        IdStyle::Stable => estimated_tokens(&format!("[{}] ", unit.id)) as u32,
        // An ordinal is a few digits.
        IdStyle::Ordinal => 4,
    };
    unit.features.tokens + handle + 1
}

fn is_body(unit: &EvidenceUnit) -> bool {
    matches!(
        unit.kind,
        UnitKind::Paragraph | UnitKind::ListItem | UnitKind::Caption | UnitKind::Other
    )
}

/// One field's score for every unit, `None` where the unit cannot carry
/// the field at all.
fn score_units(
    index: &EvidenceIndex,
    config: &RetrievalConfig,
    field: Field,
    candidates: &[u32],
) -> BTreeMap<u32, i64> {
    let bm25 = bm25_scaled(index, field, candidates);
    let weights = &config.weights;
    let mut scores = BTreeMap::new();
    for ordinal in candidates {
        let unit = &index.units()[*ordinal as usize];
        let lexical = bm25.get(ordinal).copied().unwrap_or(0);
        let Some(parts) = components(field, unit, lexical) else {
            continue;
        };
        let score = (parts.cues * i64::from(weights.cues)
            + parts.bm25 * i64::from(weights.bm25)
            + parts.structure * i64::from(weights.structure)
            + parts.heading * i64::from(weights.heading)
            + parts.position * i64::from(weights.position)
            + parts.features * i64::from(weights.features))
            / 100;
        scores.insert(*ordinal, score);
    }
    scores
}

/// The parts of a unit's score for one field, before weighting.
#[derive(Clone, Copy, Debug, Default)]
struct Components {
    cues: i64,
    bm25: i64,
    structure: i64,
    heading: i64,
    position: i64,
    features: i64,
}

fn label_has(unit: &EvidenceUnit, words: &[&str]) -> bool {
    unit.label.as_deref().is_some_and(|label| {
        let label = normalize(label);
        words.iter().any(|word| {
            label == *word
                || label
                    .split(|character: char| !character.is_alphanumeric())
                    .any(|part| part == *word)
        })
    })
}

fn flag(condition: bool, value: i64) -> i64 {
    if condition { value } else { 0 }
}

/// The scoring components of one field, or `None` when the unit cannot
/// carry it: a type needs a type word or a title's place, a date a date,
/// parties a name or a party cue, an identifier an identifier, a key fact
/// an amount or a key-fact word.
fn components(field: Field, unit: &EvidenceUnit, bm25: i64) -> Option<Components> {
    let features = &unit.features;
    let cues = &features.cues;
    let position = &features.position;
    let tags = features.section_tags;
    let heading = unit.kind == UnitKind::Heading;
    let boilerplate = flag(cues.boilerplate_cues > 0 && !heading, -70);
    let characters = unit.text.chars().count() as i64;
    let parts = match field {
        Field::DocumentType => {
            if !(cues.type_nouns > 0 || cues.type_cues > 0 || position.title || cues.self_naming) {
                return None;
            }
            Components {
                cues: i64::from(cues.type_cues) * 26
                    + i64::from(cues.type_nouns.min(3)) * 20
                    + flag(cues.self_naming, 30)
                    + flag(cues.not_a_title, -60)
                    + boilerplate,
                bm25,
                structure: flag(heading, 40)
                    + flag(heading && unit.level == Some(1), 15)
                    + flag(position.title, 80)
                    + flag(characters > 200 && !cues.self_naming, -30),
                heading: 0,
                position: flag(position.first_page, 12)
                    + flag(position.opening, 60)
                    + flag(position.page_index == 0, 20),
                features: 0,
            }
        }
        Field::Date => {
            if features.dates.is_empty() && features.date_signals == 0 {
                return None;
            }
            let best_mention = features
                .dates
                .iter()
                .map(|mention| {
                    20 + flag(mention.role.is_some(), 50)
                        + flag(mention.issue_label, 40)
                        + flag(mention.defined_term, 60)
                        + flag(mention.deadline, -60)
                        + flag(mention.reference, -80)
                })
                .max()
                .unwrap_or(0);
            Components {
                cues: i64::from(features.date_signals.min(3)) * 30
                    + i64::from(cues.date_role_cues) * 45
                    + boilerplate,
                bm25,
                structure: flag(
                    unit.kind == UnitKind::Field && label_has(unit, &["date", "dated"]),
                    20,
                ) + flag(unit.table_header.is_some(), 10)
                    + flag(heading, 20),
                heading: flag(tags & tag::DEFINITIONS != 0, 20)
                    + flag(tags & tag::SIGNATURE != 0, 15)
                    + flag(tags & tag::TERM != 0, 10),
                position: flag(position.first_page, 12)
                    + flag(position.opening, 30)
                    + flag(position.closing, 15)
                    + flag(position.last_page, 8),
                features: best_mention,
            }
        }
        Field::Parties => {
            let labelled = label_has(unit, PARTY_LABELS);
            let entities = features.organisations.len() + features.people.len();
            if entities == 0
                && cues.party_cues == 0
                && features.proper_names == 0
                && !labelled
                && !cues.defined_role
            {
                return None;
            }
            Components {
                cues: i64::from(cues.party_cues) * 22
                    + i64::from(cues.issuer_cues) * 15
                    + i64::from(cues.customer_cues) * 15
                    + i64::from(cues.signature_cues) * 8
                    + flag(cues.defined_role, 30)
                    + boilerplate,
                bm25,
                structure: flag(position.letterhead, 40)
                    + flag(position.signature, 25)
                    + flag(labelled, 40)
                    + flag(heading && entities > 0, 20),
                heading: flag(tags & (tag::PARTIES | tag::RECITALS) != 0, 30)
                    + flag(tags & tag::SIGNATURE != 0, 20)
                    + flag(tags & tag::BILL_TO != 0, 30),
                position: flag(position.opening, 40)
                    + flag(position.first_page, 12)
                    + flag(position.closing, 20),
                features: features.organisations.len().min(4) as i64 * 25
                    + features.people.len().min(4) as i64 * 20
                    + i64::from(features.proper_names.min(4)) * 5,
            }
        }
        Field::Subject => {
            let whereas = unit.normalized.starts_with("whereas");
            Components {
                cues: i64::from(cues.subject_cues) * 24 + boilerplate,
                bm25,
                structure: flag(position.first_paragraph, 40)
                    + flag(whereas, 30)
                    + flag(
                        label_has(
                            unit,
                            &[
                                "re",
                                "subject",
                                "project",
                                "matter",
                                "description",
                                "purpose",
                                "scope",
                                "regarding",
                                "services",
                            ],
                        ),
                        50,
                    )
                    + flag(heading, -10),
                heading: flag(tags & tag::SCOPE != 0, 30) + flag(tags & tag::RECITALS != 0, 20),
                position: flag(position.opening, 30) + flag(position.first_page, 10),
                features: 0,
            }
        }
        Field::Identifier => {
            if features.identifiers.is_empty() {
                return None;
            }
            let found = features
                .identifiers
                .iter()
                .take(2)
                .map(|identifier| if identifier.label.is_some() { 60 } else { 25 })
                .sum::<i64>();
            Components {
                cues: i64::from(cues.subject_cues) * 10 + boilerplate,
                bm25,
                structure: flag(unit.kind == UnitKind::Field, 20)
                    + flag(position.letterhead, 10)
                    + flag(heading, 10),
                heading: flag(tags & tag::INVOICE != 0, 10),
                position: flag(position.first_page, 15) + flag(position.opening, 20),
                features: found,
            }
        }
        Field::KeyFacts => {
            if features.money.is_empty() && bm25 == 0 {
                return None;
            }
            Components {
                cues: boilerplate,
                bm25,
                structure: flag(unit.kind == UnitKind::TableRow, 10)
                    + flag(unit.kind == UnitKind::Field, 15),
                heading: flag(tags & tag::PAYMENT != 0, 25) + flag(tags & tag::TERM != 0, 15),
                position: flag(position.first_page, 5) + flag(position.closing, 10),
                features: flag(features.money_labelled, 60)
                    + features.money.len().min(2) as i64 * 25,
            }
        }
    };
    Some(parts)
}

/// BM25 of every candidate against the field's lexicon, scaled to 0-100
/// against the best candidate. Integer arithmetic throughout: k1 = 1.2,
/// b = 0.75, and the inverse document frequency a fixed-point log2.
fn bm25_scaled(index: &EvidenceIndex, field: Field, candidates: &[u32]) -> BTreeMap<u32, i64> {
    let query = field
        .lexicon()
        .iter()
        .flat_map(|word| terms_of(&normalize(word)))
        .filter_map(|term| index.term_id(&term))
        .collect::<BTreeSet<_>>();
    let mut raw: BTreeMap<u32, u128> = BTreeMap::new();
    if query.is_empty() {
        return BTreeMap::new();
    }
    let units = u128::from(index.indexed_units().max(1));
    let total = u128::from(index.total_length().max(1));
    let idf = query
        .iter()
        .map(|term| {
            let frequency = u64::from(index.document_frequency(*term));
            (
                *term,
                u128::from(log2_fixed(2 * units as u64 + 2, 2 * frequency + 1)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for ordinal in candidates {
        let unit = &index.units()[*ordinal as usize];
        let length = u128::from(unit.features.length);
        let mut score = 0_u128;
        for (term, weight) in &unit.features.terms {
            let Some(idf) = idf.get(term) else {
                continue;
            };
            let frequency = u128::from(*weight);
            let saturation = 1_000 * 11 * frequency * total
                / (5 * frequency * total + 3 * total + 9 * length * units).max(1);
            score += idf * saturation;
        }
        if score > 0 {
            raw.insert(*ordinal, score);
        }
    }
    let best = raw.values().copied().max().unwrap_or(0);
    if best == 0 {
        return BTreeMap::new();
    }
    raw.into_iter()
        .map(|(ordinal, score)| (ordinal, (score * 100 / best) as i64))
        .collect()
}

/// `1024 × log2(numerator / denominator)`, rounded down, for a ratio of at
/// least 1; 0 below. Integer only, so every platform agrees.
pub fn log2_fixed(numerator: u64, denominator: u64) -> u32 {
    if denominator == 0 || numerator <= denominator {
        return 0;
    }
    let numerator = u128::from(numerator);
    let denominator = u128::from(denominator);
    let mut whole = 0_u32;
    while denominator << (whole + 1) <= numerator {
        whole += 1;
    }
    // The ratio over 2^whole is in [1, 2): as a Q30 fraction.
    let mut ratio = (numerator << 30) / (denominator << whole);
    let mut result = whole << 10;
    for bit in (0..10).rev() {
        ratio = (ratio * ratio) >> 30;
        if ratio >= 2 << 30 {
            ratio >>= 1;
            result |= 1 << bit;
        }
    }
    result
}

/// Whether the document's evidence is crowded enough for the dense tier:
/// six or more candidate dates whose two best score within 15% of each
/// other, five or more organisations named where parties are, or OCR the
/// reader was not sure of.
fn crowded(index: &EvidenceIndex, candidates: &[u32], date_scores: &BTreeMap<u32, i64>) -> bool {
    let units = index.units();
    let mut dates: BTreeMap<&str, i64> = BTreeMap::new();
    for (ordinal, score) in date_scores {
        for mention in &units[*ordinal as usize].features.dates {
            let best = dates.entry(mention.iso.as_str()).or_insert(i64::MIN);
            *best = (*best).max(*score);
        }
    }
    let mut best = dates.values().copied().collect::<Vec<_>>();
    best.sort_unstable_by(|left, right| right.cmp(left));
    let close_dates =
        best.len() >= 6 && best[0] > 0 && (best[0] - best[1]) * 100 <= best[0].abs() * 15;
    let organisations = candidates
        .iter()
        .map(|ordinal| &units[*ordinal as usize])
        .filter(|unit| unit.features.cues.party_cues > 0 || unit.features.cues.defined_role)
        .flat_map(|unit| unit.features.organisations.iter())
        .collect::<BTreeSet<_>>();
    let ocr = candidates
        .iter()
        .map(|ordinal| &units[*ordinal as usize])
        .filter(|unit| unit.source == TextSource::Ocr)
        .filter_map(|unit| unit.confidence)
        .map(u64::from)
        .collect::<Vec<_>>();
    let poor_ocr = !ocr.is_empty() && ocr.iter().sum::<u64>() < 80 * ocr.len() as u64;
    close_dates || organisations.len() >= 5 || poor_ocr
}

/// The units a field may choose from when retrieval is hierarchical: those
/// of its best-scoring sections, of the first page, and - for the parties
/// and the date - of the last section with a signature block.
fn allowed_units(
    index: &EvidenceIndex,
    field: Field,
    scores: &BTreeMap<u32, i64>,
    take: usize,
) -> BTreeSet<u32> {
    let sections = index.sections();
    let mut ranked = sections
        .iter()
        .enumerate()
        .filter_map(|(position, section)| {
            let mut unit_scores = section
                .units()
                .filter_map(|ordinal| scores.get(&ordinal).copied())
                .collect::<Vec<_>>();
            if unit_scores.is_empty() {
                return None;
            }
            unit_scores.sort_unstable_by(|left, right| right.cmp(left));
            let top = unit_scores.iter().take(3).sum::<i64>();
            let bonus = match field {
                Field::Date => {
                    flag(section.tags & tag::DEFINITIONS != 0, 40)
                        + flag(section.tags & tag::SIGNATURE != 0, 20)
                }
                Field::Parties => {
                    flag(section.tags & (tag::PARTIES | tag::RECITALS) != 0, 40)
                        + flag(section.tags & tag::SIGNATURE != 0, 30)
                        + flag(section.tags & tag::BILL_TO != 0, 30)
                }
                Field::Subject => {
                    flag(section.tags & tag::SCOPE != 0, 30)
                        + flag(section.tags & tag::RECITALS != 0, 20)
                }
                Field::Identifier => flag(section.tags & tag::INVOICE != 0, 20),
                Field::KeyFacts => flag(section.tags & tag::PAYMENT != 0, 30),
                Field::DocumentType => 0,
            };
            Some((unit_scores[0] + top / 4 + bonus, position))
        })
        .collect::<Vec<_>>();
    ranked.sort_unstable_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    let mut chosen = ranked
        .iter()
        .take(take)
        .map(|(_, position)| *position)
        .collect::<BTreeSet<_>>();
    let first_page = index.units().first().map(|unit| unit.page);
    for (position, section) in sections.iter().enumerate() {
        if Some(section.first_page) == first_page {
            chosen.insert(position);
        }
    }
    if matches!(field, Field::Parties | Field::Date) {
        let signature = sections.iter().rposition(|section| {
            section.units().any(|ordinal| {
                index
                    .units()
                    .get(ordinal as usize)
                    .is_some_and(|unit| unit.features.position.signature)
            })
        });
        if let Some(position) = signature {
            chosen.insert(position);
        }
    }
    chosen
        .into_iter()
        .flat_map(|position| sections[position].units())
        .collect()
}

/// One field's units within its budget: the best first, each with the
/// context it needs; repeats skipped; each date carried by at most
/// `units_per_date` units; parties by what each unit adds.
fn select(
    index: &EvidenceIndex,
    config: &RetrievalConfig,
    field: Field,
    scores: &BTreeMap<u32, i64>,
    allowed: Option<&BTreeSet<u32>>,
    budget: u32,
) -> Vec<(u32, bool)> {
    /// How many of the best units the parties' greedy cover looks at.
    const PARTY_POOL: usize = 400;
    let units = index.units();
    let names = |ordinal: u32| -> Vec<&String> {
        let features = &units[ordinal as usize].features;
        let mut names = features
            .organisations
            .iter()
            .chain(&features.people)
            .collect::<Vec<_>>();
        names.sort_unstable();
        names.dedup();
        names
    };
    let mut pool = scores
        .iter()
        .filter(|(ordinal, score)| **score > 0 && allowed.is_none_or(|set| set.contains(*ordinal)))
        .map(|(ordinal, score)| (*ordinal, *score))
        .collect::<Vec<_>>();
    pool.sort_unstable_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    // Parties: each candidate's names, and only the best few hundred by
    // what they could add, so the cover stays cheap on a long document.
    let mut party_names: BTreeMap<u32, Vec<&String>> = BTreeMap::new();
    if field == Field::Parties {
        let mut bounded = pool
            .iter()
            .map(|(ordinal, score)| {
                let found = names(*ordinal);
                (score + 30 * found.len() as i64, *ordinal, *score, found)
            })
            .collect::<Vec<_>>();
        bounded.sort_unstable_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
        bounded.truncate(PARTY_POOL);
        pool = bounded
            .iter()
            .map(|(_, ordinal, score, _)| (*ordinal, *score))
            .collect();
        party_names = bounded
            .into_iter()
            .map(|(_, ordinal, _, found)| (ordinal, found))
            .collect();
    }

    let expansion_cap = budget * config.expansion_pct / 100;
    let mut used = 0_u32;
    let mut expansion_used = 0_u32;
    let mut chosen: Vec<(u32, bool)> = Vec::new();
    let mut taken: BTreeSet<u32> = BTreeSet::new();
    let mut exact_seen: BTreeSet<String> = BTreeSet::new();
    let mut shapes_seen: BTreeSet<String> = BTreeSet::new();
    let mut per_date: BTreeMap<String, u8> = BTreeMap::new();
    let mut entities_seen: BTreeSet<String> = BTreeSet::new();

    let mut cursor = 0;
    while used < budget {
        // Parties are taken by what a unit adds: a name not seen yet is
        // worth more than another copy of one already taken. Every other
        // field takes its units best first.
        let ordinal = if field == Field::Parties {
            if pool.is_empty() {
                break;
            }
            let pick = pool
                .iter()
                .enumerate()
                .map(|(position, (ordinal, score))| {
                    let found = party_names.get(ordinal).map_or(&[][..], Vec::as_slice);
                    let new = found
                        .iter()
                        .filter(|name| !entities_seen.contains(**name))
                        .count() as i64;
                    let gain = score + new * 30 - flag(!found.is_empty() && new == 0, 40);
                    (gain, *ordinal, position)
                })
                .max_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)))
                .map_or(0, |(_, _, position)| position);
            pool.remove(pick).0
        } else {
            let Some((ordinal, _)) = pool.get(cursor) else {
                break;
            };
            cursor += 1;
            *ordinal
        };
        let unit = &units[ordinal as usize];
        if taken.contains(&ordinal) {
            // Already here as another unit's context: now chosen for
            // itself, at no further cost.
            chosen.push((ordinal, false));
            continue;
        }
        let (exact, head, tail) = duplicate_keys(&unit.text, is_body(unit));
        if exact_seen.contains(&exact) {
            continue;
        }
        // A shape repeat (digits aside) is skipped, except for dates: two
        // lines that differ only in their digits may differ in the date.
        let shaped = shapes_seen.contains(&head)
            || tail.as_ref().is_some_and(|tail| shapes_seen.contains(tail));
        if shaped && field != Field::Date {
            continue;
        }
        if field == Field::Date && !unit.features.dates.is_empty() {
            let full = unit.features.dates.iter().all(|mention| {
                per_date.get(&mention.iso).copied().unwrap_or(0) >= config.units_per_date
            });
            if full && config.units_per_date > 0 {
                continue;
            }
        }
        let cost = line_cost(unit, config);
        let fits = used + cost <= budget || (chosen.is_empty() && cost <= budget.saturating_mul(4));
        if !fits {
            continue;
        }
        used += cost;
        taken.insert(ordinal);
        chosen.push((ordinal, false));
        exact_seen.insert(exact);
        shapes_seen.insert(head);
        if let Some(tail) = tail {
            shapes_seen.insert(tail);
        }
        if field == Field::Date {
            let mut stated = BTreeSet::new();
            for mention in &unit.features.dates {
                if stated.insert(mention.iso.as_str()) {
                    *per_date.entry(mention.iso.clone()).or_insert(0) += 1;
                }
            }
        }
        for name in unit
            .features
            .organisations
            .iter()
            .chain(&unit.features.people)
        {
            entities_seen.insert(name.clone());
        }
        for context in expansions(index, unit) {
            if taken.contains(&context) {
                continue;
            }
            let context_cost = line_cost(&units[context as usize], config);
            if expansion_used + context_cost > expansion_cap || used + context_cost > budget {
                continue;
            }
            expansion_used += context_cost;
            used += context_cost;
            taken.insert(context);
            chosen.push((context, true));
        }
    }
    chosen
}

/// What a unit needs beside it to be understood: its table's header row,
/// the heading it falls under, and - for a short unit, a label standing on
/// its own line - the unit after it in the same block or section.
fn expansions(index: &EvidenceIndex, unit: &EvidenceUnit) -> Vec<u32> {
    let units = index.units();
    let mut context = Vec::new();
    if let Some(header) = unit.table_header {
        context.push(header);
    }
    if unit.kind != UnitKind::Heading
        && let Some(heading) = unit.section_path.last()
    {
        context.push(*heading);
    }
    if unit.text.chars().count() < 60
        && let Some(next) = units.get(unit.ordinal as usize + 1)
        && !next.running
        && next.kind != UnitKind::Heading
        && (next.block == unit.block
            || (next.page == unit.page && next.section_path == unit.section_path))
    {
        context.push(next.ordinal);
    }
    context
        .into_iter()
        .filter(|ordinal| {
            units
                .get(*ordinal as usize)
                .is_some_and(|context| !context.running)
        })
        .collect()
}

/// The chosen units as prompt lines, in document order.
fn render(
    index: &EvidenceIndex,
    config: &RetrievalConfig,
    tier: Tier,
    hierarchical: bool,
    selected_by: BTreeMap<u32, u8>,
) -> EvidenceContext {
    let units = index.units();
    let mut text = String::new();
    let mut handles = Vec::with_capacity(selected_by.len());
    let mut page = None;
    for (position, ordinal) in selected_by.keys().enumerate() {
        let unit = &units[*ordinal as usize];
        let handle = match config.id_style {
            IdStyle::Stable => unit.id.clone(),
            IdStyle::Ordinal => {
                if page != Some(unit.page) {
                    text.push_str(&format!("--- page {} ---\n", unit.page));
                    page = Some(unit.page);
                }
                (position + 1).to_string()
            }
        };
        text.push('[');
        text.push_str(&handle);
        text.push_str("] ");
        text.push_str(&collapse_whitespace(&unit.text));
        text.push('\n');
        handles.push((handle, *ordinal));
    }
    let text = text.trim_end().to_owned();
    EvidenceContext {
        tier,
        hierarchical,
        units: selected_by.keys().copied().collect(),
        handles,
        estimated_tokens: estimated_tokens(&text),
        characters: text.chars().count(),
        text,
        selected_by,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::source_from_text;
    use crate::domain::{PageOrigin, SourcePage};
    use crate::evidence::date_match_positions;

    fn page(number: usize, text: &str) -> SourcePage {
        SourcePage::new(number, text, PageOrigin::Native)
    }

    /// Fifty pages: parties in the preamble and on the signature page, the
    /// date only in a definition on page 20, a referenced agreement's date
    /// and a maturity date nearby, and a statement of dated rows.
    fn credit_agreement() -> DocumentSource {
        let filler = "The Borrower shall maintain its books and records in accordance with \
                      generally accepted accounting principles consistently applied. ";
        let mut pages = Vec::new();
        for number in 1..=50 {
            let mut body = String::new();
            match number {
                1 => body.push_str("CREDIT AGREEMENT\n\nThis Credit Agreement is dated as of the Closing Date by and between Marrowfield Packaging Holdings, Inc. (the \"Borrower\") and Halden Bay National Bank, N.A., as Lender.\n\n"),
                20 => body.push_str("ARTICLE I DEFINITIONS\n\n\"Closing Date\" means June 12, 2026.\n\n\"Existing Credit Agreement\" means the credit agreement dated September 30, 2021.\n\n\"Maturity Date\" means June 12, 2031.\n\n"),
                30 => {
                    body.push_str("SCHEDULE 2.01 PAYMENTS\n\n| Date | Amount |\n| --- | --- |\n");
                    for month in 1..=12 {
                        body.push_str(&format!("| {month:02}/30/2027 | $1,000.00 |\n"));
                    }
                    body.push('\n');
                }
                50 => body.push_str("IN WITNESS WHEREOF, the parties have executed this Agreement.\n\nMARROWFIELD PACKAGING HOLDINGS, INC.\n\nBy: /s/ Ada Example\nName: Ada Example\n\nHALDEN BAY NATIONAL BANK, N.A.\n\nBy: /s/ Ira Sample\nName: Ira Sample\n\n"),
                _ => {}
            }
            body.push_str(&format!("Credit Agreement - Page {number}\n\n"));
            for _ in 0..12 {
                body.push_str(filler);
                body.push('\n');
            }
            pages.push(page(number, &body));
        }
        DocumentSource::from_pages(pages)
    }

    fn holds(context: &EvidenceContext, index: &EvidenceIndex, text: &str) -> bool {
        let wanted = normalize(text);
        context.units.iter().any(|ordinal| {
            index.units()[*ordinal as usize]
                .normalized
                .contains(&wanted)
        })
    }

    #[test]
    fn a_short_document_goes_whole() {
        let index = EvidenceIndex::build(&source_from_text(
            "NOTICE OF TERMINATION\n\nDate of this Notice: December 29, 2026\n\nDear Mira Vale,\nYour employment will end effective January 31, 2027.",
        ));
        let context = retrieve(&index, &RetrievalConfig::default(), 100);
        assert_eq!(context.tier, Tier::Whole);
        assert_eq!(context.units.len(), index.units().len());
        assert!(context.text.starts_with("[p1.b1] NOTICE OF TERMINATION\n"));
        assert!(
            context
                .text
                .contains("[p1.b3] Dear Mira Vale, Your employment")
        );
    }

    #[test]
    fn every_field_reaches_its_evidence_in_a_long_document() {
        let source = credit_agreement();
        let config = RetrievalConfig::default();
        let index = EvidenceIndex::build_with(&source, config.index_options());
        for strategy in [Strategy::Flat, Strategy::Hierarchical] {
            let config = RetrievalConfig {
                strategy,
                ..RetrievalConfig::default()
            };
            let context = retrieve(&index, &config, 100);
            assert_ne!(context.tier, Tier::Whole);
            assert_eq!(context.hierarchical, strategy == Strategy::Hierarchical);
            for wanted in [
                "CREDIT AGREEMENT",
                "\"Closing Date\" means June 12, 2026.",
                "Marrowfield Packaging Holdings, Inc.",
                "Halden Bay National Bank, N.A.",
            ] {
                assert!(
                    holds(&context, &index, wanted),
                    "{strategy:?} lost {wanted}:\n{}",
                    context.text
                );
            }
            // The definition's heading comes with it.
            assert!(holds(&context, &index, "ARTICLE I DEFINITIONS"));
            // Twelve dated rows do not take the date budget: each date is
            // its own, but they shape alike only in digits, and the budget
            // still holds the definitions.
            let rows = context
                .units
                .iter()
                .filter(|ordinal| index.units()[**ordinal as usize].text.contains("/30/2027"))
                .count();
            assert!(rows <= 12);
            let budget = config.budgets.total() as usize * config.dense_pct as usize / 100;
            assert!(
                context.estimated_tokens <= budget + 40,
                "{} tokens",
                context.estimated_tokens
            );
            assert!(!context.text.contains("Credit Agreement - Page 7"));
        }
    }

    #[test]
    fn date_evidence_is_not_crowded_out_by_type_evidence() {
        // A type budget of almost nothing must not cost the date anything.
        let source = credit_agreement();
        let index = EvidenceIndex::build(&source);
        let config = RetrievalConfig {
            budgets: FieldBudgets {
                document_type: 2_000,
                ..FieldBudgets::default()
            },
            ..RetrievalConfig::default()
        };
        let generous = retrieve(&index, &config, 100);
        let plain = retrieve(&index, &RetrievalConfig::default(), 100);
        assert_eq!(
            generous.chosen_for(Field::Date),
            plain.chosen_for(Field::Date)
        );
        assert_eq!(
            generous.chosen_for(Field::Subject),
            plain.chosen_for(Field::Subject)
        );
    }

    #[test]
    fn retrieval_is_deterministic_and_names_its_configuration() {
        let source = credit_agreement();
        let config = RetrievalConfig::default();
        let first = prepare_evidence(&source, &config, 100).unwrap();
        let second = prepare_evidence(&source, &config, 100).unwrap();
        assert_eq!(first.1, second.1);
        assert_eq!(
            config.fingerprint(),
            RetrievalConfig::default().fingerprint()
        );
        let other = RetrievalConfig {
            units_per_date: 3,
            ..RetrievalConfig::default()
        };
        assert_ne!(config.fingerprint(), other.fingerprint());
        assert_eq!(config.fingerprint().len(), 64);
        // A configuration written by hand names only what it changes.
        let partial: RetrievalConfig =
            serde_json::from_str(r#"{"budgets": {"date": 600}, "strategy": "flat"}"#).unwrap();
        assert_eq!(partial.budgets.date, 600);
        assert_eq!(partial.budgets.parties, 450);
        assert_eq!(partial.strategy, Strategy::Flat);
    }

    #[test]
    fn ordinal_handles_count_from_one_and_mark_pages() {
        let source = credit_agreement();
        let index = EvidenceIndex::build(&source);
        let config = RetrievalConfig {
            id_style: IdStyle::Ordinal,
            ..RetrievalConfig::default()
        };
        let context = retrieve(&index, &config, 100);
        assert!(context.text.starts_with("--- page 1 ---\n[1] "));
        assert_eq!(context.handles[0].0, "1");
        assert_eq!(context.handles.len(), context.units.len());
    }

    #[test]
    fn a_smaller_scale_sends_less() {
        let source = credit_agreement();
        let index = EvidenceIndex::build(&source);
        let config = RetrievalConfig::default();
        let full = retrieve(&index, &config, 100);
        let half = retrieve(&index, &config, 50);
        assert!(half.estimated_tokens < full.estimated_tokens);
        assert!(
            context_states(&half, &index, "2026-06-12"),
            "the defining date survives halving:\n{}",
            half.text
        );
    }

    fn context_states(context: &EvidenceContext, index: &EvidenceIndex, iso: &str) -> bool {
        context.units.iter().any(|ordinal| {
            !date_match_positions(iso, &index.units()[*ordinal as usize].normalized).is_empty()
        })
    }

    #[test]
    fn fixed_point_logarithms_are_close_and_monotonic() {
        assert_eq!(log2_fixed(1, 1), 0);
        assert_eq!(log2_fixed(2, 1), 1024);
        assert_eq!(log2_fixed(8, 1), 3 * 1024);
        let three = log2_fixed(3, 1);
        assert!((1621..=1624).contains(&three), "{three}");
        let mut previous = 0;
        for numerator in 2..200 {
            let value = log2_fixed(numerator, 2);
            assert!(value >= previous);
            previous = value;
        }
    }
}
