//! A TIFF is a chain of image file directories, one per frame, so a fax or a
//! batch scan arrives as several pages in one file. The fixtures here are
//! built from the TIFF 6.0 specification rather than copied from a scanner,
//! so the frame count is exactly what the test says it is.

use std::path::Path;
use std::sync::{Arc, Mutex};

use image::codecs::jpeg::JpegEncoder;
use image::{
    ExtendedColorType, GenericImageView, ImageBuffer, ImageEncoder, Rgb, RgbImage, RgbaImage,
};
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

/// Reads each page as the shade of its first pixel, so a test can tell
/// which frame became which page.
struct ShadeOcr;

impl OcrBackend for ShadeOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        let shade = page.image.to_luma8().get_pixel(0, 0)[0];
        Ok(OcrResult::new(format!("shade {shade}"), 90.0))
    }
}

/// One frame of a fixture TIFF: a 2x2 greyscale image of one shade.
#[derive(Clone, Copy)]
struct Frame {
    compression: u16,
    /// Marked a reduced-resolution copy of a page (`NewSubfileType` 1).
    thumbnail: bool,
    shade: u8,
}

impl Frame {
    fn page(shade: u8) -> Self {
        Self {
            compression: 1,
            thumbnail: false,
            shade,
        }
    }

    fn tags(self) -> u32 {
        if self.thumbnail { 10 } else { 9 }
    }
}

/// The tags a minimal 2x2 greyscale frame needs, in the ascending tag order
/// the specification requires.
fn directory(frame: Frame, data_offset: u32, next_directory: u32) -> Vec<u8> {
    let mut bytes = (frame.tags() as u16).to_le_bytes().to_vec();
    let mut entry = |tag: u16, kind: u16, value: u32| {
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&kind.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&value.to_le_bytes());
    };
    if frame.thumbnail {
        entry(0x00FE, 4, 1); // NewSubfileType: reduced resolution
    }
    entry(0x0100, 3, 2); // ImageWidth
    entry(0x0101, 3, 2); // ImageLength
    entry(0x0102, 3, 8); // BitsPerSample
    entry(0x0103, 3, u32::from(frame.compression)); // Compression
    entry(0x0106, 3, 1); // PhotometricInterpretation: black is zero
    entry(0x0111, 4, data_offset); // StripOffsets
    entry(0x0115, 3, 1); // SamplesPerPixel
    entry(0x0116, 3, 2); // RowsPerStrip
    entry(0x0117, 4, 4); // StripByteCounts
    bytes.extend_from_slice(&next_directory.to_le_bytes());
    bytes
}

/// A little-endian TIFF of these frames, each its own directory followed
/// by its four pixels.
fn tiff_of(frames: &[Frame]) -> Vec<u8> {
    let mut bytes = b"II".to_vec();
    bytes.extend_from_slice(&42_u16.to_le_bytes());
    bytes.extend_from_slice(&8_u32.to_le_bytes());
    let mut start = 8;
    for (index, frame) in frames.iter().enumerate() {
        let directory_bytes = 2 + frame.tags() * 12 + 4;
        let end = start + directory_bytes + 4;
        let next = if index + 1 == frames.len() { 0 } else { end };
        bytes.extend_from_slice(&directory(*frame, start + directory_bytes, next));
        bytes.extend_from_slice(&[frame.shade; 4]);
        start = end;
    }
    bytes
}

/// `frames` frames, the first of shade 32, each 32 darker than the last.
fn tiff(frames: u32) -> Vec<u8> {
    let frames = (1..=frames)
        .map(|frame| Frame::page(frame as u8 * 32))
        .collect::<Vec<_>>();
    tiff_of(&frames)
}

fn extract_with(
    bytes: &[u8],
    ocr: &dyn OcrBackend,
    limits: &ResourceLimits,
) -> intern_worker::extract::ExtractedDocument {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("fax.tiff");
    std::fs::write(&path, bytes).unwrap();
    extract_image(&path, ocr, limits, &CancellationToken::new()).unwrap()
}

fn extract(frames: u32) -> intern_worker::extract::ExtractedDocument {
    extract_with(&tiff(frames), &FakeOcr, &ResourceLimits::default())
}

