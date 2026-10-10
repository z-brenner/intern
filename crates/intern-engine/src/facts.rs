//! Validation of an evidence-pipeline reply: every fact checked against the
//! units the model was shown, and the filename and description composed
//! from what survives.
//!
//! The digest pipeline checks a reply against the digest the model read.
//! Here the model read excerpts - the units retrieval chose - so there are
//! two views of the document ([`ValidationScope`]):
//!
//! * **context**: the units the prompt carried. A fact is accepted only if
//!   it is stated here: a type, a date, a name or an identifier whole, and
//!   a subject or a key fact word by word ([`every_word_stated`]). A fact
//!   the document states outside the excerpts is not accepted: the model
//!   could not have read it.
//! * **document**: every unit. The guards that turn a stated date away - it
//!   is another document's date, it is a deadline, its day and month could
//!   be either way round - run over both views, and either one firing is
//!   enough. A guard never sees less of the document than the digest
//!   pipeline's does, so excerpting can only make it stricter.
//!
//! Support is recorded per fact ([`Support`]): `Cited` when a unit the reply
//! cited for it states it, `Context` when only another excerpt does - the
//! support the digest pipeline accepts, so readiness is no stricter by
//! accident - and `Unsupported` otherwise. A cited id the prompt never
//! showed, or one whose unit does not state its fact, is never evidence.
//!
//! What a reviewer sees as evidence is dereferenced text: a line of the unit
//! that supports the fact, never anything the model wrote.
//!
//! The check of a subject's or a key fact's words ([`every_word_stated`])
//! guards against a model's honest mistakes: a word, a number or a symbol
//! it adds, swaps or misremembers. It is no defence against text crafted
//! to slip past it, and need not be one: whoever writes the document
//! already decides what the context states. Each word must be stated in
//! some unit, a plural's "s" aside; single letters and the glue words
//! ([`GLUE_WORDS`]) need not be. A contracted negation ("can't") is
//! checked whole. Each number must be stated whole: "248" is not stated by
//! "$1,248.00". Each currency symbol and percent sign must be stated.
//! Invisible characters are ignored by this check and by every other that
//! reads text through [`normalize`]. Word order is not checked, so a
//! subject of stated words can still misstate how they relate.

use std::collections::BTreeSet;

use unicode_normalization::char::is_combining_mark;

use crate::compose::{
    CastMember, DescriptionFacts, Relation, RelationCues, amount_label, describe, identifier_word,
    money_in, relation_from_roles,
};
use crate::cues::{CUSTOMER_CUES, ISSUER_CUES, STOPWORDS};
use crate::distill::normalize_heading;
use crate::domain::{
    DateRole, DocumentClass, Evidence, EvidenceRef, FactSupport, ModelFacts, ModelProposal,
    ParserWarning, PartyRelation, PartyRole, ProposalStatus, ReviewReason, Support, ValidatedFacts,
    ValidatedParty, ValidatedProposal, ValidationOutcome,
};
use crate::evidence::{
    Segments, contains_whole, date_match_positions, digest_contains, digest_contains_date,
    digest_contains_loosely, extract_stated_dates, is_invisible, normalize, normalize_loosely,
    numeric_dates, stated_dates,
};
use crate::index::{EvidenceIndex, EvidenceUnit, UnitKind};
use crate::infer::{infer_date_role, repair_issued_relation};
use crate::phrases::{
    has_a_kind, head_noun, is_placeholder, is_reference, label_and_value, names_a_kind,
    opens_with_party_label, phrase_positions, same_word, subject_value, tidy_case, title_phrase,
    trim_name, words,
};
use crate::retrieve::EvidenceContext;
use crate::validate::{
    GENERIC_CAPITALS, READY_CONFIDENCE, current_year, date_is_tainted, deadline_fires,
    effective_alternates, first_unsupported_claim, issue_date_alternates, push,
    reading_is_unsettled, validate_description, year_is_plausible,
};

/// Share of a subject's distinct significant words its cited units must
/// hold for it to be written into the description. It must also pass the
/// word check against the whole context ([`every_word_stated`]).
const SUBJECT_OVERLAP: f32 = 0.6;
/// The words a subject or a key fact may hold that the context need not
/// state: articles, prepositions and conjunctions, which join the words
/// that say something.
const GLUE_WORDS: &[&str] = &[
    "a", "an", "the", "of", "and", "or", "for", "to", "in", "on", "at", "by", "with", "from", "as",
    "per", "via", "into", "re",
];
/// The currency symbols a subject or a key fact may write only where the
/// context writes them too. Every symbol [`money_in`] reads is here.
const CURRENCY_SYMBOLS: &[char] = &[
    '$', '\u{20ac}', '\u{a3}', '\u{a5}', '\u{20b9}', '\u{20a9}', '\u{20bd}', '\u{a2}', '\u{20ba}',
    '\u{20aa}', '\u{20ab}', '\u{e3f}', '\u{20a6}', '\u{20b1}', '\u{20b4}', '\u{20a1}',
];
/// The percent and per-mille signs, checked as the currency symbols are: a
/// stated "$25" is not "25%".
const PERCENT_SIGNS: &[char] = &['%', '\u{2030}'];
/// How far after a party's name a defined term may stand: `("Tenant")`,
/// `(the "Borrower")`, `, as Lender`.
const DEFINED_TERM_REACH: usize = 60;
/// How far before a party's name its label may stand: `Bill To:`.
const LABEL_REACH: usize = 40;
/// Longest unit that can be a letterhead's.
const LETTERHEAD_CHARACTERS: usize = 120;
/// Longest line a date line is cut to, as distill cuts them.
const MAX_DATE_LINE_CHARACTERS: usize = 150;

/// Some of a document's units, read the way validation reads a digest:
/// each unit a segment, every heading of the document, the lines that state
/// a date.
#[derive(Clone, Debug, Default)]
pub struct ScopeView {
    segments: Vec<String>,
    headings: Vec<String>,
    date_lines: Vec<String>,
    warnings: Vec<ParserWarning>,
    ordinals: Vec<u32>,
}

impl Segments for ScopeView {
    fn segments(&self) -> &[String] {
        &self.segments
    }

    fn headings(&self) -> &[String] {
        &self.headings
    }

    fn date_lines(&self) -> &[String] {
        &self.date_lines
    }

    fn parser_warnings(&self) -> &[ParserWarning] {
        &self.warnings
    }
}

impl ScopeView {
    fn of<'u>(
        units: impl IntoIterator<Item = &'u EvidenceUnit>,
        headings: &[String],
        warnings: &[ParserWarning],
    ) -> Self {
        let mut view = Self {
            headings: headings.to_vec(),
            warnings: warnings.to_vec(),
            ..Self::default()
        };
        for unit in units {
            view.segments.push(unit.text.clone());
            view.ordinals.push(unit.ordinal);
            if unit.features.dates.is_empty() {
                continue;
            }
            for line in unit.text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty()
                    || (extract_stated_dates(trimmed).is_empty()
                        && numeric_dates(&normalize(trimmed)).is_empty())
                {
                    continue;
                }
                let shortened = trimmed
                    .chars()
                    .take(MAX_DATE_LINE_CHARACTERS)
                    .collect::<String>();
                if !view.date_lines.contains(&shortened) {
                    view.date_lines.push(shortened);
                }
            }
        }
        view
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// The ordinals of the units in the view, in document order.
    pub fn ordinals(&self) -> &[u32] {
        &self.ordinals
    }
}

/// The two views validation reads an evidence-pipeline reply against: what
/// the model was shown, and the whole document.
pub struct ValidationScope<'a> {
    index: &'a EvidenceIndex,
    shown: BTreeSet<u32>,
    context: ScopeView,
    document: ScopeView,
}

impl<'a> ValidationScope<'a> {
    pub fn new(
        index: &'a EvidenceIndex,
        context: &EvidenceContext,
        warnings: &[ParserWarning],
    ) -> Self {
        let units = index.units();
        // Every heading of the document, as the digest's outline lists
        // every heading whatever the digest kept: the type is completed
        // from the title wherever the title is.
        let headings = index
            .headings()
            .iter()
            .filter_map(|ordinal| units.get(*ordinal as usize))
            .filter(|unit| !unit.running && unit.kind == UnitKind::Heading)
            .map(|unit| normalize_heading(&unit.text))
            .filter(|heading| !heading.is_empty())
            .collect::<Vec<_>>();
        let shown = context.units.iter().copied().collect::<BTreeSet<_>>();
        let context_view = ScopeView::of(
            context
                .units
                .iter()
                .filter_map(|ordinal| units.get(*ordinal as usize)),
            &headings,
            warnings,
        );
        let document_view = ScopeView::of(
            units.iter().filter(|unit| !unit.running),
            &headings,
            warnings,
        );
        Self {
            index,
            shown,
            context: context_view,
            document: document_view,
        }
    }

    /// The units the model was shown.
    pub fn context(&self) -> &ScopeView {
        &self.context
    }

    /// Every unit of the document but repeated running lines.
    pub fn document(&self) -> &ScopeView {
        &self.document
    }

    /// The units among `ids` the model was shown, in the order cited. An id
    /// the prompt did not carry is not a unit the model could cite.
    fn cited_units(&self, ids: &[String]) -> Vec<&'a EvidenceUnit> {
        let mut units: Vec<&EvidenceUnit> = Vec::new();
        for id in ids {
            if let Some(unit) = self.index.unit(id)
                && self.shown.contains(&unit.ordinal)
                && !units.iter().any(|kept| kept.ordinal == unit.ordinal)
            {
                units.push(unit);
            }
        }
        units
    }

    /// Some units on their own, for telling which of them states a fact.
    fn view(&self, units: &[&EvidenceUnit]) -> ScopeView {
        ScopeView::of(units.iter().copied(), &[], &[])
    }

    /// The context's units, in document order.
    fn context_units(&self) -> impl Iterator<Item = &'a EvidenceUnit> + '_ {
        self.context
            .ordinals
            .iter()
            .filter_map(|ordinal| self.index.units().get(*ordinal as usize))
    }

    /// Every date the document states, in first-mention order, at most
    /// eight - the dates a reviewer is offered.
    pub fn stated_dates(&self) -> Vec<String> {
        stated_dates(&self.document)
    }
}

/// Where a fact was found: the support, and the unit and line that show it.
struct Found<'a> {
    support: Support,
    unit: Option<&'a EvidenceUnit>,
    line: Option<String>,
    miscited: u32,
}

impl Found<'_> {
    fn absent() -> Self {
        Self {
            support: Support::Absent,
            unit: None,
            line: None,
            miscited: 0,
        }
    }
}

/// Finds a fact: in the cited units first, then anywhere in the context.
/// `holds` says whether a view states the fact; `line` picks the line of a
/// unit that shows it.
fn find<'a>(
    scope: &ValidationScope<'a>,
    cited: &[String],
    holds: impl Fn(&ScopeView) -> bool,
    line: impl Fn(&EvidenceUnit) -> Option<String>,
) -> Found<'a> {
    let units = scope.cited_units(cited);
    if !units.is_empty() && holds(&scope.view(&units)) {
        let unit = units
            .iter()
            .copied()
            .find(|unit| holds(&scope.view(&[*unit])))
            .or_else(|| units.first().copied());
        return Found {
            support: Support::Cited,
            line: unit.and_then(&line),
            unit,
            miscited: 0,
        };
    }
    let miscited = units.len() as u32;
    if holds(&scope.context) {
        let unit = scope
            .context_units()
            .find(|unit| holds(&scope.view(&[*unit])));
        return Found {
            support: Support::Context,
            line: unit.and_then(&line),
            unit,
            miscited,
        };
    }
    Found {
        support: Support::Unsupported,
        unit: None,
        line: None,
        miscited,
    }
}

/// The first line of a unit whose normalized text `holds`, or the unit's
/// whole text, collapsed, when the fact spans its lines.
fn line_where(unit: &EvidenceUnit, holds: impl Fn(&str) -> bool) -> Option<String> {
    unit.text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && holds(&normalize(line)))
        .map(str::to_owned)
        .or_else(|| {
            holds(&unit.normalized)
                .then(|| unit.text.split_whitespace().collect::<Vec<_>>().join(" "))
        })
}

/// Checks an evidence-pipeline reply against the document it answered
/// about. See the module docs; the year judgment is [`validate_facts_at`]'s.
pub fn validate_facts(candidate: ModelProposal, scope: &ValidationScope<'_>) -> ValidationOutcome {
    validate_facts_at(candidate, scope, current_year())
}

/// [`validate_facts`], with the current year given.
pub fn validate_facts_at(
    mut candidate: ModelProposal,
    scope: &ValidationScope<'_>,
    current_year: i32,
) -> ValidationOutcome {
    let facts = candidate.facts.as_deref().cloned().unwrap_or_default();
    complete_candidate(&mut candidate, &facts, scope);
    let original = candidate.clone();
    let context = scope.context();
    let document = scope.document();
    let mut reasons = Vec::new();
    let mut support = FactSupport {
        unknown_ids: u32::try_from(facts.unknown_evidence.len()).unwrap_or(u32::MAX)
            + facts
                .cited_ids()
                .filter(|id| {
                    scope
                        .index
                        .unit(id)
                        .is_none_or(|unit| !scope.shown.contains(&unit.ordinal))
                })
                .count() as u32,
        ..FactSupport::default()
    };
    let mut references: Vec<EvidenceRef> = Vec::new();

    // The type: the document's own title phrase where the reply cited its
    // title, else the reply's words where the document states them whole
    // and not as another document's name, each of its marks kept only
    // where that line holds it. A type the document does not state is
    // never written; the title, if one names the same kind of document,
    // stands in for review.
    let mut type_line = None;
    let mut type_title = None;
    let proposed_type = content(facts.document_type.as_deref(), scope);
    let (mut document_type, type_supported) = match proposed_type {
        None => (None, true),
        Some(value) => match read_type(scope, &facts.type_evidence, value) {
            TypeReading::Title { phrase, found } => {
                support.document_type = found.support;
                remember(&found, &mut references);
                type_line = found.line.clone();
                type_title = Some(phrase.clone());
                (Some(phrase), true)
            }
            TypeReading::Stated { found } => {
                support.document_type = found.support;
                support.miscited_ids += found.miscited;
                remember(&found, &mut references);
                let kept = marks_its_line_holds(value, found.line.as_deref().unwrap_or_default());
                type_line = found.line.clone();
                (Some(titled(&kept)), true)
            }
            TypeReading::Retitled { phrase, line } => {
                support.document_type = Support::Unsupported;
                type_line = Some(line);
                type_title = Some(phrase.clone());
                push(&mut reasons, ReviewReason::TypeInferred);
                (Some(phrase), true)
            }
            TypeReading::Unsupported => {
                support.document_type = Support::Unsupported;
                (None, false)
            }
        },
    };
    if !type_supported {
        push(&mut reasons, ReviewReason::TypeUnsupported);
    }
    if document_type.is_none() {
        match title_of(scope) {
            Some((inferred, line)) => {
                type_line = Some(line);
                type_title = Some(inferred.clone());
                document_type = Some(inferred);
                push(&mut reasons, ReviewReason::TypeInferred);
            }
            None if type_supported => push(&mut reasons, ReviewReason::TypeMissing),
            None => {}
        }
    }

    // The date.
    let mut date_line: Option<String> = None;
    let mut date_role = facts.date_role;
    let mut document_date = None;
    let mut date_supported = true;
    let mut replaced = false;
    let mut withheld_as_deadline = false;
    if let Some(date) = present(facts.document_date.as_deref()) {
        let found = if crate::evidence::is_valid_iso_date(date) {
            find(
                scope,
                &facts.date_evidence,
                |view| digest_contains_date(view, date),
                |unit| line_where(unit, |line| !date_match_positions(date, line).is_empty()),
            )
        } else {
            Found {
                support: Support::Unsupported,
                ..Found::absent()
            }
        };
        support.document_date = found.support;
        support.miscited_ids += found.miscited;
        if found.support == Support::Unsupported {
            date_supported = false;
            date_role = None;
        } else {
            document_date = Some(date.to_owned());
            date_line = found.line.clone();
            remember(&found, &mut references);
            // Another document's date: tainted in either view.
            let tainted_context = date_is_tainted(context, date, document_type.as_deref());
            let tainted_document = date_is_tainted(document, date, document_type.as_deref());
            if tainted_context || tainted_document {
                support.guard_scope = Some(scope_name(tainted_context, tainted_document));
                match single_agreed(
                    effective_alternates(context, date),
                    effective_alternates(document, date),
                ) {
                    Some((alternate, line)) => {
                        document_date = Some(alternate);
                        date_role = Some(DateRole::Effective);
                        date_line = Some(line);
                        replaced = true;
                    }
                    None => {
                        document_date = None;
                        date_role = None;
                        date_line = None;
                        date_supported = false;
                        support.document_date = Support::Unsupported;
                    }
                }
            }
        }
    }
    if !date_supported {
        push(&mut reasons, ReviewReason::DateUnsupported);
    }
    // A deadline: labelled as one in either view.
    if !replaced && let Some(date) = document_date.clone() {
        let fires_context = deadline_fires(context, &date);
        let fires_document = deadline_fires(document, &date);
        if fires_context || fires_document {
            push(&mut reasons, ReviewReason::DateIsDeadline);
            support.guard_scope = Some(scope_name(fires_context, fires_document));
            match single_agreed(
                issue_date_alternates(context, &date),
                issue_date_alternates(document, &date),
            ) {
                Some((alternate, line)) => {
                    let invoice = document_type
                        .as_deref()
                        .is_some_and(|value| value.to_lowercase().contains("invoice"));
                    document_date = Some(alternate);
                    date_role = invoice.then_some(DateRole::Invoice);
                    date_line = Some(line);
                    replaced = true;
                }
                None => {
                    document_date = None;
                    date_role = None;
                    date_line = None;
                    withheld_as_deadline = true;
                }
            }
        }
    }
    // A signed-on date loses to a stated effective or start date: an
    // agreement or a form takes effect when it says it does, not on the day
    // it was signed. Only the one effective date both views agree on
    // replaces it.
    if !replaced
        && matches!(
            DocumentClass::of(document_type.as_deref()),
            DocumentClass::Agreement | DocumentClass::Form
        )
        && let Some(date) = document_date.clone()
    {
        let cited = scope.cited_units(&facts.date_evidence);
        let wording = (!cited.is_empty())
            .then(|| infer_date_role(&scope.view(&cited), &date, document_type.as_deref()))
            .flatten()
            .or_else(|| infer_date_role(context, &date, document_type.as_deref()));
        if wording == Some(DateRole::Execution)
            && let Some((alternate, line)) = single_agreed(
                effective_alternates(context, &date),
                effective_alternates(document, &date),
            )
        {
            document_date = Some(alternate);
            date_role = Some(DateRole::Effective);
            date_line = Some(line);
            replaced = true;
        }
    }
    if document_date.is_none() && date_supported && !withheld_as_deadline {
        push(&mut reasons, ReviewReason::DateMissing);
    }
    if let Some(date) = document_date.as_deref()
        && !year_is_plausible(date, current_year)
    {
        push(&mut reasons, ReviewReason::DateImplausible);
    }
    if let Some(date) = document_date.as_deref()
        && (reading_is_unsettled(context, date) || reading_is_unsettled(document, date))
    {
        push(&mut reasons, ReviewReason::DateAmbiguous);
    }
    // The wording around the date says what kind of date it is: the cited
    // units first, then the context, and the reply's role only where the
    // document says nothing.
    let date_role = document_date.as_deref().and_then(|date| {
        let cited = scope.cited_units(&facts.date_evidence);
        (!replaced && !cited.is_empty())
            .then(|| infer_date_role(&scope.view(&cited), date, document_type.as_deref()))
            .flatten()
            .or_else(|| infer_date_role(context, date, document_type.as_deref()))
            .or(date_role)
    });

    // The parties: each name as the document writes it, without a label or
    // an address the layout ran into it, and with the role the document
    // supports, or none.
    let mut parties: Vec<(ValidatedParty, Option<u32>)> = Vec::new();
    let mut parties_supported = true;
    for party in &facts.parties {
        let Some(written) = content(Some(party.name.as_str()), scope) else {
            parties_supported = false;
            continue;
        };
        let trimmed = trim_name(written);
        let trimmed = completed_name(scope, &trimmed).unwrap_or(trimmed);
        let name = trimmed.as_str();
        // A first name on its own is never a party: one word, not an
        // initialism, with nothing that makes it an organisation.
        if is_first_name_alone(name) {
            support.parties.push(Support::Unsupported);
            parties_supported = false;
            continue;
        }
        let loose = normalize_loosely(name);
        let normalized = normalize(name);
        let found = find(
            scope,
            &party.evidence,
            |view| digest_contains(view, name) || digest_contains_loosely(view, name),
            |unit| {
                line_where(unit, |line| {
                    contains_whole(line, &normalized)
                        || contains_whole(&normalize_loosely(line), &loose)
                })
            },
        );
        support.miscited_ids += found.miscited;
        support.parties.push(found.support);
        if found.support == Support::Unsupported {
            parties_supported = false;
            continue;
        }
        if parties
            .iter()
            .any(|(kept, _)| normalize_loosely(&kept.name) == loose)
        {
            continue;
        }
        remember(&found, &mut references);
        let copied = copied(scope, name);
        let role_support = match party.role {
            None | Some(PartyRole::Other) => Support::Absent,
            Some(role) => role_support(scope, name, role, &party.evidence),
        };
        let supported = matches!(role_support, Support::Cited | Support::Context);
        let document_role = (!supported && !copied)
            .then(|| labelled_role(scope, name))
            .flatten();
        // The role the document supports: the reply's, else the one its
        // own wording gives, else none. Someone copied is a bystander.
        let role = if copied {
            Some(PartyRole::Other)
        } else if supported {
            party.role
        } else {
            document_role
        };
        let first_seen = first_appearance(scope, name);
        parties.push((
            ValidatedParty {
                name: display_name(name),
                role,
                proposed_role: party.role,
                role_support,
                document_role,
                copied,
                signatory: false,
                support: found.support,
                evidence: found
                    .unit
                    .map(|unit| vec![unit.id.clone()])
                    .unwrap_or_default(),
            },
            first_seen,
        ));
        if parties.len() == crate::client::MAX_PARTIES {
            break;
        }
    }
    if !parties_supported {
        push(&mut reasons, ReviewReason::PartyUnsupported);
    }
    // In the order the document first names them, the reply's order
    // breaking ties.
    parties.sort_by_key(|(_, first_seen)| first_seen.unwrap_or(u32::MAX));
    let mut parties = parties
        .into_iter()
        .map(|(party, _)| party)
        .collect::<Vec<_>>();
    // A person who signs for an organisation among the parties acts for it
    // and is no party of their own.
    let organisations = parties
        .iter()
        .filter(|party| is_organisation(&party.name))
        .map(|party| normalize_loosely(&party.name))
        .collect::<Vec<_>>();
    for party in &mut parties {
        if !party.copied && signs_for(scope, &party.name, &organisations) {
            party.signatory = true;
            party.role = Some(PartyRole::Other);
        }
    }
    let class = DocumentClass::of(document_type.as_deref());
    // An issued document whose reply said what it is but named no one
    // who could have issued it - no one, or only parties a customer cue
    // labels - takes as its issuer the one organisation at the head of its
    // first page, before any customer.
    let customer_only = parties.iter().all(|party| {
        party.copied
            || party.signatory
            || labelled_role(scope, &party.name) == Some(PartyRole::Customer)
            || party
                .role
                .is_some_and(|role| RECEIVING_ROLES.contains(&role))
    });
    if class == DocumentClass::Issued
        && proposed_type.is_some()
        && type_supported
        && customer_only
        && let Some((name, unit)) = header_organisation(scope)
        && !parties
            .iter()
            .any(|party| normalize_loosely(&party.name) == normalize_loosely(&name))
    {
        let line = unit
            .text
            .lines()
            .find(|line| normalize_loosely(line).contains(&normalize_loosely(&name)))
            .unwrap_or(&unit.text)
            .trim()
            .to_owned();
        remember(
            &Found {
                support: Support::Context,
                unit: Some(unit),
                line: Some(line),
                miscited: 0,
            },
            &mut references,
        );
        parties.insert(
            0,
            ValidatedParty {
                name: display_name(&name),
                role: Some(PartyRole::Issuer),
                role_support: Support::Context,
                document_role: Some(PartyRole::Issuer),
                support: Support::Context,
                evidence: vec![unit.id.clone()],
                ..ValidatedParty::default()
            },
        );
    }
    // A notice or a letter whose reply named no one it is addressed to
    // takes as its addressee the one its first page's "To:" field names:
    // the field's own label says who the document is to.
    let addressed = parties.iter().any(|party| {
        !party.copied
            && !party.signatory
            && (party
                .role
                .is_some_and(|role| ADDRESSED_ROLES.contains(&role))
                || labelled_role(scope, &party.name)
                    .is_some_and(|role| ADDRESSED_ROLES.contains(&role)))
    });
    if matches!(class, DocumentClass::Notice | DocumentClass::Letter)
        && proposed_type.is_some()
        && type_supported
        && !addressed
        && parties.len() < crate::client::MAX_PARTIES
        && let Some((name, unit, line)) = addressee_field(scope)
        && !parties
            .iter()
            .any(|party| normalize_loosely(&party.name) == normalize_loosely(&name))
        && !copied(scope, &name)
    {
        remember(
            &Found {
                support: Support::Context,
                unit: Some(unit),
                line: Some(line),
                miscited: 0,
            },
            &mut references,
        );
        parties.insert(
            0,
            ValidatedParty {
                name: display_name(&name),
                role: Some(PartyRole::Addressee),
                role_support: Support::Context,
                document_role: Some(PartyRole::Addressee),
                support: Support::Context,
                evidence: vec![unit.id.clone()],
                ..ValidatedParty::default()
            },
        );
    }
    // Who the filename and the description can name: everyone but the
    // people copied and those who sign for a party, each with the role the
    // document supports.
    let cast = parties
        .iter()
        .filter(|party| !party.copied && !party.signatory)
        .map(|party| CastMember {
            name: party.name.clone(),
            role: party.role,
            role_supported: party.role.is_some(),
        })
        .collect::<Vec<_>>();
    let cues = RelationCues {
        issuer: cue_issuer(document_type.as_deref(), &cast, context).or_else(|| {
            (class == DocumentClass::Issued)
                .then(|| header_issuer(scope, &cast))
                .flatten()
        }),
        between: between_cue(context, &cast),
    };
    let relation = relation_from_roles(class, document_type.as_deref(), &cast, cues);

    // The subject, the identifier and the key facts: every number and name
    // in them must be in what the model was shown, and the subject and a
    // key fact must pass the word check ([`every_word_stated`]). Stray
    // invisible characters are taken out of both first; every check reads
    // them through [`normalize`], which ignores the rest as well.
    let stated_words = StatedWords::of(scope);
    let mut subject = None;
    let reply_subject = facts.subject.as_deref().map(without_stray_invisibles);
    if let Some(value) = content(reply_subject.as_deref(), scope) {
        let found = claims_found(scope, &facts.subject_evidence, value);
        support.subject = found.support;
        support.miscited_ids += found.miscited;
        if found.support == Support::Unsupported {
            push(&mut reasons, ReviewReason::DescriptionUnsupported);
        } else {
            // Passing the word check ([`every_word_stated`]), and grounded
            // in the units it cites, or - when the reply cited the wrong
            // line - in one unit of the context that holds every one of its
            // significant words. A subject that fails either is left out
            // without a review: it is optional, and never in the filename.
            let cited = scope.cited_units(&facts.subject_evidence);
            let words_of_subject = significant_words(value);
            let held_whole = !words_of_subject.is_empty()
                && scope.context_units().any(|unit| {
                    let view = scope.view(&[unit]);
                    words_of_subject
                        .iter()
                        .all(|word| digest_contains(&view, word))
                });
            if every_word_stated(value, &stated_words)
                && (subject_is_grounded(value, &scope.view(&cited)) || held_whole)
            {
                remember(&found, &mut references);
                subject = Some(value.to_owned());
            }
        }
    }
    let mut identifier = None;
    let mut identifier_line = None;
    if let Some(value) = content(facts.identifier.as_deref(), scope) {
        let loose = normalize_loosely(value);
        let found = find(
            scope,
            &facts.identifier_evidence,
            |view| digest_contains(view, value) || digest_contains_loosely(view, value),
            |unit| line_where(unit, |line| normalize_loosely(line).contains(&loose)),
        );
        support.identifier = found.support;
        support.miscited_ids += found.miscited;
        if found.support == Support::Unsupported {
            push(&mut reasons, ReviewReason::DescriptionUnsupported);
        } else {
            remember(&found, &mut references);
            let label = found
                .unit
                .and_then(|unit| identifier_label(scope.index, unit, found.line.as_deref(), value));
            identifier = Some((identifier_word(label.as_deref()), value.to_owned()));
            identifier_line = found.line.clone();
        }
    }
    // A key fact - only a reply in named fields gives them - is accepted
    // when its claims are supported, every amount [`money_in`] reads in it
    // (a currency before the number) is stated as the context writes it,
    // and it passes the word check ([`every_word_stated`]). Such an amount
    // stated otherwise is an unsupported claim; failing the word check
    // alone leaves it out, without a review. Any other amount, such as
    // "1,248.00 USD", is held to the word check only.
    let mut key_facts = Vec::new();
    let mut key_units = Vec::new();
    for fact in &facts.key_facts {
        let text = without_stray_invisibles(&fact.fact);
        let Some(value) = content(Some(text.as_str()), scope) else {
            continue;
        };
        let mut found = claims_found(scope, &fact.evidence, value);
        if amounts_in(value)
            .into_iter()
            .any(|money| !money_stated(scope, money))
        {
            found.support = Support::Unsupported;
        }
        support.key_facts.push(found.support);
        support.miscited_ids += found.miscited;
        if found.support == Support::Unsupported {
            push(&mut reasons, ReviewReason::DescriptionUnsupported);
        } else if every_word_stated(value, &stated_words) {
            remember(&found, &mut references);
            key_facts.push(value.to_owned());
            key_units.push(found.unit);
        }
    }

    // The description, composed from what was validated, then held to the
    // digest pipeline's rules. Values, not labels: a subject is what a
    // "Project:" field holds, never a "Bill to" line, the type again or a
    // party's name; an amount reads with the label the document gives it.
    // A reply that names no identifier leaves it to the document: the
    // number on the type's own title line, or a field labelled as this
    // kind of document's number.
    if identifier.is_none()
        && let Some(read) = read_identifier(scope, document_type.as_deref(), type_line.as_deref())
    {
        remember(
            &Found {
                support: Support::Context,
                unit: Some(read.unit),
                line: Some(read.line.clone()),
                miscited: 0,
            },
            &mut references,
        );
        identifier = Some((read.word, read.value));
        identifier_line = Some(read.line);
    }
    let identifier = identifier.filter(|(_, value)| {
        value.chars().any(|character| character.is_ascii_digit())
            && !repeats(value, document_type.as_deref())
    });
    // The words the type and the parties' names already say: a subject of
    // nothing else - "retail" for Quartz Meadow Retail LLC's packing slip -
    // says nothing more.
    let named_words = document_type
        .iter()
        .map(String::as_str)
        .chain(parties.iter().map(|party| party.name.as_str()))
        .flat_map(words)
        .collect::<BTreeSet<_>>();
    let says_something = |value: &str| -> Option<String> {
        let value = subject_value(value)?
            .trim()
            .trim_end_matches(['.', ';', ',']);
        let names_party = cast.iter().any(|party| {
            contains_whole(&normalize_loosely(value), &normalize_loosely(&party.name))
        });
        let subject_words = words(value)
            .into_iter()
            .filter(|word| word.len() > 2 && !STOPWORDS.contains(&word.as_str()))
            .collect::<Vec<_>>();
        let only_names = !subject_words.is_empty()
            && subject_words.iter().all(|word| named_words.contains(word));
        let holds_identifier = identifier
            .as_ref()
            .is_some_and(|(_, id)| normalize_loosely(value).contains(&normalize_loosely(id)));
        (!value.is_empty()
            && !opens_with_party_label(value)
            && !repeats(value, document_type.as_deref())
            && !names_party
            && !only_names
            && !holds_identifier
            && money_in(value).is_none())
        .then(|| value.to_owned())
    };
    // With no subject from the reply, a field that names one: the premises
    // a lease lets, the position an offer makes, the project.
    let subject = subject
        .as_deref()
        .and_then(says_something)
        .map(|value| unstated_names_lowered(&value, scope))
        .or_else(|| field_subject(scope).and_then(|value| says_something(&value)));
    // The amount: a key fact's, or the one the line a compact reply cited
    // for it states.
    let mut amount = key_facts.iter().zip(&key_units).find_map(|(fact, unit)| {
        let money = money_in(fact)?;
        Some((amount_label_of(fact, *unit, money), money.to_owned()))
    });
    // The amount the document labels as its total, or as what the
    // document is for: read from the document, not the reply.
    if amount.is_none()
        && let Some(read) = document_amount(scope, class)
    {
        remember(
            &Found {
                support: Support::Context,
                unit: Some(read.unit),
                line: Some(read.line.clone()),
                miscited: 0,
            },
            &mut references,
        );
        amount = Some((read.label, read.money));
    }
    if !facts.amount_evidence.is_empty() {
        let cited = scope.cited_units(&facts.amount_evidence);
        match cited
            .first()
            .and_then(|unit| amount_in(unit).map(|found| (*unit, found)))
        {
            Some((unit, (money, line))) => {
                support.amount = Support::Cited;
                remember(
                    &Found {
                        support: Support::Cited,
                        unit: Some(unit),
                        line: Some(line.clone()),
                        miscited: 0,
                    },
                    &mut references,
                );
                if amount.is_none() {
                    amount = Some((amount_label_of(&line, Some(unit), &money), money));
                }
            }
            None => {
                // A line with no amount on it: an optional fact left out,
                // and the id counted as a miscitation.
                support.amount = Support::Unsupported;
                support.miscited_ids += cited.len().max(1) as u32;
            }
        }
    }
    let other_fact = key_facts
        .iter()
        .filter(|fact| money_in(fact).is_none())
        .find_map(|fact| says_something(fact));
    let identifier_in_title = match (&identifier, &type_line) {
        (Some((_, value)), Some(line)) => {
            identifier_line.as_deref() == Some(line.as_str())
                && normalize_loosely(line).contains(&normalize_loosely(value))
        }
        _ => false,
    };
    let date_surface = document_date
        .as_deref()
        .zip(date_line.as_deref())
        .and_then(|(date, line)| surface_form(date, line));
    let billed_to = (class == DocumentClass::Issued)
        .then(|| customer_field(scope))
        .flatten();
    let composed = describe(&DescriptionFacts {
        class,
        document_type: type_title.as_deref().or(document_type.as_deref()),
        relation: Some(&relation),
        parties: &cast,
        subject: subject.as_deref(),
        identifier: identifier
            .as_ref()
            .map(|(word, value)| (*word, value.as_str())),
        identifier_in_title,
        amount: amount
            .as_ref()
            .map(|(label, money)| (label.as_deref(), money.as_str())),
        other_fact: other_fact.as_deref(),
        date_surface: date_surface.as_deref(),
        billed_to: billed_to.as_deref(),
    });
    let description = validate_description(&composed, context, &mut reasons);

    if !candidate.confidence.is_finite() || candidate.confidence < READY_CONFIDENCE {
        push(&mut reasons, ReviewReason::LowConfidence);
    }
    if candidate.needs_review {
        push(&mut reasons, ReviewReason::ModelRequestedReview);
    }
    if context
        .parser_warnings()
        .iter()
        .any(|warning| warning.field_affecting)
    {
        push(&mut reasons, ReviewReason::ParserWarning);
    }

    // A title too long to name both sides loses what follows its kind's
    // "and" - a "Settlement Agreement and Mutual Release" is a Settlement
    // Agreement - before the filename loses a party to fit.
    let document_type = document_type
        .map(|value| fit_type(&value, &relation, document_date.as_deref()).unwrap_or(value));

    let party_lines = relation
        .parties
        .iter()
        .filter_map(|name| {
            references
                .iter()
                .find(|reference| {
                    contains_whole(&normalize(&reference.text), &normalize(name))
                        || normalize_loosely(&reference.text).contains(&normalize_loosely(name))
                })
                .map(|reference| reference.text.clone())
        })
        .collect::<Vec<_>>();
    let status = if reasons.is_empty() {
        ProposalStatus::Ready
    } else {
        ProposalStatus::NeedsReview
    };
    ValidationOutcome {
        proposal: ValidatedProposal {
            document_type,
            document_date,
            date_role,
            parties: relation.parties.clone(),
            party_relation: relation.relation,
            description,
            confidence: candidate.confidence,
            evidence: Evidence {
                date: date_line,
                document_type: type_line,
                parties: party_lines,
            },
        },
        status,
        reasons,
        candidate: original,
        facts: Some(ValidatedFacts {
            parties,
            subject,
            identifier: identifier.map(|(_, value)| value),
            key_facts,
            document_class: class,
            relation_basis: relation.basis,
            support,
            evidence: references,
        }),
    }
}

