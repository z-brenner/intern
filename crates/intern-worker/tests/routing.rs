use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use intern_worker::extract::{
    CancellationToken, ExtractionError, OcrBackend, OcrResult, PageSource, PdfBackend,
    PdfPageInspection, RenderedPage, apply_detected_rotation, extract_pdf, normalize_vision_image,
    page_needs_ocr,
};
use intern_worker::limits::{
    MAX_PAGE_MEGAPIXELS, MAX_PAGE_PIXELS, MIN_OCR_DPI, RENDER_DPI, ResourceLimits,
    render_size_within,
};

/// A PDF whose pages are the inspections it is given, sized the way PDFium
/// sizes them: an inspection's pixels are its page at 300 DPI, and a render
/// within a budget is the size [`render_size_within`] gives that page.
#[derive(Clone)]
struct FakePdf {
    pages: Vec<PdfPageInspection>,
    renders: Arc<AtomicUsize>,
    /// The pixel budget each render was asked to keep within.
    budgets: Arc<Mutex<Vec<u64>>>,
}

impl FakePdf {
    fn new(pages: Vec<PdfPageInspection>) -> Self {
        Self {
            pages,
            renders: Arc::new(AtomicUsize::new(0)),
            budgets: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl PdfBackend for FakePdf {
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
        max_pixels: u64,
        _cancel: &CancellationToken,
    ) -> Result<RenderedPage, ExtractionError> {
        self.renders.fetch_add(1, Ordering::SeqCst);
        self.budgets.lock().unwrap().push(max_pixels);
        let page = &self.pages[page_index];
        let points = |pixels: u32| (f64::from(pixels) * 72.0 / RENDER_DPI) as f32;
        let size = render_size_within(
            points(page.width_pixels),
            points(page.height_pixels),
            max_pixels,
        );
        let image = DynamicImage::ImageRgb8(RgbImage::new(size.width, size.height));
        Ok(RenderedPage::new(page_index, image))
    }
}

#[derive(Clone)]
struct FakeOcr {
    results: Vec<OcrResult>,
}

impl OcrBackend for FakeOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        Ok(self.results[page.page_index].clone())
    }
}

/// An OCR engine that remembers the size of every page it was given.
#[derive(Clone, Default)]
struct MeasuringOcr {
    sizes: Arc<Mutex<Vec<(u32, u32)>>>,
}

impl OcrBackend for MeasuringOcr {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        self.sizes.lock().unwrap().push(page.image.dimensions());
        Ok(OcrResult::new("RECEIPT 0417 TOTAL 42.10", 91.0))
    }
}

fn page(text: &str, coverage: f32) -> PdfPageInspection {
    PdfPageInspection {
        page_index: 0,
        native_text: text.to_owned(),
        image_coverage: coverage,
        width_pixels: 100,
        height_pixels: 100,
        ..PdfPageInspection::default()
    }
}

