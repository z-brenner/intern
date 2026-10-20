//! Blocks from an OCR engine's own layout: an engine that labels its blocks
//! and puts them in reading order (Indra OCR) has already done what the
//! geometric analysis would, with a network trained for it, so its order is
//! kept - blocks as it orders them, lines as it orders them inside each -
//! and its labels become block kinds:
//!
//! | Engine label | Block kind |
//! |---|---|
//! | Text | Paragraph; KeyValue when every line is a labelled value |
//! | Section header | Heading, its level from its size |
//! | Page header, page footer | PageHeader, PageFooter |
//! | List group | ListItem, one block an item, split at list markers |
//! | Table | Table, rows and cells from the lines' geometry |
//! | Caption | Caption |
//! | Form, footnote | KeyValue for the labelled values in it, Other for the rest |
//! | Table of contents | Other |
//!
//! What the engine does not say is found as every OCR page finds it: a
//! heading's level from its size, labelled values from `Label: value`
//! lines, and the running header and footer from where small blocks sit.

use super::geometry::mark_running_lines_by;
use super::text::{escape_cell, is_label, median, split_key_value};
use super::{
    BlockKind, KeyValue, LayoutBlock, LayoutCell, LayoutLine, LayoutRow, LayoutTable, TextRun,
    TextSource, analyze_runs_within, mean_confidence, router, union,
};
use crate::extract::{OcrBlockLabel, OcrResult};

/// What became of a reading's own layout.
pub(super) enum Labelled {
    /// The reading carries no layout of its own, or one that does not
    /// account for its lines - a line in two blocks, a line in none, a
    /// block naming a line that is not there - and its lines are analysed
    /// instead.
    Absent,
    /// `stop` said to stop.
    Stopped,
    /// The page's blocks, in the engine's reading order.
    Blocks(Vec<LayoutBlock>),
}

/// One of the reading's lines, in tenths of a point.
#[derive(Clone, Debug)]
struct Line {
    text: String,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    confidence: u8,
}

impl Line {
    fn height(&self) -> f64 {
        (self.y1 - self.y0).max(1.0)
    }

    fn center_y(&self) -> f64 {
        (self.y0 + self.y1) / 2.0
    }

    fn bbox(&self) -> [u32; 4] {
        [self.x0, self.y0, self.x1, self.y1].map(|value| value.max(0.0).round() as u32)
    }

    fn layout_line(&self) -> LayoutLine {
        LayoutLine {
            text: self.text.clone(),
            bbox: Some(self.bbox()),
            confidence: Some(self.confidence),
        }
    }
}

/// The blocks of `reading`'s own layout, its pixels scaled by `to_units`
/// onto a page `width` by `height` tenths of a point.
pub(super) fn labelled_blocks(
    reading: &OcrResult,
    to_units: &dyn Fn(u32) -> u32,
    width: u32,
    height: u32,
    stop: &dyn Fn() -> bool,
) -> Labelled {
    if reading.blocks.is_empty()
        || reading.lines.len() > router::bounds::MAX_RUNS
        || !accounts_for_every_line(reading)
    {
        return Labelled::Absent;
    }
    let lines = reading
        .lines
        .iter()
        .map(|line| {
            let [x0, y0, x1, y1] = line.bbox.map(|value| f64::from(to_units(value)));
            Line {
                text: line.text.trim().to_owned(),
                x0,
                y0,
                x1,
                y1: y1.max(y0 + 1.0),
                confidence: line.confidence,
            }
        })
        .collect::<Vec<_>>();
    let body = body_height(&lines);
    let mut blocks = Vec::new();
    for block in &reading.blocks {
        if stop() {
            return Labelled::Stopped;
        }
        let members = block
            .lines
            .iter()
            .map(|index| &lines[*index])
            .filter(|line| !line.text.is_empty())
            .collect::<Vec<_>>();
        if members.is_empty() {
            continue;
        }
        match block.label {
            OcrBlockLabel::Text => blocks.extend(
                fields_only(&members)
                    .unwrap_or_else(|| vec![block_of(BlockKind::Paragraph, &members)]),
            ),
            OcrBlockLabel::SectionHeader => {
                let mut heading = block_of(BlockKind::Heading, &members);
                heading.level = heading_level(&members, body);
                blocks.push(heading);
            }
            OcrBlockLabel::PageHeader => blocks.push(block_of(BlockKind::PageHeader, &members)),
            OcrBlockLabel::PageFooter => blocks.push(block_of(BlockKind::PageFooter, &members)),
            OcrBlockLabel::ListGroup => blocks.extend(list_items(&members)),
            OcrBlockLabel::Table => blocks.extend(table(&members, width, height)),
            OcrBlockLabel::Caption => blocks.push(block_of(BlockKind::Caption, &members)),
            OcrBlockLabel::Form | OcrBlockLabel::Footnote => blocks.extend(form(&members)),
            OcrBlockLabel::TableOfContents => blocks.push(block_of(BlockKind::Other, &members)),
        }
    }
    // The same rule as every page analysed from its geometry: small blocks
    // at the very top and bottom are the running header and footer, read
    // before the page and after it. The engine's labels for them stand.
    mark_running_lines_by(&mut blocks, body, f64::from(height.max(1)));
    blocks.sort_by_key(|block| match block.kind {
        BlockKind::PageHeader => 0,
        BlockKind::PageFooter => 2,
        _ => 1,
    });
    Labelled::Blocks(blocks)
}

