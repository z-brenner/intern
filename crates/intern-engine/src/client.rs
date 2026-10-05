//! HTTP client for the local llama.cpp server.
//!
//! The endpoint is required to be loopback HTTP with no proxy and no redirects,
//! so a misconfiguration cannot silently send document text off the machine.
//! Sending it off the machine on purpose is [`crate::hosted`]'s job, behind
//! an explicit setting.

use std::time::Duration;

use reqwest::{Url, blocking::Client};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::domain::{DateRole, Evidence, ModelProposal, PartyRelation, TokenConfidence};
use crate::error::{EngineError, EngineErrorCode, EngineResult};
use crate::evidence::is_valid_iso_date;
use crate::prompt::{RESPONSE_GRAMMAR, SYSTEM_INSTRUCTION, build_prompt};

/// Room for the reply plus the grammar's fixed scaffolding. The model is not
/// writing prose, so this stays small and generation stays fast.
const MAX_REPLY_TOKENS: u32 = 420;

/// How much of a reply body is worth reading. A reply is a short JSON object;
/// even a thinking model's whole visible answer is a few tens of kilobytes.
/// Two megabytes is far above anything real and far below anything that would
/// hurt a laptop already running the model.
const MAX_REPLY_BYTES: u64 = 2 * 1024 * 1024;

/// What actually gets sent to the model for one document.
///
/// Text only, by construction. Intern reads documents as text and the local
/// server runs without a vision projector, so there is no field here that could
/// ask it for something it cannot do.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequest {
    pub prompt: String,
}

impl ModelRequest {
    pub fn from_digest(digest: &crate::distill::DocumentDigest) -> Self {
        Self {
            prompt: build_prompt(digest),
        }
    }

    /// A stable identity for this exact input, for evaluation records.
    pub fn sha256(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Anything that can turn the prompt for one document into a proposal: the
/// local server, or a hosted model standing in for it.
pub trait Proposer: Send + Sync {
    fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal>;

    /// The proposal, and how probable the model found its own date and party
    /// tokens when it reported that. Most models do not, so by default this
    /// is the proposal alone.
    fn propose_scored(
        &self,
        request: &ModelRequest,
    ) -> EngineResult<(ModelProposal, Option<TokenConfidence>)> {
        self.propose(request).map(|proposal| (proposal, None))
    }
}

pub struct ModelClient {
    endpoint: Url,
    api_key: String,
    model_id: String,
    http: Client,
    token_confidence: bool,
}

impl Proposer for ModelClient {
    fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal> {
        ModelClient::propose(self, request)
    }

    fn propose_scored(
        &self,
        request: &ModelRequest,
    ) -> EngineResult<(ModelProposal, Option<TokenConfidence>)> {
        ModelClient::propose_scored(self, request)
    }
}

impl ModelClient {
    pub fn new(
        endpoint: &str,
        api_key: impl Into<String>,
        model_id: impl Into<String>,
    ) -> EngineResult<Self> {
        let endpoint = Url::parse(endpoint).map_err(|_| request_failed())?;
        let loopback = endpoint
            .host_str()
            .and_then(|host| host.parse::<std::net::IpAddr>().ok())
            .is_some_and(|address| address.is_loopback());
        if endpoint.scheme() != "http" || !loopback {
            return Err(EngineError::new(
                EngineErrorCode::ModelRequestFailed,
                "model endpoint must be local HTTP",
            ));
        }
        // Plain HTTP to this machine never consults a root certificate, so
        // the operating system's store is not read: one with nothing usable
        // in it fails the build, and must not stop the local model.
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10 * 60))
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .tls_built_in_native_certs(false)
            .build()
            .map_err(|_| request_failed())?;
        Ok(Self {
            endpoint,
            api_key: api_key.into(),
            model_id: model_id.into(),
            http,
            token_confidence: false,
        })
    }

