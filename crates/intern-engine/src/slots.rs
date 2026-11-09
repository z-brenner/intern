//! The system turn's state, kept from one model server to the next.
//!
//! Every evidence request opens with the same system turn, 477 tokens of
//! instructions. llama-server keeps their state between documents in a
//! context checkpoint it takes at the start of the user turn, so only the
//! first document of a server lifetime processes them: a load at sign-in,
//! and a load after every idle unload ([`crate::tiers::IDLE_UNLOAD_AFTER`]).
//! On the 9B that first document took 11 to 36 seconds longer on the
//! runners than every later one.
//!
//! So the server is launched with a slot directory, its slot is saved to a
//! file after the first document it reads, and every later launch restores
//! that file before the first document arrives. The file's name says
//! everything the saved state depends on ([`SlotIdentity`]), so a file is
//! only ever restored into a server that would have computed the same state
//! itself, and the next document's reply is the one it would have got
//! anyway. Anything that goes wrong costs only the speed-up: a restore that
//! fails leaves the server processing the system turn as it always did, and
//! the file is removed so the next document saves a good one.

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::error::{EngineError, EngineErrorCode, EngineResult};

/// The folder slot files are kept in, under the app's local data.
pub const SLOT_FOLDER: &str = "slots";

const SLOT_EXTENSION: &str = "bin";
/// A save is written under this extension and renamed when it is complete,
/// so a server stopped in the middle of one never leaves a file that looks
/// whole.
const PARTIAL_EXTENSION: &str = "part";
/// A save leaves at least this much of the disk free, or is not made: a
/// file is 174 MB for either model (the instructions' 492 tokens), and the
/// speed-up is not worth a full disk.
const MIN_FREE_AFTER_SAVE: u64 = 2 * 1024 * 1024 * 1024;
/// Saving or restoring moves a few hundred megabytes; a cold disk on a busy
/// laptop is given far longer than that takes.
const TRANSFER_TIMEOUT: Duration = Duration::from_secs(120);
const QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Everything the saved state depends on, hashed into the file's name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotIdentity<'a> {
    /// Which model's file it is: one file per label is kept.
    pub label: &'a str,
    /// The model file's SHA-256.
    pub model_sha256: &'a str,
    /// The server's own build, as `/props` reports it ([`runtime_build`]).
    pub runtime_build: &'a str,
    /// What the server caches before each document's own turn
    /// ([`crate::client::cached_prefix_identity`]).
    pub prefix: &'a str,
    /// The server's launch arguments, apart from the port, the key, the
    /// model's path and the slot folder: the thread count, the context size,
    /// the batch size and every flag that shapes the arithmetic.
    pub arguments: &'a [String],
    /// The processor's instruction set ([`cpu_fingerprint`]), which picks
    /// the CPU backend that computed the state.
    pub cpu: &'a str,
}

impl SlotIdentity<'_> {
    /// `<label>-<16 hex digits>.bin`: a name llama-server accepts (no path
    /// separator, no `..`), and the same for the same configuration on every
    /// launch.
    pub fn file_name(&self) -> String {
        let mut hasher = Sha256::new();
        for part in [
            self.label,
            self.model_sha256,
            self.runtime_build,
            self.prefix,
            self.cpu,
        ] {
            hasher.update(part.as_bytes());
            hasher.update([0_u8]);
        }
        for argument in self.arguments {
            hasher.update(argument.as_bytes());
            hasher.update([0_u8, 1_u8]);
        }
        let digest = format!("{:x}", hasher.finalize());
        format!("{}-{}.{SLOT_EXTENSION}", self.label, &digest[..16])
    }
}

