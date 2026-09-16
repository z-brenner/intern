//! Local-to-remote proof that a synced folder is the provisioned SharePoint
//! library, read from the OneDrive sync client's own per-account records.
//!
//! See `docs/sharepoint-root-verification.md` for the evidence behind the
//! record layout. The OS reads sit behind [`OneDriveRecords`]; everything that
//! decides lives in [`verify_library_root`] and is fixture-tested.

use std::{
    fs::File,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
};

use crate::{SharePointDeployment, cloud::path_components};

/// Read-only access to one Windows user's OneDrive sync-client records.
pub trait OneDriveRecords {
    /// Business account names, such as `Business1`.
    fn business_accounts(&self) -> Result<Vec<String>, RecordError>;
    /// Raw bytes of `settings\<account>\<name>`, or `None` when absent.
    fn settings_file(&self, account: &str, name: &str) -> Result<Option<Vec<u8>>, RecordError>;
    /// `Accounts\<account>\ScopeIdToMountPointPathCache` as (scope id, path).
    fn scope_mount_points(&self, account: &str) -> Result<Vec<(String, String)>, RecordError>;
}

/// The remote identifiers the sync client records for the verified root,
/// spelled as lowercase dashed GUIDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedLibrary {
    pub tenant_id: String,
    pub site_id: String,
    pub web_id: String,
    pub list_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordError {
    /// A record source exists but could not be read.
    Unavailable,
    /// A record source could not be parsed; nothing in it is trusted.
    Malformed(&'static str),
    /// Records disagree or more than one record claims the same thing.
    Conflict(&'static str),
}

impl RecordError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Unavailable => "SHAREPOINT_ROOT_RECORD_UNAVAILABLE",
            Self::Malformed(_) => "SHAREPOINT_ROOT_RECORD_MALFORMED",
            Self::Conflict(_) => "SHAREPOINT_ROOT_RECORD_CONFLICT",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Unavailable => {
                "OneDrive's sync records could not be read, so the Files library cannot be verified."
                    .into()
            }
            Self::Malformed(detail) => {
                format!("OneDrive's sync records are not in a recognised format ({detail}).")
            }
            Self::Conflict(detail) => format!("OneDrive's sync records disagree ({detail})."),
        }
    }
}

/// Returns the recorded remote identity for `candidate` only when exactly one
/// OneDrive `libraryScope` record mounts the whole library at that exact
/// folder, every identifier equals the packaged deployment, and the sync
/// engine's registry scope cache maps the same scope to the same folder.
pub fn verify_library_root(
    records: &dyn OneDriveRecords,
    candidate: &Path,
    deployment: &SharePointDeployment,
) -> Result<Option<VerifiedLibrary>, RecordError> {
    let wanted = DeploymentIds::from(deployment)?;
    let candidate_key = path_components(candidate);
    let mut at_candidate = Vec::new();
    let mut deployment_mounts = 0usize;
    for account in records.business_accounts()? {
        if !is_business_account(&account) {
            continue;
        }
        for scope in account_scopes(records, &account)? {
            // An empty mount means only folders inside the library are synced
            // (recorded on separate libraryFolder lines): not the library root.
            if scope.mount.is_empty() {
                continue;
            }
            if scope.ids == wanted {
                deployment_mounts += 1;
            }
            if path_components(Path::new(&scope.mount)) == candidate_key {
                at_candidate.push((account.clone(), scope));
            }
        }
    }
    let (account, scope) = match at_candidate.len() {
        0 => return Ok(None),
        1 => at_candidate.pop().expect("one record"),
        _ => return Err(RecordError::Conflict("folder records")),
    };
    if scope.ids != wanted {
        return Ok(None);
    }
    if deployment_mounts > 1 {
        return Err(RecordError::Conflict("library records"));
    }

    // The sync engine's registry cache must independently map this scope to
    // this folder, and no other scope to it. A stale settings line alone
    // does not verify.
    let mut corroborated = false;
    for (scope_id, path) in records.scope_mount_points(&account)? {
        let same_scope = normalize_scope_id(&scope_id).as_deref() == Some(scope.scope_id.as_str());
        let same_path = path_components(Path::new(&path)) == candidate_key;
        match (same_scope, same_path) {
            (true, true) => corroborated = true,
            (false, false) => {}
            _ => return Err(RecordError::Conflict("scope cache path")),
        }
    }
    Ok(corroborated.then(|| VerifiedLibrary {
        tenant_id: dashed(&scope.ids.tenant),
        site_id: dashed(&scope.ids.site),
        web_id: dashed(&scope.ids.web),
        list_id: dashed(&scope.ids.list),
    }))
}

