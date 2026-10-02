//! Fixed OneDrive enrollment and SharePoint-path activation.
//!
//! All decisions live behind injected boundaries so tests can prove the
//! deployment URL, remote identity, filesystem, Microsoft binding, settings,
//! and autostart transaction without OneDrive, Graph, or Tauri. Registry
//! display names and paths are discovery hints, never authority; production
//! verifies roots from OneDrive's own sync records
//! (`sharepoint_root_verifier`).

use crate::commands::SettingsRuntime;
use intern_intake::{
    CloudProviderKind, CloudRoot, RegistryHive, SharePointDeployment, detect_cloud_roots,
    microsoft::{Account, proof::is_guid},
    paths_overlap, registry_string, registry_subkeys, relative_to_root,
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
    /// While enrollment is pending, the record problem that kept a registered
    /// root from verifying, so waiting on OneDrive is never silent when
    /// OneDrive's records are the reason. `null` otherwise.
    pub problem: Option<SharePointSetupError>,
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
    /// No `odopen:` protocol handler is registered, so OneDrive cannot
    /// receive the sync request even if the OS opener launches it.
    ProtocolUnavailable,
    OpenerUnavailable,
    Other(String),
}

/// The recorded remote identity for one exact canonical local root: tenant,
/// site, web, and list. There is no drive; the sync records do not carry one.
/// A verifier result is still checked here; returning a display name or path
/// alone can never satisfy the service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteLibraryIdentity {
    pub local_root: PathBuf,
    pub tenant_id: String,
    pub site_id: String,
    pub web_id: String,
    pub list_id: String,
}

pub trait RootDetector {
    fn one_drive_state(&self) -> OneDriveState;
    /// Whether a OneDrive work or school account is signed in as `account`
    /// (its mail or principal name), rather than as someone else.
    fn one_drive_signed_in_as(&self, account: &Account) -> bool;
    fn detect(&self) -> Result<Vec<CloudRoot>, SharePointSetupError>;
}

pub trait SetupFileSystem {
    fn canonicalize_directory(&self, path: &Path) -> Result<PathBuf, DirectoryFailure>;
    fn is_writable_directory(&self, path: &Path) -> bool;
}

pub trait RemoteLibraryVerifier {
    /// `fs` is the boundary that canonicalized `candidate`; recorded folders
    /// must be compared through it too.
    fn verify(
        &self,
        account: &Account,
        candidate: &Path,
        deployment: &SharePointDeployment,
        fs: &dyn SetupFileSystem,
    ) -> Result<Option<RemoteLibraryIdentity>, SharePointSetupError>;
}

/// The callback is the local settings/autostart commit. Implementations must
/// stage Microsoft protection first and restore their prior bindings if the
/// callback or publication fails, keeping the staged Inbox protection: the
/// commit may already have watched the Inbox, so an aborted activation must
/// leave it held rather than unprotected. A successful return means
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
    /// Marks an activation as running and returns the settings stored at that
    /// moment. Until `end_activation`, ordinary settings saves are refused
    /// (SHAREPOINT_ACTIVATION_IN_PROGRESS), so a failed activation's restore
    /// cannot silently undo one.
    fn begin_activation(&self) -> Result<AppSettings, SharePointSetupError>;
    fn end_activation(&self);
    /// Applies activation settings. A failure has already restored what was
    /// stored before, or reports ACTIVATION_ROLLBACK_FAILED.
    fn save(&self, settings: &AppSettings) -> Result<(), SharePointSetupError>;
    /// Puts back settings that were stored before activation. Must never fall
    /// back to re-applying the activation settings when it fails.
    fn restore(&self, previous: &AppSettings) -> Result<(), SharePointSetupError>;
}

pub trait AutostartBoundary {
    fn is_enabled(&self) -> Result<bool, SharePointSetupError>;
    fn set_enabled(&self, enabled: bool) -> Result<(), SharePointSetupError>;
}

pub trait SyncOpener {
    fn open(&self, url: &Url) -> Result<(), SyncOpenFailure>;
}

/// Whether resolution may prove writability by creating a probe file.
#[derive(Clone, Copy, Eq, PartialEq)]
enum WriteProbe {
    Require,
    Skip,
}

struct ResolvedLibrary {
    identity: RemoteLibraryIdentity,
    inbox: PathBuf,
    destination: PathBuf,
}

/// Ends the activation on every exit from `activate`.
struct ActivationRunning<'a>(&'a dyn SetupSettings);

impl Drop for ActivationRunning<'_> {
    fn drop(&mut self) {
        self.0.end_activation();
    }
}

