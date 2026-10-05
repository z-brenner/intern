//! A file saved with a password to open cannot be read, and saying so is the
//! whole of what extraction can do for it: the person removes the password
//! and adds the file again. Reported as a damaged file instead, it is
//! retried, then abandoned with a message that sends them looking for
//! corruption that is not there.

use std::io::Write;
use std::path::Path;

use intern_worker::extract::{CancellationToken, extract_anydoc};
use intern_worker::limits::ResourceLimits;
use intern_worker::sheet::extract_xlsx;

/// What Office writes for a password-protected `.docx` or `.xlsx`: not a
/// zip at all, but an OLE compound file holding the encryption parameters
/// and the encrypted package.
fn encrypted_package(path: &Path) {
    let handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    let mut file = cfb::CompoundFile::create(handle).unwrap();
    file.create_stream("/EncryptionInfo")
        .unwrap()
        .write_all(&[4, 0, 4, 0, 0x40, 0, 0, 0])
        .unwrap();
    file.create_stream("/EncryptedPackage")
        .unwrap()
        .write_all(&[0xA5; 4096])
        .unwrap();
    file.flush().unwrap();
}

#[test]
fn encrypted_ooxml_is_password_protected() {
    let directory = tempfile::tempdir().unwrap();
    let limits = ResourceLimits::default();
    let cancel = CancellationToken::new();

    for name in [
        "engagement-letter.docx",
        "engagement-letter.docm",
        "deck.pptx",
    ] {
        let path = directory.path().join(name);
        encrypted_package(&path);
        let error = extract_anydoc(&path, &limits, &cancel).unwrap_err();
        assert_eq!(error.code(), "PASSWORD_PROTECTED", "{name}: {error}");
        assert!(!error.retryable(), "{name}");
        assert_eq!(error.to_string(), "document is password-protected");
    }
    for name in ["ledger.xlsx", "ledger.xlsm"] {
        let path = directory.path().join(name);
        encrypted_package(&path);
        let error = extract_xlsx(&path, &limits, &cancel).unwrap_err();
        assert_eq!(error.code(), "PASSWORD_PROTECTED", "{name}: {error}");
        assert!(!error.retryable(), "{name}");
    }
}

/// The compound-file signature alone proves nothing - every `.doc`, `.xls`
/// and `.msg` has it. Only the encryption streams make a file a locked one.
#[test]
fn an_ordinary_compound_file_is_not_mistaken_for_an_encrypted_one() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/formats/letter.doc");
    let document =
        extract_anydoc(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    assert!(
        document.pages[0]
            .text
            .contains("Juniper Ridge Holdings Inc.")
    );
}

/// The corpus's own encrypted PDF, through the pinned PDFium. It needs the
/// native runtime this build was configured with, so it runs where
/// `INTERN_RUNTIME_DIR` points at one and the worker was built with
/// `native-pdfium`.
#[cfg(feature = "native-pdfium")]
#[test]
fn encrypted_pdf_is_password_protected() {
    use intern_worker::extract::{
        ExtractionError, OcrBackend, OcrResult, RenderedPage, extract_pdf,
    };
    use intern_worker::pdf::PdfiumBackend;

    struct NoOcr;
    impl OcrBackend for NoOcr {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, ExtractionError> {
            panic!("an unopenable PDF has no pages to recognise")
        }
    }

    let Some(runtime) = std::env::var_os("INTERN_RUNTIME_DIR") else {
        return;
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated/encrypted.pdf");
    if !path.is_file() {
        assert!(
            std::env::var_os("INTERN_REQUIRE_GENERATED_FIXTURES").is_none(),
            "required generated fixture is missing: {}",
            path.display()
        );
        return;
    }
    let error = extract_pdf(
        &path,
        &PdfiumBackend::new(runtime).unwrap(),
        &NoOcr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "PASSWORD_PROTECTED", "{error}");
    assert!(!error.retryable());
    assert!(!error.to_string().contains('\n'), "{error}");
}
