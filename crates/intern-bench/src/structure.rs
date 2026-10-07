//! Whether the text Intern reads keeps the page's structure: the reading
//! order, the tables, the labelled values, and the route each page took.
//!
//! Everything is measured over the page text the engine receives - the
//! same text distillation and the prompt are built from - not over the
//! worker's blocks, so a worker that sends no layouts (every build before
//! the router) is scored on the same footing as one that does. Only
//! `route_correct` reads the layouts, and it is left unscored when there
//! are none.
//!
//! Text is compared after [`normalise`]: every run of whitespace is one
//! space, typographic quotes are straight and dashes are hyphens, so line
//! wrapping and OCR's choice of quote mark do not count as errors. Case
//! counts. A gold string is found only where it stands on its own: one
//! that begins or ends with a letter or digit may not continue a word, and
//! one that begins or ends with a digit may not continue a number (`4` is
//! not found in `14` or in `1,4`), so a short cell cannot be credited to
//! the middle of another value.
//!
//! The text is read page by page, in page order, as *lines*: each page's
//! text split at line breaks, normalised, blank lines dropped. A table row
//! the worker linearised as `| a | b |` is one line.
//!
//! * `reading_order_accuracy`: each snippet's position is its first
//!   occurrence in the whole text (pages joined, normalised). The snippets
//!   in order are the most of the found ones whose positions rise in the
//!   gold's order (a longest increasing subsequence), and the score is
//!   their share of every gold snippet: a snippet not found is out of
//!   order. Two whole columns read the wrong way round keep only the
//!   longer column in order, where counting consecutive pairs would lose
//!   only the one pair across the swap.
//! * `table_row_accuracy`: a gold row is found when one line of its
//!   table's region (below) holds all of its non-empty cells in order, each
//!   after the end of the one before, with no cell of another row of the
//!   table wholly between two of them - two rows read across each other
//!   are neither found. A blank cell says something too. When some rows
//!   of a table leave the leading cell blank and others do not - a
//!   check-box group, whose chosen options are marked `X` - the values the
//!   others hold there are the table's *marks*, and a row whose leading
//!   cell is blank is not found on a line where a mark stands between the
//!   cell of another row before it (or the line's start) and its first
//!   cell: that `X` is beside an option the gold leaves unmarked. The
//!   score is the share of rows found, over every table.
//! * `table_cell_recall`: the share of the gold's non-empty cells found in
//!   their table's *region*: the lines from the first one holding a cell
//!   of the table's first row (the first line, if none does) to the last
//!   one, from there on, holding a cell of its last row (the last line, if
//!   none does); a table split over two pages spans the break. Only cells
//!   of two characters or more place the region - a lone `X` or digit is
//!   printed all over a page, so it neither anchors nor widens it - and a
//!   first or last row without one gives way to the nearest row with one.
//!   Cells are counted with their multiplicity: a value the table prints
//!   three times must be found three times, without overlap.
//! * `kv_accuracy`: a labelled value is found when a line holds the label
//!   and, after it but before the next gold label on the line, the value
//!   (a value after another label is that label's); or when a line holds
//!   the label and nothing else (colons, bars, dashes and full stops
//!   aside) and the next line holds the value; or when a table row
//!   (`| … |`) holds the label as a whole cell (`CONTRACT DATE` is not the
//!   cell of `DATE`) and the next line, also a table row, holds the value
//!   in the cell of the same column - a label set over its value,
//!   linearised as a table. The score is the share of pairs found.
//! * `route_correct`: the share of the pages with an expected route whose
//!   layout took that route. A page sent without a layout while others have
//!   one took no route and is wrong; a document with no layout at all is
//!   not scored.

use serde::{Deserialize, Serialize};

use intern_engine::{DocumentSource, structure::PageRoute};

use crate::gold::{StructureTruth, TableTruth};

/// The route's name as the worker writes it.
pub fn route_name(route: PageRoute) -> &'static str {
    match route {
        PageRoute::Fast => "fast",
        PageRoute::Layout => "layout",
        PageRoute::Ocr => "ocr",
        PageRoute::OcrRegions => "ocr_regions",
    }
}

/// How costly a route is to read, for a document's route class.
fn route_rank(route: PageRoute) -> u8 {
    match route {
        PageRoute::Fast => 0,
        PageRoute::Layout => 1,
        PageRoute::OcrRegions => 2,
        PageRoute::Ocr => 3,
    }
}

/// The route of every page, in page order: its name, or `none` for a page
/// sent without a layout.
pub fn page_routes(source: &DocumentSource) -> Vec<String> {
    source
        .pages
        .iter()
        .map(|page| {
            page.layout
                .as_ref()
                .map_or("none", |layout| route_name(layout.route))
                .to_owned()
        })
        .collect()
}

/// A document's route class: the most expensive route any of its pages
/// took (ocr, then ocr_regions, then layout, then fast), or `unrouted`
/// when no page came with a layout.
pub fn route_class(source: &DocumentSource) -> &'static str {
    source
        .pages
        .iter()
        .filter_map(|page| page.layout.as_ref().map(|layout| layout.route))
        .max_by_key(|route| route_rank(*route))
        .map_or("unrouted", route_name)
}

