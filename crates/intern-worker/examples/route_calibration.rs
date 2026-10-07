//! Calibrates the PDF page router against what the geometry analysis finds
//! when it reads a page anyway, and measures what each step costs.
//!
//! ```sh
//! INTERN_RUNTIME_DIR=/path/to/runtime cargo run --release -p intern-worker \
//!     --features windows-native --example route_calibration -- a.pdf b.pdf ...
//! ```
//!
//! For every page of every PDF it prints one tab-separated row: the
//! document, the page, the route the router chose, every signal, what the
//! geometry analysis finds when it reads the page's characters anyway -
//! tables of two rows or more, labelled values not on their label's line, a
//! reading order that jumps back up the page (side-by-side flows) - whether
//! that makes the page one the layout route should have taken, and the
//! microseconds each step costs. `docs/document-routing.md` is built from
//! this output.

use std::path::Path;
use std::time::Instant;

use intern_worker::extract::{CancellationToken, PdfBackend, page_needs_ocr};
use intern_worker::layout::{
    BlockKind, PageLayout, PageRoute, fast_layout, geometry_layout, measure_signals, route_page,
};
use intern_worker::pdf::PdfiumBackend;

/// Times `work` over enough repetitions to be measurable, in microseconds
/// per repetition.
fn micros<T>(mut work: impl FnMut() -> T) -> f64 {
    const REPETITIONS: u32 = 20;
    let started = Instant::now();
    for _ in 0..REPETITIONS {
        std::hint::black_box(work());
    }
    started.elapsed().as_secs_f64() * 1e6 / f64::from(REPETITIONS)
}

/// What the geometry analysis found on a page.
struct Found {
    tables: usize,
    set_apart: usize,
    jumps: usize,
}

fn found(layout: &PageLayout) -> Found {
    let tables = layout
        .blocks
        .iter()
        .filter(|block| {
            block.kind == BlockKind::Table
                && block
                    .table
                    .as_ref()
                    .is_some_and(|table| table.rows.len() >= 2)
        })
        .count();
    // A value that is not on its label's line - set under it, or running
    // on below it - is one the fast route's text keeps apart from its label.
    let set_apart = layout
        .blocks
        .iter()
        .flat_map(|block| &block.fields)
        .filter(|field| match (field.key_bbox, field.value_bbox) {
            (Some(key), Some(value)) if key != value => {
                let key_height = key[3].saturating_sub(key[1]).max(1);
                value[1] >= key[3] || value[3] - value[1] > key_height * 3 / 2
            }
            _ => false,
        })
        .count();
    let body = layout
        .blocks
        .iter()
        .filter(|block| !matches!(block.kind, BlockKind::PageHeader | BlockKind::PageFooter))
        .filter_map(|block| block.bbox)
        .collect::<Vec<_>>();
    // A block that starts above the one before it by more than a line: the
    // reading order went back up the page to another column.
    let jumps = body
        .windows(2)
        .filter(|pair| pair[1][1] + (pair[0][3] - pair[0][1]).min(200) < pair[0][1])
        .count();
    Found {
        tables,
        set_apart,
        jumps,
    }
}

fn route_name(route: PageRoute) -> &'static str {
    match route {
        PageRoute::Fast => "fast",
        PageRoute::Layout => "layout",
        PageRoute::Ocr => "ocr",
        PageRoute::OcrRegions => "ocr_regions",
    }
}

fn main() {
    let runtime = std::env::var("INTERN_RUNTIME_DIR").expect("INTERN_RUNTIME_DIR holds PDFium");
    let backend = PdfiumBackend::new(runtime).expect("PDFium loads");
    println!(
        "document\tpage\troute\tchars\tsegments\timage_coverage\tinvisible\tgarbage\tcolumns\t\
         interleave\taligned_rows\tkey_values\tkey_value_grid\toverlap\tfont_sizes\trulings\timage_region\t\
         tables\tset_apart_fields\tcolumn_jumps\tneeds_layout\tsignals_us\tfast_blocks_us\t\
         geometry_us\tinspect_us\tinspect_analysis_us\tinspect_all_runs_analysis_us"
    );
    for argument in std::env::args().skip(1) {
        let path = Path::new(&argument);
        let name = path
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        // As inspection routes: runs only where the router asks for them.
        let routed_cancel = CancellationToken::new();
        let started = Instant::now();
        let Ok(routed) = backend.inspect(path, &routed_cancel) else {
            eprintln!("{name}: not readable");
            continue;
        };
        let inspect_us = started.elapsed().as_secs_f64() * 1e6;
        let routed_analysis = routed_cancel.timings().analysis_micros;
        // Every page read into runs, whatever the router says.
        let all_cancel = CancellationToken::new();
        let everything = backend
            .inspect_routed(path, &all_cancel, |_, needs_ocr| {
                if needs_ocr {
                    PageRoute::Ocr
                } else {
                    PageRoute::Layout
                }
            })
            .expect("readable the second time");
        let all_analysis = all_cancel.timings().analysis_micros;
        let pages = routed.len().max(1) as f64;
        for (page, full) in routed.iter().zip(&everything) {
            let (Some(native), Some(signals), Some(full_native)) =
                (&page.native, page.signals, &full.native)
            else {
                continue;
            };
            let route = route_page(&signals, page_needs_ocr(page));
            let signals_us =
                micros(|| measure_signals(native, &page.native_text, page.image_coverage));
            let fast_us = micros(|| fast_layout(&page.native_text, Some(native), signals));
            let geometry_us =
                micros(|| geometry_layout(full_native, Vec::new(), signals, PageRoute::Layout));
            let layout = geometry_layout(full_native, Vec::new(), signals, PageRoute::Layout);
            let found = found(&layout);
            let needs = route != PageRoute::Ocr
                && (found.tables > 0 || found.set_apart >= 2 || found.jumps > 0);
            println!(
                "{name}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}",
                page.page_index + 1,
                route_name(route),
                signals.chars,
                signals.segments,
                signals.image_coverage,
                signals.invisible,
                signals.garbage,
                signals.columns,
                signals.interleave,
                signals.aligned_rows,
                signals.key_values,
                signals.key_value_grid,
                signals.overlap,
                signals.font_sizes,
                signals.rulings,
                signals.image_region,
                found.tables,
                found.set_apart,
                found.jumps,
                needs,
                signals_us,
                fast_us,
                geometry_us,
                inspect_us / pages,
                routed_analysis as f64 / pages,
                all_analysis as f64 / pages,
            );
        }
    }
}
