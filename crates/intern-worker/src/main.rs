use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use intern_worker::extract::{
    CancellationToken, ExtractedDocument, ExtractionError, OcrBackend, RenderedPage,
    extract_anydoc, extract_image, extract_pdf, extract_text, snapshot_source,
};
use intern_worker::limits::ResourceLimits;
use intern_worker::ocr::TesseractOcr;
use intern_worker::pdf::PdfiumBackend;
use intern_worker::timing::micros_since;

fn runtime_directory() -> Result<PathBuf, ExtractionError> {
    if let Some(path) = std::env::var_os("INTERN_RUNTIME_DIR") {
        return Ok(path.into());
    }
    let executable = std::env::current_exe().map_err(ExtractionError::io)?;
    executable.parent().map(Path::to_path_buf).ok_or_else(|| {
        ExtractionError::native_assets_missing("worker executable has no parent directory")
    })
}

fn pdf_backend() -> Result<PdfiumBackend, ExtractionError> {
    PdfiumBackend::new(runtime_directory()?)
}

/// An OCR engine any number of pages can be read with at once.
type SharedOcr = Box<dyn OcrBackend + Send + Sync>;

/// The engines that read scans, in the order the worker prefers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OcrEngine {
    Indra,
    Paddle,
    Tesseract,
}

impl OcrEngine {
    /// The name `INTERN_OCR_ENGINE` takes and the log writes.
    fn id(self) -> &'static str {
        match self {
            OcrEngine::Indra => "indra",
            OcrEngine::Paddle => "paddle",
            OcrEngine::Tesseract => "tesseract",
        }
    }

    /// The name a person reads in the log.
    fn name(self) -> &'static str {
        match self {
            OcrEngine::Indra => "Indra OCR",
            OcrEngine::Paddle => "PP-OCR",
            OcrEngine::Tesseract => "Tesseract",
        }
    }
}

/// The environment variable that names the engine to read scans with, for
/// benchmarking one engine against another and for support: `indra`,
/// `paddle` or `tesseract`. Unset, empty or `auto`, the worker takes the
/// best engine installed.
const ENGINE_VARIABLE: &str = "INTERN_OCR_ENGINE";

/// The engine `value` of [`ENGINE_VARIABLE`] asks for: none for the
/// default, and an error naming a value the worker does not know.
fn requested_engine(value: Option<&str>) -> Result<Option<OcrEngine>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    match value.to_ascii_lowercase().as_str() {
        "auto" => Ok(None),
        "indra" => Ok(Some(OcrEngine::Indra)),
        "paddle" => Ok(Some(OcrEngine::Paddle)),
        "tesseract" => Ok(Some(OcrEngine::Tesseract)),
        _ => Err(format!(
            "{ENGINE_VARIABLE}={value:?} is not indra, paddle or tesseract; \
             reading scans with the best engine installed"
        )),
    }
}

/// [`requested_engine`] for this process, read once: an unknown value is
/// a warning in the log, written once, and the default.
fn configured_engine() -> Option<OcrEngine> {
    static REQUESTED: std::sync::OnceLock<Option<OcrEngine>> = std::sync::OnceLock::new();
    *REQUESTED.get_or_init(|| {
        let value = std::env::var_os(ENGINE_VARIABLE);
        let value = value.as_ref().map(|value| value.to_string_lossy());
        requested_engine(value.as_deref()).unwrap_or_else(|message| {
            log_warning("OCR_ENGINE_UNKNOWN", &message, &[]);
            None
        })
    })
}

/// One JSON line on standard error, which the host keeps in the worker's
/// log. `fields` are written before the message, as strings.
fn log_warning(code: &str, message: &str, fields: &[(&str, &str)]) {
    let quote = |text: &str| serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_owned());
    let mut line = format!("{{\"level\":\"warning\",\"code\":{}", quote(code));
    for (key, value) in fields {
        line.push_str(&format!(",{}:{}", quote(key), quote(value)));
    }
    line.push_str(&format!(",\"message\":{}}}", quote(message)));
    let _ = writeln!(std::io::stderr().lock(), "{line}");
}

