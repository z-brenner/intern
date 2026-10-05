//! Filename composition.
//!
//! Intern's whole visible output is one line of text in a folder listing, so
//! the shape is fixed and readable:
//!
//! ```text
//! YYYY-MM-DD <what the document is> <who it is between>.<original extension>
//! 2026-12-29 Notice of Termination for John Smith.pdf
//! 2026-04-01 Statement of Work between Acme and Contoso.pdf
//! ```
//!
//! When the name would be too long to scan, detail is shed from the least
//! identifying end first: the second party, then the party clause, then the
//! document type.
//!
//! A type or a party printed in capitals - a letterhead, an OCR'd scan - is
//! title-cased for the name (`display_case`); everything else Intern keeps
//! about the document, the evidence and the description included, keeps the
//! document's own casing.

use std::collections::HashSet;

use unicode_normalization::UnicodeNormalization;

use crate::domain::{ComposedName, PartyRelation, ValidatedProposal};

/// Long enough to stay specific, short enough to read in a folder listing.
pub const MAX_FILENAME_CHARS: usize = 120;
const MIN_STEM_CHARS: usize = 4;
/// What a name says where the document type should be when there is none.
pub(crate) const DEFAULT_TYPE: &str = "Document";

pub fn compose_filename(
    proposal: &ValidatedProposal,
    extension: &str,
    existing_names: &[&str],
) -> ComposedName {
    let extension = sanitize_extension(extension);
    let date = proposal
        .document_date
        .as_deref()
        .and_then(sanitize_segment)
        .unwrap_or_default();
    let document_type = type_segment(proposal.document_type.as_deref(), &extension);
    let parties = proposal
        .parties
        .iter()
        .filter_map(|party| party_segment(party))
        .collect::<Vec<_>>();

    let existing = existing_names
        .iter()
        .map(|value| windows_name_key(value))
        .collect::<HashSet<_>>();

    let mut collision_index = 1;
    loop {
        let suffix = if collision_index == 1 {
            String::new()
        } else {
            format!(" ({collision_index})")
        };
        let value = fit(
            &date,
            &document_type,
            &parties,
            proposal.party_relation,
            &suffix,
            &extension,
        );
        if !existing.contains(&windows_name_key(&value)) {
            return ComposedName {
                value,
                collision_index,
            };
        }
        collision_index += 1;
    }
}

/// Builds `date + type + party clause`, shedding detail until it fits.
fn fit(
    date: &str,
    document_type: &str,
    parties: &[String],
    relation: PartyRelation,
    suffix: &str,
    extension: &str,
) -> String {
    let extension_part = if extension.is_empty() {
        String::new()
    } else {
        format!(".{extension}")
    };
    let reserved = suffix.chars().count() + extension_part.chars().count();
    let available = MAX_FILENAME_CHARS
        .saturating_sub(reserved)
        .max(MIN_STEM_CHARS);
    // Validation turns "between" into "with" when fewer than two parties
    // validate, but a party can still go after it: a house-style rule can
    // merge two of the document's spellings into one, and a name can
    // sanitise to nothing. "between Acme" says the document has a second
    // side it does not name.
    let relation = if parties.len() < 2 {
        single_party_relation(relation)
    } else {
        relation
    };

    let attempts = [
        stem(date, document_type, parties, relation),
        stem(
            date,
            document_type,
            parties
                .first()
                .map(std::slice::from_ref)
                .unwrap_or_default(),
            single_party_relation(relation),
        ),
        stem(date, document_type, &[], PartyRelation::None),
    ];
    for attempt in &attempts {
        if attempt.chars().count() <= available {
            return format!("{attempt}{suffix}{extension_part}");
        }
    }
    let mut truncated = attempts
        .last()
        .cloned()
        .unwrap_or_default()
        .chars()
        .take(available)
        .collect::<String>();
    while truncated.ends_with(' ') || truncated.ends_with('.') {
        truncated.pop();
    }
    if truncated.is_empty() {
        truncated.push_str("Document");
    }
    format!("{truncated}{suffix}{extension_part}")
}

