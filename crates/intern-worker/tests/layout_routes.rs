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

/// Geometry layouts are built while the pages are planned and held until
/// they are put back in order, and one-character runs are a line each, far
/// inside the characters a document may carry. Every layout a document
/// keeps counts against what its layouts may hold together: once the pages
/// before have taken it, a page is read as its text.
#[test]
fn geometry_layouts_past_what_a_documents_layouts_may_hold_are_not_kept() {
    use intern_worker::layout::RouteSignals;
    use intern_worker::limits::MAX_DOCUMENT_LAYOUT_PARTS;

    const RUNS: usize = 3_000;
    let lines = (0..RUNS).map(|line| format!("w{line}")).collect::<Vec<_>>();
    // PDFium's text runs the lines together; the geometry puts them a line
    // each.
    let text = lines.join(" ");
    let page = |page_index: usize| {
        let runs = lines
            .iter()
            .enumerate()
            .map(|(line, word)| run(54, 10 + line as u32 * 2, word, 1))
            .collect();
        let mut page = native_page(page_index, &text, runs);
        let native = page.native.as_mut().unwrap();
        native.height = 10 * (20 + 2 * RUNS as u32);
        // Two columns, so the page is rebuilt from its geometry.
        page.signals = Some(RouteSignals {
            chars: 1_000,
            columns: 2,
            ..RouteSignals::default()
        });
        page
    };
    let first = read(&StandInPdf::new(vec![page(0)]), &NoOcr);
    let layout = first.pages[0].layout.as_ref().unwrap();
    assert_eq!(layout.route, PageRoute::Layout);
    let per_page = layout.parts();
    assert!(per_page >= RUNS, "a line for every run: {per_page}");
    let fitting = MAX_DOCUMENT_LAYOUT_PARTS / per_page;

    let pages = (0..fitting + 3).map(page).collect();
    let document = read(&StandInPdf::new(pages), &NoOcr);

    let kept = document
        .pages
        .iter()
        .map(|page| page.layout.as_ref().map(|layout| layout.route))
        .collect::<Vec<_>>();
    // The pages past what the layouts may hold are not rebuilt from their
    // geometry while they are planned: they are read as their text, whose
    // one line is a layout small enough to keep.
    let mut expected = vec![Some(PageRoute::Layout); fitting];
    expected.extend([Some(PageRoute::Fast); 3]);
    assert_eq!(kept, expected);
    let held = document
        .pages
        .iter()
        .filter_map(|page| page.layout.as_ref())
        .map(|layout| layout.parts())
        .sum::<usize>();
    assert!(held <= MAX_DOCUMENT_LAYOUT_PARTS);
    assert_eq!(document.pages[fitting].text, text);
    assert_ne!(document.pages[0].text, text, "rebuilt a line a run");
}