/// An engine the worker meant to read scans with and did not, the one it
/// reads them with instead, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Fallback {
    from: OcrEngine,
    to: OcrEngine,
    message: String,
}

fn ocr_backend() -> Result<SharedOcr, ExtractionError> {
    let runtime = runtime_directory()?;
    select_ocr_backend(&runtime, configured_engine(), |fallback| {
        log_warning(
            "OCR_ENGINE_FALLBACK",
            &fallback.message,
            &[("from", fallback.from.id()), ("to", fallback.to.id())],
        );
    })
}

/// Indra OCR when its runtime and models are installed beside the worker
/// and load; PP-OCR when Indra's are not and PP-OCR's are; Tesseract,
/// exactly as before, when neither is. `requested`, from
/// [`ENGINE_VARIABLE`], starts the search at that engine instead.
///
/// An install without an engine's files is a development machine or a CI
/// runner, and says nothing. One that has them and cannot load them, or has
/// only some of them, is worth a line in the log, which `report` writes -
/// it names the engine given up on, the engine used instead, and the file
/// or the error, never a document. So is an engine that was asked for and
/// is not installed.
fn select_ocr_backend(
    runtime: &Path,
    requested: Option<OcrEngine>,
    mut report: impl FnMut(Fallback),
) -> Result<SharedOcr, ExtractionError> {
    let start = requested.unwrap_or(OcrEngine::Indra);
    // Why each engine before the one that loads was passed over, reported
    // once it is known which engine reads scans instead.
    let mut passed_over: Vec<(OcrEngine, String)> = Vec::new();
    let mut settle = |to: OcrEngine, passed_over: Vec<(OcrEngine, String)>| {
        for (from, why) in passed_over {
            report(Fallback {
                from,
                to,
                message: why.replace("{to}", to.name()),
            });
        }
    };
    if start <= OcrEngine::Indra {
        let assets = intern_worker::indra::IndraAssets::in_directory(runtime);
        if assets.present() {
            // Asked for by name, Indra reads alone, for comparing engines;
            // by default PP-OCR, when installed, reads the pages Indra is
            // not sure of.
            let second = requested.is_none().then_some(runtime);
            match indra_backend(assets, second) {
                Ok(engine) => return Ok(engine),
                Err(error) => passed_over.push((
                    OcrEngine::Indra,
                    format!("Indra OCR did not load; reading scans with {{to}}: {error}"),
                )),
            }
        } else if let Some(missing) = assets.partly_missing() {
            // Some of it is installed: something removed the rest - an
            // antivirus quarantine, a damaged update.
            passed_over.push((
                OcrEngine::Indra,
                format!(
                    "Indra OCR is incomplete ({} missing); reading scans with {{to}}",
                    missing.display()
                ),
            ));
        } else if requested == Some(OcrEngine::Indra) {
            passed_over.push((
                OcrEngine::Indra,
                format!("Indra OCR was requested by {ENGINE_VARIABLE} but is not installed; reading scans with {{to}}"),
            ));
        }
    }
    if start <= OcrEngine::Paddle {
        let assets = intern_worker::paddle::PaddleAssets::in_directory(runtime);
        if assets.present() {
            match paddle_backend(runtime) {
                Ok(engine) => {
                    settle(OcrEngine::Paddle, passed_over);
                    return Ok(engine);
                }
                Err(error) => passed_over.push((
                    OcrEngine::Paddle,
                    format!("PP-OCR did not load; reading scans with {{to}}: {error}"),
                )),
            }
        } else if let Some(missing) = assets.partly_missing() {
            passed_over.push((
                OcrEngine::Paddle,
                format!(
                    "PP-OCR is incomplete ({} missing); reading scans with {{to}}",
                    missing.display()
                ),
            ));
        } else if requested == Some(OcrEngine::Paddle) {
            passed_over.push((
                OcrEngine::Paddle,
                format!("PP-OCR was requested by {ENGINE_VARIABLE} but is not installed; reading scans with {{to}}"),
            ));
        }
    }
    settle(OcrEngine::Tesseract, passed_over);
    Ok(Box::new(ParallelTesseract(TesseractOcr::new(
        runtime.join("tesseract.exe"),
        runtime.join("tessdata"),
    )?)))
}

