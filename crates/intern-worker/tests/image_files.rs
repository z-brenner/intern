//! A TIFF is a chain of image file directories, one per frame, so a fax or a
//! batch scan arrives as several pages in one file. The fixtures here are
//! built from the TIFF 6.0 specification rather than copied from a scanner,
//! so the frame count is exactly what the test says it is.

use std::path::Path;
use std::sync::{Arc, Mutex};

use image::{GenericImageView, Rgb, RgbImage};
use intern_worker::extract::{
    CancellationToken, ExtractionError, ExtractionWarning, OcrBackend, OcrResult, PageSource,
    RenderedPage, extract_image,
};
use intern_worker::limits::{MAX_IMAGE_FILE_PIXELS, MAX_PAGE_PIXELS, ResourceLimits};

struct FakeOcr;

impl OcrBackend for FakeOcr {
    fn recognize(
        &self,
        _page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        Ok(OcrResult::new("Fax cover sheet", 88.0))
    }
}

/// The nine tags a minimal uncompressed 2x2 greyscale frame needs, in the
/// ascending tag order the specification requires.
fn directory(data_offset: u32, next_directory: u32) -> Vec<u8> {
    let mut bytes = 9_u16.to_le_bytes().to_vec();
    let mut entry = |tag: u16, kind: u16, value: u32| {
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&kind.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&value.to_le_bytes());
    };
    entry(0x0100, 3, 2); // ImageWidth
    entry(0x0101, 3, 2); // ImageLength
    entry(0x0102, 3, 8); // BitsPerSample
    entry(0x0103, 3, 1); // Compression: none
    entry(0x0106, 3, 1); // PhotometricInterpretation: black is zero
    entry(0x0111, 4, data_offset); // StripOffsets
    entry(0x0115, 3, 1); // SamplesPerPixel
    entry(0x0116, 3, 2); // RowsPerStrip
    entry(0x0117, 4, 4); // StripByteCounts
    bytes.extend_from_slice(&next_directory.to_le_bytes());
    bytes
}

/// A little-endian TIFF of `frames` frames, each its own directory followed
/// by its four pixels.
fn tiff(frames: u32) -> Vec<u8> {
    const DIRECTORY_BYTES: u32 = 2 + 9 * 12 + 4;
    const FRAME_BYTES: u32 = DIRECTORY_BYTES + 4;
    let mut bytes = b"II".to_vec();
    bytes.extend_from_slice(&42_u16.to_le_bytes());
    bytes.extend_from_slice(&8_u32.to_le_bytes());
    for frame in 0..frames {
        let start = 8 + frame * FRAME_BYTES;
        let next = if frame + 1 == frames {
            0
        } else {
            start + FRAME_BYTES
        };
        bytes.extend_from_slice(&directory(start + DIRECTORY_BYTES, next));
        bytes.extend_from_slice(&[0x20, 0x40, 0x60, 0x80]);
    }
    bytes
}

fn extract(frames: u32) -> intern_worker::extract::ExtractedDocument {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fax.tiff");
    std::fs::write(&path, tiff(frames)).unwrap();
    extract_image(
        &path,
        &FakeOcr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap()
}

#[test]
fn a_single_frame_tiff_is_one_complete_page() {
    let document = extract(1);

    assert_eq!(document.pages.len(), 1);
    assert_eq!(document.pages[0].text, "Fax cover sheet");
    assert!(!document.truncated);
    assert!(
        !document
            .warnings
            .contains(&ExtractionWarning::TextTruncated)
    );
}

/// Only the first frame is read - neither this decoder nor the CCITT
/// compression these files use supports the rest - so what matters is that
/// the pages that were not read are reported instead of vanishing.
#[test]
fn a_multi_page_tiff_reports_the_frames_it_did_not_read() {
    let document = extract(3);

    assert_eq!(document.pages.len(), 1);
    assert!(document.truncated);
    assert!(
        document
            .warnings
            .contains(&ExtractionWarning::TextTruncated)
    );
}

#[test]
fn a_png_is_never_reported_as_truncated() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("receipt.png");
    image::RgbImage::new(4, 4).save(&path).unwrap();

    let document = extract_image(
        &path,
        &FakeOcr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();

    assert!(!document.truncated);
    assert!(Path::new(&path).exists());
}

/// An OCR engine that keeps a copy of every page it was given.
#[derive(Clone, Default)]
struct KeepingOcr {
    pages: Arc<Mutex<Vec<image::DynamicImage>>>,
}

impl OcrBackend for KeepingOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        self.pages.lock().unwrap().push(page.image.clone());
        Ok(OcrResult::new("RECEIPT 0417 TOTAL 42.10", 91.0))
    }
}

/// A phone's 48-megapixel photo of a receipt used to be refused outright for
/// being over the 25-megapixel page cap, and the document was lost. It is
/// decoded and scaled down to the cap instead, keeping its proportions, and
/// read like any other page.
///
/// The limits are the real ones at a thousandth of the size - a 25,000-pixel
/// page cap and a 100,000-pixel file cap - so the test decodes and scales
/// 48,000 pixels rather than 48 million.
#[test]
fn oversized_photo_is_downscaled_before_ocr() {
    let limits = ResourceLimits {
        max_page_pixels: MAX_PAGE_PIXELS / 1_000,
        max_image_file_pixels: MAX_IMAGE_FILE_PIXELS / 1_000,
        ..ResourceLimits::default()
    };
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("receipt.jpg");
    // 240 x 200, black on the left and white on the right, so a crop would
    // show as a page of one colour.
    RgbImage::from_fn(240, 200, |x, _| {
        if x < 120 {
            Rgb([0, 0, 0])
        } else {
            Rgb([255, 255, 255])
        }
    })
    .save(&path)
    .unwrap();
    let ocr = KeepingOcr::default();

    let document = extract_image(&path, &ocr, &limits, &CancellationToken::new()).unwrap();

    let pages = ocr.pages.lock().unwrap();
    assert_eq!(pages.len(), 1);
    let (width, height) = pages[0].dimensions();
    assert!(
        u64::from(width) * u64::from(height) <= limits.max_page_pixels,
        "{width} x {height}"
    );
    // As large as fits, in the photo's proportions: 0.72 of each edge.
    assert_eq!((width, height), (173, 144));
    let left = pages[0].to_rgb8().get_pixel(10, height / 2).0[0];
    let right = pages[0].to_rgb8().get_pixel(width - 10, height / 2).0[0];
    assert!(left < 64 && right > 192, "left {left}, right {right}");
    assert_eq!(document.pages[0].source, PageSource::Ocr);
    assert_eq!(document.pages[0].text, "RECEIPT 0417 TOTAL 42.10");
    assert!(document.optional_image.is_some());
}

/// The decode cap still holds: a file over it is refused before any pixel
/// of it is decoded, and nothing reaches OCR.
#[test]
fn a_photo_over_the_decode_cap_is_still_refused() {
    let limits = ResourceLimits {
        max_page_pixels: MAX_PAGE_PIXELS / 1_000,
        max_image_file_pixels: MAX_IMAGE_FILE_PIXELS / 1_000,
        ..ResourceLimits::default()
    };
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("panorama.png");
    // 100,001 pixels.
    RgbImage::new(100_001, 1).save(&path).unwrap();
    let ocr = KeepingOcr::default();

    let error = extract_image(&path, &ocr, &limits, &CancellationToken::new()).unwrap_err();

    assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
    assert!(ocr.pages.lock().unwrap().is_empty());
}
