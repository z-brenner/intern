use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::sharepoint_setup::{SharePointSetupError, SharePointSetupPhase};
use intern_intake::SharePointDeployment;
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
    /// Required only when the managed SharePoint experience exists. Without
    /// it the app keeps its ordinary setup and manual intake settings.
    pub required: bool,
    /// The packaged SharePoint deployment parses, validates, and is enabled.
    pub share_point_available: bool,
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
    share_point_available: bool,
    gate: Arc<Mutex<()>>,
    writer: StateWriter,
    durability_uncertain: Arc<AtomicBool>,
    #[cfg(test)]
    fail_next_write: Arc<std::sync::atomic::AtomicBool>,
}

impl OnboardingStore {
    /// `deployment` is the packaged deployment resource. It is compiled into
    /// the binary, so its availability is decided once.
    pub fn new(path: PathBuf, deployment: &[u8]) -> Self {
        Self {
            path,
            share_point_available: SharePointDeployment::from_slice(deployment).is_ok(),
            gate: Arc::new(Mutex::new(())),
            writer: write_state,
            durability_uncertain: Arc::new(AtomicBool::new(false)),
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
        Ok(status_for(
            state.completed_onboarding_version,
            self.share_point_available,
        ))
    }

    /// Records the current flow only after its final readiness check. A newer
    /// stored version belongs to a newer app and must survive this build.
    ///
    /// When the SharePoint deployment is available, the backend itself checks
    /// `setup_phase` is Active first, rather than trusting the webview's last
    /// step; otherwise it is never consulted.
    pub fn complete(
        &self,
        setup_phase: impl FnOnce() -> Result<SharePointSetupPhase, SharePointSetupError>,
    ) -> Result<(), OnboardingError> {
        if self.share_point_available {
            let incomplete = |message: String| OnboardingError {
                code: "ONBOARDING_SETUP_INCOMPLETE".into(),
                message,
            };
            match setup_phase() {
                Ok(SharePointSetupPhase::Active) => {}
                Ok(_) => {
                    return Err(incomplete(
                        "SharePoint setup is not active yet, so onboarding was not completed."
                            .into(),
                    ));
                }
                Err(error) => {
                    return Err(incomplete(format!(
                        "SharePoint setup could not be confirmed, so onboarding was not completed: {} ({})",
                        error.message, error.code
                    )));
                }
            }
        }
        let _guard = self.gate.lock().map_err(|_| OnboardingError {
            code: "ONBOARDING_STATE_UNAVAILABLE".into(),
            message: "onboarding state is unavailable".into(),
        })?;
        let state = read_state(&self.path)?;
        let completed_onboarding_version = state
            .completed_onboarding_version
            .max(CURRENT_ONBOARDING_VERSION);
        if completed_onboarding_version == state.completed_onboarding_version
            && !self.durability_uncertain.load(Ordering::SeqCst)
        {
            return Ok(());
        }
        let result = self.write_state(&UiState {
            completed_onboarding_version,
        });
        match result {
            Ok(()) => {
                self.durability_uncertain.store(false, Ordering::SeqCst);
                Ok(())
            }
            Err(error) => {
                if error.code == "ONBOARDING_STATE_DURABILITY_UNCERTAIN" {
                    self.durability_uncertain.store(true, Ordering::SeqCst);
                }
                Err(error)
            }
        }
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
            share_point_available: true,
            gate: Arc::new(Mutex::new(())),
            writer,
            durability_uncertain: Arc::new(AtomicBool::new(false)),
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
pub async fn onboarding_complete(app: tauri::AppHandle) -> Result<(), OnboardingError> {
    tauri::async_runtime::spawn_blocking(move || {
        use tauri::Manager;
        app.state::<OnboardingStore>()
            .complete(|| crate::sharepoint_setup::current_phase(&app))
    })
    .await
    .map_err(|_| OnboardingError {
        code: "ONBOARDING_STATE_UNAVAILABLE".into(),
        message: "onboarding completion could not finish".into(),
    })?
}

fn status_for(completed_version: u32, share_point_available: bool) -> OnboardingStatus {
    OnboardingStatus {
        current_version: CURRENT_ONBOARDING_VERSION,
        completed_version,
        required: share_point_available && completed_version < CURRENT_ONBOARDING_VERSION,
        share_point_available,
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
    use crate::sharepoint_setup::{SharePointSetupError, SharePointSetupPhase};

    fn active() -> Result<SharePointSetupPhase, SharePointSetupError> {
        Ok(SharePointSetupPhase::Active)
    }

    const DEPLOYMENT: &str = r#"{
        "schema_version": 1,
        "enabled": ENABLED,
        "site_url": "https://teamcontoso.sharepoint.com/sites/InternTestSite",
        "library_name": "Files",
        "intake_folder_name": "Inbox",
        "destination_folder_name": "Filed",
        "tenant_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "client_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "site_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "web_id": "dddddddd-dddd-dddd-dddd-dddddddddddd",
        "list_id": "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee",
        "drive_id": "b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO",
        "intake_folder_id": "01SYNTHETICINBOXFOLDERAAAAAAAAAAAA",
        "destination_folder_id": "01SYNTHETICFILEDFOLDERAAAAAAAAAAAA"
    }"#;

    fn deployment(enabled: bool) -> Vec<u8> {
        DEPLOYMENT
            .replace("ENABLED", if enabled { "true" } else { "false" })
            .into_bytes()
    }

    static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);
    static RETRY_WRITER_CALLS: AtomicUsize = AtomicUsize::new(0);

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

    fn publish_uncertain_once_then_confirm(
        path: &Path,
        state: &UiState,
    ) -> Result<(), OnboardingError> {
        fs::write(
            path,
            serde_json::to_vec(state).expect("encode published state"),
        )
        .expect("publish state before injected sync failure");
        if RETRY_WRITER_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
            Err(OnboardingError::durability_uncertain(
                std::io::Error::other("injected first sync failure"),
            ))
        } else {
            Ok(())
        }
    }

    #[test]
    fn missing_state_requires_the_current_onboarding() {
        let dir = TempDir::new();
        let status = OnboardingStore::new(dir.state_path(), &deployment(true))
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

            let status = OnboardingStore::new(path, &deployment(true))
                .status()
                .expect("read state");

            assert_eq!(status.current_version, CURRENT_ONBOARDING_VERSION);
            assert_eq!(status.completed_version, completed_version);
            assert_eq!(status.required, required);
        }
    }