/// Keeps the line that shows a found fact as evidence, once.
fn remember(found: &Found<'_>, references: &mut Vec<EvidenceRef>) {
    if let (Some(unit), Some(line)) = (found.unit, &found.line)
        && !references
            .iter()
            .any(|known| known.id == unit.id && known.text == *line)
    {
        references.push(EvidenceRef {
            id: unit.id.clone(),
            page: unit.page,
            text: line.clone(),
            source: unit.source,
            confidence: unit.confidence,
        });
    }
}

/// A reply's value, trimmed; blank is absent.
fn present(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// A reply's value when it says something about the document: present,
/// not a placeholder ("..", "null") and not an evidence id written where a
/// value belongs ("p1.b1"). None of those is a statement the document could
/// support or contradict; each is as good as left out.
fn content<'v>(value: Option<&'v str>, scope: &ValidationScope<'_>) -> Option<&'v str> {
    let value = present(value)?;
    if is_placeholder(value) {
        return None;
    }
    let bare = value.trim_matches(|character: char| matches!(character, '[' | ']' | '"' | '\''));
    if scope.index.unit(bare).is_some() {
        return None;
    }
    Some(value)
}

/// How a reply's type reads in the document.
enum TypeReading<'a> {
    /// A cited unit's title line names the reply's kind of document: its
    /// phrase, whole.
    Title {
        phrase: String,
        found: Found<'a>,
    },
    /// The reply's own words, stated whole and not as the name of another
    /// document.
    Stated {
        found: Found<'a>,
    },
    /// The reply's words are not stated, but a title of the document names
    /// the same kind: its phrase, for a person to confirm.
    Retitled {
        phrase: String,
        line: String,
    },
    Unsupported,
}

/// Whether a line of a unit can be a title: a heading, a field, a row, or
/// a line short enough not to be a sentence.
fn title_like(unit: &EvidenceUnit, line: &str) -> bool {
    matches!(
        unit.kind,
        UnitKind::Heading | UnitKind::Field | UnitKind::TableRow | UnitKind::TableHeader
    ) || words(line).len() <= 8
}

/// Where a phrase may stand on a line to name the document: anywhere, but
/// on a "Re:" or "Subject:" line only first, right after the label - "Re:
/// Offer of Employment - ..." names a letter, "Subject: RE: Approval
/// needed: ... fleet monitoring renewal" names no email - and never as
/// another document's name ("Re: Residential Lease Agreement dated ...").
fn names_document_at(line_words: &[String], start: usize, end: usize, line: &str) -> bool {
    if is_reference(line_words, start, end) {
        return false;
    }
    if !names_a_matter(line) {
        return true;
    }
    let label = line_words
        .iter()
        .take_while(|word| {
            matches!(
                word.as_str(),
                "re" | "subject" | "subj" | "regarding" | "fw" | "fwd"
            )
        })
        .count();
    start == label
}

/// Whether a line's letters are mostly capitals, the way a title is set.
fn in_capitals(line: &str) -> bool {
    let letters = line.chars().filter(|character| character.is_alphabetic());
    let (upper, total) = letters.fold((0, 0), |(upper, total), letter| {
        (upper + usize::from(letter.is_uppercase()), total + 1)
    });
    total >= 4 && upper * 10 >= total * 8
}

/// Whether a line opens with "Re:", "Subject:" or "Regarding:".
fn names_a_matter(line: &str) -> bool {
    let opening = line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_lowercase();
    matches!(
        opening.trim_end_matches(':'),
        "re" | "subject" | "regarding" | "subj"
    ) && opening.ends_with(':')
}

/// A unit's lines as a title is read from them: a heading's lines are one
/// title wrapped across them.
fn title_lines(unit: &EvidenceUnit) -> Vec<String> {
    if unit.kind == UnitKind::Heading {
        vec![unit.text.split_whitespace().collect::<Vec<_>>().join(" ")]
    } else {
        unit.text.lines().map(str::to_owned).collect()
    }
}

/// Reads a reply's type against the document: the title phrase of a cited
/// title line that names the same kind of document - preferred when it
/// holds the reply's words or the reply's words are not on it - else the
/// reply's words where a unit states them whole, not as another document's
/// name, else a title of the document with the same kind.
fn read_type<'a>(scope: &ValidationScope<'a>, cited: &[String], value: &str) -> TypeReading<'a> {
    let reading = read_type_words(scope, cited, value);
    // A bare kind - "Agreement", "Minutes", "Declaration" - is the
    // document's title when a title of the same kind says more: "Seed
    // Production and Supply Agreement", "Commercial Package Policy
    // Declarations".
    let value_words = words(value);
    if let ([head], TypeReading::Stated { found }) = (value_words.as_slice(), &reading)
        && let Some((phrase, unit, line)) = title_units(scope).find_map(|unit| {
            title_lines(unit).into_iter().find_map(|line| {
                let phrase = title_phrase(&line, head)?;
                (title_like(unit, &line) && words(&phrase).len() > 1)
                    .then(|| (phrase, unit, line.trim().to_owned()))
            })
        })
    {
        let _ = found;
        return TypeReading::Title {
            phrase,
            found: Found {
                support: Support::Context,
                unit: Some(unit),
                line: Some(line),
                miscited: 0,
            },
        };
    }
    reading
}

/// [`read_type`] before a bare kind is read as the document's title.
fn read_type_words<'a>(
    scope: &ValidationScope<'a>,
    cited: &[String],
    value: &str,
) -> TypeReading<'a> {
    let value_words = words(value);
    let Some(head) = head_noun(&value_words).map(str::to_owned) else {
        return TypeReading::Unsupported;
    };
    // A type names a kind of document; a reply's phrase that does not -
    // a company's name, a street address - is no type, wherever it stands.
    let kind = has_a_kind(&value_words);
    let stated_on = |line: &str| {
        let line_words = words(line);
        kind && phrase_positions(&line_words, &value_words)
            .into_iter()
            .any(|at| names_document_at(&line_words, at, at + value_words.len(), line))
    };
    // A title phrase of a line, where it names this document.
    let title_on = |line: &str| {
        let phrase = title_phrase(line, &head)?;
        let line_words = words(line);
        let phrase_words = words(&phrase);
        phrase_positions(&line_words, &phrase_words)
            .into_iter()
            .any(|at| names_document_at(&line_words, at, at + phrase_words.len(), line))
            .then_some(phrase)
    };
    let found_in = |unit: &'a EvidenceUnit, line: &str, support: Support| Found {
        support,
        unit: Some(unit),
        line: Some(line.trim().to_owned()),
        miscited: 0,
    };
    let cited_units = scope.cited_units(cited);
    for unit in &cited_units {
        for line in title_lines(unit) {
            if !title_like(unit, &line) {
                continue;
            }
            let Some(phrase) = title_on(&line) else {
                continue;
            };
            let holds_reply = !phrase_positions(&words(&phrase), &value_words).is_empty();
            if holds_reply || !stated_on(&line) {
                return TypeReading::Title {
                    phrase,
                    found: found_in(unit, &line, Support::Cited),
                };
            }
            return TypeReading::Stated {
                found: found_in(unit, &line, Support::Cited),
            };
        }
    }
    for unit in &cited_units {
        if let Some(line) = unit.text.lines().find(|line| stated_on(line)) {
            return TypeReading::Stated {
                found: found_in(unit, line, Support::Cited),
            };
        }
    }
    let miscited = cited_units.len() as u32;
    for unit in scope.context_units() {
        if let Some(line) = unit.text.lines().find(|line| stated_on(line)) {
            let mut found = found_in(unit, line, Support::Context);
            found.miscited = miscited;
            return TypeReading::Stated { found };
        }
    }
    for unit in title_units(scope) {
        for line in title_lines(unit) {
            if title_like(unit, &line)
                && let Some(phrase) = title_on(&line)
            {
                return TypeReading::Retitled {
                    phrase,
                    line: line.trim().to_owned(),
                };
            }
        }
    }
    TypeReading::Unsupported
}

/// The context's title units: the units at the head of its first page and
/// its first headings, in document order - a letterhead's company aside.
fn title_units<'s, 'a>(
    scope: &'s ValidationScope<'a>,
) -> impl Iterator<Item = &'a EvidenceUnit> + 's {
    let first_page = scope.context_units().map(|unit| unit.page).min();
    scope
        .context_units()
        .filter(move |unit| {
            unit.kind == UnitKind::Heading
                || (Some(unit.page) == first_page
                    && (unit.features.position.title
                        || unit.features.position.letterhead
                        || unit.features.position.page_index < 8))
        })
        .filter(|unit| !names_an_organisation(unit))
        .take(8)
}

/// Words a delivery line or a stamp is written in, never a title.
const NOT_TITLE_WORDS: &[&str] = &[
    "certified mail",
    "return receipt",
    "via ",
    "delivered by",
    "by email",
    "by hand",
    "confidential",
    "privileged",
];

/// Whether a unit is only an organisation's name - a letterhead's
/// "Meadowlark Health Plan" - or a delivery line: no title either way.
fn names_an_organisation(unit: &EvidenceUnit) -> bool {
    let text = normalize(&unit.text);
    let text = text.trim();
    NOT_TITLE_WORDS.iter().any(|words| text.contains(words))
        || unit
            .features
            .organisations
            .iter()
            .any(|name| normalize(name).trim() == text)
        || text.split_whitespace().any(|word| {
            crate::cues::ORGANISATION_ENDINGS.contains(
                &word
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '.')
                    .trim_end_matches('.'),
            )
        })
}

/// The type the document's title states when the reply gave none it
/// supports: the first title line that names a kind of document, and the
/// line.
fn title_of(scope: &ValidationScope<'_>) -> Option<(String, String)> {
    title_units(scope).find_map(|unit| {
        title_lines(unit).into_iter().find_map(|line| {
            // A letter's "Re:" line names it when its matter opens with a
            // kind of document: "Re: Demand for Payment - Highmeadow ...".
            if names_a_matter(line.trim_matches(|c: char| c == '*' || c.is_whitespace())) {
                let line = line.trim_matches(|c: char| c == '*' || c.is_whitespace());
                let line_words = words(line);
                let label = 1;
                let head = line_words
                    .get(label..)?
                    .iter()
                    .find(|word| names_a_kind(word))?;
                let phrase = title_phrase(line, head)?;
                let phrase_words = words(&phrase);
                return phrase_positions(&line_words, &phrase_words)
                    .into_iter()
                    .any(|at| {
                        at == label
                            && names_document_at(&line_words, at, at + phrase_words.len(), line)
                    })
                    .then(|| (phrase, line.to_owned()));
            }
            // A title in capitals whose capitals OCR scattered ("DElIvery
            // RECeIPt DR-771") is still set in capitals.
            if !title_like(unit, &line)
                || !(unit.kind == UnitKind::Heading || in_capitals(&tidy_case(&line)))
            {
                return None;
            }
            // The longest title any kind word on the line heads: "Lease
            // Agreement", not "Lease"; "Packing Slip", whatever follows.
            words(&line)
                .iter()
                .filter(|word| names_a_kind(word))
                .filter_map(|word| title_phrase(&line, word))
                .max_by_key(|phrase| phrase.split_whitespace().count())
                .map(|phrase| (phrase, line.trim().to_owned()))
        })
    })
}

