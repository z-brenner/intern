use std::{
    collections::VecDeque,
    path::Path,
    sync::{Arc, Mutex},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use intern_intake::{
    SharePointDeployment,
    microsoft::{
        Account,
        hashing::QuickXor,
        proof::{FreshUploadMetadata, FreshUploadOutcome, verify_fresh_upload},
    },
};
use reqwest::Url;
use serde_json::{Value, json};

const TENANT: &str = "11111111-1111-1111-1111-111111111111";
const CLIENT: &str = "22222222-2222-2222-2222-222222222222";
const SITE: &str = "33333333-3333-3333-3333-333333333333";
const LIST: &str = "55555555-5555-5555-5555-555555555555";
const DRIVE: &str = "66666666-6666-6666-6666-666666666666";
const INBOX: &str = "77777777-7777-7777-7777-777777777777";
const ME: &str = "99999999-9999-9999-9999-999999999999";
const OTHER: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

fn deployment() -> SharePointDeployment {
    SharePointDeployment::from_slice(
        format!(
            r#"{{
              "schema_version": 1,
              "enabled": true,
              "site_url": "https://teamcontoso.sharepoint.com/sites/InternTestSite",
              "library_name": "Files",
              "intake_folder_name": "Inbox",
              "destination_folder_name": "Filed",
              "tenant_id": "{TENANT}",
              "client_id": "{CLIENT}",
              "site_id": "{SITE}",
              "web_id": "44444444-4444-4444-4444-444444444444",
              "list_id": "{LIST}",
              "drive_id": "{DRIVE}",
              "intake_folder_id": "{INBOX}",
              "destination_folder_id": "88888888-8888-8888-8888-888888888888"
            }}"#
        )
        .as_bytes(),
    )
    .unwrap()
}

fn me() -> Account {
    Account {
        tenant_id: TENANT.into(),
        id: ME.into(),
        display_name: "Pat Example".into(),
        email: "pat@example.test".into(),
        user_principal_name: "pat@example.test".into(),
    }
}

fn quick_xor(bytes: &[u8]) -> String {
    let mut hash = QuickXor::default();
    hash.update(bytes);
    STANDARD.encode(hash.finish())
}

fn metadata() -> Value {
    json!({
        "id": "item!123",
        "eTag": "\"fresh,1\"",
        "cTag": "\"content,1\"",
        "name": "agreement.pdf",
        "size": 5,
        "webUrl": "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox/agreement.pdf",
        "parentReference": { "driveId": DRIVE, "id": INBOX },
        "sharepointIds": {
            "tenantId": TENANT,
            "siteId": SITE,
            "listId": LIST,
            "listItemUniqueId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"
        },
        "createdBy": { "user": { "id": ME, "displayName": "Pat Example", "userPrincipalName": "pat@example.test" } },
        "lastModifiedBy": { "user": { "id": ME, "displayName": "Pat Example", "userPrincipalName": "pat@example.test" } },
        "createdDateTime": "2026-09-14T16:00:00Z",
        "lastModifiedDateTime": "2026-09-14T16:00:00Z",
        "file": {
            "mimeType": "application/pdf",
            "hashes": { "quickXorHash": quick_xor(b"hello") }
        }
    })
}

#[derive(Default)]
struct ScriptedMetadata {
    replies: Mutex<VecDeque<Result<(Account, Value), String>>>,
    urls: Mutex<Vec<Url>>,
}

impl ScriptedMetadata {
    fn new(replies: impl IntoIterator<Item = Result<(Account, Value), String>>) -> Arc<Self> {
        Arc::new(Self {
            replies: Mutex::new(replies.into_iter().collect()),
            urls: Mutex::new(Vec::new()),
        })
    }
}

impl FreshUploadMetadata for ScriptedMetadata {
    fn metadata(&self, url: Url) -> Result<(Account, Value), String> {
        self.urls.lock().unwrap().push(url);
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected metadata request")
    }
}

fn verify(source: &ScriptedMetadata, inbox: &Path, file: &Path) -> FreshUploadOutcome {
    verify_fresh_upload(&deployment(), source, inbox, file)
}

fn fresh_file() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("agreement.pdf");
    std::fs::write(&file, b"hello").unwrap();
    (directory, file)
}

