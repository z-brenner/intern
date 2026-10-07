//! Inferences the document's own wording supports, made after the model has
//! answered and the answer has been checked.
//!
//! Three of them, each replacing a model answer the corpus showed to be weak
//! with something read straight off the page:
//!
//! * **The date role.** The model picks the right date and then calls it
//!   `effective` whatever the document says - the corpus scored six roles
//!   right out of thirteen. The line the chosen date stands on usually says
//!   what kind of date it is ("Invoice Date:", "Date of this Notice:",
//!   "effective as of", "Signed on"), and that wording is more reliable than
//!   the model's label.
//! * **A title-derived document type.** A document with a title has a type,
//!   and the model still occasionally answers none. When the first heading
//!   names a kind of document - minutes, a journal, a receipt - it is used,
//!   and the proposal goes to review saying so.
//! * **The issuer of an invoice.** Asked for one party, the model sometimes
//!   answers two with `between`, leading with whoever was billed. A "Bill To"
//!   line settles which one issued it.
//!
//! Nothing here invents text: a role comes from a line that states the date,
//! a type from a heading, an issuer from a party the model already named.

use crate::cues::{CUSTOMER_CUES, ISSUER_CUES, NOT_A_TITLE, TYPE_NOUNS};
use crate::domain::{DateRole, PartyRelation};
use crate::evidence::{
    NumericDate, NumericOrder, Segments, date_match_positions, extract_stated_dates,
    is_valid_iso_date, normalize, normalize_loosely, numeric_dates,
};
use crate::validate::rfind_word;

/// Roles in the order one wins when a line carries several cues: the more
/// specific reading first, so "notice of termination dated" reads as a notice
/// and "will end effective" as a termination rather than an effective date.
const ROLE_PRIORITY: [DateRole; 8] = [
    DateRole::Invoice,
    DateRole::Notice,
    DateRole::Termination,
    DateRole::Amendment,
    DateRole::Filing,
    DateRole::Issuance,
    DateRole::Effective,
    DateRole::Execution,
];

const TERMINATION_CUES: &[&str] = &["terminat", "will end", "ends on", "separation date"];
const INVOICE_CUES: &[&str] = &[
    "invoice date",
    "date of invoice",
    "invoice dated",
    "invoiced on",
    "bill date",
    "billing date",
    "statement date",
];
const NOTICE_CUES: &[&str] = &[
    "date of this notice",
    "notice date",
    "date of notice",
    // A notice of termination dated a day is a notice issued that day; the
    // termination it brings about is a later date. Matched here, ahead of
    // the termination cues, so "terminat" does not claim it.
    "notice of termination dated",
    "notice of termination as of",
    "notice dated",
    "notice is given",
    "notice given",
    "notice is hereby given",
    "date of this letter",
    "letter date",
];
const AMENDMENT_CUES: &[&str] = &[
    "amendment date",
    "amended as of",
    "amendment is dated",
    "amendment is effective",
    "amendment effective",
    "amendment is made",
    "amendment made as of",
];
const FILING_CUES: &[&str] = &[
    "filed on",
    "filing date",
    "date filed",
    "date of filing",
    "recorded on",
    "recording date",
];
const ISSUANCE_CUES: &[&str] = &[
    "issued on",
    "issue date",
    "date of issue",
    "date issued",
    "issuance date",
    "journal date",
    "report date",
    "date of report",
    "order date",
    "po date",
    "ship date",
    "shipped on",
    "date shipped",
    "delivery date",
    "delivered on",
    "receipt date",
    "prepared on",
    "presented on",
    "published on",
    "publication date",
    "meeting date",
    "meeting held on",
    "held on",
    "date of meeting",
    "minutes of",
];
const EFFECTIVE_CUES: &[&str] = &[
    "effective",
    "commencement",
    "commencing",
    "commence",
    "start date",
    "starts on",
    "begins on",
    "beginning on",
    "in force",
    "entered into as of",
    "made as of",
    "made and entered into",
    "dated as of",
    "term begins",
];
const EXECUTION_CUES: &[&str] = &[
    "signed on",
    "signed this",
    "signed as of",
    "executed on",
    "executed this",
    "executed as of",
    "date signed",
    "signature date",
    "duly executed",
    "witness whereof",
];
/// A label that says "this is the date" without saying what kind.
const GENERIC_DATE_CUES: &[&str] = &["date:", "dated", "date of this"];
/// Words that qualify "Date" as some other event's date. "Due Date: May 30,
/// 2025" labels when the money is owed, not when the invoice was written,
/// and a document type's default role must not be read onto it.
const OTHER_DATE_LABELS: &[&str] = &[
    "due",
    "expiration",
    "expiry",
    "expires",
    "renewal",
    "end",
    "return",
    "deadline",
    "payable",
];
/// The labels that make a date a deadline - when money is owed, when
/// something lapses or renews - and so never the date a document was issued.
/// Narrower than [`OTHER_DATE_LABELS`] on purpose: that list only keeps a
/// type's default role off a date, while this one can take a date away from
/// the filename, so "payable" ("Payment payable upon receipt") and "return"
/// (a tax return's own date) are left out.
const DEADLINE_LABELS: &[&str] = &[
    "due",
    "expires",
    "expiration",
    "expiry",
    "renewal",
    "deadline",
];
/// Words that sit between a label and its date without changing what the
/// label says: "Due Date:", "due on or before", "Expires on".
const LABEL_FILLERS: &[&str] = &["date", "on", "or", "before", "by", "of"];
/// Labels that name a date as the day the document itself was issued.
const ISSUE_LABELS: &[&str] = &[
    "invoice date",
    "date of issue",
    "issue date",
    "statement date",
    "dated",
];

/// How far before a date the wording that names its role can sit. Long
/// enough for "This First Amendment to Consulting Agreement (this
/// "Amendment") is dated as of", short enough that a cue from an earlier
/// sentence does not leak in.
const CUE_WINDOW: usize = 96;

/// What kind of date the chosen one is, read from the lines that state it.
///
/// Every non-reference statement of the date is examined; when several lines
/// state it in different roles the most specific wins, so a date that is both
/// "effective as of" and "signed on" reads as effective, the way the prompt
/// ranks them. A line that merely labels the date ("Date:", "Dated") falls
/// back to what the document type implies. `None` when the wording says
/// nothing, in which case the model's own answer stands.
pub fn infer_date_role(
    digest: &impl Segments,
    date: &str,
    document_type: Option<&str>,
) -> Option<DateRole> {
    let mut found: Vec<DateRole> = Vec::new();
    let mut generic = false;
    for segment in digest.segments() {
        for line in wrapped_lines(segment) {
            let normalized = normalize(&line);
            for position in date_match_positions(date, &normalized) {
                if crate::validate::reference_introduced(&normalized, position) {
                    continue;
                }
                match role_from_wording(&window_before(&normalized, position)) {
                    Some(role) => {
                        if !found.contains(&role) {
                            found.push(role);
                        }
                    }
                    None => {
                        if is_generic_label(&window_before(&normalized, position)) {
                            generic = true;
                        }
                    }
                }
            }
        }
    }
    let by_wording = ROLE_PRIORITY
        .iter()
        .copied()
        .find(|role| found.contains(role));
    let kind = TypeKind::of(document_type);
    match (by_wording, kind) {
        // An amendment's own date is its amendment date whatever verb the
        // sentence used to state it.
        (Some(DateRole::Effective | DateRole::Execution) | None, TypeKind::Amendment)
            if by_wording.is_some() || generic =>
        {
            Some(DateRole::Amendment)
        }
        (Some(role), _) => Some(role),
        (None, kind) if generic => kind.default_role(),
        (None, _) => None,
    }
}