/// The processor features that choose which of ggml's CPU backends a server
/// loads, and so how it computes. Detectable on the Rust release the
/// workspace pins; AVX-VNNI is not, and changes no result here (it only adds
/// exact integer dot products the AVX2 path computes the same).
pub fn cpu_fingerprint() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        let mut features = String::from("x86_64");
        macro_rules! probe {
            ($($name:tt),*) => {$(
                if std::arch::is_x86_feature_detected!($name) {
                    features.push(' ');
                    features.push_str($name);
                }
            )*};
        }
        probe!(
            "sse4.2",
            "avx",
            "f16c",
            "fma",
            "avx2",
            "bmi2",
            "avx512f",
            "avx512bw",
            "avx512vl",
            "avx512vbmi",
            "avx512vnni",
            "avx512bf16"
        );
        features
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        std::env::consts::ARCH.to_owned()
    }
}

/// Whether llama-server can be pointed at `directory`.
///
/// The server checks the folder with `std::filesystem` from the UTF-8 text
/// of its arguments, which Windows' C++ library reads in the ANSI code page:
/// a folder under a user name with an accent would not be found, and the
/// server would refuse to start. There only a folder named in ASCII is used.
pub fn usable_directory(directory: &Path) -> bool {
    let named = directory.to_str().is_some_and(|text| !text.is_empty());
    named && (!cfg!(windows) || directory.to_str().is_some_and(str::is_ascii))
}

/// The build a running server reports, e.g. `b11946-8ca4060b`.
pub fn runtime_build(endpoint: &str, api_key: &str) -> EngineResult<String> {
    #[derive(Deserialize)]
    struct Props {
        build_info: String,
    }
    let response = http(QUERY_TIMEOUT)?
        .get(format!("{endpoint}/props"))
        .bearer_auth(api_key)
        .send()
        .map_err(|_| unavailable())?;
    if !response.status().is_success() {
        return Err(unavailable());
    }
    let props: Props = response.json().map_err(|_| unavailable())?;
    let build = props.build_info.trim();
    if build.is_empty() {
        return Err(unavailable());
    }
    Ok(build.to_owned())
}

/// What a launch's restore did.
#[derive(Clone, Debug, PartialEq)]
pub enum Restore {
    /// The server holds the saved state: `tokens` of it, read from `bytes`
    /// of file in `millis`.
    Restored {
        tokens: u64,
        bytes: u64,
        millis: f64,
    },
    /// No file for this configuration yet: the first document saves one.
    NoFile,
    /// The server could not read the file. It was removed, and the first
    /// document saves a new one.
    Failed,
}

/// What a document's save did.
#[derive(Clone, Debug, PartialEq)]
pub enum Save {
    Saved {
        tokens: u64,
        bytes: u64,
        millis: f64,
    },
    /// This server lifetime has a file already: restored, saved, or tried.
    NotNeeded,
    /// This lifetime's instructions were not processed in one piece, so
    /// their state is not the one a file may carry ([`SlotKeeper::save_once`]).
    /// The next lifetime tries again.
    Uneven,
    /// Too little room on the disk.
    NoRoom,
    /// The server saved more than the instructions-only request left in
    /// its slot - `tokens` against that request's `request_tokens` - so the
    /// file may hold part of the document read before it. It was deleted.
    HeldMore {
        tokens: u64,
        request_tokens: u64,
    },
    Failed,
}

/// The instructions' length in tokens for each configuration, by slot file
/// name, once a lifetime of it has sent the instructions-only request in
/// this session: a later lifetime whose first document is too short to save
/// from then knows it without sending the request again.
#[derive(Debug, Default)]
pub struct KnownInstructions(Mutex<HashMap<String, u64>>);

impl KnownInstructions {
    fn get(&self, file_name: &str) -> Option<u64> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(file_name)
            .copied()
    }

    fn record(&self, file_name: &str, tokens: u64) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(file_name.to_owned(), tokens);
    }
}

/// What the instructions-only request found cached, and what it processed.
struct InstructionsOnly {
    cached: u64,
    processed: u64,
}

