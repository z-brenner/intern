//! The pinned local model set.
//!
//! The manifest is compiled into the binary, so the list of files Intern will
//! ever download cannot be changed at runtime. Every file is accepted only if
//! its byte length and SHA-256 both match.
//!
//! There are two models, one per [`ModelTier`]: the accurate one, Qwen3.5 2B,
//! which reads every document it can, and an optional quick one, Bonsai 2 2B,
//! a third of the download and about two thirds of the time, whose readings
//! always go to a person (see [`crate::tiers`]). The 77-document evaluation in
//! `bench/reports/bonsai-linux-cpu-2026-10-27` chose them: 52-54 correct
//! names for Qwen against 37 for Bonsai 2 2B and 48 for Bonsai 2 4B, at 1.6
//! times Qwen's time. Which of them reads a document is [`crate::tiers`]'s
//! decision; this module only says what they are. Nothing here, nor anywhere
//! in setup, deletes a model file: an earlier build's model is this one's
//! accurate tier.

use std::{collections::HashSet, ffi::OsStr, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineErrorCode, EngineResult};
use crate::profile::PromptProfile;

/// The manifest schema this build reads. 2 pinned one model; 3 pins one per
/// tier. Schema 2's model is schema 3's accurate tier, file for file.
pub const MANIFEST_SCHEMA_VERSION: u32 = 3;

/// The most a first-run setup may ever ask a person to download: every file
/// of every tier together.
pub const MAX_TOTAL_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// What a manifest file is for.
///
/// There is exactly one role. Intern reads documents as text, and the engine
/// cannot send an image to the model at all, so pinning a vision projector
/// would download hundreds of megabytes that nothing could ever load.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    /// The text model.
    Model,
}

/// Which of the pinned models a manifest entry is.
///
/// Ordered by size: for someone setting Intern up for the first time the fast
/// tier downloads first and makes Intern usable at once, and the accurate tier
/// arrives after it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Fast,
    Accurate,
}

impl ModelTier {
    /// Every tier, in the order setup installs them.
    pub const ALL: [Self; 2] = [Self::Fast, Self::Accurate];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Accurate => "accurate",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelManifest {
    pub schema_version: u32,
    /// Value sent as the OpenAI-style `model` field.
    pub served_model_name: String,
    /// One entry per tier, each tier exactly once.
    pub models: Vec<TierModel>,
}

/// The model one tier runs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TierModel {
    pub tier: ModelTier,
    /// A stable identifier, recorded with every analysis the model makes.
    pub model_id: String,
    /// The model's name as a person reads it: "Qwen3.5 2B".
    pub display_name: String,
    /// How this model is asked. Absent: the engine's evidence request as it
    /// is, with nothing added.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_profile: Option<PromptProfile>,
    pub files: Vec<ModelFile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelFile {
    pub name: String,
    pub role: ModelRole,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

pub const fn embedded_manifest_json() -> &'static str {
    include_str!("../../../src-tauri/resources/model-manifest.json")
}

impl ModelManifest {
    pub fn embedded() -> EngineResult<Self> {
        Self::parse(embedded_manifest_json())
    }

    pub fn parse(json: &str) -> EngineResult<Self> {
        let manifest: Self = serde_json::from_str(json).map_err(|_| invalid())?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> EngineResult<()> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION || self.served_model_name.is_empty() {
            return Err(invalid());
        }
        // Each tier exactly once: a tier named twice would leave which file
        // runs it to the order of a list, and a tier missing would leave
        // Intern unable to honour a setting that asks for it.
        if self.models.len() != ModelTier::ALL.len()
            || ModelTier::ALL.iter().any(|tier| {
                self.models
                    .iter()
                    .filter(|model| model.tier == *tier)
                    .count()
                    != 1
            })
        {
            return Err(invalid());
        }
        // A name is a path in one shared model folder, so two tiers naming
        // the same file would have one download overwrite the other's model.
        let mut names = HashSet::new();
        let mut ids = HashSet::new();
        for model in &self.models {
            // Exactly one file, and it is the text model. A manifest that
            // could name more would be a manifest that could make a person
            // download something Intern never loads.
            if model.model_id.is_empty()
                || model.display_name.trim().is_empty()
                || !ids.insert(model.model_id.as_str())
                || model.files.len() != 1
                || model.files.iter().any(|file| file.role != ModelRole::Model)
            {
                return Err(invalid());
            }
            for file in &model.files {
                if !is_safe_filename(&file.name)
                    || !file.url.starts_with("https://")
                    || !pins_an_immutable_revision(&file.url)
                    || file.size == 0
                    || !names.insert(file.name.as_str())
                    || !is_lowercase_digest(&file.sha256)
                {
                    return Err(invalid());
                }
            }
        }
        if self.total_bytes() >= MAX_TOTAL_DOWNLOAD_BYTES {
            return Err(invalid());
        }
        Ok(())
    }

