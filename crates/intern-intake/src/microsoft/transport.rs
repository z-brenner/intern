use crate::SharePointDeployment;
use reqwest::{Url, blocking::Client, redirect::Policy};
use serde_json::Value;
use std::{io::Read, time::Duration};

const PROFILE_SELECT: &str = "id,displayName,mail,userPrincipalName";
const ITEM_SELECT: &str = "id,name,size,eTag,cTag,createdBy,lastModifiedBy,createdDateTime,lastModifiedDateTime,file,folder,deleted,pendingOperations,remoteItem,malware,package,bundle,specialFolder,sharepointIds,webUrl,parentReference";

pub struct Reply {
    pub status: u16,
    pub body: Value,
    pub retry_after: u64,
}

pub trait Transport: Send + Sync {
    fn request(
        &self,
        url: Url,
        form: Option<&[(&str, &str)]>,
        bearer: Option<&str>,
    ) -> Result<Reply, String>;
}

pub struct MicrosoftTransport {
    client: Client,
    deployment: SharePointDeployment,
}
impl MicrosoftTransport {
    pub fn new(deployment: &SharePointDeployment) -> Result<Self, String> {
        deployment
            .validate()
            .map_err(|_| "Microsoft deployment boundary is invalid.")?;
        Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map(|client| Self {
                client,
                deployment: deployment.clone(),
            })
            .map_err(|_| "Microsoft connection could not be initialized.".into())
    }
}

impl Transport for MicrosoftTransport {
    fn request(
        &self,
        url: Url,
        form: Option<&[(&str, &str)]>,
        bearer: Option<&str>,
    ) -> Result<Reply, String> {
        if !deployment_allows_endpoint(&self.deployment, &url, form.is_some(), bearer.is_some()) {
            return Err("Microsoft endpoint is not allowed.".into());
        }
        let request = if let Some(form) = form {
            self.client.post(url).form(form)
        } else {
            self.client.get(url)
        };
        let request = if let Some(token) = bearer {
            request.bearer_auth(token)
        } else {
            request
        };
        let response = request
            .send()
            .map_err(|_| "Microsoft could not be reached. Files remain held.".to_string())?;
        read_response(response)
    }
}

fn read_response(response: reqwest::blocking::Response) -> Result<Reply, String> {
    let status = response.status().as_u16();
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60)
        .clamp(5, 900);
    // Profiles, individual item metadata and OAuth responses are small.
    // Never follow a response into a file download or buffer an arbitrary body.
    const MAX_BODY: usize = 256 * 1024;
    let mut bytes = Vec::new();
    response
        .take((MAX_BODY + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "Microsoft returned an unreadable response.".to_string())?;
    if bytes.len() > MAX_BODY {
        return Err("Microsoft metadata response is too large.".into());
    }
    let body = serde_json::from_slice(&bytes)
        .map_err(|_| "Microsoft returned invalid metadata.".to_string())?;
    Ok(Reply {
        status,
        body,
        retry_after,
    })
}

pub fn deployment_allows_endpoint(
    deployment: &SharePointDeployment,
    url: &Url,
    form: bool,
    bearer: bool,
) -> bool {
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return false;
    }
    let parts: Vec<_> = url.path_segments().into_iter().flatten().collect();
    match (url.host_str(), form, bearer) {
        (Some("login.microsoftonline.com"), true, false) => {
            parts.len() == 4
                && parts[0].eq_ignore_ascii_case(deployment.tenant_id())
                && parts[1] == "oauth2"
                && parts[2] == "v2.0"
                && ["devicecode", "token"].contains(&parts[3])
                && url.query().is_none()
        }
        (Some("graph.microsoft.com"), false, true) => {
            if parts == ["v1.0", "me"] {
                return exact_select(url, PROFILE_SELECT);
            }
            // Drive IDs are case-sensitive base64url. DriveItem IDs are opaque;
            // the deployment holds the canonical form Graph returns, so the
            // item is also matched exactly and any other spelling fails closed.
            if parts.len() >= 5
                && parts[0] == "v1.0"
                && parts[1] == "drives"
                && parts[2] == deployment.drive_id()
                && parts[3] == "items"
                && exact_select(url, ITEM_SELECT)
            {
                if parts.len() == 5 {
                    return parts[4] == deployment.intake_folder_id();
                }
                return parts.len() == 6
                    && parts[4]
                        .strip_suffix(':')
                        .is_some_and(|folder| folder == deployment.intake_folder_id())
                    && direct_child_segment(parts[5]);
            }
            false
        }
        _ => false,
    }
}

