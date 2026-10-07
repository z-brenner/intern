//! Whether the gold's evidence reaches the context evidence retrieval
//! builds, beside the digest the engine sends today - measured from a
//! document's extracted text alone, with no model.
//!
//! An item is the gold's type (a context unit holds at least 60% of its
//! significant words, or of an acceptable type's), its date (a reviewed
//! form of it, or the date or an acceptable one in any spelling), each of
//! its parties (any reviewed form), each description fact (any form), and
//! each subject term. The context is searched unit by unit; the digest as
//! the text it is, which is how `digest_recall` reads it. The scores:
//!
//! * `context_type_recall`, `context_date_recall`, `context_party_recall`,
//!   and `context_recall`, their mean - the filename's items;
//! * `context_fact_recall` - the description facts, the most a description
//!   built from the context could say;
//! * `context_subject_recall` - the subject terms.
//!
//! `context_tokens`, `context_units`, `index_units`, `index_ms` and
//! `retrieval_ms` go with the timings. [`retrieval_command`] sweeps
//! retrieval configurations over a recording's sources and reports, per
//! configuration, recall per field against the digest's, tokens by page
//! bucket, and every document that lost an item - on a tuning half of the
//! corpus and a held-out half, split by a hash of the document id.

use std::{
    collections::BTreeMap,
    fmt::Write as _,
    path::{Path, PathBuf},
    time::Instant,
};

use intern_engine::{
    DigestBudget, DocumentSource, distill, estimated_prompt_tokens,
    evidence::{date_match_positions, normalize},
    index::EvidenceIndex,
    retrieve::{
        EXPANSION_BIT, EvidenceContext, Field, FieldBudgets, IdStyle, RetrievalConfig, Strategy,
        TierPolicy, WHOLE_BIT, Weights, field_scores, retrieve,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    gold::{GoldAnswer, GoldDocument, GoldFile},
    recording::{RecordedExtraction, Recording, sha256_hex},
    score::digest_recall,
    stats::{Distribution, round},
};

/// The context scores, in the order reports list them.
pub const CONTEXT_SCORES: [&str; 6] = [
    "context_type_recall",
    "context_date_recall",
    "context_party_recall",
    "context_recall",
    "context_fact_recall",
    "context_subject_recall",
];

/// Words of a type that say nothing about it.
const GENERIC_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "between", "by", "for", "from", "in", "of", "on", "the", "this",
    "to", "with", "it", "its", "their",
];

/// How much of each kind of item a text carries. `None` where the gold
/// defines no such item.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Recall {
    pub document_type: Option<f64>,
    pub date: Option<f64>,
    pub parties: Option<f64>,
    pub facts: Option<f64>,
    pub subject: Option<f64>,
    /// The items it lacks: `type`, `date`, `party:<name>`, `fact:<form>`,
    /// `subject:<term>`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
}

impl Recall {
    /// The mean of the type, date and party recall: the filename's items.
    pub fn filename(&self) -> Option<f64> {
        let present = [self.document_type, self.date, self.parties]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        (!present.is_empty()).then(|| present.iter().sum::<f64>() / present.len() as f64)
    }

    pub fn get(&self, item: Item) -> Option<f64> {
        match item {
            Item::Type => self.document_type,
            Item::Date => self.date,
            Item::Parties => self.parties,
            Item::Filename => self.filename(),
            Item::Facts => self.facts,
            Item::Subject => self.subject,
        }
    }
}

/// The kinds of item recall is reported for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Item {
    Type,
    Date,
    Parties,
    Filename,
    Facts,
    Subject,
}

impl Item {
    pub const ALL: [Self; 6] = [
        Self::Type,
        Self::Date,
        Self::Parties,
        Self::Filename,
        Self::Facts,
        Self::Subject,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Date => "date",
            Self::Parties => "parties",
            Self::Filename => "filename",
            Self::Facts => "facts",
            Self::Subject => "subject",
        }
    }
}

/// Where items are looked for: texts any item may be in, and the texts a
/// type's words must be found together in.
pub struct Haystack {
    texts: Vec<String>,
    type_texts: Vec<String>,
}

impl Haystack {
    /// A context, unit by unit.
    ///
    /// Two units that stand next to each other in the document and both in
    /// the context also count as one text: a scanned letter's line is its
    /// own block, and "Lead Veterinary" / "Technician" on consecutive lines
    /// read as one phrase to the model as they do on the page.
    pub fn context(index: &EvidenceIndex, context: &EvidenceContext) -> Self {
        let units = index.units();
        let type_texts = context
            .units
            .iter()
            .map(|ordinal| units[*ordinal as usize].normalized.clone())
            .collect::<Vec<_>>();
        let mut texts = type_texts.clone();
        for pair in context.units.windows(2) {
            let (first, second) = (&units[pair[0] as usize], &units[pair[1] as usize]);
            if pair[1] == pair[0] + 1 && first.page == second.page {
                texts.push(format!("{} {}", first.normalized, second.normalized));
            }
        }
        Self { texts, type_texts }
    }

    /// A digest: its text whole, and for a type each kept block and
    /// heading.
    pub fn digest(digest: &intern_engine::DocumentDigest) -> Self {
        Self {
            texts: vec![normalize(&digest.text)],
            type_texts: digest
                .segments
                .iter()
                .chain(&digest.outline)
                .map(|text| normalize(text))
                .collect(),
        }
    }

    fn empty() -> Self {
        Self {
            texts: Vec::new(),
            type_texts: Vec::new(),
        }
    }

    fn holds(&self, form: &str) -> bool {
        let form = normalize(form);
        !form.is_empty() && self.texts.iter().any(|text| text.contains(&form))
    }

    fn states(&self, iso: &str) -> bool {
        self.texts
            .iter()
            .any(|text| !date_match_positions(iso, text).is_empty())
    }
}

/// Whether `word` stands in `text` with no letter or digit running into it.
fn has_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(at, _)| {
        !text[..at]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric)
            && !text[at + word.len()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
    })
}

