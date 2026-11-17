//! The document viewer: a reviewer reads the document they are naming inside
//! Intern's own window, as the file itself where the window can draw it and
//! as the text Intern read from it, page by page.
//!
//! Both commands take an item's id, never a path, under the same rules as
//! Open (`document_to_open`): the document where it is now - the filed copy
//! once renamed, the file as it arrived before that - refused for an uploader
//! Intern could not verify, and only in a format Intern reads. Nothing here
//! widens what the webview can reach on disk: the bytes come back through
//! the command, not through a file URL or the asset protocol.

use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use intern_core::{FileSystem, StdFileSystem, is_transient_lock};
use intern_engine::{
    DocumentExtractor, DocumentSource, ExtractFailure, PageOrigin, SupervisedWorker,
};
use intern_queue::paths::parse_item_id;
use serde::Serialize;
use tauri::State;

use super::{AppState, CommandError, background_task_failed, document_to_open};

/// The largest file the viewer is handed whole. A PDF or a picture of this
/// size still draws in the window in a few seconds; a larger one is read in
/// its own program (Open), and its text is still shown.
pub(crate) const VIEWER_MAX_BYTES: u64 = 50 * 1024 * 1024;

/// What the window draws itself, by extension: PDF, with the renderer the
/// window bundles, and the pictures every webview shows. A TIFF scan is not
/// among them - neither webview Intern ships on draws one - and is shown as
/// the text Intern read from it.
const DRAWN_IN_WINDOW: &[&str] = &["pdf", "png", "jpg", "jpeg"];

/// Whether the window draws a document of this name itself.
pub(crate) fn drawn_in_window(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| DRAWN_IN_WINDOW.contains(&extension.to_ascii_lowercase().as_str()))
}

/// The bytes of a document the window draws itself, read whole, refused past
/// `max` - also when the file grows past it while it is read.
pub(crate) fn viewer_bytes(path: &Path, max: u64) -> Result<Vec<u8>, CommandError> {
    if !drawn_in_window(path) {
        return Err(CommandError {
            code: "VIEW_UNSUPPORTED".into(),
            message: "the window shows this format as the text Intern read".into(),
        });
    }
    let too_large = || CommandError {
        code: "VIEW_TOO_LARGE".into(),
        message: format!(
            "the document is larger than the {} MB the window shows",
            max / (1024 * 1024)
        ),
    };
    let file = File::open(path).map_err(read_failed)?;
    let length = file.metadata().map_err(read_failed)?.len();
    if length > max {
        return Err(too_large());
    }
    let mut bytes = Vec::with_capacity(usize::try_from(length).unwrap_or_default());
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(read_failed)?;
    if bytes.len() as u64 > max {
        return Err(too_large());
    }
    Ok(bytes)
}

/// A read refused by the file system, said as the window says the same of
/// Open and of a rename: held by another program, gone, or unreadable.
fn read_failed(error: io::Error) -> CommandError {
    if is_transient_lock(&error) {
        CommandError {
            code: "SOURCE_LOCKED".into(),
            message: "another program is holding the document".into(),
        }
    } else if error.kind() == io::ErrorKind::NotFound {
        CommandError {
            code: "PATH_UNAVAILABLE".into(),
            message: "the document is not where Intern last saw it".into(),
        }
    } else {
        CommandError {
            code: "IO_ERROR".into(),
            message: "the document could not be read".into(),
        }
    }
}

/// One page of the text Intern read.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerPageDto {
    pub number: usize,
    pub text: String,
    /// Recognized from a picture of the page rather than taken from its text.
    pub ocr: bool,
}

/// The text Intern read from a document, page by page.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerTextDto {
    pub pages: Vec<ViewerPageDto>,
    /// Read again for the viewer: the queue no longer kept its own reading.
    pub read_now: bool,
    /// The file is no longer the one Intern read, so what was read again
    /// is the file as it is now.
    pub changed: bool,
}

pub(crate) fn text_dto(source: &DocumentSource, read_now: bool, changed: bool) -> ViewerTextDto {
    ViewerTextDto {
        pages: source
            .pages
            .iter()
            .map(|page| ViewerPageDto {
                number: page.page_number,
                text: page.text.clone(),
                ocr: page.origin == PageOrigin::Ocr,
            })
            .collect(),
        read_now,
        changed,
    }
}

/// Reads a document for the viewer when the queue kept nothing of its own
/// reading - a document read before this run of Intern, or one let go from
/// the extraction memo since. It has its own parser worker, started for the
/// one reading and stopped after it, so it never waits on the queue's, nor
/// holds it up. One reading at a time: a new one stops the one before,
/// which nobody is looking at any more, rather than wait for its OCR.
pub(crate) struct ViewerReader {
    executable: PathBuf,
    temp_root: PathBuf,
    current: Mutex<Option<Arc<SupervisedWorker>>>,
}