/// Settings `.ini` files are a few kilobytes; never buffer an arbitrary file.
const MAX_SETTINGS_FILE_BYTES: u64 = 4 * 1024 * 1024;

/// The current Windows user's OneDrive records: files under
/// `%LOCALAPPDATA%\Microsoft\OneDrive\settings` and the per-account
/// `ScopeIdToMountPointPathCache` registry key. Elsewhere there are none.
#[derive(Clone, Debug)]
pub struct SystemOneDriveRecords {
    settings: Option<PathBuf>,
}

impl SystemOneDriveRecords {
    pub fn current_user() -> Self {
        let settings = cfg!(windows)
            .then(|| std::env::var_os("LOCALAPPDATA"))
            .flatten()
            .filter(|base| !base.is_empty())
            .map(|base| {
                PathBuf::from(base)
                    .join("Microsoft")
                    .join("OneDrive")
                    .join("settings")
            });
        Self { settings }
    }

    #[cfg(test)]
    fn at(settings: &Path) -> Self {
        Self {
            settings: Some(settings.to_path_buf()),
        }
    }
}

impl OneDriveRecords for SystemOneDriveRecords {
    fn business_accounts(&self) -> Result<Vec<String>, RecordError> {
        let Some(settings) = &self.settings else {
            return Ok(Vec::new());
        };
        let entries = match std::fs::read_dir(settings) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err(RecordError::Unavailable),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| RecordError::Unavailable)?;
            let is_dir = entry
                .file_type()
                .map_err(|_| RecordError::Unavailable)?
                .is_dir();
            if let Some(name) = entry.file_name().to_str()
                && is_dir
                && is_business_account(name)
            {
                names.push(name.to_owned());
            }
        }
        names.sort();
        Ok(names)
    }

    fn settings_file(&self, account: &str, name: &str) -> Result<Option<Vec<u8>>, RecordError> {
        if !is_business_account(account)
            || name.is_empty()
            || name.starts_with('.')
            || name.contains(['/', '\\', ':'])
        {
            return Err(RecordError::Malformed("settings file name"));
        }
        let Some(settings) = &self.settings else {
            return Ok(None);
        };
        let file = match File::open(settings.join(account).join(name)) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(RecordError::Unavailable),
        };
        let mut bytes = Vec::new();
        file.take(MAX_SETTINGS_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| RecordError::Unavailable)?;
        if bytes.len() as u64 > MAX_SETTINGS_FILE_BYTES {
            return Err(RecordError::Malformed("settings file size"));
        }
        Ok(Some(bytes))
    }

    fn scope_mount_points(&self, account: &str) -> Result<Vec<(String, String)>, RecordError> {
        if !is_business_account(account) {
            return Err(RecordError::Malformed("account name"));
        }
        #[cfg(windows)]
        {
            Ok(crate::cloud::windows_registry::current_user_string_values(
                &format!(
                    r"Software\Microsoft\OneDrive\Accounts\{account}\ScopeIdToMountPointPathCache"
                ),
            ))
        }
        #[cfg(not(windows))]
        {
            Ok(Vec::new())
        }
    }
}

/// Tenant, site, web, and list identifiers as 32 lowercase hex digits.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DeploymentIds {
    tenant: String,
    site: String,
    web: String,
    list: String,
}

impl DeploymentIds {
    fn from(deployment: &SharePointDeployment) -> Result<Self, RecordError> {
        let id = |value: &str| {
            normalize_hex32(value).ok_or(RecordError::Malformed("deployment identifier"))
        };
        Ok(Self {
            tenant: id(deployment.tenant_id())?,
            site: id(deployment.site_id())?,
            web: id(deployment.web_id())?,
            list: id(deployment.list_id())?,
        })
    }
}

