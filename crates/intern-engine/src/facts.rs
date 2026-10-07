//! Validation of an evidence-pipeline reply: every fact checked against the
//! units the model was shown, and the filename and description composed
//! from what survives.
//!
//! The digest pipeline checks a reply against the digest the model read.
//! Here the model read excerpts - the units retrieval chose - so there are
//! two views of the document ([`ValidationScope`]):
//!
//! * **context**: the units the prompt carried. A fact is accepted only if
//!   it is stated here, exactly as the digest pipeline accepts only what its
//!   digest states. A fact the document states outside the excerpts is not
//!   accepted: the model could not have read it.
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

use std::collections::BTreeSet;

use crate::compose::{
    CastMember, DescriptionFacts, amount_label, describe, identifier_word, money_in,
    relation_from_roles,
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
    digest_contains_loosely, extract_stated_dates, normalize, normalize_loosely, numeric_dates,
    stated_dates,
};
use crate::index::{EvidenceIndex, EvidenceUnit, UnitKind};
use crate::infer::{
    complete_type_from_title, infer_date_role, infer_document_type, repair_issued_relation,
};
use crate::retrieve::EvidenceContext;
use crate::validate::{
    GENERIC_CAPITALS, READY_CONFIDENCE, current_year, date_is_tainted, deadline_fires,
    effective_alternates, first_unsupported_claim, issue_date_alternates, push,
    reading_is_unsettled, type_is_supported, validate_description, year_is_plausible,
};

