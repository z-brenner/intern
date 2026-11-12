//! The document's own phrases, read back for facts the evidence pipeline
//! validated: a type as the document's title states it, a party's name
//! without the label or address OCR ran into it, an amount with the label
//! the document gives it.
//!
//! Everything here is pure text work over what the document says. Nothing
//! makes a fact more acceptable than validation found it: a phrase read
//! back is always words the document states, in the order it states them.

use crate::cues::{NOT_A_TITLE, ORGANIZATION_ENDINGS, TYPE_NOUNS};
use crate::evidence::normalize;
use crate::index::LEGAL_FORMS;

/// The lower-case words of `text`, split at anything that is not a letter
/// or a digit: "NON-DISCLOSURE Agreement:" is `non disclosure agreement`.
pub(crate) fn words(text: &str) -> Vec<String> {
    normalize(text)
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Where `phrase`'s words stand together, in order, among `text`'s words.
pub(crate) fn phrase_positions(text: &[String], phrase: &[String]) -> Vec<usize> {
    if phrase.is_empty() || phrase.len() > text.len() {
        return Vec::new();
    }
    (0..=text.len() - phrase.len())
        .filter(|&start| {
            text[start..start + phrase.len()]
                .iter()
                .zip(phrase)
                .all(|(word, wanted)| same_word(word, wanted))
        })
        .collect()
}

/// The same word, a plural's "s" aside: "Minutes" states "minute". The
/// plural is allowed either way for a wanted word over three letters.
pub(crate) fn same_word(word: &str, wanted: &str) -> bool {
    word == wanted
        || (wanted.len() > 3
            && (word.strip_suffix('s') == Some(wanted) || wanted.strip_suffix('s') == Some(word)))
}

/// A value a reply wrote where it had nothing to say: punctuation only
/// ("..", "-"), or a word that means nothing ("null", "N/A", "unknown").
pub(crate) fn is_placeholder(value: &str) -> bool {
    let trimmed = value.trim();
    if !trimmed.chars().any(char::is_alphanumeric) {
        return true;
    }
    matches!(
        normalize(trimmed).trim_matches(|character: char| !character.is_alphanumeric()),
        "null"
            | "none"
            | "n/a"
            | "na"
            | "nil"
            | "unknown"
            | "not stated"
            | "not given"
            | "not applicable"
            | "tbd"
    )
}

/// Words that join a type's head noun to what completes it: "Notice *of*
/// Default", "Amendment *to* Lease", "Settlement Agreement *and* Mutual
/// Release".
const CONNECTORS: &[&str] = &["of", "to", "for", "and", "on", "under"];

/// Words after which a title stops to its right: a sentence has begun.
pub(crate) const RIGHT_STOPS: &[&str] = &["is", "are", "was", "were", "hereby", "will", "shall"];

/// Whether the words from `at` open the sentence a notice is given by,
/// after its title: "NOTICE *IS HEREBY* GIVEN", "Notice *hereby* given".
/// A body's sentence that opens on a kind of document ("Receipt *is*
/// acknowledged of the Letter of Notification", "Invoice *is* attached")
/// gives no notice: its kind word is no title.
pub(crate) fn gives_notice(words: &[String], at: usize) -> bool {
    match words.get(at).map(String::as_str) {
        Some("hereby") => true,
        Some(word) if RIGHT_STOPS.contains(&word) => {
            words.get(at + 1).is_some_and(|next| next == "hereby")
        }
        _ => false,
    }
}

/// Whether the words from `at` begin a sentence about a document rather
/// than end its title: "The report on the works *is* provided below", "The
/// invoice *will* follow". A sentence that gives a notice ([`gives_notice`])
/// stands under the notice's own title.
pub(crate) fn begins_a_sentence(words: &[String], at: usize) -> bool {
    words
        .get(at)
        .is_some_and(|word| RIGHT_STOPS.contains(&word.as_str()))
        && !gives_notice(words, at)
}

/// Whether a token is the possessive of a side of a case - "Plaintiff's",
/// "Defendants'", "PETITIONER'S" - which names who files a notice, not
/// the notice.
pub(crate) fn possessive_side(token: &str) -> bool {
    let word = normalize(token).replace(['\u{2019}', '`'], "'");
    let word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'');
    let stem = word
        .strip_suffix("'s")
        .or_else(|| word.strip_suffix("s'"))
        .map(|stem| stem.trim_end_matches('\''));
    stem.is_some_and(|stem| {
        SIDE_WORDS.contains(&stem) || SIDE_WORDS.contains(&format!("{stem}s").as_str())
    })
}

/// Words a caption labels the sides of a case with, whose possessive is no
/// part of a notice's title.
const SIDE_WORDS: &[&str] = &[
    "plaintiff",
    "plaintiffs",
    "defendant",
    "defendants",
    "petitioner",
    "petitioners",
    "respondent",
    "respondents",
    "appellant",
    "appellants",
    "appellee",
    "appellees",
    "claimant",
    "claimants",
    "movant",
    "movants",
    "debtor",
    "debtors",
    "creditor",
    "creditors",
    "trustee",
    "trustees",
    "applicant",
    "applicants",
];

/// Kinds of document a title can name besides [`TYPE_NOUNS`].
const MORE_KINDS: &[&str] = &[
    "assignment",
    "assumption",
    "demand",
    "declarations",
    "explanation",
    "record",
    "advice",
    "list",
    "note",
    "deck",
    "presentation",
    "certification",
    // What a minister or a regulator directs a body with: "Direction to
    // Ofcom to Impose a Notification Condition".
    "direction",
];

/// Kinds a title names with a word a name is as readily made of: what an
/// agency answers a request with ("Response to an Official Information Act
/// Request"), cites an employer with ("Citation and Notification of
/// Penalty") or sends its offices ("Circular 002/2014"). A type is read
/// with them like any other kind; a name is not told from a title by them
/// ([`sets_a_name_apart`]): "First Response Security", "Circular Economy
/// Scotland".
const KINDS_A_NAME_HOLDS: &[&str] = &["response", "citation", "notification", "circular"];

/// Whether `word` names a kind of document.
pub(crate) fn names_a_kind(word: &str) -> bool {
    sets_a_name_apart(word) || KINDS_A_NAME_HOLDS.contains(&word)
}

/// Whether `word` names a kind of document by which a title's line is told
/// from a name's - a line of an address or a signature block that holds it
/// is no organization's, a short name that holds it no person's: every
/// kind but [`KINDS_A_NAME_HOLDS`].
pub(crate) fn sets_a_name_apart(word: &str) -> bool {
    TYPE_NOUNS.contains(&word) || MORE_KINDS.contains(&word)
}

/// The noun a type phrase is a kind of: its first word that names a kind
/// of document and ends the phrase or a part of it - "*Notice* of
/// Default", "First *Amendment* to Lease", "Seed Production and Supply
/// *Agreement*", "Order *Form*", "*NOTICE* IS HEREBY GIVEN" - or, when no
/// word names a kind, the word before its first connector, else its last
/// word. A kind before a body's sentence is no head: "Receipt is
/// acknowledged of the *Letter* of Notification".
pub(crate) fn head_noun(words: &[String]) -> Option<&str> {
    let by_kind = words.iter().enumerate().find(|(at, word)| {
        names_a_kind(word)
            && words.get(at + 1).is_none_or(|next| {
                CONNECTORS.contains(&next.as_str())
                    || TITLE_STOPS.contains(&next.as_str())
                    || gives_notice(words, at + 1)
                    || next.chars().any(|character| character.is_ascii_digit())
            })
    });
    if let Some((_, word)) = by_kind {
        return Some(word.as_str());
    }
    let end = words
        .iter()
        .skip(1)
        .position(|word| CONNECTORS.contains(&word.as_str()))
        .map_or(words.len(), |at| at + 1);
    words[..end].last().map(String::as_str)
}

