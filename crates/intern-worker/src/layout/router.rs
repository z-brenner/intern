//! Which way a PDF page is read, from signals cheap enough to take on every
//! page.
//!
//! Everything here is measured from what PDFium already hands over while a
//! page's text is read: the boxes of its text segments (runs of one style on
//! one line, in content-stream order), how many of its text objects are
//! drawn invisibly, where its images and rules are, and the text itself.
//! Nothing here touches a page's characters one by one; only a page routed
//! to [`PageRoute::Layout`] or [`PageRoute::OcrRegions`] pays for that.
//!
//! The thresholds are calibrated on InternBench and the clean-room corpus;
//! `docs/document-routing.md` has the per-page measurements behind each.

use serde::{Deserialize, Serialize};

use super::geometry::TextRun;
use super::text::{is_label, split_key_value};
use super::{PageRoute, UNITS_PER_POINT};

/// What the router measured about one page. Ratios are per mille so the
/// whole layout stays integer.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RouteSignals {
    /// Characters of native text that are neither whitespace, control, nor
    /// replacement glyphs.
    pub chars: u32,
    /// PDFium's text segments.
    pub segments: u32,
    /// Per mille of the page covered by images.
    pub image_coverage: u16,
    /// Per mille of the non-whitespace characters that are U+FFFD.
    pub replacement: u16,
    /// Per mille of the page's text objects drawn invisibly (render mode 3,
    /// how an OCR text layer sits under a scan).
    pub invisible: u16,
    /// Per mille of the text's words that look like OCR errors: digits and
    /// letters confused inside a word (`INV0ICE`, `2O26`, `Mi11work`).
    pub garbage: u16,
    /// Side-by-side flows of text: 1 for a page read straight down, 2 or
    /// more where a band of whitespace runs down part of the page with text
    /// on both sides of it - text columns, or the columns of a table.
    pub columns: u8,
    /// Per mille of consecutive segments, in content-stream order, that
    /// jump from one side of that band to the other: near zero for columns
    /// written one after the other, high for columns written row by row.
    pub interleave: u16,
    /// Lines of three or more cells whose edges line up with a line next to
    /// them: the rows of a table.
    pub aligned_rows: u16,
    /// Lines that read as a label and its value.
    pub key_values: u16,
    /// Lines holding two or more labelled values side by side: the rows of
    /// a grid of fields, or signature blocks set next to each other.
    pub key_value_grid: u16,
    /// Per mille of segments drawn over another segment.
    pub overlap: u16,
    /// Distinct text heights, to the half point.
    pub font_sizes: u8,
    /// Horizontal and vertical rules.
    pub rulings: u16,
    /// Per mille of the page in the largest image that no text overlaps.
    pub image_region: u16,
    /// Clockwise rotation of the page as displayed.
    pub rotation: u16,
}

/// What a PDF backend measured about one page for the router and for the
/// layout analysis.
///
/// Boxes are `[x0, y0, x1, y1]` in tenths of a point with the origin at the
/// top left of the page's own frame - the page before its `/Rotate` is
/// applied, which is the frame its text is set in. [`to_display`] turns
/// them the way the page is shown.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct NativePage {
    /// The page's size in its own frame.
    pub width: u32,
    pub height: u32,
    /// Clockwise rotation from the frame to the page as displayed.
    pub rotation: u16,
    /// PDFium's text segments, in content-stream order.
    pub segments: Vec<[u32; 4]>,
    /// Text objects on the page, and how many of them are invisible.
    pub text_objects: u32,
    pub invisible_text_objects: u32,
    /// Image objects.
    pub images: Vec<[u32; 4]>,
    /// Thin path objects: horizontal and vertical rules.
    pub rulings: Vec<[u32; 4]>,
    /// The page's text as runs built from its characters. Empty unless the
    /// router sent the page down a route that reads its geometry.
    pub runs: Vec<TextRun>,
}

impl NativePage {
    /// The page's displayed size.
    pub fn display_size(&self) -> (u32, u32) {
        if self.rotation % 180 == 90 {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        }
    }

    /// A box in the page's frame, as displayed.
    pub fn to_display(&self, bbox: [u32; 4]) -> [u32; 4] {
        to_display(bbox, self.rotation, self.width, self.height)
    }
}

/// Turns a box in a page's own frame (`width` by `height`) clockwise by
/// `rotation` degrees, into the frame of the page as displayed.
pub fn to_display(bbox: [u32; 4], rotation: u16, width: u32, height: u32) -> [u32; 4] {
    let [x0, y0, x1, y1] = bbox;
    match rotation % 360 {
        90 => [height.saturating_sub(y1), x0, height.saturating_sub(y0), x1],
        180 => [
            width.saturating_sub(x1),
            height.saturating_sub(y1),
            width.saturating_sub(x0),
            height.saturating_sub(y0),
        ],
        270 => [y0, width.saturating_sub(x1), y1, width.saturating_sub(x0)],
        _ => bbox,
    }
}