/// "between" only makes sense with two sides.
fn single_party_relation(relation: PartyRelation) -> PartyRelation {
    match relation {
        PartyRelation::Between => PartyRelation::With,
        other => other,
    }
}

fn stem(date: &str, document_type: &str, parties: &[String], relation: PartyRelation) -> String {
    let mut value = String::new();
    if !date.is_empty() {
        value.push_str(date);
        value.push(' ');
    }
    value.push_str(document_type);
    if let Some(clause) = party_clause(parties, relation) {
        value.push(' ');
        value.push_str(&clause);
    }
    value.trim().to_owned()
}

fn party_clause(parties: &[String], relation: PartyRelation) -> Option<String> {
    if parties.is_empty() {
        return None;
    }
    // Only "between" joins two names. A notice is "for John Smith", not "for
    // John Smith and the company that sent it"; an invoice is "from Acme", not
    // "from Acme and the customer".
    let names = match (relation, parties) {
        (PartyRelation::Between, [first, second, ..]) => format!("{first} and {second}"),
        (_, [only, ..]) => only.clone(),
        (_, []) => return None,
    };
    // A validated party is identifying information the filename exists to carry,
    // so an unstated relation costs the connecting word, not the name. The model
    // called a termination notice's subject `none` and "John Smith" vanished from
    // the filename entirely, which is the one thing that must not happen to a
    // fact that survived validation.
    //
    // A separator rather than a guessed preposition: "from" would be wrong on an
    // invoice the party was billed for, and asserting a relationship the document
    // did not state is exactly what the rest of this pipeline refuses to do.
    if relation == PartyRelation::None {
        return Some(format!("- {names}"));
    }
    Some(format!("{} {names}", relation.as_str()))
}

pub(crate) fn sanitize_extension(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('.')
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .take(16)
        .collect()
}

pub(crate) fn strip_duplicate_extension<'a>(value: &'a str, extension: &str) -> &'a str {
    if extension.is_empty() {
        return value;
    }
    let mut stripped = value;
    let suffix_length = extension.len() + 1;
    loop {
        let Some(start) = stripped.len().checked_sub(suffix_length) else {
            break;
        };
        let Some(suffix) = stripped.get(start..) else {
            break;
        };
        if suffix.starts_with('.') && suffix[1..].eq_ignore_ascii_case(extension) {
            let Some(prefix) = stripped.get(..start) else {
                break;
            };
            stripped = prefix;
        } else {
            break;
        }
    }
    stripped
}

/// A folder name Windows accepts, built from a fact the way filename segments
/// are: hostile characters dropped, whitespace collapsed, trailing dots and
/// spaces removed, reserved device names escaped. `None` when nothing is
/// left.
///
/// The trailing dots and spaces are removed again after the cut to 80
/// characters, which can land just after one. The queue creates folders
/// through verbatim `\\?\` paths, which skip the trimming Win32 would
/// otherwise do, so a folder named "Acme Holdings " would be created as
/// written - and Explorer mishandles it and OneDrive refuses to sync it. The
/// cut cannot leave a reserved device name behind: the part of the name
/// before its first dot, which is what makes a name reserved, was already
/// checked whole.
pub fn sanitize_folder_name(value: &str) -> Option<String> {
    sanitize_segment(value).and_then(|name| {
        let mut cut = name.chars().take(80).collect::<String>();
        while cut.ends_with([' ', '.']) {
            cut.pop();
        }
        (!cut.is_empty()).then_some(cut)
    })
}

/// The document type as a filename carries it: the extension a model
/// sometimes appends dropped, made safe for a filename, title-cased when it
/// was printed in capitals, and "Document" when there is none. House style
/// reads proposed names back with this, so it must stay the one place the
/// type segment is built.
pub(crate) fn type_segment(document_type: Option<&str>, extension: &str) -> String {
    document_type
        .map(|value| strip_duplicate_extension(value, extension))
        .and_then(sanitize_segment)
        .map(|segment| display_case(&segment))
        .unwrap_or_else(|| DEFAULT_TYPE.to_owned())
}