    /// Asks the server for the probability of every token it generates, so
    /// a proposal can carry a [`TokenConfidence`].
    ///
    /// Off by default. It changes no generated token - decoding is greedy
    /// either way - and nothing that identifies a request in a recording,
    /// but the server pays a softmax over the whole vocabulary per token for
    /// it, which is measurable on a CPU (see docs/model-candidates.md).
    pub fn with_token_confidence(mut self, enabled: bool) -> Self {
        self.token_confidence = enabled;
        self
    }

    /// One attempt, then one retry when the reply was malformed. A request
    /// that failed outright is not retried: the second attempt fails the same
    /// way, and against a server that has died or hung it turns one document
    /// into two full request timeouts before anyone is told.
    pub fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal> {
        self.propose_scored(request).map(|(proposal, _)| proposal)
    }

    /// [`Self::propose`], with the reply's [`TokenConfidence`] when the
    /// client asks for token probabilities.
    pub fn propose_scored(
        &self,
        request: &ModelRequest,
    ) -> EngineResult<(ModelProposal, Option<TokenConfidence>)> {
        match self.propose_once(request) {
            Ok(scored) => Ok(scored),
            Err(AttemptError(EngineErrorCode::ModelResponseInvalid)) => {
                self.propose_once(request).map_err(AttemptError::into_error)
            }
            Err(error) => Err(error.into_error()),
        }
    }

    pub(crate) fn propose_once(
        &self,
        request: &ModelRequest,
    ) -> Result<(ModelProposal, Option<TokenConfidence>), AttemptError> {
        let response = self
            .http
            .post(self.endpoint.clone())
            .bearer_auth(&self.api_key)
            .json(&self.completion_request(request))
            .send()
            .map_err(|_| AttemptError(EngineErrorCode::ModelRequestFailed))?;
        if !response.status().is_success() {
            return Err(AttemptError(EngineErrorCode::ModelRequestFailed));
        }
        let bytes = read_capped(response, EngineErrorCode::ModelResponseInvalid)?;
        let completion: ChatCompletion = serde_json::from_slice(&bytes)
            .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?;
        let confidence = completion.token_confidence();
        decode(completion).map(|proposal| (proposal, confidence))
    }

    /// The request body. Only the user turn identifies a request
    /// ([`ModelRequest::sha256`]); everything here is transport, and the
    /// token-probability fields in particular change no generated token.
    fn completion_request(&self, request: &ModelRequest) -> Value {
        let mut body = json!({
            "model": self.model_id,
            "messages": [
                {"role": "system", "content": SYSTEM_INSTRUCTION},
                {"role": "user", "content": request.prompt}
            ],
            "stream": false,
            "temperature": 0,
            "top_k": 1,
            "max_tokens": MAX_REPLY_TOKENS,
            "grammar": RESPONSE_GRAMMAR,
            "cache_prompt": true,
            // Hybrid-reasoning models must answer directly: Intern needs a form
            // filled in, not a chain of thought, and thinking tokens are pure
            // latency on a CPU.
            "chat_template_kwargs": {"enable_thinking": false}
        });
        if self.token_confidence {
            // One alternative is the least llama-server accepts; only the
            // chosen token's own probability is read.
            body["logprobs"] = json!(true);
            body["top_logprobs"] = json!(1);
        }
        body
    }
}

