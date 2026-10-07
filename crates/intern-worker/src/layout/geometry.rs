//! Blocks from where text sits on a page: reading order from the page's
//! columns, tables from rows whose cells line up, labelled values paired
//! with their labels, headings from type size and weight.
//!
//! The input is runs: text on one line with no wide gap inside it, each
//! with its box. A PDF page's runs are built from PDFium's characters; an
//! OCR page's are the engine's lines. Everything after that is the same
//! for both.
//!
//! Reading order is a recursive cut of the page. A region is split into
//! columns where a vertical band of whitespace runs through it and the two
//! sides read as separate flows - prose on both sides, a column of labels
//! on both sides, or lines that do not share baselines. A band that only
//! separates the columns of a table is not a column break: the table's rows
//! stay whole. A line that crosses the band (a title, a full-width table)
//! splits the region into the parts above and below it instead. Failing a
//! column cut, a region is split at wide horizontal gaps, and a region with
//! neither is read row by row.

use serde::{Deserialize, Serialize};

use super::router::bounds::{MAX_GUTTER_CANDIDATES, MAX_RUNS, MAX_RUNS_PER_LINE};
use super::text::{
    escape_cell, is_heading_line, is_label, is_part_heading, median, opens_a_clause,
    reads_as_label, split_key_value,
};
use super::{
    BlockKind, KeyValue, LayoutBlock, LayoutCell, LayoutLine, LayoutRow, LayoutTable, TextSource,
    mean_confidence, union,
};

/// A run of text on one line with no wide gap inside it.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    /// `[x0, y0, x1, y1]` in tenths of a point, top-left origin, in the
    /// frame the text is read in.
    pub bbox: [u32; 4],
    /// Set in a bold face.
    #[serde(default)]
    pub bold: bool,
    /// OCR confidence, 0-100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
}

#[derive(Clone, Debug)]
struct Item {
    text: String,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    bold: bool,
    confidence: Option<u8>,
}

impl Item {
    fn height(&self) -> f64 {
        (self.y1 - self.y0).max(1.0)
    }

    fn center_y(&self) -> f64 {
        (self.y0 + self.y1) / 2.0
    }

    fn bbox(&self) -> [u32; 4] {
        [
            self.x0.max(0.0).round() as u32,
            self.y0.max(0.0).round() as u32,
            self.x1.max(0.0).round() as u32,
            self.y1.max(0.0).round() as u32,
        ]
    }

    fn words(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

/// What holds for the whole page: the usual height of its text and the
/// usual gap between one line and the next.
#[derive(Clone, Copy, Debug)]
struct PageStats {
    body_height: f64,
    line_gap: f64,
    page_width: f64,
    page_height: f64,
}

/// Blocks in reading order from a page's runs. Boxes are in the runs'
/// frame; ids are left for [`super::number_blocks`]. A page past the
/// analysis's bounds ([`super::router::bounds`]) has none.
pub fn analyze_runs(
    runs: &[TextRun],
    width: u32,
    height: u32,
    rulings: &[[u32; 4]],
    source: TextSource,
) -> Vec<LayoutBlock> {
    analyze_runs_within(runs, width, height, rulings, source, &|| false).unwrap_or_default()
}

/// [`analyze_runs`], or none: for a page with more runs than the analysis
/// takes on, in all or on one line, for one whose runs have text but none
/// of them the width to place it, and as soon as `stop` says so - the
/// request canceled, or its time up - between one region of the page and
/// the next. A caller with none keeps the page's text as it was read.
pub fn analyze_runs_within(
    runs: &[TextRun],
    width: u32,
    height: u32,
    rulings: &[[u32; 4]],
    source: TextSource,
    stop: &dyn Fn() -> bool,
) -> Option<Vec<LayoutBlock>> {
    let items = items_of(runs);
    if items.is_empty() {
        // Text with no geometry to read it by - glyphs too small to have
        // width - is not an empty page: an empty layout would be written
        // out as its text, and the text it had lost.
        return runs
            .iter()
            .all(|run| run.text.trim().is_empty())
            .then(Vec::new);
    }
    if too_crowded(&items) {
        return None;
    }
    let stats = page_stats(&items, f64::from(width.max(1)), f64::from(height.max(1)));
    let mut leaves = Vec::new();
    if !cut_region(
        &items,
        (0..items.len()).collect(),
        f64::from(width.max(1)),
        0,
        &mut leaves,
        stop,
    ) {
        return None;
    }
    let rulings = rulings
        .iter()
        .map(|ruling| ruling.map(f64::from))
        .collect::<Vec<_>>();
    let mut blocks = Vec::new();
    for leaf in leaves {
        if stop() {
            return None;
        }
        blocks.extend(leaf_blocks(&items, leaf, &stats, &rulings, source));
    }
    mark_running_lines(&mut blocks, &stats);
    // A page's running header is read before it and its footer after it,
    // wherever the cut put them.
    blocks.sort_by_key(|block| match block.kind {
        BlockKind::PageHeader => 0,
        BlockKind::PageFooter => 2,
        _ => 1,
    });
    Some(blocks)
}

/// The runs the analysis reads: those with text and width.
fn items_of(runs: &[TextRun]) -> Vec<Item> {
    runs.iter()
        .filter(|run| !run.text.trim().is_empty() && run.bbox[2] > run.bbox[0])
        .map(|run| Item {
            text: run.text.trim().to_owned(),
            x0: f64::from(run.bbox[0]),
            y0: f64::from(run.bbox[1]),
            x1: f64::from(run.bbox[2]),
            y1: f64::from(run.bbox[3].max(run.bbox[1] + 1)),
            bold: run.bold,
            confidence: run.confidence,
        })
        .collect()
}

/// The band down the page an item's middle falls in: five points tall, the
/// line [`too_crowded`] counts it on.
fn band(item: &Item) -> i64 {
    (item.center_y() / 50.0).floor() as i64
}

/// Whether a page has more runs than the analysis takes on: more than
/// [`MAX_RUNS`] in all, or more than [`MAX_RUNS_PER_LINE`] within five
/// points of each other down the page - where the comparisons of each run
/// with the others on its line add up.
fn too_crowded(items: &[Item]) -> bool {
    if items.len() > MAX_RUNS {
        return true;
    }
    let mut lines: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    for item in items {
        let count = lines.entry(band(item)).or_default();
        *count += 1;
        if *count > MAX_RUNS_PER_LINE {
            return true;
        }
    }
    false
}

/// How crowded a page's runs are, counted as [`too_crowded`] counts them:
/// the runs the analysis reads, and the most of them on one line. The
/// routing calibration (`examples/route_calibration.rs`) measures the
/// corpus against [`MAX_RUNS`] and [`MAX_RUNS_PER_LINE`] with it.
pub fn crowding(runs: &[TextRun]) -> (usize, usize) {
    let items = items_of(runs);
    let mut lines: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut most = 0;
    for item in &items {
        let count = lines.entry(band(item)).or_default();
        *count += 1;
        most = most.max(*count);
    }
    (items.len(), most)
}

/// The body text height is the height most of the page's characters are set
/// in; the line gap is the usual space between a line and the one below it.
fn page_stats(items: &[Item], page_width: f64, page_height: f64) -> PageStats {
    let mut weighted = items
        .iter()
        .map(|item| (item.height(), item.text.chars().count().max(1)))
        .collect::<Vec<_>>();
    weighted.sort_by(|left, right| left.0.total_cmp(&right.0));
    let total = weighted.iter().map(|(_, count)| count).sum::<usize>();
    let mut seen = 0;
    let mut body_height = weighted[0].0;
    for (height, count) in &weighted {
        seen += count;
        if seen * 2 >= total {
            body_height = *height;
            break;
        }
    }
    let rows = rows_of(items, &(0..items.len()).collect::<Vec<_>>());
    let mut gaps = rows
        .windows(2)
        .filter_map(|pair| {
            let gap = pair[1].y0 - pair[0].y1;
            (gap >= 0.0 && gap < body_height * 3.0).then_some(gap)
        })
        .collect::<Vec<_>>();
    let line_gap = median(&mut gaps).unwrap_or(body_height * 0.3);
    PageStats {
        body_height,
        line_gap,
        page_width,
        page_height,
    }
}

/// One visual line of a region: its runs left to right.
#[derive(Clone, Debug)]
struct Row {
    members: Vec<usize>,
    y0: f64,
    y1: f64,
}

impl Row {
    fn height(&self) -> f64 {
        (self.y1 - self.y0).max(1.0)
    }
}

/// Groups runs into rows: a run joins a row when its middle is within the
/// row's band and the row's middle within its own, so text on one baseline
/// joins whatever its size and two lines half a line apart do not.
fn rows_of(items: &[Item], ids: &[usize]) -> Vec<Row> {
    let mut sorted = ids.to_vec();
    sorted.sort_by(|left, right| {
        items[*left]
            .center_y()
            .total_cmp(&items[*right].center_y())
            .then(items[*left].x0.total_cmp(&items[*right].x0))
            .then(left.cmp(right))
    });
    let mut rows: Vec<Row> = Vec::new();
    // The widest run on the row being built. Its runs are kept left to
    // right, so the only ones a run can overlap start less than that far
    // to its left, or before its right edge: the rest are checked by where
    // they start, not one by one.
    let mut widest = 0.0_f64;
    for id in sorted {
        let item = &items[id];
        let joins = rows.last().is_some_and(|row| {
            let row_center = (row.y0 + row.y1) / 2.0;
            let item_center = item.center_y();
            if item_center < row.y0
                || item_center > row.y1
                || row_center < item.y0
                || row_center > item.y1
            {
                return false;
            }
            let from = row
                .members
                .partition_point(|member| items[*member].x0 < item.x0 - widest - 1.0);
            let to = row
                .members
                .partition_point(|member| items[*member].x0 < item.x1 + 1.0);
            // A run that would overlap another on the row is a line of its
            // own, however near its baseline - unless the two share a
            // baseline and only touch: a date set a little too wide for its
            // column runs into the next column's value.
            row.members[from..to.max(from)].iter().all(|member| {
                let other = &items[*member];
                other.x1 <= item.x0 + 1.0
                    || item.x1 <= other.x0 + 1.0
                    || overflows_into(other, item)
            })
        });
        if joins {
            let row = rows.last_mut().expect("checked above");
            let at = row.members.partition_point(|member| {
                items[*member]
                    .x0
                    .total_cmp(&item.x0)
                    .then(member.cmp(&id))
                    .is_lt()
            });
            row.members.insert(at, id);
            row.y0 = row.y0.min(item.y0);
            row.y1 = row.y1.max(item.y1);
            widest = widest.max(item.x1 - item.x0);
        } else {
            rows.push(Row {
                members: vec![id],
                y0: item.y0,
                y1: item.y1,
            });
            widest = item.x1 - item.x0;
        }
    }
    for row in &mut rows {
        row.members.sort_by(|left, right| {
            items[*left]
                .x0
                .total_cmp(&items[*right].x0)
                .then(left.cmp(right))
        });
    }
    rows
}

/// Whether two runs on one baseline overlap only where the end of one runs
/// into the start of the other: the same top and bottom, and less than a
/// third of the shorter one covered. Text printed over text covers far
/// more of it.
fn overflows_into(left: &Item, right: &Item) -> bool {
    let height = left.height().min(right.height());
    let same_line =
        (left.y0 - right.y0).abs() <= height * 0.1 && (left.y1 - right.y1).abs() <= height * 0.1;
    let overlap = left.x1.min(right.x1) - left.x0.max(right.x0);
    let shorter = (left.x1 - left.x0).min(right.x1 - right.x0);
    same_line && overlap < shorter / 3.0
}

fn median_height(items: &[Item], ids: &[usize]) -> f64 {
    let mut heights = ids.iter().map(|id| items[*id].height()).collect::<Vec<_>>();
    median(&mut heights).unwrap_or(100.0)
}

/// How a region divides.
enum Cut {
    /// Read the left side, then the right.
    Columns(Vec<usize>, Vec<usize>),
    /// Read these parts top to bottom.
    Bands(Vec<Vec<usize>>),
}

/// The deepest a region is cut: far more than any page needs, and a bound
/// on a page built to make the recursion pathological.
const MAX_DEPTH: usize = 32;

/// Cuts a region into the leaves read row by row, in reading order. False
/// when `stop` said to stop, and the leaves are then not all there.
fn cut_region(
    items: &[Item],
    ids: Vec<usize>,
    page_width: f64,
    depth: usize,
    leaves: &mut Vec<Vec<usize>>,
    stop: &dyn Fn() -> bool,
) -> bool {
    if stop() {
        return false;
    }
    if ids.len() < 2 || depth >= MAX_DEPTH {
        leaves.push(ids);
        return true;
    }
    let cut = column_cut(items, &ids, page_width).or_else(|| horizontal_cut(items, &ids));
    match cut {
        Some(Cut::Columns(left, right)) => {
            cut_region(items, left, page_width, depth + 1, leaves, stop)
                && cut_region(items, right, page_width, depth + 1, leaves, stop)
        }
        Some(Cut::Bands(bands)) => bands
            .into_iter()
            .all(|band| cut_region(items, band, page_width, depth + 1, leaves, stop)),
        None => {
            leaves.push(ids);
            true
        }
    }
}

/// Splits a region at horizontal gaps of more than a line and a quarter of
/// its usual text height. A region with no such gap is not split.
fn horizontal_cut(items: &[Item], ids: &[usize]) -> Option<Cut> {
    let height = median_height(items, ids);
    let rows = rows_of(items, ids);
    let mut bands: Vec<Vec<usize>> = Vec::new();
    let mut bottom = f64::NEG_INFINITY;
    for (index, row) in rows.iter().enumerate() {
        let gap = row.y0 - bottom > height * 1.25;
        let labelled = index > 0 && labels_over(items, &rows[index - 1], row);
        if bands.is_empty() || (gap && !labelled) {
            bands.push(Vec::new());
        }
        bottom = bottom.max(row.y1);
        bands
            .last_mut()
            .expect("pushed above")
            .extend(row.members.iter().copied());
    }
    (bands.len() > 1).then_some(Cut::Bands(bands))
}

/// Whether a row of small labels stands over the values filled in under
/// them - `TO OWNER` over the owner's name, the boxes of a form - which
/// may sit further apart than lines of text do and still belong together:
/// every value starts under a label, set larger than every label, and no
/// more than two of its lines below.
fn labels_over(items: &[Item], labels: &Row, values: &Row) -> bool {
    let largest_label = labels
        .members
        .iter()
        .map(|member| items[*member].height())
        .fold(0.0, f64::max);
    let smallest_value = values
        .members
        .iter()
        .map(|member| items[*member].height())
        .fold(f64::INFINITY, f64::min);
    let tolerance = tolerance(smallest_value);
    largest_label <= smallest_value * 0.85
        && values.y0 - labels.y1 <= smallest_value * 2.0
        && values.members.iter().all(|value| {
            labels
                .members
                .iter()
                .any(|label| (items[*value].x0 - items[*label].x0).abs() <= tolerance)
        })
}

/// A gutter: a vertical band of whitespace, `[start, end]`, through the
/// consecutive rows `first..=last` of a region.
#[derive(Clone, Copy, Debug)]
struct Gutter {
    start: f64,
    end: f64,
    first: usize,
    last: usize,
}

/// How a run that crosses a gutter relates to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Crossing {
    /// Clear of the gutter on the left or right.
    Left,
    Right,
    /// Over it, but mostly on one side: a long value running a little into
    /// the gutter belongs to the side it starts on.
    SpillsLeft,
    SpillsRight,
    /// Across it: a title, a full-width line, a table row.
    Spans,
}

fn crossing(item: &Item, start: f64, end: f64, left_edge: f64, right_edge: f64) -> Crossing {
    if item.x1 <= start + 5.0 {
        return Crossing::Left;
    }
    if item.x0 >= end - 5.0 {
        return Crossing::Right;
    }
    let on_left = (start - item.x0.max(left_edge)).max(0.0) / (start - left_edge).max(1.0);
    let on_right = (item.x1.min(right_edge) - end).max(0.0) / (right_edge - end).max(1.0);
    let middle = (item.x0 + item.x1) / 2.0;
    if (on_left >= 0.3 && on_right >= 0.3) || (middle > start && middle < end) {
        Crossing::Spans
    } else if middle <= start {
        Crossing::SpillsLeft
    } else {
        Crossing::SpillsRight
    }
}

/// The free stretches of one row between `left_edge` and `right_edge`.
fn free_intervals(items: &[Item], row: &Row, left_edge: f64, right_edge: f64) -> Vec<(f64, f64)> {
    let mut free = Vec::new();
    let mut cursor = left_edge;
    for member in &row.members {
        let item = &items[*member];
        if item.x0 > cursor {
            free.push((cursor, item.x0));
        }
        cursor = cursor.max(item.x1);
    }
    if right_edge > cursor {
        free.push((cursor, right_edge));
    }
    free
}

/// Where two rows' free stretches overlap by at least `minimum`. Both are
/// left to right and do not overlap themselves, as [`free_intervals`]
/// makes them, so one pass over each finds every overlap, in order.
fn intersect(these: &[(f64, f64)], those: &[(f64, f64)], minimum: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let (mut this, mut that) = (0, 0);
    while this < these.len() && that < those.len() {
        let (a0, a1) = these[this];
        let (b0, b1) = those[that];
        let start = a0.max(b0);
        let end = a1.min(b1);
        if end - start >= minimum {
            out.push((start, end));
        }
        if a1 < b1 {
            this += 1;
        } else {
            that += 1;
        }
    }
    out
}

/// The column cut through a region, if it has one.
///
/// A gutter is found where three consecutive rows leave the same band of
/// whitespace free. It is then followed up and down the region, through
/// rows that leave it free, and through rows whose text runs a little into
/// it from one side when they sit at the region's usual line spacing; a
/// row that spans it ends it. The gutter with the most rows that have text
/// on both sides, and whose two sides read as separate flows, cuts the
/// region: into columns where it runs the region's whole height, and into
/// the parts above, alongside, and below it where it does not.
fn column_cut(items: &[Item], ids: &[usize], page_width: f64) -> Option<Cut> {
    if ids.len() < 4 {
        return None;
    }
    let height = median_height(items, ids);
    let rows = rows_of(items, ids);
    if rows.len() < 3 {
        return None;
    }
    let left_edge = ids
        .iter()
        .map(|id| items[*id].x0)
        .fold(f64::INFINITY, f64::min);
    let right_edge = ids
        .iter()
        .map(|id| items[*id].x1)
        .fold(f64::NEG_INFINITY, f64::max);
    let minimum = (height * 0.9).max(60.0);
    let free = rows
        .iter()
        .map(|row| free_intervals(items, row, left_edge, right_edge))
        .collect::<Vec<_>>();
    let mut candidates: Vec<(f64, f64)> = Vec::new();
    for window in free.windows(3) {
        if candidates.len() >= MAX_GUTTER_CANDIDATES {
            break;
        }
        for (start, end) in intersect(
            &intersect(&window[0], &window[1], minimum),
            &window[2],
            minimum,
        ) {
            // A band at the region's edge is a margin, not a gutter.
            if start <= left_edge + 1.0 || end >= right_edge - 1.0 {
                continue;
            }
            if !candidates
                .iter()
                .any(|(other_start, other_end)| start < *other_end && end > *other_start)
            {
                candidates.push((start, end));
            }
        }
    }
    let mut gutters = candidates
        .into_iter()
        .filter_map(|candidate| {
            follow_gutter(
                items, &rows, &free, candidate, minimum, left_edge, right_edge, height,
            )
        })
        .collect::<Vec<_>>();
    // Most rows with text on both sides first; then the widest.
    gutters.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then((right.1.end - right.1.start).total_cmp(&(left.1.end - left.1.start)))
            .then(left.1.start.total_cmp(&right.1.start))
    });
    for (_, gutter) in gutters {
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut spanning = Vec::new();
        for row in &rows[gutter.first..=gutter.last] {
            for member in &row.members {
                match crossing(
                    &items[*member],
                    gutter.start,
                    gutter.end,
                    left_edge,
                    right_edge,
                ) {
                    Crossing::Left | Crossing::SpillsLeft => left.push(*member),
                    Crossing::Right | Crossing::SpillsRight => right.push(*member),
                    Crossing::Spans => spanning.push(*member),
                }
            }
        }
        let left_rows = rows_of(items, &left);
        let right_rows = rows_of(items, &right);
        if !separate_flows(items, &left_rows, &right_rows, page_width) {
            continue;
        }
        if gutter.first == 0 && gutter.last == rows.len() - 1 {
            // A run over the gutter - one a row let spill into it before a
            // later row narrowed the gutter under it - goes with the side
            // its middle is on. A cut regroups text; it never loses any.
            let middle = (gutter.start + gutter.end) / 2.0;
            for member in spanning {
                let item = &items[member];
                if (item.x0 + item.x1) / 2.0 <= middle {
                    left.push(member);
                } else {
                    right.push(member);
                }
            }
            return Some(Cut::Columns(left, right));
        }
        let band = |range: std::ops::Range<usize>| {
            rows[range]
                .iter()
                .flat_map(|row| row.members.iter().copied())
                .collect::<Vec<_>>()
        };
        let bands = [
            band(0..gutter.first),
            band(gutter.first..gutter.last + 1),
            band(gutter.last + 1..rows.len()),
        ]
        .into_iter()
        .filter(|band| !band.is_empty())
        .collect::<Vec<_>>();
        return Some(Cut::Bands(bands));
    }
    None
}

