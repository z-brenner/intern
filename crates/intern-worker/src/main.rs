use std::io::Write;
use std::path::{Path, PathBuf};

use intern_worker::extract::{
    CancellationToken, ExtractedDocument, ExtractionError, OcrBackend, RenderedPage,
    extract_anydoc, extract_image, extract_pdf, extract_text, snapshot_source,
};
use intern_worker::limits::ResourceLimits;
use intern_worker::ocr::TesseractOcr;
use intern_worker::pdf::PdfiumBackend;

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

fn ocr_backend() -> Result<TesseractOcr, ExtractionError> {
    let runtime = runtime_directory()?;
    TesseractOcr::new(runtime.join("tesseract.exe"), runtime.join("tessdata"))
}

/// Builds the OCR engine the first time a page actually needs it.
///
/// Around ninety-nine per cent of documents carry usable text, and those
/// documents must not fail, wait, or load anything because an OCR engine
/// happens to be unavailable.
struct LazyOcr {
    engine: std::sync::OnceLock<Result<TesseractOcr, ExtractionError>>,
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
    let snapshot = snapshot_source(&path, &limits, &cancel)?;
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
    match reader {
        Reader::Office => extract_anydoc(path, &limits, &cancel),
        Reader::Workbook => intern_worker::sheet::extract_xlsx(path, &limits, &cancel),
        Reader::LegacyWorkbook => intern_worker::sheet::extract_xls(path, &limits, &cancel),
        Reader::OpenWorkbook => intern_worker::sheet::extract_ods(path, &limits, &cancel),
        Reader::Delimited => intern_worker::delimited::extract_delimited(path, &limits, &cancel),
        Reader::Eml => intern_worker::email::extract_eml(path, &limits, &cancel),
        Reader::Msg => intern_worker::email::extract_msg(path, &limits, &cancel),
        Reader::Text => extract_text(path, &limits, &cancel),
        Reader::Pdf => extract_pdf(path, &pdf_backend()?, &LAZY_OCR, &limits, &cancel),
        Reader::Image => extract_image(path, &LAZY_OCR, &limits, &cancel),
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
