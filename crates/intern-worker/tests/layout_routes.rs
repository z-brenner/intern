//! How a PDF's pages are routed and what each route makes of them, with
//! stand-in backends: the page text a route produces, the layout beside it,
//! and the OCR pool that reads scanned pages side by side.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use image::{DynamicImage, RgbImage};
use intern_worker::extract::{
    CancellationToken, ExtractedDocument, ExtractionError, ExtractionWarning, OcrBackend, OcrLine,
    OcrResult, PageSource, PdfBackend, PdfPageInspection, RenderedPage, extract_pdf,
};
use intern_worker::layout::{BlockKind, NativePage, PageRoute, TextRun, TextSource, linearize};
use intern_worker::limits::ResourceLimits;

/// A PDF whose pages are the inspections it is given. Renders are blank
/// pages at a tenth of the size inspection reports, which is all OCR
/// stand-ins need.
struct StandInPdf {
    pages: Vec<PdfPageInspection>,
    renders: AtomicUsize,
}

impl StandInPdf {
    fn new(pages: Vec<PdfPageInspection>) -> Self {
        Self {
            pages,
            renders: AtomicUsize::new(0),
        }
    }
}

impl PdfBackend for StandInPdf {
    fn inspect(
        &self,
        _path: &Path,
        _cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        Ok(self.pages.clone())
    }

    fn render_within(
        &self,
        _path: &Path,
        page_index: usize,
        _max_pixels: u64,
        _cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        self.renders.fetch_add(1, Ordering::SeqCst);
        let page = &self.pages[page_index];
        Ok(RenderedPage::new(
            page_index,
            DynamicImage::ImageRgb8(RgbImage::new(page.width_pixels, page.height_pixels)),
        ))
    }
}

/// A run at `x`, `y` points (top of the line), `size` points tall.
fn run(x: u32, y: u32, text: &str, size: u32) -> TextRun {
    let width = text.chars().count() as u32 * size / 2;
    TextRun {
        text: text.to_owned(),
        bbox: [x * 10, y * 10, (x + width) * 10, (y + size) * 10],
        bold: false,
        confidence: None,
    }
}

/// A letter-size native page with the given runs, its segments the runs'
/// boxes.
fn native_page(page_index: usize, text: &str, runs: Vec<TextRun>) -> PdfPageInspection {
    PdfPageInspection {
        page_index,
        native_text: text.to_owned(),
        image_coverage: 0.0,
        width_pixels: 255,
        height_pixels: 330,
        native: Some(NativePage {
            width: 6120,
            height: 7920,
            segments: runs.iter().map(|run| run.bbox).collect(),
            runs,
            ..NativePage::default()
        }),
        signals: None,
    }
}

fn scanned_page(page_index: usize) -> PdfPageInspection {
    PdfPageInspection {
        page_index,
        native_text: String::new(),
        image_coverage: 1.0,
        width_pixels: 255,
        height_pixels: 330,
        ..PdfPageInspection::default()
    }
}

fn read(pdf: &StandInPdf, ocr: &(impl OcrBackend + Sync)) -> ExtractedDocument {
    read_with(pdf, ocr)
}