/// How probable the model found the tokens of its `document_date` and
/// `parties` values, from the per-token probabilities a server reported.
///
/// The grammar forces every key, quote, and bracket, and the model's raw
/// probability for a forced token says nothing about the answer, so only
/// tokens overlapping a value's own characters count: the inside of the date
/// string or a party name, or a bare `null` or `[]`.
pub(crate) fn token_confidence(tokens: &[TokenLogprob]) -> Option<TokenConfidence> {
    let mut text = Vec::new();
    let mut ranges = Vec::with_capacity(tokens.len());
    for token in tokens {
        let start = text.len();
        match &token.bytes {
            Some(bytes) => text.extend_from_slice(bytes),
            None => text.extend_from_slice(token.token.as_bytes()),
        }
        ranges.push((start, text.len()));
    }
    let mut spans = Vec::new();
    for key in [&b"\"document_date\":"[..], &b"\"parties\":"[..]] {
        if let Some(at) = find(&text, key) {
            value_spans(&text, at + key.len(), &mut spans);
        }
    }
    let mut count = 0_u32;
    let mut sum = 0.0_f64;
    let mut min = 1.0_f64;
    for (token, (start, end)) in tokens.iter().zip(ranges) {
        if spans.iter().any(|&(a, b)| start < b && a < end) {
            let probability = token.logprob.exp().clamp(0.0, 1.0);
            count += 1;
            sum += probability;
            min = min.min(probability);
        }
    }
    (count > 0).then(|| TokenConfidence {
        min: min as f32,
        mean: (sum / f64::from(count)) as f32,
        tokens: count,
    })
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// The byte spans of the JSON value starting at `at`: each string's inside,
/// or the bare literal (`null`, `[]`) when there is no string.
fn value_spans(text: &[u8], at: usize, spans: &mut Vec<(usize, usize)>) {
    let mut index = at;
    match text.get(index) {
        Some(b'"') => {
            string_span(text, index, spans);
        }
        Some(b'[') => {
            index += 1;
            if text.get(index) == Some(&b']') {
                spans.push((at, index + 1));
                return;
            }
            while text.get(index) == Some(&b'"') {
                index = string_span(text, index, spans);
                if text.get(index) != Some(&b',') {
                    break;
                }
                index += 1;
            }
        }
        Some(_) => {
            let end = text[at..]
                .iter()
                .position(|byte| matches!(byte, b',' | b'}' | b']'))
                .map_or(text.len(), |offset| at + offset);
            spans.push((at, end));
        }
        None => {}
    }
}

/// Records the inside of the string opening at `quote` and returns the index
/// just past its closing quote.
fn string_span(text: &[u8], quote: usize, spans: &mut Vec<(usize, usize)>) -> usize {
    let start = quote + 1;
    let mut index = start;
    while index < text.len() {
        match text[index] {
            b'\\' => index += 2,
            b'"' => {
                spans.push((start, index));
                return index + 1;
            }
            _ => index += 1,
        }
    }
    spans.push((start, text.len()));
    text.len()
}

/// Reads a reply body, refusing one that could not be a reply.
///
/// Whatever is at the other end of the socket decides how many bytes arrive,
/// and a proxy's error page or a stream nobody asked for should not be read
/// into memory without a limit. A body over the cap is a malformed reply;
/// `on_io_failure` is what a body that simply stopped arriving means to the
/// caller, which differs between the local server and a hosted service.
pub(crate) fn read_capped(
    body: impl std::io::Read,
    on_io_failure: EngineErrorCode,
) -> Result<Vec<u8>, AttemptError> {
    use std::io::Read as _;

    let mut bytes = Vec::new();
    body.take(MAX_REPLY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AttemptError(on_io_failure))?;
    if bytes.len() as u64 > MAX_REPLY_BYTES {
        return Err(AttemptError(EngineErrorCode::ModelResponseInvalid));
    }
    Ok(bytes)
}

/// Reads a proposal out of a chat-completion reply: the local server's, or
/// any OpenAI-compatible service's.
pub(crate) fn decode(completion: ChatCompletion) -> Result<ModelProposal, AttemptError> {
    let choice = completion
        .choices
        .into_iter()
        .next()
        .ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    if !matches!(choice.finish_reason.as_deref(), Some("stop") | None) {
        return Err(AttemptError(EngineErrorCode::ModelResponseInvalid));
    }
    let content = choice
        .message
        .content
        .into_text()
        .ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    proposal_from_text(&content)
}

/// Reads a proposal out of the text a model replied with, fences and
/// chatter tolerated.
pub(crate) fn proposal_from_text(content: &str) -> Result<ModelProposal, AttemptError> {
    let json =
        extract_json_object(content).ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    let wire: WireProposal = serde_json::from_str(json)
        .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    wire.into_domain()
        .ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))
}

