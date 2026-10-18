//! Portable qualification inputs; no scripts, code download, or guest-host listener.
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub path: String,
    pub sha256: String,
}
pub fn relative(root: &Path, name: &str) -> Result<PathBuf, String> {
    let p = Path::new(name);
    if p.as_os_str().is_empty()
        || p.components().any(|c| !matches!(c, Component::Normal(_)))
        || name.contains(':')
        || name.contains('\\')
    {
        return Err("asset path must be relative with normal slash-separated components".into());
    }
    let mut full = root.to_owned();
    for part in p.components() {
        full.push(part);
        if std::fs::symlink_metadata(&full)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("symlink asset rejected".into());
        }
    }
    Ok(full)
}
pub fn file_hash(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub fn verified_assets(root: &Path, assets: &[Asset]) -> Result<(), String> {
    if assets.is_empty() || assets.len() > 1024 {
        return Err("invalid asset count".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for a in assets {
        if !seen.insert(a.path.to_lowercase()) {
            return Err("duplicate asset".into());
        }
        let path = relative(root, &a.path)?;
        if a.sha256.len() != 64 || file_hash(&path)? != a.sha256 {
            return Err(format!("asset SHA-256 mismatch: {}", a.path));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifies_local_bytes_before_any_execution() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("a"), b"abc").unwrap();
        let a = Asset {
            path: "a".into(),
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
        };
        verified_assets(d.path(), &[a.clone()]).unwrap();
        std::fs::write(d.path().join("a"), b"changed").unwrap();
        assert!(verified_assets(d.path(), &[a]).is_err());
    }
    #[test]
    fn refuses_parent_paths_and_duplicate_assets() {
        let d = tempfile::tempdir().unwrap();
        let a = Asset {
            path: "../a".into(),
            sha256: "0".repeat(64),
        };
        assert!(verified_assets(d.path(), &[a]).is_err());
        std::fs::write(d.path().join("a"), b"abc").unwrap();
        let a = Asset {
            path: "a".into(),
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
        };
        assert!(verified_assets(d.path(), &[a.clone(), a]).is_err());
    }
}

pub fn report_completed(report: &serde_json::Value, id: &str) -> bool {
    report["summary"]["completed"] == 1
        && report["records"]
            .as_array()
            .is_some_and(|r| r.len() == 1 && r[0]["id"] == id && r[0]["status"] == "completed")
}
#[cfg(test)]
mod completion_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn model_failure_and_wrong_case_cannot_mark_native_smoke_successful() {
        assert!(!report_completed(
            &json!({"summary":{"completed":0},"records":[{"id":"invoice","status":"model_failed"}]}),
            "invoice"
        ));
        assert!(!report_completed(
            &json!({"summary":{"completed":1},"records":[{"id":"other","status":"completed"}]}),
            "invoice"
        ));
    }
    #[test]
    fn one_completed_expected_case_is_accepted() {
        assert!(report_completed(
            &json!({"summary":{"completed":1},"records":[{"id":"invoice","status":"completed"}]}),
            "invoice"
        ));
    }
}

pub fn require_complete(attempted: usize, planned: usize) -> Result<(), String> {
    if attempted == planned {
        Ok(())
    } else {
        Err("incomplete batch; remaining cases are unmeasured".into())
    }
}
pub fn final_security_check(
    baseline: u64,
    latest: Result<(u64, Vec<u8>), String>,
    out: &Path,
) -> Result<(), String> {
    let (id, xml) = latest?;
    if id != baseline {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("defender-final-alert.xml"))
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(&xml).and_then(|_| f.sync_all())
            })
            .map_err(|e| e.to_string())?;
        return Err("new Defender detection at final check".into());
    }
    Ok(())
}
#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    #[test]
    fn incomplete_batch_returns_failure() {
        assert!(require_complete(1, 2).is_err());
        assert!(require_complete(2, 2).is_ok());
    }
    #[test]
    fn late_detection_is_preserved_and_blocks_success() {
        let d = tempfile::tempdir().unwrap();
        assert!(final_security_check(10, Ok((11, b"new detection".to_vec())), d.path()).is_err());
        assert_eq!(
            std::fs::read(d.path().join("defender-final-alert.xml")).unwrap(),
            b"new detection"
        );
    }
    #[test]
    fn failed_final_log_read_blocks_success() {
        let d = tempfile::tempdir().unwrap();
        assert!(final_security_check(10, Err("log unavailable".into()), d.path()).is_err());
    }
}
