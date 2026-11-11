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
    io::{Read, Seek, SeekFrom},
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
/// A save is made only when at least this much of the disk is free before
/// it: a file is 174 MB for either model (the instructions' 492 tokens), so
/// a save leaves at least 1.8 GB, and the speed-up is not worth a full disk.
const MIN_FREE_BEFORE_SAVE: u64 = 2 * 1024 * 1024 * 1024;
/// The rules a file was saved under: what a save keeps, and when it is
/// made. Hashed into the file's name, so a release that changes them never
/// restores a file saved under the old ones.
const SLOT_SCHEME: &str = "1";
/// The most tokens the instructions-only request may process past what it
/// finds cached: its own short turn, 15 tokens on b11946. More means the
/// server reused less than the instructions and recomputed part of them in
/// the request's own batch, so the slot no longer holds them as a long first
/// document computed them, and nothing is saved.
const FIXED_TURN_MAX_TOKENS: u64 = 32;
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
        self.file_name_under(SLOT_SCHEME)
    }

    fn file_name_under(&self, scheme: &str) -> String {
        let mut hasher = Sha256::new();
        for part in [
            scheme,
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
/// loads, and so how it computes: every feature b11946's x86 scoring reads
/// (`ggml-cpu/arch/x86/cpu-feats.cpp`), and AMX-TILE, which its
/// `sapphirerapids` backend is built for beside AMX-INT8. Without AVX-VNNI
/// and AMX an Alder Lake would share a name with a Haswell, and a Sapphire
/// Rapids with a Zen 4, though each loads another backend (`alderlake`,
/// `sapphirerapids`) - so a slot folder carried between them, in a roaming
/// profile or a migrated virtual machine, would restore state another
/// backend computed. The pinned Rust release cannot detect those three by
/// name, so they are read from CPUID as ggml reads them
/// (`x86_extended_features`).
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
            "avx512bf16",
            "avx512cd",
            "avx512dq"
        );
        for name in x86_extended_features() {
            features.push(' ');
            features.push_str(name);
        }
        features
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        std::env::consts::ARCH.to_owned()
    }
}

/// AVX-VNNI (CPUID leaf 7 subleaf 1, EAX bit 4), AMX-TILE and AMX-INT8
/// (leaf 7, EDX bits 24 and 25), as ggml reads them: from CPUID alone, when
/// the processor has leaf 7.
#[cfg(target_arch = "x86_64")]
fn x86_extended_features() -> Vec<&'static str> {
    // CPUID is part of every x86-64 processor. The intrinsic is unsafe on
    // the pinned Rust release and safe on later ones.
    #[allow(unsafe_code, unused_unsafe)]
    fn cpuid(leaf: u32, subleaf: u32) -> std::arch::x86_64::CpuidResult {
        // SAFETY: CPUID is available on every x86-64 processor, and any
        // leaf may be asked for.
        unsafe { std::arch::x86_64::__cpuid_count(leaf, subleaf) }
    }
    if cpuid(0, 0).eax < 7 {
        return Vec::new();
    }
    let leaf_7 = cpuid(7, 0);
    let leaf_7_1 = cpuid(7, 1);
    [
        ("avxvnni", leaf_7_1.eax, 4),
        ("amx-tile", leaf_7.edx, 24),
        ("amx-int8", leaf_7.edx, 25),
    ]
    .into_iter()
    .filter(|&(_, register, bit)| register & (1 << bit) != 0)
    .map(|(name, _, _)| name)
    .collect()
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
    /// The file held state for more tokens than the instructions-only
    /// request left in the slot - `tokens`, in its token list or in a
    /// context checkpoint, against that request's `request_tokens` - so it
    /// may hold part of the document read before it. It was deleted.
    HeldMore {
        tokens: u64,
        request_tokens: u64,
    },
    /// The file's own token count or its checkpoints could not be read
    /// ([`saved_state_reach`]), so what it holds is not known. It was
    /// deleted.
    Unchecked,
    /// A save earlier in this session was [`Save::HeldMore`] or
    /// [`Save::Unchecked`]: the runtime may keep a document in its slot, so
    /// no lifetime of the session saves again ([`KnownInstructions`]).
    Refused,
    Failed,
}

