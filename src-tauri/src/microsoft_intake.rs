//! Microsoft account connection, upload authorization, and attribution for the
//! desktop host. Credentials never cross IPC or enter the shared folder.
use crate::secrets::{KeyringStore, SecretStore};
use intern_core::{OwnedFileSnapshot, PrivateSnapshotDirectory};
use intern_intake::microsoft::{
    Account, DevicePrompt, FolderBinding, MicrosoftClient, SignInProgress, TokenStore,
    proof::{FreshUploadMetadata, FreshUploadOutcome, verify_fresh_upload},
    transport::item_url,
};
use intern_intake::{SharePointDeployment, classify, detect_cloud_roots, relative_to_root};
use intern_queue::{
    AdmissionEvidence, AdmissionGuard, AdmissionStage, AppSettings, FiledDocument, FilingSink,
    PipelineError, PipelineResult, SettingsStore,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::State;

#[derive(Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    bindings: Vec<FolderBinding>,
    #[serde(default)]
    protected_roots: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribution {
    pub path: String,
    pub filename: String,
    pub state: String,
    pub reason: String,
    pub uploader: Option<Account>,
    pub processed_by: Option<Account>,
    pub filed_as: Option<String>,
    pub checked_at: i64,
    #[serde(skip)]
    source_hash: Option<String>,
    #[serde(skip)]
    authorized_processor: Option<Account>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftStatus {
    pub connected: bool,
    pub account: Option<Account>,
    pub binding: Option<FolderBinding>,
    pub documents: Vec<Attribution>,
    pub error: Option<String>,
}
struct CredentialStore;
impl TokenStore for CredentialStore {
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        KeyringStore
            .get(&format!("microsoft-intake:{key}"))
            .map_err(|_| "The operating system could not read Microsoft credentials.".into())
    }
    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        KeyringStore.set(&format!("microsoft-intake:{key}"),value).map_err(|_| "Microsoft credentials could not be protected by the operating system. Sign-in was not saved.".into())
    }
    fn delete(&self, key: &str) -> Result<(), String> {
        KeyringStore.delete(&format!("microsoft-intake:{key}")).map_err(|_| "Microsoft was disconnected, but the old credential could not be deleted from the operating system store.".into())
    }
}

pub struct MicrosoftIntake {
    settings: SettingsStore,
    data: PathBuf,
    config: Mutex<PublicConfig>,
    /// Why the stored configuration could not be read. Behind a lock because
    /// connecting Microsoft again rewrites the file, and that repair has to
    /// take effect without restarting Intern.
    config_error: Mutex<Option<String>>,
    /// Why no Microsoft request can be made at all. Nothing inside the app
    /// repairs it, so it stands for the life of the process.
    deployment: Option<SharePointDeployment>,
    deployment_error: Option<String>,
    client_error: Option<String>,
    client: Option<MicrosoftClient>,
    #[cfg(test)]
    fresh_upload_metadata: Option<Arc<dyn FreshUploadMetadata>>,
    snapshot_directory: Option<PrivateSnapshotDirectory>,
    snapshot_error: Option<String>,
    generation: AtomicU64,
    documents: Mutex<BTreeMap<String, Attribution>>,
    /// The last answer to "is the saved intake folder synced or on a network
    /// share", the folder it was about, and when it was reached.
    shared_intake: Mutex<Option<(String, Instant, bool)>>,
}

/// What a folder that claims to be a private local intake but is not is told
/// about itself. Held documents inherit this, and `intake_status` shows it,
/// because a folder that started syncing after it was configured is an
/// ordinary thing to happen and the person has to be able to find out why
/// their documents stopped moving.
const LOCAL_ONLY_BUT_SHARED: &str = "This intake folder is a OneDrive, SharePoint, or network folder, although it was saved as a private local intake. Uploads to it are verified through Microsoft: connect Microsoft and pair the folder, or point intake at a folder that is not shared.";

/// How long an answer about the intake folder is reused. Deciding it reads
/// the Windows registry and the network drive table, and `scope` asks about
/// every file of every scan; the answer changes only when someone moves the
/// folder or starts syncing it, and every save re-asks anyway.
const SHARED_INTAKE_RECHECK: Duration = Duration::from_secs(30);

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FixedBindingActivationError<E> {
    Microsoft(String),
    Commit(E),
    Rollback { commit: E, restore: String },
}

fn verified_upload(
    outcome: FreshUploadOutcome,
) -> PipelineResult<(String, Account, OwnedFileSnapshot)> {
    match outcome {
        FreshUploadOutcome::Authorized {
            local_sha256,
            uploader,
            snapshot,
        } => Ok((local_sha256, uploader, snapshot)),
        FreshUploadOutcome::HeldOther { reason, .. } => Err(PipelineError::new(
            "UPLOADER_OTHER",
            format!("UPLOADER_OTHER: {reason}"),
        )),
        FreshUploadOutcome::HeldUnknown { reason } => {
            Err(PipelineError::new("UPLOADER_UNVERIFIED", reason))
        }
        FreshUploadOutcome::RetryableUnavailable { reason } => {
            Err(PipelineError::retryable("UPLOADER_UNVERIFIED", reason))
        }
    }
}

fn verified_folder_web_url(
    deployment: &SharePointDeployment,
    account: &Account,
    remote: &serde_json::Value,
) -> Result<String, String> {
    if !remote["folder"].is_object()
        || remote
            .get("remoteItem")
            .is_some_and(|value| !value.is_null())
        || remote["id"].as_str() != Some(deployment.intake_folder_id())
        || [
            ("/sharepointIds/tenantId", deployment.tenant_id()),
            ("/sharepointIds/siteId", deployment.site_id()),
            ("/sharepointIds/webId", deployment.web_id()),
            ("/sharepointIds/listId", deployment.list_id()),
            ("/parentReference/driveId", deployment.drive_id()),
        ]
        .into_iter()
        .any(|(pointer, expected)| {
            remote
                .pointer(pointer)
                .and_then(serde_json::Value::as_str)
                .is_none_or(|value| !value.eq_ignore_ascii_case(expected))
        })
        || !account
            .tenant_id
            .eq_ignore_ascii_case(deployment.tenant_id())
    {
        return Err("Microsoft did not confirm a work/school folder in the connected organization. Personal accounts and shortcut items are not supported.".into());
    }
    remote["webUrl"]
        .as_str()
        .filter(|url| deployment.is_intake_folder_web_url(url))
        .map(str::to_owned)
        .ok_or_else(|| "Microsoft did not return a supported folder address.".into())
}

impl MicrosoftIntake {
    pub fn new(settings: SettingsStore, data: PathBuf) -> Self {
        let deployment = SharePointDeployment::from_slice(include_bytes!(
            "../resources/sharepoint-deployment.json"
        ));
        Self::with_deployment(settings, data, deployment)
    }

    fn with_deployment(
        settings: SettingsStore,
        data: PathBuf,
        parsed_deployment: Result<SharePointDeployment, intern_intake::DeploymentError>,
    ) -> Self {
        let loaded = read_config(&data.join("microsoft-intake.json"));
        let (config, config_error) = match loaded {
            Ok(value) => (value, None),
            Err(error) => (PublicConfig::default(), Some(error)),
        };
        let (deployment, deployment_error) = match parsed_deployment {
            Ok(deployment) => (Some(deployment), None),
            Err(error) => (None, Some(error.to_string())),
        };
        let (client, client_error) = match deployment.as_ref() {
            Some(deployment) => {
                match MicrosoftClient::new(deployment.clone(), Arc::new(CredentialStore)) {
                    Ok(client) => (Some(client), None),
                    Err(error) => (None, Some(error)),
                }
            }
            None => (None, None),
        };
        let (snapshot_directory, snapshot_error) =
            match PrivateSnapshotDirectory::new(data.join("microsoft-upload-snapshots")) {
                Ok(directory) => (Some(directory), None),
                Err(_) => (
                    None,
                    Some("Private Microsoft upload snapshots are unavailable.".into()),
                ),
            };
        let documents = fs::read(data.join("intake-attribution.json"))
            .ok()
            .filter(|bytes| bytes.len() <= 1024 * 1024)
            .and_then(|bytes| serde_json::from_slice::<Vec<Attribution>>(&bytes).ok())
            .unwrap_or_default()
            .into_iter()
            .take(256)
            .map(|record| (record.path.clone(), record))
            .collect();
        Self {
            settings,
            data,
            config: Mutex::new(config),
            config_error: Mutex::new(config_error),
            deployment,
            deployment_error,
            client_error,
            client,
            #[cfg(test)]
            fresh_upload_metadata: None,
            snapshot_directory,
            snapshot_error,
            generation: AtomicU64::new(0),
            documents: Mutex::new(documents),
            shared_intake: Mutex::new(None),
        }
    }