/// Turns a box on the page as displayed back into the page's own frame
/// (`width` by `height`): the inverse of [`to_display`].
pub fn from_display(
    bbox: [u32; 4],
    rotation: u16,
    display_width: u32,
    display_height: u32,
) -> [u32; 4] {
    to_display(
        bbox,
        (360 - rotation % 360) % 360,
        display_width,
        display_height,
    )
}

/// Whether an image may hold text of its own: at least
/// [`thresholds::IMAGE_REGION`] of the page, with native text over no more
/// than a tenth of it.
pub(crate) fn is_text_region(native: &NativePage, image: &[u32; 4], page_area: f64) -> bool {
    let covered = native
        .segments
        .iter()
        .take(bounds::MAX_SEGMENTS)
        .map(|segment| intersection(image, segment))
        .sum::<f64>();
    covered <= area(image) * 0.1 && per_mille(area(image), page_area) >= thresholds::IMAGE_REGION
}

/// A box's area in tenths of a point squared.
fn area(bbox: &[u32; 4]) -> f64 {
    f64::from(bbox[2].saturating_sub(bbox[0])) * f64::from(bbox[3].saturating_sub(bbox[1]))
}

fn intersection(left: &[u32; 4], right: &[u32; 4]) -> f64 {
    let x0 = left[0].max(right[0]);
    let y0 = left[1].max(right[1]);
    let x1 = left[2].min(right[2]);
    let y1 = left[3].min(right[3]);
    if x1 <= x0 || y1 <= y0 {
        0.0
    } else {
        f64::from(x1 - x0) * f64::from(y1 - y0)
    }
}

fn per_mille(part: f64, whole: f64) -> u16 {
    if whole <= 0.0 {
        0
    } else {
        ((part / whole) * 1000.0).round().clamp(0.0, 1000.0) as u16
    }
}

/// Every signal for one page, from what the backend measured, the page's
/// native text, and the image coverage inspection already computes.
pub fn measure_signals(native: &NativePage, text: &str, image_coverage: f32) -> RouteSignals {
    let non_whitespace = text.chars().filter(|character| !character.is_whitespace());
    let considered = non_whitespace.clone().count();
    let replacements = non_whitespace
        .clone()
        .filter(|character| *character == '\u{fffd}')
        .count();
    let chars = non_whitespace
        .filter(|character| !character.is_control() && *character != '\u{fffd}')
        .count();
    // Lines, columns and rows are the page's as it is read: a page turned
    // by `/Rotate` has its text set across its own frame, so its segments
    // are turned the way it is displayed before they are measured.
    let turned;
    let segments = if native.rotation % 360 == 0 {
        &native.segments
    } else {
        turned = native
            .segments
            .iter()
            .map(|segment| native.to_display(*segment))
            .collect::<Vec<_>>();
        &turned
    };
    // A page with more text objects than the router takes on is not
    // measured for structure: it reads as one flow, and keeps its text.
    let crowded = segments.len() > bounds::MAX_SEGMENTS;
    let segments = &segments[..segments.len().min(bounds::MAX_SEGMENTS)];
    let structure = if crowded {
        Structure {
            columns: 1,
            interleave: 0,
            aligned_rows: 0,
        }
    } else {
        Structure::of(segments)
    };
    RouteSignals {
        chars: u32::try_from(chars).unwrap_or(u32::MAX),
        segments: u32::try_from(native.segments.len()).unwrap_or(u32::MAX),
        image_coverage: (f64::from(image_coverage.clamp(0.0, 1.0)) * 1000.0).round() as u16,
        replacement: per_mille(replacements as f64, considered as f64),
        invisible: per_mille(
            f64::from(native.invisible_text_objects),
            f64::from(native.text_objects),
        ),
        garbage: garbage_score(text),
        columns: structure.columns,
        interleave: structure.interleave,
        aligned_rows: structure.aligned_rows,
        key_values: key_value_lines(text),
        key_value_grid: key_value_grid_lines(text),
        overlap: if crowded { 0 } else { overlap(segments) },
        font_sizes: font_sizes(segments),
        rulings: u16::try_from(native.rulings.len()).unwrap_or(u16::MAX),
        image_region: image_region(native),
        rotation: native.rotation % 360,
    }
}