/// The text with whitespace collapsed, quotes straightened and dashes made
/// hyphens.
pub fn normalise(text: &str) -> String {
    let mapped = text
        .chars()
        .map(|character| match character {
            '\u{2018}' | '\u{2019}' | '\u{201a}' | '\u{2032}' => '\'',
            '\u{201c}' | '\u{201d}' | '\u{201e}' | '\u{2033}' => '"',
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
            other => other,
        })
        .collect::<String>();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether the text just before `at` continues a number into it: a digit,
/// or a digit then `.` or `,`.
fn number_before(text: &str, at: usize) -> bool {
    let mut previous = text[..at].chars().rev();
    match previous.next() {
        Some(character) if character.is_ascii_digit() => true,
        Some('.' | ',') => previous.next().is_some_and(|c| c.is_ascii_digit()),
        _ => false,
    }
}

/// Whether the text from `at` continues a number: a digit, or `.` or `,`
/// then a digit.
fn number_after(text: &str, at: usize) -> bool {
    let mut next = text[at..].chars();
    match next.next() {
        Some(character) if character.is_ascii_digit() => true,
        Some('.' | ',') => next.next().is_some_and(|c| c.is_ascii_digit()),
        _ => false,
    }
}

/// The first place at or after `from` where `needle` stands on its own in
/// `haystack` (see the module documentation).
pub fn find_bounded(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let first = needle.chars().next()?;
    let last = needle.chars().next_back()?;
    let mut start = from;
    while start <= haystack.len() {
        let at = start + haystack.get(start..)?.find(needle)?;
        let end = at + needle.len();
        let before = haystack[..at].chars().next_back();
        let after = haystack[end..].chars().next();
        let joined_before = first.is_alphanumeric() && before.is_some_and(char::is_alphanumeric)
            || first.is_ascii_digit() && number_before(haystack, at);
        let joined_after = last.is_alphanumeric() && after.is_some_and(char::is_alphanumeric)
            || last.is_ascii_digit() && number_after(haystack, end);
        if !joined_before && !joined_after {
            return Some(at);
        }
        start = at + first.len_utf8();
    }
    None
}

/// How many times `needle` stands on its own in `haystack`, without
/// overlap.
fn count_bounded(haystack: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut from = 0;
    while let Some(at) = find_bounded(haystack, needle, from) {
        count += 1;
        from = at + needle.len();
    }
    count
}

/// Every place `needle` stands on its own in `haystack`, overlapping or
/// not, in order.
fn occurrences(haystack: &str, needle: &str) -> Vec<usize> {
    let step = needle.chars().next().map_or(1, char::len_utf8);
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = find_bounded(haystack, needle, from) {
        found.push(at);
        from = at + step;
    }
    found
}

/// Whether one of `values` stands on its own wholly inside
/// `line[start..end]`. The first occurrence from `start` decides it: a
/// later one ends later still.
fn holds_within(line: &str, values: &[&str], start: usize, end: usize) -> bool {
    values
        .iter()
        .any(|value| find_bounded(line, value, start).is_some_and(|at| at + value.len() <= end))
}

/// Whether a cell may place a table: a single character - a check mark, a
/// one-digit count - is printed all over a page.
fn anchors(cell: &str) -> bool {
    cell.chars().count() > 1
}

/// A gold row as it is matched: its non-blank cells, in order, and whether
/// its leading cell is blank.
struct GoldRow {
    cells: Vec<String>,
    blank_lead: bool,
}

/// A gold table as it is matched: its rows with a non-blank cell, and its
/// marks - the values the other rows hold in the leading column when some
/// row leaves it blank, as a check-box group marks the chosen options `X`.
/// A blank leading cell is an empty box: it is part of what the row says.
struct GoldTable {
    rows: Vec<GoldRow>,
    marks: Vec<String>,
}

impl GoldTable {
    fn of(table: &TableTruth) -> Self {
        let rows = table
            .rows
            .iter()
            .filter_map(|row| {
                let cells = row.iter().map(|cell| normalise(cell)).collect::<Vec<_>>();
                let blank_lead = cells.first().is_some_and(String::is_empty);
                let cells = cells
                    .into_iter()
                    .filter(|cell| !cell.is_empty())
                    .collect::<Vec<_>>();
                (!cells.is_empty()).then_some(GoldRow { cells, blank_lead })
            })
            .collect::<Vec<_>>();
        let mut marks = Vec::new();
        if rows.iter().any(|row| row.blank_lead) {
            for row in rows.iter().filter(|row| !row.blank_lead) {
                if !marks.contains(&row.cells[0]) {
                    marks.push(row.cells[0].clone());
                }
            }
        }
        Self { rows, marks }
    }

    /// The non-blank cells of every row but `index`, marks left out: what
    /// may stand before a row on its line without being part of it.
    fn captions_besides(&self, index: usize) -> Vec<&str> {
        let mut captions = Vec::new();
        for (other, row) in self.rows.iter().enumerate() {
            if other == index {
                continue;
            }
            for cell in &row.cells {
                if !self.marks.contains(cell) && !captions.contains(&cell.as_str()) {
                    captions.push(cell.as_str());
                }
            }
        }
        captions
    }

    /// The lines the table is looked for in, as `(first, last)`: from the
    /// first line holding a cell of the first row that has a cell of two
    /// characters or more, to the last line from there holding one of the
    /// last such row. Single characters neither place nor widen it.
    fn region(&self, lines: &[String]) -> (usize, usize) {
        let anchored = |row: &GoldRow| {
            row.cells
                .iter()
                .filter(|cell| anchors(cell))
                .cloned()
                .collect::<Vec<_>>()
        };
        let holds_any = |line: &String, cells: &[String]| {
            cells
                .iter()
                .any(|cell| find_bounded(line, cell, 0).is_some())
        };
        let first = self
            .rows
            .iter()
            .map(anchored)
            .find(|cells| !cells.is_empty());
        let last = self
            .rows
            .iter()
            .rev()
            .map(anchored)
            .find(|cells| !cells.is_empty());
        let start = first
            .and_then(|cells| lines.iter().position(|line| holds_any(line, &cells)))
            .unwrap_or(0);
        let end = last
            .and_then(|cells| {
                lines
                    .iter()
                    .enumerate()
                    .skip(start)
                    .filter(|(_, line)| holds_any(line, &cells))
                    .map(|(index, _)| index)
                    .next_back()
            })
            .unwrap_or(lines.len().saturating_sub(1));
        (start, end.max(start))
    }

    /// The non-blank cells of every row but `index`, marks included.
    fn cells_besides(&self, index: usize) -> Vec<&str> {
        let mut cells = Vec::new();
        for (other, row) in self.rows.iter().enumerate() {
            if other == index {
                continue;
            }
            for cell in &row.cells {
                if !cells.contains(&cell.as_str()) {
                    cells.push(cell.as_str());
                }
            }
        }
        cells
    }

    /// Whether `line` holds row `index`: every non-blank cell in order,
    /// each starting after the end of the one before, with no cell of
    /// another row of the table wholly between two of them; and, when its
    /// leading cell is blank, no mark between the cell of another row
    /// before it on the line (or the line's start) and its first cell - an
    /// `X` there is beside this option, which the gold leaves unmarked.
    fn holds_row(&self, line: &str, index: usize) -> bool {
        let row = &self.rows[index];
        let marks = self.marks.iter().map(String::as_str).collect::<Vec<_>>();
        let captions = self.captions_besides(index);
        let others = self.cells_besides(index);
        let unmarked = |at: usize| {
            let since = captions
                .iter()
                .flat_map(|caption| {
                    occurrences(line, caption)
                        .into_iter()
                        .map(move |start| start + caption.len())
                })
                .filter(|end| *end <= at)
                .max()
                .unwrap_or(0);
            !holds_within(line, &marks, since, at)
        };
        // The ends of the cell's occurrences that complete the row so far.
        let mut ends: Vec<usize> = Vec::new();
        for (position, cell) in row.cells.iter().enumerate() {
            ends = occurrences(line, cell)
                .into_iter()
                .filter(|at| {
                    if position == 0 {
                        !row.blank_lead || unmarked(*at)
                    } else {
                        ends.iter()
                            .any(|end| end <= at && !holds_within(line, &others, *end, *at))
                    }
                })
                .map(|at| at + cell.len())
                .collect();
            if ends.is_empty() {
                return false;
            }
        }
        true
    }
}

/// Whether a line holds nothing but separators once the label is taken out.
fn only_separators(rest: &str) -> bool {
    rest.chars()
        .all(|character| character.is_whitespace() || matches!(character, ':' | '|' | '-' | '.'))
}

/// The cells of a line linearised as a table row (`| a | b |`), or none
/// for any other line.
fn table_cells(line: &str) -> Option<Vec<&str>> {
    let inner = line.strip_prefix('|')?.strip_suffix('|')?;
    Some(inner.split('|').map(str::trim).collect())
}

/// Whether a table row holds the label as one whole cell (a colon after it
/// aside) and the next line, a row of the same table, holds the value in
/// the cell below it: how a table linearises labels set over their values.
/// A cell that only contains the label - `CONTRACT DATE` for `DATE` - is
/// another label.
fn below_in_table(lines: &[String], key: &str, value: &str) -> bool {
    lines.windows(2).any(|pair| {
        let (Some(labels), Some(values)) = (table_cells(&pair[0]), table_cells(&pair[1])) else {
            return false;
        };
        labels.iter().enumerate().any(|(column, cell)| {
            cell.strip_suffix(':').unwrap_or(cell).trim_end() == key
                && values
                    .get(column)
                    .is_some_and(|below| find_bounded(below, value, 0).is_some())
        })
    })
}

/// A page the gold gives a route for, and the route it took.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RouteCheck {
    pub page: usize,
    pub expected: String,
    /// The route the page took, or `none` when it came without a layout.
    pub actual: String,
}