    #[cfg(test)]
    fn with_enabled_metadata(
        settings: SettingsStore,
        data: PathBuf,
        deployment: SharePointDeployment,
        binding: FolderBinding,
        metadata: Arc<dyn FreshUploadMetadata>,
        snapshot_root: PathBuf,
    ) -> Result<Self, String> {
        let mut intake = Self::with_deployment(settings, data, Ok(deployment));
        if !intake.binding_matches_deployment(&binding) {
            return Err("The test fixture binding is outside the validated deployment.".into());
        }
        intake.snapshot_directory = Some(
            PrivateSnapshotDirectory::new(snapshot_root)
                .map_err(|_| "The test snapshot directory is unavailable.")?,
        );
        intake.snapshot_error = None;
        intake.fresh_upload_metadata = Some(metadata);
        intake.config = Mutex::new(PublicConfig {
            enabled: true,
            protected_roots: vec![binding.local_folder.clone()],
            bindings: vec![binding],
        });
        Ok(intake)
    }

    /// Whether `folder` is a OneDrive, SharePoint, or network folder, from
    /// the remembered answer when it is still fresh.
    fn folder_is_shared(&self, folder: &str) -> bool {
        let folder = folder.trim();
        if folder.is_empty() {
            return false;
        }
        if let Ok(remembered) = self.shared_intake.lock()
            && let Some((about, decided, shared)) = remembered.as_ref()
            && about == folder
            && decided.elapsed() < SHARED_INTAKE_RECHECK
        {
            return *shared;
        }
        self.recheck_folder_is_shared(folder)
    }

    /// Asks the machine itself, and remembers the answer.
    fn recheck_folder_is_shared(&self, folder: &str) -> bool {
        let folder = folder.trim();
        if folder.is_empty() {
            return false;
        }
        let shared = classify(Path::new(folder), &detect_cloud_roots()).is_some();
        if let Ok(mut remembered) = self.shared_intake.lock() {
            *remembered = Some((folder.to_owned(), Instant::now(), shared));
        }
        shared
    }

    /// Why documents in the watched folder are held even though intake is
    /// saved as private and local, if that is what is wrong. Read by
    /// `intake_status`, so the reason reaches Settings rather than only the
    /// documents it holds.
    pub fn local_only_contradiction(&self) -> Option<String> {
        let settings = self.settings.load().unwrap_or_default();
        (settings.intake_local_only && self.folder_is_shared(&settings.intake_folder))
            .then(|| LOCAL_ONLY_BUT_SHARED.to_owned())
    }
    /// Why Microsoft verification cannot be relied on right now, if anything
    /// is wrong. A document only inherits this when it is inside a folder the
    /// verification actually covers.
    fn unavailable(&self) -> Option<String> {
        self.deployment_error
            .clone()
            .or_else(|| self.client_error.clone())
            .or_else(|| self.snapshot_error.clone())
            .or_else(|| {
                self.config_error
                    .lock()
                    .ok()
                    .and_then(|error| error.clone())
            })
    }
    fn deployment(&self) -> Result<&SharePointDeployment, String> {
        self.deployment.as_ref().ok_or_else(|| {
            self.unavailable()
                .unwrap_or_else(|| "Microsoft connection is unavailable.".into())
        })
    }
    fn client(&self) -> Result<&MicrosoftClient, String> {
        self.client.as_ref().ok_or_else(|| {
            self.unavailable()
                .unwrap_or_else(|| "Microsoft connection is unavailable.".into())
        })
    }
    fn fresh_upload_metadata(&self) -> Result<&dyn FreshUploadMetadata, String> {
        #[cfg(test)]
        if let Some(metadata) = self.fresh_upload_metadata.as_deref() {
            return Ok(metadata);
        }
        self.client()
            .map(|client| client as &dyn FreshUploadMetadata)
    }
    fn snapshot_directory(&self) -> Result<&PrivateSnapshotDirectory, String> {
        self.snapshot_directory.as_ref().ok_or_else(|| {
            self.unavailable()
                .unwrap_or_else(|| "Private Microsoft upload snapshots are unavailable.".into())
        })
    }
    fn binding_matches_deployment(&self, binding: &FolderBinding) -> bool {
        self.deployment.as_ref().is_some_and(|deployment| {
            binding
                .tenant_id
                .eq_ignore_ascii_case(deployment.tenant_id())
                && binding.activation_watermark.is_some_and(|value| value > 0)
                && binding.web_id.eq_ignore_ascii_case(deployment.web_id())
                && binding.drive_id.eq_ignore_ascii_case(deployment.drive_id())
                && binding
                    .folder_id
                    .eq_ignore_ascii_case(deployment.intake_folder_id())
                && deployment.is_intake_folder_web_url(&binding.web_url)
        })
    }

    pub(crate) fn fixed_binding_active(
        &self,
        deployment: &SharePointDeployment,
        local_inbox: &Path,
    ) -> bool {
        self.deployment.as_ref() == Some(deployment)
            && self.config.lock().is_ok_and(|config| {
                config.bindings.iter().any(|binding| {
                    self.binding_matches_deployment(binding)
                        && same_path(&binding.local_folder, &local_inbox.to_string_lossy())
                })
            })
    }