#[test]
fn exact_account_ids_authorize_only_after_two_fixed_boundary_reads_and_local_binding() {
    let (directory, file) = fresh_file();
    let source = ScriptedMetadata::new([Ok((me(), metadata())), Ok((me(), metadata()))]);

    let outcome = verify(&source, directory.path(), &file);

    assert_eq!(
        outcome,
        FreshUploadOutcome::Authorized {
            local_sha256: HELLO_SHA256.into(),
            uploader: me(),
        }
    );
    let urls = source.urls.lock().unwrap();
    assert_eq!(urls.len(), 2);
    assert_eq!(urls[0], urls[1]);
    assert_eq!(
        urls[0].path(),
        format!("/v1.0/drives/{DRIVE}/items/{INBOX}:/agreement.pdf")
    );
    assert_eq!(urls[0].query_pairs().count(), 1);
    assert_eq!(urls[0].query_pairs().next().unwrap().0, "$select");
    assert!(!urls[0].path().contains("auditLog"));
    assert!(!urls[0].path().ends_with("/content"));
}

#[test]
fn matching_verified_principals_are_a_fallback_only_when_both_ids_are_absent() {
    let (directory, file) = fresh_file();
    let mut value = metadata();
    value["createdBy"]["user"]
        .as_object_mut()
        .unwrap()
        .remove("id");
    value["lastModifiedBy"]["user"]
        .as_object_mut()
        .unwrap()
        .remove("id");
    let source = ScriptedMetadata::new([Ok((me(), value.clone())), Ok((me(), value))]);

    assert!(matches!(
        verify(&source, directory.path(), &file),
        FreshUploadOutcome::Authorized { .. }
    ));
}

#[test]
fn a_conflicting_creator_id_is_never_overridden_by_a_matching_principal() {
    let (directory, file) = fresh_file();
    let mut value = metadata();
    value["createdBy"]["user"]["id"] = json!(OTHER);
    value["lastModifiedBy"]["user"]["id"] = json!(OTHER);
    let source = ScriptedMetadata::new([Ok((me(), value))]);

    let outcome = verify(&source, directory.path(), &file);

    assert!(matches!(
        outcome,
        FreshUploadOutcome::HeldOther { ref uploader, .. } if uploader.id == OTHER
    ));
    assert_eq!(source.urls.lock().unwrap().len(), 1);
}

#[test]
fn a_known_other_creator_and_an_unknown_creator_have_distinct_hold_outcomes() {
    let (directory, file) = fresh_file();
    let mut other = metadata();
    other["createdBy"]["user"]["id"] = json!(OTHER);
    other["createdBy"]["user"]["userPrincipalName"] = json!("other@example.test");
    other["lastModifiedBy"]["user"]["id"] = json!(OTHER);
    other["lastModifiedBy"]["user"]["userPrincipalName"] = json!("other@example.test");
    let source = ScriptedMetadata::new([Ok((me(), other))]);
    assert!(matches!(
        verify(&source, directory.path(), &file),
        FreshUploadOutcome::HeldOther { .. }
    ));

    let mut unknown = metadata();
    unknown["createdBy"]["user"] = json!({ "displayName": "Unknown" });
    let source = ScriptedMetadata::new([Ok((me(), unknown))]);
    assert!(matches!(
        verify(&source, directory.path(), &file),
        FreshUploadOutcome::HeldUnknown { .. }
    ));
}

#[test]
fn a_different_or_unknown_modifier_holds_an_apparent_upload_by_me() {
    let (directory, file) = fresh_file();
    for modifier in [
        json!({ "id": OTHER, "userPrincipalName": "other@example.test" }),
        json!({ "displayName": "Unknown" }),
    ] {
        let mut value = metadata();
        value["lastModifiedBy"]["user"] = modifier;
        let source = ScriptedMetadata::new([Ok((me(), value))]);
        assert!(matches!(
            verify(&source, directory.path(), &file),
            FreshUploadOutcome::HeldUnknown { .. }
        ));
    }
}

#[test]
fn a_conflicting_actor_tenant_is_never_overridden_by_a_matching_object_id() {
    let (directory, file) = fresh_file();
    for pointer in ["/createdBy/user/tenantId", "/lastModifiedBy/user/tenantId"] {
        let mut value = metadata();
        value["createdBy"]["user"]["tenantId"] = json!(TENANT);
        value["lastModifiedBy"]["user"]["tenantId"] = json!(TENANT);
        *value.pointer_mut(pointer).unwrap() = json!(OTHER);
        let source = ScriptedMetadata::new([Ok((me(), value))]);

        assert!(
            matches!(
                verify(&source, directory.path(), &file),
                FreshUploadOutcome::HeldUnknown { .. }
            ),
            "{pointer}"
        );
    }
}

#[test]
fn creation_and_modification_must_be_valid_and_identical() {
    let (directory, file) = fresh_file();
    for (pointer, replacement) in [
        ("/lastModifiedDateTime", json!("2026-09-14T16:01:00Z")),
        ("/createdDateTime", json!("not-a-time")),
        ("/lastModifiedDateTime", Value::Null),
    ] {
        let mut value = metadata();
        *value.pointer_mut(pointer).unwrap() = replacement;
        let source = ScriptedMetadata::new([Ok((me(), value))]);
        assert!(matches!(
            verify(&source, directory.path(), &file),
            FreshUploadOutcome::HeldUnknown { .. }
        ));
    }
}