/// Whether a phrase names a kind of document anywhere in it: "Assignment
/// and Assumption of *Lease*" does, "Leasing Office - 3300 Harrow Lane"
/// does not, and neither does a name that holds a kind a name is as
/// readily made of ([`names_a_kind_at`]): "Citation Capital".
pub(crate) fn has_a_kind(words: &[String]) -> bool {
    (0..words.len()).any(|at| names_a_kind_at(words, at))
}

/// Whether word `at` of a phrase names a kind of document where it stands.
/// A kind that sets a name apart does wherever it stands. A kind a name is
/// as readily made of ([`KINDS_A_NAME_HOLDS`]) does only where it heads a
/// title: at the phrase's end, or before a connector, a word a title stops
/// at, a figure or another kind ("Response *to* an Official Information Act
/// Request", "Ship Notification", "Circular 014/2016", "Response Letter").
/// Before any other word it is a part of a name: "First Response Security",
/// "Citation Capital", "Notification Services Inc", "Circular Economy
/// Scotland" are companies, not titles.
pub(crate) fn names_a_kind_at(words: &[String], at: usize) -> bool {
    let Some(word) = words.get(at) else {
        return false;
    };
    sets_a_name_apart(word)
        || (KINDS_A_NAME_HOLDS.contains(&word.as_str())
            && words.get(at + 1).is_none_or(|next| {
                CONNECTORS.contains(&next.as_str())
                    || TITLE_STOPS.contains(&next.as_str())
                    || next.chars().any(|character| character.is_ascii_digit())
                    || names_a_kind(next)
            }))
}

/// Words a title is not read back past, to its left: an article or a
/// pronoun opens a sentence that names the document, not its title.
const LEFT_STOPS: &[&str] = &["this", "the", "a", "an", "our", "your", "its"];

