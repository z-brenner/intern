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
//! it adds, swaps or misremembers. It is no defense against text crafted
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
    CastMember, DescriptionFacts, Relation, RelationCues, SUBSTANTIVE, amount_label, describe,
    identifier_word, money_in, relation_from_roles,
};
use crate::cues::{CUSTOMER_CUES, ISSUER_CUES, PAYMENT_ADDRESS_CUES, STOPWORDS, TYPE_NOUNS};
use crate::distill::normalize_heading;
use crate::domain::{
    DateRole, DocumentClass, Evidence, EvidenceRef, FactSupport, ModelFacts, ModelProposal,
    ParserWarning, PartyRelation, PartyRole, ProposalStatus, ReviewReason, Support, ValidatedFacts,
    ValidatedParty, ValidatedProposal, ValidationOutcome,
};
use crate::evidence::{
    Segments, contains_whole, date_match_positions, digest_contains, digest_contains_date,
    digest_contains_loosely, digest_reads_date, extract_stated_dates, is_invisible, normalize,
    normalize_loosely, numeric_date_order, numeric_dates, read_date_positions, stated_dates,
};
use crate::index::{EvidenceIndex, EvidenceUnit, UnitKind};
use crate::infer::{
    infer_date_role_as, reads_as_signature_date, reads_as_signature_date_as, repair_issued_relation,
};
use crate::phrases::{
    RIGHT_STOPS, gives_notice, has_a_kind, head_noun, is_form_code, is_placeholder, is_reference,
    label_and_value, names_a_kind, opens_with_party_label, phrase_positions, possessive_side,
    same_word, subject_value, tidy_case, title_phrase, trim_name, without_identifier, words,
};
use crate::retrieve::EvidenceContext;
use crate::validate::{
    DateField, DatePositions, GENERIC_CAPITALS, READY_CONFIDENCE, current_year, date_fields,
    date_is_tainted, date_is_tainted_as, deadline_fires, deadline_fires_as, effective_alternates,
    first_unsupported_claim, issue_date_alternates, push, reading_is_unsettled,
    validate_description, year_is_plausible,
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

/// The one date a letter, a notice or an invoice gives its own date's field
/// on its own pages, with the line it stands on - when `date` has no field
/// anywhere in the document but a line item's ([`date_fields`]). The model
/// need not have been shown it: retrieval can leave a header line out. An
/// exhibit or a schedule attached after the document carries its own
/// "Dated:", and it is never this document's ([`attachments_from`]).
fn own_date_field(scope: &ValidationScope<'_>, date: &str) -> Option<(String, String)> {
    let order = numeric_date_order(scope.document());
    let rows = |units: &[&EvidenceUnit]| {
        let mut rows = Vec::new();
        for unit in units {
            let header = unit
                .table_header
                .filter(|header| *header != unit.ordinal)
                .and_then(|header| scope.unit_at(header))
                .and_then(|header| header.text.lines().next())
                .map(str::to_owned);
            let mut above = header;
            for line in unit.text.lines() {
                rows.push((line.to_owned(), above.take()));
                above = Some(line.to_owned());
            }
        }
        rows
    };
    let units = scope
        .index
        .units()
        .iter()
        .filter(|unit| !unit.running)
        .collect::<Vec<_>>();
    let fields = date_fields(&rows(&units), order);
    if fields
        .iter()
        .any(|field: &DateField| field.date == date && !field.line_item)
    {
        return None;
    }
    // The document's own pages: up to the first page that opens as an
    // attachment. A court notice signs and dates itself at its end, on page
    // two or later; the exhibit or the schedule after it carries dates of
    // its own.
    let first_page = units.iter().map(|unit| unit.page).min()?;
    let attached = attachments_from(&units, first_page);
    let own_pages = units
        .iter()
        .copied()
        .filter(|unit| attached.is_none_or(|page| unit.page < page))
        .collect::<Vec<_>>();
    let mut own: Vec<(String, String)> = Vec::new();
    for field in date_fields(&rows(&own_pages), order) {
        if field.own && !field.line_item && !own.iter().any(|(existing, _)| *existing == field.date)
        {
            own.push((field.date, field.line));
        }
    }
    match own.as_slice() {
        [only] => Some(only.clone()),
        _ => None,
    }
}

/// The first page after the first that opens as an attachment - "EXHIBIT
/// A", "SCHEDULE B", "Attachment 1" - on its own short line, or followed by
/// its title: "EXHIBIT A - VERIFIED AMENDED COMPLAINT". The heading may sit
/// under a court's filing stamp ("Case 2:08-cv-01234 Document 12-2 Filed
/// 09/03/08 Page 1 of 5"), so a page's first few lines are read. A notice's
/// own page that happens to open on "Exhibit A to the complaint ..." only
/// hides its fields from the own-date rule, which then leaves the date
/// alone.
fn attachments_from(units: &[&EvidenceUnit], first_page: usize) -> Option<usize> {
    /// How many of a page's first lines can hold its attachment heading:
    /// the heading, under a filing stamp or a running header.
    const HEADING_LINES: usize = 4;
    let mut pages = units
        .iter()
        .map(|unit| unit.page)
        .filter(|page| *page > first_page)
        .collect::<Vec<_>>();
    pages.dedup();
    pages.into_iter().find(|page| {
        units
            .iter()
            .filter(|unit| unit.page == *page)
            .flat_map(|unit| unit.text.lines())
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(HEADING_LINES)
            .any(|line| {
                let words = words(line);
                words
                    .first()
                    .is_some_and(|word| ATTACHMENT_WORDS.contains(&word.as_str()))
                    && (words.len() <= 4
                        || words.get(1).is_some_and(|word| names_an_attachment(word)))
            })
    })
}

/// Whether a word is what an attachment is called by: a letter, a number or
/// a roman numeral ("A", "12", "IV", "A1").
fn names_an_attachment(word: &str) -> bool {
    word.len() == 1
        || (word.len() <= 3 && word.chars().all(|character| character.is_ascii_digit()))
        || (word.len() <= 5 && word.chars().all(|character| "ivxlc".contains(character)))
        || (word.len() <= 3
            && word.starts_with(|character: char| character.is_ascii_alphabetic())
            && word[1..]
                .chars()
                .all(|character| character.is_ascii_digit()))
}

/// Words a page that starts an attachment opens on.
const ATTACHMENT_WORDS: &[&str] = &[
    "exhibit",
    "schedule",
    "attachment",
    "appendix",
    "annex",
    "enclosure",
];

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

    /// Any unit of the document by its ordinal, shown to the model or not:
    /// a table row's header row, say.
    fn unit_at(&self, ordinal: u32) -> Option<&'a EvidenceUnit> {
        self.index.units().get(ordinal as usize)
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
    let facts =
        without_stray_invisibles_in(candidate.facts.as_deref().cloned().unwrap_or_default());
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
    let mut type_unit = None;
    let proposed_type = content(facts.document_type.as_deref(), scope);
    // A title phrase runs on through "of the ..." to a party's name; a
    // party is no part of a type.
    let names = facts
        .parties
        .iter()
        .map(|party| party.name.as_str())
        .collect::<Vec<_>>();
    let (mut document_type, type_supported) = match proposed_type {
        None => (None, true),
        Some(value) => match read_type(scope, &facts.type_evidence, value) {
            TypeReading::Title { phrase, found } => {
                support.document_type = found.support;
                remember(&found, &mut references);
                type_line = found.line.clone();
                let phrase = stated_title(&without_identifier(&phrase), &names);
                type_title = Some(phrase.clone());
                type_unit = found.unit.map(|unit| unit.ordinal);
                (Some(phrase), true)
            }
            TypeReading::Stated { found } => {
                support.document_type = found.support;
                support.miscited_ids += found.miscited;
                remember(&found, &mut references);
                type_unit = found.unit.map(|unit| unit.ordinal);
                let kept = marks_its_line_holds(value, found.line.as_deref().unwrap_or_default());
                type_line = found.line.clone();
                let kept = stated_title(&without_identifier(&kept), &names);
                (Some(titled(&kept)), true)
            }
            TypeReading::Retitled { phrase, line } => {
                support.document_type = Support::Unsupported;
                type_unit = unit_holding(scope, &line).map(|unit| unit.ordinal);
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
    // A letter's own title stands above its salutation ("WARNING LETTER"
    // over "Dear Mr. Coleman:"). Below it, the letter is talking: a type
    // read from there names what it is about - the order it closes, the
    // statement it encloses - unless it is a kind of letter itself. The
    // name waits for a person.
    let layout = LetterLayout::of(scope);
    // The reply may cite the body's mention of a type the heading above
    // the salutation states too ("NOTICE OF TERMINATION" over "This notice
    // of termination confirms ..."): the heading is the type.
    let talking = |unit: &EvidenceUnit, line: &str, value: &str| {
        layout.below_salutation(unit, line)
            && !words(value).iter().any(|word| word == "letter")
            && !layout.states_above_salutation(scope, value)
    };
    if let Some(unit) = type_unit.and_then(|ordinal| scope.unit_at(ordinal))
        && let Some(value) = document_type.as_deref()
        && talking(unit, type_line.as_deref().unwrap_or_default(), value)
    {
        push(&mut reasons, ReviewReason::TypeUnsupported);
        // In a document laid out as a letter, what the letter talks about
        // is no kind for it: "Letter" stands in, for a person to complete
        // ("I am writing to provide you with advice", "the Order to Show
        // Cause proceeding IS TERMINATED", a question about a "Strategic
        // Plan"). A notice, an invoice or an agreement sent as a letter is
        // still one, its heading under the greeting ("Dear Ms. Roe: /
        // NOTICE OF RENT INCREASE").
        if layout.laid_out_as_letter(scope) && no_kind_of_its_own(Some(value)) {
            document_type = Some("Letter".to_owned());
            type_title = None;
            type_line = None;
            push(&mut reasons, ReviewReason::TypeInferred);
        }
    }
    // A letter's title may say whom it answers and nothing of what it is:
    // "Response to the ONS Engagement on Adjustment of Migration
    // Statistics", the reply's "ONS" among its parties. A party is no part
    // of a type, the kinds after the party's name are the party's papers,
    // and the head left before it ("Response") names no kind of document.
    // "Letter" stands in, for a person to complete, as it does for a
    // letter typed by what it talks about. A head that names a kind
    // ("Notice to ...", "Newsletter of ...") is the type, as above.
    if let Some(value) = document_type.as_deref()
        && no_kind_of_its_own(Some(value))
        && head_before_party(value, &names)
            .is_some_and(|(head, introduced)| introduced && !heads_by_its_own_kind(&head))
        && layout.laid_out_as_letter(scope)
    {
        document_type = Some("Letter".to_owned());
        type_title = None;
        type_line = None;
        push(&mut reasons, ReviewReason::TypeInferred);
    }
    // "Never 'Document', 'Agreement' or 'Letter' alone", as the prompt
    // says: a bare generic word that no title of the document completed
    // (read_type has tried) names no kind a person could file by, so the
    // name waits for a person rather than being filed. Ported from #26.
    // An amendment or an addendum alone says as little: which agreement it
    // changes is the name's point (CUAD's "AMENDMENT NO. 3", which names the
    // agreement it amends only in its first paragraph). So does a notice
    // alone: the IRS's CP161, filed as "Notice from" the taxpayer it is
    // sent to, is one of a hundred kinds the word covers. A response, a
    // notification or a citation alone says as little: what it answers,
    // tells or cites is the name's point ("Response to an Official
    // Information Act Request", "Citation and Notification of Penalty").
    if document_type.as_deref().is_some_and(|value| {
        matches!(
            normalize(value).trim(),
            "letter"
                | "document"
                | "agreement"
                | "amendment"
                | "addendum"
                | "notice"
                | "response"
                | "notification"
                | "citation"
        )
    }) {
        push(&mut reasons, ReviewReason::TypeMissing);
    }
    if document_type.is_none() {
        // A title read below a letter's salutation is the letter talking,
        // as above.
        let inferred = title_of(scope).filter(|(inferred, line)| {
            !(layout.laid_out_as_letter(scope)
                && no_kind_of_its_own(Some(inferred))
                && unit_holding(scope, line).is_some_and(|unit| talking(unit, line, inferred)))
        });
        match inferred {
            Some((inferred, line)) => {
                type_line = Some(line);
                type_title = Some(inferred.clone());
                document_type = Some(inferred);
                push(&mut reasons, ReviewReason::TypeInferred);
            }
            // A document laid out as a letter - a dateline near the top of
            // its first page, a "Dear ..." below it - is a letter, whatever
            // kind the reply could not find in it: "Letter to Target
            // Corporation" tells a person what to check where "Document to
            // Target Corporation" does not. A bare "Letter" names no kind a
            // person could file by, so it still waits for one. So an email
            // by its header block and a notice by its heading stand in for
            // a kind the reply could not find.
            None => match class_stand_in(scope, layout) {
                Some(kind) => {
                    document_type = Some(kind.to_owned());
                    push(&mut reasons, ReviewReason::TypeInferred);
                    push(&mut reasons, ReviewReason::TypeMissing);
                }
                None if type_supported => push(&mut reasons, ReviewReason::TypeMissing),
                None => {}
            },
        }
    }
    // An email's "Subject:" names its matter, as a letter's "Re:" does: the
    // agreement it sends for signature, the guide it attaches. A type read
    // from the header block's subject line says what the message is about,
    // and the message is an email. A notice or an invoice sent as an
    // email is still one - a carrier's "Ship Notification", a system's
    // "Invoice #IN0260" - and keeps the kind its subject gives.
    if type_line
        .as_deref()
        .is_some_and(|line| is_email_subject(scope, line))
        && !matches!(
            DocumentClass::of(document_type.as_deref()),
            DocumentClass::Issued | DocumentClass::Notice | DocumentClass::Email
        )
    {
        document_type = Some("Email".to_owned());
        type_title = None;
        type_line = None;
        push(&mut reasons, ReviewReason::TypeInferred);
    }

    // The date.
    let mut date_line: Option<String> = None;
    let mut date_role = facts.date_role;
    let mut document_date = None;
    let mut date_supported = true;
    let mut replaced = false;
    let mut withheld_as_deadline = false;
    let mut read_loosely: Option<String> = None;
    if let Some(replied) = present(facts.document_date.as_deref()) {
        let found_at = |date: &str, loosely: bool| {
            if !crate::evidence::is_valid_iso_date(date) {
                return Found {
                    support: Support::Unsupported,
                    ..Found::absent()
                };
            }
            let positions = |line: &str| {
                if loosely {
                    read_date_positions(date, line)
                } else {
                    date_match_positions(date, line)
                }
            };
            find(
                scope,
                &facts.date_evidence,
                |view| {
                    if loosely {
                        digest_reads_date(view, date)
                    } else {
                        digest_contains_date(view, date)
                    }
                },
                |unit| line_where(unit, |line| !positions(line).is_empty()),
            )
        };
        let mut found = found_at(replied, false);
        let mut date = replied.to_owned();
        // A date a stamp writes with its day's figures set apart ("SEP 1 8
        // 2015") is read, though less surely than one written out: the
        // name waits for a person. A reply that took the stamp's first
        // figure for the day ("2015-09-01") cited the stamp for a date it
        // does not state; the stamp's own date stands in, when the cited
        // line states that one date only. A receipt's stamp dates a copy's
        // arrival, not the document: it is neither read loosely nor
        // repaired.
        if found.support == Support::Unsupported {
            let loose = found_at(replied, true);
            if loose.support != Support::Unsupported
                && !read_only_on_a_receipt_stamp(scope, loose.unit, replied)
            {
                found = loose;
                read_loosely = Some(date.clone());
            } else if let Some(stamped) = stamp_date_cited(scope, &facts.date_evidence, replied) {
                let repaired = found_at(&stamped, true);
                if repaired.support != Support::Unsupported {
                    found = repaired;
                    date = stamped;
                    read_loosely = Some(date.clone());
                }
            }
        }
        let date = date.as_str();
        support.document_date = found.support;
        support.miscited_ids += found.miscited;
        if found.support == Support::Unsupported {
            date_supported = false;
            date_role = None;
        } else {
            document_date = Some(date.to_owned());
            date_line = found.line.clone();
            remember(&found, &mut references);
            // Another document's date: tainted in either view. A date read
            // only loosely is judged by the statements it was read from.
            let positions = date_positions(read_loosely.as_deref(), date);
            let tainted_context =
                date_is_tainted_as(context, date, document_type.as_deref(), positions);
            let tainted_document =
                date_is_tainted_as(document, date, document_type.as_deref(), positions);
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
    // A deadline: labeled as one in either view, however loosely written.
    // Its replacement is a date written out: a loosely written issue date
    // ("Invoice Date: 02-Jun-17") is never offered on its own, so the
    // deadline is then withheld.
    if !replaced && let Some(date) = document_date.clone() {
        let positions = date_positions(read_loosely.as_deref(), &date);
        let fires_context = deadline_fires_as(context, &date, positions);
        let fires_document = deadline_fires_as(document, &date, positions);
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
    // An email is dated when it was sent: its header block's "Date:" or
    // "Sent:", whatever other date its text or a message it quotes holds
    // ("I need your approval by March 20, 2026").
    if let Some((sent, line)) = email_sent_date(scope)
        && document_date.as_deref() != Some(sent.as_str())
    {
        document_date = Some(sent);
        date_role = Some(DateRole::Issuance);
        date_line = Some(line);
        replaced = true;
    }
    // A letter, a notice or an invoice dates itself in a field: "Dated:",
    // "Date:", "Released:", an "Invoice Date" column. A date it gives no
    // field at all - a violation it recounts, a judgment it appeals - gives
    // way to the one date both views give its own date's field. A date in
    // any field stands (a purchase order's "PO Date"), and so does a
    // letter's dateline, which needs no label.
    if !replaced
        && matches!(
            DocumentClass::of(document_type.as_deref()),
            DocumentClass::Letter | DocumentClass::Notice | DocumentClass::Issued
        )
        && let Some(date) = document_date.clone()
        && !LetterLayout::of(scope).dates_on_dateline(scope, &date)
        && let Some((alternate, line)) = own_date_field(scope, &date)
    {
        let invoice = document_type
            .as_deref()
            .is_some_and(|value| value.to_lowercase().contains("invoice"));
        document_date = Some(alternate);
        date_role = Some(if invoice {
            DateRole::Invoice
        } else {
            DateRole::Issuance
        });
        date_line = Some(line);
        replaced = true;
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
            .then(|| {
                infer_date_role_as(
                    &scope.view(&cited),
                    &date,
                    document_type.as_deref(),
                    date_positions(read_loosely.as_deref(), &date),
                )
            })
            .flatten()
            .or_else(|| {
                infer_date_role_as(
                    context,
                    &date,
                    document_type.as_deref(),
                    date_positions(read_loosely.as_deref(), &date),
                )
            });
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
    // An agreement is made when its last party signs it. A date the
    // document states only in its signature blocks, with a later one in
    // another of them, is an earlier party's signature: "Date: August 18,
    // 2026" under the licensor's name when the licensee signed on August 21
    // and the amendment "is effective on the date of the last signature".
    // It gives way to the latest - unless a stated effective date already
    // replaced it above. Ported from #26, where it moved Bonsai 4B's and
    // Qwen3.5-2B's trap date on InternBench's mixed-signature-region.
    //
    // Only a signature's date gives way, and only to another signature's.
    // The signature region is positional - it runs on for a dozen units
    // after a signature cue - so an "Effective Date: August 18" set beside
    // the signatures is the document's own date and stands, and a later
    // "Expiration Date" there is never taken: each date must read as signed
    // on, and a replacement must pass the deadline and other-document
    // guards the model's own date passed above.
    if !replaced
        && matches!(
            DocumentClass::of(document_type.as_deref()),
            DocumentClass::Agreement | DocumentClass::Amendment
        )
        && let Some(date) = document_date.clone()
        && only_in_signatures_as(
            scope.index,
            &date,
            date_positions(read_loosely.as_deref(), &date),
        )
        && reads_as_signature_date_as(
            document,
            &date,
            date_positions(read_loosely.as_deref(), &date),
        )
        && let Some((latest, line)) = latest_signature_date(scope, &date, |candidate| {
            reads_as_signature_date(document, candidate)
                && !deadline_fires(context, candidate)
                && !deadline_fires(document, candidate)
                && !date_is_tainted(context, candidate, document_type.as_deref())
                && !date_is_tainted(document, candidate, document_type.as_deref())
        })
    {
        document_date = Some(latest);
        date_role = Some(DateRole::Execution);
        date_line = Some(line);
        replaced = true;
    }
    if document_date.is_none() && date_supported && !withheld_as_deadline {
        push(&mut reasons, ReviewReason::DateMissing);
    }
    // A date read only loosely is not written out in the document as a
    // date is: the name keeps it, and a person confirms it.
    if read_loosely.is_some() && document_date == read_loosely {
        push(&mut reasons, ReviewReason::DateUnsupported);
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
            .then(|| {
                infer_date_role_as(
                    &scope.view(&cited),
                    date,
                    document_type.as_deref(),
                    date_positions(read_loosely.as_deref(), date),
                )
            })
            .flatten()
            .or_else(|| {
                infer_date_role_as(
                    context,
                    date,
                    document_type.as_deref(),
                    date_positions(read_loosely.as_deref(), date),
                )
            })
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
        let trimmed = without_trading_name(&trim_name(written));
        let trimmed = completed_name(scope, &trimmed).unwrap_or(trimmed);
        let trimmed = without_courtesy_title(&trimmed);
        let name = trimmed.as_str();
        // A first name on its own is never a party: one word, not an
        // initialism, with nothing that makes it an organization.
        if is_first_name_alone(name) {
            support.parties.push(Support::Unsupported);
            parties_supported = false;
            continue;
        }
        // A redaction marks where a name was taken out of a published
        // copy ("Dear [REDACTED]"): it names no one.
        if is_redaction(name) {
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
            .then(|| labeled_role(scope, name))
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
    // A person who signs for an organization among the parties acts for it
    // and is no party of their own.
    let organizations = parties
        .iter()
        .filter(|party| is_organization_name(&party.name))
        .map(|party| normalize_loosely(&party.name))
        .collect::<Vec<_>>();
    for party in &mut parties {
        if !party.copied && signs_for(scope, &party.name, &organizations) {
            party.signatory = true;
            party.role = Some(PartyRole::Other);
        }
    }
    let class = DocumentClass::of(document_type.as_deref());
    // A record or a notice is kept for the side it is about, not sent to
    // it: a reply's bare "recipient" gives way to that side when the
    // document's own label names it so - "Prepared for Cobalt Basin Water
    // Authority", "Named insured", "Tenant:".
    if matches!(class, DocumentClass::Record | DocumentClass::Notice) {
        for party in &mut parties {
            if matches!(
                party.role,
                Some(PartyRole::Recipient | PartyRole::Addressee)
            ) && let Some(labeled) = labeled_role(scope, &party.name)
                && SUBSTANTIVE.contains(&labeled)
            {
                party.role = Some(labeled);
                party.document_role = Some(labeled);
            }
        }
    }
    // An email's header block says who sent it and to whom, in the
    // domains of its "From:" and "To:" addresses: "@ups.com" to
    // "@enron.com", "@dgs.ca.gov" to "@nfpa.org". A party whose name one
    // side's domain spells, and the other's does not, is on that side,
    // whatever the reply called it: the roles an email's body gives its
    // parties - the "Contractor:" of the contract it attaches, the company
    // a shipment was sent "at the request of" - are the matter's, not the
    // email's.
    if class == DocumentClass::Email
        && let Some(sides) = EmailSides::of(scope)
    {
        for party in &mut parties {
            if party.copied {
                continue;
            }
            let Some((role, unit, line)) = sides.side_of(&party.name) else {
                continue;
            };
            remember(
                &Found {
                    support: Support::Context,
                    unit: Some(unit),
                    line: Some(line.to_owned()),
                    miscited: 0,
                },
                &mut references,
            );
            party.role = Some(role);
            party.role_support = Support::Context;
            party.document_role = Some(role);
            if !party.evidence.contains(&unit.id) {
                party.evidence.push(unit.id.clone());
            }
        }
    }
    // So does a person the document names only by the post they hold for
    // it - "Beckett Halloran (President)", "Winemaker | Esme Varga",
    // "Examined by | Delphine Moncrieff, Associate Conservator", "/s/ Arlo
    // Sinclair-Ray" - unless a line names them as the one it is about or a
    // "between" clause makes them a party.
    let standing = parties
        .iter()
        .map(|party| {
            if party.copied || party.signatory || is_organization_name(&party.name) {
                Standing::Unknown
            } else if labeled_party(scope, &party.name, class) {
                // A label that gives them a party's role ("Recipient:",
                // "Tenant:") outranks a signature or a post, and keeps the
                // others' staff from taking them along.
                Standing::Subject
            } else {
                standing_of(scope, &party.name, class)
            }
        })
        .collect::<Vec<_>>();
    let mut acting_for = false;
    for (party, standing) in parties.iter_mut().zip(&standing) {
        if *standing == Standing::Staff {
            party.signatory = true;
            party.role = Some(PartyRole::Other);
            acting_for = true;
        }
    }
    // When the reply named, besides such people, no organization and no one
    // a line names as the one the document is about, the others it named
    // are staff too (the other signatory, the crew in a log's rows), and the
    // organization they act for is the party: the two a "between" clause
    // names for an agreement, else the one at the head of the first page.
    // An issued document gets its header issuer below.
    let left = parties
        .iter()
        .zip(&standing)
        .filter(|(party, _)| !party.copied && !party.signatory)
        .collect::<Vec<_>>();
    let only_people = left.iter().all(|(party, standing)| {
        !is_organization_name(&party.name) && **standing != Standing::Subject
    });
    if acting_for && only_people {
        let found = match class {
            DocumentClass::Issued => Vec::new(),
            DocumentClass::Agreement | DocumentClass::Amendment => between_organizations(scope),
            _ => header_organization(scope)
                .or_else(|| letterhead_organization(scope))
                .into_iter()
                .collect(),
        };
        if class == DocumentClass::Issued || !found.is_empty() {
            for party in parties
                .iter_mut()
                .filter(|party| !party.copied && !party.signatory)
            {
                party.signatory = true;
                party.role = Some(PartyRole::Other);
            }
        }
        for (at, (name, unit)) in found.into_iter().enumerate() {
            let name = without_sentence_stop(&with_of_phrase(unit, &name));
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
                at,
                ValidatedParty {
                    name: display_name(&name),
                    support: Support::Context,
                    evidence: vec![unit.id.clone()],
                    ..ValidatedParty::default()
                },
            );
        }
    }
    // A record whose staff act for the organization at the head of its
    // first page comes from that organization, though the reply named
    // another besides them: a museum's loan condition report, its
    // conservator the examiner, about a lender's painting.
    if acting_for
        && !only_people
        && class == DocumentClass::Record
        && let Some((name, unit)) =
            header_organization(scope).or_else(|| letterhead_organization(scope))
    {
        let name = without_sentence_stop(&with_of_phrase(unit, &name));
        if !parties
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
    }
    // An issued document whose reply said what it is but named no one
    // who could have issued it - no one, or only parties a customer cue
    // labels - takes as its issuer the one organization at the head of its
    // first page, before any customer. The type may be the one its title
    // gave when the reply's own was not in the document.
    let customer_only = parties.iter().all(|party| {
        party.copied
            || party.signatory
            || labeled_role(scope, &party.name) == Some(PartyRole::Customer)
            || party
                .role
                .is_some_and(|role| RECEIVING_ROLES.contains(&role))
    });
    if class == DocumentClass::Issued
        && proposed_type.is_some()
        && document_type.is_some()
        && customer_only
        && let Some((name, unit)) = header_organization(scope)
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
                || labeled_role(scope, &party.name)
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
    // A letter addressed to a person at an organization is to the
    // organization: its inside address names the person, their title, then
    // the organization they receive it for ("Mr. Jeffery Coleman / Refinery
    // Manager / Toledo Refining Company, LLC"). A reply that named only the
    // person as the addressee gets the organization in their place. A
    // document of no known class laid out as a letter - an agency's
    // "Advisory Opinion" opening "Dear Mr. O'Kelly:" - is read the same way.
    if matches!(
        class,
        DocumentClass::Letter | DocumentClass::Notice | DocumentClass::Unknown
    ) {
        let layout = LetterLayout::of(scope);
        for index in 0..parties.len() {
            // A person the reply called the recipient may already stand as
            // the organization's signatory, their name and post over its
            // name read as a signature block: the organization is then a
            // party already, and only its role is settled here.
            // A letter's inside address names who the letter is to,
            // whatever role the reply gave them - a "client" it warns, a
            // "licensee" it approves - unless the reply made them its
            // sender. A notice's goes by the reply's addressee.
            let party = &parties[index];
            let addressed = if class == DocumentClass::Notice {
                party
                    .proposed_role
                    .is_some_and(|role| ADDRESSED_ROLES.contains(&role))
            } else {
                !party
                    .proposed_role
                    .is_some_and(|role| matches!(role, PartyRole::Sender | PartyRole::Issuer))
            };
            if party.copied || !addressed {
                continue;
            }
            // An organization the reply named, which the letter's inside
            // address names, receives the letter, whatever its support for
            // the reply's role: "Robert L. Sacks / Kane Kessler, P.C." over
            // "Dear Mr. Sacks" is to Kane Kessler, not from the agency whose
            // letterhead heads the page. One addressee is enough.
            if is_organization_name(&party.name) {
                if class == DocumentClass::Letter
                    && party.role != Some(PartyRole::Addressee)
                    && !party
                        .document_role
                        .is_some_and(|role| matches!(role, PartyRole::Sender | PartyRole::Issuer))
                    && layout.addresses(scope, &party.name)
                    && !parties.iter().any(|other| {
                        other.role == Some(PartyRole::Addressee) && other.name != party.name
                    })
                {
                    let party = &mut parties[index];
                    party.signatory = false;
                    party.role = Some(PartyRole::Addressee);
                    party.role_support = Support::Context;
                    party.document_role = Some(PartyRole::Addressee);
                    break;
                }
                continue;
            }
            let named = parties
                .iter()
                .filter(|other| !other.copied && other.name != party.name)
                .map(|other| other.name.clone())
                .collect::<Vec<_>>();
            let Some((name, unit)) = layout.addressee_organization(scope, &party.name, &named)
            else {
                // The letter is to the first person its inside address
                // names; when no organization stands under them, it is to
                // them, and a second addressee's organization does not
                // take their place.
                if layout.in_inside_address(scope, &party.name) {
                    break;
                }
                continue;
            };
            if let Some(kept) = parties
                .iter()
                .position(|kept| normalize_loosely(&kept.name) == normalize_loosely(&name))
            {
                // The reply named the organization too, as the letter's
                // sender: the inside address says it receives the letter.
                // Any other role - the party a notice is about - stands, and
                // so does a role the document gives it ("Tenant: ..."): the
                // reply's own role counts only when validation kept none.
                let sending = parties[kept]
                    .role
                    .or(parties[kept].proposed_role)
                    .is_some_and(|role| matches!(role, PartyRole::Sender | PartyRole::Issuer));
                if sending && !parties[kept].copied {
                    remember(
                        &Found {
                            support: Support::Context,
                            unit: Some(unit),
                            line: Some(name.clone()),
                            miscited: 0,
                        },
                        &mut references,
                    );
                    parties[index].signatory = true;
                    parties[index].role = Some(PartyRole::Other);
                    let kept = &mut parties[kept];
                    kept.role = Some(PartyRole::Addressee);
                    kept.role_support = Support::Context;
                    kept.document_role = Some(PartyRole::Addressee);
                    if !kept.evidence.contains(&unit.id) {
                        kept.evidence.push(unit.id.clone());
                    }
                    break;
                }
                continue;
            }
            if party.signatory {
                continue;
            }
            remember(
                &Found {
                    support: Support::Context,
                    unit: Some(unit),
                    line: Some(name.clone()),
                    miscited: 0,
                },
                &mut references,
            );
            parties[index].signatory = true;
            parties[index].role = Some(PartyRole::Other);
            parties.insert(
                index,
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
            break;
        }
    }
    // A letter signed by a person for an organization is from the
    // organization: its signature block names the person, their post, then
    // the organization they sign for ("Rashmi Panigrahi / FOI Officer /
    // Office of Manpower Economics"), or its letterhead names the
    // organization alone ("BUILDING STANDARDS COMMISSION"). A reply that
    // named only the person as the sender gets the organization in their
    // place; one that named the organization too has it confirmed as the
    // sender.
    let layout = LetterLayout::of(scope);
    if class == DocumentClass::Letter
        || (class == DocumentClass::Unknown && layout.laid_out_as_letter(scope))
    {
        for index in 0..parties.len() {
            let party = &parties[index];
            if party.copied
                || party.signatory
                || party.document_role.is_some()
                || matches!(
                    party.role,
                    Some(PartyRole::Addressee | PartyRole::Recipient)
                )
                || is_organization_name(&party.name)
                || named_as_an_organization(scope, &party.name)
                || !named_as_a_person(scope, &party.name)
                || !party
                    .proposed_role
                    .is_some_and(|role| matches!(role, PartyRole::Sender | PartyRole::Issuer))
            {
                continue;
            }
            let addressees = parties
                .iter()
                .filter(|other| {
                    !other.copied
                        && !other.proposed_role.is_some_and(|role| {
                            matches!(role, PartyRole::Sender | PartyRole::Issuer)
                        })
                })
                .map(|other| other.name.clone())
                .collect::<Vec<_>>();
            let found = signer_organization(scope, &party.name)
                .or_else(|| letterhead_name(scope, layout, &addressees));
            let Some((name, unit)) = found else {
                continue;
            };
            // A letterhead that holds the party's own name, or part of it,
            // names the party, not another.
            let own = normalize_loosely(&party.name);
            let found_loose = normalize_loosely(&name);
            if own.contains(&found_loose) || found_loose.contains(&own) {
                continue;
            }
            let found = Found {
                support: Support::Context,
                unit: Some(unit),
                line: Some(name.clone()),
                miscited: 0,
            };
            if let Some(kept) = parties
                .iter()
                .position(|kept| normalize_loosely(&kept.name) == normalize_loosely(&name))
            {
                if kept == index || parties[kept].copied {
                    continue;
                }
                remember(&found, &mut references);
                parties[index].signatory = true;
                parties[index].role = Some(PartyRole::Other);
                let kept = &mut parties[kept];
                if kept.role.is_none() {
                    kept.role = Some(PartyRole::Sender);
                    kept.role_support = Support::Context;
                    kept.document_role = Some(PartyRole::Sender);
                }
                if !kept.evidence.contains(&unit.id) {
                    kept.evidence.push(unit.id.clone());
                }
                break;
            }
            remember(&found, &mut references);
            parties[index].signatory = true;
            parties[index].role = Some(PartyRole::Other);
            parties.insert(
                index,
                ValidatedParty {
                    name: display_name(&name),
                    role: Some(PartyRole::Sender),
                    role_support: Support::Context,
                    document_role: Some(PartyRole::Sender),
                    support: Support::Context,
                    evidence: vec![unit.id.clone()],
                    ..ValidatedParty::default()
                },
            );
            break;
        }
    }
    // An agreement names its two sides in its opening "between" clause
    // ("This Agreement is made ... by and between"). When the reply kept
    // only one side - the other written as the document never writes it,
    // or left out - and that side is one of the two the clause names, the
    // clause's other organization is the other side. A recital's
    // "between" names another agreement's sides, and with no side left
    // nothing ties a clause to this agreement: neither is used.
    if matches!(class, DocumentClass::Agreement | DocumentClass::Amendment)
        && let [only] = parties
            .iter()
            .filter(|party| !party.copied && !party.signatory)
            .collect::<Vec<_>>()
            .as_slice()
        && let [first, second] = opening_clause_organizations(scope).as_slice()
    {
        let only = normalize_loosely(&only.name);
        let other = if normalize_loosely(&first.0) == only {
            Some((second, parties.len()))
        } else if normalize_loosely(&second.0) == only {
            Some((first, 0))
        } else {
            None
        };
        if let Some(((name, unit), at)) = other {
            let name = without_sentence_stop(name);
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
                at,
                ValidatedParty {
                    name: display_name(&name),
                    support: Support::Context,
                    evidence: vec![unit.id.clone()],
                    ..ValidatedParty::default()
                },
            );
        }
    }
    // A court document names the two sides of its case in its caption.
    // When the reply kept one side only - the other left out, or written
    // as the caption never writes it - the caption's other side is the
    // other party: the publishers who brought the case a notice of
    // subpoena is filed in, the defendants a notice of appeal is taken by.
    if class == DocumentClass::Notice
        && let Some((name, unit, bringing)) = caption_other_side(scope, &parties)
        && !parties
            .iter()
            .any(|party| normalize_loosely(&party.name) == normalize_loosely(&name))
    {
        remember(
            &Found {
                support: Support::Context,
                unit: Some(unit),
                line: Some(name.clone()),
                miscited: 0,
            },
            &mut references,
        );
        let at = if bringing { 0 } else { parties.len() };
        parties.insert(
            at,
            ValidatedParty {
                name: display_name(&name),
                support: Support::Context,
                evidence: vec![unit.id.clone()],
                ..ValidatedParty::default()
            },
        );
    }
    // A notice filed in a bankruptcy case is filed in its debtor's case:
    // "In re: Delta Mills, Inc., et al., Debtors." The reply names who
    // gives the notice and who it goes to - the creditors' committee that
    // appears, the creditors that move, the trustee that appoints - and
    // may leave the debtor out: the caption names it.
    let debtor = (class == DocumentClass::Notice)
        .then(|| caption_debtor(scope, &parties))
        .flatten();
    if let Some((name, unit)) = &debtor
        && !parties
            .iter()
            .any(|party| normalize_loosely(&party.name) == normalize_loosely(name))
    {
        remember(
            &Found {
                support: Support::Context,
                unit: Some(unit),
                line: Some(name.clone()),
                miscited: 0,
            },
            &mut references,
        );
        parties.insert(
            0,
            ValidatedParty {
                name: display_name(name),
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
            proposed_role: party.proposed_role,
        })
        .collect::<Vec<_>>();
    let cues = RelationCues {
        // A cue is the document's own word - "Remit To", "Bill To" - and
        // stands against the reply's reading. Where a party stands on the
        // page is not, and makes no issuer of the party a reply that read
        // both sides took for the customer.
        issuer: cue_issuer(document_type.as_deref(), &cast, context).or_else(|| {
            (class == DocumentClass::Issued)
                .then(|| header_issuer(scope, &cast))
                .flatten()
                .filter(|&head| !read_as_billed_by_another(&cast, head))
        }),
        between: between_cue(context, &cast),
        caption: caption_sides(scope, context, &cast),
        debtor: debtor.and_then(|(name, _)| {
            let name = normalize_loosely(&display_name(&name));
            cast.iter()
                .position(|party| normalize_loosely(&party.name) == name)
        }),
    };
    let relation = relation_from_roles(class, document_type.as_deref(), &cast, cues);
    // A name that rests on roles the document did not support waits for
    // a person: the reply read the sides, the document did not say.
    if relation.on_proposed_roles {
        push(&mut reasons, ReviewReason::PartyUnsupported);
    }
    // An agreement has two sides at least. One that names only one has
    // lost the other, which a person must supply.
    if matches!(class, DocumentClass::Agreement | DocumentClass::Amendment)
        && relation.relation == PartyRelation::With
    {
        push(&mut reasons, ReviewReason::PartyMissing);
    }
    // An invoice, a receipt or an order comes from its issuer. One left
    // naming no one - the reply named only its customer, or nobody - has
    // lost the party its name is for, which a person must supply.
    if class == DocumentClass::Issued && relation.parties.is_empty() {
        push(&mut reasons, ReviewReason::PartyMissing);
    }

    // The subject, the identifier and the key facts: every number and name
    // in them must be in what the model was shown, and the subject and a
    // key fact must pass the word check ([`every_word_stated`]).
    let stated_words = StatedWords::of(scope);
    let mut subject = None;
    if let Some(value) = content(facts.subject.as_deref(), scope) {
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
        let Some(value) = content(Some(fact.fact.as_str()), scope) else {
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
    // number on the type's own title line, or a field labeled as this
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

/// How the guards find the statements of the date validation settled on:
/// as [`date_match_positions`] finds them, or, when that date was read only
/// loosely (`read_loosely` holds it), as [`read_date_positions`] reads them,
/// so the guards judge the statements the date was read from.
fn date_positions(read_loosely: Option<&str>, date: &str) -> DatePositions {
    if read_loosely == Some(date) {
        read_date_positions
    } else {
        date_match_positions
    }
}

/// The date of a rubber stamp a reply cited for `replied` and misread: the
/// cited line states one date only, a stamp's with its day's figures set
/// apart, in `replied`'s month and year, and `replied`'s day is the stamp
/// day's first figure ("SEP 1 8 2015" read as 2015-09-01). A receipt's
/// stamp ([`under_a_receipt_label`]) repairs nothing.
fn stamp_date_cited(
    scope: &ValidationScope<'_>,
    cited: &[String],
    replied: &str,
) -> Option<String> {
    let (year_month, day) = replied.rsplit_once('-')?;
    let day = day.parse::<u32>().ok()?;
    scope.cited_units(cited).into_iter().find_map(|unit| {
        unit.text.lines().enumerate().find_map(|(at, line)| {
            let stamps = crate::evidence::stamped_dates(line);
            let [(stamped, first)] = stamps.as_slice() else {
                return None;
            };
            (!under_a_receipt_label(scope, unit, at)
                && extract_stated_dates(line)
                    .iter()
                    .all(|other| other == stamped)
                && stamped.starts_with(&format!("{year_month}-"))
                && *first == day)
                .then(|| stamped.clone())
        })
    })
}

/// Whether the line at `at` of `unit` stands under a receipt's label: a
/// "RECEIVED" or "FILED" on the line itself or on the line right over it
/// (the last line of the unit before, where the index set the label apart
/// as a heading). A stamp there dates a copy's arrival, not the document.
fn under_a_receipt_label(scope: &ValidationScope<'_>, unit: &EvidenceUnit, at: usize) -> bool {
    let labels_receipt = |line: &str| {
        words(line)
            .iter()
            .any(|word| matches!(word.as_str(), "received" | "recd" | "filed"))
    };
    let lines = unit.text.lines().collect::<Vec<_>>();
    let Some(line) = lines.get(at) else {
        return false;
    };
    let above = match at.checked_sub(1) {
        Some(before) => lines.get(before).copied(),
        None => scope
            .index
            .units()
            .iter()
            .filter(|other| other.page == unit.page && other.ordinal < unit.ordinal)
            .max_by_key(|other| other.ordinal)
            .and_then(|other| other.text.lines().last()),
    };
    labels_receipt(line) || above.is_some_and(labels_receipt)
}

/// Whether the unit a loose reading of `date` was found in reads it only
/// under receipts' labels: with every line under a "RECEIVED" or "FILED"
/// ([`under_a_receipt_label`]) taken out, together with the lines a date
/// begun on it wraps onto ("RECEIVED November" over "22,1923"), the rest
/// of the unit no longer states `date` as [`read_date_positions`] reads it,
/// whatever loose form it takes (a stamp's figures apart,
/// "November 22,1923", "02-Jun-17"). The unit is read whole, as the
/// digest reads it, so a date wrapped across lines is found as one.
fn read_only_on_a_receipt_stamp(
    scope: &ValidationScope<'_>,
    unit: Option<&EvidenceUnit>,
    date: &str,
) -> bool {
    let Some(unit) = unit else {
        return false;
    };
    let reads = |text: &str| !read_date_positions(date, &normalize(text)).is_empty();
    if !reads(&unit.text) {
        return false;
    }
    let lines = unit.text.lines().collect::<Vec<_>>();
    let mut rest = Vec::with_capacity(lines.len());
    let mut at = 0;
    while at < lines.len() {
        if !under_a_receipt_label(scope, unit, at) {
            rest.push(lines[at]);
            at += 1;
            continue;
        }
        // A date the labelled line begins may end one or two lines on: the
        // lines after it are taken with it only when the date needs it, so
        // a dateline of its own below a stamp ("RECEIVED" / "JAN 3 1924" /
        // "November 22,1923") stays.
        let wrapped = (!reads(lines[at]))
            .then(|| {
                (2..=3).find(|span| {
                    at + span <= lines.len()
                        && reads(&lines[at..at + span].join(" "))
                        && !reads(&lines[at + 1..at + span].join(" "))
                })
            })
            .flatten();
        at += wrapped.unwrap_or(1);
    }
    !reads(&rest.join("\n"))
}

/// Whether a type names no class a letter could carry of its own - a
/// notice, an invoice, an agreement, a form, an email - only a record or
/// no known kind: what a letter may talk about.
fn no_kind_of_its_own(document_type: Option<&str>) -> bool {
    matches!(
        DocumentClass::of(document_type),
        DocumentClass::Unknown | DocumentClass::Record
    )
}

/// The unit of the context that holds `line` as one of its lines.
fn unit_holding<'a>(scope: &ValidationScope<'a>, line: &str) -> Option<&'a EvidenceUnit> {
    let line = line.trim();
    scope
        .context_units()
        .find(|unit| unit.text.lines().any(|held| held.trim() == line))
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
/// title wrapped across them. A heading that is only the kind of document -
/// a cover page's "AGREEMENT" under "JOINT DEVELOPMENT AND MARKETING" - is
/// read first with the heading just above it, when that is written in the
/// same capitals: the title wrapped across two headings.
fn title_lines(scope: &ValidationScope<'_>, unit: &EvidenceUnit) -> Vec<String> {
    if unit.kind != UnitKind::Heading {
        return unit.text.lines().map(str::to_owned).collect();
    }
    let joined = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let heading = joined(&unit.text);
    let capitals =
        |text: &str| text.chars().any(char::is_alphabetic) && !text.chars().any(char::is_lowercase);
    let above = unit
        .ordinal
        .checked_sub(1)
        .and_then(|ordinal| scope.index.units().get(ordinal as usize))
        .filter(|above| {
            above.kind == UnitKind::Heading
                && above.page == unit.page
                && !above.running
                && capitals(&above.text)
                && !above.text.trim_end().ends_with([':', '.'])
                // A letterhead's "WRENFIELD SUPPLY" over "AGREEMENT" is a
                // company, not the first half of the title.
                && !names_an_organization(above)
        });
    match above {
        Some(above)
            if has_a_kind(&words(&heading)) && words(&heading).len() == 1 && capitals(&heading) =>
        {
            vec![format!("{} {heading}", joined(&above.text)), heading]
        }
        _ => vec![heading],
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
    // Declarations". So it is when the line the reply cited is a heading
    // that is the bare kind itself - a later page's "AGREEMENT" under a
    // cover page's "Strategic Alliance Agreement".
    let value_words = words(value);
    let bare = match &reading {
        TypeReading::Stated { .. } => true,
        TypeReading::Title { phrase, .. } => words(phrase).len() == 1,
        _ => false,
    };
    if let [head] = value_words.as_slice()
        && bare
        && let Some((phrase, unit, line)) = title_units(scope).find_map(|unit| {
            title_lines(scope, unit).into_iter().find_map(|line| {
                let phrase = title_phrase(&line, head)?;
                // Never another document's name: "Pursuant to Master
                // Services Agreement" names the agreement this one is under.
                let line_words = words(&line);
                let phrase_words = words(&phrase);
                let names_this = phrase_positions(&line_words, &phrase_words)
                    .into_iter()
                    .any(|at| names_document_at(&line_words, at, at + phrase_words.len(), &line));
                (title_like(unit, &line) && phrase_words.len() > 1 && names_this)
                    .then(|| (phrase, unit, line.trim().to_owned()))
            })
        })
    {
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
    // An email says what it is by its header block, never in words: the
    // reply's "Email" is the document's own when the first page opens
    // with one.
    if matches!(
        normalize(value).trim(),
        "email" | "e-mail" | "e mail" | "email message"
    ) && let Some((unit, line)) = email_header(scope)
    {
        let support = if cited.contains(&unit.id) {
            Support::Cited
        } else {
            Support::Context
        };
        return TypeReading::Stated {
            found: Found {
                support,
                unit: Some(unit),
                line: Some(line),
                miscited: 0,
            },
        };
    }
    let value_words = words(value);
    // "Report generated", "Printed on": the stamp a system puts on a sheet
    // says when it was made, not what it is. The title above it does.
    if value_words.last().is_some_and(|word| {
        matches!(
            word.as_str(),
            "generated" | "printed" | "exported" | "created" | "saved"
        )
    }) {
        return TypeReading::Unsupported;
    }
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
        for line in title_lines(scope, unit) {
            if !title_like(unit, &line) {
                continue;
            }
            let Some(phrase) = title_on(&line) else {
                continue;
            };
            let holds_reply = !phrase_positions(&words(&phrase), &value_words).is_empty();
            if holds_reply || !stated_on(&line) || reads_past_title(value, &value_words, &phrase) {
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
    // A bigger model copies a title and runs on past its line - into the
    // next line of a wrapped heading, into the paragraph under it - until
    // the reply's room for a type runs out: "Final enforcement notice to
    // Euro Car Parks Limited for failure to comply with an", "Notice of
    // Technical Conference On May 10, 2024, the Commission issued". The
    // part of the reply its line holds, read as a title, is the type.
    if kind && value_words.len() >= 3 {
        let past_line = |unit: &'a EvidenceUnit, support: Support| {
            let mut lines = title_lines(scope, unit);
            if unit.kind != UnitKind::Heading && unit.text.lines().count() > 1 {
                lines.push(unit.text.split_whitespace().collect::<Vec<_>>().join(" "));
            }
            // What the document says after the line: the rest of the unit,
            // then the next unit. The reply ran on into it, so its tail
            // is found there; a tail found nowhere is no copy of the
            // document ("OFFICIAL PUBLIC RECORD,p1.b1],").
            let following = |line: &str| {
                let unit_words = words(&unit.text);
                let line_words = words(line);
                let mut rest = phrase_positions(&unit_words, &line_words)
                    .first()
                    .map(|at| unit_words[at + line_words.len()..].to_vec())
                    .unwrap_or_default();
                if let Some(next) = scope.index.units().get(unit.ordinal as usize + 1) {
                    rest.extend(words(&next.text));
                }
                rest
            };
            lines.into_iter().find_map(|line| {
                let (kept, tail) = held_by_line(value, &value_words, &line)?;
                // The tail's first words are the document's next words; a
                // reply's last word or two may be the id it ran into.
                let tail_head = &tail[..tail.len().min(2)];
                if !tail.is_empty() && !following(&line).starts_with(tail_head) {
                    return None;
                }
                // The part the line holds is a title by its own kind. The
                // whole reply's head noun may lie in what it ran on into:
                // "Public Information Statement National Weather Service
                // Wichita KS 710 PM CDT Mon" names a kind followed by an
                // office, and ends on a weekday.
                let kept_words = words(&kept);
                let phrase = title_phrase(&kept, head_noun(&kept_words)?)?;
                let line_words = words(&line);
                let phrase_words = words(&phrase);
                (phrase_words.len() >= 2
                    && phrase_positions(&line_words, &phrase_words)
                        .into_iter()
                        .any(|at| {
                            names_document_at(&line_words, at, at + phrase_words.len(), &line)
                        }))
                .then(|| TypeReading::Title {
                    phrase,
                    found: found_in(unit, &line, support),
                })
            })
        };
        if let Some(reading) = cited_units
            .iter()
            .find_map(|unit| past_line(unit, Support::Cited))
            .or_else(|| title_units(scope).find_map(|unit| past_line(unit, Support::Context)))
        {
            return reading;
        }
    }
    // A title of the same kind stands in, for a person to confirm. A line
    // that runs on from the kind into a sentence is the body talking about
    // a document, not a title: "The report on the works is provided
    // below:" titles no email.
    for unit in title_units(scope) {
        for line in title_lines(scope, unit) {
            if title_like(unit, &line)
                && let Some(phrase) = title_on(&line)
                && !runs_into_a_sentence(&line, &phrase)
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

/// Whether a title phrase read from `line` runs on into a sentence there
/// ([`crate::phrases::begins_a_sentence`]), wherever the line states it.
fn runs_into_a_sentence(line: &str, phrase: &str) -> bool {
    let line_words = words(line);
    let phrase_words = words(phrase);
    let positions = phrase_positions(&line_words, &phrase_words);
    !positions.is_empty()
        && positions
            .into_iter()
            .all(|at| crate::phrases::begins_a_sentence(&line_words, at + phrase_words.len()))
}

/// Whether a reply's type is a title phrase with only what a title stops
/// at around it: a side's possessive before it ("PLAINTIFF'S Notice of
/// Voluntary Dismissal"), or the sentence it opens after it ("NOTICE is
/// hereby given"). The title phrase is then the type. Only a sentence that
/// gives the notice "hereby" opens on a title: a body's sentence that opens
/// on a kind ("Receipt is acknowledged of the Letter of Notification",
/// "Invoice is attached for services rendered") is no heading, and its
/// first word is no type.
fn reads_past_title(value: &str, value_words: &[String], phrase: &str) -> bool {
    let phrase_words = words(phrase);
    let Some(at) = phrase_positions(value_words, &phrase_words)
        .first()
        .copied()
    else {
        return false;
    };
    let before = &value_words[..at];
    let after = &value_words[at + phrase_words.len()..];
    let possessive = value
        .split_whitespace()
        .next()
        .is_some_and(|token| possessive_side(token) && words(token) == before);
    (before.is_empty() || possessive)
        && (after.is_empty() || gives_notice(after, 0))
        && !(before.is_empty() && after.is_empty())
}

/// The head of a reply's type that `line` holds when the reply ran on
/// past the line, and the reply's tail beyond it: the reply's words from
/// its start to where the line ends (the line ends with them) or the whole
/// reply when the line runs on past it; at least two words, fewer than the
/// reply's when the line ends them, and one of them a kind of document.
/// `None` when the line does not hold the reply's head that way.
fn held_by_line<'v>(
    value: &str,
    value_words: &'v [String],
    line: &str,
) -> Option<(String, &'v [String])> {
    let line_words = words(line);
    if line_words.len() < 2 {
        return None;
    }
    // The reply's words as written, one entry per word of `value_words`.
    let written = value
        .split_whitespace()
        .flat_map(|token| {
            let count = words(token).len();
            std::iter::repeat_n(token, count.min(1))
                .chain(std::iter::repeat_n("", count.saturating_sub(1)))
        })
        .collect::<Vec<_>>();
    let kept_words = if line_words.len() > value_words.len() {
        // The line runs on past the reply: the reply is its head.
        (line_words.starts_with(value_words)).then_some(value_words.len())?
    } else {
        // The reply runs on past the line: the line ends with its head.
        (2..value_words.len())
            .rev()
            .find(|k| line_words.ends_with(&value_words[..*k]))?
    };
    if !has_a_kind(&value_words[..kept_words]) {
        return None;
    }
    let kept = written
        .iter()
        .take(kept_words)
        .filter(|token| !token.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    Some((
        marks_its_line_holds(&kept, line),
        &value_words[kept_words..],
    ))
}

/// The date an email's header block ([`email_header_block`]) says it was
/// sent ("Date:" or "Sent:"), and that line: `None` without a header block,
/// or without exactly one date on the line.
fn email_sent_date(scope: &ValidationScope<'_>) -> Option<(String, String)> {
    email_header_block(scope)?
        .into_iter()
        .find_map(|(_, line)| {
            let (label, value) = line.split_once(':')?;
            if !matches!(label.trim().to_lowercase().as_str(), "date" | "sent") {
                return None;
            }
            match extract_stated_dates(value).as_slice() {
                [date] => Some((date.clone(), line.clone())),
                _ => None,
            }
        })
}

/// The fields an email's header block holds.
const EMAIL_FIELDS: &[&str] = &[
    "from",
    "to",
    "cc",
    "bcc",
    "date",
    "sent",
    "subject",
    "reply-to",
    "importance",
];

/// An email's header block: the run of header fields its first page opens
/// with, each line with its unit, when it holds "From:" and two more
/// ([`EMAIL_FIELDS`]). A reply that opens with its own words has none,
/// though a message it quotes further down starts "From: ... Sent: ...".
fn email_header_block<'a>(scope: &ValidationScope<'a>) -> Option<Vec<(&'a EvidenceUnit, String)>> {
    let first_page = scope.context_units().map(|unit| unit.page).min()?;
    let field = |line: &str| {
        let (label, _) = line.split_once(':')?;
        let label = label.trim().to_lowercase();
        EMAIL_FIELDS.contains(&label.as_str()).then_some(label)
    };
    let block = scope
        .context_units()
        .filter(|unit| unit.page == first_page && !unit.running)
        .flat_map(|unit| unit.text.lines().map(move |line| (unit, line.trim())))
        .filter(|(_, line)| !line.is_empty())
        .take_while(|(_, line)| field(line).is_some())
        .map(|(unit, line)| (unit, line.to_owned()))
        .collect::<Vec<_>>();
    let from = block
        .iter()
        .any(|(_, line)| field(line).as_deref() == Some("from"));
    (from && block.len() >= 3).then_some(block)
}

/// The "From:" line of an email's header block ([`email_header_block`]),
/// and its unit.
fn email_header<'a>(scope: &ValidationScope<'a>) -> Option<(&'a EvidenceUnit, String)> {
    email_header_block(scope)?.into_iter().find(|(_, line)| {
        line.split_once(':')
            .is_some_and(|(label, _)| label.trim().eq_ignore_ascii_case("from"))
    })
}

/// Whether `line` is the "Subject:" line of the email's header block
/// ([`email_header_block`]).
fn is_email_subject(scope: &ValidationScope<'_>, line: &str) -> bool {
    let subject = |held: &str| {
        held.split_once(':')
            .is_some_and(|(label, _)| label.trim().eq_ignore_ascii_case("subject"))
    };
    subject(line)
        && email_header_block(scope)
            .is_some_and(|block| block.iter().any(|(_, held)| held.trim() == line.trim()))
}

/// Mail providers, whose domains name no organization: an address there
/// says nothing of whom its holder writes for.
const MAIL_PROVIDERS: &[&str] = &[
    "gmail",
    "googlemail",
    "outlook",
    "hotmail",
    "live",
    "msn",
    "yahoo",
    "ymail",
    "aol",
    "icloud",
    "protonmail",
    "proton",
    "gmx",
    "btinternet",
    "comcast",
    "verizon",
];

/// Labels of a domain that name a kind of site or a host, never whose it
/// is: "com", "gov", "www", "mail".
const DOMAIN_KINDS: &[&str] = &[
    "com", "org", "net", "gov", "edu", "mil", "int", "info", "biz", "www", "mail", "email", "smtp",
    "mx",
];

/// The fewest letters a domain's label or a name must share for the label
/// to spell the name: two letters are a country's or a state's code.
const DOMAIN_NAME_LETTERS: usize = 3;

/// The labels of the domains a header line's addresses are at that can
/// name an organization: "dgs" of "Betts, Bruce@DGS
/// <Bruce.Betts@dgs.ca.gov>", "enron" of "<susan.bailey@enron.com>",
/// "icc-es" and "iccsafe" of a line with both. The last label names no one
/// ("com", "uk"), nor does a kind of site, a two-letter code or a mail
/// provider; an address has a dot in its domain, which a display name's
/// "Bruce@DGS" has not.
fn address_domain_labels(line: &str) -> Vec<String> {
    let mut labels = Vec::new();
    for token in line.split(|character: char| {
        character.is_whitespace() || matches!(character, '<' | '>' | '(' | ')' | ',' | ';' | '"')
    }) {
        let Some((mailbox, domain)) = token.rsplit_once('@') else {
            continue;
        };
        let domain = domain
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_lowercase();
        let parts = domain.split('.').collect::<Vec<_>>();
        if mailbox.is_empty() || parts.len() < 2 {
            continue;
        }
        for label in &parts[..parts.len() - 1] {
            if label.chars().count() >= DOMAIN_NAME_LETTERS
                && !DOMAIN_KINDS.contains(label)
                && !MAIL_PROVIDERS.contains(label)
                && !labels.iter().any(|known| known == label)
            {
                labels.push((*label).to_owned());
            }
        }
    }
    labels
}

/// Whether a domain's label spells an organization's name: the name run
/// together ("nfpa" for "NFPA", "c-spanarchives" for "C-SPAN Archives"),
/// its first word ("enron" for "ENRON CORP", "lambeth" for "Lambeth
/// Council"), the name with more after it ("hdrinc" for "HDR", "iccsafe"
/// for "ICC"), or its initials ("dgs" for "Department of General
/// Services"). Fewer than three letters spell nothing.
fn domain_spells(label: &str, name: &str) -> bool {
    let label = label
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect::<String>();
    let mut name_words = words(name);
    if name_words.first().is_some_and(|word| word == "the") {
        name_words.remove(0);
    }
    let Some(first) = name_words.first() else {
        return false;
    };
    let long_enough = |text: &str| text.chars().count() >= DOMAIN_NAME_LETTERS;
    let whole = name_words.concat();
    let initials = name_words
        .iter()
        .filter(|word| !STOPWORDS.contains(&word.as_str()))
        .filter_map(|word| word.chars().next())
        .collect::<String>();
    long_enough(&label)
        && ((long_enough(&whole) && label.starts_with(&whole))
            || (long_enough(first) && label == *first)
            || (name_words.len() > 1 && long_enough(&initials) && label == initials))
}

/// The two sides of an email as its header block's addresses give them:
/// the organizations its "From:" and its "To:" domains can name, and those
/// two lines with their units.
struct EmailSides<'a> {
    from: (Vec<String>, &'a EvidenceUnit, String),
    to: (Vec<String>, &'a EvidenceUnit, String),
}

impl<'a> EmailSides<'a> {
    /// `None` without a header block ([`email_header_block`]) that holds
    /// both lines.
    fn of(scope: &ValidationScope<'a>) -> Option<Self> {
        let block = email_header_block(scope)?;
        let field = |wanted: &str| {
            block.iter().find_map(|(unit, line)| {
                let (label, value) = line.split_once(':')?;
                label
                    .trim()
                    .eq_ignore_ascii_case(wanted)
                    .then(|| (address_domain_labels(value), *unit, line.clone()))
            })
        };
        Some(Self {
            from: field("from")?,
            to: field("to")?,
        })
    }

    /// The side `name` is on - the sender's when a "From:" domain spells
    /// it, the recipient's when a "To:" domain does - with the header line
    /// that shows it. `None` when neither does, or both do: a message
    /// within one organization says nothing of sides.
    fn side_of(&self, name: &str) -> Option<(PartyRole, &'a EvidenceUnit, &str)> {
        let spelled = |labels: &[String]| labels.iter().any(|label| domain_spells(label, name));
        match (spelled(&self.from.0), spelled(&self.to.0)) {
            (true, false) => Some((PartyRole::Sender, self.from.1, self.from.2.as_str())),
            (false, true) => Some((PartyRole::Recipient, self.to.1, self.to.2.as_str())),
            _ => None,
        }
    }
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
        .filter(|unit| !names_an_organization(unit))
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

/// Markings a letter carries at its head that are no organization's name,
/// however they are set: "WITHOUT PREJUDICE", "Subject to Contract", "By
/// Post", "URGENT". A phrase counts anywhere on the line; a single word
/// only as the whole line.
const CORRESPONDENCE_MARKINGS: &[&str] = &[
    "without prejudice",
    "subject to contract",
    "for the attention of",
    "recorded delivery",
    "special delivery",
    "first class",
    "open letter",
    "by post",
    "by fax",
    "by courier",
    "by e mail",
    "by mail",
    "by messenger",
    "save as to costs",
];
const CORRESPONDENCE_WORDS: &[&str] = &["urgent", "draft", "personal", "copy", "original"];

/// Whether a line is such a marking.
fn is_correspondence_marking(line: &str) -> bool {
    let loose = normalize_loosely(line);
    let loose = loose.trim();
    CORRESPONDENCE_MARKINGS
        .iter()
        .any(|phrase| loose.contains(phrase))
        || CORRESPONDENCE_WORDS.contains(&loose)
}

/// Whether a unit is only an organization's name - a letterhead's
/// "Meadowlark Health Plan" - or a delivery line: no title either way.
fn names_an_organization(unit: &EvidenceUnit) -> bool {
    let text = normalize(&unit.text);
    let text = text.trim();
    NOT_TITLE_WORDS.iter().any(|words| text.contains(words))
        || unit
            .features
            .organizations
            .iter()
            .any(|name| normalize(name).trim() == text)
        || text.split_whitespace().any(|word| {
            crate::cues::ORGANIZATION_ENDINGS.contains(
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
        title_lines(scope, unit).into_iter().find_map(|line| {
            // A letter's "Re:" line names it when its matter opens with a
            // kind of document: "Re: Demand for Payment - Highmeadow ...".
            // A kind its number follows names the numbered document the
            // letter is about: "RE: NDA 208090", "Re: Invoice No. 4471".
            if names_a_matter(line.trim_matches(|c: char| c == '*' || c.is_whitespace())) {
                let line = line.trim_matches(|c: char| c == '*' || c.is_whitespace());
                let line_words = words(line);
                let label = 1;
                let head = (label..line_words.len())
                    .find(|&at| crate::phrases::names_a_kind_at(&line_words, at))
                    .map(|at| &line_words[at])?;
                let phrase = title_phrase(line, head)?;
                let phrase_words = words(&phrase);
                let numbered = |end: usize| {
                    line_words.get(end).is_some_and(|next| {
                        labels_a_number(next) || next.chars().any(|c| c.is_ascii_digit())
                    })
                };
                return phrase_positions(&line_words, &phrase_words)
                    .into_iter()
                    .any(|at| {
                        at == label
                            && names_document_at(&line_words, at, at + phrase_words.len(), line)
                            && !numbered(at + phrase_words.len())
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
            let line_words = words(&line);
            (0..line_words.len())
                .filter(|&at| crate::phrases::names_a_kind_at(&line_words, at))
                .filter_map(|at| title_phrase(&line, &line_words[at]))
                .max_by_key(|phrase| phrase.split_whitespace().count())
                .map(|phrase| (phrase, line.trim().to_owned()))
        })
    })
}

/// A type the reply wrote all in lower case - "invoice", "notice of
/// default" - as a title: the words are the document's, the casing a
/// stylistic choice the model is not asked to make. Any capital in the
/// reply is kept as written.
/// A stated type as a title names its kind, not the whole line a bigger
/// model copies. The field a title line runs into is cut away ("CHANGE
/// ORDER Distribution: Briarport Library District" is a "Change Order");
/// the line's own kind phrase stands for the line ("Fieldnote S2 Product
/// Launch Plan" is a "Product Launch Plan", "Spring Wholesale Availability
/// and Price List" a "Price List", as [`title_phrase`] reads a title); and
/// a party the reply named is no part of a type ("Newsletter of the
/// Saltmarsh Point Homeowners Association" is a "Newsletter"). An
/// agreement or an amendment keeps its title whole: it is named for what
/// it is about ("Supply Agreement", "Amendment to Lease"), and a party's
/// name can be part of that.
fn stated_title(value: &str, parties: &[&str]) -> String {
    // "PLAINTIFF'S Notice of Voluntary Dismissal": the side filing a
    // notice is no part of its name.
    let tokens = value
        .split_whitespace()
        .skip_while(|token| crate::phrases::possessive_side(token))
        .collect::<Vec<_>>();
    let keep = |tokens: &[&str]| -> Option<String> {
        let kept = tokens.join(" ");
        (!kept.is_empty() && has_a_kind(&words(&kept))).then_some(kept)
    };
    // A field the title line runs into: a label ending with a colon, with
    // the field's value after it, where the title before the label ends in
    // a kind ("CHANGE ORDER Distribution: Briarport ..."). A colon that
    // ends the title itself is no label: "Notice of Hearing: Motion for
    // Summary Judgment" keeps its "Hearing".
    let ends_in_a_kind = |tokens: &[&str]| {
        words(&tokens.join(" "))
            .last()
            .is_some_and(|word| names_a_kind(word))
    };
    let mut value = tokens
        .iter()
        .enumerate()
        .skip(1)
        .find(|(at, token)| token.ends_with(':') && *at + 1 < tokens.len())
        .and_then(|(at, _)| {
            if ends_in_a_kind(&tokens[..at]) {
                keep(&tokens[..at])
            } else {
                None
            }
        })
        .unwrap_or_else(|| value.to_owned());
    if matches!(
        DocumentClass::of(Some(&value)),
        DocumentClass::Agreement | DocumentClass::Amendment
    ) {
        return value;
    }
    // The line's kind phrase, where the line is plainly more than a type:
    // three or more words around the kind phrase ("Spring Wholesale
    // Availability and Price List", "letter of 4 April 2014"), or fewer
    // with a number among them ("Fieldnote S2 Product Launch Plan",
    // "Advisory Opinion 1997-5"). A type's own modifiers stay: "Profit and
    // Loss Statement", "Health and Safety Plan", "Bank Statement", the one
    // code or year a title opens with ("W-9 Form", "2025 Annual Report")
    // and a numbered qualifier ("Phase 1 Decision Letter", "Section 5
    // Objection Letter"), all of which name the kind. Never what a
    // connector joins: an "Approval of Closure Plan" is an approval, and
    // the plan is what it approves.
    // Nor what it joins past an article or a name set in capitals
    // ([`joined_by_a_connector`]).
    let value_words = words(&value);
    if let Some(head) = head_noun(&value_words).map(str::to_owned)
        && let Some(phrase) = title_phrase(&value, &head)
        && let phrase_words = words(&phrase)
        && phrase_words.len() < value_words.len()
        && let Some(at) = phrase_positions(&value_words, &phrase_words)
            .into_iter()
            .next()
        && !joined_by_a_connector(&value, at)
    {
        // The tokens outside the phrase: a token may hold several words.
        let end = at + phrase_words.len();
        let mut seen = 0;
        let (mut before, mut after) = (Vec::new(), Vec::new());
        for token in value.split_whitespace() {
            let count = words(token).len();
            if seen + count <= at {
                before.push(token);
            } else if seen >= end {
                after.push(token);
            }
            seen += count;
        }
        let has_digit = |token: &&str| token.chars().any(|character| character.is_ascii_digit());
        let removed = before.len() + after.len();
        let numbered = before.iter().any(has_digit) || after.iter().any(has_digit);
        // The one code or year a title opens with, or a numbered
        // qualifier ("Phase 1", "Section 5"), names the kind and stays.
        let all_digits = |token: &str| {
            let core = token.trim_matches(|c: char| !c.is_ascii_alphanumeric());
            !core.is_empty() && core.chars().all(|c| c.is_ascii_digit())
        };
        let names_the_kind = after.is_empty()
            && match before[..] {
                [code] => is_form_code(code) || (code.len() == 4 && all_digits(code)),
                [word, number] => word.chars().all(char::is_alphabetic) && all_digits(number),
                _ => false,
            };
        if (removed >= 3 || numbered) && !names_the_kind {
            value = phrase;
        }
    }
    // A party's name, and the connector that introduces it, where what
    // is left names a kind of its own.
    if let Some((head, _)) = head_before_party(&value, parties)
        && heads_by_its_own_kind(&head)
        && let Some(kept) = keep(&head.split_whitespace().collect::<Vec<_>>())
    {
        value = kept;
    }
    value
}

/// Whether the head a title keeps before a party's name
/// ([`head_before_party`]) names a kind of its own: one that sets a name
/// apart ("Newsletter" of "Newsletter of the Saltmarsh Point Homeowners
/// Association", "Notice" of "Notice to the QRA of ..."). A response, a
/// notification, a citation or a circular cut from what it answers or
/// tells ("Response" of "Response to the ONS Engagement on Adjustment of
/// Migration Statistics") is a word a name holds as readily, alone: it
/// says what the paper does with the party, not what the paper is, and a
/// letter so titled is a letter.
fn heads_by_its_own_kind(head: &str) -> bool {
    words(head)
        .iter()
        .any(|word| crate::phrases::sets_a_name_apart(word))
}

/// The head of a title that names a party the reply named after its first
/// word: the title up to the party's name and the words that introduce it
/// ("Newsletter" of "Newsletter of the Saltmarsh Point Homeowners
/// Association", "Response" of "Response to the ONS Engagement on
/// Adjustment of Migration Statistics"), and whether a connector
/// introduces the party there, as "of" and "to" do. `None` when the title
/// names no party past its first word.
fn head_before_party(value: &str, parties: &[&str]) -> Option<(String, bool)> {
    const INTRODUCES: &[&str] = &[
        "of", "for", "by", "from", "to", "with", "between", "and", "the",
    ];
    let value_words = words(value);
    let mut at = parties
        .iter()
        .filter_map(|party| {
            let party_words = words(party);
            if party_words.is_empty() {
                return None;
            }
            phrase_positions(&value_words, &party_words)
                .into_iter()
                .next()
        })
        .filter(|&at| at > 0)
        .min()?;
    let mut introduced = false;
    while at > 1 && INTRODUCES.contains(&value_words[at - 1].as_str()) {
        introduced |= value_words[at - 1] != "the";
        at -= 1;
    }
    // The token the cut word opens: a token may hold several words
    // ("Girafa.com,") or none ("-").
    let tokens = value.split_whitespace().collect::<Vec<_>>();
    let mut seen = 0;
    let token_at = tokens
        .iter()
        .position(|token| {
            let count = words(token).len();
            let opens = seen == at && count > 0;
            seen += count;
            opens
        })
        .unwrap_or(tokens.len());
    Some((tokens[..token_at].join(" "), introduced))
}

/// Whether the kind phrase that opens at word `at` of a title is what a
/// connector joins to the title's head: right after the connector ("Approval
/// *of* Closure Plan"), or past an article or a name set in capitals
/// ("Response *to* the ONS Engagement on Adjustment of Migration
/// Statistics", "Approval *of* SEP Completion Report"). The head says what
/// the document is - a response, an approval - and the phrase what it
/// answers or approves, so the phrase is no type on its own. A sentence is
/// no title, and its connector joins nothing to one: "Receipt is
/// acknowledged *of* the Letter of Notification" still reads as the letter
/// it acknowledges.
fn joined_by_a_connector(value: &str, at: usize) -> bool {
    const JOINS: &[&str] = &["of", "to", "for", "on", "under", "by"];
    const ARTICLES: &[&str] = &["the", "a", "an"];
    let value_words = words(value);
    if at > 0 && JOINS.contains(&value_words[at - 1].as_str()) {
        return true;
    }
    let tokens = value.split_whitespace().collect::<Vec<_>>();
    // The tokens before the phrase: a token may hold several words
    // ("Girafa.com,") or none ("-").
    let mut seen = 0;
    let mut before = Vec::new();
    for token in &tokens {
        let count = words(token).len();
        if seen + count > at {
            break;
        }
        if count > 0 {
            before.push(*token);
        }
        seen += count;
    }
    let core = |token: &str| {
        normalize(token)
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_owned()
    };
    // A title set all in capitals sets no name apart by them.
    let in_capitals = |token: &str| {
        token.chars().any(char::is_alphabetic) && !token.chars().any(char::is_lowercase)
    };
    let all_capitals = tokens.iter().all(|token| in_capitals(token));
    let mut left = before.len();
    while left > 0
        && (ARTICLES.contains(&core(before[left - 1]).as_str())
            || (!all_capitals && in_capitals(before[left - 1])))
    {
        left -= 1;
    }
    left > 0
        && JOINS.contains(&core(before[left - 1]).as_str())
        && !words(&before[..left - 1].join(" "))
            .iter()
            .any(|word| RIGHT_STOPS.contains(&word.as_str()))
}

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

/// True when every unit that states `date` stands in or just after a
/// signature block, and one does. Running headers and footers are not read.
fn only_in_signatures(index: &EvidenceIndex, date: &str) -> bool {
    only_in_signatures_as(index, date, date_match_positions)
}

fn only_in_signatures_as(index: &EvidenceIndex, date: &str, positions: DatePositions) -> bool {
    let mut stated = false;
    for unit in index.units() {
        if unit.running || positions(date, &unit.normalized).is_empty() {
            continue;
        }
        if !unit.features.position.signature {
            return false;
        }
        stated = true;
    }
    stated
}

/// The latest date later than `date` that the document states only in its
/// signature blocks and that `acceptable` admits, with the line it stands
/// on. ISO dates order as text.
fn latest_signature_date(
    scope: &ValidationScope<'_>,
    date: &str,
    acceptable: impl Fn(&str) -> bool,
) -> Option<(String, String)> {
    let mut latest: Option<(String, String)> = None;
    for unit in scope.index.units() {
        if unit.running || !unit.features.position.signature {
            continue;
        }
        for candidate in stated_dates(&scope.view(&[unit])) {
            if candidate.as_str() > date
                && latest
                    .as_ref()
                    .is_none_or(|(kept, _)| candidate.as_str() > kept.as_str())
                && only_in_signatures(scope.index, &candidate)
                && acceptable(&candidate)
            {
                let line = line_where(unit, |line| {
                    !date_match_positions(&candidate, line).is_empty()
                })
                .unwrap_or_else(|| unit.text.trim().to_owned());
                latest = Some((candidate, line));
            }
        }
    }
    latest
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

/// Finds a free-text fact by its claims - its numbers and capitalized
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
                    stated.extend(words(&joined_across_line_breaks(&unit.text)));
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

/// `text` with each word it hyphenates across a line break joined: a
/// hyphen - a plain one, a soft one (U+00AD, as PDFs mark a break), or
/// U+2010 or U+2011 - right before "\n" or "\r\n" is taken out with the
/// break. "dis-\nplay" reads "display".
fn joined_across_line_breaks(text: &str) -> String {
    let mut joined = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if matches!(character, '-' | '\u{ad}' | '\u{2010}' | '\u{2011}') {
            let mut ahead = characters.clone();
            ahead.next_if_eq(&'\r');
            if ahead.next_if_eq(&'\n').is_some() {
                characters = ahead;
                continue;
            }
        }
        joined.push(character);
    }
    joined
}

/// `text` without its stray invisible characters ([`is_invisible`]) - a
/// soft hyphen, a zero-width space - but those a script spells with
/// ([`spells`]). Every check reads text through [`normalize`], which
/// ignores all of them.
fn without_stray_invisibles(text: &str) -> String {
    text.chars()
        .filter(|character| !is_invisible(*character) || spells(*character))
        .collect()
}

/// A reply's facts with the stray invisible characters taken out of every
/// value a check compares through [`normalize`] and a name or description
/// may write: the type, the date, each party's name, the subject, the
/// identifier, each key fact. A check that ignores a zero-width space must
/// not let one through into what is written.
fn without_stray_invisibles_in(mut facts: ModelFacts) -> ModelFacts {
    let clean = |value: &mut Option<String>| {
        if let Some(text) = value {
            *text = without_stray_invisibles(text);
        }
    };
    clean(&mut facts.document_type);
    clean(&mut facts.document_date);
    clean(&mut facts.subject);
    clean(&mut facts.identifier);
    for party in &mut facts.parties {
        party.name = without_stray_invisibles(&party.name);
    }
    for fact in &mut facts.key_facts {
        fact.fact = without_stray_invisibles(&fact.fact);
    }
    facts
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
        // An insurance document's or a health plan's client is its insured,
        // policyholder, member or patient: "Named insured: Brannock Tool &
        // Die Inc." is who a cancellation notice or a loss notice is for,
        // whichever company's form it is on.
        PartyRole::Client => (
            &[
                "client",
                "prepared for",
                "insured",
                "policyholder",
                "policy holder",
                "member",
                "patient",
                "subscriber",
            ],
            &["client", "insured", "policyholder"],
        ),
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
/// ([`Support::Context`]) - a role word labeling the name, a defined term
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
    let supports = |unit: &EvidenceUnit| {
        let header = table_header_line(scope, unit);
        unit_supports_role(
            unit,
            header.as_deref(),
            &normalized,
            &loose,
            role,
            true,
            letter,
        )
    };
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
const LABELED_ROLES: [PartyRole; 18] = [
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

/// The first role, in [`LABELED_ROLES`]' order, that what the model was
/// shown gives `name` in its own words: a label, a defined term, a
/// letterhead, an issuer or customer cue.
fn labeled_role(scope: &ValidationScope<'_>, name: &str) -> Option<PartyRole> {
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
    LABELED_ROLES.into_iter().find(|role| {
        naming.iter().any(|unit| {
            let header = table_header_line(scope, unit);
            unit_supports_role(
                unit,
                header.as_deref(),
                &normalized,
                &loose,
                *role,
                false,
                letter,
            )
        })
    })
}

/// The header row of the table a row unit belongs to, as its first line
/// reads loosely: what [`column_header`] reads a row's first line against.
fn table_header_line(scope: &ValidationScope<'_>, unit: &EvidenceUnit) -> Option<String> {
    unit.table_header
        .filter(|header| *header != unit.ordinal)
        .and_then(|header| scope.unit_at(header))
        .and_then(|header| header.text.lines().next())
        .map(normalize_loosely)
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
    /// Where the salutation stands: the first page's first line that opens
    /// "Dear ...", as its unit's ordinal and the line's place in the unit.
    salutation: Option<(u32, usize)>,
    /// Where a court document's caption stands, as the ordinals of its
    /// first and last units: from the first of the first page's first units
    /// that names a court, a tribunal or an arbitration, with no dateline
    /// between it and its sides, down to the last one that sets the case's
    /// sides against each other ("v.", "-against-") or labels one
    /// ("Plaintiff,").
    caption: Option<(u32, u32)>,
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
        let salutation = page().find_map(|unit| {
            unit.text
                .lines()
                .position(is_greeting)
                .map(|line| (unit.ordinal, line))
        });
        let dateline = page()
            .take(6)
            .find(|unit| is_bare_date(unit))
            .map(|unit| unit.ordinal)
            .filter(|_| salutation.is_some());
        let head = page().take(12).collect::<Vec<_>>();
        let has_word = |text: &str, wanted: &[&str]| {
            normalize_loosely(&text.replace(['|', '-'], " "))
                .split_whitespace()
                .any(|word| {
                    wanted.contains(&word)
                        || wanted.contains(&word.trim_end_matches(['s', ':', ';']))
                })
        };
        // A caption opens at the forum's heading: a letter to a court names
        // the court in its inside address, under the sender's letterhead,
        // and that letterhead is no part of a caption. A "Re:" line sets
        // out the case a letter is about ("Re: Nike, Inc. v. StockX LLC"),
        // and a side's word below a greeting is the letter's body: neither
        // marks a caption's side.
        let sets_out_sides = |unit: &EvidenceUnit| {
            if salutation.is_some_and(|(at, _)| unit.ordinal > at) {
                return false;
            }
            let lines = unit.text.lines().collect::<Vec<_>>();
            let above = match salutation {
                Some((at, line)) if at == unit.ordinal => &lines[..line.min(lines.len())],
                _ => &lines[..],
            };
            above
                .iter()
                .take_while(|line| !names_a_matter(line.trim().trim_matches('*')))
                .any(|line| {
                    has_word(line, VERSUS_WORDS)
                        || has_word(line, BRINGING_SIDE)
                        || has_word(line, ANSWERING_SIDE)
                })
        };
        // A caption's sides stand under its forum's heading with no
        // dateline between them: a court's own letterhead over a letter's
        // dateline heads no caption, though the letter sets out its case
        // below ("Smith v. Jones, No. 22-cv-123" with no "Re:").
        let caption = head
            .iter()
            .filter(|unit| has_word(&unit.text, FORUM_WORDS))
            .find_map(|forum| {
                let dated_below = dateline.filter(|&dateline| dateline > forum.ordinal);
                head.iter()
                    .filter(|unit| {
                        unit.ordinal >= forum.ordinal
                            && dated_below.is_none_or(|dateline| unit.ordinal < dateline)
                            && sets_out_sides(unit)
                    })
                    .map(|unit| unit.ordinal)
                    .max()
                    .map(|end| (forum.ordinal, end))
            });
        Self {
            dateline,
            salutation,
            caption,
        }
    }

    /// Whether a unit is part of a court document's caption: from the
    /// forum's heading down to the last line that sets out or labels its
    /// sides. A caption names the sides of a case, never a letterhead's
    /// sender.
    fn in_caption(self, unit: &EvidenceUnit) -> bool {
        self.caption
            .is_some_and(|(start, end)| (start..=end).contains(&unit.ordinal))
    }

    /// Whether the document is laid out as a letter: its first page greets
    /// someone ("Dear ...") below a line that is its dateline, the date and
    /// nothing else, or a "Date:" field. Extracted text often runs a letter's
    /// head into one paragraph, so the lines are read, not the units.
    fn laid_out_as_letter(self, scope: &ValidationScope<'_>) -> bool {
        let Some((salutation, at)) = self.salutation else {
            // No greeting: a letter that goes straight to its matter still
            // opens on its dateline and closes with a sign-off ("Yours
            // faithfully,") over its signature.
            return self.dateline.is_some_and(|_| signs_off(scope))
                || (first_page_lines(scope)
                    .take(12)
                    .any(|line| is_dateline(&line))
                    && signs_off(scope));
        };
        // The dateline found among the page's first units counts only above
        // the greeting: a memo opening "Dear colleagues" may date a section
        // below it. A greeting over a letter's sign-off is a letter whether
        // or not its dateline was found: a scan may run the date into its
        // letterhead, and an agency's letter may carry its date only on
        // the signature page appended to it.
        self.dateline.is_some_and(|dateline| dateline < salutation)
            || scope.index.units().iter().any(|unit| {
                if unit.running || unit.ordinal > salutation {
                    return false;
                }
                let lines = unit.text.lines().collect::<Vec<_>>();
                let above = if unit.ordinal == salutation {
                    &lines[..at.min(lines.len())]
                } else {
                    &lines[..]
                };
                above.iter().any(|line| is_dateline(line))
            })
            || self.signs_off_below_salutation(scope)
    }

    /// Whether the letter signs off anywhere below its greeting: a closing
    /// on a line of its own ("Sincerely,", "Yours sincerely,") in a later
    /// unit, or a later line of the greeting's. A letter's last page is not
    /// always its closing's: an electronic-signature page, an annex or an
    /// enclosure follows the signature, and a word processor's file is one
    /// page however long. The greeting on the first page says where the
    /// letter starts, so the closing is the letter's wherever it stands.
    fn signs_off_below_salutation(self, scope: &ValidationScope<'_>) -> bool {
        let Some((salutation, at)) = self.salutation else {
            return false;
        };
        scope.index.units().iter().any(|unit| {
            if unit.running || unit.ordinal < salutation {
                return false;
            }
            let below = if unit.ordinal == salutation {
                at + 1
            } else {
                0
            };
            unit.text.lines().skip(below).any(is_closing)
        })
    }

    /// Whether a unit stands before the dateline, where a letterhead can.
    fn before_dateline(self, unit: &EvidenceUnit) -> bool {
        self.dateline.is_none_or(|dateline| unit.ordinal < dateline)
    }

    /// The organization the inside address names under `person`: the first
    /// of the lines after the person's own, down to the street address,
    /// that is an organization's name and no job title, in the units of
    /// the first page between the dateline, when there is one, and the
    /// salutation - or the organization's line right above the person
    /// ("DUQUESNE LIGHT COMPANY / EMILY M FARAH CNSL"). A line that is
    /// exactly one of the reply's own parties (`replied`) counts too, though
    /// its name ends on no organization's word: "Chicago Mercantile
    /// Exchange"; so does a line written as a name is ("OFCOM", "Public
    /// Resource", "Daily Telegraph"), and the organization a post is held
    /// at ("Chair, Home Affairs Select Committee"). Its lines may be
    /// broken by "<br>" in a table's cell.
    fn addressee_organization<'a>(
        self,
        scope: &ValidationScope<'a>,
        person: &str,
        replied: &[String],
    ) -> Option<(String, &'a EvidenceUnit)> {
        let (salutation, _) = self.salutation?;
        let person = normalize_loosely(person);
        let in_address = |unit: &EvidenceUnit| self.in_address(unit);
        let lines_of = |unit: &EvidenceUnit| {
            let lines = unit
                .text
                .lines()
                .flat_map(|line| line.split("<br>"))
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            // The address ends at the salutation, when it shares the unit:
            // what follows "Dear ..." is the letter talking.
            let end = if unit.ordinal == salutation {
                lines
                    .iter()
                    .position(|line| is_greeting(line))
                    .unwrap_or(lines.len())
            } else {
                lines.len()
            };
            lines[..end].to_vec()
        };
        let units = scope
            .context_units()
            .filter(|unit| in_address(unit))
            .collect::<Vec<_>>();
        units.iter().enumerate().find_map(|(at, unit)| {
            let lines = lines_of(unit);
            let named = lines
                .iter()
                .position(|line| contains_whole(&normalize_loosely(line), &person))?;
            // The lines under the person, into the next units of the
            // address when theirs end.
            let mut below = lines[named + 1..].to_vec();
            for later in &units[at + 1..] {
                if below.len() >= 5 {
                    break;
                }
                below.extend(lines_of(later));
            }
            let people = units[at..]
                .iter()
                .take(3)
                .flat_map(|unit| unit.features.people.iter())
                .map(|person| normalize_loosely(person))
                .collect::<Vec<_>>();
            // The organization stands between the person and the street:
            // the address's own lines name no one, and a number in an
            // organization's own name ("3M Company") is no street number.
            // A line set like a person's name under the addressee is a
            // second person, and their post and organization are theirs,
            // so the scan ends there ("John Smith / Jane Doe / Director /
            // Acme Ltd"); under the addressee's own post such a line is
            // the organization the post is held at ("Bill Reeve / Director
            // of Rail / Transport Scotland").
            let mut under_a_post = false;
            let mut found = None;
            for raw in below.iter().take(5) {
                if is_address_line(raw) && !is_organization(raw) {
                    break;
                }
                let line = raw.trim_end_matches([',', ';', '|', '\\']).trim();
                // A post's line ("Chief Executive") is set like a name but
                // names none.
                let post = holds_a_post(line);
                let a_person = !post
                    && (people.contains(&normalize_loosely(line))
                        || names_a_person(line)
                        || (!under_a_post && looks_like_a_persons_name(line)));
                if a_person {
                    break;
                }
                under_a_post |= post;
                // "Premises: ...", "Owner: ...": a field's line, not the
                // address's.
                if line.contains(':') {
                    continue;
                }
                if let Some(organization) = organization_after_post(line) {
                    found = Some(organization);
                    break;
                }
                if (is_organization(line)
                    || replied
                        .iter()
                        .any(|name| normalize_loosely(name) == normalize_loosely(line)))
                    && line.split_whitespace().count() <= ORGANIZATION_LINE_WORDS
                    && !post
                {
                    found = Some(line.to_owned());
                    break;
                }
                if organization_line(line) {
                    found = Some(line.to_owned());
                    break;
                }
            }
            found
                .or_else(|| {
                    // The organization over the person: the line above
                    // theirs, in their unit or ending the unit before.
                    let above = match named.checked_sub(1) {
                        Some(above) => Some(lines[above].clone()),
                        None => at
                            .checked_sub(1)
                            .and_then(|previous| lines_of(units[previous]).last().cloned()),
                    }?;
                    let above = above.trim_end_matches([',', ';', '|']).trim();
                    (is_organization(above) && !holds_a_post(above) && !is_address_line(above))
                        .then(|| above.to_owned())
                })
                .map(|name| (name, *unit))
        })
    }

    /// Whether a unit can be part of the inside address: before the
    /// salutation, and after the dateline when the dateline opens the
    /// letter, as American letters are laid out. A British letter puts
    /// its date right over the greeting, nothing between them, under the
    /// address: its address stands above the dateline.
    fn in_address(self, unit: &EvidenceUnit) -> bool {
        let Some((salutation, _)) = self.salutation else {
            return false;
        };
        unit.ordinal <= salutation
            && (self.british() || self.dateline.is_none_or(|dateline| unit.ordinal > dateline))
    }

    /// Whether the letter is laid out the British way: the dateline right
    /// over the greeting, so the inside address stands above the dateline.
    fn british(self) -> bool {
        self.dateline
            .zip(self.salutation)
            .is_some_and(|(dateline, (salutation, _))| salutation == dateline + 1)
    }

    /// Whether the inside address - the first page between the dateline,
    /// when there is one, and the salutation - names `person`.
    fn in_inside_address(self, scope: &ValidationScope<'_>, person: &str) -> bool {
        let Some((salutation, at)) = self.salutation else {
            return false;
        };
        let person = normalize_loosely(person);
        scope.context_units().any(|unit| {
            if !self.in_address(unit) {
                return false;
            }
            unit.text
                .lines()
                .flat_map(|line| line.split("<br>"))
                .take(if unit.ordinal == salutation {
                    at
                } else {
                    usize::MAX
                })
                .any(|line| contains_whole(&normalize_loosely(line), &person))
        })
    }

    /// Whether the inside address names `organization` on a line of its
    /// own words: a "Re:" or "Subject:" line, and the lines of its unit
    /// after it, name the matter the letter is about ("Re: Nike, Inc. v.
    /// StockX LLC", "Re: Toledo Refining Company LLC - Warning Letter"),
    /// not who it is to.
    fn addresses(self, scope: &ValidationScope<'_>, organization: &str) -> bool {
        let Some((salutation, at)) = self.salutation else {
            return false;
        };
        let wanted = normalize_loosely(organization);
        scope.context_units().any(|unit| {
            if !self.in_address(unit) {
                return false;
            }
            unit.text
                .lines()
                .flat_map(|line| line.split("<br>"))
                .take(if unit.ordinal == salutation {
                    at
                } else {
                    usize::MAX
                })
                .take_while(|line| !names_a_matter(line.trim().trim_matches('*')))
                .any(|line| contains_whole(&normalize_loosely(line), &wanted))
        })
    }

    /// Whether the first page states `value` whole above the salutation,
    /// where a letter's own title stands, on a line that is no "Re:" line.
    fn states_above_salutation(self, scope: &ValidationScope<'_>, value: &str) -> bool {
        let Some((salutation, at)) = self.salutation else {
            return false;
        };
        let wanted = normalize_loosely(value);
        scope.index.units().iter().any(|unit| {
            if unit.running || unit.ordinal > salutation {
                return false;
            }
            let lines = unit.text.lines().collect::<Vec<_>>();
            let above = if unit.ordinal == salutation {
                &lines[..at.min(lines.len())]
            } else {
                &lines[..]
            };
            // A "Re:" or "Subject:" line names the matter the letter is
            // about ("Re: Approval of Trade Credit Application"), never the
            // letter's own title.
            above.iter().any(|line| {
                !names_a_matter(line) && contains_whole(&normalize_loosely(line), &wanted)
            })
        })
    }

    /// Whether a line of a unit stands below the salutation, where the
    /// letter is talking: a later unit, or a later line of the salutation's.
    fn below_salutation(self, unit: &EvidenceUnit, line: &str) -> bool {
        let Some((ordinal, at)) = self.salutation else {
            return false;
        };
        unit.ordinal > ordinal
            || (unit.ordinal == ordinal
                && unit
                    .text
                    .lines()
                    .skip(at + 1)
                    .any(|later| later.trim() == line.trim()))
    }

    /// Whether the dateline states `date`.
    fn dates_on_dateline(self, scope: &ValidationScope<'_>, date: &str) -> bool {
        self.dateline
            .and_then(|dateline| scope.unit_at(dateline))
            .is_some_and(|unit| digest_contains_date(&scope.view(&[unit]), date))
    }
}

/// The months, as a dateline writes them.
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

/// The months as a dateline abbreviates them ("Dec. 26, 1907").
const MONTH_ABBREVIATIONS: &[&str] = &[
    "jan", "feb", "mar", "apr", "jun", "jul", "aug", "sep", "sept", "oct", "nov", "dec",
];

/// Whether a word of a dateline names a month, written out or abbreviated.
fn names_a_month(word: &str) -> bool {
    MONTHS.contains(&word) || MONTH_ABBREVIATIONS.contains(&word)
}

/// Whether a word of a dateline is a day, a year or an ordinal day ("18th
/// October 2019").
fn is_date_number(word: &str) -> bool {
    let digits = word
        .strip_suffix("st")
        .or_else(|| word.strip_suffix("nd"))
        .or_else(|| word.strip_suffix("rd"))
        .or_else(|| word.strip_suffix("th"))
        .unwrap_or(word);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

/// The days of the week, as a dateline writes one before its date
/// ("Thursday 10 September 2020", "Thurs., Sept. 10, 2020").
const WEEKDAYS: &[&str] = &[
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
    "mon",
    "tue",
    "tues",
    "wed",
    "weds",
    "thu",
    "thur",
    "thurs",
    "fri",
    "sat",
    "sun",
];

/// The marks a word processor's file sets around a line in bold or in
/// italics ("**Thursday 10 September 2020**").
const EMPHASIS_MARKS: [char; 2] = ['*', '_'];

/// A dateline's words without the day of the week it opens on: the day
/// says nothing the date does not, and only the date's own words are read.
fn without_weekday(line_words: &[String]) -> &[String] {
    match line_words.split_first() {
        Some((first, rest)) if WEEKDAYS.contains(&first.as_str()) => rest,
        _ => line_words,
    }
}

/// Whether a unit is a date and nothing else - "March 2, 2026", "Thursday
/// 10 September 2020" - as a letter's dateline is written.
fn is_bare_date(unit: &EvidenceUnit) -> bool {
    unit.label.is_none()
        && (!unit.features.dates.is_empty() || is_dateline(&unit.text))
        && without_weekday(&words(&unit.text))
            .iter()
            .all(|word| names_a_month(word) || is_date_number(word))
}

/// Whether a line is a letter's dateline: a written date and nothing else,
/// or a "Date:" field holding one - "June 16, 2023", "Date: 28 March 2023",
/// "Dec. 26, 1907", "18th October 2019" - with or without the day of the
/// week before it ("Thursday 10 September 2020").
fn is_dateline(line: &str) -> bool {
    // A word processor's file marks a line set in bold or italics
    // ("**10 September 2020**"): the marks are no part of the date.
    let line = &line.replace(EMPHASIS_MARKS, " ");
    let line_words = words(line);
    // An agency's letter dates itself as released, issued or sent.
    let rest = match line_words.first().map(String::as_str) {
        Some("date" | "dated" | "released" | "issued" | "sent") => &line_words[1..],
        _ => &line_words[..],
    };
    let rest = without_weekday(rest);
    let written = !rest.is_empty()
        && rest.len() <= 4
        && rest.iter().any(|word| names_a_month(word))
        && rest
            .iter()
            .all(|word| names_a_month(word) || is_date_number(word))
        && !extract_stated_dates(&normalize(line)).is_empty();
    // Or a date in figures alone, its year whole: "02/09/2018".
    let figures = || {
        let normalized = normalize(line);
        let dates = numeric_dates(&normalized);
        let [date] = dates.as_slice() else {
            return false;
        };
        date.parts.iter().any(|part| part.len() == 4)
            && rest.len() == 3
            && normalized.trim().trim_end_matches('.').len() == date.end - date.start
    };
    written || figures()
}

/// Greetings a letter opens with besides "Dear ...", as a line of their own.
const GREETINGS: &[&str] = &[
    "gentlemen",
    "sir",
    "sirs",
    "madam",
    "madame",
    "dear sir or madam",
    "dear sir madam",
    "to whom it may concern",
    "ladies and gentlemen",
];

/// Whether a line is a letter's salutation: "Dear ...", "Dear," or "Dear"
/// alone (a published letter with the addressee's name removed), or
/// "Gentlemen:", "Sir:", "To Whom It May Concern". A table's cells may
/// split it ("| Dear | Mr. | Pilcher: |").
fn is_greeting(line: &str) -> bool {
    let loose = normalize_loosely(&line.replace(['|', '*', '#', '>', '_'], " "));
    let loose = loose.trim_end_matches([':', '-', ';', '!', ' ']).trim();
    loose == "dear" || loose.starts_with("dear ") || GREETINGS.contains(&loose)
}

/// The closings a letter signs off with, over its signature.
const CLOSINGS: &[&str] = &[
    "sincerely",
    "sincerely yours",
    "yours sincerely",
    "yours faithfully",
    "faithfully yours",
    "yours truly",
    "very truly yours",
    "yours very truly",
    "respectfully",
    "respectfully yours",
    "very respectfully",
    "respectfully submitted",
    "best regards",
    "kind regards",
    "warm regards",
    "with kind regards",
    "with best regards",
    "regards",
    "cordially",
    "cordially yours",
    "best wishes",
    "with best wishes",
    "yours",
    "yours ever",
];

/// Whether a line is a letter's closing: "Sincerely,", "Yours faithfully",
/// "Very respectfully," - or that, then a signature on the same line, as
/// OCR runs them together ("Sincerely, Amanda L. Edens").
fn is_closing(line: &str) -> bool {
    let written = normalize(&line.replace(['|', '*', '_'], " "));
    let written = written.trim().trim_end_matches([',', '.', ':', ';']).trim();
    CLOSINGS.contains(&written)
        || CLOSINGS.iter().any(|closing| {
            written
                .strip_prefix(closing)
                .is_some_and(|rest| rest.starts_with([',', ':', ';', '-']))
        })
}

/// Whether the document signs off as a letter does: one of its last lines
/// is a closing over a signature.
fn signs_off(scope: &ValidationScope<'_>) -> bool {
    let units = scope.index.units();
    let last_page = units.iter().map(|unit| unit.page).max();
    let lines = units
        .iter()
        .filter(|unit| Some(unit.page) == last_page && !unit.running)
        .flat_map(|unit| unit.text.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let tail = lines.len().saturating_sub(30);
    lines[tail..].iter().any(|line| is_closing(line))
}

/// The kind of document its layout says it is, when the reply named none
/// the document supports and no title stands in: "Email" for a header
/// block, "Letter" for a letter's layout, "Notice" for a heading that
/// opens on it. Each is a bare kind, so the name waits for a person;
/// "Letter to Wright Brothers" tells them what to check where "Document to
/// Wright Brothers" does not.
fn class_stand_in(scope: &ValidationScope<'_>, layout: LetterLayout) -> Option<&'static str> {
    if email_header_block(scope).is_some() {
        return Some("Email");
    }
    if layout.laid_out_as_letter(scope) {
        return Some("Letter");
    }
    notice_stand_in(scope)
}

/// "Notice" when a heading at the head of the document opens on the word
/// and no title was read from it (a title that runs past the reply's
/// reach, a heading cut by a party's name).
fn notice_stand_in(scope: &ValidationScope<'_>) -> Option<&'static str> {
    title_units(scope)
        .flat_map(|unit| title_lines(scope, unit))
        .take(6)
        .any(|line| {
            let line_words = words(&line);
            matches!(line_words.first().map(String::as_str), Some("notice"))
                && line_words.len() >= 2
                && !names_a_matter(&line)
        })
        .then_some("Notice")
}

/// The lines of the first page, in order, running lines left out.
fn first_page_lines<'a>(scope: &'a ValidationScope<'_>) -> impl Iterator<Item = String> + 'a {
    let units = scope.index.units();
    let first_page = units.iter().map(|unit| unit.page).min();
    units
        .iter()
        .filter(move |unit| Some(unit.page) == first_page && !unit.running)
        .flat_map(|unit| unit.text.lines().map(|line| line.trim().to_owned()))
        .filter(|line| !line.is_empty())
}

/// `cues`: whether an issued document's issuer and customer cues count.
/// They name the issuing and the billed side, not which of the issuing
/// side's roles - vendor, seller, issuer - a party has, so a role read from
/// the document alone takes them only as issuer and customer.
fn unit_supports_role(
    unit: &EvidenceUnit,
    table_header: Option<&str>,
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
    if labeled_on_its_line(unit, normalized, loose, labels) {
        return true;
    }
    // A table's column header over the name, for the side a document is
    // about: "| Named insured | Insured phone |" over "| Delacroix Bakehouse
    // LLC | (603) 555-0119 |". A column of recipients or signers says who
    // took delivery, not whose record it is.
    if role == PartyRole::Client && under_column(unit, table_header, loose, labels) {
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
    // A caption's head or an address block - a person's line ("The Rt Hon
    // George Osborne MP") over the organization's - is no letterhead.
    if matches!(role, PartyRole::Issuer | PartyRole::Sender)
        && unit.features.position.letterhead
        && letter.before_dateline(unit)
        && !letter.in_caption(unit)
        && unit.text.chars().count() <= LETTERHEAD_CHARACTERS
        && {
            let lines = unit.text.lines().collect::<Vec<_>>();
            lines
                .iter()
                .position(|line| normalize_loosely(line).starts_with(loose))
                .is_some_and(|at| !lines[..at].iter().any(|line| names_a_person(line)))
        }
    {
        return true;
    }
    let on_cue_line = |cues: &[&str]| {
        unit.text
            .lines()
            .any(|line| crate::infer::cue_labels(&normalize_loosely(line), loose, cues))
    };
    // The line that says where payment goes names the issuing side as a
    // "Remit To" line does: "Send Payment To: WFAA P.O. Box 637386".
    let on_issuer_line = || on_cue_line(ISSUER_CUES) || on_cue_line(PAYMENT_ADDRESS_CUES);
    match role {
        PartyRole::Issuer => on_issuer_line(),
        PartyRole::Vendor | PartyRole::Seller if cues => on_issuer_line(),
        PartyRole::Customer => on_cue_line(CUSTOMER_CUES),
        PartyRole::Recipient | PartyRole::Buyer | PartyRole::Client if cues => {
            on_cue_line(CUSTOMER_CUES)
        }
        _ => false,
    }
}

/// Whether a table's column header over `loose` in `unit` is one of
/// `labels`: the row above in the unit, else the header row of the table
/// the row belongs to.
fn under_column(
    unit: &EvidenceUnit,
    table_header: Option<&str>,
    loose: &str,
    labels: &[&str],
) -> bool {
    let lines = unit.text.lines().map(normalize_loosely).collect::<Vec<_>>();
    lines.iter().enumerate().any(|(index, line)| {
        let above = match index.checked_sub(1) {
            Some(previous) => lines.get(previous).map(String::as_str),
            None => table_header,
        };
        whole_positions(line, loose).into_iter().any(|at| {
            column_header(line, above, at)
                .is_some_and(|header| labels.iter().any(|word| label_names(&header, word)))
        })
    })
}

/// Whether a line names the party after one of `labels` - "Bill To:
/// Contoso", "| Tenant | Imogen Castellanos |", "Dear Ms. Okafor" - or
/// stands under a line that is only such a label ("BILL TO" over the
/// customer's name). What stands before the name on its line must be the
/// label and nothing more, so "payable to Acme" is no "to".
fn labeled_on_its_line(
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
/// A label naming a group of staff - "Crew member", "Board member" - names
/// no role.
fn label_names(label: &str, word: &str) -> bool {
    let staff = label
        .split(|c: char| !c.is_alphanumeric())
        .any(|part| STAFF_GROUP_WORDS.contains(&part));
    !staff
        && (label == word
            || (word.len() > 4
                && label.ends_with(word)
                && word_starts_at(label, label.len() - word.len())))
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
    // An appositive role, unless it is the label of the party's number:
    // ", company number 3899612" and "Employer ID Number NN-NNNNNNN" give
    // the party's registration, not its side.
    terms.iter().any(|term| {
        trimmed.starts_with(term)
            && !trimmed[term.len()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
            && !labels_the_partys_number(&trimmed[term.len()..])
    })
}

/// Whether `text`, after a role word, opens with the label of the party's
/// number ([`labels_a_number`]). "No." labels one only before a
/// registration's figures ("Company No. 3899612"): "Tenant No. 2" counts
/// the party among the others on its side.
fn labels_the_partys_number(text: &str) -> bool {
    let after = text.trim_start_matches(|character: char| !character.is_alphanumeric());
    let label = after
        .split(|character: char| !character.is_alphanumeric())
        .next()
        .unwrap_or_default();
    if !labels_a_number(label) {
        return false;
    }
    if !matches!(label, "no" | "nos") {
        return true;
    }
    after[label.len()..]
        .trim_start_matches(|character: char| !character.is_alphanumeric())
        .split_whitespace()
        .next()
        .is_some_and(|value| value.chars().filter(char::is_ascii_digit).count() >= 3)
}

/// Whether a word labels a number: "number", "no", "ID", "registration" -
/// or opens with "number", where a layout ran the value into its label
/// ("NumberNN-NNNNNNN").
fn labels_a_number(word: &str) -> bool {
    matches!(
        word,
        "number"
            | "numbers"
            | "no"
            | "nos"
            | "id"
            | "identification"
            | "registration"
            | "reg"
            | "ref"
            | "reference"
            | "code"
    ) || word.starts_with("number")
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

/// A name without the trading name it goes on to give - "Farringdon
/// Energy Limited trading as Champion Energy", "CWFS Insight LLC d/b/a
/// RealINSIGHT Marketplace": the company is the party, the trading name
/// a style.
fn without_trading_name(name: &str) -> String {
    // The markers are ASCII, and an ASCII lowering keeps every byte offset
    // of `name`: a Unicode lowering can change a letter's length ("İ")
    // and leave the cut inside a character.
    let lower = name.to_ascii_lowercase();
    let cut = [
        " trading as ",
        " t/a ",
        " d/b/a ",
        " dba ",
        " doing business as ",
    ]
    .iter()
    .filter_map(|marker| lower.find(marker))
    .min();
    match cut {
        Some(at) if at > 0 => name[..at].trim_end_matches([',', ';']).trim().to_owned(),
        _ => name.to_owned(),
    }
}

/// The whole organization name a reply's name is the end of - "MANUFACTURING
/// LLC" of "EMBer POSt MANUFACtURInG LLC" - when a unit of the context names
/// the organization, as the document writes it. `None` when the name is no
/// organization's end, or names it whole.
fn completed_name(scope: &ValidationScope<'_>, name: &str) -> Option<String> {
    let loose = normalize_loosely(name);
    let last = loose.split_whitespace().last()?;
    if !crate::cues::ORGANIZATION_ENDINGS.contains(&last) {
        return None;
    }
    scope.context_units().find_map(|unit| {
        // What completes the name is a few words of its own, never another
        // organization's legal form or noun: "Systems LLC and" is no part
        // of "Quill and Vane Advisory Group, Inc.". Nor is a role's or a
        // field's label: "Plaintiff" is no part of "Plaintiff Gilchrist
        // Harbor Marine Supply, LLC", nor "Account Name" of "Account Name
        // Calm.com, Inc.".
        let organization = unit.features.organizations.iter().find(|organization| {
            let Some(before) = organization
                .strip_suffix(loose.as_str())
                .filter(|before| before.ends_with(' '))
            else {
                return false;
            };
            let words = before.split_whitespace().collect::<Vec<_>>();
            (1..=4).contains(&words.len())
                && !words.iter().any(|word| {
                    crate::cues::ORGANIZATION_ENDINGS.contains(word)
                        || matches!(*word, "and" | "&" | "of")
                        || is_role_word(word)
                })
        })?;
        written_span(unit, organization)
    })
}

/// A subject with any run of capitalized words the document does not state
/// as one phrase written in lower case: "Credit Union Merger", put together
/// from a notice that writes "credit union" and "merger" apart, reads as
/// "credit union merger" - words of the document, not a name it never gives.
/// A run it states ("Aurora Catalog Project") and an initialism keep their
/// capitals.
fn unstated_names_lowered(subject: &str, scope: &ValidationScope<'_>) -> String {
    let tokens = subject.split(' ').collect::<Vec<_>>();
    let capitalized = |token: &str| {
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
        if !capitalized(tokens[at]) {
            at += 1;
            continue;
        }
        let mut end = at + 1;
        while end < tokens.len() && capitalized(tokens[end]) {
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

/// Whether a name is one capitalized word and nothing else - "Rowan",
/// "Priya" - the way a note names a person by first name. An initialism
/// ("IBM") or a word ending an organization's name is not one.
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
        && !crate::cues::ORGANIZATION_ENDINGS.contains(&word.to_lowercase().as_str())
}

/// The roles of the side a document is sent or billed to.
const RECEIVING_ROLES: &[PartyRole] = &[
    PartyRole::Customer,
    PartyRole::Recipient,
    PartyRole::Client,
    PartyRole::Buyer,
    PartyRole::Addressee,
];

/// The organization a customer field names - "Bill To: Ferncastle
/// Veterinary Hospital", a "SOLD TO" line over it - as the document writes
/// it.
fn customer_field(scope: &ValidationScope<'_>) -> Option<String> {
    scope.context_units().find_map(|unit| {
        let labeled = unit
            .label
            .as_deref()
            .is_some_and(|label| has_cue(&normalize_loosely(label), CUSTOMER_CUES))
            || unit
                .text
                .lines()
                .next()
                .is_some_and(|line| has_cue(&normalize_loosely(line), CUSTOMER_CUES));
        if !labeled {
            return None;
        }
        let organization = unit.features.organizations.first()?;
        written_span(unit, organization)
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

/// The value of the first field of the context labeled as what the
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

/// Whether a name is a redaction's marker: "REDACTED", "[name withheld]".
fn is_redaction(name: &str) -> bool {
    let name_words = words(name);
    name_words
        .iter()
        .any(|word| matches!(word.as_str(), "redacted" | "withheld"))
        && name_words.iter().all(|word| {
            matches!(
                word.as_str(),
                "redacted" | "withheld" | "name" | "names" | "personal" | "details"
            )
        })
}

/// Courtesy titles a person's name is written after, never part of the
/// name a file is known by: "Prof. A. G. Bell" is A. G. Bell. A title of
/// rank ("Sir", "Lord") is part of the name and stays.
const COURTESY_TITLES: &[&[&str]] = &[
    &["the", "rt", "hon"],
    &["the", "right", "honourable"],
    &["the", "honourable"],
    &["the", "honorable"],
    &["the", "hon"],
    &["rt", "hon"],
    &["hon"],
    &["mr"],
    &["mrs"],
    &["ms"],
    &["miss"],
    &["mx"],
    &["dr"],
    &["prof"],
    &["professor"],
];

/// Honours written after a titled person's name: a member of a
/// parliament, a King's or Queen's Counsel, an order.
const HONOURS: &[&str] = &[
    "mp", "msp", "ms", "am", "kc", "qc", "obe", "cbe", "mbe", "kcb", "dbe", "kbe", "ch",
];

/// A person's name without the courtesy title written before it, when
/// what follows is a person's whole name: two or three words, none of
/// them an organization's or a figure ("Dr. Ing. h.c. F. Porsche AG" keeps
/// its "Dr.", and "Dr. Smith" its title, a surname alone being no name).
/// A title that addresses two people at once ("Mr and Mrs Smith", "Mr &
/// Mrs Smith") stays: what follows it is no one's name.
fn without_courtesy_title(name: &str) -> String {
    let tokens = name.split_whitespace().collect::<Vec<_>>();
    let token_words = tokens.iter().map(|token| words(token)).collect::<Vec<_>>();
    for title in COURTESY_TITLES {
        if tokens.len() <= title.len()
            || !token_words[..title.len()]
                .iter()
                .zip(title.iter())
                .all(|(word, wanted)| word.len() == 1 && word[0] == *wanted)
        {
            continue;
        }
        // The honours after a titled name go with the title: "The Rt Hon
        // Alex Chalk KC MP" is Alex Chalk.
        let mut rest = &tokens[title.len()..];
        while let [kept @ .., last] = rest
            && words(last)
                .first()
                .is_some_and(|word| HONOURS.contains(&word.as_str()))
        {
            rest = kept;
        }
        let rest_name = rest.join(" ");
        let joint = rest.first().is_some_and(|first| {
            words(first)
                .first()
                .is_none_or(|word| matches!(word.as_str(), "and" | "or"))
        }) || rest.iter().any(|token| {
            let token_words = words(token);
            COURTESY_TITLES
                .iter()
                .any(|title| title.len() == 1 && token_words == [title[0]])
        });
        if (2..=3).contains(&rest.len())
            && !joint
            && !is_organization_name(&rest_name)
            && !rest_name.chars().any(|c| c.is_ascii_digit())
            && !words(&rest_name).iter().any(|word| {
                ORGANIZATION_WORDS.contains(&word.as_str())
                    || crate::cues::ORGANIZATION_ENDINGS.contains(&word.as_str())
            })
        {
            return rest_name;
        }
        break;
    }
    name.to_owned()
}

/// Whether a name is an organization's: it ends on a legal form or an
/// organization's noun.
fn is_organization(name: &str) -> bool {
    normalize_loosely(name)
        .split_whitespace()
        .last()
        .is_some_and(|word| crate::cues::ORGANIZATION_ENDINGS.contains(&word))
}

/// Nouns a public body's or a membership body's name is headed by, besides
/// the [`crate::cues::ORGANIZATION_ENDINGS`] a company's name ends on:
/// "Home Affairs Select Committee", "Horizon Compensation Advisory Board",
/// "Ministry of Defence".
const BODY_NOUNS: &[&str] = &[
    "committee",
    "board",
    "commission",
    "ministry",
    "office",
    "bureau",
    "administration",
    "directorate",
    "secretariat",
    "inspectorate",
    "tribunal",
    "federation",
    "organisation",
    "organization",
];

/// Whether a party's name is an organization's: [`is_organization`], or a
/// name headed by a body's noun, at its end or before the "of", "for" or
/// "on" that completes it - "Migration Advisory Committee", "International
/// Union of Biochemistry", "Office for National Statistics". A person's post
/// is headed by no such noun ("Secretary of State", "Chair of the Board",
/// "Treasurer of the International Union"), so the people who hold one stay
/// people, and so is a post written before the body it is held at
/// ("Director, Office of Prescription Drug Promotion", "Member, Board of
/// Directors"). Only a party's name is read this way: a line of an address
/// or a signature block that ends on "Committee" may still be a post's.
fn is_organization_name(name: &str) -> bool {
    if is_organization(name) {
        return true;
    }
    let loose = normalize_loosely(name);
    let name_words = loose.split_whitespace().collect::<Vec<_>>();
    let head = name_words
        .iter()
        .position(|word| matches!(*word, "of" | "for" | "on"))
        .unwrap_or(name_words.len());
    // "Office of the Chief Counsel, ..." is a body before its comma, though
    // it names a post.
    let post_before = name.split_once(',').is_some_and(|(post, _)| {
        let post_words = words(post);
        (holds_a_post(post) || post_words.iter().any(|word| word == "member"))
            && !post_words.iter().any(|word| {
                BODY_NOUNS.contains(&word.as_str())
                    || crate::cues::ORGANIZATION_ENDINGS.contains(&word.as_str())
            })
    });
    !post_before
        && name_words[..head].last().is_some_and(|word| {
            BODY_NOUNS.contains(word) || crate::cues::ORGANIZATION_ENDINGS.contains(word)
        })
}

/// The most words a line naming an organization in an inside address runs
/// to: more is a sentence.
const ORGANIZATION_LINE_WORDS: usize = 10;

/// Words a street address, a building or a postal line is written with.
const ADDRESS_WORDS: &[&str] = &[
    "street",
    "st",
    "road",
    "rd",
    "avenue",
    "ave",
    "lane",
    "drive",
    "boulevard",
    "blvd",
    "way",
    "box",
    "suite",
    "ste",
    "floor",
    "house",
    "building",
    "plaza",
    "square",
    "tower",
    "park",
    "place",
    "row",
    "terrace",
    "quay",
    "wharf",
    "gate",
    "court",
    "crescent",
    "close",
    "hill",
    "mail",
    "stop",
    "po",
    "p.o",
    "attn",
    "attention",
    "fax",
    "tel",
    "telephone",
    "phone",
    "email",
    "e-mail",
    "www",
];

/// Words an organization's name is made of besides its legal form, which
/// tell it from a title that shares a word with it: an "Office of
/// Management and Budget" is no budget.
const ORGANIZATION_WORDS: &[&str] = &[
    "office",
    "department",
    "commission",
    "ministry",
    "committee",
    "council",
    "board",
    "agency",
    "bureau",
    "authority",
    "administration",
    "association",
    "society",
    "institute",
    "university",
    "college",
    "school",
    "company",
    "corporation",
    "group",
    "bank",
    "trust",
    "fund",
    "foundation",
    "service",
    "services",
    "division",
    "directorate",
    "secretariat",
    "organisation",
    "organization",
    "federation",
    "union",
    "league",
    "party",
    "government",
    "treasury",
    "exchange",
    "chamber",
    "church",
    "hospital",
    "clinic",
    "centre",
    "center",
    "library",
    "museum",
    "press",
    "telegraph",
    "journal",
    "gazette",
    "news",
    "network",
    "rail",
    "railway",
    "resource",
    "guard",
    "force",
    "corps",
    "army",
    "navy",
    "inc",
    "llc",
    "ltd",
    "plc",
    "corp",
];

/// Words a line that names a person opens with.
const HONORIFICS: &[&str] = &[
    "mr",
    "mrs",
    "ms",
    "miss",
    "mx",
    "dr",
    "prof",
    "professor",
    "hon",
    "the",
    "rt",
    "sir",
    "dame",
    "lord",
    "lady",
    "gen",
    "col",
    "capt",
    "lt",
    "maj",
    "judge",
    "justice",
    "senator",
    "rep",
    "rev",
    "fr",
];

/// Whether a line names an organization the way an inside address or a
/// signature block writes one: a short line of capitalized words with no
/// figure, no post, no street and no greeting in it - "OFCOM", "Daily
/// Telegraph", "Committee on Standards in Public Life", "POST AND
/// SCHELL". A line that opens on a title of address names a person.
fn organization_line(line: &str) -> bool {
    let line = line
        .trim()
        .trim_end_matches([',', ';', '|', '\\', '.'])
        .trim();
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty()
        || tokens.len() > 7
        || line.contains(['@', ':', '(', ')'])
        || line.contains("http")
        || line.chars().any(|c| c.is_ascii_digit())
    {
        return false;
    }
    let loose = normalize_loosely(line);
    let loose_words = loose.split_whitespace().collect::<Vec<_>>();
    let line_words = words(line);
    if loose_words
        .iter()
        .any(|word| JOB_TITLE_WORDS.contains(word))
        || ends_on_an_address_word(line)
        || loose_words
            .first()
            .is_some_and(|word| HONORIFICS.contains(word))
        || line_words
            .first()
            .is_some_and(|word| crate::phrases::sets_a_name_apart(word))
        || (line_words
            .iter()
            .any(|word| crate::phrases::sets_a_name_apart(word))
            && !line_words
                .iter()
                .any(|word| ORGANIZATION_WORDS.contains(&word.as_str())))
        || is_greeting(line)
        || is_closing(line)
        || NOT_TITLE_WORDS.iter().any(|phrase| loose.contains(phrase))
        || is_correspondence_marking(line)
    {
        return false;
    }
    const SMALL: &[&str] = &["of", "and", "the", "for", "on", "in", "at", "de", "du", "&"];
    let capitalized = |token: &str| {
        token
            .chars()
            .find(|c| c.is_alphabetic())
            .is_some_and(char::is_uppercase)
    };
    let in_capitals = |token: &str| !token.chars().any(char::is_lowercase);
    // A name has a word in it: "U. S. A." and "DJ" are letters.
    if !tokens
        .iter()
        .any(|token| token.chars().filter(|c| c.is_alphabetic()).count() >= 3)
    {
        return false;
    }
    // One word alone names an organization only set in capitals, as an
    // acronym is ("OFCOM"): a lone "London" is the address's city, "DJ"
    // the typist's initials.
    if let [only] = tokens.as_slice()
        && !(in_capitals(only) && only.chars().filter(|c| c.is_alphabetic()).count() >= 4)
    {
        return false;
    }
    // "Dayton, Ohio", "Washington, D.C.": a city and its state, in two
    // short halves around a comma, neither ending on an organization's
    // word.
    if let Some((city, state)) = line.split_once(',')
        && city.split_whitespace().count() <= 2
        && state.split_whitespace().count() <= 2
        && !is_organization(line)
    {
        return false;
    }
    // "Department for" is the head of a name wrapped onto the next line.
    if line_words
        .last()
        .is_some_and(|last| SMALL.contains(&last.as_str()))
    {
        return false;
    }
    tokens
        .iter()
        .all(|token| capitalized(token) || SMALL.contains(&normalize(token).as_str()))
        && tokens.iter().filter(|token| capitalized(token)).count() >= 1
}

/// Whether a line is a street's, a building's, a city's or a contact's:
/// a figure in it ("880 Harrow Lane", "YO1 6WP", "3rd Floor") or a word
/// an address is written with.
fn is_address_line(line: &str) -> bool {
    line.chars().any(|c| c.is_ascii_digit()) || ends_on_an_address_word(line)
}

/// Whether a line ends on a word an address is written with: "Riverside
/// House", "Horse Guards Road", "Victoria Quay". A "Building Standards
/// Commission" is no building.
fn ends_on_an_address_word(line: &str) -> bool {
    words(line)
        .last()
        .is_some_and(|word| ADDRESS_WORDS.contains(&word.as_str()))
}

/// Letters after a person's name: "Jo Churchill MP", "Sir Gus O'Donnell
/// KCB", "Karen D. Coombs, Esq.".
const POST_NOMINALS: &[&str] = &[
    "mp", "msp", "ms", "am", "qc", "kc", "esq", "esquire", "jr", "sr", "ii", "iii", "phd", "md",
    "obe", "cbe", "mbe", "kcb", "dbe", "kbe", "frs", "frsa", "cnsl", "pc", "ch", "dl", "pe", "cpa",
    "rn",
];

/// Whether a line names a person by how it is written: a title of
/// address before the name, or letters after it.
fn names_a_person(line: &str) -> bool {
    let line_words = words(line);
    line_words
        .first()
        .is_some_and(|word| HONORIFICS.contains(&word.as_str()) && word != "the")
        || line_words
            .last()
            .is_some_and(|word| POST_NOMINALS.contains(&word.as_str()) && line_words.len() >= 2)
}

/// Whether `name` is a person's: the index read it as one somewhere in
/// the context, it is written as one ([`names_a_person`]), or it is a
/// short name with no word an organization's is made of ("Rashmi
/// Panigrahi", never "U.S. Department of Energy").
fn named_as_a_person(scope: &ValidationScope<'_>, name: &str) -> bool {
    let loose = normalize_loosely(name);
    let name_words = words(name);
    names_a_person(name)
        || scope.context_units().any(|unit| {
            unit.features
                .people
                .iter()
                .any(|person| normalize_loosely(person) == loose)
        })
        || (name_words.len() <= 3
            && !name.chars().any(|c| c.is_ascii_digit())
            && !name_words.iter().any(|word| {
                ORGANIZATION_WORDS.contains(&word.as_str())
                    || crate::cues::ORGANIZATION_ENDINGS.contains(&word.as_str())
                    || crate::phrases::sets_a_name_apart(word)
            })
            // "Thames Water" is as short as a person's name and has no
            // organization's word, but the index read it as a company.
            && !named_as_an_organization(scope, name))
}

/// Whether a line holds a post someone is named by.
fn holds_a_post(line: &str) -> bool {
    words(line)
        .iter()
        .any(|word| JOB_TITLE_WORDS.contains(&word.as_str()))
}

/// The organization a post is held at, when the post's line names it after
/// the post - "Chair, Home Affairs Select Committee", "Director of
/// Regulation, PacifiCorp" - and it ends on an organization's word.
fn organization_after_post(line: &str) -> Option<String> {
    if !holds_a_post(line) {
        return None;
    }
    // The post stands alone before the comma ("Chair,", "Managing
    // Director,"): "Minister of State for Crime, Policing and the Fire
    // Service" is one post, cut by its own comma.
    let (post, rest) = line.split_once(',').or_else(|| line.split_once(" at "))?;
    if post.split_whitespace().count() > 3 {
        return None;
    }
    let rest = rest.trim().trim_end_matches([',', ';', '.']).trim();
    let organization_word = words(rest)
        .iter()
        .any(|word| ORGANIZATION_WORDS.contains(&word.as_str()));
    (!holds_a_post(rest)
        && !rest.chars().any(|c| c.is_ascii_digit())
        && (is_organization(rest)
            || (organization_word
                && rest.split_whitespace().count() >= 2
                && organization_line(rest))))
    .then(|| rest.to_owned())
}

/// The organization a signature block names under `person`: the lines
/// after the person's own, past their post, down to the first street or
/// contact line, where the letter's closing stands over them on its last
/// page ("Rashmi Panigrahi / FOI Officer / Office of Manpower Economics").
fn signer_organization<'a>(
    scope: &ValidationScope<'a>,
    person: &str,
) -> Option<(String, &'a EvidenceUnit)> {
    let person = normalize_loosely(person);
    let units = scope.index.units();
    let last_page = units.iter().map(|unit| unit.page).max()?;
    let page = units
        .iter()
        .filter(|unit| unit.page == last_page && !unit.running)
        .collect::<Vec<_>>();
    // Every line of the page, with its unit.
    let lines = page
        .iter()
        .flat_map(|unit| {
            unit.text
                .lines()
                .flat_map(|line| line.split("<br>"))
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(move |line| (*unit, line))
        })
        .collect::<Vec<_>>();
    let closing = lines.iter().rposition(|(_, line)| is_closing(line))?;
    // The signer's name stands right under the closing, on the first line
    // that is not a signature mark ("/s/", "By:", a rule), or the one
    // after it when the post comes first. A name further down is in the
    // address under the signature ("Thames Water" three lines under
    // "Yours faithfully, Development Planning Department"), not the
    // signer's.
    let is_mark = |line: &str| {
        let loose = normalize_loosely(line);
        loose.is_empty()
            || matches!(
                loose.split_whitespace().next(),
                Some("s" | "by" | "signed" | "signature")
            ) && loose.split_whitespace().count() <= 2
            || line
                .chars()
                .all(|c| c == '_' || c == '-' || c.is_whitespace())
    };
    let named = lines[closing + 1..]
        .iter()
        .enumerate()
        .filter(|(_, (_, line))| !is_mark(line))
        .take(2)
        .find(|(_, (_, line))| contains_whole(&normalize_loosely(line), &person))
        .map(|(offset, _)| closing + 1 + offset)?;
    let (unit, _) = lines[named];
    // The block ends where the address begins: a street with its number,
    // an email, a telephone. A city on the line under the street ("New
    // York") is no organization of the signer's; a number in an
    // organization's own name ("3M Company") is no street number. It also
    // ends at another person's line: a co-signer's post and organization
    // are theirs ("John Smith / Jane Doe / Director / Acme Ltd").
    let people = page
        .iter()
        .flat_map(|unit| unit.features.people.iter())
        .map(|person| normalize_loosely(person))
        .collect::<Vec<_>>();
    for (_, raw) in lines[named + 1..].iter().take(6) {
        if names_a_copy(raw)
            || raw.contains('@')
            || (raw.chars().any(|c| c.is_ascii_digit()) && !is_organization(raw))
        {
            break;
        }
        let line = raw.trim_end_matches([',', ';', '|', '\\', '.']).trim();
        let post = holds_a_post(line);
        if !post
            && (people.contains(&normalize_loosely(line))
                || names_a_person(line)
                || looks_like_a_persons_name(line))
        {
            break;
        }
        if let Some(organization) = organization_after_post(line) {
            return Some((organization, unit));
        }
        if post {
            continue;
        }
        if is_organization(line) && line.split_whitespace().count() <= ORGANIZATION_LINE_WORDS {
            return Some((line.to_owned(), unit));
        }
        // A line written like a name but with no organization's word is
        // read as one: "OFCOM", "Office of Manpower Economics".
        if organization_line(line) {
            return Some((line.to_owned(), unit));
        }
    }
    None
}

/// Whether a line is set like a person's name alone: two or three words in
/// mixed case, none an organization's word and none an acronym in
/// capitals ("Jane Doe", "Mary Ann Smith"; never "OFCOM", "Daily
/// Telegraph Group" or "Pool Re Limited").
fn looks_like_a_persons_name(line: &str) -> bool {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    let line_words = words(line);
    (2..=3).contains(&tokens.len())
        && tokens.iter().all(|token| {
            token.chars().next().is_some_and(char::is_uppercase)
                && token.chars().skip(1).any(char::is_lowercase)
        })
        && !line_words.iter().any(|word| {
            ORGANIZATION_WORDS.contains(&word.as_str())
                || crate::cues::ORGANIZATION_ENDINGS.contains(&word.as_str())
                || crate::phrases::sets_a_name_apart(word)
        })
}

/// Whether a line names who is copied or what is enclosed, under a
/// signature: "cc: ...", "Enclosures", "Attachment: ...".
fn names_a_copy(line: &str) -> bool {
    let first = words(line);
    matches!(
        first.first().map(String::as_str),
        Some(
            "cc" | "bcc"
                | "c"
                | "copy"
                | "copies"
                | "enc"
                | "encl"
                | "enclosure"
                | "enclosures"
                | "attachment"
                | "attachments"
        )
    )
}

/// Whether the index read `name` as an organization anywhere in the
/// context: "Federal Communications Commission" ends on no legal form,
/// but the document names it as one.
fn named_as_an_organization(scope: &ValidationScope<'_>, name: &str) -> bool {
    let loose = normalize_loosely(name);
    scope.context_units().any(|unit| {
        unit.features
            .organizations
            .iter()
            .any(|organization| normalize_loosely(organization) == loose)
    })
}

/// The organization a letterhead names when [`letterhead_organization`]
/// finds none: the first line at the head of the first page, before the
/// dateline, that is written as an organization's name and nothing else
/// ("BUILDING STANDARDS COMMISSION", "GALLION & SPIELVOGEL").
fn letterhead_name<'a>(
    scope: &ValidationScope<'a>,
    layout: LetterLayout,
    addressees: &[String],
) -> Option<(String, &'a EvidenceUnit)> {
    let units = scope.index.units();
    let first_page = units.iter().map(|unit| unit.page).min()?;
    let (dateline, _) = layout.dateline.zip(layout.salutation)?;
    let addressees = addressees
        .iter()
        .map(|name| normalize_loosely(name))
        .collect::<Vec<_>>();
    // The letterhead stands over the inside address: a unit that names an
    // addressee, and every unit after it, is the address.
    let address = units
        .iter()
        .filter(|unit| unit.page == first_page && !unit.running)
        .find(|unit| {
            let text = normalize_loosely(&unit.text);
            addressees
                .iter()
                .any(|name| !name.is_empty() && contains_whole(&text, name))
        })
        .map(|unit| unit.ordinal);
    // In a British layout the inside address stands above the dateline,
    // so without an addressee to mark where it begins, the lines above
    // the dateline cannot be told from it: "Acme Ltd" over the person
    // written to is the address's, not the sender's letterhead.
    if address.is_none() && layout.british() {
        return None;
    }
    let address = address.unwrap_or(dateline);
    units
        .iter()
        .filter(|unit| {
            unit.page == first_page && !unit.running && unit.ordinal < dateline.min(address)
        })
        .take(4)
        .flat_map(|unit| {
            unit.text
                .lines()
                .flat_map(|line| line.split("<br>"))
                .map(move |line| (unit, line.trim().trim_matches('*').trim()))
        })
        .take(8)
        .find_map(|(unit, line)| {
            let line = line.trim_end_matches([',', ';', '|']).trim();
            let loose = normalize_loosely(line);
            // A line naming an addressee is the address's, wherever it
            // stands.
            let names_an_addressee = addressees.iter().any(|name| {
                !name.is_empty() && (contains_whole(&loose, name) || contains_whole(name, &loose))
            });
            (line.split_whitespace().count() <= ORGANIZATION_LINE_WORDS
                && !names_an_addressee
                && !holds_a_post(line)
                && !line.chars().any(|c| c.is_ascii_digit())
                && (is_organization(line) || organization_line(line)))
            .then(|| (line.to_owned(), unit))
        })
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
    "editor",
    "leader",
    "analyst",
    "specialist",
    "assistant",
    "attorney",
    "esq",
    "esquire",
    "inspector",
    "engineer",
    "consultant",
    "clerk",
    "solicitor",
    "adviser",
    "advisor",
    "commissioner",
    "minister",
    "governor",
    "mayor",
];

/// Whether a person signs for one of `organizations`: a unit names them,
/// then a job title, then the organization - "Harriet Voss, Vice President
/// of People Operations Northstar Lantern Works LLC", "Keziah Ambrose,
/// Owner, Briarport Coffee Roasters LLC".
fn signs_for(scope: &ValidationScope<'_>, name: &str, organizations: &[String]) -> bool {
    if is_organization_name(name) || organizations.is_empty() {
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
                && organizations
                    .iter()
                    .any(|organization| !whole_positions(after, organization).is_empty())
        })
    })
}

/// Words of a post someone holds for an organization, read right after
/// their name ("Delphine Moncrieff, Associate Conservator", "Beckett
/// Halloran (President)"). Not "owner": "Owner: Jane Doe" names a party.
const POST_WORDS: &[&str] = &[
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
    "chair",
    "chairman",
    "chairperson",
    "controller",
    "conservator",
    "curator",
    "registrar",
    "winemaker",
    "mechanic",
    "inspector",
    "examiner",
    "technician",
    "engineer",
    "accountant",
    "auditor",
    "clerk",
    "superintendent",
    "foreman",
];

/// Labels whose value is someone doing a job for the document's
/// organization rather than a party to it.
const STAFF_LABELS: &[&str] = &[
    "examined by",
    "inspected by",
    "prepared by",
    "approved by",
    "authorized by",
    "reviewed by",
    "completed by",
    "signed by",
    "recorded by",
    "winemaker",
    "inspector",
    "mechanic",
    "mechanics",
    "technician",
    "driver",
];

/// A bare signature: anyone signs their own letter this way, so it shows
/// someone acting for an organization only outside correspondence, and
/// never for the one the document labels as its sender.
const SIGNATURE_MARKS: &[&str] = &["signature", "/s/"];

/// Words of a label whose value is the one a document is about or for:
/// such a person is a party whatever post they also hold ("Employee name",
/// "Employee signature", "Dear", "To:").
const SUBJECT_WORDS: &[&str] = &[
    "dear",
    "to",
    "resident",
    "tenant",
    "patient",
    "employee",
    "member",
    "insured",
    "applicant",
    "borrower",
    "candidate",
    "beneficiary",
    "claimant",
    "policyholder",
    "participant",
    "student",
    "client",
    "customer",
    "owner",
    "landlord",
    "lessee",
    "lessor",
    "guarantor",
];

/// What the document makes of a person the reply named.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Standing {
    /// A line names them as the one it is about, or a "between" clause as
    /// a party.
    Subject,
    /// They are named only as someone doing a job for an organization.
    Staff,
    Unknown,
}

/// The last `n` words of `text`, read as one phrase: what a label before a
/// name on its line reads as ("| Examined by |" -> "examined by").
fn closing_words(text: &str, n: usize) -> String {
    let words = label_cell(text)
        .split_whitespace()
        .map(|word| word.trim_end_matches(':'))
        .collect::<Vec<_>>();
    words[words.len().saturating_sub(n)..].join(" ")
}

/// The table cell or line part a label before a name sits in: the text
/// after the last cell border, without the punctuation that ends a label.
fn label_cell(text: &str) -> &str {
    text.trim_end_matches(|c: char| c.is_whitespace() || matches!(c, '|' | ':' | '-' | '\u{2014}'))
        .rsplit('|')
        .next()
        .unwrap_or_default()
}

/// Whether `label` closes `before`, the text ahead of a name on its line.
fn labeled_by(before: &str, labels: &[&str]) -> bool {
    labels.iter().any(|label| {
        let n = label.split_whitespace().count();
        closing_words(before, n) == *label
    })
}

/// Words that make a label name people working for an organization: a
/// "crew member" or a "board member" is not the member a plan is about.
const STAFF_GROUP_WORDS: &[&str] = &["crew", "staff", "board", "team", "committee"];

/// Whether a label names the one a document is about.
fn subject_label(label: &str) -> bool {
    let words = label
        .split(|c: char| !c.is_alphanumeric())
        .collect::<Vec<_>>();
    words.iter().any(|word| SUBJECT_WORDS.contains(word))
        && !words.iter().any(|word| STAFF_GROUP_WORDS.contains(word))
}

/// The header cell over a table row's cell at byte `at`: the line above,
/// when it is a row of the same table, read at the same column.
fn column_header(line: &str, above: Option<&str>, at: usize) -> Option<String> {
    let above = above?;
    if !line.trim_start().starts_with('|') || !above.trim_start().starts_with('|') {
        return None;
    }
    let column = line[..at].matches('|').count();
    let header = above.split('|').nth(column)?.trim();
    (!header.is_empty() && !header.chars().all(|c| matches!(c, '-' | ':' | ' ')))
        .then(|| header.to_owned())
}

/// The label that opens `line`, as the document writes it, when `loose` is
/// in the list it introduces: "Mechanics: Linnea Hart, Joaquin Moncrieff".
/// Only names and separators may stand between the label and the name, so
/// a name the line's prose goes on to mention is not under the label:
/// "Prepared by: John Smith for Jane Doe".
fn listed_under(line: &str, loose: &str) -> Option<String> {
    let (label, rest) = line.split_once(':')?;
    let label = normalize_loosely(label);
    if label.split_whitespace().count() > 3 {
        return None;
    }
    let tokens = rest.split_whitespace().collect::<Vec<_>>();
    let start = (0..tokens.len()).find(|&start| {
        (start + 1..=tokens.len().min(start + 10))
            .any(|end| normalize_loosely(&tokens[start..end].join(" ")) == loose)
    })?;
    tokens[..start]
        .iter()
        .all(|word| {
            let bare = word.trim_matches(|c: char| !c.is_alphanumeric());
            bare.is_empty() || bare.starts_with(char::is_uppercase) || matches!(bare, "and" | "s")
        })
        .then_some(label)
}

/// Whether `line`, as the document writes it, gives the person `loose`
/// names a post right after their name: "Jane Doe, Chief Inspector",
/// "Jane Doe (Conservator)", "| Jane Doe | Winemaker |". Every word up to
/// the post must read as part of a title - capitalized, or "of", "and" or
/// "the" -
/// so a post in the prose that follows a name belongs to someone else:
/// "Jane Doe spoke to the manager".
fn titled_after(line: &str, loose: &str) -> bool {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    (0..tokens.len()).any(|start| {
        (start + 1..=tokens.len().min(start + 10)).any(|end| {
            normalize_loosely(&tokens[start..end].join(" ")) == loose
                && post_follows(&tokens[end..])
        })
    })
}

/// Whether a post opens `words`, after nothing but title words and
/// separators.
fn post_follows(words: &[&str]) -> bool {
    for word in words.iter().take(4) {
        let bare = word.trim_matches(|c: char| !c.is_alphanumeric());
        if bare.is_empty() {
            continue;
        }
        let lower = bare.to_lowercase();
        if POST_WORDS.contains(&lower.as_str()) {
            return true;
        }
        if !(bare.starts_with(char::is_uppercase) || matches!(lower.as_str(), "of" | "and" | "the"))
        {
            return false;
        }
    }
    false
}

/// Whether a line is a signature block's "Title:" field - not a field
/// whose label merely starts with the word ("Title of work:").
fn is_title_field(line: &str) -> bool {
    line.trim_start()
        .strip_prefix("title")
        .is_some_and(|rest| rest.trim_start().starts_with(':'))
}

/// Whether a line ends a signature block's "By:" ("By:", "By:\u{2014}").
fn signs_by(line: &str) -> bool {
    line.trim()
        .trim_end_matches(|c: char| c.is_whitespace() || matches!(c, ':' | '-' | '_' | '\u{2014}'))
        == "by"
}

/// Whether the document's own wording gives a person a party's role - a
/// label, a defined term, a customer cue. Not the issuer's or sender's,
/// which a letterhead or a signature gives the staff who sign, and not an
/// issued document's buyer, whose "Buyer:" names its purchasing contact.
fn labeled_party(scope: &ValidationScope<'_>, name: &str, class: DocumentClass) -> bool {
    labeled_role(scope, name).is_some_and(|role| stands_as_party(role, class))
}

/// Whether a role a label gives makes its name a party in a document of
/// `class`: see [`labeled_party`].
fn stands_as_party(role: PartyRole, class: DocumentClass) -> bool {
    match role {
        PartyRole::Issuer | PartyRole::Sender => false,
        PartyRole::Buyer => class != DocumentClass::Issued,
        _ => true,
    }
}

/// Whether a table's column header names a party's role, as a field's
/// label would: the names in a "Recipient" or "Tenant" column are the
/// document's parties, whatever else their row says.
fn header_names_party(header: &str, class: DocumentClass) -> bool {
    LABELED_ROLES
        .into_iter()
        .filter(|role| stands_as_party(*role, class))
        .any(|role| {
            role_words(role)
                .0
                .iter()
                .any(|word| label_names(header, word))
        })
}

/// What the document makes of a person the reply named: someone acting for
/// an organization when their name follows a staff label, a column header
/// or a line label naming a job, or a signature mark; carries a post right
/// after it or right before it; or is the "Name:" of a By:/Title: block. The
/// one the document is about when a label, a column header or a "between"
/// clause says so - which outranks everything else. A purchase order's or
/// an invoice's "Buyer:" is staff as well.
fn standing_of(scope: &ValidationScope<'_>, name: &str, class: DocumentClass) -> Standing {
    let loose = normalize_loosely(name);
    if loose.split_whitespace().count() < 2 {
        return Standing::Unknown;
    }
    let issued_staff: &[&str] = if class == DocumentClass::Issued {
        &["buyer", "sales rep", "salesperson", "contact"]
    } else {
        &[]
    };
    let units = scope.context_units().collect::<Vec<_>>();
    let mut staff = false;
    let mut signed = false;
    for (number, unit) in units.iter().enumerate() {
        let text = normalize_loosely(&unit.text);
        for at in whole_positions(&text, "between") {
            if !whole_positions(window_forward(&text, at, 300), &loose).is_empty() {
                return Standing::Subject;
            }
        }
        let lines = unit.text.lines().map(normalize_loosely).collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            for at in whole_positions(line, &loose) {
                let before = &line[..at];
                // The row above in the unit, else the header row of the
                // table a row unit belongs to.
                let above = match index.checked_sub(1) {
                    Some(previous) => lines.get(previous).cloned(),
                    None => unit
                        .table_header
                        .filter(|header| *header != unit.ordinal)
                        .and_then(|header| scope.unit_at(header))
                        .and_then(|header| header.text.lines().next())
                        .map(normalize_loosely),
                };
                let header = column_header(line, above.as_deref(), at);
                if subject_label(label_cell(before))
                    || header.as_deref().is_some_and(|header| {
                        subject_label(header) || header_names_party(header, class)
                    })
                {
                    return Standing::Subject;
                }
                let posted_after = unit
                    .text
                    .lines()
                    .nth(index)
                    .is_some_and(|written| titled_after(written, &loose));
                let posted_before = POST_WORDS.contains(&closing_words(before, 1).as_str());
                let signed_above = index > 0 && signs_by(&lines[index - 1]);
                // The block's "Title:" may open the next unit.
                let titled_below = match lines.get(index + 1) {
                    Some(next) => is_title_field(next),
                    None => units.get(number + 1).is_some_and(|next| {
                        is_title_field(&normalize_loosely(
                            next.text.lines().next().unwrap_or_default(),
                        ))
                    }),
                };
                let signature_block =
                    labeled_by(before, &["name"]) && (signed_above || titled_below);
                let staff_header = header.as_deref().is_some_and(|header| {
                    STAFF_LABELS.contains(&header)
                        || header.split_whitespace().any(|word| {
                            POST_WORDS.contains(&word) || STAFF_GROUP_WORDS.contains(&word)
                        })
                });
                let staff_opening = unit
                    .text
                    .lines()
                    .nth(index)
                    .and_then(|written| listed_under(written, &loose))
                    .is_some_and(|label| STAFF_LABELS.contains(&label.as_str()));
                if labeled_by(before, STAFF_LABELS)
                    || labeled_by(before, issued_staff)
                    || staff_header
                    || staff_opening
                    || posted_after
                    || posted_before
                    || signature_block
                {
                    staff = true;
                }
                // A signature mark before the name, or the column it sits
                // in: "/s/ Jane Doe", or a "Signature" column's cell.
                if labeled_by(before, SIGNATURE_MARKS)
                    || header
                        .as_deref()
                        .is_some_and(|header| has_cue(header, SIGNATURE_MARKS))
                {
                    signed = true;
                }
            }
        }
    }
    // A signature alone says someone signed, not for whom. It counts as
    // acting for another only where the document names an organization at
    // its head to sign for, and never on correspondence or a form, which
    // their own writers and filers sign.
    let signed_for_another = signed
        && !matches!(
            class,
            DocumentClass::Letter | DocumentClass::Email | DocumentClass::Form
        )
        && labeled_role(scope, name) != Some(PartyRole::Sender)
        && (header_organization(scope).is_some() || letterhead_organization(scope).is_some());
    if staff || signed_for_another {
        Standing::Staff
    } else {
        Standing::Unknown
    }
}

/// The one organization a first-page letterhead unit names, as the
/// document writes it, with its unit: "Kestrel Ridge Aero Services -
/// Repair Station Certificate KRA" over a record whose table names the
/// aircraft's owner too.
fn letterhead_organization<'a>(scope: &ValidationScope<'a>) -> Option<(String, &'a EvidenceUnit)> {
    scope
        .context_units()
        .filter(|unit| unit.features.position.letterhead && !unit.running)
        .find_map(|unit| match unit.features.organizations.as_slice() {
            [only] => written_span(unit, only).map(|span| (span, unit)),
            _ => None,
        })
}

/// A name that ended a sentence, without the sentence's full stop:
/// "Pellworth Water Company." is "Pellworth Water Company", while the stop
/// of an abbreviation stays ("Inc.", "Corp.", "N.A.").
fn without_sentence_stop(name: &str) -> String {
    let Some(stem) = name.strip_suffix('.') else {
        return name.to_owned();
    };
    let last = stem.rsplit(' ').next().unwrap_or_default();
    if last.contains('.') || last.chars().count() <= 4 {
        name.to_owned()
    } else {
        stem.to_owned()
    }
}

/// An organization's name as the document goes on to write it: the index
/// ends a name on its organization word, so "Saltash Point Museum of Art"
/// reads as "Saltash Point Museum"; up to three capitalized words after an
/// "of" belong to it.
fn with_of_phrase(unit: &EvidenceUnit, span: &str) -> String {
    let Some(rest) = unit
        .text
        .lines()
        .find_map(|line| line.find(span).map(|at| &line[at + span.len()..]))
    else {
        return span.to_owned();
    };
    // "of" as the line writes it: a caption set in capitals says "OF".
    let Some((of, tail)) = rest
        .get(..4)
        .filter(|head| head.eq_ignore_ascii_case(" of "))
        .map(|head| (head.trim(), &rest[4..]))
    else {
        return span.to_owned();
    };
    let mut words = Vec::new();
    for word in tail.split_whitespace().take(3) {
        let bare = word.trim_end_matches([',', ';', '.', ':', ')']);
        if !bare.chars().next().is_some_and(char::is_uppercase) {
            break;
        }
        words.push(bare);
        if bare.len() != word.len() {
            break;
        }
    }
    if words.is_empty() {
        span.to_owned()
    } else {
        format!("{span} {of} {}", words.join(" "))
    }
}

/// The two organizations a first-page "between" clause names, in its
/// order, as the document writes them, with their unit.
fn between_organizations<'a>(scope: &ValidationScope<'a>) -> Vec<(String, &'a EvidenceUnit)> {
    between_organizations_where(scope, |_| true, false)
}

/// [`between_organizations`], from a "between" that introduces the document
/// itself ([`opens_this_agreement`]), never one a recital names, and only
/// when the clause names two organizations and no more: in "Acme Inc., a
/// wholly owned subsidiary of Parent Corp., and Beta LLC" the second named
/// is no side of the agreement, and which of the others is cannot be told.
fn opening_clause_organizations<'a>(
    scope: &ValidationScope<'a>,
) -> Vec<(String, &'a EvidenceUnit)> {
    between_organizations_where(scope, opens_this_agreement, true)
}

/// Whether the words before a "between" open the document's own clause -
/// "This Outsourcing Agreement ... is made and entered into ... by and" -
/// rather than name another agreement: "WHEREAS, the Share Purchase
/// Agreement".
fn opens_this_agreement(before: &str) -> bool {
    let words = before.split_whitespace().collect::<Vec<_>>();
    let recent = &words[words.len().saturating_sub(40)..];
    if recent
        .iter()
        .any(|word| matches!(*word, "whereas" | "recital" | "recitals"))
    {
        return false;
    }
    recent.iter().enumerate().any(|(at, word)| {
        *word == "this"
            && recent[at + 1..].iter().take(6).any(|next| {
                matches!(
                    next.trim_matches(|c: char| !c.is_alphanumeric()),
                    "agreement"
                        | "amendment"
                        | "contract"
                        | "addendum"
                        | "lease"
                        | "license"
                        | "licence"
                )
            })
    })
}

/// [`between_organizations`], from the "between"s whose preceding words
/// (loosely normalized, in the same unit) `accept` takes; with
/// `exactly_two`, from one whose clause names no third organization.
fn between_organizations_where<'a>(
    scope: &ValidationScope<'a>,
    accept: impl Fn(&str) -> bool,
    exactly_two: bool,
) -> Vec<(String, &'a EvidenceUnit)> {
    let Some(first_page) = scope.context_units().map(|unit| unit.page).min() else {
        return Vec::new();
    };
    for unit in scope.context_units().filter(|unit| unit.page == first_page) {
        let text = normalize_loosely(&unit.text);
        for at in whole_positions(&text, "between") {
            if !accept(&text[..at]) {
                continue;
            }
            let after = window_forward(&text, at, 300);
            let mut named = unit
                .features
                .organizations
                .iter()
                .filter_map(|organization| {
                    whole_positions(after, organization)
                        .first()
                        .map(|position| (*position, organization))
                })
                .collect::<Vec<_>>();
            named.sort_by_key(|(position, _)| *position);
            let mut found: Vec<(String, &EvidenceUnit)> = Vec::new();
            for (_, organization) in named {
                if found
                    .iter()
                    .any(|(known, _)| normalize_loosely(known) == *organization)
                {
                    continue;
                }
                if let Some(span) = written_span(unit, organization) {
                    found.push((span, unit));
                }
                if found.len() == 2 && !exactly_two {
                    return found;
                }
            }
            if found.len() == 2 {
                return found;
            }
        }
    }
    Vec::new()
}

/// Roles a notice or a letter gives the one it is addressed to.
const ADDRESSED_ROLES: &[PartyRole] = &[PartyRole::Addressee, PartyRole::Recipient];

/// Labels of a field that names who a notice or a letter is to. An
/// "Attn:" line names the person who reads it for the organization it is
/// addressed to, never the addressee itself.
const ADDRESSEE_LABELS: &[&str] = &["to", "addressee"];

/// The person or organization a first-page "To:" field names, as the
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
                .chain(&unit.features.organizations)
                .filter(|known| {
                    !known.is_empty()
                        && opening.starts_with(known.as_str())
                        && opening[known.len()..]
                            .chars()
                            .next()
                            .is_none_or(|next| !next.is_alphanumeric())
                })
                .max_by_key(|known| known.len())?;
            // A name its own colon follows is the label of the next field,
            // which the layout ran into this one: "To: Inspection Number:
            // 1695478" names no one.
            if value
                .split_once(':')
                .is_some_and(|(label, _)| words(label) == words(named))
            {
                return None;
            }
            let span = written_span(unit, named)?;
            (!is_first_name_alone(&span)).then(|| (span, unit, line.to_owned()))
        })
}

/// The one organization the first page of the context names before any
/// line with a customer cue, as the document writes it, and its unit.
fn header_organization<'a>(scope: &ValidationScope<'a>) -> Option<(String, &'a EvidenceUnit)> {
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
        let before_kind = unit.text.lines().filter_map(organization_before_kind);
        for organization in unit
            .features
            .organizations
            .iter()
            .cloned()
            .chain(before_kind)
        {
            if found
                .iter()
                .any(|(known, _)| normalize_loosely(known) == organization)
            {
                continue;
            }
            if let Some(span) = written_span(unit, &organization) {
                found.push((span, unit));
            }
        }
    }
    match found.as_slice() {
        [only] => Some(only.clone()),
        _ => None,
    }
}

/// The organization a header line names before the kind of document it
/// heads, which the index reads as one run of capitals with it:
/// "Halvorsen Fixture Works LLC INVOICE".
fn organization_before_kind(line: &str) -> Option<String> {
    let words = line.split_whitespace().collect::<Vec<_>>();
    let kind = words
        .iter()
        .rev()
        .take_while(|word| {
            TYPE_NOUNS.contains(
                &word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
                    .as_str(),
            )
        })
        .count();
    if kind == 0 || kind == words.len() {
        return None;
    }
    crate::index::organizations_in(&words[..words.len() - kind].join(" ")).pop()
}

/// The words of a unit that read, loosely, as `loose`: the document's own
/// spelling of a name the index found. A "The" that opens the name's line
/// is the name's own ("THE MCGRAW-HILL COMPANIES, INC.,"), as a caption, a
/// letterhead or an address writes it: the name ends its line or its cell,
/// or a comma closes it. A sentence that opens with the name ("The Acme
/// Widget Company agrees ...") keeps its "The" to itself.
fn written_span(unit: &EvidenceUnit, loose: &str) -> Option<String> {
    unit.text.lines().find_map(|line| {
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        (0..tokens.len()).find_map(|start| {
            (start + 1..=tokens.len().min(start + 10)).find_map(|end| {
                let span = tokens[start..end].join(" ");
                (normalize_loosely(&span) == loose).then(|| {
                    let closed = end == tokens.len()
                        || tokens[end] == "|"
                        || span.ends_with([',', ';', ':']);
                    let span = span.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '"'));
                    let opens_line = |at: usize| at == 0 || tokens[at - 1] == "|";
                    match start.checked_sub(1) {
                        Some(article)
                            if tokens[article].eq_ignore_ascii_case("the")
                                && opens_line(article)
                                && closed =>
                        {
                            format!("{} {span}", tokens[article])
                        }
                        _ => span.to_owned(),
                    }
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
/// or header, and is seldom labeled as such; this is that structural cue.
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
                let labeled = unit
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
                            && !labeled
                            && !under_label
                            && !has_cue(&line[..at], CUSTOMER_CUES);
                    }
                    if labeled || has_cue(line, CUSTOMER_CUES) {
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

/// Whether the reply read both sides of an issued document and took the
/// party at `head` for the one it is billed to: it gave that party a
/// receiving role and called another party of the document the issuer,
/// the seller or the vendor. Where a name stands on a page is no word of
/// the document's, and makes no issuer of a party so read: an addressee
/// heads a dispute over an invoice typed as a sheet, and a billed agency
/// the page of an invoice whose issuer's name runs over two lines. A reply
/// whose other side the document does not name - a name it writes another
/// way, a phrase of the billhead - read no issuer to set against the page,
/// and neither did one that called the other party its sender.
fn read_as_billed_by_another(cast: &[CastMember], head: usize) -> bool {
    cast.get(head).is_some_and(|party| {
        party
            .proposed_role
            .is_some_and(|role| RECEIVING_ROLES.contains(&role))
    }) && cast.iter().enumerate().any(|(at, other)| {
        at != head
            && matches!(
                other.proposed_role,
                Some(PartyRole::Issuer | PartyRole::Seller | PartyRole::Vendor)
            )
    })
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

/// Whether a word is a role's own label - "Tenant", "Buyer", "Plaintiff" -
/// or a field's - "Name", "Account" - which names no one.
fn is_role_word(word: &str) -> bool {
    BRINGING_SIDE.contains(&word)
        || ANSWERING_SIDE.contains(&word)
        || matches!(word, "name" | "account")
        || crate::cues::PARTY_LABELS.contains(&word)
        || PartyRole::ALL
            .iter()
            .any(|role| role_words(*role).0.contains(&word))
}

/// Words a caption labels the side bringing a case with.
const BRINGING_SIDE: &[&str] = &["plaintiff", "petitioner", "appellant", "claimant"];

/// Words a caption labels the side answering a case with.
const ANSWERING_SIDE: &[&str] = &["defendant", "respondent", "appellee"];

/// Words a court's, a tribunal's or an arbitration's heading names it by.
const FORUM_WORDS: &[&str] = &["court", "tribunal", "arbitration"];

/// Words a caption sets its two sides against each other with: "v.",
/// "vs.", "versus", "- against -".
const VERSUS_WORDS: &[&str] = &["v", "vs", "versus", "against"];

/// The two sides of the case a document is filed in, by their places in
/// the cast: a party the document labels a plaintiff, petitioner,
/// appellant or claimant right beside its name ("Plaintiff Gilchrist
/// Harbor Marine Supply, LLC") and one it labels a defendant, respondent
/// or appellee ("Tavistock Boatworks, Inc., Defendant"), in a document
/// that sets the case out as "A v. B" under a forum's heading on its first
/// page ("SUPERIOR COURT OF THE STATE OF WASHINGTON"), wherever the heading
/// stands there ([`caption_units`]). A document that only mentions a case -
/// a law firm's invoice for the matter - has no such heading.
fn caption_sides(
    scope: &ValidationScope<'_>,
    context: &ScopeView,
    cast: &[CastMember],
) -> Option<(usize, usize)> {
    let caption = caption_units(scope);
    if caption.is_empty() {
        return None;
    }
    // A caption laid out as a table keeps its labels in cells of their
    // own: the cell marks are no words.
    let segments = context
        .segments()
        .iter()
        .map(|segment| normalize_loosely(&segment.replace('|', " ")))
        .collect::<Vec<_>>();
    let versus = segments.iter().any(|text| {
        VERSUS_WORDS
            .iter()
            .any(|word| !whole_positions(text, word).is_empty())
    });
    if !versus {
        return None;
    }
    let labeled = |name: &str, side: &[&str]| {
        let name = normalize_loosely(name);
        segments.iter().any(|text| {
            whole_positions(text, &name).into_iter().any(|at| {
                let before = text[..at].split_whitespace().next_back();
                let after = text[at + name.len()..].split_whitespace().next();
                [before, after]
                    .into_iter()
                    .flatten()
                    .any(|word| side.contains(&word))
            })
        })
    };
    let bringing = cast
        .iter()
        .position(|party| labeled(&party.name, BRINGING_SIDE));
    let answering = cast
        .iter()
        .position(|party| labeled(&party.name, ANSWERING_SIDE));
    match (bringing, answering) {
        (Some(bringing), Some(answering)) if bringing != answering => Some((bringing, answering)),
        _ => caption_block_sides(&caption, cast),
    }
}

/// The two sides as a caption sets them out in a block, when no label stands
/// beside a name: the parties named between the court's heading and "v.",
/// down to the label that closes them ("BARCLAYS CAPITAL INC., MERRILL
/// LYNCH ... AND MORGAN STANLEY & CO., INCORPORATED, Plaintiffs, v.
/// THEFLYONTHEWALL.COM, INC., Defendant."), and the parties named after it
/// down to theirs - a label in the plural too, and the caption laid out as
/// a table ([`CaptionLayout`]). Each side is the first of the cast the
/// caption names on it.
fn caption_block_sides(caption: &[&EvidenceUnit], cast: &[CastMember]) -> Option<(usize, usize)> {
    let text = caption_text(caption);
    let layout = CaptionLayout::of(&text)?;
    let named_in = |side: std::ops::Range<usize>| {
        cast.iter()
            .enumerate()
            .filter_map(|(index, party)| {
                whole_positions(&text, &normalize_loosely(&party.name))
                    .into_iter()
                    .find(|at| side.contains(at))
                    .map(|at| (at, index))
            })
            .min()
            .map(|(_, index)| index)
    };
    let bringing = named_in(layout.bringing)?;
    let answering = named_in(layout.answering)?;
    (bringing != answering).then_some((bringing, answering))
}

/// Where a caption sets out its two sides, as stretches of its text
/// ([`caption_text`]): the bringing side's names stand between the forum's
/// heading and "v.", the answering side's after "v.", and each side's
/// label closes its names where the caption has one.
#[derive(Clone, Debug)]
struct CaptionLayout {
    /// Where the bringing side's names stand: from the forum's heading to
    /// the side's label, or to "v.".
    bringing: std::ops::Range<usize>,
    /// Where the answering side's names stand: from "v." to the side's
    /// label, or as far as a caption's answering side can run.
    answering: std::ops::Range<usize>,
    /// Whether the answering side's label closes its names: a caption with
    /// no label sets out its sides, but nothing says where the second ends.
    answering_closed: bool,
}

impl CaptionLayout {
    /// The layout of a caption's text; `None` when it sets no two sides
    /// against each other under a forum's heading.
    fn of(text: &str) -> Option<Self> {
        // Where any of `words` stands in the caption, singular or plural.
        let positions = |words: &[&str], from: usize, to: usize| {
            words
                .iter()
                .flat_map(|word| {
                    let mut found = whole_positions(text, word);
                    found.extend(whole_positions(text, &format!("{word}s")));
                    found
                })
                .filter(|at| (from..to).contains(at))
                .collect::<Vec<_>>()
        };
        let forum = positions(FORUM_WORDS, 0, text.len()).into_iter().min()?;
        let versus = VERSUS_WORDS
            .iter()
            .flat_map(|word| whole_positions(text, word))
            .filter(|at| *at > forum)
            .min()?;
        // The bringing side's label closes its names, before "v."; the
        // answering side's follows "v." within the caption.
        let reach = (versus + CAPTION_REACH).min(text.len());
        let bringing_label = positions(BRINGING_SIDE, forum, versus).into_iter().max();
        let answering_label = positions(ANSWERING_SIDE, versus, reach).into_iter().min();
        let (bringing_end, answering_end) = match (bringing_label, answering_label) {
            (Some(bringing), Some(answering)) => (bringing, answering),
            // A caption that labels neither side - "LUXO AS, -against- THE
            // WALT DISNEY COMPANY" - still sets them on either side of its
            // "against".
            (None, None) => (versus, reach),
            // A reader takes a scanned caption's column of labels out of
            // line: "LUXO AS, / -against- / THE WALT DISNEY COMPANY, PIXAR
            // / Plaintiff, Case No. ... / ANIMATION STUDIOS ... /
            // Defendants." The bringing side's label is still the
            // caption's, under "v." instead of over it, and "v." still
            // divides the sides.
            (None, Some(answering)) if !positions(BRINGING_SIDE, versus, reach).is_empty() => {
                (versus, answering)
            }
            // One side labeled and the other's label nowhere: a body's
            // "against" and "the Defendant", not a caption's.
            _ => return None,
        };
        Some(Self {
            bringing: forum..bringing_end,
            answering: versus..answering_end,
            answering_closed: answering_label.is_some(),
        })
    }
}

/// How far after "v." a caption's answering side and its label can run.
const CAPTION_REACH: usize = 400;

/// How many of the first page's opening units name a forum as its heading
/// however they are set: a filing stamp over the court's name, a form's
/// title that names its court.
const FORUM_HEAD_UNITS: usize = 8;

/// How far down its first page a court document's forum heading can
/// stand, in units, when it is set as a heading: a pleading opens with
/// its counsel's block - their names, firm, address and telephone,
/// "Attorneys for Defendants" - and the court's name heads the caption
/// under it.
const FORUM_HEADING_REACH: usize = 20;

/// How many units a caption runs for, from its forum's heading down.
const CAPTION_UNITS: usize = 12;

/// The units a court document's caption is read from: the first page's,
/// from its forum's heading for as far as a caption runs. The heading is
/// the first of the page's opening units that names a court, a tribunal
/// or an arbitration, or, under a block that fills those, the first
/// that names one on a line in capitals ("UNITED STATES DISTRICT COURT"):
/// counsel's block above the heading is no part of the caption and takes
/// none of its room, while a sentence further down that mentions a court
/// heads nothing. Empty when no forum heads the page.
fn caption_units<'a>(scope: &ValidationScope<'a>) -> Vec<&'a EvidenceUnit> {
    let Some(first_page) = scope.context_units().map(|unit| unit.page).min() else {
        return Vec::new();
    };
    let page = || {
        scope
            .context_units()
            .filter(move |unit| unit.page == first_page)
    };
    let names_forum = |text: &str| {
        let text = normalize_loosely(text);
        FORUM_WORDS
            .iter()
            .any(|word| !whole_positions(&text, word).is_empty())
    };
    let heading = page()
        .take(FORUM_HEADING_REACH)
        .enumerate()
        .position(|(at, unit)| {
            if at < FORUM_HEAD_UNITS {
                names_forum(&unit.text)
            } else {
                unit.text.lines().any(heads_a_court)
            }
        });
    let Some(heading) = heading else {
        return Vec::new();
    };
    let caption = page().skip(heading).take(CAPTION_UNITS).collect::<Vec<_>>();
    // Under a block that fills the page's opening units, a court's heading
    // heads a caption only where one is set out under it.
    if heading >= FORUM_HEAD_UNITS && !sets_out_a_caption(&caption_cells(&caption)) {
        return Vec::new();
    }
    caption
}

/// Whether a line reads as a court's heading: set in capitals, with no
/// figure, a forum's word ending the line or a cell of it, or standing
/// before "of", "for" or "in" or its own comma ("UNITED STATES DISTRICT
/// COURT", "COURT OF APPEAL FOR ONTARIO", "SUPREME COURT, STATE OF NEW
/// YORK"). An address's "15 COURT STREET" and a contract's "SECTION 14.
/// ARBITRATION" name no forum a caption is headed by.
fn heads_a_court(line: &str) -> bool {
    if !in_capitals(line) || line.chars().any(|character| character.is_ascii_digit()) {
        return false;
    }
    line.split('|').any(|cell| {
        let tokens = cell.split_whitespace().collect::<Vec<_>>();
        tokens.iter().enumerate().any(|(at, token)| {
            let word = caption_word(token);
            let forum = FORUM_WORDS.contains(&word.as_str())
                || word
                    .strip_suffix('s')
                    .is_some_and(|word| FORUM_WORDS.contains(&word));
            forum
                && (token.ends_with(',')
                    || tokens.get(at + 1).is_none_or(|next| {
                        matches!(caption_word(next).as_str(), "" | "of" | "for" | "in")
                    }))
        })
    })
}

/// A caption's lines as its cells: each line, or each cell of a table's
/// row, as its words, with the marks a caption's rule is drawn with left
/// out (")", "§", ":"). Cells with no words are left out.
fn caption_cells<'a>(caption: &[&'a EvidenceUnit]) -> Vec<(&'a EvidenceUnit, Vec<&'a str>)> {
    caption
        .iter()
        .flat_map(|unit| {
            unit.text
                .lines()
                .flat_map(|line| line.split('|'))
                .map(move |cell| (*unit, caption_cell_words(cell)))
        })
        .filter(|(_, words)| !words.is_empty())
        .collect()
}

/// A caption's cell as its words, the marks of its rule left out.
fn caption_cell_words(cell: &str) -> Vec<&str> {
    cell.split_whitespace()
        .filter(|word| !word.chars().all(|mark| matches!(mark, ')' | '§' | ':')))
        .collect()
}

/// A caption's word as a label is compared: without its marks, in lower
/// case ("Plaintiffs," is "plaintiffs", "-against-" is "against").
fn caption_word(word: &str) -> String {
    word.trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase()
}

/// Whether a caption's cell sets out one of `labels` as a side's label,
/// singular or plural: alone, closing the cell after a name's comma
/// ("Plaintiff,", "BETA LENDING INC., Defendant."), or opening it with its
/// own mark ("Plaintiffs, JUDGE CHIN", "Defendants. Notice is hereby
/// given ..."), as a reader runs a caption's column into the next. A
/// sentence that speaks of "the Lead Plaintiffs" or ends on "the
/// Defendant." labels no side.
fn labels_a_side(words: &[&str], labels: &[&str]) -> bool {
    let label = |word: &str| {
        let word = caption_word(word);
        labels.contains(&word.as_str())
            || word
                .strip_suffix('s')
                .is_some_and(|word| labels.contains(&word))
    };
    let closes = words.split_last().is_some_and(|(last, before)| {
        label(last) && before.last().is_none_or(|word| word.ends_with(','))
    });
    let opens = words
        .first()
        .is_some_and(|first| label(first) && first.ends_with([',', '.']));
    closes || opens
}

/// Whether a caption's cell holds its versus word and nothing else:
/// "-against-", "- against -", "v.", "vs.".
fn versus_alone(words: &[&str]) -> bool {
    let words = words
        .iter()
        .map(|word| caption_word(word))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    matches!(words.as_slice(), [word] if VERSUS_WORDS.contains(&word.as_str()))
}

/// Whether a caption's cells set out a case as a caption does: an "In re"
/// opens one, a versus word stands alone in one, or a side's label is set
/// out in one ([`labels_a_side`]). A letter's inside address on a court's
/// street, or a notice's sentence that reserves rights "against" a tenant,
/// sets out none.
fn sets_out_a_caption(cells: &[(&EvidenceUnit, Vec<&str>)]) -> bool {
    let sides = [BRINGING_SIDE, ANSWERING_SIDE, DEBTOR_LABELS].concat();
    cells.iter().any(|(_, words)| {
        matches!(words.as_slice(), [first, second, ..]
            if caption_word(first) == "in" && caption_word(second) == "re")
            || versus_alone(words)
            || labels_a_side(words, &sides)
    })
}

/// A caption's units as its sides are read from them: their text, loosely
/// normalized, in one line, a table's cell marks left out.
fn caption_text(caption: &[&EvidenceUnit]) -> String {
    caption
        .iter()
        .map(|unit| normalize_loosely(&unit.text.replace('|', " ")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The side of a case the reply left out, read from the caption itself:
/// when the first page sets the case out as "A v. B" under a forum's
/// heading, one side's label closes its names and the other's follows
/// ([`CaptionLayout`]), and a party is named on one side only, the first
/// organization the caption names on the other side is the other side.
/// Returned with the unit that names it and whether it is the bringing
/// side (`true`) or the answering one.
fn caption_other_side<'a>(
    scope: &ValidationScope<'a>,
    parties: &[ValidatedParty],
) -> Option<(String, &'a EvidenceUnit, bool)> {
    let caption = caption_units(scope);
    let text = caption_text(&caption);
    let layout = CaptionLayout::of(&text)?;
    let named_in = |side: std::ops::Range<usize>| {
        parties.iter().any(|party| {
            !party.copied
                && !party.signatory
                && whole_positions(&text, &normalize_loosely(&party.name))
                    .into_iter()
                    .any(|at| side.contains(&at))
        })
    };
    // A caption that labels neither side names none the reply did not:
    // nothing says where its second side ends.
    if !layout.answering_closed {
        return None;
    }
    let (side, bringing) = match (
        named_in(layout.bringing.clone()),
        named_in(layout.answering.clone()),
    ) {
        (true, false) => (layout.answering, false),
        (false, true) => (layout.bringing, true),
        _ => return None,
    };
    caption.into_iter().find_map(|unit| {
        unit.features.organizations.iter().find_map(|organization| {
            let loose = normalize_loosely(organization);
            // The court's own heading names no side: "SNOHOMISH
            // COUNTY" under "SUPERIOR COURT" is the forum, not the
            // plaintiff.
            if names_a_forum(&loose) {
                return None;
            }
            whole_positions(&text, &loose)
                .into_iter()
                .any(|at| side.contains(&at))
                .then(|| written_span(unit, organization))
                .flatten()
                // The index stops a name at its organization's word:
                // "First National Bank of Omaha" is read whole.
                .map(|span| {
                    (
                        without_sentence_stop(&with_of_phrase(unit, &span)),
                        unit,
                        bringing,
                    )
                })
        })
    })
}

/// Words a court's, a tribunal's or a jurisdiction's heading is made of.
const JURISDICTION_WORDS: &[&str] = &[
    "county",
    "district",
    "circuit",
    "judicial",
    "parish",
    "borough of",
    "in and for",
    "state of",
    "commonwealth of",
    "province of",
];

/// Whether a name is the forum's own: a court, a tribunal, or the county,
/// district or state it sits in.
fn names_a_forum(loose: &str) -> bool {
    FORUM_WORDS
        .iter()
        .any(|word| !whole_positions(loose, word).is_empty())
        || JURISDICTION_WORDS.iter().any(|words| {
            if words.contains(' ') {
                loose.contains(words)
            } else {
                !whole_positions(loose, words).is_empty()
            }
        })
}

/// Words a bankruptcy caption labels the party its case is about with.
const DEBTOR_LABELS: &[&str] = &["debtor", "debtors"];

/// How many lines and cells a bankruptcy caption can run for from its "In
/// re" to the label that closes it: the debtor's name, which a table's
/// cells may split, its chapter and case number, "Jointly Administered".
const IN_RE_LINES: usize = 8;

/// The debtor of the bankruptcy case a document is filed in, as its
/// caption names it under a forum's heading on the first page: "In re:"
/// opens the caption, the debtor's name follows, and the label "Debtor."
/// or "Debtors." closes it, in a line or a cell of its own or after the
/// name's comma ("In re: / DELTA MILLS, INC., et al., / Debtors."). The
/// caption has no "v." and no second side: the case is the debtor's, and
/// whoever files in it - a creditors' committee, a creditor, the trustee -
/// files in the debtor's case.
///
/// The debtor is the first of `parties` the caption names there, by the
/// name validation gave it; when the reply named none of them, the first
/// organization the caption names there, as the caption writes it. Either
/// comes with the unit that names it. An "In re" with no such label - "IN
/// RE: ADAMS GOLF, INC., SECURITIES LITIGATION" - names a matter, not a
/// debtor, and a "debtor" in the sentences under it is no label.
fn caption_debtor<'a>(
    scope: &ValidationScope<'a>,
    parties: &[ValidatedParty],
) -> Option<(String, &'a EvidenceUnit)> {
    // The caption's lines, a table's cells each a line of their own, with
    // the marks a caption's rule is drawn with left out: ")", "§", ":".
    let caption = caption_units(scope);
    let lines = caption_cells(&caption);
    let bare = caption_word;
    let opens = lines.iter().position(|(_, words)| match words.as_slice() {
        [first, second, ..] => bare(first) == "in" && bare(second) == "re",
        _ => false,
    })?;
    // The label alone on its line, or closing a line after the name's
    // comma: "GSC GROUP, INC., et al., Debtors." A sentence that ends on
    // "the Debtors." labels no one.
    let label_at = |words: &[&str]| {
        let (last, before) = words.split_last()?;
        (DEBTOR_LABELS.contains(&bare(last).as_str())
            && before.last().is_none_or(|word| word.ends_with(',')))
        .then_some(before.len())
    };
    let (closes, label) = lines
        .iter()
        .enumerate()
        .skip(opens)
        .take(IN_RE_LINES)
        .find_map(|(at, (_, words))| label_at(words).map(|label| (at, label)))?;
    // What stands between "In re" and the label, each word with its unit.
    let named = (opens..=closes)
        .flat_map(|at| {
            let (unit, words) = &lines[at];
            let from = if at == opens { 2 } else { 0 };
            let to = if at == closes { label } else { words.len() };
            words
                .get(from..to.max(from))
                .unwrap_or_default()
                .iter()
                .map(move |word| (*unit, *word))
        })
        .collect::<Vec<_>>();
    let written = named
        .iter()
        .map(|(_, word)| *word)
        .collect::<Vec<_>>()
        .join(" ");
    let loose = normalize_loosely(&written);
    if let Some(party) = parties
        .iter()
        .filter_map(|party| {
            whole_positions(&loose, &normalize_loosely(&party.name))
                .first()
                .map(|at| (*at, party))
        })
        .min_by_key(|(at, _)| *at)
        .map(|(_, party)| party)
    {
        let unit = party
            .evidence
            .first()
            .and_then(|id| scope.index.unit(id))
            .or_else(|| named.first().map(|(unit, _)| *unit))?;
        return Some((party.name.clone(), unit));
    }
    let organization = crate::index::organizations_in(&written)
        .into_iter()
        .find(|organization| !names_a_forum(organization))?;
    // The organization's words as the caption writes them, a "The" before
    // them the name's own.
    (0..named.len()).find_map(|start| {
        (start + 1..=named.len().min(start + 10)).find_map(|end| {
            let span = named[start..end]
                .iter()
                .map(|(_, word)| *word)
                .collect::<Vec<_>>()
                .join(" ");
            (normalize_loosely(&span) == organization).then(|| {
                let span = span.trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '"'));
                let name = match start.checked_sub(1).map(|article| named[article].1) {
                    Some(article) if article.eq_ignore_ascii_case("the") => {
                        format!("{article} {span}")
                    }
                    _ => span.to_owned(),
                };
                (without_sentence_stop(&name), named[start].0)
            })
        })
    })
}

/// Room an extension takes at the end of a filename, its dot included.
const EXTENSION_ROOM: usize = 6;

/// A shorter type for a filename that names two parties but would not fit
/// them: a compound title's first part when that part is itself a kind of
/// document ("Settlement Agreement" of "Settlement Agreement and Mutual
/// Release"), or of an agreement named for what it joins, the agreement it
/// ends on ("Product Development Agreement" of "Exclusive License and
/// Product Development Agreement"). `None` when the name fits or there is
/// no such part.
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
    let type_words = words(document_type);
    // Only an agreement named for what it joins and nothing else: "Second
    // Amendment to Software License and Support Agreement" is an amendment.
    if type_words
        .last()
        .is_some_and(|last| matches!(last.as_str(), "agreement" | "contract"))
        && !type_words
            .iter()
            .any(|word| matches!(word.as_str(), "of" | "to" | "for" | "on" | "under"))
    {
        let at = lowered.rfind(" and ")?;
        let shorter = document_type[at + " and ".len()..].trim();
        return (words(shorter).len() >= 2 && length(shorter) <= room).then(|| shorter.to_owned());
    }
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
    let totaled = |(at, _, line): &&(usize, String, String)| {
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
        _ => found.iter().find(totaled).or_else(|| found.first()),
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
/// last labeled total) or, for any other kind, the first amount labeled
/// as what the document is for (a price, a fee, rent, a salary, a
/// principal) or as its total: in a field, a two-cell row, or the words
/// just before it on a line. `None` when no amount is labeled so.
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
    // No labeled amount: one a first-page title states - "$350,000,000
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
/// field labeled as this kind's number ("Invoice No.: 7731-B", "| Policy
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
    // A field labeled as this kind's number; else, on the first page, any
    // field labeled as a number that is not a phone's, a tax id's or a
    // page's.
    let kinds = identifier_labels(&head);
    let first_page = scope.context_units().map(|unit| unit.page).min()?;
    let labeled = |unit: &'a EvidenceUnit, own_kind: bool| -> Option<ReadIdentifier<'a>> {
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
    // or on the next line; a table whose column is so labeled.
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
        .find_map(|unit| (!kinds.is_empty()).then(|| labeled(unit, true)).flatten())
        .or_else(|| (!kinds.is_empty()).then(on_a_line).flatten())
        .or_else(|| (!kinds.is_empty()).then(in_a_column).flatten())
        .or_else(|| {
            scope
                .context_units()
                .filter(|unit| unit.page == first_page)
                .find_map(|unit| labeled(unit, false))
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
    // labeled by its column's header ("| Invoice No. | ... |").
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
            proposed_role: party.role,
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
    use crate::domain::{DocumentSource, KeyFact, PageOrigin, PartyFact};
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

    /// A stated type that is a whole title line reads as the line's kind
    /// phrase: without the field it runs into, the code or the words an
    /// "and" joins before the kind, or a party the reply named after it.
    /// What a connector joins stays ("Approval of Closure Plan"), and so
    /// does an agreement's title, which can be named for a party.
    #[test]
    fn a_stated_type_reads_as_its_titles_kind_phrase() {
        let parties = [
            "Saltmarsh Point Homeowners Association",
            "Briarport Library District",
        ];
        for (value, kind) in [
            ("Fieldnote S2 Product Launch Plan", "Product Launch Plan"),
            ("Spring Wholesale Availability and Price List", "Price List"),
            (
                "Newsletter of the Saltmarsh Point Homeowners Association",
                "Newsletter",
            ),
            (
                "CHANGE ORDER Distribution: Briarport Library District / Stonebridge",
                "CHANGE ORDER",
            ),
            (
                "Public Information Statement from Saltmarsh Point Homeowners Association",
                "Public Information Statement",
            ),
            ("Advisory Opinion 1997-5", "Advisory Opinion"),
            (
                "Notice of Hearing: Motion for Summary Judgment",
                "Notice of Hearing",
            ),
            ("letter of 4 April 2014", "letter"),
            (
                "Public Information Statement 24-45",
                "Public Information Statement",
            ),
        ] {
            assert_eq!(stated_title(value, &parties), kind, "{value}");
        }
        for kept in [
            "W-9 Form",
            "10-K Annual Report",
            "1099-MISC Form",
            "Profit and Loss Statement",
            "Health and Safety Plan",
            "Bank Statement",
            "Credit Card Statement",
            "2025 Annual Report",
            "Phase 1 Decision Letter",
            "Section 5 Objection Letter",
            "Notice of Hearing:",
            "Approval of Closure Plan",
            "EPA Approval of Closure Plan",
            "Action by Unanimous Written Consent of the Board of Directors",
            "Notice of Default and Reservation of Rights",
            "Saltmarsh Point Homeowners Association Services Agreement",
            "Amendment to Lease with Briarport Library District",
            "Guide to the Code",
            "Details for Improvement Notice",
        ] {
            assert_eq!(stated_title(kept, &parties), kept);
        }
    }

    /// A kind a connector joins to the title's head past an article or a
    /// name set in capitals is what the document answers or approves, not
    /// what it is: the title stays whole, as it does when the kind follows
    /// the connector directly. A sentence is no title, and the kind phrase
    /// it runs on to still reads.
    #[test]
    fn a_stated_title_keeps_what_its_connector_joins_past_an_article_or_a_name() {
        let parties = ["Harbour Advisory Committee"];
        for kept in [
            "Response to the QRA Engagement on Adjustment of Harbour Statistics",
            "RESPONSE TO THE QRA ENGAGEMENT ON ADJUSTMENT OF HARBOUR STATISTICS",
            "Approval of SEP Completion Report",
            "Details for the Improvement Notice",
        ] {
            assert_eq!(stated_title(kept, &parties), kept);
        }
        for (value, kind) in [
            (
                "Receipt is acknowledged of the Letter of Notification",
                "Letter of Notification",
            ),
            // No connector before the name in capitals: the line's kind
            // phrase, as ever.
            (
                "Quay Harbour QRA Statistics Engagement Plan",
                "Statistics Engagement Plan",
            ),
        ] {
            assert_eq!(stated_title(value, &parties), kind, "{value}");
        }
    }

    /// "Report generated: ..." is the stamp a system put on a sheet, not
    /// its title: the heading above it names the kind.
    #[test]
    fn a_machine_stamp_is_no_type() {
        let index = EvidenceIndex::build(&DocumentSource::from_pages(vec![
            crate::domain::SourcePage::new(
                1,
                "HARVEST LOG\n\nReport generated: 2026-10-02\n\nRowanbrae Vineyards\n\n\
                 Block | Variety | Tons\nA1 | Pinot Noir | 4.2",
                PageOrigin::PlainText,
            ),
        ]));
        let context = whole(&index);
        let facts = ModelFacts {
            document_type: Some("Report generated".into()),
            type_evidence: vec![id_of(&index, "Report generated")],
            document_date: Some("2026-10-02".into()),
            date_evidence: vec![id_of(&index, "Report generated")],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &context, &index);
        assert_ne!(
            outcome.proposal.document_type.as_deref(),
            Some("Report Generated"),
            "{:?}",
            outcome.reasons
        );
        assert!(
            outcome.reasons.contains(&ReviewReason::TypeUnsupported)
                || outcome.reasons.contains(&ReviewReason::TypeInferred),
            "{:?}",
            outcome.reasons
        );
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
             shelving for the bakery counter, invoice INV-10438, totaling $1,248.00."
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

    /// InternBench's mixed-signature-region, as its text reads: the
    /// amendment takes effect on the last signature, and each party's
    /// signature block carries its own date.
    const SIGNED_AMENDMENT: &str = "FIRST AMENDMENT TO SOFTWARE LICENSE AGREEMENT\n\n\
This First Amendment (this \"Amendment\") is between Corvane Analytics Inc. (\"Licensor\") and \
Pellworth County Water Authority (\"Licensee\"), and amends the Software License and Services \
Agreement between them dated September 30, 2024. This Amendment is effective on the date of the \
last signature below.\n\n\
IN WITNESS WHEREOF, the parties have signed this Amendment on the dates written below.\n\n\
CORVANE ANALYTICS INC.\nBy:\nName: Rosalind Achterberg\nTitle: Chief Revenue Officer\nDate: August 18, 2026\n\n\
PELLWORTH COUNTY WATER AUTHORITY\nBy:\nName: Barnaby Quist\nTitle: General Manager\nDate: August 21, 2026";

    fn amendment_facts(index: &EvidenceIndex, date: &str, cited: &str) -> ModelFacts {
        let parties_line = id_of(index, "This First Amendment");
        ModelFacts {
            document_type: Some("First Amendment to Software License Agreement".into()),
            type_evidence: vec![id_of(index, "FIRST AMENDMENT")],
            document_date: Some(date.into()),
            date_role: Some(DateRole::Amendment),
            date_evidence: vec![id_of(index, cited)],
            parties: vec![
                party(
                    "Corvane Analytics Inc.",
                    Some(PartyRole::Licensor),
                    &[parties_line.clone()],
                ),
                party(
                    "Pellworth County Water Authority",
                    Some(PartyRole::Licensee),
                    &[parties_line],
                ),
            ],
            ..ModelFacts::default()
        }
    }

    /// An agreement is made when its last party signs it: an earlier
    /// signature's date gives way to the latest.
    #[test]
    fn an_earlier_signature_gives_way_to_the_last() {
        let index = index_of(SIGNED_AMENDMENT);
        let outcome = check(
            amendment_facts(&index, "2026-08-18", "August 18"),
            &whole(&index),
            &index,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-08-21")
        );
        // The last signature's own date stands as it is.
        let outcome = check(
            amendment_facts(&index, "2026-08-21", "August 21"),
            &whole(&index),
            &index,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-08-21")
        );
    }

    #[test]
    fn a_loose_signing_date_gives_way_to_the_effective_date() {
        for signed in ["June 2, 2017", "June 2,2017", "02-Jun-17"] {
            let text =
                format!("LEASE AGREEMENT\n\nSigned on {signed}\n\nEffective Date: July 1, 2017");
            let (outcome, _) = facts_for(&text, |index| ModelFacts {
                document_type: Some("Lease Agreement".into()),
                document_date: Some("2017-06-02".into()),
                date_role: Some(DateRole::Effective),
                date_evidence: vec![id_of(index, "Signed on")],
                ..ModelFacts::default()
            });
            assert_eq!(
                outcome.proposal.document_date.as_deref(),
                Some("2017-07-01"),
                "{signed}"
            );
        }
    }

    #[test]
    fn a_loose_signing_date_keeps_its_stated_role_and_review() {
        let (outcome, _) = facts_for("LEASE AGREEMENT\n\nSigned on June 2,2017", |index| {
            ModelFacts {
                document_type: Some("Lease Agreement".into()),
                document_date: Some("2017-06-02".into()),
                date_role: Some(DateRole::Effective),
                date_evidence: vec![id_of(index, "Signed on")],
                ..ModelFacts::default()
            }
        });
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Execution));
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));
    }

    #[test]
    fn a_loose_earlier_signature_gives_way_to_the_last() {
        let text = SIGNED_AMENDMENT.replace("August 18, 2026", "August 18,2026");
        let index = index_of(&text);
        let outcome = check(
            amendment_facts(&index, "2026-08-18", "August 18"),
            &whole(&index),
            &index,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-08-21")
        );
    }

    /// A date the document also states outside its signature blocks is
    /// not only a signature's, and stands: here the amendment says it is
    /// dated the day the first party signed.
    #[test]
    fn a_signature_date_the_body_states_too_stands() {
        let text = SIGNED_AMENDMENT.replace(
            "This Amendment is effective on the date of the last signature below.",
            "This Amendment is dated August 18, 2026.",
        );
        let index = index_of(&text);
        let outcome = check(
            amendment_facts(&index, "2026-08-18", "is dated August 18"),
            &whole(&index),
            &index,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-08-18")
        );
    }

    /// A stated effective date set beside the signatures is the document's
    /// own date, not an earlier party's signature, and stands against a
    /// later signing date.
    #[test]
    fn an_effective_date_beside_the_signatures_stands() {
        let text = SIGNED_AMENDMENT.replace(
            "Title: Chief Revenue Officer\nDate: August 18, 2026",
            "Title: Chief Revenue Officer\nEffective Date: August 18, 2026",
        );
        let index = index_of(&text);
        let outcome = check(
            amendment_facts(&index, "2026-08-18", "Effective Date"),
            &whole(&index),
            &index,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-08-18")
        );
    }

    /// A later date in the signature region that names another event - an
    /// expiration - is no signature, and never replaces a signing date.
    #[test]
    fn a_later_expiration_beside_the_signatures_is_not_taken() {
        let text = SIGNED_AMENDMENT.replace(
            "Title: General Manager\nDate: August 21, 2026",
            "Title: General Manager\nDate: August 18, 2026\nExpiration Date: August 21, 2027",
        );
        let index = index_of(&text);
        let outcome = check(
            amendment_facts(&index, "2026-08-18", "August 18"),
            &whole(&index),
            &index,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-08-18")
        );
    }

    /// "Never Letter alone": a bare generic word is no type a person could
    /// file by, and goes to review.
    #[test]
    fn a_bare_letter_is_no_type() {
        let index = index_of(
            "January 9, 2026\n\nSilverbirch Ceramics Inc.\n\nThis letter confirms our engagement.\n\n\
             Halloran Ostrowski CPAs LLP",
        );
        let facts = ModelFacts {
            document_type: Some("Letter".into()),
            type_evidence: vec![id_of(&index, "This letter")],
            document_date: Some("2026-01-09".into()),
            date_evidence: vec![id_of(&index, "January 9")],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert!(outcome.reasons.contains(&ReviewReason::TypeMissing));
        assert_ne!(outcome.status, ProposalStatus::Ready);
    }

    /// CUAD's cuad-18 of the third sample: the reply typed "Agreement",
    /// citing a later page's heading "AGREEMENT", under the cover page's
    /// "Strategic Alliance Agreement".
    #[test]
    fn a_bare_kind_citing_a_bare_heading_is_read_as_the_cover_title() {
        let index = index_of(
            "Strategic Alliance Agreement\n\nIntricon Corporation\n\nand\n\n\
             Dynamic Hearing Pty Ltd\n\n\
             This Agreement is made on October 1, 2008 between Intricon Corporation and \
             Dynamic Hearing Pty Ltd.\n\nAGREEMENT\n\n1. Definitions.",
        );
        let line = id_of(&index, "This Agreement");
        let heading = index
            .units()
            .iter()
            .find(|unit| unit.text.trim() == "AGREEMENT")
            .expect("the bare heading")
            .id
            .clone();
        let facts = ModelFacts {
            document_type: Some("Agreement".into()),
            type_evidence: vec![heading],
            document_date: Some("2008-10-01".into()),
            date_role: Some(DateRole::Execution),
            date_evidence: vec![line.clone()],
            parties: vec![
                party("Intricon Corporation", None, &[line.clone()]),
                party("Dynamic Hearing Pty Ltd", None, &[line]),
            ],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Strategic Alliance Agreement")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::TypeMissing));

        // Never from another document's name: an order "Pursuant to Master
        // License Agreement" is no Master License Agreement.
        let index = index_of(
            "Pursuant to Master License Agreement\n\n\
             This order is made on October 1, 2008 between Intricon Corporation and \
             Dynamic Hearing Pty Ltd.\n\nAGREEMENT\n\n1. Services.",
        );
        let line = id_of(&index, "This order");
        let heading = index
            .units()
            .iter()
            .find(|unit| unit.text.trim() == "AGREEMENT")
            .expect("the bare heading")
            .id
            .clone();
        let facts = ModelFacts {
            document_type: Some("Agreement".into()),
            type_evidence: vec![heading],
            document_date: Some("2008-10-01".into()),
            date_evidence: vec![line],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Agreement"));
        assert!(outcome.reasons.contains(&ReviewReason::TypeMissing));
    }

    /// CUAD's cuad-15 of the third sample: "AMENDMENT NO. 3", which names
    /// the agreement it amends only in its first paragraph, typed a bare
    /// "Amendment". Which agreement it changes is the name's point.
    #[test]
    fn a_bare_amendment_goes_to_review() {
        let index = index_of(
            "AMENDMENT NO. 3\n\nDated as of February 19, 2007\n\n\
             Reference is hereby made to that certain Wireless Content License Agreement Number 12965 \
             between Fox Mobile Entertainment, Inc. and Glu Mobile Inc.",
        );
        let line = id_of(&index, "Reference");
        let facts = ModelFacts {
            document_type: Some("Amendment".into()),
            type_evidence: vec![id_of(&index, "AMENDMENT NO")],
            document_date: Some("2007-02-19".into()),
            date_evidence: vec![id_of(&index, "Dated as of")],
            parties: vec![
                party("Glu Mobile Inc.", None, &[line.clone()]),
                party("Fox Mobile Entertainment, Inc.", None, &[line]),
            ],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert!(
            outcome.reasons.contains(&ReviewReason::TypeMissing),
            "{:?}",
            outcome.reasons
        );
        assert_ne!(outcome.status, ProposalStatus::Ready);
    }

    /// InternBench's agreement-two-column-footnotes, as Qwen3.5-2B read it:
    /// typed "Agreement", citing its title. The title is read whole, its
    /// "Supply" too, which also ends many a company's name.
    #[test]
    fn a_bare_agreement_is_read_as_the_title_it_cites() {
        let index = index_of(
            "SEED PRODUCTION AND SUPPLY AGREEMENT\n\n\
             This Seed Production and Supply Agreement (this \"Agreement\") is dated as of March 2, 2026 \
             and is made between\nLarkhaven Seed Company, a Nebraska corporation (\"Company\"), and \
             Prairie Wren Growers Cooperative, an Iowa cooperative\nassociation (\"Grower\").\n\n\
             Counties, Iowa. The parties agree as follows:",
        );
        let line = id_of(&index, "This Seed");
        let facts = ModelFacts {
            document_type: Some("Agreement".into()),
            type_evidence: vec![id_of(&index, "SEED PRODUCTION")],
            document_date: Some("2026-03-02".into()),
            date_role: Some(DateRole::Execution),
            date_evidence: vec![line.clone()],
            parties: vec![
                party(
                    "Larkhaven Seed Company",
                    Some(PartyRole::Recipient),
                    &[line.clone()],
                ),
                party(
                    "Prairie Wren Growers Cooperative",
                    Some(PartyRole::Sender),
                    &[line],
                ),
            ],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        // The title, fitted to a filename with both parties' names: an
        // agreement keeps what it ends on after its last "and".
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Supply Agreement")
        );
        assert_eq!(outcome.reasons, Vec::<ReviewReason>::new());
        assert_eq!(outcome.status, ProposalStatus::Ready);
    }

    /// CUAD's cuad-20 of the held-out sample: a cover page's title wrapped
    /// over two headings, the model citing the second, "AGREEMENT".
    #[test]
    fn a_title_wrapped_over_two_headings_is_read_whole() {
        let index = index_of(
            "JOINT DEVELOPMENT AND MARKETING\n\nAGREEMENT\n\nBETWEEN\n\nLECLANCHE SA\n\nAND\n\n\
             OAK RIDGE ENERGY TECHNOLOGIES, INC.\n\n\
             This Joint Development and Marketing Agreement is made on April 6, 2014 between \
             Leclanche SA and Oak Ridge Energy Technologies, Inc.",
        );
        let line = id_of(&index, "This Joint");
        let facts = ModelFacts {
            document_type: Some("Agreement".into()),
            type_evidence: vec![id_of(&index, "AGREEMENT")],
            document_date: Some("2014-04-06".into()),
            date_role: Some(DateRole::Execution),
            date_evidence: vec![line.clone()],
            parties: vec![
                party("Leclanche SA", None, &[line.clone()]),
                party("Oak Ridge Energy Technologies, Inc.", None, &[line]),
            ],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Joint Development and Marketing Agreement")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::TypeMissing));

        // A company's heading above "AGREEMENT" is a letterhead, not half
        // of the title: the bare "Agreement" waits for a person.
        let index = index_of(
            "WRENFIELD SUPPLY\n\nAGREEMENT\n\n\
             This Agreement is made on April 6, 2014 between Wrenfield Supply and \
             Oak Ridge Energy Technologies, Inc.",
        );
        let line = id_of(&index, "This Agreement");
        let facts = ModelFacts {
            document_type: Some("Agreement".into()),
            type_evidence: vec![id_of(&index, "AGREEMENT")],
            document_date: Some("2014-04-06".into()),
            date_role: Some(DateRole::Execution),
            date_evidence: vec![line],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Agreement"));
        assert!(outcome.reasons.contains(&ReviewReason::TypeMissing));
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
    fn a_role_the_document_does_not_support_decides_the_relation_only_unopposed_and_in_review() {
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
        // claim nothing to stand on, and nothing to contradict it: the
        // reply's roles name the notice's party, and the name waits for a
        // person, saying why.
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
        assert_eq!(parties[1].document_role, None);
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        assert_eq!(
            outcome.proposal.parties,
            vec!["Cresthaven Court Holdings LLC"]
        );
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        assert!(
            outcome.reasons.contains(&ReviewReason::PartyUnsupported),
            "{:?}",
            outcome.reasons
        );
    }

    /// An invoice that names only its customer names no one: the bill-to
    /// company is never the party an invoice is filed under.
    #[test]
    fn an_invoice_naming_only_its_customer_files_under_its_header_never_its_customer() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.parties.remove(0);
        let outcome = check(facts, &whole(&index), &index);
        // The customer is never the filename's party; the organization at
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
        // With no header organization before the customer, no one.
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

    /// An identifier in a table is labeled by its column.
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
                                           Ridge Bakery, Inc., invoice INV-10438, totaling \
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

    /// A party's name or an identifier the reply wrote with a stray
    /// invisible character matches the document's plain text, and is
    /// written as the document writes it, without the character.
    #[test]
    fn a_stray_invisible_character_in_a_name_or_an_identifier_is_not_written() {
        let index = index_of(INVOICE);
        let mut facts = invoice_facts(&index);
        facts.parties[0].name = "Halvorsen\u{200b} Fixture Works LLC".into();
        facts.identifier = Some("INV-\u{200b}10438".into());
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        let validated = outcome.facts.as_ref().unwrap();
        assert_eq!(validated.identifier.as_deref(), Some("INV-10438"));
        assert_eq!(
            outcome.proposal.parties.first().map(String::as_str),
            Some("Halvorsen Fixture Works LLC")
        );
        assert!(
            !outcome.proposal.description.contains('\u{200b}'),
            "{:?}",
            outcome.proposal.description
        );
        assert!(
            outcome
                .proposal
                .description
                .starts_with("Invoice from Halvorsen Fixture Works LLC"),
            "{}",
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
        // A line break a PDF writes as "\r\n", and the soft or typographic
        // hyphen it marks the break with.
        let crlf = QUOTED.replace("dis-\n", "dis-\r\n");
        let soft = QUOTED.replace("dis-\n", "dis\u{ad}\n");
        let typographic = QUOTED.replace("dis-\n", "dis\u{2010}\r\n");
        for (text, subject) in [
            (
                QUOTED,
                "refrigerated display shelving for the bakery counter",
            ),
            (QUOTED, "refrigerated display shelving"),
            (crlf.as_str(), "refrigerated display shelving"),
            (soft.as_str(), "refrigerated display shelving"),
            (typographic.as_str(), "refrigerated display shelving"),
        ] {
            let (outcome, _) = facts_for(text, |index| ModelFacts {
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
        const TOTALED: &str = "Nimbus Orchard Supply Co.\n\nINVOICE\n\n\
Invoice date: April 30, 2025\n\nBill to Atlas Threadworks LLC\n\nTotal: $1,248.00";
        let unclaimed = with_key_facts(TOTALED, &[]).proposal.description;
        assert!(unclaimed.contains("totaling $1,248.00"), "{unclaimed}");
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
            let outcome = with_key_facts(TOTALED, &[(fact, cited)]);
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
            let outcome = with_key_facts(TOTALED, &[(fact, Some("Total"))]);
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
        // A mark the line holds once is no license to decorate the type with
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
            "Invoice INV-2048 from Nimbus Orchard Supply Co., totaling $1,248.00."
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
        // An issued document's amount is its last labeled total, not a
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
        assert!(description.contains("totaling $1,080.00"), "{description}");
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
        assert!(order.contains("totaling $84,438.00"), "{order}");
        let rent = described(
            "NOTICE OF RENT INCREASE\n\n| Charge | Current | New |\n\
| Base rent | $1,845.00 | $1,965.00 |\n\nThe base rent increase is $120.00 per month.",
            "Notice of Rent Increase",
        );
        assert!(rent.contains("base rent of $1,965.00"), "{rent}");
        // An amount alone in its row is labeled by its header.
        let loss = described(
            "PROPERTY LOSS NOTICE\n\n| Estimated amount of loss | Property damaged |\n\
| $38,500.00 | Flooring, two mixers |",
            "Property Loss Notice",
        );
        assert!(loss.contains("$38,500.00"), "{loss}");
        // With no labeled amount, the one a first-page title states.
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
    fn a_name_cut_short_is_completed_to_the_organization_and_its_scattered_capitals_tidied() {
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
    fn a_person_named_by_their_post_gives_way_to_the_organization_at_the_head() {
        const MINUTES: &str = "CAIRNFIELD COOPERATIVE GROCERS\n\n\
Minutes of the Regular Meeting of the Board of Directors\n\nSeptember 17, 2025\n\n\
Directors present: Beckett Halloran (President), Mabel Trevelyan (Vice President)\n\n\
President Beckett Halloran called the meeting to order at 6:32 p.m.";
        let (outcome, _) = facts_for(MINUTES, |index| ModelFacts {
            document_type: Some("Minutes of the Regular Meeting of the Board of Directors".into()),
            type_evidence: vec![id_of(index, "Minutes of")],
            document_date: Some("2025-09-17".into()),
            date_evidence: vec![id_of(index, "September")],
            parties: vec![party(
                "Beckett Halloran",
                Some(PartyRole::Recipient),
                &[id_of(index, "Directors present")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["CAIRNFIELD COOPERATIVE GROCERS"],
            "{:?}",
            outcome.proposal
        );
        let facts = outcome.facts.unwrap();
        assert!(facts.parties.iter().any(|party| party.signatory));
    }

    #[test]
    fn the_one_a_column_header_names_as_the_subject_keeps_the_name() {
        const FORM: &str = "| WESTBROOK KETTLE BREWING CO. | BENEFITS ENROLLMENT CHANGE FORM |\n\n\
| Employee name | Employee ID |\n| --- | --- |\n| Annika Solberg-Reyes | WK-0417 |\n\n\
| Employee signature | Date signed |\n| --- | --- |\n| /s/ Annika Solberg-Reyes | 06/29/2026 |";
        let (outcome, _) = facts_for(FORM, |index| ModelFacts {
            document_type: Some("Benefits Enrollment Change Form".into()),
            type_evidence: vec![id_of(index, "| WESTBROOK")],
            parties: vec![party(
                "Annika Solberg-Reyes",
                Some(PartyRole::Employee),
                &[id_of(index, "| Annika")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Annika Solberg-Reyes"],
            "{:?}",
            outcome.proposal
        );
    }

    #[test]
    fn signatories_alone_give_way_to_the_organizations_between_which_it_is_made() {
        const AMENDMENT: &str = "FIRST AMENDMENT TO SOFTWARE LICENSE AGREEMENT\n\n\
This First Amendment is made between Corvane Analytics Inc. and Pellworth Water Company.\n\n\
CORVANE ANALYTICS INC.\nBy:\nName: Rosalind Achterberg\nTitle: Chief Revenue Officer\n\n\
PELLWORTH WATER COMPANY\nBy:\nName: Barnaby Quist\nTitle: General Manager";
        let (outcome, _) = facts_for(AMENDMENT, |index| ModelFacts {
            document_type: Some("First Amendment to Software License Agreement".into()),
            type_evidence: vec![id_of(index, "FIRST AMENDMENT")],
            parties: vec![
                party(
                    "Rosalind Achterberg",
                    Some(PartyRole::Other),
                    &[id_of(index, "CORVANE")],
                ),
                party(
                    "Barnaby Quist",
                    Some(PartyRole::Other),
                    &[id_of(index, "PELLWORTH")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Corvane Analytics Inc.", "Pellworth Water Company"],
            "{:?}",
            outcome.proposal
        );
    }

    #[test]
    fn a_signer_alone_gives_way_to_the_letterhead_and_its_full_name() {
        const RECORD: &str = "AIRCRAFT MAINTENANCE RECORD\n\n\
Kestrel Ridge Aero Services - Repair Station Certificate KRA\n\n\
| Registration | Owner |\n| --- | --- |\n| N0612X | Highmeadow Air Charter LLC |\n\n\
| Signature | Date of return to service |\n| --- | --- |\n| /s/ Arlo Sinclair-Ray | 08/14/2026 |";
        let (outcome, _) = facts_for(RECORD, |index| ModelFacts {
            document_type: Some("Aircraft Maintenance Record".into()),
            type_evidence: vec![id_of(index, "AIRCRAFT")],
            parties: vec![party(
                "Arlo Sinclair-Ray",
                Some(PartyRole::Other),
                &[id_of(index, "| /s/")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Kestrel Ridge Aero Services"],
            "{:?}",
            outcome.proposal
        );
        // The column says who signed, with no mark in the cell.
        let unmarked = RECORD.replace("/s/ ", "");
        let (outcome, _) = facts_for(&unmarked, |index| ModelFacts {
            document_type: Some("Aircraft Maintenance Record".into()),
            type_evidence: vec![id_of(index, "AIRCRAFT")],
            parties: vec![party(
                "Arlo Sinclair-Ray",
                Some(PartyRole::Other),
                &[id_of(index, "| Arlo")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Kestrel Ridge Aero Services"],
            "{:?}",
            outcome.proposal
        );
        const REPORT: &str = "Saltash Point Museum of Art\n\nLOAN CONDITION REPORT\n\n\
| Examined by | Delphine Moncrieff, Associate Conservator of Paintings |";
        let (outcome, _) = facts_for(REPORT, |index| ModelFacts {
            document_type: Some("Loan Condition Report".into()),
            type_evidence: vec![id_of(index, "LOAN")],
            parties: vec![party(
                "Delphine Moncrieff",
                Some(PartyRole::Recipient),
                &[id_of(index, "| Examined")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Saltash Point Museum of Art"],
            "{:?}",
            outcome.proposal
        );
    }

    #[test]
    fn a_labeled_recipient_who_also_signs_stays_a_party() {
        const NOTICE: &str = "NOTICE OF POLICY CHANGE\n\nRecipient: Jane Doe\n\n\
Your coverage changes on March 1, 2026.\n\nAcknowledged: /s/ Jane Doe";
        let reply = |index: &EvidenceIndex| ModelFacts {
            document_type: Some("Notice of Policy Change".into()),
            type_evidence: vec![id_of(index, "NOTICE")],
            parties: vec![party(
                "Jane Doe",
                Some(PartyRole::Recipient),
                &[id_of(index, "Recipient")],
            )],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(NOTICE, reply);
        assert_eq!(
            outcome.proposal.parties,
            vec!["Jane Doe"],
            "{:?}",
            outcome.proposal
        );
        let facts = outcome.facts.unwrap();
        assert!(!facts.parties.iter().any(|party| party.signatory));
    }

    #[test]
    fn a_labeled_party_is_not_taken_along_when_staff_give_way() {
        const NOTICE: &str = "HARBOR FREIGHT COURIERS LLC\n\nNOTICE OF RATE CHANGE\n\n\
Recipient: Jane Doe\n\nRates change on March 1, 2026.\n\nPrepared by: John Smith";
        let (outcome, _) = facts_for(NOTICE, |index| ModelFacts {
            document_type: Some("Notice of Rate Change".into()),
            type_evidence: vec![id_of(index, "NOTICE")],
            parties: vec![
                party(
                    "Jane Doe",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Recipient")],
                ),
                party(
                    "John Smith",
                    Some(PartyRole::Sender),
                    &[id_of(index, "Prepared")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Jane Doe"],
            "{:?}",
            outcome.proposal
        );
    }

    #[test]
    fn someone_who_signs_their_own_letter_stays_its_party() {
        const LETTER: &str = "LETTER OF RESIGNATION\n\nFrom: Jane Doe\n\nMarch 2, 2026\n\n\
I resign from my position effective March 31, 2026.\n\n/s/ Jane Doe";
        let (outcome, _) = facts_for(LETTER, |index| ModelFacts {
            document_type: Some("Letter of Resignation".into()),
            type_evidence: vec![id_of(index, "LETTER")],
            parties: vec![party(
                "Jane Doe",
                Some(PartyRole::Sender),
                &[id_of(index, "From:")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Jane Doe"],
            "{:?}",
            outcome.proposal
        );
        let facts = outcome.facts.unwrap();
        assert!(!facts.parties.iter().any(|party| party.signatory));
    }

    #[test]
    fn only_a_title_field_makes_a_name_a_signature_block() {
        assert!(is_title_field("title: chief revenue officer"));
        assert!(is_title_field("  title : general manager"));
        assert!(!is_title_field("title of work: harbor at dusk"));
        assert!(!is_title_field("titled: yes"));
        const FORM: &str =
            "COPYRIGHT REGISTRATION FORM\n\nName: Jane Doe\nTitle of work: Harbor at Dusk";
        let (outcome, _) = facts_for(FORM, |index| ModelFacts {
            document_type: Some("Copyright Registration Form".into()),
            type_evidence: vec![id_of(index, "COPYRIGHT")],
            parties: vec![party("Jane Doe", None, &[id_of(index, "Name:")])],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Jane Doe"],
            "{:?}",
            outcome.proposal
        );
    }

    #[test]
    fn a_crew_member_is_staff_not_the_member_a_document_is_about() {
        assert!(!subject_label("crew member"));
        assert!(!subject_label("board member"));
        assert!(subject_label("member"));
        assert!(subject_label("employee signature"));
        assert_eq!(closing_words("| examined by |", 2), "examined by");
        assert!(signs_by("by:\u{2014}"));
        assert!(signs_by("by:"));
        assert_eq!(
            without_sentence_stop("Pellworth Water Company."),
            "Pellworth Water Company"
        );
        assert_eq!(
            without_sentence_stop("Corvane Analytics Inc."),
            "Corvane Analytics Inc."
        );
        assert_eq!(
            without_sentence_stop("Halden Bay National Bank, N.A."),
            "Halden Bay National Bank, N.A."
        );
    }

    #[test]
    fn a_signature_alone_is_not_signing_for_someone_else() {
        // A form's filer signs it: no organization stands to sign for.
        const FORM: &str = "COPYRIGHT REGISTRATION FORM\n\nName: Jane Doe\n\
Title of work: Harbor at Dusk\n\nSignature: /s/ Jane Doe";
        let (outcome, _) = facts_for(FORM, |index| ModelFacts {
            document_type: Some("Copyright Registration Form".into()),
            type_evidence: vec![id_of(index, "COPYRIGHT")],
            parties: vec![party("Jane Doe", None, &[id_of(index, "Name:")])],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.parties, vec!["Jane Doe"]);
        // Nor does a declaration that names no organization.
        const DECLARATION: &str = "DECLARATION OF RESIDENCY\n\nDated June 2, 2026\n\n\
I declare that I have lived at 14 Wren Street since 2019.\n\nSignature: /s/ Jane Doe";
        let (outcome, _) = facts_for(DECLARATION, |index| ModelFacts {
            document_type: Some("Declaration of Residency".into()),
            type_evidence: vec![id_of(index, "DECLARATION")],
            parties: vec![party("Jane Doe", None, &[id_of(index, "Signature:")])],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.parties, vec!["Jane Doe"]);
    }

    #[test]
    fn an_insurance_notice_is_for_its_insured_whoever_sends_it() {
        // The insured under a column header, on the insurer's form: the
        // insurer the reply calls its recipient does not send it.
        const LOSS: &str = "PROPERTY LOSS NOTICE Date reported: 02/16/2026\n\n\
Harrowmere Mutual Insurance Company - Claims Intake - 9 Granary Street, Hollis Bend, NH 03049\n\n\
| Named insured | Insured phone |\n| --- | --- |\n| Delacroix Bakehouse LLC | (603) 555-0119 |\n\n\
The insured must protect the property from further damage.";
        let (outcome, _) = facts_for(LOSS, |index| ModelFacts {
            document_type: Some("Property Loss Notice".into()),
            type_evidence: vec![id_of(index, "PROPERTY LOSS")],
            document_date: Some("2026-02-16".into()),
            date_evidence: vec![id_of(index, "PROPERTY LOSS")],
            parties: vec![
                party(
                    "Delacroix Bakehouse LLC",
                    Some(PartyRole::Client),
                    &[id_of(index, "| Delacroix")],
                ),
                party(
                    "Harrowmere Mutual Insurance Company",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Harrowmere")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Delacroix Bakehouse LLC"],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        // A "Named insured:" field: the producer the reply calls its issuer
        // is not who the notice is for.
        const CANCEL: &str = "Kestrel Casualty Company\n\n\
NOTICE OF CANCELLATION\n\nDate of notice: 08/12/2026\n\n\
Named insured: Brannock Tool & Die Inc.\nProducer: Ridley Cross Insurance Services, code 4471";
        let (outcome, _) = facts_for(CANCEL, |index| ModelFacts {
            document_type: Some("Notice of Cancellation".into()),
            type_evidence: vec![id_of(index, "NOTICE OF")],
            document_date: Some("2026-08-12".into()),
            date_evidence: vec![id_of(index, "Date of notice")],
            parties: vec![
                party(
                    "Brannock Tool & Die Inc.",
                    Some(PartyRole::Client),
                    &[id_of(index, "Named insured")],
                ),
                party(
                    "Ridley Cross Insurance Services",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "Named insured")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Brannock Tool & Die Inc."],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        // A crew member is no member a plan is about.
        assert!(label_names("named insured", "insured"));
        assert!(label_names("member", "member"));
        assert!(!label_names("crew member", "member"));
        assert!(!label_names("board member", "member"));
    }

    #[test]
    fn a_court_document_is_filed_between_the_two_sides_of_its_case() {
        const HEARING: &str = "SUPERIOR COURT OF THE STATE OF WASHINGTON\n\n\
GILCHRIST HARBOR MARINE SUPPLY, LLC, )\n\n\
a Washington limited liability company, ) No. 26-2-04417-1 Plaintiff, )\n\n\
v. ) NOTICE OF HEARING\n\n\
TAVISTOCK BOATWORKS, INC., ) a Washington corporation, ) Defendant. )\n\n\
AND TO: Tavistock Boatworks, Inc., Defendant, and its attorneys of record\n\n\
PLEASE TAKE NOTICE that Plaintiff Gilchrist Harbor Marine Supply, LLC will bring on \
for hearing its Motion to Compel.\n\nDATED this 29th day of July, 2026.";
        let (outcome, _) = facts_for(HEARING, |index| ModelFacts {
            document_type: Some("Notice of Hearing".into()),
            type_evidence: vec![id_of(index, "v. )")],
            document_date: Some("2026-07-29".into()),
            date_evidence: vec![id_of(index, "DATED")],
            parties: vec![
                party(
                    "Tavistock Boatworks, Inc.",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "AND TO")],
                ),
                party(
                    "Gilchrist Harbor Marine Supply, LLC",
                    Some(PartyRole::Client),
                    &[id_of(index, "PLEASE TAKE")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec![
                "Gilchrist Harbor Marine Supply, LLC",
                "Tavistock Boatworks, Inc."
            ],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        // The same words with no court over them only mention the case.
        let unheaded = HEARING.replacen(
            "SUPERIOR COURT OF THE STATE OF WASHINGTON",
            "Kavanagh Trask PLLC",
            1,
        );
        let (outcome, _) = facts_for(&unheaded, |index| ModelFacts {
            document_type: Some("Notice of Hearing".into()),
            type_evidence: vec![id_of(index, "v. )")],
            document_date: Some("2026-07-29".into()),
            date_evidence: vec![id_of(index, "DATED")],
            parties: vec![
                party(
                    "Tavistock Boatworks, Inc.",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "AND TO")],
                ),
                party(
                    "Gilchrist Harbor Marine Supply, LLC",
                    Some(PartyRole::Client),
                    &[id_of(index, "PLEASE TAKE")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_ne!(outcome.proposal.party_relation, PartyRelation::Between);
    }

    #[test]
    fn a_report_is_for_the_client_it_was_prepared_for_not_sent_to_it() {
        const REPORT: &str = "ANNUAL WATERSHED MONITORING REPORT\n\n\
Prepared for Cobalt Basin Water Authority, 900 Reservoir Road, Ashby, Vermont 05032 \
Prepared by Thornwick Basin Engineering, PLLC, 61 Foundry Street, Montpelier, Vermont 05602\n\n\
Thornwick Basin Engineering, PLLC monitored 40 stations with Bramblecote Aquatic Ecology LLC \
through the 2025 monitoring year.";
        let (outcome, _) = facts_for(REPORT, |index| ModelFacts {
            document_type: Some("Annual Watershed Monitoring Report".into()),
            type_evidence: vec![id_of(index, "ANNUAL")],
            parties: vec![
                party(
                    "Cobalt Basin Water Authority",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Prepared for")],
                ),
                party(
                    "Thornwick Basin Engineering, PLLC",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "Prepared for")],
                ),
                party(
                    "Bramblecote Aquatic Ecology LLC",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "monitored")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Cobalt Basin Water Authority"],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
    }

    #[test]
    fn a_record_its_staff_made_comes_from_their_letterhead_not_the_other_party() {
        const REPORT: &str = "Saltash Point Museum of Art\n\n\
Registration and Conservation - 1 Gallery Walk, Saltash Point, CA 94923\n\n\
LOAN CONDITION REPORT\n\n\
| Field | Entry |\n| --- | --- |\n\
| Date of examination | October 2, 2025 |\n\
| Examined by | Delphine Moncrieff, Associate Conservator of Paintings |\n\
| Lender | Haverford Family Collection |\n\
| Artist | Elias Brandvold (1871-1934) |";
        let (outcome, _) = facts_for(REPORT, |index| ModelFacts {
            document_type: Some("Loan Condition Report".into()),
            type_evidence: vec![id_of(index, "LOAN CONDITION")],
            document_date: Some("2025-10-02".into()),
            date_evidence: vec![id_of(index, "| Date of examination")],
            parties: vec![
                party(
                    "Haverford Family Collection",
                    Some(PartyRole::Lender),
                    &[id_of(index, "| Lender")],
                ),
                party(
                    "Delphine Moncrieff",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "| Examined by")],
                ),
            ],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Saltash Point Museum of Art"],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
    }

    #[test]
    fn a_compound_agreement_too_long_for_both_parties_keeps_the_agreement_it_ends_on() {
        let relation = Relation {
            relation: PartyRelation::Between,
            parties: vec![
                "Eton Pharmaceuticals, Inc.".into(),
                "Aucta Pharmaceuticals, Inc.".into(),
            ],
            basis: String::new(),
            on_proposed_roles: false,
            defaulted: false,
        };
        assert_eq!(
            fit_type(
                "Exclusive License and Product Development Agreement",
                &relation,
                Some("2019-06-12")
            )
            .as_deref(),
            Some("Product Development Agreement")
        );
        // An amendment to such an agreement is an amendment still.
        assert_eq!(
            fit_type(
                "Second Amendment to Software License and Support Agreement",
                &relation,
                Some("2025-09-29")
            ),
            None
        );
    }

    #[test]
    fn an_agreement_naming_one_of_its_parties_goes_to_review() {
        const AGREEMENT: &str = "SPONSORSHIP AGREEMENT\n\n\
This Sponsorship Agreement is made on July 16, 2012 by Peek Investments LLC.";
        let (outcome, _) = facts_for(AGREEMENT, |index| ModelFacts {
            document_type: Some("Sponsorship Agreement".into()),
            type_evidence: vec![id_of(index, "SPONSORSHIP")],
            document_date: Some("2012-07-16".into()),
            date_evidence: vec![id_of(index, "This Sponsorship")],
            parties: vec![party(
                "Peek Investments LLC",
                None,
                &[id_of(index, "This Sponsorship")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.party_relation, PartyRelation::With);
        assert!(
            outcome.reasons.contains(&ReviewReason::PartyMissing),
            "{:?}",
            outcome.reasons
        );
    }

    /// CUAD's cuad-34 of the held-out sample, as Qwen3.5-2B read it: one
    /// side named, the other left out, both in the opening clause.
    #[test]
    fn an_agreement_with_one_side_takes_the_other_from_its_between_clause() {
        const AGREEMENT: &str = "OUTSOURCING AGREEMENT\n\n\
This Outsourcing Agreement is made and entered into as of January 1, 1998, by and between \
Sykes HealthPlan Services, Inc., a Florida corporation (\"SHPS\"), and HealthPlan Services, Inc., \
a Florida corporation (\"HPS\").";
        let (outcome, _) = facts_for(AGREEMENT, |index| ModelFacts {
            document_type: Some("Outsourcing Agreement".into()),
            type_evidence: vec![id_of(index, "OUTSOURCING")],
            document_date: Some("1998-01-01".into()),
            date_evidence: vec![id_of(index, "This Outsourcing")],
            parties: vec![party(
                "Sykes HealthPlan Services, Inc.",
                None,
                &[id_of(index, "This Outsourcing")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec![
                "Sykes HealthPlan Services, Inc.",
                "HealthPlan Services, Inc."
            ],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        assert!(!outcome.reasons.contains(&ReviewReason::PartyMissing));

        // A clause the side the reply named is not in - a recital's other
        // agreement - says nothing about this one's other side.
        const RECITAL: &str = "LICENSE AGREEMENT\n\n\
This License Agreement is made on March 3, 2014 by Beijing Leju Services Co., Ltd.\n\n\
WHEREAS, the Share Purchase Agreement between SINA Corporation and CRIC Holdings Limited \
is in effect.";
        let (outcome, _) = facts_for(RECITAL, |index| ModelFacts {
            document_type: Some("License Agreement".into()),
            type_evidence: vec![id_of(index, "LICENSE")],
            document_date: Some("2014-03-03".into()),
            date_evidence: vec![id_of(index, "This License")],
            parties: vec![party(
                "Beijing Leju Services Co., Ltd.",
                None,
                &[id_of(index, "This License")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Beijing Leju Services Co., Ltd."]
        );
        assert!(outcome.reasons.contains(&ReviewReason::PartyMissing));

        // Nor does a recital's clause that names the side the reply kept:
        // CRIC is a party to the share purchase, not to this license.
        const KEPT_IN_RECITAL: &str = "LICENSE AGREEMENT\n\n\
This License Agreement is made on March 3, 2014 by SINA Corporation.\n\n\
WHEREAS, the Share Purchase Agreement between SINA Corporation and CRIC Holdings Limited \
is in effect.";
        let (outcome, _) = facts_for(KEPT_IN_RECITAL, |index| ModelFacts {
            document_type: Some("License Agreement".into()),
            type_evidence: vec![id_of(index, "LICENSE")],
            document_date: Some("2014-03-03".into()),
            date_evidence: vec![id_of(index, "This License")],
            parties: vec![party(
                "SINA Corporation",
                None,
                &[id_of(index, "This License")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.parties, vec!["SINA Corporation"]);
        assert!(outcome.reasons.contains(&ReviewReason::PartyMissing));

        // Nor a clause that names a third organization, a side's parent:
        // which of the other two is the other side cannot be told.
        const WITH_PARENT: &str = "SUPPLY AGREEMENT\n\n\
This Supply Agreement is made on May 15, 2013 between Acme Inc., a wholly owned subsidiary of \
Parent Corp., and Beta LLC.";
        let (outcome, _) = facts_for(WITH_PARENT, |index| ModelFacts {
            document_type: Some("Supply Agreement".into()),
            type_evidence: vec![id_of(index, "SUPPLY")],
            document_date: Some("2013-05-15".into()),
            date_evidence: vec![id_of(index, "This Supply")],
            parties: vec![party("Acme Inc.", None, &[id_of(index, "This Supply")])],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Acme Inc."],
            "{:?}",
            outcome.proposal
        );
        assert!(outcome.reasons.contains(&ReviewReason::PartyMissing));
    }

    /// InternBench's email-approval-thread: the reply typed it "email",
    /// which no line says; its header block does.
    #[test]
    fn an_email_is_typed_by_its_header_block() {
        const EMAIL: &str = "From: Signe Holmqvist <signe@pemberly.example>\n\
To: Saul Ravenscroft <saul@pemberly.example>\n\
Date: Tue, 17 Mar 2026 09:42:11 -0500\n\
Subject: RE: Approval needed: fleet monitoring renewal\n\n\
Saul,\n\nApproved. Please issue the purchase order today.\n\nSigne";
        let (outcome, _) = facts_for(EMAIL, |index| ModelFacts {
            document_type: Some("email".into()),
            type_evidence: vec![id_of(index, "From: Signe")],
            document_date: Some("2026-03-17".into()),
            date_evidence: vec![id_of(index, "Date: Tue")],
            parties: vec![party(
                "Signe Holmqvist",
                Some(PartyRole::Sender),
                &[id_of(index, "From: Signe")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Email"));
        assert!(
            !outcome.reasons.contains(&ReviewReason::TypeUnsupported),
            "{:?}",
            outcome.reasons
        );

        // Its date is the header's, not a date its text asks for.
        let with_deadline = format!("{EMAIL}\n\nI need your approval by March 20, 2026.");
        let (outcome, _) = facts_for(&with_deadline, |index| ModelFacts {
            document_type: Some("email".into()),
            type_evidence: vec![id_of(index, "From: Signe")],
            document_date: Some("2026-03-20".into()),
            date_evidence: vec![id_of(index, "I need your approval")],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-17")
        );

        // A short reply quoting a message is dated as itself, never by the
        // quoted message's header.
        const SHORT_REPLY: &str = "Approved on March 17, 2026.\n\n\
From: Saul Ravenscroft\nSent: Monday, March 9, 2026\nTo: Signe Holmqvist\nSubject: Renewal";
        let (outcome, _) = facts_for(SHORT_REPLY, |index| ModelFacts {
            document_type: Some("Approval".into()),
            document_date: Some("2026-03-17".into()),
            date_evidence: vec![id_of(index, "Approved on")],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-17")
        );

        // A letter is no email, nor is one that only quotes an email's
        // header below its own text.
        const LETTER: &str = "Dear Mr. Ravenscroft,\n\n\
We approve the renewal of the fleet monitoring subscription as of March 17, 2026.\n\n\
Signe Holmqvist\n\n\
From: Saul Ravenscroft\nSent: Monday, March 9, 2026\nTo: Signe Holmqvist\nSubject: Renewal";
        let (outcome, _) = facts_for(LETTER, |index| ModelFacts {
            document_type: Some("email".into()),
            type_evidence: vec![id_of(index, "Dear Mr.")],
            document_date: Some("2026-03-17".into()),
            date_evidence: vec![id_of(index, "We approve")],
            ..ModelFacts::default()
        });
        assert!(outcome.reasons.contains(&ReviewReason::TypeUnsupported));
    }

    /// An email's header block says who sent it and to whom in the domains
    /// of its addresses. The reply may read the sides from the body - the
    /// company a carrier's notice was sent "at the request of", the
    /// "Contractor:" of a contract the message attaches - and the header
    /// puts them right. A message within one organization, and a name no
    /// domain spells, are left as the reply read them.
    #[test]
    fn an_emails_header_addresses_say_who_sent_it_and_to_whom() {
        // A carrier's notification as the worker extracts an .eml file:
        // the reply called the customer's company the sender.
        let (outcome, _) = facts_for(
            "From: <pkginfo@pax.com>\nTo: <susan.bailey@kestrel.com>\nDate: Wed, 20 Mar 2002 \
             15:48:51 -0800 (PST)\nSubject: PAX Ship Notification, 1Z7261100192688747\n\n\
             ***Do not reply to this email.  PAX and KESTREL CORP will not receive your reply.\n\n\
             At the request of KESTREL CORP, this notice confirms the following shipment has \
             been sent.",
            |index| ModelFacts {
                document_type: Some("email".into()),
                type_evidence: vec![id_of(index, "From: <pkginfo")],
                document_date: Some("2002-03-20".into()),
                date_evidence: vec![id_of(index, "Date: Wed")],
                parties: vec![party(
                    "KESTREL CORP",
                    Some(PartyRole::Sender),
                    &[id_of(index, "At the request")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::To,
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.parties.len(), 1);
        assert!(!outcome.reasons.contains(&ReviewReason::PartyUnsupported));
        // An Outlook message: display names before the addresses, a "Cc:"
        // line, and the body's roles for the contract it is about.
        const MESSAGE: &str = "From: Betts, Bruce@DHS <Bruce.Betts@dhs.ca.gov>\n\
To: drose@qrpa.org <drose@qrpa.org>\n\
Cc: de la Motte, Leann@DHS <Leann.delaMotte@dhs.ca.gov>\n\
Date: Tue, 5 Apr 2016 14:42:09 +0000\n\
Subject: Harbour Code Publication\n\n\
Good Morning Debra,\n\n\
I am the Acquisition Analyst assigned to assist with the execution of Contract #3188107 \
for the Department of Harbour Services.\n\n\
Please complete and return the following documents to me via email.\n\n\
Contractor: QRPA";
        let sides = |index: &EvidenceIndex| ModelFacts {
            document_type: Some("email".into()),
            type_evidence: vec![id_of(index, "From: Betts")],
            document_date: Some("2016-04-05".into()),
            date_evidence: vec![id_of(index, "Date: Tue")],
            parties: vec![
                party(
                    "QRPA",
                    Some(PartyRole::Buyer),
                    &[id_of(index, "Contractor: QRPA")],
                ),
                party(
                    "Department of Harbour Services",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "I am the Acquisition")],
                ),
            ],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(MESSAGE, sides);
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::From,
            "{:?}",
            outcome.reasons
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Department of Harbour Services"]
        );
        let roles = |outcome: &ValidationOutcome| {
            outcome
                .facts
                .as_ref()
                .expect("facts")
                .parties
                .iter()
                .map(|party| (party.name.clone(), party.document_role))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            roles(&outcome),
            vec![
                ("QRPA".to_owned(), Some(PartyRole::Recipient)),
                (
                    "Department of Harbour Services".to_owned(),
                    Some(PartyRole::Sender)
                ),
            ]
        );
        // The same message sent within the department: both domains spell
        // "DHS", so the header gives it no side.
        let (outcome, _) = facts_for(
            &MESSAGE
                .replace("drose@qrpa.org", "drose@dhs.ca.gov")
                .replace("Department of Harbour Services", "DHS"),
            |index| ModelFacts {
                document_type: Some("email".into()),
                type_evidence: vec![id_of(index, "From: Betts")],
                parties: vec![party(
                    "DHS",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "I am the Acquisition")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(roles(&outcome), vec![("DHS".to_owned(), None)]);
        for (line, labels) in [
            ("Betts, Bruce@DHS <Bruce.Betts@dhs.ca.gov>", vec!["dhs"]),
            ("<susan.bailey@kestrel.com>", vec!["kestrel"]),
            (
                "Mark Johnson (mjohnson@icc-es.org) <mjohnson@icc-es.org>, Dave Walls \
                 <dwalls@iccsafe.org>",
                vec!["icc-es", "iccsafe"],
            ),
            (
                "Jillian Holford <Jillian.Holford@royalquay.gov.uk>",
                vec!["royalquay"],
            ),
            // A mail provider's domain, a display name's "@", no address.
            ("Mara Quill <mara.quill@gmail.com>", vec![]),
            ("Betts, Bruce@DHS", vec![]),
            ("Saul Ravenscroft", vec![]),
        ] {
            assert_eq!(address_domain_labels(line), labels, "{line}");
        }
        for (label, name, spelled) in [
            ("kestrel", "KESTREL CORP", true),
            ("qrpa", "QRPA", true),
            ("dhs", "Department of Harbour Services", true),
            ("hdrinc", "HDR", true),
            ("icc-es", "ICC", true),
            ("c-spanarchives", "C-SPAN Archives", true),
            ("lambeth", "The Lambeth Council", true),
            // A first word the label only starts with, initials of one
            // word, two letters, another organization.
            ("generalservices", "General Dynamics", false),
            ("kestrel", "Kestrelbank Outfitters", false),
            ("dhs", "Dhsworth", false),
            ("ca", "CA", false),
            ("pax", "KESTREL CORP", false),
        ] {
            assert_eq!(domain_spells(label, name), spelled, "{label} {name}");
        }
    }

    /// An email's "Subject:" names what the message is about. A reply that
    /// typed the email by it - the agreement it sends for signature - gets
    /// "Email" for a person to confirm. A notice or an invoice sent as an
    /// email keeps the kind its subject gives, and a letter's "Subject:"
    /// line, with no header block around it, still names the letter.
    #[test]
    fn an_emails_subject_names_its_matter_not_its_kind() {
        let message = |subject: &str| {
            format!(
                "From: Betts, Bruce@DHS <Bruce.Betts@dhs.ca.gov>\n\
                 To: drose@qrpa.org <drose@qrpa.org>\n\
                 Cc: de la Motte, Leann@DHS <Leann.delaMotte@dhs.ca.gov>\n\
                 Date: Tue, 5 Apr 2016 14:42:09 +0000\n\
                 Subject: {subject}\n\n\
                 Good Morning Debra,\n\n\
                 I have attached the documents for the Department of Harbour Services. Please \
                 complete and return them to me via email.\n\nContractor: QRPA"
            )
        };
        let typed = |kind: &'static str| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some(kind.into()),
                type_evidence: vec![id_of(index, "Subject:")],
                document_date: Some("2016-04-05".into()),
                date_evidence: vec![id_of(index, "Date: Tue")],
                parties: vec![
                    party(
                        "Department of Harbour Services",
                        Some(PartyRole::Employer),
                        &[id_of(index, "I have attached")],
                    ),
                    party(
                        "QRPA",
                        Some(PartyRole::Contractor),
                        &[id_of(index, "Contractor: QRPA")],
                    ),
                ],
                ..ModelFacts::default()
            }
        };
        const AGREEMENT: &str = "2016 Harbour Code Publication Agreement";
        let (outcome, _) = facts_for(&message(AGREEMENT), typed(AGREEMENT));
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Email"),
            "{:?}",
            outcome.reasons
        );
        assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
        assert_eq!(
            outcome.proposal.parties,
            vec!["Department of Harbour Services"]
        );
        // An invoice or a notice that arrives as an email is still one.
        for (subject, kind, class) in [
            (
                "Invoice for Harbour Code Printing",
                "Invoice",
                DocumentClass::Issued,
            ),
            (
                "Notice of Shipment 1Z7261100192688747",
                "Notice of Shipment",
                DocumentClass::Notice,
            ),
        ] {
            let (outcome, _) = facts_for(&message(subject), typed(kind));
            assert_eq!(
                DocumentClass::of(outcome.proposal.document_type.as_deref()),
                class,
                "{subject}: {:?}",
                outcome.proposal.document_type
            );
        }
        // A letter's "Subject:" line stands in no header block.
        let (outcome, _) = facts_for(
            "Harbour Advisory Committee\n\n5 April 2016\n\nSubject: Offer of Employment\n\n\
             Dear Ms. Rose,\n\nWe are pleased to offer you the post of analyst.\n\n\
             Yours sincerely,\n\nBruce Betts",
            |index| ModelFacts {
                document_type: Some("Offer of Employment".into()),
                type_evidence: vec![id_of(index, "Subject:")],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Offer of Employment")
        );
    }

    #[test]
    fn a_field_label_is_no_part_of_the_name_it_labels() {
        assert!(is_role_word("name"));
        assert!(is_role_word("account"));
        assert!(is_role_word("plaintiff"));
        assert!(!is_role_word("calm"));
    }

    #[test]
    fn a_column_naming_a_party_keeps_its_names_whoever_signs() {
        assert!(header_names_party("recipient", DocumentClass::Record));
        assert!(header_names_party("tenant", DocumentClass::Notice));
        assert!(!header_names_party("signature", DocumentClass::Record));
        assert!(!header_names_party("buyer", DocumentClass::Issued));
        const RECORD: &str = "Harbor Point Shipping Co.\n\nDELIVERY RECORD\n\n\
| Recipient | Signature |\n| --- | --- |\n| Jane Doe | /s/ Jane Doe |";
        let (outcome, _) = facts_for(RECORD, |index| ModelFacts {
            document_type: Some("Delivery Record".into()),
            type_evidence: vec![id_of(index, "DELIVERY")],
            parties: vec![party("Jane Doe", None, &[id_of(index, "| Jane")])],
            ..ModelFacts::default()
        });
        assert_eq!(
            outcome.proposal.parties,
            vec!["Jane Doe"],
            "{:?}",
            outcome.proposal
        );
    }

    #[test]
    fn a_label_opening_a_line_covers_only_the_names_it_lists() {
        assert_eq!(
            listed_under(
                "Mechanics: Linnea Hart, Joaquin Moncrieff",
                "joaquin moncrieff"
            )
            .as_deref(),
            Some("mechanics")
        );
        assert_eq!(
            listed_under("Prepared by: /s/ John Smith", "john smith").as_deref(),
            Some("prepared by")
        );
        assert_eq!(
            listed_under("Prepared by: John Smith for Jane Doe", "jane doe"),
            None
        );
        const REPORT: &str = "INSPECTION REPORT\n\nDated March 3, 2026\n\n\
Prepared by: John Smith for Jane Doe\n\nThe roof at 14 Wren Street needs new flashing.";
        let (outcome, _) = facts_for(REPORT, |index| ModelFacts {
            document_type: Some("Inspection Report".into()),
            type_evidence: vec![id_of(index, "INSPECTION")],
            parties: vec![party("Jane Doe", None, &[id_of(index, "Prepared")])],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.parties, vec!["Jane Doe"]);
    }

    #[test]
    fn a_post_after_a_name_must_be_theirs() {
        assert!(titled_after(
            "Jane Doe, Chief Financial Officer",
            "jane doe"
        ));
        assert!(titled_after("Jane Doe (Conservator)", "jane doe"));
        assert!(titled_after("| Jane Doe | Winemaker |", "jane doe"));
        assert!(titled_after("JANE DOE - SITE INSPECTOR", "jane doe"));
        assert!(titled_after("Jane Doe, Director of Operations", "jane doe"));
        assert!(titled_after("Jane Doe, the conservator", "jane doe"));
        assert!(!titled_after("Jane Doe spoke to the manager.", "jane doe"));
        assert!(!titled_after("Jane Doe, who met the inspector", "jane doe"));
        const REPORT: &str = "INCIDENT REPORT\n\nDated March 3, 2026\n\n\
Jane Doe spoke to the manager about the leak in Unit 4.";
        let (outcome, _) = facts_for(REPORT, |index| ModelFacts {
            document_type: Some("Incident Report".into()),
            type_evidence: vec![id_of(index, "INCIDENT")],
            parties: vec![party("Jane Doe", None, &[id_of(index, "spoke")])],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.parties, vec!["Jane Doe"]);
    }

    #[test]
    fn a_header_names_its_issuer_before_the_kind_of_document() {
        assert_eq!(
            organization_before_kind("Halvorsen Fixture Works LLC INVOICE").as_deref(),
            Some("halvorsen fixture works llc")
        );
        assert_eq!(organization_before_kind("INVOICE"), None);
        assert_eq!(
            organization_before_kind("Halvorsen Fixture Works LLC"),
            None
        );
        const INVOICE: &str = "Halvorsen Fixture Works LLC INVOICE\n\n\
4410 Ostrander Road, Halden Bay, WA 98264\n\nInvoice Date: 03/04/2026\n\n\
Bill to: Quillon Ridge Bakery, Inc.";
        let (outcome, _) = facts_for(INVOICE, |index| ModelFacts {
            document_type: Some("Invoice".into()),
            type_evidence: vec![id_of(index, "Halvorsen")],
            parties: vec![party(
                "Quillon Ridge Bakery, Inc.",
                Some(PartyRole::Customer),
                &[id_of(index, "Bill to")],
            )],
            ..ModelFacts::default()
        });
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
        assert_eq!(
            outcome.proposal.parties,
            vec!["Halvorsen Fixture Works LLC"],
            "{:?}",
            outcome.proposal
        );
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

    /// C-SPAN's invoice in the real-document set, as Qwen3.5-2B read it:
    /// only the customer its "Bill To" names, called the recipient, and no
    /// issuer. "Invoice - Ansel Pike" would read as the customer's invoice.
    #[test]
    fn an_invoice_whose_reply_named_only_its_customer_names_no_one() {
        let (outcome, _) = facts_for(
            "Invoice\n\n| Invoice # | IN0260 |\n| Terms | Due on receipt |\n\n\
             | Bill To | Ship To |\n| Ansel Pike | Ansel Pike |\n\
             | 1005 Orchard Road | 1005 Orchard Road |\n\n\
             | Item | Amount |\n| Hearing archive DVD | $29.95 |",
            |index| ModelFacts {
                document_type: Some("Invoice".into()),
                type_evidence: vec![id_of(index, "Invoice")],
                parties: vec![party(
                    "Ansel Pike",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "| Ansel Pike")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            outcome.proposal.parties.is_empty(),
            "{:?}",
            outcome.proposal
        );
        assert!(
            outcome.reasons.contains(&ReviewReason::PartyMissing),
            "{:?}",
            outcome.reasons
        );
        assert_ne!(outcome.status, ProposalStatus::Ready);
    }

    /// The real set's Varibus invoice: the issuer heads the page and the
    /// reply called it the sender, the customer below it the recipient. The
    /// document confirms neither, but the issuer comes first: it is named.
    #[test]
    fn an_invoice_headed_by_the_party_the_reply_called_its_sender_names_it() {
        let (outcome, _) = facts_for(
            "TARNBECK CORPORATION\nP.O. BOX 61000\nNEW ORLEANS, LA 70161\n\n\
             INVOICE NUMBER 00-03-06\n\nMARCH 31, 2000\n\nIN ACCOUNT WITH\n\
             HALLOWAY TRADING CORP.\n1400 SMITH, SUITE 3707\n\nNatural gas, 8,234 MMBtu",
            |index| ModelFacts {
                document_type: Some("Invoice".into()),
                type_evidence: vec![id_of(index, "INVOICE NUMBER")],
                document_date: Some("2000-03-31".into()),
                date_evidence: vec![id_of(index, "MARCH 31")],
                parties: vec![
                    party(
                        "TARNBECK CORPORATION",
                        Some(PartyRole::Sender),
                        &[id_of(index, "TARNBECK")],
                    ),
                    party(
                        "HALLOWAY TRADING CORP.",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "HALLOWAY")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.parties.len(), 1, "{:?}", outcome.proposal);
        assert!(
            outcome.proposal.parties[0]
                .to_lowercase()
                .contains("tarnbeck"),
            "{:?}",
            outcome.proposal
        );
        assert!(!outcome.reasons.contains(&ReviewReason::PartyMissing));
    }

    /// An 1880s bill in the held-out real set, as Qwen3.5-2B read it: its
    /// issuer, the "Bought of" party at the head of the page, called the
    /// buyer. The reply's unconfirmed role never makes another company in
    /// the billhead the issuer: the page's head names the issuer.
    #[test]
    fn a_bills_issuer_the_reply_called_its_buyer_still_issues_it() {
        let (outcome, _) = facts_for(
            "BILL\n\nBoston, Oct. 1, 1880.\n\nBOUGHT OF CORWIN HASKELL, Jr.\n\n\
             AUTHORIZED MANUFACTURER FOR\n\nNATIONAL BELL TELEPHONE CO.,\n\n109 COURT STREET.\n\n\
             To Prof. A. B. Hale\n\nInduction coil, 2.00",
            |index| ModelFacts {
                document_type: Some("Bill".into()),
                type_evidence: vec![id_of(index, "BILL")],
                parties: vec![party(
                    "Corwin Haskell, Jr.",
                    Some(PartyRole::Buyer),
                    &[id_of(index, "BOUGHT OF")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|party| party.to_lowercase().contains("bell telephone")),
            "{:?}",
            outcome.proposal
        );
    }

    /// A broadcast station's invoice as its billing system prints it and
    /// the worker reads it, in the real sets: the station on the line that
    /// says where payment goes, the agency it bills in an address block
    /// with no label on its own lines.
    const STATION_INVOICE: &str = "INVOICE\n\nProperty\n\n\
| KQRD | Invoice # |\n| 14 Signal Hill Road | Invoice Date |\n| Marrow Springs, CO 80513 | Invoice Month |\n\n\
Billing Address:\n\n\
Tidewater Buying LLC / POL\nAttention: Accounts Payable\n41 Wharf Street\nGilchrist Harbor, ME 04112\n\n\
Send Payment To: KQRD P.O. Box 4410 Marrow Springs, CO 80513\n\nKQRD\n\n\
2884942-1\n05/26/24\nMay 2024\n\n\
| Order # | 2884942 |\n| Flight Dates | 05/06/24 - 05/24/24 |\n\n\
| Spots: # | Ch | Day | Air Date | Rate |\n| 1 | KQRD | M | 05/06/24 | $1,300.00 |";

    fn station_invoice_facts(
        index: &EvidenceIndex,
        station: PartyRole,
        agency: PartyRole,
    ) -> ModelFacts {
        ModelFacts {
            document_type: Some("INVOICE".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            parties: vec![
                party(
                    "Tidewater Buying LLC",
                    Some(agency),
                    &[id_of(index, "Tidewater")],
                ),
                party("KQRD", Some(station), &[id_of(index, "| KQRD |")]),
            ],
            ..ModelFacts::default()
        }
    }

    /// The line that says where payment goes supports the party on it as
    /// the seller the reply called it, so the name rests on the document
    /// and not on the reply's roles; and it makes that party the issuer
    /// where the reply read the two sides the wrong way round, as
    /// Qwen3.5-4B did the station and its agency.
    #[test]
    fn the_party_payment_is_sent_to_issued_the_invoice() {
        for (station, agency) in [
            (PartyRole::Seller, PartyRole::Buyer),
            (PartyRole::Issuer, PartyRole::Client),
            (PartyRole::Buyer, PartyRole::Seller),
            (PartyRole::Recipient, PartyRole::Sender),
        ] {
            let (outcome, _) = facts_for(STATION_INVOICE, |index| {
                station_invoice_facts(index, station, agency)
            });
            assert_eq!(
                outcome.proposal.party_relation,
                PartyRelation::From,
                "{station:?}: {:?}",
                outcome.proposal
            );
            assert_eq!(outcome.proposal.parties, vec!["KQRD"], "{station:?}");
            assert!(
                !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
                "{station:?}: {:?}",
                outcome.reasons
            );
            let facts = outcome.facts.expect("facts");
            let named = facts
                .parties
                .iter()
                .find(|party| party.name == "KQRD")
                .expect("the station");
            assert!(
                matches!(
                    named.role,
                    Some(PartyRole::Seller | PartyRole::Issuer | PartyRole::Vendor)
                ),
                "{station:?}: {:?}",
                named.role
            );
        }
    }

    /// The label alone on its line gives no one a role: the billing system
    /// prints its labels apart from their values ("Billing Address:" right
    /// over the payment line), and the block that follows "Send Payment
    /// To:" in the text may be the customer's address. The agency under it
    /// is not the seller on the document's word, though the reply called it
    /// that.
    #[test]
    fn a_payment_label_alone_on_its_line_gives_no_one_a_role() {
        const PAGE: &str = "INVOICE\n\nBilling Address:\n\nSend Payment To:\n\n\
Tidewater Buying LLC / POL\nAttention: Accounts Payable\n41 Wharf Street\nGilchrist Harbor, ME 04112\n\n\
| Order # | 2884942 |\n| Flight Dates | 05/06/24 - 05/24/24 |\n\n\
| Spots: # | Ch | Day | Air Date | Rate |\n| 1 | KQRD | M | 05/06/24 | $1,300.00 |";
        let (outcome, _) = facts_for(PAGE, |index| ModelFacts {
            document_type: Some("INVOICE".into()),
            type_evidence: vec![id_of(index, "INVOICE")],
            parties: vec![
                party(
                    "Tidewater Buying LLC",
                    Some(PartyRole::Seller),
                    &[id_of(index, "Tidewater")],
                ),
                party(
                    "KQRD",
                    Some(PartyRole::Buyer),
                    &[id_of(index, "| 1 | KQRD")],
                ),
            ],
            ..ModelFacts::default()
        });
        let facts = outcome.facts.as_ref().expect("facts");
        let agency = facts
            .parties
            .iter()
            .find(|party| party.name.starts_with("Tidewater"))
            .expect("the agency");
        assert_eq!(agency.role_support, Support::Unsupported, "{agency:?}");
        assert_eq!(agency.role, None, "{agency:?}");
        assert_eq!(agency.document_role, None, "{agency:?}");
    }

    /// A table's row that sets the customer's block beside the payment
    /// line: the payment cell carries its own value, so it labels its own
    /// cell and not the customer's beside it. The customer is no issuer
    /// on the document's word, whichever way the reply read the sides and
    /// whichever spelling the label takes.
    #[test]
    fn a_payment_cell_beside_the_customers_labels_only_its_own_value() {
        for label in ["Send Payment To:", "Remit To:"] {
            for (customer_cell, issuer, customer) in [
                (
                    "Quillon Ridge Bakery, Inc.",
                    PartyRole::Seller,
                    PartyRole::Buyer,
                ),
                (
                    "Bill To: Quillon Ridge Bakery, Inc.",
                    PartyRole::Buyer,
                    PartyRole::Seller,
                ),
                (
                    "Bill To: Quillon Ridge Bakery, Inc.",
                    PartyRole::Customer,
                    PartyRole::Issuer,
                ),
            ] {
                let page = format!(
                    "INVOICE\n\nInvoice No. 4471\nInvoice Date: 03/04/2024\n\n\
                     | {customer_cell} | {label} Halvorsen Fixture Works LLC |\n\
                     | 41 Wharf Street | P.O. Box 7710 |\n\n\
                     | Item | Qty | Amount |\n| Display shelving | 4 | $1,300.00 |"
                );
                let (outcome, _) = facts_for(&page, |index| ModelFacts {
                    document_type: Some("Invoice".into()),
                    type_evidence: vec![id_of(index, "INVOICE")],
                    parties: vec![
                        party(
                            "Halvorsen Fixture Works LLC",
                            Some(issuer),
                            &[id_of(index, "Halvorsen")],
                        ),
                        party(
                            "Quillon Ridge Bakery, Inc.",
                            Some(customer),
                            &[id_of(index, "Quillon")],
                        ),
                    ],
                    ..ModelFacts::default()
                });
                assert!(
                    !(outcome.status == ProposalStatus::Ready
                        && outcome
                            .proposal
                            .parties
                            .iter()
                            .any(|party| party.starts_with("Quillon"))),
                    "{label} {customer_cell} {issuer:?}: {:?} {:?}",
                    outcome.proposal,
                    outcome.reasons
                );
                let facts = outcome.facts.as_ref().expect("facts");
                let named = facts
                    .parties
                    .iter()
                    .find(|party| party.name.starts_with("Quillon"))
                    .expect("the customer");
                assert!(
                    !matches!(
                        named.document_role,
                        Some(PartyRole::Issuer | PartyRole::Seller | PartyRole::Vendor)
                    ),
                    "{label} {customer_cell}: {named:?}"
                );
            }
        }
    }

    /// A buyer's dispute of an invoice in the real set, typed into a sheet
    /// and read by Qwen3.5-9B as the invoice: the company it is addressed
    /// to heads the sheet, the one that wrote it signs at the foot, below
    /// the "Attn:" line. The reply read both sides - the first its
    /// recipient, the second its issuer - and where the first stands makes
    /// no issuer of it: the name rests on the reply's roles, and waits.
    const SHEET: &str = "## Sheet1\n\n\
| APRIL 23, 2001 |  |  |  |\n|  |  |  |  |\n| HALLOWAY TRADING CORP. |  |  |  |\n|  |  |  |  |\n\
| ATTN:  ANSEL PIKE |  | RE:  INVOICE NO. 24636SA |  |\n| 713 555-0175 FAX |  |  |  |\n|  |  |  |  |\n\
| We show the following volumes purchased in March 2001: |  |  |  |\n\
|  | MMBTU | UNIT COST | AMOUNT |\n| Larkspur 250 | 5459 | 4.96 | 27076.64 |\n|  |  |  |  |\n\
| PLEASE SUBMIT A CORRECTED INVOICE AS SOON AS POSSIBLE. |  |  |  |\n|  |  |  |  |\n\
| DELPHINE MONCRIEFF |  |  |  |\n| TARNBECK HYDROCARBONS & RESOURCES |  |  |  |\n| 713 555-0140 |  |  |  |";

    fn sheet_facts(index: &EvidenceIndex, head: PartyRole, foot: PartyRole) -> ModelFacts {
        ModelFacts {
            document_type: Some("INVOICE NO. 24636SA".into()),
            type_evidence: vec![id_of(index, "INVOICE NO.")],
            document_date: Some("2001-04-23".into()),
            date_evidence: vec![id_of(index, "APRIL 23")],
            parties: vec![
                party(
                    "TARNBECK HYDROCARBONS & RESOURCES",
                    Some(foot),
                    &[id_of(index, "TARNBECK")],
                ),
                party(
                    "HALLOWAY TRADING CORP.",
                    Some(head),
                    &[id_of(index, "HALLOWAY")],
                ),
            ],
            ..ModelFacts::default()
        }
    }

    #[test]
    fn the_head_of_a_page_makes_no_issuer_of_the_one_the_reply_read_as_billed() {
        for head in [PartyRole::Recipient, PartyRole::Buyer, PartyRole::Client] {
            for foot in [PartyRole::Issuer, PartyRole::Seller, PartyRole::Vendor] {
                let (outcome, _) = facts_for(SHEET, |index| sheet_facts(index, head, foot));
                assert_eq!(
                    outcome.proposal.parties,
                    vec!["TARNBECK HYDROCARBONS & RESOURCES"],
                    "{head:?} {foot:?}: {:?}",
                    outcome.proposal
                );
                assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
                assert!(
                    outcome.reasons.contains(&ReviewReason::PartyUnsupported),
                    "{head:?} {foot:?}: {:?}",
                    outcome.reasons
                );
                assert_ne!(outcome.status, ProposalStatus::Ready);
            }
        }
    }

    /// The page's head still names the issuer where the reply read no
    /// issuer to set against it: one that called the party at the head its
    /// issuer, and one that called the other party the sender - a word a
    /// reply uses too loosely of an invoice's sides to read.
    #[test]
    fn the_head_of_a_page_names_the_issuer_where_the_reply_read_no_other() {
        for (head, foot) in [
            (PartyRole::Issuer, PartyRole::Recipient),
            (PartyRole::Seller, PartyRole::Buyer),
            (PartyRole::Recipient, PartyRole::Sender),
        ] {
            let (outcome, _) = facts_for(SHEET, |index| sheet_facts(index, head, foot));
            assert_eq!(
                outcome.proposal.parties,
                vec!["HALLOWAY TRADING CORP."],
                "{head:?} {foot:?}: {:?}",
                outcome.proposal
            );
            assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
            assert!(
                !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
                "{head:?} {foot:?}: {:?}",
                outcome.reasons
            );
        }
    }

    /// The same 1880s bill as Qwen3.5-4B read it in the real set: its
    /// issuer called the buyer, and a phrase of the billhead - no name the
    /// bill writes in one place - called the seller. A side the document
    /// does not name is no issuer to set against the page: its head still
    /// names the one the goods were bought of.
    #[test]
    fn a_bills_issuer_still_issues_it_though_the_reply_named_a_seller_it_does_not() {
        let (outcome, _) = facts_for(
            "BILL\n\nManufacturer of\n\nTelegraph Instruments. Boston, Oct. 1, 1880.\n\n\
             BOUGHT OF CORWIN HASKELL, Jr.\n\nAUTHORIZED MANUFACTURER FOR\n\n\
             NATIONAL BELL TELEPHONE CO.,\n\nShipped Via. 109 COURT STREET.\n\n\
             | July | 26 | Induction coil | 200 |",
            |index| ModelFacts {
                document_type: Some("Bill".into()),
                type_evidence: vec![id_of(index, "BILL")],
                parties: vec![
                    party(
                        "Corwin Haskell, Jr.",
                        Some(PartyRole::Buyer),
                        &[id_of(index, "BOUGHT OF")],
                    ),
                    party(
                        "Manufacturer of Telegraph Instruments",
                        Some(PartyRole::Seller),
                        &[id_of(index, "Manufacturer of")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Corwin Haskell, Jr."],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
    }

    /// A printer's corrected invoice of 1923 in the real set, as the scan
    /// reads: the customer typed under the date, above the billhead, so
    /// that both stand where a letterhead does. Both models read the sides
    /// right, and the customer was named the issuer for coming first.
    #[test]
    fn a_customer_typed_above_the_billhead_is_not_its_issuer() {
        const BILL: &str = "Marrow Springs, Col., November 22, 1923\n\n\
Ansel Pike,\n200 East Ave 43, City\n\nHALVORSEN PRINTING & BINDING HOUSE\n\n\
BEST SERVICE PHONE 875-631\n\n118 SOUTH BROADWAY\n\n\
ALL CLAIMS MUST BE MADE WITHIN TEN DAYS TERMS: NET CASH\n\n\
| 2000 Circulars | $12.50 |\n| Cut | 2.25 |\n\n(Corrected Invoice)";
        let reply = |index: &EvidenceIndex, customer: PartyRole| ModelFacts {
            document_type: Some("Corrected Invoice".into()),
            type_evidence: vec![id_of(index, "Corrected Invoice")],
            parties: vec![
                party("Ansel Pike", Some(customer), &[id_of(index, "Ansel Pike")]),
                party(
                    "HALVORSEN PRINTING & BINDING HOUSE",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "HALVORSEN")],
                ),
            ],
            ..ModelFacts::default()
        };
        for customer in [PartyRole::Client, PartyRole::Buyer] {
            let (outcome, _) = facts_for(BILL, |index| reply(index, customer));
            assert_eq!(
                outcome.proposal.parties,
                vec!["HALVORSEN PRINTING & BINDING HOUSE"],
                "{customer:?}: {:?}",
                outcome.proposal
            );
            assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
            // The page put both in the issuer's place: the name rests on
            // the reply's reading of them, and waits for a person.
            assert!(
                outcome.reasons.contains(&ReviewReason::PartyUnsupported),
                "{customer:?}: {:?}",
                outcome.reasons
            );
        }
        // The same page read with the first party as its issuer and no
        // customer among the two: the first is named, as the page has it.
        let (outcome, _) = facts_for(BILL, |index| ModelFacts {
            parties: vec![
                party(
                    "Ansel Pike",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "Ansel Pike")],
                ),
                party(
                    "HALVORSEN PRINTING & BINDING HOUSE",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "HALVORSEN")],
                ),
            ],
            ..reply(index, PartyRole::Issuer)
        });
        assert_eq!(outcome.proposal.parties, vec!["Ansel Pike"]);
        assert!(
            !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
            "{:?}",
            outcome.reasons
        );
    }

    /// The IRS's CP161, typed "Notice": one of a hundred kinds the word
    /// covers, which names no kind a person could file it by.
    #[test]
    fn a_bare_notice_goes_to_review() {
        let (outcome, _) = facts_for(
            "Department of Revenue\nNotice CP161\nNotice date: January 8, 2018\n\n\
             Welland Tool Company\n22 Boulder Street\n\n\
             You have unpaid taxes for the period ending December 31, 2017.",
            |index| ModelFacts {
                document_type: Some("Notice".into()),
                type_evidence: vec![id_of(index, "Notice date")],
                document_date: Some("2018-01-08".into()),
                date_evidence: vec![id_of(index, "Notice date")],
                parties: vec![party(
                    "Welland Tool Company",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Welland")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            outcome.reasons.contains(&ReviewReason::TypeMissing),
            "{:?}",
            outcome.reasons
        );
        assert_ne!(outcome.status, ProposalStatus::Ready);
    }

    /// An FCC notice of apparent liability in the real set, as Qwen3.5-2B
    /// read it: dated by the day of the violation it recounts, though its
    /// caption gives its own release date.
    #[test]
    fn a_notice_dated_by_an_event_it_recounts_takes_its_released_date() {
        let (outcome, _) = facts_for(
            "FEDERAL COMMUNICATIONS COMMISSION\n\nIn the Matter of\n\nKestrel Sports Network, Inc.\n\n\
             NOTICE OF APPARENT LIABILITY FOR FORFEITURE\n\n\
             Adopted: October 10, 2024 Released: October 17, 2024\n\n\
             1. On October 20, 2023, Kestrel Sports Network, Inc. transmitted emergency alert \
             tones during a promotional segment.",
            |index| ModelFacts {
                document_type: Some("Notice of Apparent Liability for Forfeiture".into()),
                type_evidence: vec![id_of(index, "NOTICE OF APPARENT")],
                document_date: Some("2023-10-20".into()),
                date_evidence: vec![id_of(index, "1. On October")],
                parties: vec![party(
                    "Kestrel Sports Network, Inc.",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Kestrel Sports Network, Inc.")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2024-10-17"),
            "{:?}",
            outcome.reasons
        );
    }

    /// WPTV's invoice in the real set: the reply took a line item's air date;
    /// the invoice's own date stands in its "Invoice Date" column, written
    /// with a two-digit year.
    #[test]
    fn an_invoice_dated_by_a_line_item_takes_its_invoice_date_column() {
        let (outcome, _) = facts_for(
            "MARLOW BAY TV\n\nINVOICE\n\n\
             | Station | Invoice # | Invoice Date |\n| Marlow Bay | 3136-1 | 10/14/12 |\n\n\
             | Line Start Date | End Date | Rate |\n| 10/03/12 | 10/04/12 | $1,600.00 |",
            |index| ModelFacts {
                document_type: Some("Invoice".into()),
                type_evidence: vec![id_of(index, "INVOICE")],
                document_date: Some("2012-10-03".into()),
                date_evidence: vec![id_of(index, "| 10/03/12")],
                parties: vec![party(
                    "MARLOW BAY TV",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "MARLOW BAY TV")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2012-10-14"),
            "{:?}",
            outcome.reasons
        );
    }

    /// An invoice dated by an unlabeled dateline, its items in a table with
    /// a bare "Date" column and amounts with no currency sign: an item's
    /// date is never the invoice's (Codex's case on #51).
    #[test]
    fn an_items_date_column_never_dates_the_invoice() {
        let (outcome, _) = facts_for(
            "Corvane Office Supply LLC\n\nINVOICE\n\nMarch 31, 2026\n\n\
             Bill To: Pellham Dental Group\n\n\
             | Date | Description | Amount |\n| March 15, 2026 | Services | 1600.00 |",
            |index| ModelFacts {
                document_type: Some("Invoice".into()),
                type_evidence: vec![id_of(index, "INVOICE")],
                document_date: Some("2026-03-31".into()),
                date_evidence: vec![id_of(index, "March 31, 2026")],
                parties: vec![party(
                    "Corvane Office Supply LLC",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "Corvane")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-31"),
            "{:?}",
            outcome.reasons
        );
    }

    /// A date in a field of its own stands, whatever other date a field
    /// names: InternBench's purchase order is dated by its "PO Date", and a
    /// quotation's "dated" in a sentence is never the order's.
    #[test]
    fn a_date_in_a_field_of_its_own_stands() {
        let (outcome, _) = facts_for(
            "Lamplighter Robotics Inc.\n\nPURCHASE ORDER\n\n\
             | PO Number | PO Date | Buyer |\n| LR-PO-260117 | January 29, 2026 | Juniper Owens |\n\n\
             Reference: your Quotation Q-55120 dated January 22, 2026.\n\nDate: January 22, 2026",
            |index| ModelFacts {
                document_type: Some("Purchase Order".into()),
                type_evidence: vec![id_of(index, "PURCHASE ORDER")],
                document_date: Some("2026-01-29".into()),
                date_evidence: vec![id_of(index, "LR-PO-260117")],
                parties: vec![party(
                    "Lamplighter Robotics Inc.",
                    Some(PartyRole::Buyer),
                    &[id_of(index, "Lamplighter")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-01-29")
        );
    }

    /// The real set's notice of removal: its own "Dated:" runs onto the next
    /// line, and the complaint attached as an exhibit carries a "Dated:" of
    /// its own on a later page. That one is never the notice's date.
    #[test]
    fn an_exhibits_dated_field_never_dates_the_document() {
        let index = EvidenceIndex::build(&DocumentSource::from_pages(vec![
            crate::domain::SourcePage::new(
                1,
                "UNITED STATES DISTRICT COURT\n\nNOTICE OF REMOVAL\n\n\
                 Defendant Trellis Broadcasting Network, Inc. removes this action.\n\n\
                 Dated: Islandia, New York\nAugust 31, 2009\n\nRECEIVED SEP 01 2009",
                PageOrigin::PlainText,
            ),
            crate::domain::SourcePage::new(
                2,
                "EXHIBIT A\n\nVERIFIED COMPLAINT\n\nDated: August 7, 2009",
                PageOrigin::PlainText,
            ),
        ]));
        let context = whole(&index);
        let facts = ModelFacts {
            document_type: Some("Notice of Removal".into()),
            type_evidence: vec![id_of(&index, "NOTICE OF REMOVAL")],
            document_date: Some("2009-09-01".into()),
            date_evidence: vec![id_of(&index, "RECEIVED")],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &context, &index);
        assert_ne!(
            outcome.proposal.document_date.as_deref(),
            Some("2009-08-07")
        );
    }

    /// A notice of deposition in the held-out real set, as Qwen3.5-2B read
    /// it: dated by the deposition it sets, though it dates itself at its
    /// end, on page two. The schedule attached after it carries a "Dated:" of
    /// its own, which is never the notice's.
    #[test]
    fn a_notice_dated_at_its_end_takes_that_date_and_not_a_schedules() {
        let index = EvidenceIndex::build(&DocumentSource::from_pages(vec![
            crate::domain::SourcePage::new(
                1,
                "UNITED STATES DISTRICT COURT\n\nWINDROW.COM, INC., Plaintiff,\nv.\n\
                 CASCADE WEB SERVICES LLC, Defendant.\n\nNOTICE OF DEPOSITION\n\n\
                 PLEASE TAKE NOTICE that Plaintiff will take the deposition of Defendant \
                 on October 21, 2008 at 9:00 a.m.",
                PageOrigin::PlainText,
            ),
            crate::domain::SourcePage::new(
                2,
                "Dated: September 3, 2008\n\nBy: /s/ Counsel for Plaintiff",
                PageOrigin::PlainText,
            ),
            crate::domain::SourcePage::new(
                3,
                "SCHEDULE A\n\nDefinitions\n\nDated: January 5, 2007",
                PageOrigin::PlainText,
            ),
        ]));
        let context = whole(&index);
        let facts = ModelFacts {
            document_type: Some("Notice of Deposition".into()),
            type_evidence: vec![id_of(&index, "NOTICE OF DEPOSITION")],
            document_date: Some("2008-10-21".into()),
            date_evidence: vec![id_of(&index, "PLEASE TAKE NOTICE")],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &context, &index);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2008-09-03"),
            "{:?}",
            outcome.reasons
        );
    }

    /// An exhibit that opens on its title as well as its letter is an
    /// attachment too: its "Dated:" is the exhibit's, never the notice's.
    #[test]
    fn an_exhibit_opening_on_its_title_never_dates_the_notice() {
        let index = EvidenceIndex::build(&DocumentSource::from_pages(vec![
            crate::domain::SourcePage::new(
                1,
                "UNITED STATES DISTRICT COURT\n\nWINDROW.COM, INC., Plaintiff,\nv.\n\
                 CASCADE WEB SERVICES LLC, Defendant.\n\nNOTICE OF DEPOSITION\n\n\
                 PLEASE TAKE NOTICE that Plaintiff will take the deposition of Defendant \
                 on October 21, 2008 at 9:00 a.m.",
                PageOrigin::PlainText,
            ),
            crate::domain::SourcePage::new(
                2,
                "EXHIBIT A - VERIFIED AMENDED COMPLAINT FOR DAMAGES\n\nDated: January 5, 2007",
                PageOrigin::PlainText,
            ),
        ]));
        let context = whole(&index);
        let facts = ModelFacts {
            document_type: Some("Notice of Deposition".into()),
            type_evidence: vec![id_of(&index, "NOTICE OF DEPOSITION")],
            document_date: Some("2008-10-21".into()),
            date_evidence: vec![id_of(&index, "PLEASE TAKE NOTICE")],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &context, &index);
        assert_ne!(
            outcome.proposal.document_date.as_deref(),
            Some("2007-01-05"),
            "{:?}",
            outcome.reasons
        );
    }

    /// An exhibit of a court filing opens under the court's filing stamp,
    /// and is an attachment all the same.
    #[test]
    fn an_exhibit_under_a_filing_stamp_never_dates_the_notice() {
        let index = EvidenceIndex::build(&DocumentSource::from_pages(vec![
            crate::domain::SourcePage::new(
                1,
                "UNITED STATES DISTRICT COURT\n\nWINDROW.COM, INC., Plaintiff,\nv.\n\
                 CASCADE WEB SERVICES LLC, Defendant.\n\nNOTICE OF DEPOSITION\n\n\
                 PLEASE TAKE NOTICE that Plaintiff will take the deposition of Defendant \
                 on October 21, 2008 at 9:00 a.m.",
                PageOrigin::PlainText,
            ),
            crate::domain::SourcePage::new(
                2,
                "Case 2:08-cv-01234 Document 12-2 Filed 09/03/08 Page 1 of 5\n\nEXHIBIT A\n\n\
                 Dated: January 5, 2007",
                PageOrigin::PlainText,
            ),
        ]));
        let context = whole(&index);
        let facts = ModelFacts {
            document_type: Some("Notice of Deposition".into()),
            type_evidence: vec![id_of(&index, "NOTICE OF DEPOSITION")],
            document_date: Some("2008-10-21".into()),
            date_evidence: vec![id_of(&index, "PLEASE TAKE NOTICE")],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &context, &index);
        assert_ne!(
            outcome.proposal.document_date.as_deref(),
            Some("2007-01-05"),
            "{:?}",
            outcome.reasons
        );
    }

    /// An agency's advisory opinion laid out as a letter, to a company's
    /// counsel: the reply called the company the sender, but the inside
    /// address under its counsel's name says the opinion is to it.
    #[test]
    fn an_advisory_opinion_is_to_the_company_its_inside_address_names() {
        let (outcome, _) = facts_for(
            "FEDERAL ELECTION COMMISSION\nWashington, DC 20463\n\nMay 16, 1997\n\n\
             CERTIFIED MAIL\nRETURN RECEIPT REQUESTED\n\nADVISORY OPINION 1997-5\n\n\
             Paul B. O\u{2019}Kelly, General Counsel\nChicago Mercantile Exchange\n\
             30 South Wacker Drive\nChicago, Illinois 60606\n\nDear Mr. O\u{2019}Kelly:\n\n\
             This responds to your letter dated April 1, 1997, on behalf of the Chicago \
             Mercantile Exchange (\u{201c}CME\u{201d}) and its separate segregated fund, the \
             Commodity Futures Political Fund, regarding whether CME members could be \
             solicited for contributions.",
            |index| ModelFacts {
                document_type: Some("Advisory Opinion".into()),
                type_evidence: vec![id_of(index, "ADVISORY OPINION")],
                document_date: Some("1997-05-16".into()),
                date_evidence: vec![id_of(index, "May 16, 1997")],
                parties: vec![
                    party(
                        "Paul B. O'Kelly",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "This responds")],
                    ),
                    party(
                        "Chicago Mercantile Exchange",
                        Some(PartyRole::Sender),
                        &[id_of(index, "This responds")],
                    ),
                    party(
                        "Commodity Futures Political Fund",
                        Some(PartyRole::Sender),
                        &[id_of(index, "This responds")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::To,
            "{:?}",
            outcome.reasons
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Chicago Mercantile Exchange".to_owned()]
        );
    }

    /// The reply called the inside address's organization the notice's
    /// issuer, but the notice's own "Tenant:" field says who it is about:
    /// the role the document gives stands, and the notice stays "for" it.
    #[test]
    fn an_inside_address_leaves_a_role_the_document_gives_alone() {
        let (outcome, _) = facts_for(
            "HARBORVIEW PROPERTY MANAGEMENT\n\nMarch 2, 2026\n\n\
             Jane Roe, General Counsel\nKestrel Outfitters LLC\n14 Pier Road\n\
             Portland, Maine 04101\n\nDear Ms. Roe:\n\nNOTICE OF RENT INCREASE\n\n\
             Tenant: Kestrel Outfitters LLC\n\nThe monthly rent for Suite 4 will increase \
             to $4,200 from May 1, 2026.",
            |index| ModelFacts {
                document_type: Some("Notice of Rent Increase".into()),
                type_evidence: vec![id_of(index, "NOTICE OF RENT INCREASE")],
                document_date: Some("2026-03-02".into()),
                date_evidence: vec![id_of(index, "March 2, 2026")],
                parties: vec![
                    party(
                        "Jane Roe",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "Dear Ms. Roe")],
                    ),
                    party(
                        "Kestrel Outfitters LLC",
                        Some(PartyRole::Issuer),
                        &[id_of(index, "Tenant: Kestrel")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::For,
            "{:?} {:?}",
            outcome.proposal,
            outcome.reasons
        );
    }

    /// A letter whose reply typed it by something the letter never says is
    /// still a letter: its dateline over a "Dear ..." says so, and "Letter to
    /// Target Corporation" waits for a person in place of "Document to ...".
    /// Extracted text often runs the letter's head into one paragraph.
    #[test]
    fn a_document_laid_out_as_a_letter_is_a_letter_when_its_type_is_not_found() {
        let reply = |index: &EvidenceIndex| ModelFacts {
            document_type: Some("Civil Rights Letter".into()),
            type_evidence: vec![id_of(index, "As Attorneys General")],
            document_date: Some("2023-06-16".into()),
            date_evidence: vec![id_of(index, "June 16, 2023")],
            parties: vec![party(
                "Target Corporation",
                Some(PartyRole::Recipient),
                &[id_of(index, "Target Corporation")],
            )],
            ..ModelFacts::default()
        };
        for dateline in ["June 16, 2023", "Date: June 16, 2023"] {
            let (outcome, _) = facts_for(
                &format!(
                    "{dateline}\nBrian C. Cornell\nChairman and CEO\nTarget Corporation\n\
                     Minneapolis, MN 55403\nDear Mr. Cornell,\nAs Attorneys General we write to \
                     you about the company's merchandise decisions of June 16, 2023."
                ),
                reply,
            );
            assert_eq!(
                outcome.proposal.document_type.as_deref(),
                Some("Letter"),
                "{dateline}"
            );
            assert!(outcome.reasons.contains(&ReviewReason::TypeMissing));
            assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        }
        // A greeting with a dateline only below it is no letter's layout
        // either: a memo opening "Dear colleagues" with a dated section.
        let (outcome, _) = facts_for(
            "Dear Mr. Cornell,\n\nAs Attorneys General we write to Target Corporation about \
             its merchandise decisions.\n\nJune 16, 2023\n\nThe decisions of that day are set \
             out below.",
            reply,
        );
        assert_eq!(outcome.proposal.document_type, None);
        // A greeting with no dateline over it is no letter's layout: a
        // memo or a report can say "Dear colleagues".
        let (outcome, _) = facts_for(
            "Brian C. Cornell\nChairman and CEO\nTarget Corporation\nDear Mr. Cornell,\n\
             As Attorneys General we write to you about the company's merchandise decisions \
             of June 16, 2023.",
            reply,
        );
        assert_eq!(outcome.proposal.document_type, None);
    }

    /// Real letters are laid out in more ways than a written-out dateline
    /// over "Dear ...": a scan's "Dec. 26, 1907." over "Gentlemen:-", a
    /// published reply whose addressee was removed ("Dear" alone) that signs
    /// off "Yours sincerely", a cover letter with no greeting at all that
    /// opens on its date and closes "Yours faithfully,". Each is a letter.
    #[test]
    fn a_letters_greeting_dateline_or_sign_off_in_any_of_its_forms_names_it_a_letter() {
        let untyped = |_: &EvidenceIndex| ModelFacts::default();
        for text in [
            "WAR DEPARTMENT, C-B.\nOFFICE OF THE CHIEF SIGNAL OFFICER,\nWASHINGTON.\n\n\
             Dec. 26, 1907.\n\nWright Brothers,\n1127 West Third Street,\nDayton, Ohio.\n\n\
             Gentlemen:-\n\nYour letter of the 18th inst., with comments concerning the \
             tentative specification for a flying machine, is appreciated.\n\n\
             Very respectfully,\n\nChief Signal Officer of the Army.",
            "1 The Terrace\nWellington 6140\nReference: 20240412\n\n5 July 2024\n\nDear \n\n\
             Thank you for your Official Information Act request, transferred from the \
             Minister for Social Development 11 June 2024.\n\nYours sincerely\n\n\
             Laura Berntsen\nActing Manager, Welfare",
            "Room GC.05\n1 Horse Guards Road\nLondon\n\nPublic Accounts Committee\nHouse of \
             Commons\n\n4 January 2017\n\nThe Committee on Standards in Public Life has asked \
             me to draw your attention to their report of June 2014.\n\nYours faithfully,\n\n\
             Lesley Bainsfair\nHead of Secretariat",
            "| Dear | Mr. | Pilcher: |\n| This | refers | to | the | October | 26, | 1992 | \
             redistricting | plan |\n\nSincerely,\n\nJohn R. Dunne\nAssistant Attorney General",
            // A word processor's letter, its dateline opening on the day of
            // the week, each line a paragraph set in bold.
            "**1 Quay Road**\n\n**London**\n\n**SW1A 2HQ**\n\n**Thursday 10 September 2020**\n\n\
             **Tel: 020 7946 0000**\n\n**Mara Quill MP**\n\n**Chair, Harbour Party**\n\n\
             Dear Mara Quill MP,\n\nI hope you are well. We write jointly to update you on \
             our work.",
        ] {
            let (outcome, _) = facts_for(text, untyped);
            assert_eq!(
                outcome.proposal.document_type.as_deref(),
                Some("Letter"),
                "{text}"
            );
            assert!(outcome.reasons.contains(&ReviewReason::TypeMissing));
            assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
        }
        // A greeting alone, with neither a dateline over it nor a sign-off
        // under it, is still no letter's layout.
        let (outcome, _) = facts_for(
            "Brian C. Cornell\nChairman and CEO\nTarget Corporation\nDear Mr. Cornell,\n\
             As Attorneys General we write to you about the company's merchandise decisions \
             of January 4, 2017.",
            untyped,
        );
        assert_eq!(outcome.proposal.document_type, None);
        for (line, greeting) in [
            ("Dear", true),
            ("Dear,", true),
            ("Dear Sir or Madam:", true),
            ("To Whom It May Concern", true),
            ("GENTLEMEN:", true),
            ("| Dear | Mr. | Pilcher: |", true),
            ("Dearborn Heights", false),
            ("Sir Gus O'Donnell", false),
        ] {
            assert_eq!(is_greeting(line), greeting, "{line}");
        }
        for (line, closing) in [
            ("Sincerely,", true),
            ("Yours faithfully", true),
            ("Very respectfully,", true),
            ("Sincerely, Amanda L. Edens", true),
            ("Sincerely held beliefs", false),
            ("Regards to your family", false),
        ] {
            assert_eq!(is_closing(line), closing, "{line}");
        }
        for (line, dateline) in [
            ("Dec. 26, 1907.", true),
            ("18th October 2019", true),
            ("Date: 28 March 2023", true),
            ("Released January 3, 2024", true),
            ("02/09/2018", true),
            ("Thursday 10 September 2020", true),
            ("**Thursday 10 September 2020**", true),
            ("FRIDAY, JUNE 8, 2007", true),
            ("Date: Thurs., Sept. 10, 2020", true),
            ("Thursday", false),
            ("Thursday 10 September", false),
            ("10 September 2020 Thursday meeting", false),
            ("Sat on 10 September 2020", false),
            ("June 16, 2023 at noon", false),
            ("Released for public availability", false),
            ("Invoice 02/09/2018", false),
            ("02/09/18", false),
        ] {
            assert_eq!(is_dateline(line), dateline, "{line}");
        }
    }

    /// A letter's closing is not always on its last page. An agency's
    /// letter ends "Sincerely," on its last page of text and an
    /// electronic-signature page follows; a word processor's letter is one
    /// page, the statement it encloses after its signatures. Under a
    /// greeting on the first page the closing makes it a letter wherever it
    /// stands. With no greeting nothing says where a letter starts, and
    /// only a closing at the document's end counts.
    #[test]
    fn a_letter_signs_off_before_its_signature_page_or_its_enclosure() {
        let pages = |texts: &[&str]| {
            EvidenceIndex::build(&DocumentSource::from_pages(
                texts
                    .iter()
                    .enumerate()
                    .map(|(at, text)| {
                        crate::domain::SourcePage::new(at + 1, *text, PageOrigin::PlainText)
                    })
                    .collect(),
            ))
        };
        // As the worker extracts an agency's letter: letterhead and inside
        // address in one paragraph, no dateline, the only date on the
        // signature page appended after the closing's page.
        let index = pages(&[
            "DEPARTMENT OF HARBOURS & WATERWAYS Coastal Service\nPort Licensing \
             Administration\nSilver Spring, MD 20993\nMara T. Quill, PhD\nVice President, \
             Regulatory Affairs\nKestrel Outfitters, Inc.\n780 Dedham Street, Suite 800\n\
             Canton, MA 02021\n\nRE: Licence 208090\n\nDear Dr. Quill:\n\nAs part of its \
             monitoring programme, the Port Licensing Administration has reviewed an exhibit \
             booth submitted by Kestrel Outfitters, Inc.",
            "The violations discussed in this letter do not necessarily constitute an \
             exhaustive list.\n\nSincerely,\n\n{See appended electronic signature page}\n\n\
             Jonah Reed, RPh\nRegulatory Review Officer\nOffice of Licence Promotion",
            "This is a representation of an electronic record that was signed\n\n\
             electronically and this page is the manifestation of the electronic\n\n\
             signature.\n\n/s/\n\nJONAH P REED\n\n02/09/2018",
        ]);
        let facts = ModelFacts {
            document_type: Some("Letter from the Office of Licence Promotion to Kestrel".into()),
            type_evidence: vec![id_of(&index, "DEPARTMENT OF HARBOURS")],
            document_date: Some("2018-02-09".into()),
            date_evidence: vec![id_of(&index, "02/09/2018")],
            parties: vec![
                party(
                    "Kestrel Outfitters, Inc.",
                    Some(PartyRole::Client),
                    &[id_of(&index, "DEPARTMENT OF HARBOURS")],
                ),
                party(
                    "Port Licensing Administration",
                    Some(PartyRole::Issuer),
                    &[id_of(&index, "DEPARTMENT OF HARBOURS")],
                ),
            ],
            ..ModelFacts::default()
        };
        let outcome = check(facts, &whole(&index), &index);
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Letter"),
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::To);
        assert_eq!(outcome.proposal.parties, vec!["Kestrel Outfitters, Inc."]);
        assert!(outcome.reasons.contains(&ReviewReason::TypeMissing));
        // A word processor's file is one page: the points the letter
        // encloses run on for more lines than a letter's end is read for.
        let enclosure = (1..=40)
            .map(|at| format!("{at}. Our members will lead by example in clause {at}."))
            .collect::<Vec<_>>()
            .join("\n\n");
        let untyped = |_: &EvidenceIndex| ModelFacts::default();
        let (outcome, _) = facts_for(
            &format!(
                "1 Quay Road\n\nLondon\n\nTel: 020 7946 0000\n\nMara Quill MP\n\n\
                 Dear Mara Quill MP,\n\nWe write jointly to update you on our work.\n\n\
                 Yours sincerely,\n\nJonah Reed\n\n{enclosure}"
            ),
            untyped,
        );
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Letter"));
        // No greeting: a dated paper that quotes a closing before its last
        // lines is no letter by this.
        let (outcome, _) = facts_for(
            &format!(
                "4 January 2017\n\nThe Committee has asked me to draw your attention to \
                 their report.\n\nYours faithfully,\n\nLesley Bainsfair\n\n{enclosure}"
            ),
            untyped,
        );
        assert_eq!(outcome.proposal.document_type, None);
        // A closing over the greeting is not the letter's own: the page
        // quotes the end of another letter before it greets anyone.
        let (outcome, _) = facts_for(
            &format!(
                "Yours sincerely,\n\nLesley Bainsfair\n\nDear Mara Quill MP,\n\nWe write \
                 jointly to update you on our work.\n\n{enclosure}"
            ),
            untyped,
        );
        assert_eq!(outcome.proposal.document_type, None);
    }

    /// An email by its header block and a notice by its heading stand in
    /// for a kind the reply could not find, each a bare kind that waits for
    /// a person.
    #[test]
    fn an_email_or_a_notice_stands_in_for_a_kind_the_reply_could_not_find() {
        let untyped = |_: &EvidenceIndex| ModelFacts::default();
        for (text, kind) in [
            (
                "From: <pkginfo@ups.com>\nTo: <susan.bailey@enron.com>\nDate: Wed, 20 Mar 2002 \
                 15:48:51 -0800 (PST)\nSubject: UPS Ship Notification, 1Z7261100192688747\n\n\
                 At the request of ENRON CORP, this notice confirms the following shipment \
                 has been sent.",
                "Email",
            ),
            (
                "NOTICE OF THE PRIMARY UNIVERSAL SERVICE PROVIDER FOR THE EXTENDED ZONES OF \
                 AUSTRALIA\n\nOn 1 June 2001 the Commonwealth and Telstra Corporation Limited \
                 entered into a written agreement under section 57 of the Telstra Corporation \
                 Act 1991. Notice is hereby given pursuant to subsection 12E(6) of the Act.",
                "Notice",
            ),
        ] {
            let (outcome, _) = facts_for(text, untyped);
            assert_eq!(
                outcome.proposal.document_type.as_deref(),
                Some(kind),
                "{text}"
            );
            assert!(
                outcome.reasons.contains(&ReviewReason::TypeMissing),
                "{kind}"
            );
        }
        // A sheet of figures with no such head is nothing a layout names.
        let (outcome, _) = facts_for(
            "Bill To:\nPHILIP MORRIS USA\n120 PARK AVENUE\n\n| 16MB UPGRADE | 55.00 | 110.00 |",
            untyped,
        );
        assert_eq!(outcome.proposal.document_type, None);
    }

    /// A letter's inside address names the organization it is to under
    /// the person, in more ways than an organization's word: a name set in
    /// capitals, a committee a post is held at, the organization's line
    /// over the person. The first person the address names is the
    /// addressee; a second's organization never takes their place, and a
    /// person's line or a city's never stands for an organization.
    #[test]
    fn a_letters_inside_address_names_the_organization_in_more_ways() {
        let addressed = |person: &str| {
            let person = person.to_owned();
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some("Letter".into()),
                document_date: Some("2011-07-11".into()),
                date_evidence: vec![id_of(index, "July 2011")],
                parties: vec![party(
                    &person,
                    Some(PartyRole::Addressee),
                    &[id_of(index, person.split_whitespace().next().unwrap())],
                )],
                ..ModelFacts::default()
            }
        };
        for (text, person, organization) in [
            (
                "Ed Richards\nChief Executive\nOFCOM\nRiverside House\n2a Southwark Bridge Road\n\
                 London\nSE1 9HA\n\n11 July 2011\n\nDear Ed\n\nAs you are aware, my consultation \
                 closed on Friday.\n\nYours ever\n\nRt Hon Jeremy Hunt MP",
                "Ed Richards",
                "ofcom",
            ),
            (
                "Lord Evans of Weardale KCB DL\nChair\nCommittee on Standards in Public Life\n\
                 1 Horse Guards Road\nLondon SW1A 2HQ\n\n11 July 2011\n\nDear Lord Evans,\n\n\
                 Thank you for your letter of 16 September.\n\nYours sincerely,\n\nAmelia Womack",
                "Lord Evans of Weardale",
                "committee on standards in public life",
            ),
            (
                "Rt Hon Yvette Cooper MP\nChair, Home Affairs Select Committee\nHouse of Commons\n\
                 London\n\n11 July 2011\n\nDear Chair,\n\nPlease find below a further update.\n\n\
                 Yours sincerely,\n\nRt Hon Priti Patel MP",
                "Rt Hon Yvette Cooper MP",
                "home affairs select committee",
            ),
            (
                "11 July 2011\n\nDUQUESNE LIGHT COMPANY\nEMILY M FARAH CNSL\n411 SEVENTH AVENUE\n\
                 PITTSBURGH PA 15219\n\nDear Ms. Farah:\n\nReceipt is acknowledged of the Letter \
                 of Notification.\n\nVery truly yours,\n\nRosemary Chiavetta",
                "EMILY M FARAH CNSL",
                "duquesne light company",
            ),
        ] {
            let (outcome, _) = facts_for(text, addressed(person));
            assert_eq!(
                outcome
                    .proposal
                    .parties
                    .first()
                    .map(|name| name.to_lowercase()),
                Some(organization.to_owned()),
                "{text}"
            );
            assert_eq!(outcome.proposal.party_relation, PartyRelation::To, "{text}");
        }
        // Two addressees: the first has no organization under them, so the
        // letter stays to them, not to the second's department. The lines
        // of a street never name an organization, nor does a person's line.
        let (outcome, _) = facts_for(
            "Kit Malthouse MP\nMinister of State for Crime, Policing and the Fire Service\n\
             2 Marsham Street\nLondon\nSW1P 4DF\n\nJo Churchill MP\nParliamentary Under Secretary \
             of State\nDepartment of Health and Social Care\n39 Victoria Street\nLondon\n\n\
             11 July 2011\n\nDear Ministers,\n\nThe Council has considered the proposal.\n\n\
             Yours sincerely,\n\nOwen Bowden-Jones",
            addressed("Kit Malthouse MP"),
        );
        assert_eq!(outcome.proposal.parties, vec!["Kit Malthouse MP"]);
        for (line, named) in [
            ("OFCOM", true),
            ("Daily Telegraph", true),
            ("POST AND SCHELL", true),
            ("Jo Churchill MP", false),
            ("Mr. Carl Malamud", false),
            ("London", false),
            ("DJ", false),
            ("Dayton, Ohio.", false),
            ("Department for", false),
            ("Notice of Prior Authorization Approval", false),
            ("2525 Natomas Park Drive", false),
        ] {
            assert_eq!(
                organization_line(line) && !names_a_person(line),
                named,
                "{line}"
            );
        }
        // A second person under the addressee is a person, not their
        // organization.
        let (outcome, _) = facts_for(
            "John Smith\nJane Doe\n123 Main Street\nLondon\n\n11 July 2011\n\nDear John\n\n\
             Thank you for your letter.\n\nYours sincerely,\n\nAmelia Womack",
            addressed("John Smith"),
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|name| name.contains("Jane Doe")),
            "{:?}",
            outcome.proposal.parties
        );
        // A second addressee's post and organization are theirs, not the
        // first addressee's.
        let (outcome, _) = facts_for(
            "John Smith\n\nJane Doe\nDirector\nAcme Ltd\n123 Main Street\nLondon\n\n\
             11 July 2011\n\nDear John and Jane\n\nThank you for your letter.\n\n\
             Yours sincerely,\n\nAmelia Womack",
            addressed("John Smith"),
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|name| name.contains("Acme")),
            "{:?}",
            outcome.proposal.parties
        );
    }

    /// A letter signed by a person for an organization is from the
    /// organization: the signature block names it under the signer's post,
    /// or the letterhead names it alone. A "cc:" line under the signature
    /// names no one the letter is from.
    #[test]
    fn a_letter_signed_for_an_organization_is_from_it() {
        let sent_by = |person: &str| {
            let person = person.to_owned();
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some("Letter".into()),
                document_date: Some("2019-09-16".into()),
                date_evidence: vec![id_of(index, "September 2019")],
                parties: vec![party(
                    &person,
                    Some(PartyRole::Sender),
                    &[id_of(index, person.split_whitespace().next().unwrap())],
                )],
                ..ModelFacts::default()
            }
        };
        let (outcome, _) = facts_for(
            "Fleetbank House\n2-6 Salisbury Square\nLondon EC4Y 8JX\n\n16 September 2019\n\n\
             Dear,\n\nThank you for your email of 9 September requesting information.\n\n\
             Yours sincerely,\n\nRashmi Panigrahi\nFOI Officer\nOffice of Manpower Economics\n\
             Fleetbank House",
            sent_by("Rashmi Panigrahi"),
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Office of Manpower Economics"]
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
        let (outcome, _) = facts_for(
            "BUILDING STANDARDS COMMISSION\n2525 Natomas Park Drive, Suite 130\nSacramento, \
             California 95833\n\n16 September 2019\n\nDebra Rose\nSr. Project Manager\nNational \
             Fire Protection Association, Inc.\n1 Batterymarch Park\n\nDear Debra,\n\nPlease find an \
             executed copy of the agreement enclosed.\n\nRegards,\n\nCynthia Biedermann",
            sent_by("Cynthia Biedermann"),
        );
        assert!(
            outcome
                .proposal
                .parties
                .iter()
                .any(|name| name.eq_ignore_ascii_case("Building Standards Commission")),
            "{:?}",
            outcome.proposal.parties
        );
        let (outcome, _) = facts_for(
            "Federal Communications Commission\nWashington, D.C. 20554\n\n16 September 2019\n\n\
             Andrew D. Lipman\nMorgan, Lewis & Bockius LLP\n1111 Pennsylvania Avenue, NW\n\n\
             Dear Mr. Lipman:\n\nThis responds to your letter.\n\nSincerely,\n\nKris Monteith\n\
             Chief\ncc: Morgan, Lewis & Bockius LLP",
            sent_by("Kris Monteith"),
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|name| name.contains("Bockius")),
            "{:?}",
            outcome.proposal.parties
        );
        // A name in the address under the signature is not the signer's:
        // "Thames Water" three lines under the closing stays itself, and
        // the site on the line after it is no organization of its own.
        let (outcome, _) = facts_for(
            "Thames Water
Development Planning

16 September 2019

Dear developer,

             Thank you for your enquiry.

Yours faithfully,

Development Planning Department
             Development Planning,
Thames Water,
Maple Lodge STW,
Denham Way,
Rickmansworth,
             WD3 9SQ,",
            sent_by("Thames Water"),
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|name| name.contains("Maple Lodge")),
            "{:?}",
            outcome.proposal.parties
        );
        // The signature block ends at the street: the city under it is no
        // organization, and the person who signed stays.
        let (outcome, _) = facts_for(
            "16 September 2019\n\nDear Sir,\n\nI write about the noise.\n\nSincerely,\n\n\
             John Smith\n123 Main Street\nNew York\nNY 10001",
            sent_by("John Smith"),
        );
        assert_eq!(outcome.proposal.parties, vec!["John Smith"]);
        // A co-signer under the signer is a person, not the signer's
        // organization, and their post and organization are theirs.
        let (outcome, _) = facts_for(
            "16 September 2019\n\nDear Sir,\n\nWe write about the noise.\n\nSincerely,\n\n\
             John Smith\nJane Doe",
            sent_by("John Smith"),
        );
        assert_eq!(outcome.proposal.parties, vec!["John Smith"]);
        let (outcome, _) = facts_for(
            "16 September 2019\n\nDear Sir,\n\nWe write about the noise.\n\nSincerely,\n\n\
             John Smith\nJane Doe\nDirector\nAcme Ltd",
            sent_by("John Smith"),
        );
        assert_eq!(outcome.proposal.parties, vec!["John Smith"]);
        // A number in the organization's own name is no street number.
        let (outcome, _) = facts_for(
            "16 September 2019\n\nDear Sir,\n\nPlease find the samples enclosed.\n\n\
             Sincerely,\n\nJohn Smith\n3M Company\n3M Center\nSt. Paul, MN 55144",
            sent_by("John Smith"),
        );
        assert_eq!(outcome.proposal.parties, vec!["3M Company"]);
        // A British layout puts the inside address above the dateline: with
        // no addressee named to mark where it begins, the company over the
        // person written to is not the signer's letterhead.
        let (outcome, _) = facts_for(
            "Acme Ltd\nJohn Smith\n123 Main Street\nLondon\n\n16 September 2019\n\n\
             Dear John,\n\nThank you for your letter.\n\nSincerely,\n\nJane Doe",
            sent_by("Jane Doe"),
        );
        assert_eq!(outcome.proposal.parties, vec!["Jane Doe"]);
        // A marking at the head of the letter is no organization: the
        // letterhead under it is the signer's.
        let (outcome, _) = facts_for(
            "WITHOUT PREJUDICE\nAcme Legal LLP\n\n16 September 2019\n\nJohn Smith\n\
             123 Main Street\n\nDear John,\n\nThank you for your letter.\n\nSincerely,\n\n\
             Jane Doe",
            sent_by("Jane Doe"),
        );
        assert_eq!(outcome.proposal.parties, vec!["Acme Legal LLP"]);
    }

    /// A caption sets its sides "against" each other as well as "v.", with
    /// or without a label beside each; and when the reply named one side
    /// only, the caption's other side is the other party. A trading name
    /// is no part of a party's name.
    #[test]
    fn a_caption_set_against_its_sides_or_missing_one_is_still_read() {
        let (outcome, _) = facts_for(
            "UNITED STATES DISTRICT COURT\nSOUTHERN DISTRICT OF NEW YORK\n\nSOMPO JAPAN INSURANCE \
             COMPANY OF AMERICA,\nPlaintiffs,\n- against -\nNORFOLK SOUTHERN RAILWAY COMPANY,\n\
             Defendants.\n\nNOTICE OF APPEAL\n\nNotice is hereby given that defendants hereby appeal \
             to the United States Court of Appeals.\n\nDated: October 13, 2009",
            |index| ModelFacts {
                document_type: Some("Notice of Appeal".into()),
                type_evidence: vec![id_of(index, "NOTICE OF APPEAL")],
                document_date: Some("2009-10-13".into()),
                date_evidence: vec![id_of(index, "Dated")],
                parties: vec![
                    party(
                        "Sompo Japan Insurance Company of America",
                        Some(PartyRole::Issuer),
                        &[id_of(index, "SOMPO")],
                    ),
                    party(
                        "Norfolk Southern Railway Company",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "NORFOLK")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        let (outcome, _) = facts_for(
            "UNITED STATES DISTRICT COURT\nSOUTHERN DISTRICT OF NEW YORK\n\nLUXO AS,\n-against-\n\
             THE WALT DISNEY COMPANY,\n\nNOTICE OF DISMISSAL\n\nPursuant to Rule 41(a), plaintiff \
             hereby dismisses this action with prejudice.\n\nDated: November 2, 2009",
            |index| ModelFacts {
                document_type: Some("Notice of Dismissal".into()),
                type_evidence: vec![id_of(index, "NOTICE OF DISMISSAL")],
                document_date: Some("2009-11-02".into()),
                date_evidence: vec![id_of(index, "Dated")],
                parties: vec![
                    party("Luxo AS", Some(PartyRole::Client), &[id_of(index, "LUXO")]),
                    party(
                        "The Walt Disney Company",
                        Some(PartyRole::Contractor),
                        &[id_of(index, "DISNEY")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        assert_eq!(
            outcome.proposal.parties,
            vec!["Luxo AS", "The Walt Disney Company"]
        );
        let (outcome, _) = facts_for(
            "UNITED STATES DISTRICT COURT\nSOUTHERN DISTRICT OF NEW YORK\n\nTHE MCGRAW-HILL \
             COMPANIES, INC., and PEARSON EDUCATION, INC.,\nPlaintiffs,\nv.\nGOOGLE INC.,\n\
             Defendant.\n\nNOTICE OF INTENT TO SERVE SUBPOENA\n\nGoogle Inc. will serve a subpoena \
             on the Association of American Publishers, Inc.\n\nDated: November 22, 2006",
            |index| ModelFacts {
                document_type: Some("Notice of Intent to Serve Subpoena".into()),
                type_evidence: vec![id_of(index, "NOTICE OF INTENT")],
                document_date: Some("2006-11-22".into()),
                date_evidence: vec![id_of(index, "Dated")],
                parties: vec![
                    party(
                        "Google Inc.",
                        Some(PartyRole::Sender),
                        &[id_of(index, "GOOGLE")],
                    ),
                    party(
                        "Association of American Publishers, Inc.",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "Association")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        assert!(
            outcome.proposal.parties[0]
                .to_lowercase()
                .contains("mcgraw-hill companies"),
            "{:?}",
            outcome.proposal.parties
        );
        // The court's jurisdiction is no side of the case, and a side
        // named with an "of" phrase is read whole.
        let (outcome, index) = facts_for(
            "SUPERIOR COURT\nSNOHOMISH COUNTY\n\nNORTHWIND LLC,\nPlaintiff,\nv.\n\
             ACME INC.,\nDefendant.\n\nNOTICE OF HEARING\n\nPlease take notice that the motion \
             will be heard.\n\nDated: November 22, 2006",
            |index| ModelFacts {
                document_type: Some("Notice of Hearing".into()),
                type_evidence: vec![id_of(index, "NOTICE OF HEARING")],
                document_date: Some("2006-11-22".into()),
                date_evidence: vec![id_of(index, "Dated")],
                parties: vec![party(
                    "Acme Inc.",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "ACME")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
        assert!(
            outcome.proposal.parties[0]
                .to_lowercase()
                .contains("northwind llc"),
            "{:?}",
            outcome.proposal.parties
        );
        // An "of" phrase is read whole however the caption sets it.
        let unit = index
            .units()
            .iter()
            .find(|unit| unit.text.contains("SNOHOMISH"))
            .unwrap();
        let mut caps = unit.clone();
        caps.text = "FIRST NATIONAL BANK OF OMAHA,".to_owned();
        assert_eq!(
            with_of_phrase(&caps, "FIRST NATIONAL BANK"),
            "FIRST NATIONAL BANK OF OMAHA"
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|name| name.to_lowercase().contains("snohomish")),
            "{:?}",
            outcome.proposal.parties
        );
        assert_eq!(
            without_trading_name("Farringdon Energy Limited trading as Champion Energy"),
            "Farringdon Energy Limited"
        );
        assert_eq!(
            without_trading_name("CWFS Insight LLC d/b/a RealINSIGHT Marketplace"),
            "CWFS Insight LLC"
        );
        assert_eq!(without_trading_name("Olymbec USA LLC"), "Olymbec USA LLC");
        // A letter whose lowering is longer than itself leaves the cut on
        // a character boundary.
        assert_eq!(
            without_trading_name("İstanbul Ağ Ltd. Şti. trading as Brand"),
            "İstanbul Ağ Ltd. Şti."
        );
        assert_eq!(without_trading_name("K商 trading as Brand"), "K商");
    }

    /// An FCC letter in the real set, typed by the order it closes: a type
    /// read below the salutation names what the letter is about.
    #[test]
    fn a_letter_typed_by_what_it_talks_about_goes_to_review() {
        let (outcome, _) = facts_for(
            "Federal Communications Commission\nWashington, D.C. 20554\n\nAugust 9, 2013\n\n\
             Kestrel Broadcasting, LLC\n68 Grigware Road\nPlattsburgh, New York 12901\n\n\
             Dear Licensee:\n\nThis is with respect to the October 24, 2012 Order to Show Cause \
             released by the Video Division. We find that the licensee responded adequately.\n\n\
             ACCORDINGLY, the Order to Show Cause proceeding IS TERMINATED.\n\nSincerely,",
            |index| ModelFacts {
                document_type: Some("Order to Show Cause".into()),
                type_evidence: vec![id_of(index, "This is with respect")],
                document_date: Some("2013-08-09".into()),
                date_evidence: vec![id_of(index, "August 9, 2013")],
                parties: vec![party(
                    "Kestrel Broadcasting, LLC",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Kestrel Broadcasting")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            outcome.reasons.contains(&ReviewReason::TypeUnsupported),
            "{:?}",
            outcome.reasons
        );
        assert_ne!(outcome.status, ProposalStatus::Ready);
    }

    /// A notice headed with its type, the reply citing the body's mention of
    /// it below the salutation: the heading states the type, so it stands
    /// (Codex's case on #51).
    #[test]
    fn a_type_its_heading_states_stands_when_the_body_is_cited() {
        let (outcome, _) = facts_for(
            "Harlow Street Properties LLC\n\nNOTICE OF TERMINATION\n\nMarch 2, 2026\n\n\
             Marrow Bakery Inc.\n14 Harlow Street\n\nDear Tenant:\n\n\
             This notice of termination confirms that your lease ends on April 30, 2026.",
            |index| ModelFacts {
                document_type: Some("Notice of Termination".into()),
                type_evidence: vec![id_of(index, "This notice of termination")],
                document_date: Some("2026-03-02".into()),
                date_evidence: vec![id_of(index, "March 2, 2026")],
                parties: vec![party(
                    "Marrow Bakery Inc.",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Marrow Bakery")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            !outcome.reasons.contains(&ReviewReason::TypeUnsupported),
            "{:?}",
            outcome.reasons
        );
        // A "Re:" line above the salutation names the matter, not the
        // letter's own title (InternBench's letter approving a trade credit
        // application).
        let (outcome, _) = facts_for(
            "Moonlit Paper Co.\n\nMarch 19, 2026\n\nBriarwood Coffee LLC\n310 Harbor Street\n\n\
             Re: Approval of Trade Credit Application\nDear Ms. Ambrose:\n\n\
             Thank you for the trade credit application, which we have approved.",
            |index| ModelFacts {
                document_type: Some("Trade Credit Application".into()),
                type_evidence: vec![id_of(index, "Thank you")],
                document_date: Some("2026-03-19".into()),
                date_evidence: vec![id_of(index, "March 19, 2026")],
                parties: vec![party(
                    "Briarwood Coffee LLC",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Briarwood")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            outcome.reasons.contains(&ReviewReason::TypeUnsupported),
            "{:?}",
            outcome.reasons
        );
    }

    /// PHMSA's warning letter in the real set: addressed to the refinery's
    /// manager at the refinery. It is to the company, and its own title above
    /// the salutation is its type.
    #[test]
    fn a_letter_to_a_person_at_a_company_is_to_the_company() {
        let (outcome, _) = facts_for(
            "Pipeline and Hazardous Materials Safety Administration\n\nWARNING LETTER\n\n\
             March 2, 2018\n\nMr. Jerome Colfax\nRefinery Manager\nTollard Refining Company, LLC\n\
             1819 Woodville Road\nOregon, OH 43616\n\nDear Mr. Colfax:\n\n\
             From June 5 - 9, 2017, a representative of the Administration inspected the \
             pipeline facilities of Tollard Refining Company, LLC in Toledo, Ohio.",
            |index| ModelFacts {
                document_type: Some("Warning Letter".into()),
                type_evidence: vec![id_of(index, "WARNING LETTER")],
                document_date: Some("2018-03-02".into()),
                date_evidence: vec![id_of(index, "March 2, 2018")],
                parties: vec![party(
                    "Jerome Colfax",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Mr. Jerome Colfax")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Tollard Refining Company, LLC"],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::To);
        assert!(!outcome.reasons.contains(&ReviewReason::TypeUnsupported));
        // A title that ends on an organization's word is the person's post,
        // never the organization (Codex's case on #51).
        let (outcome, _) = facts_for(
            "Pipeline and Hazardous Materials Safety Administration\n\nWARNING LETTER\n\n\
             March 2, 2018\n\nMr. Jerome Colfax\nDirector of Corporate Services\n\
             Tollard Refining Company, LLC\n1819 Woodville Road\n\nDear Mr. Colfax:\n\n\
             A representative of the Administration inspected the pipeline facilities.",
            |index| ModelFacts {
                document_type: Some("Warning Letter".into()),
                type_evidence: vec![id_of(index, "WARNING LETTER")],
                document_date: Some("2018-03-02".into()),
                date_evidence: vec![id_of(index, "March 2, 2018")],
                parties: vec![party(
                    "Jerome Colfax",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Mr. Jerome Colfax")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Tollard Refining Company, LLC"],
            "{:?}",
            outcome.proposal
        );
        // The address ends at the salutation: a sentence of the body naming
        // a firm is never the addressee's organization (Codex's case on #51).
        let (outcome, _) = facts_for(
            "Corvane Property Group\n\nDEMAND LETTER\n\nMarch 2, 2018\n\n\
             Ms. Dana Wells\n12 Elm Street\nDear Ms. Wells,\nOn behalf of Pike & Rowe LLP",
            |index| ModelFacts {
                document_type: Some("Demand Letter".into()),
                type_evidence: vec![id_of(index, "DEMAND LETTER")],
                document_date: Some("2018-03-02".into()),
                date_evidence: vec![id_of(index, "March 2, 2018")],
                parties: vec![party(
                    "Dana Wells",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Dana Wells")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|party| party.to_lowercase().contains("rowe")),
            "{:?}",
            outcome.proposal
        );
        // A person in the letterhead, above the dateline, is the sender's:
        // a reply that called them the recipient never makes the sender's
        // firm the addressee.
        let (outcome, _) = facts_for(
            "Ansel Pike\nManaging Partner\nPike & Rowe LLP\n\nDEMAND LETTER\n\nMarch 2, 2018\n\n\
             Ms. Dana Wells\n12 Elm Street\n\nDear Ms. Wells:\n\n\
             We write about the lease of 12 Elm Street.",
            |index| ModelFacts {
                document_type: Some("Demand Letter".into()),
                type_evidence: vec![id_of(index, "DEMAND LETTER")],
                document_date: Some("2018-03-02".into()),
                date_evidence: vec![id_of(index, "March 2, 2018")],
                parties: vec![party(
                    "Ansel Pike",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Ansel Pike")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|party| party.to_lowercase().contains("rowe")),
            "{:?}",
            outcome.proposal
        );
    }

    /// A notice of appeal in the real set: three plaintiffs, one label after
    /// all of them, then "v." and the defendant who appeals. The caption is
    /// read as a block: between the first plaintiff and the defendant, never
    /// "from" the first plaintiff.
    #[test]
    fn a_caption_with_several_plaintiffs_is_read_as_a_block() {
        let (outcome, _) = facts_for(
            "UNITED STATES DISTRICT COURT\nSOUTHERN DISTRICT OF NEW YORK\n\n\
             BRAMBLE CAPITAL INC., MERRIT LYNX & CO.,\nAND STANWICK & CO., INCORPORATED,\n\
             Plaintiffs,\nV.\nOWLWATCH.COM, INC.,\nDefendant.\n\nNOTICE OF APPEAL\n\n\
             Defendant Owlwatch.com, Inc. hereby appeals from the Judgment entered March 22, 2010.\n\n\
             Dated: April 9, 2010",
            |index| ModelFacts {
                document_type: Some("Notice of Appeal".into()),
                type_evidence: vec![id_of(index, "NOTICE OF APPEAL")],
                document_date: Some("2010-04-09".into()),
                date_evidence: vec![id_of(index, "Dated")],
                parties: vec![
                    party(
                        "Owlwatch.com, Inc.",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "hereby appeals")],
                    ),
                    party(
                        "Bramble Capital Inc.",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "BRAMBLE")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::Between,
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.parties.len(), 2, "{:?}", outcome.proposal);
        assert!(
            outcome.proposal.parties[0]
                .to_lowercase()
                .contains("bramble"),
            "{:?}",
            outcome.proposal
        );
    }

    /// A pleading opens with its counsel's block - names, firm, address,
    /// telephone - and the court's heading stands under it, past where a
    /// page's opening units end. The caption is read from the heading down,
    /// so the block takes none of its room: a scanned notice of appeal as
    /// the reader extracts it, the docket number and the title run into the
    /// caption's lines. A sentence that far down which only mentions a
    /// court and its case heads no caption.
    #[test]
    fn a_caption_under_counsels_block_is_read_from_its_forums_heading() {
        const COUNSEL: &str = "Rowan D. Keeling (admitted pro hac vice) Charles L. Hartwell \
(admitted pro hac vice)\n\n\
| Keeling Cove & Hartwell P.C. | PLEDU.S. DC |\n\
| One Pitcairn Place |\n\
| 165 Township Line Rd. | OCT 13 2009 |\n\n\
Jenkintown, PA 19046\n\n\
(215) 555-0110\n\n\
M: Y.\n\n\
and Barry N. Gutterfield Gutterfield & Associates The Lincoln Building 60 East 42d Street, \
46th Floor New York, NY 10165 Attorneys for Defendants\n\n";
        const CAPTION: &str = "UNITED STATES DISTRICT COURT\n\n\
SOUTHERN DISTRICT OF NEW YORK SENDAI MARINE INSURANCE COMPANY OF AMERICA, and SENDAI MARINE \
INSURANCE,\n\n\
INC. NO.: 07 CIV. 2735 (DC)\n\n\
Plaintiffs, JUDGE CHIN\n\n\
- against - NOTICE OF APPEAL\n\n\
NORTHFOLK EASTERN RAILWAY COMPANY, NORTHFOLK EASTERN CORPORATION, and THE KANSAS PLAINS \
RAILWAY COMPANY\n\n\
Defendants. Notice is hereby given that defendants hereby appeal to the United States Court \
of Appeals for the Second Circuit from the Judgment entered in this action.\n\n\
Dated: October 13, 2009";
        const LIEN: &str = "NOTICE OF CHARGING LIEN\n\n\
Please take notice that the undersigned claim a lien on any recovery in the action pending in \
the United States District Court for the Southern District of New York, in which Plaintiff \
Sendai Marine Insurance Company of America proceeds against Defendant Northfolk Eastern \
Railway Company.\n\n\
Dated: October 13, 2009";
        let sides = |title: &'static str, plaintiff: &'static str, defendant: &'static str| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some(title.into()),
                type_evidence: vec![id_of(index, title)],
                document_date: Some("2009-10-13".into()),
                date_evidence: vec![id_of(index, "Dated")],
                parties: vec![
                    party(
                        plaintiff,
                        Some(PartyRole::Client),
                        &[id_of(index, plaintiff)],
                    ),
                    party(
                        defendant,
                        Some(PartyRole::Client),
                        &[id_of(index, defendant)],
                    ),
                ],
                ..ModelFacts::default()
            }
        };
        let (outcome, index) = facts_for(
            &format!("{COUNSEL}{CAPTION}"),
            sides(
                "NOTICE OF APPEAL",
                "SENDAI MARINE INSURANCE COMPANY OF AMERICA",
                "NORTHFOLK EASTERN RAILWAY COMPANY",
            ),
        );
        // The heading stands past the page's opening units.
        let heading = index
            .units()
            .iter()
            .position(|unit| unit.text.contains("DISTRICT COURT"))
            .unwrap();
        assert!(heading >= FORUM_HEAD_UNITS, "{heading}");
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::Between,
            "{:?}",
            outcome.proposal
        );
        assert_eq!(
            outcome.proposal.parties,
            vec![
                "SENDAI MARINE INSURANCE COMPANY OF AMERICA",
                "NORTHFOLK EASTERN RAILWAY COMPANY"
            ]
        );
        assert!(
            !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
            "{:?}",
            outcome.reasons
        );
        // Under the same block, a notice that names the court and the
        // sides of a case only in the sentence it gives is no document of
        // the case.
        let (outcome, _) = facts_for(
            &format!("{COUNSEL}{LIEN}"),
            sides(
                "NOTICE OF CHARGING LIEN",
                "Sendai Marine Insurance Company of America",
                "Northfolk Eastern Railway Company",
            ),
        );
        assert_ne!(
            outcome.proposal.party_relation,
            PartyRelation::Between,
            "{:?}",
            outcome.proposal
        );
    }

    /// Past the page's opening units, only a line that reads as a court's
    /// heading, over a caption set out as one, heads a caption: a letter's
    /// inside address on a court's street and a contract's section on
    /// arbitration head none, though the notice's body reserves rights
    /// "against" its tenant. The notice stays for the tenant.
    #[test]
    fn a_court_named_far_down_a_page_heads_a_caption_only_over_one() {
        const LETTERHEAD: &str = "ACME PROPERTIES LLC\n\n200 Harbor Way\n\n\
Boston, MA 02110\n\n(617) 555-0100\n\nMarch 4, 2024\n\nVIA CERTIFIED MAIL\n\n\
RETURN RECEIPT REQUESTED\n\nAND FIRST CLASS MAIL\n\n";
        let body = |title: &str| {
            format!(
                "{title}\n\nAcme Properties LLC, as Landlord, gives notice that you are in \
                 default of the Lease for failure to pay the rent due on March 1, 2024. \
                 Landlord reserves all of its rights and remedies against Beta Bakery LLC and \
                 its guarantor.\n\nACME PROPERTIES LLC"
            )
        };
        let notice = |title: &'static str| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some(title.into()),
                type_evidence: vec![id_of(index, title)],
                document_date: Some("2024-03-04".into()),
                date_evidence: vec![id_of(index, "March 4, 2024")],
                parties: vec![
                    party(
                        "Acme Properties LLC",
                        Some(PartyRole::Landlord),
                        &[id_of(index, "ACME PROPERTIES LLC")],
                    ),
                    party(
                        "Beta Bakery LLC",
                        Some(PartyRole::Tenant),
                        &[id_of(index, "BETA BAKERY LLC")],
                    ),
                ],
                ..ModelFacts::default()
            }
        };
        for (address, title) in [
            (
                "BETA BAKERY LLC\n\n15 COURT STREET\n\nBOSTON, MA 02108\n\n",
                "NOTICE OF DEFAULT",
            ),
            (
                "BETA BAKERY LLC\n\nSECTION 14. ARBITRATION\n\nBOSTON, MA 02108\n\n",
                "NOTICE OF TERMINATION",
            ),
            (
                "BETA BAKERY LLC\n\nARBITRATION\n\nBOSTON, MA 02108\n\n",
                "NOTICE OF TERMINATION",
            ),
        ] {
            let text = format!("{LETTERHEAD}{address}{}", body(title));
            let (outcome, index) = facts_for(&text, notice(title));
            let forum = index
                .units()
                .iter()
                .position(|unit| {
                    unit.text.contains("COURT STREET") || unit.text.contains("ARBITRATION")
                })
                .unwrap();
            assert!(forum >= FORUM_HEAD_UNITS, "{forum}");
            assert_ne!(
                outcome.proposal.party_relation,
                PartyRelation::Between,
                "{address}: {:?}",
                outcome.proposal
            );
        }
        assert!(heads_a_court("UNITED STATES DISTRICT COURT"));
        assert!(heads_a_court("COURT OF APPEAL FOR ONTARIO"));
        assert!(heads_a_court("SUPREME COURT, STATE OF NEW YORK"));
        assert!(!heads_a_court("15 COURT STREET"));
        assert!(!heads_a_court("COURT STREET"));
        assert!(!heads_a_court("SECTION 14. ARBITRATION"));
    }

    /// A reader takes a scanned caption's column of labels out of line:
    /// "Plaintiff," comes after "-against-" and the other side's first
    /// names, while "Defendants." keeps its place. "-against-" still divides
    /// the sides: the notice is between them, and a reply that kept one
    /// side takes the other from the caption. A notice whose body is
    /// "against" someone and speaks of "the Defendant" under a caption
    /// that sets no sides out has no label out of place, and no side to
    /// add.
    #[test]
    fn a_caption_with_a_label_out_of_its_place_is_divided_by_its_versus() {
        const DISMISSAL: &str = "UNITED STATES DISTRICT COURT SOUTHERN DISTRICT OF NEW YORK\n\n\
LUMO AS,\n\n\
-against-\n\n\
THE WALTER DABNEY COMPANY, PIXEL\n\n\
x\n\n:\n\n:\n\n\
Plaintiff, Case No. 09 Civ 7689 (LTS)\n\n\
NOTICE OF DISMISSAL\n\n\
ANIMATION STUDIOS, JOHN DOES, JANE :\n\n\
DOES and XYZ COMPANIES\n\n\
Defendants.\n\n\
Pursuant to Rule 41(a) of the Federal Rules of Civil Procedure, plaintiff hereby dismisses\n\n\
this action with prejudice and without costs or disbursements to any party.\n\n\
Dated: New York, New York\n\n\
November 2, 2009\n\n\
Respectfully submitted,\n\n\
CARROW, LIEBERT & LATHAM, P.C.\n\n\
Attorneys for Plaintiff";
        let dismissal = |parties: Vec<(&'static str, PartyRole)>| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some("NOTICE OF DISMISSAL".into()),
                type_evidence: vec![id_of(index, "NOTICE OF DISMISSAL")],
                document_date: Some("2009-11-02".into()),
                date_evidence: vec![id_of(index, "November 2, 2009")],
                parties: parties
                    .iter()
                    .map(|(name, role)| party(name, Some(*role), &[id_of(index, name)]))
                    .collect(),
                ..ModelFacts::default()
            }
        };
        // Both sides named, with roles the document supports nowhere.
        let (outcome, _) = facts_for(
            DISMISSAL,
            dismissal(vec![
                ("LUMO AS", PartyRole::Client),
                ("THE WALTER DABNEY COMPANY", PartyRole::Contractor),
            ]),
        );
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::Between,
            "{:?}",
            outcome.proposal
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["LUMO AS", "THE WALTER DABNEY COMPANY"]
        );
        assert!(
            !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
            "{:?}",
            outcome.reasons
        );
        // One side named: the other is the caption's.
        let (outcome, _) = facts_for(DISMISSAL, dismissal(vec![("LUMO AS", PartyRole::Issuer)]));
        assert_eq!(
            outcome.proposal.party_relation,
            PartyRelation::Between,
            "{:?}",
            outcome.proposal
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["LUMO AS", "THE WALTER DABNEY COMPANY"]
        );
        // A notice filed in a consolidated matter, its caption an "In re"
        // with no sides: the "against" and "the Defendant" are its body's.
        let (outcome, _) = facts_for(
            "Case 1:99-cv-00371-GMS Document 377 Filed 09/16/2008 Page 1 of 3\n\n\
UNITED STATES DISTRICT COURT DISTRICT OF DELAWARE\n\n\
| IN RE: ALDER GOLF, INC., | con con con con | CIVIL ACTION NO. 99-371-GMS |\n\
| SECURITIES LITIGATION | | (CONSOLIDATED) |\n\n\
NOTICE OF BANKRUPTCY\n\n\
PLEASE TAKE NOTICE that, on September 15, 2008 (the \"Commencement Date\"), Lehigh Brothers \
Holdings Inc. (the \"Defendant\") commenced a case under chapter 11 of title 11 of the United \
States Code.\n\n\
PLEASE BE ADVISED that nothing herein stays the claims pending against Marlow Underwriters, \
Inc., which the Defendant does not own.\n\n\
Dated: September 16, 2008",
            |index| ModelFacts {
                document_type: Some("NOTICE OF BANKRUPTCY".into()),
                type_evidence: vec![id_of(index, "NOTICE OF BANKRUPTCY")],
                document_date: Some("2008-09-16".into()),
                date_evidence: vec![id_of(index, "Dated")],
                parties: vec![party(
                    "Lehigh Brothers Holdings Inc.",
                    Some(PartyRole::Borrower),
                    &[id_of(index, "Lehigh")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Lehigh Brothers Holdings Inc."],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
    }

    /// A bankruptcy caption sets no two sides against each other: "In re:"
    /// names the debtor, and "Debtors." closes the name. A notice filed in
    /// the case is for the debtor, whatever the reply called it and
    /// whether or not the reply named it: the creditors' committee that
    /// appears, the movants that set a hearing and the creditors the
    /// trustee appoints are not who the case is about. The caption's rule
    /// may be drawn with ")" or "§", and a table's cells may split the
    /// name. An "In re" that names a matter, with no label to close it, is
    /// no debtor's, though the notice under it speaks of "the debtor".
    #[test]
    fn a_notice_in_a_bankruptcy_case_is_for_the_debtor_its_caption_names() {
        let notice = |title: &'static str,
                      dated: &'static str,
                      date: &'static str,
                      parties: Vec<(&'static str, PartyRole)>| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some(title.into()),
                type_evidence: vec![id_of(index, title)],
                document_date: Some(date.into()),
                date_evidence: vec![id_of(index, dated)],
                parties: parties
                    .iter()
                    .map(|(name, role)| party(name, Some(*role), &[id_of(index, name)]))
                    .collect(),
                ..ModelFacts::default()
            }
        };
        // The reply named the debtor, as the committee's "employer".
        let (outcome, _) = facts_for(
            "IN THE UNITED STATES BANKRUPTCY COURT FOR THE DISTRICT OF DELAWARE\n\n\
In re: Chapter 11\n\n\
| CALDER MILLS, INC., et al., | Case No. 06-11144 (CSS) |\n\
| Debtors. | (Jointly Administered) |\n\n\
NOTICE OF APPEARANCE AND DEMAND FOR NOTICES AND PAPERS\n\n\
Please take notice that the Official Committee of Unsecured Creditors (the \"Committee\")\n\n\
of Calder Mills, Inc., et al., hereby appears in the above-captioned cases by and through its\n\n\
proposed counsel, Strand & Lowell LLP and Cozen Hartley; such counsel hereby\n\n\
enter their appearance pursuant to 11 U.S.C. § 1109(b) and Bankruptcy Rule 9010(b).\n\n\
Dated: October 25, 2006",
            notice(
                "NOTICE OF APPEARANCE AND DEMAND FOR NOTICES AND PAPERS",
                "Dated",
                "2006-10-25",
                vec![
                    (
                        "Official Committee of Unsecured Creditors",
                        PartyRole::Client,
                    ),
                    ("Calder Mills, Inc.", PartyRole::Employer),
                    ("Strand & Lowell LLP", PartyRole::Contractor),
                ],
            ),
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Calder Mills, Inc."],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        assert!(
            !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
            "{:?}",
            outcome.reasons
        );
        // The reply named only the movants; counsel's block stands over
        // the court's heading, and the rule is drawn with ")".
        let (outcome, _) = facts_for(
            "HEARING DATE: OCTOBER 24, 2011 AT 11:00 A.M. (PREVAILING EASTERN TIME)\n\
OBJECTION DEADLINE: OCTOBER 20, 2011 AT 4:00 P.M. (PREVAILING EASTERN TIME)\n\n\
LATIMER & WATTS\n\n\
885 Third Avenue New York, NY 10022-4834 (212) 555-0120 David S. Heller Adam Goldberg\n\n\
-and\n\n\
233 S. Wacker Drive, Suite 5800 Chicago, IL 60606 (312) 555-0177 Douglas Bacon\n\n\
ATTORNEYS FOR HARROW RECOVERY IIA GP, L.P., HARROW RECOVERY III GP, L.P., and HARROW RIIA, LLC\n\n\
UNITED STATES BANKRUPTCY COURT FOR THE SOUTHERN DISTRICT OF NEW YORK\n\n\
In re: ) Chapter 11 )\n\n\
HARROW GROUP, INC., et al. ) Case No. 10-14653 (AJG)\n\n\
)\n\n\
Jointly Administered\n\n\
Debtors. ) ) )\n\n\
NOTICE OF HEARING\n\n\
PLEASE TAKE NOTICE that on September 30, 2011, Harrow Recovery IIA GP, L.P.\n\n\
(\"GPIIA\") and Harrow Recovery III GP, L.P. (\"GPIII\") filed the Motion to Deem Late Claims \
Timely Filed.\n\n\
Dated: October 5, 2011",
            notice(
                "NOTICE OF HEARING",
                "Dated",
                "2011-10-05",
                vec![
                    ("Harrow Recovery IIA GP, L.P.", PartyRole::Client),
                    ("Harrow Recovery III GP, L.P.", PartyRole::Client),
                ],
            ),
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["HARROW GROUP, INC."],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        // The debtor's name split across a table's cells, the rule drawn
        // with "§", the label in the singular; the reply named the
        // creditors the notice appoints.
        let (outcome, _) = facts_for(
            "Case 05-80340 Document 26 Filed in TXSB on 02/15/05 Page 1 of 2\n\n\
UNITED STATES BANKRUPTCY COURT SOUTHERN DISTRICT OF TEXAS GALVESTON DIVISION\n\n\
IN RE: § CASE NO:\n\n\
§\n\n\
| SLATE OUTLET, | INC. | § § | 05-80340-G3-11 (Chapter 11) |\n\
| DEBTOR | | § | |\n\n\
NOTICE OF APPOINTMENT OF OFFICIAL COMMITTEE OF UNSECURED CREDITORS\n\n\
TO THE HONORABLE LETITIA Z. CLARK\n\n\
UNITED STATES BANKRUPTCY JUDGE:\n\n\
| COMES NOW | the United States | Trustee (“UST”), who | through the undersigned | attorney, |\n\
| pursuant | to Section 1102(a) and (b) | of the | Bankruptcy Code, hereby appoints the | following eligible |\n\n\
creditors to the official committee of unsecured creditors in the above captioned case:\n\n\
QRP Inc.\n\n\
Attn: Marc Applegate\n\n\
Readers Wholesale Distributors, Inc.\n\n\
Attn: James Constable\n\n\
Dated: February 15, 2005",
            notice(
                "NOTICE OF APPOINTMENT OF OFFICIAL COMMITTEE OF UNSECURED CREDITORS",
                "Dated",
                "2005-02-15",
                vec![
                    ("QRP Inc.", PartyRole::Contractor),
                    (
                        "Readers Wholesale Distributors, Inc.",
                        PartyRole::Contractor,
                    ),
                ],
            ),
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["SLATE OUTLET, INC."],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
        // A consolidated matter's "In re", under a district court's
        // heading: no label closes it, and the "debtor" of the sentence
        // under it is no one the caption names.
        let (outcome, _) = facts_for(
            "Case 1:99-cv-00371-GMS Document 377 Filed 09/16/2008 Page 1 of 3\n\n\
UNITED STATES DISTRICT COURT DISTRICT OF DELAWARE\n\n\
| IN RE: ALDER GOLF, INC., | con con con con | CIVIL ACTION NO. 99-371-GMS |\n\
| SECURITIES LITIGATION | | (CONSOLIDATED) |\n\n\
NOTICE OF BANKRUPTCY\n\n\
PLEASE TAKE NOTICE that section 362 of the Bankruptcy Code stays any further action against \
the debtor.\n\n\
Lehigh Brothers Holdings Inc. (the \"Defendant\") commenced a case under chapter 11 of title 11 \
of the United States Code on September 15, 2008.\n\n\
Dated: September 16, 2008",
            notice(
                "NOTICE OF BANKRUPTCY",
                "Dated",
                "2008-09-16",
                vec![("Lehigh Brothers Holdings Inc.", PartyRole::Borrower)],
            ),
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Lehigh Brothers Holdings Inc."],
            "{:?}",
            outcome.proposal
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::For);
    }

    /// A body's name ends on its noun ("Committee", "Board") or puts it
    /// before what completes it ("Union of Biochemistry", "Office for
    /// National Statistics"); a post is headed by no such noun.
    #[test]
    fn a_body_named_by_its_noun_is_an_organization() {
        for name in [
            "Home Affairs Select Committee",
            "Horizon Compensation Advisory Board",
            "International Union of Biochemistry",
            "Office of Fair Trading",
            "Office for National Statistics",
            "Committee on Standards in Public Life",
            "Department of Energy",
            "Northstar Lantern Works LLC",
            "Office of the Chief Counsel, Food and Drug Administration",
        ] {
            assert!(is_organization_name(name), "{name}");
        }
        for name in [
            "Secretary of State for Business",
            "Chair of the Board",
            "Lord Lang of Monkton",
            "Rashmi Panigrahi",
            // A post written before the body it is held at.
            "Director, Office of Prescription Drug Promotion",
            "Member, Board of Directors",
        ] {
            assert!(!is_organization_name(name), "{name}");
        }
    }

    /// A committee or a union the reply named is a party, never the staff
    /// of one: the post a person holds there ("Chair, Home Affairs Select
    /// Committee") does not make the committee a signatory.
    #[test]
    fn a_body_the_reply_named_is_no_one_s_staff() {
        let (outcome, _) = facts_for(
            "Home Secretary\n2 Marsham Street\nLondon SW1P 4DF\n\nRt Hon Yvette Cooper MP\n\
             Chair, Home Affairs Select Committee\nHouse of Commons\nLondon SW1A 0AA\n\n\
             28 April 2020\n\nDear Chair,\n\nPlease find below a further update on the work \
             of my department, in line with the commitments made to your Committee.\n\n\
             Yours sincerely,\n\nHome Secretary",
            |index| ModelFacts {
                document_type: Some("letter".into()),
                document_date: Some("2020-04-28".into()),
                date_evidence: vec![id_of(index, "28 April 2020")],
                parties: vec![
                    party(
                        "Home Secretary",
                        Some(PartyRole::Sender),
                        &[id_of(index, "Marsham")],
                    ),
                    party(
                        "Home Affairs Select Committee",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "Chair, Home Affairs")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        let facts = outcome.facts.expect("facts");
        let committee = facts
            .parties
            .iter()
            .find(|party| party.name == "Home Affairs Select Committee")
            .expect("the committee is kept");
        assert!(!committee.signatory, "{:?}", facts.parties);
        assert_ne!(
            committee.role,
            Some(PartyRole::Other),
            "{:?}",
            facts.parties
        );
    }

    /// A role word right after a name names its side - ", Tenant" - unless
    /// it labels the party's number: ", company number 3899612", "Employer
    /// ID Number NN-NNNNNNN".
    #[test]
    fn a_role_word_labeling_a_number_defines_no_role() {
        let employer = role_words(PartyRole::Employer).1;
        assert!(defines_role(", tenant, agrees", &["tenant"]));
        assert!(defines_role(" (the \"company\") agrees", employer));
        assert!(!defines_role(
            ", company number 3899612 and good electricity gas limited",
            employer
        ));
        assert!(!defines_role(" employer id numbernn-nnnnnnn", employer));
        assert!(!defines_role(" company reg. no. 0412", employer));
        assert!(!defines_role(", company no. 3899612, agrees", employer));
        // "No." that counts a party among its side's labels no number.
        assert!(defines_role(", tenant no. 2, agrees", &["tenant"]));
        assert!(defines_role(", tenant no 1 and", &["tenant"]));
    }

    /// An organization the reply named as the recipient, which the inside
    /// address names under the person written to, is the letter's
    /// addressee, though the agency's letterhead supports its sender.
    #[test]
    fn an_organization_in_the_inside_address_is_the_addressee() {
        let (outcome, _) = facts_for(
            "U.S. DEPARTMENT OF JUSTICE\nAntitrust Division\n950 Pennsylvania Avenue, N.W.\n\
             Washington, D.C. 20530\n\nApril 16, 2020\n\nRobert L. Sacks\nKane Kessler, P.C.\n\
             666 Third Avenue\nNew York, NY 10017\n\nDear Mr. Sacks:\n\nThis letter responds \
             to your request for a business review letter.\n\nSincerely,\n\nMakan Delrahim",
            |index| ModelFacts {
                document_type: Some("letter".into()),
                document_date: Some("2020-04-16".into()),
                date_evidence: vec![id_of(index, "April 16, 2020")],
                parties: vec![
                    party(
                        "Kane Kessler, P.C.",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "Kane Kessler")],
                    ),
                    party(
                        "U.S. Department of Justice",
                        Some(PartyRole::Sender),
                        &[id_of(index, "DEPARTMENT OF JUSTICE")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.parties, vec!["Kane Kessler, P.C."]);
        assert_eq!(outcome.proposal.party_relation, PartyRelation::To);
        // A "Re:" line names the matter, not who the letter is to.
        let (outcome, _) = facts_for(
            "DLA Piper LLP (US)\n1251 Avenue of the Americas\nNew York, New York 10020\n\n\
             October 7, 2024\n\nThe Honorable Valerie Caproni\nUnited States District Court\n\
             40 Foley Square\nRe: Nike, Inc. v. StockX LLC, No. 22 CV 00983\n\nDear Judge \
             Caproni:\n\nWe represent Nike in connection with the above-referenced matter.\n\n\
             Respectfully submitted,",
            |index| ModelFacts {
                document_type: Some("letter".into()),
                document_date: Some("2024-10-07".into()),
                date_evidence: vec![id_of(index, "October 7, 2024")],
                parties: vec![
                    party(
                        "Nike, Inc.",
                        Some(PartyRole::Client),
                        &[id_of(index, "Re: Nike")],
                    ),
                    party(
                        "DLA Piper LLP (US)",
                        Some(PartyRole::Sender),
                        &[id_of(index, "DLA Piper")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_ne!(
            (
                outcome.proposal.party_relation,
                outcome.proposal.parties.clone()
            ),
            (PartyRelation::To, vec!["Nike, Inc.".to_owned()])
        );
    }

    /// A reply that read a stamp's "SEP 1 8 2015" as September 1 cited a
    /// line that states September 18 only: the stamp's date stands in.
    #[test]
    fn a_stamp_date_misread_by_its_first_figure_is_repaired() {
        let notice = "UNITED STATES ENVIRONMENTAL PROTECTION AGENCY\n\nSEP 1 8 2015 OFFICE OF\n\
                      ENFORCEMENT\n\nVolkswagen AG\n\nRe: Notice of Violation\n\n\
                      Dear Mr. Geanacopoulos:\n\nThe EPA has investigated Volkswagen AG.";
        for (replied, expected) in [
            ("2015-09-01", Some("2015-09-18")),
            ("2015-09-18", Some("2015-09-18")),
            ("2015-09-08", None),
            ("2015-10-01", None),
        ] {
            let (outcome, _) = facts_for(notice, |index| ModelFacts {
                document_type: Some("Notice of Violation".into()),
                document_date: Some(replied.into()),
                date_evidence: vec![id_of(index, "SEP 1 8")],
                parties: vec![party(
                    "Volkswagen AG",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Volkswagen AG")],
                )],
                ..ModelFacts::default()
            });
            assert_eq!(
                outcome.proposal.document_date.as_deref(),
                expected,
                "{replied}"
            );
            assert!(
                outcome.reasons.contains(&ReviewReason::DateUnsupported),
                "{replied}"
            );
        }
        // A receipt's stamp dates the copy's arrival: it repairs no date.
        for (stamp, repaired) in [
            ("SEP 1 8 2015", true),
            ("RECEIVED\nSEP 1 8 2015", false),
            ("RECEIVED SEP 1 8 2015", false),
        ] {
            let letter = format!(
                "ARP INDUSTRIES\n\n{stamp}\n\nMr. Alfred Puccinelli\n\nDear Mr. Puccinelli:\n\n\
                 This is in response to your letter."
            );
            let (outcome, _) = facts_for(&letter, |index| ModelFacts {
                document_type: Some("letter".into()),
                document_date: Some("2015-09-01".into()),
                date_evidence: vec![id_of(index, "SEP 1 8")],
                ..ModelFacts::default()
            });
            assert_eq!(
                outcome.proposal.document_date.as_deref() == Some("2015-09-18"),
                repaired,
                "{stamp}"
            );
        }
    }

    /// A date read only loosely meets the guards a date written out meets:
    /// a due date is a deadline and another agreement's date is that
    /// agreement's, however the scan left them, and a receipt's stamp dates
    /// no document.
    #[test]
    fn a_date_read_loosely_meets_the_guards_of_one_written_out() {
        let invoice = |issued: &str, due: &str| {
            format!(
                "NORTHWIND SUPPLY CO.\n\nINVOICE\n\nInvoice No. 4471\nInvoice Date: {issued}\n\n\
                 Bill To: Contoso Ltd.\n\nWidgets 10 x $160.00 $1,600.00\n\n\
                 Payment due by {due}"
            )
        };
        // The issue date written out replaces the due date; one written
        // loosely is never offered, and the due date is withheld.
        for (issued, due, expected) in [
            ("June 2, 2017", "July 2, 2017", Some("2017-06-02")),
            ("June 2, 2017", "July 2,2017", Some("2017-06-02")),
            ("June 2,2017", "July 2,2017", None),
            ("02-Jun-17", "02-Jul-17", None),
        ] {
            let (outcome, _) = facts_for(&invoice(issued, due), |index| ModelFacts {
                document_type: Some("Invoice".into()),
                document_date: Some("2017-07-02".into()),
                date_evidence: vec![id_of(index, "Payment due")],
                ..ModelFacts::default()
            });
            assert_eq!(
                outcome.proposal.document_date.as_deref(),
                expected,
                "{issued} / {due}"
            );
            assert!(
                outcome.reasons.contains(&ReviewReason::DateIsDeadline),
                "{issued} / {due}: {:?}",
                outcome.reasons
            );
        }
        for dated in ["November 22, 1923", "November 22,1923"] {
            let letter = format!(
                "Dear Mr. Lee:\n\nWe write about the agreement dated {dated} between your \
                 company and ours.\n\nSincerely,\n\nRuth Abbott"
            );
            let (outcome, _) = facts_for(&letter, |index| ModelFacts {
                document_type: Some("letter".into()),
                document_date: Some("1923-11-22".into()),
                date_evidence: vec![id_of(index, "agreement dated")],
                ..ModelFacts::default()
            });
            assert_eq!(outcome.proposal.document_date, None, "{dated}");
            assert!(
                outcome.reasons.contains(&ReviewReason::DateUnsupported),
                "{dated}"
            );
        }
        // A receipt's stamp is not read loosely, as it repairs nothing,
        // whatever loose form its date takes.
        for (stamp, needle, date, read) in [
            ("SEP 1 8 2015", "SEP 1 8", "2015-09-18", true),
            ("RECEIVED\nSEP 1 8 2015", "SEP 1 8", "2015-09-18", false),
            ("RECEIVED SEP 1 8 2015", "SEP 1 8", "2015-09-18", false),
            ("FILED SEP 1 8 2015", "SEP 1 8", "2015-09-18", false),
            ("November 22,1923", "November 22", "1923-11-22", true),
            (
                "RECEIVED November 22,1923",
                "November 22",
                "1923-11-22",
                false,
            ),
            (
                "RECEIVED\nNovember 22,1923",
                "November 22",
                "1923-11-22",
                false,
            ),
            ("FILED November 22,1923", "November 22", "1923-11-22", false),
            ("November\n22,1923", "November", "1923-11-22", true),
            (
                "RECEIVED November\n22,1923",
                "November",
                "1923-11-22",
                false,
            ),
            (
                "RECEIVED\nNovember\n22,1923",
                "November",
                "1923-11-22",
                false,
            ),
            ("RECEIVED\nSEP 1 8\n2015", "SEP 1 8", "2015-09-18", false),
            // A dateline of its own two lines under a receipt's label is
            // no part of the receipt's date.
            (
                "Received in the Office of the Clerk on\nJAN 3 1924 by the deputy clerk\n\
                 November 22,1923, the date of the company's letter to the court",
                "November 22,1923, the date",
                "1923-11-22",
                true,
            ),
        ] {
            let letter = format!(
                "ARP INDUSTRIES\n\n{stamp}\n\nMr. Alfred Puccinelli\n\nDear Mr. Puccinelli:\n\n\
                 This is in response to your letter."
            );
            let (outcome, _) = facts_for(&letter, |index| ModelFacts {
                document_type: Some("letter".into()),
                document_date: Some(date.into()),
                date_evidence: vec![id_of(index, needle)],
                ..ModelFacts::default()
            });
            assert_eq!(
                outcome.proposal.document_date.as_deref() == Some(date),
                read,
                "{stamp}"
            );
            assert!(
                outcome.reasons.contains(&ReviewReason::DateUnsupported),
                "{stamp}"
            );
        }
    }

    /// A court document's caption names the sides of its case, and a
    /// British letter's address block names who it is to: neither is the
    /// letterhead of the party a line in it names.
    #[test]
    fn a_caption_or_an_address_block_is_no_letterhead() {
        let (outcome, _) = facts_for(
            "UNITED STATES DISTRICT COURT\nSOUTHERN DISTRICT OF NEW YORK\n\nLUXO AS,\n\n\
             -against-\n\nTHE WALT DISNEY COMPANY\n\nPlaintiff,\n\nNOTICE OF DISMISSAL\n\n\
             Defendants.\n\nPlaintiff hereby dismisses this action.\n\nDated: November 2, 2009",
            |index| ModelFacts {
                document_type: Some("NOTICE OF DISMISSAL".into()),
                document_date: Some("2009-11-02".into()),
                date_evidence: vec![id_of(index, "November 2, 2009")],
                parties: vec![party(
                    "LUXO AS",
                    Some(PartyRole::Client),
                    &[id_of(index, "LUXO")],
                )],
                ..ModelFacts::default()
            },
        );
        let facts = outcome.facts.expect("facts");
        assert_ne!(
            facts.parties[0].document_role,
            Some(PartyRole::Issuer),
            "{:?}",
            facts.parties
        );
        let (outcome, _) = facts_for(
            "From the Chief Executive Office Of Fair Trading\n\nThe Rt Hon George Osborne MP\n\
             Chancellor of the Exchequer\nHM Treasury\n1 Horse Guards Road\n\n11 September 2013\n\
             Dear Chancellor\n\nI am writing to provide you with advice.\n\nYours sincerely",
            |index| ModelFacts {
                document_type: Some("letter".into()),
                document_date: Some("2013-09-11".into()),
                date_evidence: vec![id_of(index, "11 September 2013")],
                parties: vec![party(
                    "HM Treasury",
                    Some(PartyRole::Employer),
                    &[id_of(index, "HM Treasury")],
                )],
                ..ModelFacts::default()
            },
        );
        let facts = outcome.facts.expect("facts");
        assert_ne!(
            facts.parties[0].document_role,
            Some(PartyRole::Issuer),
            "{:?}",
            facts.parties
        );
        // A law firm's letter to a court names the court in its inside
        // address and the case on its "Re:" line: its letterhead, above
        // both, is still its sender's.
        let (outcome, _) = facts_for(
            "DLA Piper LLP (US)\n1251 Avenue of the Americas\nNew York, New York 10020\n\n\
             October 7, 2024\n\nThe Honorable Valerie Caproni\nUnited States District Court\n\
             40 Foley Square\nRe: Nike, Inc. v. StockX LLC, No. 22 CV 00983\n\nDear Judge \
             Caproni:\n\nWe represent Nike in connection with the above-referenced matter and \
             write to request a conference against the defendant's objection.\n\n\
             Respectfully submitted,\n\nJane Roe",
            |index| ModelFacts {
                document_type: Some("letter".into()),
                type_evidence: vec![id_of(index, "Dear Judge")],
                document_date: Some("2024-10-07".into()),
                date_evidence: vec![id_of(index, "October 7, 2024")],
                parties: vec![party(
                    "DLA Piper LLP (US)",
                    Some(PartyRole::Sender),
                    &[id_of(index, "DLA Piper")],
                )],
                ..ModelFacts::default()
            },
        );
        assert!(
            !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
            "{:?}",
            outcome.reasons
        );
        let facts = outcome.facts.expect("facts");
        assert_eq!(facts.parties[0].role, Some(PartyRole::Sender));
        assert_eq!(facts.parties[0].role_support, Support::Cited);
        // A court's own letter sets its case out with no "Re:": the court's
        // letterhead stands over the letter's dateline and heads no
        // caption, so it is still the sender's.
        let court_letter = "UNITED STATES DISTRICT COURT\nSOUTHERN DISTRICT OF NEW YORK\n\
             Chambers of Judge Jane Roe\n\nMarch 3, 2024\n\nJohn Smith, Esq.\nSmith & Co LLP\n\
             1 Main Street\nNew York, NY 10001\n\nSmith v. Jones, No. 22-cv-123\n\n\
             Dear Counsel:\n\nThe conference set for March 10 is adjourned to April 1, \
             2024.\n\nSO ORDERED.\n\nJane Roe";
        let index = index_of(court_letter);
        let context = whole(&index);
        let scope = ValidationScope::new(&index, &context, &[]);
        assert_eq!(LetterLayout::of(&scope).caption, None);
        let (outcome, _) = facts_for(court_letter, |index| ModelFacts {
            document_type: Some("letter".into()),
            type_evidence: vec![id_of(index, "Dear Counsel")],
            document_date: Some("2024-03-03".into()),
            date_evidence: vec![id_of(index, "March 3, 2024")],
            parties: vec![party(
                "United States District Court",
                Some(PartyRole::Sender),
                &[id_of(index, "UNITED STATES DISTRICT COURT")],
            )],
            ..ModelFacts::default()
        });
        assert!(
            !outcome.reasons.contains(&ReviewReason::PartyUnsupported),
            "{:?}",
            outcome.reasons
        );
        let facts = outcome.facts.expect("facts");
        assert_eq!(facts.parties[0].role_support, Support::Cited);
        // A caption set out above a letter's dateline is still a caption.
        let index = index_of(
            "UNITED STATES DISTRICT COURT\nSOUTHERN DISTRICT OF NEW YORK\n\n\
             LUXO AS,\nPlaintiff,\n-against-\nTHE WALT DISNEY COMPANY,\nDefendant.\n\n\
             March 3, 2024\n\nDear Counsel:\n\nThe conference is adjourned.",
        );
        let context = whole(&index);
        let scope = ValidationScope::new(&index, &context, &[]);
        let layout = LetterLayout::of(&scope);
        let luxo = index
            .units()
            .iter()
            .find(|unit| unit.text.contains("LUXO"))
            .expect("unit");
        assert!(layout.caption.is_some() && layout.in_caption(&index.units()[0]));
        assert!(layout.in_caption(luxo), "{:?}", layout.caption);
    }

    /// A reply that copied a title line with the side filing it, or with
    /// the sentence the title opens, is typed by the title alone.
    #[test]
    fn a_reply_past_its_title_is_typed_by_the_title() {
        assert!(reads_past_title(
            "PLAINTIFF'S NOTICE OF VOLUNTARY DISMISSAL",
            &words("PLAINTIFF'S NOTICE OF VOLUNTARY DISMISSAL"),
            "NOTICE OF VOLUNTARY DISMISSAL"
        ));
        assert!(reads_past_title(
            "NOTICE IS HEREBY GIVEN",
            &words("NOTICE IS HEREBY GIVEN"),
            "NOTICE"
        ));
        assert_eq!(head_noun(&words("NOTICE IS HEREBY GIVEN")), Some("notice"));
        assert!(!reads_past_title(
            "Amended and Restated Lease Agreement",
            &words("Amended and Restated Lease Agreement"),
            "Lease Agreement"
        ));
        assert!(!reads_past_title(
            "Notice of Default",
            &words("Notice of Default"),
            "Notice of Default"
        ));
        assert!(reads_past_title(
            "Notice IS Hereby Given for MR. William Dew",
            &words("Notice IS Hereby Given for MR. William Dew"),
            "Notice"
        ));
        assert!(reads_past_title(
            "NOTICE HEREBY GIVEN",
            &words("NOTICE HEREBY GIVEN"),
            "NOTICE"
        ));
        // A body's sentence that opens on a kind of document is no title:
        // its first word is not the type.
        assert!(!reads_past_title(
            "Receipt is acknowledged of the Letter of Notification of PPL Electric Utilities",
            &words(
                "Receipt is acknowledged of the Letter of Notification of PPL Electric Utilities"
            ),
            "Receipt"
        ));
        assert!(!reads_past_title(
            "Invoice is attached for services rendered",
            &words("Invoice is attached for services rendered"),
            "Invoice"
        ));
        assert!(!reads_past_title(
            "Notice is",
            &words("Notice is"),
            "Notice"
        ));
        let (outcome, _) = facts_for(
            "IN THE UNITED STATES DISTRICT COURT\n\nPA ADVISORS, LLC,\nPlaintiff,\nv.\n\
             GOOGLE INC., et al.,\nDefendants.\n\n\
             PLAINTIFF\u{2019}S NOTICE OF VOLUNTARY DISMISSAL OF WPP GROUP PLC\n\n\
             Plaintiff hereby dismisses WPP GROUP PLC.\n\nDated: January 24, 2008",
            |index| ModelFacts {
                document_type: Some(
                    "PLAINTIFF'S NOTICE OF VOLUNTARY DISMISSAL OF WPP GROUP PLC".into(),
                ),
                type_evidence: vec![id_of(index, "VOLUNTARY DISMISSAL")],
                document_date: Some("2008-01-24".into()),
                date_evidence: vec![id_of(index, "January 24, 2008")],
                parties: vec![
                    party(
                        "PA ADVISORS, LLC",
                        Some(PartyRole::Client),
                        &[id_of(index, "PA ADVISORS")],
                    ),
                    party(
                        "WPP GROUP PLC",
                        Some(PartyRole::Client),
                        &[id_of(index, "VOLUNTARY DISMISSAL")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome
                .proposal
                .document_type
                .as_deref()
                .map(str::to_lowercase)
                .as_deref(),
            Some("notice of voluntary dismissal")
        );
        // A reply that copied a body's sentence opening on a kind of
        // document is not typed by its first word.
        for (sentence, cited) in [
            (
                "Invoice is attached for services rendered",
                "Invoice is attached",
            ),
            (
                "Receipt is acknowledged of the Letter of Notification of Acme Utilities",
                "Receipt is acknowledged",
            ),
        ] {
            let (outcome, _) = facts_for(
                &format!(
                    "Acme Utilities Corporation\n12 High Street\n\n3 March 2024\n\n\
                     {sentence} in February.\n\nAcme Utilities Corporation"
                ),
                |index| ModelFacts {
                    document_type: Some(sentence.into()),
                    type_evidence: vec![id_of(index, cited)],
                    document_date: Some("2024-03-03".into()),
                    date_evidence: vec![id_of(index, "3 March 2024")],
                    parties: vec![party(
                        "Acme Utilities Corporation",
                        Some(PartyRole::Issuer),
                        &[id_of(index, "12 High Street")],
                    )],
                    ..ModelFacts::default()
                },
            );
            let head = sentence.split_whitespace().next().expect("word");
            assert_ne!(
                outcome.proposal.document_type.as_deref(),
                Some(head),
                "{:?}",
                outcome.proposal
            );
        }
    }

    /// A reply that copied a title and ran on past its line - into the
    /// office and the hour a weather statement prints under its title,
    /// into the delivery line under a letter's heading - is typed by the
    /// part the line holds, read by that part's own kind: the reply's last
    /// word is no head noun. A reply whose tail the document does not go on
    /// with copied nothing, and waits for a person.
    #[test]
    fn a_reply_run_past_its_title_line_is_typed_by_the_part_the_line_holds() {
        const STATEMENT: &str = "753\nNOUS43 KHLW 200010\nPNSHLW\n\n\
             Public Information Statement\nNational Weather Service Harlow KS\n\
             710 PM CDT Mon Oct 19 2026\n\n\
             ...TEMPERATURE AND PRECIPITATION REPORTS FROM THE PAST 24 HOURS...\n\n\
             Location                     High   Low   Precip\n\
             Harlow Municipal Airport       71    44     0.00";
        let statement = |index: &EvidenceIndex, kind: &str| ModelFacts {
            document_type: Some(kind.into()),
            type_evidence: vec![id_of(index, "Public Information Statement")],
            document_date: Some("2026-10-19".into()),
            date_evidence: vec![id_of(index, "Oct 19 2026")],
            parties: vec![party(
                "National Weather Service Harlow KS",
                Some(PartyRole::Issuer),
                &[id_of(index, "National Weather Service")],
            )],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(STATEMENT, |index| {
            statement(
                index,
                "Public Information Statement National Weather Service Harlow KS 710 PM CDT Mon",
            )
        });
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Public Information Statement"),
            "{:?}",
            outcome.reasons
        );
        for reason in [ReviewReason::TypeUnsupported, ReviewReason::TypeInferred] {
            assert!(!outcome.reasons.contains(&reason), "{:?}", outcome.reasons);
        }
        // Words the document does not go on with are no copy of it.
        let (outcome, _) = facts_for(STATEMENT, |index| {
            statement(
                index,
                "Public Information Statement Harlow Rail Users Group Summer Timetable",
            )
        });
        assert!(
            outcome.reasons.contains(&ReviewReason::TypeUnsupported),
            "{:?}",
            outcome.proposal
        );
        // A letter's heading and the delivery line set under it.
        let (outcome, _) = facts_for(
            "Pipeline and Hazardous Materials\nSafety Administration\n\nWARNING LETTER\n\n\
             OVERNIGHT EXPRESS DELIVERY\n\nMarch 2, 2018\n\nMr. Jerome Colfax\n\
             Refinery Manager\nTollard Refining Company, LLC\n1819 Woodville Road\n\n\
             Dear Mr. Colfax:\n\n\
             A representative of the Administration inspected the pipeline facilities.",
            |index| ModelFacts {
                document_type: Some("WARNING LETTER OVERNIGHT EXPRESS DELIVERY".into()),
                type_evidence: vec![id_of(index, "WARNING LETTER")],
                document_date: Some("2018-03-02".into()),
                date_evidence: vec![id_of(index, "March 2, 2018")],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Warning Letter"),
            "{:?}",
            outcome.reasons
        );
        for reason in [ReviewReason::TypeUnsupported, ReviewReason::TypeInferred] {
            assert!(!outcome.reasons.contains(&reason), "{:?}", outcome.reasons);
        }
    }

    /// A response, a citation with its notification of penalty and a
    /// circular are kinds of document: a reply that names one is typed by
    /// the title line that states it, where "Document" or another page's
    /// heading stood before. A sentence of the body that talks about a
    /// response still titles nothing.
    #[test]
    fn a_response_a_citation_and_a_circular_are_typed_by_their_titles() {
        let typed = |outcome: &ValidationOutcome| {
            outcome
                .proposal
                .document_type
                .as_deref()
                .map(str::to_lowercase)
        };
        // An agency's answer to a request, titled in capitals over its
        // reference.
        let (outcome, _) = facts_for(
            "23 August 2024\n\ns9(2)(a)\n\ns9(2)(a)\nM\u{101}l\u{14d} e lelei\n\n\
             RESPONSE TO AN OFFICIAL INFORMATION ACT REQUEST (REF: DOIA007-2024/25)\n\n\
             On 26 July 2024, you contacted the Ministry for Island Peoples (the Ministry) \
             requesting under\nthe Official Information Act 1982 (OIA), information of all job \
             titles of staff members.",
            |index| ModelFacts {
                document_type: Some(
                    "RESPONSE TO AN OFFICIAL INFORMATION ACT REQUEST (REF: DOIA007-2024/25)".into(),
                ),
                type_evidence: vec![id_of(index, "RESPONSE TO")],
                document_date: Some("2024-08-23".into()),
                date_evidence: vec![id_of(index, "23 August 2024")],
                parties: vec![party(
                    "Ministry for Island Peoples",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "On 26 July")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            typed(&outcome).as_deref(),
            Some("response to an official information act request"),
            "{:?}",
            outcome.proposal
        );
        assert!(
            !outcome.reasons.contains(&ReviewReason::TypeUnsupported),
            "{:?}",
            outcome.reasons
        );
        // A safety agency's citation, its "To:" field run into the next
        // column's label: typed by its title, and to no "Inspection
        // Number".
        let (outcome, _) = facts_for(
            "U.S. Department of Works\n\nWorkplace Safety Administration\n1141 Harbour Drive\n\
             Suite 1006\nMobile, AL 36609\n\nCitation and Notification of Penalty\n\n\
             To: Inspection Number: 1695478\n\n\
             | Rowan Brick Company, Inc. | Inspection Date(s): 08/30/2023 | - | 01/02/2024 |\n\
             | and its successors | Issuance Date: | 02/20/2024 |\n\n\
             3409 Water Avenue\nSelma, AL 36703\n\n\
             This Citation and Notification of Penalty (this Citation) describes violations of \
             the Workplace Safety Act of 1970.",
            |index| ModelFacts {
                document_type: Some("Citation and Notification of Penalty".into()),
                type_evidence: vec![id_of(index, "Citation and Notification")],
                document_date: Some("2024-02-20".into()),
                date_evidence: vec![id_of(index, "Issuance Date")],
                parties: vec![
                    party(
                        "Rowan Brick Company, Inc.",
                        Some(PartyRole::Employer),
                        &[id_of(index, "Rowan Brick")],
                    ),
                    party(
                        "U.S. Department of Works",
                        Some(PartyRole::Issuer),
                        &[id_of(index, "Department of Works")],
                    ),
                ],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Citation and Notification of Penalty")
        );
        assert!(
            !outcome.reasons.contains(&ReviewReason::TypeUnsupported),
            "{:?}",
            outcome.reasons
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["U.S. Department of Works"],
            "{:?}",
            outcome.proposal
        );
        // A circular numbered on its title line, which the reply copied
        // until its room for a type ran out and cited elsewhere: the kind
        // stands in, for a person to confirm.
        let (outcome, _) = facts_for(
            "Circular 014/2016: Transport Secretary\u{2019}s decision on the findings of the Rail \
             Arbitration Tribunal (RAT) of 20 December 2015 in relation to six recommendations \
             of the Part 2 report of the Harlow Review.\n\n# **Documents**\n\n# **Detail**\n\n\
             - Broad subject: Rail service\n- Issue date: 20 February 2016\n\
             - From: rail group, workforce directorate, pay and efficiency unit\n\
             - Addressed to: chief officers of England and Wales\n\n\
             ### This circular publishes the Transport Secretary\u{2019}s decision on the \
             findings of the Rail Arbitration Tribunal.",
            |index| ModelFacts {
                document_type: Some(
                    "Circular 014/2016: Transport Secretary's decision on the findings of the \
                     Rail Arbit"
                        .into(),
                ),
                type_evidence: vec![id_of(index, "Detail")],
                document_date: Some("2016-02-20".into()),
                date_evidence: vec![id_of(index, "Issue date")],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            typed(&outcome).as_deref(),
            Some("circular"),
            "{:?}",
            outcome.proposal
        );
        assert!(
            outcome.reasons.contains(&ReviewReason::TypeInferred),
            "{:?}",
            outcome.reasons
        );
        // An email whose body talks about a response is still an email.
        let (outcome, _) = facts_for(
            "From: Marsh, Colin (DLR) <colin.marsh@dlr.example>\n\
             To: Petra Lindqvist <petra.lindqvist@borough.example>\n\
             Date: Wed, 21 Mar 2018 11:16:03 +0000\n\
             Subject: RE: 16/4008/F - Quay Trading Estate Re-consultation and response to \
             Harwell Report. (PLA Ref DC 816)\n\nPetra,\n\n\
             The response to the PLA is provided below:\n\n\
             The proposed development is a sufficient distance from the wharves.\n\n\
             Please confirm acceptance.\n\nRegards,\n\nColin",
            |index| ModelFacts {
                document_type: Some(
                    "Email thread regarding planning application consultation response".into(),
                ),
                type_evidence: vec![id_of(index, "Subject:")],
                document_date: Some("2018-03-21".into()),
                date_evidence: vec![id_of(index, "Date:")],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Email"),
            "{:?}",
            outcome.proposal
        );
    }

    /// "Response", "Citation", "Notification" and "Circular" are words a
    /// name is as readily made of as a title: a line of an inside address
    /// that holds one is still an organization's, and the letter is to it.
    /// A kind no name is made of still tells a title's line from a name's.
    #[test]
    fn a_kind_word_a_name_holds_leaves_an_organizations_line_alone() {
        assert!(organization_line("First Response Security"));
        assert!(organization_line("Circular Economy Scotland"));
        assert!(!organization_line("Final Notice Scotland"));
        let (outcome, _) = facts_for(
            "Northgate Borough Council\n\nWARNING LETTER\n\nMarch 4, 2024\n\n\
             Ms. Dana Whitlock\nOperations Manager\nFirst Response Security\n200 Mill Road\n\n\
             Dear Ms. Whitlock:\n\nAn officer of the Council inspected the stewarding of the \
             summer fair.",
            |index| ModelFacts {
                document_type: Some("Warning Letter".into()),
                type_evidence: vec![id_of(index, "WARNING LETTER")],
                document_date: Some("2024-03-04".into()),
                date_evidence: vec![id_of(index, "March 4, 2024")],
                parties: vec![party(
                    "Dana Whitlock",
                    Some(PartyRole::Recipient),
                    &[id_of(index, "Ms. Dana Whitlock")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["First Response Security"],
            "{:?}",
            outcome.proposal
        );
    }

    /// A company's name that holds a kind a name is as readily made of is
    /// no type: a reply that copied the letterhead's name as its type is
    /// not supported by it, and the letter stands in as it does for any
    /// other name ("Rapid Repair Plumbing"). A letterhead in capitals
    /// titles nothing, and neither does a field the word labels
    /// ("CITATION: R. v. ..."), as a "CASE REF:" field titles nothing.
    #[test]
    fn a_name_that_holds_a_kind_word_types_nothing() {
        let letter = |head: &str| {
            format!(
                "{head}\n12 Quay Street\nLondon\n\n4 March 2024\n\nMs. Dana Whitlock\n\
                 Harbour Fairs Ltd\n1 Pier Road\nLondon\n\nDear Ms. Whitlock:\n\n\
                 Thank you for your order of stands for the summer fair, which we will deliver \
                 on 1 May.\n\nYours sincerely,\n\nTom Hale"
            )
        };
        let reply = |company: &'static str, typed: bool| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: typed.then(|| company.into()),
                type_evidence: if typed {
                    vec![id_of(index, company)]
                } else {
                    Vec::new()
                },
                document_date: Some("2024-03-04".into()),
                date_evidence: vec![id_of(index, "4 March 2024")],
                parties: vec![
                    party(company, Some(PartyRole::Sender), &[id_of(index, company)]),
                    party(
                        "Harbour Fairs Ltd",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "Harbour Fairs Ltd")],
                    ),
                ],
                ..ModelFacts::default()
            }
        };
        let (control, _) = facts_for(
            &letter("Rapid Repair Plumbing"),
            reply("Rapid Repair Plumbing", true),
        );
        assert_eq!(control.proposal.document_type.as_deref(), Some("Letter"));
        assert!(control.reasons.contains(&ReviewReason::TypeUnsupported));
        for company in [
            "Citation Capital",
            "Notification Services Inc",
            "First Response Security Ltd",
            "Circular Economy Scotland",
            "Rapid Response Plumbing",
        ] {
            let (outcome, _) = facts_for(&letter(company), reply(company, true));
            assert_eq!(
                outcome.proposal.document_type.as_deref(),
                Some("Letter"),
                "{company}: {:?}",
                outcome.reasons
            );
            assert!(
                outcome.reasons.contains(&ReviewReason::TypeUnsupported),
                "{company}: {:?}",
                outcome.reasons
            );
            assert_eq!(outcome.status, ProposalStatus::NeedsReview, "{company}");
        }
        // A letterhead in capitals, and no type from the reply.
        for company in ["CITATION CAPITAL", "FIRST RESPONSE SECURITY"] {
            let (outcome, _) = facts_for(&letter(company), reply(company, false));
            assert_eq!(
                outcome.proposal.document_type.as_deref(),
                Some("Letter"),
                "{company}: {:?}",
                outcome.reasons
            );
        }
        // A court's decision that opens on its citation field.
        let decision = |field: &str| {
            format!(
                "{field} R. v. HARLOW, 2021 ONCA 123\nDATE: 20210304\n\n\
                 COURT OF APPEAL FOR ONTARIO\n\nBetween\n\nHer Majesty the Queen\n\n\
                 Respondent\n\nand\n\nPeter Harlow\n\nAppellant"
            )
        };
        let no_type = |_: &EvidenceIndex| ModelFacts::default();
        let (control, _) = facts_for(&decision("CASE REF:"), no_type);
        let (outcome, _) = facts_for(&decision("CITATION:"), no_type);
        assert_eq!(
            outcome.proposal.document_type, control.proposal.document_type,
            "{:?}",
            outcome.reasons
        );
        assert_ne!(outcome.proposal.document_type.as_deref(), Some("Citation"));
    }

    /// A response, a notification or a citation alone names no kind a
    /// person could file by, as a notice alone names none: the name waits
    /// for a person even when the page states the word as its heading and
    /// both parties are supported. The title that completes it is filed.
    #[test]
    fn a_bare_response_notification_or_citation_waits_for_a_person() {
        let notice = |heading: &str| {
            format!(
                "Northgate Borough Council\nTown Hall\nNorthgate NG1 1AA\n\n{heading}\n\n\
                 4 March 2024\n\nTo: Harbour Fairs Ltd, 1 Pier Road, Northgate\n\n\
                 Northgate Borough Council has considered the application of Harbour Fairs Ltd \
                 for a licence to hold the summer fair on the Common."
            )
        };
        let reply = |typed: &'static str, cited: &'static str| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some(typed.into()),
                type_evidence: vec![id_of(index, cited)],
                document_date: Some("2024-03-04".into()),
                date_evidence: vec![id_of(index, "4 March 2024")],
                parties: vec![
                    party(
                        "Northgate Borough Council",
                        Some(PartyRole::Issuer),
                        &[id_of(index, "Northgate Borough Council\nTown Hall")],
                    ),
                    party(
                        "Harbour Fairs Ltd",
                        Some(PartyRole::Recipient),
                        &[id_of(index, "To: Harbour Fairs Ltd")],
                    ),
                ],
                ..ModelFacts::default()
            }
        };
        for (heading, typed) in [
            ("RESPONSE", "Response"),
            ("NOTIFICATION", "Notification"),
            ("CITATION", "Citation"),
        ] {
            let (outcome, _) = facts_for(&notice(heading), reply(typed, heading));
            assert!(
                outcome.reasons.contains(&ReviewReason::TypeMissing),
                "{heading}: {:?} {:?}",
                outcome.proposal,
                outcome.reasons
            );
            assert_eq!(outcome.status, ProposalStatus::NeedsReview, "{heading}");
        }
        const TITLE: &str = "RESPONSE TO AN APPLICATION FOR A PREMISES LICENCE";
        let (outcome, _) = facts_for(&notice(TITLE), reply(TITLE, TITLE));
        assert!(
            !outcome.reasons.contains(&ReviewReason::TypeMissing),
            "{:?} {:?}",
            outcome.proposal,
            outcome.reasons
        );
    }

    /// A safety agency's notice as the worker extracts it: the title under
    /// the letterhead, then a "To:" field the layout ran into the next
    /// column's "Inspection Number: ...". The label of that field is nobody
    /// the notice is to; a name with its address after it still is.
    #[test]
    fn a_to_field_run_into_the_next_fields_label_names_no_one() {
        let notice = |to: &str| {
            format!(
                "U.S. Department of Works\n\nWorkplace Safety Administration\n\
                 1141 Harbour Drive\nSuite 1006\nMobile, AL 36609\n\n\
                 Notice of Proposed Penalty\n\n{to}\n\n\
                 | Rowan Brick Company, Inc. | Inspection Date(s): 08/30/2023 | - | 01/02/2024 |\n\
                 | and its successors | Issuance Date: | 02/20/2024 |\n\n\
                 3409 Water Avenue\nSelma, AL 36703\n\n\
                 This Notice of Proposed Penalty describes violations of the Workplace Safety \
                 Act of 1970."
            )
        };
        let reply = |index: &EvidenceIndex| ModelFacts {
            document_type: Some("Notice of Proposed Penalty".into()),
            type_evidence: vec![id_of(index, "Notice of Proposed")],
            document_date: Some("2024-02-20".into()),
            date_evidence: vec![id_of(index, "Issuance Date")],
            parties: vec![
                party(
                    "Rowan Brick Company, Inc.",
                    Some(PartyRole::Employer),
                    &[id_of(index, "Rowan Brick")],
                ),
                party(
                    "U.S. Department of Works",
                    Some(PartyRole::Issuer),
                    &[id_of(index, "Department of Works")],
                ),
            ],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(&notice("To: Inspection Number: 1695478"), reply);
        assert!(
            !outcome
                .proposal
                .parties
                .iter()
                .any(|party| party.contains("Inspection")),
            "{:?}",
            outcome.proposal
        );
        let (outcome, _) = facts_for(
            &notice("To: Jordan Avery, 3409 Water Avenue, Selma, AL 36703"),
            reply,
        );
        assert_eq!(
            outcome.proposal.parties,
            vec!["Jordan Avery"],
            "{:?}",
            outcome.proposal
        );
        // The field alone.
        for (to, named) in [
            ("To: Inspection Number: 1695478", None),
            (
                "To: Jordan Avery, 3409 Water Avenue, Selma, AL 36703",
                Some("Jordan Avery"),
            ),
        ] {
            let index = index_of(&notice(to));
            let context = whole(&index);
            let scope = ValidationScope::new(&index, &context, &[]);
            assert_eq!(
                addressee_field(&scope).map(|(name, _, _)| name).as_deref(),
                named,
                "{to}"
            );
        }
    }

    /// A sentence of an email's body that talks about a report is no title
    /// for the email: the reply's kind is not read back from a line that
    /// runs on from it into a sentence, and the header block stands in. A
    /// line that is the kind's own title still is, and so is a notice given
    /// "hereby".
    #[test]
    fn a_sentence_about_a_document_retitles_nothing() {
        let email = |line: &str| {
            format!(
                "From: Marsh, Colin (DLR) <colin.marsh@dlr.example>\n\
                 To: Petra Lindqvist <petra.lindqvist@borough.example>\n\
                 Date: Wed, 21 Mar 2018 11:16:03 +0000\n\
                 Subject: RE: 16/4008/F - Quay Trading Estate Re-consultation (PLA Ref DC 816)\n\n\
                 Petra,\n\n{line}\n\n\
                 The proposed development is a sufficient distance from the wharves.\n\n\
                 Please confirm acceptance.\n\nRegards,\n\nColin"
            )
        };
        let reply = |index: &EvidenceIndex| ModelFacts {
            document_type: Some("Email thread regarding planning consultation report".into()),
            type_evidence: vec![id_of(index, "Subject:")],
            document_date: Some("2018-03-21".into()),
            date_evidence: vec![id_of(index, "Date:")],
            ..ModelFacts::default()
        };
        let (outcome, _) = facts_for(&email("The report on the PLA is provided below:"), reply);
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Email"),
            "{:?}",
            outcome.proposal
        );
        let (outcome, _) = facts_for(&email("Report on the PLA Consultation"), reply);
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Report on the PLA Consultation"),
            "{:?}",
            outcome.proposal
        );
        assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
        assert!(runs_into_a_sentence(
            "The report on the PLA is provided below:",
            "report on the PLA"
        ));
        assert!(runs_into_a_sentence("Invoice will follow", "Invoice"));
        assert!(!runs_into_a_sentence("NOTICE IS HEREBY GIVEN", "NOTICE"));
        assert!(!runs_into_a_sentence(
            "Report on the PLA Consultation",
            "Report on the PLA Consultation"
        ));
    }

    /// A courtesy title is no part of a person's name in a filename; a
    /// title of rank, an organization's "Dr." and a long name keep theirs.
    #[test]
    fn a_courtesy_title_is_no_part_of_a_name() {
        for (written, name) in [
            ("Prof. A. G. Bell", "A. G. Bell"),
            ("MR. William Dew", "William Dew"),
            ("The Rt Hon Kemi Badenoch", "Kemi Badenoch"),
            ("Dr Jane Okafor", "Jane Okafor"),
            ("Sir Gus O'Donnell", "Sir Gus O'Donnell"),
            (
                "DR ING. H.C.F. PORSCHE AKTIEGESELLSCHAFT",
                "DR ING. H.C.F. PORSCHE AKTIEGESELLSCHAFT",
            ),
            ("Dr. Smith Dental Group", "Dr. Smith Dental Group"),
            ("Mr.", "Mr."),
            ("Dr. Smith", "Dr. Smith"),
            ("The Rt Hon Alex Chalk KC MP", "Alex Chalk"),
            ("John Mann MP", "John Mann MP"),
            // Two people addressed together keep their titles.
            ("Mr and Mrs Smith", "Mr and Mrs Smith"),
            ("Mr & Mrs Smith", "Mr & Mrs Smith"),
            ("Mr. and Mrs. Smith", "Mr. and Mrs. Smith"),
            ("Dr Jane and Mr Smith", "Dr Jane and Mr Smith"),
            ("Mr & Mrs J Smith", "Mr & Mrs J Smith"),
        ] {
            assert_eq!(without_courtesy_title(written), name, "{written}");
        }
    }

    /// A notice to two people addressed together is to both of them: the
    /// title before the first is no part of a name to cut.
    #[test]
    fn a_joint_addressee_keeps_the_title() {
        let (outcome, _) = facts_for(
            "NOTICE OF RENT INCREASE\n\nTo: Mr and Mrs Smith\n12 High Street\nNorthtown NT2 2BB\n\n\
             Date: 3 March 2024\n\nYour rent for 12 High Street will rise from 1 April 2024 to \
             650 pounds a month.\n\nNorthshire Council Housing Team",
            |index| ModelFacts {
                document_type: Some("Notice of Rent Increase".into()),
                type_evidence: vec![id_of(index, "NOTICE OF RENT")],
                document_date: Some("2024-03-03".into()),
                date_evidence: vec![id_of(index, "3 March 2024")],
                parties: vec![party(
                    "Mr and Mrs Smith",
                    Some(PartyRole::Tenant),
                    &[id_of(index, "To: Mr and Mrs Smith")],
                )],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.parties, vec!["Mr and Mrs Smith"]);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
    }

    /// A redaction's marker names no one; a name with the word in it is
    /// still a name.
    #[test]
    fn a_redaction_is_no_party() {
        assert!(is_redaction("REDACTED"));
        assert!(is_redaction("\\[REDACTED]"));
        assert!(is_redaction("[name withheld]"));
        assert!(!is_redaction("Redacted Records Ltd"));
        assert!(!is_redaction("Jane Doe"));
    }

    /// A "Re:" line's kind followed by its number names the document the
    /// letter is about, not the letter: the letter's layout stands in.
    #[test]
    fn a_numbered_matter_is_no_title() {
        let letter = |re: &str| {
            format!(
                "Food and Drug Administration\n\nFebruary 9, 2018\n\nCollegium Pharmaceutical, \
                 Inc.\n780 Dedham Street\n\n{re}\n\nDear Mr. Weet:\n\nThe Office of Prescription \
                 Drug Promotion has reviewed your materials.\n\nSincerely,\n\nJane Smith"
            )
        };
        for (re, title) in [
            ("RE: NDA 208090", None),
            ("Re: Invoice No. 4471", None),
            ("Re: Notice of Default", Some("Notice of Default")),
        ] {
            let index = index_of(&letter(re));
            let context = whole(&index);
            let scope = ValidationScope::new(&index, &context, &[]);
            assert_eq!(
                title_of(&scope).map(|(phrase, _)| phrase).as_deref(),
                title,
                "{re}"
            );
        }
    }

    /// In a document laid out as a letter, a type read from what the letter
    /// talks about below its salutation gives way to "Letter", still for a
    /// person to complete.
    #[test]
    fn a_letter_typed_by_its_talk_stands_in_as_a_letter() {
        let (outcome, _) = facts_for(
            "Office of Fair Trading\nFleetbank House\n\n11 September 2013\n\nDear Chancellor\n\n\
             I am writing to provide you with advice, as requested, concerning the divestments.\n\n\
             Yours sincerely,\n\nClive Maxwell",
            |index| ModelFacts {
                document_type: Some("advice".into()),
                type_evidence: vec![id_of(index, "I am writing")],
                document_date: Some("2013-09-11".into()),
                date_evidence: vec![id_of(index, "11 September 2013")],
                ..ModelFacts::default()
            },
        );
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Letter"));
        for reason in [
            ReviewReason::TypeUnsupported,
            ReviewReason::TypeInferred,
            ReviewReason::TypeMissing,
        ] {
            assert!(outcome.reasons.contains(&reason), "{:?}", outcome.reasons);
        }
    }

    /// A letter's title that says whom it answers names no kind of its
    /// own: the reply's party stands in it after a head that is no kind of
    /// document, and the kind after the party's name is the party's paper.
    /// "Letter" stands in and the name waits for a person. A head that names
    /// a kind is the type, a title on a paper that is no letter stays whole,
    /// and so does one whose party the reply did not name.
    #[test]
    fn a_letter_titled_by_whom_it_answers_stands_in_as_a_letter() {
        // As the worker extracts a word processor's letter: the letterhead
        // a table's cell, a date field, the title in bold over the greeting.
        let letter = |title: &str, greeting: &str, closing: &str| {
            format!(
                "|  |  |\n| --- | --- |\n|  | **Harbour Advisory Committee**<br>1st Floor, Quay \
                 Building <br>2 Marsh Street<br>London |\n\nDate: 24th October 2019\n\n\
                 **{title}**\n\n{greeting}The Harbour Advisory Committee (HAC) appreciates the \
                 opportunity to provide feedback on the recent adjustments to the QRA Harbour \
                 Statistics.\n\n{closing}Professor Mara Quill"
            )
        };
        let reply = |title: &'static str, recipient: &'static str| {
            move |index: &EvidenceIndex| ModelFacts {
                document_type: Some(title.into()),
                type_evidence: vec![id_of(index, title)],
                document_date: Some("2019-10-24".into()),
                date_evidence: vec![id_of(index, "24th October 2019")],
                parties: vec![
                    party(
                        "Harbour Advisory Committee",
                        Some(PartyRole::Issuer),
                        &[id_of(index, "**Harbour Advisory Committee**")],
                    ),
                    party(
                        recipient,
                        Some(PartyRole::Recipient),
                        &[id_of(index, "appreciates the opportunity")],
                    ),
                ],
                ..ModelFacts::default()
            }
        };
        const ANSWER: &str = "Response to the QRA Engagement on Adjustment of Harbour Statistics";
        let (outcome, _) = facts_for(
            &letter(ANSWER, "Dear Sir/Madam,\n\n", "Yours Sincerely\n\n"),
            reply(ANSWER, "QRA"),
        );
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Letter"),
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::To);
        assert_eq!(outcome.proposal.parties, vec!["QRA"]);
        for reason in [ReviewReason::TypeInferred, ReviewReason::TypeMissing] {
            assert!(outcome.reasons.contains(&reason), "{:?}", outcome.reasons);
        }
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        // A head that names a kind is the letter's type, the party cut
        // from it.
        const NOTICE: &str = "Notice to the QRA of Adjustment of Harbour Statistics";
        let (outcome, _) = facts_for(
            &letter(NOTICE, "Dear Sir/Madam,\n\n", "Yours Sincerely\n\n"),
            reply(NOTICE, "QRA"),
        );
        assert_eq!(outcome.proposal.document_type.as_deref(), Some("Notice"));
        // No greeting and no closing: no letter, and the paper keeps its
        // own title.
        let (outcome, _) = facts_for(&letter(ANSWER, "", ""), reply(ANSWER, "QRA"));
        assert_eq!(outcome.proposal.document_type.as_deref(), Some(ANSWER));
        // A reply that named no party the title names: the title is all
        // the type there is.
        let (outcome, _) = facts_for(
            &letter(ANSWER, "Dear Sir/Madam,\n\n", "Yours Sincerely\n\n"),
            reply(ANSWER, "Quay Regulation Authority"),
        );
        assert_eq!(outcome.proposal.document_type.as_deref(), Some(ANSWER));
    }

    /// A "The" that opens a name's line is the name's own; one in a sentence
    /// is not.
    #[test]
    fn a_line_opening_the_is_part_of_the_name() {
        let index = index_of(
            "UNITED STATES DISTRICT COURT\n\nTHE MCGRAW-HILL COMPANIES, INC.,\nPlaintiffs,\n\n\
             The parties met with the Acme Widget Company today.",
        );
        let unit = |needle: &str| {
            index
                .units()
                .iter()
                .find(|unit| unit.text.contains(needle))
                .expect("unit")
        };
        assert_eq!(
            written_span(unit("MCGRAW"), "mcgraw-hill companies inc").as_deref(),
            Some("THE MCGRAW-HILL COMPANIES, INC.")
        );
        assert_eq!(
            written_span(unit("Acme"), "acme widget company").as_deref(),
            Some("Acme Widget Company")
        );
        // A sentence that opens with the name keeps its "The".
        let index = index_of("The Acme Widget Company agrees to pay the invoice in full.");
        let sentence = index
            .units()
            .iter()
            .find(|unit| unit.text.contains("Acme"))
            .expect("unit");
        assert_eq!(
            written_span(sentence, "acme widget company").as_deref(),
            Some("Acme Widget Company")
        );
    }
}