/// What the router and the layout analysis take on of a page, at most.
///
/// Each bounds what a page built to be slow can cost - the analysis of
/// where text sits compares runs with the runs around them, and a page of
/// thousands of one-character runs on one line, or a page two hundred
/// inches wide, would otherwise take minutes - and each is many times what
/// any page of InternBench's 72 documents or the generated fixtures has:
/// the routing calibration (`examples/route_calibration.rs`) measured
/// their 442 pages, and `docs/document-routing.md` has the table. A page
/// past one keeps its text and is read on the fast route.
pub mod bounds {
    /// Text objects a page's survey keeps: 22 times the corpus's densest
    /// page (226 on the ruled inspection log). A page drawn one character
    /// to an object has a few thousand.
    pub const MAX_SEGMENTS: usize = 5_000;
    /// Objects a page's survey visits, the children of forms included: a
    /// page with more is not measured at all and keeps its text, read on
    /// the fast route. Four times the text objects the survey keeps.
    pub const MAX_SURVEY_OBJECTS: usize = 20_000;
    /// Forms drawn inside forms that the survey follows, the page's own
    /// objects at depth 0: a page nested deeper is not measured.
    pub const MAX_FORM_DEPTH: usize = 16;
    /// Images a page's survey keeps, the largest first; their area still
    /// counts every image. No corpus page has more than one.
    pub const MAX_IMAGES: usize = 64;
    /// Rules a page's survey keeps: 6.7 times the corpus's most ruled page
    /// (300, the inspection log).
    pub const MAX_RULINGS: usize = 2_000;
    /// Runs a page's layout is built from: 17 times the corpus's most (226).
    pub const MAX_RUNS: usize = 4_000;
    /// Runs inspection reads ahead for a document's pages, while it has
    /// each page open: a page past them has its characters read when it is
    /// read, one page at a time, at the cost of loading it again. The
    /// corpus's largest document has 6,735 (the 100-page annual report).
    pub const MAX_DOCUMENT_RUNS: usize = 100_000;
    /// Runs on one line - within five points of each other down the page:
    /// 31 times the corpus's most (8).
    pub const MAX_RUNS_PER_LINE: usize = 250;
    /// Candidate gutters one region of a page tries, in the order they are
    /// found down it: nine times the most gutters a corpus page has (7,
    /// between the eight columns of a table).
    pub const MAX_GUTTER_CANDIDATES: usize = 64;
    /// Bands of whitespace the router counts in one part of a page: nine
    /// times the corpus's most (7).
    pub const MAX_GUTTERS: usize = 64;
    /// Points across a part of a page the router looks for gutters in: the
    /// widest page a PDF can have is two hundred inches (14,400 points); a
    /// letter page is 612, turned on its side 792.
    pub const MAX_WIDTH_POINTS: usize = 15_000;
}

/// Thresholds, each with the measurement that set it in
/// `docs/document-routing.md`.
pub mod thresholds {
    /// An invisible text layer: at least this share of the page's text
    /// objects drawn in render mode 3.
    pub const INVISIBLE_LAYER: u16 = 500;
    /// ...over a page that is a picture of a page.
    pub const FULL_PAGE_IMAGE: u16 = 900;
    /// ...whose words look like OCR errors at least this often is a bad
    /// prior OCR, and is read again.
    pub const GARBAGE_LAYER: u16 = 40;
    /// An image this large that no text overlaps may hold text of its own.
    pub const IMAGE_REGION: u16 = 150;
    /// Text this short is not enough native text to trust a page to.
    pub const REGION_MIN_CHARS: u32 = 20;
    /// Lines of aligned cells that make a page a table page.
    pub const ALIGNED_ROWS: u16 = 3;
    /// Lines of labelled values side by side that make a page a form.
    pub const KEY_VALUE_GRID: u16 = 2;
    /// Segments drawn over each other.
    pub const OVERLAP: u16 = 100;
}

/// Where a page goes.
///
/// `needs_ocr` is the scan rule pages have always been routed by
/// ([`crate::extract::page_needs_ocr`]); it wins outright. A text layer that
/// is a bad prior OCR of a full-page image is read again. Native text with
/// a large image no text overlaps gets that image read too. A page whose
/// text PDFium would run together - side by side flows, table rows,
/// labelled grids, overprinted text - is rebuilt from its geometry; every
/// other page keeps its text exactly as it was read.
pub fn route_page(signals: &RouteSignals, needs_ocr: bool) -> PageRoute {
    use thresholds::*;
    if needs_ocr {
        return PageRoute::Ocr;
    }
    if signals.invisible >= INVISIBLE_LAYER
        && signals.image_coverage >= FULL_PAGE_IMAGE
        && signals.garbage >= GARBAGE_LAYER
    {
        return PageRoute::Ocr;
    }
    if signals.image_region >= IMAGE_REGION && signals.chars >= REGION_MIN_CHARS {
        return PageRoute::OcrRegions;
    }
    if signals.columns >= 2
        || signals.aligned_rows >= ALIGNED_ROWS
        || signals.key_value_grid >= KEY_VALUE_GRID
        || signals.overlap >= OVERLAP
    {
        return PageRoute::Layout;
    }
    PageRoute::Fast
}

/// Whether a route reads the page's characters one by one.
pub fn needs_runs(route: PageRoute) -> bool {
    matches!(route, PageRoute::Layout | PageRoute::OcrRegions)
}

