//! Turning a raw model reply into something safe to rename a file with.
//!
//! The goal is calibration, not timidity. Facts that can be checked literally
//! against the document are checked hard; everything else is left alone. A
//! proposal only goes to review when a *specific* thing is wrong with it, so
//! the review queue stays meaningful instead of collecting every long contract.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::{
    DateRole, ModelProposal, PartyRelation, ProposalStatus, ReviewReason, ValidatedProposal,
    ValidationOutcome,
};
use crate::evidence::{
    DateSpelling, NumericOrder, Segments, date_match_positions, date_statements, digest_contains,
    digest_contains_date, digest_contains_loosely, extract_stated_dates, is_valid_iso_date,
    normalize, normalize_loosely, numeric_date_order,
};
use crate::infer::{
    complete_type_from_title, dates_stated_on, infer_date_role, infer_document_type,
    labels_a_deadline, labels_the_issue_date, repair_issued_relation, window_before, wrapped_lines,
};

/// Below this self-reported confidence a proposal goes to review even when
/// every literal check passed.
pub const READY_CONFIDENCE: f32 = 0.60;
/// A description may say a little more than the filename, but only a little.
pub const MAX_DESCRIPTION_WORDS: usize = 42;
/// Share of a document type's significant words that must appear in its quote.
const TYPE_OVERLAP: f32 = 0.6;

/// Capitalised words that routinely open or punctuate a description and are not
/// claims about the document.
pub(crate) const GENERIC_CAPITALS: &[&str] = &[
    "a", "an", "and", "as", "at", "between", "by", "for", "from", "in", "of", "on", "the", "this",
    "to", "with", "it", "its", "their",
];

/// How far past the current year a document's date may lie before it reads
/// as a misread rather than a date: far enough for a lease or a term that
/// starts a few years out, short of the 2625 an OCR'd 2025 becomes.
const PLAUSIBLE_YEARS_AHEAD: i32 = 10;
/// The earliest year a document Intern files is taken to carry.
const EARLIEST_PLAUSIBLE_YEAR: i32 = 1900;

/// Checks a model proposal against the document it answered about.
///
/// Whether a date's year is plausible is judged against the current year;
/// [`validate_at`] takes that year as an argument instead.
pub fn validate(candidate: ModelProposal, digest: &impl Segments) -> ValidationOutcome {
    validate_at(candidate, digest, current_year())
}

/// [`validate`], with the current year given rather than read from the
/// clock, so a test can pin the one judgment that depends on today.
pub fn validate_at(
    candidate: ModelProposal,
    digest: &impl Segments,
    current_year: i32,
) -> ValidationOutcome {
    let mut reasons = Vec::new();
    let original = candidate.clone();

    let (mut document_type, type_supported) = validate_document_type(&candidate, digest);
    if !type_supported {
        push(&mut reasons, ReviewReason::TypeUnsupported);
    }
    // A supported type the title says more about is completed from the
    // title: "Journal" on a "Project Journal" is the document's own words,
    // whole, which is what the prompt asked for.
    if type_supported {
        document_type =
            document_type.map(|value| complete_type_from_title(&value, digest, &candidate.parties));
    }
    if document_type.is_none() {
        // A document with a title has a type. When the model gave none - or
        // invented one the document does not contain - the title is the
        // best-grounded answer available, and a person is asked to confirm it.
        match infer_document_type(digest) {
            Some(inferred) => {
                document_type = Some(inferred);
                push(&mut reasons, ReviewReason::TypeInferred);
            }
            None if type_supported => push(&mut reasons, ReviewReason::TypeMissing),
            None => {}
        }
    }

    let (mut document_date, mut date_role, date_supported, mut date_evidence_override) =
        validate_date(&candidate, digest, document_type.as_deref());
    if !date_supported {
        push(&mut reasons, ReviewReason::DateUnsupported);
    }
    // A due date is written in the document as plainly as an invoice date,
    // so the evidence check passes it, and the grammar's lack of a "due"
    // role does not stop the model reaching for it. The document's own
    // labels say which date it is: when every statement of the chosen date
    // is labelled a deadline, the one date labelled as the issue date takes
    // its place, and with no single such date the choice is a person's. A
    // date the referencing guard already replaced came from an effective
    // line, so it is left alone.
    let mut withheld_as_deadline = false;
    if date_evidence_override.is_none()
        && let Some(deadline) = document_date
            .as_deref()
            .and_then(|date| deadline_redirect(digest, date))
    {
        push(&mut reasons, ReviewReason::DateIsDeadline);
        match deadline {
            Deadline::Replaced { date, line } => {
                let invoice = document_type
                    .as_deref()
                    .is_some_and(|value| value.to_lowercase().contains("invoice"));
                document_date = Some(date);
                date_role = invoice.then_some(DateRole::Invoice);
                date_evidence_override = Some(line);
            }
            Deadline::Withheld => {
                // The model's date is still offered to the reviewer: it is
                // the candidate's, and the candidate is kept whole.
                document_date = None;
                date_role = None;
                withheld_as_deadline = true;
            }
        }
    }
    if document_date.is_none() && date_supported && !withheld_as_deadline {
        push(&mut reasons, ReviewReason::DateMissing);
    }
    // A year the document really prints can still be wrong: OCR reads 2025
    // as 2625, and a model that copies it faithfully passes every check
    // above. The date is kept - it may be right - and a person looks.
    if let Some(date) = document_date.as_deref()
        && !year_is_plausible(date, current_year)
    {
        push(&mut reasons, ReviewReason::DateImplausible);
    }
    // "Invoice Date: 04/01/2026" is 1 April in London and 4 January in New
    // York, and both readings pass every check above: each is a real date
    // the document prints. Unless the document settles it - the same date
    // in words or year first somewhere, or another numeric date that can
    // only be read one way round - the model's reading is a guess about a
    // convention. It is kept, because it may be right, and a person looks.
    if let Some(date) = document_date.as_deref()
        && reading_is_unsettled(digest, date)
    {
        push(&mut reasons, ReviewReason::DateAmbiguous);
    }
    // The wording around the date says what kind of date it is more reliably
    // than the model's label; the model's answer stands only where the
    // document says nothing.
    let date_role = document_date
        .as_deref()
        .and_then(|date| infer_date_role(digest, date, document_type.as_deref()))
        .or(date_role);

    let (parties, parties_supported) = validate_parties(&candidate, digest);
    if !parties_supported {
        push(&mut reasons, ReviewReason::PartyUnsupported);
    }
    let party_relation = if parties.is_empty() {
        PartyRelation::None
    } else if candidate.party_relation == PartyRelation::Between && parties.len() < 2 {
        // "between" needs two sides; one surviving party reads as "with".
        PartyRelation::With
    } else {
        candidate.party_relation
    };
    let (mut parties, party_relation) =
        repair_issued_relation(document_type.as_deref(), parties, party_relation, digest);
    // A stated one-sided relation is about one party. "to [John Smith,
    // Northstar Lantern Works LLC]" reads as a notice to both, which the
    // document does not say and the filename does not carry - it names the
    // first party only. So the relation the model stated decides how many
    // names the proposal may keep. `none` is left alone: it asserts nothing
    // about anybody, so a second validated name there is still just a name
    // the document contains.
    if matches!(
        party_relation,
        PartyRelation::For | PartyRelation::With | PartyRelation::From | PartyRelation::To
    ) {
        parties.truncate(1);
    }

    let description = validate_description(&candidate.description, digest, &mut reasons);

    if !candidate.confidence.is_finite() || candidate.confidence < READY_CONFIDENCE {
        push(&mut reasons, ReviewReason::LowConfidence);
    }
    if candidate.needs_review {
        push(&mut reasons, ReviewReason::ModelRequestedReview);
    }
    if digest
        .parser_warnings()
        .iter()
        .any(|warning| warning.field_affecting)
    {
        push(&mut reasons, ReviewReason::ParserWarning);
    }

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
            parties,
            party_relation,
            description,
            confidence: candidate.confidence,
            evidence: {
                let mut evidence = candidate.evidence;
                if let Some(line) = date_evidence_override {
                    evidence.date = Some(line);
                }
                evidence
            },
        },
        status,
        reasons,
        candidate: original,
        facts: None,
    }
}

/// A document type is accepted when most of its own significant words are
/// actually in the document.
///
/// Exact substring matching would reject "First Amendment to Consulting
/// Agreement" for a document headed "FIRST AMENDMENT TO THE CONSULTING
/// AGREEMENT", which is the right answer. Checking the words against the
/// document accepts that, and still rejects "Settlement Agreement" for a
/// statement of work, because "settlement" is nowhere in it.
fn validate_document_type(
    candidate: &ModelProposal,
    digest: &impl Segments,
) -> (Option<String>, bool) {
    let Some(document_type) = candidate
        .document_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return (None, true);
    };
    if !type_is_supported(document_type, digest) {
        return (None, false);
    }
    (Some(document_type.to_owned()), true)
}

/// Whether enough of a document type's significant words are in `digest`
/// ([`TYPE_OVERLAP`]).
pub(crate) fn type_is_supported(document_type: &str, digest: &impl Segments) -> bool {
    let words = normalize(document_type);
    let significant = words
        .split_whitespace()
        .filter(|word| word.len() > 2 && !GENERIC_CAPITALS.contains(word))
        .collect::<Vec<_>>();
    if significant.is_empty() {
        return false;
    }
    let matched = significant
        .iter()
        .filter(|word| digest_contains(digest, word))
        .count();
    (matched as f32) >= significant.len() as f32 * TYPE_OVERLAP
}

/// A date is accepted when it is a real calendar date that is written, in some
/// ordinary human form, in the document.
///
/// The model's quoted line is kept for the reviewer to read but is not the
/// gate: small models paraphrase their own quotes, and a correct date should
/// not be thrown away because the sentence around it was reworded.
fn validate_date(
    candidate: &ModelProposal,
    digest: &impl Segments,
    document_type: Option<&str>,
) -> (
    Option<String>,
    Option<crate::domain::DateRole>,
    bool,
    Option<String>,
) {
    let Some(date) = candidate
        .document_date
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return (None, None, true, None);
    };
    if !is_valid_iso_date(date) || !digest_contains_date(digest, date) {
        return (None, None, false, None);
    }
    // A date the document states only while naming ANOTHER agreement -
    // "Issued under the Master Services Agreement dated June 2, 2023" - is
    // that other document's date, and must never become this filename's date.
    // The model is told this and usually complies, but greedy decoding is not
    // hardware-deterministic and the two candidates can sit a rounding error
    // apart, so the guarantee lives here, where nothing wobbles: if the
    // document states exactly one other date on an effective/commencement
    // line, that date is the answer; otherwise the document goes to review.
    if date_is_tainted(digest, date, document_type) {
        if let [(alternate, line)] = effective_alternates(digest, date).as_slice() {
            return (
                Some(alternate.clone()),
                Some(crate::domain::DateRole::Effective),
                true,
                Some(line.clone()),
            );
        }
        return (None, None, false, None);
    }
    (Some(date.to_owned()), candidate.date_role, true, None)
}

/// Whether `digest` states `date` only as another document's date: every
/// statement of it introduced by a reference to another agreement, and none
/// naming this document.
pub(crate) fn date_is_tainted(
    digest: &impl Segments,
    date: &str,
    document_type: Option<&str>,
) -> bool {
    // The taint is judged over wrapped lines, not raw ones. A PDF breaks
    // "... Northstar Lantern Works LLC dated" from "March 3, 2024" wherever
    // the margin falls, and read raw the second half looks like a date
    // nothing introduced - which clears the taint and lets the referenced
    // agreement's date through. The search for a replacement
    // ([`effective_alternates`]) stays on raw lines, because there a line
    // break is a real boundary: "effective as of April 1, 2026 and
    // continues" / "through March 31, 2027" states one effective date and
    // one end of term, not two candidates.
    let wrapped: Vec<String> = digest
        .segments()
        .iter()
        .flat_map(|segment| crate::infer::wrapped_lines(segment))
        .collect();
    let mut stated = false;
    let mut tainted = true;
    for line in &wrapped {
        let normalized = normalize(line);
        for position in date_match_positions(date, &normalized) {
            stated = true;
            if names_this_document(&normalized, position, document_type)
                || !reference_introduced(&normalized, position)
            {
                tainted = false;
            }
        }
    }
    stated && tainted
}

