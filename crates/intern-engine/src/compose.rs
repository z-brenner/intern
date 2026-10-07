//! Deterministic composition: how a document's parties read in its filename,
//! and its one-sentence description, built from validated facts.
//!
//! The evidence pipeline asks the model for facts only - what the document
//! is, its parties and their roles, its subject, its number, a key fact or
//! two - and everything stylistic is decided here, by rules a test can pin:
//!
//! * [`DocumentClass::of`] sorts a validated type into a class.
//! * [`relation_from_roles`] picks the filename's parties and the word that
//!   joins them from the class and the roles validation found support for.
//!   A role nothing in the document supports never decides it.
//! * [`describe`] fills a template for the class, dropping optional parts
//!   in a fixed order until the sentence fits.
//!
//! Everything here is pure. House style and the firm's own names apply
//! afterwards, to the filename and the folder only, exactly as they do for
//! the digest pipeline; the description keeps the document's own spellings.

use crate::domain::{DocumentClass, PartyRelation, PartyRole};
use crate::validate::MAX_DESCRIPTION_WORDS;

/// The fewest words a description may have before the date is added to it.
const MIN_DESCRIPTION_WORDS: usize = 6;
/// A subject shortened to fit keeps at most this many words.
const SHORT_SUBJECT_WORDS: usize = 6;

impl DocumentClass {
    /// The class of a validated document type. Families whose names contain
    /// other families' words are tested first: an addendum is an amendment
    /// before it is an agreement, a statement of work an agreement and not
    /// a statement.
    pub fn of(document_type: Option<&str>) -> Self {
        let Some(document_type) = document_type else {
            return Self::Unknown;
        };
        let lowered = document_type.to_lowercase();
        let has = |words: &[&str]| words.iter().any(|word| contains_word(&lowered, word));
        if has(&["amendment", "addendum", "modification", "change order"]) {
            Self::Amendment
        } else if has(&["notice", "notification", "claim"]) {
            Self::Notice
        } else if has(&["declarations", "explanation of benefits"]) {
            Self::Record
        } else if !has(&["statement of work"])
            && (lowered.trim() == "statement"
                || has(&[
                    "invoice",
                    "receipt",
                    "bill",
                    "bill of lading",
                    "statement of account",
                    "account statement",
                    "bank statement",
                    "billing statement",
                    "quote",
                    "quotation",
                    "estimate",
                    "purchase order",
                    "work order",
                    "sales order",
                    "packing slip",
                    "delivery",
                    "credit note",
                    "remittance",
                    "rate confirmation",
                    "price list",
                ]))
        {
            Self::Issued
        } else if has(&[
            "agreement",
            "contract",
            "lease",
            "statement of work",
            "terms",
            "license",
            "licence",
            "deed",
            "promissory note",
            "release",
            "waiver",
        ]) {
            Self::Agreement
        } else if has(&["email", "e-mail"]) {
            Self::Email
        } else if has(&["letter", "memo", "memorandum"]) {
            Self::Letter
        } else if has(&[
            "form",
            "application",
            "registration",
            "intake",
            "questionnaire",
        ]) {
            Self::Form
        } else if has(&[
            "minutes",
            "report",
            "log",
            "register",
            "journal",
            "review",
            "plan",
            "presentation",
            "newsletter",
            "resolution",
            "consent",
            "certificate",
            "record",
            "summary",
            "agenda",
            "statement",
            // An insurance policy is issued to the insured, who it is for.
            "policy",
        ]) {
            Self::Record
        } else {
            Self::Unknown
        }
    }
}

/// Whether `phrase` stands in `text` as whole words.
fn contains_word(text: &str, phrase: &str) -> bool {
    text.match_indices(phrase).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + phrase.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// A validated party as composition sees it, in the order the document
/// first names the parties.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CastMember {
    pub name: String,
    /// The role the reply gave.
    pub role: Option<PartyRole>,
    /// Whether validation found support for that role in the document.
    pub role_supported: bool,
}

impl CastMember {
    /// The role, when the document supports it.
    fn supported_role(&self) -> Option<PartyRole> {
        self.role.filter(|_| self.role_supported)
    }
}

/// The parties a filename names, the word joining them, and why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    pub relation: PartyRelation,
    pub parties: Vec<String>,
    /// The rule that applied, for a reviewer and for tuning.
    pub basis: String,
}

/// The side a document is "about": the client, customer, tenant, employee,
/// borrower, licensee or buyer.
const SUBSTANTIVE: &[PartyRole] = &[
    PartyRole::Client,
    PartyRole::Customer,
    PartyRole::Tenant,
    PartyRole::Employee,
    PartyRole::Borrower,
    PartyRole::Licensee,
    PartyRole::Buyer,
];

/// The side a document comes from: whoever provides, lends, employs,
/// sells, issues or sends.
const PROVIDER: &[PartyRole] = &[
    PartyRole::Landlord,
    PartyRole::Lender,
    PartyRole::Employer,
    PartyRole::Seller,
    PartyRole::Contractor,
    PartyRole::Licensor,
    PartyRole::Vendor,
    PartyRole::Issuer,
    PartyRole::Sender,
];

/// Roles that name the side a document is sent or billed to.
const RECEIVING: &[PartyRole] = &[
    PartyRole::Customer,
    PartyRole::Recipient,
    PartyRole::Client,
    PartyRole::Buyer,
    PartyRole::Addressee,
    PartyRole::Tenant,
    PartyRole::Borrower,
    PartyRole::Licensee,
    PartyRole::Employee,
];