/// A context checkpoint a slot file carries: the model's state after its
/// first `n_tokens` tokens, positions `pos_min` to `pos_max`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SavedCheckpoint {
    pub n_tokens: i64,
    pub pos_min: i32,
    pub pos_max: i32,
}

/// What a slot file holds state for, read from the file itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedState {
    /// The tokens its token list holds.
    pub tokens: u64,
    /// The context checkpoints appended after the model's state.
    pub checkpoints: Vec<SavedCheckpoint>,
}

impl SavedState {
    /// The furthest token any of it holds state for.
    pub fn reach(&self) -> u64 {
        self.checkpoints
            .iter()
            .map(|checkpoint| {
                let n_tokens = u64::try_from(checkpoint.n_tokens).unwrap_or(u64::MAX);
                let after = u64::try_from(i64::from(checkpoint.pos_max) + 1).unwrap_or(0);
                n_tokens.max(after)
            })
            .fold(self.tokens, u64::max)
    }
}

/// llama.cpp's sequence state file (`llama_state_seq_save_file`): "ggsq",
/// version 4, a count of 32-bit words and the words - the slot's tokens as
/// the server packs them - then the model's state.
const STATE_MAGIC: [u8; 8] = [b'q', b's', b'g', b'g', 4, 0, 0, 0];
/// How llama-server packs the slot's tokens into those words
/// (`server_tokens::serialize`): this marker, version 1, the token count and
/// the tokens, then the media the prompt held (none here). Without the
/// marker the words are the tokens themselves, as older servers wrote them.
const PACKED_TOKENS_MARKER: u32 = u32::MAX;
const PACKED_TOKENS_VERSION: u32 = 1;
/// llama-server's checkpoint list (`save_slot_checkpoints`), appended after
/// that state: "SCKP", version 1.
const CHECKPOINT_MAGIC: [u8; 8] = [b'S', b'C', b'K', b'P', 1, 0, 0, 0];
/// How much of the file is searched at a time for the checkpoint list.
const SEARCH_CHUNK: usize = 8 * 1024 * 1024;

/// What the slot file at `path` holds state for, as llama-server b11946
/// writes it: the token count from the state's header, and the checkpoint
/// list that ends the file - a count, then each checkpoint's `n_tokens`
/// (i64), `pos_min` and `pos_max` (i32), and its target, draft and
/// speculative states, each a u64 length and that many bytes.
///
/// The model's state before the list has no length of its own, so the list
/// is found by its magic, searched through the whole file, and taken only
/// where it reads to exactly the end of the file. `None` when the header is
/// not that, when no such list ends the file, or when two places do: a
/// runtime that writes either differently, or keeps no checkpoints, gives a
/// file nothing here can vouch for, and so do state bytes that would read as
/// a shorter list. The model's state itself is not read: how far its own
/// cells reach is not known here.
pub fn saved_state_reach(path: &Path) -> Option<SavedState> {
    saved_state_in_chunks(path, SEARCH_CHUNK)
}

/// [`saved_state_reach`], searching `chunk` bytes at a time (at least the
/// magic's length).
fn saved_state_in_chunks(path: &Path, chunk: usize) -> Option<SavedState> {
    let mut file = fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let mut header = [0_u8; 12];
    file.read_exact(&mut header).ok()?;
    if header[..8] != STATE_MAGIC {
        return None;
    }
    let words = u32::from_le_bytes(header[8..12].try_into().ok()?);
    let tokens = if words >= 3 {
        let mut packed = [0_u8; 12];
        file.read_exact(&mut packed).ok()?;
        let word = |at: usize| {
            u32::from_le_bytes([packed[at], packed[at + 1], packed[at + 2], packed[at + 3]])
        };
        match (word(0), word(4), word(8)) {
            (PACKED_TOKENS_MARKER, PACKED_TOKENS_VERSION, count) if count <= words - 3 => {
                u64::from(count)
            }
            (PACKED_TOKENS_MARKER, _, _) => return None,
            _ => u64::from(words),
        }
    } else {
        u64::from(words)
    };
    let finder = memchr::memmem::Finder::new(&CHECKPOINT_MAGIC);
    let mut reader = fs::File::open(path).ok()?;
    let mut chunk = vec![0_u8; chunk.max(CHECKPOINT_MAGIC.len())];
    let mut start = 0_u64;
    let mut kept = 0_usize;
    let mut list = None;
    loop {
        let read = reader.read(&mut chunk[kept..]).ok()?;
        let filled = kept + read;
        for found in finder.find_iter(&chunk[..filled]) {
            let offset = start + found as u64;
            if let Some(checkpoints) = checkpoints_at(&mut file, offset, length) {
                // Bytes before the server's list that read as a list to the
                // end of the file too would hide which one is the server's.
                if list.replace(checkpoints).is_some() {
                    return None;
                }
            }
        }
        if read == 0 {
            return list.map(|checkpoints| SavedState {
                tokens,
                checkpoints,
            });
        }
        // Keep the last few bytes, so a magic split across two reads is
        // still found, and found once.
        kept = filled.min(CHECKPOINT_MAGIC.len() - 1);
        chunk.copy_within(filled - kept..filled, 0);
        start += (filled - kept) as u64;
    }
}

