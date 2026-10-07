//! Every page of a document as blocks with stable ids.
//!
//! The parser worker sends each page's layout beside its text: headings,
//! paragraphs, list items, tables with their rows and cells, and labelled
//! values, in reading order, each with an id (`p{page}.b{n}`), where it sits
//! on the page when the reader knows, where its text came from, and how
//! sure OCR was of it. These are the worker's own types, mirrored rather
//! than shared - the engine does not depend on the worker crate - and they
//! read the worker's JSON exactly.
//!
//! [`structured`] is the one place that turns a [`DocumentSource`] into
//! blocks. A page that arrived with a layout keeps it. A page that did not -
//! one stored before layouts existed, one past the worker's layout budget,
//! one built from plain text by a test or a tool - is segmented from its
//! text exactly as the worker segments a page it has no geometry for, with
//! the same ids, kinds and labelled values. Either way every block falls
//! under the nearest heading above it, across pages.
//!
//! Coordinates are tenths of a PDF point from the top left of the page as
//! displayed; confidence is 0-100. Everything is an integer, so the types
//! keep `Eq` and serialise the same way every time.

use serde::{Deserialize, Serialize};

use crate::domain::{DocumentSource, PageOrigin};

/// One page as blocks in reading order, as the worker built it.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PageLayout {
    /// The page's displayed size in tenths of a point; zero for a reader
    /// that knows no geometry.
    pub width: u32,
    pub height: u32,
    pub route: PageRoute,
    /// What the worker's router measured about the page.
    #[serde(default)]
    pub signals: RouteSignals,
    pub blocks: Vec<LayoutBlock>,
}

/// How the worker read a page.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageRoute {
    /// The reader's own text, as it has always been read.
    #[default]
    Fast,
    /// Rebuilt from the geometry of the page's characters.
    Layout,
    /// Read by OCR.
    Ocr,
    /// Native text, with OCR of images on the page that may hold text.
    OcrRegions,
}

/// What the worker's router measured about a page. Ratios are per mille.
/// Diagnostic only: nothing in the engine decides anything by them, and a
/// signal this build does not know is ignored rather than refused.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct RouteSignals {
    pub chars: u32,
    pub segments: u32,
    pub image_coverage: u16,
    pub replacement: u16,
    pub invisible: u16,
    pub garbage: u16,
    pub columns: u8,
    pub interleave: u16,
    pub aligned_rows: u16,
    pub key_values: u16,
    pub key_value_grid: u16,
    pub overlap: u16,
    pub font_sizes: u8,
    pub rulings: u16,
    pub image_region: u16,
    pub rotation: u16,
}

/// What kind of thing a block is.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TextSource {
    #[default]
    Native,
    Ocr,
}

/// One block of a page.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LayoutBlock {
    /// `p{page}.b{n}`, `n` counting from 1 in reading order on the page.
    pub id: String,
    pub kind: BlockKind,
    /// The block's text as the page's text carries it. A table is its rows
    /// as `| a | b |` lines; a key-value block its pairs as `Key: value`
    /// lines.
    pub text: String,
    /// `[x0, y0, x1, y1]`, or none for a reader that knows no geometry.
    pub bbox: Option<[u32; 4]>,
    /// Heading level, 1 the largest, where it can be told.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// The id of the heading the block falls under, on this page or an
    /// earlier one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    pub source: TextSource,
    /// OCR: the mean confidence of the block's lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    #[serde(default)]
    pub lines: Vec<LayoutLine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<LayoutTable>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<KeyValue>,
}

/// One line of a block.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LayoutLine {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[u32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
}