fn type_found(haystack: &Haystack, document_type: &str) -> bool {
    let words = normalize(document_type);
    let significant = words
        .split_whitespace()
        .filter(|word| word.len() > 2 && !GENERIC_WORDS.contains(word))
        .collect::<Vec<_>>();
    if significant.is_empty() {
        return false;
    }
    haystack.type_texts.iter().any(|text| {
        let found = significant
            .iter()
            .filter(|word| has_word(text, word))
            .count();
        found * 10 >= significant.len() * 6
    })
}

/// How much of the gold `haystack` carries.
pub fn recall(gold: &GoldAnswer, haystack: &Haystack) -> Recall {
    let mut recall = Recall::default();
    let fraction = |found: usize, total: usize| found as f64 / total as f64;

    if let Some(document_type) = &gold.document_type {
        let found = std::iter::once(document_type)
            .chain(&gold.acceptable_types)
            .any(|candidate| type_found(haystack, candidate));
        recall.document_type = Some(if found { 1.0 } else { 0.0 });
        if !found {
            recall.missing.push("type".to_owned());
        }
    }
    let dates = gold
        .document_date
        .iter()
        .chain(&gold.acceptable_dates)
        .collect::<Vec<_>>();
    if !gold.evidence.date_text.is_empty() || !dates.is_empty() {
        let found = gold
            .evidence
            .date_text
            .iter()
            .any(|form| haystack.holds(form))
            || dates.iter().any(|date| haystack.states(date));
        recall.date = Some(if found { 1.0 } else { 0.0 });
        if !found {
            recall.missing.push("date".to_owned());
        }
    }
    if let Some(parties) = gold.parties.as_ref().filter(|parties| !parties.is_empty()) {
        let mut found = 0;
        for party in parties {
            let forms = gold
                .evidence
                .party_text
                .get(party)
                .filter(|forms| !forms.is_empty())
                .cloned()
                .unwrap_or_else(|| vec![party.clone()]);
            if forms.iter().any(|form| haystack.holds(form)) {
                found += 1;
            } else {
                recall.missing.push(format!("party:{party}"));
            }
        }
        recall.parties = Some(fraction(found, parties.len()));
    }
    if !gold.description_facts.is_empty() {
        let mut found = 0;
        for forms in &gold.description_facts {
            if forms.iter().any(|form| haystack.holds(form)) {
                found += 1;
            } else if let Some(first) = forms.first() {
                recall.missing.push(format!("fact:{first}"));
            }
        }
        recall.facts = Some(fraction(found, gold.description_facts.len()));
    }
    if !gold.subject_terms.is_empty() {
        let mut found = 0;
        for term in &gold.subject_terms {
            if haystack.holds(term) {
                found += 1;
            } else {
                recall.missing.push(format!("subject:{term}"));
            }
        }
        recall.subject = Some(fraction(found, gold.subject_terms.len()));
    }
    recall
}

/// A context built and measured.
pub struct Measured {
    pub recall: Recall,
    pub context: EvidenceContext,
    pub index_units: usize,
    pub index_micros: u64,
    pub retrieval_micros: u64,
}

/// Builds the index and the context for `source` and measures them.
pub fn measure(gold: &GoldAnswer, source: &DocumentSource, config: &RetrievalConfig) -> Measured {
    let started = Instant::now();
    let index = EvidenceIndex::build_with(source, config.index_options());
    let index_micros = started.elapsed().as_micros() as u64;
    measure_with(gold, &index, index_micros, config)
}

fn measure_with(
    gold: &GoldAnswer,
    index: &EvidenceIndex,
    index_micros: u64,
    config: &RetrievalConfig,
) -> Measured {
    let started = Instant::now();
    let context = retrieve(index, config, 100);
    let retrieval_micros = started.elapsed().as_micros() as u64;
    Measured {
        recall: recall(gold, &Haystack::context(index, &context)),
        index_units: index.units().len(),
        index_micros,
        retrieval_micros,
        context,
    }
}

/// The context scores and counts of one document, as a record carries them.
#[derive(Clone, Debug, Default)]
pub struct ContextScores {
    pub scores: BTreeMap<String, Value>,
    pub timings: BTreeMap<String, Value>,
}

/// The context scores for a record. A document that was not read carries
/// none of the gold's items, as `digest_recall` scores it.
pub fn context_scores(
    document: &GoldDocument,
    source: Option<&DocumentSource>,
    config: &RetrievalConfig,
) -> ContextScores {
    let mut out = ContextScores::default();
    let (recall, counts) = match source {
        Some(source) => {
            let measured = measure(&document.gold, source, config);
            let counts = Some((
                measured.context.estimated_tokens,
                measured.context.units.len(),
                measured.index_units,
                measured.index_micros,
                measured.retrieval_micros,
            ));
            (measured.recall, counts)
        }
        None => (recall(&document.gold, &Haystack::empty()), None),
    };
    let values = [
        recall.document_type,
        recall.date,
        recall.parties,
        recall.filename(),
        recall.facts,
        recall.subject,
    ];
    for (key, value) in CONTEXT_SCORES.iter().zip(values) {
        if let Some(value) = value {
            out.scores.insert((*key).to_owned(), json!(round(value, 4)));
        }
    }
    if let Some((tokens, units, index_units, index_micros, retrieval_micros)) = counts {
        out.timings
            .insert("context_tokens".to_owned(), json!(tokens));
        out.timings.insert("context_units".to_owned(), json!(units));
        out.timings
            .insert("index_units".to_owned(), json!(index_units));
        out.timings.insert(
            "index_ms".to_owned(),
            json!(round(index_micros as f64 / 1000.0, 3)),
        );
        out.timings.insert(
            "retrieval_ms".to_owned(),
            json!(round(retrieval_micros as f64 / 1000.0, 3)),
        );
    }
    out
}

/// Which half of the corpus a document tunes on: the parity of the first
/// byte of its id's SHA-256. Fixed by the id alone, so adding documents
/// never moves one.
pub fn split_of(id: &str) -> &'static str {
    let digest = sha256_hex(id.as_bytes());
    let first = u8::from_str_radix(&digest[..2], 16).unwrap_or(0);
    if first % 2 == 0 { "tune" } else { "holdout" }
}

