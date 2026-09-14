//! Identity authorization is a host concern, but no processing path may bypass it.
use crate::pipeline::PipelineResult;
use std::{fs, path::{Path, PathBuf}};

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
    snapshot: Option<PathBuf>,
}

impl AdmissionEvidence {
    pub fn local() -> Self {
        Self { verified_hash: None, snapshot: None }
    }

    pub fn verified(hash: String) -> Self {
        Self { verified_hash: Some(hash), snapshot: None }
    }

    pub fn verified_snapshot(hash: String, snapshot: PathBuf) -> Self {
        Self { verified_hash: Some(hash), snapshot: Some(snapshot) }
    }

    pub fn verified_hash(&self) -> Option<&str> {
        self.verified_hash.as_deref()
    }

    pub fn extraction_path<'a>(&'a self, original: &'a Path) -> &'a Path {
        self.snapshot.as_deref().unwrap_or(original)
    }
}

impl Drop for AdmissionEvidence {
    fn drop(&mut self) {
        if let Some(path) = self.snapshot.take() {
            if let Ok(metadata) = fs::metadata(&path) {
                let mut permissions = metadata.permissions();
                if permissions.readonly() {
                    permissions.set_readonly(false);
                    let _ = fs::set_permissions(&path, permissions);
                }
            }
            let _ = fs::remove_file(path);
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