/// Whether every line with text is in exactly one block, and every block
/// names lines that are there.
fn accounts_for_every_line(reading: &OcrResult) -> bool {
    let mut seen = vec![false; reading.lines.len()];
    for index in reading.blocks.iter().flat_map(|block| &block.lines) {
        match seen.get_mut(*index) {
            Some(seen) if !*seen => *seen = true,
            _ => return false,
        }
    }
    reading
        .lines
        .iter()
        .zip(&seen)
        .all(|(line, seen)| *seen || line.text.trim().is_empty())
}

/// The height most of the page's characters are set in, as the geometric
/// analysis measures it.
fn body_height(lines: &[Line]) -> f64 {
    let mut weighted = lines
        .iter()
        .filter(|line| !line.text.is_empty())
        .map(|line| (line.height(), line.text.chars().count()))
        .collect::<Vec<_>>();
    weighted.sort_by(|left, right| left.0.total_cmp(&right.0));
    let total = weighted.iter().map(|(_, count)| count).sum::<usize>();
    let mut seen = 0;
    for (height, count) in &weighted {
        seen += count;
        if seen * 2 >= total {
            return *height;
        }
    }
    weighted.first().map_or(100.0, |(height, _)| *height)
}

fn block_of(kind: BlockKind, lines: &[&Line]) -> LayoutBlock {
    block_of_lines(kind, lines.iter().map(|line| line.layout_line()).collect())
}

fn block_of_lines(kind: BlockKind, lines: Vec<LayoutLine>) -> LayoutBlock {
    let text = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let mut block = LayoutBlock::new(kind, text, TextSource::Ocr);
    block.bbox = union(lines.iter().filter_map(|line| line.bbox));
    block.confidence = mean_confidence(lines.iter().map(|line| line.confidence));
    block.lines = lines;
    block
}

/// A section header's level from how much larger than the body text it is
/// set, by the steps the geometric analysis uses; 4 for one at body size.
fn heading_level(lines: &[&Line], body: f64) -> Option<u8> {
    let mut heights = lines.iter().map(|line| line.height()).collect::<Vec<_>>();
    let ratio = median(&mut heights)? / body.max(1.0);
    Some(if ratio >= 1.6 {
        1
    } else if ratio >= 1.3 {
        2
    } else if ratio >= 1.18 {
        3
    } else {
        4
    })
}

/// Whether a line starts a list item, as the engine's own reading of a
/// list group splits it: a bullet, `1.`, `(2)`, `a)` or `(b)`, followed by
/// a space or alone on its line (the detector often cuts a bullet off as a
/// line of its own, and the next line then joins its item).
pub(super) fn starts_list_item(text: &str) -> bool {
    let text = text.trim_start();
    let mut characters = text.chars().peekable();
    let ends_marker = |rest: &mut std::iter::Peekable<std::str::Chars>| {
        rest.peek().is_none_or(|c| c.is_whitespace())
    };
    let Some(&first) = characters.peek() else {
        return false;
    };
    if matches!(first, '•' | '-' | '–' | '▪' | '◦' | '*' | '·' | '→') {
        characters.next();
        return ends_marker(&mut characters);
    }
    if first == '(' {
        characters.next();
    }
    let mut digits = 0;
    while characters.peek().is_some_and(char::is_ascii_digit) {
        characters.next();
        digits += 1;
    }
    if digits == 0 {
        if !characters.peek().is_some_and(char::is_ascii_alphabetic) {
            return false;
        }
        characters.next();
    } else if digits > 3 {
        return false;
    }
    if characters.next_if(|c| matches!(c, '.' | ')')).is_none() {
        return false;
    }
    ends_marker(&mut characters)
}

/// A list group as one block an item: an item starts at the group's first
/// line and at each line that [`starts_list_item`]; the lines between
/// belong to the item above them.
fn list_items(lines: &[&Line]) -> Vec<LayoutBlock> {
    let mut items: Vec<Vec<&Line>> = Vec::new();
    for line in lines {
        match items.last_mut() {
            Some(item) if !starts_list_item(&line.text) => item.push(line),
            _ => items.push(vec![line]),
        }
    }
    items
        .iter()
        .map(|item| block_of(BlockKind::ListItem, item))
        .collect()
}

/// A labelled value found in a block's lines.
struct Pair {
    key: String,
    key_bbox: [u32; 4],
    value: Vec<String>,
    value_boxes: Vec<[u32; 4]>,
    confidence: Vec<u8>,
    boxes: Vec<[u32; 4]>,
    /// The value is on lines of its own, under or beside its label, and
    /// the lines that continue it start where it starts.
    value_x0: Option<f64>,
}

