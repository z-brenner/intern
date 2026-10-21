//! Intern's own step after the reference pipeline: a form's labels joined
//! to the values printed beside them.
//!
//! The reference reading-order network reads a form whose labels stand in
//! one column and their values in the next - `Date of notice:` with
//! `08/12/2026` to its right, `Policy number:` with `KC-WC-7710345`, and so
//! on - as the column of labels and then the column of values, often as two
//! blocks. The page's text then says `Date of notice:` seven lines before
//! the date it labels, and no labelled value survives into the layout.
//! Measured on InternBench's scans, that is most of what Indra loses to
//! PP-OCR in character error, and nearly all it loses in labelled values
//! (`docs/ocr.md`).
//!
//! So, after the reference pipeline and before the reading is reported, a
//! label (a short name ending in a colon, as the layout analysis recognises
//! one) whose row carries a value to its right is written as one line,
//! `Label: value`, where the label stood; the value leaves the line and the
//! block it was in. Only a column of labels is joined: a block with at
//! least two labels that pair so, and with pairs for at least half of its
//! lines, so a prose line that happens to end in a colon beside another
//! column is left alone. Tables keep their cells.
//!
//! [`super::IndraOcr::read_reference`], which parity with the reference is
//! measured on, does not run this step.

use super::types::{BlockClass, PageReading};
use crate::layout::is_label;

/// How much of the shorter of two lines' heights they must share to be on
/// one row.
const ROW_OVERLAP: f64 = 0.6;

/// How far right of its label a value may start, in the label's line
/// heights: a form's value column stands a few words from its longest
/// label, never across a page.
const MAX_GAP_HEIGHTS: f64 = 30.0;

fn height(bbox: &[f64; 4]) -> f64 {
    (bbox[3] - bbox[1]).max(1.0)
}

/// Whether `a` and `b` share a row: they overlap vertically by at least
/// [`ROW_OVERLAP`] of the shorter.
fn same_row(a: &[f64; 4], b: &[f64; 4]) -> bool {
    let overlap = a[3].min(b[3]) - a[1].max(b[1]);
    overlap >= ROW_OVERLAP * height(a).min(height(b))
}

/// A line that names the value after it: `Invoice date:`.
fn is_field_label(text: &str) -> bool {
    let text = text.trim();
    text.ends_with(':') && is_label(text)
}

