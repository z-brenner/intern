//! Delegated device authorization. Access tokens are opaque; /me establishes
//! the person. No decoded-but-unverified JWT claims, secret in settings, or
//! arbitrary OAuth endpoint is accepted.
use super::{
    proof::{Account, is_guid, text},
    transport::{MicrosoftTransport, Transport, deployment_allows_endpoint},
};
use crate::{
    SharePointDeployment,
    coordination::{Clock, SystemClock},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub const SCOPES: &str =
    "https://graph.microsoft.com/User.Read https://graph.microsoft.com/Files.Read offline_access";

/// Shown when the organization's consent policy stops sign-in. The UI detects
/// the trailing token, so both the opening sentence and the token are stable.
const CONSENT_BLOCKED: &str = "Your Microsoft organization has blocked Intern from connecting. Ask your Microsoft administrator to approve Intern for your organization, then connect again. Files remain held. (MICROSOFT_CONSENT_BLOCKED)";

/// Microsoft Entra ID sign-in errors that mean consent, not the person or the
/// network, is what stopped the connection (Microsoft Learn, "Microsoft Entra
/// authentication and authorization error codes" and "Unexpected error when
/// performing consent to an application"):
/// - 65001 DelegationDoesNotExist: nobody has consented to the application.
/// - 90093 the user is not authorized to grant the requested permissions.
/// - 90094 AdminConsentRequired.
/// - 90095 AdminConsentRequiredRequestAccess (admin consent workflow).
/// - 900941 AdminConsentRequiredDueToRiskyApp.
const CONSENT_BLOCKED_CODES: [u64; 5] = [65001, 90093, 90094, 90095, 900941];

/// Whether an OAuth error body reports a consent block, from the numeric
/// `error_codes` list or the `AADSTS<code>:` prefix of the description. The
/// description is only inspected, never echoed.
fn consent_blocked(body: &Value) -> bool {
    let listed = body["error_codes"].as_array().is_some_and(|codes| {
        codes
            .iter()
            .filter_map(Value::as_u64)
            .any(|code| CONSENT_BLOCKED_CODES.contains(&code))
    });
    let described = body["error_description"]
        .as_str()
        .and_then(|description| description.strip_prefix("AADSTS"))
        .and_then(|rest| rest.split_once(':'))
        .filter(|(code, _)| !code.is_empty() && code.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|(code, _)| code.parse::<u64>().ok())
        .is_some_and(|code| CONSENT_BLOCKED_CODES.contains(&code));
    listed || described
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub tenant_id: String,
    pub client_id: String,
}
impl AuthConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !is_guid(&self.tenant_id) || !is_guid(&self.client_id) {
            return Err("Enter the organization tenant ID and public application ID supplied by your administrator.".into());
        }
        Ok(())
    }
    fn key(&self) -> String {
        format!(
            "{}:{}",
            self.tenant_id.to_ascii_lowercase(),
            self.client_id.to_ascii_lowercase()
        )
    }
    fn endpoint(&self, operation: &str) -> Result<Url, String> {
        self.validate()?;
        Url::parse(&format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/{operation}",
            self.tenant_id
        ))
        .map_err(|_| "Microsoft sign-in URL is invalid.".into())
    }
}

pub trait TokenStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    fn set(&self, key: &str, value: &str) -> Result<(), String>;
    fn delete(&self, key: &str) -> Result<(), String>;
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePrompt {
    pub user_code: String,
    pub verification_uri: String,
    pub interval_seconds: u64,
    pub expires_at: i64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SignInProgress {
    Pending {
        #[serde(rename = "intervalSeconds")]
        interval_seconds: u64,
    },
    Connected {
        account: Account,
    },
}

// Deliberately no Debug/Serialize on access sessions and device codes.
#[derive(Clone)]
struct Session {
    token: String,
    account: Account,
    expires_at: i64,
}
struct Pending {
    code: String,
    expires_at: i64,
    next_poll: i64,
    interval: u64,
}
#[derive(Deserialize, Serialize)]
struct StoredSession {
    refresh_token: String,
    account: Account,
}
struct State {
    config: AuthConfig,
    session: Option<Session>,
    pending: Option<Pending>,
    retry_at: i64,
    allow_refresh: bool,
}

/// A failed Graph request, and whether it says anything about the connection.
///
/// Only a failure of the connection itself justifies pausing every other
/// file's verification. A 404 for one file the sync client has not finished
/// uploading is a verdict about that file, and holding the whole folder
/// behind it means the slowest document in a share decides how fast every
/// other document is checked.
struct Failure {
    message: String,
    back_off: bool,
}

impl Failure {
    fn connection(message: String) -> Self {
        Self {
            message,
            back_off: true,
        }
    }
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self {
            message,
            back_off: false,
        }
    }
}

