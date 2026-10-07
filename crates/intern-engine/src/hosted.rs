//! A hosted model behind an API key, standing in for the local one.
//!
//! Everything about how Intern understands a document is unchanged: the same
//! distillation of the whole file, the same prompt, the same evidence checks
//! on the reply, the same naming. What changes is where the prompt goes. The
//! local server never leaves `127.0.0.1`; this client sends the distilled
//! text of every document to whoever runs the endpoint, and that is the whole
//! reason it is off unless a person turns it on, supplies a key, and is told
//! in Settings what it means.
//!
//! Two wire formats cover nearly every service: Anthropic's Messages API, and
//! the chat-completions shape that OpenAI defined and most other providers
//! and local servers (LM Studio, Ollama, llama.cpp) copy. A request carries
//! only what every server understands - the model, the two messages, and for
//! Anthropic the required output cap - because a sampling knob one provider
//! rejects is a document that never gets filed.

use std::time::{Duration, SystemTime};

use reqwest::{StatusCode, Url, blocking::Client};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::client::{
    AttemptError, ChatCompletion, EvidenceHandles, ModelRequest, Proposer, decode,
    is_context_overflow, proposal_from_text, read_capped, read_error_body,
};
use crate::domain::{DocumentAnalysis, ModelProposal};
use crate::engine::Engine;
use crate::error::{EngineError, EngineErrorCode, EngineResult};
use crate::setup::{semantic_probes, validate_semantic_probe};

/// The Messages API version this client speaks.
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
/// The model offered when the Anthropic provider is chosen and none is named.
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-opus-5";

/// Room for the reply. On an Anthropic model the cap also covers the thinking
/// that precedes the answer, so it is generous; the answer itself is short.
const MAX_REPLY_TOKENS: u32 = 16_000;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// How long a service on the internet gets to answer one document.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);
/// How long a server on this machine gets. LM Studio or Ollama running a 7-8B
/// model on a laptop CPU spends minutes on prefill alone, and at 180 s every
/// long document timed out, was retried into the same timeout, and paused the
/// queue.
///
/// Not the local llama-server's ten minutes: a request that times out is
/// retried once, so two of these and the longest wait between them (860 s)
/// must end inside the queue's fifteen-minute deadline. At ten minutes they
/// did not, and the queue gave up on the document while its retry went on
/// running on the same server - in front of the next document, and of this
/// one, claimed again at once.
const LOCAL_REQUEST_TIMEOUT: Duration = Duration::from_secs(400);
/// The longest wait a `Retry-After` is taken at its word for. A service that
/// asks for longer is better answered by a paused queue and a person.
const MAX_RETRY_AFTER_SECS: u64 = 120;

/// Which wire format the endpoint speaks.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostedProvider {
    /// Anthropic's Messages API.
    #[default]
    Anthropic,
    /// OpenAI's chat completions, and every service or local server that
    /// copies the shape.
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
}

impl HostedProvider {
    pub const ALL: [Self; 2] = [Self::Anthropic, Self::OpenAiCompatible];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }

    pub const fn default_base_url(self) -> &'static str {
        match self {
            Self::Anthropic => DEFAULT_ANTHROPIC_BASE_URL,
            Self::OpenAiCompatible => DEFAULT_OPENAI_BASE_URL,
        }
    }

    /// The model used when none is named; only Anthropic has a sensible one.
    pub const fn default_model(self) -> &'static str {
        match self {
            Self::Anthropic => DEFAULT_ANTHROPIC_MODEL,
            Self::OpenAiCompatible => "",
        }
    }
}

/// Everything needed to reach one hosted model.
#[derive(Clone, Eq, PartialEq)]
pub struct HostedModelConfig {
    pub provider: HostedProvider,
    /// The API root, `https://api.anthropic.com/v1` or the like. Empty means
    /// the provider's default.
    pub base_url: String,
    /// The model name as the service knows it. Empty means the provider's
    /// default, where there is one.
    pub model: String,
    pub api_key: String,
}

/// The key is kept out of the printed form for the same reason it is kept out
/// of the settings file: it is stored in the credential store, and anything
/// that prints a config - a log line, a panic message, a debug assertion -
/// would otherwise put it somewhere nobody meant it to be.
impl std::fmt::Debug for HostedModelConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HostedModelConfig")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

impl HostedModelConfig {
    /// Fills empty fields with the provider's defaults and refuses anything
    /// a request could not be made from.
    pub fn resolved(self) -> EngineResult<Self> {
        let base_url = {
            let trimmed = self.base_url.trim();
            if trimmed.is_empty() {
                self.provider.default_base_url().to_owned()
            } else {
                trimmed.to_owned()
            }
        };
        let model = {
            let trimmed = self.model.trim();
            if trimmed.is_empty() {
                self.provider.default_model().to_owned()
            } else {
                trimmed.to_owned()
            }
        };
        let api_key = self.api_key.trim().to_owned();
        if model.is_empty() {
            return Err(misconfigured("the hosted model has no model name"));
        }
        if api_key.is_empty() {
            return Err(misconfigured("the hosted model has no API key"));
        }
        endpoint_for(self.provider, &base_url)?;
        Ok(Self {
            provider: self.provider,
            base_url,
            model,
            api_key,
        })
    }
}

