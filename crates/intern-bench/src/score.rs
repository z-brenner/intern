//! Scores one document's outcome against its gold answer.
//!
//! Every score is a key in one map: a boolean (counted as a rate), an
//! integer (summed), or a fraction (averaged). A score the gold does not
//! define is left out rather than set false, so a rate's denominator is
//! always the documents that could be right or wrong about it.
//!
//! A document that failed - the worker could not read it, or the model
//! gave no usable answer - is scored on exactly the keys the reviewed
//! answer would be scored on, and is a miss on every one: each
//! good-when-true score is false (the conditional ones too: the date role,
//! the relation word, the party's role), each fraction is 0, and a scan's
//! OCR counts as read empty. It sprang no trap and was filed under no name,
//! so the trap and unsafe scores are false. It asserted nothing, so the
//! description's claims are not checked (`description_factual` and the
//! claim counts are absent).
//!
//! The comparisons the corpus evaluator (`intern-evaluate`) has proven -
//! what counts as the right type, the same party, the right filename - are
//! the same here, copied rather than shared because they live in that
//! binary.

use std::{collections::BTreeMap, path::Path};

use intern_engine::{
    DocumentSource, Evidence, PartyRelation, ValidatedProposal, compose_filename,
    evidence::{date_match_positions, normalize},
    naming::windows_name_key,
};
use serde_json::{Value, json};

use crate::{
    claims::{Claim, DocumentText, check_claims, forbidden_hits},
    gold::{GoldDocument, PartySet},
    ocr::{OcrMeasure, measure},
    stats::round,
};

/// The keys whose `true` is the failure, not the success: a trap sprung, a
/// forbidden party named, a wrong name filed without review, a right one
/// sent to review.
pub fn bad_when_true(key: &str) -> bool {
    key.ends_with("_forbidden") || matches!(key, "unsafe_ready" | "needless_review")
}

/// Fractions where a smaller number is the better one.
pub fn lower_is_better(key: &str) -> bool {
    key.starts_with("ocr_cer") || key.starts_with("ocr_wer")
}

/// Fractional scores that are not a share of one: Tesseract's mean
/// confidence runs from 0 to 100, so it is shown as a number, never as a
/// percentage or a change in points.
pub fn is_unit_fraction(key: &str) -> bool {
    key != "ocr_mean_confidence"
}

/// What Intern produced for a document, in the terms it is scored on.
#[derive(Clone, Copy, Debug, Default)]
pub struct Outcome<'a> {
    /// False when extraction or the model failed and there is no analysis.
    /// The document still counts: every answer the gold defines is scored
    /// as missed, because a person was given no name (see the module
    /// documentation for exactly which keys).
    pub analysed: bool,
    pub filename: Option<&'a str>,
    pub description: &'a str,
    pub document_type: Option<&'a str>,
    pub document_date: Option<&'a str>,
    pub date_role: Option<&'a str>,
    pub parties: &'a [String],
    pub party_relation: Option<&'a str>,
    pub ready: bool,
    /// The evidence the model quoted, before validation withheld anything.
    pub evidence: Option<&'a Evidence>,
}

/// The texts a document's scores are checked against.
#[derive(Clone, Copy, Default)]
pub struct Texts<'a> {
    /// Extracted text, OCR truth, and any clean text, for checking the
    /// description's claims.
    pub document: Option<&'a DocumentText>,
    /// The digest a fresh distillation of the extracted text builds.
    pub digest: Option<&'a str>,
    /// The prompt last sent to the model, which a token-fitting
    /// redistillation may have condensed below the digest.
    pub prompt: Option<&'a str>,
    /// The extracted document, for OCR measurements.
    pub source: Option<&'a DocumentSource>,
}

#[derive(Clone, Debug, Default)]
pub struct Scored {
    pub scores: BTreeMap<String, Value>,
    pub claims: Vec<Claim>,
    pub forbidden_description: Vec<String>,
    /// Every trap the outcome sprang, with the gold's reason.
    pub traps: Vec<String>,
    pub ocr: Option<OcrMeasure>,
}