/// The page buckets the retrieval report groups tokens by.
pub fn bucket(pages: usize) -> &'static str {
    match pages {
        0..=3 => "1-3",
        4..=9 => "4-9",
        10..=24 => "10-24",
        _ => "25+",
    }
}

pub const BUCKETS: [&str; 4] = ["1-3", "4-9", "10-24", "25+"];

/// One document of a corpus the sweep reads: its gold and its text.
pub struct CorpusDocument {
    pub gold: GoldDocument,
    pub source: DocumentSource,
}

/// One document under one configuration.
#[derive(Clone, Debug, Serialize)]
pub struct DocumentResult {
    pub id: String,
    pub corpus: String,
    pub pages: usize,
    pub split: &'static str,
    pub context: Recall,
    pub digest: Recall,
    /// The score `digest_recall` gives (date and parties in the digest).
    pub digest_recall: Option<f64>,
    pub context_tokens: usize,
    pub digest_tokens: usize,
    pub context_units: usize,
    pub index_units: usize,
    pub tier: String,
    pub hierarchical: bool,
    pub index_ms: f64,
    pub retrieval_ms: f64,
}

impl DocumentResult {
    /// The filename's items the digest carried whole and the context does
    /// not: the acceptance gate.
    pub fn violates(&self) -> bool {
        self.digest_recall == Some(1.0)
            && [
                self.context.document_type,
                self.context.date,
                self.context.parties,
            ]
            .iter()
            .any(|value| value.is_some_and(|value| value < 1.0))
    }

    /// Items of one kind the context carries less of than the digest.
    pub fn loses(&self, item: Item) -> bool {
        match (self.context.get(item), self.digest.get(item)) {
            (Some(context), Some(digest)) => context < digest,
            _ => false,
        }
    }
}

/// Aggregates over a set of documents.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Summary {
    pub documents: usize,
    pub violations: usize,
    /// Mean recall per item kind: (context, digest).
    pub recall: BTreeMap<String, (f64, f64)>,
    /// Documents whose context carries less of an item kind than the digest.
    pub losses: BTreeMap<String, usize>,
    pub context_tokens: Option<Distribution>,
    pub digest_tokens: Option<Distribution>,
    /// Per page bucket: documents, context p50, digest p50, the context's
    /// tokens over the digest's summed.
    pub buckets: BTreeMap<String, BucketSummary>,
    /// 1 - context tokens / digest tokens, summed over documents of ten
    /// pages or more.
    pub long_reduction: Option<f64>,
    /// The smallest reduction any one document of ten pages or more got.
    pub long_worst_reduction: Option<f64>,
    /// Documents of ten pages or more whose context is not 40% smaller
    /// than their digest ([`short_of_reduction`]).
    pub long_short: usize,
    pub index_ms: Option<Distribution>,
    pub retrieval_ms: Option<Distribution>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct BucketSummary {
    pub documents: usize,
    pub context_p50: f64,
    pub digest_p50: f64,
    pub ratio: f64,
}

pub fn summarize<'a>(results: impl IntoIterator<Item = &'a DocumentResult>) -> Summary {
    let results = results.into_iter().collect::<Vec<_>>();
    let mut summary = Summary {
        documents: results.len(),
        violations: results.iter().filter(|result| result.violates()).count(),
        ..Summary::default()
    };
    for item in Item::ALL {
        let pairs = results
            .iter()
            .filter_map(|result| Some((result.context.get(item)?, result.digest.get(item)?)))
            .collect::<Vec<_>>();
        if !pairs.is_empty() {
            let count = pairs.len() as f64;
            summary.recall.insert(
                item.as_str().to_owned(),
                (
                    round(pairs.iter().map(|pair| pair.0).sum::<f64>() / count, 4),
                    round(pairs.iter().map(|pair| pair.1).sum::<f64>() / count, 4),
                ),
            );
        }
        summary.losses.insert(
            item.as_str().to_owned(),
            results.iter().filter(|result| result.loses(item)).count(),
        );
    }
    let tokens = |pick: fn(&DocumentResult) -> usize, results: &[&DocumentResult]| {
        Distribution::of(
            &results
                .iter()
                .map(|result| pick(result) as f64)
                .collect::<Vec<_>>(),
        )
    };
    summary.context_tokens = tokens(|result| result.context_tokens, &results);
    summary.digest_tokens = tokens(|result| result.digest_tokens, &results);
    for name in BUCKETS {
        let members = results
            .iter()
            .copied()
            .filter(|result| bucket(result.pages) == name)
            .collect::<Vec<_>>();
        if members.is_empty() {
            continue;
        }
        let context: usize = members.iter().map(|result| result.context_tokens).sum();
        let digest: usize = members.iter().map(|result| result.digest_tokens).sum();
        summary.buckets.insert(
            name.to_owned(),
            BucketSummary {
                documents: members.len(),
                context_p50: tokens(|result| result.context_tokens, &members)
                    .map_or(0.0, |distribution| distribution.p50),
                digest_p50: tokens(|result| result.digest_tokens, &members)
                    .map_or(0.0, |distribution| distribution.p50),
                ratio: round(context as f64 / digest.max(1) as f64, 3),
            },
        );
    }
    let long = results
        .iter()
        .filter(|result| result.pages >= 10)
        .collect::<Vec<_>>();
    summary.long_short = results
        .iter()
        .filter(|result| short_of_reduction(result))
        .count();
    if !long.is_empty() {
        let context: usize = long.iter().map(|result| result.context_tokens).sum();
        let digest: usize = long.iter().map(|result| result.digest_tokens).sum();
        summary.long_reduction = Some(round(1.0 - context as f64 / digest.max(1) as f64, 4));
        summary.long_worst_reduction = long
            .iter()
            .map(|result| 1.0 - result.context_tokens as f64 / result.digest_tokens.max(1) as f64)
            .min_by(f64::total_cmp)
            .map(|value| round(value, 4));
    }
    summary.index_ms = Distribution::of(
        &results
            .iter()
            .map(|result| result.index_ms)
            .collect::<Vec<_>>(),
    );
    summary.retrieval_ms = Distribution::of(
        &results
            .iter()
            .map(|result| result.retrieval_ms)
            .collect::<Vec<_>>(),
    );
    summary
}

