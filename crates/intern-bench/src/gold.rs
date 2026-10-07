//! The reviewed answers InternBench scores against: `bench/gold.json`.
//!
//! The generator in `bench/` writes this file; nothing here invents an
//! answer. Every field a document may leave out has a default, and fields
//! this crate does not know are ignored rather than refused, so the corpus
//! can grow a field before the scorer learns to use it.

use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

/// The schema version of `bench/gold.json` this crate reads.
pub const GOLD_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoldFile {
    pub schema_version: u32,
    #[serde(default)]
    pub suite: String,
    pub documents: Vec<GoldDocument>,
}

impl GoldFile {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let gold: Self =
            serde_json::from_slice(bytes).map_err(|error| format!("cannot parse gold: {error}"))?;
        if gold.schema_version != GOLD_SCHEMA_VERSION {
            return Err(format!(
                "gold schema {} is not the supported {GOLD_SCHEMA_VERSION}",
                gold.schema_version
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for document in &gold.documents {
            if !seen.insert(document.id.as_str()) {
                return Err(format!("gold lists document {} twice", document.id));
            }
        }
        Ok(gold)
    }

    pub fn load(path: &Path) -> Result<(Self, Vec<u8>), String> {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("cannot read gold {}: {error}", path.display()))?;
        Ok((Self::parse(&bytes)?, bytes))
    }
}

/// One benchmark document and what a careful reader would name it.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GoldDocument {
    pub id: String,
    /// Relative to the generated corpus directory.
    pub file: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub text_layer: String,
    #[serde(default)]
    pub pages: u32,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub gold: GoldAnswer,
    #[serde(default)]
    pub ocr_truth: Option<OcrTruth>,
    /// The text a document whose own text layer is deliberately corrupt
    /// really says. A description that states the true amount is not
    /// unsupported because the layer Intern read garbled it.
    #[serde(default)]
    pub clean_text: Option<String>,
    /// `"pending"` for a document added before anyone could record it.
    #[serde(default)]
    pub recording: Option<String>,
}

impl GoldDocument {
    /// The extension Intern composes the name with: the file's own.
    pub fn extension(&self) -> &str {
        Path::new(&self.file)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
    }

    pub fn is_pending(&self) -> bool {
        self.recording.as_deref() == Some("pending")
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GoldAnswer {
    #[serde(default)]
    pub document_type: Option<String>,
    #[serde(default)]
    pub acceptable_types: Vec<String>,
    #[serde(default)]
    pub document_date: Option<String>,
    #[serde(default)]
    pub acceptable_dates: Vec<String>,
    #[serde(default)]
    pub date_role: Option<String>,
    #[serde(default)]
    pub forbidden_dates: Vec<Forbidden>,
    /// Absent means the parties are not scored; empty means the right
    /// answer names nobody.
    #[serde(default)]
    pub parties: Option<Vec<String>>,
    #[serde(default)]
    pub party_relation: Option<String>,
    #[serde(default)]
    pub acceptable_party_sets: Vec<PartySet>,
    #[serde(default)]
    pub party_roles: Vec<PartyRole>,
    #[serde(default)]
    pub forbidden_parties: Vec<Forbidden>,
    /// Each fact is a list of surface forms; any one of them covers it, so
    /// every form is specific enough that a description containing it states
    /// the fact (the generator tests hold the gold to this).
    #[serde(default)]
    pub description_facts: Vec<Vec<String>>,
    /// What a careless reading would assert that the document does not say.
    /// A value the document prints is never listed: stating it is true.
    #[serde(default)]
    pub description_forbidden: Vec<String>,
    #[serde(default)]
    pub subject_terms: Vec<String>,
    #[serde(default)]
    pub expected_readiness: Option<String>,
    #[serde(default)]
    pub evidence: GoldEvidence,
}

impl GoldAnswer {
    /// The reviewed party set first, then every other defensible one, each
    /// with the relation it is named under.
    pub fn party_sets(&self) -> Vec<PartySet> {
        let mut sets = Vec::new();
        if let Some(parties) = &self.parties {
            sets.push(PartySet {
                parties: parties.clone(),
                relation: self.party_relation.clone(),
            });
        }
        for set in &self.acceptable_party_sets {
            sets.push(PartySet {
                parties: set.parties.clone(),
                relation: set.relation.clone().or_else(|| self.party_relation.clone()),
            });
        }
        sets
    }
}

/// A trap: a value a careless reading picks, and why it is wrong. Written
/// either as `{"date": .., "why": ..}` / `{"name": .., "why": ..}` or as a
/// bare string.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Forbidden {
    Plain(String),
    Date {
        date: String,
        #[serde(default)]
        why: String,
    },
    Name {
        name: String,
        #[serde(default)]
        why: String,
    },
}

impl Forbidden {
    pub fn value(&self) -> &str {
        match self {
            Self::Plain(value) => value,
            Self::Date { date, .. } => date,
            Self::Name { name, .. } => name,
        }
    }

