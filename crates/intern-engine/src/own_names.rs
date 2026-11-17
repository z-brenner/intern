//! The name a document is filed under: the organization it is filed for,
//! then what it is.
//!
//! ```text
//! YYYY-MM-DD <entity> - <subject>.<ext>
//! 2026-10-16 Vistage International - Host Chair SOW for John Smith.pdf
//! 2024-11-09 EGN Denmark - Incorporation Email Communications.pdf
//! 2025-04-30 Nimbus Orchard Supply - Invoice.pdf
//! 2026-12-29 Notice of Termination - John Smith.pdf
//! ```
//!
//! A folder of filings is read by who each one is for before what it is, so
//! the name leads with that organization - the entity - and the subject
//! follows: the type, the connecting word and the other side, never the
//! entity again. The entity is, in order:
//!
//! 1. the person's own organization, as Settings names it, wherever the
//!    reading found it among the document's parties - the configured name
//!    that matches it most specifically, or the document's own name for it
//!    where that is more specific than any configured one;
//! 2. otherwise the first organization among the name's parties, or the
//!    side that issued the document where the name's connecting word is
//!    about the other side ("Notice of Termination for John Smith" is the
//!    employer's);
//! 3. otherwise none, and the name is the engine's own.
//!
//! A legal form ends a company's name in the document and says nothing in
//! a folder of filings, so the entity is written without it: "Vistage
//! Malaysia", not "Vistage Malaysia Sdn. Bhd.".
//!
//! Your organization: a firm's own name is on almost every document it
//! files - the engagement letter it sent, the contract it signed, the
//! invoice it was billed - so a name that carries it in the subject says the
//! same thing over and over, and a Party layout scatters one counterparty's
//! documents across folders by whichever side the model happened to name
//! first. Named once in Settings, it leads the name and the subject and the
//! Party folder carry the other side.
//!
//! Like house style, this is applied deterministically after validation:
//! the evidence and the description stay the document's own words, and only
//! the name a document is filed under changes. Nothing here asks the model
//! anything. The engine's own name ([`crate::compose_filename`]) is not
//! touched, so what a reading is scored on stays the same.
//!
//! The document's own filename is read last ([`NameView::with_original`]):
//! where it already says what the reading found - the entity, the other
//! side, the type in its own words - its wording for the type is kept, and
//! a date it gives one way only dates a reading that found none.

use crate::domain::{
    ComposedName, PartyRelation, PartyRole, Support, ValidatedFacts, ValidatedProposal,
};
use crate::house_style::{HouseRule, RuleKind};
use crate::naming::{
    ENTITY_SEPARATOR, Spelling, compose_led, entity_segment, filename_dates, is_junk_filename,
    party_as_named,
};

/// An own name whose key is shorter than this would match far too much -
/// "Co", "LLP", "The" - so it is ignored rather than trusted.
const MIN_OWN_KEY_CHARS: usize = 4;

/// Whether `party` names the organization `own` names.
///
/// Matched under the same key house style uses - case, punctuation and
/// spacing disregarded - so "CONTOSO WORLDWIDE INC" is "Contoso Worldwide,
/// Inc.". A whole-word prefix also matches, either way round: a document that
/// says "Contoso" is the firm that wrote "Contoso Worldwide, Inc." in
/// Settings, and one that says "Contoso Worldwide, Inc. (UK Branch)" is too.
/// Words are never loosened: "Contoso" is not "Contosoft".
pub fn is_own_name(party: &str, own: &str) -> bool {
    let own = HouseRule::key(own);
    if own.chars().count() < MIN_OWN_KEY_CHARS {
        return false;
    }
    let party = HouseRule::key(party);
    if party.is_empty() {
        return false;
    }
    party == own || starts_with_words(&party, &own) || starts_with_words(&own, &party)
}

/// `longer` is `shorter` followed by more whole words.
fn starts_with_words(longer: &str, shorter: &str) -> bool {
    longer
        .strip_prefix(shorter)
        .is_some_and(|rest| rest.starts_with(' '))
}

/// The proposal as a name filed by the counterparty carried it before names
/// led with their entity, and the parties left out because they are the
/// person's own organization. Names composed by that build are still read
/// with it ([`NameView::read_edit`]), as house style reads a name an
/// earlier version proposed.
///
/// Unchanged when no own name is configured, when the document names fewer
/// than two parties, or when it names only the organization or only others:
/// the only party is never dropped, because a name with no party at all says
/// less than one that names the firm. Otherwise the organization's parties
/// go and the others keep their order.
///
/// The connecting word follows. "between" needs two sides, so one remaining
/// party is "with" it. A directional word - "from", "to", "for" - is about
/// the first party alone, which is why validation keeps only that one; a
/// proposal stored before it did can still carry two. When that first party
/// was the organization, the word described the firm and not the other side,
/// so the name keeps the other side with the plain separator rather than a
/// direction the document never stated about them: an invoice the firm
/// issued is not an invoice "from" its customer.
///
/// The evidence and the description are untouched.
pub fn counterparty_view(
    proposal: &ValidatedProposal,
    own: &[String],
) -> (ValidatedProposal, Vec<String>) {
    if own.is_empty() || proposal.parties.len() < 2 {
        return (proposal.clone(), Vec::new());
    }
    let is_own = |party: &String| own.iter().any(|name| is_own_name(party, name));
    let (dropped, others): (Vec<String>, Vec<String>) =
        proposal.parties.iter().cloned().partition(is_own);
    if dropped.is_empty() || others.is_empty() {
        return (proposal.clone(), Vec::new());
    }
    let first_was_own = proposal.parties.first().is_some_and(is_own);
    let mut view = proposal.clone();
    view.party_relation = match proposal.party_relation {
        PartyRelation::Between if others.len() == 1 => PartyRelation::With,
        PartyRelation::From | PartyRelation::To | PartyRelation::For if first_was_own => {
            PartyRelation::None
        }
        relation => relation,
    };
    view.parties = others;
    (view, dropped)
}

/// The legal forms a company's name ends on, compared without case, stops,
/// commas or brackets: "Inc.", "LLC", "Sdn. Bhd.", "Pte Ltd", "A/S", "B.V.".
/// "Company" is not one: it is as often part of the name ("The Coca-Cola
/// Company"), and "AS" is an English word.
const LEGAL_FORMS: &[&str] = &[
    "inc",
    "incorporated",
    "llc",
    "ltd",
    "limited",
    "corp",
    "corporation",
    "co",
    "plc",
    "llp",
    "lp",
    "pllc",
    "pc",
    "na",
    "gmbh",
    "ag",
    "kg",
    "se",
    "a/s",
    "aps",
    "ab",
    "oy",
    "oyj",
    "bv",
    "nv",
    "sa",
    "sas",
    "sarl",
    "srl",
    "spa",
    "kk",
    "sdn",
    "bhd",
    "pte",
    "pty",
    "pvt",
    "lda",
    "ltda",
];

/// `name` without the legal form it ends on, and the punctuation and joining
/// word the form leaves behind: "Vistage Malaysia Sdn. Bhd." is "Vistage
/// Malaysia", "Contoso Worldwide, Inc." is "Contoso Worldwide" and "John
/// Deere & Co." is "John Deere". A name that is nothing but a legal form,
/// or has none, is returned as it is, trimmed.
pub fn without_legal_form(name: &str) -> String {
    let trimmed = name.trim();
    let mut words = Vec::new();
    let mut start = None;
    for (at, character) in trimmed.char_indices() {
        match (character.is_whitespace(), start) {
            (true, Some(from)) => {
                words.push((from, at));
                start = None;
            }
            (false, None) => start = Some(at),
            _ => {}
        }
    }
    if let Some(from) = start {
        words.push((from, trimmed.len()));
    }
    let is_form = |(from, to): (usize, usize)| {
        let word = trimmed[from..to]
            .chars()
            .filter(|character| !matches!(character, '.' | ',' | '(' | ')'))
            .flat_map(char::to_lowercase)
            .collect::<String>();
        LEGAL_FORMS.contains(&word.as_str())
    };
    let mut kept = words.len();
    while kept > 1 && is_form(words[kept - 1]) {
        kept -= 1;
    }
    if kept == words.len() {
        return trimmed.to_owned();
    }
    let mut cut = &trimmed[..words[kept - 1].1];
    loop {
        let before = cut.len();
        cut = cut.trim_end_matches([',', ';', ' ', '-', '&', '.']);
        if let Some(rest) = cut
            .len()
            .checked_sub(4)
            .filter(|&at| cut.is_char_boundary(at) && cut[at..].eq_ignore_ascii_case(" and"))
            .map(|at| &cut[..at])
        {
            cut = rest;
        }
        if cut.len() == before {
            break;
        }
    }
    if cut.is_empty() {
        trimmed.to_owned()
    } else {
        cut.to_owned()
    }
}

/// Whether `name` is an organization's rather than a person's: it ends on a
/// legal form ([`without_legal_form`]), or the engine's own reading of a
/// party's name takes it for one - a company's noun ("Supply", "Holdings",
/// "Bank") or a body's ("Committee", "Office for ...").
pub fn is_organization(name: &str) -> bool {
    HouseRule::key(&without_legal_form(name)) != HouseRule::key(name)
        || crate::facts::is_organization_name(name)
}

/// Where a name's entity comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityKind {
    /// The person's own organization, as Settings names it.
    Own,
    /// The first organization among the name's parties.
    Party,
    /// The side that issued the document, as the reading found it, where
    /// the name's parties are the other side.
    Issuer,
}