/// Recovers the JSON object from a reply that may be fenced or prefixed.
pub fn extract_json_object(content: &str) -> Option<&str> {
    let trimmed = content.trim();
    // A fence is bounded by its own closing marker, not by the end of the
    // reply: a hosted model that fences the object and then adds a sentence
    // has still answered, and requiring the fence to be the last thing in the
    // reply threw that answer away twice and paused the queue. Everything
    // outside the fence is chatter, so its braces never enter the scan.
    let body = match trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
    {
        Some(fenced) => match fenced.find("```") {
            Some(close) => &fenced[..close],
            None => fenced,
        },
        None => trimmed,
    };
    let start = body.find('{')?;
    let end = body.rfind('}')?;
    (end > start).then(|| body[start..=end].trim())
}

#[derive(Deserialize)]
pub(crate) struct ChatCompletion {
    choices: Vec<Choice>,
}

impl ChatCompletion {
    /// The first choice's [`TokenConfidence`], when the server reported
    /// token probabilities in a shape this can read. A shape it cannot read
    /// costs the signal, never the reply.
    pub(crate) fn token_confidence(&self) -> Option<TokenConfidence> {
        let logprobs = self.choices.first()?.logprobs.as_ref()?;
        let tokens: Vec<TokenLogprob> =
            serde_json::from_value(logprobs.get("content")?.clone()).ok()?;
        token_confidence(&tokens)
    }
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
    finish_reason: Option<String>,
    /// Kept unparsed: see [`ChatCompletion::token_confidence`].
    #[serde(default)]
    logprobs: Option<Value>,
}