/// The request endpoint for a provider under an API root: `<root>/messages`
/// for Anthropic, `<root>/chat/completions` for the rest. Only HTTPS is
/// accepted, except plain HTTP to this machine, which is how a local server
/// such as LM Studio or Ollama is reached.
pub fn endpoint_for(provider: HostedProvider, base_url: &str) -> EngineResult<Url> {
    let root = Url::parse(base_url.trim().trim_end_matches('/'))
        .map_err(|_| misconfigured("the hosted model's address is not a URL"))?;
    let local = is_this_machine(&root);
    match root.scheme() {
        "https" => {}
        "http" if local => {}
        _ => {
            return Err(misconfigured(
                "the hosted model's address must use https, unless it is this machine",
            ));
        }
    }
    if root.cannot_be_a_base() || root.host_str().is_none() {
        return Err(misconfigured("the hosted model's address has no host"));
    }
    let path = match provider {
        HostedProvider::Anthropic => "messages",
        HostedProvider::OpenAiCompatible => "chat/completions",
    };
    let mut endpoint = root;
    let base_path = endpoint.path().trim_end_matches('/').to_owned();
    endpoint.set_path(&format!("{base_path}/{path}"));
    endpoint.set_query(None);
    endpoint.set_fragment(None);
    Ok(endpoint)
}

/// Whether an address is this machine.
fn is_this_machine(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    })
}

/// How long one request to this endpoint may take.
pub fn request_timeout_for(url: &Url) -> Duration {
    if is_this_machine(url) {
        LOCAL_REQUEST_TIMEOUT
    } else {
        REQUEST_TIMEOUT
    }
}

/// Whether a request to this endpoint goes through the machine's proxy.
///
/// The same judgement that lets plain HTTP through decides this, and it has
/// to: the loopback exception exists so a server on this machine can be used
/// without a certificate, not so a proxy configured for the internet can be
/// handed the API key and the whole distilled text of every document in
/// plaintext on the way to `localhost`. A hosted service on the internet is
/// reached through the proxy as before.
fn uses_system_proxy(endpoint: &Url) -> bool {
    !is_this_machine(endpoint)
}

/// The client for one hosted model.
#[derive(Clone)]
pub struct HostedClient {
    config: HostedModelConfig,
    endpoint: Url,
    http: Client,
}

impl std::fmt::Debug for HostedClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HostedClient")
            .field("provider", &self.config.provider)
            .field("endpoint", &self.endpoint.as_str())
            .field("model", &self.config.model)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

impl HostedClient {
    pub fn new(config: HostedModelConfig) -> EngineResult<Self> {
        let config = config.resolved()?;
        let endpoint = endpoint_for(config.provider, &config.base_url)?;
        // The system proxy is honoured for a service on the internet, unlike
        // for the local server: a machine that reaches the internet through a
        // proxy reaches that endpoint through it too. It is bypassed for an
        // endpoint on this machine, because a proxy would otherwise be handed
        // the key and the document text that plain HTTP was allowed for
        // exactly on the grounds that neither leaves the machine. Redirects
        // are refused either way, so a key is only ever sent to the address
        // that was configured.
        let build = |native_roots: bool| {
            let mut builder = Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .timeout(request_timeout_for(&endpoint))
                .redirect(reqwest::redirect::Policy::none())
                .tls_built_in_native_certs(native_roots);
            if !uses_system_proxy(&endpoint) {
                builder = builder.no_proxy();
            }
            builder.build()
        };
        // The operating system's roots are what make a firm's inspecting
        // proxy trusted, but a store with nothing usable in it fails the
        // build outright. The bundled roots alone are still a working client.
        let http = build(true)
            .or_else(|_| build(false))
            .map_err(|_| unreachable_error())?;
        Ok(Self {
            config,
            endpoint,
            http,
        })
    }