/// One stretch of a form: labelled values, or the lines between them.
enum Part<'a> {
    Fields(Vec<Pair>),
    Text(Vec<&'a Line>),
}

/// Whether a line is a label with its value to come: `Invoice date:`.
fn bare_label(text: &str) -> bool {
    text.ends_with(':') && is_label(text)
}

/// A form's lines, in the engine's order, as stretches of labelled values
/// and of text. A labelled value is a `Label: value` line, or a `Label:`
/// line and the lines after it - beside it or under it - that are not
/// labels themselves; a `Label: value` line that runs to the right edge of
/// the block and wraps continues on the lines flush with it under it.
fn form_parts<'a>(lines: &[&'a Line]) -> Vec<Part<'a>> {
    let left = lines
        .iter()
        .map(|line| line.x0)
        .fold(f64::INFINITY, f64::min);
    let right = lines
        .iter()
        .map(|line| line.x1)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut parts: Vec<Part> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let pair = if let Some((key, value)) = split_key_value(&line.text) {
            Some(Pair {
                key: key.trim_end_matches(':').trim().to_owned(),
                key_bbox: line.bbox(),
                value: vec![value.to_owned()],
                value_boxes: vec![line.bbox()],
                confidence: vec![line.confidence],
                boxes: vec![line.bbox()],
                value_x0: None,
            })
        } else if bare_label(&line.text)
            && lines
                .get(index + 1)
                .is_some_and(|next| continues(next, line))
        {
            Some(Pair {
                key: line.text.trim_end_matches(':').trim().to_owned(),
                key_bbox: line.bbox(),
                value: Vec::new(),
                value_boxes: Vec::new(),
                confidence: vec![line.confidence],
                boxes: vec![line.bbox()],
                value_x0: Some(lines[index + 1].x0),
            })
        } else {
            None
        };
        let Some(mut pair) = pair else {
            match parts.last_mut() {
                Some(Part::Text(text)) => text.push(line),
                _ => parts.push(Part::Text(vec![line])),
            }
            index += 1;
            continue;
        };
        index += 1;
        let tolerance = (line.height() * 0.8).max(30.0);
        let mut above = line;
        while let Some(next) = lines.get(index) {
            let joins = match pair.value_x0 {
                // A value under or beside its label goes on while lines
                // start where it starts; the first is the value itself.
                Some(x0) => {
                    continues(next, above)
                        && (pair.value.is_empty() || (next.x0 - x0).abs() <= tolerance)
                }
                // `Label: value` goes on only where it wrapped.
                None => {
                    continues(next, above)
                        && above.x1 >= right - (right - left) * 0.2
                        && (next.x0 - line.x0).abs() <= tolerance
                }
            };
            if !joins {
                break;
            }
            pair.value.push(next.text.clone());
            pair.value_boxes.push(next.bbox());
            pair.confidence.push(next.confidence);
            pair.boxes.push(next.bbox());
            above = next;
            index += 1;
        }
        match parts.last_mut() {
            Some(Part::Fields(fields)) => fields.push(pair),
            _ => parts.push(Part::Fields(vec![pair])),
        }
    }
    parts
}

/// Whether `next` can carry on the value of the labelled line `above`
/// ends: not a label itself, and no further below it than a line's height.
fn continues(next: &Line, above: &Line) -> bool {
    split_key_value(&next.text).is_none()
        && !bare_label(&next.text)
        && next.y0 - above.y1 <= above.height().max(next.height())
}

fn fields_block(pairs: Vec<Pair>) -> LayoutBlock {
    let mut lines = Vec::new();
    let mut fields = Vec::new();
    for pair in pairs {
        let value = pair.value.join(" ");
        lines.push(LayoutLine {
            text: format!("{}: {value}", pair.key),
            bbox: union(pair.boxes.iter().copied()),
            confidence: mean_confidence(pair.confidence.iter().map(|value| Some(*value))),
        });
        fields.push(KeyValue {
            id: String::new(),
            key: pair.key,
            value,
            key_bbox: Some(pair.key_bbox),
            value_bbox: union(pair.value_boxes.iter().copied()),
        });
    }
    let mut block = block_of_lines(BlockKind::KeyValue, lines);
    block.fields = fields;
    block
}

/// A form or a footnote: a KeyValue block for each stretch of labelled
/// values, an Other block for each stretch of anything else.
fn form(lines: &[&Line]) -> Vec<LayoutBlock> {
    form_parts(lines)
        .into_iter()
        .map(|part| match part {
            Part::Fields(pairs) => fields_block(pairs),
            Part::Text(text) => block_of(BlockKind::Other, &text),
        })
        .collect()
}