/// A table's rows, in reading order.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct LayoutTable {
    pub rows: Vec<LayoutRow>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LayoutRow {
    /// `p{page}.b{n}.r{row}`.
    pub id: String,
    pub cells: Vec<LayoutCell>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LayoutCell {
    /// `p{page}.b{n}.r{row}.c{column}`.
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[u32; 4]>,
    #[serde(default)]
    pub header: bool,
}

/// A labelled value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KeyValue {
    /// `p{page}.b{n}.f{field}`.
    pub id: String,
    pub key: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_bbox: Option<[u32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_bbox: Option<[u32; 4]>,
}

/// A whole document as blocks.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct StructuredDocument {
    pub pages: Vec<StructuredPage>,
}

/// One page of a [`StructuredDocument`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StructuredPage {
    pub page_number: usize,
    pub origin: PageOrigin,
    /// How the worker read the page; none for a page that came without a
    /// layout and was segmented here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<PageRoute>,
    /// The page's displayed size in tenths of a point, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<[u32; 2]>,
    pub blocks: Vec<LayoutBlock>,
}

impl StructuredDocument {
    /// Every block of the document, page by page, in reading order.
    pub fn blocks(&self) -> impl Iterator<Item = &LayoutBlock> {
        self.pages.iter().flat_map(|page| &page.blocks)
    }

    /// The block with this id, if there is one.
    pub fn block(&self, id: &str) -> Option<&LayoutBlock> {
        self.blocks().find(|block| block.id == id)
    }
}

/// Blocks for every page of a document: the worker's layout where a page
/// has one, otherwise blocks segmented from the page's text with the same
/// ids, and every block's section linked to the heading above it.
pub fn structured(source: &DocumentSource) -> StructuredDocument {
    let mut pages = source
        .pages
        .iter()
        .map(|page| match &page.layout {
            Some(layout) => StructuredPage {
                page_number: page.page_number,
                origin: page.origin,
                route: Some(layout.route),
                size: (layout.width > 0 && layout.height > 0)
                    .then_some([layout.width, layout.height]),
                blocks: layout.blocks.clone(),
            },
            None => StructuredPage {
                page_number: page.page_number,
                origin: page.origin,
                route: None,
                size: None,
                blocks: segment(page.page_number, &page.text, page.origin),
            },
        })
        .collect::<Vec<_>>();
    link_sections(&mut pages);
    StructuredDocument { pages }
}

/// Paragraphs longer than this many lines are split at the next line that
/// ends a sentence, as the worker splits them.
const MAX_PARAGRAPH_LINES: usize = 12;

/// A page's text as blocks, exactly as the parser worker builds a page that
/// has text and no geometry (`PageLayout::of_text`, `blocks_from_text`): a
/// blank line ends a block; a Markdown heading, or a short line nearly all
/// capitals that does not end in a colon, is a heading; consecutive `|`
/// lines are a table, the row above a `| --- |` separator its header;
/// consecutive `Key: value` lines are labelled values; a bulleted or
/// enumerated line starts a list item; anything else is a paragraph, split
/// after twelve lines where a line ends a sentence. Blocks are numbered the
/// worker's way, so a page stored before layouts existed reads exactly
/// like one the worker sends today. The worker's contract test holds the
/// two to each other.
fn segment(page_number: usize, text: &str, origin: PageOrigin) -> Vec<LayoutBlock> {
    let source = if origin == PageOrigin::Ocr {
        TextSource::Ocr
    } else {
        TextSource::Native
    };
    let lines = text.lines().map(str::trim_end).collect::<Vec<_>>();
    let kinds = lines.iter().map(|line| classify(line)).collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let kind = kinds[index];
        let start = index;
        match kind {
            LineKind::Blank => {
                index += 1;
            }
            LineKind::MarkdownHeading(level) => {
                index += 1;
                let mut heading = block(BlockKind::Heading, text, &lines[start..index], source);
                heading.level = Some(level);
                blocks.push(heading);
            }
            LineKind::TableRow | LineKind::TableSeparator => {
                while index < lines.len()
                    && matches!(kinds[index], LineKind::TableRow | LineKind::TableSeparator)
                {
                    index += 1;
                }
                let mut table = block(BlockKind::Table, text, &lines[start..index], source);
                table.table = Some(table_rows(&lines[start..index], &kinds[start..index]));
                blocks.push(table);
            }
            LineKind::KeyValue => {
                index += 1;
                while index < lines.len() && kinds[index] == LineKind::KeyValue {
                    index += 1;
                }
                let mut labelled = block(BlockKind::KeyValue, text, &lines[start..index], source);
                labelled.fields = lines[start..index]
                    .iter()
                    .filter_map(|line| {
                        let (key, value) = split_key_value(line.trim())?;
                        Some(KeyValue {
                            id: String::new(),
                            key: key.trim_end_matches(':').trim().to_owned(),
                            value: value.to_owned(),
                            key_bbox: None,
                            value_bbox: None,
                        })
                    })
                    .collect();
                blocks.push(labelled);
            }
            LineKind::Heading => {
                index += 1;
                blocks.push(block(
                    BlockKind::Heading,
                    text,
                    &lines[start..index],
                    source,
                ));
            }
            LineKind::ListItem | LineKind::Text => {
                index += 1;
                while index < lines.len()
                    && kinds[index] == LineKind::Text
                    && !(index - start >= MAX_PARAGRAPH_LINES && ends_sentence(lines[index - 1]))
                {
                    index += 1;
                }
                let block_kind = if kind == LineKind::ListItem {
                    BlockKind::ListItem
                } else {
                    BlockKind::Paragraph
                };
                blocks.push(block(block_kind, text, &lines[start..index], source));
            }
        }
    }
    number_blocks(page_number, &mut blocks);
    blocks
}