/// The checkpoint list at `offset`, if one starts there and ends the file.
fn checkpoints_at(file: &mut fs::File, offset: u64, length: u64) -> Option<Vec<SavedCheckpoint>> {
    fn read<const N: usize>(file: &mut fs::File) -> Option<[u8; N]> {
        let mut bytes = [0_u8; N];
        file.read_exact(&mut bytes).ok()?;
        Some(bytes)
    }
    file.seek(SeekFrom::Start(offset + CHECKPOINT_MAGIC.len() as u64))
        .ok()?;
    let count = u32::from_le_bytes(read(file)?);
    let mut checkpoints = Vec::new();
    for _ in 0..count {
        let n_tokens = i64::from_le_bytes(read(file)?);
        let pos_min = i32::from_le_bytes(read(file)?);
        let pos_max = i32::from_le_bytes(read(file)?);
        for _ in 0..3 {
            let size = u64::from_le_bytes(read(file)?);
            let at = file.stream_position().ok()?;
            if size > length.checked_sub(at)? {
                return None;
            }
            file.seek(SeekFrom::Current(i64::try_from(size).ok()?))
                .ok()?;
        }
        checkpoints.push(SavedCheckpoint {
            n_tokens,
            pos_min,
            pos_max,
        });
    }
    (file.stream_position().ok()? == length).then_some(checkpoints)
}

/// What a session has learned of its saves, shared by every server lifetime
/// of it.
///
/// The instructions' length in tokens for each configuration, by slot file
/// name, once a lifetime of it has sent the instructions-only request: a
/// later lifetime whose first document is too short to save from then
/// knows it without sending the request again.
///
/// And whether a save has held more than the instructions, or could not be
/// read back ([`Save::HeldMore`], [`Save::Unchecked`]). Either says the
/// runtime may keep a document in its slot. The file is deleted, but every
/// later lifetime - each idle unload, each switch between the models -
/// would write the document's tokens and state to the slot folder again
/// before deleting them, so none does until the app restarts.
#[derive(Debug, Default)]
pub struct KnownInstructions {
    lengths: Mutex<HashMap<String, u64>>,
    refused: AtomicBool,
}

impl KnownInstructions {
    fn get(&self, file_name: &str) -> Option<u64> {
        self.lengths
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(file_name)
            .copied()
    }

    fn record(&self, file_name: &str, tokens: u64) {
        self.lengths
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(file_name.to_owned(), tokens);
    }

    fn refuse_saves(&self) {
        self.refused.store(true, Ordering::SeqCst);
    }