fn route(
    pages: Vec<PdfPageInspection>,
    ocr: Vec<OcrResult>,
) -> (intern_worker::extract::ExtractedDocument, usize) {
    let pdf = FakePdf::new(pages);
    let result = extract_pdf(
        Path::new("fixture.pdf"),
        &pdf,
        &FakeOcr { results: ocr },
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();
    (result, pdf.renders.load(Ordering::SeqCst))
}

#[test]
fn native_text_page_does_not_render_or_ocr() {
    let (document, renders) = route(
        vec![page(
            "This page contains complete native document text.",
            0.0,
        )],
        vec![OcrResult::new("unused", 0.0)],
    );

    assert_eq!(renders, 0);
    assert_eq!(document.pages[0].source, PageSource::Native);
    assert_eq!(
        document.pages[0].text,
        "This page contains complete native document text."
    );
    assert!(document.optional_image.is_none());
}

#[test]
fn fewer_than_twenty_meaningful_characters_with_sixty_five_percent_image_coverage_uses_ocr() {
    let (document, renders) = route(
        vec![page("tiny", 0.65)],
        vec![OcrResult::new("Scanned agreement text", 91.0)],
    );

    assert_eq!(renders, 1);
    assert_eq!(document.pages[0].source, PageSource::Ocr);
    assert_eq!(document.pages[0].text, "Scanned agreement text");
}

#[test]
fn more_than_three_percent_replacement_glyphs_uses_ocr() {
    let (document, renders) = route(
        vec![page("abcdefghijklmnopqrst��", 0.1)],
        vec![OcrResult::new("Recovered text", 88.0)],
    );

    assert_eq!(renders, 1);
    assert_eq!(document.pages[0].source, PageSource::Ocr);
}

#[test]
fn selective_ocr_threshold_boundaries_are_exact() {
    assert!(!page_needs_ocr(&page(&"a".repeat(19), 0.649)));
    assert!(page_needs_ocr(&page(&"a".repeat(19), 0.65)));
    assert!(!page_needs_ocr(&page(&"a".repeat(20), 0.65)));

    // The stamp rule: short text on a page that is all image.
    assert!(page_needs_ocr(&page(&"a".repeat(199), 0.9)));
    assert!(!page_needs_ocr(&page(&"a".repeat(200), 0.9)));
    assert!(!page_needs_ocr(&page(&"a".repeat(199), 0.899)));

    let exactly_three_percent = format!("{}{}", "a".repeat(97), "�".repeat(3));
    let over_three_percent = format!("{}{}", "a".repeat(96), "�".repeat(4));
    assert!(!page_needs_ocr(&page(&exactly_three_percent, 0.0)));
    assert!(page_needs_ocr(&page(&over_three_percent, 0.0)));
}

#[test]
fn clean_mixed_page_preserves_native_text() {
    let (document, renders) = route(
        vec![page(
            "Native text remains authoritative on a mixed page because the complete business document text is already present and does not require visual interpretation.",
            0.8,
        )],
        vec![OcrResult::new("unused", 0.0)],
    );

    assert_eq!(renders, 0);
    assert_eq!(document.pages[0].source, PageSource::Native);
}

#[test]
fn low_ocr_confidence_selects_exactly_one_first_triggering_image() {
    let mut first = page("", 1.0);
    first.page_index = 0;
    let mut second = page("", 1.0);
    second.page_index = 1;
    let (document, renders) = route(
        vec![first, second],
        vec![
            OcrResult::new("first scan", 72.0),
            OcrResult::new("second scan", 61.0),
        ],
    );

    assert_eq!(renders, 2);
    assert_eq!(document.optional_image.as_ref().unwrap().page_number, 1);
    assert_eq!(
        document
            .pages
            .iter()
            .filter(|page| page.vision_escalated)
            .count(),
        1
    );
}

#[test]
fn large_non_text_page_under_one_hundred_characters_routes_first_page_to_vision() {
    let (document, renders) = route(
        vec![page(&"a".repeat(99), 0.65)],
        vec![OcrResult::new("unused", 99.0)],
    );

    assert_eq!(renders, 1);
    assert_eq!(document.pages[0].source, PageSource::Native);
    assert!(document.pages[0].vision_escalated);
    assert_eq!(document.optional_image.as_ref().unwrap().page_number, 1);
}

/// Only the first page that wants the page image is rendered for it: a
/// later one would only be thrown away.
#[test]
fn pages_after_the_one_that_brings_the_page_image_are_not_rendered_for_it() {
    let pages = (0..3)
        .map(|index| {
            let mut page = page(&"a".repeat(99), 0.65);
            page.page_index = index;
            page
        })
        .collect::<Vec<_>>();
    let (document, renders) = route(pages, vec![OcrResult::new("unused", 99.0)]);

    assert_eq!(renders, 1);
    assert_eq!(document.optional_image.as_ref().unwrap().page_number, 1);
    let escalated = document
        .pages
        .iter()
        .filter(|page| page.vision_escalated)
        .map(|page| page.page_number)
        .collect::<Vec<_>>();
    assert_eq!(escalated, [1]);
}

#[test]
fn exactly_seventy_five_confidence_does_not_escalate_but_below_does() {
    let replacement_text = "abcdefghijklmnopqrst��";
    let (exactly, _) = route(
        vec![page(replacement_text, 0.1)],
        vec![OcrResult::new("readable", 75.0)],
    );
    assert!(exactly.optional_image.is_none());

    let (below, _) = route(
        vec![page(replacement_text, 0.1)],
        vec![OcrResult::new("uncertain", 74.99)],
    );
    assert_eq!(below.optional_image.unwrap().page_number, 1);
}

#[test]
fn osd_rotation_is_applied_clockwise_for_all_supported_quarter_turns() {
    let mut source = RgbImage::from_pixel(2, 3, Rgb([0, 0, 0]));
    source.put_pixel(0, 0, Rgb([255, 0, 0]));

    let ninety = apply_detected_rotation(DynamicImage::ImageRgb8(source.clone()), 90).unwrap();
    assert_eq!(ninety.dimensions(), (3, 2));
    assert_eq!(ninety.to_rgb8().get_pixel(2, 0), &Rgb([255, 0, 0]));

    let one_eighty = apply_detected_rotation(DynamicImage::ImageRgb8(source.clone()), 180).unwrap();
    assert_eq!(one_eighty.dimensions(), (2, 3));
    assert_eq!(one_eighty.to_rgb8().get_pixel(1, 2), &Rgb([255, 0, 0]));

    let two_seventy = apply_detected_rotation(DynamicImage::ImageRgb8(source), 270).unwrap();
    assert_eq!(two_seventy.dimensions(), (3, 2));
    assert_eq!(two_seventy.to_rgb8().get_pixel(0, 1), &Rgb([255, 0, 0]));
}

/// The budget is what the render is asked for, and the cap is still what
/// the render is held to: a backend that hands back more than it was asked
/// for is refused on the size it actually produced.
#[test]
fn a_render_over_twenty_five_megapixels_is_still_refused() {
    struct IgnoresTheBudget;

    impl PdfBackend for IgnoresTheBudget {
        fn inspect(
            &self,
            _path: &Path,
            _cancel: &CancellationToken,
        ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
            let mut oversized = page("", 1.0);
            oversized.width_pixels = 5_001;
            oversized.height_pixels = 5_000;
            Ok(vec![oversized])
        }

        fn render_within(
            &self,
            _path: &Path,
            page_index: usize,
            _max_pixels: u64,
            _cancel: &CancellationToken,
        ) -> Result<RenderedPage, ExtractionError> {
            let image = DynamicImage::ImageRgb8(RgbImage::new(5_001, 5_000));
            Ok(RenderedPage::new(page_index, image))
        }
    }

    let error = extract_pdf(
        Path::new("oversized.pdf"),
        &IgnoresTheBudget,
        &FakeOcr { results: vec![] },
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap_err();

    assert_eq!(MAX_PAGE_MEGAPIXELS, 25);
    assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
}

#[test]
fn cancellation_stops_between_pages() {
    struct CancelAfterFirst {
        token: CancellationToken,
    }

    impl OcrBackend for CancelAfterFirst {
        fn recognize(
            &self,
            _page: &RenderedPage,
            _cancel: &CancellationToken,
        ) -> Result<OcrResult, ExtractionError> {
            self.token.cancel();
            Ok(OcrResult::new("first", 90.0))
        }
    }

    let token = CancellationToken::new();
    let mut first = page("", 1.0);
    first.page_index = 0;
    let mut second = page("", 1.0);
    second.page_index = 1;
    let pdf = FakePdf::new(vec![first, second]);

    let error = extract_pdf(
        Path::new("cancel.pdf"),
        &pdf,
        &CancelAfterFirst {
            token: token.clone(),
        },
        &ResourceLimits::default(),
        &token,
    )
    .unwrap_err();

    assert_eq!(error.code(), "CANCELED");
}

#[test]
fn vision_image_is_rgb_bounded_and_padded_to_twenty_eight_pixel_grid() {
    use base64::Engine as _;
    use image::GenericImageView as _;

    let image = DynamicImage::ImageRgb8(RgbImage::new(1_200, 500));
    let normalized = normalize_vision_image(7, image).unwrap();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&normalized.data_base64)
        .unwrap();
    let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).unwrap();

    assert_eq!(normalized.page_number, 8);
    assert_eq!(normalized.mime_type, "image/png");
    assert_eq!(decoded.dimensions(), (1_204, 504));
    assert_eq!(decoded.color(), image::ColorType::Rgb8);
}