/// The organization a name leads with.
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub kind: EntityKind,
    /// The party it stands for, as the styled proposal or the reading's
    /// facts spell it.
    pub party: String,
    /// The words the name leads with: a configured name, or the party,
    /// without its legal form; or a reviewer's spelling of the party.
    pub words: String,
    /// Whether the words are a person's - a name typed in Settings, or a
    /// house-style spelling - and are carried as typed, rather than the
    /// document's, which are title-cased when printed in capitals.
    pub chosen: bool,
}

/// The validated facts as the name a document is filed under carries them:
/// the entity it leads with, and the subject after it. House style has
/// already been applied (see [`crate::HouseStyle::apply`]); this is the
/// stage after it. The one view the name, the layout folder, a lesson from
/// an edit and a filing report are all read from.
#[derive(Clone, Debug, PartialEq)]
pub struct NameView {
    /// What follows the entity: the type, the connecting word and the
    /// parties other than the entity, as the engine's grammar composes them.
    pub subject: ValidatedProposal,
    pub entity: Option<Entity>,
    /// The person's own organization's parties neither the entity nor in
    /// the subject: left out of the name because it names someone else.
    pub left_out: Vec<String>,
    /// The words the document's own filename gives its type, where they
    /// replace the reading's in [`Self::subject`] (see
    /// [`Self::with_original`]).
    pub original_type: Option<String>,
    /// The house-style spellings that styled the proposal.
    applied: Vec<HouseRule>,
    /// The proposal as names composed before names led with their entity
    /// read it (see [`counterparty_view`]).
    legacy: ValidatedProposal,
    /// Every party the reading found, as the styled proposal and the facts
    /// spell them.
    known: Vec<String>,
}

/// How an approved name differs from the one proposed, around the entity.
#[derive(Debug, PartialEq)]
pub enum EditedName<'a> {
    /// The entity kept as proposed, or no entity: the two names without it,
    /// to be read with `grammar`, the proposal that composed what follows.
    Subject {
        grammar: &'a ValidatedProposal,
        proposed: String,
        approved: String,
    },
    /// Only the entity changed: the words put in its place, as typed.
    Entity { words: String },
    /// The entity and the subject both changed, or the name was rewritten.
    Rewritten,
}

/// What a person's own spellings and organization say about the parties.
struct Spellings<'a> {
    applied: &'a [HouseRule],
    own: &'a [String],
}