/// The dates other than `date` that `digest` states cleanly on an
/// effective or commencement line, each once with the line it stands on:
/// the candidates to replace a tainted date with.
pub(crate) fn effective_alternates(digest: &impl Segments, date: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = digest
        .segments()
        .iter()
        .flat_map(|segment| segment.lines())
        .collect();
    let mut alternates: Vec<(String, String)> = Vec::new();
    for line in &lines {
        let normalized = normalize(line);
        if !EFFECTIVE_CUES.iter().any(|cue| normalized.contains(cue)) {
            continue;
        }
        for found in extract_stated_dates(line) {
            if found == *date || alternates.iter().any(|(existing, _)| *existing == found) {
                continue;
            }
            let clean = date_match_positions(&found, &normalized)
                .iter()
                .any(|&position| !reference_introduced(&normalized, position));
            if clean {
                alternates.push((found, line.trim().to_owned()));
            }
        }
    }
    alternates
}

/// What became of a date the document states only as a deadline.
enum Deadline {
    /// The one date the document labels as its issue date, and the line it
    /// stands on.
    Replaced { date: String, line: String },
    /// No single issue date: the date is withheld for a person to choose.
    Withheld,
}

/// `Some` when every statement of `date` the document makes - references to
/// other documents aside - is labelled a due, expiry, renewal, or deadline
/// date ("Due Date: 05/30/2025", "Payment due", "Expires on"). The
/// replacement is the one other date labelled as the date of issue ("Invoice
/// Date:", "Date of issue", "Dated", a bare "Date:"); two or none, and the
/// date is withheld rather than guessed at. A numeric date both readings fit
/// counts as two, unless the document's other numeric dates settle which way
/// round it writes them ([`numeric_date_order`]), the way the date chips
/// read it.
///
/// A single unlabelled statement of the date anywhere clears it: the
/// document then says something besides "this is when it is due".
fn deadline_redirect(digest: &impl Segments, date: &str) -> Option<Deadline> {
    if !deadline_fires(digest, date) {
        return None;
    }
    Some(match issue_date_alternates(digest, date).as_slice() {
        [(alternate, line)] => Deadline::Replaced {
            date: alternate.clone(),
            line: line.clone(),
        },
        _ => Deadline::Withheld,
    })
}

/// Whether every statement of `date` in `digest`, references to other
/// documents aside, is labelled a deadline - and there is at least one.
pub(crate) fn deadline_fires(digest: &impl Segments, date: &str) -> bool {
    let lines: Vec<String> = digest
        .segments()
        .iter()
        .flat_map(|segment| wrapped_lines(segment))
        .collect();
    let mut stated = false;
    for line in &lines {
        let normalized = normalize(line);
        for position in date_match_positions(date, &normalized) {
            if reference_introduced(&normalized, position) {
                continue;
            }
            if !labels_a_deadline(&window_before(&normalized, position)) {
                return false;
            }
            stated = true;
        }
    }
    stated
}

/// The dates other than `date` that `digest` labels as the date of issue,
/// each once with the line it stands on: the candidates to replace a
/// deadline with.
pub(crate) fn issue_date_alternates(digest: &impl Segments, date: &str) -> Vec<(String, String)> {
    let lines: Vec<String> = digest
        .segments()
        .iter()
        .flat_map(|segment| wrapped_lines(segment))
        .collect();
    let order = numeric_date_order(digest);
    let mut alternates: Vec<(String, String)> = Vec::new();
    for line in &lines {
        let normalized = normalize(line);
        for (found, position) in dates_stated_on(&normalized, order) {
            if found == date
                || alternates.iter().any(|(existing, _)| *existing == found)
                || reference_introduced(&normalized, position)
                || !labels_the_issue_date(&window_before(&normalized, position))
            {
                continue;
            }
            alternates.push((found, line.trim().to_owned()));
        }
    }
    alternates
}

/// True when the document writes `date` only in numbers that read as two
/// different dates, and nothing in the document says which one it means.
///
/// The document settles the reading when it writes the same date with the
/// month in words or year first, or when its numeric dates show an order -
/// some numeric date can only be read one way round, and none only the
/// other ([`numeric_date_order`]). A date the document states only against
/// its own order - "03/04/2026" taken as 4 March in a document whose
/// "30/01/2026" shows it writes the day first - is unsettled too: the
/// model's reading contradicts the document's.
///
/// A day above 12 or a day equal to the month reads one way only, and is
/// never ambiguous.
pub(crate) fn reading_is_unsettled(digest: &impl Segments, date: &str) -> bool {
    let (Some(month), Some(day)) = (
        date.get(5..7).and_then(|value| value.parse::<u32>().ok()),
        date.get(8..10).and_then(|value| value.parse::<u32>().ok()),
    ) else {
        return false;
    };
    if month > 12 || day > 12 || month == day {
        return false;
    }
    let spellings = digest
        .segments()
        .iter()
        .flat_map(|segment| date_statements(date, &normalize(segment)))
        .map(|(_, spelling)| spelling)
        .collect::<Vec<_>>();
    if spellings.is_empty()
        || spellings
            .iter()
            .any(|spelling| matches!(spelling, DateSpelling::Written | DateSpelling::YearFirst))
    {
        return false;
    }
    match numeric_date_order(digest) {
        None => true,
        Some(NumericOrder::DayFirst) => !spellings.contains(&DateSpelling::DayFirst),
        Some(NumericOrder::MonthFirst) => !spellings.contains(&DateSpelling::MonthFirst),
    }
}

/// Whether an accepted ISO date's year is one a document could carry.
pub(crate) fn year_is_plausible(date: &str, current_year: i32) -> bool {
    date.get(..4)
        .and_then(|year| year.parse::<i32>().ok())
        .is_none_or(|year| {
            (EARLIEST_PLAUSIBLE_YEAR..=current_year.saturating_add(PLAUSIBLE_YEARS_AHEAD))
                .contains(&year)
        })
}

/// The current year in UTC, from the system clock. A clock set before 1970
/// reads as 1970, which only makes the plausibility check more lenient.
pub(crate) fn current_year() -> i32 {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() / 86_400);
    civil_year_from_days(i64::try_from(days).unwrap_or(0))
}

/// The Gregorian year of a day counted from 1970-01-01: the year half of
/// Howard Hinnant's `civil_from_days`, exact for every day a clock can
/// report, leap centuries included. The standard library has no calendar,
/// and one year is not worth a dependency.
fn civil_year_from_days(days: i64) -> i32 {
    // Counted from 0000-03-01, so a leap day is the last day of its year.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    // Months counted from March: 10 and 11 are January and February, which
    // belong to the next calendar year.
    let month_from_march = (5 * day_of_year + 2) / 153;
    let year = year_of_era + era * 400 + i64::from(month_from_march >= 10);
    i32::try_from(year).unwrap_or(i32::MAX)
}

/// Whether the wording immediately before a date's occurrence marks it as
/// ANOTHER document's date. Judged per occurrence, not per line, because one
/// line can state two dates in two roles: "...Amendment to the Consulting
/// Agreement dated September 1, 2020, is entered into as of September 14,
/// 2025" references the first date and owns the second.
///
/// "dated" directly before the occurrence is the referencing construction -
/// "the Master Services Agreement dated June 2, 2023" - unless the naming
/// phrase begins with "this", which is how a document dates itself.
pub(crate) fn reference_introduced(normalized: &str, position: usize) -> bool {
    const NEAR: usize = 12;
    // Wide enough to reach back over a party's full name - "the Employment
    // Agreement between you and Northstar Lantern Works LLC dated" is 70
    // characters - and no wider than the window the date's role is read in.
    const WIDE: usize = 96;
    fn window_start(normalized: &str, position: usize, span: usize) -> usize {
        let mut start = position.saturating_sub(span);
        while !normalized.is_char_boundary(start) {
            start -= 1;
        }
        start
    }
    // Words are found whole, in everything before the date, so the letters
    // around one decide what it is even where a window's edge cuts it:
    // "Contractor" is not a contract, "border" not an order, "updated" not
    // "dated".
    let before = &normalized[..position];
    if rfind_word(before, "dated").is_some_and(|at| at >= window_start(normalized, position, NEAR))
    {
        // "dated" only references another document when a document noun
        // introduces it - "the Master Services Agreement dated June 2, 2023".
        // A bare "Dated January 8, 2025" on a title block is the document
        // dating itself, and "This Agreement dated ..." is too. It is the
        // determiner on the noun nearest the date that decides which of
        // those it is, not a "this" anywhere in the window: "This First
        // Amendment to the Consulting Agreement dated September 1, 2020"
        // states the *consulting agreement's* date, and opens with "This".
        let wide = window_start(normalized, position, WIDE);
        let noun_at = last_document_noun(before).filter(|&at| at >= wide);
        return noun_at.is_some_and(|at| !determined_by_this(before, at));
    }
    // A citation runs straight into the date it cites: "issued under the MSA
    // effective June 2, 2023". Clause punctuation in between means the
    // sentence moved on - "Pursuant to Section 9.2 of the Employment
    // Agreement, your employment will terminate effective January 31, 2027"
    // dates the termination, not the agreement it cites.
    let wide = &normalized[window_start(normalized, position, WIDE)..position];
    REFERENCE_CUES.iter().any(|cue| {
        wide.rfind(cue)
            .is_some_and(|at| !ends_the_clause(&wide[at + cue.len()..]))
    })
}

/// Whether the wording between a citation and a date ends the clause the
/// citation opened: a semicolon, a colon, a full stop that ends a sentence,
/// or a comma that a clause of its own follows. The point in "Section 9.2",
/// "No. 12" or "Acme Inc." and the comma in "Contoso Worldwide, Inc." are
/// part of what is cited, and reading them as the clause ending would let
/// the cited agreement's date through as this document's own.
///
/// So are the commas that set off what a citation says about the document
/// it cites: "issued under the Master Services Agreement, effective June 2,
/// 2023," and "the Master Services Agreement, as amended, effective June 2,
/// 2023" both date the agreement. What follows such a comma only introduces
/// the date or qualifies the citation ([`CITATION_WORDS`]). A comma ends the
/// clause when anything more stands after it - "..., your employment will
/// terminate effective January 31, 2027", "..., is effective as of" - since
/// a subject or a verb of its own is a new clause, and the date is its.
fn ends_the_clause(between: &str) -> bool {
    // Everything up to the first comma is what is cited. Each part after a
    // comma must carry the citation on, or the clause has ended.
    let mut part_start = None;
    for (index, character) in between.char_indices() {
        let rest = &between[index + character.len_utf8()..];
        match character {
            ',' if !rest.split_whitespace().next().is_some_and(|word| {
                COMPANY_FORMS.contains(
                    &word.trim_end_matches(|character: char| !character.is_alphanumeric()),
                )
            }) =>
            {
                if part_start.is_some_and(|start| !continues_the_citation(&between[start..index])) {
                    return true;
                }
                part_start = Some(index + character.len_utf8());
            }
            ';' | ':' => return true,
            '.' if rest.starts_with(char::is_whitespace)
                && !is_abbreviation_period(between, index) =>
            {
                return true;
            }
            _ => {}
        }
    }
    part_start.is_some_and(|start| !continues_the_citation(&between[start..]))
}

/// Whether the words between two of a citation's commas, or between its
/// last comma and the date, still speak of the cited document: nothing but
/// [`CITATION_WORDS`].
fn continues_the_citation(part: &str) -> bool {
    part.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .all(|word| CITATION_WORDS.contains(&word))
}