fn read_with(pdf: &dyn PdfBackend, ocr: &(impl OcrBackend + Sync)) -> ExtractedDocument {
    extract_pdf(
        Path::new("document.pdf"),
        pdf,
        ocr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap()
}

/// No OCR is ever needed.
struct NoOcr;

impl OcrBackend for NoOcr {
    fn recognize(
        &self,
        _page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        panic!("a text page was sent to OCR")
    }
}

/// A two-column page written row by row: PDFium's text runs the columns
/// together, line by line.
fn interleaved_columns() -> PdfPageInspection {
    let left = [
        "The tenant leases the premises for",
        "the term and pays the base rent in",
        "monthly installments when they fall",
    ];
    let right = [
        "The landlord keeps the common areas",
        "in good repair and insures building",
        "at full replacement cost each year.",
    ];
    let mut runs = Vec::new();
    let mut text = Vec::new();
    for (row, (l, r)) in left.iter().zip(right).enumerate() {
        let y = 100 + row as u32 * 12;
        runs.push(run(54, y, l, 9));
        runs.push(run(320, y, r, 9));
        text.push(format!("{l} {r}"));
    }
    native_page(0, &text.join("\r\n"), runs)
}

#[test]
fn a_plain_page_keeps_its_text_exactly_and_gets_blocks() {
    let text = "NOTICE OF TERMINATION\r\nThe agreement ends on May 1, 2026.\r\nDate: April 2, 2026";
    let runs = vec![
        run(54, 60, "NOTICE OF TERMINATION", 14),
        run(54, 90, "The agreement ends on May 1, 2026.", 9),
        run(54, 102, "Date: April 2, 2026", 9),
    ];
    let pdf = StandInPdf::new(vec![native_page(0, text, runs)]);

    let document = read(&pdf, &NoOcr);

    let page = &document.pages[0];
    assert_eq!(page.text, text, "the fast route never touches the text");
    let layout = page.layout.as_ref().unwrap();
    assert_eq!(layout.route, PageRoute::Fast);
    assert_eq!((layout.width, layout.height), (6120, 7920));
    let kinds = layout
        .blocks
        .iter()
        .map(|block| block.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            BlockKind::Heading,
            BlockKind::Paragraph,
            BlockKind::KeyValue
        ]
    );
    assert_eq!(layout.blocks[0].id, "p1.b1");
    assert!(
        layout.blocks[0].bbox.is_some(),
        "segments give lines their boxes"
    );
    assert_eq!(layout.blocks[2].section.as_deref(), Some("p1.b1"));
    assert_eq!(pdf.renders.load(Ordering::SeqCst), 0);
}

#[test]
fn columns_written_row_by_row_are_read_column_by_column() {
    let pdf = StandInPdf::new(vec![interleaved_columns()]);

    let document = read(&pdf, &NoOcr);

    let page = &document.pages[0];
    let layout = page.layout.as_ref().unwrap();
    assert_eq!(layout.route, PageRoute::Layout);
    assert!(layout.signals.interleave >= 500, "{:?}", layout.signals);
    assert_eq!(
        page.text,
        "The tenant leases the premises for\nthe term and pays the base rent in\n\
         monthly installments when they fall\n\nThe landlord keeps the common areas\n\
         in good repair and insures building\nat full replacement cost each year."
    );
    assert_eq!(page.text, linearize(&layout.blocks));
    assert_eq!(page.source, PageSource::Native);
}

/// A PDF whose inspection carries no characters: the runs of a page are
/// handed over only when the page asks for them, and the asking counted.
struct RunsOnRequest {
    pages: Vec<PdfPageInspection>,
    runs: Vec<Vec<TextRun>>,
    asked: Mutex<Vec<usize>>,
}

impl RunsOnRequest {
    fn new(mut pages: Vec<PdfPageInspection>) -> Self {
        let runs = pages
            .iter_mut()
            .map(|page| std::mem::take(&mut page.native.as_mut().unwrap().runs))
            .collect();
        Self {
            pages,
            runs,
            asked: Mutex::new(Vec::new()),
        }
    }
}

impl PdfBackend for RunsOnRequest {
    fn inspect(
        &self,
        _path: &Path,
        _cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        Ok(self.pages.clone())
    }

    fn render_within(
        &self,
        _path: &Path,
        _page_index: usize,
        _max_pixels: u64,
        _cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        panic!("a text page was rendered")
    }

    fn page_runs(
        &self,
        _path: &Path,
        page_index: usize,
        _native: &NativePage,
        _cancel: &CancellationToken,
    ) -> Result<Vec<TextRun>, ExtractionError> {
        self.asked.lock().unwrap().push(page_index);
        Ok(self.runs[page_index].clone())
    }
}

