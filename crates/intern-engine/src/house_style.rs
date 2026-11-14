//! House style: the spellings a reviewer prefers over the document's own.
//!
//! The document says "Contoso Worldwide, Inc."; the person filing it calls it
//! "Contoso", and every name they fix says so. A rule records that
//! substitution, and naming applies it to every later document. It is applied
//! deterministically, after validation, so the evidence checks still run
//! against what the document says and the model is never asked to spell
//! anything but the document's words. Nothing here changes what Intern
//! believes about a document - only what it calls it.
//!
//! A rule is learned from an edit, never from the model: the name Intern
//! proposed and the name the reviewer approved are compared field by field,
//! and only an edit confined to one party or to the type teaches anything.
//! An edit that touches two fields, the connecting words, or the date is a
//! decision about that document, not a preference about names.

use serde::{Deserialize, Serialize};

use crate::domain::{ComposedName, PartyRelation, ValidatedProposal};
use crate::evidence::is_valid_iso_date;
use crate::naming::{
    Spelling, compose_spelled, party_segment, sanitize_extension, sanitize_segment, type_segment,
};

/// Which part of a name a rule rewrites.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// A party's name.
    Party,
    /// The document type.
    DocumentType,
}

impl RuleKind {
    pub const ALL: [Self; 2] = [Self::Party, Self::DocumentType];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Party => "party",
            Self::DocumentType => "document_type",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }
}

/// One spelling the reviewer prefers: `from` as the document writes it,
/// `to` as the reviewer wrote it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseRule {
    pub kind: RuleKind,
    pub from: String,
    pub to: String,
}

impl HouseRule {
    pub fn new(kind: RuleKind, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            kind,
            from: from.into().trim().to_owned(),
            to: to.into().trim().to_owned(),
        }
    }

    /// The form a spelling is matched under: case, punctuation, and spacing
    /// disregarded, so "Contoso Worldwide, Inc." and "CONTOSO WORLDWIDE INC"
    /// are one spelling. Words are never loosened.
    pub fn key(value: &str) -> String {
        value
            .chars()
            .filter(|character| character.is_alphanumeric() || character.is_whitespace())
            .flat_map(char::to_lowercase)
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn from_key(&self) -> String {
        Self::key(&self.from)
    }

    pub fn matches(&self, value: &str) -> bool {
        let key = Self::key(value);
        !key.is_empty() && key == self.from_key()
    }

    /// A rule that would change nothing, or that maps to nothing a name can
    /// carry, teaches nothing.
    pub fn is_meaningful(&self) -> bool {
        !self.from_key().is_empty()
            && Self::key(&self.to) != self.from_key()
            && sanitize_segment(&self.to).is_some()
    }
}

/// The rules in force, applied to a validated proposal before naming.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct HouseStyle {
    rules: Vec<HouseRule>,
}