/// A type the reply wrote all in lower case - "invoice", "notice of
/// default" - as a title: the words are the document's, the casing a
/// stylistic choice the model is not asked to make. Any capital in the
/// reply is kept as written.
fn titled(value: &str) -> String {
    if value.chars().any(char::is_uppercase) {
        return value.to_owned();
    }
    const SMALL: &[&str] = &[
        "a", "an", "and", "as", "at", "by", "for", "from", "in", "of", "on", "or", "the", "to",
        "under", "with",
    ];
    value
        .split(' ')
        .enumerate()
        .map(|(index, word)| {
            if index > 0 && SMALL.contains(&word) {
                return word.to_owned();
            }
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().chain(characters).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A stated type's value with each mark in it - a character that is no
/// letter, digit or space, and no accent on a kept letter - kept where the
/// line that states the type holds it too, compared through [`normalize`],
/// and a space otherwise. A mark at either end decorates the type rather
/// than spelling it and is left out, but for a bracket closing one the
/// type opens. A stray invisible character is left out; one a script
/// spells with ([`spells`]) is kept. "Invoice ✓✓", "Invoice ✔️" or
/// "Invoice ###" on an `INVOICE` line is `Invoice`; "Owner's Statement" on
/// `OWNER’S STATEMENT` or `OWNER´S STATEMENT` keeps its apostrophe. The
/// reply's words and their casing are left as written; [`titled`] cases an
/// all-lower-case reply afterwards.
fn marks_its_line_holds(value: &str, line: &str) -> String {
    // OCR and European keyboards write an apostrophe as an acute accent or
    // a backtick.
    let line = normalize(&line.replace(['\u{b4}', '`'], "'"));
    let mut after_letter = false;
    let kept = without_stray_invisibles(value)
        .chars()
        .map(|character| {
            let mark = normalize(&character.to_string());
            // An accent written apart from its letter stays with a kept
            // letter; the variation selector after a dropped "✔" goes.
            let accent = is_combining_mark(character) && after_letter;
            let held = character.is_alphanumeric()
                || character.is_whitespace()
                || accent
                || spells(character)
                || (!mark.is_empty() && line.contains(&mark));
            after_letter = character.is_alphanumeric() || accent;
            if held { character } else { ' ' }
        })
        .collect::<String>();
    let kept = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    let opens = kept.contains(['(', '[']);
    let edge = |character: char| {
        !(character.is_alphanumeric() || is_combining_mark(character) || spells(character))
    };
    kept.trim_start_matches(edge)
        .trim_end_matches(|character: char| {
            edge(character) && !(opens && matches!(character, ')' | ']'))
        })
        .to_owned()
}

/// Whether every word of `value` is a word of the document's type:
/// "Credit Agreement" for a Credit Agreement.
fn repeats(value: &str, document_type: Option<&str>) -> bool {
    let Some(document_type) = document_type else {
        return false;
    };
    let kind = normalize(document_type);
    let kind = kind.split_whitespace().collect::<Vec<_>>();
    normalize(value)
        .split_whitespace()
        .all(|word| kind.contains(&word))
}

/// Which view a guard fired in.
fn scope_name(context: bool, document: bool) -> String {
    match (context, document) {
        (true, true) => "both",
        (true, false) => "context",
        _ => "document",
    }
    .to_owned()
}

/// The one alternate both views agree on, when each has exactly one and it
/// is the same date: a replacement neither view would dispute.
fn single_agreed(
    context: Vec<(String, String)>,
    document: Vec<(String, String)>,
) -> Option<(String, String)> {
    match (context.as_slice(), document.as_slice()) {
        ([(left, line)], [(right, _)]) if left == right => Some((left.clone(), line.clone())),
        _ => None,
    }
}

/// The significant words of a type or a subject: longer than two letters,
/// not an article or a preposition.
fn significant_words(value: &str) -> Vec<String> {
    normalize(value)
        .split_whitespace()
        .map(|word| word.trim_matches(|character: char| !character.is_alphanumeric()))
        .filter(|word| {
            word.len() > 2 && !GENERIC_CAPITALS.contains(word) && !STOPWORDS.contains(word)
        })
        .map(str::to_owned)
        .collect()
}

/// Finds a free-text fact by its claims - its numbers and capitalised
/// names - as the digest pipeline checks a description's.
fn claims_found<'a>(scope: &ValidationScope<'a>, cited: &[String], value: &str) -> Found<'a> {
    // A leading word is never a claim in a sentence; here it is the fact's
    // own first word, so it is put after one that is not.
    let sentence = format!("About {value}");
    let words = significant_words(value);
    find(
        scope,
        cited,
        |view| first_unsupported_claim(&sentence, view).is_none() && mentions_any(view, &words),
        |unit| {
            line_where(unit, |line| {
                words.iter().any(|word| contains_whole(line, word))
            })
            .or_else(|| Some(unit.text.split_whitespace().collect::<Vec<_>>().join(" ")))
        },
    )
}

/// Whether a view states at least one of `words`, or there are none to
/// state: a fact made only of glue words is about nothing in particular.
fn mentions_any(view: &ScopeView, words: &[String]) -> bool {
    words.is_empty() || words.iter().any(|word| digest_contains(view, word))
}

/// Whether enough of a subject's significant words are in its cited units
/// to write it into the description: each distinct word counted once, so
/// a stated word said twice does not make up for one never stated. A
/// subject that fails this is still written when one unit of the context
/// holds all of them. Either way, it must also pass the word check
/// ([`every_word_stated`]).
fn subject_is_grounded(subject: &str, cited: &ScopeView) -> bool {
    if cited.is_empty() {
        return false;
    }
    let words = significant_words(subject)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if words.is_empty() {
        return false;
    }
    let held = words
        .iter()
        .filter(|word| digest_contains(cited, word))
        .count();
    held as f32 >= words.len() as f32 * SUBJECT_OVERLAP
}

/// What each unit of the context states, for the check of a subject's or
/// a key fact's words: its text through [`normalize`], and the words
/// [`words`] reads from it. A word the unit hyphenates across a line break
/// ("dis-\nplay") is read whole as well as split.
struct StatedWords(Vec<(String, Vec<String>)>);

impl StatedWords {
    fn of(scope: &ValidationScope<'_>) -> Self {
        Self(
            scope
                .context_units()
                .map(|unit| {
                    let mut stated = words(&unit.text);
                    stated.extend(words(&unit.text.replace("-\n", "")));
                    (normalize(&unit.text), stated)
                })
                .collect(),
        )
    }

    /// Whether some unit states `word`: whole, in any case, a plural's "s"
    /// aside ([`same_word`]).
    fn word(&self, word: &str) -> bool {
        self.0
            .iter()
            .any(|(_, unit)| unit.iter().any(|written| same_word(written, word)))
    }

    /// Whether some unit states `number` as a whole number, not the start
    /// or the end of a longer one ([`continues_number`]): "248" is not
    /// stated by "$1,248.00", and "10438" is by "INV-10438".
    fn number(&self, number: &str) -> bool {
        self.0.iter().any(|(text, _)| {
            text.match_indices(number).any(|(at, _)| {
                !continues_number(text[..at].chars().rev())
                    && !continues_number(text[at + number.len()..].chars())
            })
        })
    }

    /// Whether some unit writes `symbol`.
    fn symbol(&self, symbol: char) -> bool {
        self.0.iter().any(|(text, _)| text.contains(symbol))
    }

    /// Whether some unit writes `text` whole ([`contains_whole`]).
    fn whole(&self, text: &str) -> bool {
        self.0.iter().any(|(unit, _)| contains_whole(unit, text))
    }
}

/// `text` without its stray invisible characters ([`is_invisible`]) - a
/// soft hyphen, a zero-width space - but those a script spells with
/// ([`spells`]): a subject or a key fact is written without them. Every
/// check reads it through [`normalize`], which ignores all of them.
fn without_stray_invisibles(text: &str) -> String {
    text.chars()
        .filter(|character| !is_invisible(*character) || spells(*character))
        .collect()
}

/// Whether an invisible character is part of how text is spelled: a
/// zero-width non-joiner or joiner, which Persian, Indic scripts and emoji
/// spell with, or a direction mark, embedding or isolate.
fn spells(character: char) -> bool {
    matches!(
        character,
        '\u{61c}' | '\u{200c}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

/// The words of a subject or a key fact the context must state: [`words`]'
/// words of it - split at every mark that is not a letter or a digit, so
/// "shelving/Cryogenic" is two words - but the glue words ([`GLUE_WORDS`]),
/// single letters ("Dock C") and numbers, which are checked whole
/// ([`numbers_in`]). A word of letters and digits ("4c", "Q4") is a word.
fn checked_words(value: &str) -> Vec<String> {
    words(value)
        .into_iter()
        .filter(|word| {
            let single_letter = word.chars().count() == 1 && word.chars().all(char::is_alphabetic);
            let number = word.chars().all(|character| character.is_ascii_digit());
            !single_letter && !number && !GLUE_WORDS.contains(&word.as_str())
        })
        .collect()
}

/// The numbers `text` writes, read through [`normalize`]: each a longest
/// run of digits joined by single commas or points between digits -
/// "1,248.00", "2026", "4.5".
fn numbers_in(text: &str) -> Vec<String> {
    let text = normalize(text).chars().collect::<Vec<_>>();
    let mut numbers = Vec::new();
    let mut number = String::new();
    for (at, &character) in text.iter().enumerate() {
        let joins = matches!(character, ',' | '.')
            && !number.is_empty()
            && text.get(at + 1).is_some_and(char::is_ascii_digit);
        if character.is_ascii_digit() || joins {
            number.push(character);
        } else if !number.is_empty() {
            numbers.push(std::mem::take(&mut number));
        }
    }
    if !number.is_empty() {
        numbers.push(number);
    }
    numbers
}

/// Whether the context states what a subject or a key fact says, each
/// part in some unit: every word that must be stated ([`checked_words`]),
/// every contracted negation whole ("can't", whose "t" alone is a single
/// letter), every number whole ([`numbers_in`]) and every currency symbol
/// and percent sign ([`CURRENCY_SYMBOLS`], [`PERCENT_SIGNS`]). Where the
/// parts stand, and in what order, is not checked.
fn every_word_stated(value: &str, stated: &StatedWords) -> bool {
    let normalized = normalize(value);
    let mut negations = normalized
        .split(|character: char| !character.is_alphanumeric() && character != '\'')
        .map(|token| token.trim_matches('\''))
        .filter(|token| token.ends_with("n't"));
    checked_words(value).iter().all(|word| stated.word(word))
        && negations.all(|negation| stated.whole(negation))
        && numbers_in(value).iter().all(|number| stated.number(number))
        && normalized
            .chars()
            .filter(|character| {
                CURRENCY_SYMBOLS.contains(character) || PERCENT_SIGNS.contains(character)
            })
            .all(|symbol| stated.symbol(symbol))
}

/// Every amount of money a reply's text states ([`money_in`]), in order.
fn amounts_in(text: &str) -> Vec<&str> {
    let mut amounts = Vec::new();
    let mut offset = 0;
    while let Some(money) = text.get(offset..).and_then(money_in) {
        let at = offset + text[offset..].find(money).unwrap_or(0);
        amounts.push(money);
        offset = at + money.len();
    }
    amounts
}

/// Words that scale an amount: "$5.2 million".
const SCALE_WORDS: &[&str] = &["thousand", "million", "billion"];

/// Whether a unit of the context states an amount a reply wrote as it is
/// written: the same number standing on its own - never the end or the
/// start of a longer one ("$248.00" is not stated by "$1,248.00", "$1,248"
/// not by "$1,248.00") - with the reply's currency symbol or code written
/// beside it and the same scale word after it, or none on either side.
fn money_stated(scope: &ValidationScope<'_>, money: &str) -> bool {
    let Some(digits_at) = money.find(|character: char| character.is_ascii_digit()) else {
        return false;
    };
    let currency = money[..digits_at].trim();
    if currency.is_empty() {
        return false;
    }
    let rest = &money[digits_at..];
    let number_end = rest
        .find(|character: char| !(character.is_ascii_digit() || matches!(character, ',' | '.')))
        .unwrap_or(rest.len());
    let number = &rest[..number_end];
    let scale = scale_at(&rest[number_end..]);
    scope.context_units().any(|unit| {
        // Read without invisible characters, as every other check reads.
        let text = unit
            .text
            .chars()
            .filter(|character| !is_invisible(*character))
            .collect::<String>();
        let text = text.as_str();
        text.match_indices(number).any(|(at, _)| {
            let before = &text[..at];
            let after = &text[at + number.len()..];
            let written_scale = scale_at(after);
            let after_scale = written_scale.map_or(after, |word| &after.trim_start()[word.len()..]);
            !continues_number(before.chars().rev())
                && !continues_number(after.chars())
                && written_scale == scale
                && (currency_before(before, currency) || currency_after(after_scale, currency))
        })
    })
}

/// Whether the characters next to a number, read away from it, carry the
/// number on: a digit, or a separator with a digit beyond it.
fn continues_number(mut beside: impl Iterator<Item = char>) -> bool {
    match beside.next() {
        Some(character) if character.is_ascii_digit() => true,
        Some(',' | '.') => beside.next().is_some_and(|next| next.is_ascii_digit()),
        _ => false,
    }
}

/// The scale word right after a number, as [`SCALE_WORDS`] lists it.
fn scale_at(after: &str) -> Option<&'static str> {
    let tail = after.trim_start();
    SCALE_WORDS.iter().copied().find(|word| {
        tail.get(..word.len())
            .is_some_and(|written| written.eq_ignore_ascii_case(word))
            && !tail[word.len()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
    })
}

/// Whether `currency` is written right before a number, spaces aside, and
/// not as the end of a longer word.
fn currency_before(before: &str, currency: &str) -> bool {
    let before = before.trim_end();
    let Some(at) = before.len().checked_sub(currency.len()) else {
        return false;
    };
    before
        .get(at..)
        .is_some_and(|written| written.eq_ignore_ascii_case(currency))
        && !(currency.chars().all(char::is_alphabetic)
            && before[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric))
}

/// Whether `currency` is written right after a number (and its scale),
/// spaces aside, and not as the start of a longer word.
fn currency_after(after: &str, currency: &str) -> bool {
    let after = after.trim_start();
    after
        .get(..currency.len())
        .is_some_and(|written| written.eq_ignore_ascii_case(currency))
        && !(currency.chars().all(char::is_alphabetic)
            && after[currency.len()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric))
}

/// The first unit of the document that names `name`.
fn first_appearance(scope: &ValidationScope<'_>, name: &str) -> Option<u32> {
    let normalized = normalize(name);
    let loose = normalize_loosely(name);
    scope
        .index
        .units()
        .iter()
        .filter(|unit| !unit.running)
        .find(|unit| {
            contains_whole(&unit.normalized, &normalized)
                || contains_whole(&normalize_loosely(&unit.text), &loose)
        })
        .map(|unit| unit.ordinal)
}

/// The words that state a role: as a label before a name (`Bill To:`,
/// `Landlord:`, `Dear`), and as a defined term after one (`("Tenant")`,
/// `, as Lender`).
fn role_words(role: PartyRole) -> (&'static [&'static str], &'static [&'static str]) {
    match role {
        PartyRole::Client => (&["client", "prepared for"], &["client"]),
        PartyRole::Contractor => (
            &["contractor", "consultant", "service provider", "provider"],
            &[
                "contractor",
                "consultant",
                "service provider",
                "provider",
                "supplier",
                "firm",
            ],
        ),
        PartyRole::Employer => (&["employer"], &["employer", "company"]),
        PartyRole::Employee => (&["employee"], &["employee", "executive"]),
        PartyRole::Buyer => (&["buyer", "purchaser", "sold to"], &["buyer", "purchaser"]),
        PartyRole::Seller => (&["seller", "vendor"], &["seller", "vendor", "supplier"]),
        PartyRole::Landlord => (
            &["landlord", "lessor", "owner"],
            &["landlord", "lessor", "owner"],
        ),
        PartyRole::Tenant => (&["tenant", "lessee", "resident"], &["tenant", "lessee"]),
        PartyRole::Issuer => (
            &[
                "issuer",
                "issued by",
                "remit to",
                "payable to",
                "from",
                "prepared by",
                "presented by",
            ],
            &["issuer"],
        ),
        PartyRole::Recipient => (
            &[
                "recipient",
                "to",
                "issued to",
                "bill to",
                "billed to",
                "attn",
                "attention",
                "deliver to",
                "consignee",
            ],
            &["recipient"],
        ),
        PartyRole::Vendor => (&["vendor", "supplier", "remit to"], &["vendor", "supplier"]),
        PartyRole::Customer => (
            &[
                "customer",
                "bill to",
                "billed to",
                "sold to",
                "ship to",
                "invoice to",
                "client",
            ],
            &["customer", "client"],
        ),
        PartyRole::Borrower => (&["borrower"], &["borrower", "maker"]),
        PartyRole::Lender => (&["lender"], &["lender", "holder", "payee"]),
        PartyRole::Licensor => (&["licensor"], &["licensor"]),
        PartyRole::Licensee => (&["licensee"], &["licensee"]),
        PartyRole::Sender => (
            &["from", "sender", "prepared by", "presented by"],
            &["sender"],
        ),
        PartyRole::Addressee => (
            &["to", "attn", "attention", "dear", "addressee"],
            &["addressee"],
        ),
        PartyRole::Other => (&[], &[]),
    }
}

/// Whether the document supports `role` for the party `name`: in a cited
/// unit ([`Support::Cited`]) or another excerpt naming it
/// ([`Support::Context`]) - a role word labelling the name, a defined term
/// after it, a page-one letterhead for an issuer or sender, or the issuer
/// and customer cues of an issued document on its line.
fn role_support(
    scope: &ValidationScope<'_>,
    name: &str,
    role: PartyRole,
    cited: &[String],
) -> Support {
    let normalized = normalize(name);
    let loose = normalize_loosely(name);
    let letter = LetterLayout::of(scope);
    let supports =
        |unit: &EvidenceUnit| unit_supports_role(unit, &normalized, &loose, role, true, letter);
    if scope.cited_units(cited).into_iter().any(supports) {
        return Support::Cited;
    }
    if scope.context_units().any(supports) {
        return Support::Context;
    }
    Support::Unsupported
}

/// The roles a document's wording can give a party, the sides a document is
/// about and the sides it comes from first, the generic ones last.
const LABELLED_ROLES: [PartyRole; 18] = [
    PartyRole::Tenant,
    PartyRole::Landlord,
    PartyRole::Borrower,
    PartyRole::Lender,
    PartyRole::Employee,
    PartyRole::Employer,
    PartyRole::Licensee,
    PartyRole::Licensor,
    PartyRole::Buyer,
    PartyRole::Seller,
    PartyRole::Client,
    PartyRole::Contractor,
    PartyRole::Customer,
    PartyRole::Vendor,
    PartyRole::Issuer,
    PartyRole::Sender,
    PartyRole::Recipient,
    PartyRole::Addressee,
];

/// The first role, in [`LABELLED_ROLES`]' order, that what the model was
/// shown gives `name` in its own words: a label, a defined term, a
/// letterhead, an issuer or customer cue.
fn labelled_role(scope: &ValidationScope<'_>, name: &str) -> Option<PartyRole> {
    let normalized = normalize(name);
    let loose = normalize_loosely(name);
    let naming = scope
        .context_units()
        .filter(|unit| {
            contains_whole(&unit.normalized, &normalized)
                || contains_whole(&normalize_loosely(&unit.text), &loose)
        })
        .collect::<Vec<_>>();
    let letter = LetterLayout::of(scope);
    LABELLED_ROLES.into_iter().find(|role| {
        naming
            .iter()
            .any(|unit| unit_supports_role(unit, &normalized, &loose, *role, false, letter))
    })
}

/// Where a letter's first page ends its letterhead: at a dateline standing
/// alone near the top of a page that greets someone ("Dear ..."). The
/// lines after it are the inside address - who the letter is to - and
/// never the letterhead of who it is from. A receipt's or a form's date
/// line ends nothing.
#[derive(Clone, Copy, Debug, Default)]
struct LetterLayout {
    /// The ordinal of the dateline.
    dateline: Option<u32>,
}

impl LetterLayout {
    fn of(scope: &ValidationScope<'_>) -> Self {
        let units = scope.index.units();
        let first_page = units.iter().map(|unit| unit.page).min();
        let page = || {
            units
                .iter()
                .filter(move |unit| Some(unit.page) == first_page && !unit.running)
        };
        let greets = page().any(|unit| !whole_positions(&unit.normalized, "dear").is_empty());
        let dateline = page()
            .take(6)
            .find(|unit| is_bare_date(unit))
            .map(|unit| unit.ordinal)
            .filter(|_| greets);
        Self { dateline }
    }

    /// Whether a unit stands before the dateline, where a letterhead can.
    fn before_dateline(self, unit: &EvidenceUnit) -> bool {
        self.dateline.is_none_or(|dateline| unit.ordinal < dateline)
    }
}

/// Whether a unit is a date and nothing else - "March 2, 2026" - as a
/// letter's dateline is written.
fn is_bare_date(unit: &EvidenceUnit) -> bool {
    const MONTHS: &[&str] = &[
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    unit.label.is_none()
        && !unit.features.dates.is_empty()
        && words(&unit.text)
            .iter()
            .all(|word| MONTHS.contains(&word.as_str()) || word.chars().all(|c| c.is_ascii_digit()))
}

/// `cues`: whether an issued document's issuer and customer cues count.
/// They name the issuing and the billed side, not which of the issuing
/// side's roles - vendor, seller, issuer - a party has, so a role read from
/// the document alone takes them only as issuer and customer.
fn unit_supports_role(
    unit: &EvidenceUnit,
    normalized: &str,
    loose: &str,
    role: PartyRole,
    cues: bool,
    letter: LetterLayout,
) -> bool {
    let (labels, terms) = role_words(role);
    let mut positions = whole_positions(&unit.normalized, normalized)
        .into_iter()
        .map(|at| (unit.normalized.as_str(), at, normalized.len()))
        .collect::<Vec<_>>();
    let loose_text = normalize_loosely(&unit.text);
    if positions.is_empty() {
        positions = whole_positions(&loose_text, loose)
            .into_iter()
            .map(|at| (loose_text.as_str(), at, loose.len()))
            .collect();
    }
    if positions.is_empty() {
        return false;
    }
    // A field's or a row's label: "Landlord: ...", "| Tenant | ... |".
    if let Some(label) = &unit.label {
        let label = normalize(label);
        let label = label.trim_end_matches([':', '.']).trim();
        if labels.iter().any(|word| label_names(label, word)) {
            return true;
        }
    }
    if labelled_on_its_line(unit, normalized, loose, labels) {
        return true;
    }
    for (text, at, length) in positions {
        let after = window_forward(text, at + length, DEFINED_TERM_REACH);
        if defines_role(after, terms) {
            return true;
        }
    }
    // A letterhead is a few short lines at the top of the first page, not
    // the opening paragraph that happens to be among them, and its name
    // opens a line of its own: a line OCR ran two names into is no one's
    // letterhead.
    if matches!(role, PartyRole::Issuer | PartyRole::Sender)
        && unit.features.position.letterhead
        && letter.before_dateline(unit)
        && unit.text.chars().count() <= LETTERHEAD_CHARACTERS
        && unit
            .text
            .lines()
            .any(|line| normalize_loosely(line).starts_with(loose))
    {
        return true;
    }
    let on_cue_line = |cues: &[&str]| {
        unit.text.lines().any(|line| {
            let line = normalize_loosely(line);
            line.contains(loose) && cues.iter().any(|cue| line.contains(cue))
        })
    };
    match role {
        PartyRole::Issuer => on_cue_line(ISSUER_CUES),
        PartyRole::Vendor | PartyRole::Seller if cues => on_cue_line(ISSUER_CUES),
        PartyRole::Customer => on_cue_line(CUSTOMER_CUES),
        PartyRole::Recipient | PartyRole::Buyer | PartyRole::Client if cues => {
            on_cue_line(CUSTOMER_CUES)
        }
        _ => false,
    }
}

/// Whether a line names the party after one of `labels` - "Bill To:
/// Contoso", "| Tenant | Imogen Castellanos |", "Dear Ms. Okafor" - or
/// stands under a line that is only such a label ("BILL TO" over the
/// customer's name). What stands before the name on its line must be the
/// label and nothing more, so "payable to Acme" is no "to".
fn labelled_on_its_line(
    unit: &EvidenceUnit,
    normalized: &str,
    loose: &str,
    labels: &[&str],
) -> bool {
    let lines = unit.text.lines().map(normalize).collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let at = whole_positions(line, normalized)
            .first()
            .copied()
            .or_else(|| {
                let loosened = normalize_loosely(line);
                whole_positions(&loosened, loose).first().map(|_| {
                    line.find(loose.split(' ').next().unwrap_or_default())
                        .unwrap_or(0)
                })
            });
        let Some(at) = at else {
            continue;
        };
        let prefix = window_back(line, at, LABEL_REACH);
        let prefix = prefix
            .rsplit('|')
            .next()
            .unwrap_or_default()
            .trim()
            .trim_end_matches([':', '-', ',', '#'])
            .trim();
        let prefix = if prefix.is_empty() {
            // The label on the line above, alone.
            lines[..index]
                .iter()
                .rev()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().trim_end_matches(':').trim())
                .unwrap_or_default()
        } else {
            prefix
        };
        if labels.iter().any(|word| label_names(prefix, word)) {
            return true;
        }
    }
    false
}

/// Whether a label is, or ends with, a role's label word: "Bill To",
/// "Landlord", "Name of Tenant".
/// A short word - "to", "from", "attn", "dear" - must be the whole label,
/// so "payable to" is not "to".
fn label_names(label: &str, word: &str) -> bool {
    label == word
        || (word.len() > 4
            && label.ends_with(word)
            && word_starts_at(label, label.len() - word.len()))
}

fn word_starts_at(text: &str, at: usize) -> bool {
    text.is_char_boundary(at)
        && !text[..at]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric)
}

