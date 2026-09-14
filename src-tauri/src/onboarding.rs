use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::State;

pub const CURRENT_ONBOARDING_VERSION: u32 = 1;

static NEXT_TEMP_FILE: AtomicUsize = AtomicUsize::new(0);

/// The only durable UI progress state. It is deliberately separate from
/// operational settings: a damaged settings file must not dismiss setup.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiState {
    pub completed_onboarding_version: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingStatus {
    pub current_version: u32,
    pub completed_version: u32,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingError {
    pub code: String,
    pub message: String,
}

impl OnboardingError {
    fn unreadable(error: std::io::Error) -> Self {
        Self {
            code: "ONBOARDING_STATE_UNREADABLE".into(),
            message: format!("onboarding state could not be read: {error}"),
        }
    }

    fn invalid(error: serde_json::Error) -> Self {
        Self {
            code: "ONBOARDING_STATE_UNREADABLE".into(),
            message: format!("onboarding state could not be read: {error}"),
        }
    }

    fn write_failed(error: std::io::Error) -> Self {
        Self {
            code: "ONBOARDING_STATE_WRITE_FAILED".into(),
            message: format!("onboarding completion could not be saved: {error}"),
        }
    }
}

/// Backend-owned, process-local serialization for the atomic file publisher.
/// The file itself is still the source of truth across application launches.
#[derive(Clone)]
pub struct OnboardingStore {
    path: PathBuf,
    gate: Arc<Mutex<()>>,
    #[cfg(test)]
    fail_next_write: Arc<std::sync::atomic::AtomicBool>,
}

impl OnboardingStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            gate: Arc::new(Mutex::new(())),
            #[cfg(test)]
            fail_next_write: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn status(&self) -> Result<OnboardingStatus, OnboardingError> {
        let _guard = self.gate.lock().map_err(|_| OnboardingError {
            code: "ONBOARDING_STATE_UNAVAILABLE".into(),
            message: "onboarding state is unavailable".into(),
        })?;
        let state = read_state(&self.path)?;
        Ok(status_for(state.completed_onboarding_version))
    }

    /// Records the current flow only after its final readiness check. A newer
    /// stored version belongs to a newer app and must survive this build.
    pub fn complete(&self) -> Result<(), OnboardingError> {
        let _guard = self.gate.lock().map_err(|_| OnboardingError {
            code: "ONBOARDING_STATE_UNAVAILABLE".into(),
            message: "onboarding state is unavailable".into(),
        })?;
        let state = read_state(&self.path)?;
        let completed_onboarding_version = state
            .completed_onboarding_version
            .max(CURRENT_ONBOARDING_VERSION);
        if completed_onboarding_version == state.completed_onboarding_version {
            return Ok(());
        }
        self.write_state(&UiState {
            completed_onboarding_version,
        })
    }

    #[cfg(test)]
    fn fail_next_write_for_test(&self) {
        self.fail_next_write.store(true, Ordering::SeqCst);
    }

    fn write_state(&self, state: &UiState) -> Result<(), OnboardingError> {
        #[cfg(test)]
        if self.fail_next_write.swap(false, Ordering::SeqCst) {
            return Err(OnboardingError {
                code: "ONBOARDING_STATE_WRITE_FAILED".into(),
                message: "onboarding completion could not be saved: injected write failure".into(),
            });
        }
        write_state(&self.path, state)
    }
}

#[tauri::command]
pub fn onboarding_status(
    store: State<'_, OnboardingStore>,
) -> Result<OnboardingStatus, OnboardingError> {
    store.status()
}

#[tauri::command]
pub fn onboarding_complete(store: State<'_, OnboardingStore>) -> Result<(), OnboardingError> {
    store.complete()
}

fn status_for(completed_version: u32) -> OnboardingStatus {
    OnboardingStatus {
        current_version: CURRENT_ONBOARDING_VERSION,
        completed_version,
        required: completed_version < CURRENT_ONBOARDING_VERSION,
    }
}

fn read_state(path: &Path) -> Result<UiState, OnboardingError> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(UiState {
                completed_onboarding_version: 0,
            });
        }
        Err(error) => return Err(OnboardingError::unreadable(error)),
    };
    serde_json::from_str(&contents).map_err(OnboardingError::invalid)
}