    pub fn provider(&self) -> HostedProvider {
        self.config.provider
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    /// Sends the calibration document the local model is checked with, so a
    /// wrong key, model name, or address is found before a real document is.
    pub fn probe(&self) -> EngineResult<DocumentAnalysis> {
        let engine = Engine::with_proposer(Box::new(self.clone()));
        let mut last = None;
        for probe in semantic_probes()? {
            let analysis = engine.analyze(&probe.document, "pdf", &[])?;
            validate_semantic_probe(&probe, &analysis)?;
            last = Some(analysis);
        }
        last.ok_or_else(|| {
            EngineError::new(
                EngineErrorCode::ModelSelfTestFailed,
                "no calibration document to probe with",
            )
        })
    }

    /// The request body for one document, in the provider's shape.
    pub(crate) fn request_body(&self, request: &ModelRequest) -> Value {
        match self.config.provider {
            HostedProvider::Anthropic => json!({
                "model": self.config.model,
                "max_tokens": MAX_REPLY_TOKENS,
                "system": request.system_turn(),
                "messages": [{"role": "user", "content": request.prompt}],
            }),
            HostedProvider::OpenAiCompatible => json!({
                "model": self.config.model,
                "messages": [
                    {"role": "system", "content": request.system_turn()},
                    {"role": "user", "content": request.prompt}
                ],
                "stream": false,
            }),
        }
    }

    fn propose_once(&self, request: &ModelRequest) -> Result<ModelProposal, HostedFailure> {
        let post = self.http.post(self.endpoint.clone());
        let post = match self.config.provider {
            HostedProvider::Anthropic => post
                .header("x-api-key", &self.config.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION),
            HostedProvider::OpenAiCompatible => post.bearer_auth(&self.config.api_key),
        };
        let response = post
            .json(&self.request_body(request))
            .send()
            .map_err(|_| AttemptError(EngineErrorCode::HostedModelUnreachable))?;
        let status = response.status();
        if !status.is_success() {
            let retry_after_secs = retry_after(
                status,
                response
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|value| value.to_str().ok()),
                SystemTime::now(),
            );
            let body = read_error_body(response);
            return Err(HostedFailure {
                code: failure_for_status(status, &body),
                retry_after_secs,
            });
        }
        let bytes = read_capped(response, EngineErrorCode::HostedModelUnreachable)?;
        Ok(match self.config.provider {
            HostedProvider::Anthropic => decode_anthropic(&bytes, request.evidence.as_ref())?,
            HostedProvider::OpenAiCompatible => {
                let completion: ChatCompletion = serde_json::from_slice(&bytes)
                    .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?;
                decode(completion, request.evidence.as_ref())?
            }
        })
    }
}

/// One attempt's failure, and how long the service asked to be left alone.
struct HostedFailure {
    code: EngineErrorCode,
    retry_after_secs: Option<u32>,
}

impl From<AttemptError> for HostedFailure {
    fn from(AttemptError(code): AttemptError) -> Self {
        Self {
            code,
            retry_after_secs: None,
        }
    }
}

impl HostedFailure {
    fn into_error(self) -> EngineError {
        hosted_error(self.code).with_retry_after(self.retry_after_secs)
    }
}

impl Proposer for HostedClient {
    /// One attempt, then one retry when the reply was malformed; a refused
    /// key or an unreachable service is not retried, because the second
    /// answer would be the same and the first is the one to report.
    fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal> {
        match self.propose_once(request) {
            Ok(proposal) => Ok(proposal),
            Err(failure) if failure.code == EngineErrorCode::ModelResponseInvalid => self
                .propose_once(request)
                .map_err(HostedFailure::into_error),
            Err(failure) => Err(failure.into_error()),
        }
    }
}

/// What an HTTP failure means to the person who has to fix it, from the
/// status and the start of the error body.
///
/// The body comes first where it is more specific than the status. A service
/// that has run out of credit says so in several ways - 402, Anthropic's
/// `billing_error` or its 400 about the credit balance, OpenAI's 429 with
/// `insufficient_quota` - and every one of them used to read as something
/// else: a "slower pace" that resuming never fixed, or a rejected request
/// that failed the backlog one paid request at a time.
pub(crate) fn failure_for_status(status: StatusCode, body: &[u8]) -> EngineErrorCode {
    let error = ErrorObject::read(body);
    if error.is_billing(status) {
        return EngineErrorCode::HostedModelBilling;
    }
    if status == StatusCode::BAD_REQUEST
        && (is_context_overflow(body) || error.code.as_deref() == Some("context_length_exceeded"))
    {
        // A local server behind the OpenAI shape with a smaller context than
        // the document: the engine condenses it and asks once more.
        return EngineErrorCode::ModelInputTooLarge;
    }
    match status.as_u16() {
        // Redirects are refused, so a 3xx reaches this point as an answer:
        // the configured address has moved. Reporting it as an outage sends
        // a person looking at their network for a wrong address.
        300..=399 => EngineErrorCode::HostedModelMisconfigured,
        401 | 403 => EngineErrorCode::HostedModelUnauthorized,
        // An unknown or retired model name, or an API root that is not one.
        // Every document in the backlog would meet the same answer, so this
        // pauses the queue once instead of failing them one at a time.
        404 => EngineErrorCode::HostedModelMisconfigured,
        429 => EngineErrorCode::HostedModelRateLimited,
        400..=499 => EngineErrorCode::HostedModelRejected,
        // Including Anthropic's 529, "overloaded": busy, not broken.
        _ => EngineErrorCode::HostedModelUnreachable,
    }
}

/// The `error` object Anthropic and OpenAI both wrap a failure in: what each
/// reads of it, and nothing when the body is not one.
#[derive(Default)]
struct ErrorObject {
    kind: Option<String>,
    code: Option<String>,
    message: Option<String>,
}

