//! PP-OCR through the real ONNX Runtime and the pinned models.
//!
//! These run where the worker is built with the `onnx-ocr` feature and
//! `INTERN_RUNTIME_DIR` holds ONNX Runtime and the models
//! (`scripts/fetch-ocr-runtime.sh` stages them on Linux; the Windows package
//! carries them). Anywhere else they return early, unless
//! `INTERN_REQUIRE_PP_OCR` says the runtime must be there, and then its
//! absence fails them. Every page here comes from `npm run fixtures`.
#![cfg(feature = "onnx-ocr")]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use image::{DynamicImage, GenericImageView};
use intern_worker::extract::{
    CancellationToken, OcrBackend, OcrResult, PageSource, RenderedPage, extract_image,
    load_oriented_image,
};
use intern_worker::limits::ResourceLimits;
use intern_worker::paddle::{PaddleAssets, PaddleOcr};

fn runtime() -> Option<PathBuf> {
    let runtime = std::env::var_os("INTERN_RUNTIME_DIR").map(PathBuf::from);
    let present = runtime
        .as_deref()
        .is_some_and(|runtime| PaddleAssets::in_directory(runtime).present());
    assert!(
        present || std::env::var_os("INTERN_REQUIRE_PP_OCR").is_none(),
        "INTERN_REQUIRE_PP_OCR is set, but INTERN_RUNTIME_DIR holds no ONNX Runtime and OCR models"
    );
    runtime.filter(|_| present)
}

/// One engine for the whole test binary, as the worker keeps one for its
/// whole life: ONNX Runtime binds once per process.
fn engine() -> Option<&'static PaddleOcr> {
    static ENGINE: OnceLock<Option<PaddleOcr>> = OnceLock::new();
    ENGINE
        .get_or_init(|| runtime().map(|runtime| PaddleOcr::new(&runtime).unwrap()))
        .as_ref()
}

fn fixture(name: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated")
        .join(name);
    if path.is_file() {
        Some(path)
    } else {
        assert!(
            std::env::var_os("INTERN_REQUIRE_GENERATED_FIXTURES").is_none(),
            "required generated fixture is missing: {}",
            path.display()
        );
        None
    }
}

fn page(name: &str) -> Option<DynamicImage> {
    Some(load_oriented_image(&fixture(name)?, &ResourceLimits::default()).unwrap())
}

fn read(engine: &PaddleOcr, image: DynamicImage) -> OcrResult {
    engine
        .recognize(&RenderedPage::new(0, image), &CancellationToken::new())
        .unwrap()
}

fn assert_lines_inside(reading: &OcrResult, width: u32, height: u32) {
    assert!(!reading.lines.is_empty(), "{reading:?}");
    for line in &reading.lines {
        let [x0, y0, x1, y1] = line.bbox;
        assert!(x0 < x1 && y0 < y1, "{line:?}");
        assert!(
            x1 <= width && y1 <= height,
            "{line:?} outside {width} x {height}"
        );
        assert!(line.confidence > 0, "{line:?}");
    }
}

/// The packing slip reads exactly, line by line, top to bottom, each line
/// boxed inside the page it was read from.
#[test]
fn the_packing_slip_reads_line_by_line() {
    let (Some(engine), Some(image)) = (engine(), page("document-image.jpg")) else {
        return;
    };
    let (width, height) = image.dimensions();

    let reading = read(engine, image);

    assert_eq!(
        reading.text,
        "PACKING SLIP PS-311\nDATE JULY 15 2025\nQUARTZ MEADOW RETAIL LLC"
    );
    assert_eq!(reading.rotation_degrees, 0);
    assert!(reading.mean_confidence >= 90.0, "{reading:?}");
    assert_lines_inside(&reading, width, height);
    let tops: Vec<u32> = reading.lines.iter().map(|line| line.bbox[1]).collect();
    assert!(tops.windows(2).all(|pair| pair[0] < pair[1]), "{tops:?}");
}

/// Turned any quarter, the page is read the right way up and says which
/// turn righted it; its lines are boxed in the righted page.
#[test]
fn every_quarter_turn_is_read_the_right_way_up() {
    let (Some(engine), Some(image)) = (engine(), page("document-image.jpg")) else {
        return;
    };
    let (width, height) = image.dimensions();
    for (turned, righting) in [
        (image.rotate90(), 270),
        (image.rotate180(), 180),
        (image.rotate270(), 90),
    ] {
        let reading = read(engine, turned);

        assert!(
            reading.text.contains("PACKING SLIP PS-311"),
            "{righting}: {reading:?}"
        );
        assert!(
            reading.text.contains("QUARTZ MEADOW RETAIL LLC"),
            "{righting}: {reading:?}"
        );
        assert_eq!(reading.rotation_degrees, righting);
        assert_lines_inside(&reading, width, height);
    }
}

/// The installer smoke test's rotated, low-resolution receipt: the facts
/// it asserts, at the confidence floor it asserts.
#[test]
fn the_rotated_low_resolution_scan_reads_upright() {
    let (Some(engine), Some(path)) = (engine(), fixture("rotated-low-resolution-scan.png")) else {
        return;
    };

    let document = extract_image(
        &path,
        engine,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap();

    let page = &document.pages[0];
    assert_eq!(page.source, PageSource::Ocr);
    let text = page.text.to_lowercase();
    for fact in ["delivery receipt", "june 12", "violet cartography studio"] {
        assert!(text.contains(fact), "{fact}: {text}");
    }
    assert!(page.ocr_confidence.unwrap() >= 60.0, "{page:?}");
}

/// The same page gives the same reading every time, and from any number
/// of threads at once: each read borrows its own sessions, at a fixed
/// thread count.
#[test]
fn readings_are_deterministic_across_runs_and_threads() {
    let (Some(engine), Some(image)) = (engine(), page("document-image.png")) else {
        return;
    };

    let first = read(engine, image.clone());
    let second = read(engine, image.clone());
    let concurrent: Vec<OcrResult> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..3)
            .map(|_| {
                let image = image.clone();
                scope.spawn(move || read(engine, image))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect()
    });

    assert_eq!(first, second);
    for reading in concurrent {
        assert_eq!(reading, first);
    }
}

/// A blank page is blank, in one pass, without an error.
#[test]
fn a_blank_page_reads_as_nothing() {
    let Some(engine) = engine() else {
        return;
    };
    let blank = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        1700,
        2200,
        image::Rgb([255, 255, 255]),
    ));
    let cancel = CancellationToken::new();

    let reading = engine
        .recognize(&RenderedPage::new(0, blank), &cancel)
        .unwrap();

    assert_eq!(reading.text, "");
    assert!(reading.lines.is_empty());
    assert_eq!(cancel.timings().ocr_passes, 1);
}

/// A canceled request stops instead of reading.
#[test]
fn a_canceled_read_stops() {
    let (Some(engine), Some(image)) = (engine(), page("document-image.jpg")) else {
        return;
    };
    let cancel = CancellationToken::new();
    cancel.cancel();

    let error = engine
        .recognize(&RenderedPage::new(0, image), &cancel)
        .unwrap_err();

    assert_eq!(error.code(), "CANCELED");
}
