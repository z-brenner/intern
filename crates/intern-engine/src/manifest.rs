//! The pinned local model set.
//!
//! The manifest is compiled into the binary, so the list of files Intern will
//! ever download cannot be changed at runtime. Every file is accepted only if
//! its byte length and SHA-256 both match.

use std::{collections::HashSet, ffi::OsStr, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineErrorCode, EngineResult};

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

/// The local models Intern can download: one or more variants, each one
/// text model, and the variant used unless a person chooses another.
///
/// Only the variant a person selected is ever downloaded or loaded; the
/// others are named here so the choice, and the "choose existing model
/// files" match, are pinned as tightly as the default.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelManifest {
    pub schema_version: u32,
    /// Value sent as the OpenAI-style `model` field, whichever variant runs.
    pub served_model_name: String,
    /// The id of the variant used unless the person chooses another.
    pub default: String,
    pub variants: Vec<ModelVariant>,
}

/// One downloadable local model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelVariant {
    /// The id settings store (`bonsai-4b`, `bonsai-2b`).
    pub id: String,
    /// The exact model and quantization, for logs and release evidence.
    pub model_id: String,
    /// The name a person is shown.
    pub label: String,
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

/// One variant of the manifest, resolved: everything the setup, the
/// download and the server need to install and run exactly that model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedModel {
    /// The variant's id ([`ModelVariant::id`]).
    pub variant: String,
    pub model_id: String,
    pub label: String,
    pub served_model_name: String,
    pub files: Vec<ModelFile>,
}

impl SelectedModel {
    pub fn model(&self) -> Option<&ModelFile> {
        self.files.iter().find(|file| file.role == ModelRole::Model)
    }

    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }
}

pub const fn embedded_manifest_json() -> &'static str {
    include_str!("../../../src-tauri/resources/model-manifest.json")
}

/// The manifest schema this build reads.
pub const SCHEMA_VERSION: u32 = 3;

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
        if self.schema_version != SCHEMA_VERSION
            || self.served_model_name.is_empty()
            || self.variants.is_empty()
        {
            return Err(invalid());
        }
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        let mut sizes = HashSet::new();
        for variant in &self.variants {
            if !is_safe_id(&variant.id)
                || !ids.insert(variant.id.as_str())
                || variant.model_id.is_empty()
                || variant.label.is_empty()
                // Exactly one file per variant. A variant that could name
                // more would be one that could make a user download
                // something Intern never loads.
                || variant.files.len() != 1
            {
                return Err(invalid());
            }
            for file in &variant.files {
                if !is_safe_filename(&file.name)
                    || !file.url.starts_with("https://")
                    || !pins_an_immutable_revision(&file.url)
                    || file.size == 0
                    // Names and sizes are unique across variants: two
                    // variants installed side by side never share a file,
                    // and a file a person picks is matched to a variant by
                    // its size before it is hashed.
                    || !names.insert(file.name.as_str())
                    || !sizes.insert(file.size)
                    || file.sha256.len() != 64
                    || !file
                        .sha256
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                {
                    return Err(invalid());
                }
            }
            if variant
                .files
                .iter()
                .filter(|file| file.role == ModelRole::Model)
                .count()
                != 1
            {
                return Err(invalid());
            }
        }
        if !ids.contains(self.default.as_str()) {
            return Err(invalid());
        }
        Ok(())
    }

    /// The variant `id` names, resolved; `None` for an id the manifest does
    /// not have.
    pub fn select(&self, id: &str) -> Option<SelectedModel> {
        self.variants
            .iter()
            .find(|variant| variant.id == id)
            .map(|variant| self.resolve(variant))
    }

    /// The default variant, resolved.
    pub fn default_model(&self) -> SelectedModel {
        self.select(&self.default)
            .unwrap_or_else(|| self.resolve(&self.variants[0]))
    }

    /// The variant `id` names, or the default when it names none: a
    /// settings file from a build with other variants still starts.
    pub fn select_or_default(&self, id: &str) -> SelectedModel {
        self.select(id).unwrap_or_else(|| self.default_model())
    }

    /// The variant whose model file is exactly `size` bytes long, the first
    /// check a file a person picked must pass before it is hashed.
    pub fn variant_with_size(&self, size: u64) -> Option<SelectedModel> {
        self.variants
            .iter()
            .find(|variant| variant.files.iter().any(|file| file.size == size))
            .map(|variant| self.resolve(variant))
    }

    /// The variant whose model file has this SHA-256: which pinned model a
    /// recording or an evaluation was made with, if any.
    pub fn variant_with_sha256(&self, sha256: &str) -> Option<SelectedModel> {
        let sha256 = sha256.to_ascii_lowercase();
        self.variants
            .iter()
            .find(|variant| variant.files.iter().any(|file| file.sha256 == sha256))
            .map(|variant| self.resolve(variant))
    }

    fn resolve(&self, variant: &ModelVariant) -> SelectedModel {
        SelectedModel {
            variant: variant.id.clone(),
            model_id: variant.model_id.clone(),
            label: variant.label.clone(),
            served_model_name: self.served_model_name.clone(),
            files: variant.files.clone(),
        }
    }
}