/// The words a citation's commas can set off without starting a clause:
/// the ones that introduce its date (", effective", ", dated as of", ",
/// made and entered into as of") and the ones that say which version is
/// cited (", as amended and restated from time to time,").
const CITATION_WORDS: &[&str] = &[
    "effective",
    "dated",
    "as",
    "of",
    "on",
    "commencing",
    "made",
    "entered",
    "into",
    "and",
    "amended",
    "restated",
    "supplemented",
    "modified",
    "from",
    "time",
    "to",
];

/// What follows the comma in a company's name: "Contoso Worldwide, Inc.",
/// "Contoso Bank, N.A.".
const COMPANY_FORMS: &[&str] = &[
    "inc", "llc", "l.l.c", "ltd", "corp", "co", "llp", "l.l.p", "lp", "l.p", "pllc", "plc", "pc",
    "p.c", "n.a", "gmbh", "ag", "s.a", "sa", "n.v", "nv", "b.v", "bv", "pty",
];

/// Nouns that name a document, for telling "the Master Services Agreement
/// dated" from a bare "Dated".
const DOCUMENT_NOUNS: &[&str] = &["agreement", "contract", "order", "amendment", "memorandum"];
/// Wording that cites another document as the authority for this one.
const REFERENCE_CUES: &[&str] = &["issued under", "pursuant to", "as amended", "amending "];
/// Determiners and prepositions that make a document noun somebody else's:
/// "the Master Services Agreement", "under its Agreement", "of said
/// Contract".
const REFERRING_WORDS: &[&str] = &[
    "the", "that", "said", "such", "a", "an", "any", "each", "its", "their", "your", "our", "to",
    "of", "under", "with", "between", "by",
];

/// Whether the words before the document noun at `noun_at` in `before` make
/// it this document: "This Consulting Agreement", "This First Amendment",
/// `(this "Amendment")`.
///
/// Up to four words are read back from the noun, stepping over quotation
/// marks and parentheses, which are typography. "this" settles it as this
/// document. A determiner or preposition settles it as another one, and so
/// does a word that ends a clause, or running out of words: a noun nobody
/// qualified is how a document cites another, and a cover line that names
/// the document itself is recognised separately, by its type.
///
/// One determiner is not the noun's own: the article of a defined term,
/// `This Consulting Agreement (the "Agreement") dated ...`, belongs to the
/// definition. There the document noun the parenthesis defines is read
/// instead, so the term says whatever the name it stands for says - this
/// agreement here, somebody else's in `the Master Services Agreement (the
/// "Agreement") dated ...`.
fn determined_by_this(before: &str, noun_at: usize) -> bool {
    let mut end = noun_at;
    let mut read = 0;
    while read < 4 {
        let lead = before[..end].trim_end();
        if lead.is_empty() {
            return false;
        }
        let start = lead
            .char_indices()
            .rev()
            .find(|(_, character)| character.is_whitespace())
            .map_or(0, |(at, space)| at + space.len_utf8());
        let raw = &lead[start..];
        end = start;
        let word = raw.trim_matches(|character: char| matches!(character, '"' | '\'' | '(' | ')'));
        if word.is_empty() {
            continue;
        }
        read += 1;
        if word.ends_with([',', ';', ':', '.']) {
            return false;
        }
        let bare = word.trim_matches(|character: char| !character.is_alphanumeric());
        if bare == "this" {
            return true;
        }
        if REFERRING_WORDS.contains(&bare) {
            return raw.starts_with('(')
                && noun_ending(&before[..start])
                    .is_some_and(|outer| determined_by_this(before, outer));
        }
    }
    false
}

/// Where the document noun that `text` ends on begins, if it ends on one:
/// the "Agreement" a defined term's parenthesis follows.
fn noun_ending(text: &str) -> Option<usize> {
    let text = text.trim_end();
    last_word_start(text).filter(|&start| is_document_noun(&text[start..]))
}

/// Where the last document noun in `text` begins: a word [`is_document_noun`]
/// accepts, read whole.
fn last_document_noun(text: &str) -> Option<usize> {
    let mut end = text.len();
    loop {
        let lead = text[..end].trim_end_matches(|character: char| !character.is_alphanumeric());
        let start = last_word_start(lead)?;
        if is_document_noun(&lead[start..]) {
            return Some(start);
        }
        end = start;
    }
}

/// Where the run of letters and digits `text` ends on begins, or `None`
/// when `text` does not end on one.
fn last_word_start(text: &str) -> Option<usize> {
    if !text.chars().next_back().is_some_and(char::is_alphanumeric) {
        return None;
    }
    Some(
        text.char_indices()
            .rev()
            .find(|(_, character)| !character.is_alphanumeric())
            .map_or(0, |(at, character)| at + character.len_utf8()),
    )
}

/// Whether a whole word names a document: one of [`DOCUMENT_NOUNS`], in the
/// plural as well ("the Loan Agreements", "Change Orders", "memoranda"),
/// and with "sub" in front ("the Subcontract"). Nothing else runs into the
/// noun, so a contractor, a subcontractor, a border and a disagreement name
/// no document.
fn is_document_noun(word: &str) -> bool {
    let word = word.strip_prefix("sub").unwrap_or(word);
    word == "memoranda"
        || DOCUMENT_NOUNS.iter().any(|noun| {
            word.strip_prefix(noun)
                .is_some_and(|plural| plural.is_empty() || plural == "s")
        })
}

