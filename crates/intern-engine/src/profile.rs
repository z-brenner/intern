//! How a particular local model is asked: the turns and settings a model
//! family needs around the engine's own evidence request.
//!
//! The evidence request - its system turn, its user turn, and its grammar -
//! is the same for every model, and the recordings of record answer it as it
//! is. A profile adds to it and never rewrites it: a demonstration exchange
//! between the system turn and the document, and a sampler setting. It is
//! chosen by the model manifest ([`crate::manifest::TierModel::prompt_profile`])
//! and carried on the request ([`crate::client::ModelRequest::profile`]), so
//! the request's identity - and every recording keyed by it - says which
//! profile a reply answered.
//!
//! Only the evidence pipeline's requests carry one. The digest pipeline a
//! hosted model reads through is untouched by any profile.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One demonstration exchange: a user turn shaped as the engine's evidence
/// request is, and the reply the model is shown for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExampleTurn {
    pub user: &'static str,
    pub assistant: &'static str,
}

/// A named way of asking one model family.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PromptProfile {
    /// The "few-shot greedy" profile the native Windows qualification froze
    /// for both Bonsai 2 models (`bench/reports/bonsai-native-windows-2026-10-18`,
    /// `plans/profile.json` for the 2B and `plans/bonsai4-profile-17.json`
    /// for the 4B, which hold the same values).
    ///
    /// Its system turn is the production instructions word for word, so it
    /// adds only what the production request lacks: one neutral exchange - a
    /// fictional dispatch note and the reply that files it - and a fixed
    /// seed. Without the exchange, Bonsai 2B copied the instructions' own
    /// placeholder words into its replies and sent most documents to review;
    /// with it, both Bonsai models file the calibration documents' titles,
    /// dates and names. The exchange names no party any real document does,
    /// so a reply that copies it is caught by validation as an unsupported
    /// name rather than filed. Decoding is greedy either way (temperature
    /// 0, top-k 1, thinking off - the engine's own settings), so the seed
    /// changes no token; it is kept so a request is byte for byte the one
    /// that was qualified.
    #[serde(rename = "bonsai_2")]
    Bonsai2,
}