    #[test]
    fn an_enabled_deployment_is_available_and_requires_onboarding_until_completed() {
        let dir = TempDir::new();
        let store = OnboardingStore::new(dir.state_path(), &deployment(true));

        let status = store.status().expect("read missing state");
        assert!(status.share_point_available);
        assert!(status.required);
        assert_eq!(
            serde_json::to_value(&status).unwrap(),
            serde_json::json!({
                "currentVersion": CURRENT_ONBOARDING_VERSION,
                "completedVersion": 0,
                "required": true,
                "sharePointAvailable": true
            })
        );

        store.complete(active).expect("complete onboarding");
        let status = store.status().expect("read completed state");
        assert!(status.share_point_available);
        assert!(!status.required);
    }

    #[test]
    fn a_disabled_or_invalid_deployment_never_requires_onboarding() {
        for source in [
            deployment(false),
            b"{not json".to_vec(),
            // Enabled, but outside the fixed site: validation must fail.
            String::from_utf8(deployment(true))
                .unwrap()
                .replace("InternTestSite", "OtherSite")
                .into_bytes(),
        ] {
            for completed_version in [0, CURRENT_ONBOARDING_VERSION, 9] {
                let dir = TempDir::new();
                let path = dir.state_path();
                write_state(
                    &path,
                    &UiState {
                        completed_onboarding_version: completed_version,
                    },
                )
                .expect("seed onboarding state");

                let status = OnboardingStore::new(path, &source)
                    .status()
                    .expect("read state");

                assert!(!status.share_point_available);
                assert!(!status.required);
                assert_eq!(
                    status.completed_version, completed_version,
                    "stored progress, including a future version, is reported unchanged"
                );
            }
        }
    }

    #[test]
    fn state_errors_are_reported_even_without_an_available_deployment() {
        let dir = TempDir::new();
        fs::write(dir.state_path(), "{not json").expect("write corrupt state");

        let error = OnboardingStore::new(dir.state_path(), &deployment(false))
            .status()
            .expect_err("corrupt state must stay visible");

        assert_eq!(error.code, "ONBOARDING_STATE_UNREADABLE");
    }

    #[test]
    fn corrupt_state_is_reported_instead_of_restarting_onboarding() {
        let dir = TempDir::new();
        fs::write(dir.state_path(), "{not json").expect("write corrupt state");

        let error = OnboardingStore::new(dir.state_path(), &deployment(true))
            .status()
            .expect_err("corrupt state must be visible");

        assert_eq!(error.code, "ONBOARDING_STATE_UNREADABLE");
    }

