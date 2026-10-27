use std::{path::PathBuf, sync::Mutex};

use crate::domain::{DocumentAnalysis, DocumentSource, PageOrigin, SourcePage};

use crate::download::{CancellationToken, DiskSpace, SetupProgress, install_selected_file};
use crate::error::{
    EngineError as ModelError, EngineErrorCode as ModelErrorCode, EngineResult as ModelResult,
};
use crate::manifest::{ModelFile, ModelManifest, ModelRole, ModelTier};

/// A phrase that appears nowhere except the calibration document, so a model
/// that echoes it has actually read the text it was given.
const TEXT_MARKER: &str = "Northstar Calibration";

#[derive(Default)]
pub struct SetupOperationGate {
    active: Mutex<Option<Operation>>,
}

struct Operation {
    cancellation: CancellationToken,
    /// The work has produced its result and cancelling can no longer change
    /// it. The gate is still held, so no second operation may start until the
    /// outcome has been published.
    settled: bool,
}

impl SetupOperationGate {
    pub fn begin(&self) -> ModelResult<CancellationToken> {
        let mut active = self.active.lock().map_err(|_| setup_busy())?;
        if active.is_some() {
            return Err(setup_busy());
        }
        let cancellation = CancellationToken::new();
        *active = Some(Operation {
            cancellation: cancellation.clone(),
            settled: false,
        });
        Ok(cancellation)
    }

    /// Cancels the operation in flight, and says whether there was one to
    /// cancel. An operation that has already settled is not one: the model it
    /// installed is started and verified, and stopping it now would leave the
    /// interface saying it is ready with nothing running behind it.
    pub fn cancel(&self) -> bool {
        let Ok(active) = self.active.lock() else {
            return false;
        };
        match active.as_ref() {
            Some(operation) if !operation.settled => {
                operation.cancellation.cancel();
                true
            }
            _ => false,
        }
    }

    /// Records that the work is over, before its outcome is published.
    pub fn settle(&self) {
        if let Ok(mut active) = self.active.lock()
            && let Some(operation) = active.as_mut()
        {
            operation.settled = true;
        }
    }

    pub fn finish(&self) {
        if let Ok(mut active) = self.active.lock() {
            active.take();
        }
    }
}

/// Model files the user already has on disk, offered instead of downloading:
/// any of the pinned tiers' files, in any order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExistingModelSelection {
    pub model_paths: Vec<PathBuf>,
}

/// Which pinned file each chosen path is, before anything is copied.
///
/// A chosen file is recognised by its length - the pinned files all differ
/// in it - and accepted only once its digest matches too, which
/// [`install_selected_file`] checks. A file of no pinned length, two files
/// for one tier, or no file at all is refused before a byte moves, so a
/// person who picked the wrong file is told at once rather than after a
/// gigabyte was read.
pub fn match_existing_model_files<'a>(
    manifest: &'a ModelManifest,
    selection: &ExistingModelSelection,
) -> ModelResult<Vec<(ModelTier, &'a ModelFile, PathBuf)>> {
    if selection.model_paths.is_empty() || selection.model_paths.len() > ModelTier::ALL.len() {
        return Err(file_invalid());
    }
    let mut matched: Vec<(ModelTier, &ModelFile, PathBuf)> = Vec::new();
    for path in &selection.model_paths {
        let length = std::fs::metadata(path)
            .ok()
            .filter(std::fs::Metadata::is_file)
            .map(|metadata| metadata.len())
            .ok_or_else(file_invalid)?;
        let (tier, file) = manifest
            .tiers()
            .flat_map(|model| model.files.iter().map(move |file| (model.tier, file)))
            .find(|(_, file)| file.role == ModelRole::Model && file.size == length)
            .ok_or_else(file_invalid)?;
        if matched.iter().any(|(already, _, _)| *already == tier) {
            return Err(file_invalid());
        }
        matched.push((tier, file, path.clone()));
    }
    // Installed in the order setup downloads, so the tier that makes Intern
    // usable is in place first.
    matched.sort_by_key(|(tier, _, _)| *tier);
    Ok(matched)
}