/// Per mille of words that look like OCR errors.
///
/// A word is a run of non-space characters with its edge punctuation
/// removed. It is suspect when a digit sits between letters (`INV0ICE`,
/// `Ca1der`), when one of the letters OCR mistakes for a digit - O, o, l,
/// I, S, B - sits among digits (`2O26`, `12OO`, `$69.9O`), when a word
/// starts with 0 or 1 and goes on in letters (`0strander`), or when its
/// case flips back and forth (`PaYMENT`). Identifiers joined by hyphens or
/// slashes are judged part by part, so `INV-2026-0042` is clean.
pub fn garbage_score(text: &str) -> u16 {
    let mut words = 0_usize;
    let mut suspect = 0_usize;
    for token in text.split_whitespace() {
        let token = token.trim_matches(|character: char| !character.is_alphanumeric());
        if token
            .chars()
            .filter(|character| character.is_alphanumeric())
            .count()
            < 2
        {
            continue;
        }
        words += 1;
        if token.split(['-', '/', '#']).any(|part| {
            suspect_part(part.trim_matches(|character: char| !character.is_alphanumeric()))
        }) {
            suspect += 1;
        }
    }
    per_mille(suspect as f64, words as f64)
}

fn suspect_part(part: &str) -> bool {
    let characters = part
        .chars()
        .filter(|character| !matches!(character, '.' | ',' | ':' | '$' | '%' | '\''))
        .collect::<Vec<_>>();
    if characters.len() < 2 {
        return false;
    }
    let digits = characters
        .iter()
        .filter(|character| character.is_ascii_digit())
        .count();
    let letters = characters
        .iter()
        .filter(|character| character.is_alphabetic())
        .count();
    let confusable = |character: &char| matches!(character, 'O' | 'o' | 'l' | 'I' | 'S' | 'B');
    if digits > 0 && letters > 0 {
        // A digit between two letters.
        if characters.windows(3).any(|window| {
            window[0].is_alphabetic() && window[1].is_ascii_digit() && window[2].is_alphabetic()
        }) {
            return true;
        }
        // A confusable letter among digits, in a word that is mostly digits.
        if digits >= 2
            && digits >= letters
            && characters.windows(2).any(|pair| {
                (pair[0].is_ascii_digit() && confusable(&pair[1]))
                    || (confusable(&pair[0]) && pair[1].is_ascii_digit())
            })
        {
            return true;
        }
        // 0strander, 1nvoice.
        if matches!(characters[0], '0' | '1')
            && characters.len() >= 4
            && characters[1..]
                .iter()
                .all(|character| character.is_alphabetic())
        {
            return true;
        }
    }
    if letters == characters.len() && letters >= 4 {
        let flips = characters
            .windows(2)
            .filter(|pair| pair[0].is_lowercase() && pair[1].is_uppercase())
            .count();
        if flips >= 2
            || (flips == 1
                && characters[0].is_uppercase()
                && characters[1].is_lowercase()
                && characters
                    .iter()
                    .skip(2)
                    .filter(|character| character.is_uppercase())
                    .count()
                    >= 2)
        {
            return true;
        }
    }
    false
}

/// Lines that read as a label and its value, or a label waiting for one.
fn key_value_lines(text: &str) -> u16 {
    let count = text
        .lines()
        .map(str::trim)
        .filter(|line| split_key_value(line).is_some() || (line.ends_with(':') && is_label(line)))
        .count();
    u16::try_from(count).unwrap_or(u16::MAX)
}

/// Lines with two or more labels on them, each a capitalised name of a few
/// words ending in a colon: `Invoice No: 4471   Date: May 1, 2026`, or
/// `Name: Ada Example   Name: Ben Example` across two signature blocks.
/// PDFium runs such a line together, so the fast route would read the
/// second label as part of the first one's value.
fn key_value_grid_lines(text: &str) -> u16 {
    let count = text.lines().filter(|line| labels_on(line) >= 2).count();
    u16::try_from(count).unwrap_or(u16::MAX)
}

/// How many labels a line holds: words ending in a colon, with the up to
/// four words before them, that together read as a label and start with a
/// capital.
fn labels_on(line: &str) -> usize {
    let words = line.split_whitespace().collect::<Vec<_>>();
    let mut labels = 0;
    let mut start = 0;
    for (index, word) in words.iter().enumerate() {
        if !word.ends_with(':') || word.len() < 2 {
            continue;
        }
        let first = (start.max(index.saturating_sub(4))..=index).find(|first| {
            words[*first].chars().next().is_some_and(char::is_uppercase)
                && is_label(&words[*first..=index].join(" "))
        });
        if let Some(first) = first
            && words[first..index]
                .iter()
                .all(|word| !word.ends_with([',', '.', ';']))
        {
            labels += 1;
            start = index + 1;
        }
    }
    labels
}

