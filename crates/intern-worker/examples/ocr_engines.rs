//! Reads InternBench's scanned pages with one OCR engine and scores what it
//! read against the text drawn on them: the measurements in `docs/ocr.md`.
//!
//! ```sh
//! INTERN_RUNTIME_DIR=/path/to/runtime cargo run --release -p intern-worker \
//!     --features native-pdfium,onnx-ocr --example ocr_engines -- \
//!     --engine indra --corpus bench/generated --gold bench/gold.json \
//!     [--only id,id] [--threads 2] [--degrade 3] [--scale 0.667] [--dump texts.json]
//! ```
//!
//! `--engine` is `indra`, `paddle` or `indra+paddle`: Indra, with PP-OCR
//! reading again the pages Indra is not sure of, as the worker reads by
//! default (`second_opinion`). Run one engine per process: ONNX Runtime binds once per
//! process, and the peak memory printed at the end is the process's.
//!
//! Every scanned document of the gold (one with `ocr_truth`) is extracted
//! through the worker's own readers - PDFium at 300 DPI for a PDF, the
//! image reader for a PNG or TIFF - so each engine is handed exactly the
//! pixels the worker hands it, through its own `OcrBackend`. Pages are read
//! one at a time on `--threads` threads (2 by default, as the worker reads
//! each page), so the processor time charged to each page is its own.
//!
//! `--degrade N` reads every page as a scan made at 1/N of the resolution:
//! the page is shrunk N times and enlarged back, both with a triangle filter,
//! before the engine sees it, as a low-resolution scan inside a PDF reaches
//! OCR after PDFium renders it at 300 DPI. InternBench's own pages are at
//! 200-300 DPI, where both ONNX engines find every critical field; `--degrade
//! 3` (100 DPI) is where they part.
//!
//! `--scale F` hands the engine the page resized by F (0.667: as if
//! rendered at 200 DPI instead of 300) and maps the boxes it reports back
//! onto the page.
//!
//! Two texts are scored, with intern-bench's own OCR measure: the engine's
//! (`OcrResult::text`) and the page's as the worker sends it (after the
//! layout analysis). One row per document, then the totals, are printed as
//! tab-separated values; `--dump` writes every page's texts as JSON.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use intern_bench::gold::GoldFile;
use intern_bench::ocr::{OcrMeasure, measure};
use intern_engine::{DocumentSource, PageOrigin, SourcePage};
use intern_worker::extract::{
    CancellationToken, ExtractionError, OcrBackend, OcrResult, RenderedPage, extract_image,
    extract_pdf,
};
use intern_worker::indra::{IndraAssets, IndraOcr};
use intern_worker::limits::ResourceLimits;
use intern_worker::paddle::{PaddleAssets, PaddleConfig, PaddleOcr, ThreadBudget};
use intern_worker::pdf::PdfiumBackend;
use intern_worker::second_opinion::{SecondOpinion, SharedOcr};

/// Processor time this process has used, all threads, in seconds (Linux).
fn cpu_seconds() -> f64 {
    let Ok(stat) = std::fs::read_to_string("/proc/self/stat") else {
        return 0.0;
    };
    // Fields after the command name, which is in parentheses.
    let fields: Vec<&str> = stat
        .rsplit_once(')')
        .map_or("", |(_, rest)| rest)
        .split_whitespace()
        .collect();
    let ticks = |index: usize| {
        fields
            .get(index)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    // utime and stime are fields 14 and 15 of the line, 12 and 13 here; the
    // kernel's USER_HZ is 100 on every Linux this runs on.
    (ticks(11) + ticks(12)) / 100.0
}

/// The process's peak resident memory, in MB (Linux).
fn peak_rss_mb() -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("VmHWM:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|kb| kb.parse::<f64>().ok())
        })
        .map_or(0.0, |kb| kb / 1024.0)
}

/// One page as the engine read it.
#[derive(Clone, Debug, serde::Serialize)]
struct PageRead {
    page: usize,
    engine_text: String,
    page_text: String,
    confidence: f32,
    rotation: u16,
    cpu_s: f64,
    wall_s: f64,
}

/// Wraps an engine and keeps what it read of each page and what it cost.
struct Recording<'e> {
    engine: &'e (dyn OcrBackend + Sync),
    degrade: u32,
    scale: f64,
    reads: Mutex<BTreeMap<usize, (OcrResult, f64, f64)>>,
}