/// Every structure figure for one document, as counts so a corpus figure
/// pools them by item.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StructureMeasure {
    /// Reading-order snippets, how many were found at all, and how many
    /// of those the longest run in the gold's order holds.
    pub snippets: usize,
    pub snippets_found: usize,
    #[serde(default)]
    pub snippets_in_order: usize,
    /// Table rows with a non-empty cell, and how many sat on one line.
    pub rows: usize,
    pub rows_found: usize,
    /// Non-empty table cells, and how many were found in their table.
    pub cells: usize,
    pub cells_found: usize,
    pub key_values: usize,
    pub key_values_found: usize,
    /// Pages with an expected route that could be judged, and how many
    /// took it. Zero when the worker sent no layouts.
    pub route_pages: usize,
    pub routes_correct: usize,
    /// The extraction failed: nothing was read, every item missed.
    #[serde(default)]
    pub failed: bool,
    /// Every page judged on its route.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<RouteCheck>,
    /// What was not found, for a person reading why a figure fell.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub misses: Vec<String>,
}

fn share(found: usize, total: usize) -> Option<f64> {
    (total > 0).then(|| found as f64 / total as f64)
}

impl StructureMeasure {
    pub fn reading_order_accuracy(&self) -> Option<f64> {
        share(self.snippets_in_order, self.snippets)
    }