/// A defined term naming the role right after a name: in the first
/// parenthesis - `("Tenant")`, `(the "Borrower")` - or in an `as` clause or
/// an appositive - `, as Lender`, `, Tenant`.
fn defines_role(after: &str, terms: &[&str]) -> bool {
    let has_term = |text: &str| {
        terms
            .iter()
            .any(|term| !whole_positions(text, term).is_empty())
    };
    let trimmed = after.trim_start_matches([',', ' ']);
    if let Some(open) = trimmed.find('(')
        && trimmed[..open]
            .chars()
            .filter(|character| *character == ' ')
            .count()
            <= 3
    {
        let close = trimmed[open..]
            .find(')')
            .map_or(trimmed.len(), |offset| open + offset);
        if has_term(&trimmed[open..close]) {
            return true;
        }
    }
    if let Some(rest) = trimmed.strip_prefix("as ") {
        return has_term(window_forward(rest, 0, 40));
    }
    terms.iter().any(|term| {
        trimmed.starts_with(term)
            && !trimmed[term.len()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
    })
}

/// Where `needle` stands in `haystack` as whole words.
fn whole_positions(haystack: &str, needle: &str) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    haystack
        .match_indices(needle)
        .map(|(at, _)| at)
        .filter(|&at| {
            let before = haystack[..at].chars().next_back();
            let after = haystack[at + needle.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
        .collect()
}

/// Up to `reach` bytes of `text` before `at`, on a character boundary.
fn window_back(text: &str, at: usize, reach: usize) -> &str {
    let mut start = at.saturating_sub(reach);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    &text[start..at]
}

/// Up to `reach` bytes of `text` from `at`, on a character boundary.
fn window_forward(text: &str, at: usize, reach: usize) -> &str {
    let at = at.min(text.len());
    let mut end = (at + reach).min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[at..end]
}

/// The party an issued document's own cues nominate as its issuer, among
/// the first two: the digest pipeline's repair, read over the context.
fn cue_issuer(
    document_type: Option<&str>,
    cast: &[CastMember],
    context: &ScopeView,
) -> Option<usize> {
    let [first, second, ..] = cast else {
        return None;
    };
    let (kept, relation) = repair_issued_relation(
        document_type,
        vec![first.name.clone(), second.name.clone()],
        PartyRelation::Between,
        context,
    );
    match (relation, kept.as_slice()) {
        (PartyRelation::From, [issuer]) => cast.iter().position(|party| party.name == *issuer),
        _ => None,
    }
}

/// A name as a filename and a description show it: a name OCR wrote with
/// scattered capitals in capitals throughout, for the filename's casing to
/// read as it reads any name in capitals.
fn display_name(name: &str) -> String {
    tidy_case(name)
}

/// The whole organisation name a reply's name is the end of - "MANUFACTURING
/// LLC" of "EMBer POSt MANUFACtURInG LLC" - when a unit of the context names
/// the organisation, as the document writes it. `None` when the name is no
/// organisation's end, or names it whole.
fn completed_name(scope: &ValidationScope<'_>, name: &str) -> Option<String> {
    let loose = normalize_loosely(name);
    let last = loose.split_whitespace().last()?;
    if !crate::cues::ORGANISATION_ENDINGS.contains(&last) {
        return None;
    }
    scope.context_units().find_map(|unit| {
        // What completes the name is a few words of its own, never another
        // organisation's legal form or noun: "Systems LLC and" is no part
        // of "Quill and Vane Advisory Group, Inc.".
        let organisation = unit.features.organisations.iter().find(|organisation| {
            let Some(before) = organisation
                .strip_suffix(loose.as_str())
                .filter(|before| before.ends_with(' '))
            else {
                return false;
            };
            let words = before.split_whitespace().collect::<Vec<_>>();
            (1..=4).contains(&words.len())
                && !words.iter().any(|word| {
                    crate::cues::ORGANISATION_ENDINGS.contains(word)
                        || matches!(*word, "and" | "&" | "of")
                })
        })?;
        written_span(unit, organisation)
    })
}

/// A subject with any run of capitalised words the document does not state
/// as one phrase written in lower case: "Credit Union Merger", put together
/// from a notice that writes "credit union" and "merger" apart, reads as
/// "credit union merger" - words of the document, not a name it never gives.
/// A run it states ("Aurora Catalog Project") and an initialism keep their
/// capitals.
fn unstated_names_lowered(subject: &str, scope: &ValidationScope<'_>) -> String {
    let tokens = subject.split(' ').collect::<Vec<_>>();
    let capitalised = |token: &str| {
        let core = token.trim_matches(|character: char| !character.is_alphanumeric());
        core.chars().next().is_some_and(char::is_uppercase)
            && !(core.chars().count() <= 4 && core.chars().all(|c| !c.is_lowercase()))
    };
    let stated = |run: &[&str]| {
        let wanted = words(&run.join(" "));
        scope.context_units().any(|unit| {
            unit.text
                .lines()
                .any(|line| !phrase_positions(&words(line), &wanted).is_empty())
        })
    };
    let mut lowered = tokens
        .iter()
        .map(|token| (*token).to_owned())
        .collect::<Vec<_>>();
    let mut at = 0;
    while at < tokens.len() {
        if !capitalised(tokens[at]) {
            at += 1;
            continue;
        }
        let mut end = at + 1;
        while end < tokens.len() && capitalised(tokens[end]) {
            end += 1;
        }
        if end - at >= 2 && !stated(&tokens[at..end]) {
            for token in &mut lowered[at..end] {
                *token = token.to_lowercase();
            }
        }
        at = end;
    }
    lowered.join(" ")
}

/// Whether a name is one capitalised word and nothing else - "Rowan",
/// "Priya" - the way a note names a person by first name. An initialism
/// ("IBM") or a word ending an organisation's name is not one.
fn is_first_name_alone(name: &str) -> bool {
    let mut parts = name.split_whitespace();
    let (Some(word), None) = (parts.next(), parts.next()) else {
        return false;
    };
    let letters = word.chars().filter(|character| character.is_alphabetic());
    let all_capitals = word.chars().filter(|c| c.is_alphabetic()).count() > 1
        && letters.clone().all(char::is_uppercase);
    word.chars().next().is_some_and(char::is_uppercase)
        && word
            .chars()
            .all(|character| character.is_alphabetic() || character == '-' || character == '\'')
        && !all_capitals
        && !crate::cues::ORGANISATION_ENDINGS.contains(&word.to_lowercase().as_str())
}

/// The roles of the side a document is sent or billed to.
const RECEIVING_ROLES: &[PartyRole] = &[
    PartyRole::Customer,
    PartyRole::Recipient,
    PartyRole::Client,
    PartyRole::Buyer,
    PartyRole::Addressee,
];

/// The organisation a customer field names - "Bill To: Ferncastle
/// Veterinary Hospital", a "SOLD TO" line over it - as the document writes
/// it.
fn customer_field(scope: &ValidationScope<'_>) -> Option<String> {
    scope.context_units().find_map(|unit| {
        let labelled = unit
            .label
            .as_deref()
            .is_some_and(|label| has_cue(&normalize_loosely(label), CUSTOMER_CUES))
            || unit
                .text
                .lines()
                .next()
                .is_some_and(|line| has_cue(&normalize_loosely(line), CUSTOMER_CUES));
        if !labelled {
            return None;
        }
        let organisation = unit.features.organisations.first()?;
        written_span(unit, organisation)
    })
}

/// Labels a field's value is what a document is about under.
const SUBJECT_FIELDS: &[&str] = &[
    "premises",
    "leased premises",
    "property",
    "property address",
    "suite",
    "unit",
    "apartment",
    "position",
    "job title",
    "project",
    "project name",
    "aircraft",
    "vessel",
    "equipment",
    "services",
    "scope of work",
    "description of work",
    "matter",
];

/// The value of the first field of the context labelled as what the
/// document is about, cut to eight words.
fn field_subject(scope: &ValidationScope<'_>) -> Option<String> {
    scope.context_units().find_map(|unit| {
        let label = normalize(unit.label.as_deref()?);
        let label = label.trim().trim_end_matches([':', '.']).trim();
        if !SUBJECT_FIELDS.contains(&label) {
            return None;
        }
        let value = match unit.kind {
            UnitKind::Field => label_and_value(&unit.text).map(|(_, value)| value.to_owned())?,
            UnitKind::TableRow => {
                let cells = unit
                    .text
                    .trim()
                    .trim_matches('|')
                    .split('|')
                    .map(str::trim)
                    .collect::<Vec<_>>();
                match cells.as_slice() {
                    [_, value] => (*value).to_owned(),
                    _ => return None,
                }
            }
            _ => return None,
        };
        let value = value
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" ");
        (!value.is_empty()).then_some(value)
    })
}

/// Whether a name is an organisation's: it ends on a legal form or an
/// organisation's noun.
fn is_organisation(name: &str) -> bool {
    normalize_loosely(name)
        .split_whitespace()
        .last()
        .is_some_and(|word| crate::cues::ORGANISATION_ENDINGS.contains(&word))
}

/// Words a person's job title is made of.
const JOB_TITLE_WORDS: &[&str] = &[
    "president",
    "director",
    "manager",
    "officer",
    "counsel",
    "secretary",
    "treasurer",
    "chief",
    "ceo",
    "cfo",
    "coo",
    "cto",
    "head",
    "partner",
    "principal",
    "administrator",
    "coordinator",
    "supervisor",
    "representative",
    "agent",
    "chair",
    "chairman",
    "chairperson",
    "owner",
    "controller",
];

/// Whether a person signs for one of `organisations`: a unit names them,
/// then a job title, then the organisation - "Harriet Voss, Vice President
/// of People Operations Northstar Lantern Works LLC", "Keziah Ambrose,
/// Owner, Briarport Coffee Roasters LLC".
fn signs_for(scope: &ValidationScope<'_>, name: &str, organisations: &[String]) -> bool {
    if is_organisation(name) || organisations.is_empty() {
        return false;
    }
    let loose = normalize_loosely(name);
    scope.context_units().any(|unit| {
        let text = normalize_loosely(&unit.text);
        whole_positions(&text, &loose).into_iter().any(|at| {
            let after = window_forward(&text, at + loose.len(), 160);
            let titled = after
                .split_whitespace()
                .take(6)
                .any(|word| JOB_TITLE_WORDS.contains(&word));
            titled
                && organisations
                    .iter()
                    .any(|organisation| !whole_positions(after, organisation).is_empty())
        })
    })
}

/// Roles a notice or a letter gives the one it is addressed to.
const ADDRESSED_ROLES: &[PartyRole] = &[PartyRole::Addressee, PartyRole::Recipient];

/// Labels of a field that names who a notice or a letter is to. An
/// "Attn:" line names the person who reads it for the organisation it is
/// addressed to, never the addressee itself.
const ADDRESSEE_LABELS: &[&str] = &["to", "addressee"];

/// The person or organisation a first-page "To:" field names, as the
/// document writes it, with its unit and line: the name the field's value
/// opens with, an address after it left out ("To: John Smith, 1420 Fielder
/// Lane, ...").
fn addressee_field<'a>(scope: &ValidationScope<'a>) -> Option<(String, &'a EvidenceUnit, String)> {
    let first_page = scope.context_units().map(|unit| unit.page).min()?;
    scope
        .context_units()
        .filter(|unit| unit.page == first_page && !unit.running)
        .find_map(|unit| {
            let label = normalize(unit.label.as_deref()?);
            let label = label.trim().trim_end_matches([':', '.']).trim();
            if !ADDRESSEE_LABELS.contains(&label) {
                return None;
            }
            let line = unit.text.lines().next()?.trim();
            let value = line.split_once(':').map_or(line, |(_, value)| value);
            let opening = normalize_loosely(&trim_name(value.trim()));
            let named = unit
                .features
                .people
                .iter()
                .chain(&unit.features.organisations)
                .filter(|known| {
                    !known.is_empty()
                        && opening.starts_with(known.as_str())
                        && opening[known.len()..]
                            .chars()
                            .next()
                            .is_none_or(|next| !next.is_alphanumeric())
                })
                .max_by_key(|known| known.len())?;
            let span = written_span(unit, named)?;
            (!is_first_name_alone(&span)).then(|| (span, unit, line.to_owned()))
        })
}

/// The one organisation the first page of the context names before any
/// line with a customer cue, as the document writes it, and its unit.
fn header_organisation<'a>(scope: &ValidationScope<'a>) -> Option<(String, &'a EvidenceUnit)> {
    let units = scope.context_units().collect::<Vec<_>>();
    let first_page = units.iter().map(|unit| unit.page).min()?;
    let mut found: Vec<(String, &EvidenceUnit)> = Vec::new();
    for unit in units.iter().filter(|unit| unit.page == first_page) {
        if has_cue(&normalize_loosely(&unit.text), CUSTOMER_CUES)
            || unit
                .label
                .as_deref()
                .is_some_and(|label| has_cue(&normalize_loosely(label), CUSTOMER_CUES))
        {
            break;
        }
        for organisation in &unit.features.organisations {
            if found
                .iter()
                .any(|(known, _)| normalize_loosely(known) == *organisation)
            {
                continue;
            }
            if let Some(span) = written_span(unit, organisation) {
                found.push((span, unit));
            }
        }
    }
    match found.as_slice() {
        [only] => Some(only.clone()),
        _ => None,
    }
}

/// The words of a unit that read, loosely, as `loose`: the document's own
/// spelling of a name the index found.
fn written_span(unit: &EvidenceUnit, loose: &str) -> Option<String> {
    unit.text.lines().find_map(|line| {
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        (0..tokens.len()).find_map(|start| {
            (start + 1..=tokens.len().min(start + 10)).find_map(|end| {
                let span = tokens[start..end].join(" ");
                (normalize_loosely(&span) == loose).then(|| {
                    span.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '"'))
                        .to_owned()
                })
            })
        })
    })
}

/// Wording that copies someone in on a document rather than addressing it
/// to them.
const COPY_CUES: &[&str] = &["cc", "bcc", "copy to", "copies to", "copied to"];

/// Whether `text` holds one of `cues` as whole words.
fn has_cue(text: &str, cues: &[&str]) -> bool {
    cues.iter()
        .any(|cue| !whole_positions(text, cue).is_empty())
}

/// Whether every line of the context that names `name` copies them in -
/// "cc: Marcus Reyes, Esq., outside counsel" - so they are a bystander to
/// the document, never one of its parties.
fn copied(scope: &ValidationScope<'_>, name: &str) -> bool {
    let loose = normalize_loosely(name);
    let mut naming = 0;
    for unit in scope.context_units() {
        for line in unit.text.lines() {
            let line = normalize_loosely(line);
            let Some(at) = whole_positions(&line, &loose).first().copied() else {
                continue;
            };
            naming += 1;
            if !has_cue(&line[..at], COPY_CUES) {
                return false;
            }
        }
    }
    naming > 0
}

/// The one party at the head of an issued document's first page: named
/// before any line that names a customer, and not under a customer's label
/// itself. An invoice's, an order's or a slip's issuer is its letterhead
/// or header, and is seldom labelled as such; this is that structural cue.
/// `None` unless exactly one party stands there.
fn header_issuer(scope: &ValidationScope<'_>, cast: &[CastMember]) -> Option<usize> {
    let units = scope.context_units().collect::<Vec<_>>();
    let first_page = units.iter().map(|unit| unit.page).min()?;
    let page = units
        .iter()
        .filter(|unit| unit.page == first_page)
        .collect::<Vec<_>>();
    let heads = cast
        .iter()
        .enumerate()
        .filter(|(_, party)| {
            let loose = normalize_loosely(&party.name);
            let mut customer_seen = false;
            for unit in &page {
                let labelled = unit
                    .label
                    .as_deref()
                    .is_some_and(|label| has_cue(&normalize_loosely(label), CUSTOMER_CUES));
                let lines = unit.text.lines().map(normalize_loosely).collect::<Vec<_>>();
                for (index, line) in lines.iter().enumerate() {
                    if let Some(at) = whole_positions(line, &loose).first().copied() {
                        let under_label = index > 0
                            && lines[index - 1].split_whitespace().count() <= 3
                            && has_cue(&lines[index - 1], CUSTOMER_CUES);
                        return !customer_seen
                            && !labelled
                            && !under_label
                            && !has_cue(&line[..at], CUSTOMER_CUES);
                    }
                    if labelled || has_cue(line, CUSTOMER_CUES) {
                        customer_seen = true;
                    }
                }
            }
            false
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match heads.as_slice() {
        [only] => Some(*only),
        _ => None,
    }
}

/// Whether the context names the first two parties as the document's two
/// sides: "by and between Tessellate Analytics Ltd. and Contoso Worldwide,
/// Inc.".
fn between_cue(context: &ScopeView, cast: &[CastMember]) -> bool {
    let [first, second, ..] = cast else {
        return false;
    };
    let (first, second) = (
        normalize_loosely(&first.name),
        normalize_loosely(&second.name),
    );
    context.segments().iter().any(|segment| {
        let text = normalize_loosely(segment);
        whole_positions(&text, "between").into_iter().any(|at| {
            let after = window_forward(&text, at, 300);
            let named = |name: &str| whole_positions(after, name).first().copied();
            matches!((named(&first), named(&second)), (Some(a), Some(b)) if a != b)
        })
    })
}

/// Room an extension takes at the end of a filename, its dot included.
const EXTENSION_ROOM: usize = 6;

/// A shorter type for a filename that names two parties but would not fit
/// them: a compound title's first part when that part is itself a kind of
/// document ("Settlement Agreement" of "Settlement Agreement and Mutual
/// Release"). `None` when the name fits or there is no such part.
fn fit_type(document_type: &str, relation: &Relation, date: Option<&str>) -> Option<String> {
    let [first, second] = relation.parties.as_slice() else {
        return None;
    };
    if relation.relation != PartyRelation::Between {
        return None;
    }
    let length = |kind: &str| {
        date.map_or(0, |date| date.chars().count() + 1)
            + kind.chars().count()
            + " between ".len()
            + first.chars().count()
            + " and ".len()
            + second.chars().count()
    };
    let room = crate::naming::MAX_FILENAME_CHARS - EXTENSION_ROOM;
    if length(document_type) <= room {
        return None;
    }
    let lowered = document_type.to_lowercase();
    let at = lowered.find(" and ")?;
    let shorter = document_type[..at].trim();
    let shorter_words = words(shorter);
    let last = shorter_words.last()?;
    (names_a_kind(last)
        && head_noun(&shorter_words) == Some(last.as_str())
        && length(shorter) <= room)
        .then(|| shorter.to_owned())
}

/// The amount a line of the document states and the line: in a table row
/// its last amount (the row's total, or the new figure beside the old), in
/// a field its first, elsewhere the first after a word that names a total
/// ("total", "due", "sum") or else the first.
fn amount_in(unit: &EvidenceUnit) -> Option<(String, String)> {
    let mut found: Vec<(usize, String, String)> = Vec::new();
    for line in unit.text.lines() {
        let mut offset = 0;
        while let Some(money) = money_in(&line[offset..]) {
            let at = offset + line[offset..].find(money).unwrap_or(0);
            found.push((at, money.to_owned(), line.trim().to_owned()));
            offset = at + money.len();
        }
    }
    let totalled = |(at, _, line): &&(usize, String, String)| {
        let before = normalize(line.get(..*at).unwrap_or_default());
        before.split_whitespace().rev().take(6).any(|word| {
            matches!(
                word.trim_matches(|character: char| !character.is_alphanumeric()),
                "total" | "due" | "sum" | "balance" | "principal" | "price" | "amount"
            )
        })
    };
    let pick = match unit.kind {
        UnitKind::TableRow => found.last(),
        UnitKind::Field => found.first(),
        _ => found.iter().find(totalled).or_else(|| found.first()),
    }?;
    Some((pick.1.clone(), pick.2.clone()))
}

/// An amount the document states with a label that says what it is.
struct ReadAmount<'a> {
    label: Option<String>,
    money: String,
    line: String,
    unit: &'a EvidenceUnit,
}

/// Labels that say an amount is what a document is for: a price, a fee,
/// rent, a salary, a principal.
const AMOUNT_LABELS: &[&str] = &[
    "purchase price",
    "credit limit",
    "principal",
    "settlement payment",
    "base salary",
    "salary",
    "base rent",
    "monthly rent",
    "rent",
    "annual fee",
    "fixed fee",
    "fee",
    "retainer",
    "premium",
    "commitment",
    "price",
    "sum",
    "deposit",
];

/// Labels that say an amount is a document's total.
const TOTAL_LABELS: &[&str] = &[
    "total",
    "amount due",
    "balance due",
    "total due",
    "grand total",
    "amount payable",
    "new balance",
    "balance",
    "net amount",
    "net payment",
];

/// The amount the document labels as its total (an issued document's
/// last labelled total) or, for any other kind, the first amount labelled
/// as what the document is for (a price, a fee, rent, a salary, a
/// principal) or as its total: in a field, a two-cell row, or the words
/// just before it on a line. `None` when no amount is labelled so.
fn document_amount<'a>(
    scope: &ValidationScope<'a>,
    class: DocumentClass,
) -> Option<ReadAmount<'a>> {
    let named = |label: &str, labels: &[&str]| {
        let lowered = normalize(label);
        labels
            .iter()
            .find(|word| crate::evidence::contains_whole(&lowered, word))
            .map(|word| (*word).to_owned())
    };
    let mut found: Vec<(bool, ReadAmount<'a>)> = Vec::new();
    for unit in scope.context_units() {
        let candidates: Vec<(String, String, String)> = match unit.kind {
            UnitKind::Field => unit
                .label
                .clone()
                .zip(money_in(&unit.text).map(str::to_owned))
                .map(|(label, money)| vec![(label, money, unit.text.trim().to_owned())])
                .unwrap_or_default(),
            UnitKind::TableRow => {
                let cells = cells_of(&unit.text);
                let Some((at, money)) = cells
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(at, cell)| money_in(cell).map(|money| (at, money)))
                else {
                    continue;
                };
                // The cell that labels the amount: the nearest one before
                // it ("| | | Order Total (USD) | $84,438.00 |"), or, when
                // none does, the header's cell above it.
                let label = match cells[..at]
                    .iter()
                    .rev()
                    .find(|cell| !cell.is_empty() && money_in(cell).is_none())
                {
                    Some(cell) => Some((*cell).to_owned()),
                    None => unit
                        .table_header
                        .and_then(|header| {
                            scope
                                .index
                                .units()
                                .iter()
                                .find(|header_unit| header_unit.ordinal == header)
                        })
                        .and_then(|header| {
                            cells_of(&header.text)
                                .get(at)
                                .map(|cell| cell.trim_matches('*').trim().to_owned())
                        }),
                };
                match label {
                    Some(label) if is_amount_label(&label, money, &named) => {
                        vec![(label, money.to_owned(), unit.text.trim().to_owned())]
                    }
                    _ => Vec::new(),
                }
            }
            _ => unit
                .text
                .lines()
                .flat_map(|line| {
                    let mut out = Vec::new();
                    let mut offset = 0;
                    while let Some(money) = money_in(&line[offset..]) {
                        let at = offset + line[offset..].find(money).unwrap_or(0);
                        let before = window_back(line, at, 48);
                        out.push((before.to_owned(), money.to_owned(), line.trim().to_owned()));
                        offset = at + money.len();
                    }
                    out
                })
                .collect(),
        };
        for (label, money, line) in candidates {
            if let Some(total) = named(&label, TOTAL_LABELS) {
                found.push((
                    true,
                    ReadAmount {
                        label: Some(total),
                        money,
                        line,
                        unit,
                    },
                ));
            } else if let Some(kind) = named(&label, AMOUNT_LABELS).or_else(|| {
                // A field or a row that labels its value as an amount
                // ("Amount claimed", "Estimated amount of loss").
                matches!(unit.kind, UnitKind::Field | UnitKind::TableRow)
                    .then(|| named(&label, &["amount"]))
                    .flatten()
            }) {
                let label = if matches!(unit.kind, UnitKind::Field | UnitKind::TableRow) {
                    label.trim().trim_end_matches([':', '.']).to_lowercase()
                } else {
                    kind
                };
                found.push((
                    false,
                    ReadAmount {
                        label: Some(label),
                        money,
                        line,
                        unit,
                    },
                ));
            }
        }
    }
    // No labelled amount: one a first-page title states - "$350,000,000
    // Senior Secured Credit Facilities" - is what the document is for.
    if found.is_empty() && class != DocumentClass::Issued {
        let first_page = scope.context_units().map(|unit| unit.page).min()?;
        return title_units(scope).find_map(|unit| {
            if unit.page != first_page {
                return None;
            }
            let line = unit.text.lines().find(|line| {
                money_in(line).is_some() && (unit.kind == UnitKind::Heading || in_capitals(line))
            })?;
            Some(ReadAmount {
                label: None,
                money: money_in(line)?.to_owned(),
                line: line.trim().to_owned(),
                unit,
            })
        });
    }
    if class == DocumentClass::Issued {
        let (_, read) = found.into_iter().rfind(|(total, _)| *total)?;
        return Some(ReadAmount {
            label: None,
            ..read
        });
    }
    let index = found
        .iter()
        .position(|(total, _)| !*total)
        .or_else(|| (!found.is_empty()).then_some(0))?;
    Some(found.swap_remove(index).1)
}