impl HouseStyle {
    pub fn new(rules: Vec<HouseRule>) -> Self {
        Self { rules }
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn rules(&self) -> &[HouseRule] {
        &self.rules
    }

    /// Rewrites the type and the parties the rules cover. The rules that
    /// fired come back with the proposal, in the order of the fields they
    /// changed, so a reviewer can be told why a name differs from the
    /// document's words.
    pub fn apply(&self, proposal: &ValidatedProposal) -> (ValidatedProposal, Vec<HouseRule>) {
        let mut styled = proposal.clone();
        let mut applied = Vec::new();
        if let Some(document_type) = proposal.document_type.as_deref()
            && let Some(rule) = self.rule_for(RuleKind::DocumentType, document_type)
        {
            styled.document_type = Some(rule.to.clone());
            applied.push(rule.clone());
        }
        let mut parties = Vec::with_capacity(proposal.parties.len());
        for party in &proposal.parties {
            let spelled = match self.rule_for(RuleKind::Party, party) {
                Some(rule) => {
                    applied.push(rule.clone());
                    rule.to.clone()
                }
                None => party.clone(),
            };
            // Two of the document's names can be one of the reviewer's.
            if !parties
                .iter()
                .any(|existing: &String| HouseRule::key(existing) == HouseRule::key(&spelled))
            {
                parties.push(spelled);
            }
        }
        styled.parties = parties;
        // Merging two of the document's names into one leaves one side, and
        // "between" needs two; the name says "with", and the stored proposal
        // must say the same so the name it composes is the name it reads.
        if styled.party_relation == PartyRelation::Between && styled.parties.len() < 2 {
            styled.party_relation = PartyRelation::With;
        }
        (styled, applied)
    }

    fn rule_for(&self, kind: RuleKind, value: &str) -> Option<&HouseRule> {
        self.rules
            .iter()
            .find(|rule| rule.kind == kind && rule.matches(value))
    }
}

/// The name for a proposal [`HouseStyle::apply`] respelled, given the rules
/// that fired. A spelling a rule wrote is the reviewer's, and is carried
/// exactly as they typed it - "ACME WIDGETS CORP" stays in capitals - while
/// the document's own words are title-cased when printed in capitals, as
/// [`compose_filename`](crate::naming::compose_filename) does for every
/// word.
pub fn compose_styled_filename(
    styled: &ValidatedProposal,
    applied: &[HouseRule],
    extension: &str,
    existing_names: &[&str],
) -> ComposedName {
    let chosen = |kind: RuleKind| {
        applied
            .iter()
            .filter(|rule| rule.kind == kind)
            .map(|rule| rule.to.as_str())
            .collect::<Vec<_>>()
    };
    compose_spelled(
        styled,
        extension,
        existing_names,
        &chosen(RuleKind::DocumentType),
        &chosen(RuleKind::Party),
    )
}

/// What one edit teaches: the reviewer changed exactly one party or the
/// type, and nothing else, so that field's spelling is a preference.
///
/// `proposal` is the proposal the proposed name was composed from - after
/// any house style already applied to it - and `proposed` is the name as
/// Intern offered it, collision suffix and all. Both names are read without
/// their extension, their date, and any ` (2)` suffix, so a reviewer who
/// typed a date and fixed a name in one go still teaches the name.
///
/// The names are read with the grammar that composed them - type, connecting
/// word, party, `and`, party - rather than by finding the smallest edit,
/// because the smallest edit lies: "Acme and Contoso" becomes "Acme Corp and
/// Contoso Inc" by inserting text one character into the connector, and a
/// diff would credit it all to one party. A name that still ends with the
/// party clause the proposed one gave is an edit to the type, even where
/// the words added to the type begin with the clause's own connecting word
/// and so would read as a longer party too: "Application for Certification
/// for City of Victorville" for "Application for City of Victorville"
/// teaches the type "Application for Certification", as [`approved_party`]
/// reads it.
///
/// A party is never respelled into words that keep it beside a connecting
/// word or a note and more words, whatever the connecting word and on
/// either side of a "between": words on one side of it that hold the party
/// whole, as house style compares spellings ([`HouseRule::key`]), in any
/// case or punctuation and lengthened or not. Behind one, with words in
/// front that lengthen the type - "Notice to Tenants from ACME PROPERTY",
/// "Notice to Tenants to Acme Property" or "Notice to Tenants from Acme
/// Property Inc" for "Notice to Acme Property" - or in front of one, with
/// another party or a note after it: "Letter to Carl Malamud from Jane
/// Doe", "Notice to Acme Property re Lease"; a connecting word or a note's
/// start left dangling after it ("Acme Corporation re") keeps it as well.
/// Such an edit names the party and something else, a decision about this
/// document, and teaches nothing: learned, "Tenants from ACME PROPERTY"
/// would name every later document giving that party, ready ones filed
/// without review.
///
/// Nor is a party respelled into words that hold a connecting word or a
/// note's start more often than its own name does, as a whole word as house
/// style compares spellings ([`HouseRule::key`]) - "for", "from", "to",
/// "with", "between", "re", "regarding", "about", "concerning" - or, for
/// " - " and " (", as written: whether or not the words still hold the
/// party in some spelling, they name another party, or the party and
/// something else. "Trustee for Acme Corp" for "Acme Corporation", a party
/// the document names typed short, and "Tenants from City of Boston" for
/// "Acme Property", words the document gives as no party at all, teach
/// nothing. So do words that join two parties outside the name's grammar
/// ("Smith v Acme Corp", "vs", "versus", "Jane Doe on behalf of Acme Corp",
/// "Acme Corp by its Trustee", "dba", "fka", "attn"), and "and" or "&" on a
/// side of a "between", or beside a party of two words or more kept whole
/// ("Acme Corporation and Jane Doe"). A dash is found however it was typed
/// ("–", "--") and a space whatever it is; " (" is a note only where its
/// parenthetical ends the words ("Acme (signed)"), so "Acme (UK) Ltd" is a
/// spelling; and a word that ends the words introduces nothing ("Swiss
/// Re"). "and" and "of" are otherwise no such words, being as common inside
/// a party's name ("Johnson and Johnson", "Friends of City of Victorville")
/// as between two. The price is a party respelled on purpose into such
/// words ("CDC" into "Centers for Disease Control", "Acme" into "Acme
/// (UK)", "Acme Property" into "Acme Property and Casualty"), which teaches
/// nothing and is made again by hand; a type may hold most of them
/// ("Application for Certification"). What the reviewer put in the party's
/// place is still read for filing (see [`respelled_party`]).
///
/// A type is never lengthened into one given a party (see
/// [`is_a_spelling`]): with the divider or "between" ("Employment
/// Agreement - Jane Doe" in front of the party clause kept), or a party
/// added behind a connecting word to a name that gave none ("Employment
/// Agreement - Contoso Ltd", "Employment Agreement for Contoso Ltd"), which
/// [`approved_party`] reads as that party. Learned, it would name every
/// later document of the type for that party. The price is a name that gave
/// no party lengthened on purpose with a connecting word ("Invoice for
/// March"), made again by hand.
pub fn lesson_from_edit(
    proposal: &ValidatedProposal,
    extension: &str,
    proposed: &str,
    approved: &str,
) -> Option<HouseRule> {
    read_edit(proposal, extension, proposed, approved, PartyClause::lesson)
}

/// The party the name `approved` respells, as [`lesson_from_edit`] reads
/// the edit, and the words put in its place, whether or not they teach: a
/// party respelled into words that hold another connecting word or a note
/// is no spelling to learn, but they are what the reviewer named the
/// document for - "Trustee for Acme Corp" for "Acme Corporation", or one
/// side of a "between" - and it is filed and reported under them. `None`
/// where [`lesson_from_edit`] reads no respelled party: the words are the
/// party's own in another case or punctuation, the type changed, the
/// party was kept beside a connecting word or a note and more words, or
/// the name is not one `proposal` composes.
pub fn respelled_party(
    proposal: &ValidatedProposal,
    extension: &str,
    proposed: &str,
    approved: &str,
) -> Option<HouseRule> {
    read_edit(
        proposal,
        extension,
        proposed,
        approved,
        PartyClause::respelling,
    )
    .filter(|edit| edit.kind == RuleKind::Party)
}

/// The edit [`lesson_from_edit`] reads, with `party` reading a changed
/// party clause.
fn read_edit(
    proposal: &ValidatedProposal,
    extension: &str,
    proposed: &str,
    approved: &str,
    party: impl FnOnce(&PartyClause, &str) -> Option<HouseRule>,
) -> Option<HouseRule> {
    let extension = sanitize_extension(extension);
    let before = editable_stem(proposed, &extension);
    let after = editable_stem(approved, &extension);
    if before == after || after.is_empty() {
        return None;
    }
    let shape = NameShape::of(proposal, &before, &extension)?;
    let rule = match shape.clause {
        // A party added after the type of a name that gave none, behind a
        // connecting word ("Employment Agreement - Contoso Ltd", "... for
        // Contoso Ltd"), as `approved_party` reads it: not a longer type.
        None if adds_a_party(&after, &shape.type_segment) => return None,
        None => HouseRule::new(
            RuleKind::DocumentType,
            proposal.document_type.clone()?,
            after,
        ),
        Some(clause) => {
            let connector = format!(" {} ", clause.connector);
            let type_unchanged = after
                .strip_prefix(shape.type_segment.as_str())
                .and_then(|rest| rest.strip_prefix(connector.as_str()));
            let clause_unchanged = after
                .strip_suffix(clause.text.as_str())
                .and_then(|rest| rest.strip_suffix(connector.as_str()));
            match (type_unchanged, clause_unchanged) {
                // The party clause kept whole, so the type changed - also
                // where it was lengthened with the clause's own connecting
                // word, which reads as a longer party too ("Application for
                // Certification for City of Victorville").
                (_, Some(new_type)) => HouseRule::new(
                    RuleKind::DocumentType,
                    proposal.document_type.clone()?,
                    new_type,
                ),
                // The words after the type and the connecting word changed:
                // a party respelled, unless they keep it beside a connecting
                // word or a note and more words (see `PartyClause::lesson`).
                (Some(new_clause), None) => party(&clause, new_clause)?,
                // Both the type and the party clause changed, or the
                // connecting word did: a decision about this document.
                _ => return None,
            }
        }
    };
    // A type given a party is no spelling (see `is_a_spelling`); a party's
    // new words were asked as much by `PartyClause::lesson`.
    (rule.is_meaningful() && (rule.kind == RuleKind::Party || is_a_spelling(&rule))).then_some(rule)
}

/// Whether `rule` could be applied to every later name as a spelling: its
/// `to` holds no more of the words that set one party apart from another,
/// or from the type, than its `from` does. For a party (see
/// [`PartyClause::lesson`]), no connecting word of the name's grammar, a
/// note's start, or words that join two parties outside it ("Smith v Acme
/// Corp", "Jane Doe on behalf of Acme Corp", "Acme Corp by its Trustee"),
/// nor "and" or "&" where the words hold a party of two words or more
/// whole and another beside it ("Acme Corporation and Jane Doe"). For a
/// type, no divider, "between", or a caption's "v", "vs" or "versus" -
/// "Employment Agreement - Jane Doe" is a type given a party. A rule
/// traced back to the document's own word, after a house rule respelled
/// it, is asked this again of that word.
pub fn is_a_spelling(rule: &HouseRule) -> bool {
    match rule.kind {
        RuleKind::Party => !holds_another_separator(&rule.to, &rule.from, false),
        RuleKind::DocumentType => !TYPE_GIVEN_A_PARTY
            .iter()
            .any(|mark| mark.count(&rule.to, false) > mark.count(&rule.from, true)),
    }
}

/// Whether `name` is `type_segment`, a connecting word naming puts before
/// a party - " - " and "between" among them, in any case and however the
/// dash or the spaces were typed - and more words: a party added to a name
/// that gave none.
fn adds_a_party(name: &str, type_segment: &str) -> bool {
    name.strip_prefix(type_segment)
        .filter(|rest| rest.starts_with(char::is_whitespace))
        .is_some_and(|rest| {
            let rest = spaced(rest).to_lowercase();
            PartyRelation::ALL.into_iter().any(|relation| {
                rest.strip_prefix(connector_word(relation))
                    .and_then(|more| more.strip_prefix(' '))
                    .is_some_and(|more| !HouseRule::key(more).is_empty())
            })
        })
}

/// The parties the name `proposed` gives, each exactly as the name writes
/// it, in the order it gives them: the one after its connecting word, or
/// the two sides of a "between". Read with the grammar that composed the
/// name, as [`lesson_from_edit`] reads it, because a type can hold the same
/// small words as a party ("Application for Certification for the City of
/// Victorville"). Empty when the name gives no party, or is not one
/// `proposal` composes - cut short for length, or typed by hand - so a
/// party a reviewer chooses is put in place of these words only where the
/// name is known to end with them.
///
/// `proposal` is the proposal the name was composed from, after any house
/// style applied to it, as for [`lesson_from_edit`].
pub fn named_parties(proposal: &ValidatedProposal, extension: &str, proposed: &str) -> Vec<String> {
    let extension = sanitize_extension(extension);
    let stem = editable_stem(proposed, &extension);
    NameShape::of(proposal, &stem, &extension)
        .and_then(|shape| shape.clause)
        .map(|clause| {
            clause
                .parties
                .into_iter()
                .map(|(_, segment)| segment)
                .collect()
        })
        .unwrap_or_default()
}

/// The one party the name `approved` gives, where it keeps the type of the
/// name `proposed` and changes no more than what follows it: the words
/// after its connecting word, which may be another word than proposed - a
/// party chosen in review turns "to" into "from" as often as not - or a
/// " - " that adds a party to a name that gave none. Read with the grammar
/// that composed `proposed`, so a type holding "for" is kept whole.
///
/// `None` where `approved` no longer starts with the proposed type, gives no
/// party, or gives two ("between"), and where `proposed` is not a name
/// `proposal` composes; `proposal` is as for [`lesson_from_edit`]. `None`
/// too where `approved` keeps a party the proposed name gave beside a
/// connecting word or a note and more words, as [`lesson_from_edit`] reads
/// such an edit: where it lengthens the type in front of that party,
/// behind the same connecting word or another - "Notice to Tenants from
/// Acme Property", "... from ACME PROPERTY" or "... from Acme Property
/// Inc" for "Notice from Acme Property" or for "Notice to Acme Property"
/// is an edit to the type, and "Tenants from Acme Property" no party - and
/// where it puts another party or a note after it: "Letter to Carl Malamud
/// from Jane Doe" gives no party "Carl Malamud from Jane Doe". A type
/// lengthened in front of a party that changed into words not holding it
/// ("Notice to Tenants from City of Boston") reads as that party's words
/// all the same - "Tenants from City of Boston" - since nothing here says
/// which parties the document names; as a respelling such words teach
/// nothing ([`lesson_from_edit`]), but they are what the name gives.
pub fn approved_party(
    proposal: &ValidatedProposal,
    extension: &str,
    proposed: &str,
    approved: &str,
) -> Option<String> {
    let extension = sanitize_extension(extension);
    let shape = NameShape::of(proposal, &editable_stem(proposed, &extension), &extension)?;
    let after = editable_stem(approved, &extension);
    // A type lengthened in front of the party it kept, behind the same
    // connecting word or another ("Notice to Tenants from Acme Property"
    // for "Notice from Acme Property" or "Notice to Acme Property"), is an
    // edit to the type: no party is read.
    if let Some(clause) = shape.clause.as_ref()
        && let Some(kept_type) = kept_behind_a_connector(&after, &clause.text)
        && HouseRule::key(kept_type) != HouseRule::key(&shape.type_segment)
    {
        return None;
    }
    let rest = after
        .strip_prefix(shape.type_segment.as_str())?
        .strip_prefix(' ')?;
    let party = PartyRelation::ALL
        .into_iter()
        .filter(|relation| *relation != PartyRelation::Between)
        .find_map(|relation| {
            rest.strip_prefix(connector_word(relation))?
                .strip_prefix(' ')
        })
        .map(str::trim)
        .filter(|party| !party.is_empty())?;
    // The party kept with another, or a note, after it ("Carl Malamud from
    // Jane Doe" for "Carl Malamud") is no one party.
    let kept_with_more = shape.clause.iter().any(|clause| {
        clause
            .parties
            .iter()
            .any(|(from, segment)| clause.keeps(party, from, segment))
    });
    (!kept_with_more).then(|| party.to_owned())
}

/// Whether the name `approved` is the name `proposed` as review reads an
/// edit: the same once the extension, the date and any collision suffix
/// are set aside, as [`lesson_from_edit`] compares them. A name approved as
/// Intern proposed it - unchanged, filed without review, or with only its
/// date or a collision suffix different - edits nothing.
pub fn same_stem(extension: &str, proposed: &str, approved: &str) -> bool {
    let extension = sanitize_extension(extension);
    editable_stem(proposed, &extension) == editable_stem(approved, &extension)
}

/// The party the name `proposed` gives, read as a name approved as
/// proposed (see [`same_stem`]) has been read since before review edits
/// were read for filing (alpha.17): the words after the type and a
/// connecting word other than "between", whole. `None` for a name that
/// gives no party, or two, and for one `proposal` does not compose;
/// `proposal` is as for [`lesson_from_edit`].
///
/// It is [`approved_party`] as alpha.17 had it, for a name nobody edited,
/// and none of what reads an edit since - a note after a party, a party
/// kept with more words, a choice made in review - comes into it, so a
/// name filed as proposed is filed exactly as it was: "Swiss Re Ltd" and
/// "Citizens Concerning the Environment" whole.
pub fn proposed_party(
    proposal: &ValidatedProposal,
    extension: &str,
    proposed: &str,
) -> Option<String> {
    let extension = sanitize_extension(extension);
    let stem = editable_stem(proposed, &extension);
    let shape = NameShape::of(proposal, &stem, &extension)?;
    if let Some(clause) = shape.clause.as_ref()
        && let Some(kept_type) = stem
            .strip_suffix(clause.text.as_str())
            .and_then(|rest| rest.strip_suffix(format!(" {} ", clause.connector).as_str()))
        && kept_type != shape.type_segment
    {
        return None;
    }
    let rest = stem
        .strip_prefix(shape.type_segment.as_str())?
        .strip_prefix(' ')?;
    PartyRelation::ALL
        .into_iter()
        .filter(|relation| *relation != PartyRelation::Between)
        .find_map(|relation| {
            rest.strip_prefix(connector_word(relation))?
                .strip_prefix(' ')
        })
        .map(str::trim)
        .filter(|party| !party.is_empty())
        .map(str::to_owned)
}

/// The composed name's own grammar, matched against the stem Intern offered,
/// so each part can be told apart in the name the reviewer approved.
struct NameShape {
    type_segment: String,
    clause: Option<PartyClause>,
}

struct PartyClause {
    connector: &'static str,
    text: String,
    /// The parties in the clause, as the proposal spells them, with the
    /// sanitized segment each contributed to the name.
    parties: Vec<(String, String)>,
}

impl NameShape {
    /// `None` when the stem is not one this proposal composes - it was
    /// truncated for length, or came from somewhere else - because guessing
    /// which words are which would teach the wrong thing.
    ///
    /// The segments are built exactly as naming builds them, in every
    /// [`Spelling`] a name can carry a word in: the document's words
    /// title-cased, a house-style rule's spelling as the reviewer typed it,
    /// and the way alpha.10 wrote both, which also said "between" before a
    /// party a merge had left alone. A name Intern proposed for a document
    /// printed in capitals, one carrying a spelling a reviewer typed in
    /// capitals, and one proposed before an upgrade and still waiting are
    /// all names this proposal composes.
    fn of(proposal: &ValidatedProposal, stem: &str, extension: &str) -> Option<Self> {
        let types = every_spelling(|spelling| {
            Some(type_segment(
                proposal.document_type.as_deref(),
                extension,
                spelling,
            ))
        });
        let parties = proposal
            .parties
            .iter()
            .map(|party| {
                (
                    party.clone(),
                    every_spelling(|spelling| party_segment(party, spelling)),
                )
            })
            .filter(|(_, spelled)| !spelled.is_empty())
            .collect::<Vec<_>>();
        let single = match proposal.party_relation {
            PartyRelation::Between => PartyRelation::With,
            other => other,
        };
        let mut single_connectors = vec![connector_word(single)];
        // Alpha.10 kept "between" before the one party a house-style merge
        // left, and a name it proposed can still be waiting.
        if single == PartyRelation::With {
            single_connectors.push(connector_word(PartyRelation::Between));
        }
        for type_segment in types {
            let shape = |clause: PartyClause| {
                Some(Self {
                    type_segment: type_segment.clone(),
                    clause: Some(clause),
                })
            };
            // The same order naming sheds detail in: both parties, one, none.
            if let [(first, first_spellings), (second, second_spellings), ..] = parties.as_slice()
                && proposal.party_relation == PartyRelation::Between
            {
                let connector = connector_word(PartyRelation::Between);
                for first_segment in first_spellings {
                    for second_segment in second_spellings {
                        let text = format!("{first_segment} and {second_segment}");
                        if stem == format!("{type_segment} {connector} {text}") {
                            return shape(PartyClause {
                                connector,
                                text,
                                parties: vec![
                                    (first.clone(), first_segment.clone()),
                                    (second.clone(), second_segment.clone()),
                                ],
                            });
                        }
                    }
                }
            }
            if let Some((first, first_spellings)) = parties.first() {
                for &connector in &single_connectors {
                    for segment in first_spellings {
                        if stem == format!("{type_segment} {connector} {segment}") {
                            return shape(PartyClause {
                                connector,
                                text: segment.clone(),
                                parties: vec![(first.clone(), segment.clone())],
                            });
                        }
                    }
                }
            }
            if stem == type_segment {
                return Some(Self {
                    type_segment,
                    clause: None,
                });
            }
        }
        None
    }
}

/// Each distinct segment `spell` writes, in the order of [`Spelling::ALL`].
fn every_spelling(spell: impl Fn(Spelling) -> Option<String>) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for segment in Spelling::ALL.into_iter().filter_map(spell) {
        if !found.contains(&segment) {
            found.push(segment);
        }
    }
    found
}