enum Resolution {
    Resolved(ResolvedLibrary),
    /// No registered root verified. `verifier_error` is the first record
    /// problem that kept a root from verifying, if any.
    Pending {
        verifier_error: Option<SharePointSetupError>,
    },
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
    /// away from the IPC thread, as the production commands below do.
    ///
    /// Side-effect free: it never launches OneDrive or writes settings or
    /// autostart, and it skips the write probes, which would create and
    /// delete files inside the synced SharePoint library on every poll.
    /// Writability is proven again by `start_sync` and `activate`.
    pub fn status(&self) -> Result<SharePointSetupStatus, SharePointSetupError> {
        let account = self.connected_account()?;
        let resolved = match self.resolve(&account, WriteProbe::Skip)? {
            Resolution::Resolved(resolved) => resolved,
            Resolution::Pending { verifier_error } => {
                return Ok(self.pending_status(&account, verifier_error));
            }
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
        let verifier_error = match self.resolve(&account, WriteProbe::Require)? {
            Resolution::Pending { verifier_error } => verifier_error,
            Resolution::Resolved(resolved) => {
                let settings = self.settings.load()?;
                let active =
                    managed_settings_match(&settings, &resolved.inbox, &resolved.destination)
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
        };
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
            SyncOpenFailure::ProtocolUnavailable => SharePointSetupError::new(
                "SYNC_PROTOCOL_UNAVAILABLE",
                "OneDrive is not registered to receive SharePoint sync requests on this computer. Repair or reinstall OneDrive, then try again.",
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
        Ok(self.pending_status(&account, verifier_error))
    }

    pub fn activate(&self) -> Result<SharePointSetupStatus, SharePointSetupError> {
        let account = self.connected_account()?;
        let resolved = match self.resolve(&account, WriteProbe::Require)? {
            Resolution::Resolved(resolved) => resolved,
            // Activation was asked for explicitly, so a record problem that
            // kept a registered root from verifying is the more useful answer.
            Resolution::Pending {
                verifier_error: Some(error),
            } => return Err(error),
            Resolution::Pending {
                verifier_error: None,
            } => {
                return Err(SharePointSetupError::new(
                    "SHAREPOINT_SYNC_PENDING",
                    "OneDrive has not registered the verified Files library yet. Keep OneDrive open and try again.",
                ));
            }
        };
        let previous = self.settings.begin_activation()?;
        let _running = ActivationRunning(self.settings);
        let previous_autostart = self.autostart.is_enabled()?;
        let mut next = previous.clone();
        next.intake_folder = resolved.inbox.to_string_lossy().into_owned();
        next.destination = resolved.destination.to_string_lossy().into_owned();
        next.intake_enabled = true;
        next.process_others_uploads = false;
        next.intake_local_only = false;
        next.intake_my_folder = false;
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
                    "OneDrive is not installed on this computer. Install OneDrive, sign in with your work account, then try again.",
                ));
            }
            OneDriveState::AccountMissing => {
                return Err(SharePointSetupError::new(
                    "ONEDRIVE_ACCOUNT_MISSING",
                    "OneDrive is installed but not signed in to a work or school account. Sign in to OneDrive with your work account, then try again.",
                ));
            }
            OneDriveState::Available => {}
        }
        let account = self.microsoft.connected_account()?.ok_or_else(|| {
            SharePointSetupError::new(
                "MICROSOFT_ACCOUNT_MISSING",
                "Connect the Microsoft account used for uploads before starting SharePoint sync.",
            )
        })?;
        if !is_guid(&account.id)
            || !account
                .tenant_id
                .eq_ignore_ascii_case(self.deployment.tenant_id())
        {
            return Err(SharePointSetupError::new(
                "MICROSOFT_ACCOUNT_WRONG_TENANT",
                "The connected Microsoft account is outside the provisioned organization tenant.",
            ));
        }
        if !self.roots.one_drive_signed_in_as(&account) {
            return Err(SharePointSetupError::new(
                "ONEDRIVE_ACCOUNT_MISMATCH",
                format!(
                    "OneDrive is signed in to a different work or school account. Sign in to OneDrive as {}, then try again.",
                    account.email
                ),
            ));
        }
        Ok(account)
    }

    /// Finds the one registered root OneDrive's records prove is the
    /// provisioned library. Other SharePoint and Teams libraries the person
    /// already syncs are ordinary: a root that does not verify is simply not
    /// ours, so it never blocks enrollment, and overlap is only checked
    /// between roots that verified. A verifier error for one root does not
    /// stop the scan either. It fails closed for that root only: the root
    /// never verifies, and when nothing else does the error is kept: `activate`
    /// reports it, while status and sync still see enrollment as pending, so
    /// OneDrive can be asked to sync the library, and carry the error as the
    /// status `problem`.
    fn resolve(
        &self,
        account: &Account,
        probe: WriteProbe,
    ) -> Result<Resolution, SharePointSetupError> {
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

        let mut verified = Vec::new();
        let mut verifier_error = None;
        for candidate in &candidates {
            match self
                .verifier
                .verify(account, candidate, self.deployment, self.fs)
            {
                Ok(Some(identity)) if self.identity_matches(candidate, &identity) => {
                    verified.push(identity);
                }
                Ok(_) => {}
                Err(error) => {
                    verifier_error.get_or_insert(error);
                }
            }
        }
        for (index, left) in verified.iter().enumerate() {
            if verified
                .iter()
                .skip(index + 1)
                .any(|right| paths_overlap(&left.local_root, &right.local_root))
            {
                return Err(SharePointSetupError::new(
                    "SHAREPOINT_ROOT_NESTED",
                    "OneDrive registered the Files library at overlapping folders. Intern cannot safely map local files until the duplicate or nested sync is removed.",
                ));
            }
        }
        let identity = match verified.len() {
            0 => return Ok(Resolution::Pending { verifier_error }),
            1 => verified.pop().expect("one verified identity"),
            _ => {
                return Err(SharePointSetupError::new(
                    "SHAREPOINT_ROOT_AMBIGUOUS",
                    "More than one local root verified as the provisioned Files library. Activation is blocked until the duplicate sync is removed.",
                ));
            }
        };
        if probe == WriteProbe::Require && !self.fs.is_writable_directory(&identity.local_root) {
            return Err(SharePointSetupError::new(
                "SHAREPOINT_ROOT_UNWRITABLE",
                "The verified Files library is not writable.",
            ));
        }
        let inbox = self.fixed_child(
            &identity.local_root,
            self.deployment.intake_folder_name(),
            "INBOX",
            probe,
        )?;
        let destination = self.fixed_child(
            &identity.local_root,
            self.deployment.destination_folder_name(),
            "FILED",
            probe,
        )?;
        if paths_overlap(&inbox, &destination) {
            return Err(SharePointSetupError::new(
                "FIXED_FOLDERS_AMBIGUOUS",
                "Inbox and Filed did not resolve to separate fixed children of the Files library.",
            ));
        }
        Ok(Resolution::Resolved(ResolvedLibrary {
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
        probe: WriteProbe,
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
        if probe == WriteProbe::Require && !self.fs.is_writable_directory(&child) {
            return Err(SharePointSetupError::new(
                format!("{code}_UNWRITABLE"),
                format!("The fixed {name} folder is not writable."),
            ));
        }
        Ok(child)
    }

    /// Re-checks the recorded identifiers against the deployment. No drive is
    /// compared: OneDrive's records carry no Graph drive ID, and the drive is
    /// proven per item by Graph at admission instead. The production verifier
    /// echoes the candidate as `local_root`, so that comparison can only catch
    /// a verifier implementation that reports some other folder.
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
    }

    fn restore_local_activation(
        &self,
        previous: &AppSettings,
        previous_autostart: bool,
        original: SharePointSetupError,
    ) -> SharePointSetupError {
        let settings = self.settings.restore(previous);
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
            problem: None,
        }
    }

    /// Enrollment is pending; a record problem from the scan rides along.
    fn pending_status(
        &self,
        account: &Account,
        problem: Option<SharePointSetupError>,
    ) -> SharePointSetupStatus {
        SharePointSetupStatus {
            problem,
            ..self.status_for(SharePointSetupPhase::EnrollmentPending, account)
        }
    }
}

fn managed_settings_match(settings: &AppSettings, inbox: &Path, destination: &Path) -> bool {
    settings.intake_folder == inbox.to_string_lossy()
        && settings.destination == destination.to_string_lossy()
        && settings.intake_enabled
        && !settings.process_others_uploads
        && !settings.intake_local_only
        && !settings.intake_my_folder
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
    result.map_or_else(
        |error| format!("{} ({})", error.code, error.message),
        |()| "ok".into(),
    )
}

/// Read-only machine facts the production OneDrive checks consult. Nothing
/// here ever writes the registry; tests substitute a fake machine.
trait MachineFacts {
    fn registry_string(&self, hive: RegistryHive, key: &str, value: &str) -> Option<String>;
    fn registry_subkeys(&self, hive: RegistryHive, key: &str) -> Vec<String>;
    fn env_var(&self, name: &str) -> Option<String>;
    fn is_file(&self, path: &Path) -> bool;
}

struct SystemMachine;

impl MachineFacts for SystemMachine {
    fn registry_string(&self, hive: RegistryHive, key: &str, value: &str) -> Option<String> {
        registry_string(hive, key, value)
    }

    fn registry_subkeys(&self, hive: RegistryHive, key: &str) -> Vec<String> {
        registry_subkeys(hive, key)
    }

    fn env_var(&self, name: &str) -> Option<String> {
        std::env::var(name)
            .ok()
            .filter(|value| !value.trim().is_empty())
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
}

const ONEDRIVE_ACCOUNTS_KEY: &str = r"Software\Microsoft\OneDrive\Accounts";

/// Installed means a OneDrive program actually exists: the per-user trigger
/// OneDrive records, or the per-user and per-machine install locations. A
/// registry value left behind by an uninstall is not an installation.
fn one_drive_client_installed(machine: &dyn MachineFacts) -> bool {
    one_drive_executable(machine).is_some()
}

/// The OneDrive program this user would start, found the same way.
fn one_drive_executable(machine: &dyn MachineFacts) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(trigger) = machine.registry_string(
        RegistryHive::CurrentUser,
        r"Software\Microsoft\OneDrive",
        "OneDriveTrigger",
    ) {
        candidates.push(PathBuf::from(trigger));
    }
    for (variable, relative) in [
        ("LOCALAPPDATA", r"Microsoft\OneDrive\OneDrive.exe"),
        ("ProgramFiles", r"Microsoft OneDrive\OneDrive.exe"),
        ("ProgramFiles(x86)", r"Microsoft OneDrive\OneDrive.exe"),
    ] {
        if let Some(base) = machine.env_var(variable) {
            candidates.push(PathBuf::from(base).join(relative));
        }
    }
    candidates
        .into_iter()
        .find(|candidate| machine.is_file(candidate))
}