/// Roles that come in pairs, one on each side of a bilateral document.
const BILATERAL: &[(PartyRole, PartyRole)] = &[
    (PartyRole::Client, PartyRole::Contractor),
    (PartyRole::Employer, PartyRole::Employee),
    (PartyRole::Buyer, PartyRole::Seller),
    (PartyRole::Landlord, PartyRole::Tenant),
    (PartyRole::Borrower, PartyRole::Lender),
    (PartyRole::Licensor, PartyRole::Licensee),
    (PartyRole::Vendor, PartyRole::Customer),
];

/// What the document's own layout and wording say about its parties,
/// besides their roles.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationCues {
    /// The party an issued document's cues nominate as its issuer: a
    /// "Remit To" line with its name, a "Bill To" line with the other's (the
    /// digest pipeline's repair), or the one party at the head of its first
    /// page, before any line that names the customer. Used only when no
    /// supported role settles it.
    pub issuer: Option<usize>,
    /// Whether the document names its first two parties as its two sides:
    /// "by and between A and B".
    pub between: bool,
}

/// The filename's parties and joining word, from the document's class and
/// the roles validation supported.
///
/// `parties` are the validated parties in the order the document first
/// names them; `cues` what the document's layout and wording say besides
/// (see [`RelationCues`]).
///
/// Only a supported role counts. When nothing settles the question the
/// first party is kept with no joining word ([`PartyRelation::None`]): the
/// names are the document's, and the relation is not guessed at.
pub fn relation_from_roles(
    class: DocumentClass,
    document_type: Option<&str>,
    parties: &[CastMember],
    cues: RelationCues,
) -> Relation {
    let Some(first) = parties.first() else {
        return Relation {
            relation: PartyRelation::None,
            parties: Vec::new(),
            basis: "no parties".into(),
        };
    };
    let with = |roles: &[PartyRole]| {
        parties.iter().find(|party| {
            party
                .supported_role()
                .is_some_and(|role| roles.contains(&role))
        })
    };
    let one = |relation: PartyRelation, party: &CastMember, basis: &str| Relation {
        relation,
        parties: vec![party.name.clone()],
        basis: format!("{}: {basis}", class.as_str()),
    };
    // A document that comes from a provider is about the party nothing
    // else is said of: a landlord's notice about its tenant, a lender's
    // about its borrower. `with_other` lets an explicit "other" stand as
    // that party too.
    let counterpart = |with_other: bool| {
        let provider = parties.iter().any(|party| {
            party
                .supported_role()
                .is_some_and(|role| PROVIDER.contains(&role))
        });
        if !provider {
            return None;
        }
        parties.iter().find(|party| match party.supported_role() {
            None => true,
            Some(PartyRole::Other) => with_other,
            Some(_) => false,
        })
    };
    let unresolved = || Relation {
        relation: PartyRelation::None,
        parties: vec![first.name.clone()],
        basis: format!("{}: unresolved", class.as_str()),
    };
    let lowered = document_type.unwrap_or_default().to_lowercase();
    // The two sides the document itself names: "by and between A and B".
    let sides = || match parties {
        [a, b, ..] if cues.between => Some(Relation {
            relation: PartyRelation::Between,
            parties: vec![a.name.clone(), b.name.clone()],
            basis: format!("{}: by and between", class.as_str()),
        }),
        _ => None,
    };
    match class {
        DocumentClass::Agreement | DocumentClass::Amendment => match parties {
            [a, b, ..] => Relation {
                relation: PartyRelation::Between,
                parties: vec![a.name.clone(), b.name.clone()],
                basis: format!("{}: the first two parties", class.as_str()),
            },
            [only] => one(PartyRelation::With, only, "one party"),
            [] => unreachable!("an empty cast returned above"),
        },
        DocumentClass::Issued => {
            if let Some(issuer) = with(&[PartyRole::Issuer]) {
                return one(PartyRelation::From, issuer, "the issuer");
            }
            if contains_word(&lowered, "purchase order")
                && let Some(buyer) = with(&[PartyRole::Buyer])
            {
                return one(PartyRelation::From, buyer, "the buyer issues an order");
            }
            if let Some(seller) = with(&[PartyRole::Vendor, PartyRole::Seller, PartyRole::Sender]) {
                return one(PartyRelation::From, seller, "the vendor");
            }
            if let Some(index) = cues.issuer
                && let Some(issuer) = parties.get(index)
            {
                return one(PartyRelation::From, issuer, "the issuer's cues");
            }
            let receiving = |party: &CastMember| {
                party
                    .supported_role()
                    .is_some_and(|role| RECEIVING.contains(&role))
            };
            if let [a, b, ..] = parties {
                match (receiving(a), receiving(b)) {
                    (true, false) => {
                        return one(PartyRelation::From, b, "the other side of the customer");
                    }
                    (false, true) => {
                        return one(PartyRelation::From, a, "the other side of the customer");
                    }
                    _ => {}
                }
            }
            // The customer of an issued document is never its filename's
            // party: "Invoice - Quillon Ridge Bakery" reads as Quillon's
            // invoice. With no issuer to name, none is named.
            match parties.iter().find(|party| !receiving(party)) {
                Some(party) => Relation {
                    relation: PartyRelation::None,
                    parties: vec![party.name.clone()],
                    basis: "issued: unresolved".into(),
                },
                None => Relation {
                    relation: PartyRelation::None,
                    parties: Vec::new(),
                    basis: "issued: only the customer is named".into(),
                },
            }
        }
        DocumentClass::Notice => {
            if let Some(party) = with(SUBSTANTIVE) {
                return one(PartyRelation::For, party, "the party it is about");
            }
            if let Some(party) = with(&[PartyRole::Recipient, PartyRole::Addressee]) {
                return one(PartyRelation::To, party, "the recipient");
            }
            if let Some(party) = with(&[PartyRole::Issuer, PartyRole::Sender]) {
                return one(PartyRelation::From, party, "the issuer");
            }
            if let Some(party) = counterpart(false) {
                return one(PartyRelation::For, party, "the provider's counterpart");
            }
            if let Some(party) = with(PROVIDER) {
                return one(PartyRelation::From, party, "the provider");
            }
            if let [a, b, ..] = parties
                && a.supported_role() == Some(PartyRole::Other)
                && b.supported_role() == Some(PartyRole::Other)
            {
                return Relation {
                    relation: PartyRelation::Between,
                    parties: vec![a.name.clone(), b.name.clone()],
                    basis: "notice: two sides".into(),
                };
            }
            unresolved()
        }
        DocumentClass::Letter => {
            if let Some(party) = with(&[PartyRole::Addressee, PartyRole::Recipient]) {
                return one(PartyRelation::To, party, "the addressee");
            }
            if let Some(party) = with(&[PartyRole::Sender, PartyRole::Issuer]) {
                return one(PartyRelation::From, party, "the sender");
            }
            if let Some(party) = with(SUBSTANTIVE) {
                return one(PartyRelation::For, party, "the party it is about");
            }
            if let Some(party) = counterpart(false) {
                return one(PartyRelation::For, party, "the provider's counterpart");
            }
            if let Some(party) = with(PROVIDER) {
                return one(PartyRelation::From, party, "the provider");
            }
            unresolved()
        }
        DocumentClass::Email => {
            if let Some(party) = with(&[PartyRole::Sender]) {
                return one(PartyRelation::From, party, "the sender");
            }
            if let Some(party) = with(&[PartyRole::Addressee, PartyRole::Recipient]) {
                return one(PartyRelation::To, party, "the addressee");
            }
            if let Some(party) = with(&[PartyRole::Issuer]) {
                return one(PartyRelation::From, party, "the issuer");
            }
            unresolved()
        }
        DocumentClass::Record => {
            if parties.len() == 1 {
                return if first
                    .supported_role()
                    .is_some_and(|role| SUBSTANTIVE.contains(&role))
                {
                    one(PartyRelation::For, first, "the party it is about")
                } else {
                    one(PartyRelation::From, first, "one party")
                };
            }
            if let Some(party) = with(SUBSTANTIVE) {
                return one(PartyRelation::For, party, "the party it is about");
            }
            if let Some(party) = with(&[PartyRole::Recipient, PartyRole::Addressee]) {
                return one(PartyRelation::To, party, "the recipient");
            }
            if let Some(party) = counterpart(true) {
                return one(PartyRelation::For, party, "the provider's counterpart");
            }
            if let Some(party) = with(&[PartyRole::Issuer, PartyRole::Sender]) {
                return one(PartyRelation::From, party, "the issuer");
            }
            unresolved()
        }
        DocumentClass::Form => {
            if let Some(relation) = sides() {
                return relation;
            }
            if let Some(party) = with(&[PartyRole::Issuer, PartyRole::Sender]).or_else(|| {
                with(&[
                    PartyRole::Employee,
                    PartyRole::Tenant,
                    PartyRole::Vendor,
                    PartyRole::Customer,
                ])
            }) {
                return one(PartyRelation::For, party, "the filer");
            }
            if let Some(party) = counterpart(false) {
                return one(PartyRelation::For, party, "the provider's counterpart");
            }
            unresolved()
        }
        DocumentClass::Unknown => {
            for (a, b) in BILATERAL {
                if let (Some(left), Some(right)) = (with(&[*a]), with(&[*b])) {
                    return Relation {
                        relation: PartyRelation::Between,
                        parties: vec![left.name.clone(), right.name.clone()],
                        basis: "unknown: a bilateral pair".into(),
                    };
                }
            }
            if let Some(relation) = sides() {
                return relation;
            }
            if let Some(party) = with(&[PartyRole::Issuer]) {
                return one(PartyRelation::From, party, "the issuer");
            }
            if let Some(party) = with(&[PartyRole::Addressee]) {
                return one(PartyRelation::To, party, "the addressee");
            }
            unresolved()
        }
    }
}