impl From<&str> for Failure {
    fn from(message: &str) -> Self {
        Self::from(message.to_string())
    }
}

pub struct MicrosoftClient {
    deployment: SharePointDeployment,
    transport: Arc<dyn Transport>,
    tokens: Arc<dyn TokenStore>,
    clock: Arc<dyn Clock>,
    state: Mutex<State>,
    /// Who is signed in, mirrored out of `state`. A verification holds the
    /// state lock across its HTTP calls, and Settings asking who is connected
    /// must not be answered "nobody" merely because the connection is busy.
    connected: Mutex<Option<Account>>,
}
impl MicrosoftClient {
    pub fn new(
        deployment: SharePointDeployment,
        tokens: Arc<dyn TokenStore>,
    ) -> Result<Self, String> {
        let transport = Arc::new(MicrosoftTransport::new(&deployment)?);
        Ok(Self::with_transport(
            deployment,
            tokens,
            transport,
            Arc::new(SystemClock),
        ))
    }
    pub fn with_transport(
        deployment: SharePointDeployment,
        tokens: Arc<dyn TokenStore>,
        transport: Arc<dyn Transport>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let config = AuthConfig {
            tenant_id: deployment.tenant_id().to_owned(),
            client_id: deployment.client_id().to_owned(),
        };
        Self {
            deployment,
            transport,
            tokens,
            clock,
            state: Mutex::new(State {
                config,
                session: None,
                pending: None,
                retry_at: 0,
                allow_refresh: true,
            }),
            connected: Mutex::new(None),
        }
    }
    /// The signed-in person, from the mirror rather than the connection state.
    ///
    /// An access token that has expired is not a person signing out - the
    /// stored refresh token brings the next one - so this reports whoever the
    /// last established session belonged to, and nothing at all once the
    /// session is deliberately dropped.
    pub fn account(&self) -> Option<Account> {
        self.connected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    /// Every change of session goes through here so the mirror cannot drift.
    fn set_session(&self, state: &mut State, session: Option<Session>) {
        *self
            .connected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            session.as_ref().map(|session| session.account.clone());
        state.session = session;
    }
    pub fn begin(&self) -> Result<DevicePrompt, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Microsoft sign-in state is unavailable.")?;
        let config = state.config.clone();
        config.validate()?;
        self.set_session(&mut state, None);
        state.pending = None;
        state.retry_at = 0;
        state.allow_refresh = false;
        let reply = self.transport.request(
            config.endpoint("devicecode")?,
            Some(&[("client_id", &config.client_id), ("scope", SCOPES)]),
            None,
        )?;
        if reply.status != 200 {
            if consent_blocked(&reply.body) {
                return Err(CONSENT_BLOCKED.into());
            }
            return Err("Microsoft sign-in could not start. Check the application registration, public-client setting, and organization consent.".into());
        }
        let code = text(&reply.body, "/device_code")?.to_owned();
        let user_code = text(&reply.body, "/user_code")?.to_owned();
        let verification_uri = text(&reply.body, "/verification_uri")?;
        // Microsoft supplies this URL, but the UI must not become a phishing redirect.
        if ![
            "https://microsoft.com/devicelogin",
            "https://www.microsoft.com/devicelogin",
        ]
        .contains(&verification_uri)
        {
            return Err("Microsoft returned an unexpected sign-in address.".into());
        }
        let interval = reply.body["interval"].as_u64().unwrap_or(5).clamp(5, 60);
        let lifetime = reply.body["expires_in"]
            .as_i64()
            .filter(|seconds| (1..=1800).contains(seconds))
            .ok_or("Microsoft sign-in expiry is invalid.")?;
        let expires_at = self.clock.now() + lifetime;
        state.pending = Some(Pending {
            code,
            expires_at,
            next_poll: self.clock.now() + interval as i64,
            interval,
        });
        Ok(DevicePrompt {
            user_code,
            verification_uri: verification_uri.to_owned(),
            interval_seconds: interval,
            expires_at,
        })
    }
    pub fn poll(&self) -> Result<SignInProgress, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Microsoft sign-in state is unavailable.")?;
        let config = state.config.clone();
        let pending = state
            .pending
            .as_mut()
            .ok_or("Start Microsoft sign-in again.")?;
        if self.clock.now() >= pending.expires_at {
            state.pending = None;
            return Err("Microsoft sign-in expired. Start again; files remain held.".into());
        }
        if self.clock.now() < pending.next_poll {
            return Ok(SignInProgress::Pending {
                interval_seconds: pending.interval,
            });
        }
        pending.next_poll = self.clock.now() + pending.interval as i64;
        let reply = self.transport.request(
            config.endpoint("token")?,
            Some(&[
                ("client_id", &config.client_id),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &pending.code),
            ]),
            None,
        )?;
        if reply.status != 200 {
            match reply.body["error"].as_str() {
                Some("authorization_pending") => {
                    return Ok(SignInProgress::Pending {
                        interval_seconds: pending.interval,
                    });
                }
                Some("slow_down") => {
                    pending.interval = (pending.interval + 5).min(60);
                    pending.next_poll = self.clock.now() + pending.interval as i64;
                    return Ok(SignInProgress::Pending {
                        interval_seconds: pending.interval,
                    });
                }
                _ => {
                    state.pending = None;
                    if consent_blocked(&reply.body) {
                        return Err(CONSENT_BLOCKED.into());
                    }
                    return Err("Microsoft sign-in was declined, expired, or blocked by organization policy. Files remain held.".into());
                }
            }
        }
        state.pending = None;
        let session = self.establish(&config, &reply.body, None)?;
        let account = session.account.clone();
        self.set_session(&mut state, Some(session));
        state.allow_refresh = true;
        Ok(SignInProgress::Connected { account })
    }
    pub fn disconnect(&self) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Microsoft sign-in state is unavailable.")?;
        state.pending = None;
        self.set_session(&mut state, None);
        state.retry_at = 0;
        state.allow_refresh = false;
        if state.config.validate().is_ok() {
            self.tokens.delete(&state.config.key())?;
        }
        Ok(())
    }
    /// Every metadata request obtains a current delegated session. Refreshing
    /// rechecks /me against the stored tenant-scoped ID before accepting it.
    pub fn metadata(&self, url: Url) -> Result<(Account, Value), String> {
        self.graph_request(url)
    }
    pub fn start_audit_query(&self, _body: &Value) -> Result<(Account, Value), String> {
        Err("Microsoft audit endpoints are not permitted.".into())
    }
    fn graph_request(&self, url: Url) -> Result<(Account, Value), String> {
        if !deployment_allows_endpoint(&self.deployment, &url, false, true) {
            return Err("Only Microsoft intake metadata may be requested.".into());
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Microsoft connection is unavailable.")?;
        if !state.allow_refresh || state.pending.is_some() {
            return Err(
                "Microsoft is disconnected or sign-in is incomplete. Files remain held.".into(),
            );
        }
        if self.clock.now() < state.retry_at {
            return Err("Microsoft verification is waiting to retry. Files remain held.".into());
        }
        let result = (|| -> Result<(Account, Value), Failure> {
            if state
                .session
                .as_ref()
                .is_none_or(|session| session.expires_at <= self.clock.now() + 30)
            {
                self.set_session(&mut state, None);
                state.config.validate()?;
                let raw = self
                    .tokens
                    .get(&state.config.key())?
                    .ok_or("Connect your Microsoft account to verify uploads.")?;
                if raw.len() > 64 * 1024 {
                    return Err("Stored Microsoft credentials are invalid. Sign in again.".into());
                }
                let stored: StoredSession = serde_json::from_str(&raw)
                    .map_err(|_| "Stored Microsoft credentials are invalid. Sign in again.")?;
                if !stored
                    .account
                    .tenant_id
                    .eq_ignore_ascii_case(&state.config.tenant_id)
                {
                    return Err("Microsoft organization changed. Sign in again.".into());
                }
                let reply = self
                    .transport
                    .request(
                        state.config.endpoint("token")?,
                        Some(&[
                            ("client_id", &state.config.client_id),
                            ("grant_type", "refresh_token"),
                            ("refresh_token", &stored.refresh_token),
                            ("scope", SCOPES),
                        ]),
                        None,
                    )
                    .map_err(Failure::connection)?;
                if reply.status != 200 {
                    if consent_blocked(&reply.body) {
                        return Err(Failure::connection(CONSENT_BLOCKED.into()));
                    }
                    return Err(Failure::connection("Microsoft sign-in needs attention. Reconnect your account; files remain held.".into()));
                }
                let session = self
                    .establish(&state.config, &reply.body, Some(&stored))
                    .map_err(Failure::connection)?;
                self.set_session(&mut state, Some(session));
            }
            let session = state
                .session
                .as_ref()
                .ok_or("Connect Microsoft before verifying uploads.")?;
            let reply = self
                .transport
                .request(url, None, Some(&session.token))
                .map_err(Failure::connection)?;
            match reply.status {
                200 | 201 => Ok((session.account.clone(),reply.body)),
                401 => { self.set_session(&mut state, None); Err("Microsoft sign-in expired. Reconnect; files remain held.".into()) }
                403 => Err("Microsoft denied access to this folder. Ask your administrator to grant the app read access to the selected intake folder.".into()),
                404 => Err("This file is not available in the SharePoint Inbox yet.".into()),
                429 | 503 => { state.retry_at=self.clock.now()+reply.retry_after as i64; Err("Microsoft requested a slower verification rate. Files remain held until retry.".into()) }
                _ => Err("Microsoft could not verify this upload. Files remain held.".into()),
            }
        })();
        result.map_err(|failure| {
            if failure.back_off {
                state.retry_at = state.retry_at.max(self.clock.now() + 10);
            }
            failure.message
        })
    }
    fn establish(
        &self,
        config: &AuthConfig,
        reply: &Value,
        previous: Option<&StoredSession>,
    ) -> Result<Session, String> {
        if reply["token_type"]
            .as_str()
            .is_none_or(|kind| !kind.eq_ignore_ascii_case("Bearer"))
        {
            return Err("Microsoft returned an unusable sign-in token.".into());
        }
        let token = reply["access_token"]
            .as_str()
            .filter(|token| !token.is_empty() && token.len() <= 64 * 1024)
            .ok_or("Microsoft returned no access token.")?;
        let lifetime = reply["expires_in"]
            .as_i64()
            .filter(|seconds| (60..=86400).contains(seconds))
            .ok_or("Microsoft token expiry is invalid.")?;
        let profile = self.transport.request(
            Url::parse(
                "https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName",
            )
            .map_err(|_| "Microsoft profile URL is invalid.")?,
            None,
            Some(token),
        )?;
        if profile.status != 200 {
            return Err("Microsoft could not verify the signed-in person.".into());
        }
        let id = text(&profile.body, "/id")?;
        if !is_guid(id) {
            return Err(
                "This connection currently requires a Microsoft work or school account.".into(),
            );
        }
        let account = Account {
            tenant_id: config.tenant_id.to_ascii_lowercase(),
            id: id.to_ascii_lowercase(),
            display_name: text(&profile.body, "/displayName")?.to_owned(),
            user_principal_name: profile.body["userPrincipalName"]
                .as_str()
                .unwrap_or("")
                .to_owned(),
            email: profile.body["mail"]
                .as_str()
                .filter(|email| !email.trim().is_empty())
                .or_else(|| profile.body["userPrincipalName"].as_str())
                .unwrap_or("")
                .to_owned(),
        };
        if previous.is_some_and(|stored| !super::proof::same_person(&stored.account, &account)) {
            return Err("The Microsoft account changed. Sign in deliberately before processing any uploads.".into());
        }
        let refresh_token = reply["refresh_token"]
            .as_str()
            .filter(|token| !token.is_empty() && token.len() <= 64 * 1024)
            .or_else(|| previous.map(|stored| stored.refresh_token.as_str()))
            .ok_or("Microsoft did not grant a persistent sign-in. Files remain held.")?;
        let stored = serde_json::to_string(&StoredSession {
            refresh_token: refresh_token.to_owned(),
            account: account.clone(),
        })
        .map_err(|_| "Microsoft credentials could not be protected.")?;
        self.tokens.set(&config.key(), &stored)?;
        Ok(Session {
            token: token.to_owned(),
            account,
            expires_at: self.clock.now() + lifetime,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::transport::Reply;
    use super::*;
    use serde_json::json;
    use std::{
        collections::{HashMap, VecDeque},
        sync::{
            atomic::{AtomicI64, Ordering},
            mpsc,
        },
        thread,
    };
    #[derive(Default)]
    struct Memory(Mutex<HashMap<String, String>>);
    impl TokenStore for Memory {
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
    struct Time(AtomicI64);
    impl Clock for Time {
        fn now(&self) -> i64 {
            self.0.load(Ordering::SeqCst)
        }
    }
    /// One recorded request: URL, whether it was a POST, and its form fields.
    type Call = (String, bool, Vec<(String, String)>);
    struct Fake {
        replies: Mutex<VecDeque<Reply>>,
        calls: Mutex<Vec<Call>>,
        audit_calls: Mutex<usize>,
    }
    impl Fake {
        fn new(replies: Vec<Reply>) -> Self {
            Self {
                replies: Mutex::new(replies.into()),
                calls: Mutex::new(Vec::new()),
                audit_calls: Mutex::new(0),
            }
        }
    }
    impl Transport for Fake {
        fn request(
            &self,
            url: Url,
            form: Option<&[(&str, &str)]>,
            bearer: Option<&str>,
        ) -> Result<Reply, String> {
            self.calls.lock().unwrap().push((
                url.to_string(),
                bearer.is_some(),
                form.unwrap_or_default()
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                    .collect(),
            ));
            self.replies
                .lock()
                .unwrap()
                .pop_front()
                .ok_or("Unexpected request".into())
        }
    }
    fn config() -> AuthConfig {
        AuthConfig {
            tenant_id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".into(),
            client_id: "cccccccc-cccc-cccc-cccc-cccccccccccc".into(),
        }
    }
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
              "client_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
              "site_id": "11111111-1111-1111-1111-111111111111",
              "web_id": "22222222-2222-2222-2222-222222222222",
              "list_id": "33333333-3333-3333-3333-333333333333",
              "drive_id": "b!TTO6DSRqwEyBsbryPjv57vX3nytJNK-H9VILablLDZguhbtVtnKocmN6zXRm_LYO",
              "intake_folder_id": "01SYNTHETICINBOXFOLDERAAAAAAAAAAAA",
              "destination_folder_id": "01SYNTHETICFILEDFOLDERAAAAAAAAAAAA"
            }"#,
        )
        .unwrap()
    }
    fn reply(status: u16, body: Value) -> Reply {
        Reply {
            status,
            body,
            retry_after: 60,
        }
    }
    fn device() -> Reply {
        reply(
            200,
            json!({"device_code":"private-device-code","user_code":"ABCD-EFGH","verification_uri":"https://microsoft.com/devicelogin","expires_in":900,"interval":5}),
        )
    }
    fn token() -> Reply {
        reply(
            200,
            json!({"token_type":"Bearer","access_token":"private-access-token","refresh_token":"private-refresh-token","expires_in":3600}),
        )
    }
    fn me() -> Reply {
        reply(
            200,
            json!({"id":"bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb","displayName":"Zachary Brenner","mail":"zack@example.test","userPrincipalName":"zack@example.test"}),
        )
    }
    fn rig(replies: Vec<Reply>) -> (MicrosoftClient, Arc<Fake>, Arc<Memory>, Arc<Time>) {
        let http = Arc::new(Fake::new(replies));
        let store = Arc::new(Memory::default());
        let clock = Arc::new(Time(AtomicI64::new(1000)));
        (
            MicrosoftClient::with_transport(
                deployment(),
                store.clone(),
                http.clone(),
                clock.clone(),
            ),
            http,
            store,
            clock,
        )
    }
    fn item() -> Url {
        super::super::transport::item_url(
            deployment().drive_id(),
            deployment().intake_folder_id(),
            None,
        )
        .unwrap()
    }
    #[test]
    fn construction_and_status_make_no_network_calls() {
        let (client, http, _, _) = rig(vec![]);
        assert!(client.account().is_none());
        assert!(http.calls.lock().unwrap().is_empty());
    }
    #[test]
    fn device_secrets_never_leave_the_backend_prompt() {
        let (client, _, _, _) = rig(vec![device()]);
        let prompt = serde_json::to_string(&client.begin().unwrap()).unwrap();
        assert!(prompt.contains("ABCD-EFGH"));
        assert!(!prompt.contains("private-device-code"));
        assert!(!prompt.contains("private-access-token"));
    }
    #[test]
    fn device_sign_in_requests_only_profile_file_metadata_and_persistence() {
        let (client, http, _, _) = rig(vec![device()]);

        client.begin().unwrap();

        let calls = http.calls.lock().unwrap();
        assert_eq!(
            calls[0].2,
            vec![
                (
                    "client_id".to_owned(),
                    "cccccccc-cccc-cccc-cccc-cccccccccccc".to_owned(),
                ),
                (
                    "scope".to_owned(),
                    "https://graph.microsoft.com/User.Read https://graph.microsoft.com/Files.Read offline_access"
                        .to_owned(),
                ),
            ]
        );
    }
    #[test]
    fn poll_waits_for_microsoft_interval_and_establishes_identity_via_me() {
        let (client, http, store, time) = rig(vec![device(), token(), me()]);
        client.begin().unwrap();
        assert!(matches!(
            client.poll().unwrap(),
            SignInProgress::Pending { .. }
        ));
        assert_eq!(http.calls.lock().unwrap().len(), 1);
        time.0.store(1005, Ordering::SeqCst);
        let result = client.poll().unwrap();
        assert!(matches!(result, SignInProgress::Connected { .. }));
        let public = serde_json::to_string(&result).unwrap();
        assert!(!public.contains("private-"));
        let stored = store.get(&config().key()).unwrap().unwrap();
        assert!(stored.contains("private-refresh-token"));
        assert!(!stored.contains("private-access-token"));
        assert!(http.calls.lock().unwrap()[2].0.contains("/me?"));
    }
    #[test]
    fn disconnected_account_cannot_be_resurrected_from_a_late_refresh() {
        let (client, http, store, time) = rig(vec![device(), token(), me()]);
        client.begin().unwrap();
        time.0.store(1005, Ordering::SeqCst);
        client.poll().unwrap();
        let old = store.get(&config().key()).unwrap().unwrap();
        client.disconnect().unwrap();
        store.set(&config().key(), &old).unwrap();
        assert!(client.metadata(item()).is_err());
        assert_eq!(http.calls.lock().unwrap().len(), 3);
    }
    #[test]
    fn incomplete_signin_cannot_fall_back_to_previously_stored_credentials() {
        let (client, http, store, _) = rig(vec![device()]);
        store.set(&config().key(), "old-secret").unwrap();
        client.begin().unwrap();
        assert!(client.metadata(item()).is_err());
        assert_eq!(http.calls.lock().unwrap().len(), 1);
    }
    #[test]
    fn expiry_and_slow_down_are_respected() {
        let (client, http, _, time) = rig(vec![device(), reply(400, json!({"error":"slow_down"}))]);
        client.begin().unwrap();
        time.0.store(1005, Ordering::SeqCst);
        assert!(matches!(
            client.poll().unwrap(),
            SignInProgress::Pending {
                interval_seconds: 10
            }
        ));
        time.0.store(1010, Ordering::SeqCst);
        client.poll().unwrap();
        assert_eq!(http.calls.lock().unwrap().len(), 2);
        time.0.store(2000, Ordering::SeqCst);
        assert!(client.poll().is_err());
    }
    #[test]
    fn redirects_and_arbitrary_endpoints_are_never_used_for_authenticated_reads() {
        let (client, http, _, _) = rig(vec![]);
        for url in [
            "https://evil.example/me",
            "http://graph.microsoft.com/v1.0/me",
            "https://graph.microsoft.com/v1.0/drives/drive/items/id/content",
            "https://graph.microsoft.com/v1.0/users",
        ] {
            assert!(client.metadata(Url::parse(url).unwrap()).is_err());
        }
        assert!(http.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn a_connected_injected_transport_cannot_escape_the_deployment_item_boundary() {
        let (client, http, _, time) = rig(vec![device(), token(), me()]);
        client.begin().unwrap();
        time.0.store(1005, Ordering::SeqCst);
        client.poll().unwrap();
        let connected_calls = http.calls.lock().unwrap().len();

        for url in [
            super::super::transport::item_url(
                "99999999-9999-9999-9999-999999999999",
                "55555555-5555-5555-5555-555555555555",
                Some("agreement.pdf"),
            )
            .unwrap(),
            super::super::transport::item_url(
                "44444444-4444-4444-4444-444444444444",
                "99999999-9999-9999-9999-999999999999",
                None,
            )
            .unwrap(),
        ] {
            assert!(client.metadata(url).is_err());
        }
        assert_eq!(
            http.calls.lock().unwrap().len(),
            connected_calls,
            "rejected metadata URLs must never reach even an injected transport"
        );
    }
    #[test]
    fn audit_queries_are_not_a_connected_client_capability() {
        let (client, http, _, time) = rig(vec![device(), token(), me()]);
        client.begin().unwrap();
        time.0.store(1005, Ordering::SeqCst);
        client.poll().unwrap();

        let error = client.start_audit_query(&json!({})).unwrap_err();

        assert_eq!(error, "Microsoft audit endpoints are not permitted.");
        assert_eq!(*http.audit_calls.lock().unwrap(), 0);
    }
    /// A file the sync client has not finished uploading yet answers 404, and
    /// that is a verdict about one file. Pausing the whole client for it holds
    /// every other file in the folder behind the slowest one.
    #[test]
    fn a_per_file_404_does_not_block_other_requests() {
        let (client, http, _, time) = rig(vec![
            device(),
            token(),
            me(),
            reply(404, json!({})),
            reply(200, json!({"id": "item"})),
        ]);
        client.begin().unwrap();
        time.0.store(1005, Ordering::SeqCst);
        client.poll().unwrap();
        assert!(client.metadata(item()).is_err());
        assert!(
            client.metadata(item()).is_ok(),
            "the next file must still be checked"
        );
        assert_eq!(http.calls.lock().unwrap().len(), 5);
    }

    /// Microsoft being unreachable is not a verdict about any one file, so the
    /// client does back off before trying again.
    #[test]
    fn an_unreachable_microsoft_pauses_verification() {
        let (client, http, _, time) = rig(vec![device(), token(), me()]);
        client.begin().unwrap();
        time.0.store(1005, Ordering::SeqCst);
        client.poll().unwrap();
        assert!(client.metadata(item()).is_err());
        assert!(client.metadata(item()).is_err());
        assert_eq!(
            http.calls.lock().unwrap().len(),
            4,
            "the second attempt must not reach the network"
        );
    }

    /// Blocks on the last scripted reply, so a request can be caught in
    /// flight.
    struct Held {
        replies: Mutex<VecDeque<Reply>>,
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl Transport for Held {
        fn request(
            &self,
            _url: Url,
            _form: Option<&[(&str, &str)]>,
            _bearer: Option<&str>,
        ) -> Result<Reply, String> {
            let reply = self
                .replies
                .lock()
                .unwrap()
                .pop_front()
                .ok_or("Unexpected request")?;
            if self.replies.lock().unwrap().is_empty() {
                self.entered.send(()).unwrap();
                self.release.lock().unwrap().recv().unwrap();
            }
            Ok(reply)
        }
    }

    /// A verification holds the connection state across its HTTP calls, and
    /// the pipeline runs them back to back. Settings asking who is signed in
    /// must not be told "nobody" for as long as that lasts.
    #[test]
    fn the_connected_account_is_visible_while_a_request_is_in_flight() {
        let (entered, entered_here) = mpsc::channel();
        let (release, released_there) = mpsc::channel();
        let transport = Arc::new(Held {
            replies: Mutex::new(
                vec![device(), token(), me(), reply(200, json!({"id": "item"}))].into(),
            ),
            entered,
            release: Mutex::new(released_there),
        });
        let time = Arc::new(Time(AtomicI64::new(1000)));
        let client = Arc::new(MicrosoftClient::with_transport(
            deployment(),
            Arc::new(Memory::default()),
            transport,
            time.clone(),
        ));
        client.begin().unwrap();
        time.0.store(1005, Ordering::SeqCst);
        client.poll().unwrap();

        let verifying = {
            let client = client.clone();
            thread::spawn(move || client.metadata(item()))
        };
        entered_here.recv().unwrap();
        assert!(
            client.account().is_some(),
            "a busy connection is not a disconnected one"
        );
        release.send(()).unwrap();
        verifying.join().unwrap().unwrap();
    }

    const CONSENT_PREFIX: &str = "Your Microsoft organization has blocked Intern from connecting.";
    const CONSENT_TOKEN: &str = "(MICROSOFT_CONSENT_BLOCKED)";
    /// Entra ID consent failures as Microsoft reports them: the numeric
    /// `error_codes` list and an `AADSTS` prefix on the description.
    fn consent_failures() -> Vec<Value> {
        let mut failures = Vec::new();
        for code in [65001, 90093, 90094, 90095, 900941] {
            failures.push(json!({
                "error": "invalid_grant",
                "error_codes": [code],
                "error_description": format!("AADSTS{code}: private-provider-detail
            Trace ID: 1")
            }));
            failures.push(json!({
                "error": "invalid_client",
                "error_description": format!("AADSTS{code}: private-provider-detail")
            }));
        }
        failures
    }
    fn assert_consent_blocked(error: &str) {
        assert!(error.starts_with(CONSENT_PREFIX), "{error}");
        assert!(error.contains(CONSENT_TOKEN), "{error}");
        assert!(!error.contains("private-provider-detail"), "{error}");
    }
    #[test]
    fn a_consent_blocked_device_code_request_names_the_organization_block() {
        for failure in consent_failures() {
            let (client, _, _, _) = rig(vec![reply(400, failure.clone())]);
            assert_consent_blocked(&client.begin().unwrap_err());
        }
    }
    #[test]
    fn a_consent_blocked_token_poll_names_the_organization_block() {
        for failure in consent_failures() {
            let (client, _, _, time) = rig(vec![device(), reply(400, failure.clone())]);
            client.begin().unwrap();
            time.0.store(1005, Ordering::SeqCst);
            assert_consent_blocked(&client.poll().unwrap_err());
            assert!(
                client.poll().is_err(),
                "a blocked sign-in must be started again"
            );
        }
    }
    #[test]
    fn a_consent_blocked_refresh_names_the_organization_block() {
        for failure in consent_failures() {
            let (client, _, _, time) = rig(vec![device(), token(), me(), reply(400, failure)]);
            client.begin().unwrap();
            time.0.store(1005, Ordering::SeqCst);
            client.poll().unwrap();
            time.0.store(1005 + 3600, Ordering::SeqCst);
            assert_consent_blocked(&client.metadata(item()).unwrap_err());
        }
    }
    #[test]
    fn other_sign_in_failures_are_not_reported_as_consent_blocks() {
        for failure in [
            json!({"error": "expired_token", "error_codes": [70020]}),
            json!({"error": "access_denied", "error_codes": [65004], "error_description": "AADSTS65004: declined"}),
            json!({"error": "invalid_grant", "error_description": "AADSTS650010: not 65001"}),
            json!({"error": "invalid_grant", "error_description": "See AADSTS65001 elsewhere"}),
            json!({"error": "invalid_grant", "error_codes": ["65001"]}),
            json!({}),
        ] {
            let (client, _, _, time) = rig(vec![
                reply(400, failure.clone()),
                device(),
                reply(400, failure.clone()),
            ]);
            let error = client.begin().unwrap_err();
            assert!(!error.contains(CONSENT_TOKEN), "{failure}: {error}");
            client.begin().unwrap();
            time.0.store(1005, Ordering::SeqCst);
            let error = client.poll().unwrap_err();
            assert!(!error.contains(CONSENT_TOKEN), "{failure}: {error}");
        }
    }
    #[test]
    fn authentication_failure_does_not_echo_provider_response_or_tokens() {
        let (client, _, _, _) = rig(vec![reply(
            400,
            json!({"error_description":"private-client-secret"}),
        )]);
        let error = client.begin().unwrap_err();
        assert!(!error.contains("private-client-secret"));
    }
}
