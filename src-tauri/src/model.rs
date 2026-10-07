//! Which model reads documents, and the switch between them.
//!
//! The local model is the product and the default. A hosted model behind an
//! API key is an alternative a person chooses in Settings, knowing that the
//! distilled text of every document will leave the machine for the service
//! they named. Everything on either side of the model is shared: the same
//! distillation goes out, the same evidence checks are applied to what comes
//! back, and the queue never learns which one answered.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::Duration,
};

use intern_engine::{
    DocumentAnalysis, DocumentSource, Engine, EngineError, EngineErrorCode, HostedClient,
    HostedModelConfig, HostedProvider, evidence::is_valid_iso_date, hosted::endpoint_for,
};
use intern_queue::{AnalyzerBoundary, AppSettings, ModelFailure, ModelSource, SettingsStore};
use serde::Serialize;

use crate::{
    commands::CommandError,
    secrets::{HOSTED_MODEL_API_KEY, SecretStore, key_hint},
};

/// How long to wait before retrying after a hosted service asked for a
/// slower pace or briefly could not be reached, when it did not say how long.
/// Spread by a fifth either way, so a backlog does not knock in step.
const HOSTED_RETRY_DELAY: Duration = Duration::from_secs(8);
const HOSTED_RETRY_JITTER: f64 = 0.2;
/// The longest a document waits on a service's own `Retry-After` before the
/// one retry. A longer ask is answered by pausing the queue.
const MAX_HOSTED_RETRY_WAIT: Duration = Duration::from_secs(60);

/// What Settings shows about the hosted model.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostedModelStatusDto {
    /// Whether a key is in the credential store.
    pub key_stored: bool,
    /// The tail of the stored key, so a person can tell which one it is.
    pub key_hint: Option<String>,
    /// The endpoint the saved settings resolve to, when they resolve.
    pub endpoint: Option<String>,
    /// Each provider's defaults, so the dialog can show what "empty" means.
    pub providers: Vec<ProviderDefaultsDto>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDefaultsDto {
    pub provider: HostedProvider,
    pub base_url: String,
    pub model: String,
}

/// The outcome of a successful test connection: the calibration document
/// went out, and a correct name came back.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostedModelTestDto {
    pub model: String,
    pub endpoint: String,
    pub filename: String,
    pub inference_millis: u64,
}

/// The hosted model as the app holds it: the settings' half of the
/// configuration plus the key from the credential store, and an engine
/// built lazily and kept while the configuration stands.
pub struct HostedModel {
    secrets: Arc<dyn SecretStore>,
    engine: Mutex<Option<(HostedModelConfig, Arc<Engine>)>>,
    /// Whether someone has typed an API key into this window since Intern
    /// started. Entering a key is a person saying where it may go.
    key_entered: AtomicBool,
    /// The wait the service asked for in its last failure, kept for the
    /// recovery that follows it. The queue's failure carries only a code.
    retry_after: Mutex<Option<Duration>>,
}

impl HostedModel {
    pub fn new(secrets: Arc<dyn SecretStore>) -> Self {
        Self {
            secrets,
            engine: Mutex::new(None),
            key_entered: AtomicBool::new(false),
            retry_after: Mutex::new(None),
        }
    }

    /// The configuration the given settings and the stored key amount to,
    /// or why a request could not be made from them.
    pub fn config(&self, settings: &AppSettings) -> Result<HostedModelConfig, CommandError> {
        let api_key = self.stored_key()?.ok_or_else(|| CommandError {
            code: "HOSTED_MODEL_KEY_MISSING".into(),
            message: "no API key is stored for the hosted model".into(),
        })?;
        Ok(HostedModelConfig {
            provider: settings.hosted_provider,
            base_url: settings.hosted_base_url.clone(),
            model: settings.hosted_model.clone(),
            api_key,
        }
        .resolved()?)
    }

    /// Whether these settings and the stored key are enough to send with.
    pub fn configured(&self, settings: &AppSettings) -> bool {
        self.config(settings).is_ok()
    }

    pub fn status(&self, settings: &AppSettings) -> HostedModelStatusDto {
        let key = self.stored_key().ok().flatten();
        HostedModelStatusDto {
            key_stored: key.is_some(),
            key_hint: key.as_deref().map(key_hint),
            endpoint: self
                .config(settings)
                .ok()
                .and_then(|config| endpoint_for(config.provider, &config.base_url).ok())
                .map(|endpoint| endpoint.to_string()),
            providers: HostedProvider::ALL
                .iter()
                .map(|provider| ProviderDefaultsDto {
                    provider: *provider,
                    base_url: provider.default_base_url().to_owned(),
                    model: provider.default_model().to_owned(),
                })
                .collect(),
        }
    }