/// What a description is composed from: validated facts only, in the
/// document's own spellings.
#[derive(Clone, Debug, Default)]
pub struct DescriptionFacts<'a> {
    pub class: DocumentClass,
    pub document_type: Option<&'a str>,
    pub relation: Option<&'a Relation>,
    /// Every validated party, in first-appearance order.
    pub parties: &'a [CastMember],
    pub subject: Option<&'a str>,
    /// The identifier and the word its label gives it: `("invoice",
    /// "INV-10438")`.
    pub identifier: Option<(&'a str, &'a str)>,
    /// Whether the identifier stands on the title line with the type -
    /// "PACKING SLIP PS-311" - and reads after it: "Packing Slip PS-311".
    pub identifier_in_title: bool,
    /// An amount worth stating, and the label the document gives it:
    /// `(Some("annual fee"), "$96,000")`.
    pub amount: Option<(Option<&'a str>, &'a str)>,
    /// A key fact that is not an amount; it stands in for an absent subject.
    pub other_fact: Option<&'a str>,
    /// The date as the document writes it, for a description too short
    /// without it.
    pub date_surface: Option<&'a str>,
}

/// One optional part of a description, in the order parts are dropped when
/// the sentence is too long.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum Part {
    PreparedBy,
    Identifier,
    Amount,
    Subject,
    SecondParty,
}