impl PartyClause {
    /// What a respelling teaches: the party the reviewer respelled (see
    /// [`Self::respelling`]), into words that are no more than a spelling
    /// of it - not words that hold a connecting word or a note's start more
    /// often than the party's own name (see [`holds_another_separator`]).
    /// "Trustee for Acme Corp" or "Tenants from City of Boston" for "Acme
    /// Corporation" names another party, or the party and something else,
    /// and learned would name every later document giving that party.
    ///
    /// On a side of a "between", "and" and "&" are counted too, being the
    /// clause's own words for joining two parties: "Jane Doe and Acme
    /// Corporation" for "Acme Corporation" names two.
    fn lesson(&self, approved: &str) -> Option<HouseRule> {
        let between = self.connector == connector_word(PartyRelation::Between);
        self.respelling(approved)
            .filter(|edit| !holds_another_separator(&edit.to, &edit.from, between))
    }

    /// The one party the reviewer respelled, if exactly one was, and the
    /// words they put in its place: not words that keep it beside a
    /// connecting word and more words (see [`Self::keeps`]).
    fn respelling(&self, approved: &str) -> Option<HouseRule> {
        let ((from, segment), to) = match self.parties.as_slice() {
            [party] => (party, approved),
            [first, second] => {
                let second_changed = approved
                    .strip_prefix(first.1.as_str())
                    .and_then(|rest| rest.strip_prefix(" and "));
                let first_changed = approved
                    .strip_suffix(second.1.as_str())
                    .and_then(|rest| rest.strip_suffix(" and "));
                match (second_changed, first_changed) {
                    (Some(to), _) if to != second.1 => (second, to),
                    (_, Some(to)) => (first, to),
                    _ => return None,
                }
            }
            _ => return None,
        };
        (!self.keeps(to, from, segment)).then(|| HouseRule::new(RuleKind::Party, from.clone(), to))
    }

    /// Whether `words` keep the party the proposal spells `from` and the
    /// name `segment` beside a connecting word or a note and more words:
    /// the words on one side of it hold the party whole, as a run of whole
    /// words as house style compares spellings ([`HouseRule::key`]), in
    /// any case or punctuation and lengthened or not. Behind one, the type
    /// lengthened in front of it - "Tenants from ACME PROPERTY" or "Tenants
    /// from Acme Property Inc" for "Acme Property" - or in front of one,
    /// another party or a note after it: "Acme Property from Jane Doe",
    /// "Acme Property re Lease". The connecting word or the note is found in
    /// the words as written, in any case, since a key has no " - " or " ("
    /// (see [`separators_in`]).
    fn keeps(&self, words: &str, from: &str, segment: &str) -> bool {
        let words = spaced(words);
        [from, segment].into_iter().any(|kept| {
            let kept = HouseRule::key(kept);
            let holds = |side: &str| {
                !kept.is_empty()
                    && format!(" {} ", HouseRule::key(side)).contains(&format!(" {kept} "))
            };
            separators_in(&words).any(|(at, end)| holds(&words[..at]) || holds(&words[end..]))
        })
    }
}

/// The starts of a note about the document after a party, as review keeps
/// one when a party is chosen in its place (`NOTE_AFTER_PARTY` in
/// src/features/queue/check.ts): " re Merger", " re: the lease",
/// " regarding", " about", " concerning", " (signed)". " - " is a
/// connecting word as well.
const NOTES_AFTER_A_PARTY: [&str; 6] = [
    " re ",
    " re: ",
    " regarding ",
    " about ",
    " concerning ",
    " (",
];

/// Whether `words` are the party `party`, as house style compares
/// spellings ([`HouseRule::key`]), and more words after it that a
/// connecting word or a note begins: "Carl Malamud from Jane Doe", "ACME
/// PROPERTY re Lease" or "Acme (signed)" for the party "Carl Malamud",
/// "Acme Property" or "Acme". Such words name the party and something
/// else. The connecting word or the note is found in the words as written,
/// in any case, since a key has no " - " or " (" (see [`separators_in`]).
pub fn party_and_more(words: &str, party: &str) -> bool {
    let party = HouseRule::key(party);
    let words = spaced(words);
    !party.is_empty() && separators_in(&words).any(|(at, _)| HouseRule::key(&words[..at]) == party)
}

/// What begins something other than more of a party's name: a connecting
/// word naming puts before a party, spaced as a name writes it (" for ",
/// " - "), or the start of a note (see [`NOTES_AFTER_A_PARTY`]).
fn separators() -> impl Iterator<Item = String> {
    PartyRelation::ALL
        .into_iter()
        .map(|relation| format!(" {} ", connector_word(relation)))
        .chain(NOTES_AFTER_A_PARTY.map(str::to_owned))
}

/// Where [`separators`] are found in `words`, [`spaced`] already, in any
/// case, as byte ranges: each one that begins something other than more of
/// the name (see [`begins_more`]), and one left dangling at the end, with
/// nothing after it ("Acme Corporation re").
fn separators_in(words: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let lower = words.to_ascii_lowercase();
    separators()
        .flat_map(move |separator| {
            let mut found = lower
                .match_indices(separator.as_str())
                .map(|(at, _)| (at, at + separator.len()))
                .collect::<Vec<_>>();
            let dangling = separator.trim_end();
            if dangling != separator && lower.ends_with(dangling) {
                found.push((lower.len() - dangling.len(), lower.len()));
            }
            found
        })
        .filter(|&(at, end)| begins_more(words, at, end))
}

/// Whether the separator `words[at..end]` begins something other than more
/// of a party's name: always, but for " (", which begins a note only where
/// its parenthetical ends the words - "Acme (signed)", "Smith (Trustee)" -
/// and not where more of the name follows its ")", as in "Acme (UK) Ltd".
fn begins_more(words: &str, at: usize, end: usize) -> bool {
    words[at..end] != *" (" || {
        let rest = &words[end..];
        rest.find(')')
            .is_none_or(|close| !rest[close + 1..].chars().any(char::is_alphanumeric))
    }
}