    /// Stores a key, replacing any earlier one. The engine built on the old
    /// key is dropped so the next document uses the new one.
    pub fn set_key(&self, key: &str) -> Result<(), CommandError> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(CommandError {
                code: "HOSTED_MODEL_KEY_EMPTY".into(),
                message: "the API key is empty".into(),
            });
        }
        self.secrets
            .set(HOSTED_MODEL_API_KEY, trimmed)
            .map_err(store_error)?;
        self.key_entered.store(true, Ordering::SeqCst);
        self.forget_engine();
        Ok(())
    }

    pub fn clear_key(&self) -> Result<(), CommandError> {
        self.secrets
            .delete(HOSTED_MODEL_API_KEY)
            .map_err(store_error)?;
        self.forget_engine();
        Ok(())
    }

    /// Whether the stored key may be sent to the address these settings name.
    ///
    /// The address comes from the dialog rather than the settings file, so a
    /// new one can be tested before it is saved - which means the window
    /// chooses where the key goes. A page that should not be in the window
    /// could choose too, so a key that was already on this machine when
    /// Intern started only ever goes to the address the settings file names.
    /// Typing a key admits the address it was typed for, which is the flow a
    /// person actually uses; nobody can type a key they do not have.
    fn may_send_to(&self, settings: &AppSettings, saved: &AppSettings) -> bool {
        if self.key_entered.load(Ordering::SeqCst) {
            return true;
        }
        let address = |settings: &AppSettings| {
            let base_url = match settings.hosted_base_url.trim() {
                "" => settings.hosted_provider.default_base_url(),
                given => given,
            };
            endpoint_for(settings.hosted_provider, base_url)
                .ok()
                .map(|endpoint| endpoint.to_string())
        };
        address(settings).is_some_and(|wanted| address(saved) == Some(wanted))
    }

    /// Sends the calibration document, the way the local model is checked
    /// at setup, so a wrong key, model, or address is found here rather than
    /// on someone's first real document. `saved` is the settings as stored.
    pub fn test(
        &self,
        settings: &AppSettings,
        saved: &AppSettings,
    ) -> Result<HostedModelTestDto, CommandError> {
        let config = self.config(settings)?;
        if !self.may_send_to(settings, saved) {
            return Err(CommandError {
                code: "HOSTED_MODEL_ADDRESS_UNCONFIRMED".into(),
                message: "enter the API key for this address before testing it".into(),
            });
        }
        let client = HostedClient::new(config)?;
        let analysis = client.probe()?;
        Ok(HostedModelTestDto {
            model: client.model().to_owned(),
            endpoint: client.endpoint().to_string(),
            filename: analysis.filename,
            inference_millis: analysis.telemetry.inference_millis,
        })
    }

    fn analyze(
        &self,
        settings: &AppSettings,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        let config = self
            .config(settings)
            .map_err(|error| ModelFailure::fatal(error.code))?;
        let engine = self.engine(config).map_err(failure_for)?;
        let result = engine.analyze(source, extension, existing_names);
        *self
            .retry_after
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            result.as_ref().err().and_then(EngineError::retry_after);
        result.map_err(failure_for)
    }

    /// The wait the last failure asked for, once.
    fn take_retry_after(&self) -> Option<Duration> {
        self.retry_after
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    fn engine(&self, config: HostedModelConfig) -> Result<Arc<Engine>, EngineError> {
        let mut cached = self
            .engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((current, engine)) = cached.as_ref()
            && *current == config
        {
            return Ok(Arc::clone(engine));
        }
        let client = HostedClient::new(config.clone())?;
        let engine = Arc::new(client.engine());
        *cached = Some((config, Arc::clone(&engine)));
        Ok(engine)
    }

    fn forget_engine(&self) {
        *self
            .engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    fn stored_key(&self) -> Result<Option<String>, CommandError> {
        self.secrets.get(HOSTED_MODEL_API_KEY).map_err(store_error)
    }
}

fn store_error(detail: String) -> CommandError {
    CommandError {
        code: "SECRET_STORE_UNAVAILABLE".into(),
        message: format!("the credential store could not be used ({detail})"),
    }
}

/// How a hosted failure reads to the queue: a refused key or a declined
/// document will not change on a retry, so those are fatal; a busy or
/// unreachable service and a malformed reply are worth one more attempt.
pub(crate) fn failure_for(error: EngineError) -> ModelFailure {
    match error.code() {
        EngineErrorCode::HostedModelRateLimited
        | EngineErrorCode::HostedModelUnreachable
        | EngineErrorCode::ModelResponseInvalid => ModelFailure::retryable(error.code().as_str()),
        _ => ModelFailure::fatal(error.code().as_str()),
    }
}

/// The date the model proposed but validation withheld - because the
/// document never states it verbatim - offered to the reviewer to accept
/// with one click. Nothing is offered when a date was accepted, or when the
/// model gave none.
pub(crate) fn suggested_date(analysis: &DocumentAnalysis) -> Option<String> {
    if analysis.proposal.document_date.is_some() {
        return None;
    }
    analysis
        .model_proposal
        .as_ref()?
        .document_date
        .as_deref()
        .map(str::trim)
        .filter(|date| is_valid_iso_date(date))
        .map(str::to_owned)
}

const IDLE: u8 = 0;
const LOCAL: u8 = 1;
const HOSTED: u8 = 2;

/// The queue's model: whichever one the settings name at the moment a
/// document is analysed, so a change in Settings takes effect at the next
/// document with nothing restarted.
pub struct SwitchingModel<L: AnalyzerBoundary> {
    local: Arc<L>,
    hosted: Arc<HostedModel>,
    settings: SettingsStore,
    active: AtomicU8,
    /// How a retry waits: the thread sleeps, except under test.
    sleep: Box<dyn Fn(Duration) + Send + Sync>,
}

impl<L: AnalyzerBoundary> SwitchingModel<L> {
    pub fn new(local: Arc<L>, hosted: Arc<HostedModel>, settings: SettingsStore) -> Self {
        Self {
            local,
            hosted,
            settings,
            active: AtomicU8::new(IDLE),
            sleep: Box::new(std::thread::sleep),
        }
    }

    #[cfg(test)]
    fn with_sleeper(mut self, sleep: impl Fn(Duration) + Send + Sync + 'static) -> Self {
        self.sleep = Box::new(sleep);
        self
    }
}

/// How long to wait before the one retry of a busy or unreachable hosted
/// service: what it asked for, up to [`MAX_HOSTED_RETRY_WAIT`], or the fixed
/// delay moved by `jitter` (a fraction, within [`HOSTED_RETRY_JITTER`]) when it
/// did not ask. A service that names its wait and is retried sooner only
/// refuses again, and its quota window is what decides when the queue moves.
fn retry_wait(asked: Option<Duration>, jitter: f64) -> Duration {
    match asked {
        Some(asked) => asked.min(MAX_HOSTED_RETRY_WAIT),
        None => HOSTED_RETRY_DELAY
            .mul_f64(1.0 + jitter.clamp(-HOSTED_RETRY_JITTER, HOSTED_RETRY_JITTER)),
    }
}

impl<L: AnalyzerBoundary> AnalyzerBoundary for SwitchingModel<L> {
    fn analyze(
        &self,
        source: &DocumentSource,
        extension: &str,
        existing_names: &[&str],
    ) -> Result<DocumentAnalysis, ModelFailure> {
        let settings = self
            .settings
            .load()
            .map_err(|error| ModelFailure::fatal(error.code))?;
        let (which, result) = match settings.model_source {
            ModelSource::Local => {
                self.active.store(LOCAL, Ordering::SeqCst);
                (LOCAL, self.local.analyze(source, extension, existing_names))
            }
            ModelSource::Hosted => {
                self.active.store(HOSTED, Ordering::SeqCst);
                (
                    HOSTED,
                    self.hosted
                        .analyze(&settings, source, extension, existing_names),
                )
            }
        };
        let _ = self
            .active
            .compare_exchange(which, IDLE, Ordering::SeqCst, Ordering::SeqCst);
        result
    }

    fn recover(&self, failure: &ModelFailure) -> Result<(), ModelFailure> {
        match failure.code.as_str() {
            "HOSTED_MODEL_RATE_LIMITED" | "HOSTED_MODEL_UNREACHABLE" => {
                use rand::Rng as _;
                let jitter =
                    rand::thread_rng().gen_range(-HOSTED_RETRY_JITTER..=HOSTED_RETRY_JITTER);
                (self.sleep)(retry_wait(self.hosted.take_retry_after(), jitter));
                Ok(())
            }
            code if code.starts_with("HOSTED_MODEL_") => Ok(()),
            _ => self.local.recover(failure),
        }
    }

    /// A hosted request cannot be interrupted, only outwaited: the reply is
    /// discarded when it arrives. The local server is restarted as before.
    fn cancel(&self) -> Result<(), ModelFailure> {
        match self.active.load(Ordering::SeqCst) {
            HOSTED => Ok(()),
            _ => self.local.cancel(),
        }
    }

    fn shutdown(&self) -> Result<(), ModelFailure> {
        self.local.shutdown()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        path::PathBuf,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use intern_engine::{
        AnalysisTelemetry, DateRole, DocumentAnalysis, EngineError, EngineErrorCode, Evidence,
        HostedProvider, ModelProposal, PartyRelation, ProposalStatus, ValidatedProposal,
    };
    use intern_queue::{AppSettings, ModelSource, SettingsStore};

    use super::{HostedModel, SwitchingModel, failure_for, retry_wait, suggested_date};
    use crate::secrets::{HOSTED_MODEL_API_KEY, MemoryStore, SecretStore};

    fn analysis(accepted: Option<&str>, proposed: Option<&str>) -> DocumentAnalysis {
        let proposal = ValidatedProposal {
            document_type: Some("Invoice".into()),
            document_date: accepted.map(str::to_owned),
            date_role: accepted.map(|_| DateRole::Invoice),
            parties: vec!["Acme".into()],
            party_relation: PartyRelation::From,
            description: "An invoice from Acme.".into(),
            confidence: 0.8,
            evidence: Evidence::default(),
        };
        DocumentAnalysis {
            filename: "Invoice from Acme.pdf".into(),
            description: proposal.description.clone(),
            status: ProposalStatus::NeedsReview,
            review_reasons: Vec::new(),
            proposal,
            telemetry: AnalysisTelemetry::default(),
            text_fingerprint: None,
            token_confidence: None,
            model_proposal: Some(ModelProposal {
                document_type: Some("Invoice".into()),
                document_date: proposed.map(str::to_owned),
                date_role: proposed.map(|_| DateRole::Invoice),
                parties: vec!["Acme".into()],
                party_relation: PartyRelation::From,
                description: "An invoice from Acme.".into(),
                confidence: 0.8,
                needs_review: false,
                evidence: Evidence::default(),
                facts: None,
            }),
            stated_dates: Vec::new(),
            facts: None,
        }
    }

    #[test]
    fn a_withheld_date_is_offered_and_an_accepted_or_absent_one_is_not() {
        assert_eq!(
            suggested_date(&analysis(None, Some("2026-03-02"))).as_deref(),
            Some("2026-03-02")
        );
        assert_eq!(
            suggested_date(&analysis(Some("2026-03-02"), Some("2026-03-02"))),
            None,
            "already in the filename"
        );
        assert_eq!(suggested_date(&analysis(None, None)), None);
        assert_eq!(
            suggested_date(&analysis(None, Some("2026-02-30"))),
            None,
            "never a day that does not exist"
        );
        let mut legacy = analysis(None, Some("2026-03-02"));
        legacy.model_proposal = None;
        assert_eq!(
            suggested_date(&legacy),
            None,
            "stored before replies were kept"
        );
    }

    #[test]
    fn only_a_busy_service_or_a_malformed_reply_earns_a_retry() {
        for code in [
            EngineErrorCode::HostedModelRateLimited,
            EngineErrorCode::HostedModelUnreachable,
            EngineErrorCode::ModelResponseInvalid,
        ] {
            assert!(
                failure_for(EngineError::new(code, "x")).retryable,
                "{code:?}"
            );
        }
        for code in [
            EngineErrorCode::HostedModelUnauthorized,
            EngineErrorCode::HostedModelMisconfigured,
            EngineErrorCode::HostedModelRejected,
            EngineErrorCode::HostedModelRefused,
            // No retry changes an empty balance, a document too large for
            // the window, or a reply cut off at the same token again.
            EngineErrorCode::HostedModelBilling,
            EngineErrorCode::ModelInputTooLarge,
            EngineErrorCode::ModelReplyTruncated,
        ] {
            let failure = failure_for(EngineError::new(code, "x"));
            assert!(!failure.retryable, "{code:?}");
            assert_eq!(failure.code, code.as_str());
        }
    }

    struct NoLocalModel;

    impl intern_queue::AnalyzerBoundary for NoLocalModel {
        fn analyze(
            &self,
            _source: &intern_engine::DocumentSource,
            _extension: &str,
            _existing_names: &[&str],
        ) -> Result<DocumentAnalysis, intern_queue::ModelFailure> {
            panic!("the hosted model is the one chosen")
        }
    }

    /// A hosted service on this machine that answers every request with
    /// `reply`, the way the queue would meet it.
    fn hosted_answering(
        name: &str,
        reply: &'static str,
    ) -> (
        SwitchingModel<NoLocalModel>,
        Arc<Mutex<Vec<Duration>>>,
        PathBuf,
    ) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
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
                let _ = reader.read_exact(&mut vec![0_u8; length]);
                let _ = stream.write_all(reply.as_bytes());
            }
        });

        let data = std::env::temp_dir().join(format!("intern-{name}-{}", std::process::id()));
        let settings = SettingsStore::new(data.join("settings.json"));
        settings
            .save(&AppSettings {
                model_source: ModelSource::Hosted,
                hosted_provider: HostedProvider::OpenAiCompatible,
                hosted_base_url: format!("http://{address}/v1"),
                hosted_model: "local-model".into(),
                ..AppSettings::default()
            })
            .unwrap();
        let secrets: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());
        secrets.set(HOSTED_MODEL_API_KEY, "sk-test").unwrap();
        let slept = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&slept);
        let model = SwitchingModel::new(
            Arc::new(NoLocalModel),
            Arc::new(HostedModel::new(secrets)),
            settings,
        )
        .with_sleeper(move |wait| recorded.lock().unwrap().push(wait));
        (model, slept, data)
    }

    /// One recovery after one failure, and how long it slept.
    fn recovered_after(name: &str, reply: &'static str) -> (String, Duration) {
        use intern_queue::AnalyzerBoundary as _;

        let (model, slept, data) = hosted_answering(name, reply);
        let source = intern_engine::distill::source_from_text(
            "MEMO\n\nThe office moves to the fourth floor on March 2, 2026.",
        );
        let failure = model.analyze(&source, "pdf", &[]).unwrap_err();
        assert!(failure.retryable, "{}", failure.code);
        model.recover(&failure).unwrap();
        let _ = std::fs::remove_dir_all(data);
        let slept = slept.lock().unwrap().clone();
        assert_eq!(slept.len(), 1, "{slept:?}");
        (failure.code, slept[0])
    }

    /// A service that says how long to wait is waited for - up to a minute,
    /// beyond which the queue's pause is the better answer - and one that
    /// does not is given the usual delay, moved a little so a backlog of
    /// retries does not arrive in step.
    #[test]
    fn recover_sleeps_for_retry_after_capped() {
        let (code, slept) = recovered_after(
            "retry-after-7",
            "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 7\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
        assert_eq!(code, "HOSTED_MODEL_RATE_LIMITED");
        assert_eq!(slept, Duration::from_secs(7));

        let (code, slept) = recovered_after(
            "retry-after-capped",
            "HTTP/1.1 503 Service Unavailable\r\nRetry-After: 3600\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
        assert_eq!(code, "HOSTED_MODEL_UNREACHABLE");
        assert_eq!(slept, Duration::from_secs(60));

        let (code, slept) = recovered_after(
            "retry-after-none",
            "HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
        assert_eq!(code, "HOSTED_MODEL_RATE_LIMITED");
        assert!(
            (Duration::from_millis(6_400)..=Duration::from_millis(9_600)).contains(&slept),
            "{slept:?}"
        );
    }

    #[test]
    fn the_retry_wait_honours_the_service_and_otherwise_jitters_the_default() {
        assert_eq!(
            retry_wait(Some(Duration::from_secs(7)), 0.2),
            Duration::from_secs(7)
        );
        assert_eq!(
            retry_wait(Some(Duration::from_secs(120)), 0.0),
            Duration::from_secs(60)
        );
        assert_eq!(retry_wait(None, 0.0), Duration::from_secs(8));
        assert_eq!(retry_wait(None, -0.2), Duration::from_millis(6_400));
        assert_eq!(retry_wait(None, 0.2), Duration::from_millis(9_600));
        assert_eq!(
            retry_wait(None, 5.0),
            Duration::from_millis(9_600),
            "clamped"
        );
    }

    /// A request that times out is retried once, after at most the longest
    /// wait a retry takes. Both attempts and that wait must end inside the
    /// queue's deadline: past it, the queue gives up on the document and
    /// leaves the retry running - on a server on this machine, in front of
    /// the next document and of this one, claimed again at once.
    #[test]
    fn a_timed_out_request_and_its_retry_end_inside_the_queue_deadline() {
        let deadline = Duration::from_secs(intern_queue::pipeline::MODEL_TIMEOUT_SECONDS);
        for endpoint in ["http://localhost:11434/v1", "https://api.openai.com/v1"] {
            let timeout =
                intern_engine::hosted::request_timeout_for(&url::Url::parse(endpoint).unwrap());
            assert!(
                timeout * 2 + super::MAX_HOSTED_RETRY_WAIT < deadline,
                "{endpoint}: two attempts of {timeout:?} and the wait outlast {deadline:?}"
            );
        }
    }

    #[test]
    fn a_key_from_an_earlier_session_only_goes_to_the_saved_address() {
        let secrets: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());
        let hosted = HostedModel::new(Arc::clone(&secrets));
        // A key that was already on this machine when Intern started.
        secrets
            .set(HOSTED_MODEL_API_KEY, "sk-ant-api03-example-key-0042")
            .unwrap();
        let saved = AppSettings {
            model_source: ModelSource::Hosted,
            ..AppSettings::default()
        };
        assert!(hosted.may_send_to(&saved, &saved), "the saved address");

        let elsewhere = AppSettings {
            hosted_base_url: "https://collector.example.com/v1".into(),
            ..saved.clone()
        };
        assert!(!hosted.may_send_to(&elsewhere, &saved));
        assert_eq!(
            hosted
                .test(&elsewhere, &saved)
                .expect_err("an address nobody named is refused")
                .code,
            "HOSTED_MODEL_ADDRESS_UNCONFIRMED"
        );

        // Typing the key is the person saying where it may go, which is how a
        // new address is tested before it is saved.
        hosted.set_key("sk-ant-api03-example-key-0042").unwrap();
        assert!(hosted.may_send_to(&elsewhere, &saved));
    }

    #[test]
    fn the_hosted_model_is_configured_only_with_a_key_and_a_usable_address() {
        let secrets: Arc<dyn SecretStore> = Arc::new(MemoryStore::default());
        let hosted = HostedModel::new(Arc::clone(&secrets));
        let settings = AppSettings {
            model_source: ModelSource::Hosted,
            ..AppSettings::default()
        };

        assert!(!hosted.configured(&settings));
        assert_eq!(
            hosted.config(&settings).unwrap_err().code,
            "HOSTED_MODEL_KEY_MISSING"
        );
        let status = hosted.status(&settings);
        assert!(!status.key_stored);
        assert_eq!(status.endpoint, None);
        assert_eq!(status.providers.len(), 2);

        assert_eq!(
            hosted.set_key("   ").unwrap_err().code,
            "HOSTED_MODEL_KEY_EMPTY"
        );
        hosted.set_key("  sk-ant-api03-example-key-0042  ").unwrap();
        assert_eq!(
            secrets.get(HOSTED_MODEL_API_KEY).unwrap().as_deref(),
            Some("sk-ant-api03-example-key-0042"),
            "stored trimmed, in the credential store, never in settings"
        );
        assert!(hosted.configured(&settings));
        let status = hosted.status(&settings);
        assert!(status.key_stored);
        assert_eq!(status.key_hint.as_deref(), Some("…0042"));
        assert_eq!(
            status.endpoint.as_deref(),
            Some("https://api.anthropic.com/v1/messages")
        );

        let mut plain_http = settings.clone();
        plain_http.hosted_base_url = "http://api.example.com/v1".into();
        assert_eq!(
            hosted.config(&plain_http).unwrap_err().code,
            "HOSTED_MODEL_MISCONFIGURED"
        );

        hosted.clear_key().unwrap();
        assert!(!hosted.configured(&settings));
        assert_eq!(secrets.get(HOSTED_MODEL_API_KEY).unwrap(), None);
    }
}
