//! Your organisation: the names a firm's own filing should look past.
//!
//! A firm's own name is on almost every document it files - the engagement
//! letter it sent, the contract it signed, the invoice it was billed - so a
//! filename that carries every party says the same thing over and over, and
//! a Party layout scatters one counterparty's documents across folders by
//! whichever side the model happened to name first. The person names their
//! own organisation once; the name, and the Party folder, then carry the
//! other side.
//!
//! Like house style, this is applied deterministically after validation:
//! the evidence and the description stay the document's own words, and only
//! the name a document is filed under changes. Nothing here asks the model
//! anything, and nothing is applied unless a name was configured.

use crate::domain::{PartyRelation, ValidatedProposal};
use crate::house_style::HouseRule;

/// An own name whose key is shorter than this would match far too much -
/// "Co", "LLP", "The" - so it is ignored rather than trusted.
const MIN_OWN_KEY_CHARS: usize = 4;

/// Whether `party` names the organisation `own` names.
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

/// The proposal as a name filed by the counterparty carries it, and the
/// parties left out because they are the person's own organisation.
///
/// Unchanged when no own name is configured, when the document names fewer
/// than two parties, or when it names only the organisation or only others:
/// the only party is never dropped, because a name with no party at all says
/// less than one that names the firm. Otherwise the organisation's parties
/// go and the others keep their order.
///
/// The connecting word follows. "between" needs two sides, so one remaining
/// party is "with" it. A directional word - "from", "to", "for" - is about
/// the first party alone, which is why validation keeps only that one; a
/// proposal stored before it did can still carry two. When that first party
/// was the organisation, the word described the firm and not the other side,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DateRole, Evidence};
    use crate::naming::compose_filename;

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
}