/// Joins each label of a column of labels to the value on its row (see the
/// module documentation). Returns how many labels were joined.
pub fn join_labelled_values(reading: &mut PageReading) -> usize {
    let lines = &reading.lines;
    // The block each line is in, and whether it is a table's.
    let mut block_of = vec![None; lines.len()];
    for (index, block) in reading.blocks.iter().enumerate() {
        for &line in &block.lines {
            if let Some(slot) = block_of.get_mut(line) {
                *slot = Some(index);
            }
        }
    }
    let in_table = |line: usize| {
        block_of[line].is_some_and(|block| reading.blocks[block].class == BlockClass::Table)
    };
    // Each label's value: the nearest line to its right on its row, not
    // itself a label, not in a table, and not far.
    let mut value_of: Vec<Option<usize>> = vec![None; lines.len()];
    for (label, line) in lines.iter().enumerate() {
        if in_table(label) || block_of[label].is_none() || !is_field_label(&line.text) {
            continue;
        }
        let reach = line.bbox[2] + MAX_GAP_HEIGHTS * height(&line.bbox);
        value_of[label] = lines
            .iter()
            .enumerate()
            .filter(|&(other, candidate)| {
                other != label
                    && block_of[other].is_some()
                    && !in_table(other)
                    && !candidate.text.trim().is_empty()
                    && !is_field_label(&candidate.text)
                    && candidate.bbox[0] >= line.bbox[2] - 0.5 * height(&line.bbox)
                    && candidate.bbox[0] <= reach
                    && same_row(&line.bbox, &candidate.bbox)
            })
            .min_by(|a, b| a.1.bbox[0].total_cmp(&b.1.bbox[0]))
            .map(|(other, _)| other);
    }
    // A line claimed as the value of two labels belongs to the nearer.
    for label in 0..lines.len() {
        let Some(value) = value_of[label] else {
            continue;
        };
        let nearer = (0..lines.len()).any(|other| {
            other != label
                && value_of[other] == Some(value)
                && lines[other].bbox[2] > lines[label].bbox[2]
        });
        if nearer {
            value_of[label] = None;
        }
    }
    // Only a column of labels is joined.
    let values: Vec<usize> = value_of.iter().flatten().copied().collect();
    for (index, block) in reading.blocks.iter().enumerate() {
        let pairs = block
            .lines
            .iter()
            .filter(|&&line| value_of[line].is_some())
            .count();
        let rest = block
            .lines
            .iter()
            .filter(|line| !values.contains(line))
            .count();
        if pairs < 2 || pairs * 2 < rest {
            for &line in &block.lines {
                if block_of[line] == Some(index) {
                    value_of[line] = None;
                }
            }
        }
    }
    let mut joined = 0;
    let mut taken = vec![false; reading.lines.len()];
    for (label, value) in value_of.iter().enumerate() {
        let Some(value) = *value else {
            continue;
        };
        let value_line = reading.lines[value].clone();
        let line = &mut reading.lines[label];
        line.text = format!("{} {}", line.text.trim(), value_line.text.trim());
        line.bbox = union(line.bbox, value_line.bbox);
        line.conf = line.conf.min(value_line.conf);
        taken[value] = true;
        joined += 1;
    }
    if joined == 0 {
        return 0;
    }
    for block in &mut reading.blocks {
        block.lines.retain(|&line| !taken[line]);
        let boxes = block.lines.iter().map(|&line| reading.lines[line].bbox);
        if let Some(bbox) = boxes.reduce(union) {
            block.bbox = bbox;
        }
        block.text = block
            .lines
            .iter()
            .map(|&line| reading.lines[line].text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
    }
    reading.blocks.retain(|block| !block.lines.is_empty());
    for (index, line) in reading.lines.iter_mut().enumerate() {
        if taken[index] {
            line.text.clear();
        }
    }
    joined
}

fn union(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].max(b[2]),
        a[3].max(b[3]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indra::types::{PageBlock, PageLine};

    fn line(text: &str, bbox: [f64; 4]) -> PageLine {
        PageLine {
            bbox,
            text: text.into(),
            conf: 0.95,
        }
    }

    fn block(class: BlockClass, lines: Vec<usize>) -> PageBlock {
        PageBlock {
            class,
            bbox: [0.0; 4],
            text: String::new(),
            lines,
        }
    }

    fn reading(lines: Vec<PageLine>, blocks: Vec<PageBlock>) -> PageReading {
        PageReading {
            rotation: 0,
            skew: 0.0,
            lines,
            blocks,
            markdown: String::new(),
            text: String::new(),
        }
    }

    fn texts(reading: &PageReading) -> Vec<Vec<&str>> {
        reading
            .blocks
            .iter()
            .map(|block| {
                block
                    .lines
                    .iter()
                    .map(|&line| reading.lines[line].text.as_str())
                    .collect()
            })
            .collect()
    }

    /// Labels in one block, their values in the next: each value joins its
    /// label, and the emptied block goes.
    #[test]
    fn a_column_of_labels_takes_the_values_beside_it() {
        let mut page = reading(
            vec![
                line("Payment date:", [100.0, 100.0, 260.0, 120.0]),
                line("Payee:", [100.0, 130.0, 180.0, 150.0]),
                line("07/17/2026", [400.0, 101.0, 520.0, 121.0]),
                line("Sablewood Packaging Corp.", [400.0, 131.0, 700.0, 151.0]),
                line("Funds were sent by ACH.", [100.0, 200.0, 700.0, 220.0]),
            ],
            vec![
                block(BlockClass::Text, vec![0, 1]),
                block(BlockClass::Text, vec![2, 3]),
                block(BlockClass::Text, vec![4]),
            ],
        );
        assert_eq!(join_labelled_values(&mut page), 2);
        assert_eq!(
            texts(&page),
            [
                vec![
                    "Payment date: 07/17/2026",
                    "Payee: Sablewood Packaging Corp."
                ],
                vec!["Funds were sent by ACH."],
            ]
        );
        assert_eq!(page.blocks[0].bbox, [100.0, 100.0, 700.0, 151.0]);
        assert!(page.lines[2].text.is_empty());
    }

    /// Labels and values in one block, read down the labels first.
    #[test]
    fn labels_and_values_in_one_block_are_paired_row_by_row() {
        let mut page = reading(
            vec![
                line("Date of notice:", [100.0, 100.0, 280.0, 120.0]),
                line("Policy number:", [100.0, 130.0, 270.0, 150.0]),
                line("Producer:", [100.0, 160.0, 200.0, 180.0]),
                line("08/12/2026", [420.0, 100.0, 540.0, 120.0]),
                line("KC-WC-7710345", [420.0, 130.0, 600.0, 150.0]),
                line(
                    "Ridley Cross Insurance Services",
                    [420.0, 160.0, 800.0, 180.0],
                ),
            ],
            vec![block(BlockClass::Form, vec![0, 1, 2, 3, 4, 5])],
        );
        assert_eq!(join_labelled_values(&mut page), 3);
        assert_eq!(
            texts(&page),
            [vec![
                "Date of notice: 08/12/2026",
                "Policy number: KC-WC-7710345",
                "Producer: Ridley Cross Insurance Services",
            ]]
        );
    }

    /// One label beside another column is prose ending in a colon, not a
    /// form, and a table keeps its cells.
    #[test]
    fn a_lone_label_and_a_table_are_left_alone() {
        let lone = vec![
            line("Note:", [100.0, 100.0, 160.0, 120.0]),
            line(
                "The right column goes on here.",
                [600.0, 100.0, 1000.0, 120.0],
            ),
            line("More of the left column.", [100.0, 130.0, 500.0, 150.0]),
        ];
        let mut page = reading(
            lone.clone(),
            vec![
                block(BlockClass::Text, vec![0, 2]),
                block(BlockClass::Text, vec![1]),
            ],
        );
        let before = page.clone();
        assert_eq!(join_labelled_values(&mut page), 0);
        assert_eq!(page, before);

        let mut table = reading(
            vec![
                line("Item:", [100.0, 100.0, 160.0, 120.0]),
                line("Price:", [100.0, 130.0, 170.0, 150.0]),
                line("Tea", [300.0, 100.0, 340.0, 120.0]),
                line("2.50", [300.0, 130.0, 350.0, 150.0]),
            ],
            vec![block(BlockClass::Table, vec![0, 1, 2, 3])],
        );
        let before = table.clone();
        assert_eq!(join_labelled_values(&mut table), 0);
        assert_eq!(table, before);
    }

    /// A value far across the page, or on another row, is not the label's.
    #[test]
    fn values_must_be_near_and_on_the_labels_row() {
        let mut page = reading(
            vec![
                line("Ship date:", [100.0, 100.0, 200.0, 120.0]),
                line("Carrier:", [100.0, 130.0, 180.0, 150.0]),
                line("09/08/2026", [300.0, 112.0, 420.0, 132.0]),
                line(
                    "Pinecrest Motor Freight Inc.",
                    [2000.0, 130.0, 2400.0, 150.0],
                ),
            ],
            vec![block(BlockClass::Text, vec![0, 1, 2, 3])],
        );
        let before = page.clone();
        assert_eq!(join_labelled_values(&mut page), 0);
        assert_eq!(page, before);
    }
}