/// A page sent to the layout route whose runs all lack width - type too
/// small for its glyphs to have any - has no geometry to be rebuilt from:
/// it keeps its text as it was read, on the fast route, rather than going
/// out empty.
#[test]
fn a_page_whose_runs_have_no_width_keeps_its_text() {
    use intern_worker::layout::RouteSignals;

    let text = "Payment due on receipt.\r\nRemit to the address above.";
    let runs = text
        .split("\r\n")
        .enumerate()
        .map(|(line, words)| {
            let mut run = run(54, 60 + line as u32 * 12, words, 9);
            run.bbox[2] = run.bbox[0];
            run
        })
        .collect();
    let mut page = native_page(0, text, runs);
    page.signals = Some(RouteSignals {
        chars: 1_000,
        columns: 2,
        ..RouteSignals::default()
    });

    let document = read(&StandInPdf::new(vec![page]), &NoOcr);

    let page = &document.pages[0];
    assert_eq!(page.text, text);
    assert_eq!(page.layout.as_ref().unwrap().route, PageRoute::Fast);
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
    // The layer has text enough not to want the page image on its own, but
    // a reading that came back unsure says the page is hard to read either
    // way, so its image goes with the layer's text.
    assert_eq!(
        kept.optional_image.as_ref().map(|image| image.page_number),
        Some(1)
    );
    assert!(kept.pages[0].vision_escalated);

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

/// Somebody else's OCR layer under a scan, drawn twice over it, as a
/// searchable scan's layer sometimes is: every line comes twice, as PDFium
/// reads it, and every word box lies on its double. Its words are clean.
fn doubled_prior_ocr(page_index: usize) -> PdfPageInspection {
    let lines = [
        "Department of Environmental Quality",
        "June 17, 2025",
        "CERTIFIED MAIL: 7018 1830 0001 6172",
        "City of Harlow",
        "c/o Dana Pruitt, Public Works Director",
        "Re: Notice of Civil Penalty Assessment",
    ];
    let mut runs = Vec::new();
    let mut text = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        let y = 60 + row as u32 * 14;
        runs.push(run(54, y, line, 9));
        runs.push(run(54, y, line, 9));
        text.push(format!("{line}\r\n{line}"));
    }
    let mut page = native_page(page_index, &text.join("\r\n"), runs);
    page.image_coverage = 1.0;
    let native = page.native.as_mut().unwrap();
    native.text_objects = 12;
    native.invisible_text_objects = 12;
    page
}

/// PP-OCR's reading of the scan under [`doubled_prior_ocr`].
const DOUBLED_READING: &str = "Department of Environmental Quality\nJune 17, 2025\n\
CERTIFIED MAIL: 7018 1830 0001 6172\nCity of Harlow\n\
c/o Dana Pruitt, Public Works Director\nRe: Notice of Civil Penalty Assessment";

struct ScanReading(f32);

impl OcrBackend for ScanReading {
    fn recognize(
        &self,
        _page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        Ok(OcrResult::new(DOUBLED_READING, self.0))
    }
}

/// A layer whose words are clean but that would not read well as it is -
/// drawn twice, it would be rebuilt into a layout that repeats every line -
/// is read again on the pages a document's facts are on, and a confident
/// reading takes its place.
#[test]
fn a_prior_ocr_layer_that_would_not_read_well_is_read_again_on_a_key_page() {
    let layer = doubled_prior_ocr(0);
    let signals = intern_worker::layout::measure_signals(
        layer.native.as_ref().unwrap(),
        &layer.native_text,
        layer.image_coverage,
    );
    assert!(
        signals.overlap >= 100 && signals.garbage == 0,
        "{signals:?}"
    );
    let pdf = StandInPdf::new(vec![layer]);

    let document = read(&pdf, &ScanReading(97.0));

    let page = &document.pages[0];
    assert_eq!(page.source, PageSource::Ocr);
    assert_eq!(page.layout.as_ref().unwrap().route, PageRoute::Ocr);
    assert_eq!(
        page.text.matches("June 17, 2025").count(),
        1,
        "{}",
        page.text
    );
    assert_eq!(pdf.renders.load(Ordering::SeqCst), 1);
    assert!(document.warnings.is_empty());
}