impl ViewerReader {
    pub(crate) fn new(executable: PathBuf, temp_root: PathBuf) -> Self {
        Self {
            executable,
            temp_root,
            current: Mutex::new(None),
        }
    }

    fn read(&self, path: &Path) -> Result<DocumentSource, CommandError> {
        let worker = Arc::new(SupervisedWorker::with_temp_root(
            &self.executable,
            &self.temp_root,
        ));
        let previous = self
            .current
            .lock()
            .ok()
            .and_then(|mut current| current.replace(Arc::clone(&worker)));
        if let Some(previous) = previous {
            previous.stop();
        }
        let read = worker.extract("viewer", path, &mut |_| {});
        worker.stop();
        if let Ok(mut current) = self.current.lock()
            && current
                .as_ref()
                .is_some_and(|reading| Arc::ptr_eq(reading, &worker))
        {
            *current = None;
        }
        read.map_err(|failure| extraction_failed(&failure))
    }
}

/// The review code a failed reading is told by, as the queue tells the same
/// failure of the same document (`extraction_error_code` in intern-queue).
pub(crate) fn extraction_failed(failure: &ExtractFailure) -> CommandError {
    let code = match failure.code.as_str() {
        "PASSWORD_PROTECTED" => "PASSWORD_PROTECTED",
        "UNSUPPORTED_FORMAT" => "UNSUPPORTED_CONTENT",
        "RESOURCE_LIMIT_EXCEEDED" | "RESOURCE_LIMIT" => "DOCUMENT_TOO_LARGE",
        "NATIVE_ASSETS_MISSING" => "OCR_UNAVAILABLE",
        _ => "EXTRACTION_FAILED",
    };
    CommandError {
        code: code.into(),
        message: "Intern could not read the document's text".into(),
    }
}

/// The file itself, for the window to draw: a PDF or a PNG or JPEG picture
/// of at most `VIEWER_MAX_BYTES`, as raw bytes.
///
/// Blocking: the document can be on a network share, and is read whole.
#[tauri::command]
pub async fn document_file(
    id: String,
    state: State<'_, AppState>,
) -> Result<tauri::ipc::Response, CommandError> {
    let pipeline = Arc::clone(&state.pipeline);
    let id = parse_item_id(&id)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<tauri::ipc::Response, CommandError> {
        let items = pipeline.list()?;
        let path = document_to_open(items.iter().find(|item| item.id == id))?;
        viewer_bytes(&path, VIEWER_MAX_BYTES).map(tauri::ipc::Response::new)
    })
    .await
    .map_err(|_| background_task_failed("view document"))?
}