/// Publishes a fully synced replacement beside the durable state. `rename`
/// replaces the old file atomically on the supported desktop platforms.
fn write_state(path: &Path, state: &UiState) -> Result<(), OnboardingError> {
    let bytes = serde_json::to_vec(state).map_err(OnboardingError::invalid)?;
    let parent = path.parent().ok_or_else(|| OnboardingError {
        code: "ONBOARDING_STATE_WRITE_FAILED".into(),
        message: "onboarding completion could not be saved: state path has no parent directory"
            .into(),
    })?;
    let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::SeqCst);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ui-state.json");
    let temporary = parent.join(format!("{file_name}.tmp-{}-{sequence}", std::process::id()));
    let result = (|| -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(OnboardingError::write_failed(error));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{CURRENT_ONBOARDING_VERSION, OnboardingStore, UiState, write_state};

    static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let ordinal = NEXT_TEMP_DIR.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "intern-onboarding-test-{}-{ordinal}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create temporary state directory");
            Self(path)
        }

        fn state_path(&self) -> PathBuf {
            self.0.join("ui-state.json")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn state_json(path: &Path) -> String {
        fs::read_to_string(path).expect("read persisted onboarding state")
    }

    #[test]
    fn missing_state_requires_the_current_onboarding() {
        let dir = TempDir::new();
        let status = OnboardingStore::new(dir.state_path())
            .status()
            .expect("read missing state");

        assert_eq!(status.current_version, CURRENT_ONBOARDING_VERSION);
        assert_eq!(status.completed_version, 0);
        assert!(status.required);
    }

    #[test]
    fn stored_versions_preserve_older_current_and_future_completion() {
        let cases = [(0, true), (CURRENT_ONBOARDING_VERSION, false), (9, false)];

        for (completed_version, required) in cases {
            let dir = TempDir::new();
            let path = dir.state_path();
            write_state(
                &path,
                &UiState {
                    completed_onboarding_version: completed_version,
                },
            )
            .expect("seed onboarding state");

            let status = OnboardingStore::new(path).status().expect("read state");

            assert_eq!(status.current_version, CURRENT_ONBOARDING_VERSION);
            assert_eq!(status.completed_version, completed_version);
            assert_eq!(status.required, required);
        }
    }

    #[test]
    fn corrupt_state_is_reported_instead_of_restarting_onboarding() {
        let dir = TempDir::new();
        fs::write(dir.state_path(), "{not json").expect("write corrupt state");

        let error = OnboardingStore::new(dir.state_path())
            .status()
            .expect_err("corrupt state must be visible");

        assert_eq!(error.code, "ONBOARDING_STATE_UNREADABLE");
    }

    #[test]
    fn completion_round_trips_through_an_atomic_state_file() {
        let dir = TempDir::new();
        let path = dir.state_path();
        let store = OnboardingStore::new(path.clone());

        store.complete().expect("complete onboarding");

        assert_eq!(
            OnboardingStore::new(path.clone())
                .status()
                .expect("reload state")
                .completed_version,
            CURRENT_ONBOARDING_VERSION
        );
        assert_eq!(state_json(&path), "{\"completedOnboardingVersion\":1}");
        assert!(
            fs::read_dir(&dir.0)
                .expect("read state directory")
                .all(|entry| {
                    let name = entry.expect("read directory entry").file_name();
                    !name.to_string_lossy().contains("ui-state.json.tmp")
                })
        );
    }

    #[test]
    fn completion_never_downgrades_a_future_version() {
        let dir = TempDir::new();
        let path = dir.state_path();
        write_state(
            &path,
            &UiState {
                completed_onboarding_version: 9,
            },
        )
        .expect("seed future state");

        OnboardingStore::new(path.clone())
            .complete()
            .expect("complete from future state");

        assert_eq!(
            OnboardingStore::new(path)
                .status()
                .expect("reload future state")
                .completed_version,
            9
        );
    }

    #[test]
    fn write_failure_leaves_completion_unclaimed() {
        let dir = TempDir::new();
        let path = dir.state_path();
        let store = OnboardingStore::new(path.clone());
        store.fail_next_write_for_test();

        let error = store
            .complete()
            .expect_err("failed state publication must fail completion");

        assert_eq!(error.code, "ONBOARDING_STATE_WRITE_FAILED");
        assert_eq!(
            store
                .status()
                .expect("state remains readable")
                .completed_version,
            0
        );
        assert!(!path.exists());
    }
}
