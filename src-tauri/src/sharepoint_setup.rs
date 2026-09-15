//! Fixed OneDrive enrollment and SharePoint-path activation.
//!
//! All decisions live behind injected boundaries so tests can prove the
//! deployment URL, remote identity, filesystem, Microsoft binding, settings,
//! and autostart transaction without OneDrive, Graph, or Tauri. Production
//! deliberately supplies no local-to-remote verifier yet: registry display
//! names and paths are discovery hints, never authority.

use intern_intake::{
    CloudProviderKind, CloudRoot, SharePointDeployment, detect_cloud_roots,
    microsoft::{Account, proof::is_guid},
    paths_overlap, relative_to_root,
};
use intern_queue::AppSettings;
use serde::Serialize;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use url::Url;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePointSetupError {
    pub code: String,
    pub message: String,
}

impl SharePointSetupError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SharePointSetupPhase {
    EnrollmentPending,
    ReadyToActivate,
    Active,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePointSetupAccount {
    pub display_name: String,
    pub email: String,
}

/// JSON-safe setup state. It intentionally contains no tenant, site, drive,
/// folder identifiers, or arbitrary local path.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePointSetupStatus {
    pub phase: SharePointSetupPhase,
    pub account: SharePointSetupAccount,
    pub site: String,
    pub library: String,
    pub intake: String,
    pub destination: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OneDriveState {
    Available,
    Missing,
    AccountMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryFailure {
    Missing,
    NotDirectory,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncOpenFailure {
    OneDriveUnavailable,
    OpenerUnavailable,
    Other(String),
}

/// Full authoritative identity returned for one exact canonical local root.
/// A verifier result is still checked here; returning a display name or path
/// alone can never satisfy the service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteLibraryIdentity {
    pub local_root: PathBuf,
    pub tenant_id: String,
    pub site_id: String,
    pub web_id: String,
    pub list_id: String,
    pub drive_id: String,
}

pub trait RootDetector {
    fn one_drive_state(&self) -> OneDriveState;
    fn detect(&self) -> Result<Vec<CloudRoot>, SharePointSetupError>;
}

pub trait SetupFileSystem {
    fn canonicalize_directory(&self, path: &Path) -> Result<PathBuf, DirectoryFailure>;
    fn is_writable_directory(&self, path: &Path) -> bool;
}

pub trait RemoteLibraryVerifier {
    fn verify(
        &self,
        account: &Account,
        candidate: &Path,
        deployment: &SharePointDeployment,
    ) -> Result<Option<RemoteLibraryIdentity>, SharePointSetupError>;
}

/// The callback is the local settings/autostart commit. Implementations must
/// stage Microsoft protection first and restore their prior state if the
/// callback fails. A final publication failure may retain that inactive,
/// protected stage so processing stays fail-closed. A successful return means
/// the activation watermark and exact fixed Inbox binding are durable and
/// definitive; callers must not add a second fallible confirmation after the
/// local commit has succeeded.
pub trait MicrosoftSetup {
    fn connected_account(&self) -> Result<Option<Account>, SharePointSetupError>;
    fn binding_active(
        &self,
        deployment: &SharePointDeployment,
        local_inbox: &Path,
    ) -> Result<bool, SharePointSetupError>;
    fn activate_binding(
        &self,
        deployment: &SharePointDeployment,
        account: &Account,
        library: &RemoteLibraryIdentity,
        local_inbox: &Path,
        commit: &mut dyn FnMut() -> Result<(), SharePointSetupError>,
    ) -> Result<(), SharePointSetupError>;
}

pub trait SetupSettings {
    fn load(&self) -> Result<AppSettings, SharePointSetupError>;
    fn save(&self, settings: &AppSettings) -> Result<(), SharePointSetupError>;
}

pub trait AutostartBoundary {
    fn is_enabled(&self) -> Result<bool, SharePointSetupError>;
    fn set_enabled(&self, enabled: bool) -> Result<(), SharePointSetupError>;
}

pub trait SyncOpener {
    fn open(&self, url: &Url) -> Result<(), SyncOpenFailure>;
}

struct ResolvedLibrary {
    identity: RemoteLibraryIdentity,
    inbox: PathBuf,
    destination: PathBuf,
}

pub struct SharePointSetup<'a> {
    deployment: &'a SharePointDeployment,
    roots: &'a dyn RootDetector,
    fs: &'a dyn SetupFileSystem,
    verifier: &'a dyn RemoteLibraryVerifier,
    microsoft: &'a dyn MicrosoftSetup,
    settings: &'a dyn SetupSettings,
    autostart: &'a dyn AutostartBoundary,
    opener: &'a dyn SyncOpener,
}

impl<'a> SharePointSetup<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        deployment: &'a SharePointDeployment,
        roots: &'a dyn RootDetector,
        fs: &'a dyn SetupFileSystem,
        verifier: &'a dyn RemoteLibraryVerifier,
        microsoft: &'a dyn MicrosoftSetup,
        settings: &'a dyn SetupSettings,
        autostart: &'a dyn AutostartBoundary,
        opener: &'a dyn SyncOpener,
    ) -> Self {
        Self {
            deployment,
            roots,
            fs,
            verifier,
            microsoft,
            settings,
            autostart,
            opener,
        }
    }

    /// Rescans every time. Command adapters must run this blocking discovery
    /// away from the IPC thread, as the two production actions below do.
    pub fn status(&self) -> Result<SharePointSetupStatus, SharePointSetupError> {
        let account = self.connected_account()?;
        let Some(resolved) = self.resolve(&account)? else {
            return Ok(self.status_for(SharePointSetupPhase::EnrollmentPending, &account));
        };
        let settings = self.settings.load()?;
        let autostart = self.autostart.is_enabled()?;
        let active = managed_settings_match(&settings, &resolved.inbox, &resolved.destination)
            && autostart
            && self
                .microsoft
                .binding_active(self.deployment, &resolved.inbox)?;
        Ok(self.status_for(
            if active {
                SharePointSetupPhase::Active
            } else {
                SharePointSetupPhase::ReadyToActivate
            },
            &account,
        ))
    }

    pub fn start_sync(&self) -> Result<SharePointSetupStatus, SharePointSetupError> {
        let account = self.connected_account()?;
        if let Some(resolved) = self.resolve(&account)? {
            let settings = self.settings.load()?;
            let active = managed_settings_match(&settings, &resolved.inbox, &resolved.destination)
                && self.autostart.is_enabled()?
                && self
                    .microsoft
                    .binding_active(self.deployment, &resolved.inbox)?;
            return Ok(self.status_for(
                if active {
                    SharePointSetupPhase::Active
                } else {
                    SharePointSetupPhase::ReadyToActivate
                },
                &account,
            ));
        }
        let url = self
            .deployment
            .odopen_url(&account.email)
            .map_err(|error| {
                SharePointSetupError::new(
                    "MICROSOFT_ACCOUNT_INVALID",
                    format!("The connected Microsoft account cannot start OneDrive sync: {error}"),
                )
            })?;
        self.opener.open(&url).map_err(|failure| match failure {
            SyncOpenFailure::OneDriveUnavailable => SharePointSetupError::new(
                "ONEDRIVE_MISSING",
                "OneDrive could not handle the fixed SharePoint sync request. Install or open OneDrive, then try again.",
            ),
            SyncOpenFailure::OpenerUnavailable => SharePointSetupError::new(
                "SYNC_OPENER_UNAVAILABLE",
                "The operating system URL opener is unavailable; the SharePoint sync request was not launched.",
            ),
            SyncOpenFailure::Other(message) => SharePointSetupError::new(
                "ONEDRIVE_OPEN_FAILED",
                format!("OneDrive sync could not be opened: {message}"),
            ),
        })?;
        Ok(self.status_for(SharePointSetupPhase::EnrollmentPending, &account))
    }

    pub fn activate(&self) -> Result<SharePointSetupStatus, SharePointSetupError> {
        let account = self.connected_account()?;
        let resolved = self.resolve(&account)?.ok_or_else(|| {
            SharePointSetupError::new(
                "SHAREPOINT_SYNC_PENDING",
                "OneDrive has not registered the verified Files library yet. Keep OneDrive open and try again.",
            )
        })?;
        let previous = self.settings.load()?;
        let previous_autostart = self.autostart.is_enabled()?;
        let mut next = previous.clone();
        next.intake_folder = resolved.inbox.to_string_lossy().into_owned();
        next.destination = resolved.destination.to_string_lossy().into_owned();
        next.intake_enabled = true;
        next.process_others_uploads = false;
        next.intake_local_only = false;
        next.run_in_background = true;
        next.start_at_login = true;
        next.start_minimized = true;

        let mut local_committed = false;
        let mut commit = || {
            if !previous_autostart {
                self.autostart.set_enabled(true)?;
            }
            if let Err(error) = self.settings.save(&next) {
                if !previous_autostart && let Err(restore) = self.autostart.set_enabled(false) {
                    return Err(rollback_error(error, restore));
                }
                return Err(error);
            }
            local_committed = true;
            Ok(())
        };

        let binding = self.microsoft.activate_binding(
            self.deployment,
            &account,
            &resolved.identity,
            &resolved.inbox,
            &mut commit,
        );
        if let Err(error) = binding {
            if local_committed {
                return Err(self.restore_local_activation(&previous, previous_autostart, error));
            }
            return Err(error);
        }
        Ok(self.status_for(SharePointSetupPhase::Active, &account))
    }

    fn connected_account(&self) -> Result<Account, SharePointSetupError> {
        match self.roots.one_drive_state() {
            OneDriveState::Missing => {
                return Err(SharePointSetupError::new(
                    "ONEDRIVE_MISSING",
                    "OneDrive is not available on this computer.",
                ));
            }
            OneDriveState::AccountMissing => {
                return Err(SharePointSetupError::new(
                    "ONEDRIVE_ACCOUNT_MISSING",
                    "OneDrive is not signed in to a work or school account.",
                ));
            }
            OneDriveState::Available => {}
        }
        let account = self.microsoft.connected_account()?.ok_or_else(|| {
            SharePointSetupError::new(
                "MICROSOFT_ACCOUNT_MISSING",
                "Connect the Microsoft account used for Contoso uploads before starting SharePoint sync.",
            )
        })?;
        if !is_guid(&account.id)
            || !account
                .tenant_id
                .eq_ignore_ascii_case(self.deployment.tenant_id())
        {
            return Err(SharePointSetupError::new(
                "MICROSOFT_ACCOUNT_WRONG_TENANT",
                "The connected Microsoft account is outside the provisioned Contoso tenant.",
            ));
        }
        Ok(account)
    }

    fn resolve(&self, account: &Account) -> Result<Option<ResolvedLibrary>, SharePointSetupError> {
        let mut candidates = Vec::new();
        for detected in self.roots.detect()? {
            if detected.kind != CloudProviderKind::SharePoint {
                continue;
            }
            if let Ok(canonical) = self.fs.canonicalize_directory(&detected.root)
                && !candidates.contains(&canonical)
            {
                candidates.push(canonical);
            }
        }
        if candidates.is_empty() {
            return Ok(None);
        }
        for (index, left) in candidates.iter().enumerate() {
            if candidates
                .iter()
                .skip(index + 1)
                .any(|right| paths_overlap(left, right))
            {
                return Err(SharePointSetupError::new(
                    "SHAREPOINT_ROOT_NESTED",
                    "OneDrive registered overlapping SharePoint roots. Intern cannot safely map local files until the duplicate or nested sync is removed.",
                ));
            }
        }

        let mut verified = Vec::new();
        for candidate in &candidates {
            let Some(identity) = self.verifier.verify(account, candidate, self.deployment)? else {
                continue;
            };
            if self.identity_matches(candidate, &identity) {
                verified.push(identity);
            }
        }
        let identity = match verified.len() {
            0 => {
                return Err(SharePointSetupError::new(
                    "SHAREPOINT_ROOT_UNVERIFIED",
                    "OneDrive registered a SharePoint root, but an authoritative Microsoft check did not match the provisioned tenant, site, web, list, and drive.",
                ));
            }
            1 => verified.pop().expect("one verified identity"),
            _ => {
                return Err(SharePointSetupError::new(
                    "SHAREPOINT_ROOT_AMBIGUOUS",
                    "More than one local root verified as the provisioned Files library. Activation is blocked until the duplicate sync is removed.",
                ));
            }
        };
        if !self.fs.is_writable_directory(&identity.local_root) {
            return Err(SharePointSetupError::new(
                "SHAREPOINT_ROOT_UNWRITABLE",
                "The verified Files library is not writable.",
            ));
        }
        let inbox = self.fixed_child(
            &identity.local_root,
            self.deployment.intake_folder_name(),
            "INBOX",
        )?;
        let destination = self.fixed_child(
            &identity.local_root,
            self.deployment.destination_folder_name(),
            "FILED",
        )?;
        if paths_overlap(&inbox, &destination) {
            return Err(SharePointSetupError::new(
                "FIXED_FOLDERS_AMBIGUOUS",
                "Inbox and Filed did not resolve to separate fixed children of the Files library.",
            ));
        }
        Ok(Some(ResolvedLibrary {
            identity,
            inbox,
            destination,
        }))
    }

    fn fixed_child(
        &self,
        root: &Path,
        name: &str,
        code: &str,
    ) -> Result<PathBuf, SharePointSetupError> {
        let joined = root.join(name);
        let child = self.fs.canonicalize_directory(&joined).map_err(|failure| {
            let detail = match failure {
                DirectoryFailure::Missing => "is missing",
                DirectoryFailure::NotDirectory => "is not a folder",
                DirectoryFailure::Unavailable => "cannot be read",
            };
            SharePointSetupError::new(
                format!("{code}_MISSING"),
                format!("The fixed {name} folder {detail}."),
            )
        })?;
        if relative_to_root(&child, root)
            .is_none_or(|relative| relative.contains('/') || !relative.eq_ignore_ascii_case(name))
        {
            return Err(SharePointSetupError::new(
                format!("{code}_OUTSIDE_LIBRARY"),
                format!("The fixed {name} path does not resolve directly under Files."),
            ));
        }
        if !self.fs.is_writable_directory(&child) {
            return Err(SharePointSetupError::new(
                format!("{code}_UNWRITABLE"),
                format!("The fixed {name} folder is not writable."),
            ));
        }
        Ok(child)
    }

    fn identity_matches(&self, candidate: &Path, identity: &RemoteLibraryIdentity) -> bool {
        identity.local_root == candidate
            && identity
                .tenant_id
                .eq_ignore_ascii_case(self.deployment.tenant_id())
            && identity
                .site_id
                .eq_ignore_ascii_case(self.deployment.site_id())
            && identity
                .web_id
                .eq_ignore_ascii_case(self.deployment.web_id())
            && identity
                .list_id
                .eq_ignore_ascii_case(self.deployment.list_id())
            && identity
                .drive_id
                .eq_ignore_ascii_case(self.deployment.drive_id())
    }

    fn restore_local_activation(
        &self,
        previous: &AppSettings,
        previous_autostart: bool,
        original: SharePointSetupError,
    ) -> SharePointSetupError {
        let settings = self.settings.save(previous);
        let autostart = self.autostart.set_enabled(previous_autostart);
        match (settings, autostart) {
            (Ok(()), Ok(())) => original,
            (settings, autostart) => SharePointSetupError::new(
                "ACTIVATION_ROLLBACK_FAILED",
                format!(
                    "{} Activation remains incomplete and processing fails closed; settings restore: {}; autostart restore: {}.",
                    original.message,
                    result_label(settings),
                    result_label(autostart)
                ),
            ),
        }
    }

    fn status_for(&self, phase: SharePointSetupPhase, account: &Account) -> SharePointSetupStatus {
        SharePointSetupStatus {
            phase,
            account: SharePointSetupAccount {
                display_name: account.display_name.clone(),
                email: account.email.clone(),
            },
            site: "InternTestSite".into(),
            library: self.deployment.library_name().into(),
            intake: self.deployment.intake_folder_name().into(),
            destination: self.deployment.destination_folder_name().into(),
        }
    }
}