/// Follows a gutter seen in three rows up and down the region. Returns how
/// many of its rows have text on both sides, and the gutter, narrowed to
/// what every row leaves free.
#[allow(clippy::too_many_arguments)]
fn follow_gutter(
    items: &[Item],
    rows: &[Row],
    free: &[Vec<(f64, f64)>],
    (start, end): (f64, f64),
    minimum: f64,
    left_edge: f64,
    right_edge: f64,
    height: f64,
) -> Option<(usize, Gutter)> {
    let seed = (0..rows.len().saturating_sub(2)).find(|index| {
        (0..3).all(|offset| {
            free[index + offset]
                .iter()
                .any(|(free_start, free_end)| *free_start <= start + 0.5 && *free_end >= end - 0.5)
        })
    })?;
    let mut gutter = Gutter {
        start,
        end,
        first: seed,
        last: seed + 2,
    };
    // Whether a row can join the gutter, and the gutter it leaves. A gap of
    // a line and a half ends a gutter whatever the row: that is a new
    // section of the page, and its columns, if it has any, are its own.
    let joins = |gutter: &Gutter, index: usize, neighbour: usize| -> Option<(f64, f64)> {
        let gap = {
            let (row, other) = (&rows[index], &rows[neighbour]);
            if index < neighbour {
                other.y0 - row.y1
            } else {
                row.y0 - other.y1
            }
        };
        if gap > height * 1.5 {
            return None;
        }
        let narrowed = free[index]
            .iter()
            .map(|(free_start, free_end)| (free_start.max(gutter.start), free_end.min(gutter.end)))
            .filter(|(narrow_start, narrow_end)| narrow_end - narrow_start >= minimum)
            .max_by(|left, right| (left.1 - left.0).total_cmp(&(right.1 - right.0)));
        if narrowed.is_some() {
            return narrowed;
        }
        // Text running into the gutter from one side, on a row at the
        // region's line spacing.
        let row = &rows[index];
        let close = gap <= height * 0.6;
        let spills = row.members.iter().all(|member| {
            crossing(
                &items[*member],
                gutter.start,
                gutter.end,
                left_edge,
                right_edge,
            ) != Crossing::Spans
        });
        (close && spills).then_some((gutter.start, gutter.end))
    };
    while gutter.first > 0 {
        match joins(&gutter, gutter.first - 1, gutter.first) {
            Some((narrow_start, narrow_end)) => {
                gutter.start = narrow_start;
                gutter.end = narrow_end;
                gutter.first -= 1;
            }
            None => break,
        }
    }
    while gutter.last + 1 < rows.len() {
        match joins(&gutter, gutter.last + 1, gutter.last) {
            Some((narrow_start, narrow_end)) => {
                gutter.start = narrow_start;
                gutter.end = narrow_end;
                gutter.last += 1;
            }
            None => break,
        }
    }
    let both = rows[gutter.first..=gutter.last]
        .iter()
        .filter(|row| {
            let sides = row
                .members
                .iter()
                .map(|member| {
                    crossing(
                        &items[*member],
                        gutter.start,
                        gutter.end,
                        left_edge,
                        right_edge,
                    )
                })
                .collect::<Vec<_>>();
            sides
                .iter()
                .any(|side| matches!(side, Crossing::Left | Crossing::SpillsLeft))
                && sides
                    .iter()
                    .any(|side| matches!(side, Crossing::Right | Crossing::SpillsRight))
        })
        .count();
    (both >= 2).then_some((both, gutter))
}