/// The one-sentence description of a document, from its validated facts.
///
/// A template per class, filled only with what validation accepted. A
/// sentence over [`MAX_DESCRIPTION_WORDS`] sheds its optional parts in a
/// fixed order - the preparer, the identifier, the subject's tail, the
/// amount, the subject, the second party - and is never cut mid-phrase. One
/// under six words gets the document's date, as the document writes it;
/// if it is still too short, validation says so, as it would of a sentence
/// the model wrote.
pub fn describe(facts: &DescriptionFacts<'_>) -> String {
    let mut dropped: Vec<Part> = Vec::new();
    let mut short_subject = false;
    loop {
        let sentence = compose(facts, &dropped, short_subject, false);
        if word_count(&sentence) <= MAX_DESCRIPTION_WORDS {
            if word_count(&sentence) < MIN_DESCRIPTION_WORDS && facts.date_surface.is_some() {
                return compose(facts, &dropped, short_subject, true);
            }
            return sentence;
        }
        // The fixed drop order: the preparer, the identifier, the subject's
        // tail, the amount, the whole subject, the second party.
        if !dropped.contains(&Part::PreparedBy) {
            dropped.push(Part::PreparedBy);
        } else if !dropped.contains(&Part::Identifier) {
            dropped.push(Part::Identifier);
        } else if !short_subject {
            short_subject = true;
        } else if !dropped.contains(&Part::Amount) {
            dropped.push(Part::Amount);
        } else if !dropped.contains(&Part::Subject) {
            dropped.push(Part::Subject);
        } else if !dropped.contains(&Part::SecondParty) {
            dropped.push(Part::SecondParty);
        } else {
            return sentence;
        }
    }
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Fills the class's template, leaving out `dropped` parts.
fn compose(
    facts: &DescriptionFacts<'_>,
    dropped: &[Part],
    short_subject: bool,
    dated: bool,
) -> String {
    let keep = |part: Part| !dropped.contains(&part);
    let kind = facts
        .document_type
        .map(clean)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Document".to_owned());
    let subject = facts
        .subject
        .map(subject_phrase)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            facts
                .other_fact
                .map(subject_phrase)
                .filter(|value| !value.is_empty())
        })
        .map(|value| {
            if short_subject {
                shorten(&value, SHORT_SUBJECT_WORDS)
            } else {
                value
            }
        })
        .filter(|_| keep(Part::Subject));
    // An amount with no label reads beside the subject it is the price of,
    // or as what the document is for; a labelled one or a total at the end.
    let amount = facts.amount.filter(|_| keep(Part::Amount));
    let beside_subject = amount.is_some_and(|(label, _)| {
        label.is_none() && facts.class != DocumentClass::Issued && subject.is_some()
    });
    let subject = match (subject, amount) {
        (Some(subject), Some((_, money))) if beside_subject => Some(format!("{subject} ({money})")),
        (subject, _) => subject,
    };
    let relation = facts.relation;
    let named = relation
        .map(|relation| relation.parties.as_slice())
        .unwrap_or_default();
    let joining = relation.map_or(PartyRelation::None, |relation| relation.relation);
    let first = named.first().map(|name| clean(name));
    // The party on the other side of a one-sided relation: the first other
    // validated party the reply placed on that side, when it did.
    let other = |roles: &[PartyRole]| {
        facts
            .parties
            .iter()
            .filter(|party| !named.contains(&party.name))
            .find(|party| party.role.is_some_and(|role| roles.contains(&role)))
            .map(|party| clean(&party.name))
            .filter(|_| keep(Part::SecondParty))
    };
    let issuer_side = || other(PROVIDER);
    let receiving_side = || other(RECEIVING);

    let in_title = facts.identifier_in_title && keep(Part::Identifier);
    let mut text = match facts.identifier {
        Some((_, identifier)) if in_title => format!("{kind} {}", clean(identifier)),
        _ => kind,
    };
    let mut tail: Vec<String> = Vec::new();
    match (facts.class, joining) {
        (_, PartyRelation::Between) => {
            text.push_str(&format!(
                " between {}",
                join_two(named, keep(Part::SecondParty))
            ));
            if let Some(subject) = &subject {
                tail.push(format!(" for {subject}"));
            }
        }
        (DocumentClass::Agreement | DocumentClass::Amendment, PartyRelation::With) => {
            if let Some(first) = &first {
                text.push_str(&format!(" with {first}"));
            }
            if let Some(subject) = &subject {
                tail.push(format!(" for {subject}"));
            }
        }
        (DocumentClass::Issued, _) => {
            if joining == PartyRelation::From
                && let Some(first) = &first
            {
                text.push_str(&format!(" from {first}"));
                if let Some(customer) = receiving_side() {
                    text.push_str(&format!(" to {customer}"));
                }
            } else if let Some(first) = &first {
                text.push_str(&format!(" {} {first}", word_for(joining)));
            } else if let Some(customer) = receiving_side() {
                // No issuer named: the customer the document bills is
                // still who it is to.
                text.push_str(&format!(" to {customer}"));
            }
            if let Some(subject) = &subject {
                tail.push(format!(" for {subject}"));
            }
        }
        (DocumentClass::Notice | DocumentClass::Letter | DocumentClass::Email, _) => {
            let about = if facts.class == DocumentClass::Email {
                "about"
            } else {
                "regarding"
            };
            match joining {
                PartyRelation::From => {
                    if let Some(first) = &first {
                        text.push_str(&format!(" from {first}"));
                    }
                    if let Some(to) = receiving_side() {
                        text.push_str(&format!(" to {to}"));
                    }
                }
                PartyRelation::For | PartyRelation::To => {
                    if let Some(from) = issuer_side() {
                        text.push_str(&format!(" from {from}"));
                    }
                    if let Some(first) = &first {
                        text.push_str(&format!(" to {first}"));
                    }
                }
                _ => {
                    if let Some(first) = &first {
                        text.push_str(&format!(" {} {first}", word_for(joining)));
                    }
                }
            }
            if let Some(subject) = &subject {
                tail.push(format!(" {about} {subject}"));
            }
        }
        (DocumentClass::Record, _) => {
            match joining {
                PartyRelation::For | PartyRelation::To => {
                    if let Some(first) = &first {
                        text.push_str(&format!(" for {first}"));
                    }
                }
                _ => {
                    if let Some(first) = &first {
                        text.push_str(&format!(" {} {first}", word_for(joining)));
                    }
                }
            }
            if let Some(subject) = &subject {
                tail.push(format!(" covering {subject}"));
            }
            if matches!(joining, PartyRelation::For | PartyRelation::To)
                && keep(Part::PreparedBy)
                && let Some(preparer) = issuer_side()
            {
                tail.push(format!(", prepared by {preparer}"));
            }
        }
        (DocumentClass::Form, _) => {
            if let Some(first) = &first {
                let word = if joining == PartyRelation::None {
                    word_for(joining)
                } else {
                    "for"
                };
                text.push_str(&format!(" {word} {first}"));
            }
            if joining == PartyRelation::For
                && let Some(counterparty) = facts
                    .parties
                    .iter()
                    .filter(|party| !named.contains(&party.name))
                    .map(|party| clean(&party.name))
                    .find(|_| keep(Part::SecondParty))
            {
                text.push_str(&format!(" submitted to {counterparty}"));
            }
            if let Some(subject) = &subject {
                tail.push(format!(" regarding {subject}"));
            }
        }
        _ => {
            if let Some(first) = &first {
                text.push_str(&format!(" {} {first}", word_for(joining)));
            }
            if let Some(subject) = &subject {
                tail.push(format!(" concerning {subject}"));
            }
        }
    }
    for part in tail {
        text.push_str(&part);
    }
    if keep(Part::Identifier)
        && !in_title
        && let Some((label, identifier)) = facts.identifier
    {
        text.push_str(&format!(", {label} {}", clean(identifier)));
    }
    if !beside_subject && let Some((label, amount)) = amount {
        let total = label.is_none_or(is_total_label);
        match label {
            _ if total && (facts.class == DocumentClass::Issued || label.is_some()) => {
                text.push_str(&format!(", totalling {amount}"));
            }
            None => text.push_str(&format!(", for {amount}")),
            Some(label) => text.push_str(&format!(", {label} of {amount}")),
        }
    }
    if dated && let Some(date) = facts.date_surface {
        text.push_str(&format!(", dated {}", clean(date)));
    }
    finish_sentence(text)
}