impl ErrorObject {
    fn read(body: &[u8]) -> Self {
        let Some(error) = serde_json::from_slice::<Value>(body)
            .ok()
            .and_then(|reply| reply.get("error").cloned())
        else {
            return Self::default();
        };
        let field = |name: &str| error.get(name).and_then(Value::as_str).map(str::to_owned);
        Self {
            kind: field("type"),
            code: field("code"),
            message: field("message"),
        }
    }

    fn is_billing(&self, status: StatusCode) -> bool {
        status == StatusCode::PAYMENT_REQUIRED
            || self.kind.as_deref() == Some("billing_error")
            || self.code.as_deref() == Some("insufficient_quota")
            || self.kind.as_deref() == Some("insufficient_quota")
            || (status == StatusCode::BAD_REQUEST
                && self
                    .message
                    .as_deref()
                    .is_some_and(|message| message.to_ascii_lowercase().contains("credit balance")))
    }
}

/// The wait a `Retry-After` header asks for, in seconds, on the statuses
/// where a wait means something: 429, 503, and Anthropic's 529.
pub(crate) fn retry_after(
    status: StatusCode,
    header: Option<&str>,
    now: SystemTime,
) -> Option<u32> {
    if !matches!(status.as_u16(), 429 | 503 | 529) {
        return None;
    }
    parse_retry_after(header?, now)
}

/// Reads `Retry-After` as either form RFC 9110 allows - delta-seconds or an
/// HTTP-date - capped at [`MAX_RETRY_AFTER_SECS`]. A date already past means
/// no wait at all; anything unreadable means the service did not say.
pub(crate) fn parse_retry_after(value: &str, now: SystemTime) -> Option<u32> {
    let value = value.trim();
    let seconds = match value.parse::<u64>() {
        Ok(seconds) => seconds,
        Err(_) => {
            let at = chrono::DateTime::parse_from_rfc2822(value)
                .ok()?
                .timestamp();
            let now = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())?;
            u64::try_from(at.saturating_sub(now)).unwrap_or(0)
        }
    };
    u32::try_from(seconds.min(MAX_RETRY_AFTER_SECS)).ok()
}

