//! Provider metadata validation. Display names and email addresses never authorize a file.
use crate::{SharePointDeployment, microsoft::hashing::verified_local_snapshot, relative_to_root};
use intern_core::{OwnedFileSnapshot, PrivateSnapshotDirectory};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub tenant_id: String,
    pub id: String,
    pub display_name: String,
    pub email: String,
    /// Only the principal name returned by authenticated /me, never a typed alias.
    #[serde(default)]
    pub user_principal_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderBinding {
    pub local_folder: String,
    pub drive_id: String,
    pub folder_id: String,
    pub web_url: String,
    pub tenant_id: String,
    #[serde(default)]
    pub web_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation_watermark: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    pub item_id: String,
    pub etag: String,
    pub uploader: Account,
    pub quick_xor: String,
    pub size: u64,
    pub created_at: String,
    pub web_url: String,
    pub list_item_id: String,
}

/// Narrow dependency used by the proof so tests can script Graph metadata
/// without weakening the production HTTP boundary.
pub trait FreshUploadMetadata: Send + Sync {
    fn metadata(&self, url: Url) -> Result<(Account, Value), String>;
}

impl FreshUploadMetadata for super::auth::MicrosoftClient {
    fn metadata(&self, url: Url) -> Result<(Account, Value), String> {
        super::auth::MicrosoftClient::metadata(self, url)
    }
}