#[test]
fn vision_image_long_edge_is_reduced_to_1344_pixels() {
    use base64::Engine as _;
    use image::GenericImageView as _;

    let image = DynamicImage::ImageRgb8(RgbImage::new(2_000, 1_000));
    let normalized = normalize_vision_image(0, image).unwrap();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&normalized.data_base64)
        .unwrap();
    let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).unwrap();

    assert_eq!(decoded.dimensions(), (1_344, 672));
}

/// A litigation scan carries a stamp in its text layer and nothing else. The
/// stamp is long enough to clear the twenty-character veto, so the page used
/// to be filed as native text and the entire document came back as the words
/// "CONFIDENTIAL - SUBJECT TO PROTECTIVE ORDER".
#[test]
fn a_scanned_page_with_a_confidentiality_stamp_is_still_ocred() {
    let (document, renders) = route(
        vec![page("CONFIDENTIAL - SUBJECT TO PROTECTIVE ORDER", 0.99)],
        vec![OcrResult::new(
            "Settlement Agreement and Mutual Release between Acme Corporation and Ridgeline LLC",
            92.0,
        )],
    );

    assert_eq!(renders, 1);
    assert_eq!(document.pages[0].source, PageSource::Ocr);
    assert!(
        document.pages[0].text.contains("Settlement Agreement"),
        "{}",
        document.pages[0].text
    );
}