/// A party as a filename carries it, or `None` when nothing printable is
/// left of the name. The counterpart of [`type_segment`].
pub(crate) fn party_segment(party: &str) -> Option<String> {
    sanitize_segment(party).map(|segment| display_case(&segment))
}

/// Suffixes a company name writes in capitals that read in mixed case.
const SUFFIXES: &[(&str, &str)] = &[
    ("INC", "Inc"),
    ("CORP", "Corp"),
    ("CO", "Co"),
    ("LTD", "Ltd"),
    ("LIMITED", "Limited"),
    ("GMBH", "GmbH"),
];

/// Suffixes that are initialisms and stay in capitals.
const CAPITAL_SUFFIXES: &[&str] = &["LLC", "LLP", "PLC", "PC", "NA", "LP"];

/// The small words a title keeps in lower case after its first word.
const CONNECTORS: &[&str] = &["of", "and", "the", "for", "to", "with"];

/// Title-cases a segment a letterhead or a scan printed in capitals:
/// "ORION GLASS STUDIO INC" names a document "Orion Glass Studio Inc", which
/// reads like every other name in the folder.
///
/// Only a segment with no lowercase letter and at least two words of four or
/// more letters is touched. A single capitalised word - "IBM", "NASA", "KPMG
/// LLP" - is more likely an initialism than shouting, and a segment with any
/// lowercase letter already says how it wants to be written. Inside a
/// segment that qualifies, word by word:
///
/// - a word holding a digit, an apostrophe, or starting "MC" or "MAC" is
///   left alone: "O'BRIEN" and "MCDONALD" have capitals in the middle that
///   only the person who owns the name knows, and "Mcdonald" would be wrong;
/// - a company suffix reads the way it is usually written ("INC." becomes
///   "Inc.", "GMBH" becomes "GmbH"), and LLC, LLP, PLC, PC, NA and LP stay
///   in capitals;
/// - of, and, the, for, to and with are lower case, except as the first
///   word, where they are capitalised like any other word;
/// - a word of three letters or fewer stays in capitals ("ABC", "USA"), and
///   so does a longer word with no vowel, which is an initialism ("HSBC");
/// - every other word keeps its first letter and lowercases the rest, after
///   a hyphen or other mark as well ("COCA-COLA" becomes "Coca-Cola").
///
/// Y counts as a vowel, so "LYNCH" and "FLYNN" are words, not initialisms.
pub(crate) fn display_case(segment: &str) -> String {
    if segment.chars().any(char::is_lowercase) {
        return segment.to_owned();
    }
    let long_words = segment
        .split_whitespace()
        .filter(|word| {
            !word.chars().any(char::is_numeric)
                && word
                    .chars()
                    .filter(|character| character.is_alphabetic())
                    .count()
                    >= 4
        })
        .count();
    if long_words < 2 {
        return segment.to_owned();
    }
    segment
        .split(' ')
        .enumerate()
        .map(|(index, word)| display_word(word, index == 0))
        .collect::<Vec<_>>()
        .join(" ")
}

/// One word of a segment [`display_case`] rewrites, with the punctuation
/// around it kept where it was: "INC." is "INC" between "" and ".".
fn display_word(word: &str, first: bool) -> String {
    let Some(start) = word.find(char::is_alphanumeric) else {
        return word.to_owned();
    };
    let end = word
        .char_indices()
        .rev()
        .find(|(_, character)| character.is_alphanumeric())
        .map_or(word.len(), |(index, character)| {
            index + character.len_utf8()
        });
    let (lead, core, trail) = (&word[..start], &word[start..end], &word[end..]);
    if core.chars().any(char::is_numeric)
        || word.contains(['\'', '\u{2019}', '\u{02bc}'])
        || core.starts_with("MC")
        || core.starts_with("MAC")
    {
        return word.to_owned();
    }
    let lowered = core.to_lowercase();
    let letters = core
        .chars()
        .filter(|character| character.is_alphabetic())
        .count();
    let cased = if let Some((_, suffix)) = SUFFIXES.iter().find(|(from, _)| *from == core) {
        (*suffix).to_owned()
    } else if CAPITAL_SUFFIXES.contains(&core) {
        core.to_owned()
    } else if CONNECTORS.contains(&lowered.as_str()) {
        if first {
            capitalize_runs(core)
        } else {
            lowered
        }
    } else if letters <= 3 || is_initialism(core) {
        core.to_owned()
    } else {
        capitalize_runs(core)
    };
    format!("{lead}{cased}{trail}")
}

