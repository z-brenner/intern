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

use crate::domain::{
    DateRole, Evidence, KeyFact, ModelFacts, ModelProposal, ModelTimings, PartyFact, PartyRelation,
    PartyRole, TokenConfidence,
};
use crate::error::{EngineError, EngineErrorCode, EngineResult};
use crate::evidence::is_valid_iso_date;
use crate::prompt::{RESPONSE_GRAMMAR, SYSTEM_INSTRUCTION, build_prompt};

/// Room for the reply plus the grammar's fixed scaffolding.
///
/// A ceiling, not a target: the grammar's closing brace ends generation, and
/// the corpus's longest reply is about 200 tokens. It used to be 420, and a
/// contract that opens with one long paragraph naming the date and both
/// parties is quoted as evidence three times over - past 420 tokens, into a
/// reply cut off mid-string. Transport only: [`ModelRequest::sha256`] covers
/// the prompt and a grammar particular to the request, never this, so
/// recordings stay valid. An evidence-pipeline reply is capped by its
/// grammar far below it (see `the_compact_grammar_states_each_fact_with_one_id_and_nothing_else`).
pub(crate) const MAX_REPLY_TOKENS: u32 = 1_024;

/// How much of a reply body is worth reading. A reply is a short JSON object;
/// even a thinking model's whole visible answer is a few tens of kilobytes.
/// Two megabytes is far above anything real and far below anything that would
/// hurt a laptop already running the model.
const MAX_REPLY_BYTES: u64 = 2 * 1024 * 1024;

/// How much of an error reply is worth reading. Enough for any service's JSON
/// error object; the rest is somebody's HTML error page.
const MAX_ERROR_BYTES: u64 = 16 * 1024;

/// What actually gets sent to the model for one document.
///
/// Text only, by construction. Intern reads documents as text and the local
/// server runs without a vision projector, so there is no field here that could
/// ask it for something it cannot do.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModelRequest {
    pub prompt: String,
    /// The system turn, when it is particular to the request: the evidence
    /// pipeline's compact form carries its fixed instructions there, where
    /// the server's prompt cache keeps them from one document to the next.
    /// `None` is [`SYSTEM_INSTRUCTION`].
    pub system: Option<String>,
    /// The grammar this request's reply must follow, when it is particular to
    /// the request: the evidence pipeline's lists exactly the evidence ids
    /// its prompt shows. `None` is the digest pipeline's fixed
    /// [`RESPONSE_GRAMMAR`].
    pub grammar: Option<String>,
    /// The evidence handles the prompt shows, for reading an
    /// evidence-pipeline reply. `None` for the digest pipeline.
    pub evidence: Option<EvidenceHandles>,
}

impl ModelRequest {
    /// A digest-pipeline request: the prompt alone, answered under the fixed
    /// grammar.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ..Self::default()
        }
    }

    pub fn from_digest(digest: &crate::distill::DocumentDigest) -> Self {
        Self::new(build_prompt(digest))
    }

    /// A stable identity for this exact input, for evaluation records: the
    /// prompt, and the grammar when the request carries its own, so a
    /// recorded reply never answers a request whose grammar has changed. A
    /// request under the fixed grammar is identified by its prompt alone, as
    /// it always was, so every recording made before stays valid.
    ///
    /// A recorded reply's ids are already the stable ids its handles stood
    /// for. Ordinal handles do not show those ids in the prompt, so their
    /// mapping is part of the identity too: the same prompt over units whose
    /// ids changed is a different request. Stable handles are the ids
    /// themselves, already in the prompt, and add nothing.
    pub fn sha256(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.prompt.as_bytes());
        if let Some(grammar) = &self.grammar {
            hasher.update([0_u8]);
            hasher.update(grammar.as_bytes());
        }
        if let Some(system) = &self.system {
            hasher.update([0_u8, 1_u8]);
            hasher.update(system.as_bytes());
        }
        if let Some(evidence) = self.evidence.as_ref().filter(|e| e.remaps()) {
            hasher.update([0_u8, 2_u8]);
            for (handle, id) in &evidence.handles {
                hasher.update(handle.as_bytes());
                hasher.update([0_u8]);
                hasher.update(id.as_bytes());
                hasher.update([0_u8]);
            }
        }
        format!("{:x}", hasher.finalize())
    }

    /// The system turn the request is sent with.
    pub fn system_turn(&self) -> &str {
        self.system.as_deref().unwrap_or(SYSTEM_INSTRUCTION)
    }

    /// Every character the model reads for this request beyond the chat
    /// template: the system turn when it is the request's own, and the
    /// user turn.
    pub fn input_characters(&self) -> usize {
        self.system
            .as_deref()
            .map_or(0, |system| system.chars().count())
            + self.prompt.chars().count()
    }
}

/// The evidence handles an evidence-pipeline prompt shows, each with the
/// stable id of the unit it stands for.
///
/// With [`crate::retrieve::IdStyle::Stable`] a handle is the id itself; with
/// [`crate::retrieve::IdStyle::Ordinal`] it is a number local to the
/// prompt. Either way only the stable id is ever stored.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EvidenceHandles {
    handles: Vec<(String, String)>,
}

impl EvidenceHandles {
    /// `(handle, stable id)` pairs, in prompt order.
    pub fn new(handles: Vec<(String, String)>) -> Self {
        Self { handles }
    }

    /// Whether any handle stands for an id other than itself: ordinal
    /// handles do, stable ones never.
    pub fn remaps(&self) -> bool {
        self.handles.iter().any(|(handle, id)| handle != id)
    }

    pub fn handles(&self) -> impl Iterator<Item = &str> {
        self.handles.iter().map(|(handle, _)| handle.as_str())
    }

    /// The stable id of the unit a reply's id names, or `None` when the
    /// prompt showed no such handle. A reply may write a handle bare, as
    /// `[handle]` the way the prompt shows it, or as a number.
    pub fn resolve(&self, cited: &str) -> Option<&str> {
        let cited = cited.trim();
        let cited = cited
            .strip_prefix('[')
            .and_then(|inner| inner.strip_suffix(']'))
            .unwrap_or(cited)
            .trim();
        self.handles
            .iter()
            .find(|(handle, _)| handle == cited)
            .map(|(_, id)| id.as_str())
    }
}

/// A proposal and what the model reported about producing it.
#[derive(Clone, Debug, PartialEq)]
pub struct ProposerReply {
    pub proposal: ModelProposal,
    /// See [`Proposer::propose_scored`].
    pub token_confidence: Option<TokenConfidence>,
    /// How the server spent the request, when it said: the local server
    /// does, and nothing else is asked.
    pub timings: Option<ModelTimings>,
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

    /// A proposal with whatever the model reported about the work: token
    /// probabilities and server-side timings. Only the local server reports
    /// timings, so by default this is [`Self::propose_scored`] without them.
    fn propose_measured(&self, request: &ModelRequest) -> EngineResult<ProposerReply> {
        self.propose_scored(request)
            .map(|(proposal, token_confidence)| ProposerReply {
                proposal,
                token_confidence,
                timings: None,
            })
    }

    /// The context the prompt and the reply share, in tokens, when it is
    /// small enough that the engine must fit prompts to it: the local
    /// server's. A hosted service's context is many times that and not
    /// Intern's to size, so by default a prompt is sent as distilled, and
    /// only the service's own answer that it did not fit condenses it.
    fn context_tokens(&self) -> Option<usize> {
        None
    }
}

pub struct ModelClient {
    endpoint: Url,
    api_key: String,
    model_id: String,
    http: Client,
    token_confidence: bool,
    context_tokens: usize,
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

    fn propose_measured(&self, request: &ModelRequest) -> EngineResult<ProposerReply> {
        ModelClient::propose_measured(self, request)
    }