struct ScopeRecord {
    scope_id: String,
    ids: DeploymentIds,
    mount: String,
}

fn is_business_account(name: &str) -> bool {
    name.strip_prefix("Business")
        .is_some_and(|digit| digit.len() == 1 && matches!(digit.as_bytes()[0], b'1'..=b'9'))
}

/// Every `libraryScope` record of one signed-in business account. A missing
/// `global.ini`, an empty `cid`, or a missing `<cid>.ini` means the account
/// has nothing enrolled yet.
fn account_scopes(
    records: &dyn OneDriveRecords,
    account: &str,
) -> Result<Vec<ScopeRecord>, RecordError> {
    let Some(global) = records.settings_file(account, "global.ini")? else {
        return Ok(Vec::new());
    };
    let global = decode_utf16(&global)?;
    let mut cids = global
        .lines()
        .filter_map(|line| line.strip_prefix("cid = "));
    let cid = match (cids.next(), cids.next()) {
        (None, _) => return Ok(Vec::new()),
        (Some(cid), None) => cid,
        (Some(_), Some(_)) => return Err(RecordError::Malformed("duplicate cid")),
    };
    if cid.is_empty() {
        return Ok(Vec::new());
    }
    if cid.len() > 64
        || !cid
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(RecordError::Malformed("account cid"));
    }
    let Some(scopes) = records.settings_file(account, &format!("{cid}.ini"))? else {
        return Ok(Vec::new());
    };
    decode_utf16(&scopes)?
        .lines()
        .filter_map(|line| line.strip_prefix("libraryScope = "))
        .map(parse_scope)
        .collect()
}

/// Field positions in a `libraryScope` record, after the tag.
const SCOPE_INDEX: usize = 0;
const SCOPE_ID: usize = 1;
const SCOPE_TENANT: usize = 7;
const SCOPE_SITE: usize = 8;
const SCOPE_WEB: usize = 9;
const SCOPE_LIST: usize = 10;
const SCOPE_MOUNT: usize = 12;

fn parse_scope(fields: &str) -> Result<ScopeRecord, RecordError> {
    let tokens = tokenize(fields)?;
    if tokens.len() <= SCOPE_MOUNT {
        return Err(RecordError::Malformed("libraryScope field count"));
    }
    if tokens[SCOPE_INDEX].is_empty() || !tokens[SCOPE_INDEX].bytes().all(|b| b.is_ascii_digit()) {
        return Err(RecordError::Malformed("libraryScope index"));
    }
    let hex = |index: usize, what: &'static str| {
        let token = tokens[index];
        (token.len() == 32)
            .then(|| normalize_hex32(token))
            .flatten()
            .ok_or(RecordError::Malformed(what))
    };
    let scope_id = normalize_scope_id(tokens[SCOPE_ID])
        .ok_or(RecordError::Malformed("libraryScope scope id"))?;
    let tenant = normalize_hex32(tokens[SCOPE_TENANT])
        .filter(|_| tokens[SCOPE_TENANT].len() >= 36)
        .ok_or(RecordError::Malformed("libraryScope tenant id"))?;
    Ok(ScopeRecord {
        scope_id,
        ids: DeploymentIds {
            tenant,
            site: hex(SCOPE_SITE, "libraryScope site id")?,
            web: hex(SCOPE_WEB, "libraryScope web id")?,
            list: hex(SCOPE_LIST, "libraryScope list id")?,
        },
        mount: tokens[SCOPE_MOUNT].to_owned(),
    })
}

/// Splits a record into space-separated fields; a field starting with `"`
/// runs to the next `"` (Windows paths cannot contain one).
fn tokenize(line: &str) -> Result<Vec<&str>, RecordError> {
    const SEPARATORS: [char; 2] = [' ', '\t'];
    let mut tokens = Vec::new();
    let mut rest = line;
    loop {
        rest = rest.trim_start_matches(SEPARATORS);
        if rest.is_empty() {
            return Ok(tokens);
        }
        if let Some(quoted) = rest.strip_prefix('"') {
            let end = quoted
                .find('"')
                .ok_or(RecordError::Malformed("unterminated quoted field"))?;
            rest = &quoted[end + 1..];
            if !(rest.is_empty() || rest.starts_with(SEPARATORS)) {
                return Err(RecordError::Malformed("quoted field boundary"));
            }
            tokens.push(&quoted[..end]);
        } else {
            let end = rest.find(SEPARATORS).unwrap_or(rest.len());
            if rest[..end].contains('"') {
                return Err(RecordError::Malformed("stray quote"));
            }
            tokens.push(&rest[..end]);
            rest = &rest[end..];
        }
    }
}