/// Whether two sides of a gutter are separate flows rather than the columns
/// of one table: labels down both sides, prose on both sides, a grid of
/// labelled values beside a block with none, or lines that mostly do not
/// share a baseline - unless the two sides are one table whose cells wrap
/// (see [`one_table`]).
fn separate_flows(items: &[Item], left: &[Row], right: &[Row], page_width: f64) -> bool {
    if left.len() < 2 || right.len() < 2 {
        return false;
    }
    let labelled = |rows: &[Row]| {
        rows.iter()
            .filter(|row| starts_with_label(items, row))
            .count()
    };
    // Rows whose label and value are both on this side.
    let self_contained = |rows: &[Row]| {
        rows.iter()
            .filter(|row| {
                let first = &items[row.members[0]];
                (first.text.ends_with(':') && is_label(&first.text) && row.members.len() >= 2)
                    || split_key_value(&first.text).is_some()
            })
            .count()
    };
    let (left_labels, right_labels) = (labelled(left), labelled(right));
    // A table running across the gutter, its rows level on both sides and
    // none of them a label: boxes of labelled values over a schedule of
    // coverage or line items read that way, and cutting there splits every
    // row of the table in two. The labels above it say nothing then about
    // two flows side by side.
    let table_across = spanning_rows(items, left, right) >= 3;
    // Labels down both sides: two forms side by side.
    if left_labels >= 2 && right_labels >= 2 && !table_across {
        return true;
    }
    if one_table(items, left, right) {
        return false;
    }
    let prose = |rows: &[Row]| {
        let total = rows
            .iter()
            .flat_map(|row| &row.members)
            .map(|member| items[*member].text.chars().count())
            .sum::<usize>()
            .max(1);
        let prose = rows
            .iter()
            .flat_map(|row| &row.members)
            .map(|member| &items[*member])
            .filter(|item| item.words() >= 5 && item.x1 - item.x0 >= page_width * 0.2)
            .map(|item| item.text.chars().count())
            .sum::<usize>();
        prose * 2 >= total
    };
    // Lines of a column follow one another at the line spacing, with the
    // odd paragraph gap. A table column whose neighbour wraps has a line,
    // then a gap where the neighbour's next lines are.
    let dense = |rows: &[Row]| {
        let gaps = rows
            .windows(2)
            .filter(|pair| pair[1].y0 - pair[0].y1 > pair[0].height())
            .count();
        gaps * 2 < rows.len()
    };
    if left.len() >= 3
        && right.len() >= 3
        && prose(left)
        && prose(right)
        && dense(left)
        && dense(right)
    {
        return true;
    }
    // A grid of labelled values beside a block with none: an address beside
    // a statement's date and account number.
    if !table_across
        && ((left_labels == 0 && self_contained(right) >= 3)
            || (right_labels == 0 && self_contained(left) >= 3))
    {
        return true;
    }
    // Independent baselines: the two sides were set separately.
    let shared = |these: &[Row], those: &[Row]| {
        let matched = these
            .iter()
            .filter(|row| {
                those.iter().any(|other| {
                    (row.y1 - other.y1).abs() <= row.height().min(other.height()) * 0.3
                })
            })
            .count();
        matched as f64 / these.len().max(1) as f64
    };
    left.len() >= 3 && right.len() >= 3 && shared(left, right).max(shared(right, left)) < 0.5
}

/// Rows on the left level with a row on the right where both are cells -
/// two or more on each side - and neither opens with a label: a table
/// running across the gutter. A value wrapped onto a line of its own beside
/// another is one cell, and two forms side by side have those.
fn spanning_rows(items: &[Item], left: &[Row], right: &[Row]) -> usize {
    let cells = |row: &Row| row.members.len() >= 2 && !starts_with_label(items, row);
    left.iter()
        .filter(|row| cells(row))
        .filter(|row| {
            right.iter().any(|other| {
                cells(other) && (row.y1 - other.y1).abs() <= row.height().min(other.height()) * 0.3
            })
        })
        .count()
}

/// Whether a row opens with a label: `Date:`, or `Date: May 1`.
fn starts_with_label(items: &[Item], row: &Row) -> bool {
    let first = &items[row.members[0]];
    (first.text.ends_with(':') && is_label(&first.text)) || split_key_value(&first.text).is_some()
}

/// Whether the two sides of a gutter are the columns of one table: one
/// side is a grid - rows of two or more cells that line up with each other,
/// not labels with their values - or a column of figures, and every row on
/// the side with fewer rows starts level with a row on the other, which
/// only adds the lines its cells wrap onto. A test plan whose last column
/// wraps, or line items beside their amounts, reads this way; two columns
/// of prose do not, because neither side is a grid.
fn one_table(items: &[Item], left: &[Row], right: &[Row]) -> bool {
    let grid = |rows: &[Row]| {
        let height = median_height(
            items,
            &rows
                .iter()
                .flat_map(|row| row.members.iter().copied())
                .collect::<Vec<_>>(),
        );
        let tolerance = tolerance(height);
        let starts = |row: &Row| {
            row.members
                .iter()
                .skip(1)
                .map(|member| items[*member].x0)
                .collect::<Vec<_>>()
        };
        let candidates = rows
            .iter()
            .filter(|row| row.members.len() >= 2 && !starts_with_label(items, row))
            .collect::<Vec<_>>();
        let aligned = candidates
            .iter()
            .enumerate()
            .filter(|(index, row)| {
                candidates.iter().enumerate().any(|(other_index, other)| {
                    other_index != *index
                        && starts(row).iter().any(|x| {
                            starts(other)
                                .iter()
                                .any(|other_x| (x - other_x).abs() <= tolerance)
                        })
                })
            })
            .count();
        aligned >= 2 && aligned * 2 >= rows.len()
    };
    let figures = |rows: &[Row]| {
        let cells = rows
            .iter()
            .flat_map(|row| &row.members)
            .map(|member| &items[*member])
            .collect::<Vec<_>>();
        let figures = cells.iter().filter(|item| figure(&item.text)).count();
        figures >= 2 && figures * 3 >= cells.len() * 2
    };
    let (fewer, more) = if left.len() <= right.len() {
        (left, right)
    } else {
        (right, left)
    };
    let in_step = fewer
        .iter()
        .filter(|row| {
            more.iter()
                .any(|other| (row.y0 - other.y0).abs() <= row.height().min(other.height()) * 0.3)
        })
        .count()
        * 5
        >= fewer.len() * 4;
    in_step && (grid(left) || grid(right) || figures(left) || figures(right))
}

/// A figure: an amount, a quantity, a date in numbers - digits, with no
/// more than a few letters (a currency code, a unit).
fn figure(text: &str) -> bool {
    let digits = text.chars().filter(char::is_ascii_digit).count();
    let letters = text
        .chars()
        .filter(|character| character.is_alphabetic())
        .count();
    digits > 0 && letters <= 3
}

/// What one row of a leaf is.
#[derive(Clone, Debug)]
struct Line {
    cells: Vec<usize>,
    y0: f64,
    y1: f64,
    x0: f64,
    x1: f64,
}

impl Line {
    fn height(&self) -> f64 {
        (self.y1 - self.y0).max(1.0)
    }

