//! What a page says, as blocks in the order a person reads them.
//!
//! Every reader hands back page text, and that text is what the engine has
//! always read. A page's layout is the same page as structure: headings,
//! paragraphs, list items, tables with their rows and cells, and key-value
//! pairs, each with a stable id (`p{page}.b{n}`, `n` counting blocks in
//! reading order), where it sits on the page when the reader knows, where
//! its text came from, and how sure OCR was of it. It is what a later stage
//! can index and cite instead of quoting text back.
//!
//! Geometry is in tenths of a PDF point with the origin at the top left of
//! the page as it is displayed, so everything here is an integer, keeps
//! `Eq`, and serialises the same way every time. OCR pages map pixels to
//! points by the resolution they were rendered at. Confidence is 0-100.
//!
//! A PDF page is routed before it is read ([`PageRoute`]): most pages keep
//! PDFium's text exactly as it has always been read and get blocks built
//! cheaply from it; a page whose text PDFium would read in the wrong order,
//! or whose tables and labelled values it would run together, is rebuilt
//! from the geometry of its characters; a page with no usable text is read
//! by OCR. [`router`] holds the signals and the thresholds, and
//! `docs/document-routing.md` the measurements behind them.

use serde::{Deserialize, Serialize};

use crate::extract::{ExtractedPage, OcrLine, OcrResult, PageSource, PdfPageInspection};

mod geometry;
pub mod router;
mod text;

pub use geometry::{TextRun, analyze_runs};
pub use router::{NativePage, RouteSignals, measure_signals, route_page};
pub use text::{blocks_from_lines, blocks_from_text};

/// Tenths of a point in one PDF point.
pub const UNITS_PER_POINT: f64 = 10.0;

/// One page as blocks in reading order.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageLayout {
    /// The page's displayed width, in tenths of a point.
    pub width: u32,
    /// The page's displayed height, in tenths of a point.
    pub height: u32,
    pub route: PageRoute,
    /// What the router measured, kept so a benchmark can say why a page
    /// went the way it did.
    #[serde(default)]
    pub signals: RouteSignals,
    pub blocks: Vec<LayoutBlock>,
}

/// How a page was read.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PageRoute {
    /// The reader's own text, exactly as it has always been read, with
    /// blocks built cheaply from it. Every reader that is not a PDF reads
    /// this way.
    #[default]
    Fast,
    /// Rebuilt from the geometry of the page's characters: reading order
    /// from its columns, tables from aligned rows, labelled values paired.
    Layout,
    /// Read by OCR: there was no text worth keeping.
    Ocr,
    /// Native text, plus OCR of an image on the page that may hold text.
    OcrRegions,
}

/// What kind of thing a block is.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Heading,
    Paragraph,
    ListItem,
    Table,
    KeyValue,
    Caption,
    PageHeader,
    PageFooter,
    Other,
}

/// Where a block's text came from.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TextSource {
    /// The document's own text: a PDF's text layer, an Office file, a sheet,
    /// an email, a text file.
    #[default]
    Native,
    Ocr,
}

/// One block of a page.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutBlock {
    /// `p{page}.b{n}`, `n` counting from 1 in reading order on the page.
    pub id: String,
    pub kind: BlockKind,
    /// The block's text exactly as the page's text carries it. A table is
    /// its rows as `| a | b |` lines; a key-value block its pairs as
    /// `Key: value` lines.
    pub text: String,
    /// `[x0, y0, x1, y1]`, or none for a reader that knows no geometry.
    pub bbox: Option<[u32; 4]>,
    /// Heading level, 1 the largest, where it can be told.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// The id of the heading this block falls under, on this page or an
    /// earlier one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    pub source: TextSource,
    /// OCR: the mean confidence of the block's lines. None for native text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    pub lines: Vec<LayoutLine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<LayoutTable>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<KeyValue>,
}

/// One line of a block.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutLine {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[u32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
}

