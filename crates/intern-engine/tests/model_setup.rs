use std::{fs, path::Path, sync::Mutex};

use intern_engine::{
    AnalysisTelemetry, DateRole, DigestBudget, DocumentAnalysis, Evidence, ModelProposal,
    PartyRelation, distill, engine::finish, validate,
};
use intern_engine::{
    ModelFile, ModelManifest, ModelRole, ModelTier, PromptProfile, RetiredFile, TierModel,
    download::{CancellationToken, DISK_RESERVE_BYTES, DiskSpace, SetupStage},
    error::EngineErrorCode as ModelErrorCode,
    setup::{
        ExistingModelSelection, SetupOperationGate, install_existing_model_files,
        match_existing_model_files, remove_retired_files, semantic_probes, validate_semantic_probe,
    },
};
use tempfile::tempdir;

#[derive(Clone, Copy)]
struct FixedDisk(u64);

impl DiskSpace for FixedDisk {
    fn available_bytes(&self, _path: &Path) -> intern_engine::EngineResult<u64> {
        Ok(self.0)
    }
}

/// The bytes each tier's test model holds; their digests were computed once
/// and pasted in. The two differ in length, as the pinned models do.
const FAST_BYTES: &[u8] = b"model-bytes";
const FAST_SHA256: &str = "357e5d6fafa34d27360fec24b4326d3534905e33c6acdee60198fb078b7b79e5";
const ACCURATE_BYTES: &[u8] = b"the pinned model bytes";
const ACCURATE_SHA256: &str = "b1f81de2183585b9793c38b27cdd46842ab24247c628b6fbbda1200b9a25ce99";

fn tier(tier: ModelTier, name: &str, bytes: &[u8], sha256: &str) -> TierModel {
    TierModel {
        tier,
        model_id: format!("test-{}", tier.as_str()),
        display_name: format!("Test {}", tier.as_str()),
        prompt_profile: Some(PromptProfile::Bonsai2),
        files: vec![ModelFile {
            name: name.into(),
            role: ModelRole::Model,
            url: format!("https://example.invalid/0123456789abcdef0123456789abcdef01234567/{name}"),
            size: bytes.len() as u64,
            sha256: sha256.into(),
        }],
    }
}

fn manifest() -> ModelManifest {
    ModelManifest {
        schema_version: 3,
        served_model_name: "intern-local".into(),
        models: vec![
            tier(ModelTier::Fast, "fast.gguf", FAST_BYTES, FAST_SHA256),
            tier(
                ModelTier::Accurate,
                "accurate.gguf",
                ACCURATE_BYTES,
                ACCURATE_SHA256,
            ),
        ],
        retired_files: Vec::new(),
    }
}

#[test]
fn setup_gate_cancels_the_active_token_and_allows_a_fresh_resume_only_after_finish() {
    let gate = SetupOperationGate::default();
    let first = gate.begin().unwrap();

    let busy = gate.begin().err().expect("second setup must be rejected");
    assert_eq!(busy.code(), ModelErrorCode::SetupBusy);
    assert!(gate.cancel());
    assert!(first.is_canceled());

    gate.finish();
    let resumed = gate.begin().unwrap();
    assert!(!resumed.is_canceled());
}

#[test]
fn cancel_after_setup_succeeded_is_a_no_op() {
    let gate = SetupOperationGate::default();
    let cancellation = gate.begin().unwrap();
    // The operation has produced its result: the model is installed, started,
    // and verified. A cancel arriving in the moment before the gate is
    // released would otherwise stop the server that was just started and
    // leave the interface saying the model is ready with nothing running.
    gate.settle();
    assert!(!gate.cancel());
    assert!(!cancellation.is_canceled());
    gate.finish();
    assert!(!gate.cancel());
}

#[test]
fn an_existing_model_file_is_validated_and_published_with_progress() {
    let temp = tempdir().unwrap();
    let model = temp.path().join("selected-model.gguf");
    fs::write(&model, FAST_BYTES).unwrap();
    let destination = temp.path().join("installed");
    let progress = Mutex::new(Vec::new());
    let mut heard = Vec::new();

    let tiers = install_existing_model_files(
        &manifest(),
        &ExistingModelSelection {
            model_paths: vec![model],
        },
        &destination,
        &FixedDisk(DISK_RESERVE_BYTES + 100),
        &CancellationToken::new(),
        |event| progress.lock().unwrap().push(event),
        |tier| heard.push(tier),
    )
    .unwrap();

    assert_eq!(tiers, [ModelTier::Fast]);
    assert_eq!(heard, [ModelTier::Fast]);
    assert_eq!(fs::read(destination.join("fast.gguf")).unwrap(), FAST_BYTES);
    assert!(!destination.join("accurate.gguf").exists());
    let events = progress.into_inner().unwrap();
    assert_eq!(events.last().unwrap().stage, SetupStage::Complete);
    assert_eq!(events.last().unwrap().completed_bytes, 11);
    assert_eq!(events.last().unwrap().total_bytes, 11);
}