    fn text(&self, items: &[Item]) -> String {
        self.cells
            .iter()
            .map(|cell| items[*cell].text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn bbox(&self) -> [u32; 4] {
        [
            self.x0.max(0.0).round() as u32,
            self.y0.max(0.0).round() as u32,
            self.x1.max(0.0).round() as u32,
            self.y1.max(0.0).round() as u32,
        ]
    }

    fn confidence(&self, items: &[Item]) -> Option<u8> {
        mean_confidence(self.cells.iter().map(|cell| items[*cell].confidence))
    }

    fn layout_line(&self, items: &[Item]) -> LayoutLine {
        LayoutLine {
            text: self.text(items),
            bbox: Some(self.bbox()),
            confidence: self.confidence(items),
        }
    }

    fn all_bold(&self, items: &[Item]) -> bool {
        self.cells.iter().all(|cell| items[*cell].bold)
    }
}

fn lines_of(items: &[Item], ids: &[usize]) -> Vec<Line> {
    rows_of(items, ids)
        .into_iter()
        .map(|row| {
            let x0 = row
                .members
                .iter()
                .map(|member| items[*member].x0)
                .fold(f64::INFINITY, f64::min);
            let x1 = row
                .members
                .iter()
                .map(|member| items[*member].x1)
                .fold(f64::NEG_INFINITY, f64::max);
            Line {
                cells: row.members,
                y0: row.y0,
                y1: row.y1,
                x0,
                x1,
            }
        })
        .collect()
}

/// How far apart two edges may be and still line up.
fn tolerance(height: f64) -> f64 {
    (height * 0.8).max(30.0)
}

/// Whether a cell lines up with a column: the same left edge, or - for a
/// short cell, a number set flush right or a centred heading - the same
/// right edge or centre. Lines of prose end where they happen to, so a long
/// cell lines up by its right edge only when it ends exactly where the
/// column does: a total's label set flush right.
fn aligned(item: &Item, column: &Column, tolerance: f64) -> bool {
    const FLUSH: f64 = 2.0 * super::UNITS_PER_POINT;
    (item.x0 - column.x0).abs() <= tolerance
        || (item.x1 - column.x1).abs() <= FLUSH
        || (item.words() <= 3
            && ((item.x1 - column.x1).abs() <= tolerance
                || ((item.x0 + item.x1) / 2.0 - (column.x0 + column.x1) / 2.0).abs() <= tolerance))
}

/// Two cells or more of prose side by side - several words each, set at
/// the body size and filling a good part of the page - are columns of text,
/// not a row. A form's long labels are set smaller and narrower.
fn prose_pair(items: &[Item], cells: &[usize], stats: &PageStats) -> bool {
    cells
        .iter()
        .map(|cell| &items[*cell])
        .filter(|item| {
            item.words() >= 6
                && item.x1 - item.x0 >= stats.page_width * 0.3
                && item.height() >= stats.body_height * 0.85
        })
        .count()
        >= 2
}

#[derive(Clone, Copy, Debug)]
struct Column {
    x0: f64,
    x1: f64,
}

/// Blocks of one region that is read row by row.
fn leaf_blocks(
    items: &[Item],
    ids: Vec<usize>,
    stats: &PageStats,
    rulings: &[[f64; 4]],
    source: TextSource,
) -> Vec<LayoutBlock> {
    let lines = lines_of(items, &ids);
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if let Some((end, block)) = table_at(items, &lines, index, stats, rulings, source) {
            blocks.push(block);
            index = end;
            continue;
        }
        if let Some((end, block)) = key_values_at(items, &lines, index, stats, source) {
            blocks.push(block);
            index = end;
            continue;
        }
        if let Some((end, block)) = labels_over_values_at(items, &lines, index, stats, source) {
            blocks.push(block);
            index = end;
            continue;
        }
        let (end, block) = text_at(items, &lines, index, stats, source);
        blocks.push(block);
        index = end;
    }
    blocks
}

/// The table that starts at `start`, if one does: at least two rows of two
/// or more cells that line up with each other, with the one-cell rows
/// between them that continue a cell or label a group.
fn table_at(
    items: &[Item],
    lines: &[Line],
    start: usize,
    stats: &PageStats,
    rulings: &[[f64; 4]],
    source: TextSource,
) -> Option<(usize, LayoutBlock)> {
    let first = &lines[start];
    if first.cells.len() < 2
        || is_key_value_line(items, first)
        || prose_pair(items, &first.cells, stats)
    {
        return None;
    }
    let tolerance = tolerance(stats.body_height);
    let mut columns = first
        .cells
        .iter()
        .map(|cell| Column {
            x0: items[*cell].x0,
            x1: items[*cell].x1,
        })
        .collect::<Vec<_>>();
    // Rows of cells; a continuation line's cells are folded into the row
    // above, each into the cell it sits under.
    let mut rows: Vec<Vec<Vec<usize>>> = vec![first.cells.iter().map(|cell| vec![*cell]).collect()];
    let mut row_lines: Vec<Vec<usize>> = vec![vec![start]];
    let mut multi_cell_rows = 1;
    // The least space between one row and the next, once there is a second
    // row to measure it by.
    let mut row_gap: Option<f64> = None;
    let mut end = start + 1;
    while end < lines.len() {
        let line = &lines[end];
        let above = &lines[end - 1];
        // Small labels over the values filled in under them may sit further
        // apart than the rows of a table do: the boxes of a form.
        let reach = if rows.len() == 1 && smaller_than(items, &rows[0], &[line.cells.clone()]) {
            2.5
        } else {
            1.6
        };
        if line.y0 - above.y1 > stats.body_height.max(stats.line_gap * 2.0) * reach {
            break;
        }
        let matched = line
            .cells
            .iter()
            .filter(|cell| {
                columns
                    .iter()
                    .any(|column| aligned(&items[**cell], column, tolerance))
            })
            .count();
        let gap = line.y0 - above.y1;
        let close = gap <= stats.line_gap + stats.body_height * 0.5;
        // Closer under the row above than the table's rows are to each
        // other: the row's cells wrap, the first one included.
        let wraps = row_gap.is_some_and(|row_gap| gap < row_gap * 0.5);
        if line.cells.len() >= 2 {
            // Wrapped cells: nothing in the first column, close under the row
            // above, every cell under one of that row's cells, and none of
            // them a figure - an amount under an amount is the next row.
            let first_column = columns
                .iter()
                .min_by(|left, right| left.x0.total_cmp(&right.x0))
                .copied();
            let previous = rows.last_mut().expect("a table has a row");
            let positions = line
                .cells
                .iter()
                .map(|cell| {
                    previous.iter().position(|above_cell| {
                        sits_under(items, above_cell, &items[*cell], tolerance)
                    })
                })
                .collect::<Option<Vec<_>>>();
            let starts_first_column = line.cells.iter().any(|cell| {
                first_column.is_some_and(|column| aligned(&items[*cell], &column, tolerance))
            });
            let figures = line.cells.iter().any(|cell| figure(&items[*cell].text));
            if close
                && (!starts_first_column || wraps)
                && !figures
                && let Some(positions) = positions
                && positions.windows(2).all(|pair| pair[0] < pair[1])
            {
                for (cell, position) in line.cells.iter().zip(positions) {
                    previous[position].push(*cell);
                }
                row_lines.last_mut().expect("a table has a row").push(end);
                end += 1;
                continue;
            }
            if is_key_value_line(items, line)
                || prose_pair(items, &line.cells, stats)
                || matched * 2 < line.cells.len()
                || matched < 2
                || new_header(items, lines, end)
            {
                break;
            }
            for cell in &line.cells {
                let item = &items[*cell];
                match columns
                    .iter_mut()
                    .find(|column| aligned(item, column, tolerance))
                {
                    Some(column) => {
                        column.x0 = column.x0.min(item.x0);
                        column.x1 = column.x1.max(item.x1);
                    }
                    None => columns.push(Column {
                        x0: item.x0,
                        x1: item.x1,
                    }),
                }
            }
            if gap >= 0.0 {
                row_gap = Some(row_gap.map_or(gap, |row_gap: f64| row_gap.min(gap)));
            }
            rows.push(line.cells.iter().map(|cell| vec![*cell]).collect());
            row_lines.push(vec![end]);
            multi_cell_rows += 1;
            end += 1;
            continue;
        }
        // One cell. Under a column other than the first and close to the
        // row above, it continues that row's cell; flush with the first
        // column and followed by more table rows, it is a row of its own.
        let item = &items[line.cells[0]];
        // A section heading - set larger than the text, or bold under rows
        // that are not, in capitals or running on past the first column -
        // ends the table, whatever follows it. A group's label inside a
        // table keeps to its column.
        let previous_bold = rows
            .last()
            .is_some_and(|row| row.iter().flatten().all(|id| items[*id].bold));
        let past_first_column = columns
            .iter()
            .min_by(|left, right| left.x0.total_cmp(&right.x0))
            .is_some_and(|column| item.x1 > column.x1 + tolerance);
        if item.height() > stats.body_height * 1.18
            || (item.bold && !previous_bold && (is_heading_line(&item.text) || past_first_column))
        {
            break;
        }
        let previous = rows.last_mut().expect("a table has a row");
        let under = previous
            .iter()
            .position(|cell| sits_under(items, cell, item, tolerance))
            .filter(|position| *position > 0 || previous.len() == 1 || wraps);
        // A lone figure under a column of figures is a total, not the rest
        // of the amount above it.
        if close
            && (wraps || !figure(&item.text))
            && let Some(position) = under
        {
            previous[position].push(line.cells[0]);
            row_lines.last_mut().expect("a table has a row").push(end);
            end += 1;
            continue;
        }
        let first_column = columns
            .iter()
            .map(|column| column.x0)
            .fold(f64::INFINITY, f64::min);
        let next_is_row = lines.get(end + 1).is_some_and(|next| {
            next.cells.len() >= 2
                && next
                    .cells
                    .iter()
                    .filter(|cell| {
                        columns
                            .iter()
                            .any(|column| aligned(&items[**cell], column, tolerance))
                    })
                    .count()
                    >= 2
        });
        // A line ending in a period or a colon is prose, or the caption of
        // what follows: it ends the table rather than labelling a group.
        if (item.x0 - first_column).abs() <= tolerance
            && next_is_row
            && !item.text.ends_with(['.', ':'])
        {
            rows.push(vec![vec![line.cells[0]]]);
            row_lines.push(vec![end]);
            end += 1;
            continue;
        }
        break;
    }
    if multi_cell_rows < 2 {
        return None;
    }
    let header = header_row(items, lines, &rows, &row_lines, rulings, stats);
    let rows = in_columns(items, rows, tolerance);
    let mut block_lines = Vec::new();
    let mut table_rows = Vec::new();
    for (row_index, (row, members)) in rows.iter().zip(&row_lines).enumerate() {
        let cells = row
            .iter()
            .map(|cell| {
                let text = cell
                    .iter()
                    .map(|id| items[*id].text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                LayoutCell {
                    id: String::new(),
                    text,
                    bbox: union(cell.iter().map(|id| items[*id].bbox())),
                    header: header && row_index == 0,
                }
            })
            .collect::<Vec<_>>();
        let text = format!(
            "| {} |",
            cells
                .iter()
                .map(|cell| escape_cell(&cell.text))
                .collect::<Vec<_>>()
                .join(" | ")
        );
        let confidence = mean_confidence(row.iter().flatten().map(|id| items[*id].confidence));
        block_lines.push(LayoutLine {
            text,
            bbox: union(members.iter().map(|line| lines[*line].bbox())),
            confidence,
        });
        table_rows.push(LayoutRow {
            id: String::new(),
            cells,
        });
    }
    let mut block = block_of(BlockKind::Table, block_lines, source);
    // A header over a single row of values is a set of labelled values -
    // the invoice number, date, and terms across the top of an invoice. Two
    // columns could as well be a letterhead beside a title, so it takes
    // three, or labels set smaller than what is filled in under them.
    let labels = header
        && rows.len() == 2
        && rows[0]
            .iter()
            .flatten()
            .all(|id| !numeric(&items[*id].text))
        && (rows[0].len() >= 3 || smaller_than(items, &rows[0], &rows[1]));
    if labels {
        block.fields = table_rows[0]
            .cells
            .iter()
            .zip(&table_rows[1].cells)
            .filter(|(key, value)| !key.text.is_empty() && !value.text.is_empty())
            .map(|(key, value)| KeyValue {
                id: String::new(),
                key: key.text.trim_end_matches(':').trim().to_owned(),
                value: value.text.clone(),
                key_bbox: key.bbox,
                value_bbox: value.bbox,
            })
            .collect();
    }
    block.table = Some(LayoutTable { rows: table_rows });
    Some((end, block))
}

/// Whether a run on the line under a table cell sits under it: flush with
/// its left edge, or - text set flush right or centred, like a wrapped
/// column heading - with its right edge, or inside its span.
fn sits_under(items: &[Item], cell: &[usize], item: &Item, tolerance: f64) -> bool {
    let (x0, x1) = cell
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(x0, x1), id| {
            (x0.min(items[*id].x0), x1.max(items[*id].x1))
        });
    let middle = (item.x0 + item.x1) / 2.0;
    (item.x0 - x0).abs() <= tolerance
        || (item.x1 - x1).abs() <= tolerance
        || (middle > x0 && middle < x1 && item.x0 >= x0 - tolerance && item.x1 <= x1 + tolerance)
}

/// The rows with every cell in its column: a row with a cell missing - a
/// transaction with a credit and no debit - gets an empty cell where the
/// missing one would be, so the figures stay under their headings. The
/// columns are what the cells of the rows with the most of them span; a row
/// whose cells
/// do not fall one to a column, left to right, is kept as it is, and so is
/// a row of one cell, which labels a group or spans the table.
fn in_columns(items: &[Item], rows: Vec<Vec<Vec<usize>>>, tolerance: f64) -> Vec<Vec<Vec<usize>>> {
    let span = |cell: &[usize]| {
        cell.iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(x0, x1), id| {
                (x0.min(items[*id].x0), x1.max(items[*id].x1))
            })
    };
    let widest = rows.iter().map(Vec::len).max().unwrap_or(0);
    if widest < 3 {
        return rows;
    }
    // Each column spans what its cells span in every row that has them all.
    let mut columns = vec![(f64::INFINITY, f64::NEG_INFINITY); widest];
    for row in rows.iter().filter(|row| row.len() == widest) {
        for (column, cell) in columns.iter_mut().zip(row) {
            let (x0, x1) = span(cell);
            *column = (column.0.min(x0), column.1.max(x1));
        }
    }
    rows.into_iter()
        .map(|row| {
            if row.len() < 2 || row.len() == columns.len() {
                return row;
            }
            let places = row
                .iter()
                .map(|cell| {
                    let (x0, x1) = span(cell);
                    columns
                        .iter()
                        .enumerate()
                        .map(|(index, (left, right))| {
                            let overlap = x1.min(*right) - x0.max(*left);
                            let distance = ((x0 + x1) / 2.0 - (left + right) / 2.0).abs();
                            (index, overlap, distance)
                        })
                        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.2.total_cmp(&a.2)))
                        .map(|(index, _, _)| index)
                        .unwrap_or(0)
                })
                .collect::<Vec<_>>();
            // Each cell has to sit in its column - flush with its left edge
            // or its right - not merely overlap it.
            let sits = row.iter().zip(&places).all(|(cell, place)| {
                let (x0, x1) = span(cell);
                let (left, right) = columns[*place];
                (x0 - left).abs() <= tolerance || (x1 - right).abs() <= tolerance
            });
            if !sits || !places.windows(2).all(|pair| pair[0] < pair[1]) {
                return row;
            }
            let mut padded = vec![Vec::new(); columns.len()];
            for (cell, place) in row.into_iter().zip(places) {
                padded[place] = cell;
            }
            padded
        })
        .collect()
}

/// Whether line `index` heads a new table: set in bold, with no figures in
/// it, under a line that is not bold, and over one that is not bold either.
/// A total set in bold carries its figure, so it stays in its table.
fn new_header(items: &[Item], lines: &[Line], index: usize) -> bool {
    let line = &lines[index];
    let bold = |line: &Line| line.all_bold(items);
    index > 0
        && bold(line)
        && !bold(&lines[index - 1])
        && line.cells.iter().all(|cell| !numeric(&items[*cell].text))
        && lines
            .get(index + 1)
            .is_some_and(|next| next.cells.len() >= 2 && !bold(next))
}