/// Share of a subject's significant words its cited units must hold for it
/// to be written into the description.
const SUBJECT_OVERLAP: f32 = 0.6;
/// How far after a party's name a defined term may stand: `("Tenant")`,
/// `(the "Borrower")`, `, as Lender`.
const DEFINED_TERM_REACH: usize = 60;
/// How far before a party's name its label may stand: `Bill To:`.
const LABEL_REACH: usize = 40;
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

    // The type.
    let mut type_line = None;
    let proposed_type = present(facts.document_type.as_deref());
    let (mut document_type, type_supported) = match proposed_type {
        None => (None, true),
        Some(value) => {
            let words = significant_words(value);
            let found = find(
                scope,
                &facts.type_evidence,
                |view| type_is_supported(value, view),
                |unit| {
                    line_where(unit, |line| {
                        words.iter().any(|word| contains_whole(line, word))
                    })
                },
            );
            support.document_type = found.support;
            support.miscited_ids += found.miscited;
            if found.support == Support::Unsupported {
                (None, false)
            } else {
                remember(&found, &mut references);
                type_line = found.line;
                (Some(titled(value)), true)
            }
        }
    };
    if !type_supported {
        push(&mut reasons, ReviewReason::TypeUnsupported);
    }
    let names = facts
        .parties
        .iter()
        .map(|party| party.name.clone())
        .collect::<Vec<_>>();
    if type_supported {
        document_type =
            document_type.map(|value| complete_type_from_title(&value, context, &names));
    }
    if document_type.is_none() {
        match infer_document_type(context) {
            Some(inferred) => {
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

    // The parties.
    let mut parties: Vec<(ValidatedParty, Option<u32>)> = Vec::new();
    let mut parties_supported = true;
    for party in &facts.parties {
        let Some(name) = present(Some(party.name.as_str())) else {
            parties_supported = false;
            continue;
        };
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
        let role_support = match party.role {
            None | Some(PartyRole::Other) => Support::Absent,
            Some(role) => role_support(scope, name, role, &party.evidence),
        };
        let first_seen = first_appearance(scope, name);
        parties.push((
            ValidatedParty {
                name: name.to_owned(),
                role: party.role,
                role_support,
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
    let parties = parties
        .into_iter()
        .map(|(party, _)| party)
        .collect::<Vec<_>>();
    let cast = parties
        .iter()
        .map(|party| CastMember {
            name: party.name.clone(),
            role: party.role,
            role_supported: matches!(party.role_support, Support::Cited | Support::Context),
        })
        .collect::<Vec<_>>();
    let class = DocumentClass::of(document_type.as_deref());
    let cue_issuer = cue_issuer(document_type.as_deref(), &cast, context);
    let relation = relation_from_roles(class, document_type.as_deref(), &cast, cue_issuer);

    // The subject, the identifier and the key facts: every number and name
    // in them must be in what the model was shown.
    let subject_value = present(facts.subject.as_deref());
    let mut subject = None;
    if let Some(value) = subject_value {
        let found = claims_found(scope, &facts.subject_evidence, value);
        support.subject = found.support;
        support.miscited_ids += found.miscited;
        if found.support == Support::Unsupported {
            push(&mut reasons, ReviewReason::DescriptionUnsupported);
        } else {
            let cited = scope.cited_units(&facts.subject_evidence);
            if subject_is_grounded(value, &scope.view(&cited)) {
                remember(&found, &mut references);
                subject = Some(value.to_owned());
            }
        }
    }
    let mut identifier = None;
    if let Some(value) = present(facts.identifier.as_deref()) {
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
                .and_then(|unit| identifier_label(unit, found.line.as_deref(), value));
            identifier = Some((identifier_word(label.as_deref()), value.to_owned()));
        }
    }
    let mut key_facts = Vec::new();
    for fact in &facts.key_facts {
        let Some(value) = present(Some(fact.fact.as_str())) else {
            continue;
        };
        let found = claims_found(scope, &fact.evidence, value);
        support.key_facts.push(found.support);
        support.miscited_ids += found.miscited;
        if found.support == Support::Unsupported {
            push(&mut reasons, ReviewReason::DescriptionUnsupported);
        } else {
            remember(&found, &mut references);
            key_facts.push(value.to_owned());
        }
    }

    // The description, composed, then held to the digest pipeline's rules.
    let amount = key_facts.iter().find_map(|fact| {
        let money = money_in(fact)?;
        if class == DocumentClass::Issued {
            Some((None, money))
        } else {
            amount_label(fact).map(|label| (Some(label), money))
        }
    });
    let other_fact = key_facts
        .iter()
        .find(|fact| money_in(fact).is_none())
        .map(String::as_str);
    let date_surface = document_date
        .as_deref()
        .zip(date_line.as_deref())
        .and_then(|(date, line)| surface_form(date, line));
    let composed = describe(&DescriptionFacts {
        class,
        document_type: document_type.as_deref(),
        relation: Some(&relation),
        parties: &cast,
        subject: subject.as_deref(),
        identifier: identifier
            .as_ref()
            .map(|(word, value)| (*word, value.as_str())),
        amount,
        other_fact,
        date_surface: date_surface.as_deref(),
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
/// to write it into the description.
fn subject_is_grounded(subject: &str, cited: &ScopeView) -> bool {
    if cited.is_empty() {
        return false;
    }
    let words = significant_words(subject);
    if words.is_empty() {
        return false;
    }
    let held = words
        .iter()
        .filter(|word| digest_contains(cited, word))
        .count();
    held as f32 >= words.len() as f32 * SUBJECT_OVERLAP
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
        PartyRole::Landlord => (&["landlord", "lessor"], &["landlord", "lessor"]),
        PartyRole::Tenant => (&["tenant", "lessee", "resident"], &["tenant", "lessee"]),
        PartyRole::Issuer => (
            &[
                "issuer",
                "issued by",
                "remit to",
                "payable to",
                "from",
                "prepared by",
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
        PartyRole::Sender => (&["from", "sender"], &["sender"]),
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
    let supports = |unit: &EvidenceUnit| unit_supports_role(unit, &normalized, &loose, role);
    if scope.cited_units(cited).into_iter().any(supports) {
        return Support::Cited;
    }
    if scope.context_units().any(supports) {
        return Support::Context;
    }
    Support::Unsupported
}

fn unit_supports_role(unit: &EvidenceUnit, normalized: &str, loose: &str, role: PartyRole) -> bool {
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
    if matches!(role, PartyRole::Issuer | PartyRole::Sender) && unit.features.position.letterhead {
        return true;
    }
    let on_cue_line = |cues: &[&str]| {
        unit.text.lines().any(|line| {
            let line = normalize_loosely(line);
            line.contains(loose) && cues.iter().any(|cue| line.contains(cue))
        })
    };
    match role {
        PartyRole::Issuer | PartyRole::Vendor | PartyRole::Seller => on_cue_line(ISSUER_CUES),
        PartyRole::Customer | PartyRole::Recipient | PartyRole::Buyer | PartyRole::Client => {
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

/// The label an identifier carries: its field's or row's label, or the
/// words just before it on its line ("Invoice No.").
fn identifier_label(unit: &EvidenceUnit, line: Option<&str>, identifier: &str) -> Option<String> {
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
    candidate.party_relation =
        relation_from_roles(DocumentClass::of(document_type), document_type, &cast, None).relation;
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
            unknown_evidence: Vec::new(),
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
             regarding rent for Apartment 4C."
        );
        // Swapped roles: neither is stated, so neither decides anything.
        let outcome = check(
            with_roles(PartyRole::Landlord, PartyRole::Tenant),
            &context,
            &index,
        );
        let parties = &outcome.facts.as_ref().unwrap().parties;
        assert_eq!(parties[0].role_support, Support::Unsupported);
        assert_eq!(parties[1].role_support, Support::Unsupported);
        assert_eq!(outcome.proposal.party_relation, PartyRelation::None);
    }

    /// A subject's names and numbers are claims, checked as a written
    /// description's are; one the document does not contain is never
    /// written into the description.
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

    /// A subject the document states, but not in the units the reply cited
    /// for it, is supported - and not written down: the description says
    /// only what its evidence shows.
    #[test]
    fn a_subject_its_cited_units_do_not_hold_stays_out_of_the_description() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
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
}
