//! A recognizer's vocabulary, from `meta.json` (spec §1 "meta.json", §6.5;
//! `Vocab`, `vocab.py:159-246`).

use unicode_normalization::UnicodeNormalization;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

/// The first Hangul leading consonant jamo: a vocabulary holding it reads
/// Hangul syllables as jamo (`vocab.py:164`).
const HANGUL_CHOSEONG_KIYEOK: &str = "\u{1100}";

/// Class 0 is the CTC blank; class `i + 1` is `chars[i]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vocab {
    pub chars: Vec<String>,
    /// Hangul as jamo: decoding NFC-composes.
    pub jamo: bool,
    /// Whether some entries are LaTeX commands (`\alpha`).
    pub has_commands: bool,
}

/// Python's `str.isalpha()`: not empty, and every character a letter
/// (general category Lu, Ll, Lt, Lm or Lo). Rust's `char::is_alphabetic`
/// also counts letter numbers and marks such as Devanagari vowel signs,
/// which the base vocabulary holds.
pub(crate) fn python_isalpha(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_letter)
}

/// Python's `str.isalpha()` of one character.
fn is_letter(c: char) -> bool {
    c.general_category_group() == GeneralCategoryGroup::Letter
}

impl Vocab {
    /// A vocabulary of these tokens (`Vocab.__init__`, `vocab.py:160-164`):
    /// a token longer than one character (counted in code points, as Python
    /// counts) is a LaTeX command.
    pub fn new(chars: Vec<String>) -> Self {
        let has_commands = chars.iter().any(|c| c.chars().nth(1).is_some());
        let jamo = chars.iter().any(|c| c == HANGUL_CHOSEONG_KIYEOK);
        Self {
            chars,
            jamo,
            has_commands,
        }
    }

    /// From the JSON string `meta.json` holds under `vocab*`
    /// (`Vocab.from_json`, `vocab.py:241-246`): `{"blank": 0, "chars": [...]}`.
    /// A blank other than 0 is refused; an absent one is 0.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("vocabulary is not JSON: {e}"))?;
        let object = value.as_object().ok_or("vocabulary is not a JSON object")?;
        if let Some(blank) = object.get("blank") {
            // Python compares with `!= 0`, so 0.0 and false pass too.
            let zero = blank.as_f64() == Some(0.0) || *blank == serde_json::Value::Bool(false);
            if !zero {
                return Err("only blank=0 vocabularies are supported".into());
            }
        }
        let chars = object
            .get("chars")
            .and_then(serde_json::Value::as_array)
            .ok_or("vocabulary has no chars list")?
            .iter()
            .map(|c| c.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()
            .ok_or("vocabulary chars are not all strings")?;
        Ok(Self::new(chars))
    }

    /// The number of recognizer classes, the blank included.
    pub fn n_classes(&self) -> usize {
        self.chars.len() + 1
    }

    /// `Vocab.decode(ids)` (spec §6.5, `vocab.py:223-236`): ids outside
    /// `1..=chars.len()` are dropped; a jamo vocabulary's text is
    /// NFC-composed; otherwise, when the vocabulary has LaTeX commands, one
    /// space follows a command (`\` and letters, over two characters) whose
    /// next token starts with a letter, so `\alpha` `x` does not read back
    /// as `\alphax`.
    pub fn decode(&self, ids: &[u32]) -> String {
        let tokens: Vec<&str> = ids
            .iter()
            .filter_map(|&id| {
                let index = usize::try_from(id).ok()?.checked_sub(1)?;
                self.chars.get(index).map(String::as_str)
            })
            .collect();
        if self.jamo {
            return tokens.concat().nfc().collect();
        }
        if !self.has_commands {
            return tokens.concat();
        }
        let mut out = String::new();
        for (index, token) in tokens.iter().enumerate() {
            out.push_str(token);
            let next_starts_with_letter = tokens
                .get(index + 1)
                .is_some_and(|next| next.chars().next().is_some_and(is_letter));
            if next_starts_with_letter && is_command(token) {
                out.push(' ');
            }
        }
        out
    }
}

/// A LaTeX command that a following letter would run into: over two
/// characters, a backslash and then letters only.
fn is_command(token: &str) -> bool {
    token.chars().nth(2).is_some() && token.strip_prefix('\\').is_some_and(python_isalpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocab(chars: &[&str]) -> Vocab {
        Vocab::new(chars.iter().map(|c| c.to_string()).collect())
    }

    #[test]
    fn reads_json_and_refuses_another_blank() {
        let v = Vocab::from_json(r#"{"blank": 0, "chars": [" ", "a", "\\alpha"]}"#).unwrap();
        assert_eq!(v.chars, vec![" ", "a", "\\alpha"]);
        assert!(v.has_commands);
        assert!(!v.jamo);
        assert_eq!(v.n_classes(), 4);
        assert!(Vocab::from_json(r#"{"chars": ["a"]}"#).is_ok());
        assert!(Vocab::from_json(r#"{"blank": 0.0, "chars": ["a"]}"#).is_ok());
        assert!(Vocab::from_json(r#"{"blank": 1, "chars": ["a"]}"#).is_err());
        assert!(Vocab::from_json(r#"{"blank": 0}"#).is_err());
        assert!(Vocab::from_json("[1]").is_err());
    }

    #[test]
    fn spaces_a_command_from_a_following_letter() {
        let v = vocab(&[" ", "x", "1", "\\alpha", "\\{", "\u{0901}", "é", "\\,"]);
        assert_eq!(v.decode(&[4, 2]), "\\alpha x");
        assert_eq!(v.decode(&[4, 3]), "\\alpha1");
        assert_eq!(v.decode(&[4, 1, 2]), "\\alpha x");
        assert_eq!(v.decode(&[4, 7]), "\\alpha é");
        assert_eq!(v.decode(&[4, 4]), "\\alpha\\alpha");
        // A Devanagari sign is alphabetic to Rust but not a letter to Python.
        assert_eq!(v.decode(&[4, 6]), "\\alpha\u{0901}");
        assert_eq!(v.decode(&[5, 2]), "\\{x");
        assert_eq!(v.decode(&[8, 2]), "\\,x");
        // The blank and ids past the end are dropped.
        assert_eq!(v.decode(&[0, 4, 9, 2]), "\\alpha x");
    }

    #[test]
    fn composes_jamo() {
        let v = vocab(&[" ", "\u{1100}", "\u{1161}", "\u{11A8}", "\u{1112}", "a"]);
        assert!(v.jamo);
        assert!(!v.has_commands);
        // ᄀ ᅡ ᆨ ᄒ ᅡ → 각하
        assert_eq!(v.decode(&[2, 3, 4, 5, 3, 1, 6]), "\u{AC01}\u{D558} a");
    }
}