/// A page's characters are read when the page is, and only for a page
/// whose geometry is read: inspection holds none of them.
#[test]
fn a_page_s_characters_are_read_when_the_page_is_and_only_if_it_needs_them() {
    let plain = native_page(
        1,
        "Plain text page.\r\nSecond line.",
        vec![
            run(54, 60, "Plain text page.", 9),
            run(54, 72, "Second line.", 9),
        ],
    );
    let pdf = RunsOnRequest::new(vec![interleaved_columns(), plain]);

    let document = read_with(&pdf, &NoOcr);

    assert_eq!(*pdf.asked.lock().unwrap(), [0], "only the layout page");
    let layout = document.pages[0].layout.as_ref().unwrap();
    assert_eq!(layout.route, PageRoute::Layout);
    assert!(
        document.pages[0]
            .text
            .starts_with("The tenant leases the premises for\nthe term"),
        "{}",
        document.pages[0].text
    );
    assert_eq!(document.pages[1].text, "Plain text page.\r\nSecond line.");
}

/// OCR that takes longer on early pages than late ones, so that with
/// several workers the readings come back out of order.
struct UnevenOcr {
    workers: usize,
    calls: AtomicUsize,
    largest_batch: AtomicUsize,
    in_flight: AtomicUsize,
}

impl UnevenOcr {
    fn new(workers: usize) -> Self {
        Self {
            workers,
            calls: AtomicUsize::new(0),
            largest_batch: AtomicUsize::new(0),
            in_flight: AtomicUsize::new(0),
        }
    }
}

impl OcrBackend for UnevenOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.largest_batch.fetch_max(now, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(40 - 5 * page.page_index as u64));
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        // Pages 2 and 4 read unconvincingly; page 2 is the first.
        let confidence = if matches!(page.page_index, 1 | 3) {
            60.0
        } else {
            93.0
        };
        Ok(
            OcrResult::new(format!("SCANNED PAGE {}", page.page_index + 1), confidence).with_lines(
                vec![OcrLine {
                    text: format!("SCANNED PAGE {}", page.page_index + 1),
                    bbox: [20, 30, 200, 60],
                    confidence: confidence as u8,
                }],
            ),
        )
    }

    fn concurrency(&self) -> usize {
        self.workers
    }
}

#[test]
fn scanned_pages_are_read_side_by_side_and_put_back_in_order() {
    let pages = (0..6).map(scanned_page).collect::<Vec<_>>();
    let pdf = StandInPdf::new(pages.clone());
    let ocr = UnevenOcr::new(3);

    let document = read(&pdf, &ocr);

    assert_eq!(ocr.calls.load(Ordering::SeqCst), 6);
    assert!(
        ocr.largest_batch.load(Ordering::SeqCst) >= 2,
        "pages were read side by side"
    );
    assert!(ocr.largest_batch.load(Ordering::SeqCst) <= 3);
    let texts = document
        .pages
        .iter()
        .map(|page| page.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        texts,
        [
            "SCANNED PAGE 1",
            "SCANNED PAGE 2",
            "SCANNED PAGE 3",
            "SCANNED PAGE 4",
            "SCANNED PAGE 5",
            "SCANNED PAGE 6"
        ]
    );
    // The page image is the first unconvincing page, as it always was.
    assert_eq!(document.optional_image.as_ref().unwrap().page_number, 2);
    let escalated = document
        .pages
        .iter()
        .filter(|page| page.vision_escalated)
        .map(|page| page.page_number)
        .collect::<Vec<_>>();
    assert_eq!(escalated, [2]);
    assert_eq!(document.warnings, [ExtractionWarning::LowOcrConfidence]);
    // Every OCR page has blocks from its lines, marked as OCR.
    let layout = document.pages[0].layout.as_ref().unwrap();
    assert_eq!(layout.route, PageRoute::Ocr);
    assert_eq!(layout.blocks[0].source, TextSource::Ocr);
    assert_eq!(layout.blocks[0].confidence, Some(93));
    // Read again one worker at a time, the document is the same.
    let sequential = read(&StandInPdf::new(pages), &UnevenOcr::new(1));
    assert_eq!(sequential, document);
}