    /// Publishes an inactive but protected Inbox stage before the caller's
    /// local settings commit, then adds the fixed binding and watermark only
    /// after that commit succeeds. A failure on either side therefore leaves
    /// no active binding, and the Inbox stays a protected root either way so
    /// processing remains fail-closed.
    pub(crate) fn activate_fixed_binding<E>(
        &self,
        deployment: &SharePointDeployment,
        account: &Account,
        local_inbox: &Path,
        commit: impl FnOnce() -> Result<(), E>,
    ) -> Result<(), FixedBindingActivationError<E>> {
        let epoch = self.generation.load(Ordering::SeqCst);
        let current_account = self
            .client()
            .map_err(FixedBindingActivationError::Microsoft)?
            .account()
            .filter(|current| intern_intake::microsoft::proof::same_person(current, account))
            .ok_or_else(|| {
                FixedBindingActivationError::Microsoft(
                    "The connected Microsoft account changed before activation.".into(),
                )
            })?;
        if self.deployment.as_ref() != Some(deployment)
            || !intern_intake::microsoft::proof::is_guid(&account.id)
            || !intern_intake::microsoft::proof::same_person(&current_account, account)
            || !account
                .tenant_id
                .eq_ignore_ascii_case(deployment.tenant_id())
        {
            return Err(FixedBindingActivationError::Microsoft(
                "The Microsoft account or deployment changed before activation.".into(),
            ));
        }
        if !local_inbox.is_dir() {
            return Err(FixedBindingActivationError::Microsoft(
                "The verified local Inbox is no longer available.".into(),
            ));
        }
        let local_folder = local_inbox.to_string_lossy().into_owned();
        let (previous, staged) = {
            let mut config = self.config.lock().map_err(|_| {
                FixedBindingActivationError::Microsoft(
                    "Microsoft configuration is unavailable.".into(),
                )
            })?;
            if !config.enabled {
                return Err(FixedBindingActivationError::Microsoft(
                    "Connect Microsoft before activating SharePoint.".into(),
                ));
            }
            if self.generation.load(Ordering::SeqCst) != epoch {
                return Err(FixedBindingActivationError::Microsoft(
                    "The Microsoft connection changed before activation.".into(),
                ));
            }
            let previous = config.clone();
            let mut staged = previous.clone();
            staged
                .bindings
                .retain(|entry| !same_path(&entry.local_folder, &local_folder));
            if !staged
                .protected_roots
                .iter()
                .any(|root| same_path(root, &local_folder))
            {
                staged.protected_roots.push(local_folder.clone());
            }
            self.save(&staged)
                .map_err(FixedBindingActivationError::Microsoft)?;
            *config = staged.clone();
            (previous, staged)
        };

        if let Err(commit_error) = commit() {
            let mut config = match self.config.lock() {
                Ok(config) => config,
                Err(_) => {
                    return Err(FixedBindingActivationError::Rollback {
                        commit: commit_error,
                        restore: "Microsoft configuration is unavailable.".into(),
                    });
                }
            };
            if *config != staged || self.generation.load(Ordering::SeqCst) != epoch {
                return Err(FixedBindingActivationError::Rollback {
                    commit: commit_error,
                    restore: "The Microsoft connection or configuration changed during rollback; its newer fail-closed state was preserved.".into(),
                });
            }
            let restored = previous_keeping_protection(&previous, &staged);
            if let Err(restore) = self.save(&restored) {
                return Err(FixedBindingActivationError::Rollback {
                    commit: commit_error,
                    restore,
                });
            }
            *config = restored;
            return Err(FixedBindingActivationError::Commit(commit_error));
        }
        let mut config = self.config.lock().map_err(|_| {
            FixedBindingActivationError::Microsoft("Microsoft configuration is unavailable.".into())
        })?;
        let connection_unchanged = self.generation.load(Ordering::SeqCst) == epoch
            && self
                .client()
                .ok()
                .and_then(MicrosoftClient::account)
                .is_some_and(|current| {
                    intern_intake::microsoft::proof::same_person(&current, account)
                })
            && *config == staged;
        if !connection_unchanged {
            if *config == staged {
                let restored = previous_keeping_protection(&previous, &staged);
                self.save(&restored)
                    .map_err(FixedBindingActivationError::Microsoft)?;
                *config = restored;
            }
            return Err(FixedBindingActivationError::Microsoft(
                "The Microsoft connection changed while SharePoint was being activated.".into(),
            ));
        }
        let binding = FolderBinding {
            local_folder,
            drive_id: deployment.drive_id().to_owned(),
            folder_id: deployment.intake_folder_id().to_owned(),
            web_url: deployment.intake_folder_web_url(),
            tenant_id: deployment.tenant_id().to_owned(),
            web_id: deployment.web_id().to_owned(),
            activation_watermark: Some(chrono::Utc::now().timestamp_millis()),
        };
        let mut next = staged;
        next.bindings.push(binding);
        self.save(&next)
            .map_err(FixedBindingActivationError::Microsoft)?;
        *config = next;
        self.generation.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    pub fn status(&self) -> MicrosoftStatus {
        let config = self
            .config
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default();
        let settings = self.settings.load().unwrap_or_default();
        let account = if config.enabled {
            self.client.as_ref().and_then(MicrosoftClient::account)
        } else {
            None
        };
        let binding = config
            .bindings
            .iter()
            .filter(|binding| self.binding_matches_deployment(binding))
            .find(|binding| same_path(&binding.local_folder, &settings.intake_folder))
            .cloned();
        let documents = self
            .documents
            .lock()
            .map(|rows| rows.values().rev().take(100).cloned().collect())
            .unwrap_or_default();
        MicrosoftStatus {
            connected: self.deployment.is_some() && config.enabled,
            account,
            binding,
            documents,
            error: self.unavailable(),
        }
    }
    /// Writes the configuration, replacing whatever was there.
    ///
    /// A successful write is also the repair for a file that could not be
    /// read: what is on disk is now exactly what is in memory, so the earlier
    /// read error no longer describes anything.
    fn save(&self, config: &PublicConfig) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(config)
            .map_err(|_| "Microsoft configuration could not be encoded.")?;
        atomic_write(&self.data.join("microsoft-intake.json"), &bytes)?;
        if let Ok(mut error) = self.config_error.lock() {
            *error = None;
        }
        Ok(())
    }
    /// Remember strict intake roots before saving settings. Disabling watching
    /// or choosing a new root cannot bypass checks on already queued documents.
    pub fn protect_settings(&self, settings: &AppSettings) -> Result<(), String> {
        // A save asks the machine again rather than reusing a remembered
        // answer, and what it learns is what the scan path then reads.
        if settings.intake_local_only && self.recheck_folder_is_shared(&settings.intake_folder) {
            return Err("Local-only intake cannot be used for OneDrive, SharePoint, or a network share. Microsoft upload verification is required.".into());
        }
        self.generation.fetch_add(1, Ordering::SeqCst);
        if settings.intake_folder.trim().is_empty() || settings.intake_local_only {
            return Ok(());
        }
        // Only a save that names a folder Microsoft verification must cover
        // needs the configuration. Refusing every other save - a destination,
        // a house rule, turning intake off - left a broken configuration with
        // no way to reach the panel that repairs it.
        if let Some(error) = self.unavailable() {
            return Err(error);
        }
        let mut config = self
            .config
            .lock()
            .map_err(|_| "Microsoft configuration is unavailable.")?;
        if !config
            .protected_roots
            .iter()
            .any(|root| same_path(root, &settings.intake_folder))
        {
            let mut next = config.clone();
            next.protected_roots.push(settings.intake_folder.clone());
            self.save(&next)?;
            *config = next;
        }
        Ok(())
    }
    pub fn begin(&self) -> Result<DevicePrompt, String> {
        self.deployment()?;
        self.generation.fetch_add(1, Ordering::SeqCst);
        {
            let mut config = self
                .config
                .lock()
                .map_err(|_| "Microsoft configuration is unavailable.")?;
            let mut next = config.clone();
            next.enabled = false;
            self.save(&next)?;
            *config = next;
        }
        self.client()?.disconnect()?;
        self.client()?.begin()
    }
    pub fn poll(&self) -> Result<SignInProgress, String> {
        self.deployment()?;
        let epoch = self.generation.load(Ordering::SeqCst);
        let result = self.client()?.poll()?;
        if matches!(result, SignInProgress::Connected { .. }) {
            let mut config = self
                .config
                .lock()
                .map_err(|_| "Microsoft configuration is unavailable.")?;
            if self.generation.load(Ordering::SeqCst) != epoch {
                return Err("Microsoft sign-in changed or was canceled. Files remain held.".into());
            }
            let mut next = config.clone();
            next.enabled = true;
            self.save(&next)?;
            *config = next;
        }
        Ok(result)
    }
    pub fn disconnect(&self) -> Result<(), String> {
        self.deployment()?;
        self.generation.fetch_add(1, Ordering::SeqCst);
        // Disable in memory first, even if disk or credential deletion fails.
        let persist = {
            let mut config = self
                .config
                .lock()
                .map_err(|_| "Microsoft configuration is unavailable.")?;
            config.enabled = false;
            self.save(&config)
        };
        let credentials = self.client()?.disconnect();
        persist?;
        credentials
    }
    pub fn bind(&self) -> Result<FolderBinding, String> {
        let deployment = self.deployment()?.clone();
        let epoch = self.generation.load(Ordering::SeqCst);
        let settings = self.settings.load().map_err(|error| error.to_string())?;
        if settings.intake_folder.trim().is_empty() || settings.intake_local_only {
            return Err("Save a Microsoft intake folder in Settings before pairing it.".into());
        }
        if !self
            .config
            .lock()
            .map_err(|_| "Microsoft configuration is unavailable.")?
            .enabled
        {
            return Err("Connect Microsoft before pairing a folder.".into());
        }
        let local = Path::new(&settings.intake_folder)
            .canonicalize()
            .map_err(|_| "The saved intake folder could not be found.")?;
        if !local.is_dir() {
            return Err("The saved intake path is not a folder.".into());
        }
        let (account, remote) = self.client()?.metadata(item_url(
            deployment.drive_id(),
            deployment.intake_folder_id(),
            None,
        )?)?;
        let web_url = verified_folder_web_url(&deployment, &account, &remote)?;
        self.persist_verified_binding(&deployment, epoch, &settings, &local, web_url)
    }
    fn persist_verified_binding(
        &self,
        deployment: &SharePointDeployment,
        epoch: u64,
        settings: &AppSettings,
        local: &Path,
        web_url: String,
    ) -> Result<FolderBinding, String> {
        let mut config = self
            .config
            .lock()
            .map_err(|_| "Microsoft configuration is unavailable.")?;
        if self.generation.load(Ordering::SeqCst) != epoch || !config.enabled {
            return Err("Microsoft connection changed while the folder was being paired.".into());
        }
        let current = self.settings.load().map_err(|error| error.to_string())?;
        if current.intake_folder != settings.intake_folder || current.intake_local_only {
            return Err("The saved intake folder changed. Pair it again.".into());
        }
        let binding = FolderBinding {
            local_folder: local.to_string_lossy().into_owned(),
            drive_id: deployment.drive_id().to_owned(),
            folder_id: deployment.intake_folder_id().to_owned(),
            web_url,
            tenant_id: deployment.tenant_id().to_owned(),
            web_id: deployment.web_id().to_owned(),
            activation_watermark: Some(chrono::Utc::now().timestamp_millis()),
        };
        let mut next = config.clone();
        next.bindings
            .retain(|entry| !same_path(&entry.local_folder, &binding.local_folder));
        next.bindings.push(binding.clone());
        if !next
            .protected_roots
            .iter()
            .any(|root| same_path(root, &binding.local_folder))
        {
            next.protected_roots.push(binding.local_folder.clone());
        }
        self.save(&next)?;
        *config = next;
        self.generation.fetch_add(1, Ordering::SeqCst);
        Ok(binding)
    }
    fn scope(&self, path: &Path, settings: &AppSettings) -> Result<Option<FolderBinding>, String> {
        let config = self
            .config
            .lock()
            .map_err(|_| "Microsoft configuration is unavailable.")?;
        let mut protected = config.protected_roots.iter().any(|root| within(path, root));
        let watched =
            !settings.intake_folder.trim().is_empty() && within(path, &settings.intake_folder);
        // `intake_local_only` is a claim made about a folder on the day it was
        // saved, and folders change: one moved into OneDrive, or one whose
        // parent starts syncing, is a shared folder from that moment however
        // the settings still read. Trusting the stored flag defeated the whole
        // verification - documents teammates synced in were processed as this
        // machine's own uploads - so what the save refused is re-derived here.
        let contradicted =
            settings.intake_local_only && self.folder_is_shared(&settings.intake_folder);
        protected |= watched && (!settings.intake_local_only || contradicted);
        if !protected {
            return Ok(None);
        }
        if watched && contradicted {
            return Err(LOCAL_ONLY_BUT_SHARED.into());
        }
        // Whether the path is covered is decided before the configuration is
        // asked anything, so a file that could never have been a Microsoft
        // upload is not held by a configuration that cannot be read. The
        // watched intake folder itself, which the settings still name, stays
        // held.
        if let Some(error) = self.unavailable() {
            return Err(error);
        }
        if !config.enabled {
            return Err(
                "Microsoft is disconnected. Unverified uploads are never processed.".into(),
            );
        }
        let binding=config.bindings.iter().filter(|binding|self.binding_matches_deployment(binding) && within(path,&binding.local_folder)).max_by_key(|binding|binding.local_folder.len()).cloned()
            .ok_or("Pair the saved intake folder with its Microsoft drive and folder IDs. Unverified uploads remain held.")?;
        Ok(Some(binding))
    }
    fn verify(
        &self,
        path: &Path,
        settings: &AppSettings,
    ) -> PipelineResult<Option<(String, Account, Account, OwnedFileSnapshot)>> {
        let epoch = self.generation.load(Ordering::SeqCst);
        let Some(binding) = self
            .scope(path, settings)
            .map_err(|message| PipelineError::new("UPLOADER_UNVERIFIED", message))?
        else {
            return Ok(None);
        };
        let activation_watermark = binding.activation_watermark.ok_or_else(|| {
            PipelineError::new(
                "UPLOADER_UNVERIFIED",
                "Pair the saved intake folder again. Unverified uploads remain held.",
            )
        })?;
        let deployment = self
            .deployment()
            .map_err(|message| PipelineError::new("UPLOADER_UNVERIFIED", message))?;
        let outcome = verify_fresh_upload(
            deployment,
            self.fresh_upload_metadata()
                .map_err(|message| PipelineError::new("UPLOADER_UNVERIFIED", message))?,
            activation_watermark,
            self.snapshot_directory()
                .map_err(|message| PipelineError::new("UPLOADER_UNVERIFIED", message))?,
            Path::new(&binding.local_folder),
            path,
        );
        if let FreshUploadOutcome::HeldOther { uploader, reason } = &outcome {
            self.note(path, "other", Some(uploader.clone()), None, None, reason);
        }
        let (hash, uploader, snapshot) = verified_upload(outcome)?;
        if self.generation.load(Ordering::SeqCst) != epoch {
            return Err(PipelineError::new(
                "UPLOADER_UNVERIFIED",
                "The Microsoft connection changed during verification. The file remains held.",
            ));
        }
        let latest = self.settings.load()?;
        if latest.intake_folder != settings.intake_folder
            || latest.process_others_uploads != settings.process_others_uploads
            || latest.intake_local_only != settings.intake_local_only
        {
            return Err(PipelineError::new(
                "UPLOADER_UNVERIFIED",
                "Intake scope changed during verification. The file remains held.",
            ));
        }
        Ok(Some((hash, uploader.clone(), uploader, snapshot)))
    }
    fn note(
        &self,
        path: &Path,
        state: &str,
        uploader: Option<Account>,
        processor: Option<Account>,
        hash: Option<String>,
        reason: &str,
    ) {
        if let Ok(mut rows) = self.documents.lock() {
            let key = path.to_string_lossy().into_owned();
            let prior = rows.get(&key).cloned();
            let same = prior
                .as_ref()
                .is_some_and(|record| record.source_hash == hash && hash.is_some());
            let record = Attribution {
                path: key.clone(),
                filename: path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                state: state.into(),
                reason: reason.into(),
                uploader,
                processed_by: if same {
                    prior
                        .as_ref()
                        .and_then(|record| record.processed_by.clone())
                } else {
                    None
                },
                filed_as: if same {
                    prior.as_ref().and_then(|record| record.filed_as.clone())
                } else {
                    None
                },
                checked_at: crate::intake::now_unix(),
                source_hash: hash,
                authorized_processor: processor,
            };
            rows.insert(key, record);
            while rows.len() > 256 {
                if let Some(oldest) = rows
                    .iter()
                    .min_by_key(|(_, record)| record.checked_at)
                    .map(|(key, _)| key.clone())
                {
                    rows.remove(&oldest);
                }
            }
        }
    }
    fn persist_attribution(&self) {
        if let Ok(rows) = self.documents.lock()
            && let Ok(bytes) = serde_json::to_vec_pretty(&rows.values().collect::<Vec<_>>())
        {
            let _ = atomic_write(&self.data.join("intake-attribution.json"), &bytes);
        }
    }
}
impl AdmissionGuard for MicrosoftIntake {
    fn authorize(&self, path: &Path, _stage: AdmissionStage) -> PipelineResult<AdmissionEvidence> {
        let settings = self.settings.load()?;
        match self.verify(path, &settings) {
            Ok(None) => Ok(AdmissionEvidence::local()),
            Ok(Some((hash, uploader, processor, snapshot))) => {
                self.note(
                    path,
                    "verified",
                    Some(uploader),
                    Some(processor),
                    Some(hash.clone()),
                    "Uploader verified against Microsoft upload activity.",
                );
                Ok(AdmissionEvidence::verified_snapshot(hash, snapshot))
            }
            Err(error) => {
                if error.code != "UPLOADER_OTHER" && !error.is_retryable() {
                    self.note(path, "unknown", None, None, None, &error.message);
                }
                Err(error)
            }
        }
    }
    fn processed(&self, path: &Path, hash: &str) {
        if let Ok(mut rows) = self.documents.lock()
            && let Some(record) = rows.get_mut(&path.to_string_lossy().into_owned())
            && record.source_hash.as_deref() == Some(hash)
        {
            record.processed_by = record.authorized_processor.clone();
            record.state = "processed".into();
            record.reason = "Analysis completed on this installation.".into();
        }
        self.persist_attribution();
    }
}
impl FilingSink for MicrosoftIntake {
    fn unfiled(&self, document: &intern_queue::UnfiledDocument) {
        if let Ok(mut rows) = self.documents.lock()
            && let Some(record) = rows.get_mut(&document.source_path.to_string_lossy().into_owned())
        {
            record.filed_as = None;
            record.state = "processed".into();
            record.reason = "Filing was undone; the original file was restored.".into();
        }
        self.persist_attribution();
    }
    fn filed(&self, document: &FiledDocument) {
        if let Ok(mut rows) = self.documents.lock()
            && let Some(record) = rows.get_mut(&document.source_path.to_string_lossy().into_owned())
            && record.source_hash.as_deref() == Some(&document.source_hash)
        {
            record.filed_as = document
                .destination
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            record.state = "filed".into();
            record.reason = "The verified document was filed.".into();
        }
        self.persist_attribution();
    }
}
/// The configuration an aborted fixed activation returns to: the prior
/// bindings and enablement, but never fewer protected roots than the stage.
/// The local commit may already have watched the Inbox, and its rollback may
/// not have stopped that watcher or may itself have failed, so the shared
/// Inbox stays held rather than becoming an ordinary local folder.
fn previous_keeping_protection(previous: &PublicConfig, staged: &PublicConfig) -> PublicConfig {
    let mut restored = previous.clone();
    restored.protected_roots = staged.protected_roots.clone();
    restored
}
fn within(path: &Path, root: &str) -> bool {
    !root.trim().is_empty()
        && (same_path(&path.to_string_lossy(), root)
            || relative_to_root(path, Path::new(root)).is_some())
}
fn same_path(left: &str, right: &str) -> bool {
    left.replace('\\', "/")
        .trim_end_matches('/')
        .eq_ignore_ascii_case(right.replace('\\', "/").trim_end_matches('/'))
}
fn read_config(path: &Path) -> Result<PublicConfig, String> {
    match fs::read(path){Ok(bytes) if bytes.len()<=64*1024=>serde_json::from_slice(&bytes).map_err(|_| "Microsoft intake configuration is invalid. Processing is held until it is repaired.".into()),Ok(_)=>Err("Microsoft intake configuration is too large. Processing is held.".into()),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(PublicConfig::default()),Err(_)=>Err("Microsoft intake configuration is unreadable. Processing is held.".into())}
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes).map_err(|_| "Microsoft intake state could not be saved.")?;
    fs::rename(temporary, path).map_err(|_| "Microsoft intake state could not be saved.".into())
}