/// `words` as a separator is found in them: every dash a divider is typed
/// with ("–", "—", "‐", "−", "－") written "-", a token of dashes alone
/// ("--") written "-", and every space - a non-breaking one, a tab, two in
/// a row - one " ". So "Acme Corp – Jane Doe" and "Acme Corp -- Jane Doe"
/// are divided as "Acme Corp - Jane Doe" is, and " for " is found between
/// non-breaking spaces. Words are never changed otherwise.
fn spaced(words: &str) -> String {
    words
        .split_whitespace()
        .map(|token| {
            let token = token
                .chars()
                .map(|character| match character {
                    '\u{2010}'..='\u{2015}' | '\u{2212}' | '\u{fe58}' | '\u{fe63}' | '\u{ff0d}' => {
                        '-'
                    }
                    other => other,
                })
                .collect::<String>();
            if token.chars().all(|character| character == '-') {
                "-".to_owned()
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A mark that sets a party apart from another, or from the type, as
/// [`holds_another_separator`] and [`is_a_spelling`] count it.
#[derive(Clone, Copy)]
enum Mark {
    /// Whole words, as house style compares spellings ([`HouseRule::key`]):
    /// "for" in "Trustee FOR Acme Corp", "re" in "RE: the Lease". Words that
    /// end new words introduce nothing: "Swiss Re" for "Swiss Reinsurance
    /// Company Ltd" holds no note. A party's own name counts all it holds,
    /// so "Swiss Re Ltd" for "Swiss Re" holds no more.
    Words(&'static str),
    /// Written so, in [`spaced`] words, where it begins something other
    /// than more of the name (see [`begins_more`]): " - ", " (", and the
    /// " v " of a caption, in lower case so the "V" of "Fund V" is none.
    Written(&'static str),
}

impl Mark {
    /// How many times `text` holds the mark, words that end it among them
    /// where `ending`.
    fn count(self, text: &str, ending: bool) -> usize {
        match self {
            Self::Words(marked) => {
                let marked = marked.split(' ').collect::<Vec<_>>();
                let key = HouseRule::key(text);
                let words = key.split(' ').collect::<Vec<_>>();
                words
                    .windows(marked.len())
                    .enumerate()
                    .filter(|(at, run)| {
                        *run == marked && (ending || at + marked.len() < words.len())
                    })
                    .count()
            }
            Self::Written(marked) => {
                let text = spaced(text);
                text.match_indices(marked)
                    .filter(|(at, _)| begins_more(&text, *at, at + marked.len()))
                    .count()
            }
        }
    }
}

/// The connecting words naming puts before a party, and the starts of a
/// note, as [`separators`] gives them, as they are counted: a word as a
/// whole word ("re" of " re: " too), " - " and " (" as written.
fn separator_marks() -> impl Iterator<Item = Mark> {
    PartyRelation::ALL
        .into_iter()
        .map(|relation| match connector_word(relation) {
            "-" => Mark::Written(" - "),
            word => Mark::Words(word),
        })
        .chain(NOTES_AFTER_A_PARTY.into_iter().map(|note| {
            let word = note.trim().trim_end_matches(':');
            if word.chars().all(char::is_alphanumeric) {
                Mark::Words(word)
            } else {
                Mark::Written(note)
            }
        }))
}

/// Words that join two parties outside the name's grammar: a case's
/// caption ("Smith v Acme Corp", "v.", "vs", "versus"), a party acting for
/// another ("Jane Doe on behalf of Acme Corp", "Acme Corp by its
/// Trustee"), another name of a party ("John Smith dba Acme Corp", "Acme
/// Holdings fka Acme Corp"), and a person to attend ("Acme Corp attn Jane
/// Doe"). "c/o" and "d/b/a" cannot be typed in a name; "co" is as often
/// "Acme Co".
const JOINING: [Mark; 9] = [
    Mark::Written(" v "),
    Mark::Written(" v. "),
    Mark::Words("vs"),
    Mark::Words("versus"),
    Mark::Words("by"),
    Mark::Words("on behalf of"),
    Mark::Words("dba"),
    Mark::Words("fka"),
    Mark::Words("attn"),
];

/// The words that join the two sides of a "between", which join a party to
/// another wherever they are typed beside the party kept whole.
const AND: [Mark; 2] = [Mark::Words("and"), Mark::Written(" & ")];

/// What a type is never lengthened with, but a type given a party is: the
/// divider naming puts before a party, "between", and a caption's "v".
const TYPE_GIVEN_A_PARTY: [Mark; 6] = [
    Mark::Written(" - "),
    Mark::Words("between"),
    Mark::Written(" v "),
    Mark::Written(" v. "),
    Mark::Words("vs"),
    Mark::Words("versus"),
];

/// Whether `words` hold a mark of another party more often than the party
/// `party` does (see [`Mark`]): a connecting word or a note's start (see
/// [`separator_marks`]) - "for" in "Trustee for Acme Corp", "re" in "Tenants
/// RE: the Lease", "from" at the start of "from City of Boston", " - "
/// however its dash is typed - or a word joining two parties outside the
/// name's grammar (see [`JOINING`]). "Centers for Disease Control" holds
/// "for" no more often than "Center for Disease Control"; "Trustee for
/// Center for Disease Control" holds it more often. "and" and "&" (see
/// [`AND`]) are counted on a side of a "between", where `between`, and
/// where `words` hold a party of two words or more whole beside more:
/// "Acme Corporation and Jane Doe", but not "Johnson and Johnson" for
/// "Johnson", nor "Barnes and Noble" for "Barnes Noble".
fn holds_another_separator(words: &str, party: &str, between: bool) -> bool {
    let kept = HouseRule::key(party);
    let joined = between
        || (kept.split(' ').count() >= 2
            && format!(" {} ", HouseRule::key(words)).contains(&format!(" {kept} ")));
    separator_marks()
        .chain(JOINING)
        .chain(AND.into_iter().filter(|_| joined))
        .any(|mark| mark.count(words, false) > mark.count(party, true))
}

/// The words of `name` in front of a connecting word that the party `kept`
/// follows to the end, the party compared as house style compares
/// spellings ([`HouseRule::key`]): "Notice to Tenants" of "Notice to
/// Tenants from ACME PROPERTY" for "Acme Property". The connecting word is
/// found in the name as written, in any case, since a key has no " - ".
fn kept_behind_a_connector<'a>(name: &'a str, kept: &str) -> Option<&'a str> {
    let kept = HouseRule::key(kept);
    if kept.is_empty() {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    PartyRelation::ALL.into_iter().find_map(|relation| {
        let connector = format!(" {} ", connector_word(relation));
        lower
            .match_indices(connector.as_str())
            .find(|(at, _)| HouseRule::key(&name[at + connector.len()..]) == kept)
            .map(|(at, _)| &name[..at])
    })
}

/// The word naming puts between the type and the parties.
fn connector_word(relation: PartyRelation) -> &'static str {
    match relation {
        PartyRelation::None => "-",
        stated => stated.as_str(),
    }
}

/// The stem of a filename as a reviewer edits it: no extension, no leading
/// date, no collision suffix.
fn editable_stem(filename: &str, extension: &str) -> String {
    let mut stem = filename.trim();
    if !extension.is_empty()
        && let Some(prefix) = stem
            .len()
            .checked_sub(extension.len() + 1)
            .and_then(|start| stem.get(..start).zip(stem.get(start..)))
            .filter(|(_, suffix)| {
                suffix.starts_with('.') && suffix[1..].eq_ignore_ascii_case(extension)
            })
            .map(|(prefix, _)| prefix)
    {
        stem = prefix;
    }
    if let Some(date) = stem.get(..10)
        && is_valid_iso_date(date)
        && stem[10..]
            .chars()
            .next()
            .is_none_or(|next| !next.is_alphanumeric())
    {
        stem = stem[10..].trim_start();
    }
    strip_collision_suffix(stem).trim().to_owned()
}

/// `Invoice (2)` is `Invoice` with a collision the reviewer never typed.
fn strip_collision_suffix(stem: &str) -> &str {
    let trimmed = stem.trim_end();
    let Some(open) = trimmed.rfind(" (") else {
        return trimmed;
    };
    let inside = &trimmed[open + 2..];
    if inside.ends_with(')')
        && inside.len() > 1
        && inside[..inside.len() - 1]
            .bytes()
            .all(|byte| byte.is_ascii_digit())
    {
        trimmed[..open].trim_end()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DateRole, Evidence};
    use crate::naming::compose_filename;

    fn proposal(
        document_type: Option<&str>,
        parties: &[&str],
        relation: PartyRelation,
    ) -> ValidatedProposal {
        ValidatedProposal {
            document_type: document_type.map(str::to_owned),
            document_date: Some("2026-04-01".into()),
            date_role: Some(DateRole::Effective),
            parties: parties.iter().map(|value| (*value).to_owned()).collect(),
            party_relation: relation,
            description: "A description of the document.".into(),
            confidence: 0.9,
            evidence: Evidence::default(),
        }
    }

    fn name(proposal: &ValidatedProposal) -> String {
        compose_filename(proposal, "pdf", &[]).value
    }

    #[test]
    fn a_party_rule_rewrites_the_document_spelling_and_says_so() {
        let style = HouseStyle::new(vec![HouseRule::new(
            RuleKind::Party,
            "Contoso Worldwide, Inc.",
            "Contoso",
        )]);
        let proposal = proposal(
            Some("Statement of Work"),
            &["Ridgeline Cartography LLC", "CONTOSO WORLDWIDE INC"],
            PartyRelation::Between,
        );
        let (styled, applied) = style.apply(&proposal);
        assert_eq!(
            name(&styled),
            "2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso.pdf"
        );
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].to, "Contoso");
        assert_eq!(styled.evidence, proposal.evidence, "evidence is untouched");
    }

    #[test]
    fn a_type_rule_rewrites_the_type_and_no_rule_rewrites_nothing() {
        let style = HouseStyle::new(vec![HouseRule::new(
            RuleKind::DocumentType,
            "Quarterly Operations Review",
            "Meeting Minutes",
        )]);
        let minutes = proposal(
            Some("Quarterly Operations Review"),
            &[],
            PartyRelation::None,
        );
        let (styled, applied) = style.apply(&minutes);
        assert_eq!(styled.document_type.as_deref(), Some("Meeting Minutes"));
        assert_eq!(applied.len(), 1);

        let invoice = proposal(Some("Invoice"), &["Acme Corporation"], PartyRelation::From);
        let (unchanged, applied) = style.apply(&invoice);
        assert_eq!(unchanged, invoice);
        assert!(applied.is_empty());
    }

    #[test]
    fn two_of_the_documents_names_that_become_one_are_named_once() {
        let style = HouseStyle::new(vec![
            HouseRule::new(RuleKind::Party, "Acme Corporation", "Acme"),
            HouseRule::new(RuleKind::Party, "Acme Corp.", "Acme"),
        ]);
        let proposal = proposal(
            Some("Invoice"),
            &["Acme Corporation", "Acme Corp."],
            PartyRelation::Between,
        );
        let (styled, _) = style.apply(&proposal);
        assert_eq!(styled.parties, vec!["Acme"]);
        // One side is left, so the name and the stored proposal both say
        // "with" - and a name that reads the way it was composed still
        // teaches.
        assert_eq!(styled.party_relation, PartyRelation::With);
        let proposed = name(&styled);
        assert_eq!(proposed, "2026-04-01 Invoice with Acme.pdf");
        let lesson = lesson_from_edit(
            &styled,
            "pdf",
            &proposed,
            "2026-04-01 Invoice with Acme Industries.pdf",
        )
        .expect("the one party changed and nothing else did");
        assert_eq!(lesson.from, "Acme");
        assert_eq!(lesson.to, "Acme Industries");
    }

    /// Naming title-cases a party printed in capitals, so the name a
    /// reviewer edits is not the document's spelling; it must still read as
    /// a name this proposal composed, or no edit to it would ever teach.
    #[test]
    fn edits_to_title_cased_names_still_teach() {
        let proposal = proposal(
            Some("WORK ORDER"),
            &["HARBOR COMET REPAIRS LLC"],
            PartyRelation::From,
        );
        let proposed = name(&proposal);
        assert_eq!(
            proposed,
            "2026-04-01 Work Order from Harbor Comet Repairs LLC.pdf"
        );
        let party = lesson_from_edit(
            &proposal,
            "pdf",
            &proposed,
            "2026-04-01 Work Order from Harbor Comet.pdf",
        )
        .expect("the party changed and nothing else did");
        assert_eq!(party.kind, RuleKind::Party);
        assert_eq!(party.from, "HARBOR COMET REPAIRS LLC");
        assert_eq!(party.to, "Harbor Comet");

        let document_type = lesson_from_edit(
            &proposal,
            "pdf",
            &proposed,
            "2026-04-01 Repair Order from Harbor Comet Repairs LLC.pdf",
        )
        .expect("the type changed and nothing else did");
        assert_eq!(document_type.kind, RuleKind::DocumentType);
        assert_eq!(document_type.from, "WORK ORDER");
        assert_eq!(document_type.to, "Repair Order");

        // Approving the title-cased name as offered teaches nothing, and
        // neither does typing the capitals back: case is not a spelling.
        assert_eq!(
            lesson_from_edit(&proposal, "pdf", &proposed, &proposed),
            None
        );
        assert_eq!(
            lesson_from_edit(
                &proposal,
                "pdf",
                &proposed,
                "2026-04-01 Work Order from HARBOR COMET REPAIRS LLC.pdf"
            ),
            None
        );
    }

    /// A spelling a reviewer typed is theirs, capitals and all: the name
    /// carried "ACME WIDGETS CORP" title-cased as "Acme Widgets Corp", and
    /// typing the capitals back taught nothing, because case alone is not a
    /// spelling - so the spelling they chose could never be had. The
    /// document's own words beside it are still title-cased.
    #[test]
    fn a_spelling_a_rule_wrote_is_named_as_typed() {
        let style = HouseStyle::new(vec![
            HouseRule::new(
                RuleKind::Party,
                "Acme Widgets Corporation",
                "ACME WIDGETS CORP",
            ),
            HouseRule::new(
                RuleKind::DocumentType,
                "Statement of Account",
                "MONTHLY STATEMENT",
            ),
        ]);
        let (styled, applied) = style.apply(&proposal(
            Some("Statement of Account"),
            &["Acme Widgets Corporation", "ORION GLASS STUDIO INC"],
            PartyRelation::Between,
        ));
        let proposed = compose_styled_filename(&styled, &applied, "pdf", &[]).value;
        assert_eq!(
            proposed,
            "2026-04-01 MONTHLY STATEMENT between ACME WIDGETS CORP and Orion Glass Studio Inc.pdf"
        );
        // Composed without the rules, every word is the document's.
        assert_eq!(
            name(&styled),
            "2026-04-01 Monthly Statement between Acme Widgets Corp and Orion Glass Studio Inc.pdf"
        );

        // And the name still reads as one this proposal composed.
        let lesson = lesson_from_edit(
            &styled,
            "pdf",
            &proposed,
            "2026-04-01 MONTHLY STATEMENT between ACME WIDGETS and Orion Glass Studio Inc.pdf",
        )
        .expect("the first party changed and nothing else did");
        assert_eq!(
            (lesson.from.as_str(), lesson.to.as_str()),
            ("ACME WIDGETS CORP", "ACME WIDGETS")
        );
        let lesson = lesson_from_edit(
            &styled,
            "pdf",
            &proposed,
            "2026-04-01 MONTHLY STATEMENT between ACME WIDGETS CORP and Orion Glass.pdf",
        )
        .expect("the second party changed and nothing else did");
        assert_eq!(
            (lesson.from.as_str(), lesson.to.as_str()),
            ("ORION GLASS STUDIO INC", "Orion Glass")
        );
    }

    /// A proposal alpha.10 named is still waiting after the upgrade, under
    /// the name alpha.10 composed: capitals as printed, ligatures and
    /// decomposed accents as extracted, and "between" before a party a merge
    /// left alone. Nothing recomposes it, and an edit to it must still teach.
    #[test]
    fn names_composed_before_the_upgrade_still_teach() {
        for (document_type, party, relation, proposed, approved, from, to) in [
            (
                "Lease Agreement",
                "ORION GLASS STUDIO INC.",
                PartyRelation::With,
                "2026-04-01 Lease Agreement with ORION GLASS STUDIO INC.pdf",
                "2026-04-01 Lease Agreement with Orion Glass.pdf",
                "ORION GLASS STUDIO INC.",
                "Orion Glass",
            ),
            (
                "Invoice",
                "O\u{fb03}ce Supplies Direct",
                PartyRelation::From,
                "2026-04-01 Invoice from O\u{fb03}ce Supplies Direct.pdf",
                "2026-04-01 Invoice from Office Supplies.pdf",
                "O\u{fb03}ce Supplies Direct",
                "Office Supplies",
            ),
            (
                "Invoice",
                "Cafe\u{301} Lumie\u{300}re SARL",
                PartyRelation::From,
                "2026-04-01 Invoice from Cafe\u{301} Lumie\u{300}re SARL.pdf",
                "2026-04-01 Invoice from Caf\u{e9} Lumi\u{e8}re.pdf",
                "Cafe\u{301} Lumie\u{300}re SARL",
                "Caf\u{e9} Lumi\u{e8}re",
            ),
        ] {
            let proposal = proposal(Some(document_type), &[party], relation);
            assert_ne!(name(&proposal), proposed, "this build names it otherwise");
            let lesson = lesson_from_edit(&proposal, "pdf", proposed, approved)
                .unwrap_or_else(|| panic!("{proposed} still teaches"));
            assert_eq!(lesson.kind, RuleKind::Party);
            assert_eq!((lesson.from.as_str(), lesson.to.as_str()), (from, to));
        }

        let type_in_capitals = proposal(
            Some("LEASE AGREEMENT"),
            &["Orion Glass Studio Inc."],
            PartyRelation::With,
        );
        let lesson = lesson_from_edit(
            &type_in_capitals,
            "pdf",
            "2026-04-01 LEASE AGREEMENT with Orion Glass Studio Inc.pdf",
            "2026-04-01 Lease with Orion Glass Studio Inc.pdf",
        )
        .expect("the type changed and nothing else did");
        assert_eq!(lesson.kind, RuleKind::DocumentType);
        assert_eq!(lesson.from, "LEASE AGREEMENT");
        assert_eq!(lesson.to, "Lease");

        // Two spellings merged into one: alpha.10 still said "between".
        let style = HouseStyle::new(vec![
            HouseRule::new(RuleKind::Party, "Acme Corporation", "Acme"),
            HouseRule::new(RuleKind::Party, "Acme Corp.", "Acme"),
        ]);
        let (styled, _) = style.apply(&proposal(
            Some("Invoice"),
            &["Acme Corporation", "Acme Corp."],
            PartyRelation::Between,
        ));
        let lesson = lesson_from_edit(
            &styled,
            "pdf",
            "2026-04-01 Invoice between Acme.pdf",
            "2026-04-01 Invoice between Acme Industries.pdf",
        )
        .expect("the one party changed and nothing else did");
        assert_eq!(
            (lesson.from.as_str(), lesson.to.as_str()),
            ("Acme", "Acme Industries")
        );
    }

    #[test]
    fn shortening_a_party_in_review_teaches_that_partys_spelling() {
        let proposal = proposal(
            Some("Statement of Work"),
            &["Ridgeline Cartography LLC", "Contoso Worldwide, Inc."],
            PartyRelation::Between,
        );
        let proposed = name(&proposal);
        let lesson = lesson_from_edit(
            &proposal,
            "pdf",
            &proposed,
            "2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso.pdf",
        )
        .expect("the second party changed and nothing else did");
        assert_eq!(lesson.kind, RuleKind::Party);
        assert_eq!(lesson.from, "Contoso Worldwide, Inc.");
        assert_eq!(lesson.to, "Contoso");
    }

    #[test]
    fn lengthening_a_party_and_changing_the_date_at_once_still_teaches_the_party() {
        let proposal = proposal(Some("Invoice"), &["Acme"], PartyRelation::From);
        let lesson = lesson_from_edit(
            &proposal,
            "pdf",
            "2026-04-01 Invoice from Acme (2).pdf",
            "2026-05-09 Invoice from Acme Corporation.pdf",
        )
        .expect("the date and the collision suffix are not part of the lesson");
        assert_eq!(lesson.from, "Acme");
        assert_eq!(lesson.to, "Acme Corporation");
    }

    #[test]
    fn renaming_the_type_teaches_the_type_even_when_it_shares_letters() {
        let proposal = proposal(
            Some("Statement of Work"),
            &["Acme", "Contoso"],
            PartyRelation::Between,
        );
        let lesson = lesson_from_edit(
            &proposal,
            "pdf",
            "2026-04-01 Statement of Work between Acme and Contoso.pdf",
            "2026-04-01 SOW between Acme and Contoso.pdf",
        )
        .unwrap();
        assert_eq!(lesson.kind, RuleKind::DocumentType);
        assert_eq!(lesson.from, "Statement of Work");
        assert_eq!(lesson.to, "SOW");

        let prefixed = lesson_from_edit(
            &proposal,
            "pdf",
            "2026-04-01 Statement of Work between Acme and Contoso.pdf",
            "2026-04-01 Signed Statement of Work between Acme and Contoso.pdf",
        )
        .unwrap();
        assert_eq!(prefixed.to, "Signed Statement of Work");
    }

    /// A type lengthened with the connecting word of the party clause it
    /// kept reads as a longer type or as a longer party alike: "Application
    /// for Certification for City of Victorville" for "Application for City
    /// of Victorville". It is read as the type, as [`approved_party`] reads
    /// it, and teaches no party "Certification for City of Victorville".
    #[test]
    fn a_type_lengthened_with_its_partys_connecting_word_teaches_the_type() {
        let application = proposal(
            Some("Application"),
            &["City of Victorville"],
            PartyRelation::For,
        );
        let proposed = name(&application);
        assert_eq!(
            proposed,
            "2026-04-01 Application for City of Victorville.pdf"
        );
        let approved = "2026-04-01 Application for Certification for City of Victorville.pdf";
        let lesson = lesson_from_edit(&application, "pdf", &proposed, approved)
            .expect("the type changed and nothing else did");
        assert_eq!(
            (lesson.kind, lesson.from.as_str(), lesson.to.as_str()),
            (
                RuleKind::DocumentType,
                "Application",
                "Application for Certification"
            )
        );
        assert_eq!(
            approved_party(&application, "pdf", &proposed, approved),
            None
        );

        // The party alone changed, its new name ending in the old: a party.
        let lesson = lesson_from_edit(
            &application,
            "pdf",
            &proposed,
            "2026-04-01 Application for Friends of City of Victorville.pdf",
        )
        .expect("the party changed and nothing else did");
        assert_eq!(
            (lesson.kind, lesson.to.as_str()),
            (RuleKind::Party, "Friends of City of Victorville")
        );
    }

    #[test]
    fn an_edit_that_touches_two_fields_or_the_connectors_teaches_nothing() {
        let proposal = proposal(
            Some("Statement of Work"),
            &["Acme", "Contoso"],
            PartyRelation::Between,
        );
        let proposed = "2026-04-01 Statement of Work between Acme and Contoso.pdf";
        for approved in [
            // Both parties at once.
            "2026-04-01 Statement of Work between Acme Corp and Contoso Inc.pdf",
            // The relation, which is a fact about the document.
            "2026-04-01 Statement of Work with Acme.pdf",
            // The connecting word alone.
            "2026-04-01 Statement of Work for Acme and Contoso.pdf",
            // Only the date.
            "2026-05-01 Statement of Work between Acme and Contoso.pdf",
            // A rewrite from scratch.
            "2026-04-01 Acme deal.pdf",
        ] {
            assert_eq!(
                lesson_from_edit(&proposal, "pdf", proposed, approved),
                None,
                "{approved}"
            );
        }
    }

    #[test]
    fn a_removed_party_and_a_missing_type_teach_nothing() {
        let with_party = proposal(Some("Invoice"), &["Acme Corporation"], PartyRelation::From);
        assert_eq!(
            lesson_from_edit(
                &with_party,
                "pdf",
                "2026-04-01 Invoice from Acme Corporation.pdf",
                "2026-04-01 Invoice from.pdf"
            ),
            None,
            "an empty spelling is not a spelling"
        );
        let untyped = proposal(None, &[], PartyRelation::None);
        assert_eq!(
            lesson_from_edit(
                &untyped,
                "pdf",
                "2026-04-01 Document.pdf",
                "2026-04-01 Board Pack.pdf"
            ),
            None,
            "there is no document spelling to map from"
        );
    }

    #[test]
    fn a_name_intern_did_not_compose_teaches_nothing() {
        let proposal = proposal(Some("Invoice"), &["Acme"], PartyRelation::From);
        assert_eq!(
            lesson_from_edit(
                &proposal,
                "pdf",
                "2026-04-01 Receipt from Acme.pdf",
                "2026-04-01 Receipt from Acme Ltd.pdf"
            ),
            None
        );
    }

    /// The parties a name gives are read with the grammar that composed it,
    /// so a type that holds "for" is no party, the second side of a
    /// "between" is told from the first, and capitals are given as the name
    /// writes them.
    #[test]
    fn the_parties_a_name_gives_are_read_as_it_writes_them() {
        let application = proposal(
            Some("Application for Certification"),
            &["CITY OF VICTORVILLE"],
            PartyRelation::For,
        );
        let proposed = name(&application);
        assert_eq!(
            proposed,
            "2026-04-01 Application for Certification for City of Victorville.pdf"
        );
        assert_eq!(
            named_parties(&application, "pdf", &proposed),
            vec!["City of Victorville"]
        );
        // A collision suffix and a date the reviewer typed are no part of it.
        assert_eq!(
            named_parties(
                &application,
                "pdf",
                "2026-05-09 Application for Certification for City of Victorville (2).pdf"
            ),
            vec!["City of Victorville"]
        );

        let both = proposal(
            Some("Statement of Work"),
            &["Ridgeline Cartography LLC", "CONTOSO WORLDWIDE INC"],
            PartyRelation::Between,
        );
        assert_eq!(
            named_parties(&both, "pdf", &name(&both)),
            vec!["Ridgeline Cartography LLC", "Contoso Worldwide Inc"]
        );

        // A spelling a rule wrote is given as the reviewer typed it.
        let style = HouseStyle::new(vec![HouseRule::new(
            RuleKind::Party,
            "Contoso Worldwide, Inc.",
            "CONTOSO",
        )]);
        let (styled, applied) = style.apply(&proposal(
            Some("Invoice"),
            &["Contoso Worldwide, Inc."],
            PartyRelation::From,
        ));
        let proposed = compose_styled_filename(&styled, &applied, "pdf", &[]).value;
        assert_eq!(named_parties(&styled, "pdf", &proposed), vec!["CONTOSO"]);
    }

    /// Nothing is given where the name is not known to end with the
    /// parties: a name with none, one cut short for length, one typed by
    /// hand.
    #[test]
    fn a_name_without_its_parties_gives_none() {
        let untyped = proposal(Some("Board Minutes"), &[], PartyRelation::None);
        assert!(named_parties(&untyped, "pdf", &name(&untyped)).is_empty());

        let long = "Northwind Regional Transportation and Infrastructure Development Authority";
        let shed = proposal(
            Some("Memorandum of Understanding on Shared Maintenance Facilities"),
            &[long],
            PartyRelation::With,
        );
        let proposed = name(&shed);
        assert!(!proposed.contains("Northwind"), "{proposed}");
        assert!(named_parties(&shed, "pdf", &proposed).is_empty());

        let invoice = proposal(Some("Invoice"), &["Acme"], PartyRelation::From);
        assert!(named_parties(&invoice, "pdf", "2026-04-01 Acme invoice for March.pdf").is_empty());
    }

    /// The party an approved name gives after the proposed type: another
    /// party chosen in review, with the connecting word changed or not and
    /// a date or collision suffix about it, or one added after " - " to a
    /// name that gave none; a type holding "for" is kept whole.
    #[test]
    fn the_party_an_approved_name_gives_is_read_after_the_proposed_type() {
        let application = proposal(
            Some("Application for Certification"),
            &["CITY OF VICTORVILLE"],
            PartyRelation::For,
        );
        let proposed = name(&application);
        let approved = |name: &str| approved_party(&application, "pdf", &proposed, name);
        assert_eq!(
            approved(
                "2026-04-01 Application for Certification for California Energy Commission.pdf"
            )
            .as_deref(),
            Some("California Energy Commission")
        );
        assert_eq!(
            approved(
                "2026-05-09 Application for Certification from California Energy Commission (2).pdf"
            )
            .as_deref(),
            Some("California Energy Commission")
        );
        assert_eq!(
            approved(&proposed).as_deref(),
            Some("City of Victorville"),
            "an unchanged name gives its own party"
        );

        let shed = proposal(Some("Requester Invoice"), &[], PartyRelation::None);
        assert_eq!(
            approved_party(
                &shed,
                "pdf",
                &name(&shed),
                "2026-04-01 Requester Invoice - Sam Brownell.pdf"
            )
            .as_deref(),
            Some("Sam Brownell")
        );

        let notice = proposal(Some("Notice"), &["Acme Property"], PartyRelation::From);
        let proposed = name(&notice);
        assert_eq!(proposed, "2026-04-01 Notice from Acme Property.pdf");
        for (approved, party) in [
            (
                "2026-04-01 Notice from City of Boston.pdf",
                "City of Boston",
            ),
            ("2026-04-01 Notice to Acme Property.pdf", "Acme Property"),
        ] {
            assert_eq!(
                approved_party(&notice, "pdf", &proposed, approved).as_deref(),
                Some(party),
                "{approved}"
            );
        }
        // The same party behind another connecting word, the type kept.
        let addressed = proposal(Some("Notice"), &["Acme Property"], PartyRelation::To);
        assert_eq!(
            approved_party(
                &addressed,
                "pdf",
                &name(&addressed),
                "2026-04-01 Notice from Acme Property.pdf"
            )
            .as_deref(),
            Some("Acme Property")
        );
    }

    /// No party is read from a name whose type changed - replaced, or
    /// lengthened in front of the party clause it kept, or in front of the
    /// party it kept behind another connecting word - that gives two
    /// sides, or none, or from a proposed name not composed from the
    /// proposal.
    #[test]
    fn an_approved_name_with_another_type_or_two_sides_gives_no_one_party() {
        let letter = proposal(Some("Letter"), &["Carl Malamud"], PartyRelation::To);
        let proposed = name(&letter);
        for approved in [
            "2026-04-01 Letter of Complaint to Department of the Air Force.pdf",
            "2026-04-01 Letter between Carl Malamud and Department of the Air Force.pdf",
            "2026-04-01 Letter.pdf",
            "2026-04-01 Letter to .pdf",
            "2026-04-01 Letter from Jane Doe to Carl Malamud.pdf",
        ] {
            assert_eq!(
                approved_party(&letter, "pdf", &proposed, approved),
                None,
                "{approved}"
            );
        }
        for (document_type, parties, relation, approved) in [
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::From,
                "2026-04-01 Notice to Tenants from Acme Property.pdf",
            ),
            (
                "Invoice",
                &["Acme"][..],
                PartyRelation::From,
                "2026-04-01 Invoice for March from Acme.pdf",
            ),
            (
                "Agreement",
                &["Acme", "Contoso"][..],
                PartyRelation::Between,
                "2026-04-01 Agreement for Services between Acme and Contoso.pdf",
            ),
        ] {
            let lengthened = proposal(Some(document_type), parties, relation);
            assert_eq!(
                approved_party(&lengthened, "pdf", &name(&lengthened), approved),
                None,
                "{approved}"
            );
        }
        // The party kept behind another connecting word, the type
        // lengthened in front: the type and the relation changed, which
        // is a decision about the document. No party is read, and no
        // lesson either - not the party "Tenants from Acme Property".
        for (document_type, party, relation, approved) in [
            (
                "Notice",
                "Acme Property",
                PartyRelation::To,
                "2026-04-01 Notice to Tenants from Acme Property.pdf",
            ),
            (
                "Notice",
                "Acme Property",
                PartyRelation::To,
                "2026-04-01 Notice to Tenants - Acme Property.pdf",
            ),
            (
                "Application",
                "City of Victorville",
                PartyRelation::For,
                "2026-04-01 Application for Certification from City of Victorville.pdf",
            ),
            (
                "Employment Agreement",
                "Acme Corporation",
                PartyRelation::For,
                "2026-04-01 Employment Agreement for Sales Staff from Acme Corporation.pdf",
            ),
            (
                "Letter",
                "Acme",
                PartyRelation::To,
                "2026-04-01 Letter to Jane Doe from Acme.pdf",
            ),
            (
                "Invoice",
                "Acme",
                PartyRelation::To,
                "2026-04-01 Invoice for March from Acme.pdf",
            ),
            (
                "Lease",
                "Acme",
                PartyRelation::With,
                "2026-04-01 Lease with Option to Purchase for Acme.pdf",
            ),
        ] {
            let flipped = proposal(Some(document_type), &[party], relation);
            let proposed = name(&flipped);
            assert_eq!(
                approved_party(&flipped, "pdf", &proposed, approved),
                None,
                "{approved}"
            );
            assert_eq!(
                lesson_from_edit(&flipped, "pdf", &proposed, approved),
                None,
                "{approved}"
            );
        }
        assert_eq!(
            approved_party(
                &letter,
                "pdf",
                "2026-04-01 Note to Carl Malamud.pdf",
                "2026-04-01 Note to Department of the Air Force.pdf"
            ),
            None
        );
    }

    /// A party kept beside a connecting word and more words, compared as
    /// house style compares spellings, is no spelling of it, whatever the
    /// connecting word and on either side of a "between": behind one, the
    /// type lengthened in front of it - in capitals, with its punctuation
    /// changed or a suffix added, behind the same word as the name's own -
    /// or in front of one, with another party or a note after it. Each was
    /// learned, after two such edits, as that party's spelling, and named
    /// every later document giving the party, ready ones filed without
    /// review: "Acme Corporation" became "Sales Staff from ACME
    /// CORPORATION". No party is read from such a name either.
    #[test]
    fn a_party_kept_beside_a_connecting_word_and_more_is_no_spelling_of_it() {
        for (document_type, parties, relation, approved) in [
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants from ACME PROPERTY",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants to ACME PROPERTY",
            ),
            (
                "Notice",
                &["Acme Property LLC"][..],
                PartyRelation::To,
                "Notice to Tenants from Acme Property, LLC",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::For,
                "Employment Agreement for Sales Staff from ACME CORPORATION",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::For,
                "Employment Agreement for Sales Staff for ACME CORPORATION",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::For,
                "Employment Agreement for Sales Staff from Acme Corporation.",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::For,
                "Employment Agreement for Sales Staff from Acme Corporation Inc",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with ACME CORPORATION INC from Jane Doe",
            ),
            (
                "Invoice",
                &["Acme"][..],
                PartyRelation::None,
                "Invoice for March - ACME",
            ),
            (
                "Letter",
                &["Carl Malamud"][..],
                PartyRelation::To,
                "Letter to Carl Malamud from Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corporation from Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::To,
                "Employment Agreement to ACME CORPORATION for Jane Doe",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Acme Property re Lease",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Acme Property (signed)",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Sales Staff from ACME CORPORATION",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Sales Staff from Acme Corporation",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Acme Corporation from Jane Doe",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between Guardian for JOHN SMITH and Acme Corporation",
            ),
        ] {
            let kept = proposal(Some(document_type), parties, relation);
            let proposed = name(&kept);
            let approved = format!("2026-04-01 {approved}.pdf");
            assert_eq!(
                lesson_from_edit(&kept, "pdf", &proposed, &approved),
                None,
                "{approved}"
            );
            assert_eq!(
                approved_party(&kept, "pdf", &proposed, &approved),
                None,
                "{approved}"
            );
        }

        // A spelling that keeps no party beside a connecting word is still
        // one: lengthened, shortened, or a longer name ending in the old.
        for (document_type, parties, relation, approved, from, to) in [
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Acme Property Management",
                "Acme Property",
                "Acme Property Management",
            ),
            (
                "Application",
                &["City of Victorville"][..],
                PartyRelation::For,
                "Application for Friends of City of Victorville",
                "City of Victorville",
                "Friends of City of Victorville",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corp"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Acme Corporation",
                "Acme Corp",
                "Acme Corporation",
            ),
            (
                "Invoice",
                &["Acme"][..],
                PartyRelation::None,
                "Invoice - Acme Corp",
                "Acme",
                "Acme Corp",
            ),
        ] {
            let respelled = proposal(Some(document_type), parties, relation);
            let approved = format!("2026-04-01 {approved}.pdf");
            let lesson = lesson_from_edit(&respelled, "pdf", &name(&respelled), &approved)
                .unwrap_or_else(|| panic!("{approved}: a spelling"));
            assert_eq!(
                (lesson.kind, lesson.from.as_str(), lesson.to.as_str()),
                (RuleKind::Party, from, to),
                "{approved}"
            );
        }
        // And a type lengthened in front of the party kept exactly, behind
        // its own connecting word or after a " - ", is a type.
        let invoice = proposal(Some("Invoice"), &["Acme"], PartyRelation::None);
        let lesson = lesson_from_edit(
            &invoice,
            "pdf",
            &name(&invoice),
            "2026-04-01 Invoice for March - Acme.pdf",
        )
        .expect("the type changed and nothing else did");
        assert_eq!(
            (lesson.kind, lesson.to.as_str()),
            (RuleKind::DocumentType, "Invoice for March")
        );
        assert_eq!(
            approved_party(
                &invoice,
                "pdf",
                &name(&invoice),
                "2026-04-01 Invoice for March - Acme.pdf"
            ),
            None
        );
    }

    /// A party respelled into words holding a connecting word of the name's
    /// grammar, or a note's start, more often than the party's own name
    /// does - "for", "from", "to", "with", "between", " - ", "re",
    /// "regarding", "about", "concerning", " (" - is no spelling of it,
    /// whether or not the words still hold the party in some spelling: they
    /// name another party, or the party and something else. "Trustee for
    /// Acme Corp" for "Acme Corporation", a party the document names typed
    /// short, and "Tenants from City of Boston" for "Acme Property", words
    /// the document never gives as a party, were each learned, after two
    /// such edits, as that party's spelling, and named every later document
    /// giving it, ready ones filed without review.
    #[test]
    fn a_party_respelled_into_words_with_a_connecting_word_of_their_own_is_no_spelling() {
        for (document_type, parties, relation, approved) in [
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::For,
                "Employment Agreement for Trustee for Acme Corp",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants from City of Boston",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with FDIC as Receiver for Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::For,
                "Employment Agreement for Sales Staff for Acme Corp",
            ),
            (
                "Letter",
                &["Carl Malamud"][..],
                PartyRelation::To,
                "Letter to C. Malamud from Jane Doe",
            ),
            (
                "Letter",
                &["Carl Malamud"][..],
                PartyRelation::To,
                "Letter to Malamud with Enclosures",
            ),
            (
                "Settlement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Settlement with Release between Acme Corp and Jane Doe",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants - City of Boston",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants re Lease",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants RE: the Lease",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants regarding Repairs",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants about Repairs",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants concerning Repairs",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Tenants (signed)",
            ),
            (
                "Invoice",
                &["Acme"][..],
                PartyRelation::None,
                "Invoice - Contoso for March",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Trustee for Acme Corp",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between Guardian for J. Smith and Acme Corporation",
            ),
            // More often than the party's own name holds it.
            (
                "Grant Agreement",
                &["Centers for Disease Control"][..],
                PartyRelation::With,
                "Grant Agreement with Trustee for Center for Disease Control",
            ),
            // The price: a party respelled on purpose into such words is
            // made again by hand.
            (
                "Grant Agreement",
                &["CDC"][..],
                PartyRelation::With,
                "Grant Agreement with Centers for Disease Control",
            ),
        ] {
            let respelled = proposal(Some(document_type), parties, relation);
            let approved = format!("2026-04-01 {approved}.pdf");
            assert_eq!(
                lesson_from_edit(&respelled, "pdf", &name(&respelled), &approved),
                None,
                "{approved}"
            );
        }

        // What was typed in the party's place is still read, for filing;
        // a party kept beside a connecting word and more, or a type
        // changed, is no respelled party.
        let employment =
            |parties: &[&str], relation| proposal(Some("Employment Agreement"), parties, relation);
        for (proposal, approved, respelled) in [
            (
                employment(&["Acme Corporation"], PartyRelation::For),
                "Employment Agreement for Trustee for Acme Corp",
                Some(("Acme Corporation", "Trustee for Acme Corp")),
            ),
            (
                employment(&["John Smith", "Acme Corporation"], PartyRelation::Between),
                "Employment Agreement between John Smith and Trustee for Acme Corp",
                Some(("Acme Corporation", "Trustee for Acme Corp")),
            ),
            (
                employment(&["Acme Corporation"], PartyRelation::For),
                "Employment Agreement for Acme Corp",
                Some(("Acme Corporation", "Acme Corp")),
            ),
            (
                employment(&["Acme Corporation"], PartyRelation::With),
                "Employment Agreement with Acme Corporation from Jane Doe",
                None,
            ),
            (
                employment(&["Acme Corporation"], PartyRelation::For),
                "Employment Agreement for Sales Staff from Acme Corporation",
                None,
            ),
            (
                employment(&["Acme Corporation"], PartyRelation::For),
                "Services Agreement for Acme Corporation",
                None,
            ),
        ] {
            let approved = format!("2026-04-01 {approved}.pdf");
            let read = respelled_party(&proposal, "pdf", &name(&proposal), &approved);
            assert_eq!(
                read.as_ref()
                    .map(|edit| (edit.kind, edit.from.as_str(), edit.to.as_str())),
                respelled.map(|(from, to)| (RuleKind::Party, from, to)),
                "{approved}"
            );
        }

        // Words that hold no more of them than the party's own name are
        // still a spelling of it, "and" and "of" inside a name among them;
        // and a type may hold any of them.
        for (document_type, parties, relation, approved, kind, from, to) in [
            (
                "Grant Agreement",
                &["Centers for Disease Control"][..],
                PartyRelation::With,
                "Grant Agreement with Center for Disease Control",
                RuleKind::Party,
                "Centers for Disease Control",
                "Center for Disease Control",
            ),
            (
                "Lease",
                &["Back to Basics LLC"][..],
                PartyRelation::With,
                "Lease with Back to Basics",
                RuleKind::Party,
                "Back to Basics LLC",
                "Back to Basics",
            ),
            (
                "Lease",
                &["Johnson"][..],
                PartyRelation::With,
                "Lease with Johnson and Johnson",
                RuleKind::Party,
                "Johnson",
                "Johnson and Johnson",
            ),
            (
                "Application",
                &["City of Victorville"][..],
                PartyRelation::For,
                "Application for Friends of City of Victorville",
                RuleKind::Party,
                "City of Victorville",
                "Friends of City of Victorville",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Acme Property Management",
                RuleKind::Party,
                "Acme Property",
                "Acme Property Management",
            ),
            (
                "Invoice",
                &["Acme"][..],
                PartyRelation::None,
                "Invoice - Acme Corp",
                RuleKind::Party,
                "Acme",
                "Acme Corp",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corp"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Acme Corporation",
                RuleKind::Party,
                "Acme Corp",
                "Acme Corporation",
            ),
            (
                "Application",
                &["City of Victorville"][..],
                PartyRelation::For,
                "Application for Certification for City of Victorville",
                RuleKind::DocumentType,
                "Application",
                "Application for Certification",
            ),
            (
                "Invoice",
                &["Acme"][..],
                PartyRelation::None,
                "Invoice for March - Acme",
                RuleKind::DocumentType,
                "Invoice",
                "Invoice for March",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Quit to Acme Property",
                RuleKind::DocumentType,
                "Notice",
                "Notice to Quit",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::For,
                "Services Agreement for Acme Corporation",
                RuleKind::DocumentType,
                "Employment Agreement",
                "Services Agreement",
            ),
        ] {
            let respelled = proposal(Some(document_type), parties, relation);
            let approved = format!("2026-04-01 {approved}.pdf");
            let lesson = lesson_from_edit(&respelled, "pdf", &name(&respelled), &approved)
                .unwrap_or_else(|| panic!("{approved}: a spelling"));
            assert_eq!(
                (lesson.kind, lesson.from.as_str(), lesson.to.as_str()),
                (kind, from, to),
                "{approved}"
            );
        }
    }

    /// The words a party and something after it are: the party, as house
    /// style compares spellings, then a connecting word or a note.
    #[test]
    fn a_party_and_more_is_the_party_then_a_connecting_word_or_a_note() {
        for (words, party) in [
            ("Carl Malamud from Jane Doe", "Carl Malamud"),
            ("ACME PROPERTY re Lease", "Acme Property"),
            ("Acme Property re: the lease", "Acme Property"),
            ("Acme Property, LLC - signed", "Acme Property LLC"),
            ("Acme (signed)", "Acme"),
            (
                "Beacon Staffing LLC REGARDING the merger",
                "beacon staffing llc",
            ),
        ] {
            assert!(party_and_more(words, party), "{words}");
        }
        for (words, party) in [
            ("Carl Malamud", "Carl Malamud"),
            ("Carl Malamud Jr", "Carl Malamud"),
            ("Smith and Sons", "Smith"),
            ("Friends of Carl Malamud", "Carl Malamud"),
            ("Tenants from Carl Malamud", "Carl Malamud"),
            ("Carlton Malamud from Jane Doe", "Carl"),
            ("from Jane Doe", ""),
        ] {
            assert!(!party_and_more(words, party), "{words}");
        }
    }

    /// Words that join a party to another, or a type given a party, are no
    /// spelling, however the joining words are typed. Each of these was
    /// learned, after two such edits, and named every later document,
    /// ready ones filed without review: "Acme Corporation -> Acme Corp –
    /// Jane Doe" (an en dash), "Acme Corporation -> Jane Doe and Acme
    /// Corporation" (one side of a "between"), "Acme Corporation -> Smith v
    /// Acme Corp" (a case's caption), the type "Employment Agreement ->
    /// Employment Agreement - Contoso Ltd" (a party added to a name that
    /// gave none) or "Employment Agreement - Jane Doe" (in front of the
    /// party clause kept).
    #[test]
    fn a_party_joined_to_another_or_a_type_given_a_party_teaches_nothing() {
        let mut failed = Vec::new();
        for (document_type, parties, relation, approved) in [
            // A party added after the type of a name that gave none.
            (
                "Employment Agreement",
                &[][..],
                PartyRelation::None,
                "Employment Agreement - Contoso Ltd",
            ),
            (
                "Employment Agreement",
                &[][..],
                PartyRelation::None,
                "Employment Agreement for Contoso Ltd",
            ),
            (
                "Employment Agreement",
                &[][..],
                PartyRelation::None,
                "Employment Agreement with Contoso Ltd",
            ),
            (
                "Employment Agreement",
                &[][..],
                PartyRelation::None,
                "Employment Agreement between Contoso Ltd and Jane Doe",
            ),
            (
                "Employment Agreement",
                &[][..],
                PartyRelation::None,
                "Employment Agreement \u{2013} Contoso Ltd",
            ),
            // The divider typed with another dash, or other spaces.
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corp \u{2013} Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corp \u{2014} Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corp -- Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corp\u{a0}-\u{a0}Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::None,
                "Employment Agreement - Contoso Ltd \u{2013} Acme Corporation",
            ),
            // Two parties on one side of a "between".
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Jane Doe and Acme Corporation",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith & Mary Smith and Acme Corporation",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John Smith and Acme Corporation and Beacon LLC",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement between John and Mary Smith and Acme Corporation",
            ),
            // The party kept whole with another joined to it.
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corporation and Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Jane Doe and Acme Corporation",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corporation & Jane Doe",
            ),
            // Words that join two parties outside the name's grammar.
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::None,
                "Employment Agreement - Smith v Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::None,
                "Employment Agreement - Smith v. Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::None,
                "Employment Agreement - Smith vs Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::None,
                "Employment Agreement - Smith versus Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Jane Doe on behalf of Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corp by its Trustee",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corp by Jane Doe",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with John Smith dba Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Holdings fka Acme Corp",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corp attn Jane Doe",
            ),
            // A type given a party in front of the party clause kept.
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Employment Agreement - Jane Doe between John Smith and Acme Corporation",
            ),
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement between Jane Doe with Acme Corporation",
            ),
            (
                "Complaint",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Complaint Smith v Acme with Acme Corporation",
            ),
            (
                "Invoice",
                &["Acme"][..],
                PartyRelation::None,
                "Invoice - Contoso - Acme",
            ),
            // The party kept with a connecting word or a note's start left
            // dangling after it.
            (
                "Employment Agreement",
                &["Acme Corporation"][..],
                PartyRelation::With,
                "Employment Agreement with Acme Corporation re",
            ),
            (
                "Notice",
                &["Acme Property"][..],
                PartyRelation::To,
                "Notice to Acme Property from",
            ),
            // The price: a type lengthened on a name that gave no party,
            // a party put in a note's parenthesis, or a party of two words
            // or more given another with "and", is made again by hand.
            ("Invoice", &[][..], PartyRelation::None, "Invoice for March"),
            (
                "Lease",
                &["Acme"][..],
                PartyRelation::With,
                "Lease with Acme (UK)",
            ),
            (
                "Lease",
                &["Acme Property"][..],
                PartyRelation::With,
                "Lease with Acme Property and Casualty",
            ),
        ] {
            let joined = proposal(Some(document_type), parties, relation);
            let approved = format!("2026-04-01 {approved}.pdf");
            if let Some(lesson) = lesson_from_edit(&joined, "pdf", &name(&joined), &approved) {
                failed.push(format!(
                    "{approved}: {:?} {} -> {}",
                    lesson.kind, lesson.from, lesson.to
                ));
            }
        }

        // Still spellings: a parenthesis with more of the name after it, a
        // "re" that ends the words, "and" or "&" in a party of one word or
        // in words that do not hold the party whole, a "V" in capitals, a
        // comma, and a type with no party in it.
        for (document_type, parties, relation, approved, kind, from, to) in [
            (
                "Lease",
                &["Acme Ltd"][..],
                PartyRelation::With,
                "Lease with Acme (UK) Ltd",
                RuleKind::Party,
                "Acme Ltd",
                "Acme (UK) Ltd",
            ),
            (
                "Lease",
                &["Acme"][..],
                PartyRelation::With,
                "Lease with Acme (UK) Ltd",
                RuleKind::Party,
                "Acme",
                "Acme (UK) Ltd",
            ),
            (
                "Reinsurance Agreement",
                &["Swiss Reinsurance Company Ltd"][..],
                PartyRelation::With,
                "Reinsurance Agreement with Swiss Re",
                RuleKind::Party,
                "Swiss Reinsurance Company Ltd",
                "Swiss Re",
            ),
            (
                "Reinsurance Agreement",
                &["Swiss Re"][..],
                PartyRelation::With,
                "Reinsurance Agreement with Swiss Re Ltd",
                RuleKind::Party,
                "Swiss Re",
                "Swiss Re Ltd",
            ),
            (
                "Lease",
                &["Johnson"][..],
                PartyRelation::With,
                "Lease with Johnson & Johnson",
                RuleKind::Party,
                "Johnson",
                "Johnson & Johnson",
            ),
            (
                "Lease",
                &["Barnes Noble"][..],
                PartyRelation::With,
                "Lease with Barnes and Noble",
                RuleKind::Party,
                "Barnes Noble",
                "Barnes and Noble",
            ),
            (
                "Subscription Agreement",
                &["Acme Fund"][..],
                PartyRelation::With,
                "Subscription Agreement with Acme Fund V",
                RuleKind::Party,
                "Acme Fund",
                "Acme Fund V",
            ),
            (
                "Lease",
                &["Acme Corp"][..],
                PartyRelation::With,
                "Lease with Acme Corp, Inc.",
                RuleKind::Party,
                "Acme Corp",
                "Acme Corp, Inc.",
            ),
            (
                "Invoice",
                &[][..],
                PartyRelation::None,
                "Services Invoice",
                RuleKind::DocumentType,
                "Invoice",
                "Services Invoice",
            ),
            (
                "Employment Agreement",
                &["John Smith", "Acme Corporation"][..],
                PartyRelation::Between,
                "Executive Employment Agreement between John Smith and Acme Corporation",
                RuleKind::DocumentType,
                "Employment Agreement",
                "Executive Employment Agreement",
            ),
        ] {
            let respelled = proposal(Some(document_type), parties, relation);
            let approved = format!("2026-04-01 {approved}.pdf");
            let lesson = lesson_from_edit(&respelled, "pdf", &name(&respelled), &approved)
                .map(|lesson| (lesson.kind, lesson.from, lesson.to));
            if lesson != Some((kind, from.to_owned(), to.to_owned())) {
                failed.push(format!("{approved}: {lesson:?}, not a spelling"));
            }
        }

        // A parenthesis with more of the name after it is part of the
        // party, for filing too; one that ends the words is a note.
        let letter = proposal(Some("Letter"), &["Smith"], PartyRelation::To);
        for (approved, read) in [
            ("Letter to Smith (UK) Ltd", Some("Smith (UK) Ltd")),
            ("Letter to Smith (Trustee)", None),
            ("Letter to Smith (Trustee) re Lease", None),
        ] {
            let approved = format!("2026-04-01 {approved}.pdf");
            let respelled =
                respelled_party(&letter, "pdf", &name(&letter), &approved).map(|edit| edit.to);
            let party = approved_party(&letter, "pdf", &name(&letter), &approved);
            if respelled.as_deref() != read || party.as_deref() != read {
                failed.push(format!("{approved}: read {respelled:?} and {party:?}"));
            }
        }
        for (words, party, more) in [
            ("Acme (UK) Ltd", "Acme", false),
            ("Acme (signed)", "Acme", true),
            ("Acme \u{2013} signed", "Acme", true),
            ("Acme\u{a0}for\u{a0}Jane Doe", "Acme", true),
        ] {
            if party_and_more(words, party) != more {
                failed.push(format!("{words}: party and more is not {more}"));
            }
        }
        assert!(failed.is_empty(), "{failed:#?}");
    }

    #[test]
    fn keys_disregard_case_punctuation_and_spacing_but_never_words() {
        assert_eq!(
            HouseRule::key("Contoso Worldwide, Inc."),
            HouseRule::key("  CONTOSO   WORLDWIDE INC ")
        );
        assert_ne!(
            HouseRule::key("Contoso"),
            HouseRule::key("Contoso Worldwide")
        );
        let rule = HouseRule::new(RuleKind::Party, "Acme Corp.", "Acme Corp");
        assert!(
            !rule.is_meaningful(),
            "punctuation alone is not a preference"
        );
        assert!(!HouseRule::new(RuleKind::Party, "Acme", "???").is_meaningful());
    }
}