/// A scanned PDF whose renders and readings are counted together, so the
/// pages held at once - rendered and not yet read - can be seen.
struct CountedScans {
    pages: Vec<PdfPageInspection>,
    rendered: Arc<AtomicUsize>,
    read: Arc<AtomicUsize>,
    most_held: AtomicUsize,
}

impl PdfBackend for CountedScans {
    fn inspect(
        &self,
        _path: &Path,
        _cancel: &CancellationToken,
    ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
        Ok(self.pages.clone())
    }

    fn render_within(
        &self,
        _path: &Path,
        page_index: usize,
        _max_pixels: u64,
        _cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        let rendered = self.rendered.fetch_add(1, Ordering::SeqCst) + 1;
        let held = rendered - self.read.load(Ordering::SeqCst);
        self.most_held.fetch_max(held, Ordering::SeqCst);
        let page = &self.pages[page_index];
        Ok(RenderedPage::new(
            page_index,
            DynamicImage::ImageRgb8(RgbImage::new(page.width_pixels, page.height_pixels)),
        ))
    }
}

/// Slow OCR on three workers that counts the pages it has finished.
struct CountingSlowOcr(Arc<AtomicUsize>);

impl OcrBackend for CountingSlowOcr {
    fn recognize(
        &self,
        _page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        std::thread::sleep(Duration::from_millis(30));
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(OcrResult::new("PAGE", 95.0))
    }

    fn concurrency(&self) -> usize {
        3
    }
}

/// However slow OCR is, renders run no further ahead of it than the queue
/// allows: the pages held at once are the one being rendered, those
/// waiting, and one per worker.
#[test]
fn rendered_pages_held_at_once_are_bounded_by_the_workers_and_the_queue() {
    let finished = Arc::new(AtomicUsize::new(0));
    let pdf = CountedScans {
        pages: (0..12).map(scanned_page).collect(),
        rendered: Arc::new(AtomicUsize::new(0)),
        read: Arc::clone(&finished),
        most_held: AtomicUsize::new(0),
    };

    let document = read_with(&pdf, &CountingSlowOcr(finished));

    assert_eq!(document.pages.len(), 12);
    let limits = ResourceLimits::default();
    let bound = 1 + limits.max_queued_rendered_pages + 3;
    let most_held = pdf.most_held.load(Ordering::SeqCst);
    assert!(most_held <= bound, "{most_held} pages held, bound {bound}");
    assert!(most_held >= 2, "pages were read side by side");
}

/// OCR that cannot run at all.
struct MissingOcr;

impl OcrBackend for MissingOcr {
    fn recognize(
        &self,
        _page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        Err(ExtractionError::native_assets_missing("no OCR engine here"))
    }

    fn concurrency(&self) -> usize {
        2
    }
}