/// A word of capitals with no vowel - "HSBC", "KPMG" - is letters, not a
/// word. A letter outside ASCII is taken for a vowel, because the word is
/// then not an English initialism either.
fn is_initialism(word: &str) -> bool {
    word.chars()
        .filter(|character| character.is_alphabetic())
        .all(|character| {
            character.is_ascii_alphabetic()
                && !matches!(
                    character.to_ascii_uppercase(),
                    'A' | 'E' | 'I' | 'O' | 'U' | 'Y'
                )
        })
}

/// Keeps the first letter of every run of letters and lowercases the rest of
/// it, so each part of "COCA-COLA" or "SMITH/JONES" reads as a word.
fn capitalize_runs(word: &str) -> String {
    let mut output = String::with_capacity(word.len());
    let mut run = String::new();
    let flush = |output: &mut String, run: &mut String| {
        let mut characters = run.chars();
        if let Some(first) = characters.next() {
            output.push(first);
            output.push_str(&characters.as_str().to_lowercase());
        }
        run.clear();
    };
    for character in word.chars() {
        if character.is_alphabetic() {
            run.push(character);
        } else {
            flush(&mut output, &mut run);
            output.push(character);
        }
    }
    flush(&mut output, &mut run);
    output
}

/// Typographic ligatures and full-width forms are the letters a person types,
/// drawn differently. A PDF that sets "Office" with the "ffi" ligature
/// (U+FB03) or an East Asian form that writes "ＡＣＭＥ" in full-width
/// letters gives a filename nobody can search for by typing it, so both are
/// folded to plain ASCII before anything else looks at the name - which also
/// lets the hostile-character check see a full-width "：" as the colon it is.
fn fold_presentation_forms(value: &str) -> String {
    let mut folded = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\u{fb00}' => folded.push_str("ff"),
            '\u{fb01}' => folded.push_str("fi"),
            '\u{fb02}' => folded.push_str("fl"),
            '\u{fb03}' => folded.push_str("ffi"),
            '\u{fb04}' => folded.push_str("ffl"),
            '\u{fb05}' | '\u{fb06}' => folded.push_str("st"),
            '\u{ff01}'..='\u{ff5e}' => {
                folded.push(char::from_u32(character as u32 - 0xfee0).unwrap_or(character));
            }
            other => folded.push(other),
        }
    }
    folded
}

/// A filename segment: presentation forms folded, hostile and invisible
/// characters dropped, whitespace collapsed, composed to NFC, trailing dots
/// and spaces removed, reserved device names escaped. NFC because macOS and
/// some PDFs hand over "é" as "e" and a combining accent, and the two
/// spellings of one name must make one filename.
pub(crate) fn sanitize_segment(value: &str) -> Option<String> {
    let folded = fold_presentation_forms(value);
    let mut output = String::new();
    let mut pending_space = false;
    for character in folded.chars() {
        if character.is_whitespace() {
            pending_space = !output.is_empty();
            continue;
        }
        if character.is_control()
            || is_invisible_format(character)
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
        {
            continue;
        }
        if pending_space {
            output.push(' ');
            pending_space = false;
        }
        output.push(character);
    }
    // Composing never produces whitespace or one of the characters dropped
    // above, so it can follow them; it follows the invisible characters'
    // removal so that one between a letter and its accent cannot keep the
    // two apart.
    let mut output = output.nfc().collect::<String>();
    while output.ends_with(' ') || output.ends_with('.') {
        output.pop();
    }
    if output.is_empty() {
        return None;
    }
    if is_reserved_device_name(&output) {
        output.insert(0, '_');
    }
    Some(output)
}

