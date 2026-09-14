use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::State;

pub const CURRENT_ONBOARDING_VERSION: u32 = 1;

const MAX_UI_STATE_BYTES: u64 = 16 * 1024;

type StateWriter = fn(&Path, &UiState) -> Result<(), OnboardingError>;

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

    fn too_large() -> Self {
        Self {
            code: "ONBOARDING_STATE_TOO_LARGE".into(),
            message: "onboarding state is too large to read safely".into(),
        }
    }

    fn write_failed(error: std::io::Error) -> Self {
        Self {
            code: "ONBOARDING_STATE_WRITE_FAILED".into(),
            message: format!("onboarding completion could not be saved: {error}"),
        }
    }

    fn durability_uncertain(error: std::io::Error) -> Self {
        Self {
            code: "ONBOARDING_STATE_DURABILITY_UNCERTAIN".into(),
            message: format!(
                "onboarding completion was published but could not be confirmed durable: {error}"
            ),
        }
    }
}

/// Backend-owned, process-local serialization for the atomic file publisher.
/// The file itself is still the source of truth across application launches.
#[derive(Clone)]
pub struct OnboardingStore {
    path: PathBuf,
    gate: Arc<Mutex<()>>,
    writer: StateWriter,
    #[cfg(test)]
    fail_next_write: Arc<std::sync::atomic::AtomicBool>,
}

impl OnboardingStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            gate: Arc::new(Mutex::new(())),
            writer: write_state,
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
        self.fail_next_write
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(test)]
    fn with_writer_for_test(path: PathBuf, writer: StateWriter) -> Self {
        Self {
            path,
            gate: Arc::new(Mutex::new(())),
            writer,
            fail_next_write: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    fn write_state(&self, state: &UiState) -> Result<(), OnboardingError> {
        #[cfg(test)]
        if self
            .fail_next_write
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            return Err(OnboardingError {
                code: "ONBOARDING_STATE_WRITE_FAILED".into(),
                message: "onboarding completion could not be saved: injected write failure".into(),
            });
        }
        (self.writer)(&self.path, state)
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
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(UiState {
                completed_onboarding_version: 0,
            });
        }
        Err(error) => return Err(OnboardingError::unreadable(error)),
    };
    let length = file.metadata().map_err(OnboardingError::unreadable)?.len();
    if length > MAX_UI_STATE_BYTES {
        return Err(OnboardingError::too_large());
    }
    let mut contents = Vec::with_capacity(length as usize);
    file.take(MAX_UI_STATE_BYTES + 1)
        .read_to_end(&mut contents)
        .map_err(OnboardingError::unreadable)?;
    if contents.len() as u64 > MAX_UI_STATE_BYTES {
        return Err(OnboardingError::too_large());
    }
    serde_json::from_slice(&contents).map_err(OnboardingError::invalid)
}

/// Publishes with the core's same-directory, write-through replacement
/// primitive. It flushes the temporary bytes and the publication metadata.
fn write_state(path: &Path, state: &UiState) -> Result<(), OnboardingError> {
    let bytes = serde_json::to_vec(state).map_err(OnboardingError::invalid)?;
    match intern_core::replace_file_durable(path, &bytes) {
        Ok(()) => Ok(()),
        Err(intern_core::DurableReplaceError::NotPublished(error)) => {
            Err(OnboardingError::write_failed(error))
        }
        Err(intern_core::DurableReplaceError::PublishedButNotDurable(error)) => {
            Err(OnboardingError::durability_uncertain(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{
        CURRENT_ONBOARDING_VERSION, OnboardingError, OnboardingStore, UiState, write_state,
    };

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
    fn oversized_state_is_rejected_before_json_parsing() {
        let dir = TempDir::new();
        fs::write(dir.state_path(), vec![b'x'; 16 * 1024 + 1]).expect("write oversized state");

        let error = OnboardingStore::new(dir.state_path())
            .status()
            .expect_err("oversized state must be visible");

        assert_eq!(error.code, "ONBOARDING_STATE_TOO_LARGE");
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

    #[cfg(windows)]
    #[test]
    fn completion_replaces_an_existing_state_file_on_windows() {
        let dir = TempDir::new();
        let path = dir.state_path();
        write_state(
            &path,
            &UiState {
                completed_onboarding_version: 0,
            },
        )
        .expect("seed old state");

        OnboardingStore::new(path.clone())
            .complete()
            .expect("replace existing state");

        assert_eq!(state_json(&path), "{\"completedOnboardingVersion\":1}");
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

    #[test]
    fn injected_publication_failure_leaves_existing_state_unchanged() {
        let dir = TempDir::new();
        let path = dir.state_path();
        write_state(
            &path,
            &UiState {
                completed_onboarding_version: 0,
            },
        )
        .expect("seed old state");
        let store = OnboardingStore::new(path.clone());
        store.fail_next_write_for_test();

        store
            .complete()
            .expect_err("injected publication failure must fail completion");

        assert_eq!(state_json(&path), "{\"completedOnboardingVersion\":0}");
    }

    #[test]
    fn post_publication_durability_failure_keeps_completion_visible_in_status() {
        let dir = TempDir::new();
        let path = dir.state_path();
        let store = OnboardingStore::with_writer_for_test(path.clone(), |path, _| {
            fs::write(path, "{\"completedOnboardingVersion\":1}")
                .expect("publish state before injected sync failure");
            Err(OnboardingError::durability_uncertain(
                std::io::Error::other("injected parent sync failure"),
            ))
        });

        let error = store
            .complete()
            .expect_err("durability uncertainty must remain visible");

        assert_eq!(error.code, "ONBOARDING_STATE_DURABILITY_UNCERTAIN");
        assert_eq!(
            store
                .status()
                .expect("published state remains readable")
                .completed_version,
            CURRENT_ONBOARDING_VERSION
        );
    }
}
