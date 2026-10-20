//! Indra OCR through the real ONNX Runtime and the pinned models.
//!
//! These run where the worker is built with the `onnx-ocr` feature and
//! `INTERN_RUNTIME_DIR` holds ONNX Runtime and Indra's files under
//! `indra-models/`. Anywhere else they return early, unless
//! `INTERN_REQUIRE_INDRA_OCR` says the runtime must be there, and then its
//! absence fails them. The Intern pages come from `npm run fixtures`.
//!
//! Parity with the reference package (`indra-ocr` 1.1.2, Latin recognizer
//! only) is checked against its own output: for the Intern fixtures from
//! `tests/fixtures/indra/latin-only/`, committed; for its example pages
//! from `INDRA_REFERENCE_DIR` (`latin-only/*.jsonl`, and
//! `intermediate/<page>/page.gray.npy` where dumped) with the pages
//! themselves in `INDRA_INPUTS_DIR` (by default `indra-inputs` beside the
//! reference directory). `bench/tools/indra/README.md` says how both are
//! made.
#![cfg(feature = "onnx-ocr")]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use image::{DynamicImage, GenericImageView};
use intern_worker::extract::{
    CancellationToken, OcrBackend, OcrResult, PageSource, RenderedPage, extract_image,
    load_oriented_image,
};
use intern_worker::indra::types::{Gray, PageReading};
use intern_worker::indra::{IndraAssets, IndraOcr, npy};
use intern_worker::limits::ResourceLimits;
use intern_worker::paddle::ThreadBudget;
use serde_json::Value;

fn runtime() -> Option<PathBuf> {
    let runtime = std::env::var_os("INTERN_RUNTIME_DIR").map(PathBuf::from);
    let present = runtime
        .as_deref()
        .is_some_and(|runtime| IndraAssets::in_directory(runtime).present());
    assert!(
        present || std::env::var_os("INTERN_REQUIRE_INDRA_OCR").is_none(),
        "INTERN_REQUIRE_INDRA_OCR is set, but INTERN_RUNTIME_DIR holds no ONNX Runtime and Indra models"
    );
    runtime.filter(|_| present)
}

/// One engine for the whole test binary, as the worker keeps one for its
/// whole life: ONNX Runtime binds once per process.
fn engine() -> Option<&'static IndraOcr> {
    static ENGINE: OnceLock<Option<IndraOcr>> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            runtime().map(|runtime| IndraOcr::new(IndraAssets::in_directory(&runtime)).unwrap())
        })
        .as_ref()
}