/// Whether an amount's label names a total: "Total", "Amount Due",
/// "Balance due".
fn is_total_label(label: &str) -> bool {
    let lowered = label.to_lowercase();
    [
        "total",
        "amount due",
        "balance due",
        "balance",
        "amount",
        "sum",
        "grand total",
        "net amount",
        "amount payable",
    ]
    .iter()
    .any(|word| contains_word(&lowered, word))
}

/// The word a one-sided relation reads with, and "involving" for none.
fn word_for(relation: PartyRelation) -> &'static str {
    match relation {
        PartyRelation::None => "involving",
        other => other.as_str(),
    }
}

/// "A and B", or "A" alone when the second party is dropped.
fn join_two(names: &[String], second: bool) -> String {
    match names {
        [a, b, ..] if second => format!("{} and {}", clean(a), clean(b)),
        [a, ..] => clean(a),
        [] => String::new(),
    }
}

/// A value as it reads inside a sentence: whitespace collapsed, wrapping
/// quotes and trailing punctuation gone. A period that closes an
/// abbreviation ("Inc.") stays.
fn clean(value: &str) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed
        .trim_matches(|character: char| matches!(character, '"' | '\'' | '\u{201c}' | '\u{201d}'))
        .trim_end_matches([',', ';', ':'])
        .trim();
    let trimmed = if trimmed.ends_with('.') && !ends_with_abbreviation(trimmed) {
        trimmed.trim_end_matches('.')
    } else {
        trimmed
    };
    trimmed.to_owned()
}

/// Whether a value ends on an abbreviation whose period belongs to it.
fn ends_with_abbreviation(value: &str) -> bool {
    let last = value
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    matches!(
        last.as_str(),
        "inc"
            | "co"
            | "corp"
            | "ltd"
            | "llc"
            | "l.l.c"
            | "n.a"
            | "p.c"
            | "l.p"
            | "jr"
            | "sr"
            | "no"
            | "u.s"
    ) || (last.len() == 1
        && last
            .chars()
            .all(|character| character.is_ascii_alphabetic()))
}