/// Formatting characters that a filename must not carry: the bidirectional
/// controls, and the rest of the invisible ones - a soft hyphen, a
/// zero-width space, a word joiner, a byte-order mark.
///
/// They survive every visible check and produce a name nobody can type,
/// search for, or tell apart from the name beside it, which is the whole
/// point of a filename.
fn is_invisible_format(character: char) -> bool {
    matches!(
        character as u32,
        0x00ad
            | 0x0600..=0x0605
            | 0x061c
            | 0x06dd
            | 0x070f
            | 0x08e2
            | 0x180e
            | 0x200b..=0x200f
            | 0x202a..=0x202e
            | 0x2060..=0x2064
            | 0x2066..=0x206f
            | 0xfeff
            | 0xfff9..=0xfffb
            | 0x110bd
            | 0x110cd
            | 0x13430..=0x1343f
            | 0x1bca0..=0x1bca3
            | 0x1d173..=0x1d17a
            | 0xe0001
            | 0xe0020..=0xe007f
    )
}

fn is_reserved_device_name(value: &str) -> bool {
    let stem = value
        .split('.')
        .next()
        .unwrap_or(value)
        .trim()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem.strip_prefix("COM").is_some_and(is_reserved_number)
        || stem.strip_prefix("LPT").is_some_and(is_reserved_number)
}

fn is_reserved_number(value: &str) -> bool {
    value.len() == 1 && matches!(value.as_bytes()[0], b'1'..=b'9')
}

