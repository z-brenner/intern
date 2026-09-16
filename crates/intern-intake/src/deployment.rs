//! The public, versioned contract for Intern's single SharePoint deployment.

use std::{error::Error, fmt};

use serde::Deserialize;
use serde_json::Value;
use url::Url;

const SCHEMA_VERSION: u32 = 1;
const SITE_URL: &str = "https://teamcontoso.sharepoint.com/sites/InternTestSite";
const SITE_HOST: &str = "teamcontoso.sharepoint.com";
const SITE_PATH: &str = "/sites/InternTestSite";
const LIBRARY_NAME: &str = "Files";
const INTAKE_FOLDER_NAME: &str = "Inbox";
const DESTINATION_FOLDER_NAME: &str = "Filed";
const CONFIGURATION_UNAVAILABLE: &str = "provisioned identifiers are not available in this build.";

/// A validated, immutable description of the only SharePoint library Intern
/// may use in schema version 1.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharePointDeployment {
    site_url: Url,
    library_name: String,
    intake_folder_name: String,
    destination_folder_name: String,
    tenant_id: String,
    client_id: String,
    site_id: String,
    web_id: String,
    list_id: String,
    drive_id: String,
    intake_folder_id: String,
    destination_folder_id: String,
}

impl SharePointDeployment {
    /// Parses and validates a bundled deployment resource.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DeploymentError> {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|error| DeploymentError::invalid(format!("invalid JSON: {error}")))?;
        if contains_credential_field(&value) {
            return Err(DeploymentError::invalid(
                "secret, password, or token fields are not permitted",
            ));
        }
        let raw: RawDeployment = serde_json::from_value(value)
            .map_err(|error| DeploymentError::invalid(error.to_string()))?;

        if raw.schema_version != SCHEMA_VERSION {
            return Err(DeploymentError::invalid(format!(
                "unsupported schema_version {}; expected {SCHEMA_VERSION}",
                raw.schema_version
            )));
        }
        let site_url = parse_fixed_site_url(&raw.site_url)?;
        validate_component("library_name", &raw.library_name, LIBRARY_NAME)?;
        validate_component(
            "intake_folder_name",
            &raw.intake_folder_name,
            INTAKE_FOLDER_NAME,
        )?;
        validate_component(
            "destination_folder_name",
            &raw.destination_folder_name,
            DESTINATION_FOLDER_NAME,
        )?;
        if !raw.enabled {
            return Err(DeploymentError::ConfigurationUnavailable);
        }