/// A text block that is nothing but labelled values - `Invoice number:`,
/// `Invoice date:` and `Due date:` on lines of their own - as one KeyValue
/// block; none for prose, which a colon in a line or two does not make a
/// form.
fn fields_only(lines: &[&Line]) -> Option<Vec<LayoutBlock>> {
    let mut parts = form_parts(lines);
    match parts.as_slice() {
        [Part::Fields(_)] => match parts.pop() {
            Some(Part::Fields(pairs)) => Some(vec![fields_block(pairs)]),
            _ => None,
        },
        _ => None,
    }
}

/// A table block: the geometric analysis's table when it reads every line
/// of the block as one table, which knows wrapped cells, header rows and a
/// header over one row of values; otherwise the engine's own assembly
/// ([`grid`]). A block of one line is a paragraph.
fn table(lines: &[&Line], width: u32, height: u32) -> Vec<LayoutBlock> {
    if lines.len() < 2 {
        return vec![block_of(BlockKind::Paragraph, lines)];
    }
    let runs = lines
        .iter()
        .map(|line| TextRun {
            text: line.text.clone(),
            bbox: line.bbox(),
            bold: false,
            confidence: Some(line.confidence),
        })
        .collect::<Vec<_>>();
    // One block's lines are far inside the analysis's bounds, and it is
    // not stopped part of the way through one.
    let analysed = analyze_runs_within(&runs, width, height, &[], TextSource::Ocr, &|| false)
        .unwrap_or_default();
    if let [block] = analysed.as_slice()
        && block.kind == BlockKind::Table
        && same_words(lines, block)
    {
        return analysed;
    }
    grid(lines)
}

/// Whether a table's cells hold the lines' words, each once.
fn same_words(lines: &[&Line], block: &LayoutBlock) -> bool {
    let words = |texts: &mut dyn Iterator<Item = &str>| {
        let mut words = texts
            .flat_map(str::split_whitespace)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        words.sort();
        words
    };
    let Some(table) = &block.table else {
        return false;
    };
    words(&mut lines.iter().map(|line| line.text.as_str()))
        == words(
            &mut table
                .rows
                .iter()
                .flat_map(|row| &row.cells)
                .map(|cell| cell.text.as_str()),
        )
}

/// `numpy.median`: the middle value, or the mean of the two middle ones.
fn median_of(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut values = values.into_iter().collect::<Vec<_>>();
    median(&mut values).unwrap_or(0.0)
}