fn managed_settings_match(settings: &AppSettings, inbox: &Path, destination: &Path) -> bool {
    settings.intake_folder == inbox.to_string_lossy()
        && settings.destination == destination.to_string_lossy()
        && settings.intake_enabled
        && !settings.process_others_uploads
        && !settings.intake_local_only
        && settings.run_in_background
        && settings.start_at_login
        && settings.start_minimized
}

fn rollback_error(
    original: SharePointSetupError,
    restore: SharePointSetupError,
) -> SharePointSetupError {
    SharePointSetupError::new(
        "ACTIVATION_ROLLBACK_FAILED",
        format!(
            "{} Autostart could not be restored: {}. Activation remains incomplete.",
            original.message, restore.message
        ),
    )
}

fn result_label(result: Result<(), SharePointSetupError>) -> String {
    result.map_or_else(|error| error.code, |()| "ok".into())
}

struct SystemRoots;

impl RootDetector for SystemRoots {
    fn one_drive_state(&self) -> OneDriveState {
        // Root absence is the normal pre-enrollment state, not evidence that
        // OneDrive is absent. The odopen handler is the OS authority for that.
        OneDriveState::Available
    }

    fn detect(&self) -> Result<Vec<CloudRoot>, SharePointSetupError> {
        Ok(detect_cloud_roots())
    }
}