/// The look-alikes stay as they are: the same layer on a page in the middle
/// of a long document, which is not read again for its layout; a reading
/// that is not confident, which leaves a layer whose words are clean; and
/// the same text drawn visibly, which is nobody's OCR.
#[test]
fn a_prior_ocr_layer_off_the_key_pages_or_read_unsurely_is_kept() {
    // The first two pages and the last are read again; the middle ones are
    // not.
    let pdf = StandInPdf::new((0..5).map(doubled_prior_ocr).collect());

    let document = read(&pdf, &ScanReading(97.0));

    let sources = document
        .pages
        .iter()
        .map(|page| page.source)
        .collect::<Vec<_>>();
    assert_eq!(
        sources,
        [
            PageSource::Ocr,
            PageSource::Ocr,
            PageSource::Native,
            PageSource::Native,
            PageSource::Ocr
        ]
    );
    let middle = &document.pages[2];
    assert_eq!(middle.layout.as_ref().unwrap().route, PageRoute::Layout);
    assert_eq!(
        middle.text.matches("June 17, 2025").count(),
        2,
        "the layer, as it reads: {}",
        middle.text
    );
    assert_eq!(pdf.renders.load(Ordering::SeqCst), 3);

    // A reading under the confident line leaves a layer whose words are
    // clean - there are no OCR errors for it to have fewer of - and the
    // layer reads as it would have had it not been read again.
    let unsure = read(
        &StandInPdf::new(vec![doubled_prior_ocr(0)]),
        &ScanReading(68.0),
    );
    assert_eq!(unsure.pages[0].source, PageSource::Native);
    assert_eq!(
        unsure.pages[0].layout.as_ref().unwrap().route,
        PageRoute::Layout
    );
    assert_eq!(unsure.pages[0].text, middle.text);
    // Nor does it bring a page image or a warning: no OCR read the layer
    // before, and a reading that does not replace it says nothing of it.
    assert!(unsure.optional_image.is_none());
    assert!(unsure.warnings.is_empty());

    // Drawn visibly, the same text is the document's own, and is read as it
    // always was.
    let mut visible = doubled_prior_ocr(0);
    visible.native.as_mut().unwrap().invisible_text_objects = 0;
    let own = read(&StandInPdf::new(vec![visible]), &NoOcr);
    assert_eq!(own.pages[0].source, PageSource::Native);
    assert_eq!(
        own.pages[0].layout.as_ref().unwrap().route,
        PageRoute::Layout
    );
}

/// A layer that would be read again but whose page cannot be rendered for
/// it - a sheet too large to render, here an A1 drawing at 300 DPI - is
/// read as its text routes it, its characters read for that route: the
/// same layout it would have had if it were never to be read again.
#[test]
fn a_prior_ocr_layer_that_cannot_be_rendered_reads_as_its_text_routes_it() {
    let mut sheet = doubled_prior_ocr(0);
    (sheet.width_pixels, sheet.height_pixels) = (7016, 9933);
    // Its characters come only when the page asks for them, and it is
    // never rendered.
    let pdf = RunsOnRequest::new(vec![sheet]);

    let document = read_with(&pdf, &NoOcr);

    assert_eq!(*pdf.asked.lock().unwrap(), [0]);
    let page = &document.pages[0];
    assert_eq!(page.source, PageSource::Native);
    assert_eq!(page.layout.as_ref().unwrap().route, PageRoute::Layout);
    let middle = read(
        &StandInPdf::new((0..5).map(doubled_prior_ocr).collect()),
        &ScanReading(97.0),
    );
    assert_eq!(page.text, middle.pages[2].text);
}

/// The lines of a short letter.
const LETTER: [&str; 5] = [
    "Dear Ms. Okafor:",
    "Thank you for your letter of March 4, 2019, about the drainage easement.",
    "The county surveyor will walk the east boundary with your contractor",
    "on the morning of March 18. Please call the office if that does not suit.",
    "Yours sincerely,",
];

/// A sparse layer - [`LETTER`]'s few lines - over the scan it was read
/// from. It has text enough not to be a stamp on a scan, but covers too
/// little of the scan for the scan to count as text the layer holds.
fn sparse_prior_ocr() -> PdfPageInspection {
    let runs = LETTER
        .iter()
        .enumerate()
        .map(|(row, line)| run(54, 120 + row as u32 * 14, line, 9))
        .collect();
    let mut layer = native_page(0, &LETTER.join("\r\n"), runs);
    layer.image_coverage = 1.0;
    let native = layer.native.as_mut().unwrap();
    native.images = vec![[0, 0, 6120, 7920]];
    native.text_objects = 5;
    native.invisible_text_objects = 5;
    layer
}

/// PP-OCR's reading of the scan under [`sparse_prior_ocr`], at a
/// confidence.
struct LetterReading(f32);