/// One server lifetime's slot file: restored at launch, saved once after the
/// first document when there was none.
#[derive(Debug)]
pub struct SlotKeeper {
    http: Client,
    endpoint: String,
    api_key: String,
    directory: PathBuf,
    label: String,
    file_name: String,
    /// Nothing for this configuration is on disk, or what was would not
    /// restore: the next document's state is worth saving.
    unsaved: AtomicBool,
    /// [`MIN_FREE_AFTER_SAVE`], apart from tests, which run on any disk.
    min_free: u64,
    /// The request sent just before a save, which leaves the slot holding
    /// the instructions and no document
    /// ([`crate::client::instructions_only_request`]).
    instructions_only: Value,
    /// The server's micro-batch (`-ub`), which decides where it breaks a
    /// prompt it processes from nothing.
    micro_batch: u64,
    known: Arc<KnownInstructions>,
}

impl SlotKeeper {
    /// For the server at `endpoint`, which was started with `directory` as
    /// its `--slot-save-path` and `micro_batch` as its `-ub`.
    /// `instructions_only` is the chat completion sent just before a save,
    /// so the file holds the instructions and none of the document read
    /// before it; `known` is shared by every lifetime of the session.
    pub fn new(
        endpoint: &str,
        api_key: &str,
        directory: &Path,
        identity: &SlotIdentity<'_>,
        instructions_only: Value,
        micro_batch: u64,
        known: Arc<KnownInstructions>,
    ) -> EngineResult<Self> {
        Ok(Self {
            http: http(TRANSFER_TIMEOUT)?,
            endpoint: endpoint.trim_end_matches('/').to_owned(),
            api_key: api_key.to_owned(),
            directory: directory.to_path_buf(),
            label: identity.label.to_owned(),
            file_name: identity.file_name(),
            unsaved: AtomicBool::new(true),
            min_free: MIN_FREE_AFTER_SAVE,
            instructions_only,
            micro_batch,
            known,
        })
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn path(&self) -> PathBuf {
        self.directory.join(&self.file_name)
    }

    /// Removes this label's files for any other configuration - a model,
    /// runtime, prompt or thread count that no longer runs - and any save
    /// a stopped server left unfinished. Only one model's server runs at a
    /// time, so no save of another label can be under way.
    pub fn prune(&self) {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return;
        };
        let own = format!("{}-", self.label);
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let path = entry.path();
            let partial = path.extension() == Some(OsStr::new(PARTIAL_EXTENSION));
            let stale = name.starts_with(&own)
                && name != self.file_name
                && path.extension() == Some(OsStr::new(SLOT_EXTENSION));
            if (partial || stale) && entry.file_type().is_ok_and(|kind| kind.is_file()) {
                let _ = fs::remove_file(path);
            }
        }
    }

    /// Puts the saved state back into the server's slot, before its first
    /// document. Asks nothing of a server when there is no file.
    pub fn restore(&self) -> Restore {
        if !self.path().is_file() {
            return Restore::NoFile;
        }
        #[derive(Deserialize)]
        struct Restored {
            n_restored: u64,
            n_read: u64,
            timings: RestoreTimings,
        }
        #[derive(Deserialize)]
        struct RestoreTimings {
            restore_ms: f64,
        }
        match self.slot_action::<Restored>("restore", &self.file_name) {
            Ok(restored) => {
                self.unsaved.store(false, Ordering::SeqCst);
                Restore::Restored {
                    tokens: restored.n_restored,
                    bytes: restored.n_read,
                    millis: restored.timings.restore_ms,
                }
            }
            Err(_) => {
                let _ = fs::remove_file(self.path());
                Restore::Failed
            }
        }
    }

    /// Saves the slot once per server lifetime, after its first document,
    /// when no file for this configuration was there to restore.
    /// `first_request_tokens` is the prompt the server processed from
    /// nothing for that document - its whole prompt, nothing cached - or
    /// `None` when the document was not the lifetime's first request.
    ///
    /// Only an even state is saved. llama-server processes a prompt in
    /// micro-batches, and breaks the batch where the document's own turn
    /// starts and 4 and 4 + `-ub` tokens before the end. A first request
    /// shorter than the instructions plus 4 + `-ub` tokens has a break
    /// inside the instructions, which then come out a little different -
    /// enough to change a later reply (measured on the 9B: the date role
    /// and the subject of a purchase order). A lifetime's instructions stay
    /// as its first request computed them, so a file is saved only from a
    /// lifetime whose first request was a document long enough to process
    /// them in one piece, the way most documents do; restored, it gives
    /// every lifetime the state such a document gives, whatever is read
    /// first.
    ///
    /// The slot then holds that document: its tokens, and the state the
    /// model built from them. Saved as it is, a person's document would
    /// outlive its filing in a file of Intern's. So the instructions-only
    /// request goes first: it reuses the instructions' state the document
    /// left, and replaces the document's tokens with two fixed words. What
    /// is saved is the instructions, and the next document reuses them
    /// exactly as it would have reused them after the document. The server
    /// says how many tokens it saved; a file with more than that request's
    /// prompt and the one token it generated is deleted at once
    /// ([`Save::HeldMore`]), so a runtime that kept more of the slot than
    /// this one would leave nothing of a document on disk.
    ///
    /// The instructions' length is what that request finds cached. Until a
    /// lifetime of this configuration has sent it, a short first document
    /// costs the request (one generated token: 1.1-1.5 s on the runners)
    /// before it is found too short; after that the length is known for the
    /// rest of the session ([`KnownInstructions`]).
    pub fn save_once(&self, first_request_tokens: Option<u64>) -> Save {
        if !self.unsaved.swap(false, Ordering::SeqCst) || self.path().is_file() {
            return Save::NotNeeded;
        }
        let Some(first_request_tokens) = first_request_tokens else {
            return Save::Uneven;
        };
        if self
            .known
            .get(&self.file_name)
            .is_some_and(|instructions| !self.even(first_request_tokens, instructions))
        {
            return Save::Uneven;
        }
        let room = fs4::available_space(&self.directory).unwrap_or(0);
        if room < self.min_free {
            return Save::NoRoom;
        }
        let partial_name = format!("{}.{PARTIAL_EXTENSION}", self.file_name);
        let partial = self.directory.join(&partial_name);
        let request = match self.forget_the_document() {
            Ok(request) if request.cached > 0 => request,
            _ => {
                let _ = fs::remove_file(&partial);
                return Save::Failed;
            }
        };
        self.known.record(&self.file_name, request.cached);
        if !self.even(first_request_tokens, request.cached) {
            return Save::Uneven;
        }
        #[derive(Deserialize)]
        struct Saved {
            n_saved: u64,
            n_written: u64,
            timings: SaveTimings,
        }
        #[derive(Deserialize)]
        struct SaveTimings {
            save_ms: f64,
        }
        let Ok(saved) = self.slot_action::<Saved>("save", &partial_name) else {
            let _ = fs::remove_file(&partial);
            return Save::Failed;
        };
        let request_tokens = request.cached + request.processed;
        if saved.n_saved > request_tokens + 1 {
            let _ = fs::remove_file(&partial);
            return Save::HeldMore {
                tokens: saved.n_saved,
                request_tokens,
            };
        }
        if !(partial.is_file() && fs::rename(&partial, self.path()).is_ok()) {
            let _ = fs::remove_file(&partial);
            return Save::Failed;
        }
        Save::Saved {
            tokens: saved.n_saved,
            bytes: saved.n_written,
            millis: saved.timings.save_ms,
        }
    }

    /// Whether a lifetime whose first request was `first_request_tokens`
    /// long processed `instructions` tokens of instructions in one piece.
    fn even(&self, first_request_tokens: u64, instructions: u64) -> bool {
        first_request_tokens >= instructions + 4 + self.micro_batch
    }

    /// Sends the instructions-only request, and says how many of its
    /// tokens the server found cached and how many it processed.
    fn forget_the_document(&self) -> EngineResult<InstructionsOnly> {
        #[derive(Deserialize)]
        struct Completion {
            timings: CompletionTimings,
        }
        #[derive(Deserialize)]
        struct CompletionTimings {
            cache_n: u64,
            prompt_n: u64,
        }
        let response = self
            .http
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .bearer_auth(&self.api_key)
            .json(&self.instructions_only)
            .send()
            .map_err(|_| unavailable())?;
        if !response.status().is_success() {
            return Err(unavailable());
        }
        let completion: Completion = response.json().map_err(|_| unavailable())?;
        Ok(InstructionsOnly {
            cached: completion.timings.cache_n,
            processed: completion.timings.prompt_n,
        })
    }

    fn slot_action<T: serde::de::DeserializeOwned>(
        &self,
        action: &str,
        file_name: &str,
    ) -> EngineResult<T> {
        let response = self
            .http
            .post(format!("{}/slots/0?action={action}", self.endpoint))
            .bearer_auth(&self.api_key)
            .json(&json!({ "filename": file_name }))
            .send()
            .map_err(|_| unavailable())?;
        if !response.status().is_success() {
            return Err(unavailable());
        }
        response.json().map_err(|_| unavailable())
    }
}