impl Spellings<'_> {
    /// A styled party, and the document's own word a house-style rule
    /// respelled into it.
    fn sources<'b>(&'b self, party: &'b str) -> impl Iterator<Item = &'b str> + 'b {
        std::iter::once(party).chain(
            self.applied
                .iter()
                .filter(move |rule| {
                    rule.kind == RuleKind::Party && HouseRule::key(&rule.to) == HouseRule::key(party)
                })
                .map(|rule| rule.from.as_str()),
        )
    }

    fn is_own(&self, party: &str) -> bool {
        self.sources(party)
            .any(|source| self.own.iter().any(|own| is_own_name(source, own)))
    }

    fn is_organization(&self, party: &str) -> bool {
        self.sources(party).any(is_organization)
    }

    /// Whether a house-style rule wrote `party`: the reviewer's spelling.
    fn chosen(&self, party: &str) -> bool {
        self.applied
            .iter()
            .any(|rule| rule.kind == RuleKind::Party && rule.to == party)
    }

    /// Every key `party` is known by: its own, without its legal form, as a
    /// name writes it, and the document's word a rule respelled.
    fn keys(&self, party: &str) -> Vec<String> {
        let mut keys = Vec::new();
        for source in self.sources(party) {
            for key in party_keys(source) {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        keys
    }

    fn same_party(&self, left: &str, right: &str) -> bool {
        let right = self.keys(right);
        self.keys(left).iter().any(|key| right.contains(key))
    }

    /// The words an own party is named by: the configured name that
    /// matches it most specifically, or the party's own words where they
    /// are more specific than any configured name, or where two configured
    /// names, each more specific than the party, would do as well.
    fn own_words(&self, party: &str) -> (String, bool) {
        let words_of = |name: &str| {
            HouseRule::key(&without_legal_form(name))
                .split(' ')
                .filter(|word| !word.is_empty())
                .count()
        };
        let matching = self
            .own
            .iter()
            .filter(|own| self.sources(party).any(|source| is_own_name(source, own)))
            .collect::<Vec<_>>();
        let document = || {
            if self.chosen(party) {
                (party.to_owned(), true)
            } else {
                (without_legal_form(party), false)
            }
        };
        let Some(most) = matching.iter().map(|own| words_of(own)).max() else {
            return document();
        };
        let mut best = matching
            .iter()
            .filter(|own| words_of(own) == most)
            .map(|own| without_legal_form(own))
            .collect::<Vec<_>>();
        best.dedup_by(|left, right| HouseRule::key(left) == HouseRule::key(right));
        let party_key = HouseRule::key(&without_legal_form(party));
        let more_specific = party_key.split(' ').count() > most;
        match best.as_slice() {
            [configured] if !more_specific => (configured.clone(), true),
            _ => best
                .iter()
                .find(|configured| HouseRule::key(configured) == party_key)
                .map_or_else(document, |configured| (configured.clone(), true)),
        }
    }
}

/// A party's keys as house style compares spellings: as written, without
/// its legal form, and as a composed name writes it.
fn party_keys(party: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for key in [
        Some(HouseRule::key(party)),
        Some(HouseRule::key(&without_legal_form(party))),
        party_as_named(party).map(|named| HouseRule::key(&named)),
    ]
    .into_iter()
    .flatten()
    {
        if !key.is_empty() && !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

/// The roles of the side that issues a document to the other: an invoice's
/// seller, a notice's sender, an employer's letter to its employee.
fn issues(role: Option<PartyRole>) -> bool {
    matches!(
        role,
        Some(
            PartyRole::Issuer
                | PartyRole::Sender
                | PartyRole::Vendor
                | PartyRole::Seller
                | PartyRole::Employer
                | PartyRole::Landlord
                | PartyRole::Lender
                | PartyRole::Licensor
        )
    )
}

impl NameView {
    /// The view of `styled`, the proposal house style respelled with the
    /// rules `applied`, by the person's own organization's names `own`.
    /// `facts` are the reading's validated facts, where it kept them: every
    /// party the document supports, which is where the organization is
    /// found when the name's parties are only the other side.
    pub fn of(
        styled: &ValidatedProposal,
        applied: &[HouseRule],
        facts: Option<&ValidatedFacts>,
        own: &[String],
    ) -> Self {
        let spellings = Spellings { applied, own };
        // The parties the reading found besides the name's: supported, and
        // neither copied in nor signing for another.
        let found = facts
            .map(|facts| {
                facts
                    .parties
                    .iter()
                    .filter(|party| {
                        matches!(party.support, Support::Cited | Support::Context)
                            && !party.copied
                            && !party.signatory
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let beside = |name: &str| {
            !styled
                .parties
                .iter()
                .any(|party| spellings.same_party(party, name))
        };

        // 1. The person's own organization, the most specifically named.
        let mut entity: Option<Entity> = None;
        let mut most = 0;
        for party in styled.parties.iter().map(String::as_str).chain(
            found
                .iter()
                .map(|party| party.name.as_str())
                .filter(|name| beside(name)),
        ) {
            if !spellings.is_own(party) {
                continue;
            }
            let (words, chosen) = spellings.own_words(party);
            let count = HouseRule::key(&words).split(' ').count();
            if entity.is_none() || count > most {
                most = count;
                entity = Some(Entity {
                    kind: EntityKind::Own,
                    party: party.to_owned(),
                    words,
                    chosen,
                });
            }
        }

        // 2. The first organization among the name's parties, where its
        // connecting word leaves them first; or the side that issued it.
        if entity.is_none() {
            let first_side = matches!(
                styled.party_relation,
                PartyRelation::From
                    | PartyRelation::Between
                    | PartyRelation::With
                    | PartyRelation::None
            );
            let named = styled
                .parties
                .iter()
                .filter(|_| first_side)
                .map(|party| (party.as_str(), EntityKind::Party));
            let issuing = found
                .iter()
                .filter(|party| issues(party.role))
                .filter_map(|party| {
                    match styled
                        .parties
                        .iter()
                        .position(|named| spellings.same_party(named, &party.name))
                    {
                        // The party the connecting word is about is not
                        // the one that issued it to them.
                        Some(0) if !first_side => None,
                        Some(index) => Some((styled.parties[index].as_str(), EntityKind::Party)),
                        None => Some((party.name.as_str(), EntityKind::Issuer)),
                    }
                });
            entity = named
                .chain(issuing)
                .find(|(party, _)| spellings.is_organization(party))
                .map(|(party, kind)| {
                    let chosen = spellings.chosen(party);
                    Entity {
                        kind,
                        party: party.to_owned(),
                        words: if chosen {
                            party.to_owned()
                        } else {
                            without_legal_form(party)
                        },
                        chosen,
                    }
                });
        }

        // The subject: every other party, the organization's own left out
        // where someone else is left.
        let is_entity = |party: &str| {
            entity.as_ref().is_some_and(|entity| {
                spellings.same_party(party, &entity.party)
                    || (entity.kind == EntityKind::Own && is_own_name(party, &entity.words))
            })
        };
        let rest = styled
            .parties
            .iter()
            .filter(|party| !is_entity(party))
            .cloned()
            .collect::<Vec<_>>();
        let (mine, others): (Vec<String>, Vec<String>) =
            rest.iter().cloned().partition(|party| spellings.is_own(party));
        let (parties, left_out) = if others.is_empty() {
            (rest, Vec::new())
        } else {
            (others, mine)
        };
        let first_gone = styled
            .parties
            .first()
            .is_some_and(|first| !parties.contains(first));
        let mut subject = styled.clone();
        subject.party_relation = match styled.party_relation {
            _ if parties.is_empty() => PartyRelation::None,
            // The direction described the party that went, not the one left.
            PartyRelation::From | PartyRelation::To | PartyRelation::For if first_gone => {
                PartyRelation::None
            }
            PartyRelation::Between if parties.len() < 2 => PartyRelation::With,
            relation => relation,
        };
        subject.parties = parties;

        let mut known = styled.parties.clone();
        known.extend(
            found
                .iter()
                .map(|party| party.name.clone())
                .filter(|name| beside(name)),
        );
        Self {
            subject,
            entity,
            left_out,
            original_type: None,
            applied: applied.to_vec(),
            legacy: counterparty_view(styled, own).0,
            known,
        }
    }

    /// The proposal as names composed before names led with their entity
    /// carried it (see [`counterparty_view`]): how such a name, still
    /// waiting or filed, is read.
    pub fn legacy(&self) -> &ValidatedProposal {
        &self.legacy
    }

    /// The entity as the name writes it: "Vistage International".
    pub fn entity_segment(&self) -> Option<String> {
        self.entity.as_ref().and_then(|entity| {
            entity_segment(
                &entity.words,
                if entity.chosen {
                    Spelling::Chosen
                } else {
                    Spelling::Document
                },
            )
        })
    }

    /// The name this view composes, collision-free among `existing_names`.
    /// A type or a party a house-style rule wrote is carried as the
    /// reviewer typed it, as [`crate::compose_styled_filename`] carries it.
    pub fn compose(&self, extension: &str, existing_names: &[&str]) -> ComposedName {
        let chosen = |kind: RuleKind| {
            self.applied
                .iter()
                .filter(|rule| rule.kind == kind)
                .map(|rule| rule.to.as_str())
                .collect::<Vec<_>>()
        };
        compose_led(
            &self.subject,
            self.entity.as_ref().map(|entity| {
                (
                    entity.words.as_str(),
                    if entity.chosen {
                        Spelling::Chosen
                    } else {
                        Spelling::Document
                    },
                )
            }),
            extension,
            existing_names,
            &chosen(RuleKind::DocumentType),
            &chosen(RuleKind::Party),
        )
    }

    /// The facts a filing is put away and reported under: the subject's,
    /// with the entity's party first where the name leads with one that is
    /// not the person's own organization - so the Party folder is the
    /// organization the name starts with - and, for their own, the other
    /// side, or the organization itself where the document names nobody
    /// else.
    pub fn filed(&self) -> ValidatedProposal {
        self.with_parties(
            self.entity.as_ref().map(|entity| entity.party.clone()),
            self.subject.parties.clone(),
        )
    }

    /// [`Self::filed`] with `entity` standing for the entity's party and
    /// `parties` for the subject's: the parties as an approved name gives
    /// them. One given twice is filed once.
    pub fn with_parties(&self, entity: Option<String>, parties: Vec<String>) -> ValidatedProposal {
        let mut filed = self.subject.clone();
        let own = self
            .entity
            .as_ref()
            .is_some_and(|entity| entity.kind == EntityKind::Own);
        filed.parties = match entity {
            Some(entity) if !own => std::iter::once(entity).chain(parties).collect(),
            Some(entity) if parties.is_empty() => vec![entity],
            _ => parties,
        };
        let mut seen = Vec::new();
        filed.parties.retain(|party| {
            let key = HouseRule::key(party);
            let first = !seen.contains(&key);
            seen.push(key);
            first
        });
        filed
    }

    /// `filename` without the entity this view leads names with: the date,
    /// and what follows the entity. `filename` itself where the view has no
    /// entity; `None` where the name does not lead with it.
    pub fn without_entity(&self, filename: &str) -> Option<String> {
        let Some(segment) = self.entity_segment() else {
            return Some(filename.to_owned());
        };
        let trimmed = filename.trim();
        let (date, rest) = match trimmed.get(..10) {
            Some(date)
                if crate::evidence::is_valid_iso_date(date)
                    && trimmed[10..].starts_with(' ') =>
            {
                (Some(date), trimmed[10..].trim_start())
            }
            _ => (None, trimmed),
        };
        let subject = rest
            .strip_prefix(segment.as_str())?
            .strip_prefix(ENTITY_SEPARATOR)?;
        Some(match date {
            Some(date) => format!("{date} {subject}"),
            None => subject.to_owned(),
        })
    }

    /// How the name `approved` edits the name `proposed` this view composed,
    /// around the entity. A name composed before names led with their
    /// entity is read with that build's grammar (see [`counterparty_view`]).
    /// `extension` is the source file's.
    pub fn read_edit(&self, extension: &str, proposed: &str, approved: &str) -> EditedName<'_> {
        if self.entity.is_none() {
            return EditedName::Subject {
                grammar: &self.subject,
                proposed: proposed.to_owned(),
                approved: approved.to_owned(),
            };
        }
        let Some(proposed_subject) = self.without_entity(proposed) else {
            return EditedName::Subject {
                grammar: &self.legacy,
                proposed: proposed.to_owned(),
                approved: approved.to_owned(),
            };
        };
        if let Some(approved_subject) = self.without_entity(approved) {
            return EditedName::Subject {
                grammar: &self.subject,
                proposed: proposed_subject,
                approved: approved_subject,
            };
        }
        let subject = name_stem(&proposed_subject, extension);
        let after = name_stem(approved, extension);
        match after
            .strip_suffix(subject.as_str())
            .and_then(|rest| rest.strip_suffix(ENTITY_SEPARATOR))
            .map(str::trim)
            .filter(|words| !words.is_empty())
        {
            Some(words) if !subject.is_empty() => EditedName::Entity {
                words: words.to_owned(),
            },
            _ => EditedName::Rewritten,
        }
    }

    /// The view with what the document's own filename `original` says (its
    /// stem, without the extension) where that agrees with the reading: its
    /// words for the type in the subject's place, and its date where the
    /// reading found none.
    ///
    /// A filename is read only where it names what the reading found and
    /// nothing that contradicts it, so a name already right - "2025-03-26
    /// Vistage Malaysia - Employment Termination Letter for Johnny
    /// Appleseed" - is proposed as it is, and one nearly right keeps its
    /// wording with the date put first, or put in order:
    ///
    /// - it is no scanner's, camera's or copy's name ([`is_junk_filename`]);
    /// - any date it gives reads one way only and is the reading's
    ///   ([`crate::date_in_filename`]);
    /// - its parts, between " - " and around a date, are the entity, the
    ///   subject's parties - with the connecting word before one - another
    ///   party the reading found, and words for the type in one part only,
    ///   which hold no party's name;
    /// - it names the entity and every party of the subject, in any of the
    ///   spellings the reading or a name gives them;
    /// - its type words give the reading's type: all its words, the same
    ///   kind of document ("Letter", "Invoice"), or its initials ("SOW"
    ///   for "Statement of Work"). A reading with no type needs the entity
    ///   or a party named instead.
    ///
    /// The relation and the parties stay the reading's: the filename's
    /// words replace only the type's, so a filename that says "with" where
    /// the reading says "for" is named "for". A type a reviewer's spelling
    /// wrote is kept.
    pub fn with_original(mut self, original: Option<&str>) -> Self {
        let Some(reading) = original
            .map(str::trim)
            .filter(|stem| !stem.is_empty())
            .and_then(|stem| self.original_reading(stem))
        else {
            return self;
        };
        if self.subject.document_date.is_none() {
            self.subject.document_date = reading.date;
        }
        let respelled = self
            .applied
            .iter()
            .any(|rule| rule.kind == RuleKind::DocumentType);
        let same = self
            .subject
            .document_type
            .as_deref()
            .is_some_and(|document_type| {
                HouseRule::key(document_type) == HouseRule::key(&reading.wording)
            });
        if !respelled && !same {
            self.subject.document_type = Some(reading.wording.clone());
            self.original_type = Some(reading.wording);
        }
        self
    }

    /// What the document's own filename's stem says that agrees with the
    /// reading (see [`Self::with_original`]), or `None`.
    fn original_reading(&self, stem: &str) -> Option<OriginalReading> {
        if is_junk_filename(stem) {
            return None;
        }
        let dates = filename_dates(stem)?;
        let date = match dates.first() {
            Some((first, _)) if dates.iter().all(|(date, _)| date == first) => Some(first.clone()),
            Some(_) => return None,
            None => None,
        };
        if let (Some(given), Some(read)) = (&date, &self.subject.document_date)
            && given != read
        {
            return None;
        }
        // The words, a date standing as a break between parts.
        let mut words = String::with_capacity(stem.len());
        let mut last = 0;
        for (_, range) in &dates {
            words.push_str(&stem[last..range.start]);
            words.push_str(ENTITY_SEPARATOR);
            last = range.end;
        }
        words.push_str(&stem[last..]);
        let words = without_copy_marks(&parts_spaced(&words));

        let spellings = Spellings {
            applied: &self.applied,
            own: &[],
        };
        let mut known = Vec::new();
        if let Some(entity) = &self.entity {
            let mut keys = spellings.keys(&entity.party);
            keys.push(HouseRule::key(&entity.words));
            known.push(Known {
                part: Part::Entity,
                keys,
                own: (entity.kind == EntityKind::Own).then(|| entity.words.clone()),
            });
        }
        for (index, party) in self.subject.parties.iter().enumerate() {
            known.push(Known {
                part: Part::Subject(index),
                keys: spellings.keys(party),
                own: None,
            });
        }
        for party in &self.known {
            known.push(Known {
                part: Part::Other,
                keys: spellings.keys(party),
                own: None,
            });
        }

        let mut named = Vec::new();
        let mut wording: Option<String> = None;
        for part in words.split(ENTITY_SEPARATOR) {
            let part = part.trim_matches(|character: char| {
                character.is_whitespace() || matches!(character, ',' | ';' | '(' | ')' | '-')
            });
            let mut part_words = part.split_whitespace().collect::<Vec<_>>();
            named.extend(strip_known(&mut part_words, &known));
            if part_words.is_empty() {
                continue;
            }
            if wording.is_some() {
                // Words for the type in two parts: which is it?
                return None;
            }
            wording = Some(part_words.join(" "));
        }
        let wording = wording?;
        let wording_key = HouseRule::key(&wording);
        let anchored = named.iter().any(|part| *part != Part::Other);
        if self.entity.is_some() && !named.contains(&Part::Entity)
            || (0..self.subject.parties.len()).any(|index| !named.contains(&Part::Subject(index)))
            || known.iter().any(|known| {
                known
                    .keys
                    .iter()
                    .any(|key| holds_words(&wording_key, key))
            })
            || is_junk_filename(&wording)
            || wording.chars().count() > MAX_ORIGINAL_TYPE_CHARS
        {
            return None;
        }
        let supported = match self.subject.document_type.as_deref() {
            Some(document_type) => gives_type(&wording, document_type),
            None => anchored,
        };
        supported.then_some(OriginalReading { wording, date })
    }
}

/// The most characters of a filename's words for a type a name takes: more
/// is a sentence, or a filename that is not a type at all.
const MAX_ORIGINAL_TYPE_CHARS: usize = 80;

/// What a document's own filename says that agrees with its reading.
struct OriginalReading {
    /// Its words for the type.
    wording: String,
    /// The date it gives, which is the reading's or stands for none.
    date: Option<String>,
}

/// Which part of a name a run of a filename's words is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Part {
    Entity,
    Subject(usize),
    Other,
}

/// A party a filename may name, by every key it is known by.
struct Known {
    part: Part,
    keys: Vec<String>,
    /// For the person's own organization, the words it is named by, which
    /// a shorter name for it matches as Settings matches one, and the same
    /// name with a legal form.
    own: Option<String>,
}

impl Known {
    fn names(&self, words: &[&str]) -> bool {
        let text = words.join(" ");
        let key = HouseRule::key(&text);
        !key.is_empty()
            && (self.keys.contains(&key)
                || self.own.as_deref().is_some_and(|own| {
                    let own = HouseRule::key(&without_legal_form(own));
                    HouseRule::key(&without_legal_form(&text)) == own
                        || (is_own_name(&text, &own) && starts_with_words(&own, &key))
                }))
    }
}

/// The connecting words a filename puts before a party, and "and" between
/// two.
const BEFORE_A_PARTY: &[&str] = &["with", "from", "for", "to", "between", "and", "by", "&"];

/// Takes the parties `known` names off the ends of `words`, the longest
/// first, with a connecting word before one at the end: "Host Chair SOW for
/// John Smith" leaves "Host Chair SOW". The parts named, in the order taken.
fn strip_known(words: &mut Vec<&str>, known: &[Known]) -> Vec<Part> {
    let mut named = Vec::new();
    loop {
        let count = words.len();
        let at_end = (1..=count).rev().find_map(|length| {
            known
                .iter()
                .find(|known| known.names(&words[count - length..]))
                .map(|known| (length, known.part))
        });
        if let Some((length, part)) = at_end {
            words.truncate(count - length);
            named.push(part);
            if words.last().is_some_and(|word| {
                BEFORE_A_PARTY.contains(&word.to_lowercase().trim_matches(',').trim_end_matches(':'))
            }) {
                words.pop();
            }
            continue;
        }
        let at_start = (1..=count).rev().find_map(|length| {
            known
                .iter()
                .find(|known| known.names(&words[..length]))
                .map(|known| (length, known.part))
        });
        if let Some((length, part)) = at_start {
            words.drain(..length);
            named.push(part);
            continue;
        }
        return named;
    }
}

/// A filename's words with its parts set apart as a name sets them: every
/// dash between spaces ("–", "—", "--") a " - ", an underscore a space, and
/// spaces run together one.
fn parts_spaced(words: &str) -> String {
    words
        .replace('_', " ")
        .split_whitespace()
        .map(|token| {
            if !token.is_empty()
                && token
                    .chars()
                    .all(|character| matches!(character, '-' | '\u{2013}' | '\u{2014}'))
            {
                "-"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A filename's words without what a copy or a save adds: "Copy of" in
/// front, and "copy", "final", "v2" or "(2)" at the end.
fn without_copy_marks(words: &str) -> String {
    let mut tokens = words.split(' ').collect::<Vec<_>>();
    while tokens.len() >= 2
        && tokens[0].eq_ignore_ascii_case("copy")
        && tokens[1].eq_ignore_ascii_case("of")
    {
        tokens.drain(..2);
    }
    while let Some(last) = tokens.last() {
        let bare = last.trim_matches(|character: char| matches!(character, '(' | ')' | '-'));
        let lower = bare.to_lowercase();
        let version = lower
            .strip_prefix('v')
            .is_some_and(|figures| !figures.is_empty() && figures.bytes().all(|b| b.is_ascii_digit()));
        let counted = last.starts_with('(')
            && last.ends_with(')')
            && !bare.is_empty()
            && bare.bytes().all(|byte| byte.is_ascii_digit());
        if tokens.len() > 1 && (matches!(lower.as_str(), "copy" | "final" | "-") || version || counted)
        {
            tokens.pop();
        } else {
            break;
        }
    }
    tokens.join(" ")
}

/// Whether the key `spelling` holds the key `words` as a run of whole words.
fn holds_words(spelling: &str, words: &str) -> bool {
    !words.is_empty() && format!(" {spelling} ").contains(&format!(" {words} "))
}

/// Kinds of document a type is named by: two names of the same kind give
/// the same type ("Employment Termination Letter", "Termination Letter").
const DOCUMENT_KINDS: &[&str] = &[
    "agreement",
    "contract",
    "addendum",
    "amendment",
    "invoice",
    "bill",
    "receipt",
    "statement",
    "letter",
    "notice",
    "notification",
    "memo",
    "memorandum",
    "minutes",
    "report",
    "proposal",
    "quote",
    "quotation",
    "order",
    "certificate",
    "policy",
    "lease",
    "deed",
    "resolution",
    "application",
    "form",
    "return",
    "email",
    "correspondence",
    "communications",
    "offer",
    "claim",
    "complaint",
    "judgment",
    "motion",
    "plan",
    "schedule",
    "sow",
    "nda",
    "msa",
    "license",
    "licence",
    "permit",
    "waiver",
    "release",
    "consent",
    "confirmation",
    "summons",
    "subpoena",
    "petition",
    "bylaws",
    "charter",
    "budget",
    "presentation",
    "payslip",
    "timesheet",
];

/// Words that join a type's words and say nothing of its kind.
const TYPE_STOP_WORDS: &[&str] = &[
    "of", "and", "the", "a", "an", "for", "to", "with", "in", "on", "at", "by", "between", "from",
];

/// A type's or a filename's words, lower-cased, split at everything that is
/// not a letter or a digit: "Non-Disclosure Agreement" is "non",
/// "disclosure", "agreement".
fn type_words(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether a filename's `wording` gives the reading's `document_type`: it
/// holds all the type's words, names the same kind of document, or one
/// spells the other's initials ("SOW" and "Statement of Work", "NDA" and
/// "Non-Disclosure Agreement").
fn gives_type(wording: &str, document_type: &str) -> bool {
    let wording = type_words(wording);
    let typed = type_words(document_type);
    let significant = |words: &[String]| {
        words
            .iter()
            .filter(|word| !TYPE_STOP_WORDS.contains(&word.as_str()))
            .cloned()
            .collect::<Vec<_>>()
    };
    let (wording_words, type_significant) = (significant(&wording), significant(&typed));
    if type_significant.is_empty() {
        return true;
    }
    if type_significant
        .iter()
        .all(|word| wording_words.contains(word))
    {
        return true;
    }
    let kinds = |words: &[String]| {
        words
            .iter()
            .filter(|word| DOCUMENT_KINDS.contains(&word.as_str()))
            .cloned()
            .collect::<Vec<_>>()
    };
    let type_kinds = kinds(&type_significant);
    if kinds(&wording_words)
        .iter()
        .any(|kind| type_kinds.contains(kind))
    {
        return true;
    }
    let initials = |words: &[String]| {
        words
            .iter()
            .filter_map(|word| word.chars().next())
            .collect::<String>()
    };
    let spelled = |abbreviation: &str, words: &[String]| {
        abbreviation.chars().count() >= 2
            && (initials(words) == abbreviation || initials(&significant(words)) == abbreviation)
    };
    wording
        .iter()
        .any(|word| spelled(word, &typed))
        || typed.iter().any(|word| spelled(word, &wording))
}

/// The stem of a filename as an edit is compared: no extension, no leading
/// date, no collision suffix.
fn name_stem(filename: &str, extension: &str) -> String {
    let mut stem = filename.trim();
    if !extension.is_empty()
        && let Some(cut) = stem.len().checked_sub(extension.len() + 1)
        && stem.is_char_boundary(cut)
        && stem[cut..].starts_with('.')
        && stem[cut + 1..].eq_ignore_ascii_case(extension)
    {
        stem = &stem[..cut];
    }
    if let Some(date) = stem.get(..10)
        && crate::evidence::is_valid_iso_date(date)
        && stem[10..]
            .chars()
            .next()
            .is_none_or(|next| !next.is_alphanumeric())
    {
        stem = stem[10..].trim_start();
    }
    let trimmed = stem.trim_end();
    if let Some(open) = trimmed.rfind(" (")
        && trimmed.ends_with(')')
        && trimmed[open + 2..trimmed.len() - 1]
            .bytes()
            .all(|byte| byte.is_ascii_digit())
        && trimmed.len() > open + 3
    {
        return trimmed[..open].trim_end().to_owned();
    }
    trimmed.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DateRole, Evidence, ValidatedParty};
    use crate::naming::{MAX_FILENAME_CHARS, compose_filename};

    fn proposal(parties: &[&str], relation: PartyRelation) -> ValidatedProposal {
        ValidatedProposal {
            document_type: Some("Statement of Work".into()),
            document_date: Some("2026-04-01".into()),
            date_role: Some(DateRole::Effective),
            parties: parties.iter().map(|party| (*party).to_owned()).collect(),
            party_relation: relation,
            description:
                "Statement of work between Ridgeline Cartography LLC and Contoso Worldwide, Inc."
                    .into(),
            confidence: 0.9,
            evidence: Evidence {
                date: Some("effective as of April 1, 2026".into()),
                document_type: Some("STATEMENT OF WORK".into()),
                parties: vec![
                    "between Ridgeline Cartography LLC and Contoso Worldwide, Inc.".into(),
                ],
            },
        }
    }

    fn own(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn own_name_matching_is_key_and_prefix_based() {
        let firm = "Contoso Worldwide, Inc.";
        assert!(is_own_name("Contoso Worldwide, Inc.", firm));
        assert!(
            is_own_name("CONTOSO WORLDWIDE INC", firm),
            "case and punctuation"
        );
        assert!(is_own_name("  Contoso   Worldwide Inc ", firm), "spacing");
        assert!(
            is_own_name("Contoso", firm),
            "the document's shorter name for the firm"
        );
        assert!(
            is_own_name("Contoso Worldwide, Inc. (UK Branch)", firm),
            "a longer name that starts with the firm's"
        );
        assert!(
            !is_own_name("Contosoft Ltd", firm),
            "a prefix is whole words, never letters"
        );
        assert!(!is_own_name("Ridgeline Cartography LLC", firm));
        assert!(!is_own_name("Worldwide Contoso", firm));
        assert!(!is_own_name("", firm), "an empty party is no one");
        assert!(!is_own_name("Inc.", firm), "punctuation and a suffix");
    }

    #[test]
    fn short_own_keys_are_ignored() {
        // "IBM" is three characters: matching on it would have every "IBM
        // Global Services" and "ibm" in a party list treated as the firm.
        assert!(!is_own_name("IBM", "IBM"));
        assert!(!is_own_name("Co. Holdings", "Co."));
        assert!(!is_own_name("ABC Properties LLC", "A.B.C"));
        assert!(is_own_name("Acme", "Acme"), "four characters is enough");

        let view = counterparty_view(
            &proposal(
                &["IBM", "Ridgeline Cartography LLC"],
                PartyRelation::Between,
            ),
            &own(&["IBM", "  ", ""]),
        );
        assert_eq!(view.1, Vec::<String>::new());
        assert_eq!(view.0.parties, vec!["IBM", "Ridgeline Cartography LLC"]);
    }

    #[test]
    fn counterparty_view_drops_own_party_and_turns_between_into_with() {
        let sow = proposal(
            &["Ridgeline Cartography LLC", "Contoso Worldwide, Inc."],
            PartyRelation::Between,
        );
        let (view, dropped) = counterparty_view(&sow, &own(&["Contoso Worldwide, Inc."]));
        assert_eq!(view.parties, vec!["Ridgeline Cartography LLC"]);
        assert_eq!(view.party_relation, PartyRelation::With);
        assert_eq!(dropped, vec!["Contoso Worldwide, Inc."]);
        assert_eq!(
            compose_filename(&view, "pdf", &[]).value,
            "2026-04-01 Statement of Work with Ridgeline Cartography LLC.pdf"
        );
        assert_eq!(view.evidence, sow.evidence, "the evidence is untouched");
        assert_eq!(view.description, sow.description, "so is the description");

        // The firm named first, in capitals: the other side still leads.
        let (view, dropped) = counterparty_view(
            &proposal(
                &["CONTOSO WORLDWIDE INC", "Ridgeline Cartography LLC"],
                PartyRelation::Between,
            ),
            &own(&["Contoso Worldwide, Inc."]),
        );
        assert_eq!(view.parties, vec!["Ridgeline Cartography LLC"]);
        assert_eq!(dropped, vec!["CONTOSO WORLDWIDE INC"]);

        // Two others remain: still between them, in the document's order.
        let (view, _) = counterparty_view(
            &proposal(
                &["Northwind Traders", "Contoso", "Ridgeline Cartography LLC"],
                PartyRelation::Between,
            ),
            &own(&["Contoso Worldwide, Inc."]),
        );
        assert_eq!(
            view.parties,
            vec!["Northwind Traders", "Ridgeline Cartography LLC"]
        );
        assert_eq!(view.party_relation, PartyRelation::Between);
    }

    #[test]
    fn a_direction_that_described_the_firm_is_not_given_to_the_other_side() {
        let firm = own(&["Contoso Worldwide, Inc."]);
        // An invoice the firm issued is not an invoice from its customer.
        let issued = proposal(
            &["Contoso Worldwide, Inc.", "Ridgeline Cartography LLC"],
            PartyRelation::From,
        );
        let (view, dropped) = counterparty_view(&issued, &firm);
        assert_eq!(view.parties, vec!["Ridgeline Cartography LLC"]);
        assert_eq!(view.party_relation, PartyRelation::None);
        assert_eq!(dropped, vec!["Contoso Worldwide, Inc."]);
        assert_eq!(
            compose_filename(&view, "pdf", &[]).value,
            "2026-04-01 Statement of Work - Ridgeline Cartography LLC.pdf"
        );

        // One the firm received keeps its direction: it was about the sender.
        let received = proposal(
            &["Ridgeline Cartography LLC", "Contoso Worldwide, Inc."],
            PartyRelation::From,
        );
        let (view, _) = counterparty_view(&received, &firm);
        assert_eq!(view.party_relation, PartyRelation::From);
        assert_eq!(
            compose_filename(&view, "pdf", &[]).value,
            "2026-04-01 Statement of Work from Ridgeline Cartography LLC.pdf"
        );

        // "with" has no direction to misplace.
        let met = proposal(
            &["Contoso", "Ridgeline Cartography LLC"],
            PartyRelation::With,
        );
        assert_eq!(
            counterparty_view(&met, &firm).0.party_relation,
            PartyRelation::With
        );
    }

    #[test]
    fn only_party_is_never_dropped() {
        let firm = own(&["Contoso Worldwide, Inc."]);
        let alone = proposal(&["Contoso Worldwide, Inc."], PartyRelation::For);
        assert_eq!(
            counterparty_view(&alone, &firm),
            (alone.clone(), Vec::new())
        );

        // Two spellings of the firm and nobody else: nothing to file it by
        // but the firm, so it keeps the firm.
        let only_us = proposal(
            &["Contoso Worldwide, Inc.", "CONTOSO"],
            PartyRelation::Between,
        );
        assert_eq!(
            counterparty_view(&only_us, &firm),
            (only_us.clone(), Vec::new())
        );

        // And nothing configured changes nothing at all.
        let sow = proposal(
            &["Ridgeline Cartography LLC", "Contoso Worldwide, Inc."],
            PartyRelation::Between,
        );
        assert_eq!(counterparty_view(&sow, &[]), (sow.clone(), Vec::new()));
    }

    // The name view.

    fn reading(
        document_type: Option<&str>,
        date: Option<&str>,
        parties: &[&str],
        relation: PartyRelation,
    ) -> ValidatedProposal {
        ValidatedProposal {
            document_type: document_type.map(str::to_owned),
            document_date: date.map(str::to_owned),
            date_role: date.map(|_| DateRole::Effective),
            parties: parties.iter().map(|party| (*party).to_owned()).collect(),
            party_relation: relation,
            description: "A description.".into(),
            confidence: 0.9,
            evidence: Evidence::default(),
        }
    }

    fn found(parties: &[(&str, Option<PartyRole>)]) -> ValidatedFacts {
        ValidatedFacts {
            parties: parties
                .iter()
                .map(|(name, role)| ValidatedParty {
                    name: (*name).to_owned(),
                    role: *role,
                    proposed_role: *role,
                    role_support: Support::Cited,
                    document_role: None,
                    copied: false,
                    signatory: false,
                    support: Support::Cited,
                    evidence: Vec::new(),
                })
                .collect(),
            ..ValidatedFacts::default()
        }
    }

    fn named(
        proposal: &ValidatedProposal,
        facts: Option<&ValidatedFacts>,
        own: &[String],
        original: Option<&str>,
    ) -> String {
        NameView::of(proposal, &[], facts, own)
            .with_original(original)
            .compose("pdf", &[])
            .value
    }

    /// The product's three names, each composed from a validated reading
    /// with the organization's names in Settings. The firm is found among
    /// the reading's parties where validation kept only the other side for
    /// the name ("for" is about John Smith alone).
    #[test]
    fn the_three_names_lead_with_the_organization() {
        let firm = own(&["Vistage International", "Vistage Malaysia", "EGN Denmark"]);
        let sow = reading(
            Some("Host Chair SOW"),
            Some("2026-10-16"),
            &["John Smith"],
            PartyRelation::For,
        );
        let facts = found(&[
            ("Vistage International, Inc.", Some(PartyRole::Client)),
            ("John Smith", Some(PartyRole::Contractor)),
        ]);
        assert_eq!(
            named(&sow, Some(&facts), &firm, None),
            "2026-10-16 Vistage International - Host Chair SOW for John Smith.pdf"
        );

        let emails = reading(
            Some("Incorporation Email Communications"),
            Some("2024-11-09"),
            &["EGN Denmark ApS"],
            PartyRelation::None,
        );
        assert_eq!(
            named(&emails, None, &firm, None),
            "2024-11-09 EGN Denmark - Incorporation Email Communications.pdf"
        );

        let letter = reading(
            Some("Employment Termination Letter"),
            Some("2025-03-26"),
            &["Johnny Appleseed"],
            PartyRelation::For,
        );
        let facts = found(&[
            ("Vistage Malaysia Sdn. Bhd.", Some(PartyRole::Employer)),
            ("Johnny Appleseed", Some(PartyRole::Employee)),
        ]);
        assert_eq!(
            named(&letter, Some(&facts), &firm, None),
            "2025-03-26 Vistage Malaysia - Employment Termination Letter for Johnny Appleseed.pdf"
        );
    }

    /// The configured name that matches most specifically is the one the
    /// name gives; only a shorter one matching leaves the document's own,
    /// more specific name, without its legal form.
    #[test]
    fn the_most_specific_name_for_the_organization_leads() {
        let letter = reading(
            Some("Employment Termination Letter"),
            Some("2025-03-26"),
            &["Vistage Malaysia Sdn. Bhd.", "Johnny Appleseed"],
            PartyRelation::Between,
        );
        let leads = |names: &[&str]| {
            NameView::of(&letter, &[], None, &own(names))
                .entity
                .map(|entity| (entity.words, entity.kind))
        };
        assert_eq!(
            leads(&["Vistage", "Vistage Malaysia", "Vistage International"]),
            Some(("Vistage Malaysia".into(), EntityKind::Own))
        );
        assert_eq!(
            leads(&["Vistage"]),
            Some(("Vistage Malaysia".into(), EntityKind::Own)),
            "the document's party is more specific than the only name configured"
        );
        assert_eq!(
            leads(&["Vistage Malaysia Sdn Bhd"]),
            Some(("Vistage Malaysia".into(), EntityKind::Own)),
            "a configured name is written without its legal form too"
        );
        // A document that names the firm more shortly than either of two
        // configured names cannot say which: its own word is kept.
        let short = reading(
            Some("Letter"),
            Some("2025-03-26"),
            &["Vistage", "Johnny Appleseed"],
            PartyRelation::Between,
        );
        assert_eq!(
            NameView::of(
                &short,
                &[],
                None,
                &own(&["Vistage Malaysia", "Vistage International"])
            )
            .entity
            .map(|entity| entity.words),
            Some("Vistage".into())
        );
        // Of two of the firm's parties, the more specifically named leads,
        // and the other spelling of it is not given again.
        let both = reading(
            Some("Statement of Work"),
            Some("2026-04-01"),
            &["Contoso", "Contoso Worldwide, Inc.", "Ridgeline Cartography LLC"],
            PartyRelation::Between,
        );
        let view = NameView::of(&both, &[], None, &own(&["Contoso Worldwide, Inc."]));
        assert_eq!(
            view.compose("pdf", &[]).value,
            "2026-04-01 Contoso Worldwide - Statement of Work with Ridgeline Cartography LLC.pdf"
        );
        assert!(view.left_out.is_empty(), "the firm is in the name");
    }

    /// With no organization of the person's own, the first organization
    /// among the parties leads, without its legal form, and the subject
    /// does not give it again.
    #[test]
    fn with_no_own_name_the_first_organization_leads() {
        let invoice = reading(
            Some("Invoice"),
            Some("2025-04-30"),
            &["Nimbus Orchard Supply Co."],
            PartyRelation::From,
        );
        assert_eq!(
            named(&invoice, None, &[], None),
            "2025-04-30 Nimbus Orchard Supply - Invoice.pdf"
        );

        let sow = reading(
            Some("Statement of Work"),
            Some("2026-04-01"),
            &["Ridgeline Cartography LLC", "Contoso Worldwide, Inc."],
            PartyRelation::Between,
        );
        assert_eq!(
            named(&sow, None, &[], None),
            "2026-04-01 Ridgeline Cartography - Statement of Work with Contoso Worldwide, Inc.pdf"
        );

        // A person first: the organization after them leads.
        let employment = reading(
            Some("Employment Agreement"),
            Some("2024-04-12"),
            &["John Smith", "Acme Corporation"],
            PartyRelation::Between,
        );
        let view = NameView::of(&employment, &[], None, &[]);
        assert_eq!(
            view.compose("pdf", &[]).value,
            "2024-04-12 Acme - Employment Agreement with John Smith.pdf"
        );
        assert_eq!(view.entity.as_ref().unwrap().kind, EntityKind::Party);
        assert_eq!(
            view.filed().parties,
            vec!["Acme Corporation", "John Smith"],
            "the Party folder is the organization the name starts with"
        );

        // "for" is about John Smith: the employer that issued the notice
        // leads, where the reading found it.
        let notice = reading(
            Some("Notice of Termination"),
            Some("2026-12-29"),
            &["John Smith"],
            PartyRelation::For,
        );
        let facts = found(&[
            ("Northstar Lantern Works LLC", Some(PartyRole::Employer)),
            ("John Smith", Some(PartyRole::Employee)),
        ]);
        let view = NameView::of(&notice, &[], Some(&facts), &[]);
        assert_eq!(
            view.compose("pdf", &[]).value,
            "2026-12-29 Northstar Lantern Works - Notice of Termination for John Smith.pdf"
        );
        assert_eq!(view.entity.as_ref().unwrap().kind, EntityKind::Issuer);
        assert_eq!(
            view.filed().parties,
            vec!["Northstar Lantern Works LLC", "John Smith"]
        );

        // A letter "to" an organization from a person: the organization is
        // who it was sent to, not who it is for, and nobody leads.
        let letter = reading(
            Some("Letter"),
            Some("2026-01-05"),
            &["Acme Corporation"],
            PartyRelation::To,
        );
        let facts = found(&[
            ("Jane Doe", Some(PartyRole::Sender)),
            ("Acme Corporation", Some(PartyRole::Addressee)),
        ]);
        assert_eq!(
            named(&letter, Some(&facts), &[], None),
            "2026-01-05 Letter to Acme Corporation.pdf"
        );
    }

    /// A document that names only people keeps the engine's own name.
    #[test]
    fn a_document_naming_only_people_has_no_entity() {
        let notice = reading(
            Some("Notice of Termination"),
            Some("2026-12-29"),
            &["John Smith"],
            PartyRelation::None,
        );
        let view = NameView::of(&notice, &[], None, &[]);
        assert_eq!(view.entity, None);
        assert_eq!(
            view.compose("pdf", &[]).value,
            "2026-12-29 Notice of Termination - John Smith.pdf"
        );
        assert_eq!(
            view.compose("pdf", &[]).value,
            compose_filename(&notice, "pdf", &[]).value,
            "the engine's own name"
        );
        let between = reading(
            Some("Settlement Agreement"),
            Some("2026-01-05"),
            &["John Smith", "Jane Doe"],
            PartyRelation::Between,
        );
        assert_eq!(
            named(&between, None, &[], None),
            "2026-01-05 Settlement Agreement between John Smith and Jane Doe.pdf"
        );
    }

    #[test]
    fn a_legal_form_is_left_off_the_entity_only() {
        for (name, without) in [
            ("Vistage Malaysia Sdn. Bhd.", "Vistage Malaysia"),
            ("Vistage Malaysia Sdn Bhd", "Vistage Malaysia"),
            ("Contoso Worldwide, Inc.", "Contoso Worldwide"),
            ("Acme Corporation", "Acme"),
            ("Acme Corp.", "Acme"),
            ("Northwind Pte. Ltd.", "Northwind"),
            ("Kangaroo Holdings Pty Ltd", "Kangaroo Holdings"),
            ("Müller Werkzeug GmbH", "Müller Werkzeug"),
            ("Siemens AG", "Siemens"),
            ("Mærsk A/S", "Mærsk"),
            ("EGN Denmark ApS", "EGN Denmark"),
            ("Volvo AB", "Volvo"),
            ("Philips N.V.", "Philips"),
            ("Heineken B.V.", "Heineken"),
            ("Acme BV", "Acme"),
            ("Renault S.A.", "Renault"),
            ("Dupont SAS", "Dupont"),
            ("Dupont SARL", "Dupont"),
            ("John Deere & Co.", "John Deere"),
            ("Smith and Co", "Smith"),
            ("Barclays PLC", "Barclays"),
            ("KPMG LLP", "KPMG"),
            ("Ridgeline Cartography LLC", "Ridgeline Cartography"),
            ("Acme (Pty) Ltd", "Acme"),
            // Nothing to leave off, or nothing but a form.
            ("John Smith", "John Smith"),
            ("The Coca-Cola Company", "The Coca-Cola Company"),
            ("Ltd.", "Ltd."),
            ("Inc", "Inc"),
        ] {
            assert_eq!(without_legal_form(name), without, "{name}");
        }
        // The subject keeps a party's legal form: only the entity slot
        // leaves it off.
        let sow = reading(
            Some("Statement of Work"),
            Some("2026-04-01"),
            &["Ridgeline Cartography LLC", "Beacon Staffing LLC"],
            PartyRelation::Between,
        );
        assert_eq!(
            named(&sow, None, &[], None),
            "2026-04-01 Ridgeline Cartography - Statement of Work with Beacon Staffing LLC.pdf"
        );
    }

    #[test]
    fn organizations_are_told_from_people() {
        for organization in [
            "Nimbus Orchard Supply Co.",
            "Acme Corporation",
            "Vistage Malaysia Sdn. Bhd.",
            "Halden Bay National Bank",
            "Office for National Statistics",
            "Contoso Holdings",
        ] {
            assert!(is_organization(organization), "{organization}");
        }
        for person in ["John Smith", "Johnny Appleseed", "Jane A. Doe", "Acme"] {
            assert!(!is_organization(person), "{person}");
        }
    }

    /// A name over the limit sheds the subject's party before the entity,
    /// and keeps the entity while some of the type is left beside it.
    #[test]
    fn a_long_name_sheds_the_subject_before_the_entity() {
        let long = reading(
            Some("Master Professional Services and Technology Implementation Agreement"),
            Some("2026-04-01"),
            &[
                "Northstar Lantern Works Limited Liability Company Holdings",
                "Copper Wren Design Incorporated",
            ],
            PartyRelation::Between,
        );
        let facts = found(&[("Vistage International", Some(PartyRole::Client))]);
        let value = named(&long, Some(&facts), &own(&["Vistage International"]), None);
        assert!(value.chars().count() <= MAX_FILENAME_CHARS, "{value}");
        assert!(
            value.starts_with("2026-04-01 Vistage International - Master Professional"),
            "{value}"
        );
        assert!(!value.contains("Copper Wren"), "{value}");

        // A type too long for any party: cut beside the entity.
        let longer = reading(
            Some(&"Agreement ".repeat(20)),
            Some("2026-04-01"),
            &["Acme Corporation"],
            PartyRelation::From,
        );
        let value = named(&longer, None, &[], None);
        assert!(value.chars().count() <= MAX_FILENAME_CHARS, "{value}");
        assert!(value.starts_with("2026-04-01 Acme - Agreement"), "{value}");
        assert!(value.ends_with("Agreement.pdf"), "{value}");

        // An entity that leaves no room for the type goes too.
        let wide = reading(
            Some("Invoice"),
            Some("2026-04-01"),
            &[&format!("{} Holdings LLC", "Wide ".repeat(22))],
            PartyRelation::From,
        );
        let value = named(&wide, None, &[], None);
        assert!(value.chars().count() <= MAX_FILENAME_CHARS, "{value}");
        assert_eq!(value, "2026-04-01 Invoice.pdf");
    }

    #[test]
    fn the_entity_is_made_safe_for_a_filename() {
        let invoice = reading(
            Some("Invoice"),
            Some("2026-04-01"),
            &["ORION GLASS: STUDIO <EU> CORP"],
            PartyRelation::From,
        );
        assert_eq!(
            named(&invoice, None, &[], None),
            "2026-04-01 Orion Glass Studio EU - Invoice.pdf"
        );
        let accented = reading(
            Some("Invoice"),
            Some("2026-04-01"),
            &["Cafe\u{301} Rouge Holdings"],
            PartyRelation::From,
        );
        let value = named(&accented, None, &[], None);
        assert_eq!(value, "2026-04-01 Caf\u{e9} Rouge Holdings - Invoice.pdf");
        // And collides as any name does.
        let view = NameView::of(&accented, &[], None, &[]);
        let taken = ["2026-04-01 Cafe\u{301} Rouge Holdings - invoice.PDF"];
        let composed = view.compose("pdf", &taken);
        assert_eq!(composed.collision_index, 2);
        assert_eq!(
            composed.value,
            "2026-04-01 Caf\u{e9} Rouge Holdings - Invoice (2).pdf"
        );
    }

    /// A house-style spelling of the party the entity stands for is the
    /// reviewer's: carried as typed, legal form and all, and still known
    /// for the organization the document named.
    #[test]
    fn a_reviewers_spelling_of_the_entity_is_carried_as_typed() {
        let rule = HouseRule::new(RuleKind::Party, "Acme Corporation", "ACME Corp");
        let styled = reading(
            Some("Employment Agreement"),
            Some("2024-04-12"),
            &["John Smith", "ACME Corp"],
            PartyRelation::Between,
        );
        let view = NameView::of(&styled, std::slice::from_ref(&rule), None, &[]);
        assert_eq!(
            view.compose("pdf", &[]).value,
            "2024-04-12 ACME Corp - Employment Agreement with John Smith.pdf"
        );
        // Respelled to a word that says nothing of a company, it is still
        // the organization the document named.
        let rule = HouseRule::new(RuleKind::Party, "Acme Corporation", "Acme");
        let styled = reading(
            Some("Employment Agreement"),
            Some("2024-04-12"),
            &["John Smith", "Acme"],
            PartyRelation::Between,
        );
        let view = NameView::of(&styled, std::slice::from_ref(&rule), None, &[]);
        assert_eq!(
            view.compose("pdf", &[]).value,
            "2024-04-12 Acme - Employment Agreement with John Smith.pdf"
        );
    }

    #[test]
    fn an_edit_is_read_around_the_entity() {
        let employment = reading(
            Some("Employment Agreement"),
            Some("2024-04-12"),
            &["John Smith", "Acme Corporation"],
            PartyRelation::Between,
        );
        let view = NameView::of(&employment, &[], None, &[]);
        let proposed = view.compose("pdf", &[]).value;
        assert_eq!(
            view.read_edit(
                "pdf",
                &proposed,
                "2024-05-01 Acme - Employment Agreement with Jon Smith.pdf"
            ),
            EditedName::Subject {
                grammar: &view.subject,
                proposed: "2024-04-12 Employment Agreement with John Smith.pdf".into(),
                approved: "2024-05-01 Employment Agreement with Jon Smith.pdf".into(),
            }
        );
        assert_eq!(
            view.read_edit(
                "pdf",
                &proposed,
                "2024-04-12 Acme Corp - Employment Agreement with John Smith (2).pdf"
            ),
            EditedName::Entity {
                words: "Acme Corp".into()
            }
        );
        assert_eq!(
            view.read_edit(
                "pdf",
                &proposed,
                "2024-04-12 Acme Corp - Employment Contract with John Smith.pdf"
            ),
            EditedName::Rewritten
        );
        // A name an earlier build proposed is read as it composed it.
        let earlier = "2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf";
        assert!(matches!(
            view.read_edit("pdf", earlier, "2024-04-12 Employment Agreement between John Smith and Acme.pdf"),
            EditedName::Subject { grammar, .. } if grammar.parties == employment.parties
        ));
        assert_eq!(
            view.without_entity(&proposed).as_deref(),
            Some("2024-04-12 Employment Agreement with John Smith.pdf")
        );
        assert_eq!(view.without_entity(earlier), None);
    }

    #[test]
    fn dates_are_read_from_a_filename_only_one_way() {
        use crate::naming::date_in_filename;
        for (stem, date) in [
            ("2025-03-26 Vistage Malaysia - Letter", "2025-03-26"),
            ("Letter 2025_03_26", "2025-03-26"),
            ("Letter 20250326", "2025-03-26"),
            ("26.03.2025 Letter", "2025-03-26"),
            ("Letter 03-26-2025", "2025-03-26"),
            ("Letter March 26 2025", "2025-03-26"),
            ("Letter March 26, 2025", "2025-03-26"),
            ("Letter 26 Mar 2025", "2025-03-26"),
            ("Letter 26th March 2025", "2025-03-26"),
            ("2025 March 26 Letter", "2025-03-26"),
            ("Letter 26-Mar-2025", "2025-03-26"),
            ("Letter 03.03.2025", "2025-03-03"),
            ("2025-03-26 Letter (signed 2025-03-26)", "2025-03-26"),
        ] {
            assert_eq!(date_in_filename(stem).as_deref(), Some(date), "{stem}");
        }
        for stem in [
            "Letter",
            "Letter 03.04.2025",
            "Letter 26.03.25",
            "Letter 2025-03-26 and 2025-04-01",
            "Letter 2025-02-30",
            "Invoice 12345678",
            "IMG_1234",
            "Letter March 2025",
            "Letter 1850-01-01",
        ] {
            assert_eq!(date_in_filename(stem), None, "{stem}");
        }
    }

    #[test]
    fn junk_filenames_say_nothing() {
        for junk in [
            "scan0001",
            "IMG_1234",
            "Document1",
            "untitled",
            "Doc (3)",
            "3f9a8b7c6d5e4f3a2b1c",
            "550e8400-e29b-41d4-a716-446655440000",
            "final_v2",
            "Copy of final",
            "DSC_0042 copy",
            "2025-03-26",
            "PXL_20250326_101010",
        ] {
            assert!(crate::naming::is_junk_filename(junk), "{junk}");
        }
        for meaningful in [
            "Host Chair SOW",
            "Vistage Malaysia - Termination Letter",
            "Copy of Lease",
        ] {
            assert!(!crate::naming::is_junk_filename(meaningful), "{meaningful}");
        }
    }

    fn letter_view(original: Option<&str>) -> String {
        let letter = reading(
            Some("Termination Letter"),
            Some("2025-03-26"),
            &["Johnny Appleseed"],
            PartyRelation::For,
        );
        let facts = found(&[
            ("Vistage Malaysia Sdn. Bhd.", Some(PartyRole::Employer)),
            ("Johnny Appleseed", Some(PartyRole::Employee)),
        ]);
        named(&letter, Some(&facts), &own(&["Vistage Malaysia"]), original)
    }

    /// A filename already named the way Intern names documents, with the
    /// reading's date, is proposed as it is.
    #[test]
    fn an_original_already_in_shape_is_kept() {
        let original = "2025-03-26 Vistage Malaysia - Employment Termination Letter for Johnny Appleseed";
        assert_eq!(letter_view(Some(original)), format!("{original}.pdf"));
        // Without it, the reading's own words.
        assert_eq!(
            letter_view(None),
            "2025-03-26 Vistage Malaysia - Termination Letter for Johnny Appleseed.pdf"
        );
    }

    /// Its wording kept and the date put first, from wherever it stood and
    /// however it was written.
    #[test]
    fn an_original_without_a_leading_iso_date_keeps_its_wording() {
        let expected =
            "2025-03-26 Vistage Malaysia - Employment Termination Letter for Johnny Appleseed.pdf";
        for original in [
            "Vistage Malaysia - Employment Termination Letter for Johnny Appleseed",
            "26.03.2025 Vistage Malaysia - Employment Termination Letter for Johnny Appleseed",
            "Vistage Malaysia - Employment Termination Letter for Johnny Appleseed 20250326",
            "Vistage Malaysia - Employment Termination Letter for Johnny Appleseed - March 26 2025",
            "Vistage Malaysia Sdn Bhd - Employment Termination Letter - Johnny Appleseed (26 Mar 2025)",
            "Vistage_Malaysia_-_Employment_Termination_Letter_for_Johnny_Appleseed",
            "Copy of Vistage Malaysia - Employment Termination Letter for Johnny Appleseed final",
        ] {
            assert_eq!(letter_view(Some(original)), expected, "{original}");
        }
        // The filename's date dates a reading that found none.
        let undated = reading(
            Some("Termination Letter"),
            None,
            &["Johnny Appleseed"],
            PartyRelation::For,
        );
        let facts = found(&[("Vistage Malaysia Sdn. Bhd.", Some(PartyRole::Employer))]);
        assert_eq!(
            named(
                &undated,
                Some(&facts),
                &own(&["Vistage Malaysia"]),
                Some("Vistage Malaysia - Employment Termination Letter for Johnny Appleseed 26.03.2025")
            ),
            expected
        );
        assert_eq!(
            named(&undated, Some(&facts), &own(&["Vistage Malaysia"]), None),
            "Vistage Malaysia - Termination Letter for Johnny Appleseed.pdf"
        );
    }

    /// The parts of a filename put in order: its words for the type, the
    /// reading's entity, connecting word and party.
    #[test]
    fn an_original_in_another_order_gives_its_type_words() {
        let sow = reading(
            Some("Statement of Work"),
            Some("2026-10-16"),
            &["John Smith"],
            PartyRelation::For,
        );
        let facts = found(&[
            ("Vistage International, Inc.", Some(PartyRole::Client)),
            ("John Smith", Some(PartyRole::Contractor)),
        ]);
        let firm = own(&["Vistage International"]);
        let expected = "2026-10-16 Vistage International - Host Chair SOW for John Smith.pdf";
        for original in [
            "Host Chair SOW - John Smith - Vistage International",
            "Host Chair SOW with John Smith - Vistage International Inc",
            "Vistage International Host Chair SOW John Smith",
            "2026-10-16 Host Chair SOW - Vistage International - John Smith",
        ] {
            assert_eq!(
                named(&sow, Some(&facts), &firm, Some(original)),
                expected,
                "{original}"
            );
        }
        // With no organization: an invoice's own words for itself.
        let invoice = reading(
            Some("Invoice"),
            Some("2025-04-30"),
            &["Nimbus Orchard Supply Co."],
            PartyRelation::From,
        );
        assert_eq!(
            named(
                &invoice,
                None,
                &[],
                Some("Nimbus Orchard Supply - April Invoice INV-3301")
            ),
            "2025-04-30 Nimbus Orchard Supply - April Invoice INV-3301.pdf"
        );
    }

    /// A filename that contradicts the reading, or says nothing, is not
    /// read at all.
    #[test]
    fn an_original_that_contradicts_the_reading_is_ignored() {
        let composed = letter_view(None);
        for original in [
            // Junk.
            "scan0001",
            "IMG_1234",
            "Document1",
            "untitled",
            "Doc (3)",
            "a3f9b2c1e8d7f6a54b3c2d1e",
            "final_v2",
            "Copy of Document1",
            // Another date.
            "2025-03-27 Vistage Malaysia - Employment Termination Letter for Johnny Appleseed",
            "Vistage Malaysia - Employment Termination Letter for Johnny Appleseed 03.04.2025",
            // Another party, or none.
            "2025-03-26 Vistage Malaysia - Employment Termination Letter for Jane Doe",
            "2025-03-26 Vistage Malaysia - Employment Termination Letter",
            // Another organization, or none.
            "2025-03-26 Contoso - Employment Termination Letter for Johnny Appleseed",
            "Employment Termination Letter for Johnny Appleseed",
            // Another kind of document.
            "2025-03-26 Vistage Malaysia - Invoice for Johnny Appleseed",
            "2025-03-26 Vistage Malaysia - Offer Letter Draft for Johnny Appleseed - Payslip",
            // A party inside the type's words.
            "Vistage Malaysia - Letter to Johnny Appleseed re Vistage Malaysia Policy",
        ] {
            assert_eq!(letter_view(Some(original)), composed, "{original}");
        }
    }

    /// A type a reviewer's spelling wrote stays theirs, and a filename
    /// that names only the type of a reading with nothing else to check it
    /// by is not taken.
    #[test]
    fn an_original_does_not_override_a_spelling_or_go_unchecked() {
        let rule = HouseRule::new(RuleKind::DocumentType, "Termination Letter", "Separation Letter");
        let styled = reading(
            Some("Separation Letter"),
            Some("2025-03-26"),
            &["Johnny Appleseed"],
            PartyRelation::For,
        );
        let view = NameView::of(&styled, std::slice::from_ref(&rule), None, &[])
            .with_original(Some("Johnny Appleseed - Employment Termination Letter"));
        assert_eq!(view.original_type, None);
        assert_eq!(
            view.compose("pdf", &[]).value,
            "2025-03-26 Separation Letter for Johnny Appleseed.pdf"
        );
        // A person-only reading with the filename's words for its type.
        let notice = reading(
            Some("Notice of Termination"),
            Some("2026-12-29"),
            &["John Smith"],
            PartyRelation::None,
        );
        assert_eq!(
            named(&notice, None, &[], Some("Termination Notice - John Smith")),
            "2026-12-29 Termination Notice - John Smith.pdf"
        );
        // No type, no party, no entity: nothing checks the words.
        let bare = reading(None, Some("2026-12-29"), &[], PartyRelation::None);
        assert_eq!(
            named(&bare, None, &[], Some("Board Minutes")),
            "2026-12-29 Document.pdf"
        );
        let minutes = reading(Some("Meeting Minutes"), Some("2025-05-07"), &[], PartyRelation::None);
        assert_eq!(
            named(&minutes, None, &[], Some("Board Minutes 2025-05-07")),
            "2025-05-07 Board Minutes.pdf"
        );
    }

    #[test]
    fn a_filename_gives_a_type_by_its_words_kind_or_initials() {
        assert!(gives_type("Host Chair SOW", "Statement of Work"));
        assert!(gives_type("Mutual NDA", "Non-Disclosure Agreement"));
        assert!(gives_type("Statement of Work", "SOW"));
        assert!(gives_type("Employment Termination Letter", "Termination Letter"));
        assert!(gives_type("Incorporation Email Communications", "Email"));
        assert!(gives_type("April Invoice INV-3301", "Invoice"));
        assert!(!gives_type("Invoice", "Lease Agreement"));
        assert!(!gives_type("Employment Termination Letter", "Notice of Termination"));
        assert!(!gives_type("Holiday Photos", "Statement of Work"));
    }
}