/// A segment's lines with wrapped sentences rejoined. A PDF breaks "is
/// dated" from "as of September 14, 2025" wherever the margin falls, and the
/// wording that names a date's role must be read across that break. A line
/// is joined to the one before it only when the one before did not end a
/// sentence, a label, or a clause, and it itself carries a sentence on - it
/// starts in lower case, with a number, or with a month - so a header
/// block's "To:" and "From:" lines stay apart, and "Date of this Notice:
/// December 29, 2026" does not lend its cue to the sentence under it.
pub(crate) fn wrapped_lines(segment: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in segment.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match lines.last_mut() {
            Some(previous)
                if !previous.ends_with(['.', ':', ';', '!', '?']) && continues_sentence(line) =>
            {
                previous.push(' ');
                previous.push_str(line);
            }
            _ => lines.push(line.to_owned()),
        }
    }
    lines
}

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

fn continues_sentence(line: &str) -> bool {
    let Some(first) = line.chars().next() else {
        return false;
    };
    if first.is_lowercase() || first.is_ascii_digit() {
        return true;
    }
    let first_word = line
        .split(|character: char| !character.is_alphabetic())
        .next()
        .unwrap_or_default()
        .to_lowercase();
    MONTHS.iter().any(|month| {
        *month == first_word || (first_word.len() >= 3 && month.starts_with(&first_word))
    })
}

/// Whether the wording before a date merely labels it as the date: "Date:",
/// "Dated", "Date of this ...", or a bare "DATE" the way a stamped form
/// writes it. The cues are read as whole words: "Last updated:" and "Status
/// update:" contain the letters of "dated" and "date:" without labelling
/// anything as the date.
fn is_generic_label(window: &str) -> bool {
    if labels_another_date(window) {
        return false;
    }
    GENERIC_DATE_CUES
        .iter()
        .any(|cue| rfind_word(window, cue).is_some())
        || window
            .trim_end()
            .rsplit(|character: char| !character.is_alphanumeric())
            .next()
            .is_some_and(|word| word == "date")
}

/// Whether the label nearest the date qualifies it as another event's date -
/// "Due Date", "Expiration Date", "Return Date". The qualifier sits directly
/// on the word, so only the word immediately before it is read.
fn labels_another_date(window: &str) -> bool {
    let Some(at) = window.rfind("date") else {
        return false;
    };
    let before = window[..at].trim_end();
    OTHER_DATE_LABELS.iter().any(|word| before.ends_with(word))
}

/// Whether the label nearest the date makes it a deadline: "Due Date:",
/// "Payment due", "Expires on", "Renewal Date". Only the label itself is
/// read - the last word before the date once fillers like "date" and "on"
/// are stepped over - so "This Lease, including any renewal, commences on"
/// is a commencement, not a renewal date.
pub(crate) fn labels_a_deadline(window: &str) -> bool {
    window
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .rev()
        .find(|word| !LABEL_FILLERS.contains(word))
        .is_some_and(|word| DEADLINE_LABELS.contains(&word))
}

/// Whether the wording before a date names it as the day the document was
/// issued: "Invoice Date:", "Date of issue", "Dated", or a bare "Date:".
/// Whole words only, because the date found here can replace the model's: a
/// footer's "Rates updated January 1, 2024" is no issue date.
///
/// "Date:" is bare only when no word qualifies it. "Ship Date:", "Order
/// Date:" and "Service Date:" label some other event, and offering the
/// shipping date as the invoice's would file it under the wrong day at one
/// click.
pub(crate) fn labels_the_issue_date(window: &str) -> bool {
    !labels_another_date(window)
        && (ISSUE_LABELS
            .iter()
            .any(|label| rfind_word(window, label).is_some())
            || (is_generic_label(window) && !date_is_qualified(window)))
}

/// Whether a word stands directly before the last "date" in `window`,
/// saying whose date it is: "ship date", "order date". A number before it -
/// "Invoice No. 1042 Date:" - qualifies nothing.
fn date_is_qualified(window: &str) -> bool {
    rfind_word(window, "date").is_some_and(|at| {
        window[..at]
            .trim_end()
            .chars()
            .next_back()
            .is_some_and(char::is_alphabetic)
    })
}

/// The wording a date's role is read from: what stands before it on its
/// line, back to the previous date or the end of the previous sentence, and
/// never more than [`CUE_WINDOW`] bytes.
pub(crate) fn window_before(normalized: &str, position: usize) -> String {
    // A fixed distance back can land inside a multi-byte character - é, §,
    // •, °, the fraction slash NFKC makes of ½ - and slicing there panicked
    // inside the model thread, which paused the whole queue. Walking forward
    // to a boundary only shortens the window, and `position` is itself a
    // boundary, so the walk stops in time. Every later move is forward too.
    let mut start = position.saturating_sub(CUE_WINDOW);
    while !normalized.is_char_boundary(start) {
        start += 1;
    }
    // A label governs the date that follows it and stops there. An invoice
    // prints "Invoice Date: April 30, 2025    Due Date: May 30, 2025" on one
    // line, and reading a fixed distance back from the second date reached
    // the first date's label. A finished sentence is a boundary for the same
    // reason: whatever it said was about its own date.
    for other in extract_stated_dates(normalized) {
        for found in date_match_positions(&other, normalized) {
            if found < position && found > start {
                start = found;
            }
        }
    }
    // Written dates are found above; a numeric one is not, because
    // extraction leaves numeric dates alone, and "Invoice Date: 04/30/2025
    // Due Date: 05/30/2025" lent the invoice date's label to the due date.
    // Any date-shaped token bounds the window - nothing is read from it as a
    // date, so its shape is all that matters.
    if let Some(end) = last_numeric_date_end(&normalized[..position]) {
        start = start.max(end);
    }
    if let Some(stop) = normalized[start..position].rfind(". ") {
        start += stop + ". ".len();
    }
    normalized[start..position].to_owned()
}

/// Where the last numeric-date-shaped token in `window` ends.
fn last_numeric_date_end(window: &str) -> Option<usize> {
    numeric_dates(window).iter().map(|date| date.end).max()
}

