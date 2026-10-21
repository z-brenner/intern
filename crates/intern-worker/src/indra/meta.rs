//! `meta.json`: the vocabularies and sizes the networks were exported with
//! (spec §1 "meta.json"; read in `OrtOCR.__init__`, `pipeline.py:324-358`).

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::de::{Deserializer, MapAccess, Visitor};

use super::beam::{LM_ALPHA, LM_BETA, LM_GATE};
use super::vocab::Vocab;

/// The script groups of `script.onnx`'s classes when `meta.json` does not
/// list them (`GROUPS`, `pipeline.py:86`). The pinned `meta.json` lists
/// `["base", "cjk", "rtl", "world"]`.
pub const DEFAULT_SCRIPT_GROUPS: [&str; 4] = ["base", "cjk", "rtl", "other"];

/// A group's language-model settings from `meta.json` `lm_params`; an
/// absent value is the base recognizer's (`pipeline.py:357-370`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LmParams {
    pub alpha: Option<f64>,
    pub beta: Option<f64>,
    pub gate: Option<f64>,
}

impl LmParams {
    /// `(alpha, beta, gate)` with the pipeline's defaults filled in.
    pub fn resolved(&self) -> (f64, f64, f64) {
        (
            self.alpha.unwrap_or(LM_ALPHA),
            self.beta.unwrap_or(LM_BETA),
            self.gate.unwrap_or(LM_GATE),
        )
    }
}

/// The parts of `meta.json` the pipeline reads.
#[derive(Clone, Debug)]
pub struct Meta {
    pub det_long_side: usize,
    pub rec_height: usize,
    /// How many line heights `rec.onnx` reads (`rec_window`, default 1).
    pub rec_window: usize,
    pub vocab: Vocab,
    /// The script group recognizers' vocabularies, when present.
    pub vocab_world: Option<Vocab>,
    pub vocab_rtl: Option<Vocab>,
    pub vocab_cjk: Option<Vocab>,
    /// Every `vocab_<group>` group in the file's key order, the order the
    /// reference tries the groups' recognizers in (spec §6.4:
    /// world, rtl, cjk for the pinned models).
    pub groups: Vec<String>,
    /// `script.onnx`'s class order, when the file gives it.
    pub script_groups: Option<Vec<String>>,
    /// Per-group language-model settings, when given.
    pub lm_params: Option<BTreeMap<String, LmParams>>,
}

/// The top-level object's entries in the file's order: `serde_json` without
/// `preserve_order` would sort them, and the group order matters.
struct Entries(Vec<(String, serde_json::Value)>);

impl<'de> serde::Deserialize<'de> for Entries {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntriesVisitor;
        impl<'de> Visitor<'de> for EntriesVisitor {
            type Value = Entries;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Entries, A::Error> {
                let mut entries = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                    // A repeated key keeps its first place and its last value,
                    // as Python's `json.loads` keeps them.
                    match entries.iter_mut().find(|(k, _)| *k == key) {
                        Some((_, old)) => *old = value,
                        None => entries.push((key, value)),
                    }
                }
                Ok(Entries(entries))
            }
        }
        deserializer.deserialize_map(EntriesVisitor)
    }
}

fn usize_of(value: &serde_json::Value, key: &str) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| format!("meta.json {key} is not a whole number"))
}

fn vocab_of(value: &serde_json::Value, key: &str) -> Result<Vocab, String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("meta.json {key} is not a string"))?;
    Vocab::from_json(text).map_err(|e| format!("meta.json {key}: {e}"))
}

fn f64_of(value: Option<&serde_json::Value>, key: &str) -> Result<Option<f64>, String> {
    value
        .map(|v| {
            v.as_f64()
                .ok_or_else(|| format!("meta.json lm_params {key} is not a number"))
        })
        .transpose()
}