/// Whether a table's first row is its header: set in bold, ruled off from
/// the rows under it, or words over columns that hold numbers below.
fn header_row(
    items: &[Item],
    lines: &[Line],
    rows: &[Vec<Vec<usize>>],
    row_lines: &[Vec<usize>],
    rulings: &[[f64; 4]],
    stats: &PageStats,
) -> bool {
    if rows.len() < 2 {
        return false;
    }
    let first = &rows[0];
    if first.iter().flatten().all(|id| items[*id].bold) {
        let rest_bold = rows[1..]
            .iter()
            .all(|row| row.iter().flatten().all(|id| items[*id].bold));
        if !rest_bold {
            return true;
        }
    }
    let first_line = &lines[*row_lines[0].last().expect("a row has a line")];
    let second_line = &lines[row_lines[1][0]];
    let width = first_line.x1.max(second_line.x1) - first_line.x0.min(second_line.x0);
    let ruled = rulings.iter().any(|ruling| {
        let horizontal = ruling[3] - ruling[1] <= 30.0;
        let between = ruling[1] >= first_line.y1 - stats.body_height * 0.3
            && ruling[3] <= second_line.y0 + stats.body_height * 0.3;
        horizontal && between && ruling[2] - ruling[0] >= width * 0.5
    });
    if ruled || smaller_than(items, first, &rows[1]) {
        return true;
    }
    // Words over columns of figures.
    first.iter().flatten().all(|id| !numeric(&items[*id].text))
        && rows[1..]
            .iter()
            .any(|row| row.iter().flatten().any(|id| numeric(&items[*id].text)))
}

/// Mostly digits: a figure, a date, an amount.
fn numeric(text: &str) -> bool {
    let digits = text.chars().filter(char::is_ascii_digit).count();
    digits > 0
        && digits * 3
            >= text
                .chars()
                .filter(|character| !character.is_whitespace())
                .count()
}

/// Whether every cell of one row is set noticeably smaller than every cell
/// of another: a form's labels over the values filled in under them.
fn smaller_than(items: &[Item], labels: &[Vec<usize>], values: &[Vec<usize>]) -> bool {
    let largest_label = labels
        .iter()
        .flatten()
        .map(|id| items[*id].height())
        .fold(0.0, f64::max);
    let smallest_value = values
        .iter()
        .flatten()
        .map(|id| items[*id].height())
        .fold(f64::INFINITY, f64::min);
    largest_label > 0.0 && largest_label <= smallest_value * 0.85
}

/// A line of labelled values: a label cell ending in a colon, or a single
/// `Label: value`.
fn is_key_value_line(items: &[Item], line: &Line) -> bool {
    let first = &items[line.cells[0]];
    (first.text.ends_with(':') && is_label(&first.text))
        || (line.cells.len() == 1 && split_key_value(&first.text).is_some())
}

/// The labelled values that start at `start`, if any do: label cells with
/// the value beside them, `Label: value` lines, and a label on a line of
/// its own with its value on the lines under it.
fn key_values_at(
    items: &[Item],
    lines: &[Line],
    start: usize,
    stats: &PageStats,
    source: TextSource,
) -> Option<(usize, LayoutBlock)> {
    // A line set larger than the text around it is a heading, colon or
    // not: `Segment Review: Aerospace Components`.
    if !is_key_value_line(items, &lines[start]) || lines[start].height() > stats.body_height * 1.18
    {
        return None;
    }
    let left_edge = lines
        .iter()
        .map(|line| line.x0)
        .fold(f64::INFINITY, f64::min);
    let right_edge = lines
        .iter()
        .map(|line| line.x1)
        .fold(f64::NEG_INFINITY, f64::max);
    struct Pair {
        key: String,
        key_bbox: [u32; 4],
        value: Vec<String>,
        value_boxes: Vec<[u32; 4]>,
        value_x0: Option<f64>,
        value_x1: Option<f64>,
        /// The value is a cell of its own, set apart from its label, so a
        /// line under it continues it if it ends where it ends.
        set_apart: bool,
        label_x0: f64,
        confidence: Vec<Option<u8>>,
        lines: Vec<usize>,
    }
    let mut pairs: Vec<Pair> = Vec::new();
    let mut end = start;
    while end < lines.len() {
        let line = &lines[end];
        if end > start {
            let above = &lines[end - 1];
            if line.y0 - above.y1 > stats.line_gap + stats.body_height * 0.6 {
                break;
            }
        }
        if is_key_value_line(items, line) {
            let mut cells = line.cells.iter().peekable();
            while let Some(cell) = cells.next() {
                let item = &items[*cell];
                if item.text.ends_with(':') && is_label(&item.text) {
                    let mut pair = Pair {
                        key: item.text.trim_end_matches(':').trim().to_owned(),
                        key_bbox: item.bbox(),
                        value: Vec::new(),
                        value_boxes: Vec::new(),
                        value_x0: None,
                        value_x1: None,
                        set_apart: true,
                        label_x0: item.x0,
                        confidence: vec![item.confidence],
                        lines: vec![end],
                    };
                    if let Some(next) = cells.next_if(|next| {
                        let text = &items[**next].text;
                        !(text.ends_with(':') && is_label(text))
                    }) {
                        let value = &items[*next];
                        pair.value.push(value.text.clone());
                        pair.value_boxes.push(value.bbox());
                        pair.value_x0 = Some(value.x0);
                        pair.value_x1 = Some(value.x1);
                        pair.confidence.push(value.confidence);
                    }
                    pairs.push(pair);
                } else if let Some((key, value)) = split_key_value(&item.text) {
                    pairs.push(Pair {
                        key: key.to_owned(),
                        key_bbox: item.bbox(),
                        value: vec![value.to_owned()],
                        value_boxes: vec![item.bbox()],
                        value_x0: Some(item.x0),
                        value_x1: Some(item.x1),
                        set_apart: false,
                        label_x0: item.x0,
                        confidence: vec![item.confidence],
                        lines: vec![end],
                    });
                } else if let Some(pair) = pairs.last_mut() {
                    pair.value.push(item.text.clone());
                    pair.value_boxes.push(item.bbox());
                    pair.confidence.push(item.confidence);
                    if !pair.lines.contains(&end) {
                        pair.lines.push(end);
                    }
                }
            }
            end += 1;
            continue;
        }
        // A line under a label continues its value when it sits where the
        // value does - or anywhere right of the label, if the label has no
        // value yet.
        let Some(pair) = pairs.last_mut() else {
            break;
        };
        let tolerance = tolerance(stats.body_height);
        let first = &items[line.cells[0]];
        // Only a single line of text continues a value: a row of cells is
        // a table starting. A value written after its label on one line
        // goes on under it only if that line ran to the end of the text's
        // width and wrapped: a short `Location: Briarport` is followed by
        // the next item, not more of it.
        let above = &lines[end - 1];
        let wrapped = above.x1 >= right_edge - (right_edge - left_edge) * 0.2;
        let continues = line.cells.len() == 1
            && match (pair.value_x0, pair.value_x1) {
                (Some(x0), Some(x1)) if pair.set_apart => {
                    (first.x0 - x0).abs() <= tolerance || (line.x1 - x1).abs() <= tolerance
                }
                (Some(x0), Some(_)) => wrapped && (first.x0 - x0).abs() <= tolerance,
                _ => first.x0 > pair.label_x0 + stats.body_height * 0.5,
            };
        if !continues || (is_heading_line(&line.text(items)) && pair.value_x0.is_some()) {
            break;
        }
        pair.value.push(line.text(items));
        pair.value_boxes.push(line.bbox());
        pair.value_x0.get_or_insert(first.x0);
        pair.value_x1.get_or_insert(line.x1);
        pair.confidence.push(line.confidence(items));
        pair.lines.push(end);
        end += 1;
    }
    if pairs.is_empty() {
        return None;
    }
    let mut block_lines = Vec::new();
    let mut fields = Vec::new();
    for pair in pairs {
        let value = pair.value.join(" ");
        let text = if value.is_empty() {
            format!("{}:", pair.key)
        } else {
            format!("{}: {value}", pair.key)
        };
        let value_bbox = union(pair.value_boxes.iter().copied());
        block_lines.push(LayoutLine {
            text,
            bbox: union(std::iter::once(pair.key_bbox).chain(pair.value_boxes.iter().copied())),
            confidence: mean_confidence(pair.confidence.iter().copied()),
        });
        if !value.is_empty() {
            fields.push(KeyValue {
                id: String::new(),
                key: pair.key,
                value,
                key_bbox: Some(pair.key_bbox),
                value_bbox,
            });
        }
    }
    // A label with nothing after it - `THE CONTRACT IS CHANGED AS
    // FOLLOWS:` over a table - introduces what follows: it is text.
    let kind = if fields.is_empty() {
        BlockKind::Paragraph
    } else {
        BlockKind::KeyValue
    };
    let mut block = block_of(kind, block_lines, source);
    block.fields = fields;
    Some((end.max(start + 1), block))
}

/// Whether a line's text, set over another's, can be the label of the
/// value under it, whatever their sizes say.
///
/// Never a line that holds a label and its value already (`Name: Saoirse
/// Whitcombe`), nor over one that opens with a label (`By:`, `Title:`,
/// `INSURER A:`) - each is a field of its own; never a part of the document
/// (`ARTICLE 4 - OPERATING EXPENSES`), nor over the first line of a
/// numbered clause (`4.1 Beginning with`). On a page OCR read, whose line
/// heights are its letters' and not its type size, the line has to read as
/// a label as well ([`reads_as_label`]): a name over an address, a firm
/// over its signature line, or a title over the text it heads is not one.
fn label_over_value(label: &str, value: &str, source: TextSource) -> bool {
    let label = label.trim();
    let value = value.trim();
    let key = label.trim_end_matches(':').trim_end();
    if key.contains(':')
        || split_key_value(value).is_some()
        || (value.ends_with(':') && is_label(value))
        || is_part_heading(key)
        || opens_a_clause(value)
    {
        return false;
    }
    match source {
        TextSource::Native => true,
        TextSource::Ocr => (label.ends_with(':') && is_label(label)) || reads_as_label(key),
    }
}

/// A form's fields set as a small label over the value filled in under it,
/// one after another: `1. Legal business name` over `Emberglow Coatings
/// Ltd.`. Each label is a line of one cell, set smaller than the line
/// right under it, which starts where the label starts, and reads as a
/// label of it ([`label_over_value`]).
fn labels_over_values_at(
    items: &[Item],
    lines: &[Line],
    start: usize,
    stats: &PageStats,
    source: TextSource,
) -> Option<(usize, LayoutBlock)> {
    let tolerance = tolerance(stats.body_height);
    let pair_at = |index: usize| -> bool {
        let (Some(label), Some(value)) = (lines.get(index), lines.get(index + 1)) else {
            return false;
        };
        label.cells.len() == 1
            && value.cells.len() == 1
            && items[label.cells[0]].words() <= 14
            && (items[value.cells[0]].x0 - items[label.cells[0]].x0).abs() <= tolerance
            && value.y0 - label.y1 <= stats.line_gap + stats.body_height * 0.6
            && smaller_than(items, &[label.cells.clone()], &[value.cells.clone()])
            && !is_heading_line(&items[value.cells[0]].text)
            && label_over_value(
                &items[label.cells[0]].text,
                &items[value.cells[0]].text,
                source,
            )
    };
    if !pair_at(start) {
        return None;
    }
    let mut end = start;
    let mut block_lines = Vec::new();
    let mut fields = Vec::new();
    while pair_at(end) {
        let label = &items[lines[end].cells[0]];
        let value = &items[lines[end + 1].cells[0]];
        let key = label.text.trim_end_matches(':').trim().to_owned();
        block_lines.push(LayoutLine {
            text: format!("{key}: {}", value.text),
            bbox: union([label.bbox(), value.bbox()]),
            confidence: mean_confidence([label.confidence, value.confidence]),
        });
        fields.push(KeyValue {
            id: String::new(),
            key,
            value: value.text.clone(),
            key_bbox: Some(label.bbox()),
            value_bbox: Some(value.bbox()),
        });
        end += 2;
    }
    let mut block = block_of(BlockKind::KeyValue, block_lines, source);
    block.fields = fields;
    Some((end, block))
}