fn texts(document: &intern_worker::extract::ExtractedDocument) -> Vec<&str> {
    document
        .pages
        .iter()
        .map(|page| page.text.as_str())
        .collect()
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

/// A fax or batch scan is one file with a page per frame, and every frame
/// is read, in order, as a page of its own.
#[test]
fn every_frame_of_a_multi_page_tiff_is_a_page() {
    let document = extract_with(&tiff(3), &ShadeOcr, &ResourceLimits::default());

    assert_eq!(texts(&document), ["shade 32", "shade 64", "shade 96"]);
    assert_eq!(
        document
            .pages
            .iter()
            .map(|page| page.page_number)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert!(
        document
            .pages
            .iter()
            .all(|page| page.source == PageSource::Ocr)
    );
    let layout = document.pages[2].layout.as_ref().unwrap();
    assert_eq!(layout.blocks[0].id, "p3.b1");
    assert!(!document.truncated);
    assert!(
        !document
            .warnings
            .contains(&ExtractionWarning::TextTruncated)
    );
    // Only the first page brings the page image with it.
    assert!(document.optional_image.is_some());
    assert!(document.pages[0].vision_escalated);
    assert!(!document.pages[1].vision_escalated);
}

/// Frames past the document's page limit are not read, and the document
/// says it is not whole.
#[test]
fn frames_past_the_page_limit_are_reported_unread() {
    let limits = ResourceLimits {
        max_page_count: 2,
        ..ResourceLimits::default()
    };
    let document = extract_with(&tiff(3), &ShadeOcr, &limits);

    assert_eq!(texts(&document), ["shade 32", "shade 64"]);
    assert!(document.truncated);
    assert!(
        document
            .warnings
            .contains(&ExtractionWarning::TextTruncated)
    );
}

/// A frame this decoder cannot read - CCITT Group 3 here - ends the reading
/// there: the pages before it are kept, and the document is not whole.
#[test]
fn a_frame_the_decoder_cannot_read_ends_the_reading_there() {
    let unreadable = Frame {
        compression: 3,
        ..Frame::page(64)
    };
    let document = extract_with(
        &tiff_of(&[Frame::page(32), unreadable, Frame::page(96)]),
        &ShadeOcr,
        &ResourceLimits::default(),
    );

    assert_eq!(texts(&document), ["shade 32"]);
    assert!(document.truncated);
}

/// A reduced-resolution copy of a page, as some scanners store a preview,
/// is not a page of its own.
#[test]
fn a_reduced_resolution_copy_is_not_a_page() {
    let preview = Frame {
        thumbnail: true,
        ..Frame::page(200)
    };
    let document = extract_with(
        &tiff_of(&[Frame::page(32), preview, Frame::page(96)]),
        &ShadeOcr,
        &ResourceLimits::default(),
    );

    assert_eq!(texts(&document), ["shade 32", "shade 96"]);
    assert!(!document.truncated);
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

/// An image goes to OCR as soon as it is decoded, and OCR is nearly all of
/// the time it takes. Announced as page 1 of 1 on the way in, it read as
/// 100% done for the whole of that time; the worker reports pages finished,
/// and while the only page is being read none is.
#[test]
fn an_image_reports_no_page_finished_while_it_is_read() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("receipt.png");
    image::RgbImage::new(4, 4).save(&path).unwrap();
    let reported = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&reported);
    let cancel = CancellationToken::reporting_to(Arc::new(move |stage, finished, total| {
        sink.lock().unwrap().push((stage, finished, total));
    }));

    extract_image(&path, &FakeOcr, &ResourceLimits::default(), &cancel).unwrap();

    assert_eq!(*reported.lock().unwrap(), vec![("ocr", 0, Some(1))]);
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

/// A minimal EXIF block: a little-endian TIFF header and one entry,
/// Orientation (0x0112) = 6, "turn a quarter clockwise to view".
const EXIF_ROTATE_90: [u8; 26] = [
    0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x12, 0x01, 0x03, 0x00, 0x01, 0x00,
    0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// A phone stores a portrait photo the way its sensor saw it and says in its
/// EXIF which way up it goes. The photo is turned only once it has been
/// scaled down, so the turn copies a page rather than the whole photo, and
/// it still reaches OCR upright, at the page cap, in upright proportions.
#[test]
fn a_rotated_oversized_photo_is_scaled_and_turned_upright() {
    let limits = ResourceLimits {
        max_page_pixels: MAX_PAGE_PIXELS / 1_000,
        max_image_file_pixels: MAX_IMAGE_FILE_PIXELS / 1_000,
        ..ResourceLimits::default()
    };
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("portrait.jpg");
    // 240 x 200 as stored, black on the left; a quarter turn clockwise puts
    // the black at the top.
    let stored = RgbImage::from_fn(240, 200, |x, _| {
        if x < 120 {
            Rgb([0, 0, 0])
        } else {
            Rgb([255, 255, 255])
        }
    });
    let mut jpeg = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg, 90);
    encoder.set_exif_metadata(EXIF_ROTATE_90.to_vec()).unwrap();
    encoder
        .write_image(stored.as_raw(), 240, 200, ExtendedColorType::Rgb8)
        .unwrap();
    std::fs::write(&path, jpeg).unwrap();
    let ocr = KeepingOcr::default();

    extract_image(&path, &ocr, &limits, &CancellationToken::new()).unwrap();

    let pages = ocr.pages.lock().unwrap();
    let (width, height) = pages[0].dimensions();
    assert_eq!((width, height), (144, 173));
    let page = pages[0].to_rgb8();
    let top = page.get_pixel(width / 2, 10).0[0];
    let bottom = page.get_pixel(width / 2, height - 10).0[0];
    assert!(top < 64 && bottom > 192, "top {top}, bottom {bottom}");
}

/// A scanner's 48-bit colour mode decodes to six bytes a pixel, so the pixel
/// cap - written for 8-bit images - let a hundred megapixels of it take
/// 600 MB before anything was scaled. It is held to the bytes the largest
/// 8-bit image may take, and an 8-bit image with alpha at nearly the pixel
/// cap is still read.
#[test]
fn a_deep_colour_image_is_held_to_the_bytes_an_8_bit_one_may_take() {
    let limits = ResourceLimits {
        max_page_pixels: MAX_PAGE_PIXELS / 1_000,
        max_image_file_pixels: MAX_IMAGE_FILE_PIXELS / 1_000,
        ..ResourceLimits::default()
    };
    let directory = tempfile::tempdir().unwrap();
    // 70,000 pixels, well inside the 100,000-pixel cap, at six bytes each.
    let deep = directory.path().join("scan.png");
    ImageBuffer::<Rgb<u16>, Vec<u16>>::new(280, 250)
        .save(&deep)
        .unwrap();
    // 99,856 pixels at four bytes each: just inside 400,000 bytes.
    let rgba = directory.path().join("logo.png");
    RgbaImage::new(316, 316).save(&rgba).unwrap();
    let ocr = KeepingOcr::default();

    let error = extract_image(&deep, &ocr, &limits, &CancellationToken::new()).unwrap_err();

    assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
    assert!(ocr.pages.lock().unwrap().is_empty());
    extract_image(&rgba, &ocr, &limits, &CancellationToken::new()).unwrap();
    assert_eq!(ocr.pages.lock().unwrap().len(), 1);
}