/// The form Windows compares two names in: trailing dots and spaces
/// disregarded, case folded. Composed to NFC first, because a name copied
/// from macOS can spell "Café" with a combining accent, and NTFS would hold
/// both spellings side by side as two files a person cannot tell apart.
pub fn windows_name_key(value: &str) -> String {
    value
        .trim_end_matches([' ', '.'])
        .nfc()
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DateRole, Evidence};

    fn proposal(
        date: Option<&str>,
        document_type: Option<&str>,
        parties: &[&str],
        relation: PartyRelation,
    ) -> ValidatedProposal {
        ValidatedProposal {
            document_type: document_type.map(str::to_owned),
            document_date: date.map(str::to_owned),
            date_role: date.map(|_| DateRole::Effective),
            parties: parties.iter().map(|value| (*value).to_owned()).collect(),
            party_relation: relation,
            description: "A description.".into(),
            confidence: 0.9,
            evidence: Evidence::default(),
        }
    }

    fn name(proposal: &ValidatedProposal, extension: &str) -> String {
        compose_filename(proposal, extension, &[]).value
    }

    /// Measured on the corpus: the model read the termination notice correctly,
    /// answered `John Smith`, and then called the relation `none`, and the party
    /// disappeared from the filename. A validated name is the most identifying
    /// thing the filename carries; an unstated relation costs the connecting word,
    /// not the name.
    #[test]
    fn an_unstated_relation_keeps_the_party_and_drops_only_the_connector() {
        assert_eq!(
            name(
                &proposal(
                    Some("2026-12-29"),
                    Some("Notice of Termination"),
                    &["John Smith"],
                    PartyRelation::None,
                ),
                "pdf",
            ),
            "2026-12-29 Notice of Termination - John Smith.pdf"
        );
    }

    #[test]
    fn produces_the_documented_shape() {
        assert_eq!(
            name(
                &proposal(
                    Some("2026-12-29"),
                    Some("Notice of Termination"),
                    &["John Smith"],
                    PartyRelation::For
                ),
                "pdf"
            ),
            "2026-12-29 Notice of Termination for John Smith.pdf"
        );
        assert_eq!(
            name(
                &proposal(
                    Some("2026-04-01"),
                    Some("Statement of Work"),
                    &["Acme", "Contoso"],
                    PartyRelation::Between
                ),
                "pdf"
            ),
            "2026-04-01 Statement of Work between Acme and Contoso.pdf"
        );
        assert_eq!(
            name(
                &proposal(
                    Some("2025-09-14"),
                    Some("Amendment to Consulting Agreement"),
                    &["Jane Smith"],
                    PartyRelation::With
                ),
                "pdf"
            ),
            "2025-09-14 Amendment to Consulting Agreement with Jane Smith.pdf"
        );
        assert_eq!(
            name(
                &proposal(
                    Some("2026-01-05"),
                    Some("Invoice"),
                    &["Acme Corporation"],
                    PartyRelation::From
                ),
                "pdf"
            ),
            "2026-01-05 Invoice from Acme Corporation.pdf"
        );
    }

    #[test]
    fn only_an_agreement_joins_two_names() {
        assert_eq!(
            name(
                &proposal(
                    Some("2026-12-29"),
                    Some("Notice of Termination"),
                    &["John Smith", "Northstar Lantern Works LLC"],
                    PartyRelation::For
                ),
                "pdf"
            ),
            "2026-12-29 Notice of Termination for John Smith.pdf"
        );
        assert_eq!(
            name(
                &proposal(
                    Some("2026-01-05"),
                    Some("Invoice"),
                    &["Acme Corporation", "Contoso Worldwide, Inc."],
                    PartyRelation::From
                ),
                "pdf"
            ),
            "2026-01-05 Invoice from Acme Corporation.pdf"
        );
    }

    /// Bidirectional controls were already removed, but the rest of the
    /// invisible formatting characters were not, so a party name carrying a
    /// zero-width space or a soft hyphen produced a filename nobody could
    /// type, search for, or tell apart from the one beside it.
    #[test]
    fn an_invisible_character_never_reaches_a_filename() {
        assert_eq!(
            sanitize_segment("Acme\u{00ad} Cor\u{200b}poration\u{feff}").as_deref(),
            Some("Acme Corporation")
        );
        assert_eq!(
            sanitize_segment("\u{2060}\u{200d}").as_deref(),
            None,
            "a name that is nothing but invisible characters is no name at all"
        );
    }

    #[test]
    fn the_extension_is_always_preserved() {
        let value = name(
            &proposal(
                Some("2026-01-05"),
                Some("Invoice"),
                &[],
                PartyRelation::None,
            ),
            "DOCX",
        );
        assert!(value.ends_with(".docx"));
    }

    #[test]
    fn no_parties_leaves_a_clean_name() {
        assert_eq!(
            name(
                &proposal(
                    Some("2025-05-07"),
                    Some("Meeting Minutes"),
                    &[],
                    PartyRelation::None
                ),
                "md"
            ),
            "2025-05-07 Meeting Minutes.md"
        );
    }

    #[test]
    fn a_missing_type_still_produces_a_usable_name() {
        assert_eq!(
            name(
                &proposal(Some("2025-05-07"), None, &[], PartyRelation::None),
                "pdf"
            ),
            "2025-05-07 Document.pdf"
        );
    }

    #[test]
    fn overlong_names_shed_the_second_party_before_the_type() {
        let value = name(
            &proposal(
                Some("2026-04-01"),
                Some("Master Professional Services and Technology Implementation Agreement"),
                &[
                    "Northstar Lantern Works Limited Liability Company",
                    "Copper Wren Design Incorporated",
                ],
                PartyRelation::Between,
            ),
            "pdf",
        );
        assert!(value.chars().count() <= MAX_FILENAME_CHARS);
        assert!(value.contains("Master Professional Services"));
        assert!(!value.contains("Copper Wren"));
    }

    #[test]
    fn windows_hostile_characters_and_device_names_are_neutralised() {
        let value = name(
            &proposal(
                Some("2026-04-01"),
                Some("Invoice: 3/4 <draft>"),
                &["CON"],
                PartyRelation::From,
            ),
            "pdf",
        );
        assert!(!value.contains(':') && !value.contains('/') && !value.contains('<'));
        assert!(value.contains("_CON"));
    }

    #[test]
    fn collisions_get_a_numeric_suffix() {
        let candidate = proposal(
            Some("2026-01-05"),
            Some("Invoice"),
            &[],
            PartyRelation::None,
        );
        let composed = compose_filename(&candidate, "pdf", &["2026-01-05 Invoice.pdf"]);
        assert_eq!(composed.value, "2026-01-05 Invoice (2).pdf");
        assert_eq!(composed.collision_index, 2);
    }

    /// Letterheads and OCR print names in capitals, and "Lease Agreement with
    /// ORION GLASS STUDIO INC.pdf" shouts in a folder of names that do not.
    #[test]
    fn all_caps_segments_are_title_cased_with_initialisms_and_suffixes_kept() {
        for (printed, expected) in [
            ("HARBOR COMET REPAIRS LLC", "Harbor Comet Repairs LLC"),
            ("ORION GLASS STUDIO INC", "Orion Glass Studio Inc"),
            ("ORION GLASS STUDIO INC.", "Orion Glass Studio Inc."),
            ("NOTICE OF TERMINATION", "Notice of Termination"),
            ("BANK OF THE WEST, N.A.", "Bank of the West, N.A."),
            ("THE HOME DEPOT CORP.", "The Home Depot Corp."),
            ("HSBC HOLDINGS PLC", "HSBC Holdings PLC"),
            ("MÜLLER WERKZEUG GMBH", "Müller Werkzeug GmbH"),
            (
                "NORTHWIND TRADERS LTD AND CO",
                "Northwind Traders Ltd and Co",
            ),
            ("JOHN DEERE & CO.", "John Deere & Co."),
            ("COCA-COLA BOTTLING CO", "Coca-Cola Bottling Co"),
            ("LYNCH FLYNN CONSULTING LLP", "Lynch Flynn Consulting LLP"),
            ("ABC SUPPLY WAREHOUSE", "ABC Supply Warehouse"),
            ("LINCOLN TOWER 2B HOLDINGS", "Lincoln Tower 2B Holdings"),
            // Capitals in the middle of a name are the owner's to know.
            (
                "O'BRIEN MCDONALD MACKENZIE PARTNERS",
                "O'BRIEN MCDONALD MACKENZIE Partners",
            ),
        ] {
            assert_eq!(display_case(printed), expected, "{printed}");
        }

        let caps = proposal(
            Some("2026-04-01"),
            Some("LEASE AGREEMENT"),
            &["ORION GLASS STUDIO INC."],
            PartyRelation::With,
        );
        assert_eq!(
            name(&caps, "pdf"),
            "2026-04-01 Lease Agreement with Orion Glass Studio Inc.pdf"
        );
        // Only the name changes: the proposal, its evidence, and its
        // description keep the document's own casing.
        assert_eq!(caps.parties, vec!["ORION GLASS STUDIO INC."]);
        assert_eq!(caps.document_type.as_deref(), Some("LEASE AGREEMENT"));
    }

    /// A single word in capitals is more often an initialism than shouting,
    /// and a name with any lowercase letter already says how it is written.
    #[test]
    fn single_word_or_short_caps_names_are_left_alone() {
        for unchanged in [
            "KPMG LLP",
            "IBM",
            "NASA",
            "ACME LLC",
            "DEERE & CO.",
            "AT&T INC",
            "INVOICE",
            "Acme Corporation",
            "Harbor COMET Repairs",
            "eBay Marketplace Services",
        ] {
            assert_eq!(display_case(unchanged), unchanged, "{unchanged}");
        }
        assert_eq!(
            name(
                &proposal(
                    Some("2026-04-01"),
                    Some("Invoice"),
                    &["KPMG LLP"],
                    PartyRelation::From
                ),
                "pdf"
            ),
            "2026-04-01 Invoice from KPMG LLP.pdf"
        );
    }

    /// "between" with one name says the document has a second side it does
    /// not name. A party that sanitises to nothing leaves one.
    #[test]
    fn one_surviving_party_never_reads_between() {
        assert_eq!(
            name(
                &proposal(
                    Some("2026-04-01"),
                    Some("Invoice"),
                    &["Acme"],
                    PartyRelation::Between
                ),
                "pdf"
            ),
            "2026-04-01 Invoice with Acme.pdf"
        );
        assert_eq!(
            name(
                &proposal(
                    Some("2026-04-01"),
                    Some("Invoice"),
                    &["Acme Corporation", "???"],
                    PartyRelation::Between
                ),
                "pdf"
            ),
            "2026-04-01 Invoice with Acme Corporation.pdf"
        );
    }

    /// The queue creates folders through verbatim paths, so a cut that lands
    /// just after a space or a period would create a folder Windows tools
    /// mishandle and OneDrive will not sync.
    #[test]
    fn folder_name_cut_never_ends_in_space_or_period() {
        let stem = "A".repeat(79);
        let spaced = sanitize_folder_name(&format!("{stem} Holdings LLC")).unwrap();
        assert_eq!(spaced, stem, "the 80th character was a space");
        let dotted = sanitize_folder_name(&format!("{stem}. Holdings LLC")).unwrap();
        assert_eq!(dotted, stem, "the 80th character was a period");
        assert_eq!(
            sanitize_folder_name(&format!("{}x", ". ".repeat(41))),
            None,
            "nothing is left once the cut is trimmed"
        );
        assert_eq!(
            sanitize_folder_name("Acme Holdings LLC").as_deref(),
            Some("Acme Holdings LLC")
        );
    }

    /// A ligature or a full-width letter is the same letter a person types,
    /// and the composed and decomposed spellings of an accent are one name.
    #[test]
    fn ligatures_and_fullwidth_fold_and_nfc_collides() {
        assert_eq!(
            name(
                &proposal(
                    Some("2026-04-01"),
                    Some("O\u{fb03}ce Lease"),
                    &["Paci\u{fb01}c Freight"],
                    PartyRelation::With
                ),
                "pdf"
            ),
            "2026-04-01 Office Lease with Pacific Freight.pdf"
        );
        assert_eq!(
            sanitize_segment("\u{ff21}\u{ff43}\u{ff4d}\u{ff45} \u{ff23}\u{ff4f}\u{ff52}\u{ff50}")
                .as_deref(),
            Some("Acme Corp")
        );
        // A full-width colon and solidus are hostile once folded.
        assert_eq!(
            sanitize_segment("Invoice\u{ff1a} 3\u{ff0f}4").as_deref(),
            Some("Invoice 34")
        );
        let composed = "Caf\u{e9} Rouge";
        let decomposed = "Cafe\u{301} Rouge";
        assert_eq!(sanitize_segment(decomposed).as_deref(), Some(composed));
        assert_eq!(
            windows_name_key(&format!("{composed}.pdf")),
            windows_name_key(&format!("{decomposed}.pdf"))
        );
        // An invisible character between a letter and its accent does not
        // keep them apart.
        assert_eq!(
            sanitize_segment("Cafe\u{200b}\u{301}").as_deref(),
            Some("Caf\u{e9}")
        );
        let existing = ["2026-04-01 Invoice from Cafe\u{301} Rouge.pdf"];
        let composed_name = compose_filename(
            &proposal(
                Some("2026-04-01"),
                Some("Invoice"),
                &[composed],
                PartyRelation::From,
            ),
            "pdf",
            &existing,
        );
        assert_eq!(composed_name.collision_index, 2);
    }

    #[test]
    fn a_type_carrying_the_extension_does_not_double_it() {
        let value = name(
            &proposal(
                Some("2026-01-05"),
                Some("Invoice.pdf"),
                &[],
                PartyRelation::None,
            ),
            "pdf",
        );
        assert_eq!(value, "2026-01-05 Invoice.pdf");
    }
}