/// A subject as a phrase that follows a preposition: a leading
/// preposition the reply wrote is dropped ("for consulting services"), and
/// a sentence's capital on an ordinary first word is lowered.
fn subject_phrase(subject: &str) -> String {
    let mut value = clean(subject);
    for lead in [
        "for ",
        "regarding ",
        "about ",
        "concerning ",
        "covering ",
        "re: ",
        "re ",
        "subject: ",
    ] {
        if value.len() > lead.len()
            && value
                .get(..lead.len())
                .is_some_and(|start| start.eq_ignore_ascii_case(lead))
        {
            value = value[lead.len()..].trim_start().to_owned();
            break;
        }
    }
    let mut words = value.split(' ');
    if let Some(first) = words.next()
        && is_ordinary_capitalised(first)
    {
        let mut characters = first.chars();
        let lowered = characters
            .next()
            .map(|character| {
                character
                    .to_lowercase()
                    .chain(characters)
                    .collect::<String>()
            })
            .unwrap_or_default();
        let rest = words.collect::<Vec<_>>().join(" ");
        value = if rest.is_empty() {
            lowered
        } else {
            format!("{lowered} {rest}")
        };
    }
    value
}

/// Common words a reply capitalises only because they open its phrase.
const ORDINARY_OPENERS: &[&str] = &[
    "the",
    "a",
    "an",
    "implementation",
    "provision",
    "supply",
    "purchase",
    "sale",
    "lease",
    "rental",
    "delivery",
    "consulting",
    "services",
    "support",
    "maintenance",
    "installation",
    "renewal",
    "termination",
    "payment",
    "repayment",
    "development",
    "design",
    "construction",
    "approval",
    "annual",
    "monthly",
    "quarterly",
    "new",
    "proposed",
    "outstanding",
    "professional",
    "legal",
    "accounting",
    "audit",
    "tax",
    "insurance",
    "coverage",
    "employment",
    "distribution",
    "licensing",
    "license",
    "processing",
    "transfer",
    "assignment",
    "credit",
    "loan",
    "office",
    "retail",
    "residential",
    "commercial",
    "general",
    "work",
    "repairs",
    "repair",
    "inspection",
    "replacement",
    "freight",
    "shipment",
    "goods",
    "equipment",
    "materials",
    "products",
];

fn is_ordinary_capitalised(word: &str) -> bool {
    let mut characters = word.chars();
    let starts_upper = characters.next().is_some_and(char::is_uppercase);
    let rest_lower = characters.all(|character| !character.is_uppercase());
    starts_upper && rest_lower && ORDINARY_OPENERS.contains(&word.to_lowercase().as_str())
}

/// At most `most` words of a phrase, never ending on a word that needs one
/// after it.
fn shorten(phrase: &str, most: usize) -> String {
    let mut words = phrase.split_whitespace().take(most).collect::<Vec<_>>();
    while words.len() > 1
        && words.last().is_some_and(|word| {
            matches!(
                word.to_lowercase().trim_end_matches(','),
                "of" | "for"
                    | "and"
                    | "or"
                    | "the"
                    | "a"
                    | "an"
                    | "to"
                    | "in"
                    | "on"
                    | "at"
                    | "with"
                    | "by"
                    | "from"
            )
        })
    {
        words.pop();
    }
    words.join(" ").trim_end_matches([',', ';', ':']).to_owned()
}

/// Capitalises the sentence's first letter and closes it with one period.
fn finish_sentence(text: String) -> String {
    let mut characters = text.chars();
    let mut sentence = match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect::<String>(),
        None => String::new(),
    };
    let sentence_trimmed = sentence
        .trim_end()
        .trim_end_matches([',', ';', ':'])
        .to_owned();
    sentence = sentence_trimmed;
    if !sentence.ends_with('.') {
        sentence.push('.');
    }
    sentence
}

/// The word an identifier's label gives it in a description: "Invoice No."
/// is an invoice number, "PO Number" an order's.
pub fn identifier_word(label: Option<&str>) -> &'static str {
    let Some(label) = label else {
        return "no.";
    };
    let lowered = label.to_lowercase();
    let has = |word: &str| contains_word(&lowered, word);
    if has("invoice") {
        "invoice"
    } else if has("po") || has("p.o") || has("purchase order") || has("order") {
        "order"
    } else if has("policy") {
        "policy"
    } else if has("loan") {
        "loan"
    } else if has("claim") {
        "claim"
    } else if has("account") || has("acct") {
        "account"
    } else if has("case") || has("docket") {
        "case"
    } else if has("quote") || has("quotation") {
        "quote"
    } else if has("receipt") {
        "receipt"
    } else if has("contract") || has("agreement") {
        "contract"
    } else if has("reference") || has("ref") {
        "reference"
    } else {
        "no."
    }
}