    pub fn table_row_accuracy(&self) -> Option<f64> {
        share(self.rows_found, self.rows)
    }

    pub fn table_cell_recall(&self) -> Option<f64> {
        share(self.cells_found, self.cells)
    }

    pub fn kv_accuracy(&self) -> Option<f64> {
        share(self.key_values_found, self.key_values)
    }

    pub fn route_correct(&self) -> Option<f64> {
        share(self.routes_correct, self.route_pages)
    }

    /// The scores this measure defines, by key.
    pub fn scores(&self) -> [(&'static str, Option<f64>); 5] {
        [
            ("reading_order_accuracy", self.reading_order_accuracy()),
            ("table_row_accuracy", self.table_row_accuracy()),
            ("table_cell_recall", self.table_cell_recall()),
            ("kv_accuracy", self.kv_accuracy()),
            ("route_correct", self.route_correct()),
        ]
    }

    /// A document whose extraction failed: every item the gold lists is
    /// missed, every page with an expected route took none.
    pub fn unread(truth: &StructureTruth) -> Self {
        let (rows, cells) = table_counts(truth);
        Self {
            snippets: truth
                .reading_order
                .iter()
                .filter(|snippet| !normalise(snippet).is_empty())
                .count(),
            rows,
            cells,
            key_values: truth
                .key_values
                .iter()
                .filter(|pair| !normalise(&pair.value).is_empty())
                .count(),
            route_pages: truth.expected_routes.len(),
            failed: true,
            ..Self::default()
        }
    }

    /// Adds another document's counts, for a corpus figure.
    pub fn accumulate(&mut self, other: &Self) {
        self.snippets += other.snippets;
        self.snippets_found += other.snippets_found;
        self.snippets_in_order += other.snippets_in_order;
        self.rows += other.rows;
        self.rows_found += other.rows_found;
        self.cells += other.cells;
        self.cells_found += other.cells_found;
        self.key_values += other.key_values;
        self.key_values_found += other.key_values_found;
        self.route_pages += other.route_pages;
        self.routes_correct += other.routes_correct;
    }
}

/// The indices of a longest strictly increasing run of `positions`, in
/// order (the earliest such run where there are several).
fn longest_increasing(positions: &[usize]) -> Vec<usize> {
    let mut length = vec![1_usize; positions.len()];
    let mut previous: Vec<Option<usize>> = vec![None; positions.len()];
    for (index, position) in positions.iter().enumerate() {
        for (earlier, before) in positions[..index].iter().enumerate() {
            if before < position && length[earlier] + 1 > length[index] {
                length[index] = length[earlier] + 1;
                previous[index] = Some(earlier);
            }
        }
    }
    let Some(mut at) =
        (0..positions.len()).max_by_key(|index| (length[*index], std::cmp::Reverse(*index)))
    else {
        return Vec::new();
    };
    let mut run = vec![at];
    while let Some(before) = previous[at] {
        run.push(before);
        at = before;
    }
    run.reverse();
    run
}

/// Rows with a cell, and non-empty cells.
fn table_counts(truth: &StructureTruth) -> (usize, usize) {
    let cells = |row: &Vec<String>| {
        row.iter()
            .filter(|cell| !normalise(cell).is_empty())
            .count()
    };
    let rows = truth
        .tables
        .iter()
        .flat_map(|table| &table.rows)
        .filter(|row| cells(row) > 0)
        .count();
    let total = truth
        .tables
        .iter()
        .flat_map(|table| &table.rows)
        .map(cells)
        .sum();
    (rows, total)
}

/// Measures what the extractor returned against the structure the gold
/// gives.
pub fn measure(truth: &StructureTruth, source: &DocumentSource) -> StructureMeasure {
    let mut measure = StructureMeasure::default();
    let text = normalise(
        &source
            .pages
            .iter()
            .map(|page| page.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let lines = source
        .pages
        .iter()
        .flat_map(|page| page.text.split('\n'))
        .map(normalise)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    // Reading order.
    let snippets = truth
        .reading_order
        .iter()
        .map(|snippet| normalise(snippet))
        .filter(|snippet| !snippet.is_empty())
        .collect::<Vec<_>>();
    let positions = snippets
        .iter()
        .map(|snippet| find_bounded(&text, snippet, 0))
        .collect::<Vec<_>>();
    measure.snippets = snippets.len();
    let found = positions
        .iter()
        .enumerate()
        .filter_map(|(index, position)| position.map(|position| (index, position)))
        .collect::<Vec<_>>();
    measure.snippets_found = found.len();
    let mut in_order = vec![false; snippets.len()];
    for run_index in longest_increasing(&found.iter().map(|(_, at)| *at).collect::<Vec<_>>()) {
        in_order[found[run_index].0] = true;
        measure.snippets_in_order += 1;
    }
    for ((snippet, position), in_order) in snippets.iter().zip(&positions).zip(in_order) {
        if position.is_none() {
            measure
                .misses
                .push(format!("reading order: \"{snippet}\" not found"));
        } else if !in_order {
            measure
                .misses
                .push(format!("reading order: \"{snippet}\" out of order"));
        }
    }

    // Tables.
    for (table_index, table) in truth.tables.iter().enumerate() {
        let table = GoldTable::of(table);
        if table.rows.is_empty() {
            continue;
        }
        let (start, end) = table.region(&lines);
        let region_lines = lines.get(start..=end).unwrap_or_default();
        for (row_index, row) in table.rows.iter().enumerate() {
            measure.rows += 1;
            if region_lines
                .iter()
                .any(|line| table.holds_row(line, row_index))
            {
                measure.rows_found += 1;
            } else {
                measure.misses.push(format!(
                    "table {} row {}: \"{}{}\" not on one line of the table",
                    table_index + 1,
                    row_index + 1,
                    if row.blank_lead { "(blank) | " } else { "" },
                    row.cells.join(" | ")
                ));
            }
        }
        let region = region_lines.join("\n");
        let mut wanted: Vec<(&String, usize)> = Vec::new();
        for cell in table.rows.iter().flat_map(|row| &row.cells) {
            match wanted.iter_mut().find(|(value, _)| *value == cell) {
                Some((_, count)) => *count += 1,
                None => wanted.push((cell, 1)),
            }
        }
        for (cell, count) in wanted {
            measure.cells += count;
            let found = count_bounded(&region, cell).min(count);
            measure.cells_found += found;
            if found < count {
                measure.misses.push(format!(
                    "table {}: cell \"{cell}\" found {found} of {count} times",
                    table_index + 1
                ));
            }
        }
    }

    // Labelled values.
    let labels = truth
        .key_values
        .iter()
        .map(|pair| normalise(&pair.key))
        .filter(|key| !key.is_empty())
        .collect::<Vec<_>>();
    let labels = labels.iter().map(String::as_str).collect::<Vec<_>>();
    for pair in &truth.key_values {
        let key = normalise(&pair.key);
        let value = normalise(&pair.value);
        if value.is_empty() {
            continue;
        }
        measure.key_values += 1;
        let found = !key.is_empty()
            && (below_in_table(&lines, &key, &value)
                || lines.iter().enumerate().any(|(index, line)| {
                    let mut from = 0;
                    while let Some(at) = find_bounded(line, &key, from) {
                        let rest = &line[at + key.len()..];
                        // The value, before the next gold label on the
                        // line: one after it is that label's.
                        if find_bounded(rest, &value, 0)
                            .is_some_and(|start| !holds_within(rest, &labels, 0, start))
                        {
                            return true;
                        }
                        if only_separators(&line[..at])
                            && only_separators(rest)
                            && lines
                                .get(index + 1)
                                .is_some_and(|next| find_bounded(next, &value, 0).is_some())
                        {
                            return true;
                        }
                        from = at + key.len();
                    }
                    false
                }));
        if found {
            measure.key_values_found += 1;
        } else {
            measure
                .misses
                .push(format!("key value: \"{key}\" = \"{value}\" not found"));
        }
    }

    // Routes, when the worker sent layouts at all.
    if source.pages.iter().any(|page| page.layout.is_some()) {
        for (page_number, expected) in &truth.expected_routes {
            measure.route_pages += 1;
            let actual = source
                .pages
                .iter()
                .find(|page| page.page_number == *page_number)
                .and_then(|page| page.layout.as_ref())
                .map_or("none", |layout| route_name(layout.route));
            if actual == expected {
                measure.routes_correct += 1;
            } else {
                measure.misses.push(format!(
                    "route: page {page_number} expected {expected}, took {actual}"
                ));
            }
            measure.routes.push(RouteCheck {
                page: *page_number,
                expected: expected.clone(),
                actual: actual.to_owned(),
            });
        }
    }
    measure
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::gold::{KeyValueTruth, TableTruth};
    use intern_engine::{
        PageOrigin, SourcePage,
        structure::{PageLayout, RouteSignals},
    };

    fn source(pages: &[&str]) -> DocumentSource {
        DocumentSource::from_pages(
            pages
                .iter()
                .enumerate()
                .map(|(index, text)| SourcePage::new(index + 1, *text, PageOrigin::Native))
                .collect(),
        )
    }

    fn routed(mut source: DocumentSource, routes: &[Option<PageRoute>]) -> DocumentSource {
        for (page, route) in source.pages.iter_mut().zip(routes) {
            page.layout = route.map(|route| PageLayout {
                width: 6120,
                height: 7920,
                route,
                signals: RouteSignals::default(),
                blocks: Vec::new(),
            });
        }
        source
    }

    fn rows(rows: &[&[&str]]) -> TableTruth {
        TableTruth {
            rows: rows
                .iter()
                .map(|row| row.iter().map(|cell| (*cell).to_owned()).collect())
                .collect(),
        }
    }

    #[test]
    fn a_value_is_found_only_where_it_stands_on_its_own() {
        assert_eq!(find_bounded("Qty 14 at 4", "4", 0), Some(10));
        assert_eq!(find_bounded("1,4 and 4.5", "4", 0), None);
        assert_eq!(find_bounded("Total 2,385.00", "385.00", 0), None);
        assert_eq!(find_bounded("Total $2,385.00", "$2,385.00", 0), Some(6));
        assert_eq!(find_bounded("INV-20417", "20417", 0), Some(4));
        assert_eq!(find_bounded("Itemised", "Item", 0), None);
        assert_eq!(find_bounded("Item: Itemised", "Item", 0), Some(0));
        assert_eq!(count_bounded("1 | 1 | 11 | 1", "1"), 3);
        assert_eq!(
            normalise("\u{201c}Tenant\u{201d}\u{a0}\u{2013}  Suite\n4B"),
            "\"Tenant\" - Suite 4B"
        );
    }

    #[test]
    fn reading_order_is_the_longest_run_of_snippets_in_the_gold_order() {
        let truth = StructureTruth {
            reading_order: vec![
                "Annual meeting".into(),
                "Dock repairs".into(),
                "Slip fees".into(),
                "Winter storage".into(),
            ],
            ..StructureTruth::default()
        };
        // Columns read in order.
        let read = measure(
            &truth,
            &source(&[
                "Annual meeting set\nDock repairs begin",
                "Slip fees rise\nWinter storage",
            ]),
        );
        assert_eq!(read.reading_order_accuracy(), Some(1.0));
        // Row-interleaved: the second column's first snippet comes between
        // the first column's two, so three of the four are in order.
        let interleaved = measure(
            &truth,
            &source(&["Annual meeting set Slip fees rise\nDock repairs begin Winter storage"]),
        );
        assert_eq!(interleaved.snippets_in_order, 3);
        assert_eq!(interleaved.reading_order_accuracy(), Some(0.75));
        assert_eq!(
            interleaved.misses,
            vec!["reading order: \"Slip fees\" out of order"]
        );
        // A snippet OCR lost is out of order too.
        let lost = measure(
            &truth,
            &source(&["Annual meeting\nDock repalrs\nSlip fees\nWinter storage"]),
        );
        assert_eq!(lost.snippets_found, 3);
        assert_eq!(lost.reading_order_accuracy(), Some(0.75));
        assert_eq!(
            lost.misses,
            vec!["reading order: \"Dock repairs\" not found"]
        );
    }

    #[test]
    fn two_columns_read_the_wrong_way_round_keep_only_the_longer_in_order() {
        let first = [
            "Annual meeting",
            "Dock repairs",
            "Slip fees",
            "Winter storage",
            "Fuel dock",
            "Launch ramp",
            "Guest moorings",
        ];
        let second = [
            "Race committee",
            "Junior sailing",
            "Social calendar",
            "Club burgee",
        ];
        let truth = StructureTruth {
            reading_order: first
                .iter()
                .chain(&second)
                .map(|s| (*s).to_owned())
                .collect(),
            ..StructureTruth::default()
        };
        let read = second
            .iter()
            .chain(&first)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        let swapped = measure(&truth, &source(&[read.as_str()]));
        assert_eq!((swapped.snippets_found, swapped.snippets_in_order), (11, 7));
        // About 0.64; counted by consecutive pairs it was 9 of 10.
        assert_eq!(swapped.reading_order_accuracy(), Some(7.0 / 11.0));
        assert_eq!(
            swapped.misses,
            second
                .iter()
                .map(|snippet| format!("reading order: \"{snippet}\" out of order"))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_row_is_found_on_one_line_and_cells_within_the_table() {
        let truth = StructureTruth {
            tables: vec![rows(&[
                &["Item", "Description", "Qty", "Amount"],
                &["HK-1010", "Saucepan, 2 qt", "6", "$171.00"],
                &["HK-2201", "Half-sheet pan", "12", "$96.00"],
                &["", "Total", "", "$267.00"],
            ])],
            ..StructureTruth::default()
        };
        // A linearised table: every row on its own `| … |` line.
        let table = measure(
            &truth,
            &source(&[
                "Schedule B\n| Item | Description | Qty | Amount |\n| HK-1010 | Saucepan, 2 qt | 6 | $171.00 |\n| HK-2201 | Half-sheet pan | 12 | $96.00 |\n| | Total | | $267.00 |",
            ]),
        );
        assert_eq!((table.rows, table.rows_found), (4, 4));
        assert_eq!((table.cells, table.cells_found), (14, 14));
        assert_eq!(table.table_row_accuracy(), Some(1.0));

        // The same table read column by column: no row on one line, every
        // cell still in the table.
        let columns = measure(
            &truth,
            &source(&[
                "Item\nHK-1010\nHK-2201\nDescription\nSaucepan, 2 qt\nHalf-sheet pan\nTotal\nQty\n6\n12\nAmount\n$171.00\n$96.00\n$267.00",
            ]),
        );
        assert_eq!(columns.rows_found, 0);
        assert_eq!(columns.table_cell_recall(), Some(1.0));

        // A row whose cells are out of order on their line is not found; a
        // cell printed outside the table's region does not count.
        let jumbled = measure(
            &truth,
            &source(&[
                "Due $267.00 by May 1\n\nItem Description Qty Amount\nHK-1010 Saucepan, 2 qt 6 $171.00\n12 HK-2201 Half-sheet pan $96.00\nTotal",
            ]),
        );
        assert_eq!(jumbled.rows_found, 2, "{:?}", jumbled.misses);
        assert_eq!(jumbled.cells_found, 13, "{:?}", jumbled.misses);
    }

    #[test]
    fn a_row_holds_no_other_row_between_its_cells_and_lies_in_its_table() {
        let truth = StructureTruth {
            tables: vec![rows(&[&["Part", "Qty"], &["Bolt", "4"], &["Nut", "12"]])],
            ..StructureTruth::default()
        };
        // Two rows read across each other: "Bolt" and "4" are on one line
        // in order, but with the other row between them.
        let crossed = measure(&truth, &source(&["Part Qty\nBolt Nut 12 4"]));
        assert_eq!(crossed.rows_found, 2, "{:?}", crossed.misses);
        assert!(
            crossed.misses[0].contains("row 2: \"Bolt | 4\""),
            "{:?}",
            crossed.misses
        );
        // A row printed before the table, in a sentence, is not the row.
        let outside = measure(
            &truth,
            &source(&["Order Bolt 4 today\nPart Qty\nBolt\nNut 12"]),
        );
        assert_eq!(outside.rows_found, 2, "{:?}", outside.misses);
    }

    #[test]
    fn a_blank_mark_is_part_of_a_check_box_row() {
        let truth = StructureTruth {
            tables: vec![rows(&[
                &["", "Gold PPO"],
                &["X", "Silver PPO"],
                &["", "High-deductible HSA"],
                &["", "Waive coverage"],
            ])],
            ..StructureTruth::default()
        };
        // Three boxes to a line, the X beside the option chosen.
        let marked = measure(
            &truth,
            &source(&[
                "SECTION 3 - MEDICAL PLAN\nGold PPO X Silver PPO High-deductible HSA\nWaive coverage",
            ]),
        );
        assert_eq!(
            (
                marked.rows,
                marked.rows_found,
                marked.cells,
                marked.cells_found
            ),
            (4, 4, 5, 5),
            "{:?}",
            marked.misses
        );
        // The X beside the wrong option: that option is marked, and the
        // gold leaves it blank.
        let wrong = measure(
            &truth,
            &source(&["X Gold PPO Silver PPO High-deductible HSA\nWaive coverage"]),
        );
        // Nor is the chosen option's row found: another option stands
        // between its X and its caption.
        assert_eq!(wrong.rows_found, 2, "{:?}", wrong.misses);
        assert!(
            wrong.misses[0].contains("\"(blank) | Gold PPO\" not on one line"),
            "{:?}",
            wrong.misses
        );
        assert!(
            wrong.misses[1].contains("\"X | Silver PPO\" not on one line"),
            "{:?}",
            wrong.misses
        );
        // A second X marks an option the gold leaves blank.
        let extra = measure(
            &truth,
            &source(&["Gold PPO X Silver PPO X High-deductible HSA\nWaive coverage"]),
        );
        assert_eq!(extra.rows_found, 3, "{:?}", extra.misses);
        // The marks read apart from their options, after the table: the
        // chosen option's row is not found, and the X is not in the table.
        let apart = measure(
            &truth,
            &source(&["Gold PPO Silver PPO High-deductible HSA\nWaive coverage\nSigned\nX"]),
        );
        assert_eq!((apart.rows_found, apart.cells_found), (3, 4));
    }

    #[test]
    fn a_lone_character_neither_places_nor_widens_a_table() {
        let truth = StructureTruth {
            tables: vec![rows(&[
                &["", "No change"],
                &["X", "Dental"],
                &["X", "Vision"],
            ])],
            ..StructureTruth::default()
        };
        // Both marks drawn after the options, apart from them: an X is no
        // place to end the table, and the marks are not in it.
        let after = measure(&truth, &source(&["No change Dental Vision\nSigned\nX X"]));
        assert_eq!(
            (after.cells, after.cells_found),
            (5, 3),
            "{:?}",
            after.misses
        );
        // Nor a place to start it.
        let truth = StructureTruth {
            tables: vec![rows(&[
                &["X", "Dental"],
                &["X", "Vision"],
                &["", "No change"],
            ])],
            ..StructureTruth::default()
        };
        let before = measure(&truth, &source(&["X X\nDental Vision No change"]));
        assert_eq!(before.cells_found, 3, "{:?}", before.misses);
        let read = measure(&truth, &source(&["X Dental X Vision No change"]));
        assert_eq!(
            (read.rows_found, read.cells_found),
            (3, 5),
            "{:?}",
            read.misses
        );
    }

    #[test]
    fn a_table_split_over_two_pages_is_one_region() {
        let truth = StructureTruth {
            tables: vec![rows(&[&["Unit", "Reading"], &["A-1", "4"], &["A-2", "4"]])],
            ..StructureTruth::default()
        };
        let split = measure(
            &truth,
            &source(&["Unit Reading\nA-1 4\nPage 1 of 2", "Unit Reading\nA-2 4"]),
        );
        assert_eq!(split.table_row_accuracy(), Some(1.0));
        // "4" is printed twice and found twice; "Reading" repeated by the
        // second page's header is not needed twice.
        assert_eq!((split.cells, split.cells_found), (6, 6));
        let once = measure(&truth, &source(&["Unit Reading\nA-1 4\nA-2"]));
        assert_eq!(once.cells_found, 5);
    }

    #[test]
    fn a_value_follows_its_label_on_its_line_or_alone_on_the_next() {
        let truth = StructureTruth {
            key_values: vec![
                KeyValueTruth {
                    key: "Invoice Date".into(),
                    value: "03/04/2026".into(),
                },
                KeyValueTruth {
                    key: "BILL TO".into(),
                    value: "Quillon Ridge Bakery".into(),
                },
                KeyValueTruth {
                    key: "Due Date".into(),
                    value: "04/03/2026".into(),
                },
            ],
            ..StructureTruth::default()
        };
        let fields = measure(
            &truth,
            &source(&[
                "Invoice Date: 03/04/2026 | Due Date: 04/03/2026\nBILL TO\nQuillon Ridge Bakery, Inc.",
            ]),
        );
        assert_eq!(fields.kv_accuracy(), Some(1.0));
        // Labels in one row, values in the next: the label line holds more
        // than its label, so the values are not its.
        let header_row = measure(
            &truth,
            &source(&[
                "Invoice Date Due Date\n03/04/2026 04/03/2026\nBILL TO: Quillon Ridge Bakery",
            ]),
        );
        assert_eq!(header_row.key_values_found, 1);
        // A value before its label is not its value.
        let before = measure(&truth, &source(&["03/04/2026 Invoice Date"]));
        assert_eq!(before.key_values_found, 0);
        // Nor is one after the next label on the line: it is that label's.
        let crowded = measure(
            &truth,
            &source(&["Invoice Date Due Date 03/04/2026 04/03/2026"]),
        );
        assert_eq!(crowded.key_values_found, 1, "{:?}", crowded.misses);
        assert!(
            crowded.misses[0].contains("\"Invoice Date\" = \"03/04/2026\""),
            "{:?}",
            crowded.misses
        );
        // Labels over their values, linearised as a table: each value is
        // in the cell under its label, and only there.
        let grid = measure(
            &truth,
            &source(&[
                "| Invoice Date | Due Date |\n| 03/04/2026 | 04/03/2026 |\n| BILL TO |\n| Quillon Ridge Bakery, Inc. |",
            ]),
        );
        assert_eq!(grid.kv_accuracy(), Some(1.0), "{:?}", grid.misses);
        let shifted = measure(
            &truth,
            &source(&["| Invoice Date | Due Date |\n| 04/03/2026 | 03/04/2026 |"]),
        );
        assert_eq!(shifted.key_values_found, 0);
        // A label over its value is a whole cell: the date under "CONTRACT
        // DATE" is not the date under "DATE".
        let truth = StructureTruth {
            key_values: vec![KeyValueTruth {
                key: "DATE".into(),
                value: "05/07/2026".into(),
            }],
            ..StructureTruth::default()
        };
        let contract = measure(
            &truth,
            &source(&["| CHANGE ORDER NUMBER | CONTRACT DATE |\n| 006 | 05/07/2026 |"]),
        );
        assert_eq!(contract.key_values_found, 0);
        let own = measure(
            &truth,
            &source(&["| CHANGE ORDER NUMBER | DATE: |\n| 006 | 05/07/2026 |"]),
        );
        assert_eq!(own.key_values_found, 1);
    }

    #[test]
    fn routes_are_judged_only_when_the_worker_sent_layouts() {
        let truth = StructureTruth {
            expected_routes: BTreeMap::from([(1, "layout".to_owned()), (2, "ocr".to_owned())]),
            ..StructureTruth::default()
        };
        let plain = source(&["a", "b"]);
        assert_eq!(measure(&truth, &plain).route_correct(), None);
        assert_eq!(route_class(&plain), "unrouted");
        let both = routed(
            source(&["a", "b"]),
            &[Some(PageRoute::Fast), Some(PageRoute::Ocr)],
        );
        let judged = measure(&truth, &both);
        assert_eq!(judged.route_correct(), Some(0.5));
        assert_eq!(
            judged.routes[0],
            RouteCheck {
                page: 1,
                expected: "layout".into(),
                actual: "fast".into()
            }
        );
        assert_eq!(judged.routes[1].actual, "ocr");
        assert_eq!(
            judged.misses,
            vec!["route: page 1 expected layout, took fast"]
        );
        assert_eq!(route_class(&both), "ocr");
        assert_eq!(page_routes(&both), vec!["fast", "ocr"]);
        let partly = routed(source(&["a", "b"]), &[Some(PageRoute::Layout), None]);
        assert_eq!(measure(&truth, &partly).route_correct(), Some(0.5));
        assert_eq!(route_class(&partly), "layout");
        assert_eq!(page_routes(&partly), vec!["layout", "none"]);
    }

    #[test]
    fn a_failed_extraction_misses_everything_and_pools_by_item() {
        let truth = StructureTruth {
            reading_order: vec!["a".into(), "b".into(), "c".into()],
            tables: vec![rows(&[&["x", ""], &["y", "z"]])],
            key_values: vec![KeyValueTruth {
                key: "k".into(),
                value: "v".into(),
            }],
            expected_routes: BTreeMap::from([(1, "fast".to_owned())]),
        };
        let unread = StructureMeasure::unread(&truth);
        assert!(unread.failed);
        assert_eq!(
            unread.scores().map(|(_, value)| value),
            [Some(0.0), Some(0.0), Some(0.0), Some(0.0), Some(0.0)]
        );
        assert_eq!((unread.snippets, unread.rows, unread.cells), (3, 2, 3));
        let mut pooled = measure(&truth, &source(&["a b c\nx\ny z\nk: v"]));
        assert_eq!(
            pooled.scores().map(|(_, value)| value),
            [Some(1.0), Some(1.0), Some(1.0), Some(1.0), None]
        );
        pooled.accumulate(&unread);
        assert_eq!(pooled.reading_order_accuracy(), Some(0.5));
        assert_eq!(pooled.table_cell_recall(), Some(0.5));
        assert_eq!(pooled.route_correct(), Some(0.0));
    }
}