/// A table's rows, in reading order.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutTable {
    pub rows: Vec<LayoutRow>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutRow {
    /// `p{page}.b{n}.r{row}`.
    pub id: String,
    pub cells: Vec<LayoutCell>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutCell {
    /// `p{page}.b{n}.r{row}.c{column}`.
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[u32; 4]>,
    /// Whether the cell is in a header row.
    #[serde(default)]
    pub header: bool,
}

/// A labelled value: `Invoice date` and `March 4, 2026`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyValue {
    /// `p{page}.b{n}.f{field}`.
    pub id: String,
    /// The label without its trailing colon.
    pub key: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_bbox: Option<[u32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_bbox: Option<[u32; 4]>,
}

impl LayoutBlock {
    /// A block with no id yet: [`number_blocks`] gives every block its id
    /// once the page's reading order is settled.
    pub fn new(kind: BlockKind, text: impl Into<String>, source: TextSource) -> Self {
        Self {
            id: String::new(),
            kind,
            text: text.into(),
            bbox: None,
            level: None,
            section: None,
            source,
            confidence: None,
            lines: Vec::new(),
            table: None,
            fields: Vec::new(),
        }
    }
}

impl PageLayout {
    /// A page read by a reader that knows no geometry: the blocks of its
    /// text, on the fast route.
    pub fn of_text(page_number: usize, text: &str) -> Self {
        let mut blocks = blocks_from_text(text, TextSource::Native);
        number_blocks(page_number, &mut blocks);
        Self {
            width: 0,
            height: 0,
            route: PageRoute::Fast,
            signals: RouteSignals::default(),
            blocks,
        }
    }
}

/// The page text a layout reads as: every block's text in reading order,
/// a blank line between blocks.
///
/// Deterministic, and the inverse the ids rely on: each block's text occurs
/// in the page text exactly, in order.
pub fn linearize(blocks: &[LayoutBlock]) -> String {
    let mut text = String::new();
    for block in blocks {
        if block.text.is_empty() {
            continue;
        }
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(&block.text);
    }
    text
}

/// Gives each block, row, cell, and field its id from its position on page
/// `page_number`, dropping blocks with no text.
pub fn number_blocks(page_number: usize, blocks: &mut Vec<LayoutBlock>) {
    blocks.retain(|block| !block.text.trim().is_empty());
    for (index, block) in blocks.iter_mut().enumerate() {
        block.id = format!("p{page_number}.b{}", index + 1);
        if let Some(table) = &mut block.table {
            for (row_index, row) in table.rows.iter_mut().enumerate() {
                row.id = format!("{}.r{}", block.id, row_index + 1);
                for (cell_index, cell) in row.cells.iter_mut().enumerate() {
                    cell.id = format!("{}.c{}", row.id, cell_index + 1);
                }
            }
        }
        for (field_index, field) in block.fields.iter_mut().enumerate() {
            field.id = format!("{}.f{}", block.id, field_index + 1);
        }
    }
}

/// Sets every block's `section`: the heading it falls under, across pages.
///
/// A heading closes every open heading of its own level or a lower one and
/// falls under the nearest one above it; any other block falls under the
/// innermost open heading. A heading of unknown level is treated as the
/// innermost kind, so it opens a section without closing the ones around it.
pub fn assign_sections<'a>(layouts: impl IntoIterator<Item = &'a mut PageLayout>) {
    const UNKNOWN_LEVEL: u8 = 7;
    let mut open: Vec<(u8, String)> = Vec::new();
    for layout in layouts {
        for block in &mut layout.blocks {
            if block.kind == BlockKind::Heading {
                let level = block.level.unwrap_or(UNKNOWN_LEVEL);
                while open
                    .last()
                    .is_some_and(|(open_level, _)| *open_level >= level)
                {
                    open.pop();
                }
                block.section = open.last().map(|(_, id)| id.clone());
                open.push((level, block.id.clone()));
            } else {
                block.section = open.last().map(|(_, id)| id.clone());
            }
        }
    }
}

