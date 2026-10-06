//! Where a document's extraction time goes, as the readers record it on the
//! request's token.

use std::path::Path;
use std::time::Duration;

use image::{DynamicImage, GrayImage, Luma};
use intern_worker::ExtractionTimings;
use intern_worker::extract::{
    CancellationToken, ExtractionError, OcrBackend, OcrResult, PdfBackend, PdfPageInspection,
    RenderedPage, extract_image, extract_pdf,
};
use intern_worker::limits::ResourceLimits;

/// A three-page PDF: a scan, a page of text, and another scan.
struct MixedPdf;

const SCAN_WIDTH: u32 = 30;
const SCAN_HEIGHT: u32 = 20;

impl PdfBackend for MixedPdf {
    fn inspect(
        &self,
        _path: &Path,
        _cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        let scan = |page_index| PdfPageInspection {
            page_index,
            native_text: String::new(),
            image_coverage: 1.0,
            width_pixels: SCAN_WIDTH,
            height_pixels: SCAN_HEIGHT,
        };
        Ok(vec![
            scan(0),
            PdfPageInspection {
                page_index: 1,
                native_text: "This Lease is made between Harbourline Storage Co. and \
                              Fennimore Textiles LLC as of March 4, 2026."
                    .to_owned(),
                image_coverage: 0.0,
                width_pixels: SCAN_WIDTH,
                height_pixels: SCAN_HEIGHT,
            },
            scan(2),
        ])
    }

    fn render_within(
        &self,
        _path: &Path,
        page_index: usize,
        _max_pixels: u64,
        _cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        std::thread::sleep(Duration::from_millis(2));
        Ok(RenderedPage::new(
            page_index,
            DynamicImage::ImageLuma8(GrayImage::from_pixel(SCAN_WIDTH, SCAN_HEIGHT, Luma([255]))),
        ))
    }
}

/// OCR that takes 3 ms a page and is unconvincing about the first page it
/// reads, so that page also becomes the page image.
struct SlowOcr;

impl OcrBackend for SlowOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        std::thread::sleep(Duration::from_millis(3));
        let confidence = if page.page_index == 0 { 40.0 } else { 92.0 };
        Ok(OcrResult::new("SCANNED PAGE OF THE LEASE", confidence))
    }
}

#[test]
fn a_mixed_pdf_reports_each_scanned_page_rendered_and_read() {
    let cancel = CancellationToken::new();

    let document = extract_pdf(
        Path::new("mixed.pdf"),
        &MixedPdf,
        &SlowOcr,
        &ResourceLimits::default(),
        &cancel,
    )
    .unwrap();

    // Readers leave the document's own field to the protocol.
    assert_eq!(document.timings, None);
    let timings = cancel.timings();
    assert_eq!(timings.ocr_pages, 2, "{timings:?}");
    assert_eq!(
        timings.rendered_pixels,
        2 * u64::from(SCAN_WIDTH * SCAN_HEIGHT),
        "{timings:?}"
    );
    assert!(timings.render_micros >= 4_000, "{timings:?}");
    assert!(timings.ocr_micros >= 6_000, "{timings:?}");
    // The unconvincing first page became the page image.
    assert!(document.optional_image.is_some());
    assert!(timings.vision_micros > 0, "{timings:?}");
    // Nothing here decoded an image file, and the stand-in PDF backend
    // reports no parse of its own.
    assert_eq!(timings.image_decode_micros, 0, "{timings:?}");
    assert_eq!(timings.parse_micros, 0, "{timings:?}");
    // Passes are the OCR backend's to count, and this one counts none.
    assert_eq!(timings.ocr_passes, 0, "{timings:?}");
}

#[test]
fn an_image_file_reports_its_decode_ocr_and_page_image() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("receipt.png");
    GrayImage::from_fn(400, 300, |x, y| Luma([((x + y) % 256) as u8]))
        .save(&path)
        .unwrap();
    let cancel = CancellationToken::new();

    extract_image(&path, &SlowOcr, &ResourceLimits::default(), &cancel).unwrap();

    let timings = cancel.timings();
    assert_eq!(timings.ocr_pages, 1, "{timings:?}");
    assert!(timings.image_decode_micros > 0, "{timings:?}");
    assert!(timings.ocr_micros >= 3_000, "{timings:?}");
    assert!(timings.vision_micros > 0, "{timings:?}");
    // An image file is never rendered.
    assert_eq!(timings.render_micros, 0, "{timings:?}");
    assert_eq!(timings.rendered_pixels, 0, "{timings:?}");
}

/// A stage is charged for its time whether or not it succeeded, and clones
/// of a token share one account, as every reader of a request does.
#[test]
fn timed_work_is_charged_even_when_it_fails_and_clones_share_the_account() {
    let cancel = CancellationToken::new();
    let reader = cancel.clone();

    let failed: Result<(), ExtractionError> = reader.timed(
        |timings| &mut timings.render_micros,
        || {
            std::thread::sleep(Duration::from_millis(2));
            Err(ExtractionError::parse_failed("render failed"))
        },
    );
    reader.record(|timings| timings.ocr_passes += 2);
    reader.record(|timings| timings.ocr_passes += 1);

    assert!(failed.is_err());
    let timings = cancel.timings();
    assert!(timings.render_micros >= 2_000, "{timings:?}");
    assert_eq!(timings.ocr_passes, 3);
    assert_eq!(
        CancellationToken::new().timings(),
        ExtractionTimings::default()
    );
}