/// The calendar dates a numeric token can mean. A year-first token is read
/// one way. A year-last token is read month-first and day-first, and both
/// readings are kept when both are real dates, unless `order` - the order
/// the document's own numeric dates settle ([`numeric_date_order`]) - says
/// which one it writes. Without that, choosing between them would be a
/// guess. A two-digit year is not read at all, for the same reason - its
/// century is a guess.
///
/// [`numeric_date_order`]: crate::evidence::numeric_date_order
fn numeric_readings(date: &NumericDate<'_>, order: Option<NumericOrder>) -> Vec<String> {
    let [first, second, third] = date.parts;
    let candidates = if first.len() == 4 {
        vec![(first, second, third)]
    } else if third.len() == 4 {
        match order {
            Some(NumericOrder::MonthFirst) => vec![(third, first, second)],
            Some(NumericOrder::DayFirst) => vec![(third, second, first)],
            None => vec![(third, first, second), (third, second, first)],
        }
    } else {
        Vec::new()
    };
    let mut readings: Vec<String> = Vec::new();
    for (year, month, day) in candidates {
        let iso = format!("{year}-{month:0>2}-{day:0>2}");
        if is_valid_iso_date(&iso) && !readings.contains(&iso) {
            readings.push(iso);
        }
    }
    readings
}

/// Every date a normalized line states, each with the byte offset it stands
/// at: written months and ISO forms the way [`extract_stated_dates`] reads
/// them, and numeric tokens in every reading they allow - in the document's
/// `order` alone when it has settled one, as the date chips read them. A
/// reading counts only where the evidence check finds that date at that
/// token too, so a date taken from here always passes it.
pub(crate) fn dates_stated_on(
    normalized: &str,
    order: Option<NumericOrder>,
) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    for date in extract_stated_dates(normalized) {
        for position in date_match_positions(&date, normalized) {
            found.push((date.clone(), position));
        }
    }
    for token in numeric_dates(normalized) {
        for reading in numeric_readings(&token, order) {
            if date_match_positions(&reading, normalized).contains(&token.start) {
                found.push((reading, token.start));
            }
        }
    }
    found
}

pub(crate) fn role_from_wording(window: &str) -> Option<DateRole> {
    let has = |cues: &[&str]| cues.iter().any(|cue| window.contains(cue));
    if has(INVOICE_CUES) {
        return Some(DateRole::Invoice);
    }
    if has(NOTICE_CUES) {
        return Some(DateRole::Notice);
    }
    if has(TERMINATION_CUES) {
        return Some(DateRole::Termination);
    }
    if has(AMENDMENT_CUES) {
        return Some(DateRole::Amendment);
    }
    if has(FILING_CUES) {
        return Some(DateRole::Filing);
    }
    if has(ISSUANCE_CUES) {
        return Some(DateRole::Issuance);
    }
    if has(EFFECTIVE_CUES) {
        return Some(DateRole::Effective);
    }
    if has(EXECUTION_CUES) {
        return Some(DateRole::Execution);
    }
    None
}

/// The broad family a document type belongs to, for the defaults a bare
/// "Date:" label falls back to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TypeKind {
    Amendment,
    Notice,
    Invoice,
    /// Orders, slips, receipts, reports, minutes, journals, memos,
    /// certificates: things that are issued or written out on a date.
    Issued,
    /// Agreements and their relatives: things that take effect on a date.
    Agreement,
    Unknown,
}

impl TypeKind {
    fn of(document_type: Option<&str>) -> Self {
        let Some(document_type) = document_type else {
            return Self::Unknown;
        };
        let lowered = document_type.to_lowercase();
        let has = |words: &[&str]| words.iter().any(|word| lowered.contains(word));
        // The agreement family is tested first because its names contain
        // the other families' words: a statement *of work* is an agreement,
        // not a statement, and the entry for it was unreachable while
        // "statement" was tested first.
        if has(&["amendment", "addendum", "modification"]) {
            Self::Amendment
        } else if has(&["notice", "notification"]) {
            Self::Notice
        } else if has(&["invoice", "bill", "statement of account", "credit note"]) {
            Self::Invoice
        } else if has(&[
            "agreement",
            "contract",
            "lease",
            "statement of work",
            "terms",
            "policy",
            "license",
            "licence",
            "deed",
            "warranty",
            "guarantee",
            "waiver",
            "release",
            "consent",
        ]) {
            Self::Agreement
        } else if has(&[
            "order",
            "slip",
            "receipt",
            "report",
            "minutes",
            "journal",
            "memo",
            "certificate",
            "quote",
            "quotation",
            "estimate",
            "email",
            "e-mail",
            "letter",
            "log",
            "review",
            "summary",
            "agenda",
            "plan",
            "statement",
        ]) {
            Self::Issued
        } else {
            Self::Unknown
        }
    }

    fn default_role(self) -> Option<DateRole> {
        match self {
            Self::Amendment => Some(DateRole::Amendment),
            Self::Notice => Some(DateRole::Notice),
            Self::Invoice => Some(DateRole::Invoice),
            Self::Issued => Some(DateRole::Issuance),
            Self::Agreement => Some(DateRole::Effective),
            Self::Unknown => None,
        }
    }
}

