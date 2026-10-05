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
/// diff would credit it all to one party.
pub fn lesson_from_edit(
    proposal: &ValidatedProposal,
    extension: &str,
    proposed: &str,
    approved: &str,
) -> Option<HouseRule> {
    let extension = sanitize_extension(extension);
    let before = editable_stem(proposed, &extension);
    let after = editable_stem(approved, &extension);
    if before == after || after.is_empty() {
        return None;
    }
    let shape = NameShape::of(proposal, &before, &extension)?;
    let rule = match shape.clause {
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
                (Some(new_clause), _) if new_clause != clause.text => clause.lesson(new_clause)?,
                (_, Some(new_type)) => HouseRule::new(
                    RuleKind::DocumentType,
                    proposal.document_type.clone()?,
                    new_type,
                ),
                // Both the type and the party clause changed, or the
                // connecting word did: a decision about this document.
                _ => return None,
            }
        }
    };
    rule.is_meaningful().then_some(rule)
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
    /// sanitised segment each contributed to the name.
    parties: Vec<(String, String)>,
}

impl NameShape {
    /// `None` when the stem is not one this proposal composes - it was
    /// truncated for length, or came from somewhere else - because guessing
    /// which words are which would teach the wrong thing.
    ///
    /// The segments are built exactly as naming builds them, in every
    /// [`Spelling`] a name can carry a word in: the document's words
    /// title-cased, and a house-style rule's spelling as the reviewer typed
    /// it. A name Intern proposed for a document printed in capitals, and
    /// one carrying a spelling a reviewer typed in capitals, are both names
    /// this proposal composes.
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
        let single_connectors = [connector_word(single)];
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
    /// The one party the reviewer respelled, if exactly one was.
    fn lesson(&self, approved: &str) -> Option<HouseRule> {
        match self.parties.as_slice() {
            [(from, _)] => Some(HouseRule::new(RuleKind::Party, from.clone(), approved)),
            [(first, first_segment), (second, second_segment)] => {
                let second_changed = approved
                    .strip_prefix(first_segment.as_str())
                    .and_then(|rest| rest.strip_prefix(" and "));
                let first_changed = approved
                    .strip_suffix(second_segment.as_str())
                    .and_then(|rest| rest.strip_suffix(" and "));
                match (second_changed, first_changed) {
                    (Some(to), _) if to != second_segment => {
                        Some(HouseRule::new(RuleKind::Party, second.clone(), to))
                    }
                    (_, Some(to)) => Some(HouseRule::new(RuleKind::Party, first.clone(), to)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
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