#[test]
fn a_scan_that_cannot_be_read_fails_the_document_with_its_own_error() {
    let pdf = StandInPdf::new(vec![
        native_page(0, "Cover letter.", vec![run(54, 60, "Cover letter.", 9)]),
        scanned_page(1),
        scanned_page(2),
    ]);

    let error = extract_pdf(
        Path::new("document.pdf"),
        &pdf,
        &MissingOcr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap_err();

    assert_eq!(error.code(), "NATIVE_ASSETS_MISSING");
}

/// A text layer that is somebody else's bad OCR, over a full-page image.
fn bad_prior_ocr() -> PdfPageInspection {
    // Long enough not to be taken for a stamp on a scan.
    let text = "Wexcornbe Mi11work Co. INV0ICE lnvoice Date: O3/lO/2O26 Bill To: 0strander \
                Hornebuilders LLC, 12OO Ca1der Way T0TAL DUE $6,649.43 Custorn white oak \
                stair treads, 42 in. - 14 @ $186.50 = $2611.OO Handrail, 16 ft, with \
                brackets - 2 @ $314.OO = $628.00 Terms: net 30 days, payab1e to Wexcombe";
    let mut page = native_page(0, text, vec![run(54, 60, text, 9)]);
    page.image_coverage = 1.0;
    let native = page.native.as_mut().unwrap();
    native.text_objects = 10;
    native.invisible_text_objects = 10;
    page
}

struct CleanReading(f32);

impl OcrBackend for CleanReading {
    fn recognize(
        &self,
        _page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        Ok(OcrResult::new(
            "Wexcombe Millwork Co. INVOICE Invoice Date: 03/10/2026 TOTAL DUE $6,649.43",
            self.0,
        ))
    }
}

#[test]
fn a_bad_prior_ocr_layer_is_read_again_and_replaced_only_when_the_reading_is_better() {
    let pdf = StandInPdf::new(vec![bad_prior_ocr()]);

    let document = read(&pdf, &CleanReading(91.0));

    let page = &document.pages[0];
    assert_eq!(page.source, PageSource::Ocr);
    assert!(page.text.contains("Invoice Date: 03/10/2026"));
    assert_eq!(page.layout.as_ref().unwrap().route, PageRoute::Ocr);
    assert!(page.layout.as_ref().unwrap().signals.garbage >= 40);

    // OCR that is no surer of the page, and no cleaner, leaves the layer.
    struct NoBetter;
    impl OcrBackend for NoBetter {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, ExtractionError> {
            Ok(OcrResult::new("W3xc0mbe M1llw0rk C0. 1NV01CE", 40.0))
        }
    }
    let kept = read(&StandInPdf::new(vec![bad_prior_ocr()]), &NoBetter);
    assert_eq!(kept.pages[0].source, PageSource::Native);
    assert_eq!(kept.pages[0].text, bad_prior_ocr().native_text);

    // And OCR that cannot run leaves it too, rather than failing a page
    // that had text.
    let unread = read(&StandInPdf::new(vec![bad_prior_ocr()]), &MissingOcr);
    assert_eq!(unread.pages[0].text, bad_prior_ocr().native_text);
    assert!(unread.warnings.is_empty());
}

/// A page whose text layer is one bad-OCR token over an image covering
/// just under nine tenths of it: read again, and - with so little text -
/// also a page that wants the page image.
fn bad_layer_that_wants_the_page_image() -> PdfPageInspection {
    let text = "INV0ICE2O26Ca1derT0TALWexcornbeMi11work";
    let mut page = native_page(0, text, vec![run(54, 60, text, 9)]);
    page.image_coverage = 0.8996;
    let native = page.native.as_mut().unwrap();
    native.text_objects = 10;
    native.invisible_text_objects = 10;
    page
}

/// A page read again does not take the page image for certain: when the
/// fresh reading wins, the image made from its text layer goes, and the
/// next page that wants one must have been rendered for it.
#[test]
fn a_page_read_again_does_not_keep_later_pages_from_the_page_image() {
    let mut second = native_page(1, &"a".repeat(99), vec![run(54, 60, "aaaa", 9)]);
    second.image_coverage = 0.65;
    let pdf = StandInPdf::new(vec![bad_layer_that_wants_the_page_image(), second]);

    let document = read(&pdf, &CleanReading(91.0));

    assert_eq!(document.pages[0].source, PageSource::Ocr, "read again");
    assert_eq!(
        document
            .optional_image
            .as_ref()
            .map(|image| image.page_number),
        Some(2)
    );
    assert!(document.pages[1].vision_escalated);
}

/// A page of native text with a pasted image across its lower half, which
/// holds the signature date.
fn page_with_an_image_region() -> PdfPageInspection {
    let runs = vec![
        run(54, 60, "SECOND AMENDMENT TO LEASE", 14),
        run(54, 90, "The parties amend the lease as follows.", 9),
    ];
    let mut page = native_page(
        0,
        "SECOND AMENDMENT TO LEASE\r\nThe parties amend the lease as follows.",
        runs,
    );
    // Rendered at 300 DPI, so OCR's pixels are the size they would be.
    page.width_pixels = 2550;
    page.height_pixels = 3300;
    page.image_coverage = 0.4;
    page.native.as_mut().unwrap().images = vec![[540, 4000, 5580, 7000]];
    page
}

/// Reads whatever it is given as the signature block, at a confidence.
struct SignatureReader(f32);

impl OcrBackend for SignatureReader {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        // The region is cropped out of the render: it is the region's size,
        // not the page's.
        assert!(page.image.width() < 2550 && page.image.height() < 1650);
        Ok(
            OcrResult::new("Signed April 30, 2026", self.0).with_lines(vec![OcrLine {
                text: "Signed April 30, 2026".to_owned(),
                bbox: [100, 100, 1500, 150],
                confidence: self.0 as u8,
            }]),
        )
    }
}