/// One generated token and the model's log-probability for it.
#[derive(Deserialize)]
pub(crate) struct TokenLogprob {
    token: String,
    /// The token's exact bytes; `token` alone can be lossy where a token
    /// splits a multi-byte character.
    #[serde(default)]
    bytes: Option<Vec<u8>>,
    logprob: f64,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: AssistantContent,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AssistantContent {
    Text(String),
    Parts(Vec<AssistantContentPart>),
}

impl AssistantContent {
    fn into_text(self) -> Option<String> {
        match self {
            Self::Text(text) => Some(text),
            Self::Parts(parts) => {
                let mut text = String::new();
                for part in parts {
                    if part.kind == "text" {
                        text.push_str(part.text.as_deref()?);
                    }
                }
                (!text.is_empty()).then_some(text)
            }
        }
    }
}

#[derive(Deserialize)]
struct AssistantContentPart {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct WireProposal {
    #[serde(default)]
    type_evidence: Option<String>,
    #[serde(default)]
    document_type: Option<String>,
    #[serde(default)]
    date_evidence: Option<String>,
    #[serde(default)]
    document_date: Option<String>,
    #[serde(default)]
    date_role: Option<DateRole>,
    #[serde(default)]
    party_evidence: Vec<String>,
    #[serde(default)]
    parties: Vec<String>,
    #[serde(default)]
    party_relation: Option<PartyRelation>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    confidence: f32,
    #[serde(default)]
    needs_review: bool,
}

impl WireProposal {
    fn into_domain(mut self) -> Option<ModelProposal> {
        if !self.confidence.is_finite() || !(0.0..=1.0).contains(&self.confidence) {
            return None;
        }
        if self
            .document_date
            .as_deref()
            .map(str::trim)
            .is_some_and(|date| !date.is_empty() && !is_valid_iso_date(date))
        {
            // A shape-valid but non-calendar date is a hard reply failure; the
            // grammar guarantees the shape, so this is the model inventing a
            // day that does not exist.
            self.document_date = None;
            self.date_role = None;
        }
        self.parties.truncate(3);
        self.party_evidence.truncate(3);
        Some(ModelProposal {
            document_type: self.document_type,
            document_date: self.document_date,
            date_role: self.date_role,
            parties: self.parties,
            party_relation: self.party_relation.unwrap_or(PartyRelation::None),
            description: self.description,
            confidence: self.confidence,
            needs_review: self.needs_review,
            evidence: Evidence {
                date: self.date_evidence,
                document_type: self.type_evidence,
                parties: self.party_evidence,
            },
        })
    }
}

#[derive(Debug)]
pub(crate) struct AttemptError(pub(crate) EngineErrorCode);

impl AttemptError {
    fn into_error(self) -> EngineError {
        match self.0 {
            EngineErrorCode::ModelRequestFailed => request_failed(),
            _ => EngineError::new(
                EngineErrorCode::ModelResponseInvalid,
                "local model returned malformed output twice",
            ),
        }
    }
}

const fn request_failed() -> EngineError {
    EngineError::new(
        EngineErrorCode::ModelRequestFailed,
        "local model request failed",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_loopback_endpoint_is_refused() {
        assert!(ModelClient::new("http://example.com/v1/chat/completions", "k", "m").is_err());
        assert!(ModelClient::new("https://127.0.0.1/v1/chat/completions", "k", "m").is_err());
        assert!(ModelClient::new("http://127.0.0.1:8080/v1/chat/completions", "k", "m").is_ok());
    }

    #[test]
    fn json_is_recovered_from_fences_and_from_surrounding_chatter() {
        assert_eq!(
            extract_json_object("```json\n{\"a\":1}\n```"),
            Some("{\"a\":1}")
        );
        assert_eq!(
            extract_json_object("Sure!\n{\"a\":1}\nDone"),
            Some("{\"a\":1}")
        );
        assert_eq!(extract_json_object("no object here"), None);
    }

    /// Hosted models fence the object and then add a closing sentence. That
    /// reply was read as malformed, retried, read as malformed again, and the
    /// queue paused - over a reply that contained exactly what was asked for.
    #[test]
    fn json_is_recovered_from_a_fence_followed_by_chatter() {
        assert_eq!(
            extract_json_object("```json\n{\"a\":1}\n```\nLet me know if you need anything else."),
            Some("{\"a\":1}")
        );
        assert_eq!(
            extract_json_object("Here you go:\n```\n{\"a\":1}\n```\nHope that helps!"),
            Some("{\"a\":1}")
        );
        // A fence the model never closed still carries the object.
        assert_eq!(extract_json_object("```json\n{\"a\":1}"), Some("{\"a\":1}"));
        // Prose after the fence must not be scanned for braces of its own.
        assert_eq!(
            extract_json_object("```json\n{\"a\":1}\n```\nNote the {braces} above."),
            Some("{\"a\":1}")
        );
    }

    #[test]
    fn a_reply_missing_optional_fields_still_decodes() {
        let wire: WireProposal =
            serde_json::from_str(r#"{"description":"A document.","confidence":0.7}"#).unwrap();
        let proposal = wire.into_domain().unwrap();
        assert_eq!(proposal.party_relation, PartyRelation::None);
        assert!(proposal.document_date.is_none());
    }

    #[test]
    fn an_impossible_calendar_date_is_discarded_rather_than_trusted() {
        let wire: WireProposal = serde_json::from_str(
            r#"{"document_date":"2026-02-31","date_role":"effective","description":"x","confidence":0.9}"#,
        )
        .unwrap();
        let proposal = wire.into_domain().unwrap();
        assert!(proposal.document_date.is_none());
        assert!(proposal.date_role.is_none());
    }

    #[test]
    fn the_request_carries_the_grammar_and_no_thinking() {
        let client = ModelClient::new("http://127.0.0.1:9/v1/chat/completions", "k", "m").unwrap();
        let body = client.completion_request(&ModelRequest { prompt: "p".into() });
        assert_eq!(body["grammar"], serde_json::json!(RESPONSE_GRAMMAR));
        assert_eq!(body["temperature"], serde_json::json!(0));
        assert_eq!(
            body["chat_template_kwargs"]["enable_thinking"],
            serde_json::json!(false)
        );
    }

    /// The local server runs without a vision projector. A request that carried
    /// an image would be rejected by it, so the request must always be plain
    /// text - never a multimodal content array.
    #[test]
    fn every_request_is_plain_text() {
        let client = ModelClient::new("http://127.0.0.1:9/v1/chat/completions", "k", "m").unwrap();
        let body = client.completion_request(&ModelRequest { prompt: "p".into() });
        assert!(body["messages"][1]["content"].is_string());
        assert!(!body.to_string().contains("image_url"));
    }

    /// A retry is for a reply that came back malformed. A request that failed
    /// outright fails the same way twice, and against a dead or hung server
    /// the second attempt only doubles a ten-minute wait for one document.
    #[test]
    fn a_failed_request_is_not_retried() {
        use std::{
            io::{BufRead, BufReader, Write},
            net::TcpListener,
            sync::{
                Arc,
                atomic::{AtomicUsize, Ordering},
            },
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let attempts = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&attempts);
        // Never joined: after the fix there is no second connection to accept,
        // and the harness ends the process when the last test finishes.
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
                counted.fetch_add(1, Ordering::SeqCst);
                // Drain the request so closing the socket cannot reset it
                // before the status line arrives.
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut length = 0_usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() || line == "\r\n" || line.is_empty() {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap_or(0);
                    }
                }
                let _ = std::io::Read::read_exact(&mut reader, &mut vec![0_u8; length]);
                let _ = stream.write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
                let _ = stream.flush();
            }
        });