/// An engineering drawing on an A1 sheet is over the render cap at 300 DPI,
/// and it has a perfectly good text layer. Enforcing the cap before deciding
/// whether the page is ever rendered failed the whole document over a page
/// nobody was going to rasterise.
#[test]
fn a_large_format_text_page_is_extracted_without_rendering() {
    let mut drawing = page(
        "SHEET 3 OF 8 - FOUNDATION PLAN - REVISION C - ISSUED FOR CONSTRUCTION - \
         SCALE 1:50 - DRAWN BY R. OKONKWO - CHECKED BY T. HALVORSEN",
        0.2,
    );
    drawing.width_pixels = 7_020;
    drawing.height_pixels = 9_930;

    let (document, renders) = route(vec![drawing], vec![]);

    assert_eq!(renders, 0);
    assert_eq!(document.pages[0].source, PageSource::Native);
    assert!(document.pages[0].text.contains("FOUNDATION PLAN"));
}

/// The same sheet with nothing in its text layer has to be rendered to be
/// read, and at 300 DPI it is over the render cap. It used to fail the whole
/// document; it is rendered at the resolution that fits instead - a phone
/// photo of a receipt that some tool turned into a PDF at 72 DPI is a
/// 4032 x 3024 point page, about 212 megapixels at 300 DPI, and Tesseract
/// reads it perfectly well at the hundred or so that fit.
#[test]
fn large_format_scanned_page_renders_within_budget() {
    let mut photo = page("", 1.0);
    // 4032 x 3024 points at 300 DPI.
    photo.width_pixels = 16_800;
    photo.height_pixels = 12_600;
    let pdf = FakePdf::new(vec![photo]);
    let ocr = MeasuringOcr::default();

    let document = extract_pdf(
        Path::new("receipt.pdf"),
        &pdf,
        &ocr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();

    assert_eq!(pdf.renders.load(Ordering::SeqCst), 1);
    assert_eq!(*pdf.budgets.lock().unwrap(), vec![MAX_PAGE_PIXELS]);
    let sizes = ocr.sizes.lock().unwrap();
    assert_eq!(sizes.len(), 1);
    let (width, height) = sizes[0];
    assert!(
        u64::from(width) * u64::from(height) <= MAX_PAGE_PIXELS,
        "{width} x {height}"
    );
    // Downscaled, not cropped: the page keeps its proportions.
    assert!(
        (f64::from(width) / f64::from(height) - 4.0 / 3.0).abs() < 0.001,
        "{width} x {height}"
    );
    // And no smaller than it has to be: about 103 DPI, within a pixel of
    // the most that fits.
    assert_eq!((width, height), (5_773, 4_330));
    assert_eq!(document.pages[0].source, PageSource::Ocr);
    assert_eq!(document.pages[0].text, "RECEIPT 0417 TOTAL 42.10");
}

/// Downscaling has a floor. A page that would have to be rendered below
/// 50 DPI to fit is a degenerate file rather than a scan - its text would be
/// a few pixels tall - and that is still a resource limit, decided before
/// anything is rendered.
#[test]
fn below_floor_dpi_is_resource_limit() {
    // 30,000 pixels a side at 300 DPI fits 25 megapixels at exactly 50 DPI.
    let mut at_the_floor = page("", 1.0);
    at_the_floor.width_pixels = 30_000;
    at_the_floor.height_pixels = 30_000;
    let mut below_the_floor = at_the_floor.clone();
    below_the_floor.width_pixels = 30_001;
    assert_eq!(MIN_OCR_DPI, 50.0);

    let pdf = FakePdf::new(vec![below_the_floor]);
    let error = extract_pdf(
        Path::new("banner.pdf"),
        &pdf,
        &MeasuringOcr::default(),
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap_err();

    assert_eq!(error.code(), "RESOURCE_LIMIT_EXCEEDED");
    assert_eq!(pdf.renders.load(Ordering::SeqCst), 0);

    let pdf = FakePdf::new(vec![at_the_floor]);
    let ocr = MeasuringOcr::default();
    extract_pdf(
        Path::new("banner.pdf"),
        &pdf,
        &ocr,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();

    assert_eq!(pdf.renders.load(Ordering::SeqCst), 1);
    let (width, height) = ocr.sizes.lock().unwrap()[0];
    assert!(u64::from(width) * u64::from(height) <= MAX_PAGE_PIXELS);
}

/// A page the worker will cut on the way out - past the characters a page
/// may carry, or past the document's - has no layout built for it: a
/// layout repeats its text several times over.
#[test]
fn pages_the_worker_will_cut_have_no_layout_built() {
    use intern_worker::limits::{MAX_DOCUMENT_CHARS, MAX_PAGE_CHARS};

    let full = "word ".repeat(MAX_PAGE_CHARS / 5);
    let over = format!("{full}x");
    let texts = [&over, &full, &full, &full, &full, &full];
    let pages = texts
        .iter()
        .enumerate()
        .map(|(index, text)| PdfPageInspection {
            page_index: index,
            ..page(text, 0.0)
        })
        .collect();
    let (document, renders) = route(pages, Vec::new());
    assert_eq!(renders, 0);
    let built = document
        .pages
        .iter()
        .map(|page| page.layout.is_some())
        .collect::<Vec<_>>();
    // The first page is cut to a page's worth, which the document counts;
    // three whole pages use the rest of it, and the pages after are cut.
    assert_eq!(MAX_DOCUMENT_CHARS, 4 * MAX_PAGE_CHARS);
    assert_eq!(built, [false, true, true, true, false, false]);
}

/// A fast layout holds an object for every line and cell of its page, and
/// the character caps do not bound how many: a page of more than a page's
/// layout may hold, and pages past what the document's may hold together,
/// are read as their text with no layout built.
#[test]
fn pages_of_more_lines_than_layouts_may_hold_have_no_layout_built() {
    use intern_worker::limits::{MAX_DOCUMENT_LAYOUT_PARTS, MAX_PAGE_LAYOUT_PARTS};

    let dense = "A\n\n".repeat(MAX_PAGE_LAYOUT_PARTS / 2);
    let full = "A\n".repeat(MAX_PAGE_LAYOUT_PARTS - 1);
    let fitting = MAX_DOCUMENT_LAYOUT_PARTS / MAX_PAGE_LAYOUT_PARTS;
    let texts = std::iter::once(&dense)
        .chain(std::iter::repeat_n(&full, fitting + 1))
        .collect::<Vec<_>>();
    let pages = texts
        .iter()
        .enumerate()
        .map(|(index, text)| PdfPageInspection {
            page_index: index,
            ..page(text, 0.0)
        })
        .collect();
    let (document, renders) = route(pages, Vec::new());
    assert_eq!(renders, 0);
    let built = document
        .pages
        .iter()
        .map(|page| page.layout.is_some())
        .collect::<Vec<_>>();
    let mut expected = vec![false];
    expected.extend(std::iter::repeat_n(true, fitting));
    expected.push(false);
    assert_eq!(built, expected);
    assert_eq!(document.pages[0].text, dense);
    assert!(!document.truncated);
}

/// Scanned pages count against the document's characters too: a page the
/// worker will cut keeps its text for the cut but lets its layout go.
#[test]
fn scanned_pages_past_the_documents_characters_keep_no_layout() {
    use intern_worker::limits::MAX_PAGE_CHARS;

    let line = "word ".repeat(16);
    let full = format!("{line}\n").repeat(MAX_PAGE_CHARS / (line.len() + 1));
    let pages = (0..5)
        .map(|index| PdfPageInspection {
            page_index: index,
            ..page("", 1.0)
        })
        .collect();
    let readings = (0..5).map(|_| OcrResult::new(full.clone(), 91.0)).collect();
    let (document, renders) = route(pages, readings);
    assert_eq!(renders, 5);
    let built = document
        .pages
        .iter()
        .map(|page| page.layout.is_some())
        .collect::<Vec<_>>();
    assert_eq!(built, [true, true, true, true, false]);
    assert!(
        document
            .pages
            .iter()
            .all(|page| page.source == PageSource::Ocr)
    );
}

/// The document's characters are counted in page order whatever the route:
/// a scan first keeps its layout, and the native page past the document's
/// characters is the one without - as the worker cuts them on the way out.
#[test]
fn a_mixed_documents_characters_are_counted_in_page_order() {
    use intern_worker::limits::MAX_PAGE_CHARS;

    let line = "word ".repeat(16);
    let full = format!("{line}\n").repeat(MAX_PAGE_CHARS / (line.len() + 1));
    let mut pages = vec![page("", 1.0)];
    pages.extend((1..5).map(|index| PdfPageInspection {
        page_index: index,
        ..page(&full, 0.0)
    }));
    let mut readings = vec![OcrResult::new(full.clone(), 91.0)];
    readings.extend((1..5).map(|_| OcrResult::new("", 0.0)));
    let (document, renders) = route(pages, readings);
    assert_eq!(renders, 1);
    assert_eq!(document.pages[0].source, PageSource::Ocr);
    let built = document
        .pages
        .iter()
        .map(|page| page.layout.is_some())
        .collect::<Vec<_>>();
    assert_eq!(built, [true, true, true, true, false]);
}

/// A page routed to be read for its geometry whose text is longer than a
/// page may carry is never read into runs: the worker would cut it and its
/// layout on the way out. It is read as its text, without a layout.
#[test]
fn a_page_longer_than_a_page_may_carry_is_not_read_for_its_geometry() {
    use intern_worker::layout::{NativePage, RouteSignals, TextRun};
    use intern_worker::limits::MAX_PAGE_CHARS;

    struct RefusesRuns(PdfPageInspection);
    impl PdfBackend for RefusesRuns {
        fn inspect(
            &self,
            _path: &Path,
            _cancel: &CancellationToken,
        ) -> Result<Vec<PdfPageInspection>, ExtractionError> {
            Ok(vec![self.0.clone()])
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
            _page_index: usize,
            _native: &NativePage,
            _cancel: &CancellationToken,
        ) -> Result<Vec<TextRun>, ExtractionError> {
            panic!("the characters of a page the worker will cut were read")
        }
    }

    let text = "word ".repeat(MAX_PAGE_CHARS / 5 + 1);
    let inspection = PdfPageInspection {
        native: Some(NativePage::default()),
        // Two columns: the router sends it to be read for its geometry.
        signals: Some(RouteSignals {
            chars: 1_000,
            columns: 2,
            ..RouteSignals::default()
        }),
        ..page(&text, 0.0)
    };
    let document = extract_pdf(
        Path::new("long.pdf"),
        &RefusesRuns(inspection),
        &FakeOcr {
            results: Vec::new(),
        },
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();
    let page = &document.pages[0];
    assert_eq!(page.text, text);
    assert!(page.layout.is_none());
}

/// Pages routed to be read for their geometry past what the document's
/// characters carry - counting the pages read as their text before them -
/// are not read into runs while the document is planned: the worker would
/// cut them, and a layout built for each would be held until then.
#[test]
fn pages_past_the_documents_characters_are_not_read_for_their_geometry() {
    use intern_worker::layout::{NativePage, RouteSignals, TextRun};
    use intern_worker::limits::{MAX_DOCUMENT_CHARS, MAX_PAGE_CHARS};

    struct CountsRuns {
        pages: Vec<PdfPageInspection>,
        asked: AtomicUsize,
    }
    impl PdfBackend for CountsRuns {
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
            _page_index: usize,
            _native: &NativePage,
            _cancel: &CancellationToken,
        ) -> Result<Vec<TextRun>, ExtractionError> {
            self.asked.fetch_add(1, Ordering::SeqCst);
            Ok(Vec::new())
        }
    }

    let text = "word ".repeat(MAX_PAGE_CHARS / 5);
    let pages = (0..6)
        .map(|index| PdfPageInspection {
            page_index: index,
            native: Some(NativePage::default()),
            signals: Some(RouteSignals {
                chars: 1_000,
                columns: 2,
                ..RouteSignals::default()
            }),
            ..page(&text, 0.0)
        })
        .collect();
    let backend = CountsRuns {
        pages,
        asked: AtomicUsize::new(0),
    };
    let document = extract_pdf(
        Path::new("long.pdf"),
        &backend,
        &FakeOcr {
            results: Vec::new(),
        },
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(document.pages.len(), 6);
    assert_eq!(
        backend.asked.load(Ordering::SeqCst),
        MAX_DOCUMENT_CHARS / MAX_PAGE_CHARS,
        "only the pages within the document's characters are read for their geometry"
    );
}