#[test]
fn an_image_region_on_a_text_page_is_read_and_merged_in_reading_order() {
    let pdf = StandInPdf::new(vec![page_with_an_image_region()]);

    let document = read(&pdf, &SignatureReader(88.0));

    let page = &document.pages[0];
    let layout = page.layout.as_ref().unwrap();
    assert_eq!(layout.route, PageRoute::OcrRegions);
    assert_eq!(page.source, PageSource::Native);
    assert!(page.text.starts_with("SECOND AMENDMENT TO LEASE"));
    assert!(
        page.text.ends_with("Signed April 30, 2026"),
        "{}",
        page.text
    );
    let region = layout.blocks.last().unwrap();
    assert_eq!(region.source, TextSource::Ocr);
    assert_eq!(region.confidence, Some(88));
    assert!(document.warnings.is_empty());

    // A region read as noise adds nothing, and the page reads as it would
    // have without it.
    let noise = read(
        &StandInPdf::new(vec![page_with_an_image_region()]),
        &SignatureReader(31.0),
    );
    assert_eq!(noise.pages[0].text, page_with_an_image_region().native_text);
    // So does one OCR cannot read.
    let unread = read(
        &StandInPdf::new(vec![page_with_an_image_region()]),
        &MissingOcr,
    );
    assert_eq!(
        unread.pages[0].text,
        page_with_an_image_region().native_text
    );
}

/// Pages routed with no geometry from their backend - every stand-in that
/// predates layouts - keep their text and still get blocks.
#[test]
fn a_backend_that_measures_nothing_still_gets_text_blocks() {
    let pdf = StandInPdf::new(vec![PdfPageInspection {
        page_index: 0,
        native_text: "Plain text page.\r\nSecond line.".to_owned(),
        image_coverage: 0.0,
        width_pixels: 100,
        height_pixels: 100,
        ..PdfPageInspection::default()
    }]);

    let document = read(&pdf, &NoOcr);

    let page = &document.pages[0];
    assert_eq!(page.text, "Plain text page.\r\nSecond line.");
    let layout = page.layout.as_ref().unwrap();
    assert_eq!(layout.blocks.len(), 1);
    // The block is the stretch of the page text it came from, line ending
    // and all; its lines are the lines.
    assert_eq!(layout.blocks[0].text, "Plain text page.\r\nSecond line.");
    assert_eq!(layout.blocks[0].lines[1].text, "Second line.");
    assert_eq!(layout.blocks[0].bbox, None);
}

/// The OCR pool records each reading on the request's token, from every
/// worker, and the counts add up however the pages were spread.
#[test]
fn every_worker_records_its_pages() {
    let pages = (0..5).map(scanned_page).collect::<Vec<_>>();
    let cancel = CancellationToken::new();
    let seen = Mutex::new(Vec::new());
    struct Counting<'a>(&'a Mutex<Vec<usize>>);
    impl OcrBackend for Counting<'_> {
        fn recognize(
            &self,
            page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, ExtractionError> {
            self.0.lock().unwrap().push(page.page_index);
            Ok(OcrResult::new("PAGE", 95.0))
        }
        fn concurrency(&self) -> usize {
            4
        }
    }

    extract_pdf(
        Path::new("document.pdf"),
        &StandInPdf::new(pages),
        &Counting(&seen),
        &ResourceLimits::default(),
        &cancel,
    )
    .unwrap();

    let mut seen = seen.into_inner().unwrap();
    seen.sort_unstable();
    assert_eq!(seen, [0, 1, 2, 3, 4]);
    assert_eq!(cancel.timings().ocr_pages, 5);
}