    /// Why the value is a trap, when the gold says.
    pub fn why(&self) -> &str {
        match self {
            Self::Plain(_) => "",
            Self::Date { why, .. } | Self::Name { why, .. } => why,
        }
    }

    /// "2026-04-03 (payment due date)", for a person reading a miss.
    pub fn describe(&self) -> String {
        if self.why().is_empty() {
            self.value().to_owned()
        } else {
            format!("{} ({})", self.value(), self.why())
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PartySet {
    pub parties: Vec<String>,
    #[serde(default)]
    pub relation: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PartyRole {
    pub name: String,
    pub role: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GoldEvidence {
    /// Verbatim forms the document states the defining date in.
    #[serde(default)]
    pub date_text: Vec<String>,
    /// For each gold party, the verbatim forms it appears in.
    #[serde(default)]
    pub party_text: BTreeMap<String, Vec<String>>,
}

/// What was drawn on the scanned pages, for measuring OCR.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OcrTruth {
    #[serde(default)]
    pub pages: Vec<OcrTruthPage>,
    #[serde(default)]
    pub dates: Vec<String>,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub identifiers: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OcrTruthPage {
    pub page: usize,
    pub text: String,
}

/// The page-count bucket a document is grouped and timed under.
pub fn page_bucket(pages: u32) -> &'static str {
    match pages {
        0 | 1 => "1",
        2..=4 => "2-4",
        5..=9 => "5-9",
        10..=24 => "10-24",
        25..=49 => "25-49",
        50..=99 => "50-99",
        _ => "100+",
    }
}

/// Every bucket, smallest first, for tables that list them in order.
pub const PAGE_BUCKETS: [&str; 7] = ["1", "2-4", "5-9", "10-24", "25-49", "50-99", "100+"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_documented_shape_parses_with_traps_in_either_spelling() {
        let gold = GoldFile::parse(
            br#"{
              "schema_version": 1,
              "suite": "internbench",
              "documents": [{
                "id": "invoice-date-in-table",
                "file": "invoice-date-in-table.pdf",
                "kind": "invoice", "format": "pdf", "text_layer": "native", "pages": 1,
                "categories": ["invoice", "table"],
                "unknown_future_field": true,
                "gold": {
                  "document_type": "Invoice",
                  "document_date": "2026-03-04",
                  "forbidden_dates": [{"date": "2026-04-03", "why": "due date"}, "2026-02-01"],
                  "parties": ["Halvorsen Fixture Works LLC"],
                  "party_relation": "from",
                  "acceptable_party_sets": [{"parties": ["Halvorsen Fixture Works"], "relation": "for"}, {"parties": ["Halvorsen"]}],
                  "forbidden_parties": [{"name": "Quillon Ridge Bakery", "why": "bill-to"}],
                  "description_facts": [["$4,812.50", "4812.50"]],
                  "expected_readiness": "ready",
                  "evidence": {"date_text": ["March 4, 2026"], "party_text": {"Halvorsen Fixture Works LLC": ["Halvorsen Fixture Works LLC"]}}
                },
                "ocr_truth": null
              }]
            }"#,
        )
        .unwrap();
        let document = &gold.documents[0];
        assert_eq!(document.extension(), "pdf");
        let answer = &document.gold;
        assert_eq!(answer.forbidden_dates[0].value(), "2026-04-03");
        assert_eq!(answer.forbidden_dates[1].value(), "2026-02-01");
        assert_eq!(answer.forbidden_parties[0].value(), "Quillon Ridge Bakery");
        let sets = answer.party_sets();
        assert_eq!(sets.len(), 3);
        assert_eq!(sets[1].relation.as_deref(), Some("for"));
        assert_eq!(
            sets[2].relation.as_deref(),
            Some("from"),
            "an acceptable set without a relation keeps the reviewed one"
        );
    }

    #[test]
    fn a_duplicate_id_or_another_schema_is_refused() {
        let duplicate = br#"{"schema_version": 1, "documents": [{"id": "a", "file": "a.pdf"}, {"id": "a", "file": "b.pdf"}]}"#;
        assert!(GoldFile::parse(duplicate).unwrap_err().contains("twice"));
        let future = br#"{"schema_version": 2, "documents": []}"#;
        assert!(GoldFile::parse(future).is_err());
    }

    #[test]
    fn pages_fall_into_the_documented_buckets() {
        let cases = [
            (1, "1"),
            (2, "2-4"),
            (4, "2-4"),
            (5, "5-9"),
            (9, "5-9"),
            (10, "10-24"),
            (24, "10-24"),
            (25, "25-49"),
            (49, "25-49"),
            (50, "50-99"),
            (99, "50-99"),
            (100, "100+"),
            (400, "100+"),
        ];
        for (pages, bucket) in cases {
            assert_eq!(page_bucket(pages), bucket, "{pages} pages");
        }
        assert_eq!(page_bucket(0), "1", "a document always has a page");
    }
}
