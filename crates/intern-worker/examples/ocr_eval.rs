//! OCR engine evaluation harness (exploratory; not part of the worker).
//!
//! `render` writes every InternBench page that has OCR truth as a PNG, with
//! exactly the pixels the worker hands OCR: a PDF page through PDFium at
//! 300 DPI within the worker's pixel cap, an image file (every TIFF frame)
//! as the worker decodes it. `read` reads those PNGs with one engine
//! configuration, one page at a time and one document's pages after
//! another, releasing the engine's sessions after each document as the
//! worker does, and writes each page's text with its wall and CPU time.
//!
//! ```text
//! ocr_eval render --corpus bench/generated --gold bench/gold.json --out PAGES
//! ocr_eval read --pages PAGES --out READ.json --engine paddle|tesseract
//!     [--runtime DIR] [--det ONNX] [--rec ONNX --dict TXT] [--det-side 960]
//!     [--db v6|v6tiny] [--threads 2] [--no-reread] [--no-orientation]
//!     [--reread-rec ONNX --reread-dict TXT] [--only id,id]
//! ```
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use image::DynamicImage;
use intern_worker::extract::{
    CancellationToken, ExtractionError, OcrBackend, OcrResult, PdfBackend, RenderedPage,
    extract_image,
};
use intern_worker::limits::ResourceLimits;
use intern_worker::paddle::db::DbParams;
use intern_worker::paddle::tensor::DetectionSize;
use intern_worker::paddle::{PaddleAssets, PaddleConfig, PaddleOcr};
use intern_worker::pdf::PdfiumBackend;
use serde_json::{Value, json};

fn args() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let raw: Vec<String> = std::env::args().skip(2).collect();
    let mut index = 0;
    while index < raw.len() {
        let key = raw[index].trim_start_matches("--").to_owned();
        if index + 1 < raw.len() && !raw[index + 1].starts_with("--") {
            map.insert(key, raw[index + 1].clone());
            index += 2;
        } else {
            map.insert(key, String::new());
            index += 1;
        }
    }
    map
}

/// Hands back every page image it is given, in order, and reads nothing.
struct Capture(Mutex<Vec<DynamicImage>>);

impl OcrBackend for Capture {
    fn recognize(
        &self,
        page: &RenderedPage,
        _cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        self.0.lock().unwrap().push(page.image.clone());
        Ok(OcrResult::new("", 0.0))
    }
}

fn render(options: &BTreeMap<String, String>) {
    let corpus = PathBuf::from(&options["corpus"]);
    let out = PathBuf::from(&options["out"]);
    let gold: Value = serde_json::from_slice(&std::fs::read(&options["gold"]).unwrap()).unwrap();
    let runtime = PathBuf::from(std::env::var("INTERN_RUNTIME_DIR").unwrap());
    let pdf = PdfiumBackend::new(&runtime).unwrap();
    let limits = ResourceLimits::default();
    let cancel = CancellationToken::new();
    let mut index = Vec::new();
    for document in gold["documents"].as_array().unwrap() {
        let Some(truth) = document.get("ocr_truth").filter(|truth| !truth.is_null()) else {
            continue;
        };
        let id = document["id"].as_str().unwrap();
        let file = corpus.join(document["file"].as_str().unwrap());
        let pages: Vec<usize> = truth["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|page| page["page"].as_u64().unwrap() as usize)
            .collect();
        let directory = out.join(id);
        std::fs::create_dir_all(&directory).unwrap();
        let mut written = Vec::new();
        if file.extension().is_some_and(|extension| extension == "pdf") {
            for &page in &pages {
                let rendered = pdf
                    .render_within(&file, page - 1, limits.max_page_pixels, &cancel)
                    .unwrap();
                let path = directory.join(format!("p{page}.png"));
                rendered.image.save(&path).unwrap();
                written.push(json!({"page": page, "width": rendered.image.width(), "height": rendered.image.height()}));
            }
        } else {
            let capture = Capture(Mutex::new(Vec::new()));
            extract_image(&file, &capture, &limits, &cancel).unwrap();
            for (frame, image) in capture.0.into_inner().unwrap().into_iter().enumerate() {
                let page = frame + 1;
                if !pages.contains(&page) {
                    continue;
                }
                image.save(directory.join(format!("p{page}.png"))).unwrap();
                written
                    .push(json!({"page": page, "width": image.width(), "height": image.height()}));
            }
        }
        eprintln!("{id}: {} of {} truth pages", written.len(), pages.len());
        index.push(json!({"id": id, "pages": written}));
    }
    std::fs::write(
        out.join("index.json"),
        serde_json::to_vec_pretty(&index).unwrap(),
    )
    .unwrap();
}

/// This process's CPU seconds so far, its own and its reaped children's
/// (Tesseract runs as a child process per page).
fn cpu_seconds() -> f64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap();
    let after = &stat[stat.rfind(')').unwrap() + 2..];
    let fields: Vec<&str> = after.split_whitespace().collect();
    // utime, stime, cutime, cstime are fields 14-17 of the whole line, 12-15
    // after the command name and state.
    let ticks: f64 = fields[11..15]
        .iter()
        .map(|value| value.parse::<f64>().unwrap())
        .sum();
    ticks / 100.0
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .map(|value| value.trim().trim_end_matches("kB").trim().parse().unwrap())
        .unwrap_or(0)
}

