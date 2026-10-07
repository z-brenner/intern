//! Blocks from a page's text: the readers that know no geometry, and PDF
//! pages on the fast route, whose text is kept exactly as PDFium read it.
//!
//! The structure is the text's own: Markdown headings, blank lines between
//! paragraphs, `|` table rows (which is how every Office, sheet, and HTML
//! table already arrives), list markers, and `Key: value` lines. Short lines
//! that are nearly all capitals read as headings, the same rule distillation
//! uses. When a line's box is known, a gap wider than the page's line
//! spacing or a jump in type size also ends a paragraph, and a line set
//! larger than the text around it is a heading.

use super::{
    BlockKind, KeyValue, LayoutBlock, LayoutCell, LayoutLine, LayoutRow, LayoutTable, TextSource,
    mean_confidence, union,
};

/// Paragraphs longer than this many lines are split at the next line that
/// ends a sentence, so a page of PDF text with no blank lines in it is not
/// one block.
const MAX_PARAGRAPH_LINES: usize = 12;

/// Blocks from text with no geometry. Ids are left for
/// [`super::number_blocks`].
pub fn blocks_from_text(text: &str, source: TextSource) -> Vec<LayoutBlock> {
    let lines = text
        .lines()
        .map(|line| LayoutLine {
            text: line.trim_end().to_owned(),
            bbox: None,
            confidence: None,
        })
        .collect::<Vec<_>>();
    blocks_from_lines(&lines, source)
}

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

/// Blocks from lines in reading order. A line's text is kept exactly, less
/// trailing whitespace; a block's text is its lines joined with newlines.
pub fn blocks_from_lines(lines: &[LayoutLine], source: TextSource) -> Vec<LayoutBlock> {
    let kinds = lines
        .iter()
        .map(|line| classify(&line.text))
        .collect::<Vec<_>>();
    let geometry = Geometry::of(lines);
    let mut blocks = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let kind = kinds[index];
        let start = index;
        match kind {
            LineKind::Blank => {
                index += 1;
                continue;
            }
            LineKind::MarkdownHeading(level) => {
                index += 1;
                let mut block = block_of(BlockKind::Heading, &lines[start..index], source);
                block.level = Some(level);
                blocks.push(block);
            }
            LineKind::TableRow | LineKind::TableSeparator => {
                while index < lines.len()
                    && matches!(kinds[index], LineKind::TableRow | LineKind::TableSeparator)
                {
                    index += 1;
                }
                blocks.push(table_block(
                    &lines[start..index],
                    &kinds[start..index],
                    source,
                ));
            }
            LineKind::KeyValue => {
                index += 1;
                while index < lines.len()
                    && kinds[index] == LineKind::KeyValue
                    && !geometry.breaks_between(lines, index - 1, index)
                {
                    index += 1;
                }
                blocks.push(key_value_block(&lines[start..index], source));
            }
            LineKind::Heading => {
                index += 1;
                blocks.push(block_of(BlockKind::Heading, &lines[start..index], source));
            }
            LineKind::ListItem | LineKind::Text => {
                index += 1;
                while index < lines.len()
                    && kinds[index] == LineKind::Text
                    && !geometry.breaks_between(lines, index - 1, index)
                    && !geometry.is_larger(lines, index)
                    && !(index - start >= MAX_PARAGRAPH_LINES && ends_sentence(&lines[index - 1]))
                {
                    index += 1;
                }
                let block_kind = if kind == LineKind::ListItem {
                    BlockKind::ListItem
                } else if index - start == 1 && geometry.is_larger(lines, start) {
                    BlockKind::Heading
                } else {
                    BlockKind::Paragraph
                };
                blocks.push(block_of(block_kind, &lines[start..index], source));
            }
        }
    }
    blocks
}

fn ends_sentence(line: &LayoutLine) -> bool {
    line.text.trim_end().ends_with(['.', ':', ';', '!', '?'])
}

/// What the lines' boxes say about where paragraphs end, when there are
/// boxes: the usual gap between one line and the next, and the usual line
/// height.
struct Geometry {
    gap: Option<f64>,
    height: Option<f64>,
}