/// Loopback HTTP only; see `ModelClient::new` for why the operating system's
/// certificate store is left unread.
fn http(timeout: Duration) -> EngineResult<Client> {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(timeout)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .tls_built_in_native_certs(false)
        .build()
        .map_err(|_| unavailable())
}

const fn unavailable() -> EngineError {
    EngineError::new(
        EngineErrorCode::ModelRequestFailed,
        "the local model server's slot could not be saved or restored",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{RoutedRequest, routed_server};

    /// A first request long enough to process the fake server's 477 tokens
    /// of instructions in one piece: 477 + 4 + 256.
    const LONG_FIRST_REQUEST: Option<u64> = Some(737);

    fn identity<'a>(arguments: &'a [String]) -> SlotIdentity<'a> {
        SlotIdentity {
            label: "accurate",
            model_sha256: "03b74727",
            runtime_build: "b11946-8ca4060b",
            prefix: "prefix",
            arguments,
            cpu: "x86_64 avx2",
        }
    }

    /// What the fake server's instructions-only request finds: 477 tokens
    /// cached and 11 processed, as the real one finds 477 and 15.
    const REQUEST_TOKENS: u64 = 477 + 11;

    /// A fake llama-server's slot endpoints over `directory`: a save writes
    /// the named file there as the real one does, a restore reads it.
    fn fake_llama(
        directory: &Path,
        key: &'static str,
        restore_fails: bool,
    ) -> crate::test_support::RoutedServer {
        fake_llama_saving(directory, key, restore_fails, REQUEST_TOKENS)
    }

    /// The same, saving `saved_tokens` of slot.
    fn fake_llama_saving(
        directory: &Path,
        key: &'static str,
        restore_fails: bool,
        saved_tokens: u64,
    ) -> crate::test_support::RoutedServer {
        let directory = directory.to_path_buf();
        routed_server(move |request: &RoutedRequest| {
            if request.authorization.as_deref() != Some(&format!("Bearer {key}")) {
                return (401, r#"{"error":"Invalid API Key"}"#.to_owned());
            }
            let filename = serde_json::from_str::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|body| body["filename"].as_str().map(str::to_owned));
            match (request.method.as_str(), request.path.as_str(), filename) {
                ("GET", "/props", _) => (200, r#"{"build_info":"b11946-8ca4060b"}"#.to_owned()),
                ("POST", "/v1/chat/completions", _) => (
                    200,
                    r#"{"choices":[{"finish_reason":"length","message":{"content":"{"}}],"timings":{"cache_n":477,"prompt_n":11}}"#
                        .to_owned(),
                ),
                ("POST", "/slots/0?action=save", Some(name)) => {
                    fs::write(directory.join(&name), b"state").unwrap();
                    (
                        200,
                        json!({"id_slot":0,"filename":name,"n_saved":saved_tokens,"n_written":5,"timings":{"save_ms":640.5}})
                            .to_string(),
                    )
                }
                ("POST", "/slots/0?action=restore", Some(name)) => {
                    if restore_fails || !directory.join(&name).is_file() {
                        return (400, r#"{"error":"Unable to restore slot"}"#.to_owned());
                    }
                    (
                        200,
                        json!({"id_slot":0,"filename":name,"n_restored":REQUEST_TOKENS,"n_read":5,"timings":{"restore_ms":183.4}})
                            .to_string(),
                    )
                }
                _ => (404, "{}".to_owned()),
            }
        })
    }

    fn keeper(server: &crate::test_support::RoutedServer, directory: &Path) -> SlotKeeper {
        keeper_knowing(server, directory, "4", Arc::default())
    }

    /// A keeper for a server launched with `threads`, sharing `known` with
    /// the session's other lifetimes.
    fn keeper_knowing(
        server: &crate::test_support::RoutedServer,
        directory: &Path,
        threads: &str,
        known: Arc<KnownInstructions>,
    ) -> SlotKeeper {
        let mut keeper = SlotKeeper::new(
            &format!("http://{}", server.address),
            "secret",
            directory,
            &identity(&["--threads".to_owned(), threads.to_owned()]),
            crate::client::instructions_only_request("intern-local", None),
            256,
            known,
        )
        .unwrap();
        keeper.min_free = 0;
        keeper
    }

    /// The name changes with every input the state depends on, and with
    /// nothing else; llama-server's filename rules accept it.
    #[test]
    fn the_file_name_says_everything_the_state_depends_on() {
        let arguments = ["--threads".to_owned(), "4".to_owned()];
        let base = identity(&arguments);
        let name = base.file_name();
        assert_eq!(name, identity(&arguments).file_name());
        assert!(name.starts_with("accurate-") && name.ends_with(".bin"));
        assert_eq!(name.len(), "accurate-".len() + 16 + ".bin".len());
        assert!(!name.contains(['/', '\\', ':']) && !name.contains(".."));
        let eight = ["--threads".to_owned(), "8".to_owned()];
        let others = [
            SlotIdentity {
                model_sha256: "00fe7986",
                ..base.clone()
            },
            SlotIdentity {
                runtime_build: "b12140-1fa7f1c4",
                ..base.clone()
            },
            SlotIdentity {
                prefix: "another prompt",
                ..base.clone()
            },
            SlotIdentity {
                arguments: &eight,
                ..base.clone()
            },
            SlotIdentity {
                cpu: "x86_64 avx2 avx512f",
                ..base.clone()
            },
        ];
        for other in others {
            assert_ne!(other.file_name(), name, "{other:?}");
        }
        let fast = SlotIdentity {
            label: "fast",
            ..base
        };
        assert!(fast.file_name().starts_with("fast-"));
    }

    /// The first lifetime has nothing to restore and saves after its first
    /// document, once, right after the instructions-only request that
    /// leaves no document in the slot; the next restores what it saved and
    /// saves nothing. Every call carries the server's key.
    #[test]
    fn a_saved_slot_is_restored_by_the_next_launch() {
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama(directory.path(), "secret", false);

        let first = keeper(&server, directory.path());
        assert_eq!(first.restore(), Restore::NoFile);
        assert!(server.requests().is_empty(), "no file, no request");
        assert_eq!(
            first.save_once(LONG_FIRST_REQUEST),
            Save::Saved {
                tokens: REQUEST_TOKENS,
                bytes: 5,
                millis: 640.5
            }
        );
        assert!(first.path().is_file());
        assert_eq!(
            first.save_once(LONG_FIRST_REQUEST),
            Save::NotNeeded,
            "once a lifetime"
        );
        let saves = server.requests();
        assert_eq!(saves.len(), 2);
        assert_eq!(saves[0].path, "/v1/chat/completions");
        let sent: Value = serde_json::from_str(&saves[0].body).unwrap();
        assert_eq!(
            sent,
            crate::client::instructions_only_request("intern-local", None)
        );
        assert_eq!(saves[1].path, "/slots/0?action=save");
        assert!(
            saves[1]
                .body
                .contains(&format!("{}.part", first.file_name())),
            "written under a partial name and renamed: {}",
            saves[1].body
        );

        let second = keeper(&server, directory.path());
        assert_eq!(
            second.restore(),
            Restore::Restored {
                tokens: REQUEST_TOKENS,
                bytes: 5,
                millis: 183.4
            }
        );
        assert_eq!(second.save_once(LONG_FIRST_REQUEST), Save::NotNeeded);
        let requests = server.requests();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[2].path, "/slots/0?action=restore");
        assert!(
            requests[2]
                .body
                .contains(&format!("\"{}\"", second.file_name()))
        );
        assert!(
            requests
                .iter()
                .all(|request| request.authorization.as_deref() == Some("Bearer secret"))
        );
        assert_eq!(
            runtime_build(&format!("http://{}", server.address), "secret").unwrap(),
            "b11946-8ca4060b"
        );
    }

    /// A file the server will not read is removed, and the lifetime saves a
    /// fresh one after its first document.
    #[test]
    fn a_file_that_will_not_restore_is_replaced() {
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama(directory.path(), "secret", true);
        let keeper = keeper(&server, directory.path());
        fs::write(keeper.path(), b"truncated").unwrap();
        assert_eq!(keeper.restore(), Restore::Failed);
        assert!(!keeper.path().exists());
        assert!(matches!(
            keeper.save_once(LONG_FIRST_REQUEST),
            Save::Saved { .. }
        ));
        assert_eq!(fs::read(keeper.path()).unwrap(), b"state");
    }

    /// A lifetime whose instructions were not processed in one piece -
    /// its first request was shorter than the instructions plus 4 + `-ub`
    /// tokens, or was not a document - saves nothing, then or later; a
    /// lifetime whose first request was just long enough saves.
    #[test]
    fn only_instructions_processed_in_one_piece_are_saved() {
        for (first, saved) in [
            (None, false),
            (Some(736), false),
            (Some(737), true),
            (Some(2_000), true),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let server = fake_llama(directory.path(), "secret", false);
            let keeper = keeper(&server, directory.path());
            let save = keeper.save_once(first);
            assert_eq!(
                matches!(save, Save::Saved { .. }),
                saved,
                "{first:?}: {save:?}"
            );
            if !saved {
                assert_eq!(save, Save::Uneven, "{first:?}");
                assert!(!keeper.path().exists());
                assert_eq!(keeper.save_once(Some(2_000)), Save::NotNeeded);
            }
        }
    }

    /// A lifetime whose first document was too short sends the
    /// instructions-only request once in a session; every later lifetime of
    /// that configuration knows the instructions' length and sends nothing
    /// for a first document as short. A long one still saves, and another
    /// configuration's length is not taken for this one's.
    #[test]
    fn a_short_first_document_asks_the_server_once_a_session() {
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama(directory.path(), "secret", false);
        let known = Arc::new(KnownInstructions::default());
        let completions = || {
            server
                .requests()
                .iter()
                .filter(|request| request.path == "/v1/chat/completions")
                .count()
        };

        let first = keeper_knowing(&server, directory.path(), "4", Arc::clone(&known));
        assert_eq!(first.save_once(Some(600)), Save::Uneven);
        assert_eq!(completions(), 1);

        let second = keeper_knowing(&server, directory.path(), "4", Arc::clone(&known));
        assert_eq!(second.save_once(Some(736)), Save::Uneven);
        assert_eq!(completions(), 1, "the length was known");

        let other = keeper_knowing(&server, directory.path(), "8", Arc::clone(&known));
        assert_eq!(other.save_once(Some(600)), Save::Uneven);
        assert_eq!(completions(), 2, "another configuration asks for its own");

        let third = keeper_knowing(&server, directory.path(), "4", known);
        assert!(matches!(third.save_once(Some(737)), Save::Saved { .. }));
        assert_eq!(completions(), 3);
        assert!(third.path().is_file());
    }

    /// A server that saves more of its slot than the instructions-only
    /// request left there - a runtime that kept part of the document read
    /// before it - leaves no file: neither the saved one nor its partial.
    #[test]
    fn a_slot_holding_more_than_the_instructions_is_not_kept() {
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama_saving(directory.path(), "secret", false, 1_104);
        let holding_more = keeper(&server, directory.path());
        assert_eq!(
            holding_more.save_once(LONG_FIRST_REQUEST),
            Save::HeldMore {
                tokens: 1_104,
                request_tokens: REQUEST_TOKENS
            }
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
        assert_eq!(holding_more.save_once(LONG_FIRST_REQUEST), Save::NotNeeded);

        // The one token the request generated may be kept with it.
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama_saving(directory.path(), "secret", false, REQUEST_TOKENS + 1);
        let with_reply = keeper(&server, directory.path());
        assert!(matches!(
            with_reply.save_once(LONG_FIRST_REQUEST),
            Save::Saved { .. }
        ));
    }

    /// No save on a disk without room for it, and none tried later in the
    /// same lifetime.
    #[test]
    fn a_full_disk_is_left_alone() {
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama(directory.path(), "secret", false);
        let mut keeper = keeper(&server, directory.path());
        keeper.min_free = u64::MAX;
        assert_eq!(keeper.save_once(LONG_FIRST_REQUEST), Save::NoRoom);
        keeper.min_free = 0;
        assert_eq!(keeper.save_once(LONG_FIRST_REQUEST), Save::NotNeeded);
        assert!(server.requests().is_empty());
    }

    /// A save the server refuses - here, a wrong key - leaves no file, so
    /// nothing half-written is restored later.
    #[test]
    fn a_failed_save_leaves_nothing_behind() {
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama(directory.path(), "another key", false);
        let keeper = keeper(&server, directory.path());
        fs::write(
            directory
                .path()
                .join(format!("{}.part", keeper.file_name())),
            b"x",
        )
        .unwrap();
        assert_eq!(keeper.save_once(LONG_FIRST_REQUEST), Save::Failed);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
        assert!(runtime_build(&format!("http://{}", server.address), "secret").is_err());
    }

    /// At most one file per label: another configuration's file of the same
    /// label goes, as do unfinished saves; another label's file and anything
    /// that is not a slot file stay.
    #[test]
    fn pruning_keeps_one_file_per_label() {
        let directory = tempfile::tempdir().unwrap();
        let server = fake_llama(directory.path(), "secret", false);
        let keeper = keeper(&server, directory.path());
        let keep = [
            keeper.file_name().to_owned(),
            "fast-0123456789abcdef.bin".to_owned(),
            "notes.txt".to_owned(),
        ];
        let remove = [
            "accurate-fedcba9876543210.bin".to_owned(),
            "fast-0123456789abcdef.bin.part".to_owned(),
        ];
        for name in keep.iter().chain(&remove) {
            fs::write(directory.path().join(name), b"x").unwrap();
        }
        keeper.prune();
        let mut left = fs::read_dir(directory.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        left.sort();
        let mut expected = keep.to_vec();
        expected.sort();
        assert_eq!(left, expected);
    }

    #[test]
    fn only_a_folder_the_server_can_name_is_used() {
        assert!(usable_directory(Path::new(
            "/home/user/.local/share/intern/slots"
        )));
        assert!(!usable_directory(Path::new("")));
        assert_eq!(
            usable_directory(Path::new("C:\\Users\\José\\AppData\\Local\\Intern\\slots")),
            !cfg!(windows)
        );
    }

    #[test]
    fn the_cpu_fingerprint_names_the_architecture() {
        assert!(cpu_fingerprint().starts_with(std::env::consts::ARCH));
    }
}