    /// The model for `tier`. A validated manifest has one for every tier.
    pub fn tier(&self, tier: ModelTier) -> Option<&TierModel> {
        self.models.iter().find(|model| model.tier == tier)
    }

    /// The tier models, in [`ModelTier::ALL`] order.
    pub fn tiers(&self) -> impl Iterator<Item = &TierModel> {
        ModelTier::ALL
            .into_iter()
            .filter_map(|tier| self.tier(tier))
    }

    /// Every file of every tier.
    pub fn files(&self) -> impl Iterator<Item = &ModelFile> {
        self.tiers().flat_map(|model| model.files.iter())
    }

    /// Everything every tier downloads, together.
    pub fn total_bytes(&self) -> u64 {
        self.models.iter().map(TierModel::total_bytes).sum()
    }
}

impl TierModel {
    /// The text model file.
    pub fn model(&self) -> Option<&ModelFile> {
        self.files.iter().find(|file| file.role == ModelRole::Model)
    }

    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }
}

/// Whether a download URL names a revision that cannot move.
///
/// Hugging Face serves `/resolve/<ref>/<path>`, and a branch name is a ref
/// that moves: an upstream re-upload changes the bytes under a digest the
/// manifest has already promised, and the download then costs a whole model
/// on every retry and fails the same check every time with nothing to say
/// why. A 40-character commit id cannot move, so the digest either matches on
/// the first attempt or the pin itself is wrong.
fn pins_an_immutable_revision(url: &str) -> bool {
    url.split('/').any(|segment| {
        segment.len() == 40
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn is_lowercase_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_safe_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.bytes().any(|byte| byte.is_ascii_control())
        && Path::new(name).file_name() == Some(OsStr::new(name))
        && Path::new(name).components().count() == 1
}

const fn invalid() -> EngineError {
    EngineError::new(
        EngineErrorCode::ManifestInvalid,
        "embedded model manifest failed validation",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embedded() -> ModelManifest {
        ModelManifest::embedded().unwrap()
    }

    #[test]
    fn the_embedded_manifest_is_valid_and_names_one_text_model_per_tier() {
        let manifest = embedded();
        assert_eq!(manifest.schema_version, MANIFEST_SCHEMA_VERSION);
        for tier in ModelTier::ALL {
            let model = manifest.tier(tier).expect("every tier is pinned");
            assert_eq!(model.tier, tier);
            assert!(model.model().is_some());
            assert_eq!(model.files.len(), 1);
        }
        // The quick model is asked the way its qualification froze; the
        // accurate one with the production request alone, which is what the
        // recordings of record answer.
        assert_eq!(
            manifest.tier(ModelTier::Fast).unwrap().prompt_profile,
            Some(PromptProfile::Bonsai2)
        );
        assert_eq!(
            manifest.tier(ModelTier::Accurate).unwrap().prompt_profile,
            None
        );
        assert_eq!(
            manifest.tiers().map(|model| model.tier).collect::<Vec<_>>(),
            ModelTier::ALL
        );
        for file in manifest.files() {
            assert!(
                file.url.starts_with("https://huggingface.co/"),
                "{}",
                file.url
            );
        }
        // The fast tier is what a new person waits for before the first
        // document, so it must be the smaller download.
        assert!(
            manifest.tier(ModelTier::Fast).unwrap().total_bytes()
                < manifest.tier(ModelTier::Accurate).unwrap().total_bytes()
        );
    }

    /// Everyone who installed an earlier build has its one model on disk, and
    /// it is this build's accurate tier exactly: the same file, revision,
    /// length and digest. That is what lets an upgrade go straight to the
    /// queue, with no download and nothing deleted.
    #[test]
    fn the_accurate_tier_is_the_model_the_single_model_build_installed() {
        let manifest = embedded();
        let accurate = manifest.tier(ModelTier::Accurate).unwrap();
        assert_eq!(accurate.model_id, "qwen3.5-2b-instruct-q4-k-m");
        assert_eq!(accurate.display_name, "Qwen3.5 2B");
        let file = accurate.model().unwrap();
        assert_eq!(file.name, "Qwen3.5-2B-Q4_K_M.gguf");
        assert_eq!(file.size, 1_280_835_840);
        assert_eq!(
            file.sha256,
            "aaf42c8b7c3cab2bf3d69c355048d4a0ee9973d48f16c731c0520ee914699223"
        );
        assert_eq!(
            file.url,
            "https://huggingface.co/unsloth/Qwen3.5-2B-GGUF/resolve/\
             f6d5376be1edb4d416d56da11e5397a961aca8ae/Qwen3.5-2B-Q4_K_M.gguf"
        );
        let fast = manifest.tier(ModelTier::Fast).unwrap();
        assert_eq!(fast.model().unwrap().name, "Ternary-Bonsai-2-2B-PQ2_0.gguf");
    }

    #[test]
    fn the_whole_download_fits_a_sixteen_gigabyte_laptop() {
        let manifest = embedded();
        // Everything a person may be asked to download, both tiers together.
        // Nothing here may be a file the engine cannot load.
        assert!(
            manifest.total_bytes() < MAX_TOTAL_DOWNLOAD_BYTES,
            "the whole download must stay under 2 GiB, got {} bytes",
            manifest.total_bytes()
        );
        assert_eq!(manifest.files().count(), 2);

        let mut heavy = manifest;
        heavy.models[1].files[0].size = MAX_TOTAL_DOWNLOAD_BYTES;
        assert!(heavy.validate().is_err());
    }

    #[test]
    fn a_manifest_naming_more_than_the_text_model_is_refused() {
        let mut extra = embedded();
        let mut second = extra.models[0].files[0].clone();
        second.name = "mmproj.gguf".into();
        extra.models[0].files.push(second);
        assert!(extra.validate().is_err());

        let mut none = embedded();
        none.models[1].files.clear();
        assert!(none.validate().is_err());
    }

    #[test]
    fn every_tier_is_pinned_exactly_once() {
        let mut missing = embedded();
        missing.models.remove(1);
        assert!(missing.validate().is_err(), "a tier missing");

        let mut twice = embedded();
        twice.models[1].tier = ModelTier::Fast;
        assert!(twice.validate().is_err(), "a tier named twice");

        let mut three = embedded();
        let mut third = three.models[1].clone();
        third.model_id = "another".into();
        third.files[0].name = "another.gguf".into();
        three.models.push(third);
        assert!(three.validate().is_err(), "a third model");

        let mut unnamed = embedded();
        unnamed.models[0].display_name = "  ".into();
        assert!(unnamed.validate().is_err(), "a model with no name to show");

        let mut same_id = embedded();
        same_id.models[1].model_id = same_id.models[0].model_id.clone();
        assert!(same_id.validate().is_err(), "two tiers with one identity");

        let mut old = embedded();
        old.schema_version = 2;
        assert!(old.validate().is_err(), "a schema this build does not read");
    }

    /// One model folder holds every tier, so one tier's download must never
    /// be able to overwrite another's model.
    #[test]
    fn file_names_are_unique_across_tiers() {
        let mut shared = embedded();
        shared.models[1].files[0].name = shared.models[0].files[0].name.clone();
        assert!(shared.validate().is_err());

        // Nothing names files to clean away: the field a manifest that did
        // would carry is refused.
        let mut value: serde_json::Value = serde_json::from_str(embedded_manifest_json()).unwrap();
        value["retired_files"] = serde_json::json!([]);
        assert!(ModelManifest::parse(&value.to_string()).is_err());
    }

    /// A moving upstream ref makes the pinned digest a promise the manifest
    /// cannot keep: an upstream re-upload changes the bytes under it, and the
    /// download then costs a whole model on every retry and fails the same
    /// check each time with nothing to say why.
    #[test]
    fn manifest_urls_pin_an_immutable_revision() {
        let base = embedded();
        for file in base.files() {
            assert!(!file.url.contains("/resolve/main/"), "{}", file.url);
        }

        for tier in 0..2 {
            let mut moving = base.clone();
            moving.models[tier].files[0].url = "https://huggingface.co/prism-ml/Ternary-Bonsai-2-2B-gguf/resolve/main/Ternary-Bonsai-2-2B-PQ2_0.gguf".into();
            assert!(moving.validate().is_err());
        }
    }

    #[test]
    fn path_traversal_and_bad_digests_are_refused() {
        let base = embedded();
        for tier in 0..2 {
            let mut traversal = base.clone();
            traversal.models[tier].files[0].name = "../evil.gguf".into();
            assert!(traversal.validate().is_err());

            let mut digest = base.clone();
            digest.models[tier].files[0].sha256 = "NOTHEX".into();
            assert!(digest.validate().is_err());

            let mut upper = base.clone();
            upper.models[tier].files[0].sha256 = upper.models[tier].files[0].sha256.to_uppercase();
            assert!(upper.validate().is_err());

            let mut scheme = base.clone();
            scheme.models[tier].files[0].url = "http://huggingface.co/x".into();
            assert!(scheme.validate().is_err());
        }
    }

    #[test]
    fn unknown_fields_and_profiles_are_refused() {
        let json = embedded_manifest_json();
        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        value["models"][0]["prompt_profile"] = "bonsai_3".into();
        assert!(ModelManifest::parse(&value.to_string()).is_err());

        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        value["models"][0]["mmproj"] = "projector.gguf".into();
        assert!(ModelManifest::parse(&value.to_string()).is_err());

        // A model with no profile is asked with the engine's request alone.
        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        value["models"][0]
            .as_object_mut()
            .unwrap()
            .remove("prompt_profile");
        let parsed = ModelManifest::parse(&value.to_string()).unwrap();
        assert_eq!(parsed.tier(ModelTier::Fast).unwrap().prompt_profile, None);
        // And it is written without the field, as the embedded file is.
        let written = serde_json::to_value(parsed.tier(ModelTier::Accurate).unwrap()).unwrap();
        assert!(written.get("prompt_profile").is_none());
    }
}