/// Per mille of segments that overlap another by at least half the smaller
/// one's area.
fn overlap(segments: &[[u32; 4]]) -> u16 {
    const MAX_COMPARED: usize = 4_000;
    let segments = &segments[..segments.len().min(MAX_COMPARED)];
    let mut order = (0..segments.len()).collect::<Vec<_>>();
    order.sort_by_key(|index| (segments[*index][0], *index));
    let mut overlapping = vec![false; segments.len()];
    for (position, first) in order.iter().enumerate() {
        let left = &segments[*first];
        for second in &order[position + 1..] {
            let right = &segments[*second];
            if right[0] >= left[2] {
                break;
            }
            let smaller = area(left).min(area(right));
            if smaller > 0.0 && intersection(left, right) >= smaller * 0.5 {
                overlapping[*first] = true;
                overlapping[*second] = true;
            }
        }
    }
    per_mille(
        overlapping.iter().filter(|flag| **flag).count() as f64,
        segments.len() as f64,
    )
}

fn font_sizes(segments: &[[u32; 4]]) -> u8 {
    let mut sizes = segments
        .iter()
        .map(|segment| (segment[3].saturating_sub(segment[1]) + 2) / 5)
        .filter(|size| *size > 0)
        .collect::<Vec<_>>();
    sizes.sort_unstable();
    sizes.dedup();
    u8::try_from(sizes.len()).unwrap_or(u8::MAX)
}

/// The largest image, per mille of the page, that text overlaps by no more
/// than a tenth of its area.
fn image_region(native: &NativePage) -> u16 {
    let page = f64::from(native.width) * f64::from(native.height);
    native
        .images
        .iter()
        .take(bounds::MAX_IMAGES)
        .filter(|image| {
            let covered = native
                .segments
                .iter()
                .take(bounds::MAX_SEGMENTS)
                .map(|segment| intersection(image, segment))
                .sum::<f64>();
            covered <= area(image) * 0.1
        })
        .map(|image| per_mille(area(image), page))
        .max()
        .unwrap_or(0)
}

/// The side-by-side structure of a page, from its segments alone.
struct Structure {
    columns: u8,
    interleave: u16,
    aligned_rows: u16,
}

/// A row of segment cells: `(x0, x1)` of each cell left to right, and the
/// segments in it.
struct SegmentRow {
    y0: u32,
    y1: u32,
    cells: Vec<(u32, u32)>,
    members: Vec<usize>,
}

impl Structure {
    fn of(segments: &[[u32; 4]]) -> Self {
        let rows = segment_rows(segments);
        Self {
            aligned_rows: aligned_rows(&rows),
            ..side_by_side(segments, &rows)
        }
    }
}

/// Groups segments into rows by vertical overlap, and each row's segments
/// into cells: segments closer than most of a line's height are one cell.
fn segment_rows(segments: &[[u32; 4]]) -> Vec<SegmentRow> {
    let mut order = (0..segments.len())
        .filter(|index| {
            let segment = segments[*index];
            segment[2] > segment[0] && segment[3] > segment[1]
        })
        .collect::<Vec<_>>();
    let center = |index: usize| segments[index][1] + segments[index][3];
    order.sort_by_key(|index| (center(*index), segments[*index][0], *index));
    let mut rows: Vec<SegmentRow> = Vec::new();
    for index in order {
        let segment = segments[index];
        let joins = rows.last().is_some_and(|row| {
            let middle = (segment[1] + segment[3]) / 2;
            let row_middle = (row.y0 + row.y1) / 2;
            middle >= row.y0
                && middle <= row.y1
                && row_middle >= segment[1]
                && row_middle <= segment[3]
        });
        if joins {
            let row = rows.last_mut().expect("checked above");
            row.y0 = row.y0.min(segment[1]);
            row.y1 = row.y1.max(segment[3]);
            row.members.push(index);
        } else {
            rows.push(SegmentRow {
                y0: segment[1],
                y1: segment[3],
                cells: Vec::new(),
                members: vec![index],
            });
        }
    }
    for row in &mut rows {
        row.members
            .sort_by_key(|index| (segments[*index][0], *index));
        let height = row.y1 - row.y0;
        for index in &row.members {
            let segment = segments[*index];
            match row.cells.last_mut() {
                Some(cell) if segment[0] <= cell.1 + height * 4 / 5 => {
                    cell.1 = cell.1.max(segment[2])
                }
                _ => row.cells.push((segment[0], segment[2])),
            }
        }
    }
    rows
}

/// Rows of three or more cells, two of which line up with a cell of the
/// row above or below.
fn aligned_rows(rows: &[SegmentRow]) -> u16 {
    let lines_up = |row: &SegmentRow, other: &SegmentRow| {
        let tolerance = ((row.y1 - row.y0) * 4 / 5).max(30);
        row.cells
            .iter()
            .filter(|cell| {
                other.cells.iter().any(|candidate| {
                    cell.0.abs_diff(candidate.0) <= tolerance
                        || cell.1.abs_diff(candidate.1) <= tolerance
                })
            })
            .count()
            >= 2
    };
    let count = rows
        .iter()
        .enumerate()
        .filter(|(index, row)| {
            row.cells.len() >= 3
                && ((*index > 0 && lines_up(row, &rows[index - 1]))
                    || rows.get(index + 1).is_some_and(|next| lines_up(row, next)))
        })
        .count();
    u16::try_from(count).unwrap_or(u16::MAX)
}