/// The last place `word` stands in `haystack` as a whole word, with no
/// letter or digit running into it on either side.
pub(crate) fn rfind_word(haystack: &str, word: &str) -> Option<usize> {
    haystack.rmatch_indices(word).map(|(at, _)| at).find(|&at| {
        let before = haystack[..at].chars().next_back();
        let after = haystack[at + word.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// Whether the wording before the date is the document naming itself:
/// "SERVICES AGREEMENT dated as of March 1, 2026" is a cover page dating
/// itself, not a reference to somebody else's agreement, and the referencing
/// guard would otherwise throw away the only date the document has.
///
/// Only the document's own validated type counts as that naming. Every
/// referencing line in the corpus says more than the type - "the Employment
/// Agreement between you and Northstar Lantern Works LLC dated" - so the
/// guard against trap dates is untouched.
fn names_this_document(normalized: &str, position: usize, document_type: Option<&str>) -> bool {
    let Some(document_type) = document_type else {
        return false;
    };
    let lead = normalized[..position].trim_end();
    let lead = lead.strip_suffix("as of").unwrap_or(lead).trim_end();
    let lead = lead.strip_suffix("dated").unwrap_or(lead).trim_end();
    !lead.is_empty() && lead == normalize(document_type)
}

const EFFECTIVE_CUES: &[&str] = &[
    "effective",
    "commencement",
    "commencing",
    "start date",
    "in force",
    "entered into",
];

/// A party is accepted when its name appears in the document.
///
/// Verbatim first; failing that, with punctuation disregarded, because
/// "Contoso Worldwide Inc" for a document that writes "Contoso Worldwide,
/// Inc." is the same company and the corpus showed the model dropping the
/// comma often enough that a real name was reaching review over it. Words are
/// never loosened: a name the document does not contain is still rejected.
///
/// The same loosening says when two names are one party: "ACME CORP" from a
/// letterhead and "Acme Corp." from the body are one company, and the first
/// spelling is kept. A name is never merged into a longer one that contains
/// it - "Acme" and "Acme Holdings" are an affiliate agreement's two sides.
fn validate_parties(candidate: &ModelProposal, digest: &impl Segments) -> (Vec<String>, bool) {
    let mut kept = Vec::new();
    let mut all_supported = true;
    for party in &candidate.parties {
        let party = party.trim();
        if party.is_empty() {
            all_supported = false;
            continue;
        }
        if digest_contains(digest, party) || digest_contains_loosely(digest, party) {
            let key = normalize_loosely(party);
            if !kept
                .iter()
                .any(|existing: &String| normalize_loosely(existing) == key)
            {
                kept.push(party.to_owned());
            }
        } else {
            all_supported = false;
        }
    }
    kept.truncate(2);
    (kept, all_supported)
}

pub(crate) fn validate_description(
    description: &str,
    digest: &impl Segments,
    reasons: &mut Vec<ReviewReason>,
) -> String {
    let trimmed = description.trim();
    let mut sentence = trimmed.to_owned();
    // Keep the first sentence; a small model sometimes keeps going. A
    // terminator only ends a sentence when a space follows it, because the
    // period inside "$1,248.00" is a decimal point and cutting there left
    // "An invoice for $1,248." - four words, which the sentence check then
    // called invalid and sent a perfectly good invoice to review.
    for (index, character) in trimmed.char_indices() {
        if matches!(character, '.' | '!' | '?')
            && trimmed[index + character.len_utf8()..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
            && !is_abbreviation_period(trimmed, index)
        {
            sentence = trimmed[..index + character.len_utf8()].to_owned();
            break;
        }
    }
    if sentence.split_whitespace().count() > MAX_DESCRIPTION_WORDS {
        let mut words = sentence
            .split_whitespace()
            .take(MAX_DESCRIPTION_WORDS)
            .collect::<Vec<_>>();
        if let Some(last) = words.last_mut() {
            *last = last.trim_end_matches([',', ';', '.', '!', '?']);
        }
        sentence = format!("{}.", words.join(" "));
        push(reasons, ReviewReason::DescriptionInvalid);
    }
    if !is_usable_sentence(&sentence) {
        push(reasons, ReviewReason::DescriptionInvalid);
    }
    if let Some(unsupported) = first_unsupported_claim(&sentence, digest) {
        let _ = unsupported;
        push(reasons, ReviewReason::DescriptionUnsupported);
    }
    sentence
}

/// Whether the period at `index` closes an abbreviation rather than the
/// sentence: a company form ("Inc.", "GmbH."), a title, a month ("Jan. 5"),
/// a dotted initialism ("P.C.", "U.K.", "N.A.", "L.L.C."), or one letter.
/// Two-letter words that are also company forms elsewhere - "AG", "SA" - are
/// not listed bare, because as words they end real sentences; dotted, they
/// are initialisms.
///
/// An abbreviation still ends the sentence when a new one plainly starts
/// after it - "... Contoso Ltd. The invoice ..." - since a name never
/// continues with a capitalised "The".
fn is_abbreviation_period(value: &str, index: usize) -> bool {
    let before = value[..index]
        .split(|character: char| character.is_whitespace())
        .next_back()
        .unwrap_or_default()
        .trim_start_matches(|character: char| !character.is_alphanumeric());
    let lowered = before.to_ascii_lowercase();
    let abbreviation = matches!(
        lowered.as_str(),
        "inc"
            | "llc"
            | "ltd"
            | "corp"
            | "co"
            | "no"
            | "mr"
            | "mrs"
            | "ms"
            | "dr"
            | "jr"
            | "sr"
            | "st"
            | "u.s"
            | "e.g"
            | "i.e"
            | "etc"
            | "vs"
            | "approx"
            | "dept"
            | "assoc"
            | "bros"
            | "ave"
            | "blvd"
            | "ste"
            | "esq"
            | "ph.d"
            | "m.d"
            | "j.d"
            | "llp"
            | "pllc"
            | "plc"
            | "pty"
            | "gmbh"
            | "intl"
            | "mfg"
            | "assn"
            | "jan"
            | "feb"
            | "mar"
            | "apr"
            | "jun"
            | "jul"
            | "aug"
            | "sep"
            | "sept"
            | "oct"
            | "nov"
            | "dec"
    ) || is_dotted_initialism(&lowered)
        || before.chars().filter(char::is_ascii_alphabetic).count() == 1;
    abbreviation && !opens_a_sentence(&value[index + 1..])
}

/// "p.c", "u.k", "l.l.c": every dot-separated part one ASCII letter.
fn is_dotted_initialism(token: &str) -> bool {
    token.contains('.')
        && token
            .split('.')
            .all(|part| part.len() == 1 && part.bytes().all(|byte| byte.is_ascii_alphabetic()))
}

/// Capitalised words that only ever open a sentence.
const SENTENCE_OPENERS: &[&str] = &[
    "the", "this", "that", "these", "those", "it", "its", "they", "their", "there", "a", "an",
];

/// Whether the text after a period starts a new sentence with a word that
/// could not be the rest of a name.
fn opens_a_sentence(rest: &str) -> bool {
    let Some(word) = rest.split_whitespace().next() else {
        return false;
    };
    let bare = word.trim_end_matches(|character: char| !character.is_alphanumeric());
    bare.chars().next().is_some_and(char::is_uppercase)
        && SENTENCE_OPENERS.contains(&bare.to_lowercase().as_str())
}

fn is_usable_sentence(description: &str) -> bool {
    let trimmed = description.trim();
    let Some(last) = trimmed.chars().last() else {
        return false;
    };
    if !matches!(last, '.' | '!' | '?') {
        return false;
    }
    let words = trimmed.split_whitespace().count();
    let starts_like_sentence = trimmed
        .chars()
        .find(|character| character.is_alphabetic())
        .is_some_and(char::is_uppercase);
    (6..=MAX_DESCRIPTION_WORDS).contains(&words) && starts_like_sentence
}

/// Finds the first specific claim in the description that the document does not
/// contain: a number, or a capitalised name.
///
/// Only specifics are checked. Ordinary prose the model wrote to glue the
/// sentence together is not a claim about the document and must not send an
/// otherwise good proposal to review.
pub(crate) fn first_unsupported_claim(description: &str, digest: &impl Segments) -> Option<String> {
    let restated = restated_dates(description, digest);
    for (index, raw) in description.split_whitespace().enumerate() {
        if restated[index] {
            continue;
        }
        let token = raw.trim_matches(|character: char| !character.is_alphanumeric());
        if token.chars().count() < 3 {
            continue;
        }
        let has_digit = token.bytes().any(|byte| byte.is_ascii_digit());
        let capitalised = index > 0
            && token.chars().next().is_some_and(char::is_uppercase)
            && !GENERIC_CAPITALS.contains(&token.to_ascii_lowercase().as_str());
        if !(has_digit || capitalised) {
            continue;
        }
        if !claim_is_supported(digest, token) {
            return Some(token.to_owned());
        }
    }
    None
}

/// Which of the description's words restate a date the document states,
/// one flag per word. "January 5, 2026" for a document that prints
/// "01/05/2026" is the same fact, but read word by word "January" is a name
/// the document never writes. Each date is matched on the fewest words that
/// state it - month, day or ordinal, year, and the comma between - and only
/// a date the document itself states is excused; any other is checked as
/// before.
fn restated_dates(description: &str, digest: &impl Segments) -> Vec<bool> {
    // "1st day of April, 2026" is the longest shape a date is read in.
    const LONGEST: usize = 5;
    let words: Vec<&str> = description.split_whitespace().collect();
    let stated = |range: std::ops::Range<usize>| extract_stated_dates(&words[range].join(" "));
    let mut supported: Vec<(String, bool)> = Vec::new();
    let mut restated = vec![false; words.len()];
    for start in 0..words.len() {
        for end in start + 1..=(start + LONGEST).min(words.len()) {
            for date in stated(start..end) {
                // Only the tightest span: one that loses the date when
                // either end word is dropped.
                if stated(start + 1..end).contains(&date) || stated(start..end - 1).contains(&date)
                {
                    continue;
                }
                let in_document = match supported.iter().find(|(known, _)| *known == date) {
                    Some((_, in_document)) => *in_document,
                    None => {
                        let in_document = digest_contains_date(digest, &date);
                        supported.push((date, in_document));
                        in_document
                    }
                };
                if in_document {
                    restated[start..end].fill(true);
                }
            }
        }
    }
    restated
}

/// Whether one specific claim is in the document, allowing for the ways a
/// sentence reshapes a fact it is quoting.
///
/// A possessive ("Acme's" for a document that writes "Acme"), a thousands
/// separator the document omits ("248,000" for "248000"), or a hyphen the
/// model added between two words that the document keeps apart are all the
/// same fact. Words themselves are never changed; a name that is not in the
/// document is still an unsupported claim.
fn claim_is_supported(digest: &impl Segments, token: &str) -> bool {
    if digest_contains(digest, token) {
        return true;
    }
    let mut variants: Vec<String> = Vec::new();
    for possessive in ["'s", "\u{2019}s"] {
        if let Some(stem) = token.strip_suffix(possessive) {
            variants.push(stem.to_owned());
        }
    }
    if token.bytes().any(|byte| byte.is_ascii_digit()) && token.contains(',') {
        variants.push(token.replace(',', ""));
    }
    if token.contains('-') {
        variants.push(token.replace('-', " "));
    }
    variants
        .iter()
        .any(|variant| variant.chars().count() >= 3 && digest_contains(digest, variant))
}

pub(crate) fn push(reasons: &mut Vec<ReviewReason>, reason: ReviewReason) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::DocumentDigest;
    use crate::distill::{DigestBudget, distill, source_from_text};
    use crate::domain::{DateRole, Evidence};

    fn digest_of(text: &str) -> DocumentDigest {
        let digest = distill(&source_from_text(text), DigestBudget::default());
        SOURCES.with(|sources| {
            sources
                .borrow_mut()
                .push((digest.text.clone(), text.to_owned()))
        });
        digest
    }

    thread_local! {
        /// The text each digest of this module's tests was distilled from.
        static SOURCES: std::cell::RefCell<Vec<(String, String)>> =
            const { std::cell::RefCell::new(Vec::new()) };
    }

    /// Every test here validates against a digest; this validates the same
    /// reply against the evidence pipeline's view of the same document -
    /// its units, all of them, as the context of a document that goes
    /// whole - and requires the same outcome. The literal checks read the
    /// two views alike, so a difference is a difference in how the index
    /// cuts a document into units, which matters to the evidence pipeline.
    fn validate(candidate: ModelProposal, digest: &DocumentDigest) -> ValidationOutcome {
        let outcome = super::validate(candidate.clone(), digest);
        let source = SOURCES.with(|sources| {
            sources
                .borrow()
                .iter()
                .find(|(text, _)| *text == digest.text)
                .map(|(_, source)| source.clone())
        });
        if let Some(source) = source
            && !digest.compressed
        {
            let index = crate::index::EvidenceIndex::build(&source_from_text(source));
            let config = crate::retrieve::RetrievalConfig {
                whole_document_tokens: u32::MAX,
                ..crate::retrieve::RetrievalConfig::default()
            };
            let context = crate::retrieve::retrieve(&index, &config, 100);
            let scope = crate::facts::ValidationScope::new(&index, &context, &[]);
            let units = super::validate(candidate, scope.context());
            assert_eq!(
                units, outcome,
                "the evidence index's units validate this reply differently from the digest"
            );
        }
        outcome
    }

    fn proposal() -> ModelProposal {
        ModelProposal {
            document_type: Some("Statement of Work".into()),
            document_date: Some("2026-04-01".into()),
            date_role: Some(DateRole::Effective),
            parties: vec!["Acme Corporation".into(), "Contoso Worldwide, Inc.".into()],
            party_relation: PartyRelation::Between,
            description:
                "Statement of work between Acme Corporation and Contoso Worldwide, Inc. covering the 2026 CRM implementation and its fees."
                    .into(),
            confidence: 0.9,
            needs_review: false,
            evidence: Evidence {
                date: Some("effective as of April 1, 2026".into()),
                document_type: Some("STATEMENT OF WORK".into()),
                parties: vec![
                    "by and between Acme Corporation and Contoso Worldwide, Inc.".into(),
                ],
            },
            facts: None,
        }
    }

    const DOCUMENT: &str = "STATEMENT OF WORK\n\nThis Statement of Work is effective as of April 1, 2026, \
by and between Acme Corporation and Contoso Worldwide, Inc.\n\nThe work covers the 2026 CRM implementation, \
its deliverables, and its fees.\n";

    #[test]
    fn a_fully_evidenced_proposal_is_ready() {
        let outcome = validate(proposal(), &digest_of(DOCUMENT));
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01")
        );
        assert_eq!(outcome.proposal.parties.len(), 2);
    }

    const REFERENCING_DOCUMENT: &str = "STATEMENT OF WORK

Issued under the Master Services Agreement dated June 2, 2023

Capitalized terms have the meanings given to them in the Master Services
Agreement dated June 2, 2023 between the same parties.

This Statement of Work is effective as of April 1, 2026 and continues
by and between Acme Corporation and Contoso Worldwide, Inc.
";

    /// Greedy decoding is not hardware-deterministic, and the corpus showed a
    /// run filing this exact shape under the referenced agreement's date. The
    /// guard is code so it cannot wobble.
    #[test]
    fn a_date_stated_only_beside_another_agreements_name_is_replaced_by_the_effective_date() {
        let mut candidate = proposal();
        candidate.document_date = Some("2023-06-02".into());
        candidate.evidence.date =
            Some("Issued under the Master Services Agreement dated June 2, 2023".into());
        let outcome = validate(candidate, &digest_of(REFERENCING_DOCUMENT));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01"),
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Effective));
        assert_eq!(
            outcome.proposal.evidence.date.as_deref(),
            Some("This Statement of Work is effective as of April 1, 2026 and continues"),
            "the evidence must be the line the substituted date actually stands on"
        );
        assert!(!outcome.reasons.contains(&ReviewReason::DateUnsupported));
    }

    #[test]
    fn a_referenced_date_with_two_effective_candidates_goes_to_review_not_to_a_guess() {
        let document = REFERENCING_DOCUMENT.replace(
            "and continues",
            "and continues
with services commencing on October 1, 2026",
        );
        let mut candidate = proposal();
        candidate.document_date = Some("2023-06-02".into());
        let outcome = validate(candidate, &digest_of(&document));
        assert!(outcome.proposal.document_date.is_none());
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));
    }

    /// A PDF breaks the referencing phrase wherever the margin falls, and
    /// the corpus's termination notice does exactly that: "... Northstar
    /// Lantern Works LLC dated" ends one line and "March 3, 2024" opens the
    /// next. Read as raw lines the second statement looks unintroduced, the
    /// whole date stops counting as tainted, and the trap date the corpus
    /// forbids becomes a filename.
    #[test]
    fn a_referenced_date_wrapped_onto_the_next_line_is_still_a_reference() {
        let document = "NOTICE OF TERMINATION

Re: Termination of the Employment Agreement

This letter constitutes formal notice under Section 9.2 of the
Employment Agreement between you and Northstar Lantern Works LLC dated
March 3, 2024 (the \"Employment Agreement\") that the Company is
terminating the Employment Agreement without cause.

Your employment with the Company will end effective January 31, 2027.
";
        let mut candidate = proposal();
        candidate.document_type = Some("Notice of Termination".into());
        candidate.document_date = Some("2024-03-03".into());
        candidate.parties = vec!["Northstar Lantern Works LLC".into()];
        candidate.party_relation = PartyRelation::From;
        let outcome = validate(candidate, &digest_of(document));
        assert_ne!(
            outcome.proposal.document_date.as_deref(),
            Some("2024-03-03"),
            "the terminated agreement's date must never date the notice"
        );
    }

    /// A cover page that names the document and dates it in one breath -
    /// "SERVICES AGREEMENT dated as of March 1, 2026" - is the document
    /// dating itself, but the referencing guard sees a document noun before
    /// "dated" and throws the only date the document has away.
    #[test]
    fn a_title_line_dated_as_of_is_the_documents_own_date() {
        let document = "SERVICES AGREEMENT dated as of March 1, 2026

by and between Acme Corporation and Contoso Worldwide, Inc.

The work covers the 2026 CRM implementation, its deliverables, and its fees.
";
        let mut candidate = proposal();
        candidate.document_type = Some("Services Agreement".into());
        candidate.document_date = Some("2026-03-01".into());
        let outcome = validate(candidate, &digest_of(document));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-01"),
            "{:?}",
            outcome.reasons
        );
    }

    /// One line, two dates, two roles: the referenced contract's and the
    /// amendment's own. The first guard version tainted whole lines and threw
    /// away the amendment's real date; this pins the per-occurrence judgment.
    #[test]
    fn an_amendments_own_date_survives_sharing_a_line_with_the_referenced_contracts() {
        let document = "FIRST AMENDMENT

This First Amendment to the Consulting Agreement dated September 1, 2020,
is entered into as of September 14, 2025 by Acme Corporation and
Contoso Worldwide, Inc. The work covers the 2026 CRM implementation.
";
        let mut candidate = proposal();
        candidate.document_date = Some("2025-09-14".into());
        candidate.date_role = Some(DateRole::Amendment);
        candidate.evidence.date = Some("is entered into as of September 14, 2025".into());
        let outcome = validate(candidate, &digest_of(document));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-09-14"),
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Amendment));

        // And the mirror image: choosing the referenced contract's date is
        // redirected to the amendment's own.
        let mut candidate = proposal();
        candidate.document_date = Some("2020-09-01".into());
        let outcome = validate(candidate, &digest_of(document));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-09-14"),
            "{:?}",
            outcome.reasons
        );
    }

    /// mixed-signature.pdf dates itself with a bare title-block line, and the
    /// first per-occurrence guard read its "Dated" as a reference and threw
    /// the date away.
    #[test]
    fn a_bare_title_block_dated_line_is_the_documents_own_date() {
        let document = "SERVICES AGREEMENT
Dated January 8, 2025

Acme Corporation and Contoso Worldwide, Inc.

The work covers the 2026 CRM implementation, its deliverables, and fees.
";
        let mut candidate = proposal();
        candidate.document_date = Some("2025-01-08".into());
        candidate.date_role = Some(DateRole::Execution);
        candidate.evidence.date = Some("Dated January 8, 2025".into());
        let outcome = validate(candidate, &digest_of(document));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-01-08"),
            "{:?}",
            outcome.reasons
        );
    }

    #[test]
    fn a_date_the_document_also_states_on_its_own_line_is_left_alone() {
        // The chosen date appears both beside the reference and on a plain
        // line of its own, so nothing here says it belongs to another document.
        let document = format!(
            "{REFERENCING_DOCUMENT}
Countersigned June 2, 2023.
"
        );
        let mut candidate = proposal();
        candidate.document_date = Some("2023-06-02".into());
        candidate.date_role = Some(DateRole::Execution);
        let outcome = validate(candidate, &digest_of(&document));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2023-06-02")
        );
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Execution));
    }

    #[test]
    fn a_date_that_is_not_in_its_quote_is_rejected() {
        let mut candidate = proposal();
        candidate.document_date = Some("2026-05-30".into());
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert!(outcome.proposal.document_date.is_none());
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));
    }

    #[test]
    fn a_reworded_quote_does_not_throw_away_a_date_the_document_really_has() {
        let mut candidate = proposal();
        candidate.evidence.date = Some(
            "This Statement of Work is effective as of April 1, 2026, and the parties agree".into(),
        );
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01")
        );
    }

    #[test]
    fn a_date_the_document_never_states_is_rejected_however_it_is_quoted() {
        let mut candidate = proposal();
        candidate.document_date = Some("2026-04-02".into());
        candidate.evidence.date = Some("effective as of April 2, 2026".into());
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert!(outcome.proposal.document_date.is_none());
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));
    }

    #[test]
    fn an_invented_party_is_dropped_and_flagged() {
        let mut candidate = proposal();
        candidate.parties = vec!["Acme Corporation".into(), "Northwind Traders LLC".into()];
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert_eq!(
            outcome.proposal.parties,
            vec!["Acme Corporation".to_owned()]
        );
        assert!(outcome.reasons.contains(&ReviewReason::PartyUnsupported));
        assert_eq!(outcome.proposal.party_relation, PartyRelation::With);
    }

    #[test]
    fn a_document_type_worded_differently_from_its_quote_is_still_accepted() {
        let document = "FIRST AMENDMENT TO THE CONSULTING AGREEMENT\n\nThis amendment is effective as of April 1, 2026, by and between Acme Corporation and Contoso Worldwide, Inc.\n";
        let mut candidate = proposal();
        candidate.document_type = Some("First Amendment to Consulting Agreement".into());
        candidate.evidence.document_type =
            Some("FIRST AMENDMENT TO THE CONSULTING AGREEMENT".into());
        candidate.description =
            "First amendment to the consulting agreement between Acme Corporation and Contoso Worldwide, Inc. changing its fees.".into();
        let outcome = validate(candidate, &digest_of(document));
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("First Amendment to Consulting Agreement")
        );
    }

    /// An invented type is rejected, and the document's own title stands in
    /// for it so the reviewer is shown the right name rather than "Document".
    #[test]
    fn an_invented_document_type_is_rejected_and_the_title_offered_instead() {
        let mut candidate = proposal();
        candidate.document_type = Some("Settlement Agreement".into());
        candidate.evidence.document_type = Some("STATEMENT OF WORK".into());
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Statement of Work")
        );
        assert!(outcome.reasons.contains(&ReviewReason::TypeUnsupported));
        assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);

        // With no usable title either, the type stays empty.
        let mut candidate = proposal();
        candidate.document_type = Some("Settlement Agreement".into());
        let outcome = validate(
            candidate,
            &digest_of(
                "Some notes.\n\nThis Statement of Work is effective as of April 1, 2026, by and between Acme Corporation and Contoso Worldwide, Inc.\n",
            ),
        );
        assert!(outcome.proposal.document_type.is_none());
        assert!(outcome.reasons.contains(&ReviewReason::TypeUnsupported));
        assert!(!outcome.reasons.contains(&ReviewReason::TypeInferred));
    }

    #[test]
    fn a_description_asserting_an_absent_name_is_flagged() {
        let mut candidate = proposal();
        candidate.description =
            "Statement of work between Acme Corporation and Northwind Traders covering delivery."
                .into();
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert!(
            outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported)
        );
    }

    #[test]
    fn ordinary_prose_in_a_description_does_not_trigger_review() {
        let outcome = validate(proposal(), &digest_of(DOCUMENT));
        assert!(
            !outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported)
        );
    }

    /// The corpus minutes and journal fixtures got no type from the model and
    /// were named "<date> Document". Their titles name what they are.
    #[test]
    fn a_missing_type_is_taken_from_the_title_and_flagged_for_a_person() {
        let document = "# Quarterly Operations Review\n\n**Date:** May 7, 2025\n\nThe Fictional Meridian Committee reviewed inventory, safety, and the next quarterly plan.\n";
        let candidate = ModelProposal {
            document_type: None,
            document_date: Some("2025-05-07".into()),
            date_role: Some(DateRole::Effective),
            parties: Vec::new(),
            party_relation: PartyRelation::None,
            description: "Quarterly operations review minutes covering inventory, safety, and the next quarterly plan for the committee.".into(),
            confidence: 0.85,
            needs_review: false,
            evidence: Evidence {
                date: Some("**Date:** May 7, 2025".into()),
                document_type: None,
                parties: Vec::new(),
            },
            facts: None,
        };
        let outcome = validate(candidate, &digest_of(document));
        assert_eq!(
            outcome.proposal.document_type.as_deref(),
            Some("Quarterly Operations Review")
        );
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        assert!(outcome.reasons.contains(&ReviewReason::TypeInferred));
        assert!(!outcome.reasons.contains(&ReviewReason::TypeMissing));
        // The bare "Date:" label says nothing; a review is something issued.
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Issuance));
    }

    #[test]
    fn the_date_role_follows_the_documents_wording_not_the_models_habit() {
        let document = "INVOICE\n\nAcme Corporation\nInvoice Number: INV-7741\nInvoice Date: January 5, 2026\nPayment Due Date: February 4, 2026\nBill To: Contoso Worldwide, Inc.\nAnnual platform subscription for the 2026 term, $42,000.\n";
        let candidate = ModelProposal {
            document_type: Some("Invoice".into()),
            document_date: Some("2026-01-05".into()),
            // The corpus habit: everything is "effective".
            date_role: Some(DateRole::Effective),
            parties: vec![
                "Contoso Worldwide, Inc.".into(),
                "Acme Corporation".into(),
            ],
            party_relation: PartyRelation::Between,
            description: "Invoice INV-7741 from Acme Corporation to Contoso Worldwide, Inc. for the 2026 annual platform subscription of $42,000.".into(),
            confidence: 0.9,
            needs_review: false,
            evidence: Evidence {
                date: Some("Invoice Date: January 5, 2026".into()),
                document_type: Some("INVOICE".into()),
                parties: vec!["Bill To: Contoso Worldwide, Inc.".into()],
            },
            facts: None,
        };
        let outcome = validate(candidate, &digest_of(document));
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Invoice));
        // Two parties and "between" on an invoice: the bill-to line names the
        // customer, so the issuer stands alone as "from".
        assert_eq!(
            outcome.proposal.parties,
            vec!["Acme Corporation".to_owned()]
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::From);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
    }

    #[test]
    fn a_missing_date_sends_the_item_to_review() {
        let mut candidate = proposal();
        candidate.document_date = None;
        candidate.evidence.date = None;
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        assert!(outcome.reasons.contains(&ReviewReason::DateMissing));
    }

    #[test]
    fn a_company_suffix_period_does_not_truncate_the_description() {
        let outcome = validate(proposal(), &digest_of(DOCUMENT));
        assert!(outcome.proposal.description.contains("CRM implementation"));
    }

    /// The model drops the comma from "Contoso Worldwide, Inc." often enough
    /// that a correct party was reaching review, and the filename lost the
    /// name. Punctuation is typography; the words are still checked.
    #[test]
    fn a_party_that_differs_from_the_document_only_in_punctuation_is_kept() {
        let mut candidate = proposal();
        candidate.parties = vec!["Acme Corporation".into(), "Contoso Worldwide Inc".into()];
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert_eq!(
            outcome.proposal.parties,
            vec![
                "Acme Corporation".to_owned(),
                "Contoso Worldwide Inc".to_owned()
            ]
        );
        assert!(!outcome.reasons.contains(&ReviewReason::PartyUnsupported));
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);

        let mut candidate = proposal();
        candidate.parties = vec!["Contoso Worldwide LLC".into()];
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert!(outcome.proposal.parties.is_empty());
        assert!(outcome.reasons.contains(&ReviewReason::PartyUnsupported));
    }

    /// The filename grammar gives a second name only to "between"; every
    /// other connecting word takes the first party alone. Keeping the
    /// second one on the proposal published a party the name never carries,
    /// and the corpus scored the termination notice's "Northstar Lantern
    /// Works LLC" as a spurious party for exactly that reason.
    #[test]
    fn a_one_sided_relation_keeps_one_party() {
        let mut candidate = proposal();
        candidate.party_relation = PartyRelation::To;
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert_eq!(
            outcome.proposal.parties,
            vec!["Acme Corporation".to_owned()],
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::To);

        // "between" still carries both, and so does an unstated relation,
        // which asserts nothing about either name.
        let outcome = validate(proposal(), &digest_of(DOCUMENT));
        assert_eq!(outcome.proposal.parties.len(), 2);

        let mut candidate = proposal();
        candidate.party_relation = PartyRelation::None;
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert_eq!(outcome.proposal.parties.len(), 2);
    }

    #[test]
    fn a_description_may_reshape_a_fact_it_quotes_but_not_invent_one() {
        let document = format!(
            "{DOCUMENT}\nThe total fee is $248000 payable to Acme.\nWork begins in Ridgeline Cartography's office.\n"
        );
        let mut candidate = proposal();
        candidate.description = "Statement of work between Acme Corporation and Contoso Worldwide, Inc. for Ridgeline-Cartography's 2026 CRM implementation at a fee of $248,000.".into();
        let outcome = validate(candidate, &digest_of(&document));
        assert!(
            !outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported),
            "{:?}",
            outcome.reasons
        );

        let mut candidate = proposal();
        candidate.description = "Statement of work between Acme Corporation and Contoso Worldwide, Inc. for Northwind's 2026 CRM implementation.".into();
        let outcome = validate(candidate, &digest_of(&document));
        assert!(
            outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported)
        );
    }

    /// The corpus invoice reads "An invoice for $1,248.00 from Nimbus
    /// Orchard Supply Co. ..." and the first-sentence cut kept "An invoice
    /// for $1,248." - four words, so the sentence check then called it
    /// invalid and the whole document went to review over a decimal point.
    #[test]
    fn a_decimal_amount_does_not_end_the_description() {
        let document = format!(
            "{DOCUMENT}
The total fee is $248,000.00 payable on delivery.
"
        );
        let mut candidate = proposal();
        candidate.description =
            "Statement of work for Acme Corporation covering the 2026 CRM implementation at a fee of $248,000.00."
                .into();
        let outcome = validate(candidate, &digest_of(&document));
        assert_eq!(
            outcome.proposal.description,
            "Statement of work for Acme Corporation covering the 2026 CRM implementation at a fee of $248,000.00.",
            "{:?}",
            outcome.reasons
        );
        assert!(!outcome.reasons.contains(&ReviewReason::DescriptionInvalid));
    }

    #[test]
    fn an_abbreviation_inside_the_sentence_does_not_end_it() {
        let mut candidate = proposal();
        candidate.description =
            "Statement of work between Acme Corporation and Contoso Worldwide, Inc. covering deliverables (e.g. the 2026 CRM implementation) and fees."
                .into();
        let outcome = validate(candidate, &digest_of(DOCUMENT));
        assert!(
            outcome.proposal.description.ends_with("and fees."),
            "{}",
            outcome.proposal.description
        );
    }

    /// Whether the first statement of `date` on `line` reads as another
    /// document's date.
    fn reference_at(line: &str, date: &str) -> bool {
        let normalized = normalize(line);
        let position = *date_match_positions(date, &normalized)
            .first()
            .expect("the line states the date");
        reference_introduced(&normalized, position)
    }

    /// "This Consulting Agreement dated as of March 1, 2026" is the
    /// agreement dating itself, but only a "this" directly on the noun was
    /// recognised: the adjective in between made the date a reference, and
    /// it was withheld - or, with a commencement line to fall back on,
    /// silently replaced, and the file went out Ready under the wrong date.
    #[test]
    fn this_adjective_agreement_dated_is_the_documents_own_date() {
        let document = "CONSULTING AGREEMENT
This Consulting Agreement dated as of March 1, 2026 is made by and between Acme Corporation and Jane Smith.
The Consultant will provide services commencing April 15, 2026.
";
        let candidate = ModelProposal {
            document_type: Some("Consulting Agreement".into()),
            document_date: Some("2026-03-01".into()),
            date_role: Some(DateRole::Effective),
            parties: vec!["Acme Corporation".into(), "Jane Smith".into()],
            party_relation: PartyRelation::Between,
            description: "Consulting agreement between Acme Corporation and Jane Smith for services commencing April 15, 2026.".into(),
            confidence: 0.9,
            needs_review: false,
            evidence: Evidence {
                date: Some("This Consulting Agreement dated as of March 1, 2026".into()),
                document_type: Some("CONSULTING AGREEMENT".into()),
                parties: Vec::new(),
            },
            facts: None,
        };
        let outcome = validate_at(candidate, &digest_of(document), 2026);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-01"),
            "the commencement date must not replace the agreement's own"
        );
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        assert_eq!(
            outcome.proposal.evidence.date.as_deref(),
            Some("This Consulting Agreement dated as of March 1, 2026")
        );

        // With no other date to fall back on, the date was withheld.
        let mut candidate = proposal();
        candidate.document_type = Some("Lease Agreement".into());
        candidate.document_date = Some("2024-09-01".into());
        candidate.parties = vec!["Finch Properties LLC".into()];
        candidate.party_relation = PartyRelation::With;
        let outcome = validate_at(
            candidate,
            &digest_of(
                "LEASE AGREEMENT\nThis Lease Agreement dated September 1, 2024 is between Finch Properties LLC and Orion Glass Studio Inc.\n",
            ),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2024-09-01")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::DateUnsupported));

        // The determiner nearest the noun still decides, however the
        // sentence opens, and a noun nobody qualified is still a citation.
        assert!(!reference_at(
            "This First Amendment to Consulting Agreement (this \"Amendment\") is dated as of September 14, 2025",
            "2025-09-14"
        ));
        assert!(reference_at(
            "This First Amendment to the Consulting Agreement dated September 1, 2020",
            "2020-09-01"
        ));
        assert!(reference_at(
            "This First Amendment to Consulting Agreement dated September 1, 2020",
            "2020-09-01"
        ));
        assert!(reference_at(
            "Issued under the Master Services Agreement dated June 2, 2023",
            "2023-06-02"
        ));
        assert!(reference_at(
            "Agreement dated June 2, 2023 between the same parties.",
            "2023-06-02"
        ));
        assert!(reference_at(
            "Contoso Worldwide, Inc. Services Agreement dated June 2, 2023",
            "2023-06-02"
        ));
    }

    /// The document nouns were matched as raw substrings, so a contractor
    /// was a contract, a border an order, and a disagreement an agreement -
    /// each one enough to make the document's own date a reference.
    #[test]
    fn contractor_border_and_disagreement_are_not_document_nouns() {
        for line in [
            "entered into by the Contractor and the Client, dated April 1, 2026",
            "Signed at the border, dated April 1, 2026",
            "Settled after a long disagreement, dated April 1, 2026",
            "Work performed by a subcontractor dated April 1, 2026",
            "Work performed by our subcontractors dated April 1, 2026",
            "Signed by both contractors dated April 1, 2026",
            // Nor is "updated" the word "dated".
            "Exhibit B to the Agreement, updated April 1, 2026",
        ] {
            assert!(!reference_at(line, "2026-04-01"), "{line}");
        }
        for line in [
            "under the Contract dated April 1, 2026",
            "the Purchase Order dated April 1, 2026",
            "a Memorandum dated April 1, 2026",
            // Whole words are still nouns in the plural and with "sub" in
            // front: matching the singular exactly let each of these cited
            // dates through as the document's own.
            "It supersedes the prior agreements dated April 1, 2026",
            "This Amendment amends the Loan Agreements dated April 1, 2026",
            "Invoice for Purchase Orders dated April 1, 2026",
            "This Change Order supplements the Change Orders dated April 1, 2026",
            "Issued under the Subcontract dated April 1, 2026",
            "the Board memoranda dated April 1, 2026",
            "under the Loan Agreements (the \"Agreements\") dated April 1, 2026",
        ] {
            assert!(reference_at(line, "2026-04-01"), "{line}");
        }

        // End to end: the cited agreements' date was filed Ready as the
        // amendment's own, with the amendment's real date beside it.
        let mut candidate = proposal();
        candidate.document_type = Some("Amendment".into());
        candidate.document_date = Some("2023-06-02".into());
        candidate.date_role = Some(DateRole::Amendment);
        let outcome = validate_at(
            candidate,
            &digest_of(
                "AMENDMENT\nThis Amendment amends the Loan Agreements dated June 2, 2023 between Acme Corporation and Contoso Worldwide, Inc.\nThis Amendment is effective as of April 1, 2026.\n",
            ),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01"),
            "{:?}",
            outcome.reasons
        );

        let mut candidate = proposal();
        candidate.document_type = Some("Services Schedule".into());
        candidate.document_date = Some("2026-04-01".into());
        let outcome = validate_at(
            candidate,
            &digest_of(
                "SERVICES SCHEDULE\nThis schedule was entered into by the Contractor and the Client, dated April 1, 2026.\n",
            ),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01"),
            "{:?}",
            outcome.reasons
        );
    }

    /// "Pursuant to Section 9.2 of the Employment Agreement, your
    /// employment will terminate effective January 31, 2027" cites the
    /// agreement and then moves on: the date belongs to the termination.
    /// The cue tainted any date within reach, and the notice's only date
    /// was dropped.
    #[test]
    fn a_cue_separated_from_the_date_by_a_comma_does_not_taint_it() {
        assert!(!reference_at(
            "Pursuant to Section 9.2 of the Employment Agreement, your employment will terminate effective January 31, 2027",
            "2027-01-31"
        ));
        assert!(!reference_at(
            "This Amendment, amending the fee schedule, is effective as of September 14, 2025",
            "2025-09-14"
        ));
        // A cue that runs straight into the date still cites it.
        assert!(reference_at(
            "Statement of Work issued under the Master Agreement effective June 2, 2023",
            "2023-06-02"
        ));
        assert!(reference_at(
            "the Lease as amended March 3, 2024",
            "2024-03-03"
        ));
        assert!(reference_at(
            "pursuant to the Master Agreement effective June 2, 2023",
            "2023-06-02"
        ));

        let mut candidate = proposal();
        candidate.document_type = Some("Notice of Termination".into());
        candidate.document_date = Some("2027-01-31".into());
        candidate.parties = vec!["John Smith".into()];
        candidate.party_relation = PartyRelation::To;
        let outcome = validate_at(
            candidate,
            &digest_of(
                "NOTICE OF TERMINATION\nTo: John Smith\nPursuant to Section 9.2 of the Employment Agreement, your employment will terminate effective January 31, 2027.\n",
            ),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2027-01-31"),
            "{:?}",
            outcome.reasons
        );
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Termination));
    }

    /// Drafting sets a cited agreement's date off with commas - "issued
    /// under the Master Services Agreement, effective June 2, 2023, between
    /// ..." - and any comma after the citation used to end it, so the
    /// agreement's date was filed Ready as the document's own. A comma ends
    /// the citation only when a clause of its own follows it.
    #[test]
    fn a_comma_that_only_introduces_the_cited_date_keeps_the_citation() {
        for line in [
            "This Statement of Work is issued under the Master Services Agreement, effective June 2, 2023, between Acme Corporation and Contoso Worldwide, Inc.",
            "made pursuant to the Supply Agreement, effective as of June 2, 2023, between Acme Corporation and Contoso Worldwide, Inc.",
            "It is issued under the Master Services Agreement, as amended, effective June 2, 2023.",
            "issued under the MSA, effective June 2, 2023",
            "issued under the Master Services Agreement, entered into as of June 2, 2023",
            "issued under the Master Services Agreement, as amended and restated from time to time, effective June 2, 2023",
            "issued under the MSA between Acme Corporation and Contoso Worldwide, Inc., effective June 2, 2023",
        ] {
            assert!(reference_at(line, "2023-06-02"), "{line}");
        }
        // A comma a new clause follows still ends the citation, however
        // many commas the sentence has.
        for (line, date) in [
            (
                "Pursuant to Section 9.2 of the Employment Agreement, your employment will terminate, effective January 31, 2027",
                "2027-01-31",
            ),
            (
                "This Statement of Work is issued under the Master Services Agreement, and is effective as of April 1, 2026",
                "2026-04-01",
            ),
        ] {
            assert!(!reference_at(line, date), "{line}");
        }

        let document = "STATEMENT OF WORK
This Statement of Work is issued under the Master Services Agreement, effective June 2, 2023, between Acme Corporation and Contoso Worldwide, Inc.
This Statement of Work is effective as of April 1, 2026.
";
        let mut candidate = proposal();
        candidate.document_date = Some("2023-06-02".into());
        let outcome = validate_at(candidate, &digest_of(document), 2026);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01"),
            "{:?}",
            outcome.reasons
        );
        // With no date of its own to fall back on, the cited date is
        // withheld rather than filed.
        let mut candidate = proposal();
        candidate.document_date = Some("2023-06-02".into());
        let outcome = validate_at(
            candidate,
            &digest_of(
                "STATEMENT OF WORK
This Statement of Work is issued under the Master Services Agreement, effective June 2, 2023, between Acme Corporation and Contoso Worldwide, Inc.
",
            ),
            2026,
        );
        assert_eq!(outcome.proposal.document_date, None);
        assert!(outcome.reasons.contains(&ReviewReason::DateUnsupported));
    }

    /// Only punctuation that ends the clause ends a citation. The point in a
    /// section number, "No." or "Inc.", and the comma before "Inc.", belong
    /// to what is cited; read as the clause ending, each let the cited
    /// agreement's date through as this document's own.
    #[test]
    fn punctuation_inside_a_citation_does_not_end_it() {
        for line in [
            "This Statement of Work is issued pursuant to Section 2.1 of the Master Services Agreement effective June 2, 2023",
            "issued under Master Agreement No. 12 effective June 2, 2023",
            "issued under the Master Agreement with Acme Corp. effective June 2, 2023",
            "issued under the MSA between Acme Corporation and Contoso Worldwide, Inc. effective June 2, 2023",
            // Read a character at a time, not a byte at a time.
            "issued under the Master Agreement with Société Générale effective June 2, 2023",
        ] {
            assert!(reference_at(line, "2023-06-02"), "{line}");
        }
        // A full stop that ends the sentence still ends the citation.
        assert!(!reference_at(
            "Pursuant to Section 9.2 of the Employment Agreement. Your employment will terminate effective January 31, 2027",
            "2027-01-31"
        ));
    }

    /// The article of a defined term belongs to the definition: `This
    /// Consulting Agreement (the "Agreement") dated as of` is the agreement
    /// dating itself, and the "the" in the parenthesis made it a citation -
    /// withheld, or replaced by the commencement date and filed Ready. The
    /// name the term stands for decides.
    #[test]
    fn a_defined_term_reads_as_the_name_it_defines() {
        assert!(!reference_at(
            "This Consulting Agreement (the \"Agreement\") dated as of March 1, 2026",
            "2026-03-01"
        ));
        assert!(reference_at(
            "under the Master Services Agreement (the \"Agreement\") dated June 2, 2023",
            "2023-06-02"
        ));
        assert!(reference_at(
            "issued under that certain Services Agreement (the \"Agreement\") dated June 2, 2023",
            "2023-06-02"
        ));
        // A parenthesis that defines no document's name decides nothing.
        assert!(reference_at(
            "Acme Corporation (the \"Agreement\") dated June 2, 2023",
            "2023-06-02"
        ));

        let document = "CONSULTING AGREEMENT
This Consulting Agreement (the \"Agreement\") dated as of March 1, 2026 is made by and between Acme Corporation and Jane Smith.
The Consultant will provide services commencing April 15, 2026.
";
        let mut candidate = proposal();
        candidate.document_type = Some("Consulting Agreement".into());
        candidate.document_date = Some("2026-03-01".into());
        candidate.parties = vec!["Acme Corporation".into(), "Jane Smith".into()];
        candidate.description =
            "Consulting agreement between Acme Corporation and Jane Smith for consulting services."
                .into();
        let outcome = validate_at(candidate, &digest_of(document), 2026);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-01")
        );
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
    }

    fn described(description: &str, document: &str) -> ValidationOutcome {
        let mut candidate = proposal();
        candidate.description = description.into();
        validate_at(candidate, &digest_of(document), 2026)
    }

    /// The first-sentence cut ended descriptions at "P.C.", "N.A.", "U.K."
    /// and "Jan.": the stored description lost its second half, or kept so
    /// few words that a good proposal went to review.
    #[test]
    fn dotted_initialisms_and_month_abbreviations_do_not_end_the_description() {
        let document = "ENGAGEMENT LETTER
Contoso Worldwide, Inc. engages Smith & Jones, P.C. for litigation support in the 2026 contract dispute.
Contoso Bank, N.A. lends to Acme Corporation under a revolving credit facility.
Invoice from Acme U.K. Ltd for consulting services delivered in March 2026.
Invoice Date: January 5, 2026
";
        for description in [
            "Engagement letter between Contoso Worldwide, Inc. and Smith & Jones, P.C. for litigation support in the 2026 contract dispute.",
            "Loan agreement between Contoso Bank, N.A. and Acme Corporation for a revolving credit facility in 2026.",
            "Invoice from Acme U.K. Ltd to Contoso for consulting services delivered in March 2026.",
            "Invoice dated Jan. 5, 2026 from Acme Corporation to Contoso for consulting services.",
            "Agreement between Acme L.L.C. and Contoso Worldwide, Inc. for consulting services in 2026.",
        ] {
            let outcome = described(description, document);
            assert_eq!(outcome.proposal.description, description);
            assert!(
                !outcome.reasons.contains(&ReviewReason::DescriptionInvalid),
                "{description}: {:?}",
                outcome.reasons
            );
        }

        // A sentence that really ends after an abbreviation still ends
        // there: what follows is the model carrying on.
        let outcome = described(
            "Invoice from Acme U.K. Ltd for consulting services delivered to Contoso Ltd. The invoice is payable on receipt.",
            document,
        );
        assert_eq!(
            outcome.proposal.description,
            "Invoice from Acme U.K. Ltd for consulting services delivered to Contoso Ltd."
        );
        // And an ordinary full stop is untouched by any of it.
        let outcome = described(
            "Invoice from Acme Corporation for consulting services delivered in March 2026. It is payable on receipt.",
            document,
        );
        assert_eq!(
            outcome.proposal.description,
            "Invoice from Acme Corporation for consulting services delivered in March 2026."
        );
    }

    /// A letterhead in capitals and a body in mixed case are one company;
    /// listing both named it twice in the filename.
    #[test]
    fn parties_differing_only_in_case_and_punctuation_are_one_party() {
        let document = "SERVICES AGREEMENT
ACME CORP
This Services Agreement is effective as of April 1, 2026, by and between Acme Corp. and its customers.
";
        let mut candidate = proposal();
        candidate.document_type = Some("Services Agreement".into());
        candidate.parties = vec!["ACME CORP".into(), "Acme Corp.".into()];
        candidate.party_relation = PartyRelation::Between;
        let outcome = validate_at(candidate, &digest_of(document), 2026);
        assert_eq!(outcome.proposal.parties, vec!["ACME CORP".to_owned()]);
        assert_eq!(outcome.proposal.party_relation, PartyRelation::With);
        assert!(!outcome.reasons.contains(&ReviewReason::PartyUnsupported));
    }

    /// "&" and "and" are one word typed two ways; a name the model wrote
    /// with the other one was rejected as absent.
    #[test]
    fn ampersand_matches_and() {
        let document = "SETTLEMENT AGREEMENT
This Settlement Agreement is made as of July 22, 2026 between Harborline Freight Systems LLC and Quill and Vane Advisory Group, Inc.
Counsel: Smith & Jones LLP.
";
        let mut candidate = proposal();
        candidate.document_type = Some("Settlement Agreement".into());
        candidate.document_date = Some("2026-07-22".into());
        candidate.parties = vec![
            "Quill & Vane Advisory Group".into(),
            "Smith and Jones LLP".into(),
        ];
        let outcome = validate_at(candidate, &digest_of(document), 2026);
        assert_eq!(
            outcome.proposal.parties,
            vec![
                "Quill & Vane Advisory Group".to_owned(),
                "Smith and Jones LLP".to_owned()
            ]
        );
        assert!(!outcome.reasons.contains(&ReviewReason::PartyUnsupported));
        // Only a standalone "&" is a word; inside a name it is the name.
        assert_eq!(normalize_loosely("AT&T Corp."), "at&t corp");
        assert_eq!(normalize_loosely("Smith & Jones"), "smith and jones");
    }

    /// Two names where one contains the other are an affiliate agreement's
    /// two sides, not one party spelled twice.
    #[test]
    fn affiliates_are_not_merged() {
        let document = "INTERCOMPANY AGREEMENT
This Intercompany Agreement is effective as of April 1, 2026 between Acme and Acme Holdings.
";
        let mut candidate = proposal();
        candidate.document_type = Some("Intercompany Agreement".into());
        candidate.parties = vec!["Acme".into(), "Acme Holdings".into()];
        let outcome = validate_at(candidate, &digest_of(document), 2026);
        assert_eq!(
            outcome.proposal.parties,
            vec!["Acme".to_owned(), "Acme Holdings".to_owned()]
        );
        assert_eq!(outcome.proposal.party_relation, PartyRelation::Between);
    }

    fn invoice(date: &str) -> ModelProposal {
        ModelProposal {
            document_type: Some("Invoice".into()),
            document_date: Some(date.into()),
            date_role: Some(DateRole::Invoice),
            parties: vec!["Acme Corporation".into()],
            party_relation: PartyRelation::From,
            description:
                "Invoice from Acme Corporation to Contoso Worldwide, Inc. for consulting services."
                    .into(),
            confidence: 0.9,
            needs_review: false,
            evidence: Evidence::default(),
            facts: None,
        }
    }

    fn invoice_document(date_lines: &str) -> String {
        format!(
            "INVOICE INV-2048\nAcme Corporation\n{date_lines}\nBill To: Contoso Worldwide, Inc.\nConsulting services.\n"
        )
    }

    /// A scan reads 2025 as 2925, the model copies the year faithfully, and
    /// every literal check passes: the file was named and filed into a 2925
    /// folder. The date is kept, because it may be right, and a person looks.
    #[test]
    fn an_implausible_year_is_kept_but_reviewed() {
        for (printed, date) in [
            ("March 3, 2925", "2925-03-03"),
            ("March 3, 1850", "1850-03-03"),
            ("March 3, 2037", "2037-03-03"),
            ("March 3, 1899", "1899-03-03"),
        ] {
            let outcome = validate_at(
                invoice(date),
                &digest_of(&invoice_document(&format!("Invoice Date: {printed}"))),
                2026,
            );
            assert_eq!(outcome.proposal.document_date.as_deref(), Some(date));
            assert_eq!(
                outcome.reasons,
                vec![ReviewReason::DateImplausible],
                "{printed}"
            );
            assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        }
        // A term that starts a few years out, or a document from the last
        // century, is an ordinary date.
        for (printed, date) in [
            ("March 3, 2031", "2031-03-03"),
            ("March 3, 2036", "2036-03-03"),
            ("March 3, 1900", "1900-03-03"),
        ] {
            let outcome = validate_at(
                invoice(date),
                &digest_of(&invoice_document(&format!("Invoice Date: {printed}"))),
                2026,
            );
            assert_eq!(
                outcome.status,
                ProposalStatus::Ready,
                "{printed}: {:?}",
                outcome.reasons
            );
        }
        // `validate` judges against the clock, which is well short of 2915.
        let outcome = validate(
            invoice("2925-03-03"),
            &digest_of(&invoice_document("Invoice Date: March 3, 2925")),
        );
        assert!(outcome.reasons.contains(&ReviewReason::DateImplausible));
    }

    /// The year `validate` judges against comes from a calendar conversion
    /// of its own; its edges are the leap day and the turn of a year.
    #[test]
    fn the_current_year_is_read_from_the_calendar() {
        assert_eq!(civil_year_from_days(0), 1970);
        assert_eq!(civil_year_from_days(-1), 1969);
        // 2000-02-29, 2000-12-31, 2001-01-01.
        assert_eq!(civil_year_from_days(11_016), 2000);
        assert_eq!(civil_year_from_days(11_322), 2000);
        assert_eq!(civil_year_from_days(11_323), 2001);
        // 2023-12-31, 2024-01-01.
        assert_eq!(civil_year_from_days(19_722), 2023);
        assert_eq!(civil_year_from_days(19_723), 2024);
        assert!((2026..2200).contains(&current_year()));
    }

    /// The model picked the due date of an invoice that prints both dates
    /// on one line, and every check passed: the date is in the document,
    /// and the role inference read the invoice date's label onto it. The
    /// document's one issue date takes its place; with no single issue date
    /// the choice is withheld for a person, the model's date still offered.
    #[test]
    fn a_due_date_labelled_as_the_invoice_date_is_caught() {
        let both = invoice_document("Invoice Date: 04/30/2025    Due Date: 05/30/2025");
        let outcome = validate_at(invoice("2025-05-30"), &digest_of(&both), 2026);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-04-30")
        );
        assert_eq!(outcome.proposal.date_role, Some(DateRole::Invoice));
        assert_eq!(outcome.reasons, vec![ReviewReason::DateIsDeadline]);
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        assert_eq!(
            outcome.proposal.evidence.date.as_deref(),
            Some("Invoice Date: 04/30/2025    Due Date: 05/30/2025")
        );
        assert_eq!(
            outcome.candidate.document_date.as_deref(),
            Some("2025-05-30")
        );

        // The same with written dates on lines of their own, and with a
        // lease's renewal date standing in for the due date.
        let outcome = validate_at(
            invoice("2025-05-30"),
            &digest_of(&invoice_document(
                "Invoice Date: April 30, 2025\nPayment due: May 30, 2025",
            )),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-04-30")
        );
        let mut lease = proposal();
        lease.document_type = Some("Lease Agreement".into());
        lease.document_date = Some("2027-03-01".into());
        let outcome = validate_at(
            lease,
            &digest_of(
                "LEASE AGREEMENT\nThis Lease is dated March 1, 2026 between Acme Corporation and Contoso Worldwide, Inc.\nRenewal Date: March 1, 2027\n",
            ),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-01")
        );
        assert!(outcome.reasons.contains(&ReviewReason::DateIsDeadline));

        // No issue date, two of them, or one a numeric token that reads
        // either way round in a document that never says which: withheld,
        // and only the deadline is the reason.
        for (date_lines, due) in [
            ("Due Date: 05/30/2025", "2025-05-30"),
            (
                "Invoice Date: 04/30/2025\nStatement Date: May 1, 2025\nDue Date: 05/30/2025",
                "2025-05-30",
            ),
            (
                "Invoice Date: 04/05/2025\nDue Date: 05/06/2025",
                "2025-05-06",
            ),
        ] {
            let outcome = validate_at(
                invoice(due),
                &digest_of(&invoice_document(date_lines)),
                2026,
            );
            assert_eq!(outcome.proposal.document_date, None, "{date_lines}");
            assert_eq!(outcome.proposal.date_role, None);
            assert_eq!(
                outcome.reasons,
                vec![ReviewReason::DateIsDeadline],
                "{date_lines}"
            );
            assert_eq!(outcome.candidate.document_date.as_deref(), Some(due));
        }

        // A due date only month-first can read settles the document's
        // order, and the issue date is read that way round, as the date
        // chips read it: one issue date, not two.
        for (date_lines, due, issued) in [
            (
                "Invoice Date: 04/05/2025\nDue Date: 05/30/2025",
                "2025-05-30",
                "2025-04-05",
            ),
            (
                "Invoice Date: 03/04/2026\nDue Date: 04/30/2026",
                "2026-04-30",
                "2026-03-04",
            ),
            (
                "Invoice Date: 04/03/2026\nDue Date: 30/04/2026",
                "2026-04-30",
                "2026-03-04",
            ),
        ] {
            let outcome = validate_at(
                invoice(due),
                &digest_of(&invoice_document(date_lines)),
                2026,
            );
            assert_eq!(
                outcome.proposal.document_date.as_deref(),
                Some(issued),
                "{date_lines}"
            );
            assert_eq!(
                outcome.reasons,
                vec![ReviewReason::DateIsDeadline],
                "{date_lines}"
            );
        }

        // The invoice date itself, chosen correctly, is left alone, and so
        // is a due date the document also states without the label.
        let outcome = validate_at(invoice("2025-04-30"), &digest_of(&both), 2026);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-04-30")
        );
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        let outcome = validate_at(
            invoice("2025-05-30"),
            &digest_of(&invoice_document(
                "Due Date: 05/30/2025\nServices delivered 05/30/2025",
            )),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-05-30")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::DateIsDeadline));
    }

    /// The deadline check can take a date away from a filename, so only an
    /// explicit deadline label sets it off: "payable" and "return" label
    /// dates that are no deadline, and a renewal mentioned in passing does
    /// not make the date beside it a renewal date.
    #[test]
    fn payable_return_and_renewal_wording_is_not_a_deadline() {
        let outcome = validate_at(
            invoice("2025-03-03"),
            &digest_of(&invoice_document(
                "Payment payable upon receipt. Date: March 3, 2025",
            )),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-03-03")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::DateIsDeadline));

        let mut tax_return = proposal();
        tax_return.document_type = Some("Tax Return".into());
        tax_return.document_date = Some("2025-04-15".into());
        tax_return.parties = vec!["Jane Smith".into()];
        tax_return.party_relation = PartyRelation::For;
        let outcome = validate_at(
            tax_return,
            &digest_of(
                "INDIVIDUAL INCOME TAX RETURN\nTaxpayer: Jane Smith\nTax Return Date: April 15, 2025\n",
            ),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2025-04-15")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::DateIsDeadline));

        let mut lease = proposal();
        lease.document_type = Some("Lease Agreement".into());
        lease.document_date = Some("2026-03-01".into());
        let outcome = validate_at(
            lease,
            &digest_of(
                "LEASE AGREEMENT\nThis Lease, including any renewal, commences on March 1, 2026 between Acme Corporation and Contoso Worldwide, Inc.\nRenewal: the Tenant may renew by notice before March 1, 2027.\n",
            ),
            2026,
        );
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-03-01")
        );
        assert!(!outcome.reasons.contains(&ReviewReason::DateIsDeadline));
    }

    /// "Ship Date:" holds the label "Date:", and a shipping date was offered
    /// in place of the due date as the invoice's own - one click from being
    /// filed under it. Only a "Date:" nothing qualifies is the issue date.
    #[test]
    fn a_qualified_date_label_is_not_the_issue_date() {
        for date_lines in [
            "Ship Date: April 2, 2025\nDue Date: May 30, 2025",
            "Order Date: April 2, 2025\nDue Date: May 30, 2025",
            "Service Date: 04/02/2025\nDate of Invoice 04/30/2025\nDue Date: 05/30/2025",
        ] {
            let outcome = validate_at(
                invoice("2025-05-30"),
                &digest_of(&invoice_document(date_lines)),
                2026,
            );
            assert_eq!(outcome.proposal.document_date, None, "{date_lines}");
            assert_eq!(
                outcome.reasons,
                vec![ReviewReason::DateIsDeadline],
                "{date_lines}"
            );
        }
        // A bare label, or one with only a number before it, still names
        // the issue date.
        for date_lines in [
            "Date: April 2, 2025\nDue Date: May 30, 2025",
            "Invoice No. 1042 Date: April 2, 2025\nDue Date: May 30, 2025",
        ] {
            let outcome = validate_at(
                invoice("2025-05-30"),
                &digest_of(&invoice_document(date_lines)),
                2026,
            );
            assert_eq!(
                outcome.proposal.document_date.as_deref(),
                Some("2025-04-02"),
                "{date_lines}"
            );
            assert_eq!(outcome.reasons, vec![ReviewReason::DateIsDeadline]);
        }
    }

    /// "updated" holds the letters of "dated", and a footer's "Rates updated
    /// January 1, 2024" stood as the invoice's issue date, ready to replace
    /// the due date with a date that is not the invoice's either.
    #[test]
    fn an_update_is_not_the_issue_date() {
        for footer in [
            "Rates updated January 1, 2024",
            "Status update: January 1, 2024",
        ] {
            let outcome = validate_at(
                invoice("2025-05-30"),
                &digest_of(&invoice_document(&format!(
                    "Due Date: 05/30/2025\n{footer}"
                ))),
                2026,
            );
            assert_eq!(outcome.proposal.document_date, None, "{footer}");
            assert_eq!(outcome.reasons, vec![ReviewReason::DateIsDeadline]);
        }
    }

    /// "04/01/2026" is 1 April in London and 4 January in New York, and
    /// both readings are dates the document prints. Unless the document says
    /// which, the model's reading is a guess, kept but reviewed.
    #[test]
    fn a_numeric_date_that_reads_either_way_round_is_reviewed() {
        // A US invoice with nothing else to go on: either reading is a guess.
        for date in ["2026-04-01", "2026-01-04"] {
            let outcome = validate_at(
                invoice(date),
                &digest_of(&invoice_document("Invoice Date: 04/01/2026")),
                2026,
            );
            assert_eq!(outcome.proposal.document_date.as_deref(), Some(date));
            assert_eq!(outcome.reasons, vec![ReviewReason::DateAmbiguous], "{date}");
            assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        }

        // A UK invoice whose delivery date can only be read day first: the
        // invoice date read the same way is the document's, and read the
        // other way round it contradicts the document.
        let uk = digest_of(&invoice_document(
            "Invoice Date: 04/01/2026\nDelivered: 30/01/2026",
        ));
        let outcome = validate_at(invoice("2026-01-04"), &uk, 2026);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        let outcome = validate_at(invoice("2026-04-01"), &uk, 2026);
        assert_eq!(outcome.reasons, vec![ReviewReason::DateAmbiguous]);
        assert_eq!(
            outcome.proposal.document_date.as_deref(),
            Some("2026-04-01")
        );

        // The two-digit-year shape of the same invoice, which used to accept
        // only the US misreading.
        let short = digest_of(&invoice_document(
            "Invoice Date: 01/04/26\nDelivered: 30/04/26",
        ));
        let outcome = validate_at(invoice("2026-04-01"), &short, 2026);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        let outcome = validate_at(invoice("2026-01-04"), &short, 2026);
        assert_eq!(outcome.reasons, vec![ReviewReason::DateAmbiguous]);

        // The same date in words, or year first, anywhere in the document
        // settles it; so does a day above 12 or a day equal to its month.
        for date_lines in [
            "Invoice Date: 04/01/2026 (April 1, 2026)",
            "Invoice Date: 04/01/2026\nIssued: 2026-04-01",
            "Invoice Date: 2026-04-01",
        ] {
            let outcome = validate_at(
                invoice("2026-04-01"),
                &digest_of(&invoice_document(date_lines)),
                2026,
            );
            assert_eq!(
                outcome.status,
                ProposalStatus::Ready,
                "{date_lines}: {:?}",
                outcome.reasons
            );
        }
        for (printed, date) in [("04/30/2026", "2026-04-30"), ("04/04/2026", "2026-04-04")] {
            let outcome = validate_at(
                invoice(date),
                &digest_of(&invoice_document(&format!("Invoice Date: {printed}"))),
                2026,
            );
            assert_eq!(
                outcome.status,
                ProposalStatus::Ready,
                "{printed}: {:?}",
                outcome.reasons
            );
        }
    }

    /// A description that writes a date in words for a document that prints
    /// it in numbers states the same fact, but read word by word "January"
    /// was a name the document never writes.
    #[test]
    fn a_description_restating_a_numeric_date_in_words_is_supported() {
        let document = invoice_document("Invoice Date: 01/05/2026");
        for description in [
            "Invoice from Acme Corporation to Contoso Worldwide, Inc. dated January 5, 2026 for consulting services.",
            "Invoice from Acme Corporation to Contoso Worldwide, Inc. dated Jan. 5, 2026 for consulting services.",
            "Invoice from Acme Corporation dated 5 January 2026 for consulting services to Contoso.",
            "Invoice from Acme Corporation dated 2026-01-05 for consulting services to Contoso.",
        ] {
            let mut candidate = invoice("2026-01-05");
            candidate.description = description.into();
            let outcome = validate_at(candidate, &digest_of(&document), 2026);
            assert!(
                !outcome
                    .reasons
                    .contains(&ReviewReason::DescriptionUnsupported),
                "{description}: {:?}",
                outcome.reasons
            );
        }
        // A date the document does not state is still a claim to check.
        let mut candidate = invoice("2026-01-05");
        candidate.description =
            "Invoice from Acme Corporation to Contoso Worldwide, Inc. dated January 6, 2026 for consulting services.".into();
        let outcome = validate_at(candidate, &digest_of(&document), 2026);
        assert!(
            outcome
                .reasons
                .contains(&ReviewReason::DescriptionUnsupported)
        );
    }
}
