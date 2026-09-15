//! Identity authorization is a host concern, but no processing path may bypass it.
use crate::pipeline::PipelineResult;
use intern_core::OwnedFileSnapshot;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionStage {
    Enqueue,
    Extract,
    Analyze,
    Apply,
}

/// Provider proof attached to one admission check. When present, the snapshot
/// is the only path extraction may read and remains owned until extraction
/// returns. Dropping the evidence removes the private copy.
pub struct AdmissionEvidence {
    verified_hash: Option<String>,
    snapshot: Option<OwnedFileSnapshot>,
}

impl AdmissionEvidence {
    pub fn local() -> Self {
        Self {
            verified_hash: None,
            snapshot: None,
        }
    }

    pub fn verified(hash: String) -> Self {
        Self {
            verified_hash: Some(hash),
            snapshot: None,
        }
    }

    pub fn verified_snapshot(hash: String, snapshot: OwnedFileSnapshot) -> Self {
        Self {
            verified_hash: Some(hash),
            snapshot: Some(snapshot),
        }
    }

    pub fn verified_hash(&self) -> Option<&str> {
        self.verified_hash.as_deref()
    }

    pub fn extraction_path<'a>(&'a self, original: &'a Path) -> Option<&'a Path> {
        match (&self.verified_hash, &self.snapshot) {
            (None, None) => Some(original),
            (Some(_), Some(snapshot)) => Some(snapshot.path()),
            _ => None,
        }
    }
}

pub trait AdmissionGuard: Send + Sync {
    /// Local evidence has no provider hash. Verified evidence carries the
    /// SHA-256 of the exact provider-matched bytes and may own the only source
    /// path extraction is permitted to read.
    fn authorize(&self, path: &Path, stage: AdmissionStage) -> PipelineResult<AdmissionEvidence>;
    /// Called only after an analysis actually completed, never at selection.
    fn processed(&self, _path: &Path, _hash: &str) {}
}
#[derive(Default)]
pub struct LocalAdmission;
impl AdmissionGuard for LocalAdmission {
    fn authorize(&self, _path: &Path, _stage: AdmissionStage) -> PipelineResult<AdmissionEvidence> {
        Ok(AdmissionEvidence::local())
    }
}