#[test]
fn shortcuts_remote_items_conflicts_and_non_files_are_ambiguous() {
    let (directory, file) = fresh_file();
    for (field, replacement) in [
        ("remoteItem", json!({ "id": "remote" })),
        ("conflict", json!({ "behavior": "rename" })),
        ("folder", json!({ "childCount": 0 })),
        ("bundle", json!({ "childCount": 1 })),
        ("specialFolder", json!({ "name": "documents" })),
    ] {
        let mut value = metadata();
        value[field] = replacement;
        let source = ScriptedMetadata::new([Ok((me(), value))]);
        assert!(matches!(
            verify(&source, directory.path(), &file),
            FreshUploadOutcome::HeldUnknown { .. }
        ));
    }
}

#[test]
fn every_tenant_site_library_drive_and_folder_boundary_is_required() {
    let (directory, file) = fresh_file();
    for (pointer, replacement) in [
        ("/sharepointIds/tenantId", json!(OTHER)),
        ("/sharepointIds/siteId", json!(OTHER)),
        ("/sharepointIds/listId", json!(OTHER)),
        ("/parentReference/driveId", json!(OTHER)),
        ("/parentReference/id", json!(OTHER)),
        (
            "/webUrl",
            json!("https://teamcontoso.sharepoint.com/sites/OtherSite/Files/Inbox/agreement.pdf"),
        ),
        (
            "/webUrl",
            json!(
                "https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Filed/agreement.pdf"
            ),
        ),
    ] {
        let mut value = metadata();
        *value.pointer_mut(pointer).unwrap() = replacement;
        let source = ScriptedMetadata::new([Ok((me(), value))]);
        assert!(
            matches!(
                verify(&source, directory.path(), &file),
                FreshUploadOutcome::HeldUnknown { .. }
            ),
            "{pointer}"
        );
    }

    let nested = directory.path().join("subfolder");
    std::fs::create_dir(&nested).unwrap();
    let nested_file = nested.join("agreement.pdf");
    std::fs::write(&nested_file, b"hello").unwrap();
    let source = ScriptedMetadata::new([]);
    assert!(matches!(
        verify(&source, directory.path(), &nested_file),
        FreshUploadOutcome::HeldUnknown { .. }
    ));
    assert!(source.urls.lock().unwrap().is_empty());
}

#[test]
fn an_etag_or_other_revision_fact_changing_on_the_second_read_is_ambiguous() {
    let (directory, file) = fresh_file();
    for pointer in ["/eTag", "/cTag", "/size", "/file/hashes/quickXorHash"] {
        let first = metadata();
        let mut second = first.clone();
        *second.pointer_mut(pointer).unwrap() = match pointer {
            "/size" => json!(6),
            _ => json!("different"),
        };
        let source = ScriptedMetadata::new([Ok((me(), first)), Ok((me(), second))]);
        assert!(
            matches!(
                verify(&source, directory.path(), &file),
                FreshUploadOutcome::HeldUnknown { .. }
            ),
            "{pointer}"
        );
    }
}

#[test]
fn remote_size_and_quickxor_must_bind_the_local_file_before_the_second_read() {
    let (directory, file) = fresh_file();
    for (pointer, replacement) in [
        ("/size", json!(6)),
        (
            "/file/hashes/quickXorHash",
            json!("AAAAAAAAAAAAAAAAAAAAAAAAAAA="),
        ),
    ] {
        let mut value = metadata();
        *value.pointer_mut(pointer).unwrap() = replacement;
        let source = ScriptedMetadata::new([Ok((me(), value))]);
        assert!(
            matches!(
                verify(&source, directory.path(), &file),
                FreshUploadOutcome::HeldUnknown { .. }
            ),
            "{pointer}"
        );
        assert_eq!(source.urls.lock().unwrap().len(), 1);
    }
}

#[test]
fn transient_graph_failure_on_either_metadata_read_is_retryable_not_an_identity_verdict() {
    let (directory, file) = fresh_file();
    let source = ScriptedMetadata::new([Err("Microsoft requested a slower rate.".into())]);
    assert_eq!(
        verify(&source, directory.path(), &file),
        FreshUploadOutcome::RetryableUnavailable {
            reason: "Microsoft requested a slower rate.".into()
        }
    );

    let source = ScriptedMetadata::new([
        Ok((me(), metadata())),
        Err("Microsoft could not be reached.".into()),
    ]);
    assert_eq!(
        verify(&source, directory.path(), &file),
        FreshUploadOutcome::RetryableUnavailable {
            reason: "Microsoft could not be reached.".into()
        }
    );
}