impl Geometry {
    fn of(lines: &[LayoutLine]) -> Self {
        let boxes = lines.iter().map(|line| line.bbox).collect::<Vec<_>>();
        let mut heights = boxes
            .iter()
            .flatten()
            .map(|bbox| f64::from(bbox[3].saturating_sub(bbox[1])))
            .filter(|height| *height > 0.0)
            .collect::<Vec<_>>();
        let mut gaps = boxes
            .windows(2)
            .filter_map(|pair| match pair {
                [Some(above), Some(below)] if below[1] >= above[3] => {
                    Some(f64::from(below[1] - above[3]))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        Self {
            gap: median(&mut gaps),
            height: median(&mut heights),
        }
    }

    /// Whether the space between two consecutive lines is a paragraph
    /// break: wider than the page's usual line gap by half a line, or a
    /// line set noticeably larger or smaller.
    fn breaks_between(&self, lines: &[LayoutLine], above: usize, below: usize) -> bool {
        let (Some(upper), Some(lower)) = (lines[above].bbox, lines[below].bbox) else {
            return false;
        };
        let (Some(gap), Some(height)) = (self.gap, self.height) else {
            return false;
        };
        let space = f64::from(lower[1]) - f64::from(upper[3]);
        if space < -height * 0.5 {
            // The next line starts above this one: a new column or region.
            return true;
        }
        let upper_height = f64::from(upper[3].saturating_sub(upper[1]));
        let lower_height = f64::from(lower[3].saturating_sub(lower[1]));
        space > gap + height * 0.5 || (upper_height - lower_height).abs() > height * 0.2
    }

    /// Whether a line is set noticeably larger than the page's text.
    fn is_larger(&self, lines: &[LayoutLine], index: usize) -> bool {
        let (Some(bbox), Some(height)) = (lines[index].bbox, self.height) else {
            return false;
        };
        f64::from(bbox[3].saturating_sub(bbox[1])) > height * 1.18
    }
}

pub(crate) fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[values.len() / 2])
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
pub(crate) fn table_cells(line: &str) -> Vec<String> {
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
pub(crate) fn is_list_item_text(line: &str) -> bool {
    is_list_item(line.trim())
}

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

/// A short line that is nearly all capitals: the rule distillation uses.
pub(crate) fn is_heading_line(line: &str) -> bool {
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

/// `Invoice date: May 1, 2026` as its label and value.
///
/// The label is at most six words and forty characters, starts with a
/// letter, and holds no sentence inside it; the colon is followed by a
/// space. Times (`12:01`), URLs, and ratios are not labels.
pub(crate) fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(": ")?;
    let key = line[..colon].trim();
    let value = line[colon + 2..].trim();
    if value.is_empty() || !is_label(key) {
        return None;
    }
    Some((key, value))
}

/// Whether text reads as a field label: short, starting with a letter, no
/// sentence punctuation inside it.
pub(crate) fn is_label(key: &str) -> bool {
    let key = key.trim().trim_end_matches(':').trim();
    if key.is_empty() || key.chars().count() > 40 || key.split_whitespace().count() > 6 {
        return false;
    }
    let Some(first) = key.chars().next() else {
        return false;
    };
    if !first.is_alphabetic() {
        return false;
    }
    if key.contains(['"', '“', '”', ';', ',', ':']) {
        return false;
    }
    // A sentence that happens to contain a colon - "Tenant will pay the
    // following: rent" - has lowercase words and runs long; a label is a
    // name.
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

fn block_of(kind: BlockKind, lines: &[LayoutLine], source: TextSource) -> LayoutBlock {
    let mut block = LayoutBlock::new(kind, join_lines(lines), source);
    block.bbox = union(lines.iter().filter_map(|line| line.bbox));
    block.confidence = mean_confidence(lines.iter().map(|line| line.confidence));
    block.lines = lines.to_vec();
    block
}

fn join_lines(lines: &[LayoutLine]) -> String {
    lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn table_block(lines: &[LayoutLine], kinds: &[LineKind], source: TextSource) -> LayoutBlock {
    let mut block = block_of(BlockKind::Table, lines, source);
    let mut rows: Vec<LayoutRow> = Vec::new();
    for (line, kind) in lines.iter().zip(kinds) {
        if *kind == LineKind::TableSeparator {
            // The row above a separator is the header.
            if let Some(row) = rows.last_mut() {
                for cell in &mut row.cells {
                    cell.header = true;
                }
            }
            continue;
        }
        rows.push(LayoutRow {
            id: String::new(),
            cells: table_cells(&line.text)
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
    block.table = Some(LayoutTable { rows });
    block
}

fn key_value_block(lines: &[LayoutLine], source: TextSource) -> LayoutBlock {
    let mut block = block_of(BlockKind::KeyValue, lines, source);
    block.fields = lines
        .iter()
        .filter_map(|line| {
            let (key, value) = split_key_value(line.text.trim())?;
            Some(KeyValue {
                id: String::new(),
                key: key.trim_end_matches(':').trim().to_owned(),
                value: value.to_owned(),
                key_bbox: None,
                value_bbox: None,
            })
        })
        .collect();
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(blocks: &[LayoutBlock]) -> Vec<BlockKind> {
        blocks.iter().map(|block| block.kind).collect()
    }

    #[test]
    fn markdown_structure_becomes_blocks() {
        let text = "# Written Consent\n\nThe undersigned directors consent.\n\
                    It is resolved.\n\n## Resolutions\n\n| Item | Vote |\n| --- | --- |\n\
                    | Budget | For |\n|  | Against |\n\n- First point\n- Second point\n\
                    Date: March 4, 2026\nSigned by: Ada Example";

        let blocks = blocks_from_text(text, TextSource::Native);

        assert_eq!(
            kinds(&blocks),
            [
                BlockKind::Heading,
                BlockKind::Paragraph,
                BlockKind::Heading,
                BlockKind::Table,
                BlockKind::ListItem,
                BlockKind::ListItem,
                BlockKind::KeyValue,
            ]
        );
        assert_eq!(blocks[0].level, Some(1));
        assert_eq!(blocks[2].level, Some(2));
        assert_eq!(
            blocks[1].text,
            "The undersigned directors consent.\nIt is resolved."
        );
        let table = blocks[3].table.as_ref().unwrap();
        assert_eq!(table.rows.len(), 3, "the separator is not a row");
        assert!(table.rows[0].cells.iter().all(|cell| cell.header));
        assert_eq!(
            table.rows[2].cells[0].text, "",
            "an empty cell keeps its column"
        );
        assert_eq!(table.rows[2].cells[1].text, "Against");
        assert!(
            blocks[3].text.contains("| --- | --- |"),
            "the text is exact"
        );
        let fields = &blocks[6].fields;
        assert_eq!(fields.len(), 2);
        assert_eq!(
            (fields[0].key.as_str(), fields[0].value.as_str()),
            ("Date", "March 4, 2026")
        );
        assert_eq!(fields[1].key, "Signed by");
    }

    #[test]
    fn capital_lines_are_headings_and_labels_are_not_sentences() {
        let blocks = blocks_from_text(
            "NOTICE OF TERMINATION\nThe tenant will pay the following amounts: rent and fees.\n\
             Time: 12:01 a.m.\nhttp://example.test/x",
            TextSource::Native,
        );

        assert_eq!(
            kinds(&blocks),
            [
                BlockKind::Heading,
                BlockKind::Paragraph,
                BlockKind::KeyValue,
                BlockKind::Paragraph
            ]
        );
        assert_eq!(blocks[2].fields[0].value, "12:01 a.m.");
    }

    #[test]
    fn a_long_unbroken_page_is_split_at_sentence_ends() {
        let text = (0..30)
            .map(|line| {
                if line % 5 == 4 {
                    format!("line {line} ends here.")
                } else {
                    format!("line {line} runs on")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let blocks = blocks_from_text(&text, TextSource::Native);

        assert!(blocks.len() >= 2, "{blocks:#?}");
        assert!(
            blocks
                .iter()
                .all(|block| block.lines.len() <= MAX_PARAGRAPH_LINES + 5)
        );
        let rejoined = blocks
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(rejoined, text);
    }

    #[test]
    fn line_boxes_break_paragraphs_and_find_large_headings() {
        let line = |text: &str, y: u32, height: u32| LayoutLine {
            text: text.to_owned(),
            bbox: Some([540, y, 3000, y + height]),
            confidence: None,
        };
        let lines = [
            line("Retail Lease", 400, 160),
            line("This lease is made between", 700, 90),
            line("the landlord and the tenant.", 815, 90),
            line("The term begins in May", 930, 90),
            line("A second paragraph starts", 1200, 90),
            line("after a wide gap.", 1315, 90),
        ];

        let blocks = blocks_from_lines(&lines, TextSource::Native);

        assert_eq!(
            kinds(&blocks),
            [
                BlockKind::Heading,
                BlockKind::Paragraph,
                BlockKind::Paragraph
            ]
        );
        assert_eq!(blocks[1].lines.len(), 3);
        assert_eq!(blocks[1].bbox, Some([540, 700, 3000, 1020]));
    }

    #[test]
    fn list_markers_are_recognised() {
        for item in [
            "• Bullet",
            "- dash item",
            "(a) first",
            "(iv) fourth",
            "3) third",
            "a) one",
        ] {
            assert!(is_list_item(item), "{item}");
        }
        for text in ["-5 degrees", "(see above)", "2026) note", "Article 4"] {
            assert!(!is_list_item(text), "{text}");
        }
    }
}