#[derive(Deserialize)]
struct AnthropicMessage {
    #[serde(default)]
    content: Vec<AnthropicBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

/// Reads a proposal out of a Messages API reply. Thinking blocks are passed
/// over; only the text blocks carry the answer. A refusal, and a reply cut
/// off at the token cap, are reported as what they are rather than as a
/// malformed reply, so neither is sent - and paid for - again.
pub(crate) fn decode_anthropic(
    bytes: &[u8],
    evidence: Option<&EvidenceHandles>,
) -> Result<ModelProposal, AttemptError> {
    let message: AnthropicMessage = serde_json::from_slice(bytes)
        .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    match message.stop_reason.as_deref() {
        Some("refusal") => return Err(AttemptError(EngineErrorCode::HostedModelRefused)),
        Some("max_tokens") => return Err(AttemptError(EngineErrorCode::ModelReplyTruncated)),
        _ => {}
    }
    let text = message
        .content
        .into_iter()
        .filter(|block| block.kind == "text")
        .filter_map(|block| block.text)
        .collect::<Vec<_>>()
        .join("");
    if text.trim().is_empty() {
        return Err(AttemptError(EngineErrorCode::ModelResponseInvalid));
    }
    proposal_from_text(&text, evidence)
}

const fn hosted_error(code: EngineErrorCode) -> EngineError {
    match code {
        EngineErrorCode::HostedModelUnauthorized => EngineError::new(
            EngineErrorCode::HostedModelUnauthorized,
            "the hosted service rejected the API key",
        ),
        EngineErrorCode::HostedModelRateLimited => EngineError::new(
            EngineErrorCode::HostedModelRateLimited,
            "the hosted service asked for a slower pace",
        ),
        EngineErrorCode::HostedModelRejected => EngineError::new(
            EngineErrorCode::HostedModelRejected,
            "the hosted service rejected the request",
        ),
        EngineErrorCode::HostedModelRefused => EngineError::new(
            EngineErrorCode::HostedModelRefused,
            "the hosted model declined to answer about this document",
        ),
        EngineErrorCode::HostedModelBilling => EngineError::new(
            EngineErrorCode::HostedModelBilling,
            "the hosted service refused the request for billing or quota reasons",
        ),
        EngineErrorCode::ModelInputTooLarge => EngineError::new(
            EngineErrorCode::ModelInputTooLarge,
            "the document does not fit the hosted model's context window",
        ),
        EngineErrorCode::ModelReplyTruncated => EngineError::new(
            EngineErrorCode::ModelReplyTruncated,
            "the hosted model ran out of room before finishing its answer",
        ),
        EngineErrorCode::HostedModelUnreachable => unreachable_error(),
        EngineErrorCode::HostedModelMisconfigured => {
            misconfigured("the hosted service does not know that address or model name")
        }
        _ => EngineError::new(
            EngineErrorCode::ModelResponseInvalid,
            "the hosted model returned malformed output twice",
        ),
    }
}

const fn misconfigured(message: &'static str) -> EngineError {
    EngineError::new(EngineErrorCode::HostedModelMisconfigured, message)
}

const fn unreachable_error() -> EngineError {
    EngineError::new(
        EngineErrorCode::HostedModelUnreachable,
        "the hosted service could not be reached",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::SYSTEM_INSTRUCTION;

    fn config(provider: HostedProvider, base_url: &str, model: &str) -> HostedModelConfig {
        HostedModelConfig {
            provider,
            base_url: base_url.into(),
            model: model.into(),
            api_key: "sk-test".into(),
        }
    }

    #[test]
    fn empty_fields_take_the_providers_defaults_and_a_missing_key_is_refused() {
        let resolved = config(HostedProvider::Anthropic, "", "")
            .resolved()
            .unwrap();
        assert_eq!(resolved.base_url, DEFAULT_ANTHROPIC_BASE_URL);
        assert_eq!(resolved.model, DEFAULT_ANTHROPIC_MODEL);

        let openai = config(HostedProvider::OpenAiCompatible, "", "gpt-x")
            .resolved()
            .unwrap();
        assert_eq!(openai.base_url, DEFAULT_OPENAI_BASE_URL);

        let no_model = config(HostedProvider::OpenAiCompatible, "", "  ").resolved();
        assert_eq!(
            no_model.unwrap_err().code(),
            EngineErrorCode::HostedModelMisconfigured
        );
        let mut no_key = config(HostedProvider::Anthropic, "", "");
        no_key.api_key = "   ".into();
        assert_eq!(
            no_key.resolved().unwrap_err().code(),
            EngineErrorCode::HostedModelMisconfigured
        );
    }

    #[test]
    fn endpoints_follow_the_provider_and_tolerate_a_trailing_slash() {
        assert_eq!(
            endpoint_for(HostedProvider::Anthropic, "https://api.anthropic.com/v1/")
                .unwrap()
                .as_str(),
            "https://api.anthropic.com/v1/messages"
        );
        assert_eq!(
            endpoint_for(
                HostedProvider::OpenAiCompatible,
                "https://api.openai.com/v1"
            )
            .unwrap()
            .as_str(),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint_for(
                HostedProvider::OpenAiCompatible,
                "https://gateway.example.com"
            )
            .unwrap()
            .as_str(),
            "https://gateway.example.com/chat/completions"
        );
    }

    #[test]
    fn plain_http_is_allowed_only_to_this_machine() {
        assert!(
            endpoint_for(
                HostedProvider::OpenAiCompatible,
                "http://localhost:11434/v1"
            )
            .is_ok()
        );
        assert!(endpoint_for(HostedProvider::OpenAiCompatible, "http://127.0.0.1:1234/v1").is_ok());
        assert!(endpoint_for(HostedProvider::OpenAiCompatible, "http://[::1]:1234/v1").is_ok());
        for bad in [
            "http://api.example.com/v1",
            "ftp://api.example.com/v1",
            "not a url",
            "",
        ] {
            assert_eq!(
                endpoint_for(HostedProvider::OpenAiCompatible, bad)
                    .unwrap_err()
                    .code(),
                EngineErrorCode::HostedModelMisconfigured,
                "{bad:?}"
            );
        }
    }

    /// The proxy bypass has to follow the same classification that allowed
    /// plain HTTP in the first place. A machine with `HTTP_PROXY` set and no
    /// loopback entry in its `NO_PROXY` list sent the key and the whole
    /// distilled document to the proxy in cleartext on the way to Ollama.
    #[test]
    fn loopback_endpoints_never_use_a_proxy() {
        for local in [
            "http://localhost:11434/v1",
            "http://127.0.0.1:1234/v1",
            "http://[::1]:1234/v1",
            "https://localhost:8443/v1",
        ] {
            let endpoint = endpoint_for(HostedProvider::OpenAiCompatible, local).unwrap();
            assert!(!uses_system_proxy(&endpoint), "{local}");
        }
        for remote in [
            "https://api.anthropic.com/v1",
            "https://api.openai.com/v1",
            // A name that merely starts with "localhost" is somebody else's.
            "https://localhost.example.com/v1",
        ] {
            let endpoint = endpoint_for(HostedProvider::OpenAiCompatible, remote).unwrap();
            assert!(uses_system_proxy(&endpoint), "{remote}");
        }
    }

    #[test]
    fn an_anthropic_request_is_a_messages_call_with_no_sampling_knobs() {
        let client = HostedClient::new(config(HostedProvider::Anthropic, "", "")).unwrap();
        let body = client.request_body(&ModelRequest::new("File this."));
        assert_eq!(body["model"], json!(DEFAULT_ANTHROPIC_MODEL));
        assert_eq!(body["max_tokens"], json!(MAX_REPLY_TOKENS));
        assert_eq!(body["system"], json!(SYSTEM_INSTRUCTION));
        assert_eq!(body["messages"][0]["role"], json!("user"));
        assert_eq!(body["messages"][0]["content"], json!("File this."));
        // Current Anthropic models reject temperature and top_k outright, and
        // the grammar is the local server's alone.
        for absent in ["temperature", "top_k", "grammar", "thinking", "stream"] {
            assert!(body.get(absent).is_none(), "{absent} must not be sent");
        }
        assert_eq!(
            client.endpoint().as_str(),
            "https://api.anthropic.com/v1/messages"
        );
    }

    #[test]
    fn an_openai_compatible_request_is_a_plain_chat_completion() {
        let client = HostedClient::new(config(
            HostedProvider::OpenAiCompatible,
            "http://localhost:1234/v1",
            "local-model",
        ))
        .unwrap();
        let body = client.request_body(&ModelRequest::new("File this."));
        assert_eq!(body["model"], json!("local-model"));
        assert_eq!(body["messages"][0]["role"], json!("system"));
        assert_eq!(body["messages"][1]["content"], json!("File this."));
        assert_eq!(body["stream"], json!(false));
        assert!(body.get("temperature").is_none());
        assert!(body.get("max_tokens").is_none(), "left to the server");
        assert_eq!(
            client.endpoint().as_str(),
            "http://localhost:1234/v1/chat/completions"
        );
    }

    #[test]
    fn an_anthropic_reply_is_read_from_its_text_blocks_only() {
        let reply = br#"{"content":[{"type":"thinking","thinking":"..."},{"type":"text","text":"```json\n{\"document_type\":\"Invoice\",\"document_date\":\"2026-03-02\",\"date_role\":\"invoice\",\"parties\":[\"Acme\"],\"party_relation\":\"from\",\"description\":\"An invoice.\",\"confidence\":0.9,\"needs_review\":false}\n```"}],"stop_reason":"end_turn"}"#;
        let proposal = decode_anthropic(reply, None).unwrap();
        assert_eq!(proposal.document_type.as_deref(), Some("Invoice"));
        assert_eq!(proposal.document_date.as_deref(), Some("2026-03-02"));
        assert_eq!(proposal.parties, vec!["Acme".to_string()]);
    }

    /// A hosted model has no grammar to keep it to the ids it was shown.
    /// An id it invents is dropped and counted, and is never evidence.
    #[test]
    fn a_hosted_evidence_reply_never_cites_an_id_it_was_not_shown() {
        let handles = crate::client::EvidenceHandles::new(vec![
            ("p1.b1".into(), "p1.b1".into()),
            ("p1.b2".into(), "p1.b2".into()),
        ]);
        let reply = br#"{"content":[{"type":"text","text":"{\"type_ids\":[\"p1.b1\",\"p4.b9\"],\"type\":\"Notice\",\"date_ids\":[\"p2.b1\"],\"date\":\"2024-01-02\",\"date_role\":\"notice\",\"parties\":[{\"ids\":[\"p1.b2\",\"p1.b2\",\"p3.b3\"],\"name\":\"Northstar Calibration Holdings LLC\",\"role\":\"addressee\"}],\"confidence\":0.9,\"needs_review\":false}"}],"stop_reason":"end_turn"}"#;
        let proposal = decode_anthropic(reply, Some(&handles)).unwrap();
        let facts = proposal.facts.unwrap();
        assert_eq!(facts.type_evidence, vec!["p1.b1"]);
        assert!(facts.date_evidence.is_empty());
        assert_eq!(facts.parties[0].evidence, vec!["p1.b2"]);
        assert_eq!(facts.unknown_evidence, vec!["p4.b9", "p2.b1", "p3.b3"]);
        assert!(
            facts.cited_ids().all(|id| handles.resolve(id).is_some()),
            "only shown ids are cited"
        );
    }

    #[test]
    fn a_refusal_and_a_truncated_reply_are_named_not_retried_as_malformed() {
        let refused = br#"{"content":[],"stop_reason":"refusal","stop_details":{"type":"refusal","category":"cyber"}}"#;
        assert_eq!(
            decode_anthropic(refused, None).unwrap_err().0,
            EngineErrorCode::HostedModelRefused
        );
        let truncated = br#"{"content":[{"type":"text","text":"{\"document_type\":"}],"stop_reason":"max_tokens"}"#;
        assert_eq!(
            decode_anthropic(truncated, None).unwrap_err().0,
            EngineErrorCode::ModelReplyTruncated
        );
        assert_eq!(
            decode_anthropic(b"not json", None).unwrap_err().0,
            EngineErrorCode::ModelResponseInvalid
        );
    }

    #[test]
    fn http_statuses_map_to_the_codes_a_person_can_act_on() {
        assert_eq!(
            failure_for_status(StatusCode::UNAUTHORIZED, b""),
            EngineErrorCode::HostedModelUnauthorized
        );
        assert_eq!(
            failure_for_status(StatusCode::FORBIDDEN, b""),
            EngineErrorCode::HostedModelUnauthorized
        );
        assert_eq!(
            failure_for_status(StatusCode::TOO_MANY_REQUESTS, b""),
            EngineErrorCode::HostedModelRateLimited
        );
        // An unknown or retired model name, or an API root that is not one.
        // Every document in the backlog would meet it, so it is a
        // configuration problem that pauses the queue once.
        assert_eq!(
            failure_for_status(StatusCode::NOT_FOUND, b""),
            EngineErrorCode::HostedModelMisconfigured
        );
        // Redirects are refused, so a base URL that has moved arrives here as
        // a 3xx. That is a wrong address, not a network outage.
        for moved in [301, 302, 303, 307, 308] {
            assert_eq!(
                failure_for_status(StatusCode::from_u16(moved).unwrap(), b""),
                EngineErrorCode::HostedModelMisconfigured,
                "{moved}"
            );
        }
        assert_eq!(
            failure_for_status(StatusCode::BAD_REQUEST, b""),
            EngineErrorCode::HostedModelRejected
        );
        assert_eq!(
            failure_for_status(StatusCode::INTERNAL_SERVER_ERROR, b""),
            EngineErrorCode::HostedModelUnreachable
        );
        assert_eq!(
            failure_for_status(StatusCode::from_u16(529).unwrap(), b""),
            EngineErrorCode::HostedModelUnreachable
        );
    }

    fn status(code: u16) -> StatusCode {
        StatusCode::from_u16(code).unwrap()
    }

    /// Every way a service says the account is out of credit. Each one used
    /// to read as something a person could not act on: OpenAI's quota as a
    /// "slower pace" that resuming never fixed, Anthropic's balance as a
    /// rejected request that failed the backlog one paid request at a time.
    #[test]
    fn billing_shapes_map_to_billing() {
        let shapes: [(u16, &[u8]); 5] = [
            (402, b""),
            (
                402,
                br#"{"type":"error","error":{"type":"billing_error","message":"Your account has no credit."}}"#,
            ),
            (
                400,
                br#"{"type":"error","error":{"type":"invalid_request_error","message":"Your credit balance is too low to access the Anthropic API. Please go to Plans & Billing to upgrade or purchase credits."}}"#,
            ),
            (
                429,
                br#"{"error":{"message":"You exceeded your current quota, please check your plan and billing details.","type":"insufficient_quota","param":null,"code":"insufficient_quota"}}"#,
            ),
            (
                403,
                br#"{"type":"error","error":{"type":"billing_error","message":"Billing is not set up."}}"#,
            ),
        ];
        for (code, body) in shapes {
            assert_eq!(
                failure_for_status(status(code), body),
                EngineErrorCode::HostedModelBilling,
                "{code} {}",
                String::from_utf8_lossy(body)
            );
        }
        // A rate limit that is only a rate limit stays one, and "credit" in
        // some other 4xx is not a balance.
        assert_eq!(
            failure_for_status(
                status(429),
                br#"{"error":{"message":"Rate limit reached for requests","type":"requests","code":"rate_limit_exceeded"}}"#
            ),
            EngineErrorCode::HostedModelRateLimited
        );
        assert_eq!(
            failure_for_status(
                status(401),
                br#"{"error":{"message":"Your credit balance is fine but this key is revoked"}}"#
            ),
            EngineErrorCode::HostedModelUnauthorized
        );
    }

    /// A server behind the OpenAI shape with a smaller context than the
    /// document - llama.cpp's own words, or OpenAI's code - is told apart
    /// from a rejected request, so the engine can condense and ask again.
    #[test]
    fn a_context_overflow_is_an_input_too_large() {
        for body in [
            &br#"{"error":{"code":400,"message":"the request exceeds the available context size, try increasing it","type":"exceed_context_size_error"}}"#[..],
            br#"{"error":{"message":"This model's maximum context length is 8192 tokens.","type":"invalid_request_error","param":"messages","code":"context_length_exceeded"}}"#,
        ] {
            assert_eq!(
                failure_for_status(StatusCode::BAD_REQUEST, body),
                EngineErrorCode::ModelInputTooLarge
            );
        }
    }

    #[test]
    fn retry_after_seconds_and_http_date_parsed_and_capped() {
        // Thursday, 1 October 2026, 12:00:00 UTC.
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_856_000);
        assert_eq!(parse_retry_after("7", now), Some(7));
        assert_eq!(parse_retry_after(" 0 ", now), Some(0));
        assert_eq!(parse_retry_after("3600", now), Some(120), "capped");
        assert_eq!(
            parse_retry_after("Thu, 01 Oct 2026 12:00:30 GMT", now),
            Some(30)
        );
        assert_eq!(
            parse_retry_after("Thu, 01 Oct 2026 13:00:00 GMT", now),
            Some(120),
            "capped"
        );
        assert_eq!(
            parse_retry_after("Thu, 01 Oct 2026 11:59:00 GMT", now),
            Some(0),
            "already past"
        );
        for unreadable in ["", "soon", "-5", "1.5"] {
            assert_eq!(parse_retry_after(unreadable, now), None, "{unreadable:?}");
        }
        // Only where a wait means something.
        assert_eq!(retry_after(status(429), Some("7"), now), Some(7));
        assert_eq!(retry_after(status(503), Some("7"), now), Some(7));
        assert_eq!(retry_after(status(529), Some("7"), now), Some(7));
        assert_eq!(retry_after(status(500), Some("7"), now), None);
        assert_eq!(retry_after(status(429), None, now), None);
    }

    /// Anthropic's "overloaded" is a busy service, not a broken one.
    #[test]
    fn status_529_is_unreachable() {
        assert_eq!(
            failure_for_status(
                status(529),
                br#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#
            ),
            EngineErrorCode::HostedModelUnreachable
        );
    }

    #[test]
    fn request_timeout_for_loopback_is_four_hundred_seconds() {
        for local in [
            "http://localhost:11434/v1",
            "http://127.0.0.1:1234/v1",
            "http://[::1]:8080/v1",
        ] {
            assert_eq!(
                request_timeout_for(&Url::parse(local).unwrap()),
                Duration::from_secs(400),
                "{local}"
            );
        }
        for remote in [
            "https://api.anthropic.com/v1",
            "https://api.openai.com/v1",
            "https://localhost.example.com/v1",
        ] {
            assert_eq!(
                request_timeout_for(&Url::parse(remote).unwrap()),
                Duration::from_secs(180),
                "{remote}"
            );
        }
    }

    fn local_openai(address: std::net::SocketAddr) -> HostedClient {
        HostedClient::new(config(
            HostedProvider::OpenAiCompatible,
            &format!("http://{address}/v1"),
            "local-model",
        ))
        .unwrap()
    }

    /// The engine condenses prompts to the local server's context only. A
    /// hosted model - even one on this machine, whose context Intern does not
    /// set - is sent each document as distilled, and condensed only when it
    /// answers that the prompt did not fit.
    #[test]
    fn a_hosted_model_is_not_held_to_the_local_context() {
        let local = local_openai("127.0.0.1:9".parse().unwrap());
        assert_eq!(Proposer::context_tokens(&local), None);
        let remote = HostedClient::new(config(
            HostedProvider::Anthropic,
            DEFAULT_ANTHROPIC_BASE_URL,
            DEFAULT_ANTHROPIC_MODEL,
        ))
        .unwrap();
        assert_eq!(Proposer::context_tokens(&remote), None);
    }

    /// The whole path, against a service that answers 429: the wait it named
    /// rides out on the error for the queue to honour, and a quota that has
    /// run out is a billing failure, sent once.
    #[test]
    fn a_rate_limit_carries_its_wait_and_an_exhausted_quota_is_billing() {
        use crate::test_support::{http_reply, scripted_server};

        let server = scripted_server(vec![http_reply(
            "429 Too Many Requests",
            &[("Retry-After", "7"), ("Content-Type", "application/json")],
            r#"{"error":{"message":"Rate limit reached","type":"requests","code":"rate_limit_exceeded"}}"#,
        )]);
        let error = local_openai(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap_err();
        assert_eq!(error.code(), EngineErrorCode::HostedModelRateLimited);
        assert_eq!(error.retry_after(), Some(Duration::from_secs(7)));
        assert_eq!(server.attempts(), 1);

        let server = scripted_server(vec![http_reply(
            "429 Too Many Requests",
            &[("Content-Type", "application/json")],
            r#"{"error":{"message":"You exceeded your current quota.","type":"insufficient_quota","param":null,"code":"insufficient_quota"}}"#,
        )]);
        let error = local_openai(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap_err();
        assert_eq!(error.code(), EngineErrorCode::HostedModelBilling);
        assert_eq!(error.retry_after(), None);
        assert_eq!(server.attempts(), 1);
    }

    /// An Azure content filter, and a reply cut off at the token cap, are
    /// each one billed request - not four, and not a paused queue.
    #[test]
    fn a_filtered_or_truncated_reply_is_sent_once() {
        use crate::test_support::{completion_reply, scripted_server};

        for (reason, code) in [
            ("content_filter", EngineErrorCode::HostedModelRefused),
            ("length", EngineErrorCode::ModelReplyTruncated),
        ] {
            let server = scripted_server(vec![completion_reply(reason, "{\"type_evidence\":")]);
            let error = local_openai(server.address)
                .propose(&ModelRequest::new("p"))
                .unwrap_err();
            assert_eq!(error.code(), code, "{reason}");
            assert_eq!(server.attempts(), 1, "{reason}");
        }
    }

    #[test]
    fn the_client_never_prints_its_key() {
        let client = HostedClient::new(config(HostedProvider::Anthropic, "", "")).unwrap();
        let debug = format!("{client:?}");
        assert!(!debug.contains("sk-test"));
        assert!(debug.contains("[redacted]"));

        // Nor does the configuration it was built from: it is what the desktop
        // app holds on to, and anything that prints it - a log line, a panic
        // message - would otherwise carry the key with it.
        let settings = format!("{:?}", config(HostedProvider::Anthropic, "", ""));
        assert!(!settings.contains("sk-test"), "{settings}");
        assert!(settings.contains("[redacted]"));
    }
}