/// What one line of text is, as the worker classifies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineKind {
    Blank,
    MarkdownHeading(u8),
    TableRow,
    TableSeparator,
    ListItem,
    KeyValue,
    Heading,
    Text,
}

fn classify(line: &str) -> LineKind {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LineKind::Blank;
    }
    if let Some(level) = markdown_heading_level(trimmed) {
        return LineKind::MarkdownHeading(level);
    }
    if trimmed.starts_with('|') {
        return if is_table_separator(trimmed) {
            LineKind::TableSeparator
        } else {
            LineKind::TableRow
        };
    }
    if is_list_item(trimmed) {
        return LineKind::ListItem;
    }
    if split_key_value(trimmed).is_some() {
        return LineKind::KeyValue;
    }
    if is_heading_line(trimmed) {
        return LineKind::Heading;
    }
    LineKind::Text
}

fn ends_sentence(line: &str) -> bool {
    line.trim_end().ends_with(['.', ':', ';', '!', '?'])
}

/// A block of `lines`, each a stretch of the page's `text`. Its text is the
/// stretch from its first line to its last, byte for byte, as the worker
/// keeps it, so the block can be found in the page's text whatever its line
/// endings.
fn block(kind: BlockKind, text: &str, lines: &[&str], source: TextSource) -> LayoutBlock {
    let offset = |line: &str| line.as_ptr() as usize - text.as_ptr() as usize;
    let stretch = match (lines.first(), lines.last()) {
        (Some(first), Some(last)) => &text[offset(first)..offset(last) + last.len()],
        _ => "",
    };
    LayoutBlock {
        id: String::new(),
        kind,
        text: stretch.to_owned(),
        bbox: None,
        level: None,
        section: None,
        source,
        confidence: None,
        lines: lines
            .iter()
            .map(|line| LayoutLine {
                text: (*line).to_owned(),
                bbox: None,
                confidence: None,
            })
            .collect(),
        table: None,
        fields: Vec::new(),
    }
}