        let client =
            ModelClient::new(&format!("http://{address}/v1/chat/completions"), "k", "m").unwrap();
        let error = client
            .propose(&ModelRequest { prompt: "p".into() })
            .unwrap_err();

        assert_eq!(error.code(), EngineErrorCode::ModelRequestFailed);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    /// Whatever answers the socket decides how many bytes arrive, and the
    /// machine on the other end of a hosted endpoint is not Intern's.
    #[test]
    fn a_reply_body_is_read_only_up_to_a_cap() {
        assert_eq!(
            read_capped(
                &b"{\"choices\":[]}"[..],
                EngineErrorCode::ModelRequestFailed
            )
            .unwrap(),
            b"{\"choices\":[]}"
        );
        let flood = std::io::Read::take(std::io::repeat(b'{'), 8 * 1024 * 1024);
        assert_eq!(
            read_capped(flood, EngineErrorCode::ModelRequestFailed)
                .unwrap_err()
                .0,
            EngineErrorCode::ModelResponseInvalid
        );
    }

    /// Token probabilities cost the server a softmax per generated token, so
    /// they are asked for only when wanted - and asking changes nothing else
    /// about the request, least of all what identifies it in a recording.
    #[test]
    fn token_probabilities_are_requested_only_when_enabled() {
        let request = ModelRequest { prompt: "p".into() };
        let plain = ModelClient::new("http://127.0.0.1:9/v1/chat/completions", "k", "m").unwrap();
        let body = plain.completion_request(&request);
        assert!(body.get("logprobs").is_none());
        assert!(body.get("top_logprobs").is_none());

        let scored = ModelClient::new("http://127.0.0.1:9/v1/chat/completions", "k", "m")
            .unwrap()
            .with_token_confidence(true);
        let mut scored_body = scored.completion_request(&request);
        assert_eq!(scored_body["logprobs"], serde_json::json!(true));
        assert_eq!(scored_body["top_logprobs"], serde_json::json!(1));
        let fields = scored_body.as_object_mut().unwrap();
        fields.remove("logprobs");
        fields.remove("top_logprobs");
        assert_eq!(scored_body, body);
    }

    fn pieces(parts: &[(&str, f64)]) -> Vec<TokenLogprob> {
        parts
            .iter()
            .map(|(text, probability)| TokenLogprob {
                token: (*text).to_owned(),
                bytes: Some(text.as_bytes().to_vec()),
                logprob: probability.ln(),
            })
            .collect()
    }

    fn close(actual: f32, expected: f32) -> bool {
        (actual - expected).abs() < 1e-4
    }