/// One configuration over the corpus.
#[derive(Clone, Debug, Serialize)]
pub struct ConfigResult {
    pub name: String,
    pub fingerprint: String,
    pub config: RetrievalConfig,
    pub tune: Summary,
    pub holdout: Summary,
    pub all: Summary,
    pub documents: Vec<DocumentResult>,
}

impl ConfigResult {
    /// How a configuration ranks on the tuning half. First the acceptance
    /// gates: fewest violations; fact recall not below the digest's; fewest
    /// documents of ten pages or more whose context is not at least 40%
    /// smaller than their digest. Then the most filename recall, fact
    /// recall and subject recall, then the fewest tokens on long
    /// documents. Lower sorts first.
    fn rank_key(&self) -> (usize, bool, usize, i64, i64, i64, i64) {
        let recall = |item: &str| {
            self.tune
                .recall
                .get(item)
                .map_or(0, |(context, _)| (context * 10_000.0).round() as i64)
        };
        let long_tokens = self
            .documents
            .iter()
            .filter(|result| result.split == "tune" && result.pages >= 10)
            .map(|result| result.context_tokens as i64)
            .sum::<i64>();
        let facts_below_digest = self
            .tune
            .recall
            .get("facts")
            .is_some_and(|(context, digest)| context < digest);
        let long_short = self
            .documents
            .iter()
            .filter(|result| result.split == "tune")
            .filter(|result| short_of_reduction(result))
            .count();
        (
            self.tune.violations,
            facts_below_digest,
            long_short,
            -recall("filename"),
            -recall("facts"),
            -recall("subject"),
            long_tokens,
        )
    }
}

/// Whether a document of ten pages or more costs more than 60% of its
/// digest's tokens. A digest of under a hundred tokens - a long file with
/// almost no text - is too small to save on and is not held to it.
pub fn short_of_reduction(result: &DocumentResult) -> bool {
    result.pages >= 10
        && result.digest_tokens >= 100
        && result.context_tokens * 10 > result.digest_tokens * 6
}

/// Every document of `documents` under `config`, digests and indexes
/// reused from the caches.
fn run_config(
    name: &str,
    config: &RetrievalConfig,
    corpus: &[(String, CorpusDocument)],
    digests: &[(Recall, Option<f64>, usize)],
    indexes: &mut BTreeMap<(usize, usize), (EvidenceIndex, u64)>,
    dump: Option<&Path>,
) -> Result<ConfigResult, String> {
    let options = config.index_options();
    let mut documents = Vec::new();
    for (position, (corpus_name, document)) in corpus.iter().enumerate() {
        let (index, index_micros) = indexes
            .entry((position, options.max_unit_characters))
            .or_insert_with(|| {
                let started = Instant::now();
                let index = EvidenceIndex::build_with(&document.source, options);
                (index, started.elapsed().as_micros() as u64)
            });
        let measured = measure_with(&document.gold.gold, index, *index_micros, config);
        if let Some(directory) = dump {
            let directory = directory.join(name.replace('/', "-"));
            std::fs::create_dir_all(&directory)
                .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
            let path = directory.join(format!("{}.txt", document.gold.id));
            std::fs::write(&path, annotated(index, config, &measured))
                .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        }
        let (digest, digest_recall, digest_tokens) = &digests[position];
        documents.push(DocumentResult {
            id: document.gold.id.clone(),
            corpus: corpus_name.clone(),
            pages: document.source.pages.len(),
            split: split_of(&document.gold.id),
            context: measured.recall,
            digest: digest.clone(),
            digest_recall: *digest_recall,
            context_tokens: measured.context.estimated_tokens,
            digest_tokens: *digest_tokens,
            context_units: measured.context.units.len(),
            index_units: measured.index_units,
            tier: measured.context.tier.as_str().to_owned(),
            hierarchical: measured.context.hierarchical,
            index_ms: round(measured.index_micros as f64 / 1000.0, 3),
            retrieval_ms: round(measured.retrieval_micros as f64 / 1000.0, 3),
        });
    }
    Ok(ConfigResult {
        name: name.to_owned(),
        fingerprint: config.fingerprint(),
        config: config.clone(),
        tune: summarize(documents.iter().filter(|result| result.split == "tune")),
        holdout: summarize(documents.iter().filter(|result| result.split == "holdout")),
        all: summarize(&documents),
        documents,
    })
}