/// Gives each block, row, cell, and field its id from its position on the
/// page, dropping blocks with no text, as the worker numbers them.
fn number_blocks(page_number: usize, blocks: &mut Vec<LayoutBlock>) {
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

/// A Markdown table's rows: the cells between its pipes, the row above a
/// `| --- |` separator marked as the header.
fn table_rows(lines: &[&str], kinds: &[LineKind]) -> LayoutTable {
    let mut rows: Vec<LayoutRow> = Vec::new();
    for (line, kind) in lines.iter().zip(kinds) {
        if *kind == LineKind::TableSeparator {
            if let Some(row) = rows.last_mut() {
                for cell in &mut row.cells {
                    cell.header = true;
                }
            }
            continue;
        }
        rows.push(LayoutRow {
            id: String::new(),
            cells: table_cells(line)
                .into_iter()
                .map(|text| LayoutCell {
                    id: String::new(),
                    text,
                    bbox: None,
                    header: false,
                })
                .collect(),
        });
    }
    LayoutTable { rows }
}

fn markdown_heading_level(line: &str) -> Option<u8> {
    let hashes = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    let rest = &line[hashes..];
    ((1..=6).contains(&hashes) && rest.starts_with(' ') && !rest.trim().is_empty())
        .then_some(hashes as u8)
}

/// `| --- | :---: |`, the line Markdown puts under a table's header row.
fn is_table_separator(line: &str) -> bool {
    let cells = table_cells(line);
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let core = cell.trim().trim_matches(':');
            core.len() >= 3 && core.chars().all(|character| character == '-')
        })
}

/// A table row's cells: the text between unescaped pipes, trimmed. Empty
/// cells are kept, so a cell stays in its column.
fn table_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in inner.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            current.push(character);
            escaped = true;
        } else if character == '|' {
            cells.push(current.trim().to_owned());
            current.clear();
        } else {
            current.push(character);
        }
    }
    cells.push(current.trim().to_owned());
    cells
}

/// Whether a line starts with a bullet or a parenthesised enumerator.
fn is_list_item(line: &str) -> bool {
    let mut characters = line.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if matches!(first, '•' | '◦' | '▪' | '‣' | '●' | '○' | '■' | '□' | '–')
        || (matches!(first, '-' | '*' | '+') && line[1..].starts_with(' '))
    {
        return line.chars().count() > 2;
    }
    // (a) (iv) (12) a) 3)
    let marker_end = line.find(' ').unwrap_or(0);
    if marker_end == 0 || marker_end > 6 {
        return false;
    }
    let marker = &line[..marker_end];
    let core = marker.trim_start_matches('(');
    let Some(core) = core.strip_suffix(')') else {
        return false;
    };
    !core.is_empty()
        && ((core.len() <= 2 && core.chars().all(|character| character.is_ascii_digit()))
            || (core.len() == 1 && core.chars().all(|character| character.is_ascii_lowercase()))
            || core
                .chars()
                .all(|character| matches!(character, 'i' | 'v' | 'x')))
}

/// A short line that is nearly all capitals and does not end in a colon:
/// the worker's rule, which is distillation's without the colon.
fn is_heading_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 90 || trimmed.ends_with(':') {
        return false;
    }
    if trimmed.ends_with('.') && trimmed.split_whitespace().count() > 6 {
        return false;
    }
    let letters = trimmed
        .chars()
        .filter(|character| character.is_alphabetic());
    let letter_count = letters.clone().count();
    if letter_count < 3 {
        return false;
    }
    let uppercase = letters.filter(|character| character.is_uppercase()).count();
    uppercase * 10 >= letter_count * 8
}

/// `Invoice date: May 1, 2026` as its label and value, by the worker's
/// rule: a label of at most six words and forty characters that starts
/// with a capital and holds no sentence, then a colon and a space.
pub(crate) fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(": ")?;
    let key = line[..colon].trim();
    let value = line[colon + 2..].trim();
    if value.is_empty() || !is_label(key) {
        return None;
    }
    Some((key, value))
}