/// The text Intern read from the document, page by page: what the queue
/// kept of its own reading when it still has it, and otherwise the document
/// read again now, saying whether the file has changed since Intern read it.
///
/// Blocking: reading again can be minutes of OCR for a long scan.
#[tauri::command]
pub async fn document_text(
    id: String,
    state: State<'_, AppState>,
) -> Result<ViewerTextDto, CommandError> {
    let pipeline = Arc::clone(&state.pipeline);
    let reader = Arc::clone(&state.viewer);
    let id = parse_item_id(&id)?;
    tauri::async_runtime::spawn_blocking(move || -> Result<ViewerTextDto, CommandError> {
        let items = pipeline.list()?;
        let item = items.iter().find(|item| item.id == id);
        let path = document_to_open(item)?;
        if let Some(source) = pipeline.remembered_extraction(id)? {
            return Ok(text_dto(&source, false, false));
        }
        let changed = item.is_some_and(|item| {
            StdFileSystem
                .hash(&path)
                .is_ok_and(|hash| hash != item.source_hash)
        });
        let source = reader.read(&path)?;
        Ok(text_dto(&source, true, changed))
    })
    .await
    .map_err(|_| background_task_failed("read document"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use intern_engine::SourcePage;

    fn temp_folder(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("intern-viewer-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    #[test]
    fn the_window_draws_pdfs_and_pictures_it_can_show_and_nothing_else() {
        for drawn in [
            "/filed/2024-03-01 Lease.pdf",
            "/filed/Scan.PDF",
            "/intake/photo.jpg",
            "/intake/photo.JPEG",
            "/intake/receipt.png",
        ] {
            assert!(drawn_in_window(Path::new(drawn)), "{drawn}");
        }
        // Read as text: Office files, mail, plain text, and a TIFF scan no
        // webview draws.
        for text_only in [
            "/intake/scan.tif",
            "/intake/scan.tiff",
            "/intake/letter.docx",
            "/intake/sheet.xlsx",
            "/intake/mail.msg",
            "/intake/notes.txt",
            "/intake/.pdf",
            "/intake/pdf",
        ] {
            assert!(!drawn_in_window(Path::new(text_only)), "{text_only}");
        }
    }

    #[test]
    fn the_viewer_is_handed_a_drawn_file_whole_and_nothing_past_its_limit() {
        let folder = temp_folder("bytes");
        let pdf = folder.join("Lease.pdf");
        std::fs::write(&pdf, b"%PDF-1.7 lease").unwrap();
        assert_eq!(viewer_bytes(&pdf, 1024).unwrap(), b"%PDF-1.7 lease");
        // Exactly at the limit is shown; one byte past it is not.
        assert_eq!(viewer_bytes(&pdf, 14).unwrap().len(), 14);
        assert_eq!(viewer_bytes(&pdf, 13).unwrap_err().code, "VIEW_TOO_LARGE");

        // A format the window does not draw is never read for it.
        let docx = folder.join("Letter.docx");
        std::fs::write(&docx, b"PK").unwrap();
        assert_eq!(
            viewer_bytes(&docx, 1024).unwrap_err().code,
            "VIEW_UNSUPPORTED"
        );

        // Gone behind Intern's back.
        assert_eq!(
            viewer_bytes(&folder.join("gone.pdf"), 1024)
                .unwrap_err()
                .code,
            "PATH_UNAVAILABLE"
        );
        let _ = std::fs::remove_dir_all(&folder);
    }

    // The viewer finds the document as Open does: the filed copy of a
    // completed item, the file as it arrived otherwise, never a path the
    // window names, and nothing from an uploader Intern could not verify.
    #[test]
    fn the_viewer_reads_the_document_where_the_item_says_it_is_now() {
        use super::super::document_path_tests::item;
        use intern_core::ErrorCode;

        let folder = temp_folder("where");
        let arrived = folder.join("Scan 0102.pdf");
        std::fs::write(&arrived, b"%PDF arrived").unwrap();
        let mut waiting = item(QueueStatus::NeedsReview, None);
        waiting.source_path = arrived.clone();
        let path = document_to_open(Some(&waiting)).unwrap();
        assert_eq!(path, arrived);
        assert_eq!(
            viewer_bytes(&path, VIEWER_MAX_BYTES).unwrap(),
            b"%PDF arrived"
        );

        waiting.error_code = Some(ErrorCode::UploaderUnverified);
        assert_eq!(
            document_to_open(Some(&waiting)).unwrap_err().code,
            "UPLOADER_UNVERIFIED"
        );
        assert_eq!(document_to_open(None).unwrap_err().code, "ITEM_NOT_FOUND");
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[test]
    fn the_text_is_said_page_by_page_with_the_recognized_pages_marked() {
        let page = |number: usize, text: &str, origin: PageOrigin| SourcePage {
            page_number: number,
            text: text.into(),
            origin,
            ocr_confidence: None,
            layout: None,
            volatile_spans: Vec::new(),
        };
        let source = DocumentSource::from_pages(vec![
            page(1, "INVOICE INV-3301", PageOrigin::Native),
            page(2, "Total due", PageOrigin::Ocr),
        ]);
        let dto = text_dto(&source, true, false);
        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            serde_json::json!({
                "pages": [
                    { "number": 1, "text": "INVOICE INV-3301", "ocr": false },
                    { "number": 2, "text": "Total due", "ocr": true },
                ],
                "readNow": true,
                "changed": false,
            })
        );
    }

    #[test]
    fn a_failed_reading_is_told_by_the_review_code_the_queue_gives_it() {
        for (worker, code) in [
            ("PASSWORD_PROTECTED", "PASSWORD_PROTECTED"),
            ("UNSUPPORTED_FORMAT", "UNSUPPORTED_CONTENT"),
            ("RESOURCE_LIMIT", "DOCUMENT_TOO_LARGE"),
            ("RESOURCE_LIMIT_EXCEEDED", "DOCUMENT_TOO_LARGE"),
            ("NATIVE_ASSETS_MISSING", "OCR_UNAVAILABLE"),
            ("PARSE_FAILED", "EXTRACTION_FAILED"),
            ("WORKER_CRASHED", "EXTRACTION_FAILED"),
        ] {
            assert_eq!(
                extraction_failed(&ExtractFailure::new(worker, false, false)).code,
                code,
                "{worker}"
            );
        }
    }

    use intern_core::QueueStatus;
}
