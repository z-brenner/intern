use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

use base64::Engine as _;
use image::{
    DynamicImage, GenericImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader, Rgb,
    RgbImage, imageops::FilterType,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::limits::{
    MAX_EXTRACTION_DURATION, MAX_PAGE_CHARS, MAX_VISION_LONG_EDGE, MIN_OCR_DPI, RENDER_DPI,
    ResourceLimits, VISION_GRID,
};
use crate::temp::TempWorkspace;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExtractionErrorKind {
    Canceled,
    ResourceLimit,
    Unsupported,
    NativeAssetsMissing,
    ParseFailed,
    Encrypted,
    Io,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct ExtractionError {
    kind: ExtractionErrorKind,
    message: String,
}

impl ExtractionError {
    pub fn canceled() -> Self {
        Self {
            kind: ExtractionErrorKind::Canceled,
            message: "request canceled".to_owned(),
        }
    }

    pub fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            kind: ExtractionErrorKind::ResourceLimit,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            kind: ExtractionErrorKind::Unsupported,
            message: message.into(),
        }
    }

    pub fn native_assets_missing(message: impl Into<String>) -> Self {
        Self {
            kind: ExtractionErrorKind::NativeAssetsMissing,
            message: message.into(),
        }
    }

    pub fn parse_failed(message: impl Into<String>) -> Self {
        Self {
            kind: ExtractionErrorKind::ParseFailed,
            message: message.into(),
        }
    }

    /// A document that cannot be read without its password. Retrying cannot
    /// help, and saying so plainly is what lets the person fix it: remove the
    /// password and add the file again.
    pub fn encrypted() -> Self {
        Self {
            kind: ExtractionErrorKind::Encrypted,
            message: "document is password-protected".to_owned(),
        }
    }

    pub fn io(error: std::io::Error) -> Self {
        Self {
            kind: ExtractionErrorKind::Io,
            message: error.to_string(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self.kind {
            ExtractionErrorKind::Canceled => "CANCELED",
            ExtractionErrorKind::ResourceLimit => "RESOURCE_LIMIT_EXCEEDED",
            ExtractionErrorKind::Unsupported => "UNSUPPORTED_FORMAT",
            ExtractionErrorKind::NativeAssetsMissing => "NATIVE_ASSETS_MISSING",
            ExtractionErrorKind::Encrypted => "PASSWORD_PROTECTED",
            ExtractionErrorKind::ParseFailed | ExtractionErrorKind::Io => "PARSE_FAILED",
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(self.kind, ExtractionErrorKind::Io)
    }
}

/// Where a reader reports how far through a document it is: a stage name,
/// how many pages it has finished, and how many there are, if it knows.
///
/// Finished, not reached: the window shows `current / total` as a
/// percentage, and a one-page scan that announced page 1 of 1 as it went to
/// OCR read as done for the whole of the OCR it was about to spend.
pub type ProgressSink = Arc<dyn Fn(&'static str, usize, Option<usize>) + Send + Sync>;

struct CancellationState {
    canceled: AtomicBool,
    deadline: Instant,
    progress: Option<ProgressSink>,
}

impl std::fmt::Debug for CancellationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CancellationState")
            .field("canceled", &self.canceled)
            .field("deadline", &self.deadline)
            .field("progress", &self.progress.is_some())
            .finish()
    }
}

/// What a reader carries for one request: whether to stop, and where to say
/// how far it has got.
///
/// Progress rides on the token because the token already reaches every
/// reader and every page loop, and because neither belongs in the readers'
/// signatures: a reader that has nothing to report never sees a sink, and
/// one that does report cannot tell a sink from none.
#[derive(Clone, Debug)]
pub struct CancellationToken(Arc<CancellationState>);

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::build(None)
    }

    /// A token whose readers' progress goes to `sink`.
    pub fn reporting_to(sink: ProgressSink) -> Self {
        Self::build(Some(sink))
    }

    fn build(progress: Option<ProgressSink>) -> Self {
        Self(Arc::new(CancellationState {
            canceled: AtomicBool::new(false),
            deadline: Instant::now() + MAX_EXTRACTION_DURATION,
            progress,
        }))
    }

    /// Says how far through the document a reader is. Cheap enough to call
    /// once per page: deciding whether it is worth sending is the sink's.
    pub fn report_progress(&self, stage: &'static str, current: usize, total: Option<usize>) {
        if let Some(sink) = &self.0.progress {
            sink(stage, current, total);
        }
    }

    pub fn cancel(&self) {
        self.0.canceled.store(true, Ordering::SeqCst);
    }
    pub fn is_canceled(&self) -> bool {
        self.0.canceled.load(Ordering::SeqCst)
    }
    pub fn check(&self) -> Result<(), ExtractionError> {
        if self.is_canceled() {
            Err(ExtractionError::canceled())
        } else if Instant::now() > self.0.deadline {
            Err(ExtractionError::resource_limit(
                "extraction exceeded 30 minutes",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PdfPageInspection {
    pub page_index: usize,
    pub native_text: String,
    pub image_coverage: f32,
    pub width_pixels: u32,
    pub height_pixels: u32,
}

#[derive(Clone, Debug)]
pub struct RenderedPage {
    pub page_index: usize,
    pub image: DynamicImage,
}

impl RenderedPage {
    pub fn new(page_index: usize, image: DynamicImage) -> Self {
        Self { page_index, image }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OcrResult {
    pub text: String,
    pub mean_confidence: f32,
    /// Clockwise non-EXIF rotation applied before OCR.
    pub rotation_degrees: u16,
}

impl OcrResult {
    pub fn new(text: impl Into<String>, mean_confidence: f32) -> Self {
        Self {
            text: text.into(),
            mean_confidence,
            rotation_degrees: 0,
        }
    }

    pub fn with_rotation(mut self, rotation_degrees: u16) -> Self {
        self.rotation_degrees = rotation_degrees;
        self
    }
}

/// Mean word confidence at or above which a page is considered read. Below it a
/// page earns a `LowOcrConfidence` warning, may escalate to vision, and is worth
/// re-reading in another orientation before any of that.
pub const CONFIDENT_READING: f32 = 75.0;

/// Of two readings of the same page, the one to keep. A tie keeps the
/// incumbent, so the orientation OSD chose wins by default and behaviour on a
/// blank page stays predictable.
///
/// Orientation detection is trained on prose with ascenders and descenders.
/// On a dense all-caps form it can report a rotation that is 180 degrees
/// wrong, and OCR then returns a full page of gibberish rather than obviously
/// empty output - same word count, plausible shape, useless text. Volume
/// cannot tell those apart; word confidence can. Measured on one corpus page,
/// the four orientations scored 23, 14, 14, and 76.
///
/// Confidence alone cannot arbitrate either, because it is a mean over
/// whatever was read: three tokens picked out of a rotated page at 80 outrank
/// three hundred words of the real document at 74.9, and the page then comes
/// back as three tokens. A reading has to be about as dense as the one it
/// displaces before its confidence counts.
pub fn better_reading(incumbent: OcrResult, challenger: OcrResult) -> OcrResult {
    let words = |reading: &OcrResult| reading.text.split_whitespace().count();
    let comparably_dense = words(&challenger) * 2 >= words(&incumbent);
    if challenger.mean_confidence > incumbent.mean_confidence && comparably_dense {
        challenger
    } else {
        incumbent
    }
}

/// Words an upright reading has to find, besides being confident, to be
/// accepted without asking which way up the page is.
pub const UPRIGHT_WORDS: usize = 3;

/// Whether a page read as it came is done, with no orientation detection at
/// all.
///
/// Nearly every page is upright, and reading it upright first means the
/// common case costs one recognition pass. Orientation detection used to run
/// first on every page - and on dense all-caps pages it was confidently
/// wrong, reporting 180 degrees on the corpus's upright lease and buying
/// four recognition passes to find the orientation the page already had.
/// A sideways or inverted page read as-is comes back as low-confidence
/// gibberish, so confidence is the test; the word floor stops two
/// confident specks on an otherwise unread page from passing it.
pub fn upright_reading_is_accepted(reading: &OcrResult) -> bool {
    reading.mean_confidence >= CONFIDENT_READING
        && reading.text.split_whitespace().count() >= UPRIGHT_WORDS
}

/// The passes reading one page can cost, as [`read_upright`] sees them.
///
/// The Tesseract adapter runs each one as a process. The decisions between
/// them need no Tesseract at all, which is what lets them be held to account
/// on the platform this ships on.
pub trait OrientationPasses {
    /// Reads the page turned clockwise by `rotation_degrees`. Asked at most
    /// once for any one rotation.
    fn recognize(&mut self, rotation_degrees: u16) -> Result<OcrResult, ExtractionError>;

    /// The clockwise rotation orientation detection says the page needs.
    fn detect_orientation(&mut self) -> Result<u16, ExtractionError>;
}

/// Reads a page the right way up, in as few passes as the page allows.
///
/// The page is read as it came, first. A confident reading of it is done -
/// that is nearly every page, at one recognition and no detection - and so
/// is a reading that found nothing: a blank page is blank in every
/// orientation, and asking which way up it is buys nothing. Only an
/// unconvincing reading asks orientation detection, and then the search
/// over the other orientations runs as it always has. The upright reading
/// is one of its candidates and is never read twice: it is compared as it
/// already came back.
pub fn read_upright(passes: &mut dyn OrientationPasses) -> Result<OcrResult, ExtractionError> {
    let upright = passes.recognize(0)?;
    if upright_reading_is_accepted(&upright) || upright.text.trim().is_empty() {
        return Ok(upright);
    }
    let rotation = passes.detect_orientation()?;
    let mut best = if rotation == 0 {
        upright.clone()
    } else {
        passes.recognize(rotation)?
    };
    // A page that reads confidently in the orientation detection asked for
    // is done, and so is one that read as blank.
    for candidate in [270, 90, 180, 0] {
        if !orientation_search_is_worthwhile(&best) {
            break;
        }
        if candidate == rotation {
            continue;
        }
        let attempt = if candidate == 0 {
            upright.clone()
        } else {
            passes.recognize(candidate)?
        };
        best = better_reading(best, attempt);
    }
    Ok(best)
}

/// Whether reading this page again in another orientation could tell us
/// anything.
///
/// A confident reading is done. So is a reading that found no words at all:
/// that page is blank, and a blank page is blank in four orientations. It
/// scores zero confidence, though, which read as "not confident, keep
/// looking" and bought three more recognition passes and three more
/// full-page PNG encodes - on the back of every sheet of a three-hundred-page
/// double-sided scan.
pub fn orientation_search_is_worthwhile(reading: &OcrResult) -> bool {
    reading.mean_confidence < CONFIDENT_READING && !reading.text.trim().is_empty()
}

pub fn apply_detected_rotation(
    image: DynamicImage,
    rotation_degrees: u16,
) -> Result<DynamicImage, ExtractionError> {
    match rotation_degrees % 360 {
        0 => Ok(image),
        90 => Ok(image.rotate90()),
        180 => Ok(image.rotate180()),
        270 => Ok(image.rotate270()),
        degrees => Err(ExtractionError::parse_failed(format!(
            "unsupported OSD rotation {degrees}"
        ))),
    }
}

pub trait PdfBackend {
    fn inspect(
        &self,
        path: &Path,
        cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError>;

    /// Renders a page at 300 DPI, or, if that would be more than
    /// `max_pixels`, at the highest resolution that is not.
    fn render_within(
        &self,
        path: &Path,
        page_index: usize,
        max_pixels: u64,
        cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError>;

    /// Renders a page at 300 DPI.
    fn render(
        &self,
        path: &Path,
        page_index: usize,
        cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        self.render_within(path, page_index, u64::MAX, cancel)
    }
}

pub trait OcrBackend {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError>;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageSource {
    Native,
    Ocr,
    AnyDoc,
    Text,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExtractedPage {
    pub page_number: usize,
    pub text: String,
    pub source: PageSource,
    pub ocr_confidence: Option<f32>,
    pub vision_escalated: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExtractionWarning {
    LowOcrConfidence,
    NativeTextCorrupt,
    /// Text that was lost: a page cut at the size cap, frames of a TIFF that
    /// were never read. What was dropped is unknown, so it may be the fact
    /// that names the document.
    TextTruncated,
    /// Content deliberately left out by design and marked where it was left
    /// out - the rows and columns past a spreadsheet's rendered window. The
    /// reader chose what to show and says what it skipped, so this is a note
    /// about the document's size, not a doubt about what was read.
    ContentElided,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VisionImage {
    pub page_number: usize,
    pub mime_type: String,
    pub data_base64: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExtractedDocument {
    pub pages: Vec<ExtractedPage>,
    pub warnings: Vec<ExtractionWarning>,
    pub truncated: bool,
    pub optional_image: Option<VisionImage>,
}

fn timed_check(
    cancel: &CancellationToken,
    started: Instant,
    limits: &ResourceLimits,
) -> Result<(), ExtractionError> {
    cancel.check()?;
    if started.elapsed() > limits.max_duration {
        return Err(ExtractionError::resource_limit(
            "extraction exceeded 30 minutes",
        ));
    }
    Ok(())
}

pub fn page_needs_ocr(page: &PdfPageInspection) -> bool {
    let meaningful = page
        .native_text
        .chars()
        .filter(|character| {
            !character.is_whitespace() && !character.is_control() && *character != '\u{fffd}'
        })
        .count();
    let considered = page
        .native_text
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    let replacements = page
        .native_text
        .chars()
        .filter(|character| *character == '\u{fffd}')
        .count();
    let replacement_ratio = if considered == 0 {
        0.0
    } else {
        replacements as f32 / considered as f32
    };
    // A page that is essentially all image is a scan, and a scan's text layer
    // is whatever the scanner or the review platform stamped on it: a Bates
    // number, a confidentiality legend, an exhibit label. Those clear the
    // twenty-character veto while carrying none of the document, so a
    // stamped scan has to reach OCR on the strength of its coverage.
    let stamped_scan = meaningful < STAMP_CHARACTERS && page.image_coverage >= FULL_PAGE_IMAGE;
    (meaningful < 20 && page.image_coverage >= 0.65) || stamped_scan || replacement_ratio > 0.03
}

/// Native text this short on a page that is all image is a stamp, not the
/// document.
const STAMP_CHARACTERS: usize = 200;

/// Image coverage at or above which a page is a picture of a page rather
/// than a page with a picture on it. A scanner covers the sheet; a chart or a
/// letterhead on a page of prose does not come close.
const FULL_PAGE_IMAGE: f32 = 0.9;

fn page_needs_vision(page: &PdfPageInspection) -> bool {
    let meaningful = page
        .native_text
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    // Word-structured native text is trustworthy and stays text-only; a large
    // non-text region only routes to vision when extraction leaves less than
    // 100 meaningful characters without word structure.
    let word_structured = page.native_text.split_whitespace().count() > 1;
    meaningful < 100 && page.image_coverage >= 0.65 && !word_structured
}

pub fn extract_pdf(
    path: &Path,
    pdf: &dyn PdfBackend,
    ocr: &dyn OcrBackend,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    let started = Instant::now();
    timed_check(cancel, started, limits)?;
    let inspections = pdf.inspect(path, cancel)?;
    limits.validate_page_count(inspections.len())?;
    let page_count = inspections.len();
    let mut pages = Vec::with_capacity(page_count);
    let mut warnings = Vec::new();
    let mut vision_candidate: Option<VisionImage> = None;

    for inspection in inspections {
        timed_check(cancel, started, limits)?;
        let page_number = inspection.page_index + 1;
        cancel.report_progress("reading", inspection.page_index, Some(page_count));
        // The render cap belongs to rendering. A large-format sheet - an A1
        // drawing, a plan set - is over it at 300 DPI while carrying a
        // perfectly good text layer, and failing the whole document over a
        // page nobody was going to rasterise loses the document. A page that
        // is too large to render also cannot be escalated to vision, but it
        // keeps its text: the page image is the optional part.
        let renderable = limits
            .validate_page_pixels(inspection.width_pixels, inspection.height_pixels)
            .is_ok();
        if !page_needs_ocr(&inspection) {
            let vision_escalated =
                renderable && page_needs_vision(&inspection) && vision_candidate.is_none();
            if vision_escalated {
                let rendered = pdf.render(path, inspection.page_index, cancel)?;
                let (render_width, render_height) = rendered.image.dimensions();
                limits.validate_page_pixels(render_width, render_height)?;
                vision_candidate = Some(normalize_vision_image(
                    inspection.page_index,
                    rendered.image,
                )?);
            }
            pages.push(ExtractedPage {
                page_number,
                text: inspection.native_text,
                source: PageSource::Native,
                ocr_confidence: None,
                vision_escalated,
            });
            continue;
        }

        if inspection.native_text.contains('\u{fffd}')
            && !warnings.contains(&ExtractionWarning::NativeTextCorrupt)
        {
            warnings.push(ExtractionWarning::NativeTextCorrupt);
        }
        // This page has no text worth keeping, so it has to be rendered to be
        // read at all. A page too large to render at 300 DPI within the cap -
        // a phone photo some tool turned into a PDF at 72 DPI, an A2 scan -
        // is rendered at the resolution that fits, which Tesseract reads
        // perfectly well; only a page that would need less than the floor
        // to fit is a resource limit, and that is decided before anything
        // is rendered.
        if ocr_render_dpi(&inspection, limits.max_page_pixels) < MIN_OCR_DPI {
            return Err(ExtractionError::resource_limit(
                "page is too large to read within 25 megapixels at 50 DPI",
            ));
        }
        let rendered =
            pdf.render_within(path, inspection.page_index, limits.max_page_pixels, cancel)?;
        // The backend sized the render; this is what holds it to that.
        let (render_width, render_height) = rendered.image.dimensions();
        limits.validate_page_pixels(render_width, render_height)?;
        timed_check(cancel, started, limits)?;
        cancel.report_progress("ocr", inspection.page_index, Some(page_count));
        let result = ocr.recognize(&rendered, cancel)?;
        let vision_escalated =
            vision_candidate.is_none() && result.mean_confidence < CONFIDENT_READING;
        if result.mean_confidence < CONFIDENT_READING
            && !warnings.contains(&ExtractionWarning::LowOcrConfidence)
        {
            warnings.push(ExtractionWarning::LowOcrConfidence);
        }
        if vision_escalated {
            vision_candidate = Some(normalize_vision_image(
                inspection.page_index,
                apply_detected_rotation(rendered.image, result.rotation_degrees)?,
            )?);
        }
        pages.push(ExtractedPage {
            page_number,
            text: result.text,
            source: PageSource::Ocr,
            ocr_confidence: Some(result.mean_confidence),
            vision_escalated,
        });
    }

    Ok(ExtractedDocument {
        pages,
        warnings,
        truncated: false,
        optional_image: vision_candidate,
    })
}

/// The resolution a page that has to be OCR'd is rendered at: 300 DPI, or as
/// much less as brings its render within `max_pixels`. The inspection
/// measured the page at 300 DPI, so the budget scales that.
fn ocr_render_dpi(inspection: &PdfPageInspection, max_pixels: u64) -> f64 {
    let pixels = u64::from(inspection.width_pixels) * u64::from(inspection.height_pixels);
    if pixels <= max_pixels {
        RENDER_DPI
    } else {
        RENDER_DPI * (max_pixels as f64 / pixels as f64).sqrt()
    }
}

pub fn normalize_vision_image(
    page_index: usize,
    image: DynamicImage,
) -> Result<VisionImage, ExtractionError> {
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err(ExtractionError::parse_failed("image has zero dimensions"));
    }
    let scale = (MAX_VISION_LONG_EDGE as f64 / f64::from(width.max(height))).min(1.0);
    let resized_width = (f64::from(width) * scale).round().max(1.0) as u32;
    let resized_height = (f64::from(height) * scale).round().max(1.0) as u32;
    let rgb = image
        .resize_exact(resized_width, resized_height, FilterType::Lanczos3)
        .into_rgb8();
    let padded_width = resized_width.div_ceil(VISION_GRID) * VISION_GRID;
    let padded_height = resized_height.div_ceil(VISION_GRID) * VISION_GRID;
    let mut padded = RgbImage::from_pixel(padded_width, padded_height, Rgb([255, 255, 255]));
    padded
        .copy_from(&rgb, 0, 0)
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(padded)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    Ok(VisionImage {
        page_number: page_index + 1,
        mime_type: "image/png".to_owned(),
        data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}

pub fn extract_anydoc(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    reject_encrypted_ole(path)?;
    cancel.check()?;
    let metadata = std::fs::metadata(path).map_err(ExtractionError::io)?;
    limits.validate_source_size(metadata.len())?;
    let bytes = std::fs::read(path).map_err(ExtractionError::io)?;
    let format = expected_anydoc_format(path, &bytes)?;
    // The pre-pass follows what the file is rather than what it is called,
    // so a `.doc` that is really a Word 2007 package is inflated under the
    // same bound a `.docx` is, and a `.docx` that is really a binary Word
    // file is not handed to a zip reader at all.
    if format == anydoc::Format::Docx {
        enforce_office_decompressed_limit(path, limits, cancel)?;
    }
    cancel.check()?;
    let markdown = anydoc::to_markdown_bytes(&bytes, format).map_err(|error| match error {
        anydoc::ConvertError::Encrypted => ExtractionError::encrypted(),
        other => ExtractionError::parse_failed(other.to_string()),
    })?;
    cancel.check()?;
    Ok(ExtractedDocument {
        pages: vec![ExtractedPage {
            page_number: 1,
            text: markdown,
            source: PageSource::AnyDoc,
            ocr_confidence: None,
            vision_escalated: false,
        }],
        warnings: vec![],
        truncated: false,
        optional_image: None,
    })
}

/// The parser an extension names, refusing content of another kind.
///
/// Left to itself anydoc picks its parser from the file's content and treats
/// the extension as a fallback, so a workbook renamed `.docx` is rendered by
/// its Excel path with none of the row and column caps spreadsheets are
/// routed through here, and a PDF renamed `.pptx` reaches a PDF reader with
/// no page cap, no OCR, and no page image. Routing in this crate is by
/// extension, so content of a different kind than the extension names is a
/// routing failure that belongs in review, not a document to parse anyway.
///
/// Content of the same kind is a different matter. Word has saved RTF under
/// `.doc` for decades, and a `.doc` that is really a Word 2007 package (or a
/// `.pptx` that is really a 97-2003 deck) is a mislabelled file of exactly
/// the kind the extension promised. None of those reaches a reader without
/// the caps it would otherwise have had, so they are read as what they are.
fn expected_anydoc_format(path: &Path, bytes: &[u8]) -> Result<anydoc::Format, ExtractionError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let named = anydoc::Format::from_extension(&extension).ok_or_else(|| {
        ExtractionError::unsupported(format!("no Office reader handles a .{extension} file"))
    })?;
    match anydoc::Format::from_bytes(bytes) {
        // Content that identifies as nothing at all - a container this
        // version cannot recognise - is still handed to the parser the
        // extension names, which reports what is actually wrong with it far
        // better than a routing refusal would.
        None => Ok(named),
        Some(detected) if detected == named => Ok(named),
        Some(detected)
            if office_family(detected)
                .is_some_and(|family| Some(family) == office_family(named)) =>
        {
            Ok(detected)
        }
        Some(_) => Err(ExtractionError::unsupported(format!(
            "file content is not what its .{extension} extension names"
        ))),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OfficeFamily {
    WordProcessing,
    Presentation,
}

/// The kinds of document whose formats are interchangeable for routing:
/// every member is read by anydoc, uncapped, as one page of prose. Workbooks
/// and PDFs belong to no family, because each has its own capped reader.
fn office_family(format: anydoc::Format) -> Option<OfficeFamily> {
    use anydoc::Format;
    match format {
        Format::Doc | Format::Docx | Format::Rtf | Format::Odt => {
            Some(OfficeFamily::WordProcessing)
        }
        Format::Ppt | Format::Pptx | Format::Odp => Some(OfficeFamily::Presentation),
        _ => None,
    }
}

/// The signature every OLE compound file - binary Office, Outlook `.msg`, an
/// encrypted Office package - starts with.
pub(crate) const OLE_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Refuses an Office package that was saved with a password to open.
///
/// Office encrypts a `.docx` or `.xlsx` by wrapping it in an OLE compound
/// file that holds an `EncryptionInfo` stream and the encrypted zip as
/// `EncryptedPackage`. Nothing downstream can read that: the zip pre-pass
/// rejects it as a corrupt archive and anydoc's detection identifies it as
/// nothing, so without this check the person is told the file is damaged
/// when it only needs its password removed.
pub(crate) fn reject_encrypted_ole(path: &Path) -> Result<(), ExtractionError> {
    let mut file = File::open(path).map_err(ExtractionError::io)?;
    let mut magic = [0_u8; 8];
    match file.read_exact(&mut magic) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
        Err(error) => return Err(ExtractionError::io(error)),
    }
    if magic != OLE_MAGIC {
        return Ok(());
    }
    file.seek(SeekFrom::Start(0)).map_err(ExtractionError::io)?;
    // A compound file too damaged to open is not this check's to report: the
    // reader the extension names says what is wrong with it.
    let Ok(compound) = cfb::CompoundFile::open(file) else {
        return Ok(());
    };
    if compound.exists("EncryptionInfo") || compound.exists("EncryptedPackage") {
        return Err(ExtractionError::encrypted());
    }
    Ok(())
}

/// Inflates every entry of a zip-packaged Office file once, counting, and
/// refuses the file before any parser does if the total passes the bound.
pub(crate) fn enforce_office_decompressed_limit(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<(), ExtractionError> {
    let file = File::open(path).map_err(ExtractionError::io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    for index in 0..archive.len() {
        cancel.check()?;
        let mut entry = archive
            .by_index(index)
            .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
        loop {
            cancel.check()?;
            let read = entry.read(&mut buffer).map_err(ExtractionError::io)?;
            if read == 0 {
                break;
            }
            total = total.checked_add(read as u64).ok_or_else(|| {
                ExtractionError::resource_limit("Office decompression size overflow")
            })?;
            if total > limits.max_decompressed_office_bytes {
                return Err(ExtractionError::resource_limit(
                    "decompressed Office content exceeds 1 GiB",
                ));
            }
        }
    }
    Ok(())
}

/// The most of a text file that is ever read.
///
/// One page carries at most [`MAX_PAGE_CHARS`] characters and a character is
/// at most four bytes, so this many bytes always fill a page; reading a
/// gigabyte log file whole to keep its first two million characters only
/// costs memory.
pub const MAX_TEXT_FILE_BYTES: u64 = 4 * MAX_PAGE_CHARS as u64;

pub fn extract_text(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    cancel.check()?;
    let metadata = std::fs::metadata(path).map_err(ExtractionError::io)?;
    limits.validate_source_size(metadata.len())?;
    let file = File::open(path).map_err(ExtractionError::io)?;
    let mut reader = BufReader::new(file).take(MAX_TEXT_FILE_BYTES + 1);
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        cancel.check()?;
        let read = reader.read(&mut buffer).map_err(ExtractionError::io)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    let truncated = bytes.len() as u64 > MAX_TEXT_FILE_BYTES;
    bytes.truncate(MAX_TEXT_FILE_BYTES as usize);
    let (text, suspect) = decode_text(&bytes, truncated);
    cancel.check()?;
    let mut warnings = Vec::new();
    if suspect {
        warnings.push(ExtractionWarning::NativeTextCorrupt);
    }
    if truncated {
        warnings.push(ExtractionWarning::TextTruncated);
    }
    Ok(ExtractedDocument {
        pages: vec![ExtractedPage {
            page_number: 1,
            text,
            source: PageSource::Text,
            ocr_confidence: None,
            vision_escalated: false,
        }],
        warnings,
        truncated,
        optional_image: None,
    })
}

/// Decodes a text file by its byte-order mark, and says whether the result
/// is suspect.
///
/// Notepad and PowerShell's redirection still write UTF-16, and a mark on a
/// UTF-8 file is ordinary; neither is a document to refuse, and the mark
/// itself is not a character of the document. Unmarked text that is not
/// UTF-8 is, on the Windows machines these files come from, almost always
/// Windows-1252 - an accounting export, a note saved by an old editor - and
/// reads correctly as that. `cut` says the bytes stop where reading stopped
/// rather than where the file did, so a character split there is dropped
/// instead of condemning the whole file.
fn decode_text(bytes: &[u8], cut: bool) -> (String, bool) {
    fn from_utf16(mut units: Vec<u16>, cut: bool) -> (String, bool) {
        if cut
            && units
                .last()
                .is_some_and(|unit| (0xD800..0xDC00).contains(unit))
        {
            units.pop();
        }
        match String::from_utf16(&units) {
            Ok(text) => (text, false),
            Err(_) => (String::from_utf16_lossy(&units), true),
        }
    }
    match bytes {
        [0xFF, 0xFE, rest @ ..] => from_utf16(
            rest.chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect(),
            cut,
        ),
        [0xFE, 0xFF, rest @ ..] => from_utf16(
            rest.chunks_exact(2)
                .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                .collect(),
            cut,
        ),
        [0xEF, 0xBB, 0xBF, rest @ ..] => from_utf8(rest, cut),
        _ => from_utf8(bytes, cut),
    }
}

fn from_utf8(bytes: &[u8], cut: bool) -> (String, bool) {
    let bytes = match std::str::from_utf8(bytes) {
        Ok(text) => return (text.to_owned(), false),
        // Only an incomplete character at the very end, where reading
        // stopped: everything before it is good UTF-8.
        Err(error) if cut && error.error_len().is_none() => &bytes[..error.valid_up_to()],
        Err(_) => bytes,
    };
    if let Ok(text) = std::str::from_utf8(bytes) {
        return (text.to_owned(), false);
    }
    if holds_utf8_text(bytes) {
        // UTF-8 with a damaged byte or two. Reading it as Windows-1252
        // would turn every accented letter in it into two wrong ones.
        return (String::from_utf8_lossy(bytes).into_owned(), true);
    }
    let (text, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
    // Windows-1252 leaves five byte values undefined, and the WHATWG
    // decoder maps them to C1 controls. No Windows-1252 text contains them,
    // so a file that does is in some other encoding altogether.
    let suspect = text
        .chars()
        .any(|character| ('\u{80}'..='\u{9f}').contains(&character));
    (text.into_owned(), suspect)
}

/// Whether bytes that are not valid UTF-8 nevertheless contain well-formed
/// multi-byte UTF-8 characters. Windows-1252 text almost never does: an
/// accented letter there is one byte, followed by an ordinary letter that a
/// UTF-8 lead byte cannot be followed by.
fn holds_utf8_text(bytes: &[u8]) -> bool {
    bytes.utf8_chunks().any(|chunk| !chunk.valid().is_ascii())
}

/// Reads a standalone image file as a one-page document. There is no text
/// layer to prefer, so the page is OCR'd and kept as the page image.
///
/// A TIFF can hold a page per frame - a fax or a batch scan usually does -
/// and only the first frame is read here, because neither this decoder nor
/// the CCITT-compressed files these arrive in support the rest. What the
/// caller must not do is believe it received the whole document, so unread
/// frames are reported as truncation.
pub fn extract_image(
    path: &Path,
    ocr: &dyn OcrBackend,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    cancel.check()?;
    let image = load_oriented_image(path, limits)?;
    let rendered = RenderedPage::new(0, image);
    cancel.report_progress("ocr", 0, Some(1));
    let result = ocr.recognize(&rendered, cancel)?;
    let mut warnings = Vec::new();
    if result.mean_confidence < CONFIDENT_READING {
        warnings.push(ExtractionWarning::LowOcrConfidence);
    }
    let truncated = has_unread_frames(path);
    if truncated {
        warnings.push(ExtractionWarning::TextTruncated);
    }
    let optional_image = Some(normalize_vision_image(
        0,
        apply_detected_rotation(rendered.image, result.rotation_degrees)?,
    )?);
    Ok(ExtractedDocument {
        pages: vec![ExtractedPage {
            page_number: 1,
            text: result.text,
            source: PageSource::Ocr,
            ocr_confidence: Some(result.mean_confidence),
            vision_escalated: true,
        }],
        warnings,
        truncated,
        optional_image,
    })
}

/// Whether an image file holds frames after the one that was read.
///
/// A TIFF is a chain of image file directories; the first one's link to the
/// next is all this needs, and it is read rather than decoded so a
/// twelve-frame fax costs one seek. Anything that does not read as a TIFF
/// holds one image by construction, and a file whose chain cannot be
/// followed is reported as single-framed rather than as an error: the frame
/// that was read is still the document's first page.
fn has_unread_frames(path: &Path) -> bool {
    fn next_directory(path: &Path) -> Option<u64> {
        let mut file = File::open(path).ok()?;
        let mut header = [0_u8; 8];
        file.read_exact(&mut header).ok()?;
        let big_endian = match &header[..2] {
            b"II" => false,
            b"MM" => true,
            _ => return None,
        };
        let word = |bytes: [u8; 2]| {
            if big_endian {
                u16::from_be_bytes(bytes)
            } else {
                u16::from_le_bytes(bytes)
            }
        };
        let long = |bytes: [u8; 4]| {
            if big_endian {
                u32::from_be_bytes(bytes)
            } else {
                u32::from_le_bytes(bytes)
            }
        };
        let quad = |bytes: [u8; 8]| {
            if big_endian {
                u64::from_be_bytes(bytes)
            } else {
                u64::from_le_bytes(bytes)
            }
        };
        // Classic TIFF marks itself 42 and counts in 32-bit words; BigTIFF
        // marks itself 43 and counts in 64-bit ones, with wider entries.
        let (first, entry_bytes, big) = match word([header[2], header[3]]) {
            42 => (
                u64::from(long([header[4], header[5], header[6], header[7]])),
                12_u64,
                false,
            ),
            43 => {
                let mut offset = [0_u8; 8];
                file.read_exact(&mut offset).ok()?;
                (quad(offset), 20_u64, true)
            }
            _ => return None,
        };
        file.seek(SeekFrom::Start(first)).ok()?;
        let entries = if big {
            let mut count = [0_u8; 8];
            file.read_exact(&mut count).ok()?;
            quad(count)
        } else {
            let mut count = [0_u8; 2];
            file.read_exact(&mut count).ok()?;
            u64::from(word(count))
        };
        file.seek(SeekFrom::Current(
            i64::try_from(entries.checked_mul(entry_bytes)?).ok()?,
        ))
        .ok()?;
        if big {
            let mut next = [0_u8; 8];
            file.read_exact(&mut next).ok()?;
            Some(quad(next))
        } else {
            let mut next = [0_u8; 4];
            file.read_exact(&mut next).ok()?;
            Some(u64::from(long(next)))
        }
    }
    next_directory(path).is_some_and(|next| next != 0)
}

/// Decodes an image file the right way up and no larger than a rendered page
/// may be.
///
/// The file's own size is checked before any pixel is decoded, against a cap
/// four times the page cap: a 48- or 50-megapixel phone photo of a receipt is
/// an ordinary document, and refusing it outright lost the document. What is
/// decoded is then scaled down to the page cap, so OCR, the page image, and
/// everything after them see exactly the size a rendered PDF page would be.
///
/// The decoded image is the largest thing this holds, and nothing else of its
/// size is made: it is scaled down before it is turned upright, so turning it
/// copies a page-sized image rather than the photo, and the scaling writes
/// straight into the page-sized copy. A 100-megapixel photo costs its 300 MB
/// of pixels and a 75 MB page beside them.
pub fn load_oriented_image(
    path: &Path,
    limits: &ResourceLimits,
) -> Result<DynamicImage, ExtractionError> {
    let metadata = std::fs::metadata(path).map_err(ExtractionError::io)?;
    limits.validate_source_size(metadata.len())?;
    let mut decoder = ImageReader::open(path)
        .map_err(ExtractionError::io)?
        .with_guessed_format()
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?
        .into_decoder()
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    let (encoded_width, encoded_height) = decoder.dimensions();
    limits.validate_image_file_pixels(encoded_width, encoded_height)?;
    limits.validate_image_file_bytes(decoder.total_bytes())?;
    let orientation = decoder
        .orientation()
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    let image = DynamicImage::from_decoder(decoder)
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    let mut image = within_page_pixels(image, limits.max_page_pixels);
    // Turning an image keeps its pixel count, so the page it was scaled to
    // is still within the cap once it is upright.
    image.apply_orientation(orientation);
    limits.validate_page_pixels(image.width(), image.height())?;
    Ok(image.into_rgb8().into())
}

/// The image scaled down, keeping its proportions, to at most `max_pixels`;
/// an image already within them is returned as it is.
fn within_page_pixels(image: DynamicImage, max_pixels: u64) -> DynamicImage {
    let (width, height) = image.dimensions();
    let pixels = u64::from(width) * u64::from(height);
    if pixels <= max_pixels {
        return image;
    }
    // Each edge rounded down, so the two together cannot round back over
    // the budget they were scaled to meet.
    let scale = (max_pixels as f64 / pixels as f64).sqrt();
    let scaled_width = (f64::from(width) * scale).floor().max(1.0) as u32;
    let scaled_height = (f64::from(height) * scale).floor().max(1.0) as u32;
    // Each page pixel is the average of the photo pixels it covers, summed in
    // integers straight into the page. The filtered resamplers first build a
    // full-width copy in 32-bit floats per channel - half a gigabyte for a
    // 48-megapixel photo - and at a reduction this large an area average is
    // all OCR can see of the difference.
    image.thumbnail_exact(scaled_width, scaled_height)
}

#[derive(Debug)]
pub struct SourceSnapshot {
    _workspace: TempWorkspace,
    path: PathBuf,
}

impl SourceSnapshot {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn snapshot_source(
    source: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<SourceSnapshot, ExtractionError> {
    snapshot_source_after_open(source, limits, cancel, || {})
}

fn snapshot_source_after_open<F>(
    source: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
    after_open: F,
) -> Result<SourceSnapshot, ExtractionError>
where
    F: FnOnce(),
{
    cancel.check()?;
    let mut input = File::open(source).map_err(ExtractionError::io)?;
    let before = input.metadata().map_err(ExtractionError::io)?;
    limits.validate_source_size(before.len())?;
    after_open();
    let workspace = TempWorkspace::create("source", limits.max_temp_bytes)?;
    let relative = match source.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if !extension.is_empty() => format!("source.{extension}"),
        _ => "source".to_owned(),
    };
    let path =
        workspace.write_from_reader(relative, &mut input, limits.max_source_bytes, cancel)?;
    let after = input.metadata().map_err(ExtractionError::io)?;
    if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
        return Err(ExtractionError::parse_failed(
            "source file changed while it was being snapshotted",
        ));
    }
    cancel.check()?;
    Ok(SourceSnapshot {
        _workspace: workspace,
        path,
    })
}

#[cfg(all(test, unix))]
mod snapshot_tests {
    use super::*;

    #[test]
    fn snapshot_stays_bound_to_open_file_when_source_path_is_replaced() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.txt");
        std::fs::write(&source, b"original").unwrap();
        let replacement = directory.path().join("replacement.txt");
        std::fs::write(&replacement, b"replacement").unwrap();

        let snapshot = snapshot_source_after_open(
            &source,
            &ResourceLimits::default(),
            &CancellationToken::new(),
            || std::fs::rename(&replacement, &source).unwrap(),
        )
        .unwrap();

        assert_eq!(std::fs::read(snapshot.path()).unwrap(), b"original");
        assert_eq!(std::fs::read(source).unwrap(), b"replacement");
    }
}