/// A table row's cells, its outer pipes taken off.
fn cells_of(row: &str) -> Vec<&str> {
    row.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

/// Whether a row's cell labels the amount after it: not the amount
/// itself, and not an item's code or quantity - a label with a digit
/// ("Ending balance on March 31, 2026") only when it opens with a word and
/// names a total or an amount.
fn is_amount_label(
    label: &str,
    money: &str,
    named: &impl Fn(&str, &[&str]) -> Option<String>,
) -> bool {
    if label.is_empty() || label.contains(money) || money_in(label).is_some() {
        return false;
    }
    if !label.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }
    let opens_with_word = label
        .split_whitespace()
        .next()
        .is_some_and(|word| !word.chars().any(|c| c.is_ascii_digit()));
    opens_with_word
        && (named(label, TOTAL_LABELS).is_some()
            || named(label, AMOUNT_LABELS).is_some()
            || named(label, &["amount"]).is_some())
}

/// An identifier the document gives itself, read without the reply.
struct ReadIdentifier<'a> {
    value: String,
    /// The word its label gives it ([`identifier_word`]).
    word: &'static str,
    line: String,
    unit: &'a EvidenceUnit,
}

/// Whether a token is an identifier: letters, digits and joining marks,
/// at least one digit, and not a date, a year, an amount or a phone number.
fn looks_like_identifier(token: &str) -> bool {
    let core = token
        .trim_matches(|character: char| !character.is_alphanumeric())
        .trim_start_matches('#');
    let digits = core.chars().filter(char::is_ascii_digit).count();
    let letters = core
        .chars()
        .filter(|character| character.is_alphabetic())
        .count();
    (2..=30).contains(&core.chars().count())
        && digits > 0
        && (letters > 0 || digits >= 3)
        && core
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '-' | '/' | '.'))
        && !token.contains('$')
        && extract_stated_dates(core).is_empty()
        && numeric_dates(&normalize(core)).is_empty()
        && !(digits == 4 && core.len() == 4 && (core.starts_with("19") || core.starts_with("20")))
}

/// Words that label a number on a document: "No.", "#", "Number", "ID".
const NUMBER_WORDS: &[&str] = &["no", "number", "num", "id", "ref", "reference"];

/// The kinds of number a document of a kind carries, by the words their
/// labels use.
fn identifier_labels(head: &str) -> &'static [&'static str] {
    match head {
        "invoice" | "bill" => &["invoice", "inv"],
        "order" | "slip" | "confirmation" => &["order", "po", "slip", "confirmation"],
        "policy" | "declarations" | "certificate" => &["policy", "certificate"],
        "quote" | "quotation" | "estimate" => &["quote", "quotation", "estimate"],
        "receipt" => &["receipt"],
        "statement" => &["statement"],
        "notice" | "letter" | "claim" => &["claim", "loan", "policy", "case", "file"],
        _ => &[],
    }
}

/// The identifier a document states for itself: the number right after
/// its type on its title line ("PACKING SLIP PS-311"), or the value of a
/// field labelled as this kind's number ("Invoice No.: 7731-B", "| Policy
/// Number | CPP-4471 |"). `None` when it states neither.
fn read_identifier<'a>(
    scope: &ValidationScope<'a>,
    document_type: Option<&str>,
    type_line: Option<&str>,
) -> Option<ReadIdentifier<'a>> {
    let type_words = words(document_type?);
    let head = head_noun(&type_words)?.to_owned();
    if let Some(line) = type_line
        && let Some(unit) = scope.context_units().find(|unit| {
            unit.text.lines().any(|text| text.trim() == line.trim())
                || unit.text.split_whitespace().collect::<Vec<_>>().join(" ") == line.trim()
        })
        && let Some(value) = after_type(line, &type_words)
    {
        return Some(ReadIdentifier {
            value,
            word: "no.",
            line: line.trim().to_owned(),
            unit,
        });
    }
    // A field labelled as this kind's number; else, on the first page, any
    // field labelled as a number that is not a phone's, a tax id's or a
    // page's.
    let kinds = identifier_labels(&head);
    let first_page = scope.context_units().map(|unit| unit.page).min()?;
    let labelled = |unit: &'a EvidenceUnit, own_kind: bool| -> Option<ReadIdentifier<'a>> {
        let label = unit.label.as_deref()?;
        let label_words = words(label);
        let names_kind = label_words
            .iter()
            .any(|word| kinds.contains(&word.as_str()));
        let names_number = label.contains('#')
            || label_words
                .iter()
                .any(|word| NUMBER_WORDS.contains(&word.as_str()));
        let not_a_reference = label_words.iter().any(|word| {
            matches!(
                word.as_str(),
                "phone"
                    | "tel"
                    | "telephone"
                    | "fax"
                    | "tax"
                    | "ein"
                    | "tin"
                    | "ssn"
                    | "routing"
                    | "zip"
                    | "postal"
                    | "page"
                    | "vat"
                    | "license"
                    | "licence"
                    | "npi"
                    | "dea"
            )
        });
        if !names_number || not_a_reference || (own_kind && !names_kind) {
            return None;
        }
        let value = match unit.kind {
            UnitKind::TableRow => unit
                .text
                .trim()
                .trim_matches('|')
                .split('|')
                .nth(1)?
                .trim()
                .to_owned(),
            UnitKind::Field => label_and_value(&unit.text)
                .map(|(_, value)| value.to_owned())
                .unwrap_or_else(|| unit.text.clone()),
            _ => return None,
        };
        let token = value
            .split_whitespace()
            .next()
            .filter(|token| looks_like_identifier(token))?;
        Some(ReadIdentifier {
            value: token
                .trim_matches(|character: char| !character.is_alphanumeric())
                .to_owned(),
            word: identifier_word(Some(label)),
            line: unit.text.trim().to_owned(),
            unit,
        })
    };
    // A line that labels this kind's number and gives it after the label,
    // or on the next line; a table whose column is so labelled.
    let on_a_line = || {
        scope.context_units().find_map(|unit| {
            let lines = unit.text.lines().collect::<Vec<_>>();
            lines.iter().enumerate().find_map(|(index, line)| {
                let tokens = line.split_whitespace().collect::<Vec<_>>();
                let at = (0..tokens.len()).find(|&at| {
                    let here = words(tokens[at]);
                    here.iter().any(|word| kinds.contains(&word.as_str()))
                        && tokens[at + 1..]
                            .iter()
                            .take(2)
                            .chain(std::iter::once(&tokens[at]))
                            .any(|token| {
                                token.contains('#')
                                    || words(token)
                                        .iter()
                                        .any(|word| NUMBER_WORDS.contains(&word.as_str()))
                            })
                })?;
                let after = tokens[at + 1..]
                    .iter()
                    .find(|token| {
                        let token_words = words(token);
                        !(token_words.is_empty()
                            || token_words
                                .iter()
                                .all(|word| NUMBER_WORDS.contains(&word.as_str())))
                    })
                    .copied()
                    .or_else(|| {
                        lines
                            .get(index + 1)
                            .and_then(|next| next.split_whitespace().next())
                    })?;
                looks_like_identifier(after).then(|| ReadIdentifier {
                    value: after
                        .trim_matches(|character: char| !character.is_alphanumeric())
                        .to_owned(),
                    word: identifier_word(Some(&tokens[at..].join(" "))),
                    line: line.trim().to_owned(),
                    unit,
                })
            })
        })
    };
    let in_a_column = || {
        scope.context_units().find_map(|unit| {
            if unit.kind != UnitKind::TableRow {
                return None;
            }
            let header = unit
                .table_header
                .and_then(|ordinal| scope.index.units().get(ordinal as usize))?;
            let cells = |text: &str| {
                text.trim()
                    .trim_matches('|')
                    .split('|')
                    .map(|cell| cell.trim().to_owned())
                    .collect::<Vec<_>>()
            };
            let column = cells(&header.text).iter().position(|label| {
                let label_words = words(label);
                label_words
                    .iter()
                    .any(|word| kinds.contains(&word.as_str()))
                    && (label.contains('#')
                        || label_words
                            .iter()
                            .any(|word| NUMBER_WORDS.contains(&word.as_str())))
            })?;
            let cell = cells(&unit.text).into_iter().nth(column)?;
            let token = cell.split_whitespace().next()?.to_owned();
            looks_like_identifier(&token).then(|| ReadIdentifier {
                value: token
                    .trim_matches(|character: char| !character.is_alphanumeric())
                    .to_owned(),
                word: identifier_word(cells(&header.text).get(column).map(String::as_str)),
                line: unit.text.trim().to_owned(),
                unit,
            })
        })
    };
    scope
        .context_units()
        .find_map(|unit| (!kinds.is_empty()).then(|| labelled(unit, true)).flatten())
        .or_else(|| (!kinds.is_empty()).then(on_a_line).flatten())
        .or_else(|| (!kinds.is_empty()).then(in_a_column).flatten())
        .or_else(|| {
            scope
                .context_units()
                .filter(|unit| unit.page == first_page)
                .find_map(|unit| labelled(unit, false))
        })
}

/// The number that follows a type on its title line, past a "No." or a
/// "#": "PACKING SLIP PS-311" gives "PS-311", "STATEMENT OF WORK NO. 4"
/// gives nothing (one digit is no identifier).
fn after_type(line: &str, type_words: &[String]) -> Option<String> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let line_words = tokens.iter().map(|token| words(token)).collect::<Vec<_>>();
    let flat = line_words.iter().flatten().cloned().collect::<Vec<_>>();
    let start = phrase_positions(&flat, type_words).into_iter().next()?;
    let end_word = start + type_words.len();
    // The token the type's last word ends in.
    let mut seen = 0;
    let mut at = 0;
    while at < tokens.len() && seen < end_word {
        seen += line_words[at].len();
        at += 1;
    }
    while at < tokens.len()
        && (line_words[at].is_empty()
            || line_words[at]
                .iter()
                .all(|word| NUMBER_WORDS.contains(&word.as_str())))
    {
        at += 1;
    }
    let token = tokens.get(at)?;
    looks_like_identifier(token).then(|| {
        token
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_owned()
    })
}

/// The label the document gives an amount: a "Total:" before it in the
/// fact, the field's label or the row's first cell it stands in ("| Annual
/// Fee | $96,000 |"), or a word the fact itself names it by ("principal").
/// Lower case, at most four words, and none with a digit.
fn amount_label_of(fact: &str, unit: Option<&EvidenceUnit>, money: &str) -> Option<String> {
    let from_fact = label_and_value(fact)
        .filter(|(_, value)| value.contains(money))
        .map(|(label, _)| label.to_owned());
    let from_unit = unit.and_then(|unit| match unit.kind {
        UnitKind::Field => unit.label.clone(),
        UnitKind::TableRow => unit
            .text
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .next()
            .filter(|cell| !cell.is_empty() && !cell.contains(money))
            .map(str::to_owned),
        _ => None,
    });
    // What else a short fact says of the amount: "$9,500 monthly retainer",
    // "$412,000 in bookings".
    let from_words = || {
        let rest = fact.replacen(money, " ", 1);
        let kept = rest
            .split_whitespace()
            .map(|word| word.trim_matches(|character: char| !character.is_alphanumeric()))
            .filter(|word| {
                !word.is_empty()
                    && !matches!(
                        word.to_lowercase().as_str(),
                        "in" | "of" | "for" | "at" | "a" | "an" | "the" | "per" | "is" | "was"
                    )
            })
            .collect::<Vec<_>>();
        (!kept.is_empty() && kept.len() <= 2).then(|| kept.join(" "))
    };
    from_fact
        .or(from_unit)
        .or_else(|| amount_label(fact).map(str::to_owned))
        .or_else(from_words)
        .map(|label| {
            label
                .trim()
                .trim_end_matches([':', '.'])
                .trim()
                .to_lowercase()
        })
        .filter(|label| {
            label
                .chars()
                .filter(|character| character.is_alphabetic())
                .count()
                >= 3
                && label.split_whitespace().count() <= 4
                && !label.chars().any(|character| character.is_ascii_digit())
                && !label.split_whitespace().all(|word| {
                    matches!(
                        word,
                        "year"
                            | "month"
                            | "week"
                            | "day"
                            | "hour"
                            | "annum"
                            | "annually"
                            | "each"
                            | "unit"
                            | "item"
                            | "per"
                    )
                })
        })
}

