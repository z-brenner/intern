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
//! text with the same id scheme, the distiller's own heading and table
//! detection deciding what each block is. Either way every block falls
//! under the nearest heading above it, across pages.
//!
//! Coordinates are tenths of a PDF point from the top left of the page as
//! displayed; confidence is 0-100. Everything is an integer, so the types
//! keep `Eq` and serialise the same way every time.

use serde::{Deserialize, Serialize};

use crate::distill::is_heading_line;
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

/// A page's text as blocks, the way distillation segments it: a blank line
/// ends a block, a line that reads as a heading is one, and consecutive `|`
/// lines are a table.
fn segment(page_number: usize, text: &str, origin: PageOrigin) -> Vec<LayoutBlock> {
    let source = if origin == PageOrigin::Ocr {
        TextSource::Ocr
    } else {
        TextSource::Native
    };
    let mut blocks = Vec::new();
    let mut pending: Vec<&str> = Vec::new();
    let mut pending_table = false;
    let flush = |pending: &mut Vec<&str>, table: bool, blocks: &mut Vec<LayoutBlock>| {
        if pending.is_empty() {
            return;
        }
        let kind = if table {
            BlockKind::Table
        } else {
            BlockKind::Paragraph
        };
        blocks.push(block(kind, text, std::mem::take(pending), source));
    };
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            flush(&mut pending, pending_table, &mut blocks);
            continue;
        }
        let table_line = trimmed.trim_start().starts_with('|');
        if table_line != pending_table && !pending.is_empty() {
            flush(&mut pending, pending_table, &mut blocks);
        }
        pending_table = table_line;
        if !table_line && is_heading_line(trimmed) {
            flush(&mut pending, pending_table, &mut blocks);
            let mut heading = block(BlockKind::Heading, text, vec![trimmed.trim()], source);
            heading.level = markdown_level(trimmed.trim());
            blocks.push(heading);
            continue;
        }
        pending.push(trimmed);
    }
    flush(&mut pending, pending_table, &mut blocks);
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
    }
    blocks
}

/// A block of `lines`, each a stretch of the page's `text`. Its text is the
/// stretch from its first line to its last, byte for byte, so the block
/// can be found in the page's text whatever its line endings.
fn block(kind: BlockKind, text: &str, lines: Vec<&str>, source: TextSource) -> LayoutBlock {
    let table = (kind == BlockKind::Table).then(|| table_rows(&lines));
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
        table,
        fields: Vec::new(),
    }
}

/// A Markdown table's rows: the cells between its pipes, the row above a
/// `| --- |` separator marked as the header.
fn table_rows(lines: &[&str]) -> LayoutTable {
    let mut rows: Vec<LayoutRow> = Vec::new();
    for line in lines {
        let cells = cells_of(line);
        let separator = !cells.is_empty()
            && cells.iter().all(|cell| {
                let core = cell.trim_matches(':');
                core.len() >= 3 && core.chars().all(|character| character == '-')
            });
        if separator {
            if let Some(row) = rows.last_mut() {
                for cell in &mut row.cells {
                    cell.header = true;
                }
            }
            continue;
        }
        rows.push(LayoutRow {
            id: String::new(),
            cells: cells
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

/// A Markdown row's cells, read as the worker reads them (its
/// `table_cells`): an escaped pipe is part of its cell, not a column, and
/// stays escaped, so a page segmented here has the cells it would have had
/// with a layout.
fn cells_of(line: &str) -> Vec<String> {
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

fn markdown_level(line: &str) -> Option<u8> {
    let hashes = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    ((1..=6).contains(&hashes) && line[hashes..].starts_with(' ')).then_some(hashes as u8)
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

    /// An escaped pipe is part of its cell, as the worker reads it.
    #[test]
    fn an_escaped_pipe_in_a_segmented_table_is_part_of_its_cell() {
        let document = structured(&DocumentSource::from_pages(vec![SourcePage::new(
            1,
            "| Clause | Terms |\n| --- | --- |\n| 4 | Net 30 \\| Net 45 |",
            PageOrigin::Office,
        )]));

        let table = document.block("p1.b1").unwrap().table.as_ref().unwrap();
        assert_eq!(table.rows.len(), 2);
        let cells = table.rows[1]
            .cells
            .iter()
            .map(|cell| cell.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(cells, ["4", "Net 30 \\| Net 45"]);
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