/// Whether text reads as a field label: short, starting with a capital, no
/// sentence punctuation inside it.
fn is_label(key: &str) -> bool {
    let key = key.trim().trim_end_matches(':').trim();
    if key.is_empty() || key.chars().count() > 40 || key.split_whitespace().count() > 6 {
        return false;
    }
    let Some(first) = key.chars().next() else {
        return false;
    };
    if !first.is_uppercase() {
        return false;
    }
    if key.contains(['"', '“', '”', ';', ',', ':']) {
        return false;
    }
    let words = key.split_whitespace().collect::<Vec<_>>();
    let lowercase_words = words
        .iter()
        .filter(|word| word.chars().next().is_some_and(char::is_lowercase))
        .filter(|word| {
            !matches!(
                **word,
                "of" | "and"
                    | "or"
                    | "to"
                    | "the"
                    | "for"
                    | "no."
                    | "no"
                    | "by"
                    | "in"
                    | "on"
                    | "at"
                    | "per"
            )
        })
        .count();
    lowercase_words <= 2 && !(words.len() > 3 && lowercase_words > 1)
}

/// Links each block to the heading it falls under, across pages: a heading
/// closes every open heading of its level or below and falls under the
/// nearest above it; any other block falls under the innermost open one. A
/// heading of unknown level is the innermost kind.
fn link_sections(pages: &mut [StructuredPage]) {
    const UNKNOWN_LEVEL: u8 = 7;
    let mut open: Vec<(u8, String)> = Vec::new();
    for block in pages.iter_mut().flat_map(|page| &mut page.blocks) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SourcePage;

    /// What the worker sends for one page of a two-column lease, as it
    /// sends it.
    const WORKER_LAYOUT: &str = r#"{
        "width": 6120, "height": 7920, "route": "layout",
        "signals": {"chars": 3381, "segments": 90, "columns": 2, "interleave": 18, "future_signal": 7},
        "blocks": [
            {"id": "p1.b1", "kind": "heading", "text": "RETAIL LEASE", "bbox": [2482, 706, 3638, 896],
             "level": 1, "source": "native", "lines": [{"text": "RETAIL LEASE", "bbox": [2482, 706, 3638, 896]}]},
            {"id": "p1.b2", "kind": "table", "text": "| Term | Provision |\n| Premises | Suite 108 |",
             "bbox": [570, 1839, 5115, 2106], "section": "p1.b1", "source": "native",
             "lines": [{"text": "| Term | Provision |"}, {"text": "| Premises | Suite 108 |"}],
             "table": {"rows": [
                {"id": "p1.b2.r1", "cells": [{"id": "p1.b2.r1.c1", "text": "Term", "header": true},
                                             {"id": "p1.b2.r1.c2", "text": "Provision", "header": true}]},
                {"id": "p1.b2.r2", "cells": [{"id": "p1.b2.r2.c1", "text": "Premises"},
                                             {"id": "p1.b2.r2.c2", "text": "Suite 108"}]}]}},
            {"id": "p1.b3", "kind": "key_value", "text": "Date: May 18, 2026", "bbox": null,
             "section": "p1.b1", "source": "ocr", "confidence": 91,
             "lines": [{"text": "Date: May 18, 2026", "confidence": 91}],
             "fields": [{"id": "p1.b3.f1", "key": "Date", "value": "May 18, 2026"}]}
        ]
    }"#;

    #[test]
    fn the_worker_layout_reads_as_it_is_sent() {
        let layout: PageLayout = serde_json::from_str(WORKER_LAYOUT).unwrap();

        assert_eq!(layout.route, PageRoute::Layout);
        assert_eq!(layout.signals.columns, 2);
        assert_eq!(layout.blocks.len(), 3);
        assert_eq!(layout.blocks[1].kind, BlockKind::Table);
        let table = layout.blocks[1].table.as_ref().unwrap();
        assert!(table.rows[0].cells[0].header);
        assert_eq!(table.rows[1].cells[1].text, "Suite 108");
        assert_eq!(layout.blocks[2].fields[0].value, "May 18, 2026");
        assert_eq!(layout.blocks[2].source, TextSource::Ocr);
        let again: PageLayout =
            serde_json::from_value(serde_json::to_value(&layout).unwrap()).unwrap();
        assert_eq!(again, layout);
    }

    #[test]
    fn a_page_with_a_layout_keeps_it_and_one_without_is_segmented() {
        let mut first = SourcePage::new(1, "ignored when there is a layout", PageOrigin::Native);
        first.layout = Some(serde_json::from_str(WORKER_LAYOUT).unwrap());
        let second = SourcePage::new(
            2,
            "ARTICLE 2. RENT\nTenant pays rent monthly.\nIn advance.\n\n| Year | Rent |\n| --- | --- |\n| 1 | $34.00 |\n\nSigned.",
            PageOrigin::Native,
        );
        let document = structured(&DocumentSource::from_pages(vec![first, second]));

        assert_eq!(document.pages[0].route, Some(PageRoute::Layout));
        assert_eq!(document.pages[0].size, Some([6120, 7920]));
        assert_eq!(document.pages[0].blocks[0].text, "RETAIL LEASE");
        let page = &document.pages[1];
        assert_eq!(page.route, None);
        let kinds = page
            .blocks
            .iter()
            .map(|block| block.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                BlockKind::Heading,
                BlockKind::Paragraph,
                BlockKind::Table,
                BlockKind::Paragraph
            ]
        );
        let ids = page
            .blocks
            .iter()
            .map(|block| block.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["p2.b1", "p2.b2", "p2.b3", "p2.b4"]);
        assert_eq!(
            page.blocks[1].text,
            "Tenant pays rent monthly.\nIn advance."
        );
        let table = page.blocks[2].table.as_ref().unwrap();
        assert_eq!(table.rows.len(), 2);
        assert!(table.rows[0].cells.iter().all(|cell| cell.header));
        assert_eq!(table.rows[1].cells[1].id, "p2.b3.r2.c2");
        // A heading of unknown level after a level-1 heading falls under
        // it; the blocks after it fall under the new heading.
        assert_eq!(page.blocks[0].section.as_deref(), Some("p1.b1"));
        assert_eq!(page.blocks[1].section.as_deref(), Some("p2.b1"));
        assert_eq!(document.block("p2.b3").unwrap().kind, BlockKind::Table);
        assert_eq!(document.blocks().count(), 7);
    }

    /// A segmented block is the stretch of the page text it came from,
    /// whatever ends the page's lines.
    #[test]
    fn a_segmented_block_is_an_exact_stretch_of_the_page_text() {
        let text = "ARTICLE 2. RENT\r\nTenant pays rent monthly.  \r\nIn advance.\r\n\r\n| Year | Rent |\r\n| 1 | $34.00 |";
        let document = structured(&DocumentSource::from_pages(vec![SourcePage::new(
            1,
            text,
            PageOrigin::Native,
        )]));

        let mut from = 0;
        for block in document.blocks() {
            let at = text[from..].find(&block.text).unwrap();
            from += at + block.text.len();
        }
        assert_eq!(
            document.block("p1.b2").unwrap().text,
            "Tenant pays rent monthly.  \r\nIn advance."
        );
        assert_eq!(
            document.block("p1.b2").unwrap().lines[0].text,
            "Tenant pays rent monthly."
        );
    }

    #[test]
    fn ocr_text_is_segmented_as_ocr_and_markdown_headings_keep_their_level() {
        let document = structured(&DocumentSource::from_pages(vec![
            SourcePage::new(1, "PACKING SLIP PS-311\nDATE JULY 15 2025", PageOrigin::Ocr),
            SourcePage::new(2, "## Terms\n\nNet 30.", PageOrigin::Office),
        ]));

        assert!(
            document
                .blocks()
                .take(2)
                .all(|block| block.source == TextSource::Ocr)
        );
        let heading = document.block("p2.b1").unwrap();
        assert_eq!(heading.kind, BlockKind::Heading);
        assert_eq!(heading.level, Some(2));
    }
}
