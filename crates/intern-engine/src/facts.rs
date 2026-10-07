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
    digest_contains_loosely, extract_stated_dates, normalize, normalize_loosely, numeric_dates,
    stated_dates,
};
use crate::index::{EvidenceIndex, EvidenceUnit, UnitKind};
use crate::infer::{infer_date_role, repair_issued_relation};
use crate::phrases::{
    has_a_kind, head_noun, is_placeholder, is_reference, label_and_value, names_a_kind,
    opens_with_party_label, phrase_positions, subject_value, tidy_case, title_phrase, trim_name,
    words,
};
use crate::retrieve::EvidenceContext;
use crate::validate::{
    GENERIC_CAPITALS, READY_CONFIDENCE, current_year, date_is_tainted, deadline_fires,
    effective_alternates, first_unsupported_claim, issue_date_alternates, push,
    reading_is_unsettled, validate_description, year_is_plausible,
};

/// Share of a subject's significant words its cited units must hold for it
/// to be written into the description.
const SUBJECT_OVERLAP: f32 = 0.6;
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
    // and not as another document's name. A type the document does not
    // state is never written; the title, if one names the same kind of
    // document, stands in for review.
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
                type_line = found.line.clone();
                (Some(titled(value)), true)
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
    // An issued document whose reply said what it is but named no one: the
    // one organisation at the head of its first page, before any customer,
    // is its issuer.
    if class == DocumentClass::Issued
        && proposed_type.is_some()
        && type_supported
        && parties.is_empty()
        && let Some((name, unit)) = header_organisation(scope)
    {
        parties.push(ValidatedParty {
            name: display_name(&name),
            role: Some(PartyRole::Issuer),
            document_role: Some(PartyRole::Issuer),
            support: Support::Context,
            evidence: vec![unit.id.clone()],
            ..ValidatedParty::default()
        });
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
    // in them must be in what the model was shown.
    let mut subject = None;
    if let Some(value) = content(facts.subject.as_deref(), scope) {
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
    let mut key_facts = Vec::new();
    let mut key_units = Vec::new();
    for fact in &facts.key_facts {
        let Some(value) = content(Some(fact.fact.as_str()), scope) else {
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
    let says_something = |value: &str| -> Option<String> {
        let value = subject_value(value)?
            .trim()
            .trim_end_matches(['.', ';', ',']);
        let names_party = cast.iter().any(|party| {
            contains_whole(&normalize_loosely(value), &normalize_loosely(&party.name))
        });
        let holds_identifier = identifier
            .as_ref()
            .is_some_and(|(_, id)| normalize_loosely(value).contains(&normalize_loosely(id)));
        (!value.is_empty()
            && !opens_with_party_label(value)
            && !repeats(value, document_type.as_deref())
            && !names_party
            && !holds_identifier
            && money_in(value).is_none())
        .then(|| value.to_owned())
    };
    let subject = subject.as_deref().and_then(says_something);
    // The amount: a key fact's, or the one the line a compact reply cited
    // for it states.
    let mut amount = key_facts.iter().zip(&key_units).find_map(|(fact, unit)| {
        let money = money_in(fact)?;
        Some((amount_label_of(fact, *unit, money), money.to_owned()))
    });
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
            if !title_like(unit, &line) || !(unit.kind == UnitKind::Heading || in_capitals(&line)) {
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
    let supports = |unit: &EvidenceUnit| unit_supports_role(unit, &normalized, &loose, role, true);
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
    LABELLED_ROLES.into_iter().find(|role| {
        naming
            .iter()
            .any(|unit| unit_supports_role(unit, &normalized, &loose, *role, false))
    })
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
    let kinds = identifier_labels(&head);
    if kinds.is_empty() {
        return None;
    }
    scope.context_units().find_map(|unit| {
        let label = unit.label.as_deref()?;
        let label_words = words(label);
        let names_kind = label_words
            .iter()
            .any(|word| kinds.contains(&word.as_str()));
        let names_number = label.contains('#')
            || label_words
                .iter()
                .any(|word| NUMBER_WORDS.contains(&word.as_str()));
        if !(names_kind && names_number) {
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
            _ => label_and_value(&unit.text)
                .map(|(_, value)| value.to_owned())
                .unwrap_or_else(|| unit.text.clone()),
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
             regarding rent for Apartment 4C."
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
    fn an_invoice_naming_only_its_customer_files_under_no_party() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.parties.remove(0);
        let outcome = check(facts, &whole(&index), &index);
        assert!(
            outcome.proposal.parties.is_empty(),
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::None);
        assert!(
            outcome
                .proposal
                .description
                .starts_with("Invoice to Quillon Ridge Bakery, Inc. for display shelving"),
            "{}",
            outcome.proposal.description
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
    fn a_compact_replys_amount_is_read_from_the_line_it_cites() {
        const LETTER: &str = "DEMAND FOR PAYMENT\n\nJuly 8, 2026\n\n\
To: Highmeadow Orchard Supply Co.\n\n\
Interest accrued through the date of this letter is $1,287.40, for a total now due of \
$28,703.00.\n\n| Charge | Before | Now |\n| Base rent | $1,845.00 | $1,965.00 |";
        let reply = |index: &EvidenceIndex, cited: &str| ModelFacts {
            document_type: Some("Demand for Payment".into()),
            type_evidence: vec![id_of(index, "DEMAND")],
            document_date: Some("2026-07-08".into()),
            date_evidence: vec![id_of(index, "July 8")],
            amount_evidence: vec![id_of(index, cited)],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(LETTER, |index| reply(index, "total now due"));
        let description = &outcome.proposal.description;
        assert!(description.contains("$28,703.00"), "{description}");
        assert!(!description.contains("$1,287.40"), "{description}");
        assert_eq!(
            outcome.facts.as_ref().unwrap().support.amount,
            Support::Cited
        );
        let (outcome, _) = facts_for(LETTER, |index| reply(index, "Base rent"));
        let description = &outcome.proposal.description;
        assert!(
            description.contains("base rent of $1,965.00"),
            "{description}"
        );
        // A line with no amount on it gives none, and is no claim either.
        let (outcome, _) = facts_for(LETTER, |index| reply(index, "To: Highmeadow"));
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
}