impl Meta {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Self::parse(&text)
    }

    /// Reads `meta.json`'s text. `det_long_side`, `rec_height` and `vocab`
    /// are required, as the reference indexes them; the rest are optional.
    pub fn parse(text: &str) -> Result<Self, String> {
        let Entries(entries) = serde_json::from_str(text)
            .map_err(|e| format!("meta.json is not a JSON object: {e}"))?;
        let get = |key: &str| entries.iter().find(|(k, _)| k == key).map(|(_, v)| v);
        let need = |key: &str| get(key).ok_or_else(|| format!("meta.json has no {key}"));
        let det_long_side = usize_of(need("det_long_side")?, "det_long_side")?;
        let rec_height = usize_of(need("rec_height")?, "rec_height")?;
        let rec_window = match get("rec_window") {
            Some(value) => usize_of(value, "rec_window")?,
            None => 1,
        };
        let vocab = vocab_of(need("vocab")?, "vocab")?;
        let optional_vocab = |key: &str| get(key).map(|v| vocab_of(v, key)).transpose();
        let groups = entries
            .iter()
            .filter_map(|(k, _)| k.strip_prefix("vocab_"))
            .map(str::to_owned)
            .collect();
        let script_groups = get("script_groups")
            .map(|value| {
                value
                    .as_array()
                    .and_then(|groups| {
                        groups
                            .iter()
                            .map(|g| g.as_str().map(str::to_owned))
                            .collect::<Option<Vec<_>>>()
                    })
                    .ok_or_else(|| "meta.json script_groups is not a list of names".to_owned())
            })
            .transpose()?;
        let lm_params = get("lm_params")
            .map(|value| {
                let params = value
                    .as_object()
                    .ok_or("meta.json lm_params is not an object")?;
                params
                    .iter()
                    .map(|(group, p)| {
                        let p = p.as_object().ok_or_else(|| {
                            format!("meta.json lm_params {group} is not an object")
                        })?;
                        let params = LmParams {
                            alpha: f64_of(p.get("alpha"), "alpha")?,
                            beta: f64_of(p.get("beta"), "beta")?,
                            gate: f64_of(p.get("gate"), "gate")?,
                        };
                        Ok((group.clone(), params))
                    })
                    .collect::<Result<BTreeMap<_, _>, String>>()
            })
            .transpose()?;
        Ok(Self {
            det_long_side,
            rec_height,
            rec_window,
            vocab,
            vocab_world: optional_vocab("vocab_world")?,
            vocab_rtl: optional_vocab("vocab_rtl")?,
            vocab_cjk: optional_vocab("vocab_cjk")?,
            groups,
            script_groups,
            lm_params,
        })
    }

    /// `(alpha, beta, gate)` for a recognizer's language model: `"base"`
    /// for `rec.onnx`, or a group name (`pipeline.py:357-370`).
    pub fn lm_settings(&self, group: &str) -> (f64, f64, f64) {
        self.lm_params
            .as_ref()
            .and_then(|params| params.get(group))
            .copied()
            .unwrap_or_default()
            .resolved()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sizes_vocabularies_and_group_order() {
        let text = r#"{
            "det_long_side": 1024,
            "vocab": "{\"blank\": 0, \"chars\": [\" \", \"a\", \"\\\\alpha\"]}",
            "rec_height": 32,
            "vocab_world": "{\"blank\": 0, \"chars\": [\" \", \"क\"]}",
            "script_groups": ["base", "cjk", "rtl", "world"],
            "vocab_rtl": "{\"blank\": 0, \"chars\": [\" \"]}",
            "vocab_cjk": "{\"blank\": 0, \"chars\": [\" \", \"ᄀ\"]}",
            "rec_window": 3,
            "lm_params": {"world": {"alpha": 0.2, "beta": 1.5, "gate": 0.95}, "cjk": {"alpha": 0.25}}
        }"#;
        let meta = Meta::parse(text).unwrap();
        assert_eq!(meta.det_long_side, 1024);
        assert_eq!(meta.rec_height, 32);
        assert_eq!(meta.rec_window, 3);
        assert_eq!(meta.vocab.chars, vec![" ", "a", "\\alpha"]);
        assert!(meta.vocab.has_commands);
        assert_eq!(meta.groups, vec!["world", "rtl", "cjk"]);
        assert!(meta.vocab_cjk.as_ref().unwrap().jamo);
        assert_eq!(meta.vocab_world.as_ref().unwrap().chars[1], "\u{915}");
        assert_eq!(
            meta.script_groups.as_deref().unwrap(),
            ["base", "cjk", "rtl", "world"]
        );
        assert_eq!(meta.lm_settings("base"), (0.5, 1.5, 0.9));
        assert_eq!(meta.lm_settings("world"), (0.2, 1.5, 0.95));
        assert_eq!(meta.lm_settings("cjk"), (0.25, 1.5, 0.9));
        assert_eq!(meta.lm_params.unwrap().len(), 2);
    }

    #[test]
    fn defaults_the_window_and_requires_the_base_fields() {
        let base = r#""vocab": "{\"chars\": [\" \"]}""#;
        let meta = Meta::parse(&format!(
            r#"{{"det_long_side": 640, "rec_height": 32, {base}}}"#
        ))
        .unwrap();
        assert_eq!(meta.rec_window, 1);
        assert!(meta.groups.is_empty());
        assert!(
            meta.vocab_cjk.is_none() && meta.script_groups.is_none() && meta.lm_params.is_none()
        );
        assert!(Meta::parse(&format!(r#"{{"rec_height": 32, {base}}}"#)).is_err());
        assert!(Meta::parse(r#"{"det_long_side": 640, "rec_height": 32}"#).is_err());
        assert!(Meta::parse("[]").is_err());
    }
}
