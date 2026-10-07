//! The document's own phrases, read back for facts the evidence pipeline
//! validated: a type as the document's title states it, a party's name
//! without the label or address OCR ran into it, an amount with the label
//! the document gives it.
//!
//! Everything here is pure text work over what the document says. Nothing
//! makes a fact more acceptable than validation found it: a phrase read
//! back is always words the document states, in the order it states them.

use crate::cues::{NOT_A_TITLE, ORGANISATION_ENDINGS, TYPE_NOUNS};
use crate::evidence::normalize;

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

/// The same word, a plural's "s" aside: "Minutes" states "minute".
fn same_word(word: &str, wanted: &str) -> bool {
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
];

/// Whether `word` names a kind of document.
pub(crate) fn names_a_kind(word: &str) -> bool {
    TYPE_NOUNS.contains(&word) || MORE_KINDS.contains(&word)
}

/// The noun a type phrase is a kind of: its first word that names a kind
/// of document and ends the phrase or a part of it - "*Notice* of
/// Default", "First *Amendment* to Lease", "Seed Production and Supply
/// *Agreement*", "Order *Form*" - or, when no word names a kind, the word
/// before its first connector, else its last word.
pub(crate) fn head_noun(words: &[String]) -> Option<&str> {
    let by_kind = words.iter().enumerate().find(|(at, word)| {
        names_a_kind(word)
            && words.get(at + 1).is_none_or(|next| {
                CONNECTORS.contains(&next.as_str())
                    || TITLE_STOPS.contains(&next.as_str())
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
/// does not.
pub(crate) fn has_a_kind(words: &[String]) -> bool {
    words.iter().any(|word| names_a_kind(word))
}

/// Words a title is not read back past, to its left: an article or a
/// pronoun opens a sentence that names the document, not its title.
const LEFT_STOPS: &[&str] = &["this", "the", "a", "an", "our", "your", "its"];

/// Words after which a title line stops naming the document: its date
/// ("EFFECTIVE SEPTEMBER 1, 2024", "Date reported: ..."), its page.
const TITLE_STOPS: &[&str] = &[
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
    // To the left, words written the way the noun is - a title in capitals
    // is all capitals - and never across a connector (the noun would be a
    // complement then), a company's legal form or an organisation's noun.
    let style = writing(tokens[at]);
    let mut start = at;
    while start > 0 {
        let previous = tokens[start - 1];
        let word = core(previous);
        if breaks(previous)
            || previous.ends_with(':')
            || TITLE_STOPS.contains(&word.as_str())
            || LEFT_STOPS.contains(&word.as_str())
            || CONNECTORS.contains(&word.as_str())
            || ORGANISATION_ENDINGS.contains(&word.trim_end_matches('.'))
            || writing(previous) != style
        {
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
    let mut end = at + 1;
    if !tokens[at].ends_with(':') {
        while end + 1 < tokens.len()
            && CONNECTORS.contains(&core(tokens[end]).as_str())
            && !breaks(tokens[end + 1])
            && !TITLE_STOPS.contains(&core(tokens[end + 1]).as_str())
        {
            end += 2;
            while end < tokens.len()
                && !breaks(tokens[end])
                && !tokens[end - 1].ends_with(':')
                && !TITLE_STOPS.contains(&core(tokens[end]).as_str())
                && !CONNECTORS.contains(&core(tokens[end]).as_str())
            {
                end += 1;
            }
        }
    }
    let phrase = tokens[start..end]
        .iter()
        .map(|token| {
            token
                .trim_start_matches(['#', '*', '(', '"', '\u{201c}'])
                .trim_end_matches([':', ',', ';', '.', '*', ')', '"', '\u{201d}'])
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

/// How a word is written: in capitals, with a capital, or in lower case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Writing {
    Capitals,
    Capitalised,
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
        Writing::Capitalised
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
    use super::*;

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
                "SETTLEMENT AGREEMENT AND MUTUAL RELEASE",
                "agreement",
                Some("Settlement Agreement and Mutual Release"),
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
    fn a_labelled_subject_is_its_value_and_a_party_field_is_none() {
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
}
