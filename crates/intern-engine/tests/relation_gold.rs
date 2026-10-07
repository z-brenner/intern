//! The relation table against every InternBench gold document.
//!
//! For each document in `bench/gold.json`, every party is given each role a
//! reply could reasonably give it - the gold's own roles, mapped onto
//! [`PartyRole`] - in both orders of first appearance, and the filename's
//! relation and parties the table derives must be one the gold accepts.
//!
//! The gold's "subject" and "patient" are not roles a reply can give, so
//! they stand for a party with no supported role. One combination may then
//! leave nothing to decide by: a party with no role whose only counterpart
//! is "other". The table keeps the names without a joining word there
//! rather than guess, and the test holds it to exactly that.

use std::collections::BTreeSet;

use intern_engine::compose::{CastMember, relation_from_roles};
use intern_engine::{DocumentClass, PartyRelation, PartyRole};
use serde_json::Value;

/// The gold's role words as a reply's roles. `None` is a party the reply
/// gives no role the document supports.
fn reply_role(gold: &str) -> Option<PartyRole> {
    match gold {
        "provider" | "firm" => Some(PartyRole::Contractor),
        "payer" | "fund" => Some(PartyRole::Issuer),
        "investor" => Some(PartyRole::Recipient),
        "counterparty" | "assignor" | "assignee" => Some(PartyRole::Other),
        other => PartyRole::parse(other),
    }
}

fn relation_word(word: &str) -> PartyRelation {
    PartyRelation::ALL
        .into_iter()
        .find(|relation| relation.as_str() == word)
        .unwrap_or_else(|| panic!("unknown relation {word}"))
}

/// A relation and its parties as the gold compares them: a set for
/// "between", in order otherwise.
fn key(relation: PartyRelation, parties: &[String]) -> (&'static str, Vec<String>) {
    let mut parties = parties.to_vec();
    if relation == PartyRelation::Between {
        parties.sort();
    }
    (relation.as_str(), parties)
}

#[test]
fn the_relation_table_gives_every_gold_document_an_accepted_relation() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../bench/gold.json");
    let gold: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let documents = gold["documents"].as_array().unwrap();
    assert!(documents.len() >= 72, "{}", documents.len());
    let mut checked = 0;
    for document in documents {
        let id = document["id"].as_str().unwrap();
        let answer = &document["gold"];
        let document_type = answer["document_type"].as_str().unwrap();
        let mut names: Vec<String> = Vec::new();
        let mut roles: Vec<Vec<Option<PartyRole>>> = Vec::new();
        for entry in answer["party_roles"].as_array().into_iter().flatten() {
            let name = entry["name"].as_str().unwrap().to_owned();
            let role = reply_role(entry["role"].as_str().unwrap());
            let index = match names.iter().position(|known| *known == name) {
                Some(index) => index,
                None => {
                    names.push(name);
                    roles.push(Vec::new());
                    names.len() - 1
                }
            };
            if !roles[index].contains(&role) {
                roles[index].push(role);
            }
        }
        if names.is_empty() {
            continue;
        }
        let mut accepted = BTreeSet::new();
        let parties_of = |value: &Value| {
            value
                .as_array()
                .into_iter()
                .flatten()
                .map(|name| name.as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        };
        accepted.insert(key(
            relation_word(answer["party_relation"].as_str().unwrap()),
            &parties_of(&answer["parties"]),
        ));
        for set in answer["acceptable_party_sets"]
            .as_array()
            .into_iter()
            .flatten()
        {
            accepted.insert(key(
                relation_word(set["relation"].as_str().unwrap()),
                &parties_of(&set["parties"]),
            ));
        }
        let none_accepted = accepted
            .iter()
            .any(|(relation, _)| *relation == PartyRelation::None.as_str());
        let class = DocumentClass::of(Some(document_type));
        let mut orders = vec![(0..names.len()).collect::<Vec<_>>()];
        orders.push((0..names.len()).rev().collect());
        for order in orders {
            // Every combination of the roles each party could be given.
            let mut choice = vec![0_usize; names.len()];
            loop {
                let cast = order
                    .iter()
                    .map(|&index| {
                        let role = roles[index][choice[index]];
                        CastMember {
                            name: names[index].clone(),
                            role,
                            role_supported: role.is_some(),
                        }
                    })
                    .collect::<Vec<_>>();
                let derived = relation_from_roles(class, Some(document_type), &cast, None);
                checked += 1;
                let accepted_here = accepted.contains(&key(derived.relation, &derived.parties))
                    || (derived.relation == PartyRelation::None && none_accepted);
                if !accepted_here {
                    // The one combination the table may leave unresolved:
                    // a party with no role and only "other" beside it.
                    let nothing_to_decide_by = cast.iter().any(|party| party.role.is_none())
                        && cast
                            .iter()
                            .all(|party| matches!(party.role, None | Some(PartyRole::Other)));
                    assert!(
                        nothing_to_decide_by && derived.relation == PartyRelation::None,
                        "{id} ({}): {cast:?} gave {derived:?}, the gold accepts {accepted:?}",
                        class.as_str()
                    );
                }
                // The next combination.
                let mut position = 0;
                loop {
                    if position == choice.len() {
                        break;
                    }
                    choice[position] += 1;
                    if choice[position] < roles[position].len() {
                        break;
                    }
                    choice[position] = 0;
                    position += 1;
                }
                if position == choice.len() {
                    break;
                }
            }
        }
    }
    assert!(checked > 300, "{checked}");
}