struct SystemFileSystem;

impl SetupFileSystem for SystemFileSystem {
    fn canonicalize_directory(&self, path: &Path) -> Result<PathBuf, DirectoryFailure> {
        let canonical = path.canonicalize().map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => DirectoryFailure::Missing,
            _ => DirectoryFailure::Unavailable,
        })?;
        if canonical.is_dir() {
            Ok(canonical)
        } else {
            Err(DirectoryFailure::NotDirectory)
        }
    }

    fn is_writable_directory(&self, path: &Path) -> bool {
        static NEXT_PROBE: AtomicU64 = AtomicU64::new(0);
        let probe = path.join(format!(
            ".intern-write-probe-{}-{}",
            std::process::id(),
            NEXT_PROBE.fetch_add(1, Ordering::Relaxed)
        ));
        let opened = OpenOptions::new().write(true).create_new(true).open(&probe);
        match opened {
            Ok(file) => {
                let synced = file.sync_all().is_ok();
                drop(file);
                let removed = std::fs::remove_file(probe).is_ok();
                synced && removed
            }
            Err(_) => false,
        }
    }
}

struct UnavailableRemoteVerifier;

impl RemoteLibraryVerifier for UnavailableRemoteVerifier {
    fn verify(
        &self,
        _account: &Account,
        _candidate: &Path,
        _deployment: &SharePointDeployment,
    ) -> Result<Option<RemoteLibraryIdentity>, SharePointSetupError> {
        Err(SharePointSetupError::new(
            "SHAREPOINT_ROOT_VERIFIER_UNAVAILABLE",
            "This build cannot authoritatively map a registered local OneDrive root to the provisioned tenant, site, web, list, and drive. Sync enrollment remains pending; activation is disabled.",
        ))
    }
}

