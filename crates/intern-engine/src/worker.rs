//! Client for the out-of-process parser worker.
//!
//! Extraction runs in a separate process so that a malformed PDF can only take
//! down the parser, never the app. This module owns the JSONL protocol, the
//! supervision, and the translation from worker pages into a [`DocumentSource`].

use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, RecvTimeoutError},
    },
    time::{Duration, Instant},
};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::domain::{DocumentSource, PageImage, PageOrigin, ParserWarning, SourcePage};
use crate::structure::PageLayout;

pub const PROTOCOL_VERSION: u32 = 1;
pub const EXTRACTION_TIMEOUT_SECONDS: u64 = 30 * 60;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const STALE_WORKSPACE_AGE: Duration = Duration::from_secs(24 * 60 * 60);
/// OCR below this mean confidence is reported as a fact-affecting warning.
const LOW_OCR_CONFIDENCE: f32 = 75.0;
/// The longest response line read from the worker.
///
/// The worker caps a document at eight million characters, so an honest
/// reply - JSON escaping and a page image included - stays well inside this.
/// A longer line is a worker gone wrong, and reading it whole would put an
/// unbounded allocation in the app's own process rather than the worker's.
const MAX_RESPONSE_LINE_BYTES: usize = 64 * 1024 * 1024;
/// The worker warning for rows and columns a spreadsheet's window
/// deliberately leaves out, with a marker where it does. The rest of the
/// sheet was read and chosen against, so nothing the window shows is in
/// doubt.
const CONTENT_ELIDED: &str = "CONTENT_ELIDED";
/// The worker's standard error, under the log directory when one is set.
const WORKER_LOG: &str = "worker.log";

/// Progress from the extraction stage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractProgress {
    pub stage: String,
    pub current: usize,
    pub total: Option<usize>,
}

/// Why extraction did not produce a document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractFailure {
    pub code: String,
    /// What the worker said went wrong, in its own words, when it said
    /// anything; empty for failures the host detected itself. Diagnostic
    /// only: the queue decides by `code`, and a person is shown a sentence
    /// chosen for that code rather than this text.
    pub message: String,
    pub retryable: bool,
    pub crashed: bool,
    pub canceled: bool,
}

impl ExtractFailure {
    pub fn new(code: impl Into<String>, retryable: bool, crashed: bool) -> Self {
        Self {
            code: code.into(),
            message: String::new(),
            retryable,
            crashed,
            canceled: false,
        }
    }

    /// A failure the worker reported for a request, keeping what it said.
    pub fn reported(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            message: message.into(),
            ..Self::new(code, retryable, false)
        }
    }

    pub fn crashed() -> Self {
        Self::new("WORKER_CRASHED", true, true)
    }

    pub fn canceled() -> Self {
        Self {
            code: "CANCELED".into(),
            message: String::new(),
            retryable: false,
            crashed: false,
            canceled: true,
        }
    }
}