/// Text from `start`: a heading, a list item and the lines that hang under
/// it, or a paragraph that runs until the spacing, the type, or the kind of
/// line changes.
fn text_at(
    items: &[Item],
    lines: &[Line],
    start: usize,
    stats: &PageStats,
    source: TextSource,
) -> (usize, LayoutBlock) {
    let first = &lines[start];
    let first_text = first.text(items);
    if let Some(level) = heading_level(items, first, &first_text, stats) {
        // A title set over two lines is one heading.
        let mut end = start + 1;
        while end < lines.len() {
            let line = &lines[end];
            let text = line.text(items);
            let same_style = (line.height() - first.height()).abs() <= first.height() * 0.1
                && line.all_bold(items) == first.all_bold(items)
                && heading_level(items, line, &text, stats) == Some(level);
            // A bold row of cells under a bold heading is the header of
            // the table the heading introduces.
            let starts_table = line.cells.len() >= 2
                && table_at(items, lines, end, stats, &[], TextSource::Native).is_some();
            if !same_style
                || starts_table
                || line.y0 - lines[end - 1].y1 > stats.line_gap + stats.body_height * 0.5
            {
                break;
            }
            end += 1;
        }
        let mut block = block_of(
            BlockKind::Heading,
            lines[start..end]
                .iter()
                .map(|line| line.layout_line(items))
                .collect(),
            source,
        );
        block.level = Some(level);
        return (end, block);
    }
    let list = super::text::is_list_item_text(&first_text);
    let mut end = start + 1;
    while end < lines.len() {
        let line = &lines[end];
        let above = &lines[end - 1];
        let text = line.text(items);
        let gap = line.y0 - above.y1;
        let breaks = gap > stats.line_gap + stats.body_height * 0.4
            || gap < -stats.body_height * 0.5
            || (line.height() - above.height()).abs() > stats.body_height * 0.2
            || heading_level(items, line, &text, stats).is_some()
            || super::text::is_list_item_text(&text)
            || is_key_value_line(items, line)
            || (line.cells.len() >= 2 && table_at(items, lines, end, stats, &[], source).is_some())
            || (list && line.x0 < first.x0 - tolerance(stats.body_height))
            || end - start >= 40;
        if breaks {
            break;
        }
        end += 1;
    }
    let kind = if list {
        BlockKind::ListItem
    } else {
        BlockKind::Paragraph
    };
    let block = block_of(
        kind,
        lines[start..end]
            .iter()
            .map(|line| line.layout_line(items))
            .collect(),
        source,
    );
    (end, block)
}

/// A line's heading level, if it is a heading: set larger than the body
/// text (1 for the largest), or at body size in bold or capitals and short.
fn heading_level(items: &[Item], line: &Line, text: &str, stats: &PageStats) -> Option<u8> {
    let words = text.split_whitespace().count();
    if words == 0 || text.chars().count() > 120 || text.ends_with(':') {
        return None;
    }
    // The size the line's text is mostly set in: a tick mark or a symbol
    // set large does not make a line of options a heading.
    let dominant = line
        .cells
        .iter()
        .map(|cell| &items[*cell])
        .max_by_key(|item| item.text.chars().count())
        .map_or(line.height(), Item::height);
    let ratio = dominant / stats.body_height;
    let letters = text
        .chars()
        .filter(|character| character.is_alphabetic())
        .count();
    if letters < 2 {
        return None;
    }
    if ratio >= 1.18 && words <= 20 {
        return Some(if ratio >= 1.6 {
            1
        } else if ratio >= 1.3 {
            2
        } else {
            3
        });
    }
    let sentence = text.ends_with('.') && words > 6;
    if !sentence && words <= 14 && (line.all_bold(items) || is_heading_line(text)) {
        return Some(4);
    }
    None
}

/// A block of lines. Lines that all carry an OCR confidence came from OCR,
/// whatever the page's own text is.
fn block_of(kind: BlockKind, lines: Vec<LayoutLine>, source: TextSource) -> LayoutBlock {
    let source = if !lines.is_empty() && lines.iter().all(|line| line.confidence.is_some()) {
        TextSource::Ocr
    } else {
        source
    };
    let text = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let mut block = LayoutBlock::new(kind, text, source);
    block.bbox = union(lines.iter().filter_map(|line| line.bbox));
    block.confidence = mean_confidence(lines.iter().map(|line| line.confidence));
    block.lines = lines;
    block
}