impl PromptProfile {
    pub const ALL: [Self; 1] = [Self::Bonsai2];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bonsai2 => "bonsai_2",
        }
    }

    pub fn parse(word: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|profile| profile.as_str() == word.trim())
    }

    /// The exchanges shown between the system turn and the document, in
    /// order.
    pub const fn examples(self) -> &'static [ExampleTurn] {
        match self {
            Self::Bonsai2 => &BONSAI_2_EXAMPLES,
        }
    }

    /// The sampler seed the request is sent with, when the profile fixes one.
    pub const fn seed(self) -> Option<u64> {
        match self {
            Self::Bonsai2 => Some(17),
        }
    }

    /// Characters the profile adds to every request, for fitting a prompt to
    /// the model's context.
    pub fn added_characters(self) -> usize {
        self.examples()
            .iter()
            .map(|turn| turn.user.chars().count() + turn.assistant.chars().count())
            .sum()
    }

    /// Everything the profile adds to a request, hashed: what a request's
    /// identity ([`crate::client::ModelRequest::sha256`]) and the prompt
    /// version a recording carries take from it. A change to an example or
    /// to the seed is a different request, and makes a recording stale.
    pub fn identity(self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.as_str().as_bytes());
        for turn in self.examples() {
            hasher.update([0_u8]);
            hasher.update(turn.user.as_bytes());
            hasher.update([0_u8]);
            hasher.update(turn.assistant.as_bytes());
        }
        if let Some(seed) = self.seed() {
            hasher.update([0_u8, 1_u8]);
            hasher.update(seed.to_le_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

/// The frozen neutral example. Its user turn has the engine's evidence
/// request shape - the scope line, the delimiters, "JSON only." - and its
/// reply the compact grammar's.
const BONSAI_2_EXAMPLES: [ExampleTurn; 1] = [ExampleTurn {
    user: "The whole 1-page document follows.\n--- BEGIN EVIDENCE ---\nL1 DISPATCH NOTE\nL2 Issued January 16, 2026\nL3 Issuer: Harbor Packing Ltd.\nL4 Recipient: Alder Stone LLC\nL5 Shipment of twelve storage crates.\n--- END EVIDENCE ---\n\nJSON only.",
    assistant: r#"{"type":["DISPATCH NOTE","L1"],"date":["2026-01-16","issuance","L2"],"parties":[["Harbor Packing Ltd.","issuer","L3"],["Alder Stone LLC","recipient","L4"]],"subject":["twelve storage crates","L5"]}"#,
}];

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    use crate::prompt::evidence_system;

    /// The profiles the qualification froze, as committed beside its report.
    fn frozen() -> [Value; 2] {
        [
            serde_json::from_str(include_str!(
                "../../../bench/reports/bonsai-native-windows-2026-10-18/plans/profile.json"
            ))
            .unwrap(),
            serde_json::from_str(include_str!(
                "../../../bench/reports/bonsai-native-windows-2026-10-18/plans/bonsai4-profile-17.json"
            ))
            .unwrap(),
        ]
    }

    /// The production profile is the qualified one, not a copy that has
    /// drifted from it: the same example exchange, the same seed, and a
    /// system turn the production request already sends word for word. The
    /// 2B and 4B profiles were frozen separately and hold the same values.
    #[test]
    fn the_bonsai_profile_is_the_one_the_qualification_froze() {
        let [two, four] = frozen();
        assert_eq!(two["prompt"], four["prompt"], "the two Bonsai prompts");
        assert_eq!(two["settings"], four["settings"], "the two Bonsai settings");
        for profile in [two, four] {
            let prompt = &profile["prompt"];
            assert_eq!(
                prompt["system"].as_str(),
                Some(evidence_system().as_str()),
                "the profile's system turn is the production one"
            );
            assert_eq!(prompt["prefix"], "");
            assert_eq!(prompt["suffix"], "");
            let examples = prompt["examples"].as_array().unwrap();
            let ours = PromptProfile::Bonsai2.examples();
            assert_eq!(examples.len(), ours.len() * 2);
            for (pair, turn) in examples.chunks(2).zip(ours) {
                assert_eq!(pair[0]["role"], "user");
                assert_eq!(pair[0]["content"].as_str(), Some(turn.user));
                assert_eq!(pair[1]["role"], "assistant");
                assert_eq!(pair[1]["content"].as_str(), Some(turn.assistant));
            }
            let settings = &profile["settings"];
            assert_eq!(settings["seed"].as_u64(), PromptProfile::Bonsai2.seed());
            // Everything else the profile set is what every local request
            // already sends: greedy, the reply cap, the prompt cache, and no
            // thinking.
            assert_eq!(settings["temperature"], 0);
            assert_eq!(settings["top_k"], 1);
            assert_eq!(
                settings["max_tokens"].as_u64(),
                Some(u64::from(crate::client::MAX_REPLY_TOKENS))
            );
            assert_eq!(settings["cache_prompt"], true);
            assert_eq!(
                settings["chat_template_kwargs"],
                serde_json::json!({"enable_thinking": false})
            );
            assert_eq!(settings.as_object().unwrap().len(), 6, "nothing else");
        }
    }

    /// The example is a request and a reply the engine itself would make
    /// and accept, so the model is shown exactly the shape it must answer.
    #[test]
    fn the_example_has_the_evidence_request_and_reply_shapes() {
        for turn in PromptProfile::Bonsai2.examples() {
            assert!(
                turn.user
                    .starts_with("The whole 1-page document follows.\n")
            );
            assert!(turn.user.contains("--- BEGIN EVIDENCE ---\n"));
            assert!(turn.user.ends_with("\n--- END EVIDENCE ---\n\nJSON only."));
            let reply: Value = serde_json::from_str(turn.assistant).unwrap();
            assert_eq!(reply.as_object().unwrap().len(), 4);
            // In the grammar's order.
            let at = |key: &str| turn.assistant.find(&format!("\"{key}\":[")).unwrap();
            assert_eq!(at("type"), 1);
            assert!(at("type") < at("date") && at("date") < at("parties"));
            assert!(at("parties") < at("subject"));
            // Compact, as the grammar writes it: no space between tokens.
            for loose in ["\", ", "\": ", "] ,", ", ["] {
                assert!(!turn.assistant.contains(loose), "{loose:?}");
            }
        }
    }

    #[test]
    fn a_profile_reads_and_writes_by_its_manifest_name() {
        for profile in PromptProfile::ALL {
            assert_eq!(PromptProfile::parse(profile.as_str()), Some(profile));
            assert_eq!(
                serde_json::to_value(profile).unwrap(),
                Value::String(profile.as_str().into())
            );
        }
        assert_eq!(PromptProfile::parse("bonsai2"), None);
        assert_eq!(PromptProfile::parse(""), None);
        assert!(serde_json::from_value::<PromptProfile>("bonsai2".into()).is_err());
    }

    #[test]
    fn the_identity_moves_with_what_the_profile_adds() {
        let identity = PromptProfile::Bonsai2.identity();
        assert_eq!(identity.len(), 64);
        assert_eq!(identity, PromptProfile::Bonsai2.identity(), "stable");
        assert!(PromptProfile::Bonsai2.added_characters() > 300);
    }
}