pub fn score(document: &GoldDocument, outcome: &Outcome<'_>, texts: &Texts<'_>) -> Scored {
    let gold = &document.gold;
    let mut scored = Scored::default();
    let scores = &mut scored.scores;
    let flag = |scores: &mut BTreeMap<String, Value>, key: &str, value: bool| {
        scores.insert(key.to_owned(), Value::Bool(value));
    };
    let fraction = |scores: &mut BTreeMap<String, Value>, key: &str, value: f64| {
        scores.insert(key.to_owned(), json!(round(value, 4)));
    };

    // Filename: the whole name a person sees.
    let gold_names = gold_filenames(document);
    let filename_correct = (!gold_names.is_empty()).then(|| {
        outcome.filename.is_some_and(|name| {
            let key = windows_name_key(name);
            gold_names.iter().any(|gold| windows_name_key(gold) == key)
        })
    });
    if let Some(correct) = filename_correct {
        flag(scores, "filename_correct", correct);
    }

    // Type.
    let types = gold
        .document_type
        .iter()
        .chain(&gold.acceptable_types)
        .collect::<Vec<_>>();
    if !types.is_empty() {
        flag(
            scores,
            "type_correct",
            outcome
                .document_type
                .is_some_and(|value| types.iter().any(|gold| type_matches(gold, value))),
        );
        flag(scores, "type_present", outcome.document_type.is_some());
    }

    // Date.
    let dates = gold
        .document_date
        .iter()
        .chain(&gold.acceptable_dates)
        .collect::<Vec<_>>();
    if !dates.is_empty() {
        flag(
            scores,
            "date_correct",
            outcome
                .document_date
                .is_some_and(|date| dates.iter().any(|gold| *gold == date)),
        );
        flag(
            scores,
            "date_exact",
            outcome.document_date.is_some()
                && outcome.document_date == gold.document_date.as_deref(),
        );
        flag(scores, "date_present", outcome.document_date.is_some());
        // Knowing why a date is the right one is what keeps picking it
        // from being luck. The gold's role is the role of the reviewed
        // date, so it is judged only when that date was chosen: another
        // acceptable date can rightly carry another role, and a role
        // attached to no date, or to a wrong one, measures nothing. A
        // failed document is a miss here as everywhere.
        if let Some(role) = gold.date_role.as_deref().filter(|role| !role.is_empty()) {
            if !outcome.analysed {
                flag(scores, "date_role_correct", false);
            } else if outcome.document_date.is_some()
                && outcome.document_date == gold.document_date.as_deref()
            {
                flag(scores, "date_role_correct", outcome.date_role == Some(role));
            }
        }
    }
    if !dates.is_empty() || !gold.forbidden_dates.is_empty() {
        let trap = outcome.document_date.and_then(|date| {
            gold.forbidden_dates
                .iter()
                .find(|forbidden| forbidden.value() == date)
        });
        flag(scores, "date_forbidden", trap.is_some());
        scored
            .traps
            .extend(trap.map(|trap| format!("date {}", trap.describe())));
    }

    // Parties. A failed document named nobody by failing, not by choice:
    // it matches no set, not even a reviewed answer that names nobody.
    let sets = gold.party_sets();
    let judged = if outcome.analysed {
        judge_parties(&sets, outcome.parties)
    } else {
        PartyJudgement::failed(&sets)
    };
    if gold.parties.is_some() {
        let counts = &judged.counts;
        flag(scores, "parties_correct", judged.correct_set.is_some());
        scores.insert("parties_matched".into(), json!(counts.matched));
        scores.insert("parties_expected".into(), json!(counts.expected));
        scores.insert("parties_spurious".into(), json!(counts.spurious));
        let reviewed_parties = gold.parties.as_deref().unwrap_or_default();
        let reviewed_relation = gold
            .party_relation
            .as_deref()
            .filter(|value| !value.is_empty());
        if !outcome.analysed {
            // Missed wherever the reviewed answer would be judged.
            if reviewed_relation.is_some() && !reviewed_parties.is_empty() {
                flag(scores, "relation_correct", false);
            }
            if let Some(relation) = reviewed_relation
                && party_role_correct(document, &sets, relation, reviewed_parties).is_some()
            {
                flag(scores, "party_role_correct", false);
            }
        }
        // The connecting word is judged for the parties actually named:
        // "for" is right for the tenant of a rent notice and wrong for its
        // landlord, though both are acceptable parties - the landlord with
        // "from".
        if reviewed_relation.is_some()
            && outcome.analysed
            && !outcome.parties.is_empty()
            && let Some(produced) = outcome.party_relation
        {
            let judged_sets = if !judged.exact.is_empty() {
                judged.exact.clone()
            } else if !judged.overlapping.is_empty() {
                judged.overlapping.clone()
            } else {
                vec![0]
            };
            flag(
                scores,
                "relation_correct",
                judged_sets
                    .iter()
                    .any(|&index| sets[index].relation.as_deref() == Some(produced)),
            );
        }
        if outcome.analysed
            && let Some(relation) = outcome.party_relation
            && let Some(correct) = party_role_correct(document, &sets, relation, outcome.parties)
        {
            flag(scores, "party_role_correct", correct);
        }
    }
    if gold.parties.is_some() || !gold.forbidden_parties.is_empty() {
        // A party the trap names is still right inside a party set that
        // names it - the bill-to customer of an invoice filed "between"
        // issuer and customer - and only there.
        let in_matched_set = |party: &str| {
            judged
                .correct_set
                .and_then(|index| sets.get(index))
                .is_some_and(|set| set.parties.iter().any(|gold| party_matches(gold, party)))
        };
        let traps = outcome
            .parties
            .iter()
            .filter(|party| !in_matched_set(party))
            .filter_map(|party| {
                gold.forbidden_parties
                    .iter()
                    .find(|forbidden| party_matches(forbidden.value(), party))
            })
            .collect::<Vec<_>>();
        flag(scores, "party_forbidden", !traps.is_empty());
        scored.traps.extend(
            traps
                .iter()
                .map(|trap| format!("party {}", trap.describe())),
        );
    }

    // Description.
    let description = outcome.description;
    if !gold.description_facts.is_empty() {
        let lowered = description.to_lowercase();
        let covered = gold
            .description_facts
            .iter()
            .filter(|forms| {
                forms
                    .iter()
                    .any(|form| !form.is_empty() && lowered.contains(&form.to_lowercase()))
            })
            .count();
        let completeness = covered as f64 / gold.description_facts.len() as f64;
        fraction(scores, "description_completeness", completeness);
        flag(
            scores,
            "description_complete",
            covered == gold.description_facts.len(),
        );
    }
    if outcome.analysed {
        if let Some(text) = texts.document {
            scored.claims = check_claims(description, text);
        }
        scored.forbidden_description = forbidden_hits(description, &gold.description_forbidden);
        let unsupported = scored
            .claims
            .iter()
            .filter(|claim| !claim.supported)
            .count();
        scores.insert("description_claims".into(), json!(scored.claims.len()));
        scores.insert("description_unsupported".into(), json!(unsupported));
        flag(
            scores,
            "description_factual",
            unsupported == 0 && scored.forbidden_description.is_empty(),
        );
    }
    let specificity = specificity(document, description, &scored.claims);
    fraction(scores, "description_specificity", specificity as f64 / 3.0);
    flag(scores, "description_specific", specificity >= 2);

    // Evidence: did the model quote the right lines, and did they reach it.
    if let Some(recall) = evidence_recall(document, outcome.evidence) {
        fraction(scores, "evidence_recall", recall);
    }
    if let Some(recall) = texts
        .digest
        .and_then(|digest| text_recall(document, digest))
    {
        fraction(scores, "digest_recall", recall);
    }
    if let Some(recall) = texts
        .prompt
        .and_then(|prompt| text_recall(document, prompt))
    {
        fraction(scores, "prompt_recall", recall);
    }

    // Readiness. `ready` is reported but never gated: it is a routing
    // decision, and `readiness_match` scores whether it was the right one.
    flag(scores, "ready", outcome.ready);
    if let Some(expected) = gold
        .expected_readiness
        .as_deref()
        .filter(|value| matches!(*value, "ready" | "needs_review"))
    {
        // A failed document was routed nowhere: it is not the review the
        // gold asks for any more than it is a ready name.
        flag(
            scores,
            "readiness_match",
            outcome.analysed && (expected == "ready") == outcome.ready,
        );
    }
    if let Some(correct) = filename_correct {
        flag(scores, "unsafe_ready", outcome.ready && !correct);
        if gold.expected_readiness.as_deref() == Some("ready") {
            flag(scores, "needless_review", !outcome.ready && correct);
        }
    }

    // OCR, on the pages that were scanned. A scan the extractor failed on
    // is read as empty rather than left out, so failing is never better
    // than reading badly.
    if let Some(truth) = &document.ocr_truth {
        let measured = match texts.source {
            Some(source) => measure(truth, source),
            None => OcrMeasure::unread(truth),
        };
        let ocr_scores = [
            ("ocr_cer", measured.cer()),
            ("ocr_cer_ci", measured.cer_ci()),
            ("ocr_wer", measured.wer()),
            ("ocr_date_accuracy", measured.date_accuracy()),
            ("ocr_name_accuracy", measured.name_accuracy()),
            ("ocr_identifier_accuracy", measured.identifier_accuracy()),
            ("ocr_mean_confidence", measured.mean_confidence),
        ];
        for (key, value) in ocr_scores {
            if let Some(value) = value {
                fraction(scores, key, value);
            }
        }
        scored.ocr = Some(measured);
    }
    scored
}