/// Indra OCR, with PP-OCR from `second`'s runtime directory as the second
/// reader of the pages it is not sure of when PP-OCR is installed there
/// ([`intern_worker::second_opinion`]).
#[cfg(feature = "onnx-ocr")]
fn indra_backend(
    assets: intern_worker::indra::IndraAssets,
    second: Option<&Path>,
) -> Result<SharedOcr, ExtractionError> {
    let indra: SharedOcr = Box::new(intern_worker::indra::IndraOcr::new(assets)?);
    let Some(runtime) = second
        .filter(|runtime| intern_worker::paddle::PaddleAssets::in_directory(runtime).present())
    else {
        return Ok(indra);
    };
    let runtime = runtime.to_path_buf();
    Ok(Box::new(intern_worker::second_opinion::SecondOpinion::new(
        indra,
        Box::new(move || paddle_backend(&runtime)),
        Box::new(|error| {
            log_warning(
                "OCR_SECOND_READING_UNAVAILABLE",
                &format!(
                    "PP-OCR did not load to read again the pages Indra OCR is unsure of: {error}"
                ),
                &[],
            );
        }),
    )))
}

#[cfg(not(feature = "onnx-ocr"))]
fn indra_backend(
    _assets: intern_worker::indra::IndraAssets,
    _second: Option<&Path>,
) -> Result<SharedOcr, ExtractionError> {
    Err(ExtractionError::native_assets_missing(
        "intern-worker was built without the onnx-ocr feature",
    ))
}

#[cfg(feature = "onnx-ocr")]
fn paddle_backend(runtime: &Path) -> Result<SharedOcr, ExtractionError> {
    Ok(Box::new(intern_worker::paddle::PaddleOcr::new(runtime)?))
}

#[cfg(not(feature = "onnx-ocr"))]
fn paddle_backend(_runtime: &Path) -> Result<SharedOcr, ExtractionError> {
    Err(ExtractionError::native_assets_missing(
        "intern-worker was built without the onnx-ocr feature",
    ))
}

/// Builds the OCR engine the first time a page actually needs it.
///
/// Around ninety-nine per cent of documents carry usable text, and those
/// documents must not fail, wait, or load anything because an OCR engine
/// happens to be unavailable.
struct LazyOcr {
    engine: std::sync::OnceLock<Result<SharedOcr, ExtractionError>>,
}

static LAZY_OCR: LazyOcr = LazyOcr {
    engine: std::sync::OnceLock::new(),
};

impl OcrBackend for LazyOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<intern_worker::extract::OcrResult, ExtractionError> {
        match self.engine.get_or_init(ocr_backend) {
            Ok(engine) => engine.recognize(page, cancel),
            Err(error) => Err(error.clone()),
        }
    }

    /// How many pages the engine reads at once. Asked before the first page
    /// is read, so it is answered from what is installed rather than by
    /// loading an engine a document with a text layer will never use.
    fn concurrency(&self) -> usize {
        match self.engine.get() {
            Some(Ok(engine)) => engine.concurrency(),
            _ => planned_ocr_workers(),
        }
    }

    /// The engine lets go of what it holds; one never loaded holds nothing.
    fn release(&self) {
        if let Some(Ok(engine)) = self.engine.get() {
            engine.release();
        }
    }
}