        let deployment = Self {
            site_url,
            library_name: raw.library_name,
            intake_folder_name: raw.intake_folder_name,
            destination_folder_name: raw.destination_folder_name,
            tenant_id: required_id("tenant_id", raw.tenant_id, IdKind::Guid)?,
            client_id: required_id("client_id", raw.client_id, IdKind::Guid)?,
            site_id: required_id("site_id", raw.site_id, IdKind::Guid)?,
            web_id: required_id("web_id", raw.web_id, IdKind::Guid)?,
            list_id: required_id("list_id", raw.list_id, IdKind::Guid)?,
            drive_id: required_id("drive_id", raw.drive_id, IdKind::Drive)?,
            intake_folder_id: required_id(
                "intake_folder_id",
                raw.intake_folder_id,
                IdKind::DriveItem,
            )?,
            destination_folder_id: required_id(
                "destination_folder_id",
                raw.destination_folder_id,
                IdKind::DriveItem,
            )?,
        };
        deployment.validate()?;
        Ok(deployment)
    }

    /// Checks that every field remains within the fixed schema-1 boundary.
    pub fn validate(&self) -> Result<(), DeploymentError> {
        if self.site_url.as_str() != SITE_URL {
            return Err(DeploymentError::invalid(
                "site_url must be the fixed HTTPS site",
            ));
        }
        validate_component("library_name", &self.library_name, LIBRARY_NAME)?;
        validate_component(
            "intake_folder_name",
            &self.intake_folder_name,
            INTAKE_FOLDER_NAME,
        )?;
        validate_component(
            "destination_folder_name",
            &self.destination_folder_name,
            DESTINATION_FOLDER_NAME,
        )?;
        for (name, value, kind) in [
            ("tenant_id", &self.tenant_id, IdKind::Guid),
            ("client_id", &self.client_id, IdKind::Guid),
            ("site_id", &self.site_id, IdKind::Guid),
            ("web_id", &self.web_id, IdKind::Guid),
            ("list_id", &self.list_id, IdKind::Guid),
            ("drive_id", &self.drive_id, IdKind::Drive),
            (
                "intake_folder_id",
                &self.intake_folder_id,
                IdKind::DriveItem,
            ),
            (
                "destination_folder_id",
                &self.destination_folder_id,
                IdKind::DriveItem,
            ),
        ] {
            kind.validate(name, value)?;
        }
        Ok(())
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn site_id(&self) -> &str {
        &self.site_id
    }

    pub fn web_id(&self) -> &str {
        &self.web_id
    }

    pub fn list_id(&self) -> &str {
        &self.list_id
    }

    pub fn drive_id(&self) -> &str {
        &self.drive_id
    }

    pub fn library_name(&self) -> &str {
        &self.library_name
    }

    pub fn intake_folder_name(&self) -> &str {
        &self.intake_folder_name
    }

    pub fn destination_folder_name(&self) -> &str {
        &self.destination_folder_name
    }

    pub fn intake_folder_web_url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.site_url, self.library_name, self.intake_folder_name
        )
    }

    pub fn intake_folder_id(&self) -> &str {
        &self.intake_folder_id
    }

    /// Returns whether a returned SharePoint web URL stays inside the
    /// configured site. The comparison is origin- and path-boundary-aware.
    pub fn contains_web_url(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.host_str() == Some(SITE_HOST)
            && url.port().is_none()
            && url.username().is_empty()
            && url.password().is_none()
            && (url.path() == SITE_PATH
                || url
                    .path()
                    .strip_prefix(SITE_PATH)
                    .is_some_and(|suffix| suffix.starts_with('/')))
    }

    pub fn is_intake_folder_web_url(&self, value: &str) -> bool {
        Url::parse(value).is_ok_and(|url| {
            self.contains_web_url(&url)
                && url.query().is_none()
                && url.fragment().is_none()
                && url.path() == format!("{SITE_PATH}/{LIBRARY_NAME}/{INTAKE_FOLDER_NAME}")
        })
    }

    pub fn contains_intake_child_web_url(&self, url: &Url) -> bool {
        let prefix = format!("{SITE_PATH}/{LIBRARY_NAME}/{INTAKE_FOLDER_NAME}/");
        self.contains_web_url(url)
            && url.query().is_none()
            && url.fragment().is_none()
            && url
                .path()
                .strip_prefix(&prefix)
                .is_some_and(|child| !child.is_empty() && !child.contains('/'))
    }

    /// Builds the supported OneDrive library-sync request for a verified
    /// account email. Query values use URL form encoding rather than manual
    /// string interpolation.
    pub fn odopen_url(&self, email: &str) -> Result<Url, DeploymentError> {
        if !is_valid_email(email) {
            return Err(DeploymentError::invalid(
                "a valid connected-account email is required for OneDrive sync",
            ));
        }
        let mut url = Url::parse("odopen://sync/")
            .map_err(|error| DeploymentError::invalid(format!("odopen URL: {error}")))?;
        let site_id = format!("{{{}}}", self.site_id);
        let web_id = format!("{{{}}}", self.web_id);
        let list_id = format!("{{{}}}", self.list_id);
        url.query_pairs_mut()
            .append_pair("siteId", &site_id)
            .append_pair("webId", &web_id)
            .append_pair("listId", &list_id)
            .append_pair("userEmail", email)
            .append_pair("webUrl", self.site_url.as_str())
            .append_pair("listTitle", &self.library_name);
        Ok(url)
    }
}

/// Errors safe to surface in onboarding and support diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeploymentError {
    ConfigurationUnavailable,
    InvalidConfiguration(String),
}

impl DeploymentError {
    fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidConfiguration(message.into())
    }
}

impl fmt::Display for DeploymentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigurationUnavailable => write!(
                formatter,
                "SharePoint deployment configuration is unavailable: {CONFIGURATION_UNAVAILABLE}"
            ),
            Self::InvalidConfiguration(message) => {
                write!(
                    formatter,
                    "invalid SharePoint deployment configuration: {message}"
                )
            }
        }
    }
}