fn direct_child_segment(value: &str) -> bool {
    if value.is_empty() || value.contains(['/', '\\', ':', '\0']) {
        return false;
    }
    let folded = value.to_ascii_lowercase();
    let decoded_dots = folded.replace("%2e", ".");
    decoded_dots != "."
        && decoded_dots != ".."
        && !["%00", "%2f", "%3a", "%5c"]
            .iter()
            .any(|encoded| folded.contains(encoded))
}

fn exact_select(url: &Url, expected: &str) -> bool {
    let pairs: Vec<_> = url.query_pairs().collect();
    pairs.len() == 1 && pairs[0].0 == "$select" && pairs[0].1 == expected
}

pub fn item_url(drive: &str, folder: &str, relative: Option<&str>) -> Result<Url, String> {
    fn identifier(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 256
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"!_-".contains(&byte))
    }
    if !identifier(drive) || !identifier(folder) {
        return Err("Microsoft drive or folder ID is invalid.".into());
    }
    let mut url = Url::parse("https://graph.microsoft.com/v1.0/")
        .map_err(|_| "Microsoft URL is unavailable.")?;
    {
        let mut parts = url
            .path_segments_mut()
            .map_err(|_| "Microsoft URL is unavailable.")?;
        parts
            .pop_if_empty()
            .push("drives")
            .push(drive)
            .push("items");
        if let Some(relative) = relative {
            if relative.is_empty()
                || relative.len() > 32768
                || relative == "."
                || relative == ".."
                || relative.contains(['/', '\\', '\0', ':'])
            {
                return Err("Intake relative path is invalid.".into());
            }
            parts.push(&format!("{folder}:"));
            parts.push(relative);
        } else {
            parts.push(folder);
        }
    }
    url.query_pairs_mut().append_pair("$select", ITEM_SELECT);
    Ok(url)
}

