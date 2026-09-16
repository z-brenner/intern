//! Production local-to-remote verifier for SharePoint activation.
//!
//! The OneDrive sync client's own per-account records must name exactly one
//! whole-library sync at the candidate folder with the provisioned tenant,
//! site, web, and list. The records carry no Graph drive identifier, so none
//! is returned; the packaged drive's binding to that site, web, and list is
//! proven by Microsoft Graph on every admitted item
//! (`parentReference.driveId` together with `sharepointIds`). See
//! `docs/sharepoint-root-verification.md`.

use crate::sharepoint_setup::{
    RemoteLibraryIdentity, RemoteLibraryVerifier, SetupFileSystem, SharePointSetupError,
};
use intern_intake::{
    SharePointDeployment,
    microsoft::Account,
    onedrive_identity::{OneDriveRecords, SystemOneDriveRecords, verify_library_root},
};
use std::path::Path;

pub struct OneDriveRecordVerifier<R = SystemOneDriveRecords> {
    records: R,
}

impl OneDriveRecordVerifier {
    pub fn current_user() -> Self {
        Self {
            records: SystemOneDriveRecords::current_user(),
        }
    }
}

impl<R: OneDriveRecords> RemoteLibraryVerifier for OneDriveRecordVerifier<R> {
    /// Only records of the OneDrive account signed in as the connected
    /// Microsoft account count, and recorded folders are canonicalized through
    /// the same `fs` that canonicalized `candidate`.
    fn verify(
        &self,
        account: &Account,
        candidate: &Path,
        deployment: &SharePointDeployment,
        fs: &dyn SetupFileSystem,
    ) -> Result<Option<RemoteLibraryIdentity>, SharePointSetupError> {
        let library = verify_library_root(
            &self.records,
            candidate,
            deployment,
            &[&account.email, &account.user_principal_name],
            &|path| fs.canonicalize_directory(path).ok(),
        )
        .map_err(|error| SharePointSetupError::new(error.code(), error.message()))?;
        Ok(library.map(|library| RemoteLibraryIdentity {
            local_root: candidate.to_path_buf(),
            tenant_id: library.tenant_id,
            site_id: library.site_id,
            web_id: library.web_id,
            list_id: library.list_id,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sharepoint_setup::DirectoryFailure;
    use intern_intake::onedrive_identity::RecordError;
    use std::path::PathBuf;

    // Synthetic record in the documented libraryScope layout; all values are
    // invented. A tenant-backed Windows run must confirm the real format.
    const ROOT: &str = r"C:\Users\Pat\Contoso\InternTestSite - Files";
    const SCOPE: &str = "0123456789abcdef0123456789abcdef";
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

    fn account() -> Account {
        Account {
            tenant_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
            id: "33333333-3333-3333-3333-333333333333".into(),
            display_name: "Pat Contoso".into(),
            email: "pat@contoso.com".into(),
            user_principal_name: "pat@contoso.com".into(),
        }
    }

    fn utf16(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    struct Records {
        scope_ini: Option<String>,
        user_email: &'static str,
    }

    /// Leaves every path exactly as written.
    struct AsWritten;

    impl SetupFileSystem for AsWritten {
        fn canonicalize_directory(&self, _path: &Path) -> Result<PathBuf, DirectoryFailure> {
            Err(DirectoryFailure::Missing)
        }

        fn is_writable_directory(&self, _path: &Path) -> bool {
            false
        }
    }

    /// Canonicalizes only the recorded C: profile path, to its D: junction
    /// target; everything else is left as written.
    struct JunctionedProfile;

    impl SetupFileSystem for JunctionedProfile {
        fn canonicalize_directory(&self, path: &Path) -> Result<PathBuf, DirectoryFailure> {
            if path == Path::new(ROOT) {
                Ok(PathBuf::from(JUNCTION_TARGET))
            } else {
                Err(DirectoryFailure::Missing)
            }
        }

        fn is_writable_directory(&self, _path: &Path) -> bool {
            false
        }
    }

    const JUNCTION_TARGET: &str = r"\\?\D:\Users\Pat\Contoso\InternTestSite - Files";

    impl OneDriveRecords for Records {
        fn business_accounts(&self) -> Result<Vec<String>, RecordError> {
            Ok(vec!["Business1".into()])
        }

        fn settings_file(
            &self,
            _account: &str,
            name: &str,
        ) -> Result<Option<Vec<u8>>, RecordError> {
            Ok(match name {
                "global.ini" => Some(utf16(&format!("cid = {CID}\r\n"))),
                _ if name == format!("{CID}.ini") => self.scope_ini.as_deref().map(utf16),
                _ => None,
            })
        }

        fn scope_mount_points(&self, _account: &str) -> Result<Vec<(String, String)>, RecordError> {
            Ok(vec![(SCOPE.into(), ROOT.into())])
        }

        fn user_email(&self, _account: &str) -> Result<Option<String>, RecordError> {
            Ok(Some(self.user_email.into()))
        }
    }

    fn scope_line(list: &str) -> String {
        format!(
            "libraryScope = 1 {SCOPE} 5 \"InternTestSite\" \"Files\" 4 \
             \"https://teamcontoso.sharepoint.com/sites/InternTestSite\" \
             \"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa\" cccccccccccccccccccccccccccccccc \
             dddddddddddddddddddddddddddddddd {list} 1760590648 \"{ROOT}\" 1 \
             a547d3cb-8666-43cd-aa14-02270f7cee87  - 0 0 00000000-0000-0000-0000-000000000000\r\n"
        )
    }

    fn verifier(scope_ini: Option<String>) -> OneDriveRecordVerifier<Records> {
        OneDriveRecordVerifier {
            records: Records {
                scope_ini,
                user_email: "PAT@contoso.com",
            },
        }
    }

    const LIBRARY: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

    #[test]
    fn a_verified_record_returns_the_full_identity_for_the_candidate() {
        let candidate = PathBuf::from(r"\\?\C:\Users\Pat\Contoso\InternTestSite - Files");
        let identity = verifier(Some(scope_line(LIBRARY)))
            .verify(&account(), &candidate, &deployment(), &AsWritten)
            .expect("verification")
            .expect("verified identity");
        assert_eq!(
            identity,
            RemoteLibraryIdentity {
                local_root: candidate,
                tenant_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
                site_id: "cccccccc-cccc-cccc-cccc-cccccccccccc".into(),
                web_id: "dddddddd-dddd-dddd-dddd-dddddddddddd".into(),
                list_id: "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee".into(),
            }
        );
    }

    #[test]
    fn unmatched_or_absent_records_do_not_verify() {
        for scope_ini in [None, Some(scope_line("abababababababababababababababab"))] {
            assert_eq!(
                verifier(scope_ini).verify(&account(), Path::new(ROOT), &deployment(), &AsWritten),
                Ok(None)
            );
        }
    }

    #[test]
    fn broken_records_surface_a_stable_setup_code() {
        let error = verifier(Some("libraryScope = 1 \"unterminated\r\n".into()))
            .verify(&account(), Path::new(ROOT), &deployment(), &AsWritten)
            .unwrap_err();
        assert_eq!(error.code, "SHAREPOINT_ROOT_RECORD_MALFORMED");
    }

    #[test]
    fn a_junctioned_profile_verifies_through_the_setup_filesystem() {
        let candidate = PathBuf::from(JUNCTION_TARGET);
        let identity = verifier(Some(scope_line(LIBRARY)))
            .verify(&account(), &candidate, &deployment(), &JunctionedProfile)
            .expect("verification")
            .expect("the recorded C: folder canonicalizes to the candidate");
        assert_eq!(identity.local_root, candidate);
    }

    #[test]
    fn a_library_synced_by_another_onedrive_account_does_not_verify() {
        let mut verifier = verifier(Some(scope_line(LIBRARY)));
        verifier.records.user_email = "sam@contoso.com";
        assert_eq!(
            verifier.verify(&account(), Path::new(ROOT), &deployment(), &AsWritten),
            Ok(None)
        );
    }
}