/// How many pages the engine [`select_ocr_backend`] would choose reads at
/// once: Indra's or PP-OCR's thread budget when either is installed - both
/// share [`intern_worker::paddle::thread_budget`] - and [`ocr_workers`]
/// Tesseract processes otherwise.
fn planned_ocr_workers() -> usize {
    let engine = runtime_directory().map_or(OcrEngine::Tesseract, |runtime| {
        planned_engine(&runtime, configured_engine())
    });
    match engine {
        OcrEngine::Indra | OcrEngine::Paddle => {
            intern_worker::paddle::default_thread_budget().workers
        }
        OcrEngine::Tesseract => ocr_workers(),
    }
}

/// The engine [`select_ocr_backend`] would choose, judged by which files
/// are installed and without loading any: the first engine from
/// `requested` on whose files are all there.
fn planned_engine(runtime: &Path, requested: Option<OcrEngine>) -> OcrEngine {
    let start = requested.unwrap_or(OcrEngine::Indra);
    let onnx = cfg!(feature = "onnx-ocr");
    if onnx
        && start <= OcrEngine::Indra
        && intern_worker::indra::IndraAssets::in_directory(runtime).present()
    {
        OcrEngine::Indra
    } else if onnx
        && start <= OcrEngine::Paddle
        && intern_worker::paddle::PaddleAssets::in_directory(runtime).present()
    {
        OcrEngine::Paddle
    } else {
        OcrEngine::Tesseract
    }
}

/// How many pages Tesseract reads at once: half the machine's logical
/// cores, so the app and the model server keep the rest, and never more
/// than four. Each page is a process of its own, held to one thread.
fn ocr_workers() -> usize {
    let cores = std::thread::available_parallelism().map_or(2, std::num::NonZero::get);
    (cores / 2).clamp(1, 4)
}

/// Tesseract with the number of pages it may read at once.
struct ParallelTesseract(TesseractOcr);

impl OcrBackend for ParallelTesseract {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<intern_worker::extract::OcrResult, ExtractionError> {
        self.0.recognize(page, cancel)
    }

    fn concurrency(&self) -> usize {
        ocr_workers()
    }
}

/// The reader a file is handed to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reader {
    /// Word processing and presentation formats, through anydoc to Markdown.
    Office,
    /// Excel 2007 workbooks through the capped streaming reader.
    Workbook,
    /// Excel 97-2003 workbooks through the guarded binary reader.
    LegacyWorkbook,
    /// OpenDocument workbooks, parsed by anydoc and cut to the same window.
    OpenWorkbook,
    Delimited,
    Eml,
    Msg,
    Text,
    Pdf,
    Image,
}

/// Every extension the worker reads, and what reads it. intern-core's
/// `SUPPORTED_EXTENSIONS` is what the app admits; a test below holds the two
/// to exactly the same set, so nothing is admitted that cannot be read and
/// nothing is readable that is never admitted.
const ROUTES: &[(&str, Reader)] = &[
    ("pdf", Reader::Pdf),
    ("docx", Reader::Office),
    ("docm", Reader::Office),
    ("doc", Reader::Office),
    ("rtf", Reader::Office),
    ("odt", Reader::Office),
    ("pptx", Reader::Office),
    ("pptm", Reader::Office),
    ("ppsx", Reader::Office),
    ("ppt", Reader::Office),
    ("odp", Reader::Office),
    // anydoc's OpenDocument reader has its own expansion limits, which a
    // spreadsheet format built on repeated-row runs needs; what it parsed is
    // then rendered through the same window as every other workbook.
    ("ods", Reader::OpenWorkbook),
    ("xlsx", Reader::Workbook),
    ("xlsm", Reader::Workbook),
    ("xls", Reader::LegacyWorkbook),
    ("csv", Reader::Delimited),
    ("eml", Reader::Eml),
    ("msg", Reader::Msg),
    ("txt", Reader::Text),
    ("md", Reader::Text),
    ("markdown", Reader::Text),
    ("png", Reader::Image),
    ("jpg", Reader::Image),
    ("jpeg", Reader::Image),
    ("tif", Reader::Image),
    ("tiff", Reader::Image),
];