/// The engine's own table assembly (Indra's `table_html`): lines grouped
/// into rows by their vertical centres, wrapped rows joined to the row
/// above, columns anchored on the rows with the most cells, and each line
/// placed in the columns it covers. A first row that is one line across
/// most of the table is its title, a caption before it. The first row is
/// the header when there is more than one.
fn grid(lines: &[&Line]) -> Vec<LayoutBlock> {
    let mut rows = rows_by_centre(lines);
    let span_x = lines
        .iter()
        .map(|line| line.x1)
        .fold(f64::NEG_INFINITY, f64::max)
        - lines
            .iter()
            .map(|line| line.x0)
            .fold(f64::INFINITY, f64::min);
    let widest = rows.iter().map(Vec::len).max().unwrap_or(0);
    if rows.len() > 2
        && rows[0].len() == 1
        && widest > 1
        && rows[0][0].x1 - rows[0][0].x0 > 0.6 * span_x
    {
        let title = rows.remove(0)[0];
        let rest = lines
            .iter()
            .copied()
            .filter(|line| !std::ptr::eq(*line, title))
            .collect::<Vec<_>>();
        let mut blocks = vec![block_of(BlockKind::Caption, &[title])];
        blocks.extend(grid(&rest));
        return blocks;
    }
    let full = rows
        .iter()
        .filter(|row| row.len() == widest)
        .cloned()
        .collect::<Vec<_>>();
    rows = merge_wrapped(rows, lines);
    let bands = (0..widest)
        .map(|column| {
            (
                median_of(full.iter().map(|row| row[column].x0)),
                median_of(full.iter().map(|row| row[column].x1)),
            )
        })
        .collect::<Vec<_>>();
    let mut edges = vec![f64::NEG_INFINITY];
    edges.extend(bands.windows(2).map(|pair| (pair[0].1 + pair[1].0) / 2.0));
    edges.push(f64::INFINITY);
    // `numpy.searchsorted(edges, x, side="right") - 1`.
    let band_of = |x: f64| edges.partition_point(|edge| *edge <= x) as isize - 1;
    let header = rows.len() > 1;
    let mut block_lines = Vec::new();
    let mut table_rows = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        let mut members = row.clone();
        members.sort_by(|left, right| {
            left.y0
                .total_cmp(&right.y0)
                .then(left.x0.total_cmp(&right.x0))
        });
        // The column each cell starts in, with the lines in it and how
        // many columns it spans.
        let mut cells: Vec<Option<(usize, Vec<&Line>)>> = vec![None; widest];
        let last_column = widest as isize - 1;
        for line in members {
            let first = band_of(line.x0);
            let last = band_of(line.x1);
            let mut span = (first.max(0)..=last.min(last_column))
                .map(|column| column as usize)
                .filter(|column| {
                    let (x0, x1) = bands[*column];
                    line.x1.min(x1) - line.x0.max(x0) > 0.5 * (x1 - x0)
                })
                .collect::<Vec<_>>();
            if span.is_empty() {
                span.push(first.clamp(0, last_column) as usize);
            }
            let start = span[0];
            match &mut cells[start] {
                Some((columns, cell)) => {
                    *columns = (*columns).max(span.len());
                    cell.push(line);
                }
                empty => *empty = Some((span.len(), vec![line])),
            }
        }
        let mut row_cells = Vec::new();
        let mut column = 0;
        while column < widest {
            let (columns, mut cell) = cells[column].take().unwrap_or((1, Vec::new()));
            // A line that starts in a column a cell before it spans joins
            // that cell rather than being lost: Indra's own table drops it.
            let covered = column + 1..(column + columns).min(widest);
            for (_, more) in cells[covered].iter_mut().filter_map(Option::take) {
                cell.extend(more);
            }
            row_cells.push(LayoutCell {
                id: String::new(),
                text: cell
                    .iter()
                    .map(|line| line.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                bbox: union(cell.iter().map(|line| line.bbox())),
                header: header && row_index == 0,
            });
            // A cell across several columns keeps them, empty, so the
            // cells after it stay under their headings.
            for _ in 1..columns.min(widest - column) {
                row_cells.push(LayoutCell {
                    id: String::new(),
                    text: String::new(),
                    bbox: None,
                    header: header && row_index == 0,
                });
            }
            column += columns.max(1);
        }
        block_lines.push(LayoutLine {
            text: format!(
                "| {} |",
                row_cells
                    .iter()
                    .map(|cell| escape_cell(&cell.text))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
            bbox: union(row.iter().map(|line| line.bbox())),
            confidence: mean_confidence(row.iter().map(|line| Some(line.confidence))),
        });
        table_rows.push(LayoutRow {
            id: String::new(),
            cells: row_cells,
        });
    }
    let mut block = block_of_lines(BlockKind::Table, block_lines);
    block.table = Some(LayoutTable { rows: table_rows });
    vec![block]
}

/// Indra's `_rows`: lines in order of their vertical centres, each joining
/// the row before it when its centre is within half the median line height
/// of that row's mean centre; each row left to right.
fn rows_by_centre<'a>(lines: &[&'a Line]) -> Vec<Vec<&'a Line>> {
    let height = median_of(lines.iter().map(|line| line.y1 - line.y0));
    let mut order = lines.to_vec();
    order.sort_by(|left, right| left.center_y().total_cmp(&right.center_y()));
    let mut rows: Vec<Vec<&Line>> = Vec::new();
    for line in order {
        let joins = rows.last().is_some_and(|row| {
            let mean = row.iter().map(|member| member.center_y()).sum::<f64>() / row.len() as f64;
            (line.center_y() - mean).abs() < 0.5 * height
        });
        match rows.last_mut() {
            Some(row) if joins => row.push(line),
            _ => rows.push(vec![line]),
        }
    }
    for row in &mut rows {
        row.sort_by(|left, right| left.x0.total_cmp(&right.x0));
    }
    rows
}