#[tauri::command]
pub fn microsoft_intake_status(state: State<'_, Arc<MicrosoftIntake>>) -> MicrosoftStatus {
    state.status()
}
#[tauri::command]
pub async fn microsoft_sign_in_start(
    state: State<'_, Arc<MicrosoftIntake>>,
) -> Result<DevicePrompt, String> {
    let manager = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || manager.begin())
        .await
        .map_err(|_| "Microsoft sign-in task could not finish.")?
}
#[tauri::command]
pub async fn microsoft_sign_in_poll(
    state: State<'_, Arc<MicrosoftIntake>>,
) -> Result<SignInProgress, String> {
    let manager = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || manager.poll())
        .await
        .map_err(|_| "Microsoft sign-in task could not finish.")?
}
#[tauri::command]
pub async fn microsoft_disconnect(state: State<'_, Arc<MicrosoftIntake>>) -> Result<(), String> {
    let manager = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || manager.disconnect())
        .await
        .map_err(|_| "Microsoft disconnect task could not finish.")?
}
#[tauri::command]
pub async fn microsoft_bind_intake(
    state: State<'_, Arc<MicrosoftIntake>>,
) -> Result<FolderBinding, String> {
    let manager = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || manager.bind())
        .await
        .map_err(|_| "Microsoft folder pairing could not finish.")?
}
#[tauri::command]
pub fn microsoft_open_sign_in(
    app: tauri::AppHandle,
    state: State<'_, Arc<MicrosoftIntake>>,
) -> Result<(), String> {
    state.deployment()?;
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url("https://microsoft.com/devicelogin",None::<&str>).map_err(|_| "Microsoft sign-in could not be opened. Type https://microsoft.com/devicelogin in your browser.".into())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use intern_intake::{
        Clock,
        microsoft::{
            TokenStore,
            transport::{Reply, Transport},
        },
    };
    use std::{
        collections::{HashMap, VecDeque},
        sync::atomic::AtomicI64,
    };

    #[derive(Default)]
    pub(crate) struct ReconnectTokens(pub(crate) Mutex<HashMap<String, String>>);

    impl TokenStore for ReconnectTokens {
        fn get(&self, key: &str) -> Result<Option<String>, String> {
            Ok(self.0.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: &str) -> Result<(), String> {
            self.0.lock().unwrap().insert(key.into(), value.into());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), String> {
            self.0.lock().unwrap().remove(key);
            Ok(())
        }
    }

    pub(crate) struct ReconnectTransport(pub(crate) Mutex<VecDeque<Reply>>);

    impl Transport for ReconnectTransport {
        fn request(
            &self,
            _url: url::Url,
            _form: Option<&[(&str, &str)]>,
            _bearer: Option<&str>,
        ) -> Result<Reply, String> {
            self.0
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| "unexpected Microsoft request".into())
        }
    }

    pub(crate) struct ReconnectClock(pub(crate) AtomicI64);

    impl Clock for ReconnectClock {
        fn now(&self) -> i64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    pub(crate) fn reconnect_reply(status: u16, body: serde_json::Value) -> Reply {
        Reply {
            status,
            body,
            retry_after: 60,
        }
    }

    /// A manager with `deployment`, a persisted enabled configuration in
    /// `data`, and `account` signed in through a scripted device flow. No
    /// further Microsoft request is answered, so any verification that would
    /// need Graph fails rather than authorizes.
    pub(crate) fn connected_manager(
        settings: SettingsStore,
        data: PathBuf,
        deployment: SharePointDeployment,
        account: &Account,
    ) -> MicrosoftIntake {
        fs::create_dir_all(&data).unwrap();
        fs::write(
            data.join("microsoft-intake.json"),
            br#"{"enabled":true,"bindings":[],"protectedRoots":[]}"#,
        )
        .unwrap();
        let clock = Arc::new(ReconnectClock(AtomicI64::new(1_000)));
        let client = MicrosoftClient::with_transport(
            deployment.clone(),
            Arc::new(ReconnectTokens::default()),
            Arc::new(ReconnectTransport(Mutex::new(VecDeque::from([
                reconnect_reply(
                    200,
                    serde_json::json!({
                        "device_code": "private-device-code",
                        "user_code": "ABCD-EFGH",
                        "verification_uri": "https://microsoft.com/devicelogin",
                        "expires_in": 900,
                        "interval": 5
                    }),
                ),
                reconnect_reply(
                    200,
                    serde_json::json!({
                        "token_type": "Bearer",
                        "access_token": "private-access-token",
                        "refresh_token": "private-refresh-token",
                        "expires_in": 3600
                    }),
                ),
                reconnect_reply(
                    200,
                    serde_json::json!({
                        "id": account.id,
                        "displayName": account.display_name,
                        "mail": account.email,
                        "userPrincipalName": account.user_principal_name
                    }),
                ),
            ])))),
            Arc::clone(&clock) as Arc<dyn Clock>,
        );
        client.begin().unwrap();
        clock.0.store(1_005, Ordering::SeqCst);
        client.poll().unwrap();
        let mut intake = MicrosoftIntake::with_deployment(settings, data, Ok(deployment));
        intake.client = Some(client);
        intake
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{
        ReconnectClock, ReconnectTokens, ReconnectTransport, reconnect_reply,
    };
    use super::*;
    use intern_intake::microsoft::{MicrosoftClient, proof::FreshUploadMetadata};
    use intern_queue::{WorkerBoundary, WorkerFailure};
    use std::{collections::VecDeque, sync::atomic::AtomicI64};
    const DEPLOYMENT_UNAVAILABLE: &str = "SharePoint deployment configuration is unavailable: provisioned identifiers are not available in this build.";
    const PAIR_REQUIRED: &str = "Pair the saved intake folder with its Microsoft drive and folder IDs. Unverified uploads remain held.";

    fn test_deployment() -> SharePointDeployment {
        SharePointDeployment::from_slice(
            br#"{
              "schema_version": 1,
              "enabled": true,
              "site_url": "https://teamcontoso.sharepoint.com/sites/InternTestSite",
              "library_name": "Files",
              "intake_folder_name": "Inbox",
              "destination_folder_name": "Filed",
              "tenant_id": "11111111-1111-1111-1111-111111111111",
              "client_id": "22222222-2222-2222-2222-222222222222",
              "site_id": "33333333-3333-3333-3333-333333333333",
              "web_id": "44444444-4444-4444-4444-444444444444",
              "list_id": "55555555-5555-5555-5555-555555555555",
              "drive_id": "66666666-6666-6666-6666-666666666666",
              "intake_folder_id": "77777777-7777-7777-7777-777777777777",
              "destination_folder_id": "88888888-8888-8888-8888-888888888888"
            }"#,
        )
        .unwrap()
    }

    fn account() -> Account {
        Account {
            tenant_id: "11111111-1111-1111-1111-111111111111".into(),
            id: "99999999-9999-9999-9999-999999999999".into(),
            display_name: "Pat Example".into(),
            email: "pat@example.test".into(),
            user_principal_name: "pat@example.test".into(),
        }
    }

    fn reconnect_client(clock: Arc<ReconnectClock>) -> MicrosoftClient {
        let device = || {
            reconnect_reply(
                200,
                serde_json::json!({
                    "device_code": "private-device-code",
                    "user_code": "ABCD-EFGH",
                    "verification_uri": "https://microsoft.com/devicelogin",
                    "expires_in": 900,
                    "interval": 5
                }),
            )
        };
        let token = || {
            reconnect_reply(
                200,
                serde_json::json!({
                    "token_type": "Bearer",
                    "access_token": "private-access-token",
                    "refresh_token": "private-refresh-token",
                    "expires_in": 3600
                }),
            )
        };
        let profile = |id: &str| {
            reconnect_reply(
                200,
                serde_json::json!({
                    "id": id,
                    "displayName": "Pat Example",
                    "mail": "pat@example.test",
                    "userPrincipalName": "pat@example.test"
                }),
            )
        };
        MicrosoftClient::with_transport(
            test_deployment(),
            Arc::new(ReconnectTokens::default()),
            Arc::new(ReconnectTransport(Mutex::new(VecDeque::from([
                device(),
                token(),
                profile("99999999-9999-9999-9999-999999999999"),
                device(),
                token(),
                profile("aaaaaaaa-9999-9999-9999-999999999999"),
            ])))),
            clock,
        )
    }

    fn connect_next_account(client: &MicrosoftClient, clock: &ReconnectClock, now: i64) {
        client.begin().unwrap();
        clock.0.store(now + 5, Ordering::SeqCst);
        client.poll().unwrap();
    }

    fn folder_metadata() -> serde_json::Value {
        serde_json::json!({
            "id": "77777777-7777-7777-7777-777777777777",
            "folder": { "childCount": 0 },
            "webUrl": "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox",
            "sharepointIds": {
                "tenantId": "11111111-1111-1111-1111-111111111111",
                "siteId": "33333333-3333-3333-3333-333333333333",
                "webId": "44444444-4444-4444-4444-444444444444",
                "listId": "55555555-5555-5555-5555-555555555555"
            },
            "parentReference": {
                "driveId": "66666666-6666-6666-6666-666666666666"
            }
        })
    }

    struct SwappingFreshUploadMetadata {
        source: PathBuf,
        calls: AtomicU64,
    }

    impl FreshUploadMetadata for SwappingFreshUploadMetadata {
        fn metadata(&self, _url: url::Url) -> Result<(Account, serde_json::Value), String> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if call == 4 {
                fs::write(&self.source, b"replacement bytes")
                    .expect("the public source can be swapped after proof snapshots its bytes");
            }
            Ok((
                account(),
                serde_json::json!({
                    "id": "item!123",
                    "eTag": "\"fresh,1\"",
                    "cTag": "\"content,1\"",
                    "name": "swap.pdf",
                    "size": 5,
                    "webUrl": "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox/swap.pdf",
                    "parentReference": {
                        "driveId": "66666666-6666-6666-6666-666666666666",
                        "id": "77777777-7777-7777-7777-777777777777"
                    },
                    "sharepointIds": {
                        "tenantId": "11111111-1111-1111-1111-111111111111",
                        "siteId": "33333333-3333-3333-3333-333333333333",
                        "webId": "44444444-4444-4444-4444-444444444444",
                        "listId": "55555555-5555-5555-5555-555555555555",
                        "listItemUniqueId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
                    },
                    "createdBy": { "user": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "userPrincipalName": "pat@example.test"
                    }},
                    "lastModifiedBy": { "user": {
                        "id": "99999999-9999-9999-9999-999999999999",
                        "userPrincipalName": "pat@example.test"
                    }},
                    "createdDateTime": "2026-09-14T16:00:00Z",
                    "lastModifiedDateTime": "2026-09-14T16:00:00Z",
                    "file": {
                        "mimeType": "application/pdf",
                        "hashes": { "quickXorHash": "aCgDG9jwBgAAAAAABQAAAAAAAAA=" }
                    }
                }),
            ))
        }
    }

    struct SnapshotReadingBoundary {
        public_source: PathBuf,
        snapshot_root: PathBuf,
        read: Mutex<Vec<u8>>,
    }

    impl WorkerBoundary for SnapshotReadingBoundary {
        fn extract(
            &self,
            _request_id: &str,
            path: &Path,
            _progress: &mut dyn FnMut(intern_engine::ExtractProgress),
        ) -> Result<intern_engine::DocumentSource, WorkerFailure> {
            assert!(
                path.starts_with(&self.snapshot_root),
                "the extractor receives only the private verified snapshot"
            );
            *self.read.lock().unwrap() = fs::read(path).unwrap();
            fs::write(&self.public_source, b"hello").unwrap();
            Err(WorkerFailure::new("TEST_STOP", false, false))
        }

        fn cancel(&self, _request_id: &str) -> Result<(), WorkerFailure> {
            Ok(())
        }

        fn restart(&self) -> Result<(), WorkerFailure> {
            Ok(())
        }
    }

    struct NeverAnalyze;

    impl intern_queue::AnalyzerBoundary for NeverAnalyze {
        fn analyze(
            &self,
            _source: &intern_engine::DocumentSource,
            _extension: &str,
            _existing_names: &[&str],
        ) -> Result<intern_engine::DocumentAnalysis, intern_queue::ModelFailure> {
            panic!("the test worker stops after reading the extractor path")
        }
    }

    struct NoEvents;

    impl intern_queue::PipelineEventSink for NoEvents {
        fn queue_changed(&self) {}

        fn progress(&self, _progress: intern_queue::PipelineProgress) {}
    }

    #[test]
    fn enabled_injected_manager_admits_only_snapshot_bytes_through_the_real_queue() {
        let data = std::env::temp_dir().join(format!(
            "intern-microsoft-enabled-seam-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let inbox = data.join("Inbox");
        let snapshot_root = data.join("proof-snapshots");
        fs::create_dir_all(&inbox).unwrap();
        let source = inbox.join("swap.pdf");
        fs::write(&source, b"hello").unwrap();
        let source = source.canonicalize().unwrap();
        let local_folder = inbox.canonicalize().unwrap().to_string_lossy().into_owned();
        let settings = AppSettings {
            intake_folder: local_folder.clone(),
            ..AppSettings::default()
        };
        let settings_store = SettingsStore::new(data.join("settings.json"));
        settings_store.save(&settings).unwrap();
        let binding = FolderBinding {
            local_folder,
            drive_id: "66666666-6666-6666-6666-666666666666".into(),
            folder_id: "77777777-7777-7777-7777-777777777777".into(),
            web_url: "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox".into(),
            tenant_id: "11111111-1111-1111-1111-111111111111".into(),
            web_id: "44444444-4444-4444-4444-444444444444".into(),
            activation_watermark: Some(1_789_401_599_000),
        };
        let metadata = Arc::new(SwappingFreshUploadMetadata {
            source: source.clone(),
            calls: AtomicU64::new(0),
        });
        let manager = Arc::new(
            MicrosoftIntake::with_enabled_metadata(
                settings_store.clone(),
                data.clone(),
                test_deployment(),
                binding,
                Arc::clone(&metadata) as Arc<dyn FreshUploadMetadata>,
                snapshot_root.clone(),
            )
            .unwrap(),
        );
        let worker = Arc::new(SnapshotReadingBoundary {
            public_source: source.clone(),
            snapshot_root: snapshot_root.clone(),
            read: Mutex::new(Vec::new()),
        });
        let pipeline = intern_queue::Pipeline::with_local_files(
            data.join("queue.sqlite3"),
            Arc::clone(&worker) as Arc<dyn WorkerBoundary>,
            Arc::new(NeverAnalyze),
            Arc::new(NoEvents),
            settings_store,
        )
        .unwrap()
        .with_admission_guard(Arc::clone(&manager) as Arc<dyn AdmissionGuard>);

        pipeline
            .enqueue_files(std::slice::from_ref(&source))
            .unwrap();
        pipeline.run_next().unwrap();

        assert_eq!(&*worker.read.lock().unwrap(), b"hello");
        assert_eq!(fs::read(&source).unwrap(), b"hello");
        assert_eq!(metadata.calls.load(Ordering::SeqCst), 4);
        assert!(fs::read_dir(&snapshot_root).unwrap().next().is_none());
        drop(pipeline);
        drop(manager);
        let _ = fs::remove_dir_all(data);
    }

    #[test]
    fn fixed_folder_binding_requires_the_deployment_web_identity() {
        let deployment = test_deployment();
        assert!(verified_folder_web_url(&deployment, &account(), &folder_metadata()).is_ok());

        for replacement in [
            None,
            Some(serde_json::json!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")),
        ] {
            let mut metadata = folder_metadata();
            match replacement {
                Some(value) => metadata["sharepointIds"]["webId"] = value,
                None => {
                    metadata["sharepointIds"]
                        .as_object_mut()
                        .unwrap()
                        .remove("webId");
                }
            }
            assert!(
                verified_folder_web_url(&deployment, &account(), &metadata).is_err(),
                "a missing or different web must not bind"
            );
        }
    }

    #[test]
    fn retryable_proof_outcome_becomes_a_retryable_pipeline_error() {
        let outcome = FreshUploadOutcome::RetryableUnavailable {
            reason: "Microsoft Graph is temporarily unavailable.".into(),
        };

        let Err(error) = verified_upload(outcome) else {
            panic!("an unavailable proof cannot authorize bytes");
        };

        assert_eq!(error.code, "UPLOADER_UNVERIFIED");
        assert_eq!(error.message, "Microsoft Graph is temporarily unavailable.");
        assert!(error.is_retryable());
    }

    #[test]
    fn disabled_packaged_deployment_fails_every_microsoft_operation_closed() {
        let data = std::env::temp_dir().join(format!(
            "intern-microsoft-disabled-deployment-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&data);
        fs::create_dir_all(&data).unwrap();
        let intake =
            MicrosoftIntake::new(SettingsStore::new(data.join("settings.json")), data.clone());

        assert_eq!(
            intake.status().error.as_deref(),
            Some(DEPLOYMENT_UNAVAILABLE)
        );
        assert_eq!(intake.begin().unwrap_err(), DEPLOYMENT_UNAVAILABLE);
        assert_eq!(intake.poll().unwrap_err(), DEPLOYMENT_UNAVAILABLE);
        assert_eq!(intake.disconnect().unwrap_err(), DEPLOYMENT_UNAVAILABLE);
        assert_eq!(intake.bind().unwrap_err(), DEPLOYMENT_UNAVAILABLE);
        let _ = fs::remove_dir_all(data);
    }

    #[test]
    fn path_scope_is_component_based_not_a_prefix() {
        assert!(within(Path::new("C:/Intake/one.pdf"), "C:/Intake"));
        assert!(!within(Path::new("C:/IntakeOther/one.pdf"), "C:/Intake"));
    }
    /// A Microsoft intake whose stored configuration cannot be read, in a
    /// directory of its own.
    fn unreadable_config(name: &str) -> (PathBuf, MicrosoftIntake) {
        let data =
            std::env::temp_dir().join(format!("intern-microsoft-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&data);
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("microsoft-intake.json"), b"not json").unwrap();
        let intake =
            MicrosoftIntake::new(SettingsStore::new(data.join("settings.json")), data.clone());
        assert!(intake.status().error.is_some(), "the trouble is reported");
        (data, intake)
    }

    #[test]
    fn a_corrupt_microsoft_config_does_not_hold_local_files() {
        let (data, intake) = unreadable_config("corrupt");
        let elsewhere = AppSettings::default();
        // A file from an ordinary local folder was never a Microsoft upload,
        // so nothing about it depends on the configuration.
        assert!(
            intake
                .scope(&data.join("Desktop/scan.pdf"), &elsewhere)
                .unwrap()
                .is_none()
        );
        // And a settings save that names no Microsoft intake folder must go
        // through, or Settings cannot be reached to repair the configuration.
        assert!(intake.protect_settings(&elsewhere).is_ok());

        // The folder the guard exists for is still held.
        let watched = AppSettings {
            intake_folder: data.to_string_lossy().into_owned(),
            ..AppSettings::default()
        };
        assert!(intake.scope(&data.join("upload.pdf"), &watched).is_err());
        assert!(intake.protect_settings(&watched).is_err());
        let _ = fs::remove_dir_all(&data);
    }

    #[test]
    fn repairing_local_state_cannot_override_a_disabled_deployment() {
        let (data, intake) = unreadable_config("repair");
        intake.save(&PublicConfig::default()).unwrap();
        assert!(read_config(&data.join("microsoft-intake.json")).is_ok());
        assert_eq!(
            intake.status().error.as_deref(),
            Some(DEPLOYMENT_UNAVAILABLE)
        );
        let watched = AppSettings {
            intake_folder: data.to_string_lossy().into_owned(),
            ..AppSettings::default()
        };
        assert_eq!(
            intake.protect_settings(&watched).unwrap_err(),
            DEPLOYMENT_UNAVAILABLE
        );
        let _ = fs::remove_dir_all(&data);
    }

    /// A Microsoft intake whose stored configuration is simply absent, in a
    /// directory of its own, with the settings it will read already saved.
    fn configured(name: &str, settings: &AppSettings) -> (PathBuf, MicrosoftIntake) {
        let data =
            std::env::temp_dir().join(format!("intern-microsoft-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&data);
        fs::create_dir_all(&data).unwrap();
        let store = SettingsStore::new(data.join("settings.json"));
        store.save(settings).unwrap();
        let intake = MicrosoftIntake::new(store, data.clone());
        (data, intake)
    }

    #[test]
    fn an_intake_folder_that_is_shared_is_verified_whatever_local_only_claims() {
        // The folder was saved as a private local intake and has since become
        // a network share - or was moved into OneDrive, which reaches this
        // same decision through the detected sync roots.
        let settings = AppSettings {
            intake_folder: r"\\fileserver\legal\intake".into(),
            intake_local_only: true,
            ..AppSettings::default()
        };
        let (data, intake) = configured("shared-local-only", &settings);

        let scope = intake.scope(
            Path::new(r"\\fileserver\legal\intake\contract.pdf"),
            &settings,
        );

        assert_eq!(scope.err().as_deref(), Some(LOCAL_ONLY_BUT_SHARED));
        assert_eq!(
            intake.local_only_contradiction().as_deref(),
            Some(LOCAL_ONLY_BUT_SHARED),
            "and the reason reaches the intake status a person can read"
        );
        let _ = fs::remove_dir_all(&data);
    }

    #[test]
    fn an_ordinary_local_folder_saved_as_local_only_is_still_not_verified() {
        let folder =
            std::env::temp_dir().join(format!("intern-microsoft-local-{}", std::process::id()));
        let settings = AppSettings {
            intake_folder: folder.to_string_lossy().into_owned(),
            intake_local_only: true,
            ..AppSettings::default()
        };
        let (data, intake) = configured("local", &settings);

        assert!(
            intake
                .scope(&folder.join("scan.pdf"), &settings)
                .unwrap()
                .is_none()
        );
        assert!(intake.local_only_contradiction().is_none());
        let _ = fs::remove_dir_all(&data);
    }

    #[test]
    fn malformed_config_is_not_treated_as_disconnected_defaults() {
        let path =
            std::env::temp_dir().join(format!("intern-invalid-config-{}.json", std::process::id()));
        fs::write(&path, b"not json").unwrap();
        assert!(read_config(&path).is_err());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn a_legacy_binding_without_an_activation_watermark_requires_re_pairing() {
        let data = std::env::temp_dir().join(format!(
            "intern-microsoft-legacy-watermark-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&data);
        let inbox = data.join("Inbox");
        fs::create_dir_all(&inbox).unwrap();
        let settings = AppSettings {
            intake_folder: inbox.to_string_lossy().into_owned(),
            ..AppSettings::default()
        };
        let settings_store = SettingsStore::new(data.join("settings.json"));
        settings_store.save(&settings).unwrap();
        let local_folder = inbox.canonicalize().unwrap().to_string_lossy().into_owned();
        let legacy = serde_json::json!({
            "enabled": true,
            "bindings": [{
                "localFolder": local_folder,
                "driveId": "66666666-6666-6666-6666-666666666666",
                "folderId": "77777777-7777-7777-7777-777777777777",
                "webUrl": "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox",
                "tenantId": "11111111-1111-1111-1111-111111111111"
            }],
            "protectedRoots": [local_folder]
        });
        fs::write(
            data.join("microsoft-intake.json"),
            serde_json::to_vec_pretty(&legacy).unwrap(),
        )
        .unwrap();
        let intake =
            MicrosoftIntake::with_deployment(settings_store, data.clone(), Ok(test_deployment()));
        let loaded_binding = intake.config.lock().unwrap().bindings[0].clone();
        let deployment = intake.deployment().unwrap();
        assert!(
            loaded_binding
                .tenant_id
                .eq_ignore_ascii_case(deployment.tenant_id())
        );
        assert!(
            loaded_binding
                .drive_id
                .eq_ignore_ascii_case(deployment.drive_id())
        );
        assert!(
            loaded_binding
                .folder_id
                .eq_ignore_ascii_case(deployment.intake_folder_id())
        );
        assert!(deployment.is_intake_folder_web_url(&loaded_binding.web_url));
        let candidate = Path::new(&loaded_binding.local_folder).join("agreement.pdf");
        assert!(within(&candidate, &loaded_binding.local_folder));

        assert_eq!(
            intake.scope(&candidate, &settings).unwrap_err(),
            PAIR_REQUIRED
        );
        let _ = fs::remove_dir_all(data);
    }

    #[test]
    fn a_successful_fixed_binding_persists_its_activation_watermark_across_restart() {
        let data = std::env::temp_dir().join(format!(
            "intern-microsoft-persisted-watermark-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&data);
        let inbox = data.join("Inbox");
        fs::create_dir_all(&inbox).unwrap();
        let settings = AppSettings {
            intake_folder: inbox.to_string_lossy().into_owned(),
            ..AppSettings::default()
        };
        let settings_path = data.join("settings.json");
        let settings_store = SettingsStore::new(&settings_path);
        settings_store.save(&settings).unwrap();
        fs::write(
            data.join("microsoft-intake.json"),
            br#"{"enabled":true,"bindings":[],"protectedRoots":[]}"#,
        )
        .unwrap();
        let intake =
            MicrosoftIntake::with_deployment(settings_store, data.clone(), Ok(test_deployment()));
        let deployment = intake.deployment().unwrap().clone();
        let local = inbox.canonicalize().unwrap();
        let before = chrono::Utc::now().timestamp_millis();

        let binding = intake
            .persist_verified_binding(
                &deployment,
                intake.generation.load(Ordering::SeqCst),
                &settings,
                &local,
                "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox".into(),
            )
            .unwrap();

        let after = chrono::Utc::now().timestamp_millis();
        let watermark = binding
            .activation_watermark
            .expect("a successful binding has an activation watermark");
        assert!((before..=after).contains(&watermark));
        drop(intake);

        let restarted = MicrosoftIntake::with_deployment(
            SettingsStore::new(settings_path),
            data.clone(),
            Ok(test_deployment()),
        );
        let candidate = local.join("agreement.pdf");
        let reloaded = restarted.scope(&candidate, &settings).unwrap().unwrap();
        assert_eq!(reloaded, binding);
        assert_eq!(
            read_config(&data.join("microsoft-intake.json"))
                .unwrap()
                .bindings,
            vec![binding]
        );
        drop(restarted);
        let _ = fs::remove_dir_all(data);
    }

    #[test]
    fn setup_binding_is_published_with_a_watermark_and_rolled_back_if_local_commit_fails() {
        let data = std::env::temp_dir().join(format!(
            "intern-microsoft-setup-binding-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&data);
        let inbox = data.join("Files").join("Inbox");
        fs::create_dir_all(&inbox).unwrap();
        fs::write(
            data.join("microsoft-intake.json"),
            br#"{"enabled":true,"bindings":[],"protectedRoots":[]}"#,
        )
        .unwrap();
        let clock = Arc::new(ReconnectClock(AtomicI64::new(1_000)));
        let client = reconnect_client(Arc::clone(&clock));
        connect_next_account(&client, &clock, 1_000);
        let mut intake = MicrosoftIntake::with_deployment(
            SettingsStore::new(data.join("settings.json")),
            data.clone(),
            Ok(test_deployment()),
        );
        intake.client = Some(client);
        let deployment = intake.deployment().unwrap().clone();
        let staged_closed = std::cell::Cell::new(false);

        let error = intake
            .activate_fixed_binding(&deployment, &account(), &inbox, || {
                let staged = read_config(&data.join("microsoft-intake.json")).unwrap();
                staged_closed.set(staged.bindings.is_empty() && staged.protected_roots.len() == 1);
                Err("local save failed")
            })
            .expect_err("failed local activation rolls back Microsoft binding");

        assert!(
            staged_closed.get(),
            "the local commit runs with a protected but inactive Microsoft binding"
        );
        assert_eq!(
            error,
            FixedBindingActivationError::Commit("local save failed")
        );
        let rolled_back = read_config(&data.join("microsoft-intake.json")).unwrap();
        assert!(rolled_back.bindings.is_empty());
        assert_eq!(
            rolled_back.protected_roots.len(),
            1,
            "an aborted activation leaves the shared Inbox protected"
        );

        intake
            .activate_fixed_binding(&deployment, &account(), &inbox, || Ok::<_, &str>(()))
            .expect("successful local activation commits binding");
        let committed = read_config(&data.join("microsoft-intake.json")).unwrap();
        assert_eq!(committed.bindings.len(), 1);
        assert_eq!(committed.protected_roots.len(), 1);
        assert!(
            committed.bindings[0]
                .activation_watermark
                .is_some_and(|value| value > 0)
        );
        assert!(intake.fixed_binding_active(&deployment, &inbox));
        drop(intake);
        let _ = fs::remove_dir_all(data);
    }

    #[test]
    fn reconnect_during_fixed_activation_restores_the_staged_configuration() {
        let data = std::env::temp_dir().join(format!(
            "intern-microsoft-setup-reconnect-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&data);
        let inbox = data.join("Files").join("Inbox");
        fs::create_dir_all(&inbox).unwrap();
        fs::write(
            data.join("microsoft-intake.json"),
            br#"{"enabled":true,"bindings":[],"protectedRoots":[]}"#,
        )
        .unwrap();
        let clock = Arc::new(ReconnectClock(AtomicI64::new(1_000)));
        let client = reconnect_client(Arc::clone(&clock));
        connect_next_account(&client, &clock, 1_000);
        let mut intake = MicrosoftIntake::with_deployment(
            SettingsStore::new(data.join("settings.json")),
            data.clone(),
            Ok(test_deployment()),
        );
        intake.client = Some(client);
        let deployment = intake.deployment().unwrap().clone();

        let error = intake
            .activate_fixed_binding(&deployment, &account(), &inbox, || {
                let client = intake.client().unwrap();
                client.disconnect().unwrap();
                client.begin().unwrap();
                clock.0.store(1_010, Ordering::SeqCst);
                client.poll().unwrap();
                intake.generation.fetch_add(1, Ordering::SeqCst);
                Ok::<_, &str>(())
            })
            .expect_err("a reconnect must invalidate the stale activation account");

        assert!(matches!(error, FixedBindingActivationError::Microsoft(_)));
        let restored = read_config(&data.join("microsoft-intake.json")).unwrap();
        assert!(restored.bindings.is_empty());
        assert_eq!(
            restored.protected_roots.len(),
            1,
            "an aborted activation leaves the shared Inbox protected"
        );
        drop(intake);
        let _ = fs::remove_dir_all(data);
    }

    #[test]
    fn fixed_activation_releases_microsoft_config_for_the_live_settings_commit() {
        let data = std::env::temp_dir().join(format!(
            "intern-microsoft-setup-live-settings-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&data);
        let inbox = data.join("Files").join("Inbox");
        fs::create_dir_all(&inbox).unwrap();
        fs::write(
            data.join("microsoft-intake.json"),
            br#"{"enabled":true,"bindings":[],"protectedRoots":[]}"#,
        )
        .unwrap();
        let clock = Arc::new(ReconnectClock(AtomicI64::new(1_000)));
        let client = reconnect_client(Arc::clone(&clock));
        connect_next_account(&client, &clock, 1_000);
        let mut intake = MicrosoftIntake::with_deployment(
            SettingsStore::new(data.join("settings.json")),
            data.clone(),
            Ok(test_deployment()),
        );
        intake.client = Some(client);
        let deployment = intake.deployment().unwrap().clone();
        let config_was_available = std::cell::Cell::new(false);

        intake
            .activate_fixed_binding(&deployment, &account(), &inbox, || {
                config_was_available.set(intake.config.try_lock().is_ok());
                Ok::<_, &str>(())
            })
            .unwrap();

        assert!(
            config_was_available.get(),
            "the AppState settings path must be able to apply Microsoft protection"
        );
        drop(intake);
        let _ = fs::remove_dir_all(data);
    }
}