/// The first amount of money a key fact states - "$12,450.00", "USD
/// 1,000", "EUR 3.500,00" - verbatim.
pub fn money_in(fact: &str) -> Option<&str> {
    const SYMBOLS: &[char] = &['$', '\u{20ac}', '\u{a3}', '\u{a5}'];
    const CODES: &[&str] = &["USD", "EUR", "GBP", "CAD", "AUD", "CHF", "JPY"];
    let bytes = fact.as_bytes();
    let mut index = 0;
    while index < fact.len() {
        let rest = &fact[index..];
        let lead = if rest.starts_with(SYMBOLS) {
            rest.chars().next().map(char::len_utf8)
        } else {
            CODES
                .iter()
                .find(|code| {
                    rest.starts_with(**code)
                        && !fact[..index]
                            .chars()
                            .next_back()
                            .is_some_and(char::is_alphanumeric)
                })
                .map(|code| code.len())
        };
        if let Some(lead) = lead {
            let mut end = index + lead;
            while end < fact.len() && bytes[end] == b' ' {
                end += 1;
            }
            let digits_start = end;
            while end < fact.len()
                && (bytes[end].is_ascii_digit()
                    || (matches!(bytes[end], b',' | b'.')
                        && bytes.get(end + 1).is_some_and(u8::is_ascii_digit)))
            {
                end += 1;
            }
            if end > digits_start {
                // "$5.2 million" is the amount, not "$5.2".
                let rest = &fact[end..];
                for scale in [" million", " billion", " thousand"] {
                    if rest
                        .get(..scale.len())
                        .is_some_and(|word| word.eq_ignore_ascii_case(scale))
                        && !rest[scale.len()..]
                            .chars()
                            .next()
                            .is_some_and(char::is_alphanumeric)
                    {
                        end += scale.len();
                        break;
                    }
                }
                return Some(&fact[index..end]);
            }
        }
        index += rest.chars().next().map_or(1, char::len_utf8);
    }
    None
}