#[derive(Debug)]
pub enum FreshUploadOutcome {
    Authorized {
        local_sha256: String,
        uploader: Account,
        snapshot: OwnedFileSnapshot,
    },
    HeldOther {
        uploader: Account,
        reason: String,
    },
    HeldUnknown {
        reason: String,
    },
    RetryableUnavailable {
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FreshFacts {
    item_id: String,
    etag: String,
    content_tag: String,
    name: String,
    size: u64,
    quick_xor: String,
    created_at: i64,
    web_url: String,
    list_item_id: String,
    creator_id: ActorField,
    creator_tenant_id: ActorField,
    creator_principal: ActorField,
    modifier_id: ActorField,
    modifier_tenant_id: ActorField,
    modifier_principal: ActorField,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ActorField {
    Missing,
    Valid(String),
    Malformed,
}

enum Actor {
    Me,
    Other(Account),
    Unknown,
}

/// Establishes the only supported admission proof: a new, unchanged file
/// directly inside the provisioned Inbox, created and last modified by `/me`.
/// No local file bytes are read until the first metadata response establishes
/// the fixed boundary and eligible identity.
pub fn verify_fresh_upload(
    deployment: &SharePointDeployment,
    source: &dyn FreshUploadMetadata,
    activation_watermark: i64,
    snapshots: &PrivateSnapshotDirectory,
    local_inbox: &Path,
    local_file: &Path,
) -> FreshUploadOutcome {
    let Some(relative) = relative_to_root(local_file, local_inbox) else {
        return held_unknown("The local file is outside the verified Inbox.");
    };
    if relative.contains('/') {
        return held_unknown("Only files uploaded directly into the verified Inbox are supported.");
    }
    let filename = match local_file.file_name().and_then(|name| name.to_str()) {
        Some(filename)
            if filename == relative
                && !filename.to_ascii_lowercase().contains("conflicted copy") =>
        {
            filename
        }
        _ => return held_unknown("The local filename or conflict state is ambiguous."),
    };
    let url = match super::transport::item_url(
        deployment.drive_id(),
        deployment.intake_folder_id(),
        Some(&relative),
    ) {
        Ok(url) => url,
        Err(reason) => return held_unknown(reason),
    };

    let (account, first_value) = match source.metadata(url.clone()) {
        Ok(reply) => reply,
        Err(reason) => return FreshUploadOutcome::RetryableUnavailable { reason },
    };
    if !account_matches_deployment(&account, deployment) {
        return held_unknown("The connected Microsoft account is outside the provisioned tenant.");
    }
    let first = match fresh_facts(&first_value, deployment, filename) {
        Ok(facts) => facts,
        Err(reason) => return held_unknown(reason),
    };
    if first.created_at <= activation_watermark {
        return held_unknown("The Microsoft item was not created after this folder was paired.");
    }
    match actor(&first, &account, true) {
        Actor::Me => {}
        Actor::Other(uploader) => {
            return FreshUploadOutcome::HeldOther {
                uploader,
                reason: "The upload belongs to another Microsoft account.".into(),
            };
        }
        Actor::Unknown => {
            return held_unknown("Microsoft did not provide an unambiguous creator identity.");
        }
    }
    if !matches!(actor(&first, &account, false), Actor::Me) {
        return held_unknown("The latest modifier is not the connected Microsoft account.");
    }

    let (local_sha256, snapshot) =
        match verified_local_snapshot(local_file, first.size, &first.quick_xor, snapshots) {
            Ok(verified) => verified,
            Err(_) => {
                return held_unknown(
                    "The local file does not match the settled Microsoft size and checksum.",
                );
            }
        };

    let (current_account, second_value) = match source.metadata(url) {
        Ok(reply) => reply,
        Err(reason) => return FreshUploadOutcome::RetryableUnavailable { reason },
    };
    if !same_person(&account, &current_account)
        || !account_matches_deployment(&current_account, deployment)
    {
        return held_unknown("The connected Microsoft account changed during verification.");
    }
    let second = match fresh_facts(&second_value, deployment, filename) {
        Ok(facts) => facts,
        Err(reason) => return held_unknown(reason),
    };
    if first != second
        || !matches!(actor(&second, &current_account, true), Actor::Me)
        || !matches!(actor(&second, &current_account, false), Actor::Me)
    {
        return held_unknown("The Microsoft item or revision changed during verification.");
    }

    FreshUploadOutcome::Authorized {
        local_sha256,
        uploader: account,
        snapshot,
    }
}

fn held_unknown(reason: impl Into<String>) -> FreshUploadOutcome {
    FreshUploadOutcome::HeldUnknown {
        reason: reason.into(),
    }
}

fn account_matches_deployment(account: &Account, deployment: &SharePointDeployment) -> bool {
    is_guid(&account.id)
        && account
            .tenant_id
            .eq_ignore_ascii_case(deployment.tenant_id())
}

fn fresh_facts(
    metadata: &Value,
    deployment: &SharePointDeployment,
    filename: &str,
) -> Result<FreshFacts, &'static str> {
    for facet in [
        "deleted",
        "folder",
        "remoteItem",
        "pendingOperations",
        "malware",
        "package",
        "bundle",
        "specialFolder",
    ] {
        if metadata.get(facet).is_some_and(|value| !value.is_null()) {
            return Err("Microsoft reports a shortcut, conflict, non-file, or unsettled item.");
        }
    }
    if !metadata.get("file").is_some_and(Value::is_object)
        || text(metadata, "/file/mimeType").is_err()
    {
        return Err("Microsoft did not report an ordinary file facet.");
    }
    if text(metadata, "/name")? != filename {
        return Err("The Microsoft filename does not match the local file.");
    }
    for (pointer, expected) in [
        ("/sharepointIds/tenantId", deployment.tenant_id()),
        ("/sharepointIds/siteId", deployment.site_id()),
        ("/sharepointIds/webId", deployment.web_id()),
        ("/sharepointIds/listId", deployment.list_id()),
        ("/parentReference/driveId", deployment.drive_id()),
        ("/parentReference/id", deployment.intake_folder_id()),
    ] {
        if !text(metadata, pointer)?.eq_ignore_ascii_case(expected) {
            return Err(
                "The Microsoft item is outside the provisioned tenant, site, library, or Inbox.",
            );
        }
    }
    let web_url = text(metadata, "/webUrl")?;
    let parsed = Url::parse(web_url).map_err(|_| "The Microsoft item URL is invalid.")?;
    if !deployment.contains_intake_child_web_url(&parsed) {
        return Err("The Microsoft item URL is outside the provisioned Inbox.");
    }
    let created_at = text(metadata, "/createdDateTime")?;
    let modified_at = text(metadata, "/lastModifiedDateTime")?;
    if created_at != modified_at {
        return Err(
            "The Microsoft creation and modification facts do not show a new unchanged upload.",
        );
    }
    let created_at = timestamp_epoch_millis(created_at).ok_or(
        "The Microsoft creation and modification facts do not show a new unchanged upload.",
    )?;
    let size = metadata
        .get("size")
        .and_then(Value::as_u64)
        .filter(|size| *size > 0)
        .ok_or("Microsoft did not provide a settled positive file size.")?;
    let list_item_id = text(metadata, "/sharepointIds/listItemUniqueId")?;
    if !is_guid(list_item_id) {
        return Err("Microsoft did not provide an unambiguous SharePoint item identity.");
    }
    Ok(FreshFacts {
        item_id: text(metadata, "/id")?.to_owned(),
        etag: text(metadata, "/eTag")?.to_owned(),
        content_tag: text(metadata, "/cTag")?.to_owned(),
        name: filename.to_owned(),
        size,
        quick_xor: text(metadata, "/file/hashes/quickXorHash")?.to_owned(),
        created_at,
        web_url: web_url.to_owned(),
        list_item_id: list_item_id.to_owned(),
        creator_id: actor_guid_field(metadata, "/createdBy/user/id"),
        creator_tenant_id: actor_guid_field(metadata, "/createdBy/user/tenantId"),
        creator_principal: actor_field(metadata, "/createdBy/user/userPrincipalName"),
        modifier_id: actor_guid_field(metadata, "/lastModifiedBy/user/id"),
        modifier_tenant_id: actor_guid_field(metadata, "/lastModifiedBy/user/tenantId"),
        modifier_principal: actor_field(metadata, "/lastModifiedBy/user/userPrincipalName"),
    })
}

fn actor(facts: &FreshFacts, account: &Account, creator: bool) -> Actor {
    let (id, tenant_id, principal) = if creator {
        (
            &facts.creator_id,
            &facts.creator_tenant_id,
            &facts.creator_principal,
        )
    } else {
        (
            &facts.modifier_id,
            &facts.modifier_tenant_id,
            &facts.modifier_principal,
        )
    };
    match tenant_id {
        ActorField::Malformed => return Actor::Unknown,
        ActorField::Valid(tenant_id) if !tenant_id.eq_ignore_ascii_case(&account.tenant_id) => {
            return Actor::Unknown;
        }
        ActorField::Missing | ActorField::Valid(_) => {}
    }
    if let ActorField::Valid(id) = id {
        if id.eq_ignore_ascii_case(&account.id) {
            return Actor::Me;
        }
        let principal = match principal {
            ActorField::Valid(principal) => principal.clone(),
            ActorField::Missing | ActorField::Malformed => String::new(),
        };
        return Actor::Other(Account {
            tenant_id: account.tenant_id.clone(),
            id: id.to_ascii_lowercase(),
            display_name: String::new(),
            email: principal.clone(),
            user_principal_name: principal,
        });
    }
    match (id, principal) {
        (ActorField::Missing, ActorField::Valid(principal))
            if !account.user_principal_name.is_empty()
                && principal.eq_ignore_ascii_case(&account.user_principal_name) =>
        {
            Actor::Me
        }
        _ => Actor::Unknown,
    }
}

fn actor_guid_field(value: &Value, pointer: &str) -> ActorField {
    match actor_field(value, pointer) {
        ActorField::Valid(text) if is_guid(&text) => ActorField::Valid(text),
        ActorField::Valid(_) => ActorField::Malformed,
        other => other,
    }
}

fn actor_field(value: &Value, pointer: &str) -> ActorField {
    match value.pointer(pointer) {
        None => ActorField::Missing,
        Some(Value::String(text)) if !text.trim().is_empty() && text.len() <= 4096 => {
            ActorField::Valid(text.clone())
        }
        Some(_) => ActorField::Malformed,
    }
}

/// Legacy first-read parser retained for compatibility with inactive audit
/// code. It is not an admission proof; production admission uses
/// [`verify_fresh_upload`].
pub fn candidate(
    metadata: &Value,
    account: &Account,
    filename: &str,
) -> Result<Candidate, &'static str> {
    for facet in [
        "deleted",
        "folder",
        "remoteItem",
        "pendingOperations",
        "malware",
    ] {
        if metadata.get(facet).is_some_and(|value| !value.is_null()) {
            return Err("Microsoft reports an unresolved, moved/shared, or unavailable item.");
        }
    }
    let tenant = text(metadata, "/sharepointIds/tenantId")?;
    if !tenant.eq_ignore_ascii_case(&account.tenant_id) {
        return Err("The Microsoft item belongs to a different organization.");
    }
    if text(metadata, "/name")? != filename {
        return Err("The Microsoft filename does not match the local file.");
    }
    let creator = text(metadata, "/createdBy/user/id")?;
    let editor = text(metadata, "/lastModifiedBy/user/id")?;
    if !is_guid(creator) || !creator.eq_ignore_ascii_case(editor) {
        return Err("The creator and latest editor cannot establish one upload identity.");
    }
    let created_at = text(metadata, "/createdDateTime")?;
    let modified_at = text(metadata, "/lastModifiedDateTime")?;
    if created_at != modified_at || !valid_timestamp(created_at) {
        return Err(
            "This item changed after creation; the uploader needs independent verification.",
        );
    }
    let quick_xor = text(metadata, "/file/hashes/quickXorHash")?;
    let size = metadata
        .get("size")
        .and_then(Value::as_u64)
        .filter(|size| *size > 0)
        .ok_or("Microsoft has not provided a settled file size.")?;
    Ok(Candidate {
        item_id: text(metadata, "/id")?.to_owned(),
        etag: text(metadata, "/eTag")?.to_owned(),
        uploader: Account {
            tenant_id: tenant.to_ascii_lowercase(),
            id: creator.to_ascii_lowercase(),
            display_name: metadata
                .pointer("/createdBy/user/displayName")
                .and_then(Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or("Microsoft user")
                .to_owned(),
            user_principal_name: if creator.eq_ignore_ascii_case(&account.id) {
                account.user_principal_name.clone()
            } else {
                String::new()
            },
            email: if creator.eq_ignore_ascii_case(&account.id) {
                account.email.clone()
            } else {
                String::new()
            },
        },
        quick_xor: quick_xor.to_owned(),
        size,
        created_at: created_at.to_owned(),
        web_url: text(metadata, "/webUrl")?.to_owned(),
        list_item_id: text(metadata, "/sharepointIds/listItemUniqueId")?.to_owned(),
    })
}

pub fn same_person(left: &Account, right: &Account) -> bool {
    is_guid(&left.id)
        && is_guid(&right.id)
        && is_guid(&left.tenant_id)
        && left.id.eq_ignore_ascii_case(&right.id)
        && left.tenant_id.eq_ignore_ascii_case(&right.tenant_id)
}

pub(crate) fn text<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, &'static str> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty() && text.len() <= 4096)
        .ok_or("Microsoft has not provided complete upload identity metadata.")
}

pub fn is_guid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

fn valid_timestamp(value: &str) -> bool {
    timestamp_epoch_millis(value).is_some()
}

fn timestamp_epoch_millis(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn person() -> Account {
        Account {
            tenant_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
            id: "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb".into(),
            display_name: "Zachary Brenner".into(),
            email: "zack@example.test".into(),
            user_principal_name: "zack@example.test".into(),
        }
    }
    fn metadata() -> Value {
        json!({"id":"document-id","eTag":"revision-1","name":"agreement.pdf","size":12,"webUrl":"https://example.sharepoint.com/Legal/intake/agreement.pdf","sharepointIds":{"tenantId":person().tenant_id,"listItemUniqueId":"eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee"},"createdBy":{"user":{"id":person().id,"displayName":"Zachary Brenner"}},"lastModifiedBy":{"user":{"id":person().id}},"createdDateTime":"2026-09-08T12:00:00Z","lastModifiedDateTime":"2026-09-08T12:00:00Z","file":{"hashes":{"quickXorHash":"AAAAAAAAAAAAAAAAAAAAAAAAAAA="}}})
    }
    #[test]
    fn accepts_complete_provider_identity_for_an_unchanged_initial_item() {
        let proof = candidate(&metadata(), &person(), "agreement.pdf").unwrap();
        assert!(same_person(&proof.uploader, &person()));
    }
    #[test]
    fn missing_creator_is_unknown_not_the_last_editor() {
        let mut v = metadata();
        v.as_object_mut().unwrap().remove("createdBy");
        assert!(candidate(&v, &person(), "agreement.pdf").is_err());
    }
    #[test]
    fn names_and_emails_never_match_different_ids() {
        let mut other = person();
        other.id = "cccccccc-cccc-cccc-cccc-cccccccccccc".into();
        assert!(!same_person(&person(), &other));
    }
    #[test]
    fn a_matching_id_in_another_tenant_is_not_me() {
        let mut other = person();
        other.tenant_id = "dddddddd-dddd-dddd-dddd-dddddddddddd".into();
        assert!(!same_person(&person(), &other));
    }
    #[test]
    fn numeric_sharepoint_lookup_ids_are_not_directory_ids() {
        let mut v = metadata();
        v["createdBy"]["user"]["id"] = json!("17");
        v["lastModifiedBy"]["user"]["id"] = json!("17");
        assert!(candidate(&v, &person(), "agreement.pdf").is_err());
    }
    #[test]
    fn conflicting_editor_or_later_revision_is_held() {
        for pointer in ["/lastModifiedBy/user/id", "/lastModifiedDateTime"] {
            let mut v = metadata();
            *v.pointer_mut(pointer).unwrap() = json!("different");
            assert!(candidate(&v, &person(), "agreement.pdf").is_err());
        }
    }
    #[test]
    fn every_ambiguous_facet_is_held_even_when_empty() {
        for facet in [
            "deleted",
            "folder",
            "remoteItem",
            "pendingOperations",
            "malware",
        ] {
            let mut v = metadata();
            v[facet] = json!({});
            assert!(candidate(&v, &person(), "agreement.pdf").is_err());
        }
    }
    #[test]
    fn missing_or_blank_required_metadata_fails_closed() {
        for pointer in [
            "/id",
            "/eTag",
            "/createdBy/user/id",
            "/lastModifiedBy/user/id",
            "/file/hashes/quickXorHash",
            "/sharepointIds/tenantId",
            "/createdDateTime",
        ] {
            let mut v = metadata();
            *v.pointer_mut(pointer).unwrap() = json!("");
            assert!(
                candidate(&v, &person(), "agreement.pdf").is_err(),
                "{pointer}"
            );
        }
    }
    #[test]
    fn wrong_file_or_tenant_is_held() {
        assert!(candidate(&metadata(), &person(), "other.pdf").is_err());
        let mut v = metadata();
        v["sharepointIds"]["tenantId"] = json!("dddddddd-dddd-dddd-dddd-dddddddddddd");
        assert!(candidate(&v, &person(), "agreement.pdf").is_err());
    }
}