/// Turns a path into extracted pages. Implemented by the supervised worker and
/// by test doubles.
pub trait DocumentExtractor: Send + Sync {
    fn extract(
        &self,
        request_id: &str,
        path: &Path,
        progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, ExtractFailure>;
    fn cancel(&self, request_id: &str) -> Result<(), ExtractFailure>;
    fn restart(&self) -> Result<(), ExtractFailure>;
    fn shutdown(&self) -> Result<(), ExtractFailure> {
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkerResponse {
    pub protocol_version: u32,
    pub request_id: String,
    pub event: WorkerEvent,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerEvent {
    Hello {
        worker_version: String,
    },
    Progress {
        stage: String,
        current: usize,
        total: Option<usize>,
    },
    Parsed {
        document: WorkerDocument,
    },
    Error {
        code: String,
        message: String,
        retryable: bool,
    },
}

/// A parsed document as the worker sends it.
///
/// Unknown fields are refused, so the worker and this host must agree on
/// the shape. `timings` is the one optional field: a worker of this build
/// fills it on every parsed document, and this host accepts a document with
/// or without it. A host built before the field existed refuses it
/// (`WORKER_PROTOCOL_INVALID` on every document) even though both sides say
/// protocol version 1. The version was deliberately not raised: the worker
/// and the host ship together in one installer and are never paired across
/// builds, and the field changes nothing the host decides.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkerDocument {
    pages: Vec<WorkerPage>,
    warnings: Vec<String>,
    truncated: bool,
    optional_image: Option<WorkerImage>,
    /// Where the worker's time went. Optional: absent from a worker that
    /// predates it, and never an input to anything the host decides.
    #[serde(default)]
    timings: Option<ExtractionTimings>,
}

/// Where the worker says one extraction's time went, in microseconds, and
/// how much OCR it needed. The worker's own type, mirrored rather than
/// shared: the engine does not depend on the worker crate.
///
/// The figures are measurements, never inputs: nothing in the engine reads
/// them, and a stage this build does not know is ignored rather than
/// refused, as one it expects but was not sent reads as zero.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct ExtractionTimings {
    /// The whole extraction in the worker, the snapshot included.
    pub total_micros: u64,
    /// Copying the file into the worker's private workspace.
    pub snapshot_micros: u64,
    /// Reading the format: loading a PDF and its pages' text, or the whole
    /// of every other reader less the stages below that it reported.
    pub parse_micros: u64,
    /// Measuring a PDF page's image coverage and deciding whether it is a
    /// scan.
    pub analysis_micros: u64,
    /// PDFium rasterising pages: the ones that go to OCR, and the rare page
    /// rendered only to be the page image.
    pub render_micros: u64,
    /// Decoding a standalone image file, scaling and turning it.
    pub image_decode_micros: u64,
    /// All OCR for the document: every recognition and orientation pass.
    pub ocr_micros: u64,
    /// Of `ocr_micros`: turning pages grey and encoding the PNGs Tesseract
    /// is handed.
    pub ocr_encode_micros: u64,
    /// Of `ocr_micros`: waiting on Tesseract processes.
    pub ocr_engine_micros: u64,
    /// Building the optional page image.
    pub vision_micros: u64,
    /// Pages read by OCR.
    pub ocr_pages: u32,
    /// Recognition passes, in every orientation tried.
    pub ocr_passes: u32,
    /// Orientation-detection passes.
    pub orientation_passes: u32,
    /// Pixels PDFium rendered, summed over the pages counted in
    /// `render_micros`.
    pub rendered_pixels: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct WorkerPage {
    page_number: usize,
    text: String,
    source: WorkerPageSource,
    ocr_confidence: Option<f32>,
    vision_escalated: bool,
    /// Absent from a worker that predates layouts.
    #[serde(default)]
    layout: Option<PageLayout>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum WorkerPageSource {
    Native,
    Ocr,
    AnyDoc,
    Text,
}

impl From<WorkerPageSource> for PageOrigin {
    fn from(value: WorkerPageSource) -> Self {
        match value {
            WorkerPageSource::Native => Self::Native,
            WorkerPageSource::Ocr => Self::Ocr,
            WorkerPageSource::AnyDoc => Self::Office,
            WorkerPageSource::Text => Self::PlainText,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct WorkerImage {
    page_number: usize,
    mime_type: String,
    data_base64: String,
}

pub fn decode_worker_response(line: &str) -> Result<WorkerResponse, ExtractFailure> {
    let response: WorkerResponse = serde_json::from_str(line)
        .map_err(|_| ExtractFailure::new("WORKER_PROTOCOL_INVALID", false, false))?;
    if response.protocol_version != PROTOCOL_VERSION {
        return Err(ExtractFailure::new(
            "PROTOCOL_VERSION_UNSUPPORTED",
            false,
            false,
        ));
    }
    Ok(response)
}

/// Converts a worker reply into the engine's input type, keeping page
/// boundaries intact so distillation can reason about position.
pub fn adapt_document(document: WorkerDocument) -> Result<DocumentSource, ExtractFailure> {
    let numbers = document
        .pages
        .iter()
        .map(|page| page.page_number)
        .collect::<HashSet<_>>();
    if document.pages.iter().any(|page| page.page_number == 0)
        || numbers.len() != document.pages.len()
    {
        return Err(ExtractFailure::new("WORKER_PROTOCOL_INVALID", false, false));
    }

    // Every warning can corrupt what was read except a deliberate, marked
    // elision. A code this build does not know stays fact-affecting, so a
    // newer worker's warning errs towards review.
    let mut parser_warnings = document
        .warnings
        .into_iter()
        .map(|code| {
            let field_affecting = code != CONTENT_ELIDED;
            ParserWarning::new(code, field_affecting)
        })
        .collect::<Vec<_>>();
    let low_confidence = document.pages.iter().any(|page| {
        page.source == WorkerPageSource::Ocr
            && page
                .ocr_confidence
                .is_some_and(|confidence| confidence < LOW_OCR_CONFIDENCE)
    });
    if low_confidence
        && !parser_warnings
            .iter()
            .any(|warning| warning.code == "LOW_OCR_CONFIDENCE")
    {
        parser_warnings.push(ParserWarning::new("LOW_OCR_CONFIDENCE", true));
    }
    if document.truncated
        && !parser_warnings
            .iter()
            .any(|warning| warning.code == "TEXT_TRUNCATED")
    {
        parser_warnings.push(ParserWarning::new("TEXT_TRUNCATED", true));
    }

    let page_image = document
        .optional_image
        .map(|image| {
            if !numbers.contains(&image.page_number) || image.mime_type != "image/png" {
                return Err(ExtractFailure::new("WORKER_PROTOCOL_INVALID", false, false));
            }
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(image.data_base64)
                .map_err(|_| ExtractFailure::new("WORKER_PROTOCOL_INVALID", false, false))?;
            Ok(PageImage {
                page_number: image.page_number,
                media_type: image.mime_type,
                bytes,
            })
        })
        .transpose()?;

    Ok(DocumentSource {
        pages: document
            .pages
            .into_iter()
            .map(|page| SourcePage {
                page_number: page.page_number,
                text: page.text,
                origin: page.source.into(),
                ocr_confidence: page
                    .ocr_confidence
                    .map(|confidence| confidence.round().clamp(0.0, 100.0) as u32),
                layout: page.layout,
            })
            .collect(),
        parser_warnings,
        page_image,
    })
}

/// What a worker's error event for a request means to the host. The code,
/// the retry hint and the worker's own message all travel on: the queue
/// decides by the code, and the message is what a diagnosis will want.
fn request_failure(code: String, message: String, retryable: bool) -> ExtractFailure {
    if code == "CANCELED" {
        return ExtractFailure::canceled();
    }
    ExtractFailure::reported(code, message, retryable)
}

struct WorkerProcess {
    child: Mutex<Child>,
    input: Mutex<ChildStdin>,
    output: Mutex<Receiver<Result<String, ()>>>,
}

impl WorkerProcess {
    fn write(&self, value: serde_json::Value) -> Result<(), ExtractFailure> {
        let mut input = self.input.lock().map_err(|_| ExtractFailure::crashed())?;
        serde_json::to_writer(&mut *input, &value)
            .map_err(|_| ExtractFailure::new("WORKER_PROTOCOL_INVALID", false, false))?;
        input
            .write_all(b"\n")
            .map_err(|_| ExtractFailure::crashed())?;
        input.flush().map_err(|_| ExtractFailure::crashed())
    }

    fn receive(&self, timeout: Duration) -> Result<String, ExtractFailure> {
        match self
            .output
            .lock()
            .map_err(|_| ExtractFailure::crashed())?
            .recv_timeout(timeout)
        {
            Ok(Ok(line)) => Ok(line),
            Ok(Err(())) | Err(RecvTimeoutError::Disconnected) => Err(ExtractFailure::crashed()),
            Err(RecvTimeoutError::Timeout) => {
                Err(ExtractFailure::new("WORKER_POLL_TIMEOUT", true, false))
            }
        }
    }

    fn terminate(&self) {
        if let Ok(mut child) = self.child.lock() {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

pub struct SupervisedWorker {
    executable: PathBuf,
    temp_root: Option<PathBuf>,
    handshake_timeout: Duration,
    extraction_timeout: Duration,
    running: Mutex<Option<Arc<WorkerProcess>>>,
    active: Mutex<Option<String>>,
    canceled: Mutex<HashSet<String>>,
}

impl SupervisedWorker {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            temp_root: None,
            handshake_timeout: HANDSHAKE_TIMEOUT,
            extraction_timeout: Duration::from_secs(EXTRACTION_TIMEOUT_SECONDS),
            running: Mutex::new(None),
            active: Mutex::new(None),
            canceled: Mutex::new(HashSet::new()),
        }
    }

    pub fn with_temp_root(executable: impl Into<PathBuf>, temp_root: impl Into<PathBuf>) -> Self {
        let mut worker = Self::new(executable);
        worker.temp_root = Some(temp_root.into());
        worker
    }

    #[doc(hidden)]
    pub fn with_timeouts(
        executable: impl Into<PathBuf>,
        handshake_timeout: Duration,
        extraction_timeout: Duration,
    ) -> Self {
        let mut worker = Self::new(executable);
        worker.handshake_timeout = handshake_timeout;
        worker.extraction_timeout = extraction_timeout;
        worker
    }

    fn ensure_running(&self) -> Result<Arc<WorkerProcess>, ExtractFailure> {
        let mut running = self.running.lock().map_err(|_| ExtractFailure::crashed())?;
        if let Some(process) = running.as_ref() {
            return Ok(Arc::clone(process));
        }
        let process = Arc::new(launch(&self.executable, self.temp_root.as_deref())?);
        handshake(&process, self.handshake_timeout)?;
        *running = Some(Arc::clone(&process));
        Ok(process)
    }

    fn clear_running(&self, expected: &Arc<WorkerProcess>) {
        if let Ok(mut running) = self.running.lock()
            && running
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, expected))
        {
            *running = None;
        }
    }

    pub fn stop(&self) {
        let process = self
            .running
            .lock()
            .ok()
            .and_then(|mut running| running.take());
        if let Some(process) = process {
            let _ = process.write(json!({
                "protocol_version": PROTOCOL_VERSION,
                "request_id": "shutdown",
                "command": {"type": "shutdown"},
            }));
            process.terminate();
        }
    }

    fn was_canceled(&self, request_id: &str) -> bool {
        self.canceled
            .lock()
            .map(|mut canceled| canceled.remove(request_id))
            .unwrap_or(false)
    }

    /// `extract`, plus how the worker spent the time, when it reported that.
    pub fn extract_timed(
        &self,
        request_id: &str,
        path: &Path,
        progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<(DocumentSource, Option<ExtractionTimings>), ExtractFailure> {
        {
            let mut active = self.active.lock().map_err(|_| ExtractFailure::crashed())?;
            if active.is_some() {
                return Err(ExtractFailure::new("WORKER_BUSY", true, false));
            }
            *active = Some(request_id.to_owned());
        }
        let result = (|| {
            let process = self.ensure_running()?;
            if let Err(error) = process.write(json!({
                "protocol_version": PROTOCOL_VERSION,
                "request_id": request_id,
                "command": {"type": "parse", "path": path},
            })) {
                process.terminate();
                self.clear_running(&process);
                // A cancel that lands between the worker starting and this
                // command reaching it kills the worker first, which is why the
                // write failed. That is the cancellation the person asked for,
                // not a crash: reporting a crash restarts the worker and puts
                // the document they just cancelled back in the queue.
                if self.was_canceled(request_id) {
                    return Err(ExtractFailure::canceled());
                }
                return Err(error);
            }
            let deadline = Instant::now() + self.extraction_timeout;
            loop {
                if Instant::now() >= deadline {
                    process.terminate();
                    self.clear_running(&process);
                    return Err(ExtractFailure::new("RESOURCE_LIMIT", false, false));
                }
                let line = match process.receive(POLL_INTERVAL) {
                    Err(error) if error.code == "WORKER_POLL_TIMEOUT" => continue,
                    Err(error) => {
                        self.clear_running(&process);
                        if self.was_canceled(request_id) {
                            return Err(ExtractFailure::canceled());
                        }
                        return Err(error);
                    }
                    Ok(line) => line,
                };
                let response = match decode_worker_response(&line) {
                    Ok(response) => response,
                    Err(error) => {
                        process.terminate();
                        self.clear_running(&process);
                        return Err(error);
                    }
                };
                if response.request_id != request_id {
                    continue;
                }
                match response.event {
                    WorkerEvent::Progress {
                        stage,
                        current,
                        total,
                    } => progress(ExtractProgress {
                        stage,
                        current,
                        total,
                    }),
                    WorkerEvent::Parsed { document } => {
                        let timings = document.timings;
                        return match adapt_document(document) {
                            Ok(document) => Ok((document, timings)),
                            Err(error) => {
                                process.terminate();
                                self.clear_running(&process);
                                Err(error)
                            }
                        };
                    }
                    WorkerEvent::Error {
                        code,
                        message,
                        retryable,
                    } => return Err(request_failure(code, message, retryable)),
                    WorkerEvent::Hello { .. } => {
                        process.terminate();
                        self.clear_running(&process);
                        return Err(ExtractFailure::new("WORKER_PROTOCOL_INVALID", false, false));
                    }
                }
            }
        })();
        if let Ok(mut active) = self.active.lock() {
            *active = None;
        }
        result
    }
}

impl DocumentExtractor for SupervisedWorker {
    fn extract(
        &self,
        request_id: &str,
        path: &Path,
        progress: &mut dyn FnMut(ExtractProgress),
    ) -> Result<DocumentSource, ExtractFailure> {
        self.extract_timed(request_id, path, progress)
            .map(|(document, _)| document)
    }

    fn cancel(&self, request_id: &str) -> Result<(), ExtractFailure> {
        let active_matches = self
            .active
            .lock()
            .map_err(|_| ExtractFailure::crashed())?
            .as_deref()
            == Some(request_id);
        if !active_matches {
            return Err(ExtractFailure::new("ITEM_NOT_ACTIVE", false, false));
        }
        self.canceled
            .lock()
            .map_err(|_| ExtractFailure::crashed())?
            .insert(request_id.to_owned());
        let process = self
            .running
            .lock()
            .map_err(|_| ExtractFailure::crashed())?
            .take()
            .ok_or_else(ExtractFailure::crashed)?;
        let _ = process.write(json!({
            "protocol_version": PROTOCOL_VERSION,
            "request_id": format!("cancel-{request_id}"),
            "command": {"type": "cancel", "target_request_id": request_id},
        }));
        process.terminate();
        Ok(())
    }

    fn restart(&self) -> Result<(), ExtractFailure> {
        if let Some(process) = self
            .running
            .lock()
            .map_err(|_| ExtractFailure::crashed())?
            .take()
        {
            process.terminate();
        }
        let process = Arc::new(launch(&self.executable, self.temp_root.as_deref())?);
        handshake(&process, self.handshake_timeout)?;
        *self.running.lock().map_err(|_| ExtractFailure::crashed())? = Some(process);
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ExtractFailure> {
        self.stop();
        Ok(())
    }
}

impl Drop for SupervisedWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

fn launch(executable: &Path, temp_root: Option<&Path>) -> Result<WorkerProcess, ExtractFailure> {
    launch_logged(executable, temp_root, crate::logs::log_directory())
}

/// [`launch`], with the worker's standard error kept in `worker.log` under
/// `log_directory` when there is one. The worker writes warning codes there,
/// never document text: its panic hook reports where a panic happened, not
/// the message, which can quote the document.
fn launch_logged(
    executable: &Path,
    temp_root: Option<&Path>,
    log_directory: Option<&Path>,
) -> Result<WorkerProcess, ExtractFailure> {
    let mut command = Command::new(executable);
    if let Some(temp_root) = temp_root {
        command.env("INTERN_TEMP_ROOT", temp_root);
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(crate::logs::stderr_for(log_directory, WORKER_LOG));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(crate::process::sidecar_creation_flags());
    }
    let mut child = command.spawn().map_err(|_| ExtractFailure::crashed())?;
    crate::process::tie_to_this_process(&child);
    let input = match child.stdin.take() {
        Some(input) => input,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExtractFailure::crashed());
        }
    };
    let output = match child.stdout.take() {
        Some(output) => output,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExtractFailure::crashed());
        }
    };
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::Builder::new()
        .name("intern-worker-jsonl".into())
        .spawn(move || forward_lines(output, &sender, MAX_RESPONSE_LINE_BYTES));
    if reader.is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(ExtractFailure::crashed());
    }
    Ok(WorkerProcess {
        child: Mutex::new(child),
        input: Mutex::new(input),
        output: Mutex::new(receiver),
    })
}

/// Sends each line the worker writes, then `Err(())` once the output ends
/// or cannot be read - including a line longer than `limit`, which is
/// refused as soon as it passes the limit rather than read to its end. The
/// receiving side treats `Err(())` as a crashed worker and stops it.
fn forward_lines(output: impl Read, sender: &mpsc::Sender<Result<String, ()>>, limit: usize) {
    let mut reader = BufReader::new(output);
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = match reader
            .by_ref()
            .take(limit as u64 + 1)
            .read_until(b'\n', &mut line)
        {
            Ok(read) => read,
            Err(_) => break,
        };
        if read == 0 {
            break;
        }
        if line.last() == Some(&b'\n') {
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
        } else if line.len() > limit {
            break;
        }
        let Ok(text) = String::from_utf8(std::mem::take(&mut line)) else {
            break;
        };
        if sender.send(Ok(text)).is_err() {
            return;
        }
    }
    let _ = sender.send(Err(()));
}

pub fn prepare_worker_temp_root(root: &Path, max_entries: usize) -> std::io::Result<usize> {
    std::fs::create_dir_all(root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700))?;
    }
    let mut removed = 0;
    for entry in std::fs::read_dir(root)? {
        if removed >= max_entries {
            break;
        }
        let entry = entry?;
        let file_type = entry.file_type()?;
        #[cfg(windows)]
        let metadata = std::fs::symlink_metadata(entry.path())?;
        let owned_name = entry
            .file_name()
            .to_str()
            .is_some_and(is_stale_owned_workspace_name);
        #[cfg(windows)]
        let reparse_point = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let reparse_point = false;
        if owned_name && file_type.is_dir() && !file_type.is_symlink() && !reparse_point {
            std::fs::remove_dir_all(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}

fn is_stale_owned_workspace_name(name: &str) -> bool {
    if !name.starts_with("intern-worker-") {
        return false;
    }
    let mut suffix = name.rsplit('-');
    let Some(_nonce) = suffix.next().and_then(|value| value.parse::<u64>().ok()) else {
        return false;
    };
    let Some(created_nanos) = suffix.next().and_then(|value| value.parse::<u128>().ok()) else {
        return false;
    };
    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    now_nanos.saturating_sub(created_nanos) >= STALE_WORKSPACE_AGE.as_nanos()
}

fn handshake(process: &WorkerProcess, timeout: Duration) -> Result<(), ExtractFailure> {
    process.write(json!({
        "protocol_version": PROTOCOL_VERSION,
        "request_id": "hello",
        "command": {"type": "hello"},
    }))?;
    let response = decode_worker_response(&process.receive(timeout)?)?;
    if response.request_id == "hello" && matches!(response.event, WorkerEvent::Hello { .. }) {
        Ok(())
    } else {
        process.terminate();
        Err(ExtractFailure::new("WORKER_HANDSHAKE_FAILED", false, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(json: &str) -> Result<DocumentSource, ExtractFailure> {
        let response = decode_worker_response(json).unwrap();
        match response.event {
            WorkerEvent::Parsed { document } => adapt_document(document),
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn page_structure_survives_the_protocol() {
        let source = parsed(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"parsed","document":{
                "pages":[
                  {"page_number":1,"text":"First page.","source":"native","ocr_confidence":null,"vision_escalated":false},
                  {"page_number":2,"text":"Second page.","source":"native","ocr_confidence":null,"vision_escalated":false}
                ],"warnings":[],"truncated":false,"optional_image":null}}}"#,
        )
        .unwrap();
        assert_eq!(source.pages.len(), 2);
        assert_eq!(source.pages[1].page_number, 2);
        assert_eq!(source.pages[0].origin, PageOrigin::Native);
        assert!(source.parser_warnings.is_empty());
    }

    /// A page's layout comes through beside its text; a page without one,
    /// from a worker that predates layouts, comes through as it always did.
    #[test]
    fn a_page_layout_survives_the_protocol() {
        let source = parsed(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"parsed","document":{
                "pages":[
                  {"page_number":1,"text":"INVOICE\n\nInvoice date: May 1, 2026","source":"native","ocr_confidence":null,"vision_escalated":false,
                   "layout":{"width":6120,"height":7920,"route":"fast","signals":{"chars":30},"blocks":[
                     {"id":"p1.b1","kind":"heading","text":"INVOICE","bbox":null,"source":"native","lines":[{"text":"INVOICE"}]},
                     {"id":"p1.b2","kind":"key_value","text":"Invoice date: May 1, 2026","bbox":null,"section":"p1.b1","source":"native",
                      "lines":[{"text":"Invoice date: May 1, 2026"}],
                      "fields":[{"id":"p1.b2.f1","key":"Invoice date","value":"May 1, 2026"}]}]}},
                  {"page_number":2,"text":"Second page.","source":"native","ocr_confidence":null,"vision_escalated":false}
                ],"warnings":[],"truncated":false,"optional_image":null}}}"#,
        )
        .unwrap();

        let layout = source.pages[0].layout.as_ref().unwrap();
        assert_eq!(layout.blocks[1].fields[0].value, "May 1, 2026");
        assert_eq!(source.pages[1].layout, None);
        let document = crate::structure::structured(&source);
        assert_eq!(
            document.block("p1.b2").unwrap().section.as_deref(),
            Some("p1.b1")
        );
        assert_eq!(document.block("p2.b1").unwrap().text, "Second page.");
    }

    #[test]
    fn low_confidence_ocr_becomes_a_fact_affecting_warning() {
        let source = parsed(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"parsed","document":{
                "pages":[{"page_number":1,"text":"scan","source":"ocr","ocr_confidence":41.5,"vision_escalated":true}],
                "warnings":[],"truncated":false,"optional_image":null}}}"#,
        )
        .unwrap();
        assert!(
            source
                .parser_warnings
                .iter()
                .any(|warning| warning.code == "LOW_OCR_CONFIDENCE" && warning.field_affecting)
        );
        assert_eq!(source.pages[0].ocr_confidence, Some(42));
    }

    #[test]
    fn a_duplicate_page_number_is_a_protocol_violation() {
        assert!(parsed(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"parsed","document":{
                "pages":[
                  {"page_number":1,"text":"a","source":"native","ocr_confidence":null,"vision_escalated":false},
                  {"page_number":1,"text":"b","source":"native","ocr_confidence":null,"vision_escalated":false}
                ],"warnings":[],"truncated":false,"optional_image":null}}}"#
        )
        .is_err());
    }