/// Words after which a title line stops naming the document: its date
/// ("EFFECTIVE SEPTEMBER 1, 2024", "Date reported: ..."), its page, or the
/// authority it is given under ("NOTICE OF DEPOSITION PURSUANT TO RULE
/// 30(b)(6)"), which its number would cut to a dangling "Pursuant to Rule".
/// A type never keeps its own date.
const TITLE_STOPS: &[&str] = &[
    "pursuant",
    "no",
    "number",
    "effective",
    "dated",
    "date",
    "page",
    "issued",
    "as",
    "between",
    "by",
    "from",
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

/// The weekday a title's date can open with ("MINUTES OF THE TUESDAY,
/// OCTOBER 29, 2024, HYBRID MEETING"), which stops the title to its right
/// as the date does. To the left, and in telling a kind from a name, a
/// weekday is a word like any other: the "Good Friday Agreement", a
/// "Cyber Monday Promotion Agreement".
const WEEKDAYS: &[&str] = &[
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

/// Words a title's complement can open with that complete nothing by
/// themselves: a complement cut right after one ends without it.
const ARTICLES: &[&str] = &["the", "a", "an"];

/// Words that label a number on a title's line: "No.", "Number".
const NUMBER_LABELS: &[&str] = &["no", "nos", "number", "num"];

/// Words that name what a reference number on a title's line belongs to,
/// standing before its label: a case, a docket, a file, a matter, a cause
/// or an index (what some courts number a case by), a project, a
/// reference, a control or a tracking number. Only these: any other word
/// before a number is read as the title's own ("Notice of Lease
/// *Violation* No. 4", "Contract *Modification* No. P00003").
const NUMBER_HOLDERS: &[&str] = &[
    "case",
    "docket",
    "file",
    "matter",
    "cause",
    "index",
    "project",
    "reference",
    "ref",
    "control",
    "tracking",
];

/// Whether the token before `label` names what the number there belongs
/// to, rather than ending the title before them: one of
/// [`NUMBER_HOLDERS`], before "No." or "Number" and a figure (or the
/// line's end, where the figure wrapped) or before a "#" figure, right
/// after a kind of document or after a stop - "Notice of Civil Penalty
/// Assessment and Order *Case* No. WQ-SW-WR-2023-626", "Notice of
/// Violation and Order *Docket* No. 24-0113", "Notice of Intent to Revoke
/// Permit *File* #4471", "Notice of Violation, *Case* No. 24-0113". The
/// number is the case's, the docket's or the file's, not the document's,
/// and the title ends before the word. After any other word the same word
/// can be the title's own - "Notice of Related Case No. 3", "Order to Show
/// Cause No. 2024-15" - and stays, as it does where a stop sets it apart
/// from the number ("Notice of Intent to Close Permit File, No. 2024-15")
/// or no figure follows ("... Permit File No Further Action Required").
fn labels_the_number(tokens: &[&str], label: usize) -> bool {
    let core = |token: &str| -> String {
        normalize(token)
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_owned()
    };
    let (Some(number), Some(word), Some(before)) = (
        tokens.get(label),
        label.checked_sub(1).and_then(|at| tokens.get(at)),
        label.checked_sub(2).and_then(|at| tokens.get(at)),
    ) else {
        return false;
    };
    let has_digit = |token: &str| token.chars().any(|character| character.is_ascii_digit());
    let stops = |token: &str| token.ends_with([',', ';', ':', '.']);
    let numbered = (NUMBER_LABELS.contains(&core(number).as_str())
        && tokens.get(label + 1).is_none_or(|next| has_digit(next)))
        || (number.starts_with('#') && has_digit(number));
    numbered
        && NUMBER_HOLDERS.contains(&core(word).as_str())
        && !stops(word)
        && (stops(before) || names_a_kind(&core(before)))
}

/// The type a title line states for a document whose kind is `head`:
/// the run of the line's words around that noun - back to anything with a
/// digit in it ("Fieldnote S2 Product Launch Plan" is a "Product Launch
/// Plan"), a separator or the start of the line, and on through what
/// completes it ("Notice of Default and Reservation of Rights") up to its
/// date or its number. Of a table row, the cell the noun is in. `None` when
/// the line does not name a kind with `head` as its noun.
///
/// The phrase is the line's own words in the line's own order, title-cased
/// only if the line is written in capitals.
pub(crate) fn title_phrase(line: &str, head: &str) -> Option<String> {
    if !names_a_kind(head) {
        return None;
    }
    // Tokens as written, with the separators that end a title kept as
    // tokens of their own.
    let mut tokens: Vec<&str> = Vec::new();
    for cell in line.split(['|', '\u{2013}', '\u{2014}']) {
        if !tokens.is_empty() {
            tokens.push("|");
        }
        tokens.extend(cell.split_whitespace());
    }
    let core = |token: &str| -> String {
        normalize(token)
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_owned()
    };
    let breaks = |token: &str| {
        let word = core(token);
        word.is_empty()
            || token == "|"
            || token == "-"
            || token.chars().any(|character| character.is_ascii_digit())
    };
    let at = tokens.iter().position(|token| {
        let word = core(token);
        word == head || word.strip_suffix('s') == Some(head)
    })?;
    if breaks(tokens[at]) {
        return None;
    }
    // A kind a name is as readily made of heads no title where the line
    // goes on with a name's next word ("CITATION CAPITAL", "First Response
    // Security"), nor where it labels a field ("CITATION: R. v. Harlow,
    // 2021 ONCA 123"), as [`names_a_kind_at`] reads a phrase.
    if KINDS_A_NAME_HOLDS.contains(&head)
        && (tokens[at].ends_with(':')
            || (!tokens[at].ends_with([',', ';', '.'])
                && tokens.get(at + 1).is_some_and(|next| {
                    let word = core(next);
                    !(breaks(next)
                        || CONNECTORS.contains(&word.as_str())
                        || TITLE_STOPS.contains(&word.as_str())
                        || names_a_kind(&word))
                })))
    {
        return None;
    }
    // To the left, words written the way the noun is - a title in capitals
    // is all capitals - and never across a connector (the noun would be a
    // complement then), a company's legal form or an organization's noun.
    // An agreement is named for what it is about, which an organization's
    // noun often is: a "Supply Agreement", a "Transportation Services
    // Agreement", an "Agency Agreement". Only a company's legal form ends
    // one.
    let style = writing(tokens[at]);
    let named_for_its_subject = matches!(head, "agreement" | "contract");
    // "Plaintiff's Notice of Motion", "Defendants' Notice of Appeal": the
    // side filing a notice is no part of its name. "White Finding, Notice
    // of Violation": a tag before a comma is not the title either, though
    // a comma inside an agreement's title is its own.
    let stops_left = |token: &str| {
        let word = core(token);
        let ending = word.trim_end_matches('.');
        breaks(token)
            || token.ends_with(':')
            || possessive_side(token)
            || (token.ends_with(',') && !named_for_its_subject)
            || TITLE_STOPS.contains(&word.as_str())
            || LEFT_STOPS.contains(&word.as_str())
            || CONNECTORS.contains(&word.as_str())
            || if named_for_its_subject {
                LEGAL_FORMS.contains(&ending) || matches!(ending, "corporation" | "company")
            } else {
                ORGANIZATION_ENDINGS.contains(&ending)
            }
            || writing(token) != style
    };
    let mut start = at;
    while start > 0 {
        let previous = tokens[start - 1];
        let word = core(previous);
        // An agreement is named for what it joins: "Purchase and Sale
        // Agreement", "Co-Branding and Advertising Agreement". Any other
        // kind stops at "and" - "Availability and Price List" is a price
        // list.
        if word == "and" && named_for_its_subject && start >= 2 && !stops_left(tokens[start - 2]) {
            start -= 1;
            continue;
        }
        if stops_left(previous) {
            break;
        }
        start -= 1;
    }
    // Markdown marks before the title.
    while start < at
        && tokens[start]
            .chars()
            .all(|character| matches!(character, '#' | '*'))
    {
        start += 1;
    }
    // To the right, the title ends where a sentence starts ("Notice IS
    // Hereby Given") or an aside opens ("Notice of Variation and
    // Consolidation with introductory note"); an agreement can be named
    // "with" what it concerns.
    // "Will" is the noun, not the verb that starts a sentence, after
    // "of", "the" or "last" and before "Contest" or "and Testament":
    // "Notice of Will Contest", "Notice of Probate of Will".
    let will_is_a_noun = |index: usize| {
        let before = index.checked_sub(1).map(|at| core(tokens[at]));
        let after = tokens.get(index + 1).map(|token| core(token));
        after.is_none()
            || before.as_deref().is_some_and(|word| {
                matches!(
                    word,
                    "of" | "the" | "a" | "an" | "last" | "his" | "her" | "their" | "and"
                )
            })
            || after
                .as_deref()
                .is_some_and(|word| matches!(word, "contest" | "contests" | "and" | "testament"))
    };
    let stops_right = |index: usize| {
        let word = core(tokens[index]);
        TITLE_STOPS.contains(&word.as_str())
            || WEEKDAYS.contains(&word.as_str())
            || (RIGHT_STOPS.contains(&word.as_str()) && !(word == "will" && will_is_a_noun(index)))
            || (word == "with" && !named_for_its_subject)
    };
    let mut end = at + 1;
    if !tokens[at].ends_with(':') {
        while end + 1 < tokens.len()
            && CONNECTORS.contains(&core(tokens[end]).as_str())
            && !breaks(tokens[end + 1])
            && !stops_right(end + 1)
        {
            end += 2;
            while end < tokens.len()
                && !breaks(tokens[end])
                && !tokens[end - 1].ends_with(':')
                && !stops_right(end)
                && !CONNECTORS.contains(&core(tokens[end]).as_str())
            {
                end += 1;
            }
        }
    }
    // A complement cut by a stop right after its article - "Minutes of the
    // Tuesday, October 29" - leaves "of the" dangling, which completes
    // nothing: the title ends before it. Only a connector and its article
    // dangle so, and only before a stop: a letter that ends the line or
    // stands after a noun is the title's own ("Amendment to Exhibit A",
    // "Notice of Exercise of Option A").
    if end < tokens.len()
        && stops_right(end)
        && end >= at + 3
        && ARTICLES.contains(&core(tokens[end - 1]).as_str())
        && CONNECTORS.contains(&core(tokens[end - 2]).as_str())
    {
        end -= 2;
    }
    // A complement that ran up to a number's label ends before the word
    // that names what the number belongs to ([`labels_the_number`]):
    // "Notice of Civil Penalty Assessment and Order Case No.
    // WQ-SW-WR-2023-626" is a "Notice of Civil Penalty Assessment and
    // Order".
    if end >= at + 3 && labels_the_number(&tokens, end) {
        end -= 1;
    }
    // A comma inside the title is the title's own ("Software License,
    // Customization and Maintenance Agreement"); only its ends lose
    // punctuation.
    let last = end - start - 1;
    let phrase = tokens[start..end]
        .iter()
        .enumerate()
        .map(|(at, token)| {
            let token = token.trim_start_matches(['#', '*', '(', '"', '\u{201c}']);
            if at == last {
                token.trim_end_matches([':', ',', ';', '.', '*', ')', '"', '\u{201d}'])
            } else {
                token.trim_end_matches([':', ';', '.', '*', ')', '"', '\u{201d}'])
            }
        })
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let lowered = phrase.to_lowercase();
    if phrase.is_empty()
        || phrase.split_whitespace().count() > 10
        || NOT_A_TITLE.iter().any(|part| lowered == *part)
        || !has_a_kind(&words(&phrase))
    {
        return None;
    }
    Some(crate::infer::title_case(&tidy_case(&phrase)))
}

/// A document's type without the identifier a title carries beside its
/// kind: the number after its kind ("Invoice No. 2248", "Invoice Number
/// 00-03-06", "Invoice #PMr0197", "Capital Call Notice No. 7"), a
/// subtitle a dash sets off ("Quarterly Business Review - Q3 2026",
/// "Straight Bill of Lading - Short Form"), a written date before the kind ("January
/// 1, 1997 Invoice"), or a code before it ("FreeBSD-EN-26:25.tzdata Errata
/// Notice"). The number names this one copy, which the description keeps
/// as the identifier; the type names the kind. A year, a phase, a rule or
/// a section the title is named by stays: "2025 Annual Report", "Phase 1
/// Decision Letter", "Section 5 Objection Letter", "Notice of Deposition
/// Pursuant to Rule 30(b)(6)", "CP161 Notice".
pub(crate) fn without_identifier(value: &str) -> String {
    let tokens = value.split_whitespace().collect::<Vec<_>>();
    let core = |token: &str| -> String {
        normalize(token)
            .trim_matches(|character: char| !character.is_alphanumeric())
            .to_owned()
    };
    let has_digit = |token: &str| token.chars().any(|character| character.is_ascii_digit());
    let Some(head) = tokens.iter().position(|token| names_a_kind(&core(token))) else {
        return value.to_owned();
    };
    let mut start = 0;
    // "January 1, 1997 Invoice": a month, a day and a year before the kind.
    if head >= 3
        && MONTHS.contains(&core(tokens[0]).as_str())
        && core(tokens[1])
            .chars()
            .all(|character| character.is_ascii_digit())
        && core(tokens[2]).len() == 4
        && has_digit(tokens[2])
    {
        start = 3;
    }
    // A reference code in one token before the kind: digits and letters
    // joined by two or more dots, dashes or colons ("FreeBSD-EN-26:25.tzdata
    // Errata Notice", "INV-2024-001 Invoice"). A standardized form's code is
    // the kind itself, not one copy of it, and is short, with one dash at
    // most: "W-9 Form", "10-K Annual Report", "1099-MISC Form" keep it.
    if start == 0
        && head >= 1
        && has_digit(tokens[0])
        && tokens[0].chars().any(char::is_alphabetic)
        && !is_form_code(tokens[0])
    {
        start = 1;
    }
    let identifier_word = |token: &str| {
        matches!(
            core(token).as_str(),
            "no" | "number" | "num" | "nos" | "ref" | "id"
        )
    };
    // An amendment, an addendum or a change order is numbered as its kind:
    // "Amendment No. 3" is the third amendment, which a bare "Amendment"
    // would not say.
    let numbered_kind = matches!(
        core(tokens[head]).as_str(),
        "amendment" | "addendum" | "order"
    ) && tokens[..head]
        .last()
        .is_none_or(|previous| core(previous) != "purchase");
    let mut end = tokens.len();
    let mut held = false;
    for at in head + 1..tokens.len() {
        if numbered_kind && identifier_word(tokens[at]) {
            break;
        }
        let token = tokens[at];
        let next = tokens.get(at + 1).copied();
        let introduces = identifier_word(token) && next.is_none_or(has_digit);
        let numbered = token.starts_with('#') && has_digit(token);
        let dashed = matches!(token, "-" | "\u{2013}" | "\u{2014}") && at + 1 < tokens.len();
        if introduces || numbered || dashed {
            end = at;
            // The word that names what the number belongs to goes with it
            // ([`labels_the_number`]): "... and Order Case No. 626",
            // "Notice of Violation, Case No. 24-0113".
            if (introduces || numbered) && at >= head + 2 && labels_the_number(&tokens, at) {
                end = at - 1;
                held = true;
            }
            break;
        }
    }
    let kept = tokens[start..end].join(" ");
    let kept = if held {
        kept.trim_end_matches([',', ';', ':', '.']).to_owned()
    } else {
        kept
    };
    if kept.is_empty() || !has_a_kind(&words(&kept)) {
        return value.to_owned();
    }
    kept
}

/// A standardized form's code, which is the kind itself and no part of one
/// copy's identifier: letters and digits with one dot, dash or colon at most
/// ("W-9", "10-K", "1099-MISC", "SF-86", "CP161"). A reference code has two or
/// more ("FreeBSD-EN-26:25.tzdata", "INV-2024-001").
pub(crate) fn is_form_code(token: &str) -> bool {
    token.chars().any(|character| character.is_ascii_digit())
        && token.chars().any(char::is_alphabetic)
        && token
            .chars()
            .filter(|character| matches!(character, '-' | '.' | ':'))
            .count()
            < 2
}

const MONTHS: [&str; 12] = [
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

/// How a word is written: in capitals, with a capital, or in lower case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Writing {
    Capitals,
    Capitalized,
    Lower,
}

fn writing(token: &str) -> Writing {
    let letters = token.chars().filter(|character| character.is_alphabetic());
    let (mut upper, mut lower, mut first_upper, mut first) = (0, 0, false, true);
    for letter in letters {
        if first {
            first_upper = letter.is_uppercase();
            first = false;
        }
        if letter.is_uppercase() {
            upper += 1;
        } else {
            lower += 1;
        }
    }
    if upper > 1 && lower == 0 {
        Writing::Capitals
    } else if first_upper {
        Writing::Capitalized
    } else if upper == 1 && lower == 0 {
        Writing::Capitals
    } else {
        Writing::Lower
    }
}

/// Whether a word's capitals are where OCR put them, not where a writer
/// would: two capitals and then a lower-case letter ("EMBer", "JUniper",
/// "RECeIPt"), or lower case broken by a capital more than once
/// ("MANUFACtURInG"). "McKinsey", "MacBook" and "LinkedIn" are written so.
fn is_irregular_word(word: &str) -> bool {
    let letters = word
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect::<Vec<_>>();
    if letters.len() <= 3 {
        return false;
    }
    let two_capitals_then_lower = letters
        .windows(3)
        .any(|w| w[0].is_uppercase() && w[1].is_uppercase() && w[2].is_lowercase());
    let breaks = letters
        .windows(2)
        .filter(|w| w[0].is_lowercase() && w[1].is_uppercase())
        .count();
    two_capitals_then_lower || breaks >= 2
}

/// Text whose capitals OCR scattered ("EMBer POSt MANUFACtURInG LLC"), as
/// capitals throughout, so the name and title casing that reads a heading
/// in capitals reads it too; any other text as it is.
pub(crate) fn tidy_case(text: &str) -> String {
    if text.split_whitespace().any(is_irregular_word) {
        text.to_uppercase()
    } else {
        text.to_owned()
    }
}

/// Whether the statement of a phrase at `start..end` among `text`'s words
/// names another document - "Re: Residential Lease Agreement dated August
/// 1, 2024", "the Loan Agreement under which", "First Amendment to Software
/// License Agreement" - rather than this one.
pub(crate) fn is_reference(text: &[String], start: usize, end: usize) -> bool {
    let after = text.get(end..(end + 2).min(text.len())).unwrap_or_default();
    if after.iter().any(|word| word == "dated") {
        return true;
    }
    let before = &text[start.saturating_sub(3)..start];
    before.iter().any(|word| {
        matches!(
            word.as_str(),
            "under" | "pursuant" | "per" | "to" | "amends" | "amending" | "amend"
        )
    })
}

/// Labels a field's value is a document's subject under: "Project:",
/// "Premises:", "Position:".
const SUBJECT_LABELS: &[&str] = &[
    "project",
    "purpose",
    "confidential purpose",
    "subject",
    "re",
    "regarding",
    "matter",
    "premises",
    "property",
    "position",
    "job title",
    "title",
    "role",
    "services",
    "scope",
    "description",
    "goods",
    "work",
    "equipment",
];

/// A "Label: value" phrase split in two, when what stands before the colon
/// is a short label.
pub(crate) fn label_and_value(text: &str) -> Option<(&str, &str)> {
    let (label, value) = text.split_once(':')?;
    let label = label.trim();
    let value = value.trim();
    (!label.is_empty()
        && label.split_whitespace().count() <= 4
        && !label.chars().any(|character| character.is_ascii_digit())
        && value.chars().any(char::is_alphanumeric))
    .then_some((label, value))
}

/// A subject as the document's value: "Project: Aurora Catalog Project" is
/// "Aurora Catalog Project", and a label that names something other than a
/// subject ("Bill to: ...", "Journal date: ...") is no subject at all.
pub(crate) fn subject_value(subject: &str) -> Option<&str> {
    match label_and_value(subject) {
        Some((label, value)) => {
            let label = normalize(label);
            SUBJECT_LABELS
                .contains(&label.trim_matches(|character: char| !character.is_alphanumeric()))
                .then_some(value)
        }
        None => Some(subject.trim()),
    }
}

/// Words that open a field naming who a document goes to or comes from,
/// never what it is about: a subject that begins with one is a party.
const PARTY_OPENERS: &[&[&str]] = &[
    &["bill", "to"],
    &["billed", "to"],
    &["ship", "to"],
    &["sold", "to"],
    &["remit", "to"],
    &["invoice", "to"],
    &["deliver", "to"],
    &["to"],
    &["from"],
    &["attn"],
    &["attention"],
    &["dear"],
];

/// Whether a subject is a party's field rather than a subject: "Bill to
/// Atlas Threadworks LLC".
pub(crate) fn opens_with_party_label(subject: &str) -> bool {
    let words = words(subject);
    PARTY_OPENERS.iter().any(|opener| {
        words.len() > opener.len()
            && words
                .iter()
                .zip(opener.iter())
                .all(|(word, wanted)| word == wanted)
    })
}

/// The legal forms that end a company's name. A name that runs on past one
/// is two names, or a name and what OCR ran into it.
const ENTITY_SUFFIXES: &[&str] = &[
    "llc", "l.l.c.", "inc", "inc.", "ltd", "ltd.", "llp", "pllc", "corp.", "gmbh", "plc",
];

/// The last words of a street address: "47 Juniper Loop", "88 Harbour
/// Street".
const STREET_WORDS: &[&str] = &[
    "street",
    "st",
    "avenue",
    "ave",
    "road",
    "rd",
    "lane",
    "ln",
    "way",
    "drive",
    "dr",
    "boulevard",
    "blvd",
    "court",
    "ct",
    "loop",
    "place",
    "pl",
    "terrace",
    "circle",
    "parkway",
    "pkwy",
    "highway",
    "hwy",
    "trail",
    "crescent",
    "plaza",
    "square",
];

/// Labels a field's value can be run into a name with: "Landlord: Acme".
const NAME_LABELS: &[&str] = &[
    "property",
    "premises",
    "address",
    "name",
    "company",
    "party",
    "bill to",
    "ship to",
    "sold to",
    "remit to",
    "attn",
    "attention",
    "to",
    "from",
    "client",
    "customer",
    "vendor",
    "supplier",
    "landlord",
    "tenant",
    "lessor",
    "lessee",
    "owner",
    "resident",
    "buyer",
    "seller",
    "employer",
    "employee",
    "borrower",
    "lender",
    "licensor",
    "licensee",
    "contractor",
    "consultant",
    "insured",
    "carrier",
    "shipper",
    "consignee",
];

/// A party's name with what a line's layout ran into it taken away: a
/// field's label ("Landlord: Acme LLC"), a street address on either side
/// of it ("Property 47 Juniper Loop Cedar Finch Properties LLC"), and a
/// second name after the legal form that ends the first ("Cedar Finch
/// Properties LLC Orion Glass Studio Inc"). What is kept is a run of the
/// name's own words; a name with none of these is returned as it is.
pub(crate) fn trim_name(name: &str) -> String {
    let mut tokens = name.split_whitespace().collect::<Vec<_>>();
    let core = |token: &str| {
        normalize(token)
            .trim_matches(|character: char| !character.is_alphanumeric() && character != '.')
            .to_owned()
    };
    // A label before a colon.
    if let Some(colon) = tokens.iter().position(|token| token.ends_with(':')) {
        let label = tokens[..=colon]
            .iter()
            .map(|token| core(token))
            .collect::<Vec<_>>()
            .join(" ");
        if colon + 1 < tokens.len()
            && NAME_LABELS.contains(&label.trim_end_matches(':').trim_end_matches('.'))
        {
            tokens.drain(..=colon);
        }
    }
    // A street address: a house number and, a few words on, a street word.
    let street = tokens.iter().enumerate().find_map(|(at, token)| {
        let number = token.trim_end_matches(',');
        let is_number = number
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
            && number.chars().filter(char::is_ascii_digit).count() <= 6
            && number.chars().all(char::is_alphanumeric);
        if !is_number {
            return None;
        }
        (at + 1..(at + 5).min(tokens.len()))
            .find(|&end| STREET_WORDS.contains(&core(tokens[end]).trim_end_matches('.')))
            .map(|end| (at, end))
    });
    if let Some((number, end)) = street {
        let after = tokens[end + 1..].to_vec();
        let before = tokens[..number].to_vec();
        let named = |part: &[&str]| {
            part.iter()
                .any(|token| token.chars().next().is_some_and(char::is_uppercase))
        };
        // What follows a street word and its comma - "Lane, Cedar Rapids,
        // IA 52402" - or ends in a postal code is the rest of the address.
        let postal = after.last().is_some_and(|token| {
            let digits = token.trim_end_matches([',', '.']);
            (5..=10).contains(&digits.len())
                && digits.chars().all(|c| c.is_ascii_digit() || c == '-')
        });
        let address_goes_on = tokens[end].ends_with(',') || postal;
        tokens = if named(&after) && !(address_goes_on && named(&before)) {
            after
        } else {
            before
        };
    }
    // A legal form ends a company's name.
    if let Some(at) = tokens
        .iter()
        .position(|token| ENTITY_SUFFIXES.contains(&core(token).as_str()))
        && at + 2 < tokens.len()
    {
        tokens.truncate(at + 1);
    }
    let trimmed = tokens
        .join(" ")
        .trim_end_matches([',', ';', ':'])
        .trim()
        .to_owned();
    if trimmed
        .chars()
        .filter(|character| character.is_alphabetic())
        .count()
        < 2
    {
        name.trim().to_owned()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    /// A title stops where a sentence starts or an aside opens to its
    /// right, and before the side filing it or a tag set off by a comma to
    /// its left. An agreement's title is its own across a comma.
    #[test]
    fn a_title_stops_at_a_side_a_tag_a_verb_or_an_aside() {
        for (line, head, phrase) in [
            (
                "Plaintiff's Notice of Voluntary Dismissal",
                "notice",
                "Notice of Voluntary Dismissal",
            ),
            ("DEFENDANTS' NOTICE OF APPEAL", "notice", "Notice of Appeal"),
            (
                "White Finding, Notice of Violation",
                "notice",
                "Notice of Violation",
            ),
            ("Notice IS Hereby Given that", "notice", "Notice"),
            ("Notice will be given to creditors", "notice", "Notice"),
            ("NOTICE OF WILL CONTEST", "notice", "Notice of Will Contest"),
            (
                "Notice of Probate of Will",
                "notice",
                "Notice of Probate of Will",
            ),
            (
                "Notice of Variation and Consolidation with introductory note",
                "notice",
                "Notice of Variation and Consolidation",
            ),
            (
                "Software License, Customization and Maintenance Agreement",
                "agreement",
                "Software License, Customization and Maintenance Agreement",
            ),
        ] {
            assert_eq!(title_phrase(line, head).as_deref(), Some(phrase), "{line}");
        }
        assert!(possessive_side("Plaintiff\u{2019}s"));
        assert!(possessive_side("Defendants'"));
        assert!(!possessive_side("Owner's"));
    }

    use super::*;

    #[test]
    fn a_titles_identifier_is_no_part_of_the_type() {
        for (value, kind) in [
            ("INVOICE No. 2248", "INVOICE"),
            ("Invoice Number 00-03-06", "Invoice"),
            ("Invoice #PMr0197", "Invoice"),
            ("Capital Call Notice No. 7", "Capital Call Notice"),
            ("Invoice No", "Invoice"),
            ("Purchase Order No. 4471", "Purchase Order"),
            (
                "Quarterly Business Review - Q3 2026",
                "Quarterly Business Review",
            ),
            (
                "Straight Bill of Lading - Short Form",
                "Straight Bill of Lading",
            ),
            (
                "Offer of Employment - Senior Data Engineer",
                "Offer of Employment",
            ),
            ("January 1, 1997 Invoice", "Invoice"),
            ("FreeBSD-EN-26:25.tzdata Errata Notice", "Errata Notice"),
            ("INV-2024-001 Invoice", "Invoice"),
        ] {
            assert_eq!(without_identifier(value), kind, "{value}");
        }
        for kept in [
            "2025 Annual Report",
            "Annual Report Fiscal Year 2026",
            "Phase 1 Decision Letter",
            "Section 5 Objection Letter",
            "Notice of Deposition Pursuant to Rule 30(b)(6)",
            "CP161 Notice",
            "Notice in Accordance with Section 57C of the Railways Act 1993",
            "Second Amendment",
            "Form 1099",
            "W-9 Form",
            "10-K Annual Report",
            "1099-MISC Form",
            "SF-86 Questionnaire",
            "Amendment No. 3",
            "Change Order No. 7",
            "ADDENDUM NO. 2",
        ] {
            assert_eq!(without_identifier(kept), kept);
        }
    }

    fn w(text: &str) -> Vec<String> {
        words(text)
    }

    #[test]
    fn a_title_line_gives_the_type_it_names_and_nothing_after_it() {
        for (line, head, phrase) in [
            ("INVOICE INV-2048", "invoice", Some("Invoice")),
            ("PACKING SLIP PS-311", "slip", Some("Packing Slip")),
            (
                "LEASE AGREEMENT EFFECTIVE SEPTEMBER 1 2024",
                "agreement",
                Some("Lease Agreement"),
            ),
            (
                "MOONLIT ARCHIVE PROJECT JOURNAL - PAGE 1",
                "journal",
                Some("Moonlit Archive Project Journal"),
            ),
            (
                "NOTICE OF DEFAULT AND RESERVATION OF RIGHTS",
                "notice",
                Some("Notice of Default and Reservation of Rights"),
            ),
            (
                "PROPERTY LOSS NOTICE Date reported: 02/16/2026",
                "notice",
                Some("Property Loss Notice"),
            ),
            (
                "| HARTWELL COUNTY | VENDOR REGISTRATION FORM |",
                "form",
                Some("Vendor Registration Form"),
            ),
            (
                "## Fieldnote S2 Product Launch Plan",
                "plan",
                Some("Product Launch Plan"),
            ),
            (
                "# Mutual Non-Disclosure Agreement",
                "agreement",
                Some("Mutual Non-Disclosure Agreement"),
            ),
            (
                "CO-BRANDING AND ADVERTISING AGREEMENT",
                "agreement",
                Some("Co-branding and Advertising Agreement"),
            ),
            (
                "This Purchase and Sale Agreement",
                "agreement",
                Some("Purchase and Sale Agreement"),
            ),
            (
                "SOFTWARE LICENSE, CUSTOMIZATION AND MAINTENANCE AGREEMENT",
                "agreement",
                Some("Software License, Customization and Maintenance Agreement"),
            ),
            (
                "SETTLEMENT AGREEMENT AND MUTUAL RELEASE",
                "agreement",
                Some("Settlement Agreement and Mutual Release"),
            ),
            // An agreement named for what an organization's noun names.
            (
                "SEED PRODUCTION AND SUPPLY AGREEMENT",
                "agreement",
                Some("Seed Production and Supply Agreement"),
            ),
            (
                "TRANSPORTATION SERVICES AGREEMENT",
                "agreement",
                Some("Transportation Services Agreement"),
            ),
            ("AGENCY AGREEMENT", "agreement", Some("Agency Agreement")),
            // A company's legal form still ends it, and other kinds still
            // stop at the noun that ends their issuer's name.
            (
                "Wrenfield Supply Co. Agreement",
                "agreement",
                Some("Agreement"),
            ),
            (
                "ACME CORPORATION SERVICES AGREEMENT",
                "agreement",
                Some("Services Agreement"),
            ),
            (
                "WRENFIELD RESTAURANT SUPPLY INVOICE",
                "invoice",
                Some("Invoice"),
            ),
            (
                "NOTICE OF SPECIAL MEETING OF MEMBERS",
                "notice",
                Some("Notice of Special Meeting of Members"),
            ),
            ("THISTLEDOWN WHOLESALE NURSERY", "nursery", None),
            (
                "DElIvery RECeIPt DR-771",
                "receipt",
                Some("Delivery Receipt"),
            ),
            (
                "ASSIGNMENT AND ASSUMPTION OF LEASE",
                "assignment",
                Some("Assignment and Assumption of Lease"),
            ),
            (
                "CERTIFIED MAIL - RETURN RECEIPT REQUESTED",
                "requested",
                None,
            ),
            (
                "Halvorsen Fixture Works LLC INVOICE",
                "invoice",
                Some("Invoice"),
            ),
            (
                "Whitlock & Sons Plumbing LLC SERVICE INVOICE",
                "invoice",
                Some("Service Invoice"),
            ),
            (
                "Tolliver Grain & Feed Cooperative Remittance Advice",
                "advice",
                Some("Remittance Advice"),
            ),
            (
                "STATEMENT OF WORK NO. 4",
                "statement",
                Some("Statement of Work"),
            ),
            (
                "SPRING WHOLESALE AVAILABILITY AND PRICE LIST",
                "list",
                Some("Price List"),
            ),
            (
                "Re: Approval of Trade Credit Application Dear Keziah Ambrose:",
                "application",
                Some("Trade Credit Application"),
            ),
            ("Kingsfold Community Bank", "statement", None),
            (
                "NOTICE OF DEPOSITION PURSUANT TO RULE 30(b)(6)",
                "notice",
                Some("Notice of Deposition"),
            ),
            // A title never keeps its date: the weekday it opens with stops
            // it, and the "of the" left before it completes nothing.
            (
                "MINUTES OF THE TUESDAY, OCTOBER 29, 2024, HYBRID MEETING",
                "minutes",
                Some("Minutes"),
            ),
            (
                "Minutes of the Wednesday, February 14, 2024 Webinar Meeting",
                "minutes",
                Some("Minutes"),
            ),
            (
                "Minutes of the Regular Meeting of the Board of Directors",
                "minutes",
                Some("Minutes of the Regular Meeting of the Board of Directors"),
            ),
            (
                "NOTICE OF THE ANNUAL MEETING OF SHAREHOLDERS",
                "notice",
                Some("Notice of the Annual Meeting of Shareholders"),
            ),
            // The letter a schedule or an option is named by is no article:
            // it stays at the line's end and before the date alike.
            (
                "Amendment to Exhibit A",
                "amendment",
                Some("Amendment to Exhibit A"),
            ),
            (
                "Addendum to Exhibit A dated March 3, 2024",
                "addendum",
                Some("Addendum to Exhibit A"),
            ),
            (
                "Notice of Exercise of Option A",
                "notice",
                Some("Notice of Exercise of Option A"),
            ),
            // A weekday stops a title to its right only: to its left it is
            // the title's own word.
            (
                "Good Friday Agreement",
                "agreement",
                Some("Good Friday Agreement"),
            ),
            (
                "Cyber Monday Promotion Agreement",
                "agreement",
                Some("Cyber Monday Promotion Agreement"),
            ),
        ] {
            assert_eq!(title_phrase(line, head).as_deref(), phrase, "{line}");
        }
    }

    #[test]
    fn a_head_noun_is_the_word_before_what_completes_it() {
        assert_eq!(head_noun(&w("Notice of Default")), Some("notice"));
        assert_eq!(head_noun(&w("First Amendment to Lease")), Some("amendment"));
        assert_eq!(
            head_noun(&w("Settlement Agreement and Mutual Release")),
            Some("agreement")
        );
        assert_eq!(head_noun(&w("Credit Union Merger Notice")), Some("notice"));
        assert_eq!(head_noun(&w("Invoice")), Some("invoice"));
        assert_eq!(head_noun(&w("NOTICE IS HEREBY GIVEN")), Some("notice"));
        assert_eq!(head_noun(&w("Notice hereby given")), Some("notice"));
        assert_eq!(
            head_noun(&w(
                "Receipt is acknowledged of the Letter of Notification of PPL Electric Utilities"
            )),
            Some("letter")
        );
        assert_eq!(
            head_noun(&w("Seed Production and Supply Agreement")),
            Some("agreement")
        );
        assert_eq!(head_noun(&w("Order Form")), Some("form"));
        assert_eq!(head_noun(&w("Lease Agreement")), Some("agreement"));
        assert_eq!(
            head_noun(&w("LEASE AGREEMENT EFFECTIVE SEPTEMBER 1 2024")),
            Some("agreement")
        );
        assert_eq!(
            head_noun(&w("MOONLIT ARCHIVE PROJECT JOURNAL - PAGE 1")),
            Some("journal")
        );
        assert_eq!(
            head_noun(&w("Assignment and Assumption of Lease")),
            Some("assignment")
        );
    }

    #[test]
    fn a_phrase_is_found_whole_and_a_reference_is_told_apart() {
        let text = w("Re: Residential Lease Agreement dated August 1, 2024 for Apartment 4C");
        let phrase = w("Residential Lease Agreement");
        let found = phrase_positions(&text, &phrase);
        assert_eq!(found, vec![1]);
        assert!(is_reference(&text, 1, 4));
        let offer = w("Re: Offer of Employment - Senior Data Engineer");
        assert!(!is_reference(&offer, 1, 4));
        let title = w("NOTICE OF RENT INCREASE");
        assert!(phrase_positions(&title, &w("Rent Increase Notice")).is_empty());
        let notice = w("Notice of Rent Increase");
        assert_eq!(phrase_positions(&title, &notice), vec![0]);
        assert!(!is_reference(&title, 0, 4));
        assert_eq!(
            phrase_positions(&w("BOARD MINUTES"), &w("Board Minute")),
            vec![0]
        );
    }

    #[test]
    fn scattered_capitals_are_read_as_capitals_and_a_writers_are_kept() {
        assert_eq!(
            tidy_case("EMBer POSt MANUFACtURInG LLC"),
            "EMBER POST MANUFACTURING LLC"
        );
        for kept in [
            "McKinsey & Company",
            "MacBook Repairs LLC",
            "LinkedIn",
            "NDA",
            "Acme LLC",
        ] {
            assert_eq!(tidy_case(kept), kept);
        }
    }

    #[test]
    fn placeholders_say_nothing() {
        for value in ["..", "-", " ", "null", "N/A", "Unknown", "none."] {
            assert!(is_placeholder(value), "{value:?}");
        }
        for value in ["INV-2048", "006", "Acme"] {
            assert!(!is_placeholder(value), "{value:?}");
        }
    }

    #[test]
    fn a_name_loses_the_label_and_address_ocr_ran_into_it() {
        assert_eq!(
            trim_name("PrOperty 47 JUniper LOOP Cedar Finch Properties Llc"),
            "Cedar Finch Properties Llc"
        );
        assert_eq!(
            trim_name("Cedar Finch Properties Llc Orion Glass Studio inc"),
            "Cedar Finch Properties Llc"
        );
        assert_eq!(
            trim_name("Landlord: Cedar Finch Properties LLC"),
            "Cedar Finch Properties LLC"
        );
        assert_eq!(
            trim_name("Acme Corporation 12 Main Street"),
            "Acme Corporation"
        );
        assert_eq!(
            trim_name("John Smith, 1420 Fielder Lane, Cedar Rapids, IA 52402"),
            "John Smith"
        );
        for kept in [
            "Orion Glass Studio inc",
            "Contoso Worldwide, Inc.",
            "Ravensmoor Growth Partners III, L.P.",
            "Tenant Holdings LLC",
            "John Smith",
            "Quill and Vane Advisory Group, Inc.",
        ] {
            assert_eq!(trim_name(kept), kept);
        }
    }

    #[test]
    fn a_labeled_subject_is_its_value_and_a_party_field_is_none() {
        assert_eq!(
            subject_value("Project: Aurora Catalog Project"),
            Some("Aurora Catalog Project")
        );
        assert_eq!(
            subject_value("Confidential purpose: Project Marigold."),
            Some("Project Marigold.")
        );
        assert_eq!(subject_value("Journal date: July 1, 2025"), None);
        assert_eq!(subject_value("display shelving"), Some("display shelving"));
        assert!(opens_with_party_label("Bill to Atlas Threadworks LLC"));
        assert!(!opens_with_party_label("billing services"));
    }

    /// A direction is a kind of document a title can name: "Direction to
    /// Ofcom to Impose a Notification Condition".
    #[test]
    fn a_direction_is_a_kind() {
        assert!(names_a_kind("direction"));
        assert_eq!(
            title_phrase(
                "DIRECTION TO OFCOM TO IMPOSE A NOTIFICATION CONDITION",
                "direction"
            )
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
            .map(|phrase| phrase.starts_with("direction")),
            Some(true)
        );
    }

    /// A response, a citation, a notification and a circular are kinds a
    /// title names, each read as the head of its title line. They are
    /// words a name holds too, so none of them sets a name apart.
    #[test]
    fn a_response_a_citation_a_notification_and_a_circular_are_kinds() {
        for (line, head, phrase) in [
            (
                "RESPONSE TO AN OFFICIAL INFORMATION ACT REQUEST (REF: DOIA007-2024/25)",
                "response",
                "Response to an Official Information Act Request Ref",
            ),
            (
                "Citation and Notification of Penalty",
                "citation",
                "Citation and Notification of Penalty",
            ),
            (
                "Circular 014/2016: Transport Secretary\u{2019}s decision on the findings",
                "circular",
                "Circular",
            ),
            (
                "Subject: UPS Ship Notification, 1Z7261100192688747",
                "notification",
                "Ship Notification",
            ),
        ] {
            assert!(names_a_kind(head), "{head}");
            assert!(!sets_a_name_apart(head), "{head}");
            assert_eq!(head_noun(&words(line)), Some(head), "{line}");
            assert_eq!(title_phrase(line, head).as_deref(), Some(phrase), "{line}");
        }
        assert_eq!(
            without_identifier(
                "RESPONSE TO AN OFFICIAL INFORMATION ACT REQUEST (REF: DOIA007-2024/25)"
            ),
            "RESPONSE TO AN OFFICIAL INFORMATION ACT REQUEST"
        );
        // A kind of another type's title is still the title's head: a
        // letter of response is a letter, a response plan a plan.
        assert_eq!(
            head_noun(&words("letter of response to appeal")),
            Some("letter")
        );
        assert_eq!(head_noun(&words("Emergency Response Plan")), Some("plan"));
        // Every other kind sets a name apart, as before.
        for kind in ["notice", "invoice", "direction"] {
            assert!(names_a_kind(kind) && sets_a_name_apart(kind), "{kind}");
        }
    }

    /// A kind a name is as readily made of names one where it heads a
    /// title - at the phrase's end, before a connector, a stop, a figure or
    /// another kind - and is a part of a name before any other word.
    #[test]
    fn a_kind_a_name_holds_names_one_only_where_it_heads_a_title() {
        for phrase in [
            "Response to an Official Information Act Request",
            "UPS Ship Notification",
            "Circular 014/2016",
            "Citation and Notification of Penalty",
            "Response Letter",
            "Consultation Response",
        ] {
            assert!(has_a_kind(&words(phrase)), "{phrase}");
        }
        for name in [
            "First Response Security Ltd",
            "Citation Capital",
            "Notification Services Inc",
            "Circular Economy Scotland",
        ] {
            assert!(!has_a_kind(&words(name)), "{name}");
        }
        assert_eq!(title_phrase("CITATION CAPITAL", "citation"), None);
        assert_eq!(
            title_phrase("CITATION: R. v. HARLOW, 2021 ONCA 123", "citation"),
            None
        );
        assert_eq!(
            title_phrase(
                "Subject: UPS Ship Notification, 1Z7261100192688747",
                "notification"
            )
            .as_deref(),
            Some("Ship Notification")
        );
        // A kind that sets a name apart names one wherever it stands.
        assert!(has_a_kind(&words("Notice Board Services")));
    }

    /// A sentence begins after a kind where a verb follows it; a notice
    /// given "hereby" is its title's own sentence.
    #[test]
    fn a_verb_after_a_kind_begins_a_sentence() {
        let w = |text: &str| words(text);
        assert!(begins_a_sentence(
            &w("The report on the works is provided below"),
            5
        ));
        assert!(begins_a_sentence(&w("Invoice will follow"), 1));
        assert!(!begins_a_sentence(&w("NOTICE IS HEREBY GIVEN"), 1));
        assert!(!begins_a_sentence(&w("Notice hereby given"), 1));
        assert!(!begins_a_sentence(&w("Report on the Consultation"), 1));
        assert!(!begins_a_sentence(&w("Report"), 1));
    }

    /// A word that names what a number belongs to - a case, a docket, a
    /// file, an index - standing between a title's kind (or a stop) and
    /// the number's label is no part of the title, on the line or on a
    /// type copied from it. Any other word there is the title's own, and
    /// so is a case or a cause after a word that is no kind ("Notice of
    /// Related Case", "Order to Show Cause"), set apart from the number by
    /// a stop, or before a "No" that labels no figure.
    #[test]
    fn a_numbers_label_is_no_part_of_the_title_before_it() {
        for (line, head, phrase) in [
            (
                "Re: Notice of Civil Penalty Assessment and Order Case No. WQ-SW-WR-2023-626",
                "notice",
                "Notice of Civil Penalty Assessment and Order",
            ),
            (
                "Re: Notice of Civil Penalty Assessment and Order Case No.",
                "notice",
                "Notice of Civil Penalty Assessment and Order",
            ),
            (
                "NOTICE OF VIOLATION AND ORDER DOCKET NO. 24-0113",
                "notice",
                "Notice of Violation and Order",
            ),
            (
                "Notice of Intent to Revoke Permit File #4471",
                "notice",
                "Notice of Intent to Revoke Permit",
            ),
            (
                "Notice of Motion Index No. 652113/2024",
                "notice",
                "Notice of Motion",
            ),
            // After a stop, whatever the word before it.
            (
                "Notice of Violation, Case No. 24-0113",
                "notice",
                "Notice of Violation",
            ),
            (
                "Notice of Violation and Order, Case No. 24-0113",
                "notice",
                "Notice of Violation and Order",
            ),
            // The title's own word after a kind, before a number.
            (
                "Notice of Lease Violation No. 4",
                "notice",
                "Notice of Lease Violation",
            ),
            (
                "NOTICE OF CONTRACT AWARD NO. 2025-15",
                "notice",
                "Notice of Contract Award",
            ),
            (
                "NOTICE OF LICENSE SUSPENSION NO. 2025-0412",
                "notice",
                "Notice of License Suspension",
            ),
            (
                "Notice of Permit Violation No. 12",
                "notice",
                "Notice of Permit Violation",
            ),
            (
                "Notice of Prior Authorization Approval No. 12345",
                "notice",
                "Notice of Prior Authorization Approval",
            ),
            (
                "Notice of Policy Cancellation No. 7",
                "notice",
                "Notice of Policy Cancellation",
            ),
            (
                "Report of Audit Findings No. 2",
                "report",
                "Report of Audit Findings",
            ),
            // A case or a cause after a word that is no kind.
            (
                "Notice of Related Case No. 3:21-cv-00412",
                "notice",
                "Notice of Related Case",
            ),
            (
                "Order to Show Cause No. 2024-15",
                "order",
                "Order to Show Cause",
            ),
            // Set apart from the number, or before a "No" with no figure.
            (
                "Notice of Intent to Close Permit File, No. 2024-15",
                "notice",
                "Notice of Intent to Close Permit File",
            ),
            (
                "Notice of Intent to Close Permit File No Further Action Required",
                "notice",
                "Notice of Intent to Close Permit File",
            ),
            (
                "Statement of Account No. 1182",
                "statement",
                "Statement of Account",
            ),
            ("Notice of Claim No. 77", "notice", "Notice of Claim"),
            (
                "Notice of Default and Reservation of Rights No. 5",
                "notice",
                "Notice of Default and Reservation of Rights",
            ),
        ] {
            assert_eq!(title_phrase(line, head).as_deref(), Some(phrase), "{line}");
        }
        for (value, kind) in [
            (
                "Notice of Civil Penalty Assessment and Order Case No. WQ/M-ER-2024-037",
                "Notice of Civil Penalty Assessment and Order",
            ),
            (
                "Notice of Violation and Order Docket No. 24-0113",
                "Notice of Violation and Order",
            ),
            (
                "Notice of Violation and Order, Case No. 24-0113",
                "Notice of Violation and Order",
            ),
            (
                "Notice of Intent to Revoke Permit File #4471",
                "Notice of Intent to Revoke Permit",
            ),
            ("Purchase Order Reference No. 55", "Purchase Order"),
            ("Notice of Related Case No. 3", "Notice of Related Case"),
            ("Statement of Account No. 1182", "Statement of Account"),
            ("Capital Call Notice No. 7", "Capital Call Notice"),
            // The type's own word after a kind, before a number.
            (
                "Notice of Contract Award No. 2025-15",
                "Notice of Contract Award",
            ),
            ("Contract Modification No. P00003", "Contract Modification"),
            ("Purchase Order Change No. 3", "Purchase Order Change"),
            ("Invoice Adjustment No. 3", "Invoice Adjustment"),
            ("Lease Extension No. 2", "Lease Extension"),
            ("Permit Modification No. 4", "Permit Modification"),
            ("Policy Endorsement No. 3", "Policy Endorsement"),
            ("Policy Cancellation No. 9", "Policy Cancellation"),
        ] {
            assert_eq!(without_identifier(value), kind, "{value}");
        }
        // A kind numbered as its kind keeps its number, whatever precedes.
        assert_eq!(without_identifier("Amendment No. 3"), "Amendment No. 3");
    }
}