/// Starts OneDrive, or brings its folder forward when it is already running:
/// a second launch of OneDrive.exe opens the OneDrive folder. Health checks
/// offer this when new documents cannot arrive.
pub(crate) fn open_one_drive() -> Result<(), SharePointSetupError> {
    let executable = one_drive_executable(&SystemMachine).ok_or_else(|| {
        SharePointSetupError::new(
            "ONEDRIVE_MISSING",
            "OneDrive is not installed on this computer.",
        )
    })?;
    std::process::Command::new(executable)
        .spawn()
        .map(drop)
        .map_err(|error| {
            SharePointSetupError::new(
                "ONEDRIVE_OPEN_FAILED",
                format!("OneDrive could not be started: {error}"),
            )
        })
}

/// OneDrive keeps an `Accounts\Business<N>` key per work or school account
/// slot, and records `UserEmail` only once someone has signed in to it.
fn one_drive_work_account_emails(machine: &dyn MachineFacts) -> Vec<String> {
    machine
        .registry_subkeys(RegistryHive::CurrentUser, ONEDRIVE_ACCOUNTS_KEY)
        .iter()
        .filter(|name| name.to_ascii_lowercase().starts_with("business"))
        .filter_map(|name| {
            machine.registry_string(
                RegistryHive::CurrentUser,
                &format!(r"{ONEDRIVE_ACCOUNTS_KEY}\{name}"),
                "UserEmail",
            )
        })
        .filter(|email| !email.trim().is_empty())
        .collect()
}

fn one_drive_work_account_signed_in(machine: &dyn MachineFacts) -> bool {
    !one_drive_work_account_emails(machine).is_empty()
}

fn one_drive_signed_in_as(machine: &dyn MachineFacts, account: &Account) -> bool {
    one_drive_work_account_emails(machine).iter().any(|email| {
        [&account.email, &account.user_principal_name]
            .iter()
            .any(|wanted| {
                !wanted.trim().is_empty() && email.trim().eq_ignore_ascii_case(wanted.trim())
            })
    })
}

fn one_drive_state(machine: &dyn MachineFacts) -> OneDriveState {
    if !one_drive_client_installed(machine) {
        OneDriveState::Missing
    } else if !one_drive_work_account_signed_in(machine) {
        OneDriveState::AccountMissing
    } else {
        OneDriveState::Available
    }
}

/// `HKEY_CLASSES_ROOT` merges the per-user and per-machine class
/// registrations, which is what the shell resolves `odopen:` against.
fn sync_protocol_registered(machine: &dyn MachineFacts) -> bool {
    machine
        .registry_string(RegistryHive::ClassesRoot, r"odopen\shell\open\command", "")
        .is_some()
}

struct SystemRoots<'a> {
    machine: &'a dyn MachineFacts,
}

impl RootDetector for SystemRoots<'_> {
    fn one_drive_state(&self) -> OneDriveState {
        // Root absence is the normal pre-enrollment state, not evidence that
        // OneDrive is absent, so only the client and account are checked.
        one_drive_state(self.machine)
    }

    fn one_drive_signed_in_as(&self, account: &Account) -> bool {
        one_drive_signed_in_as(self.machine, account)
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

/// The settings boundary over the live application runtime (`AppState` in
/// production). Generic so tests drive this exact adapter.
struct ProductionSettings<'a, R>(&'a R);

impl<R: SettingsRuntime> SetupSettings for ProductionSettings<'_, R> {
    fn load(&self) -> Result<AppSettings, SharePointSetupError> {
        self.0
            .load_settings()
            .map_err(|error| SharePointSetupError::new(error.code, error.message))
    }

    fn begin_activation(&self) -> Result<AppSettings, SharePointSetupError> {
        crate::commands::begin_sharepoint_activation(self.0)
            .map_err(|error| SharePointSetupError::new(error.code, error.message))
    }

    fn end_activation(&self) {
        crate::commands::end_sharepoint_activation(self.0);
    }

    fn save(&self, settings: &AppSettings) -> Result<(), SharePointSetupError> {
        crate::commands::activate_sharepoint_settings(self.0, settings)
            .map_err(|error| SharePointSetupError::new(error.code, error.message))
    }

    fn restore(&self, previous: &AppSettings) -> Result<(), SharePointSetupError> {
        crate::commands::restore_sharepoint_settings(self.0, previous)
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

type Launch<'a> = &'a dyn Fn(&str) -> Result<(), tauri_plugin_opener::Error>;

struct ProductionOpener<'a> {
    machine: &'a dyn MachineFacts,
    launch: Launch<'a>,
}

impl SyncOpener for ProductionOpener<'_> {
    /// The OS opener only reports whether a launcher process started; it
    /// cannot tell that nothing handles `odopen:`, so that is checked first.
    fn open(&self, url: &Url) -> Result<(), SyncOpenFailure> {
        if !sync_protocol_registered(self.machine) {
            return Err(SyncOpenFailure::ProtocolUnavailable);
        }
        (self.launch)(url.as_str()).map_err(|error| match error {
            tauri_plugin_opener::Error::UnsupportedPlatform => SyncOpenFailure::OpenerUnavailable,
            tauri_plugin_opener::Error::Io(error)
                if error.kind() == std::io::ErrorKind::NotFound =>
            {
                SyncOpenFailure::OpenerUnavailable
            }
            tauri_plugin_opener::Error::Io(error) => SyncOpenFailure::Other(error.to_string()),
            other => SyncOpenFailure::Other(other.to_string()),
        })
    }
}

fn launch_url(url: &str) -> Result<(), tauri_plugin_opener::Error> {
    tauri_plugin_opener::open_url(url, None::<&str>)
}

/// The deployment resource compiled into this build.
pub const PACKAGED_DEPLOYMENT: &[u8] = include_bytes!("../resources/sharepoint-deployment.json");

