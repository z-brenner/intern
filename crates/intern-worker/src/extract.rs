use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex, PoisonError,
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

use crate::layout::{NativePage, PageLayout, PageRoute, RouteSignals, measure_signals, route_page};
use crate::limits::{
    MAX_DOCUMENT_CHARS, MAX_DOCUMENT_LAYOUT_PARTS, MAX_EXTRACTION_DURATION, MAX_PAGE_CHARS,
    MAX_PAGE_LAYOUT_PARTS, MAX_VISION_LONG_EDGE, MIN_OCR_DPI, RENDER_DPI, ResourceLimits,
    VISION_GRID,
};
use crate::temp::TempWorkspace;
use crate::timing::{ExtractionTimings, micros_since};

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
    timings: Mutex<ExtractionTimings>,
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

/// What a reader carries for one request: whether to stop, where to say how
/// far it has got, and where to say how long each stage took.
///
/// Progress and timings ride on the token because the token already reaches
/// every reader and every page loop, and because neither belongs in the
/// readers' signatures: a reader that has nothing to report never sees a
/// sink, and one that does report cannot tell a sink from none.
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
            timings: Mutex::new(ExtractionTimings::default()),
        }))
    }

    /// Says how far through the document a reader is. Cheap enough to call
    /// once per page: deciding whether it is worth sending is the sink's.
    pub fn report_progress(&self, stage: &'static str, current: usize, total: Option<usize>) {
        if let Some(sink) = &self.0.progress {
            sink(stage, current, total);
        }
    }

    /// Adds to this request's [`ExtractionTimings`].
    ///
    /// Timings are a measurement, never a reason to fail a document, so a
    /// lock poisoned by a panic elsewhere is recorded into all the same.
    pub fn record(&self, update: impl FnOnce(&mut ExtractionTimings)) {
        update(
            &mut self
                .0
                .timings
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
    }

    /// Runs `work` and adds the time it took to the stage `stage` names,
    /// whether or not it succeeded.
    pub fn timed<T>(
        &self,
        stage: impl FnOnce(&mut ExtractionTimings) -> &mut u64,
        work: impl FnOnce() -> T,
    ) -> T {
        let started = Instant::now();
        let result = work();
        let elapsed = micros_since(started);
        self.record(|timings| {
            let total = stage(timings);
            *total = total.saturating_add(elapsed);
        });
        result
    }

    /// What has been recorded so far.
    pub fn timings(&self) -> ExtractionTimings {
        *self
            .0
            .timings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct PdfPageInspection {
    pub page_index: usize,
    pub native_text: String,
    pub image_coverage: f32,
    pub width_pixels: u32,
    pub height_pixels: u32,
    /// What the backend measured about the page's geometry for the router
    /// and the layout analysis. None from a backend that measures nothing,
    /// whose pages are routed on their text and image coverage alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native: Option<NativePage>,
    /// The router's signals, when the backend took them while it had the
    /// page open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signals: Option<RouteSignals>,
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
    /// Clockwise non-EXIF rotation that stands the image read upright: the
    /// turn applied before OCR, plus any the engine made itself (Tesseract
    /// reads a page lying on its side as vertical text).
    pub rotation_degrees: u16,
    /// The reading line by line, where the engine reports lines. Empty when
    /// it does not, and then `text` is all there is.
    #[serde(default)]
    pub lines: Vec<OcrLine>,
}

/// One line of an OCR reading: what it says, where it sits, and how sure
/// the engine was of it.
///
/// `bbox` is `[x0, y0, x1, y1]` in pixels of the image the engine read,
/// turned by `rotation_degrees`, so it is in the page's upright orientation
/// with the origin at the top left. `confidence` is 0-100.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OcrLine {
    pub text: String,
    pub bbox: [u32; 4],
    pub confidence: u8,
}

impl OcrResult {
    pub fn new(text: impl Into<String>, mean_confidence: f32) -> Self {
        Self {
            text: text.into(),
            mean_confidence,
            rotation_degrees: 0,
            lines: Vec::new(),
        }
    }

    pub fn with_rotation(mut self, rotation_degrees: u16) -> Self {
        self.rotation_degrees = rotation_degrees;
        self
    }

    pub fn with_lines(mut self, lines: Vec<OcrLine>) -> Self {
        self.lines = lines;
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

    /// The page's text as runs ([`crate::layout::TextRun`]), for a page
    /// routed to read its geometry whose inspection carries none - one past
    /// the runs a document reads ahead
    /// ([`crate::layout::router::bounds::MAX_DOCUMENT_RUNS`]). `native` is
    /// what inspection measured of it. Asked for one page at a time, as
    /// each is read. A backend with no characters to give returns none.
    fn page_runs(
        &self,
        _path: &Path,
        _page_index: usize,
        _native: &NativePage,
        _cancel: &CancellationToken,
    ) -> Result<Vec<crate::layout::TextRun>, ExtractionError> {
        Ok(Vec::new())
    }
}

pub trait OcrBackend {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError>;

    /// How many pages the engine can usefully read at once. A PDF's scanned
    /// pages are rendered one at a time - PDFium is not thread-safe - and
    /// handed to this many OCR workers. One unless the engine says more.
    fn concurrency(&self) -> usize {
        1
    }

    /// Lets go of what the engine keeps between pages once a document has
    /// been read, so a worker that outlives every document does not hold a
    /// document's working memory while it waits for the next. The next page
    /// builds what it needs again. Nothing to let go of unless the engine
    /// says so.
    fn release(&self) {}
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
    /// The page as blocks in reading order (see [`crate::layout`]). Every
    /// reader's pages carry one by the time the protocol sends them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<PageLayout>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExtractionWarning {
    LowOcrConfidence,
    NativeTextCorrupt,
    /// Text that was lost: a page cut at the size cap, frames of a TIFF past
    /// the page limit or that could not be decoded. What was dropped is
    /// unknown, so it may be the fact that names the document.
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
    /// Where the extraction's time went. Readers leave this empty: the
    /// protocol fills it from the request's token once the reader returns,
    /// so every parsed document the worker sends carries it. Only a host
    /// that knows the field accepts it (see the host's `WorkerDocument`);
    /// the worker and the host ship together.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timings: Option<ExtractionTimings>,
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

/// The most OCR workers a PDF is read with, whatever the engine offers.
const MAX_OCR_WORKERS: usize = 8;

/// What becomes of one page while a PDF is read: finished already, or
/// waiting for OCR.
enum PagePlan {
    Done {
        page: NativeRead,
        /// The page image this page would become, if it is the first page
        /// that wants one.
        vision: Option<VisionImage>,
    },
    /// A scan: OCR's reading is the page.
    Scan {
        page_number: usize,
        /// Its text layer held replacement glyphs.
        corrupt: bool,
        signals: RouteSignals,
        /// Tenths of a point per rendered pixel.
        scale: f64,
    },
    /// A text layer that is a bad prior OCR, read again. The fresh reading
    /// replaces it only if it is better.
    Reread {
        inspection: PdfPageInspection,
        signals: RouteSignals,
        vision: Option<VisionImage>,
    },
    /// Native text, with images that may hold text of their own. Its
    /// characters are read once OCR's readings of those images are back.
    Regions {
        inspection: PdfPageInspection,
        signals: RouteSignals,
        /// Each region's box in the rendered page, in pixels.
        crops: Vec<[u32; 4]>,
        /// Tenths of a point per rendered pixel.
        scale: f64,
        vision: Option<VisionImage>,
    },
}

impl PagePlan {
    /// Whether the page brings the page image with it for certain. A page
    /// read again does not: if the fresh reading wins, the image made from
    /// its text layer goes with the layer, and a later page that wants one
    /// has to have been rendered for it.
    fn has_page_image(&self) -> bool {
        match self {
            PagePlan::Done { vision, .. } | PagePlan::Regions { vision, .. } => vision.is_some(),
            PagePlan::Reread { .. } | PagePlan::Scan { .. } => false,
        }
    }
}

/// Whether an error ends the whole document rather than an optional reading
/// it was for: the person canceled, or the time ran out.
fn ends_the_document(error: &ExtractionError) -> bool {
    matches!(
        error.kind,
        ExtractionErrorKind::Canceled | ExtractionErrorKind::ResourceLimit
    )
}

/// Reads a PDF.
///
/// Every page is routed ([`crate::layout::route_page`]) on what inspection
/// measured. Pages read from their text are finished in the loop. Pages
/// that need OCR are rendered one at a time - PDFium is not thread-safe, and
/// the backend that inspected the document is the one that renders it -
/// and handed through a bounded queue to a pool of OCR workers, as many as
/// the engine says it can use ([`OcrBackend::concurrency`]). The pages are
/// put back in order at the end, and every decision that depends on order -
/// which page becomes the page image, the order of warnings, which error is
/// reported - is made then, exactly as reading the pages one after another
/// would make it.
pub fn extract_pdf<O>(
    path: &Path,
    pdf: &dyn PdfBackend,
    ocr: &O,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError>
where
    O: OcrBackend + Sync + ?Sized,
{
    let started = Instant::now();
    timed_check(cancel, started, limits)?;
    let inspections = pdf.inspect(path, cancel)?;
    limits.validate_page_count(inspections.len())?;
    let page_count = inspections.len();
    let workers = ocr.concurrency().clamp(1, MAX_OCR_WORKERS);
    let queue = limits.max_queued_rendered_pages.max(1);
    let (plans, outcomes, failure) = std::thread::scope(|scope| {
        let mut pool = OcrPool::new(scope, ocr, cancel, workers, queue, page_count);
        let mut plans = Vec::with_capacity(page_count);
        let mut failure: Option<(usize, ExtractionError)> = None;
        // Whether a page already planned brings the page image with it, so
        // no later page needs to be rendered for one. A scan's reading may
        // still turn out to want it, and an earlier page's claim wins when
        // the pages are put back in order.
        let mut vision_taken = false;
        // The characters of the pages read as their text so far, in page
        // order: a page past the document's is not read for its geometry.
        // And the geometry layouts built so far, which are held until the
        // pages are put back in order: one past what a document's layouts
        // may hold is not kept, and its page is read as its text.
        let mut planned = TextBudget::document();
        for inspection in inspections {
            let page_index = inspection.page_index;
            if let Err(error) = timed_check(cancel, started, limits) {
                failure = Some((page_index, error));
                break;
            }
            // A scan that could not be read ends the document; there is
            // no point rendering the pages after it.
            if pool.failed() {
                break;
            }
            pool.collect_ready();
            cancel.report_progress("reading", page_index, Some(page_count));
            match plan_page(
                inspection,
                pdf,
                path,
                limits,
                cancel,
                started,
                &mut pool,
                vision_taken,
                &mut planned,
            ) {
                Ok(plan) => {
                    if matches!(plan, PagePlan::Done { .. }) {
                        pool.finished += 1;
                    }
                    vision_taken |= plan.has_page_image();
                    plans.push(plan);
                }
                Err(error) => {
                    failure = Some((page_index, error));
                    break;
                }
            }
        }
        let outcomes = pool.finish();
        (plans, outcomes, failure)
    });

    // The error reading the pages in order would have met first: a scan
    // OCR could not read, or anything that ends the document. A reading
    // skipped as canceled is not one: the workers skip what is left once a
    // scan has failed - even a region of an earlier page, still queued - and
    // that scan's own error is the one to report. A request that really was
    // canceled is reported by the check below.
    let ocr_failure = outcomes.iter().find_map(|outcome| {
        let scan = matches!(plans.get(outcome.page_index), Some(PagePlan::Scan { .. }));
        outcome.readings.iter().find_map(|reading| match reading {
            Err(error)
                if error.kind != ExtractionErrorKind::Canceled
                    && (scan || ends_the_document(error)) =>
            {
                Some((outcome.page_index, error.clone()))
            }
            _ => None,
        })
    });
    let first_failure = match (failure, ocr_failure) {
        (Some(main), Some(ocr)) => Some(if ocr.0 <= main.0 { ocr } else { main }),
        (main, ocr) => main.or(ocr),
    };
    if let Some((_, error)) = first_failure {
        return Err(error);
    }
    // A request canceled while its last pages were being read is canceled,
    // even if every page it was waiting for came back.
    cancel.check()?;
    let stop = || halted(cancel, started, limits);
    let runs =
        |inspection: &mut PdfPageInspection, route| read_runs(inspection, route, pdf, path, cancel);
    let document = cancel.timed(
        |timings| &mut timings.analysis_micros,
        || assemble(plans, outcomes, &runs, &stop),
    )?;
    // A layout the time ran out on was left unbuilt; the document is not
    // returned half-read as if it were whole.
    timed_check(cancel, started, limits)?;
    Ok(document)
}

/// Whether the request was canceled or its time is up - the token's own
/// deadline, or the extraction's: what the layout analysis asks between
/// the regions of a page.
fn halted(cancel: &CancellationToken, started: Instant, limits: &ResourceLimits) -> bool {
    cancel.check().is_err() || started.elapsed() > limits.max_duration
}

/// Decides how one page is read, finishes it if it needs no OCR, and hands
/// it to the OCR workers if it does.
#[allow(clippy::too_many_arguments)]
fn plan_page<O: OcrBackend + Sync + ?Sized>(
    mut inspection: PdfPageInspection,
    pdf: &dyn PdfBackend,
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
    started: Instant,
    pool: &mut OcrPool<'_, '_, O>,
    vision_taken: bool,
    planned: &mut TextBudget,
) -> Result<PagePlan, ExtractionError> {
    let page_index = inspection.page_index;
    let page_number = page_index + 1;
    // The render cap belongs to rendering. A large-format sheet - an A1
    // drawing, a plan set - is over it at 300 DPI while carrying a
    // perfectly good text layer, and failing the whole document over a
    // page nobody was going to rasterise loses the document. A page that
    // is too large to render also cannot be escalated to vision, but it
    // keeps its text: the page image is the optional part.
    let renderable = limits
        .validate_page_pixels(inspection.width_pixels, inspection.height_pixels)
        .is_ok();
    let (needs_ocr, signals) = cancel.timed(
        |timings| &mut timings.analysis_micros,
        || (page_needs_ocr(&inspection), signals_of(&inspection)),
    );
    let route = route_page(&signals, needs_ocr);

    if needs_ocr {
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
        let rendered = render_for_ocr(pdf, path, page_index, limits, cancel)?;
        let scale = display_width(&inspection) / f64::from(rendered.image.width().max(1));
        pool.submit(OcrJob {
            page_index,
            pieces: vec![rendered],
            whole_page: true,
            scan: true,
        });
        return Ok(PagePlan::Scan {
            page_number,
            corrupt: inspection.native_text.contains('\u{fffd}'),
            signals,
            scale,
        });
    }

    // Native text from here on. Whatever OCR adds is optional: a page that
    // cannot be rendered or read again keeps the text it has. The page image
    // is the first page's that wants one; once an earlier page has brought
    // it, this one is not rendered for it.
    let vision = if renderable && !vision_taken && page_needs_vision(&inspection) {
        let rendered = cancel.timed(
            |timings| &mut timings.render_micros,
            || pdf.render(path, page_index, cancel),
        )?;
        let (render_width, render_height) = rendered.image.dimensions();
        record_rendered(cancel, render_width, render_height);
        limits.validate_page_pixels(render_width, render_height)?;
        Some(cancel.timed(
            |timings| &mut timings.vision_micros,
            || page_image(page_index, &rendered.image, 0),
        )?)
    } else {
        None
    };
    let rerenderable =
        renderable && ocr_render_dpi(&inspection, limits.max_page_pixels) >= MIN_OCR_DPI;

    if route == PageRoute::Ocr && rerenderable {
        match render_for_ocr(pdf, path, page_index, limits, cancel) {
            Ok(rendered) => {
                pool.submit(OcrJob {
                    page_index,
                    pieces: vec![rendered],
                    whole_page: true,
                    scan: false,
                });
                return Ok(PagePlan::Reread {
                    inspection,
                    signals,
                    vision,
                });
            }
            Err(error) if ends_the_document(&error) => return Err(error),
            Err(_) => {}
        }
    }

    if route == PageRoute::OcrRegions
        && rerenderable
        && let Some(native) = &inspection.native
    {
        match render_for_ocr(pdf, path, page_index, limits, cancel) {
            Ok(rendered) => {
                let (width, height) = rendered.image.dimensions();
                let scale = f64::from(native.display_size().0) / f64::from(width.max(1));
                let crops = crate::layout::text_regions(native)
                    .into_iter()
                    .filter_map(|region| {
                        let display = native.to_display(region);
                        let [x0, y0, x1, y1] =
                            display.map(|value| (f64::from(value) / scale).round() as u32);
                        let (x0, x1) = (x0.min(width), x1.min(width));
                        let (y0, y1) = (y0.min(height), y1.min(height));
                        (x1 > x0 + 8 && y1 > y0 + 8).then_some([x0, y0, x1, y1])
                    })
                    .collect::<Vec<_>>();
                if !crops.is_empty() {
                    let pieces = crops
                        .iter()
                        .map(|crop| {
                            RenderedPage::new(
                                page_index,
                                rendered.image.crop_imm(
                                    crop[0],
                                    crop[1],
                                    crop[2] - crop[0],
                                    crop[3] - crop[1],
                                ),
                            )
                        })
                        .collect();
                    pool.submit(OcrJob {
                        page_index,
                        pieces,
                        whole_page: false,
                        scan: false,
                    });
                    return Ok(PagePlan::Regions {
                        inspection,
                        signals,
                        crops,
                        scale,
                        vision,
                    });
                }
            }
            Err(error) if ends_the_document(&error) => return Err(error),
            Err(_) => {}
        }
    }

    // Counted with the pages read as their text before it, a page past the
    // document's characters is cut on the way out whatever the scans among
    // them read, so it is not read for its geometry: no layout is built
    // for it and held until the pages are put in order, only to be dropped.
    // What the layouts held while planning repeat is bounded by the
    // document's characters and a page.
    let route = if goes_out_whole(&inspection.native_text, &mut planned.characters) {
        route
    } else {
        PageRoute::Fast
    };
    cancel.timed(
        |timings| &mut timings.analysis_micros,
        || read_runs(&mut inspection, route, pdf, path, cancel),
    )?;
    let route = native_route(route, &signals);
    let stop = || halted(cancel, started, limits);
    let page = cancel.timed(
        |timings| &mut timings.analysis_micros,
        || native_page(page_number, inspection, signals, route, &stop, planned),
    );
    Ok(PagePlan::Done { page, vision })
}

/// Reads a page's characters into runs if `route` reads its geometry and
/// inspection did not: a page past the runs a document reads ahead.
///
/// They are read when the page is - once it is planned, or for a page
/// whose image regions OCR reads, once those readings are back - so past
/// that budget no more than one page's characters are held at a time. A
/// page whose characters cannot be read keeps its text.
fn read_runs(
    inspection: &mut PdfPageInspection,
    route: PageRoute,
    pdf: &dyn PdfBackend,
    path: &Path,
    cancel: &CancellationToken,
) -> Result<(), ExtractionError> {
    let page_index = inspection.page_index;
    if crate::layout::router::needs_runs(route)
        && fits_a_page(&inspection.native_text)
        && let Some(native) = inspection.native.as_mut()
        && native.runs.is_empty()
    {
        match pdf.page_runs(path, page_index, native, cancel) {
            Ok(runs) => native.runs = runs,
            Err(error) if ends_the_document(&error) => return Err(error),
            Err(_) => {}
        }
    }
    Ok(())
}

/// The router's signals for a page: what the backend measured while it had
/// the page open, or, from a backend that measures nothing, the image
/// coverage alone.
fn signals_of(inspection: &PdfPageInspection) -> RouteSignals {
    inspection
        .signals
        .unwrap_or_else(|| match &inspection.native {
            Some(native) => {
                measure_signals(native, &inspection.native_text, inspection.image_coverage)
            }
            None => RouteSignals {
                image_coverage: (inspection.image_coverage.clamp(0.0, 1.0) * 1000.0).round() as u16,
                ..RouteSignals::default()
            },
        })
}

/// A page's displayed width in tenths of a point: what the backend
/// measured, or the width inspection gives in pixels at 300 DPI.
fn display_width(inspection: &PdfPageInspection) -> f64 {
    match &inspection.native {
        Some(native) => f64::from(native.display_size().0),
        None => {
            f64::from(inspection.width_pixels) * 72.0 / RENDER_DPI * crate::layout::UNITS_PER_POINT
        }
    }
}

/// The route a page read from its own text takes: the one it was given,
/// unless OCR was what it wanted and could not have, in which case its text
/// routes it as if it had no image to read.
fn native_route(route: PageRoute, signals: &RouteSignals) -> PageRoute {
    match route {
        PageRoute::Fast | PageRoute::Layout => route,
        PageRoute::Ocr | PageRoute::OcrRegions => route_page(
            &RouteSignals {
                image_region: 0,
                garbage: 0,
                ..*signals
            },
            false,
        ),
    }
}

/// Renders a page for OCR within the pixel budget, counting what was
/// rendered and holding the render to the budget it was sized for.
fn render_for_ocr(
    pdf: &dyn PdfBackend,
    path: &Path,
    page_index: usize,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<RenderedPage, ExtractionError> {
    let rendered = cancel.timed(
        |timings| &mut timings.render_micros,
        || pdf.render_within(path, page_index, limits.max_page_pixels, cancel),
    )?;
    // The backend sized the render; this is what holds it to that.
    let (render_width, render_height) = rendered.image.dimensions();
    record_rendered(cancel, render_width, render_height);
    limits.validate_page_pixels(render_width, render_height)?;
    Ok(rendered)
}

/// A page read from its native text: exactly as PDFium read it on the fast
/// route, or the linearization of its blocks on the layout route - unless
/// the page is past what the analysis takes on, or `stop` says the request
/// is canceled or out of time, when it is read as on the fast route.
///
/// A page read as its text has its layout built only when the pages are
/// put in order (see [`NativeRead::finish`]), and only if the worker will
/// send that text whole.
fn native_page(
    page_number: usize,
    inspection: PdfPageInspection,
    signals: RouteSignals,
    route: PageRoute,
    stop: &dyn Fn() -> bool,
    held: &mut TextBudget,
) -> NativeRead {
    // A page past the analysis's bounds, or one the time ran out on, is
    // read as its text, and so is one whose layout is more than what is
    // left of `held`.
    let geometry = inspection
        .native
        .as_ref()
        .filter(|native| route == PageRoute::Layout && !native.runs.is_empty())
        .and_then(|native| crate::layout::geometry_layout(native, Vec::new(), signals, route, stop))
        .filter(|layout| held.keeps(layout));
    let (text, layout, fast) = match geometry {
        Some(mut layout) => {
            // The page's text is written from this layout, which the bounds
            // on the analysis already keep small.
            crate::layout::number_blocks(page_number, &mut layout.blocks);
            (crate::layout::linearize(&layout.blocks), Some(layout), None)
        }
        None => (
            inspection.native_text,
            None,
            Some(FastLayout {
                native: inspection.native,
                signals,
            }),
        ),
    };
    NativeRead {
        page: ExtractedPage {
            page_number,
            text,
            source: PageSource::Native,
            ocr_confidence: None,
            vision_escalated: false,
            layout,
        },
        fast,
    }
}

/// What a fast layout is built from, kept until the pages are in order.
struct FastLayout {
    native: Option<NativePage>,
    signals: RouteSignals,
}

/// A page read from its own text: with the layout its text was written
/// from, or, read as its text, with what its fast layout would be built
/// from.
struct NativeRead {
    page: ExtractedPage,
    fast: Option<FastLayout>,
}

impl NativeRead {
    /// The page, counted against the document's characters in page order
    /// as the worker will count them (see [`goes_out_whole`]): a page read
    /// as its text gets its fast layout only if it goes out whole - a
    /// layout repeats its text several times over - and its lines and
    /// cells fit what is left (see [`TextBudget`]), and a page whose text
    /// was written from its layout lets that layout go if it does not.
    fn finish(self, budget: &mut TextBudget) -> ExtractedPage {
        let Self { mut page, fast } = self;
        match fast {
            Some(FastLayout { native, signals }) => {
                if budget.admits(&page.text) {
                    let mut layout =
                        crate::layout::fast_layout(&page.text, native.as_ref(), signals);
                    crate::layout::number_blocks(page.page_number, &mut layout.blocks);
                    page.layout = Some(layout).filter(|layout| budget.keeps(layout));
                }
                page
            }
            None => counted(page, budget),
        }
    }
}

/// Whether a fresh OCR reading of a page whose text layer looked like bad
/// OCR is better than that layer: confident, or with fewer of the errors
/// that sent the page to be read again.
fn reread_is_better(reading: &OcrResult, signals: &RouteSignals) -> bool {
    !reading.text.trim().is_empty()
        && (reading.mean_confidence >= CONFIDENT_READING
            || crate::layout::router::garbage_score(&reading.text) < signals.garbage)
}

/// Puts the pages back in order and makes every decision that depends on
/// it: the page image is the first page that wants one, and warnings are
/// raised in the order the pages raise them.
fn assemble(
    plans: Vec<PagePlan>,
    outcomes: Vec<OcrOutcome>,
    runs: &dyn Fn(&mut PdfPageInspection, PageRoute) -> Result<(), ExtractionError>,
    stop: &dyn Fn() -> bool,
) -> Result<ExtractedDocument, ExtractionError> {
    // The document's characters, counted down in page order as the worker
    // will count them when it sends the pages: a page it will cut has no
    // layout built, or keeps none. Fast layouts also count their lines and
    // cells against it.
    let mut budget = TextBudget::document();
    // A page read as its text here, when reading it again or its regions
    // came to nothing, is counted as it is put in order, not as it is read.
    let mut unheld = TextBudget {
        characters: usize::MAX,
        layout_parts: usize::MAX,
    };
    let mut outcomes = outcomes
        .into_iter()
        .map(|outcome| (outcome.page_index, outcome))
        .collect::<std::collections::HashMap<_, _>>();
    let mut pages = Vec::with_capacity(plans.len());
    let mut warnings: Vec<ExtractionWarning> = Vec::new();
    let mut vision_candidate: Option<VisionImage> = None;
    for (page_index, plan) in plans.into_iter().enumerate() {
        let outcome = outcomes.remove(&page_index);
        let (mut page, vision) = match plan {
            PagePlan::Done { page, vision } => (page.finish(&mut budget), vision),
            PagePlan::Scan {
                page_number,
                corrupt,
                signals,
                scale,
            } => {
                if corrupt && !warnings.contains(&ExtractionWarning::NativeTextCorrupt) {
                    warnings.push(ExtractionWarning::NativeTextCorrupt);
                }
                let unread = || ExtractionError::parse_failed("a scanned page was never read");
                let OcrOutcome {
                    readings,
                    sizes,
                    vision,
                    ..
                } = outcome.ok_or_else(unread)?;
                let reading = readings.into_iter().next().ok_or_else(unread)??;
                let size = sizes.first().copied().ok_or_else(unread)?;
                let vision = vision.transpose()?;
                (
                    counted(
                        crate::layout::ocr_page(
                            page_number,
                            reading,
                            size,
                            Some(scale),
                            signals,
                            stop,
                        ),
                        &mut budget,
                    ),
                    vision,
                )
            }
            PagePlan::Reread {
                inspection,
                signals,
                vision,
            } => {
                let page_number = inspection.page_index + 1;
                let fresh = outcome.and_then(|outcome| {
                    let size = *outcome.sizes.first()?;
                    let page_vision = outcome.vision.and_then(Result::ok);
                    let reading = outcome.readings.into_iter().next()?.ok()?;
                    Some((reading, size, page_vision))
                });
                match fresh {
                    Some((reading, size, page_vision)) if reread_is_better(&reading, &signals) => {
                        let scale = display_width(&inspection) / f64::from(size.0.max(1));
                        (
                            counted(
                                crate::layout::ocr_page(
                                    page_number,
                                    reading,
                                    size,
                                    Some(scale),
                                    signals,
                                    stop,
                                ),
                                &mut budget,
                            ),
                            page_vision,
                        )
                    }
                    fresh => {
                        // The layer stays, but a reading that came back
                        // unsure says the page is hard to read either way:
                        // the image it was rendered with goes with the
                        // layer's text, as it would with a scan's.
                        let reread_vision = fresh.and_then(|(_, _, page_vision)| page_vision);
                        let route = native_route(PageRoute::Ocr, &signals);
                        (
                            native_page(page_number, inspection, signals, route, stop, &mut unheld)
                                .finish(&mut budget),
                            vision.or(reread_vision),
                        )
                    }
                }
            }
            PagePlan::Regions {
                mut inspection,
                signals,
                crops,
                scale,
                vision,
            } => {
                let page_number = inspection.page_index + 1;
                runs(&mut inspection, PageRoute::OcrRegions)?;
                let readings = outcome
                    .map(|outcome| {
                        outcome
                            .readings
                            .into_iter()
                            .zip(crops)
                            .filter_map(|(reading, crop)| Some((reading.ok()?, crop)))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let page = crate::layout::regions_page(
                    page_number,
                    &inspection,
                    signals,
                    &readings,
                    scale,
                    stop,
                )
                .map(|page| counted(page, &mut budget))
                .unwrap_or_else(|| {
                    let route = native_route(PageRoute::OcrRegions, &signals);
                    native_page(page_number, inspection, signals, route, stop, &mut unheld)
                        .finish(&mut budget)
                });
                (page, vision)
            }
        };
        if page.source == PageSource::Ocr
            && page
                .ocr_confidence
                .is_some_and(|confidence| confidence < CONFIDENT_READING)
            && !warnings.contains(&ExtractionWarning::LowOcrConfidence)
        {
            warnings.push(ExtractionWarning::LowOcrConfidence);
        }
        page.vision_escalated = vision.is_some() && vision_candidate.is_none();
        if page.vision_escalated {
            vision_candidate = vision;
        }
        pages.push(page);
    }
    link_sections(&mut pages);
    Ok(ExtractedDocument {
        pages,
        warnings,
        truncated: false,
        optional_image: vision_candidate,
        timings: None,
    })
}

/// Marks every page's running header and footer, and links every block on
/// every page to the heading it falls under, across pages.
pub fn link_sections(pages: &mut [ExtractedPage]) {
    let mut layouts = pages
        .iter_mut()
        .filter_map(|page| page.layout.as_mut())
        .collect::<Vec<_>>();
    crate::layout::mark_running_blocks(&mut layouts);
    crate::layout::assign_sections(layouts);
}

impl ExtractedPage {
    /// A page from a reader that knows no geometry, with the blocks of its
    /// text if it fits what the worker sends (see
    /// [`ExtractedPage::of_text_within`]).
    pub fn of_text(page_number: usize, text: String, source: PageSource) -> Self {
        Self::of_text_within(page_number, text, source, &mut TextBudget::document())
    }

    /// A page from a reader that knows no geometry, with the blocks of its
    /// text while the text fits what the worker sends. The worker cuts a
    /// page past [`MAX_PAGE_CHARS`], and every page past a document's first
    /// [`MAX_DOCUMENT_CHARS`], on the way out, and a page it cuts loses its
    /// layout there; a layout repeats its text several times over, so for
    /// such a page none is built. Nor is one for a page whose lines and
    /// cells are more than a layout may hold (see [`TextBudget`]). `budget`
    /// is what is left of the document's, counted down here as the worker
    /// will count it.
    pub fn of_text_within(
        page_number: usize,
        text: String,
        source: PageSource,
        budget: &mut TextBudget,
    ) -> Self {
        let layout = budget
            .admits(&text)
            .then(|| {
                let mut layout = PageLayout::of_text(page_number, &text);
                crate::layout::assign_sections([&mut layout]);
                layout
            })
            .filter(|layout| budget.keeps(layout));
        Self {
            page_number,
            text,
            source,
            ocr_confidence: None,
            vision_escalated: false,
            layout,
        }
    }
}

/// What is left of a document's allowance for the pages it sends: its
/// characters, as the worker counts them on the way out (see
/// [`goes_out_whole`]), and the lines, table cells and labelled values its
/// layouts may hold (see [`PageLayout::parts`]).
///
/// A layout holds an object for every line, cell and value, and the
/// character caps alone do not bound how many: two million characters can
/// be hundreds of thousands of one-letter lines, or of pipes, and a page of
/// one-character runs is a line for every character. So every layout a
/// document keeps is counted in page order against
/// [`MAX_DOCUMENT_LAYOUT_PARTS`], whatever route its page took, and one
/// past what is left is let go; the page keeps its text. A layout built
/// from text is not built at all when the text alone shows it would hold
/// more than [`MAX_PAGE_LAYOUT_PARTS`] or than the document has left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextBudget {
    pub characters: usize,
    pub layout_parts: usize,
}

impl TextBudget {
    /// A whole document's.
    pub const fn document() -> Self {
        Self {
            characters: MAX_DOCUMENT_CHARS,
            layout_parts: MAX_DOCUMENT_LAYOUT_PARTS,
        }
    }

    /// Counts the characters of a page built from `text` against the
    /// budget, and whether its layout is worth building: the page goes out
    /// whole, and its text has no more lines and pipes than a page's layout
    /// may hold or the document has left. The layout, once built, is
    /// counted by [`TextBudget::keeps`].
    fn admits(&mut self, text: &str) -> bool {
        if !goes_out_whole(text, &mut self.characters) {
            return false;
        }
        let parts = layout_parts(text);
        parts <= MAX_PAGE_LAYOUT_PARTS && parts <= self.layout_parts
    }

    /// Counts a layout against what the document's layouts may hold, and
    /// whether it fits: one that does not is let go, and its page keeps its
    /// text.
    fn keeps(&mut self, layout: &PageLayout) -> bool {
        let parts = layout.parts();
        if parts > self.layout_parts {
            return false;
        }
        self.layout_parts -= parts;
        true
    }
}

/// How many lines and table cells a layout built from `text` can hold at
/// most: one for each line, and one for each pipe, which is as many cells
/// as a table row can split into.
pub fn layout_parts(text: &str) -> usize {
    1 + text
        .bytes()
        .filter(|byte| matches!(byte, b'\n' | b'|'))
        .count()
}

/// Whether a page's text is short enough to be read for its geometry. A
/// page longer than [`MAX_PAGE_CHARS`] is cut on the way out and loses its
/// layout there, so its characters are never read into runs - a run holds
/// its text a second time, and the analysis copies it again - and it is
/// read as its text.
pub fn fits_a_page(text: &str) -> bool {
    text.char_indices().nth(MAX_PAGE_CHARS).is_none()
}

/// Whether a page of `text` leaves the worker whole, counting it against
/// `budget` - what is left of the document's [`MAX_DOCUMENT_CHARS`] - as the
/// worker will on the way out: a page keeps at most [`MAX_PAGE_CHARS`], and
/// no more than the document has left. A page that does not is cut there
/// and loses its layout, so none is worth building for it.
fn goes_out_whole(text: &str, budget: &mut usize) -> bool {
    let allowed = MAX_PAGE_CHARS.min(*budget);
    match text.char_indices().nth(allowed) {
        Some(_) => {
            *budget -= allowed;
            false
        }
        None => {
            *budget -= text.chars().count();
            true
        }
    }
}

/// A page whose text was written from its layout - read by OCR, or from
/// its geometry - counted against the document's characters as the worker
/// will count them (see [`goes_out_whole`]), and its layout against what
/// the document's layouts may hold (see [`TextBudget`]). The layout is
/// built either way; a page the worker will cut, or one whose layout does
/// not fit, lets it go here, as the page is read, instead of holding it
/// until the whole document has been. The page keeps its text.
fn counted(mut page: ExtractedPage, budget: &mut TextBudget) -> ExtractedPage {
    if !goes_out_whole(&page.text, &mut budget.characters) {
        page.layout = None;
    } else if let Some(layout) = &page.layout
        && !budget.keeps(layout)
    {
        page.layout = None;
    }
    page
}

/// One page handed to an OCR worker: the whole page, or the regions of it
/// worth reading.
struct OcrJob {
    page_index: usize,
    pieces: Vec<RenderedPage>,
    /// The whole page, which becomes the page image if it reads badly.
    whole_page: bool,
    /// A scan: if it cannot be read, neither can the document, and the
    /// pages queued after it are not worth reading.
    scan: bool,
}

/// What an OCR worker made of one job.
struct OcrOutcome {
    page_index: usize,
    readings: Vec<Result<OcrResult, ExtractionError>>,
    /// Each piece's size in pixels, as it was rendered.
    sizes: Vec<(u32, u32)>,
    /// The page image, when the whole page read unconvincingly.
    vision: Option<Result<VisionImage, ExtractionError>>,
}

/// The OCR workers for one document, started the first time a page needs
/// one: a text PDF never starts a thread.
///
/// Rendered pages wait in a queue that holds at most `queue` of them
/// ([`crate::limits::MAX_QUEUED_RENDERED_PAGES`]), so a document's renders
/// run at most that far ahead of its OCR: memory holds the page being
/// rendered, at most `queue` waiting, and one being read by each of the
/// `workers`.
struct OcrPool<'scope, 'env, O: OcrBackend + Sync + ?Sized> {
    scope: &'scope std::thread::Scope<'scope, 'env>,
    ocr: &'env O,
    cancel: &'env CancellationToken,
    workers: usize,
    queue: usize,
    jobs: Option<std::sync::mpsc::SyncSender<OcrJob>>,
    results: Option<std::sync::mpsc::Receiver<OcrOutcome>>,
    /// Set by a worker when a scan could not be read.
    failed: std::sync::Arc<AtomicBool>,
    outcomes: Vec<OcrOutcome>,
    submitted: usize,
    /// Pages finished so far, read or not, for progress.
    finished: usize,
    page_count: usize,
}

impl<'scope, 'env, O: OcrBackend + Sync + ?Sized> OcrPool<'scope, 'env, O> {
    fn new(
        scope: &'scope std::thread::Scope<'scope, 'env>,
        ocr: &'env O,
        cancel: &'env CancellationToken,
        workers: usize,
        queue: usize,
        page_count: usize,
    ) -> Self {
        Self {
            scope,
            ocr,
            cancel,
            workers,
            queue,
            jobs: None,
            results: None,
            failed: std::sync::Arc::new(AtomicBool::new(false)),
            outcomes: Vec::new(),
            submitted: 0,
            finished: 0,
            page_count,
        }
    }

    fn failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }

    fn start(&mut self) {
        let (jobs, queued) = std::sync::mpsc::sync_channel::<OcrJob>(self.queue);
        let (done, results) = std::sync::mpsc::channel::<OcrOutcome>();
        let queued = std::sync::Arc::new(Mutex::new(queued));
        for _ in 0..self.workers {
            let queued = std::sync::Arc::clone(&queued);
            let done = done.clone();
            let failed = std::sync::Arc::clone(&self.failed);
            let ocr = self.ocr;
            let cancel = self.cancel;
            self.scope.spawn(move || {
                loop {
                    // Only waiting for a job holds the lock; reading one
                    // does not.
                    let job = queued.lock().unwrap_or_else(PoisonError::into_inner).recv();
                    let Ok(job) = job else {
                        break;
                    };
                    let outcome = read_job(job, ocr, cancel, &failed);
                    if done.send(outcome).is_err() {
                        break;
                    }
                }
            });
        }
        self.jobs = Some(jobs);
        self.results = Some(results);
    }

    /// Hands a page to the workers, waiting while the queue is full.
    fn submit(&mut self, job: OcrJob) {
        if self.jobs.is_none() {
            self.start();
        }
        self.cancel
            .report_progress("ocr", self.finished, Some(self.page_count));
        // Every worker gone means every worker panicked; the scope reports
        // that when it ends.
        if self.jobs.as_ref().expect("started above").send(job).is_ok() {
            self.submitted += 1;
        }
    }

    /// Takes in whatever the workers have finished, without waiting.
    fn collect_ready(&mut self) {
        let Some(results) = &self.results else {
            return;
        };
        while let Ok(outcome) = results.try_recv() {
            self.finished += 1;
            self.outcomes.push(outcome);
        }
    }

    /// Lets the workers finish what they were given and returns every
    /// outcome, in page order.
    fn finish(mut self) -> Vec<OcrOutcome> {
        let page_count = self.page_count;
        drop(self.jobs.take());
        if let Some(results) = self.results.take() {
            while self.outcomes.len() < self.submitted {
                let Ok(outcome) = results.recv() else {
                    break;
                };
                self.finished += 1;
                // The document is not finished until its pages are put back
                // together, so the count stops one short of the total.
                self.cancel.report_progress(
                    "ocr",
                    self.finished.min(page_count.saturating_sub(1)),
                    Some(page_count),
                );
                self.outcomes.push(outcome);
            }
        }
        let mut outcomes = std::mem::take(&mut self.outcomes);
        outcomes.sort_by_key(|outcome| outcome.page_index);
        outcomes
    }
}

/// Reads one job's pieces, and makes the page image when a whole page reads
/// unconvincingly. Once a scan has failed, or the request is canceled, the
/// rest of the queue is not read.
fn read_job<O: OcrBackend + ?Sized>(
    job: OcrJob,
    ocr: &O,
    cancel: &CancellationToken,
    failed: &AtomicBool,
) -> OcrOutcome {
    let sizes = job
        .pieces
        .iter()
        .map(|piece| piece.image.dimensions())
        .collect::<Vec<_>>();
    let mut readings = Vec::with_capacity(job.pieces.len());
    for piece in &job.pieces {
        let reading = if failed.load(Ordering::SeqCst) || cancel.is_canceled() {
            Err(ExtractionError::canceled())
        } else {
            recognize_timed(ocr, piece, cancel)
        };
        if job.scan && reading.is_err() {
            failed.store(true, Ordering::SeqCst);
        }
        readings.push(reading);
    }
    let vision = match (job.whole_page, readings.first()) {
        (true, Some(Ok(reading))) if reading.mean_confidence < CONFIDENT_READING => {
            let rotation = reading.rotation_degrees;
            job.pieces.first().map(|piece| {
                cancel.timed(
                    |timings| &mut timings.vision_micros,
                    || page_image(job.page_index, &piece.image, rotation),
                )
            })
        }
        _ => None,
    };
    OcrOutcome {
        page_index: job.page_index,
        readings,
        sizes,
        vision,
    }
}

/// Counts a page PDFium rendered towards the request's timings.
fn record_rendered(cancel: &CancellationToken, width: u32, height: u32) {
    cancel.record(|timings| {
        timings.rendered_pixels = timings
            .rendered_pixels
            .saturating_add(u64::from(width) * u64::from(height));
    });
}

/// Reads one page with OCR, counting the page and the time it took. The
/// passes inside it are the OCR backend's to count.
///
/// Pages read in parallel each add their own time, so with more than one
/// OCR worker `ocr_micros` is OCR work done, which can be more than the
/// time the document took.
fn recognize_timed<O: OcrBackend + ?Sized>(
    ocr: &O,
    page: &RenderedPage,
    cancel: &CancellationToken,
) -> Result<OcrResult, ExtractionError> {
    cancel.record(|timings| timings.ocr_pages += 1);
    cancel.timed(
        |timings| &mut timings.ocr_micros,
        || ocr.recognize(page, cancel),
    )
}

/// The long edge of the page image, in pixels.
///
/// Nothing reads the page image's pixels: its presence is what tells the
/// engine a page could not be read. So it is a thumbnail - averaged down
/// before it is turned or encoded, grey, and small enough that making it
/// costs a few milliseconds instead of a full-page resample and encode.
pub const PAGE_IMAGE_LONG_EDGE: u32 = 256;

/// The page image for page `page_index`: a grey PNG thumbnail of the page,
/// turned clockwise by `rotation_degrees`.
pub fn page_image(
    page_index: usize,
    image: &DynamicImage,
    rotation_degrees: u16,
) -> Result<VisionImage, ExtractionError> {
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err(ExtractionError::parse_failed("image has zero dimensions"));
    }
    let scale = (f64::from(PAGE_IMAGE_LONG_EDGE) / f64::from(width.max(height))).min(1.0);
    let thumbnail_width = (f64::from(width) * scale).round().max(1.0) as u32;
    let thumbnail_height = (f64::from(height) * scale).round().max(1.0) as u32;
    let thumbnail = DynamicImage::ImageLuma8(
        image
            .thumbnail_exact(thumbnail_width, thumbnail_height)
            .to_luma8(),
    );
    let upright = apply_detected_rotation(thumbnail, rotation_degrees)?;
    let mut bytes = Vec::new();
    upright
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    Ok(VisionImage {
        page_number: page_index + 1,
        mime_type: "image/png".to_owned(),
        data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
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
    let (bytes, format) = office_source(path, limits, cancel)?;
    let markdown = anydoc::to_markdown_bytes(&bytes, format).map_err(office_error)?;
    cancel.check()?;
    Ok(ExtractedDocument {
        pages: vec![ExtractedPage::of_text(1, markdown, PageSource::AnyDoc)],
        warnings: vec![],
        truncated: false,
        optional_image: None,
        timings: None,
    })
}

/// [`extract_anydoc`]'s parse as anydoc's document model rather than
/// Markdown, for a reader that renders the content itself.
pub(crate) fn anydoc_document(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<anydoc::model::Document, ExtractionError> {
    let (bytes, format) = office_source(path, limits, cancel)?;
    let document = anydoc::to_document(&bytes, format).map_err(office_error)?;
    cancel.check()?;
    Ok(document)
}

/// An Office file read whole, with the parser its content and its extension
/// agree on.
fn office_source(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<(Vec<u8>, anydoc::Format), ExtractionError> {
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
    Ok((bytes, format))
}

fn office_error(error: anydoc::ConvertError) -> ExtractionError {
    match error {
        anydoc::ConvertError::Encrypted => ExtractionError::encrypted(),
        other => ExtractionError::parse_failed(other.to_string()),
    }
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
        pages: vec![ExtractedPage::of_text(1, text, PageSource::Text)],
        warnings,
        truncated,
        optional_image: None,
        timings: None,
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
/// and every frame is a page, read in order up to the document's page
/// limit. A frame this decoder cannot read (CCITT Group 3 compression, say)
/// ends the reading there, and so does the limit: what the caller must not
/// do is believe it received the whole document, so the frames that were
/// not read are reported as truncation.
pub fn extract_image(
    path: &Path,
    ocr: &dyn OcrBackend,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    cancel.check()?;
    let frames = later_frames(path, limits.max_page_count.saturating_sub(1), cancel)?;
    // A TIFF that opens with a thumbnail has its first page further on.
    let image = cancel.timed(
        |timings| &mut timings.image_decode_micros,
        || match frames.first {
            Some(directory) => decode_tiff_frame(path, directory, limits),
            None => load_oriented_image(path, limits),
        },
    )?;
    let page_count = 1 + frames.directories.len();
    let rendered = RenderedPage::new(0, image);
    cancel.report_progress("ocr", 0, Some(page_count));
    let result = recognize_timed(ocr, &rendered, cancel)?;
    let mut low_confidence = result.mean_confidence < CONFIDENT_READING;
    let optional_image = Some(cancel.timed(
        |timings| &mut timings.vision_micros,
        || page_image(0, &rendered.image, result.rotation_degrees),
    )?);
    let size = rendered.image.dimensions();
    drop(rendered);
    // An image file has no physical size to go by; its pixels are taken to
    // be 300 DPI, which is what scanners write.
    // The document's characters, counted down frame by frame as the worker
    // will count them when it sends them, and its layouts (see
    // [`counted`]).
    let mut budget = TextBudget::document();
    let mut page = cancel.timed(
        |timings| &mut timings.analysis_micros,
        || {
            crate::layout::ocr_page(1, result, size, None, RouteSignals::default(), &|| {
                cancel.check().is_err()
            })
        },
    );
    page = counted(page, &mut budget);
    page.vision_escalated = true;
    let mut pages = vec![page];

    let mut truncated = frames.beyond_limit;
    // Each frame is held to the same pixel and decode caps as the first, and
    // read from the file as it is decoded: a batch scan near the source cap
    // is never held in memory whole.
    for (index, directory) in frames.directories.iter().enumerate() {
        cancel.check()?;
        cancel.report_progress("ocr", index + 1, Some(page_count));
        let decoded = cancel.timed(
            |timings| &mut timings.image_decode_micros,
            || decode_tiff_frame(path, *directory, limits),
        );
        let Ok(image) = decoded else {
            truncated = true;
            break;
        };
        let rendered = RenderedPage::new(index + 1, image);
        let result = recognize_timed(ocr, &rendered, cancel)?;
        low_confidence |= result.mean_confidence < CONFIDENT_READING;
        let size = rendered.image.dimensions();
        drop(rendered);
        let page = cancel.timed(
            |timings| &mut timings.analysis_micros,
            || {
                crate::layout::ocr_page(
                    index + 2,
                    result,
                    size,
                    None,
                    RouteSignals::default(),
                    &|| cancel.check().is_err(),
                )
            },
        );
        pages.push(counted(page, &mut budget));
    }
    let mut warnings = Vec::new();
    if low_confidence {
        warnings.push(ExtractionWarning::LowOcrConfidence);
    }
    if truncated {
        warnings.push(ExtractionWarning::TextTruncated);
    }
    link_sections(&mut pages);
    Ok(ExtractedDocument {
        pages,
        warnings,
        truncated,
        optional_image,
        timings: None,
    })
}

/// The frames of an image file after its first: where each one's image file
/// directory is, and whether there were more than the limit allowed. And,
/// when the file's first directory is a reduced-resolution copy, where the
/// first page's own directory is: that page is read in its place.
#[derive(Debug, Default, PartialEq, Eq)]
struct LaterFrames {
    first: Option<u64>,
    directories: Vec<u64>,
    beyond_limit: bool,
}

/// The frames after the first, up to `limit` of them, following the TIFF's
/// chain of image file directories without decoding anything.
///
/// A reduced-resolution copy of a page (`NewSubfileType` bit 0) is not a
/// page of its own and is passed over, the file's first directory included:
/// a file that opens with a thumbnail has its first page where the first
/// full-resolution directory is. Anything that does not read as a
/// TIFF holds one image by construction, and a chain that cannot be
/// followed, or that loops, ends where it stops making sense: the frames
/// read up to there are still the document's pages.
///
/// The walk reads at most [`MAX_TIFF_ENTRIES`] directory entries in all, and
/// stops when the request is canceled: a chain of directories each with a
/// table of tens of thousands of entries reads as truncated where it stops.
fn later_frames(
    path: &Path,
    limit: usize,
    cancel: &CancellationToken,
) -> Result<LaterFrames, ExtractionError> {
    let mut frames = LaterFrames::default();
    let Ok(file) = File::open(path) else {
        return Ok(frames);
    };
    let mut file = BufReader::new(file);
    let Some(format) = TiffFormat::read(&mut file) else {
        return Ok(frames);
    };
    let mut entries_left = MAX_TIFF_ENTRIES;
    let mut seen = std::collections::HashSet::new();
    let mut next = format.first;
    let mut first_page_found = false;
    // A chain can be no longer than the file has room for directories.
    while next != 0 && seen.insert(next) {
        if seen.len() > MAX_TIFF_DIRECTORIES {
            // The chain goes on past what any document this reads could
            // need: whatever its rest holds goes unread, so say so.
            frames.beyond_limit = true;
            break;
        }
        cancel.check()?;
        let directory = match format.directory(&mut file, next, &mut entries_left) {
            DirectoryRead::Read(directory) => directory,
            DirectoryRead::Unreadable => break,
            DirectoryRead::PastBudget => {
                frames.beyond_limit = true;
                break;
            }
        };
        if !directory.reduced_resolution {
            if !first_page_found {
                first_page_found = true;
                if next != format.first {
                    frames.first = Some(next);
                }
            } else {
                if frames.directories.len() == limit {
                    frames.beyond_limit = true;
                    break;
                }
                frames.directories.push(next);
            }
        }
        next = directory.next;
    }
    Ok(frames)
}

/// More directories than any document this reads could be pages of: the
/// page limit, and then some for reduced-resolution copies.
const MAX_TIFF_DIRECTORIES: usize = 4_096;

/// The most directory entries a TIFF's walk reads in all. A directory
/// has a few dozen; a table may say it has 65,535, and every directory of
/// a chain may say so.
const MAX_TIFF_ENTRIES: u64 = 262_144;

/// What reading one image file directory came to.
enum DirectoryRead {
    Read(TiffDirectory),
    /// Not a directory this can read: the chain ends there.
    Unreadable,
    /// More entries than the walk has left to read.
    PastBudget,
}

/// How a TIFF counts: its byte order, classic or BigTIFF, and where its
/// first image file directory is.
struct TiffFormat {
    big_endian: bool,
    big: bool,
    first: u64,
}

/// What a frame's directory says that matters here.
struct TiffDirectory {
    reduced_resolution: bool,
    next: u64,
}

const NEW_SUBFILE_TYPE: u16 = 254;

impl TiffFormat {
    fn read(file: &mut (impl Read + Seek)) -> Option<Self> {
        let mut header = [0_u8; 8];
        file.read_exact(&mut header).ok()?;
        let big_endian = match &header[..2] {
            b"II" => false,
            b"MM" => true,
            _ => return None,
        };
        let mut format = Self {
            big_endian,
            big: false,
            first: 0,
        };
        // Classic TIFF marks itself 42 and counts in 32-bit words; BigTIFF
        // marks itself 43 and counts in 64-bit ones, with wider entries.
        match format.word([header[2], header[3]]) {
            42 => format.first = u64::from(format.long(header[4..8].try_into().ok()?)),
            43 => {
                format.big = true;
                let mut offset = [0_u8; 8];
                file.read_exact(&mut offset).ok()?;
                format.first = format.quad(offset);
            }
            _ => return None,
        }
        Some(format)
    }

    fn word(&self, bytes: [u8; 2]) -> u16 {
        if self.big_endian {
            u16::from_be_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        }
    }

    fn long(&self, bytes: [u8; 4]) -> u32 {
        if self.big_endian {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        }
    }

    fn quad(&self, bytes: [u8; 8]) -> u64 {
        if self.big_endian {
            u64::from_be_bytes(bytes)
        } else {
            u64::from_le_bytes(bytes)
        }
    }

    /// Reads the directory at `offset`, counting its entries against
    /// `entries_left`.
    fn directory(
        &self,
        file: &mut (impl Read + Seek),
        offset: u64,
        entries_left: &mut u64,
    ) -> DirectoryRead {
        let Some(entries) = self.entry_count(file, offset) else {
            return DirectoryRead::Unreadable;
        };
        if entries > *entries_left {
            return DirectoryRead::PastBudget;
        }
        *entries_left -= entries;
        match self.entries(file, entries) {
            Some(directory) => DirectoryRead::Read(directory),
            None => DirectoryRead::Unreadable,
        }
    }

    fn entry_count(&self, file: &mut (impl Read + Seek), offset: u64) -> Option<u64> {
        file.seek(SeekFrom::Start(offset)).ok()?;
        let entries = if self.big {
            let mut count = [0_u8; 8];
            file.read_exact(&mut count).ok()?;
            self.quad(count)
        } else {
            let mut count = [0_u8; 2];
            file.read_exact(&mut count).ok()?;
            u64::from(self.word(count))
        };
        (entries <= u64::from(u16::MAX)).then_some(entries)
    }

    /// A directory's `entries` entries and the offset after them, read from
    /// where its count ends.
    fn entries(&self, file: &mut impl Read, entries: u64) -> Option<TiffDirectory> {
        let entry_bytes = if self.big { 20 } else { 12 };
        let value_at = if self.big { 12 } else { 8 };
        let mut reduced_resolution = false;
        let mut entry = [0_u8; 20];
        for _ in 0..entries {
            file.read_exact(&mut entry[..entry_bytes]).ok()?;
            if self.word([entry[0], entry[1]]) != NEW_SUBFILE_TYPE {
                continue;
            }
            // A LONG by the specification; a SHORT from some writers. Either
            // sits at the start of the entry's value field.
            let value = match self.word([entry[2], entry[3]]) {
                3 => u32::from(self.word([entry[value_at], entry[value_at + 1]])),
                _ => self.long(entry[value_at..value_at + 4].try_into().ok()?),
            };
            reduced_resolution = value & 1 == 1;
        }
        let next = if self.big {
            let mut next = [0_u8; 8];
            file.read_exact(&mut next).ok()?;
            self.quad(next)
        } else {
            let mut next = [0_u8; 4];
            file.read_exact(&mut next).ok()?;
            u64::from(self.long(next))
        };
        Some(TiffDirectory {
            reduced_resolution,
            next,
        })
    }
}

/// Decodes the TIFF frame whose image file directory is at `directory`.
///
/// Every offset in a TIFF is from the start of the file, so pointing the
/// header at another directory makes that frame the file's first image,
/// and the decoder reads it exactly as it reads a first frame - colour,
/// bit depth, compression and orientation included. The header is pointed
/// there as the file is read, never on disk.
fn decode_tiff_frame(
    path: &Path,
    directory: u64,
    limits: &ResourceLimits,
) -> Result<DynamicImage, ExtractionError> {
    let mut file = File::open(path).map_err(ExtractionError::io)?;
    let format =
        TiffFormat::read(&mut file).ok_or_else(|| ExtractionError::parse_failed("not a TIFF"))?;
    file.seek(SeekFrom::Start(0)).map_err(ExtractionError::io)?;
    let mut header = [0_u8; 16];
    let length = if format.big { 16 } else { 8 };
    file.read_exact(&mut header[..length])
        .map_err(ExtractionError::io)?;
    let (start, pointer) = if format.big {
        (
            8,
            if format.big_endian {
                directory.to_be_bytes().to_vec()
            } else {
                directory.to_le_bytes().to_vec()
            },
        )
    } else {
        let directory = u32::try_from(directory)
            .map_err(|_| ExtractionError::parse_failed("TIFF directory out of range"))?;
        (
            4,
            if format.big_endian {
                directory.to_be_bytes().to_vec()
            } else {
                directory.to_le_bytes().to_vec()
            },
        )
    };
    header[start..start + pointer.len()].copy_from_slice(&pointer);
    file.seek(SeekFrom::Start(0)).map_err(ExtractionError::io)?;
    let reader = PatchedHeader {
        file,
        header,
        length: length as u64,
        position: 0,
    };
    ImageReader::with_format(BufReader::new(reader), ImageFormat::Tiff)
        .into_decoder()
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))
        .and_then(|decoder| decode_oriented(decoder, limits))
}

/// A file read with its first `length` bytes replaced by `header`.
struct PatchedHeader {
    file: File,
    header: [u8; 16],
    length: u64,
    position: u64,
}

impl Read for PatchedHeader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.file.read(buffer)?;
        for (offset, byte) in buffer[..read].iter_mut().enumerate() {
            let at = self.position + offset as u64;
            if at >= self.length {
                break;
            }
            *byte = self.header[at as usize];
        }
        self.position += read as u64;
        Ok(read)
    }
}

impl Seek for PatchedHeader {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.position = self.file.seek(position)?;
        Ok(self.position)
    }
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
    let decoder = ImageReader::open(path)
        .map_err(ExtractionError::io)?
        .with_guessed_format()
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?
        .into_decoder()
        .map_err(|error| ExtractionError::parse_failed(error.to_string()))?;
    decode_oriented(decoder, limits)
}

/// Decodes an image the right way up within the caps a rendered page is held
/// to: its encoded size and decode buffer first, then the page's pixels.
fn decode_oriented(
    mut decoder: impl ImageDecoder,
    limits: &ResourceLimits,
) -> Result<DynamicImage, ExtractionError> {
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

#[cfg(test)]
mod tiff_chains {
    use super::*;

    /// A little-endian TIFF of image file directories only, each saying
    /// whether it is a reduced-resolution copy, chained in order; the last
    /// points at `last_next` (0 ends the chain).
    fn chain(reduced: &[bool], last_next: Option<usize>) -> tempfile::NamedTempFile {
        const DIRECTORY: usize = 2 + 12 + 4;
        let at = |index: usize| (8 + index * DIRECTORY) as u32;
        let mut bytes = b"II".to_vec();
        bytes.extend(42_u16.to_le_bytes());
        bytes.extend(at(0).to_le_bytes());
        for (index, copy) in reduced.iter().enumerate() {
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(NEW_SUBFILE_TYPE.to_le_bytes());
            bytes.extend(4_u16.to_le_bytes());
            bytes.extend(1_u32.to_le_bytes());
            bytes.extend(u32::from(*copy).to_le_bytes());
            let next = if index + 1 < reduced.len() {
                at(index + 1)
            } else {
                last_next.map_or(0, at)
            };
            bytes.extend(next.to_le_bytes());
        }
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), bytes).unwrap();
        file
    }

    #[test]
    fn a_chain_cut_short_by_the_directory_bound_is_marked_as_going_on() {
        // A page, then more reduced-resolution copies than the bound
        // allows, then a page the walk never reaches.
        let mut reduced = vec![false];
        reduced.extend(std::iter::repeat_n(true, MAX_TIFF_DIRECTORIES + 10));
        reduced.push(false);
        let file = chain(&reduced, None);
        let frames = later_frames(file.path(), 10, &CancellationToken::new()).unwrap();
        assert!(frames.directories.is_empty());
        assert!(frames.beyond_limit, "pages may lie past the bound");
    }

    /// A file that opens with a thumbnail of its first page: the thumbnail
    /// is passed over, the page after it is read in its place, and is not
    /// read again as a later page.
    #[test]
    fn a_thumbnail_first_is_passed_over_for_the_page_after_it() {
        const DIRECTORY: u64 = 2 + 12 + 4;
        let file = chain(&[true, false, false], None);
        let frames = later_frames(file.path(), 10, &CancellationToken::new()).unwrap();
        assert_eq!(frames.first, Some(8 + DIRECTORY));
        assert_eq!(frames.directories, vec![8 + 2 * DIRECTORY]);

        let plain = chain(&[false, true, false], None);
        let frames = later_frames(plain.path(), 10, &CancellationToken::new()).unwrap();
        assert_eq!(frames.first, None, "the first directory is the first page");
        assert_eq!(frames.directories, vec![8 + 2 * DIRECTORY]);

        // Thumbnails only: the first directory is still read, as before.
        let thumbnails = chain(&[true, true], None);
        let frames = later_frames(thumbnails.path(), 10, &CancellationToken::new()).unwrap();
        assert_eq!(frames.first, None);
        assert!(frames.directories.is_empty());
    }

    /// A chain of directories whose tables say they have more entries than
    /// the walk reads in all stops where the budget runs out, and reads as
    /// going on.
    #[test]
    fn a_chain_of_huge_entry_tables_stops_at_the_entry_budget() {
        // A page, then directories that each claim the most entries a
        // table may, all pointing at the same table.
        let mut bytes = b"II".to_vec();
        bytes.extend(42_u16.to_le_bytes());
        bytes.extend(8_u32.to_le_bytes());
        let directory = |bytes: &mut Vec<u8>, count: u16, next: u32| {
            bytes.extend(count.to_le_bytes());
            bytes.extend(std::iter::repeat_n(0_u8, 12 * usize::from(count)));
            bytes.extend(next.to_le_bytes());
        };
        let huge_at = (8 + 2 + 4) as u32;
        directory(&mut bytes, 0, huge_at);
        let huge_end = huge_at as usize + 2 + 12 * usize::from(u16::MAX);
        let after = (huge_end + 4) as u32;
        directory(&mut bytes, u16::MAX, after);
        // Each next directory a fresh table, so the chain does not loop.
        for _ in 0..8 {
            let next = bytes.len() as u32 + 2 + 12 * u32::from(u16::MAX) + 4;
            directory(&mut bytes, u16::MAX, next);
        }
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), bytes).unwrap();

        let frames = later_frames(file.path(), 100, &CancellationToken::new()).unwrap();
        assert!(frames.beyond_limit, "pages may lie past the budget");

        let canceled = CancellationToken::new();
        canceled.cancel();
        assert!(later_frames(file.path(), 100, &canceled).is_err());
    }

    #[test]
    fn a_whole_chain_or_a_looping_one_is_not_marked_as_going_on() {
        let pages = chain(&[false, false, true, false], None);
        let frames = later_frames(pages.path(), 10, &CancellationToken::new()).unwrap();
        assert_eq!(frames.directories.len(), 2);
        assert!(!frames.beyond_limit);

        let looping = chain(&[false, false, false], Some(0));
        let frames = later_frames(looping.path(), 10, &CancellationToken::new()).unwrap();
        assert_eq!(frames.directories.len(), 2);
        assert!(
            !frames.beyond_limit,
            "a loop ends the chain; nothing is past it"
        );
    }
}

#[cfg(test)]
mod text_page_layouts {
    use super::*;

    #[test]
    fn a_page_the_worker_will_cut_gets_no_layout() {
        let short = ExtractedPage::of_text(1, "Notice of Termination".into(), PageSource::Text);
        assert!(short.layout.is_some());

        let long = "word ".repeat(MAX_PAGE_CHARS / 5 + 1);
        let page = ExtractedPage::of_text(1, long, PageSource::Text);
        assert!(page.layout.is_none(), "the text is cut on the way out");
    }

    #[test]
    fn pages_past_the_documents_characters_get_no_layout() {
        let mut budget = TextBudget::document();
        let sheet = "cell ".repeat(MAX_PAGE_CHARS / 5);
        let pages = (1..=6)
            .map(|number| {
                ExtractedPage::of_text_within(
                    number,
                    sheet.clone(),
                    PageSource::AnyDoc,
                    &mut budget,
                )
            })
            .collect::<Vec<_>>();
        let with_layouts = pages.iter().filter(|page| page.layout.is_some()).count();
        assert_eq!(with_layouts, MAX_DOCUMENT_CHARS / MAX_PAGE_CHARS);
        assert_eq!(budget.characters, 0);
    }

    /// A page of one-letter lines is well inside the characters, but its
    /// layout would hold an object for every line: past what a page's
    /// layout may hold, it goes without one, its text whole.
    #[test]
    fn a_page_of_more_lines_than_a_layout_may_hold_gets_no_layout() {
        let lines = "A\n\n".repeat(MAX_PAGE_LAYOUT_PARTS / 2);
        let page = ExtractedPage::of_text(1, lines.clone(), PageSource::Text);
        assert!(page.layout.is_none());
        assert_eq!(page.text, lines);

        let pipes = "|".repeat(MAX_PAGE_LAYOUT_PARTS);
        let page = ExtractedPage::of_text(1, pipes, PageSource::Text);
        assert!(page.layout.is_none(), "every pipe could be a cell");

        let fits = "A\n".repeat(MAX_PAGE_LAYOUT_PARTS / 2);
        assert!(
            ExtractedPage::of_text(1, fits, PageSource::Text)
                .layout
                .is_some()
        );
    }

    /// An OCR reading whose lines were not analysed, with more lines of
    /// text than a page's layout may hold, goes without a layout and keeps
    /// its text.
    #[test]
    fn an_ocr_reading_too_dense_for_a_layout_keeps_its_text_and_no_layout() {
        let dense = "A\n\n".repeat(MAX_PAGE_LAYOUT_PARTS / 2);
        let page = crate::layout::ocr_page(
            1,
            OcrResult::new(dense.clone(), 90.0),
            (100, 100),
            None,
            RouteSignals::default(),
            &|| false,
        );
        assert!(page.layout.is_none());
        assert_eq!(page.text, dense);

        let page = crate::layout::ocr_page(
            1,
            OcrResult::new("RECEIPT\nTotal 4.00", 90.0),
            (100, 100),
            None,
            RouteSignals::default(),
            &|| false,
        );
        assert!(page.layout.is_some());
    }

    /// Lines and cells are counted across the document in page order: once
    /// the pages before have taken what the document may hold, a page that
    /// would fit on its own goes without a layout, and its characters are
    /// still counted.
    #[test]
    fn pages_past_the_documents_lines_and_cells_get_no_layout() {
        let mut budget = TextBudget::document();
        let page_text = "A\n".repeat(MAX_PAGE_LAYOUT_PARTS - 1);
        let per_page = layout_parts(&page_text);
        assert!(per_page <= MAX_PAGE_LAYOUT_PARTS);
        let pages = (1..=MAX_DOCUMENT_LAYOUT_PARTS / per_page + 2)
            .map(|number| {
                ExtractedPage::of_text_within(
                    number,
                    page_text.clone(),
                    PageSource::Text,
                    &mut budget,
                )
            })
            .collect::<Vec<_>>();
        let with_layouts = pages
            .iter()
            .take_while(|page| page.layout.is_some())
            .count();
        assert_eq!(with_layouts, MAX_DOCUMENT_LAYOUT_PARTS / per_page);
        assert!(
            pages[with_layouts..]
                .iter()
                .all(|page| page.layout.is_none())
        );
        assert_eq!(
            budget.characters,
            MAX_DOCUMENT_CHARS - pages.len() * page_text.chars().count()
        );
    }
}