impl OcrBackend for Recording<'_> {
    fn recognize(
        &self,
        page: &RenderedPage,
        cancel: &CancellationToken,
    ) -> Result<OcrResult, ExtractionError> {
        let degraded;
        let page = if self.degrade > 1 {
            let (width, height) = (page.image.width(), page.image.height());
            let small = page.image.resize_exact(
                (width / self.degrade).max(1),
                (height / self.degrade).max(1),
                image::imageops::FilterType::Triangle,
            );
            degraded = RenderedPage::new(
                page.page_index,
                small.resize_exact(width, height, image::imageops::FilterType::Triangle),
            );
            &degraded
        } else {
            page
        };
        let scaled;
        let page = if self.scale != 1.0 {
            let size = |side: u32| ((f64::from(side) * self.scale).round() as u32).max(1);
            scaled = RenderedPage::new(
                page.page_index,
                page.image.resize_exact(
                    size(page.image.width()),
                    size(page.image.height()),
                    image::imageops::FilterType::Triangle,
                ),
            );
            &scaled
        } else {
            page
        };
        let (cpu, wall) = (cpu_seconds(), Instant::now());
        let mut result = self.engine.recognize(page, cancel)?;
        if self.scale != 1.0 {
            let back = |bbox: [u32; 4]| bbox.map(|v| (f64::from(v) / self.scale).round() as u32);
            for line in &mut result.lines {
                line.bbox = back(line.bbox);
            }
            for block in &mut result.blocks {
                block.bbox = back(block.bbox);
            }
        }
        let costs = (cpu_seconds() - cpu, wall.elapsed().as_secs_f64());
        self.reads
            .lock()
            .unwrap()
            .insert(page.page_index, (result.clone(), costs.0, costs.1));
        Ok(result)
    }

    /// One page at a time, so each page's processor time is its own.
    fn concurrency(&self) -> usize {
        1
    }
}

struct Args {
    engine: String,
    corpus: PathBuf,
    gold: PathBuf,
    only: Option<Vec<String>>,
    threads: usize,
    degrade: u32,
    scale: f64,
    dump: Option<PathBuf>,
}