/// OneDrive writes its settings `.ini` files as UTF-16LE, usually without a
/// byte-order mark.
fn decode_utf16(bytes: &[u8]) -> Result<String, RecordError> {
    let bytes = bytes.strip_prefix(&[0xFF, 0xFE]).unwrap_or(bytes);
    if bytes.len() % 2 != 0 {
        return Err(RecordError::Malformed("UTF-16 length"));
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16(&units).map_err(|_| RecordError::Malformed("UTF-16 text"))
}

/// A scope ID as 32 lowercase hex digits. Business records have been seen
/// with a `+<number>` suffix (`<32 hex>+1`); the suffix only distinguishes
/// entries and is dropped so the settings and registry spellings compare.
fn normalize_scope_id(value: &str) -> Option<String> {
    let hex = match value.split_once('+') {
        Some((hex, number))
            if !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            hex
        }
        Some(_) => return None,
        None => value,
    };
    (hex.len() == 32 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| hex.to_ascii_lowercase())
}

/// A GUID as 32 lowercase hex digits, accepting the dashed, braced, and bare
/// spellings the sync client and deployment use.
fn normalize_hex32(value: &str) -> Option<String> {
    let inner = value
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
        .unwrap_or(value);
    let hex: String = match inner.len() {
        32 => inner.to_owned(),
        36 if inner
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 8 | 13 | 18 | 23) == (byte == b'-')) =>
        {
            inner.replace('-', "")
        }
        _ => return None,
    };
    (hex.len() == 32 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| hex.to_ascii_lowercase())
}