impl Error for DeploymentError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDeployment {
    schema_version: u32,
    enabled: bool,
    site_url: String,
    library_name: String,
    intake_folder_name: String,
    destination_folder_name: String,
    tenant_id: Option<String>,
    client_id: Option<String>,
    site_id: Option<String>,
    web_id: Option<String>,
    list_id: Option<String>,
    drive_id: Option<String>,
    intake_folder_id: Option<String>,
    destination_folder_id: Option<String>,
}

fn parse_fixed_site_url(value: &str) -> Result<Url, DeploymentError> {
    if value != SITE_URL {
        return Err(DeploymentError::invalid(
            "site_url must be the fixed HTTPS site",
        ));
    }
    let url = Url::parse(value)
        .map_err(|error| DeploymentError::invalid(format!("invalid site_url: {error}")))?;
    if url.scheme() != "https"
        || url.host_str() != Some(SITE_HOST)
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != SITE_PATH
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(DeploymentError::invalid(
            "site_url must be the fixed HTTPS site",
        ));
    }
    Ok(url)
}

/// The public identifier shapes a schema-1 deployment may carry. Every shape
/// is a bounded ASCII alphabet with no URL delimiters (`/ \ ? # % & :`),
/// whitespace, or `.`, so a validated value can never leave a single Graph
/// path segment or alter a query.
#[derive(Clone, Copy)]
enum IdKind {
    /// Entra tenant/application and SharePoint site/web/list GUIDs.
    Guid,
    /// A SharePoint document-library drive ID as Microsoft Graph returns it:
    /// `b!` followed by the unpadded base64url encoding of the 48-byte
    /// site, web, and list GUIDs (64 characters). Base64url is
    /// case-sensitive, so consumers must compare it exactly.
    Drive,
    /// A SharePoint driveItem ID as Microsoft Graph returns it: `01`
    /// followed by 32 upper-case RFC 4648 base32 characters.
    DriveItem,
}

impl IdKind {
    fn validate(self, name: &str, value: &str) -> Result<(), DeploymentError> {
        let (valid, shape) = match self {
            Self::Guid => (is_guid(value), "a GUID-shaped public identifier"),
            Self::Drive => (
                is_drive_id(value),
                "a SharePoint drive ID (`b!` and 64 base64url characters)",
            ),
            Self::DriveItem => (
                is_drive_item_id(value),
                "a SharePoint driveItem ID (`01` and 32 upper-case base32 characters)",
            ),
        };
        if valid {
            Ok(())
        } else {
            Err(DeploymentError::invalid(format!("{name} must be {shape}")))
        }
    }
}

fn required_id(name: &str, value: Option<String>, kind: IdKind) -> Result<String, DeploymentError> {
    let value = value.ok_or_else(|| DeploymentError::invalid(format!("missing {name}")))?;
    kind.validate(name, &value)?;
    Ok(value)
}

fn validate_component(name: &str, value: &str, expected: &str) -> Result<(), DeploymentError> {
    let simple = !value.is_empty()
        && value.trim() == value
        && value != "."
        && value != ".."
        && !value.contains(['/', '\\'])
        && !value.chars().any(char::is_control);
    if !simple || value != expected {
        return Err(DeploymentError::invalid(format!(
            "{name} must be the simple fixed component {expected:?}"
        )));
    }
    Ok(())
}

fn contains_credential_field(value: &Value) -> bool {
    match value {
        Value::Object(object) => object.iter().any(|(name, value)| {
            let name = name.to_ascii_lowercase();
            name.contains("secret")
                || name.contains("password")
                || name.contains("token")
                || contains_credential_field(value)
        }),
        Value::Array(values) => values.iter().any(contains_credential_field),
        _ => false,
    }
}

fn is_guid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            matches!(index, 8 | 13 | 18 | 23) && byte == b'-'
                || !matches!(index, 8 | 13 | 18 | 23) && byte.is_ascii_hexdigit()
        })
}

fn is_drive_id(value: &str) -> bool {
    value.strip_prefix("b!").is_some_and(|encoded| {
        encoded.len() == 64
            && encoded
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    })
}

fn is_drive_item_id(value: &str) -> bool {
    value.strip_prefix("01").is_some_and(|encoded| {
        encoded.len() == 32
            && encoded
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || matches!(byte, b'2'..=b'7'))
    })
}

fn is_valid_email(value: &str) -> bool {
    if value.is_empty() || value.trim() != value || !value.is_ascii() {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.contains('.')
        && !domain.contains('@')
        && !value.chars().any(char::is_whitespace)
}