/// The reader for a lowercase extension, if any reads it.
fn route(extension: &str) -> Option<Reader> {
    ROUTES
        .iter()
        .find(|(routed, _)| *routed == extension)
        .map(|(_, reader)| *reader)
}

fn extract_path(
    path: PathBuf,
    cancel: CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    let limits = ResourceLimits::default();
    let snapshot = cancel.timed(
        |timings| &mut timings.snapshot_micros,
        || snapshot_source(&path, &limits, &cancel),
    )?;
    let path = snapshot.path();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let Some(reader) = route(&extension) else {
        return Err(ExtractionError::unsupported(format!(
            "no reader handles a .{extension} file"
        )));
    };
    let reading = Instant::now();
    let document = read_with(reader, path, &limits, &cancel);
    // The worker waits for the next document beside the model server, which
    // is reading this one: OCR keeps none of this document's memory.
    LAZY_OCR.release();
    // PDFium's reader times binding the library and loading the format
    // apart from analysing pages; every other reader reads in one call, so
    // its time less the stages it reported is its parse.
    if reader != Reader::Pdf {
        let reader_micros = micros_since(reading);
        cancel.record(|timings| timings.attribute_remainder_to_parse(reader_micros));
    }
    document
}

/// Hands the snapshot to the reader its extension routes to.
fn read_with(
    reader: Reader,
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    match reader {
        Reader::Office => extract_anydoc(path, limits, cancel),
        Reader::Workbook => intern_worker::sheet::extract_xlsx(path, limits, cancel),
        Reader::LegacyWorkbook => intern_worker::sheet::extract_xls(path, limits, cancel),
        Reader::OpenWorkbook => intern_worker::sheet::extract_ods(path, limits, cancel),
        Reader::Delimited => intern_worker::delimited::extract_delimited(path, limits, cancel),
        Reader::Eml => intern_worker::email::extract_eml(path, limits, cancel),
        Reader::Msg => intern_worker::email::extract_msg(path, limits, cancel),
        Reader::Text => extract_text(path, limits, cancel),
        Reader::Pdf => {
            // Binding PDFium - the first PDF a worker process reads loads and
            // initialises the library - is part of reading the format, so it
            // is charged to parse rather than left in no stage.
            let backend = cancel.timed(|timings| &mut timings.parse_micros, pdf_backend)?;
            extract_pdf(path, &backend, &LAZY_OCR, limits, cancel)
        }
        Reader::Image => extract_image(path, &LAZY_OCR, limits, cancel),
    }
}