/// A variant id: lowercase letters, digits and dashes, as settings store it.
fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Whether a download URL names a revision that cannot move.
///
/// Hugging Face serves `/resolve/<ref>/<path>`, and a branch name is a ref
/// that moves: an upstream re-upload changes the bytes under a digest the
/// manifest has already promised, and the download then costs a full model
/// (up to 1.07 GiB) on every retry and fails the same check every time with nothing to say why.
/// A 40-character commit id cannot move, so the digest either matches on the
/// first attempt or the pin itself is wrong.
fn pins_an_immutable_revision(url: &str) -> bool {
    url.split('/').any(|segment| {
        segment.len() == 40
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
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

    #[test]
    fn the_embedded_manifest_is_valid_and_names_the_bonsai_variants() {
        let manifest = ModelManifest::embedded().unwrap();
        assert_eq!(manifest.schema_version, 3);
        assert_eq!(manifest.default, "bonsai-4b");
        let ids = manifest
            .variants
            .iter()
            .map(|variant| variant.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["bonsai-4b", "bonsai-2b"]);
        for variant in &manifest.variants {
            for file in &variant.files {
                assert!(
                    file.url.starts_with("https://huggingface.co/prism-ml/"),
                    "{}",
                    file.url
                );
            }
        }
    }

    /// The exact files the native Windows evaluation measured
    /// (`bench/reports/bonsai-native-windows-2026-10-18`), and no other
    /// quantization from the same repositories.
    #[test]
    fn the_variants_pin_the_evaluated_pq2_0_files() {
        let manifest = ModelManifest::embedded().unwrap();
        let four = manifest.select("bonsai-4b").unwrap();
        let model = four.model().unwrap();
        assert_eq!(model.name, "Ternary-Bonsai-2-4B-PQ2_0.gguf");
        assert_eq!(model.size, 1_146_465_120);
        assert_eq!(
            model.sha256,
            "d406b1e912c0d2a4c124ad10bf38a76ae662fdc4d24b176451403dc7b7f30409"
        );
        let two = manifest.select("bonsai-2b").unwrap();
        let model = two.model().unwrap();
        assert_eq!(model.name, "Ternary-Bonsai-2-2B-PQ2_0.gguf");
        assert_eq!(model.size, 517_312_992);
        assert_eq!(
            model.sha256,
            "d5f01a4ecaf49239653308ee732ad1a0c5f802c0ddb997549fc7b694c8b45928"
        );
        assert_eq!(four.served_model_name, "intern-local");
        assert_eq!(two.served_model_name, "intern-local");
    }

    #[test]
    fn every_variant_fits_a_sixteen_gigabyte_laptop() {
        let manifest = ModelManifest::embedded().unwrap();
        // Everything a user must download before Intern will name a
        // document, whichever variant they chose. Nothing here may be a
        // file the engine cannot load.
        for variant in &manifest.variants {
            let selected = manifest.select(&variant.id).unwrap();
            assert_eq!(selected.files.len(), 1);
            assert!(
                selected.total_bytes() < 2 * 1024 * 1024 * 1024,
                "first-run download must stay under 2 GiB, got {} bytes for {}",
                selected.total_bytes(),
                variant.id
            );
        }
    }

    #[test]
    fn a_variant_naming_more_than_the_text_model_is_refused() {
        let base = ModelManifest::embedded().unwrap();
        let mut extra = base.clone();
        let mut second = extra.variants[0].files[0].clone();
        second.name = "mmproj.gguf".into();
        second.size += 1;
        extra.variants[0].files.push(second);
        assert!(extra.validate().is_err());
    }

    #[test]
    fn a_default_that_names_no_variant_or_a_repeated_variant_is_refused() {
        let base = ModelManifest::embedded().unwrap();
        let mut missing = base.clone();
        missing.default = "bonsai-8b".into();
        assert!(missing.validate().is_err());

        let mut repeated = base.clone();
        repeated.variants[1].id = repeated.variants[0].id.clone();
        assert!(repeated.validate().is_err());

        let mut same_size = base.clone();
        same_size.variants[1].files[0].size = same_size.variants[0].files[0].size;
        assert!(same_size.validate().is_err());

        let mut none = base.clone();
        none.variants.clear();
        assert!(none.validate().is_err());

        let mut old = base;
        old.schema_version = 2;
        assert!(old.validate().is_err());
    }

    #[test]
    fn the_previous_schema_is_refused() {
        let qwen = r#"{"schema_version":2,"model_id":"qwen","served_model_name":"intern-local","files":[]}"#;
        assert!(ModelManifest::parse(qwen).is_err());
    }

    #[test]
    fn a_variant_is_found_by_id_size_or_digest() {
        let manifest = ModelManifest::embedded().unwrap();
        assert_eq!(manifest.default_model().variant, "bonsai-4b");
        assert_eq!(manifest.select_or_default("bonsai-2b").variant, "bonsai-2b");
        assert_eq!(manifest.select_or_default("qwen").variant, "bonsai-4b");
        assert!(manifest.select("qwen").is_none());
        assert_eq!(
            manifest.variant_with_size(517_312_992).unwrap().variant,
            "bonsai-2b"
        );
        assert!(manifest.variant_with_size(1_280_835_840).is_none());
        assert_eq!(
            manifest
                .variant_with_sha256(
                    "D406B1E912C0D2A4C124AD10BF38A76AE662FDC4D24B176451403DC7B7F30409"
                )
                .unwrap()
                .variant,
            "bonsai-4b"
        );
        assert!(
            manifest
                .variant_with_sha256(
                    "aaf42c8b7c3cab2bf3d69c355048d4a0ee9973d48f16c731c0520ee914699223"
                )
                .is_none(),
            "the previous Qwen model is not a variant"
        );
    }

    /// A moving upstream ref makes the pinned digest a promise the manifest
    /// cannot keep: an upstream re-upload changes the bytes under it, and the
    /// download then costs a full model (up to 1.07 GiB) on every retry and
    /// fails the same check each time with nothing to say why.
    #[test]
    fn manifest_urls_pin_an_immutable_revision() {
        let base = ModelManifest::embedded().unwrap();
        for variant in &base.variants {
            for file in &variant.files {
                assert!(!file.url.contains("/resolve/main/"), "{}", file.url);
            }
        }

        let mut moving = base;
        moving.variants[0].files[0].url =
            "https://huggingface.co/prism-ml/Ternary-Bonsai-2-4B-gguf/resolve/main/Ternary-Bonsai-2-4B-PQ2_0.gguf"
                .into();
        assert!(moving.validate().is_err());
    }

    #[test]
    fn path_traversal_bad_ids_and_bad_digests_are_refused() {
        let base = ModelManifest::embedded().unwrap();
        let mut traversal = base.clone();
        traversal.variants[0].files[0].name = "../evil.gguf".into();
        assert!(traversal.validate().is_err());

        let mut digest = base.clone();
        digest.variants[0].files[0].sha256 = "NOTHEX".into();
        assert!(digest.validate().is_err());

        let mut scheme = base.clone();
        scheme.variants[0].files[0].url = "http://huggingface.co/x".into();
        assert!(scheme.validate().is_err());

        let mut id = base;
        id.variants[1].id = "../2b".into();
        assert!(id.validate().is_err());
    }
}