fn read(options: &BTreeMap<String, String>) {
    let pages_directory = PathBuf::from(&options["pages"]);
    let runtime = options
        .get("runtime")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var("INTERN_RUNTIME_DIR").unwrap()));
    let index: Value =
        serde_json::from_slice(&std::fs::read(pages_directory.join("index.json")).unwrap())
            .unwrap();
    let only: Option<Vec<String>> = options
        .get("only")
        .map(|list| list.split(',').map(str::to_owned).collect());
    let started_cpu = cpu_seconds();
    let started = Instant::now();
    let engine: Box<dyn OcrBackend> = match options["engine"].as_str() {
        "paddle" => {
            let mut assets = PaddleAssets::in_directory(&runtime);
            if let Some(det) = options.get("det") {
                assets.detection_model = det.into();
            }
            if let Some(rec) = options.get("rec") {
                assets.recognition_model = rec.into();
            }
            if options.contains_key("no-orientation") {
                assets.orientation_model = PathBuf::from("/nonexistent/orientation.onnx");
            }
            let mut config = PaddleConfig::default();
            let threads: usize = options
                .get("threads")
                .map_or(2, |threads| threads.parse().unwrap());
            config.intra_threads = threads;
            config.workers = 1;
            if let Some(side) = options.get("det-side") {
                config.detection_size = DetectionSize::LongSide {
                    max: side.parse().unwrap(),
                    min: 0,
                };
            }
            // The newer detectors' own post-processing (their inference.yml):
            // tiny keeps boxes from 0.4, small and medium from 0.45.
            match options.get("db").map(String::as_str) {
                Some(db @ ("v6" | "v6tiny")) => {
                    config.db = DbParams {
                        thresh: 0.2,
                        box_thresh: if db == "v6tiny" { 0.4 } else { 0.45 },
                        unclip_ratio: 1.4,
                        max_candidates: 3000,
                        ..DbParams::MOBILE
                    };
                }
                Some(other) => panic!("unknown --db {other}"),
                None => {}
            }
            if let Some(dict) = options.get("dict") {
                config.recognition_dictionary = Some(std::fs::read_to_string(dict).unwrap());
            }
            if let Some(rec) = options.get("reread-rec") {
                config.reread_recognizer = Some((
                    rec.into(),
                    std::fs::read_to_string(&options["reread-dict"]).unwrap(),
                ));
            }
            if options.contains_key("no-reread") {
                config.reread = None;
            }
            eprintln!("config: {config:?}\nassets: {assets:?}");
            Box::new(PaddleOcr::with_config(assets, config).unwrap())
        }
        "tesseract" => Box::new(
            intern_worker::ocr::TesseractOcr::new(
                runtime.join("tesseract.exe"),
                runtime.join("tessdata"),
            )
            .unwrap(),
        ),
        other => panic!("unknown engine {other}"),
    };
    let load_seconds = started.elapsed().as_secs_f64();
    let load_cpu = cpu_seconds() - started_cpu;
    let cancel = CancellationToken::new();
    let mut documents = serde_json::Map::new();
    let (mut total_wall, mut total_cpu, mut total_pages) = (0.0, 0.0, 0);
    for document in index.as_array().unwrap() {
        let id = document["id"].as_str().unwrap();
        if only
            .as_ref()
            .is_some_and(|only| !only.iter().any(|o| o == id))
        {
            continue;
        }
        let mut pages = Vec::new();
        for page in document["pages"].as_array().unwrap() {
            let number = page["page"].as_u64().unwrap() as usize;
            let image =
                image::open(pages_directory.join(id).join(format!("p{number}.png"))).unwrap();
            let cpu = cpu_seconds();
            let wall = Instant::now();
            let reading = engine.recognize(&RenderedPage::new(number - 1, image), &cancel);
            let wall = wall.elapsed().as_secs_f64();
            let cpu = cpu_seconds() - cpu;
            total_wall += wall;
            total_cpu += cpu;
            total_pages += 1;
            let entry = match reading {
                Ok(reading) => json!({
                    "page": number,
                    "text": reading.text,
                    "confidence": reading.mean_confidence,
                    "rotation": reading.rotation_degrees,
                    "lines": reading.lines,
                    "wall_s": wall,
                    "cpu_s": cpu,
                }),
                Err(error) => {
                    json!({"page": number, "error": error.to_string(), "wall_s": wall, "cpu_s": cpu})
                }
            };
            pages.push(entry);
        }
        engine.release();
        eprintln!("{id}: {} pages", pages.len());
        documents.insert(id.to_owned(), Value::Array(pages));
    }
    let summary = json!({
        "options": options,
        "pages": total_pages,
        "ocr_wall_s": total_wall,
        "ocr_cpu_s": total_cpu,
        "wall_per_page_s": total_wall / total_pages.max(1) as f64,
        "cpu_per_page_s": total_cpu / total_pages.max(1) as f64,
        "engine_load_wall_s": load_seconds,
        "engine_load_cpu_s": load_cpu,
        "harness_peak_rss_mib": peak_rss_kib() as f64 / 1024.0,
        "documents": documents,
    });
    std::fs::write(
        &options["out"],
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
    eprintln!(
        "{} pages: {:.2} s CPU and {:.2} s wall a page; peak RSS {:.0} MiB",
        total_pages,
        total_cpu / total_pages.max(1) as f64,
        total_wall / total_pages.max(1) as f64,
        peak_rss_kib() as f64 / 1024.0
    );
}

fn main() {
    let options = args();
    match std::env::args().nth(1).as_deref() {
        Some("render") => render(&options),
        Some("read") => read(&options),
        _ => {
            eprintln!("usage: ocr_eval render|read ...");
            std::process::exit(1);
        }
    }
}