pub(crate) fn packaged_deployment() -> Result<SharePointDeployment, SharePointSetupError> {
    SharePointDeployment::from_slice(PACKAGED_DEPLOYMENT).map_err(|error| {
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
    let machine = SystemMachine;
    let roots = SystemRoots { machine: &machine };
    let fs = SystemFileSystem;
    let verifier = crate::sharepoint_root_verifier::OneDriveRecordVerifier::current_user();
    let microsoft_state = app.state::<std::sync::Arc<crate::microsoft_intake::MicrosoftIntake>>();
    let microsoft = ProductionMicrosoft(microsoft_state.as_ref());
    let app_state = app.state::<crate::commands::AppState>();
    let settings = ProductionSettings(app_state.inner());
    let autostart = ProductionAutostart(app);
    let opener = ProductionOpener {
        machine: &machine,
        launch: &launch_url,
    };
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

/// The current setup phase, read the way `onboarding_sharepoint_status` reads
/// it. Blocking: callers must run it away from the IPC thread.
pub(crate) fn current_phase(app: &AppHandle) -> Result<SharePointSetupPhase, SharePointSetupError> {
    production_operation(app, |setup| setup.status()).map(|status| status.phase)
}

/// No-payload, side-effect-free IPC command for polling setup progress: it
/// rescans and reads, but never launches OneDrive, writes settings or
/// autostart, or probes the synced library with a write.
#[tauri::command]
pub async fn onboarding_sharepoint_status(
    app: AppHandle,
) -> Result<SharePointSetupStatus, SharePointSetupError> {
    tauri::async_runtime::spawn_blocking(move || production_operation(&app, |setup| setup.status()))
        .await
        .map_err(|_| {
            SharePointSetupError::new(
                "SHAREPOINT_SETUP_TASK_FAILED",
                "The SharePoint setup status could not be read.",
            )
        })?
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
                "drive_id": "b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO",
                "intake_folder_id": "01SYNTHETICINBOXFOLDERAAAAAAAAAAAA",
                "destination_folder_id": "01SYNTHETICFILEDFOLDERAAAAAAAAAAAA"
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
        }
    }

    struct FakeRoots {
        state: OneDriveState,
        signed_in_as_connected: bool,
        roots: Vec<CloudRoot>,
    }

    impl RootDetector for FakeRoots {
        fn one_drive_state(&self) -> OneDriveState {
            self.state
        }

        fn one_drive_signed_in_as(&self, _account: &Account) -> bool {
            self.signed_in_as_connected
        }

        fn detect(&self) -> Result<Vec<CloudRoot>, SharePointSetupError> {
            Ok(self.roots.clone())
        }
    }

    #[derive(Default)]
    struct FakeFileSystem {
        canonical: HashMap<PathBuf, PathBuf>,
        writable: HashSet<PathBuf>,
        write_probes: Mutex<usize>,
    }

    impl SetupFileSystem for FakeFileSystem {
        fn canonicalize_directory(&self, path: &Path) -> Result<PathBuf, DirectoryFailure> {
            self.canonical
                .get(path)
                .cloned()
                .ok_or(DirectoryFailure::Missing)
        }

        fn is_writable_directory(&self, path: &Path) -> bool {
            *self.write_probes.lock().unwrap() += 1;
            self.writable.contains(path)
        }
    }

    #[derive(Default)]
    struct FakeVerifier {
        identities: Mutex<HashMap<PathBuf, RemoteLibraryIdentity>>,
        errors: Mutex<HashMap<PathBuf, SharePointSetupError>>,
        seen: Mutex<Vec<PathBuf>>,
    }

    impl RemoteLibraryVerifier for FakeVerifier {
        fn verify(
            &self,
            _account: &Account,
            candidate: &Path,
            _deployment: &SharePointDeployment,
            _fs: &dyn SetupFileSystem,
        ) -> Result<Option<RemoteLibraryIdentity>, SharePointSetupError> {
            self.seen.lock().unwrap().push(candidate.to_path_buf());
            if let Some(error) = self.errors.lock().unwrap().get(candidate) {
                return Err(error.clone());
            }
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
        activating: Mutex<bool>,
        /// Whether an activation was marked running at each save.
        saved_while_activating: Mutex<Vec<bool>>,
    }

    impl SetupSettings for FakeSettings {
        fn load(&self) -> Result<AppSettings, SharePointSetupError> {
            Ok(self.saved.lock().unwrap().clone())
        }

        fn begin_activation(&self) -> Result<AppSettings, SharePointSetupError> {
            let mut activating = self.activating.lock().unwrap();
            if *activating {
                return Err(SharePointSetupError::new(
                    "SHAREPOINT_ACTIVATION_IN_PROGRESS",
                    "already activating",
                ));
            }
            *activating = true;
            self.load()
        }

        fn end_activation(&self) {
            *self.activating.lock().unwrap() = false;
        }

        fn save(&self, settings: &AppSettings) -> Result<(), SharePointSetupError> {
            self.saved_while_activating
                .lock()
                .unwrap()
                .push(*self.activating.lock().unwrap());
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

        fn restore(&self, previous: &AppSettings) -> Result<(), SharePointSetupError> {
            self.save(previous)
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
                    signed_in_as_connected: true,
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
                    activating: Mutex::new(false),
                    saved_while_activating: Mutex::new(Vec::new()),
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

        /// A SharePoint root OneDrive registered for some other library, so
        /// the verifier does not claim it.
        fn with_unverified_root(mut self, path: &str) -> Self {
            let root = PathBuf::from(path);
            self.roots.roots.push(super::tests::root(path));
            self.fs.canonical.insert(root.clone(), root);
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
    fn status_never_launches_writes_or_probes_the_synced_library() {
        let rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        let previous = rig.settings.saved.lock().unwrap().clone();

        assert_eq!(
            rig.setup().status().unwrap().phase,
            SharePointSetupPhase::ReadyToActivate
        );
        rig.setup().activate().expect("activate");
        let events = rig.settings.events.lock().unwrap().clone();
        let activated = rig.settings.saved.lock().unwrap().clone();
        assert_ne!(activated, previous);
        *rig.fs.write_probes.lock().unwrap() = 0;

        assert_eq!(
            rig.setup().status().unwrap().phase,
            SharePointSetupPhase::Active
        );

        assert!(rig.opener.opened.lock().unwrap().is_empty());
        assert_eq!(*rig.settings.events.lock().unwrap(), events);
        assert_eq!(*rig.settings.saved.lock().unwrap(), activated);
        assert_eq!(
            *rig.fs.write_probes.lock().unwrap(),
            0,
            "a status poll must not create probe files in the synced SharePoint library"
        );
    }

    #[test]
    fn setup_commands_do_not_run_on_the_ipc_thread() {
        fn leaves_the_ipc_thread<A, R: std::future::Future, F: FnOnce(A) -> R>(_: F) {}
        leaves_the_ipc_thread(onboarding_sharepoint_status);
        leaves_the_ipc_thread(onboarding_start_sharepoint_sync);
        leaves_the_ipc_thread(onboarding_activate);
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

        let mut rig = Rig::empty();
        rig.roots.signed_in_as_connected = false;
        assert_eq!(code(rig.setup().start_sync()), "ONEDRIVE_ACCOUNT_MISMATCH");
        assert_eq!(code(rig.setup().status()), "ONEDRIVE_ACCOUNT_MISMATCH");
        assert!(rig.opener.opened.lock().unwrap().is_empty());
    }

    #[test]
    fn a_missing_url_opener_is_not_reported_as_enrollment_pending() {
        let mut rig = Rig::empty();
        rig.opener.failure = Some(SyncOpenFailure::OpenerUnavailable);

        assert_eq!(code(rig.setup().start_sync()), "SYNC_OPENER_UNAVAILABLE");
    }

    #[test]
    fn every_opener_failure_has_its_own_code() {
        let mut rig = Rig::empty();
        rig.opener.failure = Some(SyncOpenFailure::ProtocolUnavailable);
        assert_eq!(code(rig.setup().start_sync()), "SYNC_PROTOCOL_UNAVAILABLE");

        let mut rig = Rig::empty();
        rig.opener.failure = Some(SyncOpenFailure::Other("access is denied".into()));
        let error = rig.setup().start_sync().expect_err("launch failed");
        assert_eq!(error.code, "ONEDRIVE_OPEN_FAILED");
        assert!(error.message.contains("access is denied"));
    }

    /// Registry values, environment, and files the production OneDrive
    /// checks read, with nothing present unless a test adds it.
    #[derive(Default)]
    struct FakeMachine {
        strings: HashMap<(RegistryHive, String, String), String>,
        subkeys: HashMap<(RegistryHive, String), Vec<String>>,
        env: HashMap<String, String>,
        files: HashSet<PathBuf>,
    }

    impl FakeMachine {
        fn string(mut self, hive: RegistryHive, key: &str, value: &str, data: &str) -> Self {
            self.strings
                .insert((hive, key.into(), value.into()), data.into());
            self
        }

        fn accounts(mut self, names: &[&str]) -> Self {
            self.subkeys.insert(
                (RegistryHive::CurrentUser, ONEDRIVE_ACCOUNTS_KEY.into()),
                names.iter().map(|name| (*name).to_owned()).collect(),
            );
            self
        }

        fn installed(mut self) -> Self {
            self.env
                .insert("ProgramFiles".into(), r"C:\Program Files".into());
            self.files.insert(PathBuf::from(
                r"C:\Program Files\Microsoft OneDrive\OneDrive.exe",
            ));
            self
        }

        fn work_account(self) -> Self {
            self.accounts(&["Business1", "Personal"]).string(
                RegistryHive::CurrentUser,
                &format!(r"{ONEDRIVE_ACCOUNTS_KEY}\Business1"),
                "UserEmail",
                "pat@contoso.com",
            )
        }

        fn odopen(self) -> Self {
            self.string(
                RegistryHive::ClassesRoot,
                r"odopen\shell\open\command",
                "",
                r#""C:\Program Files\Microsoft OneDrive\OneDrive.exe" /url:"%1""#,
            )
        }
    }

    impl MachineFacts for FakeMachine {
        fn registry_string(&self, hive: RegistryHive, key: &str, value: &str) -> Option<String> {
            self.strings.get(&(hive, key.into(), value.into())).cloned()
        }

        fn registry_subkeys(&self, hive: RegistryHive, key: &str) -> Vec<String> {
            self.subkeys
                .get(&(hive, key.into()))
                .cloned()
                .unwrap_or_default()
        }

        fn env_var(&self, name: &str) -> Option<String> {
            self.env.get(name).cloned()
        }

        fn is_file(&self, path: &Path) -> bool {
            self.files.contains(path)
        }
    }

    #[test]
    fn production_onedrive_state_distinguishes_client_and_work_account() {
        assert_eq!(
            one_drive_state(&FakeMachine::default()),
            OneDriveState::Missing
        );
        // A stale trigger value without the program is not an installation.
        let stale = FakeMachine::default().string(
            RegistryHive::CurrentUser,
            r"Software\Microsoft\OneDrive",
            "OneDriveTrigger",
            r"C:\Gone\OneDrive.exe",
        );
        assert_eq!(one_drive_state(&stale), OneDriveState::Missing);
        let per_user = FakeMachine::default()
            .string(
                RegistryHive::CurrentUser,
                r"Software\Microsoft\OneDrive",
                "OneDriveTrigger",
                r"C:\Users\pat\AppData\Local\Microsoft\OneDrive\OneDrive.exe",
            )
            .work_account();
        let mut per_user = per_user;
        per_user.files.insert(PathBuf::from(
            r"C:\Users\pat\AppData\Local\Microsoft\OneDrive\OneDrive.exe",
        ));
        assert_eq!(one_drive_state(&per_user), OneDriveState::Available);

        assert_eq!(
            one_drive_state(&FakeMachine::default().installed()),
            OneDriveState::AccountMissing
        );
        // An account slot OneDrive created but never signed in, and a
        // personal account, are not a work account.
        let unsigned = FakeMachine::default()
            .installed()
            .accounts(&["Business1", "Personal"])
            .string(
                RegistryHive::CurrentUser,
                &format!(r"{ONEDRIVE_ACCOUNTS_KEY}\Personal"),
                "UserEmail",
                "pat@outlook.com",
            );
        assert_eq!(one_drive_state(&unsigned), OneDriveState::AccountMissing);
        assert_eq!(
            one_drive_state(&FakeMachine::default().installed().work_account()),
            OneDriveState::Available
        );
    }

    #[test]
    fn the_onedrive_program_is_found_where_it_is_installed() {
        assert_eq!(one_drive_executable(&FakeMachine::default()), None);
        assert_eq!(
            one_drive_executable(&FakeMachine::default().installed()),
            Some(PathBuf::from(
                r"C:\Program Files\Microsoft OneDrive\OneDrive.exe"
            ))
        );
    }

    #[test]
    fn production_onedrive_checks_block_sync_before_any_launch() {
        let launches = std::cell::Cell::new(0);
        let launch = |_: &str| {
            launches.set(launches.get() + 1);
            Ok(())
        };
        for (machine, expected) in [
            (FakeMachine::default(), "ONEDRIVE_MISSING"),
            (
                FakeMachine::default().installed().odopen(),
                "ONEDRIVE_ACCOUNT_MISSING",
            ),
            // Signed in, but as pat@ while Intern is connected as pat+intern@.
            (
                FakeMachine::default().installed().work_account().odopen(),
                "ONEDRIVE_ACCOUNT_MISMATCH",
            ),
        ] {
            let rig = Rig::empty();
            let roots = SystemRoots { machine: &machine };
            let opener = ProductionOpener {
                machine: &machine,
                launch: &launch,
            };
            let setup = SharePointSetup::new(
                &rig.deployment,
                &roots,
                &rig.fs,
                &rig.verifier,
                &rig.microsoft,
                &rig.settings,
                &rig.autostart,
                &opener,
            );
            assert_eq!(code(setup.start_sync()), expected);
        }
        assert_eq!(launches.get(), 0);
    }

    #[test]
    fn production_onedrive_account_matches_the_connected_mail_or_principal_name() {
        let launches = std::cell::Cell::new(0);
        let launch = |_: &str| {
            launches.set(launches.get() + 1);
            Ok(())
        };
        for signed_in in ["PAT+Intern@Contoso.com", "pat.principal@contoso.com"] {
            let machine = FakeMachine::default().installed().odopen().string(
                RegistryHive::CurrentUser,
                &format!(r"{ONEDRIVE_ACCOUNTS_KEY}\Business2"),
                "UserEmail",
                signed_in,
            );
            let machine = machine.accounts(&["Business1", "Business2"]).string(
                RegistryHive::CurrentUser,
                &format!(r"{ONEDRIVE_ACCOUNTS_KEY}\Business1"),
                "UserEmail",
                "someone.else@contoso.com",
            );
            let rig = Rig::empty();
            rig.microsoft
                .account
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .user_principal_name = "pat.principal@contoso.com".into();
            let roots = SystemRoots { machine: &machine };
            let opener = ProductionOpener {
                machine: &machine,
                launch: &launch,
            };
            let setup = SharePointSetup::new(
                &rig.deployment,
                &roots,
                &rig.fs,
                &rig.verifier,
                &rig.microsoft,
                &rig.settings,
                &rig.autostart,
                &opener,
            );
            assert_eq!(
                setup.start_sync().expect(signed_in).phase,
                SharePointSetupPhase::EnrollmentPending
            );
        }
        assert_eq!(launches.get(), 2);
    }

    #[test]
    fn production_opener_distinguishes_protocol_opener_and_launch_failures() {
        let url = Url::parse("odopen://sync/?userEmail=pat%40contoso.com").unwrap();
        let unregistered = FakeMachine::default().installed().work_account();
        let registered = FakeMachine::default().installed().work_account().odopen();
        let launches = std::cell::Cell::new(0);
        let succeed = |_: &str| {
            launches.set(launches.get() + 1);
            Ok(())
        };

        let opener = ProductionOpener {
            machine: &unregistered,
            launch: &succeed,
        };
        assert_eq!(opener.open(&url), Err(SyncOpenFailure::ProtocolUnavailable));
        assert_eq!(launches.get(), 0, "no launch without an odopen handler");

        let opener = ProductionOpener {
            machine: &registered,
            launch: &succeed,
        };
        assert_eq!(opener.open(&url), Ok(()));
        assert_eq!(launches.get(), 1);

        let missing_launcher = |_: &str| {
            Err(tauri_plugin_opener::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "program not found",
            )))
        };
        let opener = ProductionOpener {
            machine: &registered,
            launch: &missing_launcher,
        };
        assert_eq!(opener.open(&url), Err(SyncOpenFailure::OpenerUnavailable));

        let unsupported = |_: &str| Err(tauri_plugin_opener::Error::UnsupportedPlatform);
        let opener = ProductionOpener {
            machine: &registered,
            launch: &unsupported,
        };
        assert_eq!(opener.open(&url), Err(SyncOpenFailure::OpenerUnavailable));

        let denied = |_: &str| {
            Err(tauri_plugin_opener::Error::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "access is denied",
            )))
        };
        let opener = ProductionOpener {
            machine: &registered,
            launch: &denied,
        };
        assert_eq!(
            opener.open(&url),
            Err(SyncOpenFailure::Other("access is denied".into()))
        );
    }

    #[test]
    fn display_names_and_paths_do_not_override_wrong_remote_identity() {
        let rig = Rig::empty()
            .with_verified_root(r"C:\Sync\Files-A")
            .with_unverified_root(r"C:\Sync\Files-B");
        rig.verifier.identities.lock().unwrap().insert(
            PathBuf::from(r"C:\Sync\Files-A"),
            RemoteLibraryIdentity {
                site_id: "99999999-9999-9999-9999-999999999999".into(),
                ..identity(r"C:\Sync\Files-A")
            },
        );

        // Neither root is the provisioned library, so it is simply not synced
        // yet: status waits and sync can be launched, and nothing activates.
        assert_eq!(
            rig.setup().status().unwrap().phase,
            SharePointSetupPhase::EnrollmentPending
        );
        assert_eq!(code(rig.setup().activate()), "SHAREPOINT_SYNC_PENDING");
        assert_eq!(
            rig.setup().start_sync().unwrap().phase,
            SharePointSetupPhase::EnrollmentPending
        );
        assert_eq!(rig.opener.opened.lock().unwrap().len(), 1);
    }

    #[test]
    fn other_synced_sharepoint_libraries_do_not_block_enrollment() {
        let rig = Rig::empty().with_unverified_root(r"C:\Sync\Team Site - Documents");

        let status = rig.setup().status().unwrap();
        assert_eq!(status.phase, SharePointSetupPhase::EnrollmentPending);
        assert_eq!(status.problem, None);
        let started = rig.setup().start_sync().expect("launch sync");
        assert_eq!(started.phase, SharePointSetupPhase::EnrollmentPending);
        assert_eq!(started.problem, None);
        assert_eq!(rig.opener.opened.lock().unwrap().len(), 1);
        assert_eq!(code(rig.setup().activate()), "SHAREPOINT_SYNC_PENDING");
    }

    #[test]
    fn a_verifier_error_for_another_candidate_does_not_abort_the_scan() {
        let rig = Rig::empty()
            .with_unverified_root(r"C:\Sync\Broken")
            .with_verified_root(r"C:\Sync\Files");
        rig.verifier.errors.lock().unwrap().insert(
            PathBuf::from(r"C:\Sync\Broken"),
            SharePointSetupError::new("SHAREPOINT_ROOT_RECORD_CONFLICT", "records disagree"),
        );

        assert_eq!(
            rig.setup().status().unwrap().phase,
            SharePointSetupPhase::ReadyToActivate
        );
        assert_eq!(
            rig.setup().activate().expect("activate").phase,
            SharePointSetupPhase::Active
        );
        assert!(
            rig.verifier
                .seen
                .lock()
                .unwrap()
                .contains(&PathBuf::from(r"C:\Sync\Files")),
            "the scan continues past the failing candidate"
        );
    }

    #[test]
    fn a_verifier_error_with_nothing_verified_waits_but_activation_reports_it() {
        let rig = Rig::empty().with_unverified_root(r"C:\Sync\Files");
        rig.verifier.errors.lock().unwrap().insert(
            PathBuf::from(r"C:\Sync\Files"),
            SharePointSetupError::new("SHAREPOINT_ROOT_RECORD_MALFORMED", "unreadable record"),
        );

        let problem = Some(SharePointSetupError::new(
            "SHAREPOINT_ROOT_RECORD_MALFORMED",
            "unreadable record",
        ));
        let status = rig.setup().status().unwrap();
        assert_eq!(status.phase, SharePointSetupPhase::EnrollmentPending);
        assert_eq!(status.problem, problem, "waiting is never silent");
        let started = rig.setup().start_sync().unwrap();
        assert_eq!(started.phase, SharePointSetupPhase::EnrollmentPending);
        assert_eq!(started.problem, problem);
        assert_eq!(rig.opener.opened.lock().unwrap().len(), 1);
        let previous = rig.settings.saved.lock().unwrap().clone();
        assert_eq!(
            code(rig.setup().activate()),
            "SHAREPOINT_ROOT_RECORD_MALFORMED"
        );
        assert_eq!(*rig.settings.saved.lock().unwrap(), previous);
    }

    #[test]
    fn a_verified_library_carries_no_problem_from_another_candidate() {
        let rig = Rig::empty()
            .with_unverified_root(r"C:\Sync\Broken")
            .with_verified_root(r"C:\Sync\Files");
        rig.verifier.errors.lock().unwrap().insert(
            PathBuf::from(r"C:\Sync\Broken"),
            SharePointSetupError::new("SHAREPOINT_ROOT_RECORD_CONFLICT", "records disagree"),
        );

        assert_eq!(rig.setup().status().unwrap().problem, None);
        assert_eq!(rig.setup().activate().unwrap().problem, None);
    }

    #[test]
    fn the_status_problem_serializes_as_a_code_and_message_or_null() {
        let rig = Rig::empty().with_unverified_root(r"C:\Sync\Files");
        let plain = serde_json::to_value(rig.setup().status().unwrap()).unwrap();
        assert_eq!(plain["problem"], serde_json::Value::Null);

        rig.verifier.errors.lock().unwrap().insert(
            PathBuf::from(r"C:\Sync\Files"),
            SharePointSetupError::new("SHAREPOINT_ROOT_RECORD_CONFLICT", "records disagree"),
        );
        let waiting = serde_json::to_value(rig.setup().status().unwrap()).unwrap();
        assert_eq!(waiting["phase"], "enrollment_pending");
        assert_eq!(
            waiting["problem"],
            serde_json::json!({
                "code": "SHAREPOINT_ROOT_RECORD_CONFLICT",
                "message": "records disagree"
            })
        );
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
    fn nested_verified_library_roots_are_rejected() {
        let rig = Rig::empty()
            .with_verified_root(r"C:\Sync\Files")
            .with_verified_root(r"C:\Sync\Files\Nested");

        assert_eq!(code(rig.setup().status()), "SHAREPOINT_ROOT_NESTED");
    }

    #[test]
    fn an_unverified_root_nested_with_the_verified_library_is_not_ours_to_refuse() {
        for (verified, other) in [
            (r"C:\Sync\Files", r"C:\Sync\Files\Shortcut"),
            (r"C:\Sync\Team\Files", r"C:\Sync\Team"),
        ] {
            let rig = Rig::empty()
                .with_verified_root(verified)
                .with_unverified_root(other);

            assert_eq!(
                rig.setup().status().unwrap().phase,
                SharePointSetupPhase::ReadyToActivate,
                "{verified} with {other}"
            );
        }
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
    fn activation_is_marked_running_for_its_whole_transaction_and_always_ended() {
        let rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        rig.setup().activate().expect("activate");
        assert_eq!(*rig.settings.saved_while_activating.lock().unwrap(), [true]);
        assert!(!*rig.settings.activating.lock().unwrap());

        let mut rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        rig.microsoft.activation_mode = ActivationMode::FailAfter;
        assert_eq!(code(rig.setup().activate()), "MICROSOFT_BINDING_FAILED");
        assert_eq!(
            *rig.settings.saved_while_activating.lock().unwrap(),
            [true, true],
            "the commit and its restore"
        );
        assert!(!*rig.settings.activating.lock().unwrap());

        let rig = Rig::empty().with_verified_root(r"C:\Sync\Files");
        *rig.settings.activating.lock().unwrap() = true;
        assert_eq!(
            code(rig.setup().activate()),
            "SHAREPOINT_ACTIVATION_IN_PROGRESS"
        );
        assert!(
            *rig.settings.activating.lock().unwrap(),
            "a refused activation does not end the one already running"
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

    /// The production settings and Microsoft adapters over a real settings
    /// file, a real connected `MicrosoftIntake`, real folders, and the real
    /// settings application code. Only OneDrive discovery, the remote
    /// verifier, and the Tauri-owned live effects are recorded fakes.
    mod live {
        use super::*;
        use crate::commands::test_runtime::{Live, RecordingRuntime};
        use crate::microsoft_intake::{MicrosoftIntake, test_support::connected_manager};
        use intern_queue::{AdmissionGuard, AdmissionStage, SettingsStore};

        struct LiveAutostart<'a>(&'a LiveRig);

        impl AutostartBoundary for LiveAutostart<'_> {
            fn is_enabled(&self) -> Result<bool, SharePointSetupError> {
                Ok(self.0.runtime.live().autostart)
            }

            /// Enabling autostart is the first step of the activation commit,
            /// outside the settings gate: where a Settings save can race it.
            fn set_enabled(&self, enabled: bool) -> Result<(), SharePointSetupError> {
                if enabled && let Some(payload) = self.0.racing_save.lock().unwrap().take() {
                    let result = crate::commands::save_settings(&self.0.runtime, payload)
                        .map_err(|error| error.code);
                    *self.0.racing_result.lock().unwrap() = Some(result);
                }
                self.0
                    .runtime
                    .set_autostart(enabled)
                    .map_err(|error| SharePointSetupError::new(error.code, error.message))
            }
        }

        pub(super) struct LiveRig {
            pub(super) dir: PathBuf,
            pub(super) deployment: SharePointDeployment,
            pub(super) inbox: PathBuf,
            pub(super) filed: PathBuf,
            pub(super) legacy: PathBuf,
            pub(super) previous: AppSettings,
            roots: FakeRoots,
            verifier: FakeVerifier,
            pub(super) microsoft: Arc<MicrosoftIntake>,
            pub(super) runtime: RecordingRuntime,
            opener: FakeOpener,
            /// A Settings save to make while activation is under way.
            racing_save: Mutex<Option<AppSettings>>,
            racing_result: Mutex<Option<Result<(), String>>>,
        }

        impl Drop for LiveRig {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.dir);
            }
        }

        pub(super) fn text(path: &Path) -> String {
            path.to_string_lossy().into_owned()
        }

        impl LiveRig {
            pub(super) fn new(name: &str) -> Self {
                let dir = std::env::temp_dir().join(format!(
                    "intern-sharepoint-live-{name}-{}",
                    std::process::id()
                ));
                let _ = std::fs::remove_dir_all(&dir);
                for folder in ["Files/Inbox", "Files/Filed", "Legacy", "LegacyFiled"] {
                    std::fs::create_dir_all(dir.join(folder)).unwrap();
                }
                let root = dir.join("Files").canonicalize().unwrap();
                let inbox = dir.join("Files/Inbox").canonicalize().unwrap();
                let filed = dir.join("Files/Filed").canonicalize().unwrap();
                let legacy = dir.join("Legacy").canonicalize().unwrap();
                let previous = AppSettings {
                    destination: text(&dir.join("LegacyFiled").canonicalize().unwrap()),
                    intake_folder: text(&legacy),
                    intake_enabled: true,
                    intake_local_only: true,
                    process_others_uploads: false,
                    run_in_background: false,
                    start_at_login: false,
                    start_minimized: false,
                    automatic_rename: true,
                    ..AppSettings::default()
                };
                let mut runtime = RecordingRuntime::new(dir.join("settings.json"));
                runtime.store.save(&previous).unwrap();
                *runtime.live.lock().unwrap() = Live {
                    tray: false,
                    watcher: Some(text(&legacy)),
                    autostart: false,
                    intake_events: 0,
                };
                let microsoft = Arc::new(connected_manager(
                    SettingsStore::new(dir.join("settings.json")),
                    dir.join("app-data"),
                    deployment(),
                    &account(),
                ));
                runtime.microsoft = Some(Arc::clone(&microsoft));
                let verifier = FakeVerifier::default();
                verifier.identities.lock().unwrap().insert(
                    root.clone(),
                    RemoteLibraryIdentity {
                        local_root: root.clone(),
                        ..identity("")
                    },
                );
                Self {
                    deployment: deployment(),
                    roots: FakeRoots {
                        state: OneDriveState::Available,
                        signed_in_as_connected: true,
                        roots: vec![CloudRoot {
                            kind: CloudProviderKind::SharePoint,
                            display_name: "Contoso - Files".into(),
                            root,
                        }],
                    },
                    verifier,
                    microsoft,
                    runtime,
                    opener: FakeOpener {
                        opened: Mutex::new(Vec::new()),
                        failure: None,
                    },
                    racing_save: Mutex::new(None),
                    racing_result: Mutex::new(None),
                    dir,
                    inbox,
                    filed,
                    legacy,
                    previous,
                }
            }

            pub(super) fn run<T>(&self, operation: impl FnOnce(&SharePointSetup<'_>) -> T) -> T {
                let fs = SystemFileSystem;
                let microsoft = ProductionMicrosoft(self.microsoft.as_ref());
                let settings = ProductionSettings(&self.runtime);
                let autostart = LiveAutostart(self);
                operation(&SharePointSetup::new(
                    &self.deployment,
                    &self.roots,
                    &fs,
                    &self.verifier,
                    &microsoft,
                    &settings,
                    &autostart,
                    &self.opener,
                ))
            }

            pub(super) fn persisted(&self) -> AppSettings {
                self.runtime.store.load().unwrap()
            }

            fn binding_active(&self) -> bool {
                self.microsoft
                    .fixed_binding_active(&self.deployment, &self.inbox)
            }

            /// Whether a teammate's new Inbox upload would be admitted
            /// without Microsoft proof. Every Graph request fails in this
            /// rig, so only an unprotected path can be admitted.
            fn inbox_upload_admitted(&self) -> bool {
                let upload = self.inbox.join("teammate.pdf");
                std::fs::write(&upload, b"%PDF-1.7 teammate").unwrap();
                self.microsoft
                    .authorize(&upload, AdmissionStage::Enqueue)
                    .is_ok()
            }

            fn assert_previous_live_state(&self) {
                assert_eq!(self.persisted(), self.previous);
                let live = self.runtime.live();
                assert!(!live.tray, "tray restored");
                assert!(!live.autostart, "autostart restored");
            }
        }

        fn fail_restart_for(rig: &LiveRig, folder: &Path) {
            let folder = text(folder);
            *rig.runtime.fail_intake_restart.lock().unwrap() =
                Some(Box::new(move |settings| settings.intake_folder == folder));
        }

        /// A concurrent settings save during the local commit changes the
        /// Microsoft generation, so the binding must not be published.
        fn reconnect_during_commit(rig: &LiveRig) {
            let inbox = text(&rig.inbox);
            let microsoft = Arc::clone(&rig.microsoft);
            *rig.runtime.after_persist.lock().unwrap() = Some(Box::new(move |settings| {
                if settings.intake_folder == inbox {
                    microsoft.protect_settings(&AppSettings::default()).unwrap();
                }
            }));
        }

        #[test]
        fn activation_applies_every_live_effect_through_the_production_adapter() {
            let rig = LiveRig::new("success");

            let status = rig.run(|setup| setup.activate()).expect("activate");

            assert_eq!(status.phase, SharePointSetupPhase::Active);
            let saved = rig.persisted();
            assert_eq!(saved.intake_folder, text(&rig.inbox));
            assert_eq!(saved.destination, text(&rig.filed));
            assert!(saved.intake_enabled && saved.run_in_background && saved.start_at_login);
            assert!(!saved.intake_local_only && !saved.process_others_uploads);
            let live = rig.runtime.live();
            assert!(live.tray && live.autostart);
            assert_eq!(live.watcher, Some(text(&rig.inbox)));
            assert_eq!(live.intake_events, 1);
            assert!(rig.binding_active());
            assert_eq!(
                rig.run(|setup| setup.status()).unwrap().phase,
                SharePointSetupPhase::Active
            );
        }

        /// Everything the Settings dialog could send after activation, with
        /// every managed field pointed back at the legacy intake and every
        /// unrelated field changed.
        fn tampered_payload(rig: &LiveRig) -> AppSettings {
            let mut payload = rig.persisted();
            payload.intake_folder = text(&rig.legacy);
            payload.destination = rig.previous.destination.clone();
            payload.intake_enabled = false;
            payload.process_others_uploads = true;
            payload.intake_local_only = true;
            payload.run_in_background = false;
            payload.start_at_login = false;
            payload.start_minimized = false;
            payload.model_source = intern_queue::ModelSource::Hosted;
            payload.hosted_base_url = "https://api.example.test".into();
            payload.hosted_model = "model-v2".into();
            payload.destination_layout = intern_queue::DestinationLayout::YearType;
            payload.automatic_rename = !payload.automatic_rename;
            payload.record_descriptions = !payload.record_descriptions;
            payload.machine_label = "Front desk".into();
            payload
        }

        #[test]
        fn settings_saves_after_activation_keep_managed_paths_and_flags() {
            let rig = LiveRig::new("managed-save");
            rig.run(|setup| setup.activate()).expect("activate");
            let managed = rig.persisted();
            let payload = tampered_payload(&rig);

            crate::commands::save_settings(&rig.runtime, payload.clone())
                .expect("unrelated settings still save");

            let saved = rig.persisted();
            assert_eq!(saved.intake_folder, managed.intake_folder);
            assert_eq!(saved.destination, managed.destination);
            assert!(saved.intake_enabled);
            assert!(!saved.process_others_uploads);
            assert!(!saved.intake_local_only);
            assert!(saved.run_in_background && saved.start_at_login && saved.start_minimized);
            assert_eq!(saved.model_source, payload.model_source);
            assert_eq!(saved.hosted_model, payload.hosted_model);
            assert_eq!(saved.destination_layout, payload.destination_layout);
            assert_eq!(saved.automatic_rename, payload.automatic_rename);
            assert_eq!(saved.record_descriptions, payload.record_descriptions);
            assert_eq!(saved.machine_label, payload.machine_label);
            let live = rig.runtime.live();
            assert_eq!(live.watcher, Some(text(&rig.inbox)));
            assert!(live.tray && live.autostart);
            assert_eq!(
                rig.run(|setup| setup.status()).unwrap().phase,
                SharePointSetupPhase::Active
            );
        }

        #[test]
        fn settings_saves_before_activation_are_not_managed() {
            let rig = LiveRig::new("unmanaged-save");
            let mut payload = rig.previous.clone();
            payload.intake_enabled = false;
            payload.run_in_background = true;

            crate::commands::save_settings(&rig.runtime, payload.clone()).expect("ordinary save");

            assert_eq!(rig.persisted(), payload);
        }

        #[test]
        fn a_managed_binding_that_no_longer_matches_saved_settings_refuses_saves() {
            let rig = LiveRig::new("managed-mismatch");
            rig.run(|setup| setup.activate()).expect("activate");
            let mut edited = rig.persisted();
            edited.intake_folder = text(&rig.legacy);
            rig.runtime.store.save(&edited).unwrap();

            let error = crate::commands::save_settings(&rig.runtime, tampered_payload(&rig))
                .expect_err("cannot tell which paths are managed");

            assert_eq!(error.code, "SHAREPOINT_MANAGED_SETTINGS_UNAVAILABLE");
            assert_eq!(rig.persisted(), edited);
        }

        #[test]
        fn a_watcher_failure_restores_settings_runtime_and_autostart_and_holds_inbox_uploads() {
            let rig = LiveRig::new("watcher-failure");
            fail_restart_for(&rig, &rig.inbox);

            let error = rig
                .run(|setup| setup.activate())
                .expect_err("watcher failed");

            assert_eq!(error.code, "APP_DATA_UNAVAILABLE");
            rig.assert_previous_live_state();
            assert_eq!(rig.runtime.live().watcher, Some(text(&rig.legacy)));
            assert!(!rig.binding_active());
            assert!(
                !rig.inbox_upload_admitted(),
                "an aborted activation must leave the shared Inbox protected"
            );
        }

        #[test]
        fn restoring_prior_settings_does_not_revalidate_folders_that_have_since_gone() {
            let rig = LiveRig::new("stale-previous");
            fail_restart_for(&rig, &rig.inbox);
            std::fs::remove_dir_all(&rig.legacy).unwrap();

            let error = rig
                .run(|setup| setup.activate())
                .expect_err("watcher failed");

            assert_eq!(error.code, "APP_DATA_UNAVAILABLE", "{}", error.message);
            rig.assert_previous_live_state();
            assert!(!rig.binding_active());
        }

        #[test]
        fn a_failed_runtime_rollback_reports_both_errors_and_stays_fail_closed() {
            let rig = LiveRig::new("rollback-failure");
            rig.runtime
                .fail_intake_events
                .store(true, std::sync::atomic::Ordering::SeqCst);
            fail_restart_for(&rig, &rig.legacy);

            let error = rig
                .run(|setup| setup.activate())
                .expect_err("rollback failed");

            assert_eq!(error.code, "ACTIVATION_ROLLBACK_FAILED");
            assert!(
                error.message.contains("injected intake event failure")
                    && error.message.contains("injected watcher restart failure"),
                "{}",
                error.message
            );
            // The Inbox watcher could not be stopped, so admission itself must
            // hold the Inbox whatever the persisted settings say.
            assert_eq!(rig.runtime.live().watcher, Some(text(&rig.inbox)));
            assert!(!rig.binding_active());
            assert!(!rig.inbox_upload_admitted());
            assert_ne!(
                rig.run(|setup| setup.status()).unwrap().phase,
                SharePointSetupPhase::Active
            );
        }

        #[test]
        fn a_publication_failure_restores_prior_settings_and_holds_inbox_uploads() {
            let rig = LiveRig::new("publication-failure");
            reconnect_during_commit(&rig);

            let error = rig
                .run(|setup| setup.activate())
                .expect_err("publication failed");

            assert_eq!(error.code, "MICROSOFT_BINDING_FAILED");
            rig.assert_previous_live_state();
            assert_eq!(rig.runtime.live().watcher, Some(text(&rig.legacy)));
            assert!(!rig.binding_active());
            assert!(!rig.inbox_upload_admitted());
        }

        #[test]
        fn a_failed_restore_never_reapplies_the_managed_settings() {
            let rig = LiveRig::new("restore-failure");
            reconnect_during_commit(&rig);
            fail_restart_for(&rig, &rig.legacy);

            let error = rig
                .run(|setup| setup.activate())
                .expect_err("restore failed");

            assert_eq!(error.code, "ACTIVATION_ROLLBACK_FAILED");
            assert!(
                error.message.contains("injected watcher restart failure"),
                "{}",
                error.message
            );
            assert_eq!(rig.persisted(), rig.previous);
            assert!(!rig.runtime.live().autostart);
            assert!(!rig.binding_active());
            assert!(!rig.inbox_upload_admitted());
        }

        #[test]
        fn a_settings_save_racing_activation_fails_instead_of_being_silently_undone() {
            let rig = LiveRig::new("racing-save");
            fail_restart_for(&rig, &rig.inbox);
            let racing = AppSettings {
                machine_label: "Front desk".into(),
                ..rig.previous.clone()
            };
            *rig.racing_save.lock().unwrap() = Some(racing.clone());

            let error = rig
                .run(|setup| setup.activate())
                .expect_err("watcher failed");

            assert_eq!(error.code, "APP_DATA_UNAVAILABLE");
            assert_eq!(
                *rig.racing_result.lock().unwrap(),
                Some(Err("SHAREPOINT_ACTIVATION_IN_PROGRESS".into())),
                "the racing save was refused rather than reported saved and then rolled back"
            );
            rig.assert_previous_live_state();

            crate::commands::save_settings(&rig.runtime, racing.clone())
                .expect("the same save goes through once activation is over");
            assert_eq!(rig.persisted(), racing);
        }

        #[test]
        fn a_successful_activation_also_releases_settings_saves() {
            let rig = LiveRig::new("racing-success");
            *rig.racing_save.lock().unwrap() = Some(rig.previous.clone());

            rig.run(|setup| setup.activate()).expect("activate");

            assert_eq!(
                *rig.racing_result.lock().unwrap(),
                Some(Err("SHAREPOINT_ACTIVATION_IN_PROGRESS".into()))
            );
            crate::commands::save_settings(&rig.runtime, tampered_payload(&rig))
                .expect("saves work after activation");
        }
    }
}