/// Every unit of a document with its six field scores (type, date,
/// parties, subject, identifier, key facts; `·` where it cannot carry the
/// field); the units in the context marked with the fields that chose them
/// (T D P S I K, `+` context for another unit, W whole document).
fn annotated(index: &EvidenceIndex, config: &RetrievalConfig, measured: &Measured) -> String {
    let context = &measured.context;
    let scores = Field::ALL.map(|field| field_scores(index, config, field));
    let mut out = format!(
        "tier {} hierarchical {} tokens {} units {}/{}\nmissing: {}\n\n",
        context.tier.as_str(),
        context.hierarchical,
        context.estimated_tokens,
        context.units.len(),
        measured.index_units,
        measured.recall.missing.join(", ")
    );
    let handles = context
        .handles
        .iter()
        .map(|(handle, ordinal)| (*ordinal, handle.as_str()))
        .collect::<BTreeMap<_, _>>();
    for unit in index.units() {
        let ordinal = &unit.ordinal;
        let Some(handle) = handles.get(ordinal) else {
            let _ = writeln!(
                out,
                "{:8} {} ({}) {}",
                if unit.running { "running" } else { "-" },
                unit.id,
                scores
                    .iter()
                    .map(|scores| scores.get(ordinal).map_or("·".to_owned(), i64::to_string))
                    .collect::<Vec<_>>()
                    .join(" "),
                unit.text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(120)
                    .collect::<String>()
            );
            continue;
        };
        let bits = context.selected_by.get(ordinal).copied().unwrap_or(0);
        let marks = Field::ALL
            .iter()
            .zip(["T", "D", "P", "S", "I", "K"])
            .map(|(field, mark)| if bits & field.bit() != 0 { mark } else { "." })
            .chain([
                if bits & EXPANSION_BIT != 0 { "+" } else { "." },
                if bits & WHOLE_BIT != 0 { "W" } else { "." },
            ])
            .collect::<String>();
        let _ = writeln!(
            out,
            "{marks} [{handle}] ({}) {}",
            scores
                .iter()
                .map(|scores| scores.get(ordinal).map_or("·".to_owned(), i64::to_string))
                .collect::<Vec<_>>()
                .join(" "),
            unit.text.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }
    out
}

/// The scoring components, by the names the sweep gives them.
pub const COMPONENTS: [&str; 6] = [
    "cues",
    "bm25",
    "structure",
    "heading",
    "position",
    "features",
];

fn set_weight(weights: &mut Weights, component: &str, value: u32) {
    match component {
        "cues" => weights.cues = value,
        "bm25" => weights.bm25 = value,
        "structure" => weights.structure = value,
        "heading" => weights.heading = value,
        "position" => weights.position = value,
        "features" => weights.features = value,
        _ => {}
    }
}

/// The configurations the sweep compares, each named once (a
/// configuration another name already covers is left out):
///
/// * `chosen`, [`RetrievalConfig::default`], and `plan_start`, the
///   phase plan's starting point ([`RetrievalConfig::plan_start`]);
/// * from each, every scoring component off in turn (`without_*`), each
///   alone (`only_*`), and the combinations of cue logic, BM25 and
///   structure;
/// * from `chosen`: flat against hierarchical, the hierarchical threshold
///   and sections per field, budgets, the parties' budget, the dense tier,
///   the whole-document threshold, unit size, dates per unit, expansion,
///   the tier policy and the id style;
/// * a grid over BM25's weight, the dense tier and the parties' budget;
/// * `forced/*`: the component ablations with retrieval forced on every
///   document, because most of the corpus is short enough to go whole,
///   which hides how well retrieval itself chooses.
pub fn sweep_configs() -> Vec<(String, RetrievalConfig)> {
    let chosen = RetrievalConfig::default();
    let start = RetrievalConfig::plan_start();
    let mut configs: Vec<(String, RetrievalConfig)> = vec![
        ("chosen".to_owned(), chosen.clone()),
        ("plan_start".to_owned(), start.clone()),
    ];
    let off = Weights {
        cues: 0,
        bm25: 0,
        structure: 0,
        heading: 0,
        position: 0,
        features: 0,
    };
    for (prefix, base) in [("chosen", &chosen), ("plan_start", &start)] {
        let with_weights = |weights: Weights| RetrievalConfig {
            weights,
            ..base.clone()
        };
        for name in COMPONENTS {
            let mut without = base.weights.clone();
            set_weight(&mut without, name, 0);
            configs.push((format!("{prefix}/without_{name}"), with_weights(without)));
            let mut alone = off.clone();
            set_weight(&mut alone, name, 100);
            configs.push((format!("{prefix}/only_{name}"), with_weights(alone)));
        }
        for (name, weights) in [
            (
                "cues_and_bm25",
                Weights {
                    cues: 100,
                    bm25: 100,
                    ..off.clone()
                },
            ),
            (
                "cues_bm25_structure",
                Weights {
                    cues: 100,
                    bm25: 100,
                    structure: 100,
                    ..off.clone()
                },
            ),
            ("all_components", Weights::all()),
        ] {
            configs.push((format!("{prefix}/{name}"), with_weights(weights)));
        }
    }
    let base = chosen.clone();
    let mut vary = |name: String, config: RetrievalConfig| configs.push((name, config));
    for (name, strategy) in [
        ("flat", Strategy::Flat),
        ("hierarchical", Strategy::Hierarchical),
    ] {
        vary(
            name.to_owned(),
            RetrievalConfig {
                strategy,
                ..base.clone()
            },
        );
    }
    for minimum in [100, 150, 600] {
        vary(
            format!("hierarchical_from_{minimum}"),
            RetrievalConfig {
                hierarchical_min_units: minimum,
                ..base.clone()
            },
        );
    }
    for sections in [2, 5] {
        vary(
            format!("sections_{sections}"),
            RetrievalConfig {
                strategy: Strategy::Hierarchical,
                sections_per_field: sections,
                ..base.clone()
            },
        );
    }
    for percent in [50, 75, 125, 150] {
        let scaled = |value: u32| value * percent / 100;
        let budgets = &base.budgets;
        vary(
            format!("budgets_{percent}"),
            RetrievalConfig {
                budgets: FieldBudgets {
                    document_type: scaled(budgets.document_type),
                    date: scaled(budgets.date),
                    parties: scaled(budgets.parties),
                    subject: scaled(budgets.subject),
                    identifier: scaled(budgets.identifier),
                    key_facts: scaled(budgets.key_facts),
                },
                ..base.clone()
            },
        );
    }
    for parties in [250, 300, 450] {
        vary(
            format!("parties_{parties}"),
            RetrievalConfig {
                budgets: FieldBudgets {
                    parties,
                    ..base.budgets.clone()
                },
                ..base.clone()
            },
        );
    }
    for dense in [125, 175, 200] {
        vary(
            format!("dense_{dense}"),
            RetrievalConfig {
                dense_pct: dense,
                ..base.clone()
            },
        );
    }
    vary(
        "no_dense_triggers".to_owned(),
        RetrievalConfig {
            dense_triggers: false,
            ..base.clone()
        },
    );
    for tokens in [0, 1_200, 3_000] {
        vary(
            format!("whole_{tokens}"),
            RetrievalConfig {
                whole_document_tokens: tokens,
                ..base.clone()
            },
        );
    }
    for characters in [250, 600] {
        vary(
            format!("unit_chars_{characters}"),
            RetrievalConfig {
                max_unit_chars: characters,
                ..base.clone()
            },
        );
    }
    for per_date in [1, 3] {
        vary(
            format!("units_per_date_{per_date}"),
            RetrievalConfig {
                units_per_date: per_date,
                ..base.clone()
            },
        );
    }
    for percent in [0, 15, 50] {
        vary(
            format!("expansion_{percent}"),
            RetrievalConfig {
                expansion_pct: percent,
                ..base.clone()
            },
        );
    }
    for (name, tier) in [
        ("tier_normal", TierPolicy::Normal),
        ("tier_dense", TierPolicy::Dense),
        ("tier_small", TierPolicy::Small),
    ] {
        vary(
            name.to_owned(),
            RetrievalConfig {
                tier,
                ..base.clone()
            },
        );
    }
    vary(
        "ordinal_ids".to_owned(),
        RetrievalConfig {
            id_style: IdStyle::Ordinal,
            ..base.clone()
        },
    );
    for bm25 in [0, 25, 50, 100] {
        for dense in [150, 175] {
            for parties in [350, 450] {
                vary(
                    format!("grid/bm25_{bm25}_dense_{dense}_parties_{parties}"),
                    RetrievalConfig {
                        weights: Weights {
                            bm25,
                            ..base.weights.clone()
                        },
                        dense_pct: dense,
                        budgets: FieldBudgets {
                            parties,
                            ..base.budgets.clone()
                        },
                        ..base.clone()
                    },
                );
            }
        }
    }
    let forced = configs
        .iter()
        .filter(|(name, _)| {
            name == "chosen"
                || name == "plan_start"
                || name.starts_with("chosen/")
                || matches!(
                    name.as_str(),
                    "flat" | "hierarchical" | "tier_normal" | "units_per_date_1"
                )
        })
        .map(|(name, config)| {
            (
                format!("forced/{name}"),
                RetrievalConfig {
                    whole_document_tokens: 0,
                    ..config.clone()
                },
            )
        })
        .collect::<Vec<_>>();
    configs.extend(forced);
    // Each configuration once, under the first name that reaches it.
    let mut seen = std::collections::BTreeSet::new();
    configs.retain(|(_, config)| seen.insert(config.fingerprint()));
    configs
}

/// What `intern-bench retrieval` was asked.
pub struct RetrievalOptions {
    pub recordings: Vec<PathBuf>,
    pub gold: PathBuf,
    /// The fixture corpus: its recording and its expected answers.
    pub fixtures: Option<(PathBuf, PathBuf)>,
    /// Documents to extract now rather than read from a recording: the
    /// corpus directory and the worker to read it with. For documents no
    /// recording holds yet - the gold's `pending` ones.
    pub extract: Option<(PathBuf, PathBuf)>,
    /// Configurations from files, by name.
    pub configs: Vec<PathBuf>,
    /// Run the built-in sweep as well.
    pub sweep: bool,
    pub only: Vec<String>,
    pub output: Option<PathBuf>,
    pub markdown: Option<PathBuf>,
    /// A directory to write each document's context into, one file per
    /// configuration and document, every line marked with the fields that
    /// chose it.
    pub dump: Option<PathBuf>,
}

/// Reads the sources a bench recording holds, with their gold.
pub fn load_bench_corpus(
    recording: &Path,
    gold: &GoldFile,
    only: &[String],
) -> Result<Vec<CorpusDocument>, String> {
    let (recording, _) = Recording::load(recording)?;
    let mut documents = Vec::new();
    for recorded in &recording.documents {
        if !only.is_empty() && !only.contains(&recorded.id) {
            continue;
        }
        let RecordedExtraction::Parsed { source } = &recorded.extraction else {
            continue;
        };
        let Some(document) = gold
            .documents
            .iter()
            .find(|document| document.id == recorded.id)
        else {
            continue;
        };
        documents.push(CorpusDocument {
            gold: document.clone(),
            source: source.clone(),
        });
    }
    Ok(documents)
}

/// Reads the gold's documents (those `only` names, or all) from `corpus`
/// with `worker`, as a live run would, and keeps what it extracts.
pub fn extract_corpus(
    corpus: &Path,
    worker: &Path,
    gold: &GoldFile,
    only: &[String],
) -> Result<Vec<CorpusDocument>, String> {
    let worker = intern_engine::SupervisedWorker::new(worker);
    let mut documents = Vec::new();
    for document in &gold.documents {
        if !only.is_empty() && !only.contains(&document.id) {
            continue;
        }
        let path = corpus.join(&document.file);
        eprintln!("extracting {}", document.id);
        match crate::live::extract(&worker, &format!("retrieval-{}", document.id), &path).0 {
            Ok(source) => documents.push(CorpusDocument {
                gold: document.clone(),
                source,
            }),
            Err(failure) => eprintln!("{}: extraction failed: {}", document.id, failure.code),
        }
    }
    worker.stop();
    Ok(documents)
}

#[derive(Deserialize)]
struct FixtureRecording {
    fixtures: Vec<FixtureRecorded>,
}

#[derive(Deserialize)]
struct FixtureRecorded {
    file: String,
    extraction: RecordedExtraction,
}

#[derive(Deserialize)]
struct FixtureExpectations {
    fixtures: Vec<FixtureExpected>,
}

#[derive(Deserialize)]
struct FixtureExpected {
    file: String,
    #[serde(default)]
    document_type: Option<String>,
    #[serde(default)]
    acceptable_types: Vec<String>,
    #[serde(default)]
    document_date: Option<String>,
    #[serde(default)]
    acceptable_dates: Vec<String>,
    #[serde(default)]
    parties: Option<Vec<String>>,
    #[serde(default)]
    acceptable_description_facts: Vec<String>,
}

/// Reads the fixture corpus's recorded sources with its expected answers
/// as gold: the type, the date, the parties, and each description fact.
pub fn load_fixture_corpus(
    recording: &Path,
    expected: &Path,
    only: &[String],
) -> Result<Vec<CorpusDocument>, String> {
    let read = |path: &Path| {
        std::fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
    };
    let recording: FixtureRecording = serde_json::from_slice(&read(recording)?)
        .map_err(|error| format!("cannot parse the fixture recording: {error}"))?;
    let expected: FixtureExpectations = serde_json::from_slice(&read(expected)?)
        .map_err(|error| format!("cannot parse the fixture expectations: {error}"))?;
    let mut documents = Vec::new();
    for recorded in recording.fixtures {
        if !only.is_empty() && !only.contains(&recorded.file) {
            continue;
        }
        let RecordedExtraction::Parsed { source } = recorded.extraction else {
            continue;
        };
        let Some(fixture) = expected
            .fixtures
            .iter()
            .find(|fixture| fixture.file == recorded.file)
        else {
            continue;
        };
        documents.push(CorpusDocument {
            gold: GoldDocument {
                id: fixture.file.clone(),
                file: fixture.file.clone(),
                pages: source.pages.len() as u32,
                gold: GoldAnswer {
                    document_type: fixture.document_type.clone(),
                    acceptable_types: fixture.acceptable_types.clone(),
                    document_date: fixture.document_date.clone(),
                    acceptable_dates: fixture.acceptable_dates.clone(),
                    parties: fixture.parties.clone(),
                    description_facts: fixture
                        .acceptable_description_facts
                        .iter()
                        .map(|fact| vec![fact.clone()])
                        .collect(),
                    ..GoldAnswer::default()
                },
                ..GoldDocument::default()
            },
            source,
        });
    }
    Ok(documents)
}

/// `intern-bench retrieval`: every configuration over every document, no
/// model. Exit 0 when it ran.
pub fn retrieval_command(options: &RetrievalOptions) -> Result<i32, String> {
    let (gold, _) = GoldFile::load(&options.gold)?;
    let mut corpus: Vec<(String, CorpusDocument)> = Vec::new();
    for recording in &options.recordings {
        let name = recording
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("recording")
            .to_owned();
        for document in load_bench_corpus(recording, &gold, &options.only)? {
            corpus.push((name.clone(), document));
        }
    }
    if let Some((recording, expected)) = &options.fixtures {
        for document in load_fixture_corpus(recording, expected, &options.only)? {
            corpus.push(("fixtures".to_owned(), document));
        }
    }
    if let Some((directory, worker)) = &options.extract {
        for document in extract_corpus(directory, worker, &gold, &options.only)? {
            corpus.push(("extracted".to_owned(), document));
        }
    }
    if corpus.is_empty() {
        return Err("no recorded source to retrieve from".to_owned());
    }
    let mut configs = Vec::new();
    for path in &options.configs {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let config: RetrievalConfig = serde_json::from_slice(&bytes)
            .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("config")
            .to_owned();
        configs.push((name, config));
    }
    if options.sweep || configs.is_empty() {
        configs.extend(sweep_configs());
    }

    let digests = corpus
        .iter()
        .map(|(_, document)| {
            let digest = distill(&document.source, DigestBudget::default());
            (
                recall(&document.gold.gold, &Haystack::digest(&digest)),
                digest_recall(
                    &document.gold,
                    Some(&document.source),
                    DigestBudget::default(),
                ),
                estimated_prompt_tokens(&digest.text),
            )
        })
        .collect::<Vec<_>>();
    let mut indexes = BTreeMap::new();
    let mut results = Vec::new();
    for (name, config) in &configs {
        let started = Instant::now();
        let result = run_config(
            name,
            config,
            &corpus,
            &digests,
            &mut indexes,
            options.dump.as_deref(),
        )?;
        eprintln!(
            "{name}: {} violations, filename recall {:?}, {:.1} s",
            result.all.violations,
            result.all.recall.get("filename"),
            started.elapsed().as_secs_f64()
        );
        results.push(result);
    }
    let mut ranked = (0..results.len()).collect::<Vec<_>>();
    ranked.sort_by_key(|position| (results[*position].rank_key(), *position));
    let report = json!({
        "corpora": corpus.iter().map(|(name, _)| name.clone()).collect::<std::collections::BTreeSet<_>>(),
        "documents": corpus.len(),
        "ranking": ranked.iter().map(|position| results[*position].name.clone()).collect::<Vec<_>>(),
        "results": results,
    });
    if let Some(path) = &options.output {
        let rendered = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("cannot render the report: {error}"))?;
        std::fs::write(path, rendered + "\n")
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    }
    let markdown = render_markdown(&results, &ranked);
    if let Some(path) = &options.markdown {
        std::fs::write(path, &markdown)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    } else {
        println!("{markdown}");
    }
    Ok(0)
}