    #[test]
    fn an_image_for_a_page_that_does_not_exist_is_refused() {
        assert!(parsed(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"parsed","document":{
                "pages":[{"page_number":1,"text":"a","source":"native","ocr_confidence":null,"vision_escalated":false}],
                "warnings":[],"truncated":false,
                "optional_image":{"page_number":9,"mime_type":"image/png","data_base64":"AAA="}}}}"#
        )
        .is_err());
    }

    /// A process that reads its standard input and stays there, standing in
    /// for a worker that has started and is waiting for a command.
    fn idle_helper() -> &'static str {
        if cfg!(windows) { "cmd.exe" } else { "cat" }
    }

    #[test]
    fn a_cancel_that_beats_the_parse_command_is_still_a_cancel() {
        let worker = SupervisedWorker::new("already-running");
        let process = Arc::new(launch(Path::new(idle_helper()), None).expect("a helper process"));
        *worker.running.lock().unwrap() = Some(Arc::clone(&process));
        *worker.active.lock().unwrap() = Some("r1".to_owned());

        // The cancel arrives first and kills the worker, as a cancel always
        // does.
        worker
            .cancel("r1")
            .expect("an active request is cancellable");

        // Extraction had taken its handle on that same worker before the
        // cancel landed, so the parse command it is about to write is going to
        // a process that is already gone.
        *worker.running.lock().unwrap() = Some(process);
        *worker.active.lock().unwrap() = None;

        let failure = worker
            .extract("r1", Path::new("document.pdf"), &mut |_| {})
            .expect_err("a parse command cannot reach a killed worker");
        assert_eq!(failure.code, "CANCELED");
        assert!(failure.canceled);
        assert!(!failure.retryable);
    }

    /// The worker says what went wrong and whether trying again can help. The
    /// host used to keep only the code and the hint and throw the message
    /// away, so nothing downstream could tell a password from a damaged file
    /// except by the code alone.
    #[test]
    fn a_worker_error_keeps_its_code_message_and_retry_hint() {
        let failure = match decode_worker_response(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"error",
                "code":"PASSWORD_PROTECTED","message":"document is password-protected","retryable":false}}"#,
        )
        .unwrap()
        .event
        {
            WorkerEvent::Error {
                code,
                message,
                retryable,
            } => request_failure(code, message, retryable),
            other => panic!("unexpected event: {other:?}"),
        };
        assert_eq!(failure.code, "PASSWORD_PROTECTED");
        assert_eq!(failure.message, "document is password-protected");
        assert!(!failure.retryable);
        assert!(!failure.crashed);
        assert!(!failure.canceled);

        let canceled = request_failure("CANCELED".into(), "request canceled".into(), false);
        assert!(canceled.canceled, "a worker-side cancel is still a cancel");
    }

    /// A worker reply carrying one page and the given warnings.
    fn one_page(text: &str, warnings: &[&str], truncated: bool) -> DocumentSource {
        let reply = json!({
            "protocol_version": PROTOCOL_VERSION,
            "request_id": "r",
            "event": {"type": "parsed", "document": {
                "pages": [{"page_number": 1, "text": text, "source": "any_doc",
                    "ocr_confidence": null, "vision_escalated": false}],
                "warnings": warnings,
                "truncated": truncated,
                "optional_image": null,
            }},
        });
        parsed(&reply.to_string()).unwrap()
    }

    /// A sheet whose facts are in its first rows and whose ledger runs on
    /// past the window, as the worker renders it.
    const LONG_SHEET: &str = "## Statement of Work\n\n\
STATEMENT OF WORK\n\n\
This Statement of Work is effective as of April 1, 2026, by and between Acme Corporation \
and Contoso Worldwide, Inc.\n\n\
The work covers the 2026 CRM implementation, its deliverables, and its fees.\n\n\
| Item | Amount |\n| --- | --- |\n| CRM licences | 1200 |\n| Onboarding | 800 |\n\n\
[... 150 more rows not shown]\n";

    fn long_sheet_proposal() -> crate::domain::ModelProposal {
        use crate::domain::{DateRole, Evidence, ModelProposal, PartyRelation};
        ModelProposal {
            document_type: Some("Statement of Work".into()),
            document_date: Some("2026-04-01".into()),
            date_role: Some(DateRole::Effective),
            parties: vec!["Acme Corporation".into(), "Contoso Worldwide, Inc.".into()],
            party_relation: PartyRelation::Between,
            description:
                "Statement of work between Acme Corporation and Contoso Worldwide, Inc. covering the 2026 CRM implementation and its fees."
                    .into(),
            confidence: 0.9,
            needs_review: false,
            evidence: Evidence {
                date: Some("effective as of April 1, 2026".into()),
                document_type: Some("STATEMENT OF WORK".into()),
                parties: vec![
                    "by and between Acme Corporation and Contoso Worldwide, Inc.".into(),
                ],
            },
            facts: None,
        }
    }

    fn validated(source: &DocumentSource) -> crate::domain::ValidationOutcome {
        let digest = crate::distill::distill(source, crate::distill::DigestBudget::default());
        crate::validate::validate(long_sheet_proposal(), &digest)
    }

    /// Rows past a spreadsheet's window are left out by design and marked
    /// where they are. That says the sheet is long, not that anything read
    /// from it is in doubt, so a proposal the sheet supports is still Ready.
    /// Text that was cut is a different matter and still goes to review.
    #[test]
    fn content_elided_is_not_field_affecting() {
        use crate::domain::{ProposalStatus, ReviewReason};

        let elided = one_page(LONG_SHEET, &["CONTENT_ELIDED"], false);
        assert_eq!(
            elided.parser_warnings,
            vec![ParserWarning::new("CONTENT_ELIDED", false)]
        );
        let outcome = validated(&elided);
        assert_eq!(
            outcome.status,
            ProposalStatus::Ready,
            "{:?}",
            outcome.reasons
        );
        assert!(!outcome.reasons.contains(&ReviewReason::ParserWarning));

        let truncated = one_page(LONG_SHEET, &["TEXT_TRUNCATED"], true);
        assert_eq!(
            truncated.parser_warnings,
            vec![ParserWarning::new("TEXT_TRUNCATED", true)]
        );
        let outcome = validated(&truncated);
        assert_eq!(outcome.status, ProposalStatus::NeedsReview);
        assert!(outcome.reasons.contains(&ReviewReason::ParserWarning));

        // A code this build has never heard of errs towards review.
        let unknown = one_page(LONG_SHEET, &["SOMETHING_NEW"], false);
        assert_eq!(
            unknown.parser_warnings,
            vec![ParserWarning::new("SOMETHING_NEW", true)]
        );
    }

    /// Reads everything `forward_lines` sends for the given output.
    fn forwarded(output: impl Read, limit: usize) -> Vec<Result<String, ()>> {
        let (sender, receiver) = mpsc::channel();
        forward_lines(output, &sender, limit);
        drop(sender);
        receiver.into_iter().collect()
    }

    /// The worker's reply lines are read with a bound. A line past it is a
    /// worker gone wrong: it is refused once the bound is passed, never
    /// read to its end, and reported as the crash the caller already knows
    /// how to recover from. Nothing after it is read.
    #[test]
    fn oversized_response_line_fails_cleanly() {
        assert_eq!(
            forwarded(&b"{\"a\":1}\r\n{\"b\":2}\nlast"[..], 16),
            vec![
                Ok("{\"a\":1}".to_owned()),
                Ok("{\"b\":2}".to_owned()),
                Ok("last".to_owned()),
                Err(()),
            ]
        );
        assert_eq!(
            forwarded(&b"short\n0123456789abcdefXYZ\nnever read\n"[..], 16),
            vec![Ok("short".to_owned()), Err(())]
        );
        // A line exactly at the bound is still a line.
        assert_eq!(
            forwarded(&b"0123456789abcdef\n"[..], 16),
            vec![Ok("0123456789abcdef".to_owned()), Err(())]
        );

        // At the real bound, an endless line costs the bound and no more.
        let endless = std::io::repeat(b'x');
        assert_eq!(forwarded(endless, MAX_RESPONSE_LINE_BYTES), vec![Err(())]);
    }

    /// The worker's standard error is kept beside the server's when a log
    /// directory is set, in a file emptied once it passes the cap.
    #[cfg(unix)]
    #[test]
    fn worker_stderr_goes_to_its_log_when_directory_set() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let logs = directory.path().join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::write(logs.join(WORKER_LOG), vec![b'x'; 300 * 1024]).unwrap();
        // A worker that reports a warning code and then waits for commands.
        let helper = directory.path().join("worker.sh");
        std::fs::write(
            &helper,
            "#!/bin/sh\necho '{\"level\":\"warning\",\"code\":\"PARSE_FAILED\"}' >&2\nexec cat\n",
        )
        .unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o700)).unwrap();

        // A script written a moment ago can briefly be "text file busy" while
        // another test's fork still holds the write handle; try again.
        let process = (0..20)
            .find_map(|_| {
                launch_logged(&helper, None, Some(&logs)).ok().or_else(|| {
                    std::thread::sleep(Duration::from_millis(50));
                    None
                })
            })
            .expect("the helper starts");
        let deadline = Instant::now() + Duration::from_secs(10);
        let log = logs.join(WORKER_LOG);
        while std::fs::read_to_string(&log).unwrap_or_default().is_empty()
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        process.terminate();
        assert_eq!(
            std::fs::read_to_string(&log).unwrap(),
            "{\"level\":\"warning\",\"code\":\"PARSE_FAILED\"}\n"
        );
    }

    /// The worker's report of where the time went is accepted beside the
    /// pages, and changes nothing about the document built from them.
    #[test]
    fn worker_timings_are_accepted_and_change_nothing_else() {
        let timed = decode_worker_response(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"parsed","document":{
                "pages":[{"page_number":1,"text":"scan","source":"ocr","ocr_confidence":88.0,"vision_escalated":false}],
                "warnings":[],"truncated":false,"optional_image":null,
                "timings":{"total_micros":5100,"snapshot_micros":40,"parse_micros":300,
                  "analysis_micros":20,"render_micros":900,"image_decode_micros":0,
                  "ocr_micros":3800,"ocr_encode_micros":600,"ocr_engine_micros":3100,
                  "vision_micros":0,"ocr_pages":1,"ocr_passes":2,"orientation_passes":1,
                  "rendered_pixels":8415000,"a_stage_from_a_newer_worker":7}}}}"#,
        )
        .unwrap();
        let WorkerEvent::Parsed { document } = timed.event else {
            panic!("a parsed event");
        };
        assert_eq!(
            document.timings,
            Some(ExtractionTimings {
                total_micros: 5_100,
                snapshot_micros: 40,
                parse_micros: 300,
                analysis_micros: 20,
                render_micros: 900,
                image_decode_micros: 0,
                ocr_micros: 3_800,
                ocr_encode_micros: 600,
                ocr_engine_micros: 3_100,
                vision_micros: 0,
                ocr_pages: 1,
                ocr_passes: 2,
                orientation_passes: 1,
                rendered_pixels: 8_415_000,
            })
        );
        let untimed = parsed(
            r#"{"protocol_version":1,"request_id":"r","event":{"type":"parsed","document":{
                "pages":[{"page_number":1,"text":"scan","source":"ocr","ocr_confidence":88.0,"vision_escalated":false}],
                "warnings":[],"truncated":false,"optional_image":null}}}"#,
        )
        .unwrap();
        assert_eq!(adapt_document(document).unwrap(), untimed);

        // A stage this build expects but was not sent reads as zero.
        let partial: ExtractionTimings =
            serde_json::from_str(r#"{"total_micros":9,"ocr_pages":2}"#).unwrap();
        assert_eq!(partial.total_micros, 9);
        assert_eq!(partial.ocr_pages, 2);
        assert_eq!(partial.render_micros, 0);
    }

    /// A stand-in worker that answers the handshake and then each parse
    /// request by its id: `timed` with a document and its timings, anything
    /// else with the same document as a worker that predates them sends it.
    #[cfg(unix)]
    fn scripted_worker(directory: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let document = r#""pages":[{"page_number":1,"text":"Harbourline Storage Co. lease","source":"native","ocr_confidence":null,"vision_escalated":false}],"warnings":[],"truncated":false,"optional_image":null"#;
        let script = format!(
            "#!/bin/sh\n\
             read hello\n\
             echo '{{\"protocol_version\":1,\"request_id\":\"hello\",\"event\":{{\"type\":\"hello\",\"worker_version\":\"test\"}}}}'\n\
             while read line; do\n\
             case \"$line\" in\n\
             *'\"timed\"'*)\n\
             echo '{{\"protocol_version\":1,\"request_id\":\"timed\",\"event\":{{\"type\":\"progress\",\"stage\":\"extracting\",\"current\":0,\"total\":null}}}}'\n\
             echo '{{\"protocol_version\":1,\"request_id\":\"timed\",\"event\":{{\"type\":\"parsed\",\"document\":{{{document},\"timings\":{{\"total_micros\":2500,\"snapshot_micros\":100,\"parse_micros\":2300,\"ocr_pages\":0}}}}}}}}' ;;\n\
             *'\"parse\"'*)\n\
             id=$(echo \"$line\" | sed 's/.*\"request_id\":\"\\([^\"]*\\)\".*/\\1/')\n\
             echo '{{\"protocol_version\":1,\"request_id\":\"'\"$id\"'\",\"event\":{{\"type\":\"parsed\",\"document\":{{{document}}}}}}}' ;;\n\
             esac\n\
             done\n"
        );
        let helper = directory.join("worker.sh");
        std::fs::write(&helper, script).unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o700)).unwrap();
        helper
    }

    /// `extract_timed` hands back what the worker reported about its time
    /// beside the document, and nothing when a worker reported nothing;
    /// `extract` is the same document either way.
    #[cfg(unix)]
    #[test]
    fn extract_timed_returns_the_workers_timings() {
        let directory = tempfile::tempdir().unwrap();
        let worker = SupervisedWorker::new(scripted_worker(directory.path()));
        // A script written a moment ago can briefly be "text file busy" while
        // another test's fork still holds the write handle; try again.
        let mut progress = Vec::new();
        let (timed, timings) = (0..20)
            .find_map(|_| {
                match worker.extract_timed("timed", Path::new("lease.pdf"), &mut |event| {
                    progress.push(event)
                }) {
                    Err(failure) if failure.code == "WORKER_CRASHED" => {
                        std::thread::sleep(Duration::from_millis(50));
                        None
                    }
                    result => Some(result),
                }
            })
            .expect("the stand-in worker starts")
            .unwrap();

        assert_eq!(
            timings,
            Some(ExtractionTimings {
                total_micros: 2_500,
                snapshot_micros: 100,
                parse_micros: 2_300,
                ..ExtractionTimings::default()
            })
        );
        assert_eq!(timed.pages[0].text, "Harbourline Storage Co. lease");
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].stage, "extracting");

        let (untimed, none) = worker
            .extract_timed("older", Path::new("lease.pdf"), &mut |_| {})
            .unwrap();
        assert_eq!(none, None);
        assert_eq!(untimed, timed);
        let extracted = worker
            .extract("plain", Path::new("lease.pdf"), &mut |_| {})
            .unwrap();
        assert_eq!(extracted, timed);
        worker.stop();
    }

    #[test]
    fn an_unsupported_protocol_version_is_rejected() {
        assert_eq!(
            decode_worker_response(
                r#"{"protocol_version":2,"request_id":"r","event":{"type":"hello","worker_version":"x"}}"#
            )
            .unwrap_err()
            .code,
            "PROTOCOL_VERSION_UNSUPPORTED"
        );
    }
}