/// Small lines at the very top and bottom of a page are its running header
/// and footer: a page number, a document title repeated on every page.
fn mark_running_lines(blocks: &mut [LayoutBlock], stats: &PageStats) {
    let margin = stats.page_height * 0.06;
    for block in blocks.iter_mut() {
        let Some(bbox) = block.bbox else {
            continue;
        };
        if block.kind == BlockKind::Table
            || block.text.split_whitespace().count() > 20
            || block.lines.len() > 2
        {
            continue;
        }
        let small = block.lines.iter().all(|line| {
            line.bbox
                .is_some_and(|bbox| f64::from(bbox[3] - bbox[1]) <= stats.body_height * 1.2)
        });
        if !small {
            continue;
        }
        if f64::from(bbox[3]) <= margin {
            block.kind = BlockKind::PageHeader;
            block.level = None;
        } else if f64::from(bbox[1]) >= stats.page_height - margin {
            block.kind = BlockKind::PageFooter;
            block.level = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run at `x`, `y` (points, top of the line) of `size` points.
    fn run(x: f64, y: f64, text: &str, size: f64) -> TextRun {
        let width = text.chars().count() as f64 * size * 0.5;
        TextRun {
            text: text.to_owned(),
            bbox: [
                (x * 10.0) as u32,
                (y * 10.0) as u32,
                ((x + width) * 10.0) as u32,
                ((y + size) * 10.0) as u32,
            ],
            bold: false,
            confidence: None,
        }
    }

    fn bold(mut run: TextRun) -> TextRun {
        run.bold = true;
        run
    }

    fn analyze(runs: &[TextRun]) -> Vec<LayoutBlock> {
        let blocks = analyze_runs(runs, 6120, 7920, &[], TextSource::Native);
        assert_words_kept(runs, &blocks);
        blocks
    }

    /// Every word of every run is in the blocks, once: the analysis orders,
    /// groups, and labels text - a label gains a colon, a table its pipes -
    /// but never loses or repeats any.
    fn assert_words_kept(runs: &[TextRun], blocks: &[LayoutBlock]) {
        fn words<'a>(texts: impl Iterator<Item = &'a str>) -> Vec<String> {
            let mut words = texts
                .flat_map(str::split_whitespace)
                .filter(|word| *word != "|")
                .map(|word| word.trim_end_matches(':').to_owned())
                .filter(|word| !word.is_empty())
                .collect::<Vec<_>>();
            words.sort();
            words
        }
        assert_eq!(
            words(runs.iter().map(|run| run.text.as_str())),
            words(blocks.iter().map(|block| block.text.as_str())),
            "{blocks:#?}"
        );
    }

    fn texts(blocks: &[LayoutBlock]) -> Vec<&str> {
        blocks.iter().map(|block| block.text.as_str()).collect()
    }

    #[test]
    fn two_columns_written_row_by_row_read_column_by_column() {
        let left = [
            "The tenant leases the premises for",
            "the term and pays the base rent in",
            "monthly installments when they fall",
            "due under this lease agreement now.",
        ];
        let right = [
            "The landlord keeps the common areas",
            "in good repair and insures building",
            "at full replacement cost each year",
            "as the lease requires of the owner.",
        ];
        let mut runs = Vec::new();
        for (row, (l, r)) in left.iter().zip(right).enumerate() {
            let y = 100.0 + row as f64 * 12.0;
            runs.push(run(54.0, y, l, 9.0));
            runs.push(run(320.0, y, r, 9.0));
        }

        let blocks = analyze(&runs);

        assert_eq!(texts(&blocks), [left.join("\n"), right.join("\n")]);
        assert!(
            blocks
                .iter()
                .all(|block| block.kind == BlockKind::Paragraph)
        );
    }

    #[test]
    fn a_table_keeps_its_rows_and_its_header() {
        let mut runs = vec![
            bold(run(54.0, 100.0, "Item", 9.0)),
            bold(run(300.0, 100.0, "Qty", 9.0)),
            bold(run(450.0, 100.0, "Amount", 9.0)),
        ];
        for (row, (item, quantity, amount)) in [
            ("Bicycle tune-up", "2", "$180.00"),
            ("Brake cables", "4", "$36.00"),
            ("Labour", "3", "$255.00"),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 114.0 + row as f64 * 14.0;
            runs.push(run(54.0, y, item, 9.0));
            runs.push(run(300.0, y, quantity, 9.0));
            runs.push(run(450.0, y, amount, 9.0));
        }

        let blocks = analyze(&runs);

        assert_eq!(blocks.len(), 1, "{blocks:#?}");
        assert_eq!(blocks[0].kind, BlockKind::Table);
        assert_eq!(
            blocks[0].text,
            "| Item | Qty | Amount |\n| Bicycle tune-up | 2 | $180.00 |\n\
             | Brake cables | 4 | $36.00 |\n| Labour | 3 | $255.00 |"
        );
        let table = blocks[0].table.as_ref().unwrap();
        assert!(table.rows[0].cells.iter().all(|cell| cell.header));
        assert!(!table.rows[1].cells[0].header);
    }

    /// Boxes of a certificate - a captioned producer on the left, labelled
    /// insurers on the right - over a schedule that runs across the gap
    /// between them: the schedule is one table, its rows whole.
    #[test]
    fn a_table_across_the_gutter_under_labelled_boxes_stays_whole() {
        let mut runs = vec![
            run(54.0, 80.0, "PRODUCER", 8.0),
            run(54.0, 92.0, "Hartsfield Insurance Brokers", 9.0),
            run(54.0, 104.0, "300 Bell Tower Road", 9.0),
            run(54.0, 116.0, "Calloway, SC 29630", 9.0),
            // The boxes are set apart, their lines not level with each other.
            run(320.0, 85.0, "INSURER A: Bramblecote Casualty Company", 9.0),
            run(320.0, 96.0, "INSURER B: Wexley Indemnity Company", 9.0),
            run(320.0, 107.0, "INSURER C: Fenmoor Specialty Co.", 9.0),
            run(320.0, 120.0, "CERTIFICATE NUMBER: HQ-26-06-4415", 9.0),
        ];
        let rows = [
            ["LTR", "TYPE", "POLICY NUMBER", "EFF", "EXP", "LIMIT"],
            [
                "A",
                "General",
                "BCC-GL-4471902",
                "04/01/2026",
                "04/01/2027",
                "$1,000,000",
            ],
            [
                "B",
                "Auto",
                "WIC-CA-208815",
                "01/15/2026",
                "01/15/2027",
                "$1,000,000",
            ],
            [
                "C",
                "Umbrella",
                "FSI-UMB-77310",
                "04/01/2026",
                "04/01/2027",
                "$5,000,000",
            ],
        ];
        for (index, cells) in rows.iter().enumerate() {
            let y = 128.0 + index as f64 * 12.0;
            for (cell, x) in cells.iter().zip([54.0, 80.0, 200.0, 320.0, 390.0, 460.0]) {
                runs.push(run(x, y, cell, 9.0));
            }
        }

        let blocks = analyze(&runs);

        let tables = blocks
            .iter()
            .filter(|block| block.kind == BlockKind::Table)
            .collect::<Vec<_>>();
        assert_eq!(tables.len(), 1, "{:#?}", texts(&blocks));
        assert!(
            tables[0].text.contains(
                "| A | General | BCC-GL-4471902 | 04/01/2026 | 04/01/2027 | $1,000,000 |"
            ),
            "{:#?}",
            texts(&blocks)
        );
    }

    #[test]
    fn a_wrapped_cell_continues_its_row() {
        let runs = vec![
            bold(run(54.0, 100.0, "Term", 9.0)),
            bold(run(180.0, 100.0, "Provision", 9.0)),
            run(54.0, 114.0, "Premises", 9.0),
            run(180.0, 114.0, "Suite 108, about 2,140 square feet,", 9.0),
            run(180.0, 125.0, "Oakhaven Commons", 9.0),
            run(54.0, 139.0, "Expiration Date", 9.0),
            run(180.0, 139.0, "June 30, 2031", 9.0),
        ];

        let blocks = analyze(&runs);

        assert_eq!(blocks.len(), 1, "{blocks:#?}");
        assert_eq!(
            blocks[0].text,
            "| Term | Provision |\n\
             | Premises | Suite 108, about 2,140 square feet, Oakhaven Commons |\n\
             | Expiration Date | June 30, 2031 |"
        );
    }

    #[test]
    fn labelled_values_in_two_columns_pair_with_their_labels() {
        let rows = [
            ("POLICY NUMBER:", "CPP-4471-208815", "AGENT / PRODUCER:", ""),
            (
                "POLICY PERIOD:",
                "From 07/01/2026 To 07/01/2027",
                "",
                "Kingsfold Insurance Agency, Inc.",
            ),
            (
                "",
                "12:01 A.M. standard time at the address",
                "",
                "Agent code 00-4417",
            ),
            ("NAMED INSURED:", "", "DATE ISSUED:", "06/18/2026"),
            ("", "Wexcombe Bicycle Cooperative", "", ""),
            ("PRIOR POLICY:", "CPP-4471-197322", "", ""),
        ];
        let mut runs = Vec::new();
        for (row, (left_label, left_value, right_label, right_value)) in rows.iter().enumerate() {
            let y = 132.0 + row as f64 * 15.0;
            if !left_label.is_empty() {
                runs.push(bold(run(54.0, y, left_label, 8.5)));
            }
            if !left_value.is_empty() {
                runs.push(run(150.0, y, left_value, 9.0));
            }
            if !right_label.is_empty() {
                runs.push(bold(run(340.0, y, right_label, 8.5)));
            }
            if !right_value.is_empty() {
                let width = right_value.chars().count() as f64 * 4.5;
                runs.push(run(558.0 - width, y, right_value, 9.0));
            }
        }

        let blocks = analyze(&runs);

        assert_eq!(
            texts(&blocks),
            [
                "POLICY NUMBER: CPP-4471-208815\n\
                 POLICY PERIOD: From 07/01/2026 To 07/01/2027 12:01 A.M. standard time at the address\n\
                 NAMED INSURED: Wexcombe Bicycle Cooperative\n\
                 PRIOR POLICY: CPP-4471-197322",
                "AGENT / PRODUCER: Kingsfold Insurance Agency, Inc. Agent code 00-4417\n\
                 DATE ISSUED: 06/18/2026",
            ],
            "{blocks:#?}"
        );
        let fields = &blocks[0].fields;
        assert_eq!(fields[2].key, "NAMED INSURED");
        assert_eq!(fields[2].value, "Wexcombe Bicycle Cooperative");
        assert_eq!(blocks[1].fields[1].value, "06/18/2026");
    }

    #[test]
    fn a_title_over_two_columns_is_read_first_and_a_footer_last() {
        let mut runs = vec![bold(run(200.0, 60.0, "RETAIL LEASE AGREEMENT", 16.0))];
        for row in 0..6 {
            let y = 100.0 + row as f64 * 12.0;
            runs.push(run(
                54.0,
                y,
                &format!("left column line {row} of the lease"),
                9.0,
            ));
            runs.push(run(
                320.0,
                y,
                &format!("right column line {row} of the lease"),
                9.0,
            ));
        }
        runs.push(run(54.0, 760.0, "Page 1 of 3", 7.0));

        let blocks = analyze(&runs);

        assert_eq!(blocks[0].kind, BlockKind::Heading);
        assert_eq!(blocks[0].level, Some(1));
        assert!(blocks[1].text.starts_with("left column line 0"));
        assert!(blocks[1].text.ends_with("line 5 of the lease"));
        assert!(blocks[2].text.starts_with("right column line 0"));
        assert_eq!(blocks.last().unwrap().kind, BlockKind::PageFooter);
    }

    #[test]
    fn a_header_over_one_row_of_values_is_also_labelled_values() {
        let runs = vec![
            run(54.0, 100.0, "Invoice Date", 8.0),
            run(200.0, 100.0, "Invoice No.", 8.0),
            run(350.0, 100.0, "Terms", 8.0),
            run(54.0, 112.0, "March 4, 2026", 9.0),
            run(200.0, 112.0, "INV-20417", 9.0),
            run(350.0, 112.0, "Net 30", 9.0),
        ];

        let blocks = analyze(&runs);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].kind, BlockKind::Table);
        let fields = &blocks[0].fields;
        assert_eq!(fields.len(), 3);
        assert_eq!(
            (fields[0].key.as_str(), fields[0].value.as_str()),
            ("Invoice Date", "March 4, 2026")
        );
    }

    /// A test plan's rows: an id, a scenario, and a pass condition that
    /// wraps onto a second line now and then. The band between the
    /// scenario and the condition runs the table's height, and both sides
    /// are prose, but the left side is a grid of ids and scenarios whose
    /// rows start level with the conditions: one table, not two columns.
    #[test]
    fn a_table_whose_last_column_wraps_is_not_cut_into_columns() {
        let rows = [
            (
                "T4",
                "Pressure transient below 35 psi at two loggers",
                "Transient alert with map location",
                "",
            ),
            (
                "T5",
                "Operator confirms an alert and opens a work order",
                "Work order created with asset ID and",
                "coordinates",
            ),
            (
                "T6",
                "Stale meter excluded from the leak scoring run",
                "Meter listed on data quality report",
                "",
            ),
            (
                "T7",
                "Reverse-flow alarm routed to cross-connection",
                "Alert assigned to the backflow program",
                "queue",
            ),
            (
                "T8",
                "Role-based access for the customer service team",
                "Edit control hidden and API refused",
                "",
            ),
        ];
        let mut runs = vec![
            bold(run(75.0, 69.0, "Test", 9.0)),
            bold(run(112.0, 69.0, "Scenario", 9.0)),
            bold(run(374.0, 69.0, "Pass condition", 9.0)),
        ];
        let mut y = 86.0;
        for (id, scenario, condition, wrapped) in rows {
            runs.push(run(75.0, y, id, 9.0));
            runs.push(run(112.0, y, scenario, 9.0));
            runs.push(run(374.0, y, condition, 9.0));
            if !wrapped.is_empty() {
                runs.push(run(374.0, y + 10.6, wrapped, 9.0));
                y += 10.6;
            }
            y += 16.6;
        }

        let blocks = analyze_runs(&runs, 6120, 7920, &[], TextSource::Native);
        assert_words_kept(&runs, &blocks);

        assert_eq!(blocks.len(), 1, "{:#?}", texts(&blocks));
        let table = blocks[0].table.as_ref().unwrap();
        assert_eq!(table.rows.len(), 6);
        assert_eq!(
            table.rows[2].cells[2].text,
            "Work order created with asset ID and coordinates"
        );
        assert_eq!(table.rows[4].cells[0].text, "T7");
        assert_eq!(
            table.rows[4].cells[2].text,
            "Alert assigned to the backflow program queue"
        );
    }

    /// A schedule whose first column wraps, whose lease date runs into the
    /// next column, and whose heading wraps flush right.
    #[test]
    fn wrapped_first_cells_overflowing_dates_and_wrapped_headings_stay_in_their_rows() {
        let mut runs = vec![
            bold(run(75.0, 92.7, "Premises", 10.0)),
            bold(run(225.0, 92.7, "Landlord", 10.0)),
            bold(run(346.0, 92.7, "Lease dated", 10.0)),
            bold(run(412.0, 92.7, "Expires", 10.0)),
            bold(run(500.0, 92.7, "Cases on", 10.0)),
            bold(run(520.0, 102.8, "hand", 10.0)),
        ];
        let rows = [
            (
                "1410 Orchard Row",
                "Orchard Row LLC",
                "May 1, 2016",
                "April 30, 2031",
                "48,200",
                "(plant and offices)",
                119.5,
            ),
            (
                "22 Pennant Street",
                "Pennant Park LLC",
                "September 15, 2021",
                "September 14, 2026",
                "31,500",
                "(warehouse)",
                146.8,
            ),
        ];
        for (premises, landlord, dated, expires, size, wrapped, y) in rows {
            runs.push(run(75.0, y, premises, 10.0));
            runs.push(run(225.0, y, landlord, 10.0));
            runs.push(run(346.0, y, dated, 10.0));
            runs.push(run(412.0, y, expires, 10.0));
            runs.push(run(511.0, y, size, 10.0));
            runs.push(run(75.0, y + 10.6, wrapped, 10.0));
        }

        let blocks = analyze_runs(&runs, 6120, 7920, &[], TextSource::Native);
        assert_words_kept(&runs, &blocks);

        assert_eq!(blocks.len(), 1, "{:#?}", texts(&blocks));
        assert_eq!(
            blocks[0].text,
            "| Premises | Landlord | Lease dated | Expires | Cases on hand |\n\
             | 1410 Orchard Row (plant and offices) | Orchard Row LLC | May 1, 2016 | April 30, 2031 | 48,200 |\n\
             | 22 Pennant Street (warehouse) | Pennant Park LLC | September 15, 2021 | September 14, 2026 | 31,500 |"
        );
    }

    /// Line items, the totals set flush right under them, and the section
    /// heading after them.
    #[test]
    fn totals_are_rows_of_their_own_and_a_section_heading_ends_the_table() {
        let mut runs = vec![
            bold(run(54.0, 100.0, "Item", 9.0)),
            bold(run(300.0, 100.0, "Qty", 9.0)),
            bold(run(380.0, 100.0, "Unit Price", 9.0)),
            bold(run(500.0, 100.0, "Amount", 9.0)),
        ];
        for (row, (item, quantity, price, amount)) in [
            ("Pastry display case", "1", "2,385.00", "2,385.00"),
            ("Bread shelving", "3", "264.50", "793.50"),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 116.0 + row as f64 * 16.0;
            runs.push(run(54.0, y, item, 9.0));
            runs.push(run(300.0, y, quantity, 9.0));
            runs.push(run(380.0, y, price, 9.0));
            runs.push(run(500.0, y, amount, 9.0));
        }
        // Labels flush right with the Unit Price column, which ends at 425.
        for (row, (label, amount)) in [
            ("Subtotal", "3,178.50"),
            ("Sales tax 8.7% (fixtures)", "276.53"),
        ]
        .into_iter()
        .enumerate()
        {
            let y = 148.0 + row as f64 * 14.0;
            let width = label.chars().count() as f64 * 4.5;
            runs.push(run(425.0 - width, y, label, 9.0));
            runs.push(run(500.0, y, amount, 9.0));
        }
        runs.push(bold(run(
            54.0,
            190.0,
            "SECTION D - Certifications (attach)",
            9.0,
        )));
        runs.push(run(
            54.0,
            206.0,
            "Attach every certificate the business holds.",
            9.0,
        ));

        let blocks = analyze(&runs);

        assert_eq!(
            blocks[0].text,
            "| Item | Qty | Unit Price | Amount |\n\
             | Pastry display case | 1 | 2,385.00 | 2,385.00 |\n\
             | Bread shelving | 3 | 264.50 | 793.50 |\n\
             |  |  | Subtotal | 3,178.50 |\n\
             |  |  | Sales tax 8.7% (fixtures) | 276.53 |",
            "{:#?}",
            texts(&blocks)
        );
        assert_eq!(blocks[1].text, "SECTION D - Certifications (attach)");
        assert_eq!(blocks[1].kind, BlockKind::Heading);
    }

    /// The boxes of a form: small labels with the values filled in under
    /// them, further apart than lines of text, and a label over nothing.
    #[test]
    fn small_labels_over_their_values_are_fields_and_a_bare_label_is_text() {
        let runs = vec![
            run(43.0, 102.0, "TO OWNER", 7.5),
            run(309.0, 102.0, "TO CONTRACTOR", 7.5),
            run(45.0, 126.0, "Briarport Library District", 11.0),
            run(311.0, 126.0, "Stonebridge Builders Inc.", 11.0),
            run(40.0, 168.0, "THE CONTRACT IS CHANGED AS FOLLOWS:", 10.0),
        ];

        let blocks = analyze(&runs);

        assert_eq!(blocks[0].kind, BlockKind::Table, "{:#?}", texts(&blocks));
        let fields = &blocks[0].fields;
        assert_eq!(
            (fields[1].key.as_str(), fields[1].value.as_str()),
            ("TO CONTRACTOR", "Stonebridge Builders Inc.")
        );
        assert_eq!(blocks[1].kind, BlockKind::Paragraph);
        assert!(blocks[1].fields.is_empty());
    }

    /// Every labelled value OCR's layout found, as key and value.
    fn fields_of(blocks: &[LayoutBlock]) -> Vec<(&str, &str)> {
        blocks
            .iter()
            .flat_map(|block| &block.fields)
            .map(|field| (field.key.as_str(), field.value.as_str()))
            .collect()
    }

    fn analyze_ocr(runs: &[TextRun]) -> Vec<LayoutBlock> {
        let blocks = analyze_runs(runs, 6120, 7920, &[], TextSource::Ocr);
        assert_words_kept(runs, &blocks);
        blocks
    }

    /// OCR measures a line of capitals smaller than a line of prose under
    /// it, but a part's heading is not the label of the clause it opens:
    /// it stays a line of its own, and gains no colon.
    #[test]
    fn a_heading_read_by_ocr_is_not_the_label_of_its_first_clause() {
        let runs = vec![
            run(
                54.0,
                100.0,
                "3.2 Rent payments will be made by electronic funds transfer",
                10.0,
            ),
            run(
                54.0,
                114.0,
                "to the account Landlord designates in writing each month.",
                10.0,
            ),
            run(54.0, 140.0, "ARTICLE 4 - OPERATING EXPENSES AND TAXES", 7.0),
            run(
                54.0,
                152.0,
                "4.1 Beginning with the second calendar year of the Term, Tenant",
                10.0,
            ),
            run(
                54.0,
                166.0,
                "will pay, as additional rent, its share of Operating Expenses.",
                10.0,
            ),
        ];

        let blocks = analyze_ocr(&runs);

        assert!(fields_of(&blocks).is_empty(), "{:#?}", texts(&blocks));
        assert!(
            blocks
                .iter()
                .any(|block| block.text == "ARTICLE 4 - OPERATING EXPENSES AND TAXES"),
            "{:#?}",
            texts(&blocks)
        );
        assert!(!crate::layout::linearize(&blocks).contains("TAXES:"));
    }

    /// A signature block read by OCR: the firm is not the label of its
    /// signature line, and a line holding a label and its value is not the
    /// label of the next one. Each labelled line is a field of its own.
    #[test]
    fn a_signature_block_read_by_ocr_pairs_only_its_own_labels() {
        let runs = vec![
            run(54.0, 100.0, "Palisade Tower Partners LLC", 7.5),
            run(54.0, 112.0, "By: Wexcombe", 10.0),
            run(54.0, 126.0, "Name: Saoirse Whitcombe", 7.5),
            run(54.0, 138.0, "Title: Authorized Signatory", 10.0),
        ];

        let blocks = analyze_ocr(&runs);

        let fields = fields_of(&blocks);
        assert!(
            !fields
                .iter()
                .any(|(key, _)| *key == "Palisade Tower Partners LLC" || key.contains(':')),
            "{fields:?}"
        );
        assert!(
            fields.contains(&("Name", "Saoirse Whitcombe")),
            "{fields:?}"
        );
        assert!(
            fields.contains(&("Title", "Authorized Signatory")),
            "{fields:?}"
        );
        let text = crate::layout::linearize(&blocks);
        assert!(
            !text.contains("LLC:") && !text.contains("Whitcombe:"),
            "{text}"
        );
    }

    /// A letter's sender over their address is a name, not a label.
    #[test]
    fn a_name_over_an_address_read_by_ocr_is_not_a_field() {
        let runs = vec![
            run(54.0, 100.0, "Mireille Saltonstall", 8.0),
            run(54.0, 111.0, "18 Alder Court", 10.0),
            run(54.0, 123.0, "Wexcombe, OR 97321", 10.0),
            run(54.0, 150.0, "Corriveau Millwork Supply Co.", 8.0),
            run(54.0, 161.0, "4180 Sawmill Creek Road", 10.0),
        ];

        let blocks = analyze_ocr(&runs);

        assert!(fields_of(&blocks).is_empty(), "{:#?}", texts(&blocks));
        assert!(!crate::layout::linearize(&blocks).contains(':'));
    }

    /// What OCR reads of a form's small captions over the boxes filled in
    /// under them stays a set of fields: a caption in capitals, one in
    /// sentence case, one that names its field.
    #[test]
    fn small_captions_read_by_ocr_still_label_their_boxes() {
        let runs = vec![
            run(54.0, 100.0, "PRODUCER", 7.0),
            run(54.0, 110.0, "Hartsfield & Quail Insurance Brokers", 10.0),
            run(54.0, 140.0, "Full legal name", 7.0),
            run(54.0, 150.0, "Ione Kowalczyk", 10.0),
            run(54.0, 180.0, "Insurance Plan", 7.0),
            run(54.0, 190.0, "Meadowlark Health Plan", 10.0),
        ];

        let blocks = analyze_ocr(&runs);

        let fields = fields_of(&blocks);
        assert!(
            fields.contains(&("PRODUCER", "Hartsfield & Quail Insurance Brokers")),
            "{fields:?}"
        );
        assert!(
            fields.contains(&("Full legal name", "Ione Kowalczyk")),
            "{fields:?}"
        );
        assert!(
            fields.contains(&("Insurance Plan", "Meadowlark Health Plan")),
            "{fields:?}"
        );
    }

    /// A value written after its label goes on under it only where its
    /// line wrapped.
    #[test]
    fn a_labelled_line_continues_only_where_it_wrapped() {
        let runs = vec![
            run(
                54.0,
                100.0,
                "Re: Loan No. 40-118273 under the Loan Agreement dated March 18, 2022 between",
                10.0,
            ),
            run(
                54.0,
                112.0,
                "the lender and the borrower, and the related promissory note",
                10.0,
            ),
            run(54.0, 124.0, "Location: Briarport", 10.0),
            run(54.0, 136.0, "Members in Alamosa Bend County", 10.0),
        ];

        let blocks = analyze(&runs);

        assert_eq!(blocks[0].fields.len(), 2, "{:#?}", texts(&blocks));
        assert!(
            blocks[0].fields[0]
                .value
                .ends_with("related promissory note")
        );
        assert_eq!(blocks[0].fields[1].value, "Briarport");
        assert_eq!(blocks[1].text, "Members in Alamosa Bend County");
    }

    /// A page of more runs than the analysis takes on, or of too many on
    /// one line, has no geometry layout: it is read as its text instead.
    #[test]
    fn a_page_past_the_analysis_bounds_is_not_analysed() {
        let line = (0..=MAX_RUNS_PER_LINE)
            .map(|index| run(10.0 + index as f64 * 2.0, 100.0, "x", 3.0))
            .collect::<Vec<_>>();
        assert_eq!(
            analyze_runs_within(&line, 6120, 7920, &[], TextSource::Native, &|| false),
            None
        );
        let page = (0..=MAX_RUNS)
            .map(|index| run(54.0, 10.0 + index as f64 * 0.19, "y", 3.0))
            .collect::<Vec<_>>();
        assert_eq!(
            analyze_runs_within(&page, 6120, 7920, &[], TextSource::Native, &|| false),
            None
        );
        // Told to stop, the analysis stops.
        let two = [
            run(54.0, 100.0, "One line.", 9.0),
            run(54.0, 140.0, "Another.", 9.0),
        ];
        assert_eq!(
            analyze_runs_within(&two, 6120, 7920, &[], TextSource::Native, &|| true),
            None
        );
        assert!(
            analyze_runs_within(&two, 6120, 7920, &[], TextSource::Native, &|| false).is_some()
        );
    }

    /// Rows as they were found before a run was checked only against the
    /// runs near it: against every run on its row.
    fn rows_checked_against_every_run(items: &[Item], ids: &[usize]) -> Vec<Vec<usize>> {
        let mut sorted = ids.to_vec();
        sorted.sort_by(|left, right| {
            items[*left]
                .center_y()
                .total_cmp(&items[*right].center_y())
                .then(items[*left].x0.total_cmp(&items[*right].x0))
                .then(left.cmp(right))
        });
        let mut rows: Vec<Row> = Vec::new();
        for id in sorted {
            let item = &items[id];
            let joins = rows.last().is_some_and(|row| {
                let row_center = (row.y0 + row.y1) / 2.0;
                let item_center = item.center_y();
                item_center >= row.y0
                    && item_center <= row.y1
                    && row_center >= item.y0
                    && row_center <= item.y1
                    && row.members.iter().all(|member| {
                        let other = &items[*member];
                        other.x1 <= item.x0 + 1.0
                            || item.x1 <= other.x0 + 1.0
                            || overflows_into(other, item)
                    })
            });
            if joins {
                let row = rows.last_mut().expect("checked above");
                row.members.push(id);
                row.y0 = row.y0.min(item.y0);
                row.y1 = row.y1.max(item.y1);
            } else {
                rows.push(Row {
                    members: vec![id],
                    y0: item.y0,
                    y1: item.y1,
                });
            }
        }
        rows.into_iter()
            .map(|mut row| {
                row.members.sort_by(|left, right| {
                    items[*left]
                        .x0
                        .total_cmp(&items[*right].x0)
                        .then(left.cmp(right))
                });
                row.members
            })
            .collect()
    }

    /// A run is checked only against the runs on its row that start near
    /// it, and the rows come out as they did when it was checked against
    /// every one: on a fixed scatter of runs that overlap, overprint, run
    /// into each other, and sit over one run as wide as the page.
    #[test]
    fn rows_check_only_the_runs_near_each_run() {
        let mut seed = 0x2545_f491_u64;
        let mut next = |below: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % below
        };
        let mut runs = Vec::new();
        for _ in 0..600 {
            let x = next(500) as f64;
            let y = 100.0 + next(12) as f64 * 6.0 + next(3) as f64;
            let letters = 1 + next(12) as usize;
            let size = 8.0 + next(3) as f64;
            runs.push(run(x, y, &"w".repeat(letters), size));
        }
        runs.push(run(40.0, 106.0, &"W".repeat(110), 9.0));
        let items = items_of(&runs);
        let ids = (0..items.len()).collect::<Vec<_>>();

        let rows = rows_of(&items, &ids)
            .into_iter()
            .map(|row| row.members)
            .collect::<Vec<_>>();

        assert!(rows.len() > 12, "the scatter makes rows of its own");
        assert_eq!(rows, rows_checked_against_every_run(&items, &ids));
    }

    /// One pass over two rows' free stretches finds what every pair would.
    #[test]
    fn free_stretches_intersect_in_one_pass() {
        let these = [(0.0, 100.0), (150.0, 300.0), (400.0, 900.0)];
        let those = [
            (50.0, 200.0),
            (250.0, 450.0),
            (500.0, 600.0),
            (800.0, 1000.0),
        ];
        let mut every_pair = Vec::new();
        for (a0, a1) in these {
            for (b0, b1) in those {
                let (start, end) = (f64::max(a0, b0), f64::min(a1, b1));
                if end - start >= 30.0 {
                    every_pair.push((start, end));
                }
            }
        }

        assert_eq!(intersect(&these, &those, 30.0), every_pair);
    }

    #[test]
    fn analysis_is_deterministic() {
        let runs = (0..30)
            .map(|index| {
                run(
                    54.0 + (index % 3) as f64 * 160.0,
                    100.0 + (index / 3) as f64 * 12.0,
                    &format!("cell {index}"),
                    9.0,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(analyze(&runs), analyze(&runs));
    }
}