struct ProductionMicrosoft<'a>(&'a crate::microsoft_intake::MicrosoftIntake);

impl MicrosoftSetup for ProductionMicrosoft<'_> {
    fn connected_account(&self) -> Result<Option<Account>, SharePointSetupError> {
        let status = self.0.status();
        if let Some(error) = status.error {
            return Err(SharePointSetupError::new(
                "MICROSOFT_CONNECTION_UNAVAILABLE",
                error,
            ));
        }
        Ok(status.account)
    }

    fn binding_active(
        &self,
        deployment: &SharePointDeployment,
        local_inbox: &Path,
    ) -> Result<bool, SharePointSetupError> {
        Ok(self.0.fixed_binding_active(deployment, local_inbox))
    }

    fn activate_binding(
        &self,
        deployment: &SharePointDeployment,
        account: &Account,
        _library: &RemoteLibraryIdentity,
        local_inbox: &Path,
        commit: &mut dyn FnMut() -> Result<(), SharePointSetupError>,
    ) -> Result<(), SharePointSetupError> {
        self.0
            .activate_fixed_binding(deployment, account, local_inbox, commit)
            .map_err(|error| match error {
                crate::microsoft_intake::FixedBindingActivationError::Microsoft(message) => {
                    SharePointSetupError::new("MICROSOFT_BINDING_FAILED", message)
                }
                crate::microsoft_intake::FixedBindingActivationError::Commit(error) => error,
                crate::microsoft_intake::FixedBindingActivationError::Rollback {
                    commit,
                    restore,
                } => SharePointSetupError::new(
                    "ACTIVATION_ROLLBACK_FAILED",
                    format!(
                        "{} Microsoft binding rollback also failed: {restore}. Activation remains visibly incomplete and processing fails closed.",
                        commit.message
                    ),
                ),
            })
    }
}

struct ProductionSettings<'a>(&'a crate::commands::AppState);

impl SetupSettings for ProductionSettings<'_> {
    fn load(&self) -> Result<AppSettings, SharePointSetupError> {
        self.0
            .sharepoint_settings_snapshot()
            .map_err(|error| SharePointSetupError::new(error.code, error.message))
    }

    fn save(&self, settings: &AppSettings) -> Result<(), SharePointSetupError> {
        self.0
            .activate_sharepoint_settings(settings)
            .map_err(|error| SharePointSetupError::new(error.code, error.message))
    }
}

struct ProductionAutostart<'a>(&'a AppHandle);

impl AutostartBoundary for ProductionAutostart<'_> {
    fn is_enabled(&self) -> Result<bool, SharePointSetupError> {
        self.0.autolaunch().is_enabled().map_err(|_| {
            SharePointSetupError::new(
                "AUTOSTART_FAILED",
                "Intern could not read its start-at-login registration.",
            )
        })
    }

    fn set_enabled(&self, enabled: bool) -> Result<(), SharePointSetupError> {
        let manager = self.0.autolaunch();
        let result = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };
        result.map_err(|_| {
            SharePointSetupError::new(
                "AUTOSTART_FAILED",
                "Intern could not update its start-at-login registration.",
            )
        })
    }
}