impl OcrBackend for LetterReading {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        // The whole page, not a region cut from it.
        assert_eq!(page.image.width(), 255);
        Ok(OcrResult::new(LETTER.join("\n"), self.0))
    }
}

/// The sparse layer's scan would be read as an image region and its lines
/// merged into the layer's, every line twice. Read again instead, the page
/// is read once, with the same one OCR pass.
#[test]
fn a_sparse_prior_ocr_layer_is_read_once_not_merged_with_its_own_scan() {
    let layer = sparse_prior_ocr();
    assert!(!intern_worker::extract::page_needs_ocr(&layer));
    let signals = intern_worker::layout::measure_signals(
        layer.native.as_ref().unwrap(),
        &layer.native_text,
        layer.image_coverage,
    );
    assert_eq!(
        intern_worker::layout::router::text_route(&signals),
        PageRoute::OcrRegions,
        "{signals:?}"
    );
    let pdf = StandInPdf::new(vec![layer]);

    let document = read(&pdf, &LetterReading(98.0));

    let page = &document.pages[0];
    assert_eq!(page.source, PageSource::Ocr);
    assert_eq!(
        page.text.matches("March 4, 2019").count(),
        1,
        "{}",
        page.text
    );
    assert_eq!(pdf.renders.load(Ordering::SeqCst), 1);
    assert!(document.warnings.is_empty());
}

/// A sparse layer whose reading is not confident stays as it read when its
/// scan was read as a region: the reading merged in, the page no surer than
/// that reading, and the document saying so - an unsure reading of a page
/// still sends it to be looked at. A reading of noise adds nothing, as a
/// region's never did, and neither brings a page image the region never had.
#[test]
fn a_sparse_prior_ocr_layer_read_unsurely_reads_as_its_region_did() {
    let unsure = read(
        &StandInPdf::new(vec![sparse_prior_ocr()]),
        &LetterReading(68.0),
    );

    let page = &unsure.pages[0];
    assert_eq!(page.source, PageSource::Native);
    assert_eq!(page.layout.as_ref().unwrap().route, PageRoute::OcrRegions);
    assert_eq!(
        page.text.matches("March 4, 2019").count(),
        2,
        "the layer and the reading merged in: {}",
        page.text
    );
    assert_eq!(page.ocr_confidence, Some(68.0));
    assert_eq!(unsure.warnings, vec![ExtractionWarning::LowOcrConfidence]);
    assert!(unsure.optional_image.is_none());

    let noise = read(
        &StandInPdf::new(vec![sparse_prior_ocr()]),
        &LetterReading(31.0),
    );
    assert_eq!(noise.pages[0].source, PageSource::Native);
    assert_eq!(noise.pages[0].text, sparse_prior_ocr().native_text);
    assert_eq!(noise.pages[0].ocr_confidence, None);
    assert!(noise.warnings.is_empty());
    assert!(noise.optional_image.is_none());
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
    assert_eq!(page.ocr_confidence, Some(88.0));
    assert!(document.warnings.is_empty());

    // A region kept though OCR was unsure of it: the page is native text,
    // but no surer than that reading, and the document says so.
    let unsure = read(
        &StandInPdf::new(vec![page_with_an_image_region()]),
        &SignatureReader(68.0),
    );
    assert!(unsure.pages[0].text.ends_with("Signed April 30, 2026"));
    assert_eq!(unsure.pages[0].source, PageSource::Native);
    assert_eq!(unsure.pages[0].ocr_confidence, Some(68.0));
    assert_eq!(unsure.warnings, vec![ExtractionWarning::LowOcrConfidence]);

    // A region read as noise adds nothing, and the page reads as it would
    // have without it.
    let noise = read(
        &StandInPdf::new(vec![page_with_an_image_region()]),
        &SignatureReader(31.0),
    );
    assert_eq!(noise.pages[0].text, page_with_an_image_region().native_text);
    assert_eq!(noise.pages[0].ocr_confidence, None);
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