    /// The grammar forces the scaffolding, and the model's raw probability for
    /// a forced token says nothing about the answer - the server reported
    /// e^-14 for a quote mark the grammar required. Only the characters of the
    /// date and party values count.
    #[test]
    fn token_confidence_reads_only_the_date_and_party_values() {
        let tokens = pieces(&[
            (
                r#"{"type_evidence":"INVOICE","document_type":"Invoice","date_evidence":"Invoice date: May 1, 2025","document_date":""#,
                0.01,
            ),
            ("2025-05", 0.8),
            ("-01", 0.6),
            (r#"","date_role":"invoice","parties":[""#, 0.02),
            ("Acme", 0.9),
            (" Corp", 0.5),
            (
                r#""],"party_evidence":["Acme Corp"],"party_relation":"from","description":"An invoice.","confidence":0.9,"needs_review":false}"#,
                0.03,
            ),
            ("", 0.001),
        ]);
        let confidence = token_confidence(&tokens).unwrap();
        assert_eq!(confidence.tokens, 4);
        assert!(close(confidence.min, 0.5), "{confidence:?}");
        assert!(close(confidence.mean, 0.7), "{confidence:?}");
    }

    /// Choosing to say nothing is a choice too: a null date and an empty party
    /// list are scored like any other value.
    #[test]
    fn a_null_date_and_an_empty_party_list_are_scored_as_choices() {
        let tokens = pieces(&[
            (
                r#"{"type_evidence":null,"document_type":"Memo","date_evidence":null,"document_date":"#,
                0.01,
            ),
            ("null", 0.4),
            (r#","date_role":null,"parties":"#, 0.01),
            ("[]", 0.7),
            (
                r#","party_evidence":[],"party_relation":"none","description":"A memo.","confidence":0.5,"needs_review":false}"#,
                0.01,
            ),
        ]);
        let confidence = token_confidence(&tokens).unwrap();
        assert_eq!(confidence.tokens, 2);
        assert!(close(confidence.min, 0.4), "{confidence:?}");
        assert!(close(confidence.mean, 0.55), "{confidence:?}");
    }

    /// A token that carries the end of the scaffolding and the start of a
    /// value is part of the value; an escaped quote inside a name does not
    /// end it, and every party counts.
    #[test]
    fn straddling_tokens_escapes_and_every_party_are_counted() {
        let tokens = pieces(&[
            (r#"{"document_date":"2026"#, 0.3),
            ("-03-03", 0.9),
            (r#"","parties":[""#, 0.01),
            (r#"O\"Hara"#, 0.6),
            (r#" Ltd",""#, 0.8),
            ("Acme", 0.7),
            (r#""],"party_evidence":[]}"#, 0.01),
        ]);
        let confidence = token_confidence(&tokens).unwrap();
        assert_eq!(confidence.tokens, 5);
        assert!(close(confidence.min, 0.3), "{confidence:?}");
        assert!(close(confidence.mean, 0.66), "{confidence:?}");
    }

    #[test]
    fn a_reply_without_token_probabilities_has_no_token_confidence() {
        let completion: ChatCompletion = serde_json::from_str(
            r#"{"choices":[{"message":{"content":"{}"},"finish_reason":"stop"}]}"#,
        )
        .unwrap();
        assert!(completion.token_confidence().is_none());
        assert!(token_confidence(&[]).is_none());
    }

    /// The server's own reply shape, trimmed from a live b10361 response.
    #[test]
    fn token_confidence_is_read_from_the_servers_reply() {
        let completion: ChatCompletion = serde_json::from_str(
            r#"{"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"{\"document_date\":\"2025-02-14\"}"},
               "logprobs":{"content":[
                 {"id":1,"token":"{\"document_date\":\"","bytes":[123,34,100,111,99,117,109,101,110,116,95,100,97,116,101,34,58,34],"logprob":-14.2,"top_logprobs":[]},
                 {"id":2,"token":"2025-02-14","bytes":[50,48,50,53,45,48,50,45,49,52],"logprob":-0.01,"top_logprobs":[]},
                 {"id":3,"token":"\"}","bytes":[34,125],"logprob":-0.4,"top_logprobs":[]}]}}]}"#,
        )
        .unwrap();
        let confidence = completion.token_confidence().unwrap();
        assert_eq!(confidence.tokens, 1);
        assert!(close(confidence.min, (-0.01_f32).exp()));
    }

    #[test]
    fn the_request_identity_follows_the_prompt() {
        assert_ne!(
            ModelRequest { prompt: "a".into() }.sha256(),
            ModelRequest { prompt: "b".into() }.sha256()
        );
    }
}