/// Indra's `_merge_wrapped`: a row joins the row above when the gap to it
/// is under half the table's median gap between rows and under 0.6 line
/// heights, and it has fewer lines than the row above (a wrapped cell's
/// continuation fills only the columns that wrapped).
fn merge_wrapped<'a>(rows: Vec<Vec<&'a Line>>, lines: &[&Line]) -> Vec<Vec<&'a Line>> {
    if rows.len() < 3 {
        return rows;
    }
    let height = median_of(lines.iter().map(|line| line.y1 - line.y0));
    let gaps = rows
        .windows(2)
        .map(|pair| {
            let top = pair[1]
                .iter()
                .map(|line| line.y0)
                .fold(f64::INFINITY, f64::min);
            let bottom = pair[0]
                .iter()
                .map(|line| line.y1)
                .fold(f64::NEG_INFINITY, f64::max);
            top - bottom
        })
        .collect::<Vec<_>>();
    let usual = median_of(gaps.iter().copied());
    let mut merged = vec![rows[0].clone()];
    for (gap, pair) in gaps.iter().zip(rows.windows(2)) {
        let (above, row) = (&pair[0], &pair[1]);
        if *gap < 0.5 * usual && *gap < 0.6 * height && row.len() < above.len() {
            merged
                .last_mut()
                .expect("starts with a row")
                .extend(row.iter().copied());
        } else {
            merged.push(row.clone());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::{OcrBlock, OcrLine, PageSource};
    use crate::layout::{PageLayout, RouteSignals, ocr_page};

    /// A letter page scanned at 300 DPI, in pixels.
    const PAGE: (u32, u32) = (2550, 3300);

    /// A line at `x`, `y` pixels (its top left), 40 pixels tall - 9.6
    /// points - and 20 pixels a character.
    fn line(x: u32, y: u32, text: &str) -> OcrLine {
        sized(x, y, text, 40)
    }

    fn sized(x: u32, y: u32, text: &str, height: u32) -> OcrLine {
        OcrLine {
            text: text.to_owned(),
            bbox: [
                x,
                y,
                x + text.chars().count() as u32 * height / 2,
                y + height,
            ],
            confidence: 90,
        }
    }

    fn block(label: OcrBlockLabel, lines: impl IntoIterator<Item = usize>) -> OcrBlock {
        OcrBlock {
            label,
            bbox: [0, 0, 0, 0],
            lines: lines.into_iter().collect(),
        }
    }

    /// The page as the worker reads it, from the engine's lines and blocks.
    fn read(lines: Vec<OcrLine>, blocks: Vec<OcrBlock>) -> (String, PageLayout) {
        let text = lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let reading = OcrResult::new(text, 90.0)
            .with_lines(lines)
            .with_blocks(blocks);
        let page = ocr_page(1, reading, PAGE, None, RouteSignals::default(), &|| false);
        assert_eq!(page.source, PageSource::Ocr);
        (page.text, page.layout.expect("an OCR page has a layout"))
    }

    fn kinds(layout: &PageLayout) -> Vec<BlockKind> {
        layout.blocks.iter().map(|block| block.kind).collect()
    }

    /// Two columns, the engine's lines listed across them row by row, and a
    /// note at the top of the right column that the engine reads after the
    /// left column: its blocks and their lines are read in its order, not
    /// re-derived from where they sit.
    #[test]
    fn the_engines_block_and_line_order_is_kept() {
        let left = [
            "The tenant leases the premises for",
            "the term and pays the base rent in",
            "monthly installments when due.",
        ];
        let right = [
            "The landlord keeps the common areas",
            "in good repair and insures the",
            "building at replacement cost.",
        ];
        let mut lines = vec![line(1400, 300, "NOTE TO TENANT")];
        for (row, (l, r)) in left.iter().zip(right).enumerate() {
            let y = 420 + row as u32 * 50;
            lines.push(line(150, y, l));
            lines.push(line(1400, y, r));
        }
        let blocks = vec![
            block(OcrBlockLabel::Text, [1, 3, 5]),
            block(OcrBlockLabel::SectionHeader, [0]),
            block(OcrBlockLabel::Text, [2, 4, 6]),
        ];

        let (text, layout) = read(lines, blocks);

        assert_eq!(
            text,
            format!(
                "{}\n\nNOTE TO TENANT\n\n{}",
                left.join("\n"),
                right.join("\n")
            )
        );
        assert_eq!(
            kinds(&layout),
            [
                BlockKind::Paragraph,
                BlockKind::Heading,
                BlockKind::Paragraph
            ]
        );
        let ids = layout
            .blocks
            .iter()
            .map(|block| block.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["p1.b1", "p1.b2", "p1.b3"]);
        let first = &layout.blocks[0];
        assert_eq!(first.source, TextSource::Ocr);
        assert_eq!(first.confidence, Some(90));
        assert_eq!(first.lines.len(), 3);
        // Pixels at 300 DPI are 2.4 tenths of a point.
        assert_eq!(first.lines[0].bbox, Some([360, 1008, 1992, 1104]));
        assert_eq!(layout.blocks[1].level, Some(4));
    }

    /// A table block becomes a table: its rows, its cells, the header, and
    /// a `| a | b |` line a row in the page text.
    #[test]
    fn a_table_block_has_rows_and_cells() {
        let mut lines = Vec::new();
        for (row, cells) in [
            ["Item", "Qty", "Amount"],
            ["Bicycle tune-up", "2", "$180.00"],
            ["Brake cables", "4", "$36.00"],
            ["Labour", "3", "$255.00"],
        ]
        .iter()
        .enumerate()
        {
            let y = 600 + row as u32 * 60;
            lines.push(line(150, y, cells[0]));
            lines.push(line(1200, y, cells[1]));
            lines.push(line(1800, y, cells[2]));
        }
        let (text, layout) = read(lines, vec![block(OcrBlockLabel::Table, 0..12)]);

        assert_eq!(kinds(&layout), [BlockKind::Table]);
        let table = layout.blocks[0].table.as_ref().unwrap();
        let rows = table
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| cell.text.as_str())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            rows,
            [
                ["Item", "Qty", "Amount"],
                ["Bicycle tune-up", "2", "$180.00"],
                ["Brake cables", "4", "$36.00"],
                ["Labour", "3", "$255.00"],
            ]
        );
        assert!(table.rows[0].cells.iter().all(|cell| cell.header));
        assert!(table.rows[1].cells.iter().all(|cell| !cell.header));
        assert_eq!(table.rows[2].cells[1].id, "p1.b1.r3.c2");
        assert_eq!(
            text,
            "| Item | Qty | Amount |\n| Bicycle tune-up | 2 | $180.00 |\n\
             | Brake cables | 4 | $36.00 |\n| Labour | 3 | $255.00 |"
        );
    }

    fn cell_texts(block: &LayoutBlock) -> Vec<Vec<&str>> {
        block
            .table
            .as_ref()
            .unwrap()
            .rows
            .iter()
            .map(|row| row.cells.iter().map(|cell| cell.text.as_str()).collect())
            .collect()
    }

    fn units(line: &OcrLine) -> Line {
        let [x0, y0, x1, y1] = line.bbox.map(f64::from);
        Line {
            text: line.text.clone(),
            x0,
            y0,
            x1,
            y1,
            confidence: line.confidence,
        }
    }

    /// Indra's own assembly: a title across the table is its caption, a
    /// heading over two columns spans them, a wrapped cell's second line
    /// joins its row, and every word is kept.
    #[test]
    fn the_engines_table_assembly_finds_titles_spans_and_wrapped_cells() {
        let lines = [
            line(100, 100, "Quarterly results by region and segment"),
            line(100, 200, "Region"),
            line(700, 200, "Revenue and margin"),
            line(100, 300, "North"),
            line(700, 300, "$1.2M"),
            line(1100, 300, "18%"),
            line(100, 400, "South and"),
            line(700, 400, "$0.9M"),
            line(1100, 400, "12%"),
            line(100, 445, "islands"),
            line(100, 560, "West"),
            line(700, 560, "$2.1M"),
            line(1100, 560, "22%"),
        ]
        .iter()
        .map(units)
        .collect::<Vec<_>>();
        let refs = lines.iter().collect::<Vec<_>>();

        let blocks = grid(&refs);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].kind, BlockKind::Caption);
        assert_eq!(blocks[0].text, "Quarterly results by region and segment");
        assert_eq!(
            cell_texts(&blocks[1]),
            [
                vec!["Region", "Revenue and margin", ""],
                vec!["North", "$1.2M", "18%"],
                vec!["South and islands", "$0.9M", "12%"],
                vec!["West", "$2.1M", "22%"],
            ]
        );
        assert!(
            blocks[1].table.as_ref().unwrap().rows[0]
                .cells
                .iter()
                .all(|cell| cell.header)
        );
    }

    /// A form's labelled values are fields, whether on one line or a label
    /// with its value under it; the form's other lines stay text, in place.
    #[test]
    fn a_form_block_is_labelled_values() {
        let lines = vec![
            line(150, 300, "Invoice number: INV-2041"),
            line(150, 350, "Invoice date: 2026-04-01"),
            line(150, 450, "Bill to:"),
            line(150, 500, "Juniper Ridge Holdings Inc."),
            line(150, 550, "14 Harbour Road, Briarport"),
            line(150, 700, "Thank you for your business"),
        ];
        let (text, layout) = read(lines, vec![block(OcrBlockLabel::Form, 0..6)]);

        assert_eq!(kinds(&layout), [BlockKind::KeyValue, BlockKind::Other]);
        let fields = layout.blocks[0]
            .fields
            .iter()
            .map(|field| (field.key.as_str(), field.value.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            fields,
            [
                ("Invoice number", "INV-2041"),
                ("Invoice date", "2026-04-01"),
                (
                    "Bill to",
                    "Juniper Ridge Holdings Inc. 14 Harbour Road, Briarport"
                ),
            ]
        );
        assert_eq!(layout.blocks[0].fields[2].id, "p1.b1.f3");
        assert_eq!(
            text,
            "Invoice number: INV-2041\nInvoice date: 2026-04-01\n\
             Bill to: Juniper Ridge Holdings Inc. 14 Harbour Road, Briarport\n\n\
             Thank you for your business"
        );
    }

    /// A text block of nothing but labelled values is fields too; prose
    /// with a colon in it stays a paragraph.
    #[test]
    fn a_text_block_of_labelled_values_is_fields_and_prose_is_not() {
        let lines = vec![
            line(150, 300, "Policy number: HX-55120"),
            line(150, 350, "Effective date: March 4, 2026"),
            line(150, 500, "The insurer pays for: damage to the premises"),
            line(150, 550, "caused by fire, flood or storm."),
        ];
        let (_, layout) = read(
            lines,
            vec![
                block(OcrBlockLabel::Text, [0, 1]),
                block(OcrBlockLabel::Text, [2, 3]),
            ],
        );

        assert_eq!(kinds(&layout), [BlockKind::KeyValue, BlockKind::Paragraph]);
        assert_eq!(layout.blocks[0].fields.len(), 2);
        assert!(layout.blocks[1].fields.is_empty());
    }

    /// A list group is one block an item, split where a line starts with a
    /// marker, as the engine splits it: the group's first line starts an
    /// item whatever it is, and a marker cut off as a line of its own takes
    /// the line after it.
    #[test]
    fn a_list_group_is_split_into_items() {
        let lines = vec![
            line(150, 300, "The tenant shall:"),
            line(150, 350, "• keep the premises clean and"),
            line(190, 400, "in good repair;"),
            line(150, 450, "2) pay rent when due;"),
            line(150, 500, "(c)"),
            line(190, 550, "insure its own fixtures."),
        ];
        let (text, layout) = read(lines, vec![block(OcrBlockLabel::ListGroup, 0..6)]);

        assert_eq!(kinds(&layout), [BlockKind::ListItem; 4]);
        let texts = layout
            .blocks
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            texts,
            [
                "The tenant shall:",
                "• keep the premises clean and\nin good repair;",
                "2) pay rent when due;",
                "(c)\ninsure its own fixtures.",
            ]
        );
        assert_eq!(text, texts.join("\n\n"));
    }

    #[test]
    fn list_markers_follow_the_engines_rule() {
        for marker in [
            "• a", "- a", "–", "* a", "→ a", "1. a", "12) a", "(3) a", "a. a", "(B) a", "Q.",
        ] {
            assert!(starts_list_item(marker), "{marker}");
        }
        for text in [
            "-5 degrees",
            "1234. a",
            "e.g. this",
            "ab) a",
            "Total 3",
            "",
            "(",
            "1.5 m",
        ] {
            assert!(!starts_list_item(text), "{text}");
        }
    }

    /// The engine's page header and footer are kept as such, and a page
    /// number it read as text at the foot of the page is marked as the
    /// geometric analysis marks one; the header is read first and the
    /// footer last.
    #[test]
    fn page_furniture_is_marked_and_read_at_the_ends() {
        let lines = vec![
            line(150, 60, "Harbourline Lease - Execution Copy"),
            line(150, 400, "The tenant leases the premises."),
            line(1200, 3200, "7"),
            line(150, 3150, "Confidential"),
        ];
        let (text, layout) = read(
            lines,
            vec![
                block(OcrBlockLabel::Text, [2]),
                block(OcrBlockLabel::PageHeader, [0]),
                block(OcrBlockLabel::Text, [1]),
                block(OcrBlockLabel::PageFooter, [3]),
            ],
        );

        assert_eq!(
            kinds(&layout),
            [
                BlockKind::PageHeader,
                BlockKind::Paragraph,
                BlockKind::PageFooter,
                BlockKind::PageFooter
            ]
        );
        assert_eq!(
            text,
            "Harbourline Lease - Execution Copy\n\nThe tenant leases the premises.\n\n7\n\nConfidential"
        );
    }

    /// A section header's level is how much larger than the body it is.
    #[test]
    fn a_section_headers_level_is_its_size() {
        let lines = vec![
            sized(150, 300, "Lease Agreement", 80),
            sized(150, 450, "Premises", 52),
            line(150, 550, "The landlord leases to the tenant the premises"),
            line(150, 600, "described in the schedule to this agreement."),
        ];
        let (_, layout) = read(
            lines,
            vec![
                block(OcrBlockLabel::SectionHeader, [0]),
                block(OcrBlockLabel::SectionHeader, [1]),
                block(OcrBlockLabel::Text, [2, 3]),
            ],
        );

        let levels = layout
            .blocks
            .iter()
            .map(|block| block.level)
            .collect::<Vec<_>>();
        assert_eq!(levels, [Some(1), Some(2), None]);
    }

    /// Blocks that do not account for every line - one left out, one named
    /// twice, one that is not there - are not trusted: the page is analysed
    /// from its lines as if the engine had given none.
    #[test]
    fn blocks_that_do_not_account_for_every_line_are_not_used() {
        let lines = vec![
            line(150, 300, "First paragraph of the letter."),
            line(150, 500, "Second paragraph of the letter."),
        ];
        let (expected_text, expected) = read(lines.clone(), Vec::new());
        for blocks in [
            vec![block(OcrBlockLabel::Caption, [0])],
            vec![
                block(OcrBlockLabel::Caption, [0, 1]),
                block(OcrBlockLabel::Text, [1]),
            ],
            vec![block(OcrBlockLabel::Caption, [0, 1, 2])],
        ] {
            let (text, layout) = read(lines.clone(), blocks);
            assert_eq!(text, expected_text);
            assert_eq!(layout, expected);
        }
        // A line with no text need be in no block.
        let mut with_blank = lines.clone();
        with_blank.push(line(150, 700, " "));
        let (_, layout) = read(with_blank, vec![block(OcrBlockLabel::Caption, [0, 1])]);
        assert_eq!(kinds(&layout), [BlockKind::Caption]);
    }

    /// A stopped request keeps the engine's text, as a stopped analysis does.
    #[test]
    fn a_stopped_page_keeps_the_engines_text() {
        let reading = OcrResult::new("engine text", 80.0)
            .with_lines(vec![line(150, 300, "Some line")])
            .with_blocks(vec![block(OcrBlockLabel::Text, [0])]);
        let page = ocr_page(1, reading, PAGE, None, RouteSignals::default(), &|| true);
        assert_eq!(page.text, "engine text");
    }
}