fn main() {
    // First, before anything can panic: standard error is kept in a log file,
    // and the default hook would print a panic's message there, document text
    // and all.
    intern_worker::panic_hook::install();
    if let Err(error) = intern_worker::protocol::run_concurrent_worker(
        std::io::stdin(),
        std::io::stdout(),
        std::io::stderr(),
        extract_path,
    ) {
        let _ = writeln!(
            std::io::stderr().lock(),
            "{{\"level\":\"error\",\"code\":\"WORKER_IO_FAILED\",\"message\":{}}}",
            serde_json::to_string(&error.to_string())
                .unwrap_or_else(|_| "\"worker I/O failed\"".to_owned())
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use intern_core::PrivateSnapshotDirectory;
    use intern_intake::microsoft::hashing::{QuickXor, verified_local_snapshot};

    #[test]
    fn verified_text_snapshot_routes_through_the_actual_worker_dispatch() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("notes.TXT");
        let contents = b"Verified snapshot routing\n";
        std::fs::write(&source, contents).unwrap();
        let snapshots =
            PrivateSnapshotDirectory::new(temporary.path().join("private-snapshots")).unwrap();
        let mut quick_xor = QuickXor::default();
        quick_xor.update(contents);
        let (_, snapshot) = verified_local_snapshot(
            &source,
            contents.len() as u64,
            &STANDARD.encode(quick_xor.finish()),
            &snapshots,
        )
        .unwrap();

        let document =
            extract_path(snapshot.path().to_path_buf(), CancellationToken::new()).unwrap();

        assert_eq!(document.pages[0].text, "Verified snapshot routing\n");
    }

    /// A reader with no parse figure of its own is charged its whole time
    /// less the stages it reported, and the snapshot before it is counted on
    /// its own.
    #[test]
    fn a_text_file_is_charged_its_snapshot_and_its_parse() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("notes.txt");
        std::fs::write(
            &source,
            "Meeting notes for the Harbourline lease.\n".repeat(2_000),
        )
        .unwrap();
        let cancel = CancellationToken::new();

        extract_path(source, cancel.clone()).unwrap();

        let timings = cancel.timings();
        assert!(timings.snapshot_micros > 0, "{timings:?}");
        assert!(timings.parse_micros > 0, "{timings:?}");
        assert_eq!(timings.ocr_micros, 0, "{timings:?}");
        // The protocol, not the dispatch, takes the whole extraction's time.
        assert_eq!(timings.total_micros, 0, "{timings:?}");
    }

    /// Files that stand in for an engine's: right names, wrong contents, so
    /// the selection sees the engine installed and its loading fails.
    fn stand_in(paths: &[&Path]) {
        for path in paths {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"not a library or a model").unwrap();
        }
    }

    fn stand_in_indra(runtime: &Path) -> intern_worker::indra::IndraAssets {
        let assets = intern_worker::indra::IndraAssets::in_directory(runtime);
        stand_in(&[
            &assets.runtime_library,
            &assets.detection_model,
            &assets.orientation_model,
            &assets.recognition_model,
            &assets.order_model,
            &assets.language_model,
            &assets.meta,
        ]);
        assets
    }

    fn stand_in_paddle(runtime: &Path) {
        let assets = intern_worker::paddle::PaddleAssets::in_directory(runtime);
        stand_in(&[
            &assets.runtime_library,
            &assets.detection_model,
            &assets.recognition_model,
        ]);
    }

    /// Selects with `requested`, keeping what it reports.
    fn select(
        runtime: &Path,
        requested: Option<OcrEngine>,
    ) -> (Result<SharedOcr, ExtractionError>, Vec<Fallback>) {
        let mut reported = Vec::new();
        let selected = select_ocr_backend(runtime, requested, |fallback| reported.push(fallback));
        (selected, reported)
    }

    /// Tesseract is built only with the native feature; without it the
    /// selection fails exactly as building Tesseract always has.
    fn assert_tesseract(selected: Result<SharedOcr, ExtractionError>) {
        if cfg!(feature = "native-tesseract") {
            // Tesseract pages are processes of their own, read side by side.
            assert_eq!(selected.unwrap().concurrency(), ocr_workers());
        } else {
            assert_eq!(selected.err().unwrap().code(), "NATIVE_ASSETS_MISSING");
        }
    }

    /// A runtime directory without ONNX Runtime and the models - every
    /// development machine and CI runner - reads scans with Tesseract as it
    /// always has, and says nothing about it.
    #[test]
    fn without_an_onnx_runtime_scans_go_to_tesseract_silently() {
        let runtime = tempfile::tempdir().unwrap();

        let (selected, reported) = select(runtime.path(), None);

        assert!(reported.is_empty(), "{reported:?}");
        assert_eq!(planned_engine(runtime.path(), None), OcrEngine::Tesseract);
        assert_tesseract(selected);
    }

    /// Files that look like the PP-OCR runtime but do not load are not a
    /// reason to fail a scan: the worker says so once, in the log, and reads
    /// with Tesseract.
    #[test]
    fn a_pp_ocr_runtime_that_does_not_load_falls_back_to_tesseract() {
        let runtime = tempfile::tempdir().unwrap();
        stand_in_paddle(runtime.path());

        let (selected, reported) = select(runtime.path(), None);

        assert_eq!(reported.len(), 1, "{reported:?}");
        assert_eq!(
            (reported[0].from, reported[0].to),
            (OcrEngine::Paddle, OcrEngine::Tesseract)
        );
        assert!(
            reported[0]
                .message
                .starts_with("PP-OCR did not load; reading scans with Tesseract"),
            "{reported:?}"
        );
        if cfg!(feature = "native-tesseract") {
            assert!(selected.is_ok());
        }
    }

    /// Indra comes first. An Indra install that does not load is reported
    /// as Indra given up for the engine that does read the scans - here
    /// neither ONNX engine loads, so each is named, in order, falling back
    /// to Tesseract. A smoke test can tell a fallback away from Indra from
    /// any other by its `from`.
    #[test]
    fn an_indra_install_that_does_not_load_is_reported_with_where_scans_went() {
        let runtime = tempfile::tempdir().unwrap();
        stand_in_indra(runtime.path());
        stand_in_paddle(runtime.path());

        let (selected, reported) = select(runtime.path(), None);

        let pairs = reported
            .iter()
            .map(|fallback| (fallback.from, fallback.to))
            .collect::<Vec<_>>();
        assert_eq!(
            pairs,
            [
                (OcrEngine::Indra, OcrEngine::Tesseract),
                (OcrEngine::Paddle, OcrEngine::Tesseract)
            ]
        );
        // Built with ONNX Runtime support, the stand-in library is what
        // fails to load; without it, the build says so.
        let reason = if cfg!(feature = "onnx-ocr") {
            "ONNX Runtime did not load from"
        } else {
            "intern-worker was built without the onnx-ocr feature"
        };
        assert!(
            reported[0].message.starts_with(&format!(
                "Indra OCR did not load; reading scans with Tesseract: {reason}"
            )),
            "{reported:?}"
        );
        assert_tesseract(selected);
    }

    /// Some of Indra's files and not the rest is something worth the log's
    /// line - an antivirus quarantine, a damaged update - and names the
    /// first file missing and the engine read with instead.
    #[test]
    fn a_partial_indra_install_is_reported() {
        let runtime = tempfile::tempdir().unwrap();
        let assets = intern_worker::indra::IndraAssets::in_directory(runtime.path());
        stand_in(&[&assets.detection_model, &assets.recognition_model]);

        let (selected, reported) = select(runtime.path(), None);

        assert_eq!(reported.len(), 1, "{reported:?}");
        assert_eq!(
            (reported[0].from, reported[0].to),
            (OcrEngine::Indra, OcrEngine::Tesseract)
        );
        assert!(
            reported[0].message.starts_with("Indra OCR is incomplete (")
                && reported[0]
                    .message
                    .contains(&assets.orientation_model.display().to_string())
                && reported[0]
                    .message
                    .ends_with("missing); reading scans with Tesseract"),
            "{reported:?}"
        );
        assert_eq!(planned_engine(runtime.path(), None), OcrEngine::Tesseract);
        assert_tesseract(selected);
    }

    /// `INTERN_OCR_ENGINE` starts the search at the engine it names: asking
    /// for Tesseract passes over the ONNX engines without a word, and
    /// asking for an engine that is not installed says so.
    #[test]
    fn a_requested_engine_starts_the_search() {
        let runtime = tempfile::tempdir().unwrap();
        stand_in_indra(runtime.path());
        stand_in_paddle(runtime.path());

        let (selected, reported) = select(runtime.path(), Some(OcrEngine::Tesseract));
        assert!(reported.is_empty(), "{reported:?}");
        assert_tesseract(selected);
        assert_eq!(
            planned_engine(runtime.path(), Some(OcrEngine::Tesseract)),
            OcrEngine::Tesseract
        );

        let empty = tempfile::tempdir().unwrap();
        let (_, reported) = select(empty.path(), Some(OcrEngine::Indra));
        assert_eq!(reported.len(), 1, "{reported:?}");
        assert_eq!(
            reported[0].message,
            "Indra OCR was requested by INTERN_OCR_ENGINE but is not installed; \
             reading scans with Tesseract"
        );
        let (_, reported) = select(empty.path(), Some(OcrEngine::Paddle));
        assert_eq!(
            reported
                .iter()
                .map(|fallback| (fallback.from, fallback.to))
                .collect::<Vec<_>>(),
            [(OcrEngine::Paddle, OcrEngine::Tesseract)]
        );
    }

    /// The worker plans its OCR workers by the engine it would choose, from
    /// the files installed: Indra before PP-OCR, unless PP-OCR is asked for.
    /// A build without ONNX Runtime support plans for Tesseract whatever is
    /// installed, since that is what it reads with.
    #[test]
    fn the_planned_engine_follows_the_files_installed() {
        let runtime = tempfile::tempdir().unwrap();
        stand_in_paddle(runtime.path());
        let onnx = |engine| {
            if cfg!(feature = "onnx-ocr") {
                engine
            } else {
                OcrEngine::Tesseract
            }
        };
        assert_eq!(
            planned_engine(runtime.path(), None),
            onnx(OcrEngine::Paddle)
        );
        stand_in_indra(runtime.path());
        assert_eq!(planned_engine(runtime.path(), None), onnx(OcrEngine::Indra));
        assert_eq!(
            planned_engine(runtime.path(), Some(OcrEngine::Paddle)),
            onnx(OcrEngine::Paddle)
        );
    }

    #[test]
    fn the_engine_setting_names_an_engine_or_the_default() {
        assert_eq!(requested_engine(None), Ok(None));
        assert_eq!(requested_engine(Some("")), Ok(None));
        assert_eq!(requested_engine(Some(" auto ")), Ok(None));
        assert_eq!(requested_engine(Some("indra")), Ok(Some(OcrEngine::Indra)));
        assert_eq!(
            requested_engine(Some("Paddle")),
            Ok(Some(OcrEngine::Paddle))
        );
        assert_eq!(
            requested_engine(Some("TESSERACT\n")),
            Ok(Some(OcrEngine::Tesseract))
        );
        let unknown = requested_engine(Some("ppocr")).unwrap_err();
        assert!(
            unknown.starts_with("INTERN_OCR_ENGINE=\"ppocr\" is not indra, paddle or tesseract"),
            "{unknown}"
        );
    }

    /// The admission list and the router are two lists that must name the
    /// same formats. They drifted once: `.docm` was routed here and refused
    /// by every admission check, so the reader for it could never run.
    #[test]
    fn every_admitted_extension_routes_and_every_route_is_admitted() {
        for extension in intern_core::SUPPORTED_EXTENSIONS {
            assert!(
                route(extension).is_some(),
                ".{extension} is admitted but unrouted"
            );
        }
        for (extension, _) in ROUTES {
            assert!(
                intern_core::SUPPORTED_EXTENSIONS.contains(extension),
                ".{extension} is routed but never admitted"
            );
        }
        assert_eq!(ROUTES.len(), intern_core::SUPPORTED_EXTENSIONS.len());
    }

    /// The dispatch itself, not just the table: a committed fixture of each
    /// new kind reaches a reader that reads it.
    #[test]
    fn the_new_formats_reach_a_reader_through_the_actual_dispatch() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/formats");
        for name in [
            "letter.doc",
            "letter.docm",
            "letter.rtf",
            "letter.odt",
            "deck.ppt",
            "deck.odp",
            "ledger.xls",
            "ledger.xlsm",
            "ledger.ods",
            "ledger.csv",
        ] {
            let document = extract_path(fixtures.join(name), CancellationToken::new())
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            let text = document
                .pages
                .iter()
                .map(|page| page.text.as_str())
                .collect::<String>();
            assert!(
                text.contains("Juniper Ridge Holdings Inc."),
                "{name}: {text}"
            );
        }
    }
}