fn percent(value: Option<&(f64, f64)>, digest: bool) -> String {
    value.map_or_else(
        || "–".to_owned(),
        |(context, from_digest)| {
            format!(
                "{:.1}",
                100.0 * if digest { *from_digest } else { *context }
            )
        },
    )
}

/// The sweep as tables: the ranking with tuning and held-out figures side
/// by side, tokens by page bucket for the best configuration, and every
/// document it lost an item on.
pub fn render_markdown(results: &[ConfigResult], ranked: &[usize]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# Retrieval sweep\n");
    let Some(first) = results.first() else {
        return out;
    };
    let _ = writeln!(
        out,
        "{} documents ({} tuning, {} held out). Context recall in percent; type, date and parties over every document. Ranked on the tuning half: acceptance gates first (violations, fact recall below the digest's, long documents not 40% smaller), then recall, then long-document tokens. A violation is a document whose digest carried the date and every party (`digest_recall` 1.0) and whose context lacks the type, the date or a party.\n",
        first.all.documents, first.tune.documents, first.holdout.documents
    );
    let _ = writeln!(
        out,
        "| Rank | Configuration | Violations tune / held out | Filename tune / held out | Facts tune / held out | Subject tune / held out | Type | Date | Parties | Tokens p50 | Long-doc reduction | Long docs under 40% |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|---|---|");
    for (rank, position) in ranked.iter().enumerate() {
        let result = &results[*position];
        let all = &result.all;
        let pair = |item: &str| {
            format!(
                "{} / {}",
                percent(result.tune.recall.get(item), false),
                percent(result.holdout.recall.get(item), false)
            )
        };
        let _ = writeln!(
            out,
            "| {} | `{}` | {} / {} | {} | {} | {} | {} | {} | {} | {:.0} | {} | {} |",
            rank + 1,
            result.name,
            result.tune.violations,
            result.holdout.violations,
            pair("filename"),
            pair("facts"),
            pair("subject"),
            percent(all.recall.get("type"), false),
            percent(all.recall.get("date"), false),
            percent(all.recall.get("parties"), false),
            all.context_tokens
                .map_or(0.0, |distribution| distribution.p50),
            all.long_reduction
                .map_or_else(|| "–".to_owned(), |value| format!("{:.1}%", value * 100.0)),
            all.long_short,
        );
    }
    let digest = &first.all;
    let _ = writeln!(
        out,
        "\nThe digest: type {}, date {}, parties {}, facts {} (tune {} / held out {}), subject {} (tune {} / held out {}); tokens p50 {:.0}.\n",
        percent(digest.recall.get("type"), true),
        percent(digest.recall.get("date"), true),
        percent(digest.recall.get("parties"), true),
        percent(digest.recall.get("facts"), true),
        percent(first.tune.recall.get("facts"), true),
        percent(first.holdout.recall.get("facts"), true),
        percent(digest.recall.get("subject"), true),
        percent(first.tune.recall.get("subject"), true),
        percent(first.holdout.recall.get("subject"), true),
        digest
            .digest_tokens
            .map_or(0.0, |distribution| distribution.p50),
    );
    if let Some(best) = ranked.first().map(|position| &results[*position]) {
        let _ = writeln!(out, "## Best on the tuning half: `{}`\n", best.name);
        let _ = writeln!(
            out,
            "| Pages | Documents | Context tokens p50 | Digest tokens p50 | Context / digest |"
        );
        let _ = writeln!(out, "|---|---|---|---|---|");
        for name in BUCKETS {
            if let Some(bucket) = best.all.buckets.get(name) {
                let _ = writeln!(
                    out,
                    "| {name} | {} | {:.0} | {:.0} | {:.2} |",
                    bucket.documents, bucket.context_p50, bucket.digest_p50, bucket.ratio
                );
            }
        }
        let _ = writeln!(out, "\n| Document | Split | Pages | Lost | Digest lost |");
        let _ = writeln!(out, "|---|---|---|---|---|");
        for result in &best.documents {
            let lost = Item::ALL.iter().any(|item| result.loses(*item)) || result.violates();
            if !lost && result.context.missing.is_empty() {
                continue;
            }
            let _ = writeln!(
                out,
                "| `{}` ({}) | {} | {} | {} | {} |",
                result.id,
                result.corpus,
                result.split,
                result.pages,
                result.context.missing.join(", "),
                result.digest.missing.join(", "),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gold::GoldEvidence;
    use intern_engine::{PageOrigin, SourcePage};

    fn notice() -> GoldDocument {
        GoldDocument {
            id: "notice".into(),
            file: "notice.pdf".into(),
            pages: 1,
            gold: GoldAnswer {
                document_type: Some("Notice of Default".into()),
                document_date: Some("2025-10-14".into()),
                parties: Some(vec!["Glasswing Ceramics LLC".into()]),
                description_facts: vec![vec!["$12,400.00".into()], vec!["Kiln 4".into()]],
                subject_terms: vec!["default".into(), "lease".into()],
                evidence: GoldEvidence {
                    date_text: vec!["October 14, 2025".into()],
                    ..GoldEvidence::default()
                },
                ..GoldAnswer::default()
            },
            ..GoldDocument::default()
        }
    }

    fn source(text: &str) -> DocumentSource {
        DocumentSource::from_pages(vec![SourcePage::new(1, text, PageOrigin::Native)])
    }

    #[test]
    fn a_document_sent_whole_carries_every_item_it_states() {
        let source = source(
            "NOTICE OF DEFAULT\n\nDate: October 14, 2025\n\nTo: Glasswing Ceramics LLC\n\nYou are in default under the lease. Amount past due: $12,400.00.",
        );
        let scored = context_scores(&notice(), Some(&source), &RetrievalConfig::default());
        let value = |key: &str| scored.scores.get(key).and_then(Value::as_f64);
        assert_eq!(value("context_type_recall"), Some(1.0));
        assert_eq!(value("context_date_recall"), Some(1.0));
        assert_eq!(value("context_party_recall"), Some(1.0));
        assert_eq!(value("context_recall"), Some(1.0));
        assert_eq!(
            value("context_fact_recall"),
            Some(0.5),
            "Kiln 4 is not stated"
        );
        assert_eq!(value("context_subject_recall"), Some(1.0));
        assert!(scored.timings["context_tokens"].as_u64().unwrap() > 0);
        assert!(scored.timings.contains_key("index_ms"));
    }

    #[test]
    fn a_document_not_read_carries_nothing() {
        let scored = context_scores(&notice(), None, &RetrievalConfig::default());
        for key in CONTEXT_SCORES {
            assert_eq!(scored.scores[key].as_f64(), Some(0.0), "{key}");
        }
        assert!(scored.timings.is_empty());
    }

    #[test]
    fn a_type_needs_most_of_its_words_together() {
        let gold = notice().gold;
        let found = |text: &str| {
            recall(
                &gold,
                &Haystack {
                    texts: vec![normalize(text)],
                    type_texts: text.split('\n').map(normalize).collect(),
                },
            )
            .document_type
        };
        assert_eq!(found("NOTICE OF DEFAULT"), Some(1.0));
        assert_eq!(found("NOTICE\nDEFAULT"), Some(0.0));
        assert_eq!(found("Defaulted notice"), Some(0.0));
    }

    #[test]
    fn the_split_is_fixed_by_the_id() {
        assert_eq!(
            split_of("credit-agreement-50p"),
            split_of("credit-agreement-50p")
        );
        let ids = (0..64)
            .map(|index| format!("doc-{index}"))
            .collect::<Vec<_>>();
        let tune = ids.iter().filter(|id| split_of(id) == "tune").count();
        assert!((16..=48).contains(&tune), "{tune}");
        assert_eq!(bucket(3), "1-3");
        assert_eq!(bucket(10), "10-24");
        assert_eq!(bucket(100), "25+");
    }

    #[test]
    fn every_sweep_configuration_has_its_own_fingerprint() {
        let configs = sweep_configs();
        let fingerprints = configs
            .iter()
            .map(|(_, config)| config.fingerprint())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(fingerprints.len(), configs.len());
        let names = configs
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), configs.len());
    }
}
