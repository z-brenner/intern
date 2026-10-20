//! A recognizer's vocabulary, from `meta.json` (spec §1 "meta.json", §6.5).

/// Class 0 is the CTC blank; class `i + 1` is `chars[i]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vocab {
    pub chars: Vec<String>,
    /// Hangul as jamo: decoding NFC-composes.
    pub jamo: bool,
    /// Whether some entries are LaTeX commands (`\alpha`).
    pub has_commands: bool,
}

impl Vocab {
    /// From the JSON string `meta.json` holds under `vocab*`.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let _ = text;
        todo!("spec §1")
    }

    /// `Vocab.decode(ids)` (spec §6.5).
    pub fn decode(&self, ids: &[u32]) -> String {
        let _ = ids;
        todo!("spec §6.5")
    }
}