/// Bands of whitespace down the page. The page is first divided where a
/// gap of more than a line and a quarter runs across it; in each part, a
/// band that at most one row in eight crosses, with text on both sides in
/// at least three rows, is a column break. `interleave` is measured over
/// the part with the most rows on both sides of its band.
fn side_by_side(segments: &[[u32; 4]], rows: &[SegmentRow]) -> Structure {
    let mut heights = rows
        .iter()
        .map(|row| f64::from(row.y1 - row.y0))
        .collect::<Vec<_>>();
    let height = super::text::median(&mut heights).unwrap_or(100.0);
    let mut parts: Vec<&[SegmentRow]> = Vec::new();
    let mut start = 0;
    for index in 1..=rows.len() {
        let breaks = index == rows.len()
            || f64::from(rows[index].y0) - f64::from(rows[index - 1].y1) > height * 1.25;
        if breaks {
            parts.push(&rows[start..index]);
            start = index;
        }
    }
    let mut columns = 1_u8;
    let mut interleave = 0_u16;
    let mut best_rows = 0_usize;
    for part in parts {
        if part.len() < 3 {
            continue;
        }
        let gutters = gutters(part, height);
        columns = columns.max(u8::try_from(gutters.len() + 1).unwrap_or(u8::MAX));
        for (start, end, both) in gutters {
            if both <= best_rows {
                continue;
            }
            best_rows = both;
            let mut sides = part
                .iter()
                .flat_map(|row| row.members.iter().copied())
                .filter_map(|index| {
                    let segment = segments[index];
                    if f64::from(segment[2]) <= start + 50.0 {
                        Some((index, false))
                    } else if f64::from(segment[0]) >= end - 50.0 {
                        Some((index, true))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            sides.sort_by_key(|(index, _)| *index);
            let jumps = sides
                .windows(2)
                .filter(|pair| pair[0].1 != pair[1].1)
                .count();
            interleave = per_mille(jumps as f64, sides.len().saturating_sub(1) as f64);
        }
    }
    Structure {
        columns,
        interleave,
        aligned_rows: 0,
    }
}

/// The column breaks in a part of a page: `(start, end, rows with text on
/// both sides)` for each band of whitespace at least six points wide.
fn gutters(rows: &[SegmentRow], height: f64) -> Vec<(f64, f64, usize)> {
    let left = rows
        .iter()
        .flat_map(|row| row.cells.iter().map(|cell| cell.0))
        .min()
        .unwrap_or(0);
    let right = rows
        .iter()
        .flat_map(|row| row.cells.iter().map(|cell| cell.1))
        .max()
        .unwrap_or(0);
    let bin = UNITS_PER_POINT as u32;
    let bins = ((right.saturating_sub(left)) / bin + 1) as usize;
    if bins > bounds::MAX_WIDTH_POINTS {
        return Vec::new();
    }
    // How many rows cover each point across, counted where each cell starts
    // and ends. A row's cells run left to right without overlapping, but two
    // can share the point one ends and the next starts in; the second then
    // starts after it, so a row counts once at every point it covers.
    let mut steps = vec![0_i64; bins + 1];
    for row in rows {
        let mut covered_to = 0;
        for cell in &row.cells {
            let first = (((cell.0 - left) / bin) as usize).clamp(covered_to, bins);
            let last = ((cell.1 - left).div_ceil(bin) as usize).min(bins);
            if first < last {
                steps[first] += 1;
                steps[last] -= 1;
                covered_to = last;
            }
        }
    }
    let mut coverage = Vec::with_capacity(bins);
    let mut running = 0_i64;
    for step in &steps[..bins] {
        running += step;
        coverage.push(u32::try_from(running.max(0)).unwrap_or(u32::MAX));
    }
    let tolerance = (rows.len() as u32 / 8).max(1);
    let minimum = (height * 0.9).max(60.0);
    let mut found = Vec::new();
    let mut start: Option<usize> = None;
    // One step past the last bin closes a band that runs to the edge.
    let lows = coverage
        .iter()
        .map(|count| *count <= tolerance)
        .chain(std::iter::once(false));
    for (index, low) in lows.enumerate() {
        if found.len() >= bounds::MAX_GUTTERS {
            break;
        }
        match (low, start) {
            (true, None) => start = Some(index),
            (false, Some(first)) => {
                let x0 = f64::from(left) + (first as f64) * f64::from(bin);
                let x1 = f64::from(left) + (index as f64) * f64::from(bin);
                if first > 0 && index < bins && x1 - x0 >= minimum {
                    let both = rows
                        .iter()
                        .filter(|row| {
                            row.cells.iter().any(|cell| f64::from(cell.1) <= x0 + 50.0)
                                && row.cells.iter().any(|cell| f64::from(cell.0) >= x1 - 50.0)
                        })
                        .count();
                    let left_rows = rows
                        .iter()
                        .filter(|row| row.cells.iter().any(|cell| f64::from(cell.1) <= x0 + 50.0))
                        .count();
                    let right_rows = rows
                        .iter()
                        .filter(|row| row.cells.iter().any(|cell| f64::from(cell.0) >= x1 - 50.0))
                        .count();
                    if left_rows >= 3 && right_rows >= 3 {
                        found.push((x0, x1, both));
                    }
                }
                start = None;
            }
            _ => {}
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(x: u32, y: u32, width: u32) -> [u32; 4] {
        [x * 10, y * 10, (x + width) * 10, (y + 9) * 10]
    }

    #[test]
    fn garbage_finds_ocr_confusions_and_spares_identifiers() {
        let corrupted = "Wexcornbe Mi11work Co. INV0ICE Invoice Date: O3/lO/2O26 \
                         Bill To: 0strander Hornebuilders LLC, 12OO Ca1der Way $69.9O T0TAL";
        let clean = "Invoice INV-2026-0042 dated 03/10/2026 for $1,537.80 under PO 4471 \
                     to Wexcombe Millwork Co., Suite 4, 1200 Calder Way";

        assert!(
            garbage_score(corrupted) >= 300,
            "{}",
            garbage_score(corrupted)
        );
        assert_eq!(garbage_score(clean), 0);
    }

    #[test]
    fn row_by_row_columns_interleave_and_column_by_column_ones_do_not() {
        let mut by_row = Vec::new();
        let mut by_column = Vec::new();
        for row in 0..10 {
            by_row.push(segment(54, 100 + row * 12, 200));
            by_row.push(segment(320, 100 + row * 12, 200));
        }
        for row in 0..10 {
            by_column.push(segment(54, 100 + row * 12, 200));
        }
        for row in 0..10 {
            by_column.push(segment(320, 100 + row * 12, 200));
        }
        let page = |segments: Vec<[u32; 4]>| NativePage {
            width: 6120,
            height: 7920,
            segments,
            ..NativePage::default()
        };

        let interleaved = measure_signals(&page(by_row), "", 0.0);
        let ordered = measure_signals(&page(by_column), "", 0.0);

        assert_eq!(interleaved.columns, 2);
        assert!(interleaved.interleave >= 900, "{interleaved:?}");
        assert_eq!(ordered.columns, 2);
        assert!(ordered.interleave <= 100, "{ordered:?}");
    }

    #[test]
    fn a_table_has_aligned_rows_and_prose_does_not() {
        let mut table = Vec::new();
        for row in 0..5 {
            table.push(segment(54, 100 + row * 14, 80));
            table.push(segment(250, 100 + row * 14, 30));
            table.push(segment(450, 100 + row * 14, 60));
        }
        let prose = (0..5)
            .map(|row| segment(54, 100 + row * 12, 500))
            .collect::<Vec<_>>();
        let measure = |segments| {
            measure_signals(
                &NativePage {
                    width: 6120,
                    height: 7920,
                    segments,
                    ..NativePage::default()
                },
                "",
                0.0,
            )
        };

        assert_eq!(measure(table).aligned_rows, 5);
        let prose = measure(prose);
        assert_eq!(prose.aligned_rows, 0);
        assert_eq!(prose.columns, 1);
    }

    /// A landscape table stored as a portrait page turned a quarter is set
    /// down the page's own frame; measured as displayed, it is the same
    /// table, and the page is routed as one.
    #[test]
    fn a_quarter_turned_table_is_measured_as_displayed() {
        let mut displayed = Vec::new();
        for row in 0..5 {
            displayed.push(segment(54, 100 + row * 14, 80));
            displayed.push(segment(250, 100 + row * 14, 30));
            displayed.push(segment(450, 100 + row * 14, 60));
        }
        let upright = measure_signals(
            &NativePage {
                width: 7920,
                height: 6120,
                segments: displayed.clone(),
                ..NativePage::default()
            },
            "",
            0.0,
        );
        let turned = measure_signals(
            &NativePage {
                width: 6120,
                height: 7920,
                rotation: 90,
                segments: displayed
                    .iter()
                    .map(|segment| from_display(*segment, 90, 7920, 6120))
                    .collect(),
                ..NativePage::default()
            },
            "",
            0.0,
        );

        assert_eq!(upright.aligned_rows, 5);
        assert_eq!(
            RouteSignals {
                rotation: 0,
                ..turned
            },
            upright
        );
        assert_eq!(route_page(&turned, false), PageRoute::Layout);
    }

    #[test]
    fn a_grid_of_labelled_values_is_told_from_one_label_per_line() {
        let grid = "Invoice No: INV-20417   Date: March 4, 2026\n\
                    By: /s/ Ada Example   By: /s/ Ben Example\n\
                    Name: Ada Example   Name: Ben Example\n\
                    Time: 12:01 a.m. standard time";
        assert_eq!(key_value_grid_lines(grid), 3);
        let single = "Invoice Date: January 5, 2026\nBill To: Contoso Worldwide, Inc.\n\
                      The tenant will pay the following: rent and fees: monthly.";
        assert_eq!(key_value_grid_lines(single), 0);
    }

    /// A page with more text objects than the router takes on reads as one
    /// flow, measured no further: two columns of one-character objects
    /// written row by row would otherwise cost a comparison of every one
    /// with every other.
    #[test]
    fn a_crowded_page_is_not_measured_for_structure() {
        let mut segments = Vec::new();
        for row in 0..120 {
            for column in 0..45 {
                let x = 54 + column * 5 + if column >= 22 { 100 } else { 0 };
                segments.push([
                    x * 10,
                    (100 + row * 5) * 10,
                    (x + 4) * 10,
                    (104 + row * 5) * 10,
                ]);
            }
        }
        assert!(segments.len() > bounds::MAX_SEGMENTS);
        let page = NativePage {
            width: 6120,
            height: 7920,
            segments,
            ..NativePage::default()
        };

        let signals = measure_signals(&page, "", 0.0);

        assert_eq!(signals.columns, 1);
        assert_eq!(signals.aligned_rows, 0);
        assert_eq!(route_page(&signals, false), PageRoute::Fast);
    }

    /// Gutters are counted from where each row's cells start and end; two
    /// columns are two columns however many rows there are.
    #[test]
    fn gutters_count_columns_however_tall_the_page() {
        let mut segments = Vec::new();
        for row in 0..400 {
            segments.push(segment(54, 100 + row * 12, 200));
            segments.push(segment(320, 100 + row * 12, 200));
        }
        let page = NativePage {
            width: 6120,
            height: 79_200,
            segments,
            ..NativePage::default()
        };

        assert_eq!(measure_signals(&page, "", 0.0).columns, 2);
    }

    #[test]
    fn routes_follow_the_signals() {
        let plain = RouteSignals {
            chars: 2_000,
            segments: 60,
            columns: 1,
            ..RouteSignals::default()
        };
        assert_eq!(route_page(&plain, false), PageRoute::Fast);
        assert_eq!(route_page(&plain, true), PageRoute::Ocr);
        assert_eq!(
            route_page(
                &RouteSignals {
                    columns: 2,
                    ..plain
                },
                false
            ),
            PageRoute::Layout
        );
        assert_eq!(
            route_page(
                &RouteSignals {
                    aligned_rows: 4,
                    ..plain
                },
                false
            ),
            PageRoute::Layout
        );
        assert_eq!(
            route_page(
                &RouteSignals {
                    key_values: 12,
                    ..plain
                },
                false
            ),
            PageRoute::Fast,
            "one label to a line reads well as it is"
        );
        assert_eq!(
            route_page(
                &RouteSignals {
                    key_value_grid: 2,
                    ..plain
                },
                false
            ),
            PageRoute::Layout
        );
        let bad_layer = RouteSignals {
            invisible: 1_000,
            image_coverage: 1_000,
            garbage: 120,
            ..plain
        };
        assert_eq!(route_page(&bad_layer, false), PageRoute::Ocr);
        assert_eq!(
            route_page(
                &RouteSignals {
                    garbage: 5,
                    ..bad_layer
                },
                false
            ),
            PageRoute::Fast,
            "a clean invisible layer is kept"
        );
        assert_eq!(
            route_page(
                &RouteSignals {
                    image_region: 300,
                    ..plain
                },
                false
            ),
            PageRoute::OcrRegions
        );
    }

    #[test]
    fn boxes_turn_with_the_page() {
        // A 100 x 200 frame; a box at its top left.
        let bbox = [0, 0, 10, 20];
        assert_eq!(to_display(bbox, 0, 100, 200), bbox);
        // Turned clockwise, the top left goes to the top right.
        assert_eq!(to_display(bbox, 90, 100, 200), [180, 0, 200, 10]);
        assert_eq!(to_display(bbox, 180, 100, 200), [90, 180, 100, 200]);
        assert_eq!(to_display(bbox, 270, 100, 200), [0, 90, 20, 100]);
    }

    #[test]
    fn an_image_no_text_overlaps_is_a_region_and_a_scan_under_text_is_not() {
        let mut page = NativePage {
            width: 6120,
            height: 7920,
            segments: vec![segment(54, 100, 500)],
            images: vec![[540, 5000, 5580, 7000]],
            ..NativePage::default()
        };
        assert!(image_region(&page) >= 200);
        page.segments.push([600, 5100, 5000, 6900]);
        assert_eq!(image_region(&page), 0);
    }
}