/// Installs the chosen files, each only once its length and digest match
/// the file its tier pins. `installed` hears of every tier as it is put in
/// place, so a second file that fails its check does not undo the first.
pub fn install_existing_model_files<D, F, I>(
    manifest: &ModelManifest,
    selection: &ExistingModelSelection,
    destination_directory: &std::path::Path,
    disk: &D,
    cancellation: &CancellationToken,
    mut progress: F,
    mut installed: I,
) -> ModelResult<Vec<ModelTier>>
where
    D: DiskSpace,
    F: FnMut(SetupProgress),
    I: FnMut(ModelTier),
{
    let matched = match_existing_model_files(manifest, selection)?;
    let total = matched.iter().map(|(_, file, _)| file.size).sum::<u64>();
    let mut completed_before = 0_u64;
    let mut tiers = Vec::with_capacity(matched.len());
    for (tier, expected, selected) in matched {
        if cancellation.is_canceled() {
            return Err(download_canceled());
        }
        let offset = completed_before;
        install_selected_file(
            &selected,
            expected,
            destination_directory,
            disk,
            cancellation,
            |event| {
                progress(SetupProgress {
                    stage: event.stage,
                    completed_bytes: offset + event.completed_bytes,
                    total_bytes: total,
                });
            },
        )?;
        installed(tier);
        tiers.push(tier);
        completed_before += expected.size;
    }
    Ok(tiers)
}

/// A document the installed model must be able to file correctly before Intern
/// will use it.
///
/// The probe exercises the shipping path end to end - the evidence index and
/// retrieval, the real prompt, the grammar, fact validation, and naming - so
/// a model that installs but cannot actually do the job is caught at setup
/// rather than on the user's first document.
pub struct SemanticProbe {
    pub document: DocumentSource,
    expected_marker: &'static str,
}

pub fn semantic_probes() -> ModelResult<Vec<SemanticProbe>> {
    Ok(vec![SemanticProbe {
        document: DocumentSource::from_pages(vec![SourcePage::new(
            1,
            format!(
                "NOTICE OF CALIBRATION\n\nDate of this Notice: January 2, 2024\n\n\
                 To: {TEXT_MARKER} Holdings LLC\n\n\
                 This notice confirms that the local text model path is working and that \
                 evidence quoted back from this document can be checked against it.\n"
            ),
            PageOrigin::PlainText,
        )]),
        expected_marker: TEXT_MARKER,
    }])
}

pub fn validate_semantic_probe(
    probe: &SemanticProbe,
    analysis: &DocumentAnalysis,
) -> ModelResult<()> {
    let marker = probe.expected_marker.to_lowercase();
    let names_marker = |values: &mut dyn Iterator<Item = &String>| {
        let mut found = false;
        for value in values {
            found |= value.to_lowercase().contains(&marker);
        }
        found
    };
    // The model itself must have named what the document says, as a party
    // or in its description: the evidence pipeline reads an addressee from
    // the document's "To:" field when a reply names none, and quotes the
    // lines a reply cites, neither of which proves the model read anything.
    let the_model_named_it = analysis.model_proposal.as_ref().is_none_or(|reply| {
        names_marker(
            &mut reply
                .parties
                .iter()
                .chain(std::iter::once(&reply.description))
                .chain(
                    reply
                        .facts
                        .iter()
                        .flat_map(|facts| facts.parties.iter().map(|party| &party.name)),
                ),
        )
    });
    let read_the_document = the_model_named_it
        && names_marker(
            &mut analysis
                .proposal
                .parties
                .iter()
                .chain(std::iter::once(&analysis.description))
                .chain(analysis.proposal.evidence.parties.iter()),
        );
    let found_the_date = analysis.proposal.document_date.as_deref() == Some("2024-01-02");
    if read_the_document && found_the_date {
        Ok(())
    } else {
        Err(self_test_failed())
    }
}

const fn setup_busy() -> ModelError {
    ModelError::new(
        ModelErrorCode::SetupBusy,
        "a model setup operation is already active",
    )
}

const fn download_canceled() -> ModelError {
    ModelError::new(ModelErrorCode::DownloadCanceled, "model setup was canceled")
}

const fn file_invalid() -> ModelError {
    ModelError::new(
        ModelErrorCode::ModelFileInvalid,
        "the chosen file is not a model file Intern pins",
    )
}

const fn self_test_failed() -> ModelError {
    ModelError::new(
        ModelErrorCode::ModelSelfTestFailed,
        "local model semantic self-test failed",
    )
}