/// Both tiers chosen at once, in either order: each lands under its own
/// pinned name, the fast one first, and the progress covers both.
#[test]
fn both_tiers_install_from_files_chosen_in_any_order() {
    let temp = tempdir().unwrap();
    let fast = temp.path().join("small.gguf");
    let accurate = temp.path().join("large.gguf");
    fs::write(&fast, FAST_BYTES).unwrap();
    fs::write(&accurate, ACCURATE_BYTES).unwrap();
    let destination = temp.path().join("installed");
    let progress = Mutex::new(Vec::new());

    let tiers = install_existing_model_files(
        &manifest(),
        &ExistingModelSelection {
            model_paths: vec![accurate, fast],
        },
        &destination,
        &FixedDisk(u64::MAX),
        &CancellationToken::new(),
        |event| progress.lock().unwrap().push(event),
        |_| {},
    )
    .unwrap();

    assert_eq!(tiers, [ModelTier::Fast, ModelTier::Accurate]);
    assert_eq!(fs::read(destination.join("fast.gguf")).unwrap(), FAST_BYTES);
    assert_eq!(
        fs::read(destination.join("accurate.gguf")).unwrap(),
        ACCURATE_BYTES
    );
    let events = progress.into_inner().unwrap();
    let total = (FAST_BYTES.len() + ACCURATE_BYTES.len()) as u64;
    assert!(events.iter().all(|event| event.total_bytes == total));
    assert_eq!(events.last().unwrap().completed_bytes, total);
}

/// A file is recognised by its length and accepted only by its digest. One
/// that matches no pinned length, two for the same tier, or an empty choice
/// are refused before anything is copied; one of the right length and the
/// wrong bytes is refused by its digest, and the tier before it stays
/// installed.
#[test]
fn a_file_that_is_not_a_pinned_model_is_refused() {
    let temp = tempdir().unwrap();
    let manifest = manifest();
    let unknown = temp.path().join("other.gguf");
    fs::write(&unknown, b"not one of the pinned models").unwrap();
    let fast = temp.path().join("small.gguf");
    fs::write(&fast, FAST_BYTES).unwrap();
    let fast_again = temp.path().join("small-copy.gguf");
    fs::write(&fast_again, FAST_BYTES).unwrap();
    let missing = temp.path().join("missing.gguf");

    for paths in [
        vec![],
        vec![unknown.clone()],
        vec![fast.clone(), fast_again],
        vec![missing],
        vec![temp.path().to_path_buf()],
        vec![fast.clone(), unknown.clone(), unknown],
    ] {
        let selection = ExistingModelSelection { model_paths: paths };
        assert_eq!(
            match_existing_model_files(&manifest, &selection)
                .unwrap_err()
                .code(),
            ModelErrorCode::ModelFileInvalid,
            "{selection:?}"
        );
    }

    // The accurate tier's length, other bytes.
    let forged = temp.path().join("forged.gguf");
    fs::write(&forged, b"the pinned model bytez").unwrap();
    let destination = temp.path().join("installed");
    let mut heard = Vec::new();
    let error = install_existing_model_files(
        &manifest,
        &ExistingModelSelection {
            model_paths: vec![forged, fast],
        },
        &destination,
        &FixedDisk(u64::MAX),
        &CancellationToken::new(),
        |_| {},
        |tier| heard.push(tier),
    )
    .unwrap_err();
    assert_eq!(error.code(), ModelErrorCode::ModelFileInvalid);
    assert_eq!(heard, [ModelTier::Fast], "the fast tier went in first");
    assert!(destination.join("fast.gguf").is_file());
    assert!(!destination.join("accurate.gguf").exists());
}

/// Cancelling must leave nothing half-installed, and a second attempt must
/// still work. With one file there is no partial set to resume, so the
/// guarantee is that the destination stays clean and the retry succeeds.
#[test]
fn a_canceled_install_publishes_nothing_and_can_be_retried() {
    let temp = tempdir().unwrap();
    let model = temp.path().join("selected-model.gguf");
    fs::write(&model, FAST_BYTES).unwrap();
    let destination = temp.path().join("installed");
    let selection = ExistingModelSelection {
        model_paths: vec![model],
    };
    let cancellation = CancellationToken::new();
    cancellation.cancel();

    let error = install_existing_model_files(
        &manifest(),
        &selection,
        &destination,
        &FixedDisk(u64::MAX),
        &cancellation,
        |_| {},
        |_| {},
    )
    .unwrap_err();

    assert_eq!(error.code(), ModelErrorCode::DownloadCanceled);
    assert!(!destination.join("fast.gguf").exists());

    install_existing_model_files(
        &manifest(),
        &selection,
        &destination,
        &FixedDisk(u64::MAX),
        &CancellationToken::new(),
        |_| {},
        |_| {},
    )
    .unwrap();
    assert_eq!(fs::read(destination.join("fast.gguf")).unwrap(), FAST_BYTES);
}