/// The label an identifier carries: its field's or row's label, or the
/// words just before it on its line ("Invoice No.").
fn identifier_label(
    index: &EvidenceIndex,
    unit: &EvidenceUnit,
    line: Option<&str>,
    identifier: &str,
) -> Option<String> {
    // A table row's label is its first cell; an identifier in a row is
    // labelled by its column's header ("| Invoice No. | ... |").
    if unit.kind == UnitKind::TableRow
        && let Some(header) = unit
            .table_header
            .and_then(|ordinal| index.units().get(ordinal as usize))
    {
        let cells = |text: &str| {
            text.trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        let row = cells(&unit.text);
        let wanted = normalize_loosely(identifier);
        if let Some(column) = row
            .iter()
            .position(|cell| normalize_loosely(cell).contains(&wanted))
            && let Some(label) = cells(&header.text).into_iter().nth(column)
            && !label.is_empty()
        {
            return Some(label);
        }
    }
    if let Some(label) = &unit.label {
        return Some(label.clone());
    }
    let line = line?;
    let at = line.find(identifier)?;
    let before = line[..at]
        .trim_end()
        .trim_end_matches([':', '#', '-'])
        .trim_end();
    let words = before.split_whitespace().collect::<Vec<_>>();
    let start = words.len().saturating_sub(3);
    let label = words[start..].join(" ");
    (!label.is_empty()).then_some(label)
}

/// The date as `line` writes it - "May 12, 2026", "12/05/2026" - found the
/// way a description's restated dates are: the fewest words that state it.
fn surface_form(date: &str, line: &str) -> Option<String> {
    const LONGEST: usize = 5;
    let words = line.split_whitespace().collect::<Vec<_>>();
    let states = |from: usize, to: usize| {
        !date_match_positions(date, &normalize(&words[from..to].join(" "))).is_empty()
    };
    for start in 0..words.len() {
        for end in start + 1..=(start + LONGEST).min(words.len()) {
            if states(start, end) && !states(start + 1, end) && !states(start, end - 1) {
                let surface = words[start..end].join(" ");
                let surface = surface
                    .trim_start_matches(|character: char| !character.is_alphanumeric())
                    .trim_end_matches(|character: char| !character.is_alphanumeric())
                    .to_owned();
                return (!surface.is_empty()).then_some(surface);
            }
        }
    }
    None
}

/// Fills the reply's digest-shaped fields from its facts, so a stored
/// candidate reads like any other: the evidence is the text of the first
/// unit each field cited that the prompt showed, and the relation is the
/// one the replied roles would give, unvalidated and for information only.
fn complete_candidate(
    candidate: &mut ModelProposal,
    facts: &ModelFacts,
    scope: &ValidationScope<'_>,
) {
    let first_line = |ids: &[String], holds: &dyn Fn(&str) -> bool| {
        let unit = scope.cited_units(ids).into_iter().next()?;
        Some(
            line_where(unit, holds)
                .unwrap_or_else(|| unit.text.split_whitespace().collect::<Vec<_>>().join(" ")),
        )
    };
    let date = facts.document_date.clone().unwrap_or_default();
    let type_words = facts
        .document_type
        .as_deref()
        .map(significant_words)
        .unwrap_or_default();
    candidate.evidence = Evidence {
        date: first_line(&facts.date_evidence, &|line| {
            !date_match_positions(&date, line).is_empty()
        })
        .or_else(|| candidate.evidence.date.take()),
        document_type: first_line(&facts.type_evidence, &|line| {
            type_words.iter().any(|word| contains_whole(line, word))
        })
        .or_else(|| candidate.evidence.document_type.take()),
        parties: facts
            .parties
            .iter()
            .filter_map(|party| {
                let name = normalize(&party.name);
                first_line(&party.evidence, &|line| contains_whole(line, &name))
            })
            .collect(),
    };
    let cast = facts
        .parties
        .iter()
        .map(|party| CastMember {
            name: party.name.clone(),
            role: party.role,
            role_supported: true,
        })
        .collect::<Vec<_>>();
    let document_type = facts.document_type.as_deref();
    candidate.party_relation = relation_from_roles(
        DocumentClass::of(document_type),
        document_type,
        &cast,
        RelationCues::default(),
    )
    .relation;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::source_from_text;
    use crate::domain::{KeyFact, PartyFact};
    use crate::retrieve::{RetrievalConfig, Tier, retrieve};

    const YEAR: i32 = 2026;

    fn index_of(text: &str) -> EvidenceIndex {
        EvidenceIndex::build(&source_from_text(text))
    }

    /// The context retrieval gives a short document: all of it.
    fn whole(index: &EvidenceIndex) -> EvidenceContext {
        let context = retrieve(index, &RetrievalConfig::default(), 100);
        assert_eq!(context.tier, Tier::Whole);
        context
    }

    /// A context of exactly the units `keep` picks.
    fn context_of(index: &EvidenceIndex, keep: impl Fn(&EvidenceUnit) -> bool) -> EvidenceContext {
        let units = index
            .units()
            .iter()
            .filter(|unit| !unit.running && keep(unit))
            .map(|unit| unit.ordinal)
            .collect::<Vec<_>>();
        EvidenceContext {
            tier: Tier::Normal,
            hierarchical: false,
            handles: units
                .iter()
                .map(|ordinal| (index.units()[*ordinal as usize].id.clone(), *ordinal))
                .collect(),
            selected_by: units.iter().map(|ordinal| (*ordinal, 1)).collect(),
            text: String::new(),
            estimated_tokens: 0,
            characters: 0,
            units,
        }
    }

    /// The id of the first unit whose text holds `needle`.
    fn id_of(index: &EvidenceIndex, needle: &str) -> String {
        index
            .units()
            .iter()
            .find(|unit| unit.text.contains(needle))
            .unwrap_or_else(|| panic!("no unit holds {needle:?}"))
            .id
            .clone()
    }

    fn reply(facts: ModelFacts) -> ModelProposal {
        ModelProposal {
            document_type: facts.document_type.clone(),
            document_date: facts.document_date.clone(),
            date_role: facts.date_role,
            parties: facts
                .parties
                .iter()
                .map(|party| party.name.clone())
                .collect(),
            confidence: 0.9,
            facts: Some(Box::new(facts)),
            ..ModelProposal::default()
        }
    }

    fn party(name: &str, role: Option<PartyRole>, ids: &[String]) -> PartyFact {
        PartyFact {
            name: name.into(),
            role,
            evidence: ids.to_vec(),
        }
    }

    fn check(
        facts: ModelFacts,
        context: &EvidenceContext,
        index: &EvidenceIndex,
    ) -> ValidationOutcome {
        let scope = ValidationScope::new(index, context, &[]);
        validate_facts_at(reply(facts), &scope, YEAR)
    }

    const INVOICE: &str = "Halvorsen Fixture Works LLC\n12 Quay Street, Brackenridge\n\nINVOICE\n\n\
Invoice No.: INV-10438\n\nInvoice Date: May 1, 2025\n\nDue Date: May 31, 2025\n\n\
Bill To: Quillon Ridge Bakery, Inc.\n\nDisplay shelving for the bakery counter, $1,248.00\n\n\
Payment terms: net 30. Remit to Halvorsen Fixture Works LLC.";

    fn invoice_facts(index: &EvidenceIndex) -> ModelFacts {
        ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            document_date: Some("2025-05-01".into()),
            date_role: Some(DateRole::Invoice),
            date_evidence: vec![id_of(index, "Invoice Date")],
            parties: vec![
                party(
                    "Halvorsen Fixture Works LLC",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "Halvorsen Fixture Works LLC")],
                ),
                party(
                    "Quillon Ridge Bakery, Inc.",
                    Some(PartyRole::Customer),
                    &[id_of(index, "Bill To")],
                ),
            ],
            subject: Some("display shelving for the bakery counter".into()),
            subject_evidence: vec![id_of(index, "Display shelving")],
            identifier: Some("INV-10438".into()),
            identifier_evidence: vec![id_of(index, "INV-10438")],
            key_facts: vec![KeyFact {
                fact: "$1,248.00".into(),
                evidence: vec![id_of(index, "Display shelving")],
            }],
            ..ModelFacts::default()
        }
    }

    #[test]
    fn a_fully_cited_invoice_is_ready_named_from_its_issuer_and_described_with_both() {
        let index = index_of(INVOICE);
        let context = whole(&index);
        let outcome = check(invoice_facts(&index), &context, &index);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        let proposal = &outcome.proposal;
        assert_eq!(proposal.party_relation, PartyRelation::From);
        assert_eq!(proposal.parties, vec!["Halvorsen Fixture Works LLC"]);
        assert_eq!(proposal.document_date.as_deref(), Some("2025-05-01"));
        assert_eq!(proposal.date_role, Some(DateRole::Invoice));
        assert_eq!(
            proposal.description,
            "Invoice from Halvorsen Fixture Works LLC to Quillon Ridge Bakery, Inc. for display \
             shelving for the bakery counter, invoice INV-10438, totalling $1,248.00."
        );
        // What a reviewer is shown is the document's own lines.
        let lines = INVOICE.lines().map(str::trim).collect::<Vec<_>>();
        assert_eq!(
            proposal.evidence.date.as_deref(),
            Some("Invoice Date: May 1, 2025")
        );
        assert_eq!(proposal.evidence.document_type.as_deref(), Some("INVOICE"));
        assert_eq!(proposal.evidence.parties.len(), 1);
        for line in proposal
            .evidence
            .date
            .iter()
            .chain(&proposal.evidence.document_type)
            .chain(&proposal.evidence.parties)
        {
            assert!(
                lines.contains(&line.as_str()),
                "{line:?} is not a line of the document"
            );
        }
        let facts = outcome.facts.as_ref().unwrap();
        assert_eq!(facts.document_class, DocumentClass::Issued);
        assert_eq!(facts.parties.len(), 2, "the customer is kept in the facts");
        assert_eq!(facts.parties[1].role_support, Support::Cited);
        assert_eq!(facts.support.document_type, Support::Cited);
        assert_eq!(facts.support.document_date, Support::Cited);
        assert_eq!(facts.support.parties, vec![Support::Cited, Support::Cited]);
        assert_eq!(facts.support.identifier, Support::Cited);
        assert_eq!(facts.support.key_facts, vec![Support::Cited]);
        assert_eq!(facts.support.unknown_ids, 0);
        assert_eq!(facts.support.miscited_ids, 0);
        for reference in &facts.evidence {
            assert!(index.unit(&reference.id).is_some());
            assert!(
                index
                    .unit(&reference.id)
                    .unwrap()
                    .text
                    .contains(reference.text.as_str())
                    || reference.text.contains('\n')
                    || lines.contains(&reference.text.as_str()),
                "{reference:?}"
            );
        }
        // The candidate keeps the reply's facts and reads like a digest reply.
        assert_eq!(
            outcome.candidate.evidence.date.as_deref(),
            Some("Invoice Date: May 1, 2025")
        );
        assert_eq!(outcome.candidate.party_relation, PartyRelation::From);
    }

    /// An id the prompt never carried is not evidence, whatever unit it
    /// names; a fact only it could support is unsupported.
    #[test]
    fn an_id_the_prompt_never_showed_is_never_evidence() {
        let index = index_of(INVOICE);
        let context = context_of(&index, |unit| {
            !unit.text.contains("Bill To") && !unit.text.contains("Remit")
        });
        let mut facts = invoice_facts(&index);
        facts.date_evidence = vec!["p9.b9".into()];
        let outcome = check(facts, &context, &index);
        let support = &outcome.facts.as_ref().unwrap().support;
        // "p9.b9" names no unit; the bill-to unit was not shown.
        assert_eq!(support.unknown_ids, 2);
        // The date is still in what the model was shown, as the digest
        // pipeline would accept it.
        assert_eq!(support.document_date, Support::Context);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-05-01")
        );
        // The customer is named only in a unit the model never saw.
        assert_eq!(support.parties, vec![Support::Cited, Support::Unsupported]);
        assert!(outcome.reasons.contains(&ReviewReason::PartyUnsupported));
        let bill_to = id_of(&index, "Bill To");
        assert!(
            outcome
                .facts
                .as_ref()
                .unwrap()
                .evidence
                .iter()
                .all(|reference| reference.id != bill_to)
        );
        assert!(!outcome.proposal.description.contains("Quillon"));
    }

    #[test]
    fn a_cited_unit_that_does_not_state_its_fact_is_not_its_evidence() {
        let index = index_of(INVOICE);
        let context = whole(&index);
        let mut facts = invoice_facts(&index);
        facts.date_evidence = vec![id_of(&index, "INVOICE")];
        let outcome = check(facts, &context, &index);
        let support = &outcome.facts.as_ref().unwrap().support;
        assert_eq!(support.document_date, Support::Context);
        assert_eq!(support.miscited_ids, 1);
        assert_eq!(
            outcome.proposal.evidence.date.as_deref(),
            Some("Invoice Date: May 1, 2025")
        );
    }

    /// The model cannot have read a date the excerpts do not carry, so the
    /// document stating it elsewhere is no support.
    #[test]
    fn a_fact_stated_only_outside_the_excerpts_is_not_accepted() {
        let index = index_of(INVOICE);
        let context = context_of(&index, |unit| !unit.text.contains("Invoice Date"));
        let outcome = check(invoice_facts(&index), &context, &index);
        assert_eq!(outcome.proposal.document_date, None);
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));
        assert_eq!(
            outcome.facts.as_ref().unwrap().support.document_date,
            Support::Unsupported
        );
    }

    #[test]
    fn a_deadline_is_replaced_only_by_an_issue_date_both_views_agree_on() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.document_date = Some("2025-05-31".into());
        facts.date_evidence = vec![id_of(&index, "Due Date")];
        // Everything shown: the invoice date replaces the due date.
        let outcome = check(facts.clone(), &whole(&index), &index);
        assert!(outcome.reasons.contains(&ReviewReason::DateIsDeadline));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-05-01")
        );
        // The invoice date not shown: the document has one, the excerpts do
        // not, so nothing replaces the deadline.
        let context = context_of(&index, |unit| !unit.text.contains("Invoice Date"));
        let outcome = check(facts, &context, &index);
        assert!(outcome.reasons.contains(&ReviewReason::DateIsDeadline));
        assert_eq!(outcome.proposal.document_date, None);
        assert_eq!(
            outcome
                .facts
                .as_ref()
                .unwrap()
                .support
                .guard_scope
                .as_deref(),
            Some("both")
        );
    }

    const SOW_UNDER_MSA: &str = "STATEMENT OF WORK NO. 7\n\n\
Issued under the Master Services Agreement dated June 2, 2023 between Thornbury Data Labs LLC and \
Saltmarsh Regional Water Authority.\n\n\
This Statement of Work is effective as of April 1, 2026.\n\n\
The work covers the replacement of meter telemetry across the eastern district.";

    fn sow_facts(index: &EvidenceIndex, date: &str, cited: &str) -> ModelFacts {
        let parties_line = id_of(index, "Issued under");
        ModelFacts {
            document_type: Some("Statement of Work".into()),
            type_evidence: vec![id_of(index, "STATEMENT OF WORK")],
            document_date: Some(date.into()),
            date_role: Some(DateRole::Effective),
            date_evidence: vec![id_of(index, cited)],
            parties: vec![
                party(
                    "Thornbury Data Labs LLC",
                    Some(PartyRole::Contractor),
                    &[parties_line.clone()],
                ),
                party(
                    "Saltmarsh Regional Water Authority",
                    Some(PartyRole::Client),
                    &[parties_line],
                ),
            ],
            ..ModelFacts::default()
        }
    }

    /// The referenced agreement's date is never this document's. Replaced
    /// by the effective date only when the excerpts carry it as well as the
    /// document; otherwise withheld.
    #[test]
    fn another_agreements_date_is_replaced_only_from_the_excerpts() {
        let index = index_of(SOW_UNDER_MSA);
        let facts = sow_facts(&index, "2023-06-02", "Issued under");
        let outcome = check(facts.clone(), &whole(&index), &index);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01")
        );
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Effective));
        assert!(!outcome.reasons.contains(&ReviewReason::DateUnsupported));
        assert_eq!(
            outcome.proposal.evidence.date.as_deref(),
            Some("This Statement of Work is effective as of April 1, 2026.")
        );
        assert_eq!(
            outcome
                .facts
                .as_ref()
                .unwrap()
                .support
                .guard_scope
                .as_deref(),
            Some("both")
        );

        let excerpt = context_of(&index, |unit| !unit.text.contains("effective as of"));
        let outcome = check(facts, &excerpt, &index);
        assert_eq!(outcome.proposal.document_date, None);
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));

        // The right date, cited, is simply accepted.
        let facts = sow_facts(&index, "2026-04-01", "effective as of");
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        assert_eq!(
            outcome.facts.as_ref().unwrap().support.document_date,
            Support::Cited
        );
    }

    const CREDIT_AGREEMENT: &str = "CREDIT AGREEMENT\n\n\
dated as of March 3, 2025\n\n\
among MARROWFIELD PACKAGING HOLDINGS, INC., as Borrower, and HALDEN BAY NATIONAL BANK, N.A., as Lender\n\n\
This Agreement is effective as of March 3, 2025.\n\n\
This Agreement amends and restates the Existing Credit Agreement dated as of June 1, 2021 between the same parties.\n\n\
\"Maturity Date\" means March 3, 2030.";

    #[test]
    fn a_credit_agreements_restated_predecessor_never_dates_it() {
        let index = index_of(CREDIT_AGREEMENT);
        let preamble = id_of(&index, "among MARROWFIELD");
        let facts = ModelFacts {
            document_type: Some("Credit Agreement".into()),
            type_evidence: vec![id_of(&index, "CREDIT AGREEMENT")],
            document_date: Some("2021-06-01".into()),
            date_role: Some(DateRole::Effective),
            date_evidence: vec![id_of(&index, "Existing Credit Agreement")],
            parties: vec![
                party(
                    "MARROWFIELD PACKAGING HOLDINGS, INC.",
                    Some(PartyRole::Borrower),
                    &[preamble.clone()],
                ),
                party(
                    "HALDEN BAY NATIONAL BANK, N.A.",
                    Some(PartyRole::Lender),
                    &[preamble],
                ),
            ],
            ..ModelFacts::default()
        };
        let outcome = check(facts.clone(), &whole(&index), &index);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-03-03")
        );
        let roles = &outcome.facts.as_ref().unwrap().parties;
        assert_eq!(roles[0].role_support, Support::Cited, "{roles:?}");
        assert_eq!(roles[1].role_support, Support::Cited, "{roles:?}");

        // Excerpts without the agreement's own effective line: the
        // predecessor's date is withheld, not swapped for one the model
        // was never shown.
        let excerpt = context_of(&index, |unit| !unit.text.contains("effective as of"));
        let outcome = check(facts, &excerpt, &index);
        assert_eq!(outcome.proposal.document_date, None);
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));
    }

    /// Numeric dates read either way round are settled by the document's
    /// other numeric dates - all of them, not only the excerpts'.
    #[test]
    fn a_day_and_month_the_whole_document_leaves_open_are_ambiguous() {
        let text = "PACKING SLIP\n\nShipped by Corriveau Millwork Supply Co.\n\n\
Slip Date: 04/05/2026\n\nShip Date: 04/13/2026\n\nNotes\n\nReceived at dock 13/04/2026.";
        let index = index_of(text);
        let facts = ModelFacts {
            document_type: Some("Packing Slip".into()),
            type_evidence: vec![id_of(&index, "PACKING SLIP")],
            document_date: Some("2026-04-05".into()),
            date_evidence: vec![id_of(&index, "Slip Date")],
            ..ModelFacts::default()
        };
        // The excerpts alone settle month-first.
        let excerpt = context_of(&index, |unit| !unit.text.contains("Received"));
        let scope = ValidationScope::new(&index, &excerpt, &[]);
        assert!(!reading_is_unsettled(scope.context(), "2026-04-05"));
        let outcome = check(facts, &excerpt, &index);
        assert!(
            outcome.reasons.contains(&ReviewReason::DateAmbiguous),
            "{:?}",
            outcome.reasons
        );
    }

    #[test]
    fn a_role_the_document_does_not_support_never_decides_the_relation() {
        let text = "NOTICE OF RENT INCREASE\n\nTo: Imogen Castellanos, Tenant\n\n\
From: Cresthaven Court Holdings LLC\n\nYour rent for Apartment 4C rises to $1,965.00 on August 1, 2026.\n\n\
Dated May 12, 2026";
        let index = index_of(text);
        let context = whole(&index);
        let to = id_of(&index, "To: Imogen");
        let from = id_of(&index, "From: Cresthaven");
        let with_roles = |tenant: PartyRole, landlord: PartyRole| ModelFacts {
            document_type: Some("Notice of Rent Increase".into()),
            type_evidence: vec![id_of(&index, "NOTICE OF RENT INCREASE")],
            document_date: Some("2026-05-12".into()),
            date_evidence: vec![id_of(&index, "Dated May")],
            parties: vec![
                party("Imogen Castellanos", Some(tenant), &[to.clone()]),
                party(
                    "Cresthaven Court Holdings LLC",
                    Some(landlord),
                    &[from.clone()],
                ),
            ],
            subject: Some("rent for Apartment 4C".into()),
            subject_evidence: vec![id_of(&index, "Your rent")],
            ..ModelFacts::default()
        };
        let outcome = check(
            with_roles(PartyRole::Tenant, PartyRole::Sender),
            &context,
            &index,
        );
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        assert_eq!(outcome.proposal.parties, vec!["Imogen Castellanos"]);
        assert_eq!(
            outcome.proposal.description,
            "Notice of Rent Increase from Cresthaven Court Holdings LLC to Imogen Castellanos \
             regarding rent for Apartment 4C, rent of $1,965.00."
        );
        // Swapped roles: the document states neither, so neither decides
        // anything - the document's own words do.
        let outcome = check(
            with_roles(PartyRole::Landlord, PartyRole::Tenant),
            &context,
            &index,
        );
        let parties = &outcome.facts.as_ref().unwrap().parties;
        assert_eq!(parties[0].role_support, Support::Unsupported);
        assert_eq!(parties[1].role_support, Support::Unsupported);
        assert_eq!(parties[0].document_role, Some(PartyRole::Tenant));
        assert_eq!(parties[1].document_role, Some(PartyRole::Issuer));
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        assert_eq!(outcome.proposal.parties, vec!["Imogen Castellanos"]);

        // A document that gives no role in words leaves an unsupported
        // claim nothing to stand on: the names are kept without a joining
        // word.
        let plain = "NOTICE OF RENT INCREASE\n\nImogen Castellanos and Cresthaven Court Holdings LLC \
                     agree that the rent for Apartment 4C rises on August 1, 2026, and that every other \
                     term of the residential lease stays as it is.\n\nDated May 12, 2026";
        let index = index_of(plain);
        let line = id_of(&index, "Imogen Castellanos and");
        let facts = ModelFacts {
            document_type: Some("Notice of Rent Increase".into()),
            type_evidence: vec![id_of(&index, "NOTICE OF RENT INCREASE")],
            document_date: Some("2026-05-12".into()),
            date_evidence: vec![id_of(&index, "Dated May")],
            parties: vec![
                party(
                    "Imogen Castellanos",
                    Some(PartyRole::Landlord),
                    &[line.clone()],
                ),
                party(
                    "Cresthaven Court Holdings LLC",
                    Some(PartyRole::Tenant),
                    &[line],
                ),
            ],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        let parties = &outcome.facts.as_ref().unwrap().parties;
        assert_eq!(parties[0].document_role, None);
        assert_eq!(outcome.proposal.party_relation, PartyRelation::None);
    }

    /// An invoice that names only its customer names no one: the bill-to
    /// company is never the party an invoice is filed under.
    #[test]
    fn an_invoice_naming_only_its_customer_files_under_its_header_never_its_customer() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.parties.remove(0);
        let outcome = check(facts, &whole(&index), &index);
        // The customer is never the filename's party; the organisation at
        // the head of the page, before the "Bill To", issued it.
        assert_eq!(
            outcome.proposal.parties,
            vec!["Halvorsen Fixture Works LLC"],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
        assert!(!outcome.proposal.evidence.parties.is_empty());
        assert!(
            outcome.proposal.description.starts_with(
                "Invoice from Halvorsen Fixture Works LLC to Quillon Ridge Bakery, Inc."
            ),
            "{}",
            outcome.proposal.description
        );
        // With no header organisation before the customer, no one.
        let text = "INVOICE\n\nBill To: Quillon Ridge Bakery, Inc.\n\nTotal: $10.00";
        let index = index_of(text);
        let outcome = check(
            ModelFacts {
                document_type: Some("Invoice".into()),
                type_evidence: vec![id_of(&index, "INVOICE")],
                parties: vec![party(
                    "Quillon Ridge Bakery, Inc.",
                    Some(PartyRole::Customer),
                    &[id_of(&index, "Bill To")],
                )],
                ..ModelFacts::default()
            },
            &whole(&index),
            &index,
        );
        assert!(
            outcome.proposal.parties.is_empty(),
            "{:?}",
            outcome.proposal
        );
    }

    /// An identifier in a table is labelled by its column.
    #[test]
    fn an_identifier_in_a_table_reads_with_its_columns_header() {
        let text = "Halvorsen Fixture Works LLC\n\nINVOICE\n\n| Invoice No. | Invoice Date | Terms |\n\
| --- | --- | --- |\n| INV-20417 | March 4, 2026 | Net 30 |\n\nBill To: Quillon Ridge Bakery, Inc.";
        let index = index_of(text);
        let row = id_of(&index, "| INV-20417");
        let facts = ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(&index, "INVOICE")],
            document_date: Some("2026-03-04".into()),
            date_evidence: vec![row.clone()],
            parties: vec![party(
                "Halvorsen Fixture Works LLC",
                Some(PartyRole::Issuer),
                &[id_of(&index, "Halvorsen")],
            )],
            identifier: Some("INV-20417".into()),
            identifier_evidence: vec![row],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert!(
            outcome.proposal.description.contains(", invoice INV-20417"),
            "{}",
            outcome.proposal.description
        );
    }

    /// A subject's names and numbers are claims, checked as a written
    /// description's are: one the context does not contain sends the
    /// document to review, and the subject is never written into the
    /// description. A word that is no claim must still be stated, and keeps
    /// the subject out without a review when it is not.
    #[test]
    fn an_invented_subject_is_flagged_and_left_out() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.subject = Some("renovation of the Zanzibar wing".into());
        let outcome = check(facts, &whole(&index), &index);
        assert!(
            outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported)
        );
        assert!(!outcome.proposal.description.contains("Zanzibar"));
        assert_eq!(
            outcome.facts.as_ref().unwrap().support.subject,
            Support::Unsupported
        );
        assert_eq!(outcome.facts.as_ref().unwrap().subject, None);
    }

    /// A subject cited to the wrong line is written into the description
    /// when one unit the model was shown holds all its words. One that no
    /// unit holds whole is supported - its claims are - and not written
    /// down. The second subject here is kept out twice over: its words are
    /// scattered over several units, and one of them ("remittance"; the
    /// document writes "Remit") is stated nowhere. Words that are all
    /// stated but scattered are tested on their own in
    /// `a_subject_of_stated_words_still_needs_its_cited_share_each_word_counted_once`.
    #[test]
    fn a_subject_no_one_unit_holds_stays_out_of_the_description() {
        // Cited wrongly, but one unit holds all its words: grounded there.
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.subject_evidence = vec![id_of(&index, "Invoice No.")];
        let outcome = check(facts, &whole(&index), &index);
        assert!(
            outcome.proposal.description.contains("shelving"),
            "{}",
            outcome.proposal.description
        );
        // No unit holds its words whole, and one is stated nowhere: it
        // stays out, without a review.
        let mut facts = invoice_facts(&index);
        facts.subject = Some("bakery fixture shelving remittance".into());
        facts.subject_evidence = vec![id_of(&index, "Invoice No.")];
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        assert!(
            !outcome.proposal.description.contains("shelving"),
            "{}",
            outcome.proposal.description
        );
    }

    /// The description `invoice_facts` gives with no subject.
    const INVOICE_WITHOUT_SUBJECT: &str = "Invoice from Halvorsen Fixture Works LLC to Quillon \
                                           Ridge Bakery, Inc., invoice INV-10438, totalling \
                                           $1,248.00.";

    /// `invoice_facts` with another subject, cited to the first unit
    /// holding each needle and checked against all of [`INVOICE`]: the
    /// outcome, which must be ready, and the subject kept.
    fn with_subject(subject: &str, cited: &[&str]) -> (ValidationOutcome, Option<String>) {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.subject = Some(subject.into());
        facts.subject_evidence = cited.iter().map(|needle| id_of(&index, needle)).collect();
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{subject}: {:?}",
            outcome.reasons
        );
        let kept = outcome
            .facts
            .as_ref()
            .and_then(|facts| facts.subject.clone());
        (outcome, kept)
    }

    /// Every word of a subject but glue words and single letters must be
    /// stated somewhere in what the model was shown. A word in lower case
    /// is no claim, so the share of cited words was all that held one
    /// back: 60% let one invented word in five through. It stays out now,
    /// without a review - the subject is optional and never in the
    /// filename.
    #[test]
    fn a_subject_with_a_word_the_context_never_states_stays_out_of_the_description() {
        for (invented, cited) in [
            // Four of five significant words cited.
            (
                "refrigerated display shelving for the bakery counter",
                &["Display shelving"][..],
            ),
            // Three of five: exactly 60%.
            (
                "refrigerated walnut shelving for the bakery counter",
                &["Display shelving"],
            ),
            // Three of five, across the two units it cites.
            (
                "Brackenridge refrigerated walnut bakery counter",
                &["Quay Street", "Display shelving"],
            ),
        ] {
            let (outcome, kept) = with_subject(invented, cited);
            assert_eq!(kept, None, "{invented}");
            assert_eq!(outcome.proposal.description, INVOICE_WITHOUT_SUBJECT);
        }
        // A label's own word padded the share before the label was taken
        // off what was written.
        let text = "Halvorsen Fixture Works LLC\n\nINVOICE\n\nInvoice No.: INV-10438\n\n\
Invoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\n\
Description: display shelving for the bakery counter\n\nTotal due: $1,248.00";
        let (outcome, _) = facts_for(text, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            document_date: Some("2025-05-01".into()),
            date_evidence: vec![id_of(index, "Invoice Date")],
            subject: Some("Description: cryogenic walnut display shelving".into()),
            subject_evidence: vec![id_of(index, "Description:")],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.facts.as_ref().unwrap().subject, None);
        assert!(
            !outcome.proposal.description.contains("cryogenic"),
            "{}",
            outcome.proposal.description
        );
        // Every word stated: written, as before.
        let (_, kept) = with_subject(
            "display shelving for the bakery counter",
            &["Display shelving"],
        );
        assert_eq!(
            kept.as_deref(),
            Some("display shelving for the bakery counter")
        );
    }

    /// A subject's words are split at every mark that is not a letter or a
    /// digit and checked one by one: a capital glued after a slash, or
    /// inside a word, is no claim a sentence's check sees.
    #[test]
    fn a_word_glued_to_another_in_a_subject_is_checked_on_its_own() {
        for glued in [
            "display shelving/Cryogenic for the bakery counter",
            "eFrost display shelving for the bakery counter",
        ] {
            let (outcome, kept) = with_subject(glued, &["Display shelving"]);
            assert_eq!(kept, None, "{glued}");
            assert_eq!(outcome.proposal.description, INVOICE_WITHOUT_SUBJECT);
        }
        // Each part stated, even in different units: written.
        const LAB_INVOICE: &str = "Ferncastle Veterinary Laboratory LLC\n\nINVOICE\n\n\
Invoice Date: May 1, 2025\n\nComplete blood count, canine, $48.00\n\n\
Feline panel, $52.00\n\nTotal: $100.00";
        let (outcome, _) = facts_for(LAB_INVOICE, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            document_date: Some("2025-05-01".into()),
            date_evidence: vec![id_of(index, "Invoice Date")],
            subject: Some("Complete blood count, canine/feline".into()),
            subject_evidence: vec![id_of(index, "Complete blood")],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.facts.as_ref().unwrap().subject.as_deref(),
            Some("Complete blood count, canine/feline")
        );
        assert!(
            outcome.proposal.description.contains("canine/feline"),
            "{}",
            outcome.proposal.description
        );
    }

    /// A number in a subject must be stated whole, by a unit that does not
    /// carry it on either side: "48" and "248" are not stated by
    /// "$1,248.00", and a two-digit number is too short for a sentence's
    /// claims check to see. "10438" is stated by "INV-10438".
    #[test]
    fn a_subject_with_a_number_the_context_never_states_stays_out_of_the_description() {
        for invented in [
            "48 display shelving for the bakery counter",
            "248 display shelving for the bakery counter",
            "2 display shelving for the bakery counter",
        ] {
            let (outcome, kept) = with_subject(invented, &["Display shelving"]);
            assert_eq!(kept, None, "{invented}");
            assert_eq!(outcome.proposal.description, INVOICE_WITHOUT_SUBJECT);
        }
        let stated = "display shelving for the bakery counter on invoice 10438";
        let (_, kept) = with_subject(stated, &["Display shelving"]);
        assert_eq!(kept.as_deref(), Some(stated));
    }

    /// A negation is an ordinary word: kept when the context states it,
    /// in any unit, and keeping a subject out when it does not.
    #[test]
    fn an_unstated_negation_keeps_a_subject_out() {
        const ORDERED: &str = "Halvorsen Fixture Works LLC\n\nINVOICE\n\n\
Invoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\nPO No 4471\n\n\
Display shelving for the bakery counter, $1,248.00";
        let reply = |index: &EvidenceIndex, subject: &str| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            document_date: Some("2025-05-01".into()),
            date_evidence: vec![id_of(index, "Invoice Date")],
            subject: Some(subject.into()),
            subject_evidence: vec![id_of(index, "PO No"), id_of(index, "Display shelving")],
            ..ModelFacts::default()
        };
        // The document never says "not".
        let (outcome, _) = facts_for(ORDERED, |index| {
            reply(index, "display shelving not for the bakery counter")
        });
        assert_eq!(outcome.facts.as_ref().unwrap().subject, None);
        assert!(
            !outcome.proposal.description.contains(" not "),
            "{}",
            outcome.proposal.description
        );
        // It says "No", in a unit of its own: written.
        let stated = "display shelving for the bakery counter, PO No 4471";
        let (outcome, _) = facts_for(ORDERED, |index| reply(index, stated));
        assert_eq!(
            outcome.facts.as_ref().unwrap().subject.as_deref(),
            Some(stated)
        );
        // A contracted negation is checked whole, not as "can" and a
        // single letter: the lease says "can", never "can't".
        for (subject, written) in [
            ("tenant can't keep one cat in the apartment", false),
            ("tenant can\u{2019}t keep one cat in the apartment", false),
            ("tenant can keep one cat in the apartment", true),
        ] {
            let outcome = lease_outcome(Some(subject), &[]);
            let kept = outcome.facts.as_ref().unwrap().subject.as_deref();
            assert_eq!(kept, written.then_some(subject), "{subject}");
        }
        let outcome = lease_outcome(None, &["Tenant can't keep one cat in the apartment"]);
        assert!(outcome.facts.as_ref().unwrap().key_facts.is_empty());
        assert!(
            !outcome.proposal.description.contains("can't"),
            "{}",
            outcome.proposal.description
        );
    }

    /// A lease that lets its tenant keep a cat and charges a late fee in
    /// dollars. It never writes "can't" or a percent sign.
    const PET_LEASE: &str = "Corvane Holdings LLC\n\nRESIDENTIAL LEASE\n\n\
Lease Date: June 1, 2025\n\nTenant: Delphine Okonkwo-Reyes\n\n\
Tenant can keep one cat in the apartment.\n\nLate fee: $25 after the fifth day.";

    /// A reply about [`PET_LEASE`] with this subject and these key facts,
    /// each cited to the cat's and the late fee's units.
    fn lease_outcome(subject: Option<&str>, key_facts: &[&str]) -> ValidationOutcome {
        facts_for(PET_LEASE, |index| {
            let cited = vec![id_of(index, "Tenant can"), id_of(index, "Late fee")];
            ModelFacts {
                document_type: Some("Residential Lease".into()),
                type_evidence: vec![id_of(index, "RESIDENTIAL")],
                subject: subject.map(str::to_owned),
                subject_evidence: cited.clone(),
                key_facts: key_facts
                    .iter()
                    .map(|fact| KeyFact {
                        fact: (*fact).into(),
                        evidence: cited.clone(),
                    })
                    .collect(),
                ..ModelFacts::default()
            }
        })
        .0
    }

    /// A word the document states only outside the units the model was
    /// shown keeps a subject out: the model could not have read it there.
    #[test]
    fn a_subject_with_a_word_stated_only_outside_the_context_stays_out() {
        const FINISHED: &str = "Halvorsen Fixture Works LLC\n\nINVOICE\n\n\
Invoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\n\
Display shelving for the bakery counter, $1,248.00\n\nWalnut finish, refrigerated base.";
        let index = index_of(FINISHED);
        let facts = ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(&index, "INVOICE")],
            document_date: Some("2025-05-01".into()),
            date_evidence: vec![id_of(&index, "Invoice Date")],
            subject: Some("refrigerated display shelving for the bakery counter".into()),
            subject_evidence: vec![id_of(&index, "Display shelving")],
            ..ModelFacts::default()
        };
        // The whole document shown: written.
        let outcome = check(facts.clone(), &whole(&index), &index);
        assert_eq!(
            outcome.facts.as_ref().unwrap().subject.as_deref(),
            Some("refrigerated display shelving for the bakery counter")
        );
        // The unit that says "refrigerated" not shown: left out.
        let shown = context_of(&index, |unit| !unit.text.contains("Walnut"));
        let outcome = check(facts, &shown, &index);
        assert_eq!(outcome.facts.as_ref().unwrap().subject, None);
        assert!(
            !outcome.proposal.description.contains("refrigerated"),
            "{}",
            outcome.proposal.description
        );
    }

    /// Only the glue words and single letters need not be stated. Other
    /// short common words - "all", "other", "under" - say something, and
    /// keep a subject out when the context never writes them.
    #[test]
    fn a_common_word_that_is_no_glue_word_must_be_stated() {
        for invented in [
            "all display shelving for the bakery counter",
            "other display shelving for the bakery counter",
            "display shelving under the bakery counter",
        ] {
            let (outcome, kept) = with_subject(invented, &["Display shelving"]);
            assert_eq!(kept, None, "{invented}");
            assert_eq!(outcome.proposal.description, INVOICE_WITHOUT_SUBJECT);
        }
    }

    /// A currency symbol the context never writes keeps a subject and a
    /// key fact out, wherever it stands: "1,248.00 €" in a document of
    /// dollars.
    #[test]
    fn a_currency_symbol_the_context_never_writes_keeps_a_subject_and_a_key_fact_out() {
        let (outcome, kept) = with_subject(
            "display shelving for the bakery counter, 1,248.00 \u{20ac}",
            &["Display shelving"],
        );
        assert_eq!(kept, None);
        assert_eq!(outcome.proposal.description, INVOICE_WITHOUT_SUBJECT);
        let outcome = with_key_facts(INVOICE, &[("1,248.00 \u{20ac}", Some("Display shelving"))]);
        assert!(outcome.facts.as_ref().unwrap().key_facts.is_empty());
        assert!(
            !outcome.proposal.description.contains('\u{20ac}'),
            "{}",
            outcome.proposal.description
        );
        // The symbol the document writes: kept.
        let outcome = with_key_facts(INVOICE, &[("$1,248.00", Some("Display shelving"))]);
        assert_eq!(outcome.facts.as_ref().unwrap().key_facts, vec!["$1,248.00"]);
    }

    /// An invisible character neither splits a word nor hides one: the
    /// word is checked whole, and written without it.
    #[test]
    fn an_invisible_character_in_a_subject_is_taken_out_before_it_is_checked() {
        // "net" and "work" are each stated; "network" is not.
        let (outcome, kept) = with_subject(
            "net\u{200b}work display shelving for the bakery counter",
            &["Display shelving"],
        );
        assert_eq!(kept, None);
        assert_eq!(outcome.proposal.description, INVOICE_WITHOUT_SUBJECT);
        let (outcome, kept) = with_subject(
            "dis\u{200b}play shelving for the bakery counter",
            &["Display shelving"],
        );
        assert_eq!(
            kept.as_deref(),
            Some("display shelving for the bakery counter")
        );
        assert!(
            outcome
                .proposal
                .description
                .contains(" for display shelving for the bakery counter,"),
            "{}",
            outcome.proposal.description
        );
    }

    /// A zero-width non-joiner is part of how Persian spells a word: it is
    /// ignored when the subject is checked, and kept in what is written.
    #[test]
    fn a_joiner_a_script_spells_with_stays_in_the_subject() {
        const CARPETS: &str = "Kavir Textile Trading LLC\n\nINVOICE\n\n\
Invoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\n\
فرش\u{200c}های دستباف, $1,248.00";
        let subject = "فرش\u{200c}های دستباف";
        let (outcome, _) = facts_for(CARPETS, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            document_date: Some("2025-05-01".into()),
            date_evidence: vec![id_of(index, "Invoice Date")],
            subject: Some(subject.into()),
            subject_evidence: vec![id_of(index, "$1,248.00")],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.facts.as_ref().unwrap().subject.as_deref(),
            Some(subject)
        );
        assert!(
            outcome.proposal.description.contains(subject),
            "{:?}",
            outcome.proposal.description
        );
    }

    /// A percent sign is checked as a currency symbol is: a lease that
    /// charges "$25" never states "25%".
    #[test]
    fn a_percent_sign_the_context_never_writes_keeps_a_subject_and_a_key_fact_out() {
        let outcome = lease_outcome(Some("one cat, late fee 25%"), &["Late fee: 25%"]);
        let facts = outcome.facts.as_ref().unwrap();
        assert_eq!(facts.subject, None);
        assert!(facts.key_facts.is_empty(), "{:?}", facts.key_facts);
        assert!(
            !outcome.proposal.description.contains('%'),
            "{}",
            outcome.proposal.description
        );
        // Without the sign, or with the lease's own "$": kept.
        let outcome = lease_outcome(Some("one cat, late fee 25"), &["Late fee: $25"]);
        let facts = outcome.facts.as_ref().unwrap();
        assert_eq!(facts.subject.as_deref(), Some("one cat, late fee 25"));
        assert_eq!(facts.key_facts, vec!["Late fee: $25"]);
    }

    /// A word of letters and digits is a word, checked whole: "Q3" is not
    /// stated by a document that writes "Q4" and "3".
    #[test]
    fn a_word_of_letters_and_digits_is_checked_as_a_word() {
        const SERVICED: &str = "Halvorsen Fixture Works LLC\n\nINVOICE\n\n\
Invoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\n\
Q4 shelving service, 3 visits, $360.00";
        for (subject, written) in [
            ("Q3 shelving service visits", false),
            ("Q4 shelving service visits", true),
        ] {
            let (outcome, _) = facts_for(SERVICED, |index| ModelFacts {
                document_type: Some("Invoice".into()),
                type_evidence: vec![id_of(index, "INVOICE")],
                document_date: Some("2025-05-01".into()),
                date_evidence: vec![id_of(index, "Invoice Date")],
                subject: Some(subject.into()),
                subject_evidence: vec![id_of(index, "Q4")],
                ..ModelFacts::default()
            });
            let kept = outcome.facts.as_ref().unwrap().subject.as_deref();
            assert_eq!(kept.is_some(), written, "{subject}: {kept:?}");
            assert_eq!(
                outcome.proposal.description.contains(subject),
                written,
                "{}",
                outcome.proposal.description
            );
        }
    }

    /// A word the document hyphenates across a line break, as justified
    /// PDF text and OCR do, is stated whole: "display" for "dis-\nplay".
    #[test]
    fn a_word_hyphenated_across_a_line_break_is_stated_whole() {
        const QUOTED: &str = "Halvorsen Fixture Works LLC\n\nINVOICE\n\n\
Invoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\n\
Supply and installation of refrigerated dis-\nplay shelving for the bakery counter, as quoted.";
        for subject in [
            "refrigerated display shelving for the bakery counter",
            "refrigerated display shelving",
        ] {
            let (outcome, _) = facts_for(QUOTED, |index| ModelFacts {
                document_type: Some("Invoice".into()),
                type_evidence: vec![id_of(index, "INVOICE")],
                document_date: Some("2025-05-01".into()),
                date_evidence: vec![id_of(index, "Invoice Date")],
                subject: Some(subject.into()),
                subject_evidence: vec![id_of(index, "Supply and")],
                ..ModelFacts::default()
            });
            assert_eq!(
                outcome.facts.as_ref().unwrap().subject.as_deref(),
                Some(subject)
            );
        }
    }

    /// An invisible character the document holds inside a word is ignored
    /// wherever a subject is checked: a subject that leaves it out or
    /// copies it is written, without it, and the proposal stays ready.
    #[test]
    fn an_invisible_character_in_the_document_neither_splits_nor_hides_a_word() {
        const MARKED: &str = "Halvorsen Fixture Works LLC\n12 Quay Street, Brack\u{ad}enridge\n\n\
INVOICE\n\nInvoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\n\
Dis\u{ad}play shelving for the bak\u{ad}ery counter, $1,248.00";
        for invisible in ["\u{ad}", "\u{200b}"] {
            let text = MARKED.replace('\u{ad}', invisible);
            for (subject, written) in [
                (
                    "display shelving for the bakery counter".to_owned(),
                    "display shelving for the bakery counter",
                ),
                (format!("dis{invisible}play shelving"), "display shelving"),
                (
                    format!("Brack{invisible}enridge display shelving"),
                    "Brackenridge display shelving",
                ),
            ] {
                let (outcome, _) = facts_for(&text, |index| ModelFacts {
                    document_type: Some("Invoice".into()),
                    type_evidence: vec![id_of(index, "INVOICE")],
                    document_date: Some("2025-05-01".into()),
                    date_role: Some(DateRole::Invoice),
                    date_evidence: vec![id_of(index, "Invoice Date")],
                    parties: vec![
                        party(
                            "Halvorsen Fixture Works LLC",
                            Some(PartyRole::Issuer),
                            &[id_of(index, "Halvorsen")],
                        ),
                        party(
                            "Quillon Ridge Bakery, Inc.",
                            Some(PartyRole::Customer),
                            &[id_of(index, "Bill To")],
                        ),
                    ],
                    subject: Some(subject.clone()),
                    subject_evidence: vec![id_of(index, "shelving"), id_of(index, "Quay")],
                    ..ModelFacts::default()
                });
                assert_eq!(
                    outcome.status,
                    ProposalStatus::Ready,
                    "{subject:?}: {:?}",
                    outcome.reasons
                );
                assert_eq!(
                    outcome.facts.as_ref().unwrap().subject.as_deref(),
                    Some(written),
                    "{subject:?}"
                );
                assert!(
                    outcome.proposal.description.contains(written),
                    "{}",
                    outcome.proposal.description
                );
            }
        }
    }

    /// A subject whose every word is stated still needs the share of its
    /// cited units, or one unit holding it whole. Each distinct word counts
    /// once: saying a stated word again does not make up for words the
    /// cited units do not hold.
    #[test]
    fn a_subject_of_stated_words_still_needs_its_cited_share_each_word_counted_once() {
        // quay, street, counter, shelving: two of four in the cited unit.
        // Counted with its repeats, four of six passed.
        let (outcome, kept) = with_subject(
            "quay street counter shelving, counter to counter",
            &["Display shelving"],
        );
        assert_eq!(kept, None);
        assert_eq!(outcome.proposal.description, INVOICE_WITHOUT_SUBJECT);
        // Every word stated, but cited wrongly and held by no one unit.
        let (_, kept) = with_subject("bakery fixture shelving", &["Invoice No."]);
        assert_eq!(kept, None);
    }

    /// A reply that names an invoice and these key facts, each cited to
    /// the first unit holding its needle, about all of `text`.
    fn with_key_facts(text: &str, key_facts: &[(&str, Option<&str>)]) -> ValidationOutcome {
        facts_for(text, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            key_facts: key_facts
                .iter()
                .map(|(fact, cited)| KeyFact {
                    fact: (*fact).into(),
                    evidence: cited
                        .map(|needle| vec![id_of(index, needle)])
                        .unwrap_or_default(),
                })
                .collect(),
            ..ModelFacts::default()
        })
        .0
    }

    /// A key fact - only a reply in named fields gives them - is held to
    /// the subject's word check. One that fails it is left out, so it
    /// neither stands in for the subject nor labels an amount.
    #[test]
    fn a_key_fact_with_a_word_the_context_never_states_is_left_out() {
        let shelving = Some("Display shelving");
        for (facts, invented) in [
            // In the subject's place, with one stated word in four.
            (
                vec![
                    ("refrigerated cryogenic walnut shelving", shelving),
                    ("$1,248.00", shelving),
                ],
                "refrigerated",
            ),
            // Words too short for a claim.
            (vec![("6 ft", shelving), ("$1,248.00", shelving)], "6 ft"),
            // Its only stated word a number.
            (
                vec![
                    ("2025 cryogenic walnut refrigeration", Some("Invoice Date")),
                    ("$1,248.00", shelving),
                ],
                "walnut",
            ),
            // As an amount's label: before a colon, beside the amount.
            (
                vec![("cryogenic shelving: $1,248.00", shelving)],
                "cryogenic",
            ),
            (
                vec![("$1,248.00 cryogenic shelving", shelving)],
                "cryogenic",
            ),
            (vec![("cryogenic walnut: $1,248.00", shelving)], "walnut"),
        ] {
            let outcome = with_key_facts(INVOICE, &facts);
            // Left out on its words alone: no review.
            assert!(
                !outcome
                    .reasons
                    .contains(&ReviewReason::DescriptionUnsupported),
                "{facts:?}: {:?}",
                outcome.reasons
            );
            let description = &outcome.proposal.description;
            assert!(!description.contains(invented), "{description}");
            let kept = &outcome.facts.as_ref().unwrap().key_facts;
            assert!(kept.iter().all(|fact| !fact.contains(invented)), "{kept:?}");
        }
        // Every word stated: it labels the amount.
        let outcome = with_key_facts(INVOICE, &[("$1,248.00 display shelving", shelving)]);
        assert!(
            outcome
                .proposal
                .description
                .contains("display shelving of $1,248.00"),
            "{}",
            outcome.proposal.description
        );
    }

    /// A key fact's amount must be stated as the context writes it: the
    /// same number on its own, its currency beside it, its scale after
    /// it. One that is not is an unsupported claim, sends the document to
    /// review and never takes the place of the document's own total.
    #[test]
    fn a_key_fact_amount_is_accepted_only_as_the_context_writes_it() {
        const TOTALLED: &str = "Nimbus Orchard Supply Co.\n\nINVOICE\n\n\
Invoice date: April 30, 2025\n\nBill to Atlas Threadworks LLC\n\nTotal: $1,248.00";
        let unclaimed = with_key_facts(TOTALLED, &[]).proposal.description;
        assert!(unclaimed.contains("totalling $1,248.00"), "{unclaimed}");
        for (fact, cited) in [
            ("$48", Some("Total")),
            ("$7", None),
            // The end of a longer number, or its start.
            ("$248.00", Some("Total")),
            ("$1,248", Some("Total")),
            // Another currency, or a scale the document does not write.
            ("USD 1,248.00", Some("Total")),
            ("\u{20ac}1,248.00", Some("Total")),
            ("$1,248.00 million", Some("Total")),
        ] {
            let outcome = with_key_facts(TOTALLED, &[(fact, cited)]);
            let facts = outcome.facts.as_ref().unwrap();
            assert_eq!(
                facts.support.key_facts,
                vec![Support::Unsupported],
                "{fact}"
            );
            assert!(facts.key_facts.is_empty(), "{fact}");
            assert!(
                outcome
                    .reasons
                    .contains(&ReviewReason::DescriptionUnsupported),
                "{fact}"
            );
            // The document's own total stands.
            assert_eq!(outcome.proposal.description, unclaimed, "{fact}");
        }
        for fact in ["$1,248.00", "Total: $1,248.00"] {
            let outcome = with_key_facts(TOTALLED, &[(fact, Some("Total"))]);
            assert!(
                !outcome
                    .reasons
                    .contains(&ReviewReason::DescriptionUnsupported),
                "{fact}: {:?}",
                outcome.reasons
            );
            assert_eq!(outcome.facts.as_ref().unwrap().key_facts, vec![fact]);
        }
        // A code the document writes, but not beside the number.
        const CODED: &str = "Nimbus Orchard Supply Co.\n\nINVOICE\n\n\
All amounts are in USD.\n\nTotal: $1,248.00";
        for (fact, stated) in [("USD 1,248.00", false), ("$1,248.00", true)] {
            let outcome = with_key_facts(CODED, &[(fact, Some("Total"))]);
            let support = &outcome.facts.as_ref().unwrap().support.key_facts;
            assert_eq!(support[0] != Support::Unsupported, stated, "{fact}");
        }
        // A currency written after the number is beside it; a scale must
        // be the document's.
        const SCALED: &str = "Corriveau Capital LLC\n\nINVOICE\n\n\
Arrangement fee on a facility of $5.2 million: 1,248.00 USD";
        for (fact, stated) in [
            ("USD 1,248.00", true),
            ("$5.2 million", true),
            ("$5.2", false),
            ("$5.2 billion", false),
        ] {
            let outcome = with_key_facts(SCALED, &[(fact, Some("Arrangement"))]);
            let support = &outcome.facts.as_ref().unwrap().support.key_facts;
            assert_eq!(support[0] != Support::Unsupported, stated, "{fact}");
        }
        // An invisible character inside the document's amount hides
        // nothing: the reply's plain copy of it is stated.
        const SPACED: &str = "Nimbus Orchard Supply Co.\n\nINVOICE\n\n\
Invoice date: April 30, 2025\n\nBill to Atlas Threadworks LLC\n\nTotal: $1,\u{200b}248.00";
        let outcome = with_key_facts(SPACED, &[("$1,248.00", Some("Total"))]);
        assert!(
            !outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported),
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.facts.as_ref().unwrap().key_facts, vec!["$1,248.00"]);
    }

    /// A stated type is written in the reply's words, but for a mark the
    /// line that states it does not hold: "Invoice ✓✓" or "Invoice ✔️" on
    /// an `INVOICE` line is written "Invoice". The reply's words are kept,
    /// and its casing unless it is all lower case.
    #[test]
    fn a_stated_type_keeps_only_the_marks_its_line_holds() {
        let index = index_of(INVOICE);
        for (reply, written) in [
            ("Invoice \u{2713}\u{2713}", "Invoice"),
            // A check mark in emoji form: its variation selector goes with it.
            ("Invoice \u{2714}\u{fe0f}", "Invoice"),
            ("invoice \u{2714}\u{fe0f}", "Invoice"),
            ("Invoice!!", "Invoice"),
            ("In\u{200b}voice", "Invoice"),
            ("Invoice", "Invoice"),
            ("invoice", "Invoice"),
            ("INVOICE", "INVOICE"),
        ] {
            let mut facts = invoice_facts(&index);
            facts.document_type = Some(reply.into());
            facts.type_evidence = Vec::new();
            let outcome = check(facts, &whole(&index), &index);
            assert_eq!(
                outcome.proposal.document_type.as_deref(),
                Some(written),
                "{reply}"
            );
            let filename = crate::naming::compose_filename(&outcome.proposal, "pdf", &[]).value;
            assert!(
                filename.starts_with(&format!("2025-05-01 {written} from")),
                "{filename}"
            );
            for text in [&filename, &outcome.proposal.description] {
                assert!(
                    !text.contains(['\u{2713}', '\u{2714}', '\u{fe0f}', '\u{200b}', '!']),
                    "{text:?}"
                );
            }
        }
        // The reply's singular stands where its line writes a plural.
        const PAYABLE: &str = "Halvorsen Fixture Works LLC\n12 Quay Street, Brackenridge\n\n\
Invoice Date: May 1, 2025\n\nBill To: Quillon Ridge Bakery, Inc.\n\n\
Display shelving for the bakery counter, $1,248.00\n\n\
All invoices are payable within 30 days.";
        let (outcome, _) = facts_for(PAYABLE, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "All invoices")],
            document_date: Some("2025-05-01".into()),
            date_evidence: vec![id_of(index, "Invoice Date")],
            parties: vec![party(
                "Halvorsen Fixture Works LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "Halvorsen Fixture Works LLC")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Invoice"));
        let filename = crate::naming::compose_filename(&outcome.proposal, "pdf", &[]).value;
        assert_eq!(
            filename,
            "2025-05-01 Invoice from Halvorsen Fixture Works LLC.pdf"
        );
        // A mark the line holds once is no licence to decorate the type with
        // it, at either end.
        const NUMBERED: &str = "Halvorsen Fixture Works LLC\n12 Quay Street, Brackenridge\n\n\
Invoice Date: May 1, 2025\n\nAll invoices #123 are payable within 30 days.";
        for reply in ["Invoice ###", "## Invoice", "# Invoice #"] {
            let (outcome, _) = facts_for(NUMBERED, |index| ModelFacts {
                document_type: Some(reply.into()),
                type_evidence: vec![id_of(index, "All invoices")],
                document_date: Some("2025-05-01".into()),
                date_evidence: vec![id_of(index, "Invoice Date")],
                ..ModelFacts::default()
            });
            assert_eq!(
                outcome.proposal.document_type.as_deref(),
                Some("Invoice"),
                "{reply}"
            );
        }
        // A mark the line holds is kept, compared through normalizing: the
        // reply's straight apostrophe for the line's curly one, or for the
        // acute accent or backtick OCR reads one as; a bracket for a
        // bracket. An accent on a kept letter stays with it.
        for (text, reply) in [
            (
                "Larkspur Property Management LLC\n\nOWNER\u{2019}S STATEMENT\n\n\
Statement Date: June 30, 2025\n\nOwner: Delphine Okonkwo-Reyes",
                "Owner's Statement",
            ),
            (
                "Larkspur Property Management LLC\n\nOWNER\u{b4}S STATEMENT\n\n\
Statement Date: June 30, 2025\n\nOwner: Delphine Okonkwo-Reyes",
                "Owner's Statement",
            ),
            (
                "Larkspur Property Management LLC\n\nOWNER`S STATEMENT\n\n\
Statement Date: June 30, 2025\n\nOwner: Delphine Okonkwo-Reyes",
                "Owner's Statement",
            ),
            (
                "CAF\u{c9} SUPPLY AGREEMENT\n\nThis Agreement is made on March 3, 2026 \
between Corvane Analytics Inc. and Larkhaven Seed Company.",
                "Cafe\u{301} Supply Agreement",
            ),
            (
                "NON-DISCLOSURE AGREEMENT (NDA)\n\nThis Agreement is made on March 3, 2026 \
between Corvane Analytics Inc. and Larkhaven Seed Company.",
                "Non-Disclosure Agreement (NDA)",
            ),
        ] {
            let (outcome, _) = facts_for(text, |_| ModelFacts {
                document_type: Some(reply.into()),
                ..ModelFacts::default()
            });
            assert_eq!(outcome.proposal.document_type.as_deref(), Some(reply));
        }
        const NOTICE: &str = "Basalt Commercial Credit Corp.\n\nOctober 14, 2025\n\n\
NOTICE OF DEFAULT AND RESERVATION OF RIGHTS\n\n\
Glasswing Ceramics LLC is in default under the Loan Agreement dated March 3, 2023.";
        let (outcome, _) = facts_for(NOTICE, |index| ModelFacts {
            document_type: Some("Notice-of-Default \u{2713}".into()),
            document_date: Some("2025-10-14".into()),
            date_evidence: vec![id_of(index, "October 14")],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Notice of Default")
        );
    }

    #[test]
    fn a_type_written_in_lower_case_is_filed_as_a_title() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.document_type = Some("invoice".into());
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Invoice"));
        assert_eq!(titled("notice of default"), "Notice of Default");
        assert_eq!(titled("NOTICE of default"), "NOTICE of default");
    }

    #[test]
    fn a_reply_with_no_facts_goes_to_review_and_composes_from_the_title() {
        let index = index_of(INVOICE);
        let outcome = check(ModelFacts::default(), &whole(&index), &index);
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        assert!(outcome.reasons.contains(&ReviewReason::DateMissing));
        assert!(outcome.proposal.parties.is_empty());
    }

    /// A reply of the given facts about the whole of `text`.
    fn facts_for(
        text: &str,
        build: impl FnOnce(&EvidenceIndex) -> ModelFacts,
    ) -> (ValidationOutcome, EvidenceIndex) {
        let index = index_of(text);
        let context = whole(&index);
        let facts = build(&index);
        let outcome = check(facts, &context, &index);
        (outcome, index)
    }

    #[test]
    fn an_id_or_a_placeholder_written_as_a_value_is_no_value_at_all() {
        let (outcome, _) = facts_for(INVOICE, |index| ModelFacts {
            identifier: Some(id_of(index, "INVOICE")),
            subject: Some("..".into()),
            ..invoice_facts(index)
        });
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        let facts = outcome.facts.expect("facts");
        // The echoed id is no identifier; the document's own number on its
        // "Invoice No." line stands in.
        assert_eq!(facts.identifier.as_deref(), Some("INV-10438"));
        assert_eq!(facts.subject, None);
    }

    #[test]
    fn a_type_is_the_cited_titles_phrase_and_an_unstated_one_goes_to_review() {
        const NOTICE: &str = "Basalt Commercial Credit Corp.\n\nOctober 14, 2025\n\n\
NOTICE OF DEFAULT AND RESERVATION OF RIGHTS\n\n\
Glasswing Ceramics LLC is in default under the Loan Agreement dated March 3, 2023.";
        let notice = |index: &EvidenceIndex, kind: &str, cited: &str| ModelFacts {
            document_type: Some(kind.into()),
            type_evidence: vec![id_of(index, cited)],
            document_date: Some("2025-10-14".into()),
            date_evidence: vec![id_of(index, "October 14")],
            ..ModelFacts::default()
        };
        // The reply's "Loan Notice" cites the title: the title's words are
        // the type.
        let (outcome, _) = facts_for(NOTICE, |index| notice(index, "Loan Notice", "NOTICE OF"));
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Notice of Default and Reservation of Rights")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::TypeUnsupported));
        // Cited elsewhere, its words stated nowhere: the title stands in,
        // for review.
        let (outcome, _) = facts_for(NOTICE, |index| {
            notice(index, "Loan Notice", "Glasswing Ceramics")
        });
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Notice of Default and Reservation of Rights")
        );
        assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
        // Another document's name is not this one's type.
        let (outcome, _) = facts_for(NOTICE, |index| {
            notice(index, "Loan Agreement", "Glasswing Ceramics")
        });
        assert_ne!(
            outcome.proposal.document_type.as_deref(),
            Some("Loan Agreement")
        );
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
    }

    #[test]
    fn a_name_loses_the_address_ocr_ran_into_it() {
        const LEASE: &str = "LEASE AGREEMENT EFFECTIVE SEPTEMBER 1 2024\n\n\
PrOperty 47 JUniper LOOP Cedar Finch Properties Llc Orion Glass Studio inc";
        let (outcome, _) = facts_for(LEASE, |index| ModelFacts {
            document_type: Some("Lease Agreement".into()),
            type_evidence: vec![id_of(index, "LEASE")],
            document_date: Some("2024-09-01".into()),
            date_evidence: vec![id_of(index, "LEASE")],
            parties: vec![
                party(
                    "PrOperty 47 JUniper LOOP Cedar Finch Properties Llc",
                    Some(PartyRole::Client),
                    &[id_of(index, "Orion")],
                ),
                party(
                    "Orion Glass Studio inc",
                    Some(PartyRole::Tenant),
                    &[id_of(index, "Orion")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Cedar Finch Properties Llc", "Orion Glass Studio inc"]
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        let facts = outcome.facts.expect("facts");
        // Neither role is the document's: neither is kept.
        assert!(
            facts.parties.iter().all(|party| party.role.is_none()),
            "{:?}",
            facts.parties
        );
        assert_eq!(facts.parties[1].proposed_role, Some(PartyRole::Tenant));
    }

    #[test]
    fn someone_copied_in_is_never_a_filename_party() {
        const NOTICE: &str = "NOTICE OF TERMINATION\n\nDate of this Notice: December 29, 2026\n\n\
To: John Smith, 1420 Fielder Lane\n\n\
Northstar Lantern Works LLC, 88 Harbour Street cc: Marcus Reyes, Esq., outside counsel";
        let (outcome, _) = facts_for(NOTICE, |index| ModelFacts {
            document_type: Some("Notice of Termination".into()),
            type_evidence: vec![id_of(index, "NOTICE OF")],
            document_date: Some("2026-12-29".into()),
            date_evidence: vec![id_of(index, "Date of this")],
            parties: vec![
                party(
                    "Marcus Reyes",
                    Some(PartyRole::Addressee),
                    &[id_of(index, "cc:")],
                ),
                party(
                    "John Smith",
                    Some(PartyRole::Addressee),
                    &[id_of(index, "To:")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.parties, vec!["John Smith"]);
        let facts = outcome.facts.expect("facts");
        let reyes = facts
            .parties
            .iter()
            .find(|party| party.name == "Marcus Reyes")
            .expect("validated");
        assert!(reyes.copied);
        assert_eq!(reyes.role, Some(PartyRole::Other));
    }

    #[test]
    fn a_notice_whose_reply_named_no_addressee_is_to_its_to_field() {
        const NOTICE: &str = "NOTICE OF TERMINATION\n\nDate of this Notice: December 29, 2026\n\n\
To: John Smith, 1420 Fielder Lane, Cedar Rapids, IA 52402\n\n\
From: Harriet Voss, Vice President of People Operations\n\n\
Northstar Lantern Works LLC, 88 Harbour Street cc: Marcus Reyes, Esq., outside counsel\n\n\
Sincerely, Harriet Voss, Vice President of People Operations Northstar Lantern Works LLC";
        let reply = |index: &EvidenceIndex| ModelFacts {
            document_type: Some("Notice of Termination".into()),
            type_evidence: vec![id_of(index, "NOTICE OF")],
            document_date: Some("2026-12-29".into()),
            date_evidence: vec![id_of(index, "Date of this")],
            parties: vec![
                party(
                    "Harriet Voss",
                    Some(PartyRole::Other),
                    &[id_of(index, "From:")],
                ),
                party(
                    "Northstar Lantern Works LLC",
                    Some(PartyRole::Other),
                    &[id_of(index, "Northstar")],
                ),
            ],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(NOTICE, reply);
        assert_eq!(outcome.proposal.parties, vec!["John Smith"]);
        assert_eq!(
            outcome.proposal.description,
            "Notice of Termination from Northstar Lantern Works LLC to John Smith."
        );
        let facts = outcome.facts.expect("facts");
        assert_eq!(facts.parties[0].role, Some(PartyRole::Addressee));
        assert_eq!(facts.parties[0].support, Support::Context);
        // A reply that names the addressee itself is left as it is.
        let (named, _) = facts_for(NOTICE, |index| {
            let mut facts = reply(index);
            facts.parties.insert(
                0,
                party(
                    "John Smith",
                    Some(PartyRole::Addressee),
                    &[id_of(index, "To:")],
                ),
            );
            facts
        });
        assert_eq!(named.facts.expect("facts").parties.len(), 3);
    }

    #[test]
    fn a_subject_of_the_types_and_the_parties_words_says_nothing() {
        const SLIP: &str = "PACKING SLIP PS-311\n\nDATE JULY 15 2025 QUARTZ MEADOW RETAIL LLC";
        let (outcome, _) = facts_for(SLIP, |index| ModelFacts {
            document_type: Some("Packing Slip".into()),
            type_evidence: vec![id_of(index, "PACKING")],
            document_date: Some("2025-07-15".into()),
            date_evidence: vec![id_of(index, "DATE")],
            parties: vec![party(
                "Quartz Meadow Retail LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "DATE")],
            )],
            subject: Some("retail packing".into()),
            subject_evidence: vec![id_of(index, "DATE")],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.facts.expect("facts").subject, None);
        assert!(!outcome.proposal.description.contains("retail packing"));
    }

    #[test]
    fn a_title_whose_capitals_ocr_scattered_names_a_document_the_reply_did_not() {
        const RECEIPT: &str = "DElIvery RECeIPt DR-771\n\nJUNE 12 2025.\n\n\
Pine Echo Couriers LLC Violet Cartography Studio";
        let (outcome, _) = facts_for(RECEIPT, |index| ModelFacts {
            document_type: Some("Document".into()),
            type_evidence: vec![id_of(index, "DElIvery")],
            document_date: Some("2025-06-12".into()),
            date_evidence: vec![id_of(index, "JUNE")],
            parties: vec![party(
                "Pine Echo Couriers LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "Pine")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Delivery Receipt")
        );
        assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
    }

    #[test]
    fn a_deck_presented_by_a_party_is_from_it() {
        const DECK: &str = "QUARTERLY BUSINESS REVIEW\n\nPrepared for Contoso Worldwide, Inc.\n\n\
Presented by Ridgeline Cartography LLC\n\nPresented on May 21, 2026";
        let (outcome, _) = facts_for(DECK, |index| ModelFacts {
            document_type: Some("Quarterly Business Review".into()),
            type_evidence: vec![id_of(index, "QUARTERLY")],
            document_date: Some("2026-05-21".into()),
            date_evidence: vec![id_of(index, "Presented on")],
            parties: vec![
                party(
                    "Contoso Worldwide, Inc.",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Prepared for")],
                ),
                party(
                    "Ridgeline Cartography LLC",
                    Some(PartyRole::Sender),
                    &[id_of(index, "Presented by")],
                ),
            ],
            ..ModelFacts::default()
        });
        let facts = outcome.facts.expect("facts");
        assert_eq!(facts.parties[1].role, Some(PartyRole::Sender));
        assert!(
            outcome
                .proposal
                .description
                .contains("Ridgeline Cartography LLC"),
            "{}",
            outcome.proposal.description
        );
    }

    #[test]
    fn an_issued_documents_header_party_is_its_issuer() {
        const INVOICE: &str = "INVOICE INV-2048\n\nInvoice date: April 30, 2025\n\n\
Due date: May 30, 2025\n\nNimbus Orchard Supply Co. Bill to Atlas Threadworks LLC\n\n\
Total: $1,248.00";
        let (outcome, _) = facts_for(INVOICE, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            document_date: Some("2025-04-30".into()),
            date_role: Some(DateRole::Invoice),
            date_evidence: vec![id_of(index, "Invoice date")],
            parties: vec![party(
                "Nimbus Orchard Supply Co.",
                Some(PartyRole::Vendor),
                &[id_of(index, "Nimbus")],
            )],
            subject: Some("Bill to Atlas Threadworks LLC".into()),
            subject_evidence: vec![id_of(index, "Nimbus")],
            identifier: Some("INV-2048".into()),
            identifier_evidence: vec![id_of(index, "INVOICE")],
            key_facts: vec![KeyFact {
                fact: "Total: $1,248.00".into(),
                evidence: vec![id_of(index, "Total")],
            }],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
        assert_eq!(outcome.proposal.parties, vec!["Nimbus Orchard Supply Co."]);
        assert_eq!(
            outcome.proposal.description,
            "Invoice INV-2048 from Nimbus Orchard Supply Co., totalling $1,248.00."
        );
        // The customer alone, under its label, is never the issuer.
        let (outcome, _) = facts_for(INVOICE, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            parties: vec![party(
                "Atlas Threadworks LLC",
                Some(PartyRole::Vendor),
                &[id_of(index, "Nimbus")],
            )],
            ..ModelFacts::default()
        });
        assert_ne!(outcome.proposal.party_relation, PartyRelation::From);
    }

    #[test]
    fn a_form_by_and_between_two_parties_is_between_them_and_dated_when_it_starts() {
        const FORM: &str = "# Order Form\n\nThis Order Form is entered into by and between \
Tessellate Analytics Ltd. and Contoso Worldwide, Inc. under the Master Subscription Agreement \
dated August 8, 2022.\n\n| Subscription Start Date | February 1, 2026 |\n| Annual Fee | $96,000 |\n\n\
Signed on January 14, 2026 by authorized representatives of both parties.";
        let (outcome, _) = facts_for(FORM, |index| ModelFacts {
            document_type: Some("Order Form".into()),
            type_evidence: vec![id_of(index, "Order Form")],
            document_date: Some("2026-01-14".into()),
            date_role: Some(DateRole::Execution),
            date_evidence: vec![id_of(index, "Signed on")],
            parties: vec![
                party(
                    "Tessellate Analytics Ltd.",
                    Some(PartyRole::Client),
                    &[id_of(index, "by and between")],
                ),
                party(
                    "Contoso Worldwide, Inc.",
                    Some(PartyRole::Contractor),
                    &[id_of(index, "by and between")],
                ),
            ],
            key_facts: vec![KeyFact {
                fact: "$96,000".into(),
                evidence: vec![id_of(index, "Annual Fee")],
            }],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-02-01")
        );
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Effective));
        assert!(
            outcome
                .proposal
                .description
                .contains("annual fee of $96,000"),
            "{}",
            outcome.proposal.description
        );
    }

    #[test]
    fn a_compound_title_too_long_for_both_parties_keeps_its_first_kind() {
        const SETTLEMENT: &str = "SETTLEMENT AGREEMENT AND MUTUAL RELEASE\n\n\
This Settlement Agreement and Mutual Release is made effective as of July 22, 2026, by and \
between Harborline Freight Systems LLC and Quill and Vane Advisory Group, Inc.";
        let (outcome, _) = facts_for(SETTLEMENT, |index| ModelFacts {
            document_type: Some("Settlement Agreement and Mutual Release".into()),
            type_evidence: vec![id_of(index, "SETTLEMENT")],
            document_date: Some("2026-07-22".into()),
            date_evidence: vec![id_of(index, "July 22")],
            parties: vec![
                party(
                    "Harborline Freight Systems LLC",
                    None,
                    &[id_of(index, "Harborline")],
                ),
                party(
                    "Quill and Vane Advisory Group, Inc.",
                    None,
                    &[id_of(index, "Harborline")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Settlement Agreement"),
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.parties.len(), 2);
        assert!(
            outcome
                .proposal
                .description
                .starts_with("Settlement Agreement and Mutual Release between"),
            "{}",
            outcome.proposal.description
        );
    }

    #[test]
    fn an_amount_is_the_one_the_document_labels() {
        const LETTER: &str = "DEMAND FOR PAYMENT\n\nJuly 8, 2026\n\n\
To: Highmeadow Orchard Supply Co.\n\n\
Interest accrued through the date of this letter is $1,287.40, for a total now due of \
$28,703.00.";
        let reply = |index: &EvidenceIndex, kind: &str| ModelFacts {
            document_type: Some(kind.into()),
            type_evidence: vec![id_of(index, "DEMAND")],
            document_date: Some("2026-07-08".into()),
            date_evidence: vec![id_of(index, "July 8")],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(LETTER, |index| reply(index, "Demand for Payment"));
        let description = &outcome.proposal.description;
        assert!(description.contains("$28,703.00"), "{description}");
        assert!(!description.contains("$1,287.40"), "{description}");
        // An issued document's amount is its last labelled total, not a
        // line item or a subtotal.
        const INVOICE: &str = "INVOICE INV-7\n\nInvoice date: May 1, 2025\n\n\
| Item | Amount |\n| Shelving | $1,000.00 |\n| Subtotal | $1,000.00 |\n| Tax | $80.00 |\n\
| Total | $1,080.00 |";
        let (outcome, _) = facts_for(INVOICE, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            ..ModelFacts::default()
        });
        let description = &outcome.proposal.description;
        assert!(description.contains("totalling $1,080.00"), "{description}");
        // A line a hosted reply cites for the amount stands in when the
        // document labels none; a line with no amount gives none and is no
        // claim either.
        const NOTE: &str = "MEMO\n\nJuly 8, 2026\n\nWe paid $500.00 on Friday.\n\nThanks.";
        let (outcome, _) = facts_for(NOTE, |index| ModelFacts {
            document_type: Some("Memo".into()),
            type_evidence: vec![id_of(index, "MEMO")],
            amount_evidence: vec![id_of(index, "We paid")],
            ..ModelFacts::default()
        });
        assert!(
            outcome.proposal.description.contains("$500.00"),
            "{}",
            outcome.proposal.description
        );
        let (outcome, _) = facts_for(NOTE, |index| ModelFacts {
            document_type: Some("Memo".into()),
            type_evidence: vec![id_of(index, "MEMO")],
            amount_evidence: vec![id_of(index, "Thanks")],
            ..ModelFacts::default()
        });
        assert!(!outcome.proposal.description.contains('$'));
        let facts = outcome.facts.unwrap();
        assert_eq!(facts.support.amount, Support::Unsupported);
        assert!(
            !outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported)
        );
    }

    #[test]
    fn an_amount_is_read_from_its_rows_label_its_header_or_its_title() {
        let described = |text: &str, kind: &str| {
            let (outcome, _) = facts_for(text, |index| ModelFacts {
                document_type: Some(kind.into()),
                type_evidence: vec![id_of(index, &kind.to_uppercase())],
                ..ModelFacts::default()
            });
            outcome.proposal.description
        };
        // The nearest cell before the amount labels it, past empty cells
        // and the amount it replaces.
        let order = described(
            "PURCHASE ORDER\n\n| Line | Item | Ext. Price |\n| 1 | Servo motor | $37,530.00 |\n\
| | | | Order Total (USD) | $84,438.00 |",
            "Purchase Order",
        );
        assert!(order.contains("totalling $84,438.00"), "{order}");
        let rent = described(
            "NOTICE OF RENT INCREASE\n\n| Charge | Current | New |\n\
| Base rent | $1,845.00 | $1,965.00 |\n\nThe base rent increase is $120.00 per month.",
            "Notice of Rent Increase",
        );
        assert!(rent.contains("base rent of $1,965.00"), "{rent}");
        // An amount alone in its row is labelled by its header.
        let loss = described(
            "PROPERTY LOSS NOTICE\n\n| Estimated amount of loss | Property damaged |\n\
| $38,500.00 | Flooring, two mixers |",
            "Property Loss Notice",
        );
        assert!(loss.contains("$38,500.00"), "{loss}");
        // With no labelled amount, the one a first-page title states.
        let credit = described(
            "CREDIT AGREEMENT\n\n$350,000,000 SENIOR SECURED CREDIT FACILITIES\n\n\
The Lenders agree to lend on the terms below.",
            "Credit Agreement",
        );
        assert!(credit.contains("$350,000,000"), "{credit}");
    }

    #[test]
    fn a_letters_inside_address_after_its_dateline_is_no_letterhead() {
        const LETTER: &str = "Mireille Saltonstall 18 Alder Court Bellmoor, WI 53511\n\n\
March 2, 2026\n\nDr. Rufus Pemberton, Practice Owner\n\n\
Ashgrove Veterinary Clinic 640 Kingsfold Avenue\n\nRe: Letter of Resignation\n\n\
Dear Dr. Pemberton:\n\nPlease accept this letter as formal notice of my resignation.";
        let (outcome, _) = facts_for(LETTER, |index| ModelFacts {
            document_type: Some("Letter of Resignation".into()),
            type_evidence: vec![id_of(index, "Re:")],
            document_date: Some("2026-03-02".into()),
            date_evidence: vec![id_of(index, "March 2")],
            parties: vec![
                party(
                    "Mireille Saltonstall",
                    Some(PartyRole::Sender),
                    &[id_of(index, "Mireille")],
                ),
                party(
                    "Dr. Rufus Pemberton",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "Dr. Rufus")],
                ),
            ],
            ..ModelFacts::default()
        });
        let facts = outcome.facts.expect("facts");
        let role_of = |name: &str| {
            facts
                .parties
                .iter()
                .find(|party| party.name == name)
                .and_then(|party| party.role)
        };
        assert_eq!(role_of("Mireille Saltonstall"), Some(PartyRole::Sender));
        assert_ne!(role_of("Dr. Rufus Pemberton"), Some(PartyRole::Issuer));
        // A receipt greets no one: its date line ends no letterhead.
        const RECEIPT: &str = "DELIVERY RECEIPT DR-771\n\nJUNE 12 2025\n\nPine Echo Couriers LLC";
        let (outcome, _) = facts_for(RECEIPT, |index| ModelFacts {
            document_type: Some("Delivery Receipt".into()),
            type_evidence: vec![id_of(index, "DELIVERY")],
            parties: vec![party(
                "Pine Echo Couriers LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "Pine")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.facts.expect("facts").parties[0].role,
            Some(PartyRole::Issuer)
        );
    }

    #[test]
    fn a_letters_re_line_names_it_when_the_reply_named_no_kind_it_states() {
        const LETTER: &str = "February 10, 2026\n\nRe: Offer of Employment - Senior Data Engineer\n\n\
Dear Ms. Vance:\n\nWe are pleased to offer you the position of Senior Data Engineer.";
        let (outcome, _) = facts_for(LETTER, |index| ModelFacts {
            document_type: Some("Job Offer Letter".into()),
            type_evidence: vec![id_of(index, "Re:")],
            document_date: Some("2026-02-10".into()),
            date_evidence: vec![id_of(index, "February")],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Offer of Employment")
        );
        assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
    }

    #[test]
    fn a_document_states_its_own_number_on_its_title_or_in_its_field() {
        const SLIP: &str = "PACKING SLIP PS-311\n\nDATE JULY 15 2025 QUARTZ MEADOW RETAIL LLC";
        let (outcome, _) = facts_for(SLIP, |index| ModelFacts {
            document_type: Some("Packing Slip".into()),
            type_evidence: vec![id_of(index, "PACKING")],
            ..ModelFacts::default()
        });
        assert!(
            outcome
                .proposal
                .description
                .starts_with("Packing Slip PS-311"),
            "{}",
            outcome.proposal.description
        );
        const POLICY: &str = "NOTICE OF CANCELLATION\n\nPolicy Number: KC-WC-7710345\n\n\
Date of notice: August 12, 2026";
        let (outcome, _) = facts_for(POLICY, |index| ModelFacts {
            document_type: Some("Notice of Cancellation".into()),
            type_evidence: vec![id_of(index, "NOTICE")],
            ..ModelFacts::default()
        });
        assert!(
            outcome
                .proposal
                .description
                .contains("policy KC-WC-7710345"),
            "{}",
            outcome.proposal.description
        );
        // A date, a year or a page after the title is no number.
        const LEASE: &str =
            "LEASE AGREEMENT EFFECTIVE SEPTEMBER 1 2024\n\nAcme LLC and Contoso Inc.";
        let (outcome, _) = facts_for(LEASE, |index| ModelFacts {
            document_type: Some("Lease Agreement".into()),
            type_evidence: vec![id_of(index, "LEASE")],
            ..ModelFacts::default()
        });
        assert!(outcome.facts.unwrap().identifier.is_none());
    }

    #[test]
    fn a_name_cut_short_is_completed_to_the_organisation_and_its_scattered_capitals_tidied() {
        const ORDER: &str =
            "PURCHASE ORDER PO-310\n\nDAtE JULY 14 20z5\n\nEMBer POSt MANUFACtURInG LLC";
        let (outcome, _) = facts_for(ORDER, |index| ModelFacts {
            document_type: Some("Purchase Order".into()),
            type_evidence: vec![id_of(index, "PURCHASE")],
            parties: vec![party(
                "MANUFACTURING LLC",
                Some(PartyRole::Issuer),
                &[id_of(index, "EMBer")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["EMBER POST MANUFACTURING LLC"]
        );
    }

    #[test]
    fn someone_who_signs_for_a_party_is_no_party_of_their_own() {
        const NOTICE: &str = "NOTICE OF TERMINATION\n\nDate of this Notice: December 29, 2026\n\n\
From: Harriet Voss, Vice President of People Operations\n\n\
This letter is notice that Northstar Lantern Works LLC is ending your employment.\n\n\
Sincerely, Harriet Voss, Vice President of People Operations, Northstar Lantern Works LLC";
        let (outcome, _) = facts_for(NOTICE, |index| ModelFacts {
            document_type: Some("Notice of Termination".into()),
            type_evidence: vec![id_of(index, "NOTICE OF")],
            document_date: Some("2026-12-29".into()),
            date_evidence: vec![id_of(index, "Date of this")],
            parties: vec![
                party(
                    "Harriet Voss",
                    Some(PartyRole::Other),
                    &[id_of(index, "From:")],
                ),
                party(
                    "Northstar Lantern Works LLC",
                    Some(PartyRole::Other),
                    &[id_of(index, "This letter")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|party| party.contains("Harriet")),
            "{:?}",
            outcome.proposal
        );
        let facts = outcome.facts.unwrap();
        assert!(facts.parties.iter().any(|party| party.signatory));
    }

    #[test]
    fn an_issued_document_whose_reply_named_no_one_is_from_its_header() {
        const SLIP: &str = "PACKING SLIP PS-311\n\nDATE JULY 15 2025 QUARTZ MEADOW RETAIL LLC\n\n\
Ship to: Larkspur Bistro LLC";
        let (outcome, _) = facts_for(SLIP, |index| ModelFacts {
            document_type: Some("Packing Slip".into()),
            type_evidence: vec![id_of(index, "PACKING")],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
        assert_eq!(outcome.proposal.parties, vec!["QUARTZ MEADOW RETAIL LLC"]);
    }

    #[test]
    fn a_subject_names_only_what_the_document_names() {
        const NOTICE: &str = "NOTICE OF SPECIAL MEETING OF MEMBERS\n\nDated August 3, 2026\n\n\
Members will vote on a plan to merge the credit union into Ironbridge Community Credit Union. \
The merger needs a majority of votes cast.";
        let reply = |index: &EvidenceIndex, subject: &str| ModelFacts {
            document_type: Some("Notice of Special Meeting of Members".into()),
            type_evidence: vec![id_of(index, "NOTICE OF")],
            document_date: Some("2026-08-03".into()),
            date_evidence: vec![id_of(index, "Dated")],
            subject: Some(subject.into()),
            subject_evidence: vec![id_of(index, "Members will")],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(NOTICE, |index| reply(index, "Credit Union Merger"));
        let description = &outcome.proposal.description;
        assert!(description.contains("credit union merger"), "{description}");
        let (outcome, _) = facts_for(NOTICE, |index| {
            reply(index, "merger with Ironbridge Community Credit Union")
        });
        let description = &outcome.proposal.description;
        assert!(
            description.contains("Ironbridge Community Credit Union"),
            "{description}"
        );
    }
}