/// The word a labelled amount reads with outside an issued document - a
/// loan's principal, a lease's rent - or `None` for an amount nothing
/// labels as one of those.
pub fn amount_label(fact: &str) -> Option<&'static str> {
    let lowered = fact.to_lowercase();
    [
        ("purchase price", "purchase price"),
        ("principal", "principal"),
        ("commitment", "commitment"),
        ("premium", "premium"),
        ("base rent", "base rent"),
        ("rent", "rent"),
        ("base salary", "base salary"),
        ("salary", "salary"),
        ("settlement payment", "settlement payment"),
        ("retainer", "retainer"),
        ("security deposit", "security deposit"),
        ("deposit", "deposit"),
        ("fee", "fee"),
        ("price", "price"),
    ]
    .into_iter()
    .find(|(word, _)| contains_word(&lowered, word))
    .map(|(_, label)| label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cast(parties: &[(&str, Option<PartyRole>)]) -> Vec<CastMember> {
        parties
            .iter()
            .map(|(name, role)| CastMember {
                name: (*name).to_owned(),
                role: *role,
                role_supported: role.is_some(),
            })
            .collect()
    }

    #[test]
    fn the_class_of_a_type_tests_the_families_that_contain_other_families_words_first() {
        for (kind, class) in [
            ("Data Processing Addendum", DocumentClass::Amendment),
            ("Change Order", DocumentClass::Amendment),
            (
                "Second Amendment to Software License Agreement",
                DocumentClass::Amendment,
            ),
            ("Notice of Freight Claim", DocumentClass::Notice),
            ("Property Loss Notice", DocumentClass::Notice),
            (
                "Commercial Package Policy Declarations",
                DocumentClass::Record,
            ),
            ("Explanation of Benefits", DocumentClass::Record),
            ("Statement of Work", DocumentClass::Agreement),
            ("Statement of Account", DocumentClass::Issued),
            ("Account Statement", DocumentClass::Issued),
            ("Bill of Lading", DocumentClass::Issued),
            ("Carrier Rate Confirmation", DocumentClass::Issued),
            ("Price List", DocumentClass::Issued),
            ("Remittance Advice", DocumentClass::Issued),
            ("Purchase Order", DocumentClass::Issued),
            ("Promissory Note", DocumentClass::Agreement),
            (
                "Assignment and Assumption of Lease",
                DocumentClass::Agreement,
            ),
            ("Approval Email", DocumentClass::Email),
            ("Letter of Resignation", DocumentClass::Letter),
            ("New Patient Intake Form", DocumentClass::Form),
            ("Vendor Registration Form", DocumentClass::Form),
            ("Aircraft Maintenance Record", DocumentClass::Record),
            ("Accounts Payable Aging Summary", DocumentClass::Record),
            (
                "Action by Unanimous Written Consent of the Board of Directors",
                DocumentClass::Record,
            ),
            ("Quarterly Business Review", DocumentClass::Record),
            ("Billboard Placement Brief", DocumentClass::Unknown),
        ] {
            assert_eq!(DocumentClass::of(Some(kind)), class, "{kind}");
        }
        assert_eq!(DocumentClass::of(None), DocumentClass::Unknown);
    }

    #[test]
    fn an_unsupported_role_never_decides_the_relation() {
        let mut parties = cast(&[
            ("Halvorsen Fixture Works LLC", Some(PartyRole::Customer)),
            ("Quillon Ridge Bakery, Inc.", Some(PartyRole::Issuer)),
        ]);
        parties[1].role_supported = false;
        parties[0].role_supported = false;
        let relation = relation_from_roles(
            DocumentClass::Issued,
            Some("Invoice"),
            &parties,
            RelationCues::default(),
        );
        assert_eq!(relation.relation, PartyRelation::None);
        assert_eq!(relation.parties, vec!["Halvorsen Fixture Works LLC"]);
        // The document's own cues still settle it, as the digest pipeline's
        // repair does.
        let relation = relation_from_roles(
            DocumentClass::Issued,
            Some("Invoice"),
            &parties,
            RelationCues {
                issuer: Some(0),
                between: false,
            },
        );
        assert_eq!(relation.relation, PartyRelation::From);
        assert_eq!(relation.parties, vec!["Halvorsen Fixture Works LLC"]);
    }

    #[test]
    fn an_invoice_names_its_issuer_alone_and_its_description_names_both() {
        let parties = cast(&[
            ("Acme", Some(PartyRole::Issuer)),
            ("Contoso", Some(PartyRole::Customer)),
        ]);
        let relation = relation_from_roles(
            DocumentClass::Issued,
            Some("Invoice"),
            &parties,
            RelationCues::default(),
        );
        assert_eq!(relation.relation, PartyRelation::From);
        assert_eq!(relation.parties, vec!["Acme"]);
        let description = describe(&DescriptionFacts {
            class: DocumentClass::Issued,
            document_type: Some("Invoice"),
            relation: Some(&relation),
            parties: &parties,
            subject: Some("October 2026 consulting services"),
            identifier: Some(("invoice", "INV-10438")),
            ..DescriptionFacts::default()
        });
        assert_eq!(
            description,
            "Invoice from Acme to Contoso for October 2026 consulting services, invoice INV-10438."
        );
    }

    #[test]
    fn an_agreement_reads_between_its_first_two_parties() {
        let parties = cast(&[("Acme", Some(PartyRole::Contractor)), ("Contoso", None)]);
        let relation = relation_from_roles(
            DocumentClass::Agreement,
            Some("Services Agreement"),
            &parties,
            RelationCues::default(),
        );
        assert_eq!(relation.relation, PartyRelation::Between);
        let description = describe(&DescriptionFacts {
            class: DocumentClass::Agreement,
            document_type: Some("Services Agreement"),
            relation: Some(&relation),
            parties: &parties,
            subject: Some("Implementation and support of the Orion analytics platform"),
            ..DescriptionFacts::default()
        });
        assert_eq!(
            description,
            "Services Agreement between Acme and Contoso for implementation and support of the \
             Orion analytics platform."
        );
        let alone = relation_from_roles(
            DocumentClass::Agreement,
            Some("Services Agreement"),
            &parties[..1],
            RelationCues::default(),
        );
        assert_eq!(alone.relation, PartyRelation::With);
    }

    #[test]
    fn a_long_description_drops_its_parts_in_the_fixed_order_and_is_never_cut() {
        let parties = cast(&[
            (
                "Wrenfield Restaurant Supply Company of the Greater Northern Valley Region",
                Some(PartyRole::Issuer),
            ),
            (
                "Larkspur Bistro and Catering Collective of the Lower Harbour District",
                Some(PartyRole::Customer),
            ),
        ]);
        let relation = relation_from_roles(
            DocumentClass::Issued,
            Some("Invoice"),
            &parties,
            RelationCues::default(),
        );
        let facts = DescriptionFacts {
            class: DocumentClass::Issued,
            document_type: Some("Invoice"),
            relation: Some(&relation),
            parties: &parties,
            subject: Some(
                "restaurant equipment, kitchen supplies, cleaning products, staff uniforms, table linens \
                 and weekly delivery for the spring and summer menus",
            ),
            identifier: Some(("invoice", "INV-2026-0042")),
            amount: Some((None, "$12,450.00")),
            ..DescriptionFacts::default()
        };
        let full = compose(&facts, &[], false, false);
        assert!(word_count(&full) > MAX_DESCRIPTION_WORDS, "{full}");
        let description = describe(&facts);
        assert!(
            word_count(&description) <= MAX_DESCRIPTION_WORDS,
            "{description}"
        );
        assert!(
            !description.contains("INV-2026-0042"),
            "the identifier goes before the amount: {description}"
        );
        assert!(description.contains("$12,450.00"), "{description}");
        assert!(description.ends_with('.'));
        assert!(description.contains("Larkspur"), "{description}");
    }

    #[test]
    fn a_short_description_gets_the_documents_own_date() {
        let parties = cast(&[("Rowanbrae Vineyards", Some(PartyRole::Issuer))]);
        let relation = relation_from_roles(
            DocumentClass::Record,
            Some("Harvest Log"),
            &parties,
            RelationCues::default(),
        );
        let description = describe(&DescriptionFacts {
            class: DocumentClass::Record,
            document_type: Some("Harvest Log"),
            relation: Some(&relation),
            parties: &parties,
            date_surface: Some("September 14, 2025"),
            ..DescriptionFacts::default()
        });
        assert_eq!(
            description,
            "Harvest Log from Rowanbrae Vineyards, dated September 14, 2025."
        );
    }

    #[test]
    fn money_and_identifier_labels_are_read_from_the_facts() {
        assert_eq!(
            money_in("Total due: $12,450.00 by May 1"),
            Some("$12,450.00")
        );
        assert_eq!(money_in("USD 1,000 per month"), Some("USD 1,000"));
        assert_eq!(money_in("net 30 days"), None);
        assert_eq!(
            money_in("revenue of $5.2 million in 2026"),
            Some("$5.2 million")
        );
        assert_eq!(money_in("FOCUSD 12"), None);
        assert_eq!(
            amount_label("Principal amount of $2,500,000"),
            Some("principal")
        );
        assert_eq!(amount_label("Total due"), None);
        assert_eq!(identifier_word(Some("Invoice No.")), "invoice");
        assert_eq!(identifier_word(Some("PO Number")), "order");
        assert_eq!(identifier_word(Some("Policy Number")), "policy");
        assert_eq!(identifier_word(Some("Loan No.")), "loan");
        assert_eq!(identifier_word(None), "no.");
    }
}