    /// The context the local server runs with: [`crate::server::CONTEXT_TOKENS`]
    /// unless [`ModelClient::with_context_tokens`] says otherwise.
    fn context_tokens(&self) -> Option<usize> {
        Some(self.context_tokens)
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
            context_tokens: crate::server::CONTEXT_TOKENS as usize,
        })
    }

    /// The context the server was started with, for a server started with
    /// another `--ctx-size` than the app's: prompts are fitted to it.
    pub fn with_context_tokens(mut self, tokens: usize) -> Self {
        self.context_tokens = tokens;
        self
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

    /// One attempt, then one retry when the reply was complete but
    /// malformed. A request that failed outright is not retried: the second
    /// attempt fails the same way, and against a server that has died or hung
    /// it turns one document into two full request timeouts before anyone is
    /// told. Neither is a reply that ran out of tokens or a prompt the server
    /// could not fit: decoding is greedy, so the same request is cut off at
    /// the same token or refused for the same size every time.
    pub fn propose(&self, request: &ModelRequest) -> EngineResult<ModelProposal> {
        self.propose_scored(request).map(|(proposal, _)| proposal)
    }

    /// [`Self::propose`], with the reply's [`TokenConfidence`] when the
    /// client asks for token probabilities.
    pub fn propose_scored(
        &self,
        request: &ModelRequest,
    ) -> EngineResult<(ModelProposal, Option<TokenConfidence>)> {
        self.propose_measured(request)
            .map(|reply| (reply.proposal, reply.token_confidence))
    }

    /// [`Self::propose_scored`], with the server's timings for the attempt
    /// that was answered.
    pub fn propose_measured(&self, request: &ModelRequest) -> EngineResult<ProposerReply> {
        match self.propose_once(request) {
            Ok(reply) => Ok(reply),
            Err(AttemptError(EngineErrorCode::ModelResponseInvalid)) => {
                self.propose_once(request).map_err(AttemptError::into_error)
            }
            Err(error) => Err(error.into_error()),
        }
    }

    pub(crate) fn propose_once(
        &self,
        request: &ModelRequest,
    ) -> Result<ProposerReply, AttemptError> {
        let response = self
            .http
            .post(self.endpoint.clone())
            .bearer_auth(&self.api_key)
            .json(&self.completion_request(request))
            .send()
            .map_err(|_| AttemptError(EngineErrorCode::ModelRequestFailed))?;
        let status = response.status();
        if !status.is_success() {
            // A prompt too long for the context window is the document's
            // problem, not the server's: restarting the server and sending
            // the same prompt again only ends in the same 400.
            let body = read_error_body(response);
            if status == reqwest::StatusCode::BAD_REQUEST && is_context_overflow(&body) {
                return Err(AttemptError(EngineErrorCode::ModelInputTooLarge));
            }
            return Err(AttemptError(EngineErrorCode::ModelRequestFailed));
        }
        let bytes = read_capped(response, EngineErrorCode::ModelResponseInvalid)?;
        let completion: ChatCompletion = serde_json::from_slice(&bytes)
            .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?;
        let token_confidence = completion.token_confidence();
        let timings = completion.model_timings();
        decode(completion, request.evidence.as_ref()).map(|proposal| ProposerReply {
            proposal,
            token_confidence,
            timings,
        })
    }

    /// The request body. Only the user turn, and a grammar particular to the
    /// request, identify it ([`ModelRequest::sha256`]); everything else here
    /// is transport, and the token-probability fields in particular change
    /// no generated token.
    fn completion_request(&self, request: &ModelRequest) -> Value {
        let mut body = json!({
            "model": self.model_id,
            "messages": [
                {"role": "system", "content": request.system_turn()},
                {"role": "user", "content": request.prompt}
            ],
            "stream": false,
            "temperature": 0,
            "top_k": 1,
            "max_tokens": MAX_REPLY_TOKENS,
            "grammar": request.grammar.as_deref().unwrap_or(RESPONSE_GRAMMAR),
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
/// `parties` values, from the per-token probabilities a server reported -
/// in an evidence-pipeline reply, its `date` and each party's name.
///
/// The grammar forces every key, quote, and bracket, and the model's raw
/// probability for a forced token says nothing about the answer, so only
/// tokens overlapping a value's own characters count: the inside of the date
/// string or a party name, or a bare `null` or `[]`. A compact fact,
/// `[value, role, id]`, counts by its value alone: its role and id are
/// answers of their own, not the date or the name.
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
    let date = &b"\"date\":"[..];
    if let Some(at) = find(&text, date).map(|at| at + date.len()) {
        if text.get(at) == Some(&b'[') {
            fact_span(&text, at, &mut spans);
        } else {
            value_spans(&text, at, &mut spans);
        }
    }
    // An evidence-pipeline reply names each party in a `"name"` of its own.
    // The key cannot occur in the digest pipeline's reply, whose strings
    // escape every quote, so that reply's spans are unchanged.
    let name = &b"\"name\":"[..];
    let mut from = 0;
    while let Some(offset) = find(&text[from..], name) {
        let at = from + offset + name.len();
        value_spans(&text, at, &mut spans);
        from = at;
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
/// or the bare literal (`null`, `[]`) when there is no string. A list of
/// compact facts (`[[name, role, id], ...]`) spans each fact's value.
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
            loop {
                index = match text.get(index) {
                    Some(b'"') => string_span(text, index, spans),
                    Some(b'[') => fact_span(text, index, spans),
                    _ => break,
                };
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
    let (inside, next) = string_at(text, quote);
    spans.push(inside);
    next
}

/// The inside of the string opening at `quote`, and the index just past its
/// closing quote.
fn string_at(text: &[u8], quote: usize) -> ((usize, usize), usize) {
    let start = quote + 1;
    let mut index = start;
    while index < text.len() {
        match text[index] {
            b'\\' => index += 2,
            b'"' => return ((start, index), index + 1),
            _ => index += 1,
        }
    }
    ((start, text.len()), text.len())
}

/// Records the value of the compact fact opening at `open`, `[value, role,
/// id]`: the grammar writes the value first, and only its inside is the
/// answer measured. Returns the index just past the fact's closing bracket.
fn fact_span(text: &[u8], open: usize, spans: &mut Vec<(usize, usize)>) -> usize {
    let mut index = open + 1;
    if text.get(index) == Some(&b'"') {
        index = string_span(text, index, spans);
    }
    while let Some(&byte) = text.get(index) {
        match byte {
            b'"' => index = string_at(text, index).1,
            b']' => return index + 1,
            _ => index += 1,
        }
    }
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

/// The start of an error reply, for telling one failure from another. A body
/// that cannot be read is an empty one: the status still says what happened.
pub(crate) fn read_error_body(body: impl std::io::Read) -> Vec<u8> {
    use std::io::Read as _;

    let mut bytes = Vec::new();
    let _ = body.take(MAX_ERROR_BYTES).read_to_end(&mut bytes);
    bytes
}

/// Whether an error reply says the prompt did not fit the context window, in
/// llama.cpp's words: the error type its server sends (b10361), and the
/// sentence it and the servers built on it use.
pub(crate) fn is_context_overflow(body: &[u8]) -> bool {
    let text = String::from_utf8_lossy(body);
    text.contains("exceed_context_size_error")
        || text.contains("exceeds the available context size")
}

/// Reads a proposal out of a chat-completion reply: the local server's, or
/// any OpenAI-compatible service's.
///
/// Why the reply ended matters more than its exact word for it. A refusal or
/// a content filter is the service declining this document, which no retry
/// changes; a reply cut off at the token limit is truncated, not malformed,
/// and the same request is cut off again. Any other reason - `stop`, a
/// server's own `eos_token` or `end_turn` - is accepted when the content reads
/// as a proposal, because the content is what is checked.
///
/// `evidence` is the request's evidence handles: an evidence-pipeline reply
/// is read as facts and evidence ids ([`facts_from_text`]), any other as the
/// digest pipeline's proposal.
pub(crate) fn decode(
    completion: ChatCompletion,
    evidence: Option<&EvidenceHandles>,
) -> Result<ModelProposal, AttemptError> {
    let choice = completion
        .choices
        .into_iter()
        .next()
        .ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    let finish_reason = choice.finish_reason.as_deref();
    let refused = choice
        .message
        .refusal
        .as_deref()
        .is_some_and(|refusal| !refusal.trim().is_empty());
    if refused || finish_reason == Some("content_filter") {
        return Err(AttemptError(EngineErrorCode::HostedModelRefused));
    }
    if finish_reason == Some("length") {
        return Err(AttemptError(EngineErrorCode::ModelReplyTruncated));
    }
    let content = choice
        .message
        .content
        .and_then(AssistantContent::into_text)
        .ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    proposal_from_text(&content, evidence)
}

/// Reads a proposal out of the text a model replied with, fences and
/// chatter tolerated: as facts and evidence ids when the request showed
/// evidence handles, as the digest pipeline's proposal otherwise.
pub(crate) fn proposal_from_text(
    content: &str,
    evidence: Option<&EvidenceHandles>,
) -> Result<ModelProposal, AttemptError> {
    if let Some(handles) = evidence {
        return facts_from_text(content, handles);
    }
    let json =
        extract_json_object(content).ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    let wire: WireProposal = serde_json::from_str(json)
        .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    wire.into_domain()
        .ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))
}

/// Reads an evidence-pipeline reply: facts, each after the ids of the
/// evidence it cites.
///
/// Lenient about shape, because a hosted model answers without a grammar:
/// `type` or `document_type`, one id or a list, ids as strings or numbers,
/// `[p1.b2]` as the prompt shows it. Strict about meaning: every id is
/// mapped back to the stable id of the unit its handle stands for, and an id
/// the prompt did not show is set aside in
/// [`ModelFacts::unknown_evidence`] - never evidence for anything. A role
/// that is not one of [`PartyRole`]'s is no role, and a date that is not on
/// the calendar is no date, as in the digest pipeline.
pub(crate) fn facts_from_text(
    content: &str,
    handles: &EvidenceHandles,
) -> Result<ModelProposal, AttemptError> {
    let json =
        extract_json_object(content).ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    let value: Value = serde_json::from_str(json)
        .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?;
    let wire = match compact_wire(&value) {
        Some(wire) => wire,
        None => serde_json::from_value::<WireFacts>(value)
            .map_err(|_| AttemptError(EngineErrorCode::ModelResponseInvalid))?,
    };
    wire.into_domain(handles)
        .ok_or(AttemptError(EngineErrorCode::ModelResponseInvalid))
}

/// A compact reply ([`crate::prompt::COMPACT_INSTRUCTIONS`]) read into the
/// same shape as a fields reply, or `None` for a reply that is not one: a
/// fact is an array of its value - a date with its role, a party with its
/// role - and the ids of its lines, `amount` the id of the amount's line,
/// `review` the one request for a person. Lenient as [`facts_from_text`]
/// is: the parts of an array are told apart by what they are, not only by
/// where they stand, so `[id, value]` reads as well as `[value, id]`. Only
/// a stable id is told apart that way: the grammar writes the value first
/// and ordinal ids as bare numbers, so the first other string is the value
/// even when it is all digits, and a quoted number after it an id.
///
/// A compact reply asks for no confidence of its own; it reads as full,
/// and only `review` or the document's own checks send it to a person.
fn compact_wire(value: &Value) -> Option<WireFacts> {
    let object = value.as_object()?;
    let arrays = ["type", "date", "subject"]
        .iter()
        .any(|key| object.get(*key).is_some_and(Value::is_array))
        || object
            .get("parties")
            .and_then(Value::as_array)
            .is_some_and(|parties| parties.iter().any(Value::is_array));
    if !arrays && object.contains_key("confidence") {
        return None;
    }
    let ids_of = |values: Vec<&Value>| {
        WireIds::Many(
            values
                .into_iter()
                .filter_map(|value| match value {
                    Value::String(text) => Some(WireId::Text(text.clone())),
                    Value::Number(number) => number.as_u64().map(WireId::Number),
                    _ => None,
                })
                .collect(),
        )
    };
    // A fact's value and its ids: the first string that is not a stable
    // id is the value, even all digits, and everything else its ids.
    fn split(value: Option<&Value>) -> (Option<String>, Vec<&Value>) {
        match value {
            Some(Value::String(text)) => (Some(text.clone()), Vec::new()),
            Some(Value::Array(parts)) => {
                let mut text = None;
                let mut ids = Vec::new();
                for part in parts {
                    match part {
                        Value::String(word) if text.is_none() && !is_stable_id(word) => {
                            text = Some(word.clone());
                        }
                        Value::String(word) if looks_like_id(word) => ids.push(part),
                        Value::Number(_) => ids.push(part),
                        _ => {}
                    }
                }
                (text, ids)
            }
            _ => (None, Vec::new()),
        }
    }
    let (document_type, type_ids) = split(object.get("type"));
    let (mut document_date, mut date_role) = (None, None);
    let mut date_ids = Vec::new();
    match object.get("date") {
        Some(Value::String(date)) => document_date = Some(date.clone()),
        Some(Value::Array(parts)) => {
            for part in parts {
                match part {
                    Value::String(text) if document_date.is_none() && is_date_shaped(text) => {
                        document_date = Some(text.clone());
                    }
                    Value::String(text)
                        if date_role.is_none()
                            && DateRole::ALL
                                .iter()
                                .any(|role| role.as_str().eq_ignore_ascii_case(text.trim())) =>
                    {
                        date_role = Some(text.clone());
                    }
                    Value::String(text) if looks_like_id(text) => date_ids.push(part),
                    Value::Number(_) => date_ids.push(part),
                    _ => {}
                }
            }
        }
        _ => {}
    }
    let mut parties = Vec::new();
    for party in object
        .get("parties")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        match party {
            Value::Array(parts) => {
                let (mut name, mut role) = (None, None);
                let mut ids = Vec::new();
                for part in parts {
                    match part {
                        Value::String(text) if name.is_none() && !is_stable_id(text) => {
                            name = Some(text.clone());
                        }
                        Value::String(text) if looks_like_id(text) => ids.push(part),
                        // A role, or a word that is none: the reply's role
                        // either way, and no role if it is not one.
                        Value::String(text) if role.is_none() => role = Some(text.clone()),
                        Value::Number(_) => ids.push(part),
                        _ => {}
                    }
                }
                parties.push(WireParty {
                    ids: ids_of(ids),
                    name,
                    role,
                });
            }
            other => parties.push(serde_json::from_value(other.clone()).ok()?),
        }
    }
    let (subject, subject_ids) = split(object.get("subject"));
    let amount_ids = match object.get("amount") {
        Some(amount @ Value::Array(_)) => ids_of(split(Some(amount)).1),
        Some(other) => ids_of(vec![other]),
        None => WireIds::None,
    };
    Some(WireFacts {
        type_ids: ids_of(type_ids),
        document_type,
        date_ids: ids_of(date_ids),
        document_date,
        date_role,
        parties,
        subject_ids: ids_of(subject_ids),
        subject,
        identifier_ids: WireIds::None,
        identifier: None,
        facts: Vec::new(),
        amount_ids,
        confidence: 1.0,
        needs_review: object
            .get("review")
            .or_else(|| object.get("needs_review"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

/// Whether a reply's string is an evidence id as a prompt shows one:
/// `p1.b2`, `p3.b7.r4`, `[p1.b2]`, or a bare number.
fn looks_like_id(text: &str) -> bool {
    let bare = text.trim().trim_start_matches('[').trim_end_matches(']');
    (!bare.is_empty() && bare.chars().all(|character| character.is_ascii_digit()))
        || is_stable_id(bare)
}

/// Whether a string is a unit's stable id, `p3.b2.f1`, bracketed or not -
/// never a value, wherever it stands. An all-digit string can be either:
/// a quoted ordinal id, or a value such as a `1099` or a suite `101`.
fn is_stable_id(text: &str) -> bool {
    let bare = text.trim().trim_start_matches('[').trim_end_matches(']');
    let mut parts = bare.split('.');
    parts.next().is_some_and(|page| {
        page.strip_prefix('p')
            .is_some_and(|number| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
    }) && parts.all(|part| {
        let mut characters = part.chars();
        characters.next().is_some_and(|c| c.is_ascii_lowercase())
            && part.len() > 1
            && characters.all(|c| c.is_ascii_digit())
    })
}

/// Whether a string has a date's shape, `YYYY-MM-DD`; the calendar is
/// checked later.
fn is_date_shaped(text: &str) -> bool {
    let bytes = text.trim().as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(at, byte)| at == 4 || at == 7 || byte.is_ascii_digit())
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
    /// llama-server's account of the request. Kept unparsed, like
    /// [`Choice::logprobs`]: a service that sends something else under the
    /// same name costs the measurement, never the reply.
    #[serde(default)]
    timings: Option<Value>,
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

    /// How the server spent the request, when it reported that in
    /// llama-server's shape.
    pub(crate) fn model_timings(&self) -> Option<ModelTimings> {
        let timings: ServerTimings = serde_json::from_value(self.timings.clone()?).ok()?;
        Some(ModelTimings {
            prompt_tokens: timings.prompt_n,
            cached_tokens: timings.cache_n,
            prefill_micros: micros_from_millis(timings.prompt_ms),
            generated_tokens: timings.predicted_n,
            generation_micros: micros_from_millis(timings.predicted_ms),
        })
    }
}

/// The `timings` object llama-server adds to a chat completion. It carries
/// per-token and per-second rates too; those follow from these.
#[derive(Deserialize)]
struct ServerTimings {
    /// Absent from servers that predate prompt caching, which reused nothing.
    #[serde(default)]
    cache_n: u64,
    prompt_n: u64,
    prompt_ms: f64,
    predicted_n: u64,
    predicted_ms: f64,
}

/// Fractional milliseconds as whole microseconds. A float cast saturates, so
/// a negative or non-finite figure from a confused server reads as zero
/// rather than wrapping.
fn micros_from_millis(millis: f64) -> u64 {
    (millis * 1_000.0).round() as u64
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
    /// Null when the model refused: OpenAI then says why in `refusal`.
    #[serde(default)]
    content: Option<AssistantContent>,
    #[serde(default)]
    refusal: Option<String>,
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
            facts: None,
        })
    }
}

/// Most evidence ids a single fact may cite; the grammar's `ids` rule.
pub(crate) const MAX_IDS: usize = 3;
/// Most evidence ids a party, an identifier or a key fact may cite; the
/// grammar's `ids2` rule.
pub(crate) const MAX_IDS_SHORT: usize = 2;
/// Most parties a reply may name.
pub(crate) const MAX_PARTIES: usize = 3;
/// Most key facts a reply may name.
pub(crate) const MAX_KEY_FACTS: usize = 2;

#[derive(Deserialize)]
struct WireFacts {
    #[serde(default, alias = "document_type_ids", alias = "type_evidence")]
    type_ids: WireIds,
    #[serde(default, rename = "type", alias = "document_type")]
    document_type: Option<String>,
    #[serde(default, alias = "document_date_ids", alias = "date_evidence")]
    date_ids: WireIds,
    #[serde(default, rename = "date", alias = "document_date")]
    document_date: Option<String>,
    #[serde(default)]
    date_role: Option<String>,
    #[serde(default)]
    parties: Vec<WireParty>,
    #[serde(default, alias = "subject_evidence")]
    subject_ids: WireIds,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default, alias = "identifier_evidence")]
    identifier_ids: WireIds,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(default, alias = "key_facts")]
    facts: Vec<WireFact>,
    /// Only a compact reply cites an amount's line.
    #[serde(skip)]
    amount_ids: WireIds,
    #[serde(default)]
    confidence: f32,
    #[serde(default)]
    needs_review: bool,
}

#[derive(Deserialize)]
struct WireParty {
    #[serde(default, alias = "evidence", alias = "party_ids")]
    ids: WireIds,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    role: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum WireFact {
    Cited {
        #[serde(default, alias = "evidence")]
        ids: WireIds,
        #[serde(alias = "value", alias = "text")]
        fact: String,
    },
    Bare(String),
}

/// One id or a list of them, each a string or a number; or nothing.
#[derive(Default, Deserialize)]
#[serde(untagged)]
enum WireIds {
    #[default]
    None,
    One(WireId),
    Many(Vec<WireId>),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum WireId {
    Text(String),
    Number(u64),
}

impl WireId {
    fn text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Number(number) => number.to_string(),
        }
    }
}

impl WireIds {
    /// The stable ids of the handles cited, each once and at most `cap` of
    /// them; ids the prompt did not show go to `unknown`.
    fn resolve(
        self,
        handles: &EvidenceHandles,
        cap: usize,
        unknown: &mut Vec<String>,
    ) -> Vec<String> {
        let cited = match self {
            Self::None => Vec::new(),
            Self::One(id) => vec![id],
            Self::Many(ids) => ids,
        };
        let mut resolved = Vec::new();
        for id in cited {
            let text = id.text();
            match handles.resolve(&text) {
                Some(stable) => {
                    if resolved.len() < cap && !resolved.iter().any(|kept| kept == stable) {
                        resolved.push(stable.to_owned());
                    }
                }
                None => {
                    if !unknown.contains(&text) {
                        unknown.push(text);
                    }
                }
            }
        }
        resolved
    }
}

/// A value the reply gave, trimmed; blank is absent.
fn present(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("null"))
}

impl WireFacts {
    fn into_domain(self, handles: &EvidenceHandles) -> Option<ModelProposal> {
        if !self.confidence.is_finite() || !(0.0..=1.0).contains(&self.confidence) {
            return None;
        }
        let mut unknown = Vec::new();
        let type_evidence = self.type_ids.resolve(handles, MAX_IDS, &mut unknown);
        let date_evidence = self.date_ids.resolve(handles, MAX_IDS, &mut unknown);
        let mut document_date = present(self.document_date);
        let mut date_role = self.date_role.as_deref().and_then(|role| {
            DateRole::ALL
                .into_iter()
                .find(|known| known.as_str().eq_ignore_ascii_case(role.trim()))
        });
        if document_date
            .as_deref()
            .is_some_and(|date| !is_valid_iso_date(date))
        {
            // As in the digest pipeline: a date that is not on the calendar
            // is the model inventing one, and its role goes with it.
            document_date = None;
            date_role = None;
        }
        if document_date.is_none() {
            date_role = None;
        }
        let mut parties = Vec::new();
        for party in self.parties {
            let evidence = party.ids.resolve(handles, MAX_IDS_SHORT, &mut unknown);
            let Some(name) = present(party.name) else {
                continue;
            };
            if parties.len() == MAX_PARTIES {
                continue;
            }
            parties.push(PartyFact {
                name,
                role: party.role.as_deref().and_then(PartyRole::parse),
                evidence,
            });
        }
        let subject_evidence = self.subject_ids.resolve(handles, MAX_IDS, &mut unknown);
        let identifier_evidence = self
            .identifier_ids
            .resolve(handles, MAX_IDS_SHORT, &mut unknown);
        let mut key_facts = Vec::new();
        for fact in self.facts {
            let (ids, fact) = match fact {
                WireFact::Cited { ids, fact } => (ids, fact),
                WireFact::Bare(fact) => (WireIds::None, fact),
            };
            let evidence = ids.resolve(handles, MAX_IDS_SHORT, &mut unknown);
            let Some(fact) = present(Some(fact)) else {
                continue;
            };
            if key_facts.len() < MAX_KEY_FACTS {
                key_facts.push(KeyFact { fact, evidence });
            }
        }
        let facts = ModelFacts {
            document_type: present(self.document_type),
            type_evidence,
            document_date,
            date_role,
            date_evidence,
            parties,
            subject: present(self.subject),
            subject_evidence,
            identifier: present(self.identifier),
            identifier_evidence,
            key_facts,
            amount_evidence: self.amount_ids.resolve(handles, 1, &mut unknown),
            unknown_evidence: unknown,
        };
        Some(ModelProposal {
            document_type: facts.document_type.clone(),
            document_date: facts.document_date.clone(),
            date_role: facts.date_role,
            parties: facts
                .parties
                .iter()
                .map(|party| party.name.clone())
                .collect(),
            party_relation: PartyRelation::None,
            description: String::new(),
            confidence: self.confidence,
            needs_review: self.needs_review,
            evidence: Evidence::default(),
            facts: Some(Box::new(facts)),
        })
    }
}

#[derive(Debug)]
pub(crate) struct AttemptError(pub(crate) EngineErrorCode);

impl AttemptError {
    fn into_error(self) -> EngineError {
        match self.0 {
            EngineErrorCode::ModelRequestFailed => request_failed(),
            EngineErrorCode::ModelInputTooLarge => EngineError::new(
                EngineErrorCode::ModelInputTooLarge,
                "the document does not fit the local model's context window",
            ),
            EngineErrorCode::ModelReplyTruncated => EngineError::new(
                EngineErrorCode::ModelReplyTruncated,
                "the local model ran out of room before finishing its answer",
            ),
            EngineErrorCode::HostedModelRefused => EngineError::new(
                EngineErrorCode::HostedModelRefused,
                "the model declined to answer about this document",
            ),
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
    use crate::test_support::{VALID_REPLY, completion_reply, http_reply, scripted_server};

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
        let body = client.completion_request(&ModelRequest::new("p"));
        assert_eq!(body["grammar"], serde_json::json!(RESPONSE_GRAMMAR));
        assert_eq!(body["temperature"], serde_json::json!(0));
        assert_eq!(body["max_tokens"], serde_json::json!(1_024));
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
        let body = client.completion_request(&ModelRequest::new("p"));
        assert!(body["messages"][1]["content"].is_string());
        assert!(!body.to_string().contains("image_url"));
    }

    fn local_client(address: std::net::SocketAddr) -> ModelClient {
        ModelClient::new(&format!("http://{address}/v1/chat/completions"), "k", "m").unwrap()
    }

    /// A retry is for a reply that came back malformed. A request that failed
    /// outright fails the same way twice, and against a dead or hung server
    /// the second attempt only doubles a ten-minute wait for one document.
    #[test]
    fn a_failed_request_is_not_retried() {
        let server = scripted_server(vec![http_reply("503 Service Unavailable", &[], "")]);

        let error = local_client(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap_err();

        assert_eq!(error.code(), EngineErrorCode::ModelRequestFailed);
        assert_eq!(server.attempts(), 1);
    }

    /// llama-server answers a prompt longer than its context with a 400. That
    /// was a "failed request": the server was restarted, the same prompt sent
    /// again, and the queue paused over one oversized document.
    #[test]
    fn exceed_context_400_maps_to_input_too_large() {
        // The body b10361 sends, as a live server answered a prompt too long
        // for its 8,192-token context.
        let overflow = r#"{"error":{"code":400,"message":"request (9214 tokens) exceeds the available context size (8192 tokens), try increasing it","type":"exceed_context_size_error","n_prompt_tokens":9214,"n_ctx":8192}}"#;
        let server = scripted_server(vec![http_reply(
            "400 Bad Request",
            &[("Content-Type", "application/json")],
            overflow,
        )]);
        let error = local_client(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap_err();
        assert_eq!(error.code(), EngineErrorCode::ModelInputTooLarge);
        assert_eq!(server.attempts(), 1);

        // Any other 400 is still a failed request.
        let server = scripted_server(vec![http_reply(
            "400 Bad Request",
            &[],
            r#"{"error":{"code":400,"message":"invalid grammar","type":"invalid_request_error"}}"#,
        )]);
        let error = local_client(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap_err();
        assert_eq!(error.code(), EngineErrorCode::ModelRequestFailed);
    }

    /// Decoding is greedy, so a reply cut off at the token limit is cut off at
    /// the same token the second time. One request, and a code that says what
    /// happened instead of "malformed".
    #[test]
    fn finish_reason_length_is_truncated_and_not_retried() {
        let server = scripted_server(vec![completion_reply(
            "length",
            r#"{"type_evidence":"This Master Services Agreement is entered into as of March 1, 2025 by and between"#,
        )]);
        let error = local_client(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap_err();
        assert_eq!(error.code(), EngineErrorCode::ModelReplyTruncated);
        assert_eq!(server.attempts(), 1);
    }

    /// A reply that finished but cannot be read is the one case a second
    /// attempt is for - once.
    #[test]
    fn malformed_complete_reply_is_retried_once() {
        let server = scripted_server(vec![completion_reply("stop", "I think this is a memo.")]);
        let error = local_client(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap_err();
        assert_eq!(error.code(), EngineErrorCode::ModelResponseInvalid);
        assert_eq!(server.attempts(), 2);

        let server = scripted_server(vec![
            completion_reply("stop", "I think this is a memo."),
            completion_reply("stop", VALID_REPLY),
        ]);
        let proposal = local_client(server.address)
            .propose(&ModelRequest::new("p"))
            .unwrap();
        assert_eq!(proposal.document_type.as_deref(), Some("Memo"));
        assert_eq!(server.attempts(), 2);
    }

    fn decoded(reply: Value) -> Result<ModelProposal, EngineErrorCode> {
        let completion: ChatCompletion = serde_json::from_value(reply).unwrap();
        decode(completion, None).map_err(|AttemptError(code)| code)
    }

    /// OpenAI's refusal shape - `content: null` beside a `refusal` - did not
    /// even deserialize, and Azure's content filter read as malformed: four
    /// billed requests, then the whole queue paused over one document.
    #[test]
    fn refusal_null_content_and_content_filter_map_to_refused() {
        assert_eq!(
            decoded(
                json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":null,"refusal":"I can't help with that request."}}]})
            ),
            Err(EngineErrorCode::HostedModelRefused)
        );
        assert_eq!(
            decoded(
                json!({"choices":[{"finish_reason":"content_filter","message":{"role":"assistant","content":null}}]})
            ),
            Err(EngineErrorCode::HostedModelRefused)
        );
        // A filter that cut in after the model had answered is still a filter.
        assert_eq!(
            decoded(
                json!({"choices":[{"finish_reason":"content_filter","message":{"role":"assistant","content":VALID_REPLY}}]})
            ),
            Err(EngineErrorCode::HostedModelRefused)
        );
        // An empty refusal field is no refusal.
        assert!(
            decoded(json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":VALID_REPLY,"refusal":""}}]}))
                .is_ok()
        );
        // Null content with nothing to say why is a malformed reply.
        assert_eq!(
            decoded(
                json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":null}}]})
            ),
            Err(EngineErrorCode::ModelResponseInvalid)
        );
        assert_eq!(
            decoded(
                json!({"choices":[{"finish_reason":"length","message":{"role":"assistant","content":"{\"type_evidence\":"}}]})
            ),
            Err(EngineErrorCode::ModelReplyTruncated)
        );
    }

    /// Hugging Face TGI ends with `eos_token`, other servers with words of
    /// their own. The content is what is checked; the label is not.
    #[test]
    fn unknown_finish_reason_with_valid_content_is_accepted() {
        for reason in ["eos_token", "end_turn", "stop_sequence"] {
            let proposal = decoded(json!({"choices":[{"finish_reason":reason,"message":{"role":"assistant","content":VALID_REPLY}}]}))
                .unwrap_or_else(|code| panic!("{reason}: {code:?}"));
            assert_eq!(proposal.document_type.as_deref(), Some("Memo"));
        }
        assert!(
            decoded(json!({"choices":[{"message":{"role":"assistant","content":VALID_REPLY}}]}))
                .is_ok(),
            "no finish reason at all"
        );
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
        let request = ModelRequest::new("p");
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

    /// A live llama-server reply to a request shaped as this client sends
    /// it, the second of two identical ones: all but four prompt tokens came
    /// from the slot's cache. Only the echoed model path was replaced.
    const LIVE_REPLY: &str = r#"{"choices":[{"finish_reason":"stop","index":0,"message":{"role":"assistant","content":"{\"type_evidence\":null,\"document_type\":\"invoice\",\"date_evidence\":\"March 4, 2026\",\"document_date\":\"2026-03-04\",\"date_role\":\"invoice\",\"parties\":[\"Halvorsen Fixture Works LLC\",\"Quillon Ridge Bakery\"],\"party_evidence\":[\"Halvorsen Fixture Works LLC\",\"Quillon Ridge Bakery\"],\"party_relation\":\"between\",\"description\":\"invoice\",\"confidence\":0.99,\"needs_review\":false}"}}],"created":1791321013,"model":"intern-local","system_fingerprint":"b1-14e78dd","object":"chat.completion","usage":{"completion_tokens":115,"prompt_tokens":127,"total_tokens":242,"prompt_tokens_details":{"cached_tokens":123}},"id":"chatcmpl-zUHUpNHINXuQa7x9KBvmW751fnA4oVxa","timings":{"cache_n":123,"prompt_n":4,"prompt_ms":206.87,"prompt_per_token_ms":51.7175,"prompt_per_second":19.33581476289457,"predicted_n":115,"predicted_ms":16131.292,"predicted_per_token_ms":140.2721043478261,"predicted_per_second":7.129001198416097}}"#;

    #[test]
    fn server_timings_are_read_from_the_servers_reply() {
        let completion: ChatCompletion = serde_json::from_str(LIVE_REPLY).unwrap();

        assert_eq!(
            completion.model_timings(),
            Some(ModelTimings {
                prompt_tokens: 4,
                cached_tokens: 123,
                prefill_micros: 206_870,
                generated_tokens: 115,
                generation_micros: 16_131_292,
            })
        );
        let proposal = decode(completion, None).unwrap();
        assert_eq!(proposal.document_date.as_deref(), Some("2026-03-04"));
    }

    /// A hosted service sends no timings, and one that sends something else
    /// under the name costs the measurement, never the reply.
    #[test]
    fn a_reply_without_readable_timings_still_decodes() {
        for reply in [
            json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":VALID_REPLY}}]}),
            json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":VALID_REPLY}}],"timings":null}),
            json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":VALID_REPLY}}],"timings":"fast"}),
            json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":VALID_REPLY}}],"timings":{"prompt_n":-1,"prompt_ms":1.0,"predicted_n":2,"predicted_ms":3.0}}),
        ] {
            let completion: ChatCompletion = serde_json::from_value(reply.clone()).unwrap();
            assert_eq!(completion.model_timings(), None, "{reply}");
            assert!(decode(completion, None).is_ok(), "{reply}");
        }
        // A server that predates prompt caching reused nothing.
        let completion: ChatCompletion = serde_json::from_value(json!({"choices":[],
            "timings":{"prompt_n":10,"prompt_ms":2.5,"predicted_n":3,"predicted_ms":0.0004}}))
        .unwrap();
        assert_eq!(
            completion.model_timings(),
            Some(ModelTimings {
                prompt_tokens: 10,
                cached_tokens: 0,
                prefill_micros: 2_500,
                generated_tokens: 3,
                generation_micros: 0,
            })
        );
    }

    /// The retry on a malformed reply is unchanged, and the timings that
    /// come back are those of the attempt that was answered.
    #[test]
    fn measured_replies_carry_the_answered_attempts_timings() {
        let timed = |content: &str, prompt_n: u64| {
            http_reply(
                "200 OK",
                &[("Content-Type", "application/json")],
                &json!({
                    "choices": [{"finish_reason": "stop",
                        "message": {"role": "assistant", "content": content}}],
                    "timings": {"cache_n": 0, "prompt_n": prompt_n, "prompt_ms": 1.5,
                        "predicted_n": 40, "predicted_ms": 900.25},
                })
                .to_string(),
            )
        };
        let server = scripted_server(vec![
            timed("I think this is a memo.", 111),
            timed(VALID_REPLY, 222),
        ]);

        let reply = local_client(server.address)
            .propose_measured(&ModelRequest::new("p"))
            .unwrap();

        assert_eq!(server.attempts(), 2);
        assert_eq!(reply.proposal.document_type.as_deref(), Some("Memo"));
        assert_eq!(reply.token_confidence, None);
        assert_eq!(
            reply.timings,
            Some(ModelTimings {
                prompt_tokens: 222,
                cached_tokens: 0,
                prefill_micros: 1_500,
                generated_tokens: 40,
                generation_micros: 900_250,
            })
        );

        // The scored and plain answers are the same reply without them.
        let server = scripted_server(vec![timed(VALID_REPLY, 7)]);
        let client = local_client(server.address);
        let request = ModelRequest::new("p");
        let measured = client.propose_measured(&request).unwrap();
        assert_eq!(
            client.propose_scored(&request).unwrap(),
            (measured.proposal.clone(), None)
        );
        assert_eq!(client.propose(&request).unwrap(), measured.proposal);
        assert_eq!(server.attempts(), 3);
    }

    /// A proposer that reports no timings - a hosted model, a test double -
    /// is measured as its scored reply with none.
    #[test]
    fn the_default_measured_reply_is_the_scored_one_without_timings() {
        struct Fixed;
        impl Proposer for Fixed {
            fn propose(&self, _request: &ModelRequest) -> EngineResult<ModelProposal> {
                proposal_from_text(VALID_REPLY, None).map_err(AttemptError::into_error)
            }
        }

        let reply = Fixed.propose_measured(&ModelRequest::new("p")).unwrap();

        assert_eq!(reply.proposal.document_type.as_deref(), Some("Memo"));
        assert_eq!(reply.token_confidence, None);
        assert_eq!(reply.timings, None);
    }

    #[test]
    fn the_request_identity_follows_the_prompt() {
        assert_ne!(
            ModelRequest::new("a").sha256(),
            ModelRequest::new("b").sha256()
        );
    }

    fn ordinal_handles() -> EvidenceHandles {
        EvidenceHandles::new(vec![
            ("1".into(), "p1.b1".into()),
            ("2".into(), "p1.b3.f1".into()),
            ("3".into(), "p1.b5".into()),
        ])
    }

    fn stable_handles() -> EvidenceHandles {
        EvidenceHandles::new(
            ["p1.b1", "p1.b3.f1", "p1.b5", "p1.b6.r2"]
                .iter()
                .map(|id| ((*id).to_owned(), (*id).to_owned()))
                .collect(),
        )
    }

    /// A compact reply reads into the same facts as a fields reply: each
    /// fact's id mapped back, an id the prompt never showed set aside, the
    /// amount's line kept, and no confidence of its own to gate on.
    #[test]
    fn a_compact_reply_reads_as_facts_with_their_lines() {
        let reply = r#"{"type":["Invoice","p1.b1"],"date":["2025-05-01","invoice","p1.b3.f1"],"parties":[["Halvorsen Fixture Works LLC","issuer","p1.b1"],["Quillon Ridge Bakery, Inc.","customer","p9.b9"]],"subject":["display shelving","p1.b5"],"amount":"p1.b6.r2"}"#;
        let proposal = proposal_from_text(reply, Some(&stable_handles())).unwrap();
        let facts = proposal.facts.as_ref().unwrap();
        assert_eq!(facts.document_type.as_deref(), Some("Invoice"));
        assert_eq!(facts.type_evidence, vec!["p1.b1"]);
        assert_eq!(facts.document_date.as_deref(), Some("2025-05-01"));
        assert_eq!(facts.date_role, Some(DateRole::Invoice));
        assert_eq!(facts.date_evidence, vec!["p1.b3.f1"]);
        assert_eq!(facts.parties.len(), 2);
        assert_eq!(facts.parties[0].role, Some(PartyRole::Issuer));
        assert_eq!(facts.parties[0].evidence, vec!["p1.b1"]);
        assert!(facts.parties[1].evidence.is_empty());
        assert_eq!(facts.unknown_evidence, vec!["p9.b9"]);
        assert_eq!(facts.subject.as_deref(), Some("display shelving"));
        assert_eq!(facts.subject_evidence, vec!["p1.b5"]);
        assert_eq!(facts.amount_evidence, vec!["p1.b6.r2"]);
        assert_eq!(facts.identifier, None);
        assert!(facts.key_facts.is_empty());
        assert!((proposal.confidence - 1.0).abs() < f32::EPSILON);
        assert!(!proposal.needs_review);

        // Only "review" asks for a person; an empty reply is still a reply.
        let reply = r#"{"type":null,"date":null,"parties":[],"review":true}"#;
        let proposal = proposal_from_text(reply, Some(&stable_handles())).unwrap();
        assert!(proposal.needs_review);
        assert_eq!(proposal.document_type, None);

        // A hosted model's looser arrays read the same: id first, a role
        // the list does not have, a calendar day that does not exist.
        let reply = r#"{"type":["p1.b1","Invoice"],"date":["2025-02-30","invoice","p1.b3.f1"],"parties":[["p1.b5","Quillon Ridge Bakery, Inc.","patron"]],"amount":["$1,248.00","p1.b6.r2"]}"#;
        let proposal = proposal_from_text(reply, Some(&stable_handles())).unwrap();
        let facts = proposal.facts.as_ref().unwrap();
        assert_eq!(facts.document_type.as_deref(), Some("Invoice"));
        assert_eq!(facts.type_evidence, vec!["p1.b1"]);
        assert_eq!(facts.document_date, None);
        assert_eq!(facts.date_role, None);
        assert_eq!(facts.parties[0].name, "Quillon Ridge Bakery, Inc.");
        assert_eq!(facts.parties[0].role, None);
        assert_eq!(facts.amount_evidence, vec!["p1.b6.r2"]);
        assert!(facts.unknown_evidence.is_empty());
    }

    /// With ordinal handles the grammar writes ids as bare numbers, so an
    /// all-digit string where a fact's value stands is the value - a form
    /// 1099, a suite 101 - and a quoted number after it is still an id.
    #[test]
    fn an_all_digit_value_is_not_taken_for_an_ordinal_id() {
        let reply = r#"{"type":["1099",1],"date":["2025-05-01","invoice",2],"parties":[["2024","tenant",3]],"subject":["101","3"]}"#;
        let proposal = proposal_from_text(reply, Some(&ordinal_handles())).unwrap();
        let facts = proposal.facts.as_ref().unwrap();
        assert_eq!(facts.document_type.as_deref(), Some("1099"));
        assert_eq!(facts.type_evidence, vec!["p1.b1"]);
        assert_eq!(facts.date_evidence, vec!["p1.b3.f1"]);
        assert_eq!(facts.parties[0].name, "2024");
        assert_eq!(facts.parties[0].role, Some(PartyRole::Tenant));
        assert_eq!(facts.parties[0].evidence, vec!["p1.b5"]);
        assert_eq!(facts.subject.as_deref(), Some("101"));
        assert_eq!(facts.subject_evidence, vec!["p1.b5"]);
        assert!(facts.unknown_evidence.is_empty());
    }

    /// An ordinal request's identity covers what its numbers stand for: the
    /// same prompt over units whose stable ids moved is a different request,
    /// and a recorded reply to the old one is stale. A stable request is
    /// identified as before.
    #[test]
    fn an_ordinal_requests_identity_covers_the_ids_its_numbers_stand_for() {
        let request = |handles: EvidenceHandles| ModelRequest {
            system: Some("s".into()),
            grammar: Some("g".into()),
            evidence: Some(handles),
            ..ModelRequest::new("p")
        };
        let moved = EvidenceHandles::new(vec![
            ("1".into(), "p1.b1".into()),
            ("2".into(), "p1.b4.f1".into()),
            ("3".into(), "p1.b5".into()),
        ]);
        assert_ne!(request(ordinal_handles()).sha256(), request(moved).sha256());
        let unmapped = ModelRequest {
            evidence: None,
            ..request(stable_handles())
        };
        assert_eq!(request(stable_handles()).sha256(), unmapped.sha256());
    }

    /// The request's identity covers a system turn of its own; a request
    /// without one is identified as it always was.
    #[test]
    fn a_requests_own_system_turn_is_part_of_its_identity() {
        let plain = ModelRequest::new("p");
        let systemed = ModelRequest {
            system: Some("s".into()),
            ..ModelRequest::new("p")
        };
        assert_ne!(plain.sha256(), systemed.sha256());
        assert_eq!(plain.system_turn(), SYSTEM_INSTRUCTION);
        assert_eq!(systemed.system_turn(), "s");
        assert_eq!(systemed.input_characters(), 2);
    }

    /// An evidence reply's ids are the prompt's handles, mapped back to the
    /// stable ids of their units before anything is kept; an id the prompt
    /// did not show is set aside and never cited.
    #[test]
    fn an_evidence_reply_maps_handles_back_to_stable_ids() {
        let reply = r#"{"type_ids":[1],"type":"Invoice","date_ids":[2],"date":"2025-05-01","date_role":"invoice","parties":[{"ids":[1,9],"name":"Halvorsen Fixture Works LLC","role":"issuer"},{"ids":[3],"name":"Quillon Ridge Bakery, Inc.","role":"customer"}],"identifier_ids":[2],"identifier":"INV-10438","confidence":0.9,"needs_review":false}"#;
        let proposal = proposal_from_text(reply, Some(&ordinal_handles())).unwrap();
        let facts = proposal.facts.as_ref().unwrap();
        assert_eq!(facts.type_evidence, vec!["p1.b1"]);
        assert_eq!(facts.date_evidence, vec!["p1.b3.f1"]);
        assert_eq!(facts.parties[0].evidence, vec!["p1.b1"]);
        assert_eq!(facts.parties[0].role, Some(PartyRole::Issuer));
        assert_eq!(facts.parties[1].evidence, vec!["p1.b5"]);
        assert_eq!(facts.identifier.as_deref(), Some("INV-10438"));
        assert_eq!(facts.unknown_evidence, vec!["9"]);
        assert!(facts.cited_ids().all(|id| id.starts_with("p1.")));
        // The digest-shaped fields carry the same facts; the description is
        // the engine's to compose.
        assert_eq!(proposal.document_type.as_deref(), Some("Invoice"));
        assert_eq!(proposal.document_date.as_deref(), Some("2025-05-01"));
        assert_eq!(proposal.parties.len(), 2);
        assert_eq!(proposal.description, "");
        assert_eq!(proposal.evidence, Evidence::default());
    }

    /// A hosted model answers without a grammar, so the shape is read
    /// leniently - and the meaning strictly.
    #[test]
    fn a_hosted_evidence_reply_is_read_leniently_and_strictly() {
        let handles = EvidenceHandles::new(vec![
            ("p1.b1".into(), "p1.b1".into()),
            ("p1.b2".into(), "p1.b2".into()),
        ]);
        let reply = r#"Here are the facts:
```json
{"document_type_ids":"[p1.b1]","document_type":"Notice","date_evidence":["p1.b2","p7.b7"],"document_date":"2025-02-30","date_role":"notice","parties":[{"evidence":"p1.b2","name":"Imogen Castellanos","role":"resident"},{"ids":[],"name":"  ","role":"tenant"},{"ids":["p1.b1"],"name":"Cresthaven Court Holdings LLC","role":"OTHER"}],"key_facts":["rent rises to $1,965.00",{"ids":["p1.b2"],"fact":"effective August 1, 2026"}],"confidence":0.8}
```"#;
        let proposal = proposal_from_text(reply, Some(&handles)).unwrap();
        let facts = proposal.facts.unwrap();
        assert_eq!(facts.document_type.as_deref(), Some("Notice"));
        assert_eq!(facts.type_evidence, vec!["p1.b1"]);
        assert_eq!(facts.date_evidence, vec!["p1.b2"]);
        assert_eq!(facts.unknown_evidence, vec!["p7.b7"]);
        // Not a calendar date: no date, and no role for it.
        assert_eq!(facts.document_date, None);
        assert_eq!(facts.date_role, None);
        // A role that is not one of ours is no role, never "other"; a
        // blank name is no party.
        assert_eq!(facts.parties.len(), 2);
        assert_eq!(facts.parties[0].role, None);
        assert_eq!(facts.parties[0].evidence, vec!["p1.b2"]);
        assert_eq!(facts.parties[1].role, Some(PartyRole::Other));
        assert_eq!(facts.key_facts.len(), 2);
        assert!(facts.key_facts[0].evidence.is_empty());
        assert!(!proposal.needs_review);
    }

    #[test]
    fn an_evidence_reply_with_an_impossible_confidence_is_malformed() {
        let reply = r#"{"type_ids":[],"type":null,"date_ids":[],"date":null,"date_role":null,"parties":[],"confidence":7,"needs_review":false}"#;
        assert_eq!(
            proposal_from_text(reply, Some(&ordinal_handles()))
                .unwrap_err()
                .0,
            EngineErrorCode::ModelResponseInvalid
        );
    }

    /// The grammar is part of a request's identity only when the request
    /// carries its own, so every digest-pipeline recording stays valid.
    #[test]
    fn the_request_identity_covers_a_grammar_of_its_own() {
        let plain = ModelRequest::new("p");
        let mut expected = Sha256::new();
        expected.update(b"p");
        assert_eq!(plain.sha256(), format!("{:x}", expected.finalize()));
        let with = |grammar: &str| ModelRequest {
            grammar: Some(grammar.into()),
            ..ModelRequest::new("p")
        };
        assert_ne!(with("root ::= \"a\"").sha256(), plain.sha256());
        assert_ne!(
            with("root ::= \"a\"").sha256(),
            with("root ::= \"b\"").sha256()
        );
        // Stable handles are the ids the prompt shows, so their map
        // identifies nothing; ordinal numbers do not show what they stand
        // for, so theirs does.
        let mapped = |handles: EvidenceHandles| ModelRequest {
            evidence: Some(handles),
            ..with("root ::= \"a\"")
        };
        assert_eq!(
            mapped(stable_handles()).sha256(),
            with("root ::= \"a\"").sha256()
        );
        assert_ne!(
            mapped(ordinal_handles()).sha256(),
            with("root ::= \"a\"").sha256()
        );
    }

    /// The local server is sent the request's own grammar, and its reply is
    /// read as facts.
    #[test]
    fn a_local_evidence_request_sends_its_grammar_and_reads_ids() {
        let server = scripted_server(vec![completion_reply(
            "stop",
            r#"{"type_ids":[1],"type":"Invoice","date_ids":[],"date":null,"date_role":null,"parties":[],"confidence":0.7,"needs_review":false}"#,
        )]);
        let client = ModelClient::new(
            &format!("http://{}/v1/chat/completions", server.address),
            "k",
            "m",
        )
        .unwrap();
        let request = ModelRequest {
            prompt: "p".into(),
            grammar: Some("root ::= \"{}\"".into()),
            evidence: Some(ordinal_handles()),
            ..ModelRequest::default()
        };
        let proposal = client.propose(&request).unwrap();
        assert_eq!(
            proposal.facts.unwrap().type_evidence,
            vec!["p1.b1".to_owned()]
        );
        let body: Value = serde_json::from_str(&server.bodies()[0]).unwrap();
        assert_eq!(body["grammar"], json!("root ::= \"{}\""));
        // The digest pipeline's request still carries the fixed grammar.
        assert_eq!(
            client.completion_request(&ModelRequest::new("p"))["grammar"],
            json!(RESPONSE_GRAMMAR)
        );
    }

    /// In an evidence reply the date and every party's name are the values
    /// read; ids, roles and keys are not.
    #[test]
    fn token_confidence_reads_the_evidence_replys_date_and_names() {
        let tokens = pieces(&[
            (
                r#"{"type_ids":[1],"type":"Invoice","date_ids":[2],"date":""#,
                0.01,
            ),
            ("2025-05-01", 0.8),
            (
                r#"","date_role":"invoice","parties":[{"ids":[1],"name":""#,
                0.02,
            ),
            ("Acme", 0.6),
            (r#"","role":"issuer"},{"ids":[3],"name":""#, 0.03),
            ("Contoso", 0.4),
            (
                r#"","role":"customer"}],"confidence":0.9,"needs_review":false}"#,
                0.05,
            ),
        ]);
        let confidence = token_confidence(&tokens).unwrap();
        assert_eq!(confidence.tokens, 3);
        assert!(close(confidence.min, 0.4), "{confidence:?}");
        assert!(close(confidence.mean, 0.6), "{confidence:?}");
    }

    /// The compact reply the grammar writes states each fact as an array,
    /// value first: the date and each party's name are read; the roles and
    /// ids beside them, stable or ordinal, are not.
    #[test]
    fn token_confidence_reads_the_compact_replys_date_and_names() {
        for (date_id, first_id, second_id) in
            [(r#""p1.b2""#, r#""p1.b1""#, r#""p2.b3""#), ("4", "1", "7")]
        {
            let tokens = pieces(&[
                (r#"{"type":["Invoice","p1.b1"],"date":[""#, 0.01),
                ("2025-05-01", 0.8),
                (r#"",""#, 0.02),
                ("invoice", 0.1),
                (r#"","#, 0.02),
                (date_id, 0.15),
                (r#"],"parties":[[""#, 0.02),
                ("Acme", 0.6),
                (r#"",""#, 0.03),
                ("issuer", 0.05),
                (r#"","#, 0.03),
                (first_id, 0.07),
                (r#"],[""#, 0.03),
                ("Contoso", 0.4),
                (r#"",""#, 0.03),
                ("customer", 0.05),
                (r#"","#, 0.03),
                (second_id, 0.07),
                (r#"]]}"#, 0.05),
            ]);
            let confidence = token_confidence(&tokens).unwrap();
            assert_eq!(confidence.tokens, 3, "{date_id}: {confidence:?}");
            assert!(close(confidence.min, 0.4), "{date_id}: {confidence:?}");
            assert!(close(confidence.mean, 0.6), "{date_id}: {confidence:?}");
        }
    }
}