struct ProductionOpener;

impl SyncOpener for ProductionOpener {
    fn open(&self, url: &Url) -> Result<(), SyncOpenFailure> {
        tauri_plugin_opener::open_url(url.as_str(), None::<&str>)
            .map_err(|_| SyncOpenFailure::OneDriveUnavailable)
    }
}

fn packaged_deployment() -> Result<SharePointDeployment, SharePointSetupError> {
    SharePointDeployment::from_slice(include_bytes!("../resources/sharepoint-deployment.json"))
        .map_err(|error| {
            SharePointSetupError::new("SHAREPOINT_DEPLOYMENT_UNAVAILABLE", error.to_string())
        })
}

fn production_operation(
    app: &AppHandle,
    operation: impl FnOnce(&SharePointSetup<'_>) -> Result<SharePointSetupStatus, SharePointSetupError>,
) -> Result<SharePointSetupStatus, SharePointSetupError> {
    // Deployment validation deliberately happens before any local discovery
    // or system handoff. The current packaged resource is disabled, so both
    // production commands fail with this exact support error and cannot ever
    // activate by a registry display name.
    let deployment = packaged_deployment()?;
    let roots = SystemRoots;
    let fs = SystemFileSystem;
    let verifier = UnavailableRemoteVerifier;
    let microsoft_state = app.state::<std::sync::Arc<crate::microsoft_intake::MicrosoftIntake>>();
    let microsoft = ProductionMicrosoft(microsoft_state.as_ref());
    let app_state = app.state::<crate::commands::AppState>();
    let settings = ProductionSettings(&app_state);
    let autostart = ProductionAutostart(app);
    let opener = ProductionOpener;
    let setup = SharePointSetup::new(
        &deployment,
        &roots,
        &fs,
        &verifier,
        &microsoft,
        &settings,
        &autostart,
        &opener,
    );
    operation(&setup)
}

/// No-payload IPC command. Work that can touch OneDrive/registry state runs
/// away from WebView2's invoke thread.
#[tauri::command]
pub async fn onboarding_start_sharepoint_sync(
    app: AppHandle,
) -> Result<SharePointSetupStatus, SharePointSetupError> {
    tauri::async_runtime::spawn_blocking(move || {
        production_operation(&app, |setup| setup.start_sync())
    })
    .await
    .map_err(|_| {
        SharePointSetupError::new(
            "SHAREPOINT_SETUP_TASK_FAILED",
            "The SharePoint sync check could not finish.",
        )
    })?
}

/// No-payload IPC command. Activation rescans and re-verifies from scratch;
/// nothing supplied by the webview can select a tenant, account, or path.
#[tauri::command]
pub async fn onboarding_activate(
    app: AppHandle,
) -> Result<SharePointSetupStatus, SharePointSetupError> {
    tauri::async_runtime::spawn_blocking(move || {
        production_operation(&app, |setup| setup.activate())
    })
    .await
    .map_err(|_| {
        SharePointSetupError::new(
            "SHAREPOINT_SETUP_TASK_FAILED",
            "SharePoint activation could not finish.",
        )
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use intern_intake::microsoft::Account;
    use intern_intake::{CloudProviderKind, CloudRoot, SharePointDeployment};
    use intern_queue::{AppSettings, DestinationLayout, ModelSource};
    use std::{
        collections::{HashMap, HashSet},
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    const TENANT_ID: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
    const SITE_ID: &str = "cccccccc-cccc-cccc-cccc-cccccccccccc";
    const WEB_ID: &str = "dddddddd-dddd-dddd-dddd-dddddddddddd";
    const LIST_ID: &str = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
    const DRIVE_ID: &str = "ffffffff-ffff-ffff-ffff-ffffffffffff";

    fn deployment() -> SharePointDeployment {
        SharePointDeployment::from_slice(
            br#"{
                "schema_version": 1,
                "enabled": true,
                "site_url": "https://teamcontoso.sharepoint.com/sites/InternTestSite",
                "library_name": "Files",
                "intake_folder_name": "Inbox",
                "destination_folder_name": "Filed",
                "tenant_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "client_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                "site_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
                "web_id": "dddddddd-dddd-dddd-dddd-dddddddddddd",
                "list_id": "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee",
                "drive_id": "ffffffff-ffff-ffff-ffff-ffffffffffff",
                "intake_folder_id": "11111111-1111-1111-1111-111111111111",
                "destination_folder_id": "22222222-2222-2222-2222-222222222222"
            }"#,
        )
        .expect("enabled test deployment")
    }

    fn account() -> Account {
        Account {
            tenant_id: TENANT_ID.into(),
            id: "33333333-3333-3333-3333-333333333333".into(),
            display_name: "Pat Contoso".into(),
            email: "pat+intern@contoso.com".into(),
            user_principal_name: "pat+intern@contoso.com".into(),
        }
    }

    fn root(path: &str) -> CloudRoot {
        CloudRoot {
            kind: CloudProviderKind::SharePoint,
            display_name: "Contoso - Files".into(),
            root: PathBuf::from(path),
        }
    }

    fn identity(path: &str) -> RemoteLibraryIdentity {
        RemoteLibraryIdentity {
            local_root: PathBuf::from(path),
            tenant_id: TENANT_ID.into(),
            site_id: SITE_ID.into(),
            web_id: WEB_ID.into(),
            list_id: LIST_ID.into(),
            drive_id: DRIVE_ID.into(),
        }
    }

    struct FakeRoots {
        state: OneDriveState,
        roots: Vec<CloudRoot>,
    }

    impl RootDetector for FakeRoots {
        fn one_drive_state(&self) -> OneDriveState {
            self.state
        }

        fn detect(&self) -> Result<Vec<CloudRoot>, SharePointSetupError> {
            Ok(self.roots.clone())
        }
    }

    #[derive(Default)]
    struct FakeFileSystem {
        canonical: HashMap<PathBuf, PathBuf>,
        writable: HashSet<PathBuf>,
    }

    impl SetupFileSystem for FakeFileSystem {
        fn canonicalize_directory(&self, path: &Path) -> Result<PathBuf, DirectoryFailure> {
            self.canonical
                .get(path)
                .cloned()
                .ok_or(DirectoryFailure::Missing)
        }

        fn is_writable_directory(&self, path: &Path) -> bool {
            self.writable.contains(path)
        }
    }

    #[derive(Default)]
    struct FakeVerifier {
        identities: Mutex<HashMap<PathBuf, RemoteLibraryIdentity>>,
        seen: Mutex<Vec<PathBuf>>,
    }

    impl RemoteLibraryVerifier for FakeVerifier {
        fn verify(
            &self,
            _account: &Account,
            candidate: &Path,
            _deployment: &SharePointDeployment,
        ) -> Result<Option<RemoteLibraryIdentity>, SharePointSetupError> {
            self.seen.lock().unwrap().push(candidate.to_path_buf());
            Ok(self.identities.lock().unwrap().get(candidate).cloned())
        }
    }

    enum ActivationMode {
        Success,
        FailBefore,
        FailAfter,
    }

    struct FakeMicrosoft {
        account: Mutex<Option<Account>>,
        binding_active: Mutex<bool>,
        binding_status_fails: bool,
        activation_mode: ActivationMode,
    }

    impl MicrosoftSetup for FakeMicrosoft {
        fn connected_account(&self) -> Result<Option<Account>, SharePointSetupError> {
            Ok(self.account.lock().unwrap().clone())
        }

        fn binding_active(
            &self,
            _deployment: &SharePointDeployment,
            _local_inbox: &Path,
        ) -> Result<bool, SharePointSetupError> {
            if self.binding_status_fails {
                return Err(SharePointSetupError::new(
                    "MICROSOFT_BINDING_STATUS_FAILED",
                    "binding status unavailable",
                ));
            }
            Ok(*self.binding_active.lock().unwrap())
        }

        fn activate_binding(
            &self,
            _deployment: &SharePointDeployment,
            _account: &Account,
            _library: &RemoteLibraryIdentity,
            _local_inbox: &Path,
            commit: &mut dyn FnMut() -> Result<(), SharePointSetupError>,
        ) -> Result<(), SharePointSetupError> {
            if matches!(self.activation_mode, ActivationMode::FailBefore) {
                return Err(SharePointSetupError::new(
                    "MICROSOFT_BINDING_FAILED",
                    "Microsoft binding could not be staged",
                ));
            }
            commit()?;
            if matches!(self.activation_mode, ActivationMode::FailAfter) {
                *self.binding_active.lock().unwrap() = false;
                return Err(SharePointSetupError::new(
                    "MICROSOFT_BINDING_FAILED",
                    "Microsoft binding could not be committed",
                ));
            }
            *self.binding_active.lock().unwrap() = true;
            Ok(())
        }
    }

    struct FakeSettings {
        saved: Mutex<AppSettings>,
        fail_next: Mutex<bool>,
        events: Arc<Mutex<Vec<&'static str>>>,
    }

    impl SetupSettings for FakeSettings {
        fn load(&self) -> Result<AppSettings, SharePointSetupError> {
            Ok(self.saved.lock().unwrap().clone())
        }

        fn save(&self, settings: &AppSettings) -> Result<(), SharePointSetupError> {
            self.events.lock().unwrap().push("settings");
            let mut fail = self.fail_next.lock().unwrap();
            if *fail {
                *fail = false;
                return Err(SharePointSetupError::new(
                    "SETTINGS_WRITE_FAILED",
                    "settings were not saved",
                ));
            }
            *self.saved.lock().unwrap() = settings.clone();
            Ok(())
        }
    }

    struct FakeAutostart {
        enabled: Mutex<bool>,
        fail_enable: bool,
        events: Arc<Mutex<Vec<&'static str>>>,
    }

    impl AutostartBoundary for FakeAutostart {
        fn is_enabled(&self) -> Result<bool, SharePointSetupError> {
            Ok(*self.enabled.lock().unwrap())
        }

        fn set_enabled(&self, enabled: bool) -> Result<(), SharePointSetupError> {
            self.events.lock().unwrap().push(if enabled {
                "autostart_on"
            } else {
                "autostart_off"
            });
            if enabled && self.fail_enable {
                return Err(SharePointSetupError::new(
                    "AUTOSTART_FAILED",
                    "start at login could not be enabled",
                ));
            }
            *self.enabled.lock().unwrap() = enabled;
            Ok(())
        }
    }

    struct FakeOpener {
        opened: Mutex<Vec<String>>,
        failure: Option<SyncOpenFailure>,
    }

    impl SyncOpener for FakeOpener {
        fn open(&self, url: &url::Url) -> Result<(), SyncOpenFailure> {
            self.opened.lock().unwrap().push(url.as_str().to_owned());
            self.failure.clone().map_or(Ok(()), Err)
        }
    }

    struct Rig {
        deployment: SharePointDeployment,
        roots: FakeRoots,
        fs: FakeFileSystem,
        verifier: FakeVerifier,
        microsoft: FakeMicrosoft,
        settings: FakeSettings,
        autostart: FakeAutostart,
        opener: FakeOpener,
    }

    impl Rig {
        fn empty() -> Self {
            let events = Arc::new(Mutex::new(Vec::new()));
            let previous = AppSettings {
                destination: r"C:\Legacy\Filed".into(),
                destination_layout: DestinationLayout::YearType,
                automatic_rename: true,
                intake_folder: r"C:\Legacy\Inbox".into(),
                intake_enabled: false,
                intake_local_only: true,
                process_others_uploads: true,
                machine_label: "Reception".into(),
                run_in_background: false,
                start_at_login: false,
                start_minimized: false,
                record_descriptions: true,
                model_source: ModelSource::Hosted,
                hosted_base_url: "https://api.example.test".into(),
                hosted_model: "model-v1".into(),
                ..AppSettings::default()
            };
            Self {
                deployment: deployment(),
                roots: FakeRoots {
                    state: OneDriveState::Available,
                    roots: Vec::new(),
                },
                fs: FakeFileSystem::default(),
                verifier: FakeVerifier::default(),
                microsoft: FakeMicrosoft {
                    account: Mutex::new(Some(account())),
                    binding_active: Mutex::new(false),
                    binding_status_fails: false,
                    activation_mode: ActivationMode::Success,
                },
                settings: FakeSettings {
                    saved: Mutex::new(previous),
                    fail_next: Mutex::new(false),
                    events: Arc::clone(&events),
                },
                autostart: FakeAutostart {
                    enabled: Mutex::new(false),
                    fail_enable: false,
                    events,
                },
                opener: FakeOpener {
                    opened: Mutex::new(Vec::new()),
                    failure: None,
                },
            }
        }

        fn with_verified_root(mut self, path: &str) -> Self {
            let root = PathBuf::from(path);
            let inbox = root.join("Inbox");
            let filed = root.join("Filed");
            self.roots.roots.push(super::tests::root(path));
            self.fs.canonical.insert(root.clone(), root.clone());
            self.fs.canonical.insert(inbox.clone(), inbox.clone());
            self.fs.canonical.insert(filed.clone(), filed.clone());
            self.fs.writable.extend([root.clone(), inbox, filed]);
            self.verifier
                .identities
                .lock()
                .unwrap()
                .insert(root, identity(path));
            self
        }

        fn setup(&self) -> SharePointSetup<'_> {
            SharePointSetup::new(
                &self.deployment,
                &self.roots,
                &self.fs,
                &self.verifier,
                &self.microsoft,
                &self.settings,
                &self.autostart,
                &self.opener,
            )
        }
    }

    fn code(result: Result<SharePointSetupStatus, SharePointSetupError>) -> String {
        result.expect_err("setup operation should fail").code
    }

    #[test]
    fn an_existing_root_is_ready_only_after_authoritative_identity_verification() {
        let rig = Rig::empty().with_verified_root(r"C:\Sync\Files");

        let status = rig.setup().status().expect("verified root status");

        assert_eq!(status.phase, SharePointSetupPhase::ReadyToActivate);
        assert!(rig.opener.opened.lock().unwrap().is_empty());
        assert_eq!(
            *rig.verifier.seen.lock().unwrap(),
            vec![PathBuf::from(r"C:\Sync\Files")]
        );
    }

    #[test]
    fn enrollment_uses_only_the_connected_account_and_encoded_deployment_url() {
        let rig = Rig::empty();

        let status = rig.setup().start_sync().expect("start fixed sync");

        assert_eq!(status.phase, SharePointSetupPhase::EnrollmentPending);
        let opened = rig.opener.opened.lock().unwrap();
        assert_eq!(opened.len(), 1);
        assert!(opened[0].starts_with("odopen://sync/"));
        assert!(opened[0].contains("userEmail=pat%2Bintern%40contoso.com"));
        assert!(opened[0].contains("listTitle=Files"));
    }

    #[test]
    fn missing_account_onedrive_account_and_onedrive_are_distinct() {
        let rig = Rig::empty();
        *rig.microsoft.account.lock().unwrap() = None;
        assert_eq!(code(rig.setup().start_sync()), "MICROSOFT_ACCOUNT_MISSING");

        let mut rig = Rig::empty();
        rig.roots.state = OneDriveState::AccountMissing;
        assert_eq!(code(rig.setup().start_sync()), "ONEDRIVE_ACCOUNT_MISSING");

        let mut rig = Rig::empty();
        rig.roots.state = OneDriveState::Missing;
        assert_eq!(code(rig.setup().start_sync()), "ONEDRIVE_MISSING");
    }

    #[test]
    fn a_missing_url_opener_is_not_reported_as_enrollment_pending() {
        let mut rig = Rig::empty();
        rig.opener.failure = Some(SyncOpenFailure::OpenerUnavailable);

        assert_eq!(code(rig.setup().start_sync()), "SYNC_OPENER_UNAVAILABLE");
    }

    #[test]
    fn display_names_and_paths_do_not_override_wrong_remote_identity() {
        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files-A");
        rig.roots.roots.push(root(r"C:\Sync\Files-B"));
        rig.fs.canonical.insert(
            PathBuf::from(r"C:\Sync\Files-B"),
            PathBuf::from(r"C:\Sync\Files-B"),
        );
        rig.verifier.identities.lock().unwrap().insert(
            PathBuf::from(r"C:\Sync\Files-A"),
            RemoteLibraryIdentity {
                site_id: "99999999-9999-9999-9999-999999999999".into(),
                ..identity(r"C:\Sync\Files-A")
            },
        );

        assert_eq!(code(rig.setup().status()), "SHAREPOINT_ROOT_UNVERIFIED");
    }

    #[test]
    fn two_exact_remote_matches_are_ambiguous_even_with_the_same_display_name() {
        let mut rig = Rig::empty()
            .with_verified_root(r"C:\Sync\Files-A")
            .with_verified_root(r"D:\Sync\Files-B");
        rig.roots.roots[1].display_name = rig.roots.roots[0].display_name.clone();

        assert_eq!(code(rig.setup().status()), "SHAREPOINT_ROOT_AMBIGUOUS");
    }

    #[test]
    fn nested_registered_library_roots_are_rejected() {
        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        let nested = PathBuf::from(r"C:\Sync\Files\Nested");
        rig.roots.roots.push(root(nested.to_str().unwrap()));
        rig.fs.canonical.insert(nested.clone(), nested);

        assert_eq!(code(rig.setup().status()), "SHAREPOINT_ROOT_NESTED");
    }

    #[test]
    fn missing_or_unwritable_fixed_children_never_change_settings() {
        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        let previous = rig.settings.saved.lock().unwrap().clone();
        rig.fs
            .canonical
            .remove(&PathBuf::from(r"C:\Sync\Files\Inbox"));
        assert_eq!(code(rig.setup().activate()), "INBOX_MISSING");
        assert_eq!(*rig.settings.saved.lock().unwrap(), previous);

        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        let previous = rig.settings.saved.lock().unwrap().clone();
        rig.fs
            .writable
            .remove(&PathBuf::from(r"C:\Sync\Files\Filed"));
        assert_eq!(code(rig.setup().activate()), "FILED_UNWRITABLE");
        assert_eq!(*rig.settings.saved.lock().unwrap(), previous);
    }

    #[test]
    fn activation_derives_managed_fields_and_preserves_unrelated_settings() {
        let rig = Rig::empty().with_verified_root(r"C:\Sync\Files");

        let status = rig.setup().activate().expect("activate verified root");

        assert_eq!(status.phase, SharePointSetupPhase::Active);
        let saved = rig.settings.saved.lock().unwrap();
        assert_eq!(saved.intake_folder, r"C:\Sync\Files\Inbox");
        assert_eq!(saved.destination, r"C:\Sync\Files\Filed");
        assert!(saved.intake_enabled);
        assert!(!saved.process_others_uploads);
        assert!(!saved.intake_local_only);
        assert!(saved.run_in_background);
        assert!(saved.start_at_login);
        assert!(saved.start_minimized);
        assert_eq!(saved.destination_layout, DestinationLayout::YearType);
        assert!(saved.automatic_rename);
        assert!(saved.record_descriptions);
        assert_eq!(saved.model_source, ModelSource::Hosted);
        assert_eq!(saved.machine_label, "Reception");
    }

    #[test]
    fn autostart_is_validated_before_settings_and_restored_when_save_fails() {
        let rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        *rig.settings.fail_next.lock().unwrap() = true;
        let previous = rig.settings.saved.lock().unwrap().clone();

        assert_eq!(code(rig.setup().activate()), "SETTINGS_WRITE_FAILED");

        assert_eq!(*rig.settings.saved.lock().unwrap(), previous);
        assert!(!*rig.autostart.enabled.lock().unwrap());
        assert_eq!(
            *rig.settings.events.lock().unwrap(),
            vec!["autostart_on", "settings", "autostart_off"]
        );
    }

    #[test]
    fn binding_failures_restore_settings_and_autostart_and_remain_incomplete() {
        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        rig.microsoft.activation_mode = ActivationMode::FailAfter;
        let previous = rig.settings.saved.lock().unwrap().clone();

        assert_eq!(code(rig.setup().activate()), "MICROSOFT_BINDING_FAILED");

        assert_eq!(*rig.settings.saved.lock().unwrap(), previous);
        assert!(!*rig.autostart.enabled.lock().unwrap());
        assert!(!*rig.microsoft.binding_active.lock().unwrap());
        assert_ne!(
            rig.setup().status().unwrap().phase,
            SharePointSetupPhase::Active
        );
    }

    #[test]
    fn binding_stage_failure_never_touches_autostart_or_settings() {
        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        rig.microsoft.activation_mode = ActivationMode::FailBefore;
        let previous = rig.settings.saved.lock().unwrap().clone();

        assert_eq!(code(rig.setup().activate()), "MICROSOFT_BINDING_FAILED");
        assert_eq!(*rig.settings.saved.lock().unwrap(), previous);
        assert!(rig.settings.events.lock().unwrap().is_empty());
    }

    #[test]
    fn a_successful_binding_publication_is_definitive_without_a_second_confirmation() {
        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        rig.microsoft.binding_status_fails = true;

        let activated = rig
            .setup()
            .activate()
            .expect("durably published binding needs no fallible status re-check");

        assert_eq!(activated.phase, SharePointSetupPhase::Active);
        assert!(rig.settings.saved.lock().unwrap().intake_enabled);
        assert!(*rig.autostart.enabled.lock().unwrap());
        assert!(*rig.microsoft.binding_active.lock().unwrap());
    }

    #[test]
    fn packaged_production_setup_stays_disabled_with_a_precise_error() {
        let error = packaged_deployment().expect_err("production identifiers are disabled");

        assert_eq!(error.code, "SHAREPOINT_DEPLOYMENT_UNAVAILABLE");
        assert!(error.message.contains(
            "SharePoint deployment configuration is unavailable: provisioned identifiers are not available in this build."
        ));
    }
}