/// The filenames the reviewed answer composes to, the reviewed one first:
/// every reviewed or acceptable type, date, and party set, composed by the
/// engine's own naming with the document's extension. Empty when the gold
/// gives no date or no type, because then there is no name to compare.
///
/// A two-party `between` set is composed in both orders. The prompt never
/// says which side comes first, and `party_role_correct` already accepts
/// either, so "between A and B" and "between B and A" are the same right
/// answer; the reviewed order comes first.
pub fn gold_filenames(document: &GoldDocument) -> Vec<String> {
    let gold = &document.gold;
    let (Some(gold_type), Some(gold_date)) = (&gold.document_type, &gold.document_date) else {
        return Vec::new();
    };
    let types = std::iter::once(gold_type)
        .chain(&gold.acceptable_types)
        .collect::<Vec<_>>();
    let dates = std::iter::once(gold_date)
        .chain(&gold.acceptable_dates)
        .collect::<Vec<_>>();
    let mut sets = gold.party_sets();
    if sets.is_empty() {
        sets.push(PartySet::default());
    }
    let mut names = Vec::new();
    for set in &sets {
        let relation = set
            .relation
            .as_deref()
            .and_then(|value| {
                PartyRelation::ALL
                    .into_iter()
                    .find(|relation| relation.as_str() == value)
            })
            .unwrap_or(PartyRelation::None);
        let mut orders = vec![set.parties.clone()];
        if relation == PartyRelation::Between
            && let [first, second] = set.parties.as_slice()
        {
            orders.push(vec![second.clone(), first.clone()]);
        }
        for parties in &orders {
            for document_type in &types {
                for date in &dates {
                    let proposal = ValidatedProposal {
                        document_type: Some((*document_type).clone()),
                        document_date: Some((*date).clone()),
                        date_role: None,
                        parties: parties.clone(),
                        party_relation: relation,
                        description: String::new(),
                        confidence: 1.0,
                        evidence: Evidence::default(),
                    };
                    let name = compose_filename(&proposal, document.extension(), &[]).value;
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
            }
        }
    }
    names
}

/// A predicted type counts as correct when it carries every meaningful word
/// of the reviewed type. "Statement of Work No. 4" passes for "Statement of
/// Work"; "Employment Termination" does not pass for "Notice of Termination".
pub fn type_matches(gold: &str, actual: &str) -> bool {
    let actual = actual.to_lowercase();
    gold.to_lowercase()
        .split_whitespace()
        .filter(|word| word.len() > 2 && !matches!(*word, "the" | "and" | "for" | "with"))
        .all(|word| actual.contains(word))
}

/// Party names match when one contains the other after dropping
/// punctuation, so "Halvorsen Fixture Works, LLC" and "Halvorsen Fixture
/// Works LLC" are the same party.
pub fn party_matches(gold: &str, actual: &str) -> bool {
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_alphanumeric() || character.is_whitespace())
            .flat_map(char::to_lowercase)
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let gold = normalize(gold);
    let actual = normalize(actual);
    !gold.is_empty() && !actual.is_empty() && (gold.contains(&actual) || actual.contains(&gold))
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PartyCounts {
    matched: usize,
    expected: usize,
    spurious: usize,
}

struct PartyJudgement {
    /// The index of the first party set the output names exactly: every
    /// party, nobody else.
    correct_set: Option<usize>,
    /// Every set the output names exactly.
    exact: Vec<usize>,
    /// Every set the output names at least one party of.
    overlapping: Vec<usize>,
    /// Against the first exact set, or the reviewed one when none is.
    counts: PartyCounts,
}

impl PartyJudgement {
    /// A failed document: no set matched, counted against the reviewed
    /// one.
    fn failed(sets: &[PartySet]) -> Self {
        Self {
            correct_set: None,
            exact: Vec::new(),
            overlapping: Vec::new(),
            counts: PartyCounts {
                matched: 0,
                expected: sets.first().map_or(0, |set| set.parties.len()),
                spurious: 0,
            },
        }
    }
}

fn judge_parties(sets: &[PartySet], produced: &[String]) -> PartyJudgement {
    let counts = |set: &PartySet| PartyCounts {
        matched: set
            .parties
            .iter()
            .filter(|gold| produced.iter().any(|value| party_matches(gold, value)))
            .count(),
        expected: set.parties.len(),
        spurious: produced
            .iter()
            .filter(|value| !set.parties.iter().any(|gold| party_matches(gold, value)))
            .count(),
    };
    let all = sets.iter().map(counts).collect::<Vec<_>>();
    let exact = (0..sets.len())
        .filter(|&index| all[index].matched == all[index].expected && all[index].spurious == 0)
        .collect::<Vec<_>>();
    let overlapping = (0..sets.len())
        .filter(|&index| all[index].matched > 0)
        .collect::<Vec<_>>();
    let correct_set = exact.first().copied();
    PartyJudgement {
        counts: all
            .get(correct_set.unwrap_or(0))
            .copied()
            .unwrap_or_default(),
        correct_set,
        exact,
        overlapping,
    }
}

/// Whether the parties the filename names hold the roles its connecting
/// word claims for them.
///
/// For a one-sided relation the first party named must hold the role that
/// word implies: the issuer (or sender) of something `from`, the subject of
/// something `for`, the recipient of something `to`, the counterparty of
/// something `with`. "Notice for" the landlord fails even though the
/// landlord is an acceptable party, because the landlord is not its
/// subject. For `between`, every party the gold gives a role in its
/// `between` answer (or, without one, in any answer) must be named, in any
/// order. Not scored when the gold gives no party the role in question, or
/// the output names nobody.
fn party_role_correct(
    document: &GoldDocument,
    sets: &[PartySet],
    relation: &str,
    produced: &[String],
) -> Option<bool> {
    let roles = &document.gold.party_roles;
    if roles.is_empty() || produced.is_empty() {
        return None;
    }
    let expected: &[&str] = match relation {
        "from" => &["issuer", "sender"],
        "for" => &["subject"],
        "to" => &["recipient"],
        "with" => &["counterparty"],
        "between" => {
            let between = sets
                .iter()
                .filter(|set| set.relation.as_deref() == Some("between"))
                .collect::<Vec<_>>();
            let answers = if between.is_empty() {
                sets.iter().collect()
            } else {
                between
            };
            // A party is listed once per role it holds; it is one holder.
            let mut holders = roles
                .iter()
                .filter(|role| role.role != "other")
                .filter(|role| {
                    answers
                        .iter()
                        .flat_map(|set| &set.parties)
                        .any(|party| party_matches(party, &role.name))
                })
                .map(|role| role.name.as_str())
                .collect::<Vec<_>>();
            holders.sort_unstable();
            holders.dedup();
            if holders.len() < 2 {
                return None;
            }
            return Some(
                holders
                    .iter()
                    .all(|holder| produced.iter().any(|party| party_matches(holder, party))),
            );
        }
        _ => return None,
    };
    let holders = roles
        .iter()
        .filter(|role| expected.contains(&role.role.as_str()))
        .collect::<Vec<_>>();
    if holders.is_empty() {
        return None;
    }
    Some(
        holders
            .iter()
            .any(|holder| party_matches(&holder.name, &produced[0])),
    )
}

/// How many of the three marks of a specific description it carries: it
/// names a gold party or a gold fact; it carries a concrete detail (an
/// amount, a date, an identifier, a quantity, or a subject term); and it is
/// 10 to 42 words long.
fn specificity(document: &GoldDocument, description: &str, claims: &[Claim]) -> usize {
    let gold = &document.gold;
    let text = DocumentText::new([description]);
    let lowered = description.to_lowercase();
    let names_party = gold
        .party_sets()
        .iter()
        .flat_map(|set| set.parties.clone())
        .chain(gold.party_roles.iter().map(|role| role.name.clone()))
        .chain(gold.evidence.party_text.values().flatten().cloned())
        .any(|party| text.contains_words(&party));
    let names_fact = gold
        .description_facts
        .iter()
        .flatten()
        .any(|form| !form.is_empty() && lowered.contains(&form.to_lowercase()));
    let concrete = claims.iter().any(|claim| claim.kind.is_concrete())
        || crate::claims::extract_claims(description)
            .iter()
            .any(|claim| claim.kind.is_concrete())
        || gold
            .subject_terms
            .iter()
            .any(|term| !term.is_empty() && lowered.contains(&term.to_lowercase()));
    let words = description.split_whitespace().count();
    usize::from(names_party || names_fact)
        + usize::from(concrete)
        + usize::from((10..=42).contains(&words))
}

/// The fraction of the reviewed answer the model's evidence quotes: the
/// defining date (any of its reviewed surface forms, or the date itself in
/// any spelling), and each reviewed party. `None` when the gold defines
/// neither.
fn evidence_recall(document: &GoldDocument, evidence: Option<&Evidence>) -> Option<f64> {
    let gold = &document.gold;
    let mut items = 0;
    let mut satisfied = 0;
    let date_defined = gold.document_date.is_some() || !gold.evidence.date_text.is_empty();
    if date_defined {
        items += 1;
        let quoted = evidence.and_then(|evidence| evidence.date.as_deref());
        if let Some(quoted) = quoted {
            let quoted_normalized = normalize(quoted);
            let has_form = gold
                .evidence
                .date_text
                .iter()
                .any(|form| contains_normalized(&quoted_normalized, form));
            let states_date = gold
                .document_date
                .iter()
                .chain(&gold.acceptable_dates)
                .any(|date| !date_match_positions(date, &quoted_normalized).is_empty());
            satisfied += usize::from(has_form || states_date);
        }
    }
    for party in gold.parties.iter().flatten() {
        items += 1;
        let quoted = evidence.map_or(&[][..], |evidence| evidence.parties.as_slice());
        let forms = party_forms(document, party);
        let found = quoted.iter().any(|excerpt| {
            let excerpt = normalize(excerpt);
            forms.iter().any(|form| contains_normalized(&excerpt, form))
                || party_matches(party, &excerpt)
        });
        satisfied += usize::from(found);
    }
    (items > 0).then(|| satisfied as f64 / items as f64)
}

/// The fraction of the gold's evidence strings present in `text` (a digest
/// or a prompt): the date as any of its reviewed forms, counted once, and
/// each reviewed party in any of its forms.
fn text_recall(document: &GoldDocument, text: &str) -> Option<f64> {
    let gold = &document.gold;
    let normalized = normalize(text);
    let mut items = 0;
    let mut satisfied = 0;
    if !gold.evidence.date_text.is_empty() {
        items += 1;
        satisfied += usize::from(
            gold.evidence
                .date_text
                .iter()
                .any(|form| contains_normalized(&normalized, form)),
        );
    } else if let Some(date) = &gold.document_date {
        items += 1;
        satisfied += usize::from(!date_match_positions(date, &normalized).is_empty());
    }
    for party in gold.parties.iter().flatten() {
        items += 1;
        satisfied += usize::from(
            party_forms(document, party)
                .iter()
                .any(|form| contains_normalized(&normalized, form)),
        );
    }
    (items > 0).then(|| satisfied as f64 / items as f64)
}

/// The verbatim forms a party appears in, or its name when the gold lists
/// none.
fn party_forms(document: &GoldDocument, party: &str) -> Vec<String> {
    document
        .gold
        .evidence
        .party_text
        .get(party)
        .filter(|forms| !forms.is_empty())
        .cloned()
        .unwrap_or_else(|| vec![party.to_owned()])
}

fn contains_normalized(normalized_haystack: &str, needle: &str) -> bool {
    let needle = normalize(needle);
    !needle.is_empty() && normalized_haystack.contains(&needle)
}

/// The path a document is read from.
pub fn document_path(corpus: &Path, document: &GoldDocument) -> std::path::PathBuf {
    corpus.join(&document.file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gold::{Forbidden, GoldAnswer, GoldEvidence, PartyRole};

    fn invoice() -> GoldDocument {
        GoldDocument {
            id: "invoice".into(),
            file: "invoice.pdf".into(),
            kind: "invoice".into(),
            format: "pdf".into(),
            text_layer: "native".into(),
            pages: 1,
            gold: GoldAnswer {
                document_type: Some("Invoice".into()),
                document_date: Some("2026-03-04".into()),
                date_role: Some("invoice".into()),
                forbidden_dates: vec![Forbidden::Plain("2026-04-03".into())],
                parties: Some(vec!["Halvorsen Fixture Works LLC".into()]),
                party_relation: Some("from".into()),
                acceptable_party_sets: vec![PartySet {
                    parties: vec![
                        "Halvorsen Fixture Works LLC".into(),
                        "Quillon Ridge Bakery".into(),
                    ],
                    relation: Some("between".into()),
                }],
                party_roles: vec![
                    PartyRole {
                        name: "Halvorsen Fixture Works LLC".into(),
                        role: "issuer".into(),
                    },
                    PartyRole {
                        name: "Quillon Ridge Bakery".into(),
                        role: "customer".into(),
                    },
                ],
                forbidden_parties: vec![Forbidden::Plain("Quillon Ridge Bakery".into())],
                description_facts: vec![
                    vec!["$4,812.50".into(), "4812.50".into()],
                    vec!["display fixtures".into()],
                ],
                description_forbidden: vec!["$1,200.00".into()],
                subject_terms: vec!["installation".into()],
                expected_readiness: Some("ready".into()),
                evidence: GoldEvidence {
                    date_text: vec!["March 4, 2026".into()],
                    party_text: [(
                        "Halvorsen Fixture Works LLC".to_owned(),
                        vec!["HALVORSEN FIXTURE WORKS, LLC".to_owned()],
                    )]
                    .into(),
                },
                ..GoldAnswer::default()
            },
            ..GoldDocument::default()
        }
    }

    fn halvorsen() -> Vec<String> {
        vec!["Halvorsen Fixture Works LLC".into()]
    }

    fn outcome<'a>(parties: &'a [String], relation: &'a str, filename: &'a str) -> Outcome<'a> {
        Outcome {
            analysed: true,
            filename: Some(filename),
            description: "Invoice from Halvorsen Fixture Works LLC for display fixtures totalling $4,812.50, issued March 4, 2026.",
            document_type: Some("Invoice"),
            document_date: Some("2026-03-04"),
            date_role: Some("invoice"),
            parties,
            party_relation: Some(relation),
            ready: true,
            evidence: None,
        }
    }

    #[test]
    fn gold_filenames_compose_every_acceptable_answer_with_the_reviewed_one_first() {
        let names = gold_filenames(&invoice());
        assert_eq!(
            names[0],
            "2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf"
        );
        assert!(names.contains(
            &"2026-03-04 Invoice between Halvorsen Fixture Works LLC and Quillon Ridge Bakery.pdf"
                .to_owned()
        ));
        let mut undated = invoice();
        undated.gold.document_date = None;
        assert!(
            gold_filenames(&undated).is_empty(),
            "no date, no reviewed name"
        );

        let mut more = invoice();
        more.gold.acceptable_types = vec!["Commercial Invoice".into()];
        more.gold.acceptable_dates = vec!["2026-03-05".into()];
        let names = gold_filenames(&more);
        // Two types, two dates, and the "from" set plus the two-party
        // "between" set in either order.
        assert_eq!(names.len(), 2 * 2 * 3);
        assert!(names.contains(
            &"2026-03-05 Commercial Invoice from Halvorsen Fixture Works LLC.pdf".to_owned()
        ));
    }

    #[test]
    fn a_name_matching_an_acceptable_set_is_correct_and_compared_as_windows_does() {
        let document = invoice();
        let parties = halvorsen();
        let right = score(
            &document,
            &outcome(
                &parties,
                "from",
                "2026-03-04 INVOICE FROM HALVORSEN FIXTURE WORKS LLC.PDF",
            ),
            &Texts::default(),
        );
        assert_eq!(right.scores["filename_correct"], json!(true));
        assert_eq!(right.scores["parties_correct"], json!(true));
        assert_eq!(right.scores["relation_correct"], json!(true));
        assert_eq!(right.scores["unsafe_ready"], json!(false));

        let both = vec![
            "Halvorsen Fixture Works LLC".to_owned(),
            "Quillon Ridge Bakery".to_owned(),
        ];
        let between = score(
            &document,
            &outcome(
                &both,
                "between",
                "2026-03-04 Invoice between Halvorsen Fixture Works LLC and Quillon Ridge Bakery.pdf",
            ),
            &Texts::default(),
        );
        assert_eq!(between.scores["filename_correct"], json!(true));
        assert_eq!(
            between.scores["parties_correct"],
            json!(true),
            "an acceptable set"
        );
        assert_eq!(between.scores["relation_correct"], json!(true));
        assert_eq!(
            between.scores["party_forbidden"],
            json!(false),
            "a forbidden party an acceptable set names is not a trap sprung"
        );

        let wrong = score(
            &document,
            &outcome(
                &parties,
                "to",
                "2026-03-04 Invoice to Halvorsen Fixture Works LLC.pdf",
            ),
            &Texts::default(),
        );
        assert_eq!(wrong.scores["filename_correct"], json!(false));
        assert_eq!(wrong.scores["relation_correct"], json!(false));
        assert_eq!(wrong.scores["unsafe_ready"], json!(true));
    }

    #[test]
    fn party_role_judges_the_role_the_connecting_word_claims() {
        let document = invoice();
        let issuer = halvorsen();
        let customer = vec!["Quillon Ridge Bakery".to_owned()];
        let both = vec![
            "Quillon Ridge Bakery".to_owned(),
            "Halvorsen Fixture Works LLC".to_owned(),
        ];
        let role = |parties: &[String], relation: &str| {
            score(
                &document,
                &outcome(parties, relation, "x.pdf"),
                &Texts::default(),
            )
            .scores
            .get("party_role_correct")
            .cloned()
        };
        // The issuer, from: right. To the issuer claims a recipient, and
        // the gold names none, so there is nothing to judge.
        assert_eq!(role(&issuer, "from"), Some(json!(true)));
        assert_eq!(role(&issuer, "to"), None);
        // The bill-to customer in the issuer's place: wrong, and a trap.
        assert_eq!(role(&customer, "from"), Some(json!(false)));
        let trapped = score(
            &document,
            &outcome(&customer, "from", "x.pdf"),
            &Texts::default(),
        );
        assert_eq!(trapped.scores["party_forbidden"], json!(true));
        assert_eq!(trapped.traps, vec!["party Quillon Ridge Bakery"]);
        // Both, matching the acceptable `between` set: both role holders,
        // in either order.
        assert_eq!(role(&both, "between"), Some(json!(true)));
        // Nobody named: nothing to judge.
        assert_eq!(role(&[], "none"), None);
        // Between needs two distinct holders: one party listed under two
        // roles is one holder, and the other must still be named.
        let mut doubled = invoice();
        doubled.gold.party_roles.push(PartyRole {
            name: "Halvorsen Fixture Works LLC".into(),
            role: "seller".into(),
        });
        let scored = score(
            &doubled,
            &outcome(&both, "between", "x.pdf"),
            &Texts::default(),
        );
        assert_eq!(scored.scores["party_role_correct"], json!(true));
        doubled
            .gold
            .party_roles
            .retain(|role| role.name.starts_with("Halvorsen"));
        let scored = score(
            &doubled,
            &outcome(&both, "between", "x.pdf"),
            &Texts::default(),
        );
        assert!(
            !scored.scores.contains_key("party_role_correct"),
            "one holder"
        );

        let mut notice = invoice();
        notice.gold.party_relation = Some("for".into());
        notice.gold.acceptable_party_sets.clear();
        notice.gold.party_roles = vec![PartyRole {
            name: "Halvorsen Fixture Works LLC".into(),
            role: "subject".into(),
        }];
        let scored = score(
            &notice,
            &outcome(&issuer, "for", "x.pdf"),
            &Texts::default(),
        );
        assert_eq!(scored.scores["party_role_correct"], json!(true));
        let mut unlabelled = notice.clone();
        unlabelled.gold.party_roles[0].role = "tenant".into();
        let scored = score(
            &unlabelled,
            &outcome(&issuer, "for", "x.pdf"),
            &Texts::default(),
        );
        assert!(
            !scored.scores.contains_key("party_role_correct"),
            "no holder of the role"
        );
    }

    /// A rent notice is for its tenant or from its landlord. Naming the
    /// landlord with "for" names an acceptable party under the wrong word:
    /// the parties score, the relation and the role do not.
    #[test]
    fn an_acceptable_party_under_the_wrong_word_is_caught() {
        let notice = GoldDocument {
            id: "notice".into(),
            file: "notice.pdf".into(),
            gold: GoldAnswer {
                document_type: Some("Notice of Rent Increase".into()),
                document_date: Some("2026-05-12".into()),
                parties: Some(vec!["Imogen Castellanos".into()]),
                party_relation: Some("for".into()),
                acceptable_party_sets: vec![
                    PartySet {
                        parties: vec!["Imogen Castellanos".into()],
                        relation: Some("to".into()),
                    },
                    PartySet {
                        parties: vec!["Cresthaven Court Holdings LLC".into()],
                        relation: Some("from".into()),
                    },
                ],
                party_roles: vec![
                    PartyRole {
                        name: "Imogen Castellanos".into(),
                        role: "subject".into(),
                    },
                    PartyRole {
                        name: "Imogen Castellanos".into(),
                        role: "recipient".into(),
                    },
                    PartyRole {
                        name: "Cresthaven Court Holdings LLC".into(),
                        role: "issuer".into(),
                    },
                ],
                ..GoldAnswer::default()
            },
            ..GoldDocument::default()
        };
        let judge = |party: &str, relation: &str| {
            let parties = vec![party.to_owned()];
            let name = format!("2026-05-12 Notice of Rent Increase {relation} {party}.pdf");
            let mut outcome = outcome(&parties, relation, &name);
            outcome.document_type = Some("Notice of Rent Increase");
            outcome.document_date = Some("2026-05-12");
            let scores = score(&notice, &outcome, &Texts::default()).scores;
            [
                "parties_correct",
                "relation_correct",
                "party_role_correct",
                "filename_correct",
            ]
            .map(|key| scores[key].as_bool().unwrap())
        };
        assert_eq!(judge("Imogen Castellanos", "for"), [true; 4]);
        assert_eq!(judge("Imogen Castellanos", "to"), [true; 4]);
        assert_eq!(judge("Cresthaven Court Holdings LLC", "from"), [true; 4]);
        assert_eq!(
            judge("Cresthaven Court Holdings LLC", "for"),
            [true, false, false, false]
        );
    }

    #[test]
    fn traps_dates_and_readiness_are_scored_where_the_gold_defines_them() {
        let document = invoice();
        let parties = halvorsen();
        let mut trapped = outcome(
            &parties,
            "from",
            "2026-04-03 Invoice from Halvorsen Fixture Works LLC.pdf",
        );
        trapped.document_date = Some("2026-04-03");
        let scored = score(&document, &trapped, &Texts::default());
        assert_eq!(scored.scores["date_correct"], json!(false));
        assert_eq!(scored.scores["date_forbidden"], json!(true));
        assert_eq!(scored.traps, vec!["date 2026-04-03"]);
        assert_eq!(scored.scores["date_present"], json!(true));
        assert_eq!(scored.scores["readiness_match"], json!(true));
        assert_eq!(scored.scores["unsafe_ready"], json!(true));

        let mut either = invoice();
        either.gold.expected_readiness = Some("either".into());
        let scored = score(&either, &trapped, &Texts::default());
        assert!(!scored.scores.contains_key("readiness_match"));
        assert!(!scored.scores.contains_key("needless_review"));

        let mut reviewed = outcome(
            &parties,
            "from",
            "2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf",
        );
        reviewed.ready = false;
        let scored = score(&document, &reviewed, &Texts::default());
        assert_eq!(scored.scores["needless_review"], json!(true));
        assert_eq!(scored.scores["unsafe_ready"], json!(false));
    }

    /// A failed document is scored on exactly the keys the reviewed answer
    /// is scored on, every one a miss; it springs no trap and asserts
    /// nothing.
    #[test]
    fn a_failed_document_misses_every_defined_answer_and_asserts_nothing() {
        let document = invoice();
        let failed = score(&document, &Outcome::default(), &Texts::default());
        for key in [
            "filename_correct",
            "type_correct",
            "type_present",
            "date_correct",
            "date_exact",
            "date_present",
            "date_role_correct",
            "parties_correct",
            "relation_correct",
            "party_role_correct",
            "readiness_match",
            "description_complete",
            "description_specific",
            "ready",
        ] {
            assert_eq!(failed.scores[key], json!(false), "{key}");
        }
        for key in [
            "date_forbidden",
            "party_forbidden",
            "unsafe_ready",
            "needless_review",
        ] {
            assert_eq!(failed.scores[key], json!(false), "{key}: no trap sprung");
        }
        assert_eq!(failed.scores["description_completeness"], json!(0.0));
        assert_eq!(failed.scores["description_specificity"], json!(0.0));
        assert_eq!(failed.scores["evidence_recall"], json!(0.0));
        assert_eq!(failed.scores["parties_matched"], json!(0));
        assert_eq!(failed.scores["parties_expected"], json!(1));
        assert!(!failed.scores.contains_key("description_factual"));
        assert!(!failed.scores.contains_key("description_claims"));

        // The keys are the reviewed answer's own, and each is a miss.
        let parties = halvorsen();
        let reviewed = score(
            &document,
            &outcome(
                &parties,
                "from",
                "2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf",
            ),
            &Texts::default(),
        );
        let boolean_keys = |scored: &Scored| {
            scored
                .scores
                .iter()
                .filter(|(_, value)| value.is_boolean())
                .map(|(key, _)| key.clone())
                .filter(|key| key != "description_factual")
                .collect::<Vec<_>>()
        };
        assert_eq!(boolean_keys(&failed), boolean_keys(&reviewed));
        for key in boolean_keys(&reviewed) {
            if !bad_when_true(&key) && key != "ready" {
                assert_eq!(reviewed.scores[&key], json!(true), "{key}");
            }
        }
    }

    /// The two answers a failed document could be mistaken as getting
    /// right: naming nobody where the gold names nobody, and not being
    /// ready where the gold wants review.
    #[test]
    fn a_failed_document_is_not_right_by_naming_nobody_or_by_not_being_ready() {
        let mut nobody = invoice();
        nobody.gold.parties = Some(Vec::new());
        nobody.gold.party_relation = Some("none".into());
        nobody.gold.acceptable_party_sets = vec![PartySet {
            parties: Vec::new(),
            relation: Some("none".into()),
        }];
        nobody.gold.expected_readiness = Some("needs_review".into());
        let failed = score(&nobody, &Outcome::default(), &Texts::default());
        assert_eq!(failed.scores["parties_correct"], json!(false));
        assert_eq!(failed.scores["parties_expected"], json!(0));
        assert_eq!(failed.scores["readiness_match"], json!(false));
        assert!(
            !failed.scores.contains_key("relation_correct"),
            "the reviewed answer names nobody, so no relation word is judged"
        );

        // A completed outcome that names nobody and asks for review is
        // right on both.
        let mut answered = outcome(&[], "none", "x.pdf");
        answered.ready = false;
        let right = score(&nobody, &answered, &Texts::default());
        assert_eq!(right.scores["parties_correct"], json!(true));
        assert_eq!(right.scores["readiness_match"], json!(true));
    }

    /// The gold's role is the reviewed date's: it is judged only when that
    /// date was chosen.
    #[test]
    fn the_date_role_is_judged_only_for_the_reviewed_date() {
        let mut document = invoice();
        document.gold.date_role = Some("effective".into());
        document.gold.acceptable_dates = vec!["2026-02-18".into()];
        let parties = halvorsen();
        let role = |date: &str, role: &str| {
            let mut answer = outcome(&parties, "from", "x.pdf");
            answer.document_date = Some(date);
            answer.date_role = Some(role);
            score(&document, &answer, &Texts::default())
                .scores
                .get("date_role_correct")
                .cloned()
        };
        assert_eq!(role("2026-03-04", "effective"), Some(json!(true)));
        assert_eq!(role("2026-03-04", "issuance"), Some(json!(false)));
        // The acceptable date with its own role, and the trap date with the
        // reviewed role: neither is judged against the reviewed date's role.
        assert_eq!(role("2026-02-18", "issuance"), None);
        assert_eq!(role("2026-04-03", "effective"), None);
    }

    /// "between" names two sides in no particular order.
    #[test]
    fn a_two_party_between_name_is_right_in_either_order() {
        let mut agreement = invoice();
        agreement.gold.document_type = Some("Master Services Agreement".into());
        agreement.gold.parties = Some(vec![
            "Cedarmark Cloud Services Inc.".into(),
            "Pinehollow Credit Union".into(),
        ]);
        agreement.gold.party_relation = Some("between".into());
        agreement.gold.acceptable_party_sets.clear();
        agreement.gold.forbidden_parties.clear();
        agreement.gold.party_roles = vec![
            PartyRole {
                name: "Cedarmark Cloud Services Inc.".into(),
                role: "counterparty".into(),
            },
            PartyRole {
                name: "Pinehollow Credit Union".into(),
                role: "counterparty".into(),
            },
        ];
        let names = gold_filenames(&agreement);
        assert_eq!(
            names,
            vec![
                "2026-03-04 Master Services Agreement between Cedarmark Cloud Services Inc and Pinehollow Credit Union.pdf",
                "2026-03-04 Master Services Agreement between Pinehollow Credit Union and Cedarmark Cloud Services Inc.pdf",
            ],
            "the reviewed order first"
        );
        let reversed = vec![
            "Pinehollow Credit Union".to_owned(),
            "Cedarmark Cloud Services Inc.".to_owned(),
        ];
        let mut answer = outcome(&reversed, "between", &names[1]);
        answer.document_type = Some("Master Services Agreement");
        let scores = score(&agreement, &answer, &Texts::default()).scores;
        for key in [
            "filename_correct",
            "parties_correct",
            "relation_correct",
            "party_role_correct",
        ] {
            assert_eq!(scores[key], json!(true), "{key}");
        }
        assert_eq!(scores["unsafe_ready"], json!(false));
        // A one-sided relation keeps its order: only "between" is
        // symmetric.
        assert_eq!(gold_filenames(&invoice()).len(), 3);
    }

    #[test]
    fn description_scores_count_facts_claims_and_specificity() {
        let document = invoice();
        let text = DocumentText::new([
            "HALVORSEN FIXTURE WORKS, LLC\nINVOICE\nInvoice Date: March 4, 2026\nDue Date: April 3, 2026\n\
             Bill To: Quillon Ridge Bakery\nDisplay fixtures and installation 4,812.50",
        ]);
        let texts = Texts {
            document: Some(&text),
            ..Texts::default()
        };
        let parties = halvorsen();
        let good = score(&document, &outcome(&parties, "from", "x.pdf"), &texts);
        assert_eq!(good.scores["description_completeness"], json!(1.0));
        assert_eq!(good.scores["description_complete"], json!(true));
        assert_eq!(
            good.scores["description_unsupported"],
            json!(0),
            "{:?}",
            good.claims
        );
        assert_eq!(good.scores["description_factual"], json!(true));
        assert_eq!(good.scores["description_specificity"], json!(1.0));
        assert_eq!(good.scores["description_specific"], json!(true));

        let mut careless = outcome(&parties, "from", "x.pdf");
        careless.description = "Invoice for $1,200.00 from Brightwater Tooling.";
        let bad = score(&document, &careless, &texts);
        assert_eq!(bad.scores["description_completeness"], json!(0.0));
        assert_eq!(bad.scores["description_unsupported"], json!(2));
        assert_eq!(bad.forbidden_description, vec!["$1,200.00"]);
        assert_eq!(bad.scores["description_factual"], json!(false));
        // A concrete detail, but no gold party and only six words.
        assert_eq!(bad.scores["description_specificity"], json!(0.3333));
        assert_eq!(bad.scores["description_specific"], json!(false));
    }

    #[test]
    fn evidence_and_digest_recall_count_the_date_once_and_each_party() {
        let document = invoice();
        let parties = halvorsen();
        let evidence = Evidence {
            date: Some("Invoice Date: March 4, 2026".into()),
            document_type: Some("INVOICE".into()),
            parties: vec!["Bill To: Quillon Ridge Bakery".into()],
        };
        let mut quoted = outcome(&parties, "from", "x.pdf");
        quoted.evidence = Some(&evidence);
        let digest = "INVOICE\nHalvorsen Fixture Works, LLC\nInvoice Date: 2026-03-04";
        let prompt = "Document:\nHALVORSEN FIXTURE WORKS, LLC\nInvoice Date: March 4, 2026";
        let scored = score(
            &document,
            &quoted,
            &Texts {
                digest: Some(digest),
                prompt: Some(prompt),
                ..Texts::default()
            },
        );
        assert_eq!(
            scored.scores["evidence_recall"],
            json!(0.5),
            "the date, not the issuer"
        );
        // The digest has the issuer in its verbatim form but the date only
        // in a form the gold did not list.
        assert_eq!(scored.scores["digest_recall"], json!(0.5));
        assert_eq!(scored.scores["prompt_recall"], json!(1.0));
    }

    #[test]
    fn the_evaluators_matching_rules_carry_over() {
        assert!(type_matches("Statement of Work", "Statement of Work No. 4"));
        assert!(!type_matches(
            "Notice of Termination",
            "Employment Termination"
        ));
        assert!(!type_matches("Settlement Agreement", "Agreement"));
        assert!(party_matches(
            "Halvorsen Fixture Works, LLC",
            "Halvorsen Fixture Works LLC"
        ));
        assert!(!party_matches("Marta Quillon", "Quillon Ridge Bakery"));
        assert!(bad_when_true("date_forbidden") && bad_when_true("unsafe_ready"));
        assert!(!bad_when_true("date_correct"));
    }
}