fn args() -> Args {
    let mut args = Args {
        engine: "indra".into(),
        corpus: "bench/generated".into(),
        gold: "bench/gold.json".into(),
        only: None,
        threads: 2,
        degrade: 1,
        scale: 1.0,
        dump: None,
    };
    let mut iter = std::env::args().skip(1);
    while let Some(flag) = iter.next() {
        let mut value = || {
            iter.next()
                .unwrap_or_else(|| panic!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--engine" => args.engine = value(),
            "--corpus" => args.corpus = value().into(),
            "--gold" => args.gold = value().into(),
            "--only" => args.only = Some(value().split(',').map(str::to_owned).collect()),
            "--threads" => args.threads = value().parse().expect("--threads takes a number"),
            "--degrade" => args.degrade = value().parse().expect("--degrade takes a number"),
            "--scale" => args.scale = value().parse().expect("--scale takes a number"),
            "--dump" => args.dump = Some(value().into()),
            other => panic!("unknown argument {other}"),
        }
    }
    args
}

/// PP-OCR reading one page at a time on `threads` threads.
fn paddle(runtime: &Path, threads: usize) -> Result<SharedOcr, ExtractionError> {
    let config = PaddleConfig {
        intra_threads: threads,
        workers: 1,
        ..PaddleConfig::default()
    };
    let engine = PaddleOcr::with_config(PaddleAssets::in_directory(runtime), config)?;
    Ok(Box::new(engine))
}

fn engine(name: &str, runtime: &Path, threads: usize) -> SharedOcr {
    let budget = ThreadBudget {
        workers: 1,
        intra_threads: threads,
    };
    let indra =
        || IndraOcr::with_budget(IndraAssets::in_directory(runtime), budget).expect("Indra loads");
    match name {
        "indra" => Box::new(indra()),
        "paddle" => paddle(runtime, threads).expect("PP-OCR loads"),
        "indra+paddle" => {
            let runtime = runtime.to_path_buf();
            Box::new(SecondOpinion::new(
                Box::new(indra()),
                Box::new(move || paddle(&runtime, threads)),
                Box::new(|error| panic!("PP-OCR did not load: {error}")),
            ))
        }
        other => panic!("unknown engine {other}"),
    }
}

fn add(total: &mut OcrMeasure, part: &OcrMeasure) {
    total.accumulate(part);
}

fn percent(distance: usize, length: usize) -> String {
    if length == 0 {
        "-".into()
    } else {
        format!("{:.2}%", distance as f64 * 100.0 / length as f64)
    }
}

fn row(name: &str, m: &OcrMeasure) -> String {
    format!(
        "{name}\t{}\t{}\t{}/{}\t{}/{}\t{}/{}",
        percent(m.char_distance, m.truth_chars),
        percent(m.word_distance, m.truth_words),
        m.dates_found,
        m.dates_total,
        m.names_found,
        m.names_total,
        m.identifiers_found,
        m.identifiers_total
    )
}

fn main() {
    let args = args();
    let runtime = PathBuf::from(
        std::env::var_os("INTERN_RUNTIME_DIR").expect("INTERN_RUNTIME_DIR names the runtime"),
    );
    let (gold, _) = GoldFile::load(&args.gold).expect("the gold loads");
    let engine = engine(&args.engine, &runtime, args.threads);
    let pdf = PdfiumBackend::new(&runtime).expect("PDFium loads");
    let limits = ResourceLimits::default();
    let cancel = CancellationToken::new();

    let mut engine_total = OcrMeasure::default();
    let mut page_total = OcrMeasure::default();
    let (mut cpu_total, mut wall_total, mut pages_total) = (0.0, 0.0, 0_usize);
    let mut dump: BTreeMap<String, Vec<PageRead>> = BTreeMap::new();
    println!(
        "document\tengine CER\tWER\tdates\tnames\tidentifiers\tpage CER\tWER\tdates\tnames\tidentifiers\tpages\tCPU s/page\twall s/page"
    );
    for document in &gold.documents {
        let Some(truth) = &document.ocr_truth else {
            continue;
        };
        if let Some(only) = &args.only
            && !only.contains(&document.id)
        {
            continue;
        }
        let path = args.corpus.join(&document.file);
        let recording = Recording {
            engine: engine.as_ref(),
            degrade: args.degrade,
            scale: args.scale,
            reads: Mutex::new(BTreeMap::new()),
        };
        let extracted = if path.extension().is_some_and(|ext| ext == "pdf") {
            extract_pdf(&path, &pdf, &recording, &limits, &cancel)
        } else {
            extract_image(&path, &recording, &limits, &cancel)
        }
        .unwrap_or_else(|error| panic!("{} did not extract: {error}", document.id));
        let reads = recording.reads.into_inner().unwrap();
        let mut engine_pages = Vec::new();
        let mut worker_pages = Vec::new();
        let mut rows = Vec::new();
        let (mut cpu, mut wall) = (0.0, 0.0);
        for page in &extracted.pages {
            let ocr = reads.get(&(page.page_number - 1));
            // A page OCR never saw (a native page of a mixed document) reads
            // as its text layer on both sides.
            let engine_text = ocr.map_or(page.text.clone(), |(result, _, _)| result.text.clone());
            let origin = if ocr.is_some() {
                PageOrigin::Ocr
            } else {
                PageOrigin::Native
            };
            let mut engine_page = SourcePage::new(page.page_number, engine_text.clone(), origin);
            engine_page.ocr_confidence =
                ocr.map(|(result, _, _)| result.mean_confidence.round() as u32);
            engine_pages.push(engine_page);
            let mut worker_page = SourcePage::new(page.page_number, page.text.clone(), origin);
            worker_page.ocr_confidence = page.ocr_confidence.map(|value| value.round() as u32);
            worker_pages.push(worker_page);
            if let Some((result, page_cpu, page_wall)) = ocr {
                cpu += page_cpu;
                wall += page_wall;
                rows.push(PageRead {
                    page: page.page_number,
                    engine_text,
                    page_text: page.text.clone(),
                    confidence: result.mean_confidence,
                    rotation: result.rotation_degrees,
                    cpu_s: *page_cpu,
                    wall_s: *page_wall,
                });
            }
        }
        let engine_measure = measure(truth, &DocumentSource::from_pages(engine_pages));
        let page_measure = measure(truth, &DocumentSource::from_pages(worker_pages));
        let read = rows.len().max(1) as f64;
        println!(
            "{}\t{}\t{}\t{:.3}\t{:.3}",
            row(&document.id, &engine_measure),
            row("", &page_measure).trim_start_matches('\t'),
            rows.len(),
            cpu / read,
            wall / read
        );
        add(&mut engine_total, &engine_measure);
        add(&mut page_total, &page_measure);
        cpu_total += cpu;
        wall_total += wall;
        pages_total += rows.len();
        dump.insert(document.id.clone(), rows);
    }
    let pages = pages_total.max(1) as f64;
    println!(
        "{}\t{}\t{}\t{:.3}\t{:.3}",
        row("ALL", &engine_total),
        row("", &page_total).trim_start_matches('\t'),
        pages_total,
        cpu_total / pages,
        wall_total / pages
    );
    println!("peak RSS\t{:.0} MB", peak_rss_mb());
    if let Some(path) = &args.dump {
        std::fs::write(path, serde_json::to_vec_pretty(&dump).unwrap()).expect("the dump writes");
    }
}