    fn saves_refused(&self) -> bool {
        self.refused.load(Ordering::SeqCst)
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
    /// [`MIN_FREE_BEFORE_SAVE`], apart from tests, which run on any disk.
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
            min_free: MIN_FREE_BEFORE_SAVE,
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
    ///
    /// A file a crash cut short, or left with a zero-filled tail, still
    /// restores in part: b11946 loads the model's state, finds no checkpoint
    /// list after it, and answers as if all was well, having read less than
    /// the file. Kept, such a file would cost every later launch its reading
    /// and the speed-up for good. So the file is read back first
    /// ([`saved_state_reach`]), and the server must say it read all of it;
    /// otherwise the slot is emptied, as a fresh server's is, and the file
    /// is removed, so the next document saves a whole one.
    pub fn restore(&self) -> Restore {
        let path = self.path();
        let Some(length) = fs::metadata(&path)
            .ok()
            .filter(fs::Metadata::is_file)
            .map(|metadata| metadata.len())
        else {
            return Restore::NoFile;
        };
        if saved_state_reach(&path).is_none() {
            let _ = fs::remove_file(&path);
            return Restore::Failed;
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
            Ok(restored) if restored.n_read == length => {
                self.unsaved.store(false, Ordering::SeqCst);
                Restore::Restored {
                    tokens: restored.n_restored,
                    bytes: restored.n_read,
                    millis: restored.timings.restore_ms,
                }
            }
            Ok(_) => {
                self.erase();
                let _ = fs::remove_file(&path);
                Restore::Failed
            }
            Err(_) => {
                let _ = fs::remove_file(&path);
                Restore::Failed
            }
        }
    }

    /// Empties the server's slot, as a server that restored nothing holds it.
    fn erase(&self) {
        let _ = self
            .http
            .post(format!("{}/slots/0?action=erase", self.endpoint))
            .bearer_auth(&self.api_key)
            .send();
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
    /// model built from them, in the slot and in the context checkpoints
    /// the server took while it read them. Saved as it is, a person's
    /// document would outlive its filing in a file of Intern's. So the
    /// instructions-only request goes first: it reuses the instructions'
    /// state the document left, and replaces the document's tokens with two
    /// fixed words; b11946 drops every checkpoint past where the two
    /// prompts part. What is saved is the instructions, and the next
    /// document reuses them exactly as it would have reused them after the
    /// document.
    ///
    /// The saved file is then read back before it is kept: its token list's
    /// count and the position each of its checkpoints reaches
    /// ([`saved_state_reach`]), and the server's own count of what it saved.
    /// A file reaching past that request's prompt and the one token it
    /// generated is deleted ([`Save::HeldMore`]), and so is one whose
    /// checkpoints cannot be read ([`Save::Unchecked`]); after either, no
    /// lifetime of the session saves again ([`Save::Refused`]). What is
    /// checked is how far the token list and the checkpoints reach, by
    /// count and position, not the positions held in the model's state
    /// itself, nor where each checkpoint came from: one inside the
    /// request's own 15 tokens is taken to be the request's, as on b11946
    /// it is, since the server keeps no checkpoint and no cell past where
    /// the prompts part. The speed battery's `live_slots` job
    /// runs the app's own save against the real server; it is the check to
    /// run after any runtime change.
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
        if self.known.saves_refused() {
            return Save::Refused;
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
        if request.processed > FIXED_TURN_MAX_TOKENS {
            return Save::Uneven;
        }
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
        let Some(state) = saved_state_reach(&partial) else {
            let _ = fs::remove_file(&partial);
            self.known.refuse_saves();
            return Save::Unchecked;
        };
        let reach = state.reach().max(saved.n_saved);
        if reach > request_tokens + 1 {
            let _ = fs::remove_file(&partial);
            self.known.refuse_saves();
            return Save::HeldMore {
                tokens: reach,
                request_tokens,
            };
        }
        // The server wrote the file without syncing it; synced before it is
        // renamed, a crash just after leaves it whole or not there. Should
        // either sync fail, the check before a restore catches what is left.
        let _ = fs::OpenOptions::new()
            .write(true)
            .open(&partial)
            .and_then(|file| file.sync_all());
        if !(partial.is_file() && fs::rename(&partial, self.path()).is_ok()) {
            let _ = fs::remove_file(&partial);
            return Save::Failed;
        }
        #[cfg(unix)]
        let _ = fs::File::open(&self.directory).and_then(|folder| folder.sync_all());
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

    /// A slot file as b11946 writes one: the state's header and `tokens`
    /// tokens, `state` standing in for the model's state, then the
    /// checkpoint list, each checkpoint `(n_tokens, data bytes)` with no
    /// draft or speculative state, unless `checkpoints` is `None`.
    fn slot_file(tokens: u32, state: &[u8], checkpoints: Option<&[(i64, usize)]>) -> Vec<u8> {
        let mut file = STATE_MAGIC.to_vec();
        file.extend((tokens + 4).to_le_bytes());
        file.extend(PACKED_TOKENS_MARKER.to_le_bytes());
        file.extend(PACKED_TOKENS_VERSION.to_le_bytes());
        file.extend(tokens.to_le_bytes());
        file.extend((0..tokens).flat_map(|token| token.to_le_bytes()));
        file.extend(0_u32.to_le_bytes());
        file.extend(state);
        if let Some(checkpoints) = checkpoints {
            file.extend(CHECKPOINT_MAGIC);
            file.extend(u32::try_from(checkpoints.len()).unwrap().to_le_bytes());
            for &(n_tokens, bytes) in checkpoints {
                file.extend(n_tokens.to_le_bytes());
                file.extend(0_i32.to_le_bytes());
                file.extend((i32::try_from(n_tokens).unwrap() - 1).to_le_bytes());
                file.extend(u64::try_from(bytes).unwrap().to_le_bytes());
                file.extend(vec![7_u8; bytes]);
                file.extend(0_u64.to_le_bytes());
                file.extend(0_u64.to_le_bytes());
            }
        }
        file
    }

    /// What the fake server saves, what its instructions-only request
    /// processed, and how much less than the whole file a restore reads.
    #[derive(Clone)]
    struct Saving {
        n_saved: u64,
        file: Vec<u8>,
        processed: u64,
        restore_reads_short: u64,
    }

    /// As b11946 saves after the instructions-only request: its tokens,
    /// and checkpoints where the user turn starts and 4 before the end.
    fn as_b11946() -> Saving {
        Saving {
            n_saved: REQUEST_TOKENS,
            file: slot_file(488, b"state", Some(&[(477, 64), (484, 64)])),
            processed: 11,
            restore_reads_short: 0,
        }
    }

    /// A fake llama-server's slot endpoints over `directory`: a save writes
    /// the named file there as the real one does, a restore reads it and
    /// says how much of it it read, and an erase empties the slot.
    fn fake_llama(
        directory: &Path,
        key: &'static str,
        restore_fails: bool,
    ) -> crate::test_support::RoutedServer {
        fake_llama_saving(directory, key, restore_fails, as_b11946())
    }

    /// The same, saving `saving`.
    fn fake_llama_saving(
        directory: &Path,
        key: &'static str,
        restore_fails: bool,
        saving: Saving,
    ) -> crate::test_support::RoutedServer {
        let directory = directory.to_path_buf();
        let completion = json!({
            "choices": [{"finish_reason": "length", "message": {"content": "{"}}],
            "timings": {"cache_n": 477, "prompt_n": saving.processed},
        })
        .to_string();
        let written = saving.file.len();
        routed_server(move |request: &RoutedRequest| {
            if request.authorization.as_deref() != Some(&format!("Bearer {key}")) {
                return (401, r#"{"error":"Invalid API Key"}"#.to_owned());
            }
            let filename = serde_json::from_str::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|body| body["filename"].as_str().map(str::to_owned));
            match (request.method.as_str(), request.path.as_str(), filename) {
                ("GET", "/props", _) => (200, r#"{"build_info":"b11946-8ca4060b"}"#.to_owned()),
                ("POST", "/v1/chat/completions", _) => (200, completion.clone()),
                ("POST", "/slots/0?action=save", Some(name)) => {
                    fs::write(directory.join(&name), &saving.file).unwrap();
                    (
                        200,
                        json!({"id_slot":0,"filename":name,"n_saved":saving.n_saved,"n_written":written,"timings":{"save_ms":640.5}})
                            .to_string(),
                    )
                }
                ("POST", "/slots/0?action=restore", Some(name)) => {
                    let Ok(on_disk) = fs::metadata(directory.join(&name)) else {
                        return (400, r#"{"error":"Unable to restore slot"}"#.to_owned());
                    };
                    if restore_fails {
                        return (400, r#"{"error":"Unable to restore slot"}"#.to_owned());
                    }
                    let read = on_disk.len() - saving.restore_reads_short;
                    (
                        200,
                        json!({"id_slot":0,"filename":name,"n_restored":REQUEST_TOKENS,"n_read":read,"timings":{"restore_ms":183.4}})
                            .to_string(),
                    )
                }
                ("POST", "/slots/0?action=erase", _) => {
                    (200, r#"{"id_slot":0,"n_erased":488}"#.to_owned())
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
        assert_ne!(
            base.file_name_under("2"),
            name,
            "the rules it was saved under"
        );
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
        let written = as_b11946().file.len() as u64;
        assert_eq!(
            first.save_once(LONG_FIRST_REQUEST),
            Save::Saved {
                tokens: REQUEST_TOKENS,
                bytes: written,
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
                bytes: written,
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
        fs::write(keeper.path(), as_b11946().file).unwrap();
        assert_eq!(keeper.restore(), Restore::Failed);
        assert!(!keeper.path().exists());
        assert!(matches!(
            keeper.save_once(LONG_FIRST_REQUEST),
            Save::Saved { .. }
        ));
        assert_eq!(fs::read(keeper.path()).unwrap(), as_b11946().file);
    }

    /// A file a crash cut short, or left with zeros where its checkpoint
    /// list should be, is never handed to the server: it is removed, and
    /// the lifetime saves a whole one. One the server reads only part of,
    /// answering as if it had restored it, is removed as well, and the slot
    /// emptied of what was read.
    #[test]
    fn a_file_that_does_not_restore_whole_is_replaced() {
        let whole = as_b11946().file;
        let list = whole.len() - (8 + 4 + 2 * (8 + 4 + 4 + 8 + 64 + 16));
        let mut zero_tail = whole.clone();
        zero_tail[list..].fill(0);
        for torn in [
            whole[..whole.len() - 1].to_vec(),
            whole[..list].to_vec(),
            zero_tail,
            Vec::new(),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let server = fake_llama(directory.path(), "secret", false);
            let keeper = keeper(&server, directory.path());
            fs::write(keeper.path(), &torn).unwrap();
            assert_eq!(keeper.restore(), Restore::Failed, "{} bytes", torn.len());
            assert!(!keeper.path().exists());
            assert!(server.requests().is_empty(), "the server is not asked");
            assert!(matches!(
                keeper.save_once(LONG_FIRST_REQUEST),
                Save::Saved { .. }
            ));
            assert_eq!(fs::read(keeper.path()).unwrap(), whole);
        }

        // b11946 reading the model's state and no checkpoint list after it.
        let directory = tempfile::tempdir().unwrap();
        let read_short = Saving {
            restore_reads_short: (whole.len() - list) as u64,
            ..as_b11946()
        };
        let server = fake_llama_saving(directory.path(), "secret", false, read_short);
        let keeper = keeper(&server, directory.path());
        fs::write(keeper.path(), &whole).unwrap();
        assert_eq!(keeper.restore(), Restore::Failed);
        assert!(!keeper.path().exists());
        let requests = server.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].path, "/slots/0?action=restore");
        assert_eq!(requests[1].path, "/slots/0?action=erase");
        assert_eq!(requests[1].authorization.as_deref(), Some("Bearer secret"));
        assert!(matches!(
            keeper.save_once(LONG_FIRST_REQUEST),
            Save::Saved { .. }
        ));
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

    /// A save that keeps state for more than the instructions-only request
    /// left in the slot - a runtime that kept part of the document read
    /// before it, by the server's count, in the file's token list, or in a
    /// checkpoint past the request - leaves no file: neither the saved one
    /// nor its partial. So does a file whose checkpoints cannot be read.
    #[test]
    fn a_slot_holding_more_than_the_instructions_is_not_kept() {
        let b11946 = as_b11946();
        let document_checkpoints: &[(i64, usize)] = &[(477, 64), (484, 64), (1_090, 64)];
        for (saving, expected) in [
            (
                Saving {
                    n_saved: 1_104,
                    ..b11946.clone()
                },
                Save::HeldMore {
                    tokens: 1_104,
                    request_tokens: REQUEST_TOKENS,
                },
            ),
            (
                Saving {
                    file: slot_file(1_104, b"state", Some(&[(477, 64)])),
                    ..b11946.clone()
                },
                Save::HeldMore {
                    tokens: 1_104,
                    request_tokens: REQUEST_TOKENS,
                },
            ),
            (
                Saving {
                    file: slot_file(488, b"state", Some(document_checkpoints)),
                    ..b11946.clone()
                },
                Save::HeldMore {
                    tokens: 1_090,
                    request_tokens: REQUEST_TOKENS,
                },
            ),
            (
                Saving {
                    file: slot_file(488, b"state", None),
                    ..b11946.clone()
                },
                Save::Unchecked,
            ),
            (
                Saving {
                    file: b"not a slot file".to_vec(),
                    ..b11946.clone()
                },
                Save::Unchecked,
            ),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let server = fake_llama_saving(directory.path(), "secret", false, saving);
            let known = Arc::new(KnownInstructions::default());
            let holding_more = keeper_knowing(&server, directory.path(), "4", Arc::clone(&known));
            assert_eq!(holding_more.save_once(LONG_FIRST_REQUEST), expected);
            assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
            assert_eq!(holding_more.save_once(LONG_FIRST_REQUEST), Save::NotNeeded);

            // No later lifetime of the session writes its slot again, of
            // this configuration or another; a restore is still made.
            let sent = server.requests().len();
            for threads in ["4", "8"] {
                let later = keeper_knowing(&server, directory.path(), threads, Arc::clone(&known));
                assert_eq!(later.restore(), Restore::NoFile);
                assert_eq!(later.save_once(LONG_FIRST_REQUEST), Save::Refused);
            }
            assert_eq!(server.requests().len(), sent, "nothing more is asked");
            assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
        }

        // The one token the request generated may be kept with it.
        let directory = tempfile::tempdir().unwrap();
        let with_reply = Saving {
            n_saved: REQUEST_TOKENS + 1,
            file: slot_file(489, b"state", Some(&[(477, 64), (484, 64)])),
            ..b11946
        };
        let server = fake_llama_saving(directory.path(), "secret", false, with_reply);
        let with_reply = keeper(&server, directory.path());
        assert!(matches!(
            with_reply.save_once(LONG_FIRST_REQUEST),
            Save::Saved { .. }
        ));
    }

    /// An instructions-only request that processed more than its own short
    /// turn found less than the instructions cached and recomputed the rest
    /// in its own batch: the slot is not the even state, and nothing is
    /// saved.
    #[test]
    fn a_request_that_recomputed_part_of_the_instructions_saves_nothing() {
        let directory = tempfile::tempdir().unwrap();
        let recomputed = Saving {
            processed: 120,
            ..as_b11946()
        };
        let server = fake_llama_saving(directory.path(), "secret", false, recomputed);
        let keeper = keeper(&server, directory.path());
        assert_eq!(keeper.save_once(Some(2_000)), Save::Uneven);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
        assert!(
            server
                .requests()
                .iter()
                .all(|request| request.path != "/slots/0?action=save")
        );
    }

    /// The checkpoint list is found wherever it starts, also across two
    /// reads, past state bytes that happen to hold its magic, and only when
    /// it reads to the end of the file.
    #[test]
    fn the_saved_state_is_read_from_the_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("slot.bin");
        let mut state = vec![0_u8; 300];
        state[100..108].copy_from_slice(&CHECKPOINT_MAGIC);
        state[108..112].copy_from_slice(&2_u32.to_le_bytes());
        let file = slot_file(492, &state, Some(&[(477, 40), (488, 40)]));
        fs::write(&path, &file).unwrap();
        let expected = SavedState {
            tokens: 492,
            checkpoints: vec![
                SavedCheckpoint {
                    n_tokens: 477,
                    pos_min: 0,
                    pos_max: 476,
                },
                SavedCheckpoint {
                    n_tokens: 488,
                    pos_min: 0,
                    pos_max: 487,
                },
            ],
        };
        for chunk in [8, 9, 13, 64, 1_000, SEARCH_CHUNK] {
            assert_eq!(saved_state_in_chunks(&path, chunk), Some(expected.clone()));
        }
        assert_eq!(saved_state_reach(&path).unwrap().reach(), 492);

        // State bytes that read as a list to the end of the file as well -
        // one checkpoint of 10 tokens whose state runs up to the server's
        // list's last two lengths, both empty - leave which is the
        // server's unknown, wherever the reads split the file.
        let mut planted = vec![0_u8; 300];
        planted[100..108].copy_from_slice(&CHECKPOINT_MAGIC);
        planted[108..112].copy_from_slice(&1_u32.to_le_bytes());
        planted[112..120].copy_from_slice(&10_i64.to_le_bytes());
        planted[124..128].copy_from_slice(&9_i32.to_le_bytes());
        let mut masked = slot_file(492, &planted, Some(&[(477, 40), (488, 40)]));
        let size_at = memchr::memmem::find(&masked, &CHECKPOINT_MAGIC).unwrap() + 28;
        let to_the_end = (masked.len() - 16 - (size_at + 8)) as u64;
        masked[size_at..size_at + 8].copy_from_slice(&to_the_end.to_le_bytes());
        fs::write(&path, &masked).unwrap();
        let length = masked.len() as u64;
        let server_list = length - (8 + 4 + 2 * (8 + 4 + 4 + 8 + 40 + 16));
        let mut handle = fs::File::open(&path).unwrap();
        for (at, count) in [((size_at - 28) as u64, 1), (server_list, 2)] {
            let list = checkpoints_at(&mut handle, at, length);
            assert_eq!(list.map(|list| list.len()), Some(count), "at {at}");
        }
        for chunk in [8, 9, 13, 64, 1_000, SEARCH_CHUNK] {
            assert_eq!(saved_state_in_chunks(&path, chunk), None, "{chunk}");
        }

        // A list that stops short of the end, or runs past it.
        let mut longer = file.clone();
        longer.push(0);
        fs::write(&path, &longer).unwrap();
        assert_eq!(saved_state_reach(&path), None);
        fs::write(&path, &file[..file.len() - 1]).unwrap();
        assert_eq!(saved_state_reach(&path), None);
        // Another state version, or another packing of the tokens.
        for (at, value) in [(4, 5), (16, 2)] {
            let mut other = file.clone();
            other[at] = value;
            fs::write(&path, &other).unwrap();
            assert_eq!(saved_state_reach(&path), None, "byte {at}");
        }
        // Tokens written as they are, by an older server.
        let mut plain = STATE_MAGIC.to_vec();
        plain.extend(3_u32.to_le_bytes());
        plain.extend([0_u8; 12]);
        plain.extend(&file[file.len() - 2 * (8 + 4 + 4 + 8 + 40 + 16) - 12..]);
        fs::write(&path, &plain).unwrap();
        assert_eq!(saved_state_reach(&path).map(|state| state.tokens), Some(3));
        assert_eq!(saved_state_reach(&directory.path().join("missing")), None);
    }

    /// Set INTERN_SLOT_FILE to a file llama-server saved to read what it
    /// holds; skipped otherwise.
    #[test]
    fn a_real_slot_file_is_read() {
        let Some(path) = std::env::var_os("INTERN_SLOT_FILE") else {
            return;
        };
        let started = std::time::Instant::now();
        let state = saved_state_reach(Path::new(&path)).expect("a b11946 slot file");
        eprintln!(
            "{state:?}, reaching {}, read in {:?}",
            state.reach(),
            started.elapsed()
        );
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

    /// Every feature ggml's choice of backend reads that the kernel says
    /// the processor has is named - AVX-VNNI and AMX among them, which the
    /// pinned Rust release cannot detect by name. (The kernel may hide a
    /// feature it does not support, never show one the processor lacks.)
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    #[test]
    fn the_cpu_fingerprint_names_what_chooses_the_backend() {
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap();
        let flags = cpuinfo
            .lines()
            .find(|line| line.starts_with("flags"))
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .unwrap();
        let fingerprint = cpu_fingerprint();
        let named = fingerprint.split(' ').collect::<Vec<_>>();
        for (flag, name) in [
            ("avx_vnni", "avxvnni"),
            ("amx_tile", "amx-tile"),
            ("amx_int8", "amx-int8"),
            ("avx512cd", "avx512cd"),
            ("avx512dq", "avx512dq"),
            ("avx512_bf16", "avx512bf16"),
        ] {
            if flags.contains(&flag) {
                assert!(named.contains(&name), "{flag}: {fingerprint}");
            }
        }
        eprintln!("{fingerprint}");
    }
}