    #[test]
    fn oversized_state_is_rejected_before_json_parsing() {
        let dir = TempDir::new();
        fs::write(dir.state_path(), vec![b'x'; 16 * 1024 + 1]).expect("write oversized state");

        let error = OnboardingStore::new(dir.state_path(), &deployment(true))
            .status()
            .expect_err("oversized state must be visible");

        assert_eq!(error.code, "ONBOARDING_STATE_TOO_LARGE");
    }

    #[test]
    fn completion_round_trips_through_an_atomic_state_file() {
        let dir = TempDir::new();
        let path = dir.state_path();
        let store = OnboardingStore::new(path.clone(), &deployment(true));

        store.complete(active).expect("complete onboarding");

        assert_eq!(
            OnboardingStore::new(path.clone(), &deployment(true))
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

        OnboardingStore::new(path.clone(), &deployment(true))
            .complete(active)
            .expect("complete from future state");

        assert_eq!(
            OnboardingStore::new(path, &deployment(true))
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

        OnboardingStore::new(path.clone(), &deployment(true))
            .complete(active)
            .expect("replace existing state");

        assert_eq!(state_json(&path), "{\"completedOnboardingVersion\":1}");
    }

    #[test]
    fn write_failure_leaves_completion_unclaimed() {
        let dir = TempDir::new();
        let path = dir.state_path();
        let store = OnboardingStore::new(path.clone(), &deployment(true));
        store.fail_next_write_for_test();

        let error = store
            .complete(active)
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
        let store = OnboardingStore::new(path.clone(), &deployment(true));
        store.fail_next_write_for_test();

        store
            .complete(active)
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
            .complete(active)
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

    #[test]
    fn completion_retries_after_durability_uncertainty_even_when_disk_is_current() {
        RETRY_WRITER_CALLS.store(0, Ordering::SeqCst);
        let dir = TempDir::new();
        let store = OnboardingStore::with_writer_for_test(
            dir.state_path(),
            publish_uncertain_once_then_confirm,
        );

        let first = store
            .complete(active)
            .expect_err("first publication is intentionally uncertain");
        assert_eq!(first.code, "ONBOARDING_STATE_DURABILITY_UNCERTAIN");
        assert_eq!(
            store
                .status()
                .expect("published state is readable")
                .completed_version,
            CURRENT_ONBOARDING_VERSION
        );

        store
            .complete(active)
            .expect("second completion re-syncs the published state");

        assert_eq!(RETRY_WRITER_CALLS.load(Ordering::SeqCst), 2);
        assert_eq!(
            store
                .status()
                .expect("confirmed state remains readable")
                .completed_version,
            CURRENT_ONBOARDING_VERSION
        );
    }

    #[test]
    fn completion_requires_active_sharepoint_setup_when_the_deployment_is_enabled() {
        let dir = TempDir::new();
        let store = OnboardingStore::new(dir.state_path(), &deployment(true));

        for phase in [
            SharePointSetupPhase::EnrollmentPending,
            SharePointSetupPhase::ReadyToActivate,
        ] {
            let error = store
                .complete(|| Ok(phase))
                .expect_err("setup is not active");
            assert_eq!(error.code, "ONBOARDING_SETUP_INCOMPLETE");
        }
        let error = store
            .complete(|| {
                Err(SharePointSetupError::new(
                    "ONEDRIVE_MISSING",
                    "OneDrive is not installed on this computer.",
                ))
            })
            .expect_err("setup could not be confirmed");
        assert_eq!(error.code, "ONBOARDING_SETUP_INCOMPLETE");
        assert!(
            error.message.contains("ONEDRIVE_MISSING"),
            "{}",
            error.message
        );
        assert!(!dir.state_path().exists(), "nothing was recorded");
        assert!(store.status().unwrap().required);

        store.complete(active).expect("active setup completes");
        assert!(!store.status().unwrap().required);
    }

    #[test]
    fn completion_without_an_available_deployment_never_consults_sharepoint() {
        let dir = TempDir::new();
        let store = OnboardingStore::new(dir.state_path(), &deployment(false));

        store
            .complete(|| panic!("SharePoint setup is not part of this build"))
            .expect("complete as before");

        assert_eq!(
            store.status().unwrap().completed_version,
            CURRENT_ONBOARDING_VERSION
        );
    }

    #[test]
    fn completion_does_not_run_on_the_ipc_thread() {
        fn leaves_the_ipc_thread<A, R: std::future::Future, F: FnOnce(A) -> R>(_: F) {}
        leaves_the_ipc_thread(super::onboarding_complete);
    }
}