#[test]
fn the_test_manifest_is_a_valid_tiered_manifest() {
    manifest()
        .validate()
        .expect("the helper builds what Intern accepts");
}

fn analysis(
    probe: &intern_engine::setup::SemanticProbe,
    party: Option<&str>,
    date: Option<&str>,
) -> DocumentAnalysis {
    let digest = distill(&probe.document, DigestBudget::default());
    let proposal = ModelProposal {
        document_type: Some("Notice of Calibration".into()),
        document_date: date.map(str::to_owned),
        date_role: date.map(|_| DateRole::Notice),
        parties: party.map(str::to_owned).into_iter().collect(),
        party_relation: PartyRelation::To,
        description: format!(
            "Notice of calibration confirming that the local text model path is working for {}.",
            party.unwrap_or("this installation")
        ),
        confidence: 0.99,
        needs_review: false,
        evidence: Evidence {
            date: Some("Date of this Notice: January 2, 2024".into()),
            document_type: Some("NOTICE OF CALIBRATION".into()),
            parties: party
                .map(|value| format!("To: {value}"))
                .into_iter()
                .collect(),
        },
        facts: None,
    };
    let outcome = validate(proposal, &digest);
    finish(outcome, &digest, "pdf", &[], AnalysisTelemetry::default())
}

#[test]
fn the_self_test_requires_the_model_to_read_the_calibration_document() {
    let probes = semantic_probes().unwrap();
    assert_eq!(probes.len(), 1);
    assert!(probes[0].document.page_image.is_none());

    validate_semantic_probe(
        &probes[0],
        &analysis(
            &probes[0],
            Some("Northstar Calibration Holdings LLC"),
            Some("2024-01-02"),
        ),
    )
    .unwrap();

    // Reading the document but not its defining date is not good enough.
    assert_eq!(
        validate_semantic_probe(
            &probes[0],
            &analysis(&probes[0], Some("Northstar Calibration Holdings LLC"), None)
        )
        .unwrap_err()
        .code(),
        ModelErrorCode::ModelSelfTestFailed
    );
    // Neither is producing a date without ever naming what the document says.
    assert_eq!(
        validate_semantic_probe(&probes[0], &analysis(&probes[0], None, Some("2024-01-02")))
            .unwrap_err()
            .code(),
        ModelErrorCode::ModelSelfTestFailed
    );
}

/// Upgrading from a single-model build leaves its model behind. It is
/// removed only when it is exactly the file that build pinned - name, length
/// and digest - and nothing else in the folder is touched: not a file of the
/// same name that differs, not a partial download, not a pinned model.
#[test]
fn a_retired_model_is_removed_only_when_it_is_exactly_the_retired_file() {
    let temp = tempdir().unwrap();
    let mut manifest = manifest();
    manifest.retired_files = vec![RetiredFile {
        name: "old.gguf".into(),
        size: FAST_BYTES.len() as u64,
        sha256: FAST_SHA256.into(),
    }];
    let folder = temp.path();
    fs::write(folder.join("fast.gguf"), FAST_BYTES).unwrap();
    fs::write(folder.join("old.gguf.partial"), FAST_BYTES).unwrap();
    fs::write(folder.join("unrelated.gguf"), FAST_BYTES).unwrap();

    // Nothing there: nothing to do.
    assert!(remove_retired_files(&manifest, folder).is_empty());

    // The right name and length, other bytes: not the file Intern put there.
    fs::write(folder.join("old.gguf"), b"model-bytez").unwrap();
    assert!(remove_retired_files(&manifest, folder).is_empty());
    // The right name, another length.
    fs::write(folder.join("old.gguf"), b"model-bytes and more").unwrap();
    assert!(remove_retired_files(&manifest, folder).is_empty());
    assert!(folder.join("old.gguf").exists());

    // Exactly the retired file.
    fs::write(folder.join("old.gguf"), FAST_BYTES).unwrap();
    assert_eq!(remove_retired_files(&manifest, folder), ["old.gguf"]);
    assert!(!folder.join("old.gguf").exists());
    for kept in ["fast.gguf", "old.gguf.partial", "unrelated.gguf"] {
        assert!(folder.join(kept).exists(), "{kept} was left alone");
    }
    assert!(remove_retired_files(&manifest, folder).is_empty());
}
