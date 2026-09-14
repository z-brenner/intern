use reqwest::{Url, blocking::Client, redirect::Policy};
use serde_json::Value;
use std::{io::Read, time::Duration};

const PROFILE_SELECT: &str = "id,displayName,mail,userPrincipalName";
const ITEM_SELECT: &str = "id,name,size,eTag,cTag,createdBy,lastModifiedBy,createdDateTime,lastModifiedDateTime,file,fileSystemInfo,folder,deleted,pendingOperations,remoteItem,malware,package,bundle,specialFolder,sharepointIds,webUrl,parentReference";

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
}
impl MicrosoftTransport {
    pub fn new() -> Result<Self, String> {
        Client::builder()
            .redirect(Policy::none())
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map(|client| Self { client })
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
        if !allowed_endpoint(&url, form.is_some(), bearer.is_some()) {
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

pub fn allowed_endpoint(url: &Url, form: bool, bearer: bool) -> bool {
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
                && super::proof::is_guid(parts[0])
                && parts[1] == "oauth2"
                && parts[2] == "v2.0"
                && ["devicecode", "token"].contains(&parts[3])
                && url.query().is_none()
        }
        (Some("graph.microsoft.com"), false, true) => {
            if parts == ["v1.0", "me"] {
                return exact_select(url, PROFILE_SELECT);
            }
            if parts.len() >= 5 && parts[0] == "v1.0" && parts[1] == "drives" && parts[3] == "items"
            {
                // By-ID metadata or a path beneath a fixed parent ID. No /content,
                // preview, versions, permissions, or upload endpoint.
                return (parts.len() == 5
                    || parts[4].ends_with(':')
                        && !parts[5..].iter().any(|part| part.ends_with(':')))
                    && exact_select(url, ITEM_SELECT);
            }
            false
        }
        _ => false,
    }
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
            if relative.is_empty() || relative.len() > 32768 {
                return Err("Intake relative path is invalid.".into());
            }
            parts.push(&format!("{folder}:"));
            for component in relative.split('/') {
                if component.is_empty()
                    || component == "."
                    || component == ".."
                    || component.contains(['\\', '\0', ':'])
                {
                    return Err("Intake relative path is invalid.".into());
                }
                parts.push(component);
            }
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
    #[test]
    fn authenticated_allowlist_excludes_audit_and_content_endpoints() {
        for address in [
            "https://graph.microsoft.com/v1.0/security/auditLog/queries/11111111-1111-1111-1111-111111111111",
            "https://graph.microsoft.com/v1.0/security/auditLog/queries/11111111-1111-1111-1111-111111111111/records",
            "https://graph.microsoft.com/v1.0/drives/drive/items/item/content",
            "https://graph.microsoft.com/v1.0/drives/drive/items/folder:/agreement.pdf:/content?$select=id",
        ] {
            assert!(
                !allowed_endpoint(&Url::parse(address).unwrap(), false, true),
                "{address} must never be available through the metadata transport"
            );
        }
    }

    #[test]
    fn metadata_allowlist_requires_the_exact_bounded_select_query() {
        let profile = Url::parse(
            "https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName",
        )
        .unwrap();
        assert!(allowed_endpoint(&profile, false, true));
        assert!(!allowed_endpoint(
            &Url::parse("https://graph.microsoft.com/v1.0/me?$select=id,aboutMe").unwrap(),
            false,
            true,
        ));

        let item = item_url("drive", "inbox", Some("agreement.pdf")).unwrap();
        assert!(allowed_endpoint(&item, false, true));
        let selected = item
            .query_pairs()
            .find_map(|(name, value)| (name == "$select").then(|| value.into_owned()))
            .unwrap();
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
            "sharepointIds",
            "parentReference",
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
        assert!(!allowed_endpoint(&arbitrary, false, true));
    }

    #[test]
    fn filenames_cannot_escape_the_graph_item_path() {
        let url = item_url("drive!1", "folder-1", Some("Legal/100% # résumé?.pdf")).unwrap();
        assert_eq!(url.host_str(), Some("graph.microsoft.com"));
        assert!(
            url.as_str()
                .contains("100%25%20%23%20r%C3%A9sum%C3%A9%3F.pdf")
        );
        assert!(url.fragment().is_none());
        for path in [
            "../secrets",
            "/absolute",
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