/// Only the global work/school SharePoint service is supported in this release.
pub fn sharepoint_url(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url
                .host_str()
                .is_some_and(|host| host.ends_with(".sharepoint.com"))
            && url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none()
            && url.port().is_none()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SharePointDeployment;

    // Synthetic identifiers shaped like real Microsoft Graph values; they do
    // not name any real tenant, drive, or item.
    const TENANT: &str = "11111111-1111-1111-1111-111111111111";
    const DRIVE: &str = "b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO";
    const INBOX: &str = "01SYNTHETICINBOXFOLDERAAAAAAAAAAAA";

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
                  "client_id": "22222222-2222-2222-2222-222222222222",
                  "site_id": "33333333-3333-3333-3333-333333333333",
                  "web_id": "44444444-4444-4444-4444-444444444444",
                  "list_id": "55555555-5555-5555-5555-555555555555",
                  "drive_id": "{DRIVE}",
                  "intake_folder_id": "{INBOX}",
                  "destination_folder_id": "01SYNTHETICFILEDFOLDERAAAAAAAAAAAA"
                }}"#
            )
            .as_bytes(),
        )
        .unwrap()
    }

    #[test]
    fn deployment_boundary_allows_only_its_oauth_profile_binding_and_direct_child_metadata() {
        let deployment = deployment();
        for address in [
            format!("https://login.microsoftonline.com/{TENANT}/oauth2/v2.0/devicecode"),
            format!("https://login.microsoftonline.com/{TENANT}/oauth2/v2.0/token"),
        ] {
            assert!(deployment_allows_endpoint(
                &deployment,
                &Url::parse(&address).unwrap(),
                true,
                false,
            ));
        }
        let profile = Url::parse(
            "https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName",
        )
        .unwrap();
        assert!(deployment_allows_endpoint(
            &deployment,
            &profile,
            false,
            true,
        ));
        assert!(deployment_allows_endpoint(
            &deployment,
            &item_url(DRIVE, INBOX, None).unwrap(),
            false,
            true,
        ));
        assert!(deployment_allows_endpoint(
            &deployment,
            &item_url(DRIVE, INBOX, Some("agreement.pdf")).unwrap(),
            false,
            true,
        ));
    }

    #[test]
    fn deployment_boundary_rejects_other_tenants_drives_items_and_nested_children() {
        let deployment = deployment();
        let selected = format!("?$select={ITEM_SELECT}");
        for address in [
            "https://login.microsoftonline.com/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa/oauth2/v2.0/devicecode".to_owned(),
            "https://login.microsoftonline.com/11111111-1111-1111-1111-111111111111.evil.example/oauth2/v2.0/token".to_owned(),
            format!("https://graph.microsoft.com/v1.0/drives/b!OTHERDSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO/items/{INBOX}:/agreement.pdf{selected}"),
            format!("https://graph.microsoft.com/v1.0/drives/{DRIVE}/items/01SYNTHETICOTHERFOLDERAAAAAAAAAAA{selected}"),
            format!("https://graph.microsoft.com/v1.0/drives/{DRIVE}/items/{INBOX}:/nested/agreement.pdf{selected}"),
            format!("https://graph.microsoft.com/v1.0/drives/{DRIVE}/items/{INBOX}:/nested%2Fagreement.pdf{selected}"),
        ] {
            assert!(
                !deployment_allows_endpoint(
                    &deployment,
                    &Url::parse(&address).unwrap(),
                    address.contains("login.microsoftonline.com"),
                    address.contains("graph.microsoft.com"),
                ),
                "{address}"
            );
        }
    }

    #[test]
    fn deployment_boundary_rejects_query_pagination_content_audit_and_arbitrary_graph() {
        let deployment = deployment();
        for address in [
            "https://graph.microsoft.com/v1.0/security/auditLog/queries/11111111-1111-1111-1111-111111111111",
            "https://graph.microsoft.com/v1.0/security/auditLog/queries/11111111-1111-1111-1111-111111111111/records",
            "https://graph.microsoft.com/v1.0/users",
            "https://graph.microsoft.com/v1.0/drives/b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO/items/01SYNTHETICINBOXFOLDERAAAAAAAAAAAA/content",
            "https://graph.microsoft.com/v1.0/drives/b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO/items/01SYNTHETICINBOXFOLDERAAAAAAAAAAAA:/agreement.pdf:/content?$select=id",
            "https://graph.microsoft.com/v1.0/drives/b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO/items/01SYNTHETICINBOXFOLDERAAAAAAAAAAAA:/agreement.pdf?$select=id&$top=1",
            "https://graph.microsoft.com/v1.0/drives/b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO/items/01SYNTHETICINBOXFOLDERAAAAAAAAAAAA:/agreement.pdf?$select=id&$skiptoken=secret",
            "https://graph.microsoft.com/v1.0/me?$select=id&$select=displayName",
        ] {
            assert!(
                !deployment_allows_endpoint(
                    &deployment,
                    &Url::parse(address).unwrap(),
                    false,
                    true,
                ),
                "{address} must never be available through the metadata transport"
            );
        }
    }

    #[test]
    fn metadata_allowlist_requires_the_exact_bounded_select_query() {
        let deployment = deployment();
        let profile = Url::parse(
            "https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName",
        )
        .unwrap();
        assert!(deployment_allows_endpoint(
            &deployment,
            &profile,
            false,
            true,
        ));
        assert!(!deployment_allows_endpoint(
            &deployment,
            &Url::parse("https://graph.microsoft.com/v1.0/me?$select=id,aboutMe").unwrap(),
            false,
            true,
        ));

        let item = item_url(DRIVE, INBOX, Some("agreement.pdf")).unwrap();
        assert!(deployment_allows_endpoint(&deployment, &item, false, true,));
        let selected = item
            .query_pairs()
            .find_map(|(name, value)| (name == "$select").then(|| value.into_owned()))
            .unwrap();
        assert_eq!(
            selected,
            "id,name,size,eTag,cTag,createdBy,lastModifiedBy,createdDateTime,lastModifiedDateTime,file,folder,deleted,pendingOperations,remoteItem,malware,package,bundle,specialFolder,sharepointIds,webUrl,parentReference"
        );
        for required in [
            "id",
            "eTag",
            "cTag",
            "file",
            "folder",
            "package",
            "bundle",
            "remoteItem",
            "pendingOperations",
            "malware",
            "deleted",
            "specialFolder",
            "sharepointIds",
            "parentReference",
            "webUrl",
            "name",
            "size",
            "createdBy",
            "lastModifiedBy",
            "createdDateTime",
            "lastModifiedDateTime",
        ] {
            assert!(
                selected.split(',').any(|field| field == required),
                "{required}"
            );
        }
        let arbitrary = Url::parse(
            "https://graph.microsoft.com/v1.0/drives/drive/items/item?$expand=permissions",
        )
        .unwrap();
        assert!(!deployment_allows_endpoint(
            &deployment,
            &arbitrary,
            false,
            true,
        ));
    }

    #[test]
    fn a_base64url_drive_id_reaches_graph_unencoded_and_matches_only_exactly() {
        let deployment = deployment();
        let item = item_url(DRIVE, INBOX, Some("agreement.pdf")).unwrap();
        assert!(
            item.as_str().starts_with(&format!(
                "https://graph.microsoft.com/v1.0/drives/{DRIVE}/items/{INBOX}:/agreement.pdf?"
            )),
            "{item}"
        );
        assert!(deployment_allows_endpoint(&deployment, &item, false, true));

        let selected = format!("?$select={ITEM_SELECT}");
        let percent_encoded = DRIVE.replacen('!', "%21", 1);
        // Base64url drive IDs are case-sensitive. Graph driveItem IDs are
        // opaque, and Graph returns them upper-case; the deployment stores that
        // canonical form, so any other spelling is treated as a different item.
        for (drive, folder) in [
            (DRIVE.to_ascii_lowercase(), INBOX.to_owned()),
            (DRIVE.to_ascii_uppercase(), INBOX.to_owned()),
            (percent_encoded, INBOX.to_owned()),
            (DRIVE.to_owned(), INBOX.to_ascii_lowercase()),
        ] {
            for address in [
                format!("https://graph.microsoft.com/v1.0/drives/{drive}/items/{folder}{selected}"),
                format!(
                    "https://graph.microsoft.com/v1.0/drives/{drive}/items/{folder}:/agreement.pdf{selected}"
                ),
            ] {
                assert!(
                    !deployment_allows_endpoint(
                        &deployment,
                        &Url::parse(&address).unwrap(),
                        false,
                        true,
                    ),
                    "{address}"
                );
            }
        }
    }

    #[test]
    fn filenames_cannot_escape_the_graph_item_path() {
        let url = item_url("drive!1", "folder-1", Some("100% # résumé?.pdf")).unwrap();
        assert_eq!(url.host_str(), Some("graph.microsoft.com"));
        assert!(
            url.as_str()
                .contains("100%25%20%23%20r%C3%A9sum%C3%A9%3F.pdf")
        );
        assert!(url.fragment().is_none());
        for path in [
            "../secrets",
            "/absolute",
            "Legal/agreement.pdf",
            "a//b",
            "a/./b",
            "a\\b",
            "https://evil.test/file",
        ] {
            assert!(item_url("drive", "folder", Some(path)).is_err());
        }
        for id in ["../me", "drive?token=x", "bad/id", ""] {
            assert!(item_url(id, "folder", None).is_err());
        }
    }
}