fn engine_with(budget: ThreadBudget) -> Option<IndraOcr> {
    let runtime = runtime()?;
    Some(IndraOcr::with_budget(IndraAssets::in_directory(&runtime), budget).unwrap())
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

fn page_image(path: &Path) -> DynamicImage {
    load_oriented_image(path, &ResourceLimits::default()).unwrap()
}

fn read(engine: &IndraOcr, image: DynamicImage) -> OcrResult {
    engine
        .recognize(&RenderedPage::new(0, image), &CancellationToken::new())
        .unwrap()
}

fn texts(reading: &OcrResult) -> Vec<&str> {
    reading
        .lines
        .iter()
        .map(|line| line.text.as_str())
        .collect()
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
    for block in &reading.blocks {
        assert!(!block.lines.is_empty(), "{block:?}");
        assert!(block.lines.iter().all(|&line| line < reading.lines.len()));
    }
}

/// The packing slip reads exactly, line by line, top to bottom, each line
/// boxed inside the page it was read from, its three blocks as three
/// paragraphs.
#[test]
fn the_packing_slip_reads_line_by_line() {
    let (Some(engine), Some(image)) = (engine(), page("document-image.jpg")) else {
        return;
    };
    let (width, height) = image.dimensions();

    let reading = read(engine, image);

    assert_eq!(
        texts(&reading),
        [
            "PACKING SLIP PS-311",
            "DATE JULY 15 2025",
            "QUARTZ MEADOW RETAIL LLC"
        ]
    );
    assert_eq!(
        reading.text,
        "PACKING SLIP PS-311\n\nDATE JULY 15 2025\n\nQUARTZ MEADOW RETAIL LLC"
    );
    assert_eq!(reading.rotation_degrees, 0);
    assert!(reading.mean_confidence >= 90.0, "{reading:?}");
    assert_lines_inside(&reading, width, height);
    let tops: Vec<u32> = reading.lines.iter().map(|line| line.bbox[1]).collect();
    assert!(tops.windows(2).all(|pair| pair[0] < pair[1]), "{tops:?}");
    assert_eq!(reading.blocks.len(), 3, "{reading:?}");
}

/// Turned any quarter, the page is read the right way up and says which
/// clockwise turn righted it; its lines are boxed in the righted page.
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

/// The installer smoke test's rotated, low-resolution receipt. The
/// orientation network is not sure of it (0.92 for the right turn, under
/// its 0.95), and the reference package reads it on its side, into
/// gibberish; the confidence-driven search reads it upright, in two passes.
///
/// The recognizer reads this page's V as U ("DELIUERY", "UIOLET"),
/// confidently: the reference package reads the righted page so too.
#[test]
fn the_rotated_low_resolution_scan_reads_upright() {
    let (Some(engine), Some(path)) = (engine(), fixture("rotated-low-resolution-scan.png")) else {
        return;
    };

    let cancel = CancellationToken::new();
    let document = extract_image(&path, engine, &ResourceLimits::default(), &cancel).unwrap();

    let page = &document.pages[0];
    assert_eq!(page.source, PageSource::Ocr);
    let text = page.text.to_lowercase();
    for fact in [
        "receipt dr-771",
        "june 12 2025",
        "pine echo couriers llc",
        "cartography studio",
    ] {
        assert!(text.contains(fact), "{fact}: {text}");
    }
    assert!(page.ocr_confidence.unwrap() >= 75.0, "{page:?}");
    assert_eq!(cancel.timings().ocr_passes, 2);

    let image = page_image(&path);
    let reading = read(engine, image);
    assert_eq!(reading.rotation_degrees, 270);
}

/// The same page gives the same reading every time, from any number of
/// threads at once (each read borrows its own sessions), and at one thread
/// a network or two.
#[test]
fn readings_are_deterministic_across_runs_threads_and_thread_counts() {
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

    for intra_threads in [1, 2, 4] {
        let engine = engine_with(ThreadBudget {
            workers: 1,
            intra_threads,
        })
        .unwrap();
        let reading = read(&engine, image.clone());
        assert_eq!(reading.text, first.text, "{intra_threads} threads");
        assert_eq!(texts(&reading), texts(&first), "{intra_threads} threads");
        assert_eq!(reading.rotation_degrees, first.rotation_degrees);
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
    assert_eq!(reading.mean_confidence, 0.0);
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

/// Releasing the sessions after a document loses nothing: the next page
/// builds them again and reads the same.
#[test]
fn a_released_engine_reads_the_same() {
    let (Some(engine), Some(image)) = (engine(), page("document-image.tiff")) else {
        return;
    };
    let before = read(engine, image.clone());
    engine.release();
    assert_eq!(read(engine, image), before);
}

/// Grey levels as OpenCV's `IMREAD_GRAYSCALE` reads the file.
fn gray_of(path: &Path) -> Gray {
    IndraOcr::gray_page(&load_oriented_image(path, &ResourceLimits::default()).unwrap())
}

/// Boxes within this many pixels of the reference's; confidences within
/// this much. Batching and padding move a confidence by up to 5e-3 (spec
/// §1); OpenCV's area resize is matched to within a grey level on a few
/// pixels, which can move a box edge by a fraction of a pixel.
const BOX_TOLERANCE: f64 = 0.5;
const CONF_TOLERANCE: f64 = 5e-3;

/// The largest difference of any edge from the reference's.
fn assert_box_near(got: [f64; 4], want: &Value, what: &str) -> f64 {
    let want: Vec<f64> = want
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap())
        .collect();
    let mut worst = 0.0_f64;
    for (got, want) in got.iter().zip(&want) {
        assert!(
            (got - want).abs() <= BOX_TOLERANCE,
            "{what}: box {got:?} against {want:?}"
        );
        worst = worst.max((got - want).abs());
    }
    worst
}

/// Compares a reading with the reference's JSON line for the page: every
/// line's text, block label and text, and the markdown exactly; boxes and
/// confidences within tolerance.
fn assert_matches_reference(name: &str, reading: &PageReading, reference: &Value) {
    assert_eq!(
        u64::from(reading.rotation),
        reference["rotation"].as_u64().unwrap(),
        "{name}: rotation"
    );
    assert!(
        (reading.skew - reference["skew"].as_f64().unwrap()).abs() <= 0.011,
        "{name}: skew {} against {}",
        reading.skew,
        reference["skew"]
    );
    let lines = reference["lines"].as_array().unwrap();
    let got: Vec<&str> = reading
        .lines
        .iter()
        .map(|line| line.text.as_str())
        .collect();
    let want: Vec<&str> = lines
        .iter()
        .map(|line| line["text"].as_str().unwrap())
        .collect();
    assert_eq!(got, want, "{name}: lines");
    let (mut worst_box, mut worst_conf, mut exact) = (0.0_f64, 0.0_f64, 0);
    for (index, (line, expected)) in reading.lines.iter().zip(lines).enumerate() {
        let what = format!("{name} line {index} {:?}", line.text);
        let off = assert_box_near(line.bbox, &expected["box"], &what);
        let conf = expected["conf"].as_f64().unwrap();
        worst_box = worst_box.max(off);
        worst_conf = worst_conf.max((line.conf - conf).abs());
        exact += usize::from(off == 0.0 && line.conf == conf);
        assert!(
            (line.conf - conf).abs() <= CONF_TOLERANCE,
            "{what}: conf {} against {conf}",
            line.conf
        );
    }
    let blocks = reference["blocks"].as_array().unwrap();
    assert_eq!(reading.blocks.len(), blocks.len(), "{name}: blocks");
    for (index, (block, expected)) in reading.blocks.iter().zip(blocks).enumerate() {
        let what = format!("{name} block {index}");
        assert_eq!(block.class.label(), expected["label"], "{what}");
        assert_eq!(block.text, expected["text"].as_str().unwrap(), "{what}");
        worst_box = worst_box.max(assert_box_near(block.bbox, &expected["bbox"], &what));
    }
    eprintln!(
        "{name}: {} lines and {} blocks as the reference's; {exact} lines exact to the \
         reference's rounding; largest box difference {worst_box:.1} px, confidence {worst_conf:.4}",
        lines.len(),
        blocks.len()
    );
    assert_eq!(
        reading.markdown,
        reference["markdown"].as_str().unwrap(),
        "{name}: markdown"
    );
}

fn reference_line(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(text.lines().next().unwrap()).unwrap()
}

/// Each Intern fixture reads as the reference package reads it, its
/// orientation network's turn alone deciding (the reference's own
/// rotated-scan gibberish included: parity, not quality, is measured here).
#[test]
fn intern_fixtures_read_as_the_reference_reads_them() {
    let Some(engine) = engine() else {
        return;
    };
    let goldens = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/indra/latin-only");
    for name in [
        "document-image.png",
        "document-image.jpg",
        "document-image.tiff",
        "rotated-low-resolution-scan.png",
    ] {
        let Some(path) = fixture(name) else {
            continue;
        };
        let reading = engine.read_reference(&gray_of(&path)).unwrap();
        let reference = reference_line(&goldens.join(format!("{name}.jsonl")));
        assert_matches_reference(name, &reading, &reference);
    }
}

/// Indra's own example pages - two columns, a table, a letter on its side,
/// a memo tilted thirty degrees - read as the reference reads them.
#[test]
fn the_reference_examples_read_as_the_reference_reads_them() {
    let Some(engine) = engine() else {
        return;
    };
    let Some(reference) = std::env::var_os("INDRA_REFERENCE_DIR").map(PathBuf::from) else {
        eprintln!("INDRA_REFERENCE_DIR is not set: the reference examples are skipped");
        return;
    };
    let inputs = std::env::var_os("INDRA_INPUTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| reference.join("../indra-inputs"));
    for name in [
        "multi_column.png",
        "table_pmc.png",
        "rotated_letter.png",
        "tilted_memo.png",
    ] {
        let stem = name.trim_end_matches(".png");
        let dumped = reference.join(format!("intermediate/{stem}/page.gray.npy"));
        // The page exactly as OpenCV decoded it, where it was dumped.
        let gray = if dumped.is_file() {
            let array = npy::read(&dumped).unwrap();
            Gray::new(
                array.shape[1],
                array.shape[0],
                array.as_u8().unwrap().to_vec(),
            )
        } else {
            gray_of(&inputs.join(name))
        };
        let reading = engine.read_reference(&gray).unwrap();
        let expected = reference_line(&reference.join(format!("latin-only/{name}.jsonl")));
        assert_matches_reference(name, &reading, &expected);
    }
}