/// Marks the running headers and footers of a document: a short block at
/// the top or the bottom of a page whose text, figures aside, is at the top
/// or the bottom of at least a quarter of the pages, two at the least
/// (`Annual Report, Fiscal Year 2026`, `Asset Purchase Agreement - 17`), or
/// that is only a page number. It takes three words to repeat: `PART 2` at
/// the top of a page is a heading, however many parts there are. Their text
/// is left as it is, and where they stand in the page; only their kind
/// changes, so that what repeats on every page can be told from what the
/// page says. A letterhead on the first page alone is not one.
pub fn mark_running_blocks(layouts: &mut [&mut PageLayout]) {
    /// How far into a page, from either end, a running block may be.
    const REACH: usize = 2;
    let candidate = |block: &LayoutBlock| {
        matches!(
            block.kind,
            BlockKind::Paragraph | BlockKind::Heading | BlockKind::Other
        ) && block.lines.len() <= 2
            && block.text.split_whitespace().count() <= 20
    };
    // The text with its case folded and every run of digits one `#`, so
    // `Page 9` and `Page 10` repeat.
    let shape = |text: &str| {
        text.split_whitespace()
            .map(|word| {
                let mut shaped = String::new();
                for character in word.chars() {
                    if character.is_ascii_digit() {
                        if !shaped.ends_with('#') {
                            shaped.push('#');
                        }
                    } else {
                        shaped.extend(character.to_lowercase());
                    }
                }
                shaped
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let ends = |layout: &PageLayout| {
        let count = layout.blocks.len();
        let top = (0..count.min(REACH)).collect::<Vec<_>>();
        let bottom =
            (count.saturating_sub(REACH).max(top.len().min(count))..count).collect::<Vec<_>>();
        (top, bottom)
    };
    let pages = layouts.len();
    let mut seen_top: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut seen_bottom: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for layout in layouts.iter() {
        let (top, bottom) = ends(layout);
        let mut page_top = std::collections::HashSet::new();
        let mut page_bottom = std::collections::HashSet::new();
        for index in top {
            let block = &layout.blocks[index];
            if candidate(block) {
                page_top.insert(shape(&block.text));
            }
        }
        for index in bottom {
            let block = &layout.blocks[index];
            if candidate(block) {
                page_bottom.insert(shape(&block.text));
            }
        }
        for text in page_top {
            *seen_top.entry(text).or_default() += 1;
        }
        for text in page_bottom {
            *seen_bottom.entry(text).or_default() += 1;
        }
    }
    for layout in layouts.iter_mut() {
        let (top, bottom) = ends(layout);
        for (indexes, seen, kind) in [
            (top, &seen_top, BlockKind::PageHeader),
            (bottom, &seen_bottom, BlockKind::PageFooter),
        ] {
            for index in indexes {
                let block = &mut layout.blocks[index];
                if !candidate(block) {
                    continue;
                }
                let shape = shape(&block.text);
                let repeated = shape.split(' ').count() >= 3
                    && seen
                        .get(&shape)
                        .is_some_and(|count| *count >= 2 && *count * 4 >= pages);
                if repeated || is_page_number(&block.text) {
                    block.kind = kind;
                    block.level = None;
                }
            }
        }
    }
}

/// `7`, `- 7 -`, `Page 7`, `Page 7 of 12`, `7/12`. A bare number has at
/// most three digits: `2026` alone is a year.
fn is_page_number(text: &str) -> bool {
    let words = text
        .split(|character: char| {
            character.is_whitespace() || matches!(character, '-' | '/' | '|' | '.' | '(' | ')')
        })
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    let number = |word: &&str| word.chars().all(|character| character.is_ascii_digit());
    let named = words
        .iter()
        .any(|word| word.eq_ignore_ascii_case("page") || word.eq_ignore_ascii_case("of"));
    !words.is_empty()
        && words.len() <= 4
        && words.iter().any(number)
        && words.iter().all(|word| {
            number(word) || word.eq_ignore_ascii_case("page") || word.eq_ignore_ascii_case("of")
        })
        && (named || words.iter().all(|word| word.len() <= 3))
}

/// The layout of a native PDF page on the fast route: the blocks of its
/// text, exactly as PDFium read it, with each line's box where the page's
/// segments say where its lines are.
pub fn fast_layout(text: &str, native: Option<&NativePage>, signals: RouteSignals) -> PageLayout {
    let mut lines = text
        .lines()
        .map(|line| LayoutLine {
            text: line.trim_end().to_owned(),
            bbox: None,
            confidence: None,
        })
        .collect::<Vec<_>>();
    if let Some(native) = native {
        let boxes = segment_lines(&native.segments);
        let written = lines.iter().filter(|line| !line.text.is_empty()).count();
        // Only a page whose segments fall into exactly as many lines as its
        // text has gets boxes: matching them up any other way would be a
        // guess.
        if written > 0 && boxes.len() == written {
            for (line, bbox) in lines
                .iter_mut()
                .filter(|line| !line.text.is_empty())
                .zip(boxes)
            {
                line.bbox = Some(bbox);
            }
        }
    }
    let mut blocks = blocks_from_lines(&lines, TextSource::Native);
    let (width, height) = match native {
        Some(native) => {
            to_display_blocks(&mut blocks, native);
            native.display_size()
        }
        None => (0, 0),
    };
    PageLayout {
        width,
        height,
        route: PageRoute::Fast,
        signals,
        blocks,
    }
}

/// The boxes of a page's lines, from its segments in content-stream order:
/// a segment starts a new line unless it sits on the line before it.
fn segment_lines(segments: &[[u32; 4]]) -> Vec<[u32; 4]> {
    let mut lines: Vec<[u32; 4]> = Vec::new();
    for segment in segments {
        if segment[3] <= segment[1] {
            continue;
        }
        let middle = (segment[1] + segment[3]) / 2;
        match lines.last_mut() {
            Some(line)
                if middle >= line[1]
                    && middle <= line[3]
                    && (line[1] + line[3]) / 2 >= segment[1]
                    && (line[1] + line[3]) / 2 <= segment[3] =>
            {
                *line = [
                    line[0].min(segment[0]),
                    line[1].min(segment[1]),
                    line[2].max(segment[2]),
                    line[3].max(segment[3]),
                ];
            }
            _ => lines.push(*segment),
        }
    }
    lines
}

/// The layout of a page rebuilt from its geometry: its own runs, and any
/// runs OCR read from images on it (already in the page's frame).
pub fn geometry_layout(
    native: &NativePage,
    extra_runs: Vec<TextRun>,
    signals: RouteSignals,
    route: PageRoute,
) -> PageLayout {
    let runs;
    let all = if extra_runs.is_empty() {
        &native.runs
    } else {
        runs = native
            .runs
            .iter()
            .cloned()
            .chain(extra_runs)
            .collect::<Vec<_>>();
        &runs
    };
    let mut blocks = analyze_runs(
        all,
        native.width,
        native.height,
        &native.rulings,
        TextSource::Native,
    );
    to_display_blocks(&mut blocks, native);
    let (width, height) = native.display_size();
    PageLayout {
        width,
        height,
        route,
        signals,
        blocks,
    }
}

/// Every box in a page's blocks, turned from the page's frame to the page
/// as displayed.
fn to_display_blocks(blocks: &mut [LayoutBlock], native: &NativePage) {
    if native.rotation % 360 == 0 {
        return;
    }
    let turn = |bbox: &mut Option<[u32; 4]>| {
        if let Some(value) = bbox {
            *value = native.to_display(*value);
        }
    };
    for block in blocks {
        turn(&mut block.bbox);
        for line in &mut block.lines {
            turn(&mut line.bbox);
        }
        if let Some(table) = &mut block.table {
            for cell in table.rows.iter_mut().flat_map(|row| &mut row.cells) {
                turn(&mut cell.bbox);
            }
        }
        for field in &mut block.fields {
            turn(&mut field.key_bbox);
            turn(&mut field.value_bbox);
        }
    }
}

/// Tenths of a point per pixel of an image with no physical size of its
/// own: 300 DPI, what scanners write.
pub const NOMINAL_UNITS_PER_PIXEL: f64 = UNITS_PER_POINT * 72.0 / 300.0;

/// A page read by OCR: its text is the engine's reading, exactly, and its
/// blocks are built from the engine's lines by the same analysis native
/// pages get.
///
/// `size` is the image read, in pixels, before the engine turned it;
/// `scale` is tenths of a point per pixel, [`NOMINAL_UNITS_PER_PIXEL`] when
/// none is given. An engine that reports no lines still gets blocks, from
/// its text, carrying its mean confidence.
pub fn ocr_page(
    page_number: usize,
    reading: OcrResult,
    size: (u32, u32),
    scale: Option<f64>,
    signals: RouteSignals,
) -> ExtractedPage {
    let mut layout = ocr_layout(&reading, size, scale, signals);
    number_blocks(page_number, &mut layout.blocks);
    ExtractedPage {
        page_number,
        text: reading.text,
        source: PageSource::Ocr,
        ocr_confidence: Some(reading.mean_confidence),
        vision_escalated: false,
        layout: Some(layout),
    }
}

/// The layout of an OCR reading, before its blocks are numbered. An OCR
/// engine that wants its page text to be the layout's reading order can use
/// [`linearize`] on these blocks.
pub fn ocr_layout(
    reading: &OcrResult,
    size: (u32, u32),
    scale: Option<f64>,
    signals: RouteSignals,
) -> PageLayout {
    let scale = scale.unwrap_or(NOMINAL_UNITS_PER_PIXEL);
    let (width, height) = if reading.rotation_degrees % 180 == 90 {
        (size.1, size.0)
    } else {
        size
    };
    let to_units = |pixels: u32| (f64::from(pixels) * scale).round().max(0.0) as u32;
    let (width, height) = (to_units(width), to_units(height));
    let blocks = if reading.lines.is_empty() {
        let confidence = Some(reading.mean_confidence.round().clamp(0.0, 100.0) as u8);
        let mut blocks = blocks_from_text(&reading.text, TextSource::Ocr);
        for block in &mut blocks {
            block.confidence = confidence;
        }
        blocks
    } else {
        let runs = reading
            .lines
            .iter()
            .map(|line| TextRun {
                text: line.text.clone(),
                bbox: line.bbox.map(to_units),
                bold: false,
                confidence: Some(line.confidence),
            })
            .collect::<Vec<_>>();
        analyze_runs(&runs, width, height, &[], TextSource::Ocr)
    };
    PageLayout {
        width,
        height,
        route: PageRoute::Ocr,
        signals,
        blocks,
    }
}

/// Images on a page that may hold text of their own: large, and with no
/// native text over them. In the page's frame, largest first.
pub fn text_regions(native: &NativePage) -> Vec<[u32; 4]> {
    const MAX_REGIONS: usize = 4;
    let page = f64::from(native.width) * f64::from(native.height);
    let mut regions = native
        .images
        .iter()
        .copied()
        .filter(|image| router::is_text_region(native, image, page))
        .collect::<Vec<_>>();
    regions.sort_by_key(|image| {
        std::cmp::Reverse(u64::from(image[2] - image[0]) * u64::from(image[3] - image[1]))
    });
    regions.dedup();
    regions.truncate(MAX_REGIONS);
    regions
}

/// The least mean confidence at which an OCR'd region's text joins the
/// page's, and the least a line of it needs. A region below it is a
/// photograph, a signature, or a logo, and its reading is noise.
const REGION_CONFIDENCE: f32 = 60.0;
const REGION_LINE_CONFIDENCE: u8 = 50;

/// A native page with the text OCR read from its image regions merged into
/// its blocks, in reading order. `readings` pairs each reading with its
/// region's box in the rendered page, in pixels; `scale` is tenths of a
/// point per rendered pixel. None when the regions held no text worth
/// keeping, and the page is then read as if they were not there.
pub fn regions_page(
    page_number: usize,
    inspection: &PdfPageInspection,
    signals: RouteSignals,
    readings: &[(OcrResult, [u32; 4])],
    scale: f64,
) -> Option<ExtractedPage> {
    let native = inspection.native.as_ref()?;
    if native.runs.is_empty() {
        return None;
    }
    let (display_width, display_height) = native.display_size();
    let mut extra = Vec::new();
    for (reading, crop) in readings {
        if reading.text.trim().is_empty() || reading.mean_confidence < REGION_CONFIDENCE {
            continue;
        }
        let whole = [crop[0], crop[1], crop[2], crop[3]];
        let lines = if reading.lines.is_empty() {
            vec![OcrLine {
                text: reading
                    .text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
                bbox: [0, 0, crop[2] - crop[0], crop[3] - crop[1]],
                confidence: reading.mean_confidence.round().clamp(0.0, 100.0) as u8,
            }]
        } else {
            reading.lines.clone()
        };
        for line in lines {
            if line.confidence < REGION_LINE_CONFIDENCE || line.text.trim().is_empty() {
                continue;
            }
            // A region read turned is placed as a whole: where its lines
            // sit inside it no longer matches the page.
            let pixels = if reading.rotation_degrees % 360 == 0 {
                [
                    crop[0] + line.bbox[0],
                    crop[1] + line.bbox[1],
                    crop[0] + line.bbox[2],
                    crop[1] + line.bbox[3],
                ]
            } else {
                whole
            };
            let display = pixels.map(|value| (f64::from(value) * scale).round() as u32);
            extra.push(TextRun {
                text: line.text,
                bbox: router::from_display(display, native.rotation, display_width, display_height),
                bold: false,
                confidence: Some(line.confidence),
            });
        }
    }
    if extra.is_empty() {
        return None;
    }
    let mut layout = geometry_layout(native, extra, signals, PageRoute::OcrRegions);
    number_blocks(page_number, &mut layout.blocks);
    Some(ExtractedPage {
        page_number,
        text: linearize(&layout.blocks),
        source: PageSource::Native,
        ocr_confidence: None,
        vision_escalated: false,
        layout: Some(layout),
    })
}

/// The union of boxes, if there are any.
pub(crate) fn union(boxes: impl IntoIterator<Item = [u32; 4]>) -> Option<[u32; 4]> {
    boxes.into_iter().reduce(|left, right| {
        [
            left[0].min(right[0]),
            left[1].min(right[1]),
            left[2].max(right[2]),
            left[3].max(right[3]),
        ]
    })
}

/// The mean of OCR confidences, if any line has one.
pub(crate) fn mean_confidence(values: impl IntoIterator<Item = Option<u8>>) -> Option<u8> {
    let (sum, count) = values
        .into_iter()
        .flatten()
        .fold((0_u32, 0_u32), |(sum, count), value| {
            (sum + u32::from(value), count + 1)
        });
    (count > 0).then(|| ((sum + count / 2) / count) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading(text: &str, level: Option<u8>) -> LayoutBlock {
        LayoutBlock {
            level,
            ..LayoutBlock::new(BlockKind::Heading, text, TextSource::Native)
        }
    }

    fn paragraph(text: &str) -> LayoutBlock {
        LayoutBlock::new(BlockKind::Paragraph, text, TextSource::Native)
    }

    #[test]
    fn blocks_are_numbered_in_reading_order_with_their_rows_cells_and_fields() {
        let mut table = LayoutBlock::new(BlockKind::Table, "| a | b |", TextSource::Native);
        table.table = Some(LayoutTable {
            rows: vec![LayoutRow {
                id: String::new(),
                cells: vec![
                    LayoutCell {
                        id: String::new(),
                        text: "a".into(),
                        bbox: None,
                        header: true,
                    },
                    LayoutCell {
                        id: String::new(),
                        text: "b".into(),
                        bbox: None,
                        header: true,
                    },
                ],
            }],
        });
        let mut pairs = LayoutBlock::new(BlockKind::KeyValue, "Date: May 1", TextSource::Native);
        pairs.fields.push(KeyValue {
            id: String::new(),
            key: "Date".into(),
            value: "May 1".into(),
            key_bbox: None,
            value_bbox: None,
        });
        let mut blocks = vec![paragraph("  "), table, pairs];

        number_blocks(3, &mut blocks);

        assert_eq!(blocks.len(), 2, "a block with no text is dropped");
        assert_eq!(blocks[0].id, "p3.b1");
        let row = &blocks[0].table.as_ref().unwrap().rows[0];
        assert_eq!(row.id, "p3.b1.r1");
        assert_eq!(row.cells[1].id, "p3.b1.r1.c2");
        assert_eq!(blocks[1].fields[0].id, "p3.b2.f1");
    }

    #[test]
    fn sections_follow_headings_across_pages() {
        let mut first = PageLayout {
            blocks: vec![
                heading("LEASE", Some(1)),
                heading("Article 1", Some(2)),
                paragraph("The premises."),
            ],
            ..PageLayout::default()
        };
        number_blocks(1, &mut first.blocks);
        let mut second = PageLayout {
            blocks: vec![
                paragraph("Continued."),
                heading("Article 2", Some(2)),
                paragraph("Rent."),
            ],
            ..PageLayout::default()
        };
        number_blocks(2, &mut second.blocks);

        assign_sections([&mut first, &mut second]);

        assert_eq!(first.blocks[0].section, None);
        assert_eq!(first.blocks[1].section.as_deref(), Some("p1.b1"));
        assert_eq!(first.blocks[2].section.as_deref(), Some("p1.b2"));
        assert_eq!(second.blocks[0].section.as_deref(), Some("p1.b2"));
        assert_eq!(second.blocks[1].section.as_deref(), Some("p1.b1"));
        assert_eq!(second.blocks[2].section.as_deref(), Some("p2.b2"));
    }

    #[test]
    fn running_headers_footers_and_page_numbers_are_marked_across_pages() {
        let page = |number: usize, body: &str| {
            let mut layout = PageLayout::of_text(
                number,
                &format!(
                    "Kingsfold Specialty Foods - Annual Report 2026\n\nPART {number}\n\n{body}\n\nPage {number} of 3"
                ),
            );
            layout.blocks[1].kind = BlockKind::Heading;
            layout
        };
        let mut first = PageLayout::of_text(
            1,
            "KINGSFOLD SPECIALTY FOODS INC.\n\nLetter to members.\n\n1",
        );
        let mut second = page(2, "Revenue rose.");
        let mut third = page(3, "Margins held.");

        mark_running_blocks(&mut [&mut first, &mut second, &mut third]);

        assert_eq!(
            first.blocks[0].kind,
            BlockKind::Heading,
            "a letterhead once"
        );
        assert_eq!(first.blocks[2].kind, BlockKind::PageFooter, "a page number");
        for layout in [&second, &third] {
            assert_eq!(layout.blocks[0].kind, BlockKind::PageHeader);
            assert_eq!(layout.blocks[1].kind, BlockKind::Heading);
            assert_eq!(layout.blocks[2].kind, BlockKind::Paragraph);
            assert_eq!(layout.blocks[3].kind, BlockKind::PageFooter);
        }
        assert!(!is_page_number("2026"), "a year alone");
        assert!(is_page_number("- 17 -") && is_page_number("Page 4 of 12"));
        assert!(!is_page_number("Section 4"));
    }

    #[test]
    fn linearization_separates_blocks_with_a_blank_line() {
        let blocks = vec![heading("NOTICE", Some(1)), paragraph("Line one\nLine two")];

        assert_eq!(linearize(&blocks), "NOTICE\n\nLine one\nLine two");
    }

    #[test]
    fn layouts_serialise_in_snake_case_without_empty_parts() {
        let mut layout = PageLayout::of_text(1, "INVOICE\n\nInvoice date: May 1, 2026");
        layout.route = PageRoute::OcrRegions;
        let value = serde_json::to_value(&layout).unwrap();

        assert_eq!(value["route"], "ocr_regions");
        assert_eq!(value["blocks"][0]["kind"], "heading");
        assert_eq!(value["blocks"][1]["kind"], "key_value");
        assert_eq!(value["blocks"][1]["fields"][0]["key"], "Invoice date");
        assert!(value["blocks"][0].get("table").is_none());
        assert!(value["blocks"][0].get("fields").is_none());
        let back: PageLayout = serde_json::from_value(value).unwrap();
        assert_eq!(back, layout);
    }

    #[test]
    fn mean_confidence_rounds_and_ignores_missing_values() {
        assert_eq!(mean_confidence([Some(90), None, Some(81)]), Some(86));
        assert_eq!(mean_confidence([None, None]), None);
    }
}