fn dashed(hex: &str) -> String {
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Synthetic fixtures. The layout follows the sync client's
    // `libraryScope` record as documented in
    // docs/sharepoint-root-verification.md; every identifier, path, and URL
    // below is invented. A tenant-backed Windows run must confirm the real
    // record before release.
    const TENANT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
    const SITE: &str = "cccccccccccccccccccccccccccccccc";
    const WEB: &str = "dddddddddddddddddddddddddddddddd";
    const LIST: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const SCOPE: &str = "0123456789abcdef0123456789abcdef";
    const MYSITE_SCOPE: &str = "fedcba9876543210fedcba9876543210";
    const ROOT: &str = r"C:\Users\Pat\Contoso\InternTestSite - Files";
    const CID: &str = "9f8e7d6c-5b4a-4938-8271-605f4e3d2c1b";

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

    fn scope_line(
        index: u32,
        scope: &str,
        tenant: &str,
        ids: (&str, &str, &str),
        mount: &str,
    ) -> String {
        format!(
            "libraryScope = {index} {scope} 5 \"InternTestSite\" \"Files\" 4 \
             \"https://teamcontoso.sharepoint.com/sites/InternTestSite\" \"{tenant}\" \
             {} {} {} 1760590648 \"{mount}\" 1 a547d3cb-8666-43cd-aa14-02270f7cee87  - \
             8725724278134823 3167371729 00000000-0000-0000-0000-000000000000 ",
            ids.0, ids.1, ids.2
        )
    }

    fn library_line(mount: &str) -> String {
        scope_line(1, SCOPE, TENANT, (SITE, WEB, LIST), mount)
    }

    fn mysite_line() -> String {
        format!(
            "libraryScope = 0 {MYSITE_SCOPE} 5 \"MySite\" \"ODB\" 2 \
             \"https://teamcontoso-my.sharepoint.com/personal/pat_contoso_com\" \"{TENANT}\" \
             11111111111111111111111111111111 22222222222222222222222222222222 \
             33333333333333333333333333333333 1760590648 \"C:\\Users\\Pat\\OneDrive - Contoso\" 1 \
             a547d3cb-8666-43cd-aa14-02270f7cee87  - 0 0 00000000-0000-0000-0000-000000000000 "
        )
    }

    fn utf16(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    #[derive(Default)]
    struct Fixture {
        accounts: Vec<String>,
        files: HashMap<(String, String), Vec<u8>>,
        caches: HashMap<String, Vec<(String, String)>>,
        unavailable: bool,
    }

    impl Fixture {
        /// One Business1 account with a personal OneDrive scope and the
        /// provisioned library mounted at `ROOT`.
        fn standard() -> Self {
            let mut fixture = Self::default();
            fixture.account(
                "Business1",
                CID,
                &[mysite_line(), library_line(ROOT)],
                &[
                    (MYSITE_SCOPE, r"C:\Users\Pat\OneDrive - Contoso"),
                    (SCOPE, ROOT),
                ],
            );
            fixture
        }

        fn account(&mut self, name: &str, cid: &str, lines: &[String], cache: &[(&str, &str)]) {
            self.accounts.push(name.into());
            self.files.insert(
                (name.into(), "global.ini".into()),
                utf16(&format!("mode = 1\r\ncid = {cid}\r\n")),
            );
            self.files.insert(
                (name.into(), format!("{cid}.ini")),
                utf16(&(lines.join("\r\n") + "\r\n")),
            );
            self.caches.insert(
                name.into(),
                cache
                    .iter()
                    .map(|(scope, path)| ((*scope).into(), (*path).into()))
                    .collect(),
            );
        }

        fn set_scope_ini(&mut self, account: &str, bytes: Vec<u8>) {
            self.files
                .insert((account.into(), format!("{CID}.ini")), bytes);
        }
    }

    impl OneDriveRecords for Fixture {
        fn business_accounts(&self) -> Result<Vec<String>, RecordError> {
            if self.unavailable {
                return Err(RecordError::Unavailable);
            }
            Ok(self.accounts.clone())
        }

        fn settings_file(&self, account: &str, name: &str) -> Result<Option<Vec<u8>>, RecordError> {
            Ok(self.files.get(&(account.into(), name.into())).cloned())
        }

        fn scope_mount_points(&self, account: &str) -> Result<Vec<(String, String)>, RecordError> {
            Ok(self.caches.get(account).cloned().unwrap_or_default())
        }
    }

    fn verify(fixture: &Fixture, candidate: &str) -> Result<Option<VerifiedLibrary>, RecordError> {
        verify_library_root(fixture, Path::new(candidate), &deployment())
    }

    fn expected() -> VerifiedLibrary {
        VerifiedLibrary {
            tenant_id: TENANT.into(),
            site_id: "cccccccc-cccc-cccc-cccc-cccccccccccc".into(),
            web_id: "dddddddd-dddd-dddd-dddd-dddddddddddd".into(),
            list_id: "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee".into(),
        }
    }

    #[test]
    fn an_exact_library_scope_record_verifies_the_canonical_root() {
        let fixture = Fixture::standard();
        assert_eq!(verify(&fixture, ROOT), Ok(Some(expected())));
        assert_eq!(
            verify(&fixture, r"\\?\c:\users\pat\CONTOSO\InternTestSite - Files"),
            Ok(Some(expected())),
            "the canonical verbatim spelling and case are the same folder"
        );
    }

    #[test]
    fn identifiers_compare_case_insensitively_without_dashes_or_braces() {
        let mut fixture = Fixture::default();
        let upper = scope_line(
            1,
            &SCOPE.to_ascii_uppercase(),
            &format!("{{{}}}", TENANT.to_ascii_uppercase()),
            (
                &SITE.to_ascii_uppercase(),
                &WEB.to_ascii_uppercase(),
                &LIST.to_ascii_uppercase(),
            ),
            ROOT,
        );
        fixture.account("Business1", CID, &[upper], &[(SCOPE, ROOT)]);
        assert_eq!(verify(&fixture, ROOT), Ok(Some(expected())));
    }

    #[test]
    fn a_numbered_scope_id_matches_its_registry_cache_entry() {
        let mut fixture = Fixture::default();
        fixture.account(
            "Business1",
            CID,
            &[scope_line(
                1,
                &format!("{SCOPE}+1"),
                TENANT,
                (SITE, WEB, LIST),
                ROOT,
            )],
            &[(SCOPE, ROOT)],
        );
        assert_eq!(verify(&fixture, ROOT), Ok(Some(expected())));
    }

    #[test]
    fn a_leading_byte_order_mark_is_accepted() {
        let mut fixture = Fixture::standard();
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend(utf16(&(library_line(ROOT) + "\r\n")));
        fixture.set_scope_ini("Business1", bytes);
        assert_eq!(verify(&fixture, ROOT), Ok(Some(expected())));
    }

    #[test]
    fn any_mismatched_identifier_does_not_verify() {
        let other = "99999999999999999999999999999999";
        let variants = [
            scope_line(
                1,
                SCOPE,
                "99999999-9999-9999-9999-999999999999",
                (SITE, WEB, LIST),
                ROOT,
            ),
            scope_line(1, SCOPE, TENANT, (other, WEB, LIST), ROOT),
            scope_line(1, SCOPE, TENANT, (SITE, other, LIST), ROOT),
            scope_line(1, SCOPE, TENANT, (SITE, WEB, other), ROOT),
        ];
        for line in variants {
            let mut fixture = Fixture::default();
            fixture.account("Business1", CID, &[line.clone()], &[(SCOPE, ROOT)]);
            assert_eq!(verify(&fixture, ROOT), Ok(None), "{line}");
        }
    }

    #[test]
    fn site_url_and_titles_never_authorize_a_different_list() {
        // Same site URL, same "Files" title, same folder name; only the list
        // identifier differs, as for a second library with a copied title.
        let line = scope_line(
            1,
            SCOPE,
            TENANT,
            (SITE, WEB, "abababababababababababababababab"),
            ROOT,
        );
        let mut fixture = Fixture::default();
        fixture.account("Business1", CID, &[line], &[(SCOPE, ROOT)]);
        assert_eq!(verify(&fixture, ROOT), Ok(None));
    }

    #[test]
    fn a_different_or_nested_folder_does_not_verify() {
        let fixture = Fixture::standard();
        for candidate in [
            r"C:\Users\Pat\Contoso",
            r"C:\Users\Pat\Contoso\InternTestSite - Files\Inbox",
            r"C:\Users\Pat\Contoso\InternTestSite - Files2",
            r"C:\Users\Pat\OneDrive - Contoso",
        ] {
            assert_eq!(verify(&fixture, candidate), Ok(None), "{candidate}");
        }
    }

    #[test]
    fn a_subfolder_sync_is_not_the_library_root() {
        // Syncing a folder inside the library leaves the scope's mount empty
        // and records the folder on a separate libraryFolder line.
        let mut fixture = Fixture::default();
        fixture.account(
            "Business1",
            CID,
            &[
                library_line(""),
                format!(
                    "libraryFolder = 3 1 {SCOPE}+1 1656974198 \"{ROOT}\" 1 \"Inbox\" \
                     68da3f5f-721e-4b7d-86cb-904988ac670f 32932572275197086 1017134003 \
                     00000000-0000-0000-0000-000000000000 "
                ),
            ],
            &[(SCOPE, ROOT)],
        );
        assert_eq!(verify(&fixture, ROOT), Ok(None));
    }

    #[test]
    fn missing_accounts_or_settings_files_are_not_yet_enrolled() {
        assert_eq!(verify(&Fixture::default(), ROOT), Ok(None));

        let mut no_scope_file = Fixture::standard();
        no_scope_file
            .files
            .remove(&("Business1".into(), format!("{CID}.ini")));
        assert_eq!(verify(&no_scope_file, ROOT), Ok(None));

        let mut no_global = Fixture::standard();
        no_global
            .files
            .remove(&("Business1".into(), "global.ini".into()));
        assert_eq!(verify(&no_global, ROOT), Ok(None));

        let mut signed_out = Fixture::standard();
        signed_out.files.insert(
            ("Business1".into(), "global.ini".into()),
            utf16("mode = 1\r\ncid = \r\n"),
        );
        assert_eq!(verify(&signed_out, ROOT), Ok(None));
    }

    #[test]
    fn the_registry_scope_cache_must_corroborate_the_record() {
        let mut missing = Fixture::standard();
        missing.caches.insert(
            "Business1".into(),
            vec![(
                MYSITE_SCOPE.into(),
                r"C:\Users\Pat\OneDrive - Contoso".into(),
            )],
        );
        assert_eq!(verify(&missing, ROOT), Ok(None));

        let mut elsewhere = Fixture::standard();
        elsewhere.caches.insert(
            "Business1".into(),
            vec![(SCOPE.into(), r"C:\Users\Pat\Elsewhere".into())],
        );
        assert_eq!(
            verify(&elsewhere, ROOT),
            Err(RecordError::Conflict("scope cache path"))
        );

        let mut other_scope = Fixture::standard();
        other_scope.caches.insert(
            "Business1".into(),
            vec![
                (SCOPE.into(), ROOT.into()),
                (MYSITE_SCOPE.into(), ROOT.to_ascii_lowercase()),
            ],
        );
        assert_eq!(
            verify(&other_scope, ROOT),
            Err(RecordError::Conflict("scope cache path"))
        );
    }

    #[test]
    fn two_records_for_the_same_folder_fail_closed() {
        let mut same_account = Fixture::default();
        same_account.account(
            "Business1",
            CID,
            &[library_line(ROOT), library_line(&ROOT.to_ascii_uppercase())],
            &[(SCOPE, ROOT)],
        );
        assert_eq!(
            verify(&same_account, ROOT),
            Err(RecordError::Conflict("folder records"))
        );

        let mut two_accounts = Fixture::standard();
        two_accounts.account(
            "Business2",
            "0f1e2d3c-4b5a-4968-8778-695a4b3c2d1e",
            &[scope_line(
                0,
                "abcdefabcdefabcdefabcdefabcdefab",
                TENANT,
                ("12121212121212121212121212121212", WEB, LIST),
                ROOT,
            )],
            &[("abcdefabcdefabcdefabcdefabcdefab", ROOT)],
        );
        assert_eq!(
            verify(&two_accounts, ROOT),
            Err(RecordError::Conflict("folder records"))
        );
    }

    #[test]
    fn the_library_synced_to_two_folders_fails_closed() {
        let mut fixture = Fixture::standard();
        fixture.account(
            "Business2",
            "0f1e2d3c-4b5a-4968-8778-695a4b3c2d1e",
            &[scope_line(
                0,
                "abcdefabcdefabcdefabcdefabcdefab",
                TENANT,
                (SITE, WEB, LIST),
                r"D:\Contoso\InternTestSite - Files",
            )],
            &[(
                "abcdefabcdefabcdefabcdefabcdefab",
                r"D:\Contoso\InternTestSite - Files",
            )],
        );
        assert_eq!(
            verify(&fixture, ROOT),
            Err(RecordError::Conflict("library records"))
        );
    }

    #[test]
    fn malformed_records_fail_closed() {
        let unterminated = library_line(ROOT).replace("\"Files\"", "\"Files");
        let short = format!("libraryScope = 1 {SCOPE} 5 \"InternTestSite\"");
        let bad_scope = scope_line(1, "not-a-scope", TENANT, (SITE, WEB, LIST), ROOT);
        let bad_suffix = scope_line(1, &format!("{SCOPE}+x"), TENANT, (SITE, WEB, LIST), ROOT);
        let bad_tenant = scope_line(1, SCOPE, "tenant", (SITE, WEB, LIST), ROOT);
        let bad_site = scope_line(1, SCOPE, TENANT, ("c0ffee", WEB, LIST), ROOT);
        let bad_index = scope_line(1, SCOPE, TENANT, (SITE, WEB, LIST), ROOT).replacen(
            "libraryScope = 1",
            "libraryScope = x",
            1,
        );
        for line in [
            unterminated,
            short,
            bad_scope,
            bad_suffix,
            bad_tenant,
            bad_site,
            bad_index,
        ] {
            let mut fixture = Fixture::default();
            fixture.account("Business1", CID, &[line.clone()], &[(SCOPE, ROOT)]);
            assert!(
                matches!(verify(&fixture, ROOT), Err(RecordError::Malformed(_))),
                "{line}"
            );
        }

        let mut odd = Fixture::standard();
        let mut bytes = utf16(&library_line(ROOT));
        bytes.push(0);
        odd.set_scope_ini("Business1", bytes);
        assert!(matches!(verify(&odd, ROOT), Err(RecordError::Malformed(_))));

        let mut unpaired = Fixture::standard();
        let mut bytes = utf16(&library_line(ROOT));
        bytes.extend([0x00, 0xD8]);
        unpaired.set_scope_ini("Business1", bytes);
        assert!(matches!(
            verify(&unpaired, ROOT),
            Err(RecordError::Malformed(_))
        ));
    }

    #[test]
    fn a_settings_cid_cannot_name_another_file() {
        for cid in [r"..\Business2\x", "a/b", "..", "a b", "a:b"] {
            let mut fixture = Fixture::standard();
            fixture.files.insert(
                ("Business1".into(), "global.ini".into()),
                utf16(&format!("cid = {cid}\r\n")),
            );
            let result = verify(&fixture, ROOT);
            assert!(
                matches!(result, Err(RecordError::Malformed(_))),
                "{cid}: {result:?}"
            );
        }
    }

    #[test]
    fn duplicate_cid_lines_fail_closed() {
        let mut fixture = Fixture::standard();
        fixture.files.insert(
            ("Business1".into(), "global.ini".into()),
            utf16(&format!(
                "cid = {CID}\r\ncid = 0f1e2d3c-4b5a-4968-8778-695a4b3c2d1e\r\n"
            )),
        );
        assert!(matches!(
            verify(&fixture, ROOT),
            Err(RecordError::Malformed(_))
        ));
    }

    #[test]
    fn system_records_read_only_business_settings_files() {
        let settings = tempfile::tempdir().expect("settings directory");
        for directory in ["Business1", "Business2", "Personal", "Business10"] {
            std::fs::create_dir(settings.path().join(directory)).expect("account");
        }
        std::fs::write(settings.path().join("Business12"), b"not a folder").expect("file");
        std::fs::write(
            settings.path().join("Business1").join("global.ini"),
            utf16("cid = x\r\n"),
        )
        .expect("global.ini");
        let records = SystemOneDriveRecords::at(settings.path());

        assert_eq!(
            records.business_accounts(),
            Ok(vec!["Business1".to_owned(), "Business2".to_owned()])
        );
        assert_eq!(
            records.settings_file("Business1", "global.ini"),
            Ok(Some(utf16("cid = x\r\n")))
        );
        assert_eq!(records.settings_file("Business2", "global.ini"), Ok(None));
        for (account, name) in [
            ("Personal", "global.ini"),
            ("Business1", r"..\Personal\global.ini"),
            ("Business1", "../global.ini"),
            ("Business1", "C:global.ini"),
            ("Business1", ""),
        ] {
            assert!(
                matches!(
                    records.settings_file(account, name),
                    Err(RecordError::Malformed(_))
                ),
                "{account} {name}"
            );
        }

        let missing = SystemOneDriveRecords::at(&settings.path().join("absent"));
        assert_eq!(missing.business_accounts(), Ok(Vec::new()));
    }

    #[test]
    fn system_records_refuse_oversized_settings_files() {
        let settings = tempfile::tempdir().expect("settings directory");
        std::fs::create_dir(settings.path().join("Business1")).expect("account");
        let file = File::create(settings.path().join("Business1").join("big.ini")).expect("file");
        file.set_len(MAX_SETTINGS_FILE_BYTES + 2).expect("size");
        let records = SystemOneDriveRecords::at(settings.path());
        assert_eq!(
            records.settings_file("Business1", "big.ini"),
            Err(RecordError::Malformed("settings file size"))
        );
    }

    #[test]
    fn unreadable_sources_fail_closed_with_a_stable_code() {
        let fixture = Fixture {
            unavailable: true,
            ..Fixture::standard()
        };
        let error = verify(&fixture, ROOT).unwrap_err();
        assert_eq!(error.code(), "SHAREPOINT_ROOT_RECORD_UNAVAILABLE");
        assert_eq!(
            RecordError::Malformed("x").code(),
            "SHAREPOINT_ROOT_RECORD_MALFORMED"
        );
        assert_eq!(
            RecordError::Conflict("x").code(),
            "SHAREPOINT_ROOT_RECORD_CONFLICT"
        );
    }
}