/// The document's own title completing the model's type. "Journal" on a
/// document headed "Moonlit Archive Project Journal" is that title, whole:
/// the prompt asks for the document's words for its type, and the title is
/// where they are. Completed only when the title ends with the type and adds
/// a few plain words to it - not an exhibit label, not a party's name, not a
/// leftover "No." from a stripped number - so a completion is always the
/// document's title and never a guess.
pub fn complete_type_from_title(
    document_type: &str,
    digest: &impl Segments,
    parties: &[String],
) -> String {
    let Some(title) = infer_document_type(digest) else {
        return document_type.to_owned();
    };
    let title_words = normalize(&title)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let type_words = normalize(document_type)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if type_words.is_empty()
        || title_words.len() <= type_words.len()
        || !title_words.ends_with(&type_words)
    {
        return document_type.to_owned();
    }
    let extra = &title_words[..title_words.len() - type_words.len()];
    let party_words = parties
        .iter()
        .flat_map(|party| {
            normalize(party)
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let plain = extra.len() <= 3
        && extra.iter().all(|word| {
            word.chars().count() > 1
                && word.chars().all(char::is_alphabetic)
                && !NOT_A_TITLE.contains(&word.as_str())
                && !party_words.contains(word)
        });
    if plain {
        title
    } else {
        document_type.to_owned()
    }
}

/// Words kept lowercase inside a title-cased heading.
const SMALL_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "by", "for", "from", "in", "of", "on", "or", "the", "to", "with",
];

/// Short all-capital words that stay capitals in a title: initialisms a
/// filing clerk would never write as "Nda".
const INITIALISMS: &[&str] = &[
    "nda", "sow", "po", "wo", "msa", "loi", "mou", "rfp", "rfq", "sla", "ip", "hr", "it", "llc",
    "inc", "ltd", "plc", "gmbh", "lp", "llp", "usa", "uk", "eu",
];

/// The document type its title states, when the model gave none.
///
/// Only the first heading or two are considered - a title sits at the top -
/// and only a short one that names a kind of document. Identifiers, page
/// fragments, and markdown marks are trimmed away, so "DELIVERY RECEIPT
/// DR-771" becomes "Delivery Receipt" and "MOONLIT ARCHIVE PROJECT JOURNAL -
/// PAGE 1" becomes "Moonlit Archive Project Journal". A heading that names a
/// part ("EXHIBIT A", "CONFIDENTIAL") is skipped, and a document whose first
/// headings name nothing gets no type from here.
pub fn infer_document_type(digest: &impl Segments) -> Option<String> {
    digest
        .headings()
        .iter()
        .take(2)
        .find_map(|heading| title_type(heading))
}

fn title_type(heading: &str) -> Option<String> {
    let cleaned = clean_title(heading);
    let lowered = cleaned.to_lowercase();
    if lowered.is_empty()
        || NOT_A_TITLE
            .iter()
            .any(|part| lowered == *part || lowered.starts_with(&format!("{part} ")))
    {
        return None;
    }
    let words: Vec<&str> = lowered.split_whitespace().collect();
    if words.is_empty() || words.len() > 8 {
        return None;
    }
    let names_a_kind = words.iter().any(|word| {
        let word = word.trim_matches(|character: char| !character.is_alphanumeric());
        TYPE_NOUNS.contains(&word)
    });
    if !names_a_kind {
        return None;
    }
    Some(title_case(&cleaned))
}

/// Strips markdown marks, a trailing page fragment, identifiers, and dangling
/// punctuation from a heading.
fn clean_title(heading: &str) -> String {
    let mut text = heading.trim().trim_start_matches('#').trim().to_owned();
    for marker in [" - page ", " – page ", " — page ", " page "] {
        let found = rfind_ignoring_ascii_case(&text, marker).filter(|index| {
            text[index + marker.len()..]
                .trim()
                .chars()
                .all(|character| character.is_ascii_digit() || character.is_whitespace())
        });
        if let Some(index) = found {
            text.truncate(index);
        }
    }
    let kept: Vec<&str> = text
        .split_whitespace()
        .filter(|token| !token.chars().any(|character| character.is_ascii_digit()))
        .filter(|token| token.chars().any(char::is_alphanumeric))
        .collect();
    kept.join(" ")
        .trim_end_matches([':', '-', '–', '—', ',', ';', '.'])
        .trim()
        .to_owned()
}

/// The last occurrence of `marker`, ignoring the case of its ASCII letters,
/// as a byte offset into `haystack` itself.
///
/// Searching a lowercased copy gives an offset into that copy, and case
/// folding does not preserve byte lengths - "STRAẞE Ü - Page 2" folds one
/// byte shorter, so the offset lands inside a character and truncating there
/// panics. A marker begins and ends on a byte matched exactly, so the offset
/// this returns is always a character boundary.
fn rfind_ignoring_ascii_case(haystack: &str, marker: &str) -> Option<usize> {
    haystack
        .as_bytes()
        .windows(marker.len())
        .rposition(|window| window.eq_ignore_ascii_case(marker.as_bytes()))
}

/// Title case for an all-capitals heading; a mixed-case heading is left as
/// the document wrote it.
fn title_case(value: &str) -> String {
    let all_capitals = value
        .chars()
        .filter(|character| character.is_alphabetic())
        .all(char::is_uppercase);
    if !all_capitals {
        return value.to_owned();
    }
    value
        .split_whitespace()
        .enumerate()
        .map(|(index, word)| {
            let lowered = word.to_lowercase();
            let core = lowered.trim_matches(|character: char| !character.is_alphanumeric());
            if INITIALISMS.contains(&core) {
                word.to_uppercase()
            } else if index > 0 && SMALL_WORDS.contains(&core) {
                lowered
            } else {
                capitalize(&lowered)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

/// Document types that have one party - the one that issued them.
///
/// "statement" on its own is an account statement; it is matched whole,
/// because "Statement of Work" contains the word and is an agreement between
/// two parties, and the corpus showed the second party of one being dropped
/// as if it were a customer on a bill.
const ISSUED_TYPES: &[&str] = &[
    "invoice",
    "receipt",
    "bill",
    "statement of account",
    "account statement",
    "bank statement",
    "billing statement",
    "packing slip",
    "purchase order",
    "work order",
    "sales order",
    "quote",
    "quotation",
    "estimate",
    "delivery",
    "credit note",
    "remittance",
];

/// For an invoice-like document the model gave two parties and "between",
/// the party that issued it, alone, "from". The bill-to, sold-to, or ship-to
/// line names the customer; the other party issued it. Unchanged when the
/// document does not settle the question.
pub fn repair_issued_relation(
    document_type: Option<&str>,
    parties: Vec<String>,
    relation: PartyRelation,
    digest: &impl Segments,
) -> (Vec<String>, PartyRelation) {
    let issued_type = document_type.is_some_and(|value| {
        let lowered = value.to_lowercase();
        lowered.trim() == "statement" || ISSUED_TYPES.iter().any(|kind| lowered.contains(kind))
    });
    if !issued_type || parties.len() != 2 || relation != PartyRelation::Between {
        return (parties, relation);
    }
    let lines: Vec<String> = digest
        .segments()
        .iter()
        .flat_map(|segment| segment.lines())
        .map(normalize_loosely)
        .collect();
    let on_line_with = |party: &str, cues: &[&str]| {
        let party = normalize_loosely(party);
        !party.is_empty()
            && lines
                .iter()
                .any(|line| line.contains(&party) && cues.iter().any(|cue| line.contains(cue)))
    };
    let customers: Vec<bool> = parties
        .iter()
        .map(|party| on_line_with(party, CUSTOMER_CUES))
        .collect();
    let issuers: Vec<bool> = parties
        .iter()
        .map(|party| on_line_with(party, ISSUER_CUES))
        .collect();
    // Every cue nominates an issuer: an issuer line nominates its own party,
    // a customer line nominates the other one. One nominee, however many
    // cues agree on it, settles the question; two nominees is a document
    // that contradicts itself, and the model's answer stands. (An earlier
    // version wanted exactly one cue, and an invoice with both a "Bill To"
    // and a "Remit To" line - the clearest case there is - was left as
    // "between".)
    let mut nominees = Vec::new();
    for index in 0..2 {
        if issuers[index] {
            nominees.push(index);
        }
        if customers[index] {
            nominees.push(1 - index);
        }
    }
    nominees.sort_unstable();
    nominees.dedup();
    let issuer = match nominees.as_slice() {
        [only] => Some(*only),
        _ => None,
    };
    match issuer {
        Some(index) => (vec![parties[index].clone()], PartyRelation::From),
        None => (parties, relation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::DocumentDigest;
    use crate::distill::{DigestBudget, distill, source_from_text};

    fn digest_of(text: &str) -> DocumentDigest {
        distill(&source_from_text(text), DigestBudget::default())
    }

    /// Replay of the recorded corpus: the model read the statement of work as
    /// "between" its two parties, and the issued-document repair, matching
    /// "statement", dropped the client as if it were the bill-to on an
    /// invoice.
    #[test]
    fn a_statement_of_work_is_not_an_issued_document() {
        let digest = digest_of(
            "STATEMENT OF WORK NO. 4\nThis Statement of Work is entered into by and between \
             Ridgeline Cartography LLC (\"Provider\") and Contoso Worldwide, Inc. (\"Client\").",
        );
        let parties = vec![
            "Ridgeline Cartography LLC".to_owned(),
            "Contoso Worldwide, Inc.".to_owned(),
        ];
        let (kept, relation) = repair_issued_relation(
            Some("Statement of Work"),
            parties.clone(),
            PartyRelation::Between,
            &digest,
        );
        assert_eq!(kept, parties);
        assert_eq!(relation, PartyRelation::Between);

        let statement = digest_of("STATEMENT\nCustomer: Acme Corporation\nRemit to: First Bank");
        let (kept, relation) = repair_issued_relation(
            Some("Statement"),
            vec!["First Bank".to_owned(), "Acme Corporation".to_owned()],
            PartyRelation::Between,
            &statement,
        );
        assert_eq!(kept, vec!["First Bank"]);
        assert_eq!(relation, PartyRelation::From);
    }

    /// Replay of the recorded corpus: the vendor invoice names its customer
    /// on a "Bill To" line and itself on a "Remit To" line, and the repair
    /// refused to choose because two cues spoke instead of one.
    #[test]
    fn cues_that_agree_on_the_issuer_settle_it() {
        let digest = digest_of(
            "INVOICE\nAcme Corporation, 500 Foundry Road\nBill To: Contoso Worldwide, Inc., Accounts Payable\n\
             Remit To: Acme Corporation, Account 4471-9920",
        );
        let (kept, relation) = repair_issued_relation(
            Some("Invoice"),
            vec![
                "Acme Corporation".to_owned(),
                "Contoso Worldwide, Inc.".to_owned(),
            ],
            PartyRelation::Between,
            &digest,
        );
        assert_eq!(kept, vec!["Acme Corporation"]);
        assert_eq!(relation, PartyRelation::From);

        // Cues that name both parties as the issuer leave the model's answer.
        let contradictory =
            digest_of("INVOICE\nBill To: Acme Corporation\nBill To: Contoso Worldwide, Inc.");
        let (kept, _) = repair_issued_relation(
            Some("Invoice"),
            vec![
                "Acme Corporation".to_owned(),
                "Contoso Worldwide, Inc.".to_owned(),
            ],
            PartyRelation::Between,
            &contradictory,
        );
        assert_eq!(kept.len(), 2);
    }

    /// "Due Date:" is a label, but it is not a label for *this* document's
    /// date, and reading it as one made the invoice's due date into an
    /// invoice date. The document says what kind of date it is not, so the
    /// wording says nothing and the model's own answer stands.
    #[test]
    fn a_due_date_label_never_becomes_an_invoice_date() {
        let digest = digest_of(
            "INVOICE INV-2048
Invoice Date: April 30, 2025
Due Date: May 30, 2025",
        );
        assert_eq!(
            infer_date_role(&digest, "2025-05-30", Some("Invoice")),
            None,
            "a due date is not the invoice's date"
        );
        assert_eq!(
            infer_date_role(&digest, "2025-04-30", Some("Invoice")),
            Some(DateRole::Invoice)
        );
    }

    /// An invoice prints both of its dates on one line. Reading a fixed
    /// distance back from the second one reaches the first one's label, and
    /// the due date came back labelled as the invoice date.
    #[test]
    fn a_label_governs_only_the_date_that_follows_it() {
        let digest = digest_of(
            "INVOICE INV-2048
Invoice Date: April 30, 2025    Due Date: May 30, 2025",
        );
        assert_eq!(
            infer_date_role(&digest, "2025-05-30", Some("Invoice")),
            None,
            "the invoice date's label stops at the invoice date"
        );
        assert_eq!(
            infer_date_role(&digest, "2025-04-30", Some("Invoice")),
            Some(DateRole::Invoice)
        );
    }

    /// The numeric twin of the test above. Extraction leaves numeric dates
    /// alone, so they never bounded the window, and the due date read the
    /// invoice date's label.
    #[test]
    fn a_numeric_label_governs_only_the_date_that_follows_it() {
        let digest = digest_of(
            "INVOICE INV-2048
Invoice Date: 04/30/2025    Due Date: 05/30/2025",
        );
        assert_eq!(
            infer_date_role(&digest, "2025-05-30", Some("Invoice")),
            None,
            "the invoice date's label stops at the invoice date"
        );
        assert_eq!(
            infer_date_role(&digest, "2025-04-30", Some("Invoice")),
            Some(DateRole::Invoice)
        );
        // Day-first and ISO spellings bound it the same way.
        for line in [
            "Invoice Date: 30.04.2025    Due Date: 30.05.2025",
            "Invoice Date: 2025-04-30    Due Date: 2025-05-30",
        ] {
            let digest = digest_of(&format!("INVOICE INV-2048\n{line}"));
            assert_eq!(
                infer_date_role(&digest, "2025-05-30", Some("Invoice")),
                None,
                "{line}"
            );
        }
    }

    /// The shape is all that bounds a window, so it is pinned exactly: a
    /// longer number is not a date, and neither is a two-part one.
    #[test]
    fn only_a_whole_date_shaped_token_bounds_the_window() {
        assert_eq!(last_numeric_date_end("date: 04/30/2025 due"), Some(16));
        assert_eq!(last_numeric_date_end("4/30/25 then 2025-04-30 x"), Some(23));
        assert_eq!(last_numeric_date_end("30.04.2025"), Some(10));
        for not_a_date in [
            "account 12345/1/2026",
            "ref 04/30/20255",
            "04/30-2025",
            "page 4/30",
            "section 9.2.1",
            "104/300/2025",
        ] {
            assert_eq!(last_numeric_date_end(not_a_date), None, "{not_a_date}");
        }
        // Only an unambiguous reading of a numeric token is a single date;
        // a token both readings fit stays two, and a two-digit year none.
        let readings = |text: &str, order: Option<NumericOrder>| -> Vec<String> {
            numeric_dates(text)
                .iter()
                .flat_map(|token| numeric_readings(token, order))
                .collect()
        };
        assert_eq!(readings("04/30/2025", None), vec!["2025-04-30"]);
        assert_eq!(readings("30.04.2025", None), vec!["2025-04-30"]);
        assert_eq!(readings("2025-04-30", None), vec!["2025-04-30"]);
        assert_eq!(
            readings("04/05/2025", None),
            vec!["2025-04-05", "2025-05-04"]
        );
        assert!(readings("04/30/25", None).is_empty());
        // The order the document settles reads a token both ways fit one
        // way round; a year-first token is read one way whatever it is.
        assert_eq!(
            readings("04/05/2025", Some(NumericOrder::MonthFirst)),
            vec!["2025-04-05"]
        );
        assert_eq!(
            readings("04/05/2025", Some(NumericOrder::DayFirst)),
            vec!["2025-05-04"]
        );
        assert_eq!(
            readings("2025-04-05", Some(NumericOrder::DayFirst)),
            vec!["2025-04-05"]
        );
    }

    /// "updated" and "update:" hold the letters of "dated" and "date:", and
    /// read as a bare date label they lent the type's default role to a date
    /// that only says when something last changed.
    #[test]
    fn an_update_is_not_a_date_label() {
        for line in [
            "Prices last updated April 1, 2026",
            "Status update: April 1, 2026",
        ] {
            let digest = digest_of(&format!("INVOICE INV-2048\n{line}"));
            assert_eq!(
                infer_date_role(&digest, "2026-04-01", Some("Invoice")),
                None,
                "{line}"
            );
        }
        let digest = digest_of("INVOICE INV-2048\nDate: April 1, 2026");
        assert_eq!(
            infer_date_role(&digest, "2026-04-01", Some("Invoice")),
            Some(DateRole::Invoice)
        );
    }

    /// A notice dated a day was given that day, whatever it terminates; the
    /// role list says so, and the termination cue used to claim it first.
    #[test]
    fn notice_of_termination_dated_reads_as_a_notice() {
        let digest = digest_of("NOTICE OF TERMINATION dated December 29, 2026");
        assert_eq!(
            infer_date_role(&digest, "2026-12-29", Some("Notice of Termination")),
            Some(DateRole::Notice)
        );
        let digest = digest_of(
            "NOTICE OF TERMINATION\nThis notice of termination as of December 29, 2026 ends the lease.",
        );
        assert_eq!(
            infer_date_role(&digest, "2026-12-29", Some("Notice of Termination")),
            Some(DateRole::Notice)
        );
        // The date the termination takes effect is still a termination date.
        let digest = digest_of(
            "NOTICE OF TERMINATION dated December 29, 2026\nYour employment will end effective January 31, 2027.",
        );
        assert_eq!(
            infer_date_role(&digest, "2027-01-31", Some("Notice of Termination")),
            Some(DateRole::Termination)
        );
    }

    /// Only the label nearest the date makes it a deadline, and only an
    /// explicit one: "payable" and "return" label other dates without
    /// making them deadlines, and a renewal mentioned in passing is not a
    /// renewal date.
    #[test]
    fn only_an_explicit_label_on_the_date_makes_it_a_deadline() {
        for deadline in [
            "due date: ",
            "payment due date: ",
            "payment due: ",
            "due on or before ",
            "expires on ",
            "expiration date ",
            "expiry: ",
            "renewal date: ",
            "deadline: ",
        ] {
            assert!(labels_a_deadline(deadline), "{deadline}");
        }
        for not_a_deadline in [
            "invoice date: ",
            "date: ",
            "payment payable upon receipt date: ",
            "amount payable date: ",
            "tax return date: ",
            "this lease, including any renewal, commences on ",
            "the due diligence report is dated ",
            "",
        ] {
            assert!(!labels_a_deadline(not_a_deadline), "{not_a_deadline}");
        }
    }

    /// The cue window starts a fixed number of bytes before the date, and
    /// that offset landed inside é, §, •, ° or the fraction slash NFKC makes
    /// of ½ often enough to panic on every French invoice: the panic took
    /// down the model thread and paused the whole queue. Each padding moves
    /// the window's start one byte further through the multi-byte
    /// characters, so every offset inside one of them is reached.
    #[test]
    fn a_multibyte_character_inside_the_cue_window_does_not_panic() {
        let run = |character: &str, count: usize| character.repeat(count);
        let templates = [
            "La présente convention conclue entre la Société Générale et Acme Corporation \
             {pad}prend effet le 01/04/2026"
                .to_owned(),
            format!(
                "Article {} 4 {} Durée {} {} {{pad}}prend effet le 01/04/2026",
                run("§", 12),
                run("§", 9),
                run("§", 9),
                run("§", 9)
            ),
            format!(
                "{} Acme Corporation {} Contoso {{pad}}effective 01/04/2026",
                run("•", 20),
                run("•", 20)
            ),
            format!(
                "Stored at 4{} Juniper Loop {} {{pad}}effective 01/04/2026",
                run("°", 25),
                run("°", 25)
            ),
            format!(
                "Interest {} per month {} {{pad}}effective 01/04/2026",
                run("½", 20),
                run("½", 20)
            ),
        ];
        for template in &templates {
            for pad in 0..60 {
                let line = template.replace("{pad}", &format!("{} ", "x".repeat(pad)));
                let digest = digest_of(&format!("CONVENTION\n{line}"));
                // Reaching the assertion at all is most of the test.
                let _ = infer_date_role(&digest, "2026-04-01", Some("Convention"));
                let outcome = crate::validate::validate(
                    crate::domain::ModelProposal {
                        document_type: None,
                        document_date: Some("2026-04-01".into()),
                        date_role: None,
                        parties: Vec::new(),
                        party_relation: PartyRelation::None,
                        description: String::new(),
                        confidence: 0.9,
                        needs_review: false,
                        evidence: crate::domain::Evidence::default(),
                        facts: None,
                    },
                    &digest,
                );
                assert_eq!(
                    outcome.proposal.document_date.as_deref(),
                    Some("2026-04-01"),
                    "{line}"
                );
            }
        }
    }

    /// Replay of the recorded corpus: the board deck is dated "Presented on
    /// May 21, 2026", which is a deck being issued on a date, and the model
    /// called it a notice date.
    #[test]
    fn a_deck_presented_on_a_date_was_issued_on_it() {
        let digest = digest_of("QUARTERLY BUSINESS REVIEW\n\nPresented on May 21, 2026");
        assert_eq!(
            infer_date_role(&digest, "2026-05-21", Some("Quarterly Business Review")),
            Some(DateRole::Issuance)
        );
    }

    /// A statement of work is an agreement, not a statement, and the list
    /// says so - but "statement" was tested first, so the entry was never
    /// reached and a bare "Date:" read as an issuance date.
    #[test]
    fn a_bare_date_on_a_statement_of_work_is_its_effective_date() {
        let digest = digest_of(
            "STATEMENT OF WORK NO. 4
Date: April 1, 2026",
        );
        assert_eq!(
            infer_date_role(&digest, "2026-04-01", Some("Statement of Work")),
            Some(DateRole::Effective)
        );
        // An account statement is still something issued on a date.
        let digest = digest_of(
            "ACCOUNT STATEMENT
Date: April 1, 2026",
        );
        assert_eq!(
            infer_date_role(&digest, "2026-04-01", Some("Account Statement")),
            Some(DateRole::Issuance)
        );
    }

    /// Case folding does not preserve byte lengths - ẞ folds to ß and İ
    /// folds to two characters - so a page marker found in a lowercased copy
    /// of the heading is at the wrong offset in the heading itself, and
    /// truncating there lands inside a character and panics.
    #[test]
    fn a_title_with_length_changing_case_folding_does_not_panic() {
        assert_eq!(clean_title("STRAẞE Ü - Page 2"), "STRAẞE Ü");
        assert_eq!(
            clean_title("İSTANBUL WORKS AGREEMENT - Page 2"),
            "İSTANBUL WORKS AGREEMENT"
        );
    }

    /// Replay of the recorded corpus: the amendment's PDF wraps "is dated"
    /// and "as of September 14, 2025" onto different lines, and the role was
    /// read from the second line alone.
    #[test]
    fn the_role_is_read_across_a_wrapped_line() {
        let digest = digest_of(
            "FIRST AMENDMENT TO CONSULTING AGREEMENT\nThis First Amendment to Consulting Agreement (this \"Amendment\") is dated\n\
             as of September 14, 2025, and amends the Consulting Agreement dated\nJanuary 12, 2023.",
        );
        assert_eq!(
            infer_date_role(
                &digest,
                "2025-09-14",
                Some("First Amendment to Consulting Agreement")
            ),
            Some(DateRole::Amendment)
        );
        // A line that ends a sentence, or one that starts a new one, is not
        // joined to its neighbour: the notice date's label keeps its cue.
        assert_eq!(
            wrapped_lines(
                "Date of this Notice: December 29, 2026\nYour employment will end effective\nJanuary 31, 2027.\nTo: John Smith\nFrom: Harriet Voss"
            ),
            vec![
                "Date of this Notice: December 29, 2026".to_owned(),
                "Your employment will end effective January 31, 2027.".to_owned(),
                "To: John Smith".to_owned(),
                "From: Harriet Voss".to_owned(),
            ]
        );
    }

    /// Replay of the recorded corpus: the 100-page journal is headed
    /// "MOONLIT ARCHIVE PROJECT JOURNAL" and the model said "Journal".
    #[test]
    fn the_title_completes_a_type_it_ends_with_and_nothing_else() {
        let journal =
            digest_of("MOONLIT ARCHIVE PROJECT JOURNAL - PAGE 1\nJournal date: July 1, 2025");
        assert_eq!(
            complete_type_from_title("Journal", &journal, &[]),
            "Moonlit Archive Project Journal"
        );
        assert_eq!(
            complete_type_from_title("Project Journal", &journal, &[]),
            "Moonlit Archive Project Journal"
        );
        assert_eq!(
            complete_type_from_title("Minutes", &journal, &[]),
            "Minutes"
        );

        // A stripped number leaves "No", which is not part of any type.
        let numbered = digest_of("STATEMENT OF WORK NO. 4\nEffective April 1, 2026.");
        assert_eq!(
            complete_type_from_title("Statement of Work", &numbered, &[]),
            "Statement of Work"
        );
        // An exhibit label is not a type, and neither is a party's name.
        let exhibit = digest_of("EXHIBIT A STATEMENT OF WORK\nEffective April 1, 2026.");
        assert_eq!(
            complete_type_from_title("Statement of Work", &exhibit, &[]),
            "Statement of Work"
        );
        let branded = digest_of("ACME CORPORATION INVOICE\nInvoice date: May 1, 2025");
        assert_eq!(
            complete_type_from_title("Invoice", &branded, &["Acme Corporation".to_owned()]),
            "Invoice"
        );
    }

    /// Each line is the shape one corpus fixture states its date in, with the
    /// role the corpus says is right.
    #[test]
    fn the_role_is_read_from_the_line_the_date_stands_on() {
        let cases: &[(&str, &str, Option<&str>, DateRole)] = &[
            (
                "INVOICE\nInvoice Number: INV-7741\nInvoice Date: January 5, 2026\nPayment Due Date: February 4, 2026",
                "2026-01-05",
                Some("Invoice"),
                DateRole::Invoice,
            ),
            (
                "NOTICE OF TERMINATION\nDate of this Notice: December 29, 2026\nYour employment will end effective January 31, 2027.",
                "2026-12-29",
                Some("Notice of Termination"),
                DateRole::Notice,
            ),
            (
                "NOTICE OF TERMINATION\nDate of this Notice: December 29, 2026\nYour employment with the Company will end effective January 31, 2027 (the \"Separation Date\").",
                "2027-01-31",
                Some("Notice of Termination"),
                DateRole::Termination,
            ),
            (
                "FIRST AMENDMENT TO CONSULTING AGREEMENT\nThis First Amendment to Consulting Agreement (this \"Amendment\") is dated as of September 14, 2025, and amends the Consulting Agreement dated January 12, 2023.",
                "2025-09-14",
                Some("First Amendment to Consulting Agreement"),
                DateRole::Amendment,
            ),
            (
                "STATEMENT OF WORK\n4.1 Effective Date.\nThis Statement of Work is effective as of April 1, 2026 and continues through March 31, 2027.\nExecuted on April 9, 2026.",
                "2026-04-01",
                Some("Statement of Work"),
                DateRole::Effective,
            ),
            (
                "STATEMENT OF WORK\nThis Statement of Work is effective as of April 1, 2026.\nExecuted on April 9, 2026.",
                "2026-04-09",
                Some("Statement of Work"),
                DateRole::Execution,
            ),
            (
                "ORDER FORM\nSubscription Start Date | February 1, 2026\nSigned on January 14, 2026 by authorized representatives.",
                "2026-02-01",
                Some("Order Form"),
                DateRole::Effective,
            ),
            (
                "MOONLIT ARCHIVE PROJECT JOURNAL - PAGE 1\nJournal date: July 1, 2025\nFictional observation 001",
                "2025-07-01",
                Some("Project Journal"),
                DateRole::Issuance,
            ),
            (
                "CERTIFICATE OF GOOD STANDING\nFiled on March 3, 2026 with the Secretary of State.",
                "2026-03-03",
                Some("Certificate of Good Standing"),
                DateRole::Filing,
            ),
        ];
        for (text, date, document_type, expected) in cases {
            assert_eq!(
                infer_date_role(&digest_of(text), date, *document_type),
                Some(*expected),
                "{text}"
            );
        }
    }

    /// A bare "Date:" says nothing about the kind of date; the kind of
    /// document does.
    #[test]
    fn a_bare_date_label_falls_back_to_what_the_document_type_implies() {
        let cases: &[(&str, &str, Option<&str>, Option<DateRole>)] = &[
            (
                "PURCHASE ORDER PO-310\nDATE JULY 14 2025\nEMBER POST MANUFACTURING LLC",
                "2025-07-14",
                Some("Purchase Order"),
                Some(DateRole::Issuance),
            ),
            (
                "# Quarterly Operations Review\n\n**Date:** May 7, 2025\n\nThe committee reviewed inventory.",
                "2025-05-07",
                Some("Meeting Minutes"),
                Some(DateRole::Issuance),
            ),
            (
                "SERVICES AGREEMENT\nDated January 8, 2025\n\nAcme Corporation and Contoso Worldwide, Inc.",
                "2025-01-08",
                Some("Services Agreement"),
                Some(DateRole::Effective),
            ),
            (
                "INVOICE\nDate: May 1, 2025\nTotal due: $1,248.00",
                "2025-05-01",
                Some("Invoice"),
                Some(DateRole::Invoice),
            ),
            (
                "NOTICE OF DEFAULT\nDated: May 1, 2025",
                "2025-05-01",
                Some("Notice of Default"),
                Some(DateRole::Notice),
            ),
            // Nothing to go on: the model's answer stands.
            ("SOMETHING\nDate: May 1, 2025", "2025-05-01", None, None),
            (
                "SOMETHING\nWe met on May 1, 2025 and agreed nothing.",
                "2025-05-01",
                Some("Services Agreement"),
                None,
            ),
        ];
        for (text, date, document_type, expected) in cases {
            assert_eq!(
                infer_date_role(&digest_of(text), date, *document_type),
                *expected,
                "{text}"
            );
        }
    }

    #[test]
    fn a_referenced_agreements_date_line_carries_no_role_for_this_document() {
        // The chosen date is stated only beside another agreement's name;
        // that occurrence is a reference, and no role is read from it.
        let digest = digest_of(
            "STATEMENT OF WORK\nIssued under the Master Services Agreement dated June 2, 2023\nThis Statement of Work is effective as of April 1, 2026.",
        );
        assert_eq!(
            infer_date_role(&digest, "2023-06-02", Some("Statement of Work")),
            None
        );
    }

    #[test]
    fn a_title_that_names_a_kind_of_document_becomes_its_type() {
        let cases: &[(&str, Option<&str>)] = &[
            (
                "# Quarterly Operations Review\n\n**Date:** May 7, 2025\n\nThe committee reviewed inventory.",
                Some("Quarterly Operations Review"),
            ),
            (
                "MOONLIT ARCHIVE PROJECT JOURNAL - PAGE 1\nJournal date: July 1, 2025\nFictional observation 001",
                Some("Moonlit Archive Project Journal"),
            ),
            (
                "DELIVERY RECEIPT DR-771\nJUNE 12 2025\nPINE ECHO COURIERS LLC",
                Some("Delivery Receipt"),
            ),
            (
                "PURCHASE ORDER PO-310\nDATE JULY 14 2025",
                Some("Purchase Order"),
            ),
            (
                "FIRST AMENDMENT TO CONSULTING AGREEMENT\nThis amendment is dated as of September 14, 2025.",
                Some("First Amendment to Consulting Agreement"),
            ),
            (
                "MUTUAL NDA\nThis Mutual NDA is effective March 3, 2025.",
                Some("Mutual NDA"),
            ),
            // A heading that names a part of a document, not the document.
            ("EXHIBIT A\nSome text follows here.", None),
            ("CONFIDENTIAL\nNotes from the call.", None),
            // The first heading is a part; the title is the second.
            (
                "CONFIDENTIAL\n\nSETTLEMENT AGREEMENT\n\nThis Settlement Agreement is made as of July 22, 2026.",
                Some("Settlement Agreement"),
            ),
            // No heading names a kind of document.
            (
                "ACME CORPORATION\n123 Foundry Road\nRowan called Priya.",
                None,
            ),
            // Prose is not a title.
            (
                "The parties met on Tuesday and agreed to circulate a draft agreement soon.",
                None,
            ),
        ];
        for (text, expected) in cases {
            assert_eq!(
                infer_document_type(&digest_of(text)).as_deref(),
                *expected,
                "{text}"
            );
        }
    }

    #[test]
    fn title_case_keeps_initialisms_and_small_words_in_their_place() {
        assert_eq!(title_case("NOTICE OF TERMINATION"), "Notice of Termination");
        assert_eq!(title_case("MUTUAL NDA"), "Mutual NDA");
        assert_eq!(
            title_case("Quarterly Operations Review"),
            "Quarterly Operations Review"
        );
        assert_eq!(title_case("OF COUNSEL LETTER"), "Of Counsel Letter");
    }

    const INVOICE: &str = "INVOICE\nAcme Corporation\n500 Foundry Road\nInvoice Number: INV-7741\nInvoice Date: January 5, 2026\nBill To: Contoso Worldwide, Inc.\nAnnual platform subscription $42,000\n";

    #[test]
    fn an_invoice_with_two_parties_keeps_the_issuer_alone_as_from() {
        let digest = digest_of(INVOICE);
        let (parties, relation) = repair_issued_relation(
            Some("Invoice"),
            vec![
                "Contoso Worldwide, Inc.".to_owned(),
                "Acme Corporation".to_owned(),
            ],
            PartyRelation::Between,
            &digest,
        );
        assert_eq!(parties, vec!["Acme Corporation".to_owned()]);
        assert_eq!(relation, PartyRelation::From);
    }

    #[test]
    fn a_remit_to_line_names_the_issuer_directly() {
        let digest = digest_of(
            "INVOICE\nRemit to: Acme Corporation\nInvoice Date: January 5, 2026\nContoso Worldwide, Inc.\n",
        );
        let (parties, relation) = repair_issued_relation(
            Some("Invoice"),
            vec![
                "Contoso Worldwide, Inc.".to_owned(),
                "Acme Corporation".to_owned(),
            ],
            PartyRelation::Between,
            &digest,
        );
        assert_eq!(parties, vec!["Acme Corporation".to_owned()]);
        assert_eq!(relation, PartyRelation::From);
    }

    #[test]
    fn the_relation_is_left_alone_when_the_document_does_not_settle_it() {
        let digest = digest_of(
            "INVOICE\nInvoice Date: January 5, 2026\nAcme Corporation\nContoso Worldwide, Inc.\n",
        );
        let parties = vec![
            "Contoso Worldwide, Inc.".to_owned(),
            "Acme Corporation".to_owned(),
        ];
        let (kept, relation) = repair_issued_relation(
            Some("Invoice"),
            parties.clone(),
            PartyRelation::Between,
            &digest,
        );
        assert_eq!(kept, parties);
        assert_eq!(relation, PartyRelation::Between);
        // Not an issued document: never touched.
        let (kept, relation) = repair_issued_relation(
            Some("Services Agreement"),
            parties.clone(),
            PartyRelation::Between,
            &digest_of(INVOICE),
        );
        assert_eq!(kept, parties);
        assert_eq!(relation, PartyRelation::Between);
    }
}
