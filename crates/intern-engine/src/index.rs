//! The evidence index: a document as the units evidence can be retrieved
//! by, each with what it says about the document already read off it.
//!
//! A unit is a heading, a paragraph (or a sentence-bounded chunk of a long
//! one), a list item, one row of a table, or one labelled value. Units come
//! from [`structured`], so the worker's layout is used where a page has
//! one and the worker's own text segmentation where it does not. Every
//! unit keeps its block's id (`p3.b7`), its own id (`p3.b7.r4`, `p3.b7.f2`,
//! `p3.b7.s3`), its page, its kind, its verbatim text, the headings it
//! falls under, the header row of its table, where its text came from and
//! how sure OCR was of it.
//!
//! Features are computed once, when the index is built:
//!
//! * **Dates**, with the wording around each read by the very functions
//!   validation reads it with ([`window_before`], [`role_from_wording`],
//!   [`labels_a_deadline`], [`labels_the_issue_date`],
//!   [`reference_introduced`]), so retrieval and validation agree on what a
//!   date means.
//! * **Money**, **likely organisations** and **likely people**, and
//!   **identifiers**.
//! * **Cue counts** from the cue lists distillation and inference use
//!   ([`crate::cues`]).
//! * **Lexical terms** for BM25, and where the unit sits: its page, the
//!   opening or closing of the document, a letterhead, a signature block,
//!   and what its section is about.
//!
//! Everything is an integer and every collection is ordered, so the same
//! document always builds the same index. Unit text is never rewritten:
//! each unit's text is a piece of its page's text, which is what lets a
//! cited unit be checked against the document.

use std::collections::{BTreeMap, BTreeSet};

use crate::cues::{
    AMOUNT_LABELS, BOILERPLATE_CUES, CUSTOMER_CUES, DATE_ROLE_CUES, HONORIFICS, IDENTIFIER_LABELS,
    ISSUER_CUES, NOT_A_TITLE, ORGANISATION_ENDINGS, PARTY_CUES, PARTY_LABELS, SECTION_TAG_WORDS,
    SIGNATURE_CUES, STOPWORDS, SUBJECT_CUES, TYPE_CUES, TYPE_NOUNS,
};
use crate::domain::{DateRole, DocumentSource, PageOrigin};
use crate::engine::estimated_tokens;
use crate::evidence::{NumericOrder, normalize, normalize_loosely, numeric_date_order_of};
use crate::infer::{
    dates_stated_on, labels_a_deadline, labels_the_issue_date, role_from_wording, window_before,
    wrapped_lines,
};
use crate::structure::{BlockKind, LayoutBlock, TextSource, structured};
use crate::text::{
    contains_identifier, count_cues, date_signal_count, digit_masked, split_sentences,
};
use crate::validate::{reference_introduced, rfind_word};

/// Paragraphs longer than this are split into sentence-bounded chunks, each
/// its own unit, unless the index is built with another size.
pub const DEFAULT_MAX_UNIT_CHARACTERS: usize = 400;
/// A repeated line must be this short to read as a running header or
/// footer, as in distillation.
const MAX_RUNNING_CHARACTERS: usize = 120;
/// Units from the start of the document that are its opening, and from the
/// end that are its closing, as distillation counts blocks.
const OPENING_UNITS: u32 = 4;
const CLOSING_UNITS: u32 = 3;
/// How many units after a signature cue still read as the signature block.
const SIGNATURE_REACH: u32 = 12;
/// The legal forms among [`ORGANISATION_ENDINGS`].
const LEGAL_FORMS: &[&str] = &[
    "inc",
    "incorporated",
    "llc",
    "l.l.c",
    "ltd",
    "limited",
    "corp",
    "co",
    "llp",
    "l.l.p",
    "lp",
    "l.p",
    "pllc",
    "plc",
    "pc",
    "p.c",
    "n.a",
    "gmbh",
    "ag",
    "s.a",
    "n.v",
    "b.v",
    "pty",
];

/// Section tags: what a unit's headings say its section is about.
pub mod tag {
    pub const DEFINITIONS: u16 = 1;
    pub const TERM: u16 = 2;
    pub const PARTIES: u16 = 4;
    pub const RECITALS: u16 = 8;
    pub const INVOICE: u16 = 16;
    pub const BILL_TO: u16 = 32;
    pub const SIGNATURE: u16 = 64;
    pub const NOTICES: u16 = 128;
    pub const SCHEDULE: u16 = 256;
    pub const PAYMENT: u16 = 512;
    pub const SCOPE: u16 = 1024;
}

/// What kind of thing a unit is.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UnitKind {
    Heading,
    Paragraph,
    ListItem,
    TableHeader,
    TableRow,
    Field,
    Caption,
    RunningHeader,
    RunningFooter,
    Other,
}

impl UnitKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Heading => "heading",
            Self::Paragraph => "paragraph",
            Self::ListItem => "list_item",
            Self::TableHeader => "table_header",
            Self::TableRow => "table_row",
            Self::Field => "field",
            Self::Caption => "caption",
            Self::RunningHeader => "running_header",
            Self::RunningFooter => "running_footer",
            Self::Other => "other",
        }
    }
}

/// One statement of a date in a unit, and what the words before it say.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateMention {
    /// The date, ISO 8601.
    pub iso: String,
    /// Byte offset in the normalized wrapped line the date stands on.
    pub at: usize,
    /// Written only in numbers that could be read either way round.
    pub numeric: bool,
    /// The role the wording names (`Invoice Date:`, `effective as of`).
    pub role: Option<DateRole>,
    /// Labelled due, expiry, renewal or deadline.
    pub deadline: bool,
    /// Labelled the date the document was issued (`Invoice Date:`, `Dated`).
    pub issue_label: bool,
    /// Introduced as another document's date ("the Agreement dated").
    pub reference: bool,
    /// The definition of a dated term: `"Closing Date" means June 12, 2026`.
    pub defined_term: bool,
    /// For a date in a table row, what its column's header says, read as
    /// the date's label: `| Invoice Date | Due Date |` over `| 03/04/2026 |
    /// 04/03/2026 |` names the first an issue date and the second a
    /// deadline, which the words before each date on its own line cannot.
    pub column_role: Option<DateRole>,
    pub column_deadline: bool,
    pub column_issue: bool,
}

/// A value that looks like a document's own number.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    pub value: String,
    /// The label it stands under, when one says it is a number.
    pub label: Option<String>,
}

/// Counts of the cue words a unit carries, by list.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CueCounts {
    pub type_cues: u8,
    pub type_nouns: u8,
    pub party_cues: u8,
    pub date_role_cues: u8,
    pub subject_cues: u8,
    pub signature_cues: u8,
    pub boilerplate_cues: u8,
    pub issuer_cues: u8,
    pub customer_cues: u8,
    /// The unit opens with a part heading ("EXHIBIT A", "SCHEDULE 2").
    pub not_a_title: bool,
    /// "This <document noun> ..." - the document naming itself.
    pub self_naming: bool,
    /// A defined-term parenthesis naming a role: `("Tenant")`.
    pub defined_role: bool,
}

/// Where a unit sits in the document.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Position {
    pub first_page: bool,
    pub last_page: bool,
    /// Among the first units of the document.
    pub opening: bool,
    /// Among the last units of the document.
    pub closing: bool,
    /// Its place among the units of its page, from 0.
    pub page_index: u16,
    /// Among the first three units of the first page, or that page's header.
    pub letterhead: bool,
    /// In or just after a signature block.
    pub signature: bool,
    /// One of the first two headings of the document, where a title is.
    pub title: bool,
    /// The first paragraph of the document.
    pub first_paragraph: bool,
    /// In the first tenth of the document (and at least its first thirty
    /// units): the cover, the preamble, the recitals.
    pub early: bool,
}

/// Everything read off a unit when the index is built.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnitFeatures {
    pub dates: Vec<DateMention>,
    /// Date-shaped spans, counted the generous way distillation counts them.
    pub date_signals: u8,
    /// Amounts of money as written: `$4,805.98`, `USD 1,200`.
    pub money: Vec<String>,
    /// An amount under a label that makes it a key fact (`Total`, `Rent`).
    pub money_labelled: bool,
    /// Likely organisations, loosely normalized.
    pub organisations: Vec<String>,
    /// Likely people, loosely normalized.
    pub people: Vec<String>,
    /// Runs of two or more capitalised words: names of some kind.
    pub proper_names: u8,
    pub identifiers: Vec<Identifier>,
    pub cues: CueCounts,
    /// BM25 terms as (term id, weight in halves): a word of the unit's own
    /// text counts 2, a word of its heading or table header 1.
    pub terms: Vec<(u32, u16)>,
    /// The sum of the term weights, in halves.
    pub length: u32,
    /// What the unit costs the model as a prompt line, by the engine's
    /// estimate (its text, whitespace collapsed).
    pub tokens: u32,
    pub position: Position,
    /// Section tags of the unit's headings, and of itself for a heading.
    pub section_tags: u16,
    /// The terms the unit defines, folded: `services` for `"Services"
    /// means ...`.
    pub defines: Vec<String>,
    /// The unit is a date standing alone, or with a word or two: a
    /// letter's dateline, a form's "Date:" box.
    pub dateline: bool,
}

/// One unit of evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceUnit {
    /// `p3.b7`, `p3.b7.r4`, `p3.b7.f2`, `p3.b7.s3`; unique in the index.
    pub id: String,
    /// Document order, from 0.
    pub ordinal: u32,
    pub page: usize,
    /// The id of the block the unit comes from.
    pub block_id: String,
    /// The block's place in the document, from 0.
    pub block: u32,
    pub kind: UnitKind,
    /// Verbatim: the block's text, a chunk of it, a table row's `| a | b |`
    /// line, or a `Key: value` line.
    pub text: String,
    /// [`normalize`]d text.
    pub normalized: String,
    /// A labelled value's label, or the first cell of a two-cell row.
    pub label: Option<String>,
    /// A heading's level, where it can be told.
    pub level: Option<u8>,
    /// The ordinals of the heading units the unit falls under, outermost
    /// first.
    pub section_path: Vec<u32>,
    /// The ordinal of the header row of a table row's table.
    pub table_header: Option<u32>,
    pub source: TextSource,
    /// OCR confidence, 0-100, where the reader gave one.
    pub confidence: Option<u8>,
    /// A repeat of a running header or footer, or of a line that recurs on
    /// most pages. The first instance is not running.
    pub running: bool,
    pub features: UnitFeatures,
}

/// A stretch of units under one top-level heading - or, before the first
/// heading or in a document without any, one page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Section {
    /// The heading unit that opens it, if any.
    pub heading: Option<u32>,
    pub first_unit: u32,
    /// Inclusive.
    pub last_unit: u32,
    pub first_page: usize,
    pub last_page: usize,
    pub tags: u16,
}

impl Section {
    pub fn units(&self) -> std::ops::RangeInclusive<u32> {
        self.first_unit..=self.last_unit
    }
}

/// How an index is built.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexOptions {
    /// Paragraphs longer than this many characters are split into
    /// sentence-bounded chunks.
    pub max_unit_characters: usize,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self {
            max_unit_characters: DEFAULT_MAX_UNIT_CHARACTERS,
        }
    }
}

/// A document as units of evidence, with what each says already read off.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceIndex {
    units: Vec<EvidenceUnit>,
    by_id: BTreeMap<String, u32>,
    sections: Vec<Section>,
    vocabulary: BTreeMap<String, u32>,
    document_frequency: Vec<u32>,
    /// Units BM25 counts: every unit that is not running.
    indexed_units: u32,
    /// Their summed lengths, in halves.
    total_length: u64,
    numeric_order: Option<NumericOrder>,
    headings: Vec<u32>,
    page_count: usize,
    options: IndexOptions,
}

impl EvidenceIndex {
    /// The index of a document, units split at the default size.
    pub fn build(source: &DocumentSource) -> Self {
        Self::build_with(source, IndexOptions::default())
    }

    pub fn build_with(source: &DocumentSource, options: IndexOptions) -> Self {
        let mut builder = Builder::new(options);
        builder.read(source);
        builder.finish(source)
    }

    pub fn units(&self) -> &[EvidenceUnit] {
        &self.units
    }

    pub fn unit(&self, id: &str) -> Option<&EvidenceUnit> {
        self.by_id
            .get(id)
            .and_then(|ordinal| self.units.get(*ordinal as usize))
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// The heading units, in document order.
    pub fn headings(&self) -> &[u32] {
        &self.headings
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }

    pub fn options(&self) -> IndexOptions {
        self.options
    }

    /// The order the document writes its numeric dates in, when it settles
    /// one.
    pub fn numeric_order(&self) -> Option<NumericOrder> {
        self.numeric_order
    }

    /// The texts of the headings a unit falls under, outermost first.
    pub fn heading_context(&self, ordinal: u32) -> Vec<&str> {
        self.units
            .get(ordinal as usize)
            .map(|unit| {
                unit.section_path
                    .iter()
                    .filter_map(|heading| self.units.get(*heading as usize))
                    .map(|heading| heading.text.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The id a term has in this index, after the same folding and stemming
    /// units' text gets.
    pub fn term(&self, word: &str) -> Option<u32> {
        let folded = normalize(word);
        let stemmed = stem(&folded);
        self.vocabulary.get(&stemmed).copied()
    }

    /// The id of a term already folded and stemmed, as [`terms_of`] gives
    /// it.
    pub fn term_id(&self, term: &str) -> Option<u32> {
        self.vocabulary.get(term).copied()
    }

    /// How many indexed units carry the term.
    pub fn document_frequency(&self, term: u32) -> u32 {
        self.document_frequency
            .get(term as usize)
            .copied()
            .unwrap_or(0)
    }

    pub fn indexed_units(&self) -> u32 {
        self.indexed_units
    }

    pub fn total_length(&self) -> u64 {
        self.total_length
    }

    /// The section a unit belongs to.
    pub fn section_of(&self, ordinal: u32) -> Option<usize> {
        self.sections
            .iter()
            .position(|section| section.units().contains(&ordinal))
    }
}

/// A unit before its features are read.
struct Draft {
    id: String,
    page: usize,
    block_id: String,
    block: u32,
    kind: UnitKind,
    text: String,
    label: Option<String>,
    level: Option<u8>,
    /// The id of the heading block the unit's block falls under.
    section: Option<String>,
    /// For a heading unit, its own block id: other units point at it.
    heading_id: Option<String>,
    table_header: Option<u32>,
    source: TextSource,
    confidence: Option<u8>,
    /// From a block the worker marked as a page header or footer.
    marked_running: bool,
    /// A table header's cells, which label the dates in a row's columns.
    cells: Vec<String>,
}

struct Builder {
    options: IndexOptions,
    drafts: Vec<Draft>,
    used_ids: BTreeSet<String>,
    blocks: u32,
}

impl Builder {
    fn new(options: IndexOptions) -> Self {
        Self {
            options,
            drafts: Vec::new(),
            used_ids: BTreeSet::new(),
            blocks: 0,
        }
    }

    fn read(&mut self, source: &DocumentSource) {
        let document = structured(source);
        for (page, structured_page) in source.pages.iter().zip(&document.pages) {
            let page_confidence = (page.origin == PageOrigin::Ocr)
                .then(|| page.ocr_confidence.map(|value| value.min(100) as u8))
                .flatten();
            for (index, block) in structured_page.blocks.iter().enumerate() {
                let block_id = if block.id.trim().is_empty() {
                    format!("p{}.b{}", structured_page.page_number, index + 1)
                } else {
                    block.id.clone()
                };
                let confidence = block.confidence.or(if block.source == TextSource::Ocr {
                    page_confidence
                } else {
                    None
                });
                self.block(structured_page.page_number, &block_id, block, confidence);
                self.blocks += 1;
            }
        }
    }

    /// A unit id not used yet: the id itself, or with an `x{n}` suffix.
    fn unique(&mut self, id: String) -> String {
        if self.used_ids.insert(id.clone()) {
            return id;
        }
        let mut suffix = 2;
        loop {
            let candidate = format!("{id}x{suffix}");
            if self.used_ids.insert(candidate.clone()) {
                return candidate;
            }
            suffix += 1;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        page: usize,
        block_id: &str,
        block: &LayoutBlock,
        id: String,
        kind: UnitKind,
        text: &str,
        label: Option<String>,
        table_header: Option<u32>,
        confidence: Option<u8>,
    ) -> Option<u32> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let id = self.unique(id);
        let ordinal = self.drafts.len() as u32;
        self.drafts.push(Draft {
            id,
            page,
            block_id: block_id.to_owned(),
            block: self.blocks,
            kind,
            text: text.to_owned(),
            label,
            level: if kind == UnitKind::Heading {
                block.level
            } else {
                None
            },
            section: block.section.clone(),
            heading_id: (kind == UnitKind::Heading).then(|| block_id.to_owned()),
            table_header,
            source: block.source,
            confidence,
            marked_running: matches!(block.kind, BlockKind::PageHeader | BlockKind::PageFooter),
            cells: Vec::new(),
        });
        Some(ordinal)
    }

    fn block(&mut self, page: usize, block_id: &str, block: &LayoutBlock, confidence: Option<u8>) {
        match block.kind {
            BlockKind::Table => self.table(page, block_id, block, confidence),
            BlockKind::KeyValue if !block.fields.is_empty() => {
                self.fields(page, block_id, block, confidence)
            }
            kind => {
                let unit_kind = match kind {
                    BlockKind::Heading => UnitKind::Heading,
                    BlockKind::ListItem => UnitKind::ListItem,
                    BlockKind::Caption => UnitKind::Caption,
                    BlockKind::PageHeader => UnitKind::RunningHeader,
                    BlockKind::PageFooter => UnitKind::RunningFooter,
                    BlockKind::Other => UnitKind::Other,
                    _ => UnitKind::Paragraph,
                };
                self.chunks(page, block_id, block, unit_kind, confidence);
            }
        }
    }

    /// A block as one unit, or as sentence-bounded chunks when it is long.
    fn chunks(
        &mut self,
        page: usize,
        block_id: &str,
        block: &LayoutBlock,
        kind: UnitKind,
        confidence: Option<u8>,
    ) {
        let chunks = split_sentences(&block.text, self.options.max_unit_characters.max(1))
            .into_iter()
            .filter(|chunk| !chunk.trim().is_empty())
            .collect::<Vec<_>>();
        if chunks.len() <= 1 {
            self.push(
                page,
                block_id,
                block,
                block_id.to_owned(),
                kind,
                &block.text,
                None,
                None,
                confidence,
            );
            return;
        }
        for (index, chunk) in chunks.iter().enumerate() {
            self.push(
                page,
                block_id,
                block,
                format!("{block_id}.s{}", index + 1),
                kind,
                chunk,
                None,
                None,
                confidence,
            );
        }
    }

    /// A table as one unit per row, each with the `| a | b |` line it is in
    /// the page's text. The row under which every cell is a header is the
    /// header row; a table with none whose first row holds no figures reads
    /// its first row as the header.
    fn table(&mut self, page: usize, block_id: &str, block: &LayoutBlock, confidence: Option<u8>) {
        let rows = block
            .table
            .as_ref()
            .map(|table| table.rows.as_slice())
            .unwrap_or_default();
        let lines = block
            .text
            .lines()
            .filter(|line| !line.trim().is_empty() && !is_separator_line(line))
            .collect::<Vec<_>>();
        if rows.is_empty() || lines.len() != rows.len() {
            self.chunks(page, block_id, block, UnitKind::TableRow, confidence);
            return;
        }
        let marked = rows
            .iter()
            .map(|row| !row.cells.is_empty() && row.cells.iter().all(|cell| cell.header))
            .collect::<Vec<_>>();
        let implicit_header = !marked.iter().any(|header| *header)
            && rows.len() >= 2
            && rows[0].cells.len() >= 2
            && rows[0].cells.iter().all(|cell| {
                !cell
                    .text
                    .chars()
                    .any(|character| character.is_ascii_digit())
            })
            && rows[0]
                .cells
                .iter()
                .any(|cell| !cell.text.trim().is_empty());
        let mut header: Option<u32> = None;
        for (index, (row, line)) in rows.iter().zip(&lines).enumerate() {
            let is_header = marked[index] || (implicit_header && index == 0);
            let id = if row.id.trim().is_empty() {
                format!("{block_id}.r{}", index + 1)
            } else {
                row.id.clone()
            };
            let label = (row.cells.len() == 2)
                .then(|| row.cells[0].text.trim().to_owned())
                .filter(|label| !label.is_empty());
            let kind = if is_header {
                UnitKind::TableHeader
            } else {
                UnitKind::TableRow
            };
            let pushed = self.push(
                page,
                block_id,
                block,
                id,
                kind,
                line,
                label,
                if is_header { None } else { header },
                confidence,
            );
            if let Some(ordinal) = pushed {
                self.drafts[ordinal as usize].cells =
                    row.cells.iter().map(|cell| cell.text.clone()).collect();
            }
            if is_header && pushed.is_some() {
                header = pushed;
            }
        }
    }

    /// A block of labelled values as one unit per value: its `Key: value`
    /// line, labelled with the key. A line that is no field's - a label
    /// with nothing after it - goes with the field before it, or the first.
    fn fields(&mut self, page: usize, block_id: &str, block: &LayoutBlock, confidence: Option<u8>) {
        let lines = block
            .text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        // Which line each field is on, in order.
        let mut owner: Vec<Option<usize>> = vec![None; lines.len()];
        let mut next_line = 0;
        for (field_index, field) in block.fields.iter().enumerate() {
            let found = (next_line..lines.len()).find(|&line| {
                let line = lines[line].trim();
                line.starts_with(field.key.trim()) && line.contains(field.value.trim())
            });
            let Some(line) = found else {
                // The text does not hold the fields line by line: read the
                // block as a paragraph rather than invent lines.
                self.chunks(page, block_id, block, UnitKind::Paragraph, confidence);
                return;
            };
            owner[line] = Some(field_index);
            next_line = line + 1;
        }
        // Lines no field claims join the field above them.
        let mut groups: Vec<(usize, Vec<&str>)> = Vec::new();
        for (line, text) in lines.iter().enumerate() {
            match owner[line] {
                Some(field) => groups.push((field, vec![*text])),
                None => match groups.last_mut() {
                    Some((_, members)) => members.push(*text),
                    None => groups.push((usize::MAX, vec![*text])),
                },
            }
        }
        // Leading orphan lines join the first field.
        if groups.len() > 1 && groups[0].0 == usize::MAX {
            let orphans = groups.remove(0).1;
            let mut members = orphans;
            members.append(&mut groups[0].1);
            groups[0].1 = members;
        }
        for (field_index, members) in groups {
            let text = members.join("\n");
            let field = block.fields.get(field_index);
            let id = match field {
                Some(field) if !field.id.trim().is_empty() => field.id.clone(),
                Some(_) => format!("{block_id}.f{}", field_index + 1),
                None => block_id.to_owned(),
            };
            let label = field.map(|field| field.key.trim().to_owned());
            self.push(
                page,
                block_id,
                block,
                id,
                UnitKind::Field,
                &text,
                label,
                None,
                confidence,
            );
        }
    }

    fn finish(self, source: &DocumentSource) -> EvidenceIndex {
        let Builder {
            options, drafts, ..
        } = self;
        let page_count = source.pages.len();
        let first_page = source.pages.first().map_or(0, |page| page.page_number);
        let last_page = source.pages.last().map_or(0, |page| page.page_number);
        let running = running_units(&drafts, page_count);

        // Headings by block id, first instance wins.
        let mut heading_ordinals: BTreeMap<&str, u32> = BTreeMap::new();
        for (ordinal, draft) in drafts.iter().enumerate() {
            if let Some(id) = &draft.heading_id {
                heading_ordinals
                    .entry(id.as_str())
                    .or_insert(ordinal as u32);
            }
        }
        // The heading a heading falls under, by ordinal.
        let parent_of = |ordinal: u32| -> Option<u32> {
            let draft = &drafts[ordinal as usize];
            draft
                .section
                .as_deref()
                .and_then(|id| heading_ordinals.get(id).copied())
                .filter(|parent| *parent < ordinal)
        };
        let section_paths = drafts
            .iter()
            .enumerate()
            .map(|(ordinal, draft)| {
                let mut path = Vec::new();
                let mut current = if draft.kind == UnitKind::Heading {
                    parent_of(ordinal as u32)
                } else {
                    draft
                        .section
                        .as_deref()
                        .and_then(|id| heading_ordinals.get(id).copied())
                        .filter(|heading| *heading < ordinal as u32)
                };
                while let Some(heading) = current {
                    if path.contains(&heading) || path.len() > 16 {
                        break;
                    }
                    path.push(heading);
                    current = parent_of(heading);
                }
                path.reverse();
                path
            })
            .collect::<Vec<_>>();

        let normalized = drafts
            .iter()
            .map(|draft| normalize(&draft.text))
            .collect::<Vec<_>>();
        let numeric_order = numeric_date_order_of(drafts.iter().map(|draft| draft.text.as_str()));

        // Positions over the units that are not running.
        let live = (0..drafts.len())
            .filter(|ordinal| !running[*ordinal])
            .collect::<Vec<_>>();
        let early = live
            .iter()
            .take((live.len() / 10).max(30))
            .copied()
            .collect::<BTreeSet<_>>();
        let opening = live
            .iter()
            .take(OPENING_UNITS as usize)
            .copied()
            .collect::<BTreeSet<_>>();
        let closing = live
            .iter()
            .rev()
            .take(CLOSING_UNITS as usize)
            .copied()
            .collect::<BTreeSet<_>>();
        let titles = drafts
            .iter()
            .enumerate()
            .filter(|(ordinal, draft)| draft.kind == UnitKind::Heading && !running[*ordinal])
            .map(|(ordinal, _)| ordinal)
            .take(2)
            .collect::<BTreeSet<_>>();
        let first_paragraph = drafts
            .iter()
            .enumerate()
            .find(|(ordinal, draft)| {
                matches!(draft.kind, UnitKind::Paragraph | UnitKind::ListItem)
                    && !running[*ordinal]
                    && draft.text.chars().count() >= 40
            })
            .map(|(ordinal, _)| ordinal);

        let mut vocabulary: BTreeMap<String, u32> = BTreeMap::new();
        let mut units: Vec<EvidenceUnit> = Vec::with_capacity(drafts.len());
        let mut page_index: BTreeMap<usize, u16> = BTreeMap::new();
        let mut signature_left = 0_u32;
        for (ordinal, draft) in drafts.iter().enumerate() {
            let text_normalized = &normalized[ordinal];
            let heading_texts = section_paths[ordinal]
                .iter()
                .map(|heading| normalized[*heading as usize].as_str())
                .collect::<Vec<_>>();
            let header_text = draft
                .table_header
                .map(|header| normalized[header as usize].as_str());
            let mut tags = heading_texts
                .iter()
                .fold(0_u16, |tags, heading| tags | section_tags(heading));
            if draft.kind == UnitKind::Heading {
                tags |= section_tags(text_normalized);
            }

            let mut features = UnitFeatures {
                dates: date_mentions(
                    &draft.text,
                    numeric_order,
                    draft
                        .table_header
                        .map(|header| drafts[header as usize].cells.as_slice()),
                ),
                date_signals: saturate(date_signal_count(&draft.text)),
                section_tags: tags,
                ..UnitFeatures::default()
            };
            features.cues = cue_counts(&draft.text, text_normalized);
            features.defines = defined_terms(text_normalized);
            features.dateline = !features.dates.is_empty()
                && draft.text.chars().count() <= 40
                && features
                    .dates
                    .iter()
                    .all(|mention| mention.iso == features.dates[0].iso);
            let label = draft.label.as_deref().map(normalize);
            features.money = money_amounts(&draft.text);
            features.money_labelled = !features.money.is_empty()
                && AMOUNT_LABELS.iter().any(|word| {
                    label
                        .as_deref()
                        .is_some_and(|label| contains_word(label, word))
                        || contains_word(text_normalized, word)
                        || header_text.is_some_and(|header| contains_word(header, word))
                });
            features.organisations = organisations(&draft.text);
            features.people = people(&draft.text, label.as_deref());
            features.proper_names = saturate(proper_name_runs(&draft.text));
            features.identifiers = identifiers(&draft.text, label.as_deref());

            // Terms: the unit's own words count 2, its innermost heading's
            // and its table header's words 1.
            let mut weights: BTreeMap<u32, u16> = BTreeMap::new();
            let mut add = |text: &str, weight: u16, vocabulary: &mut BTreeMap<String, u32>| {
                for term in terms_of(text) {
                    let next = vocabulary.len() as u32;
                    let id = *vocabulary.entry(term).or_insert(next);
                    let slot = weights.entry(id).or_insert(0);
                    *slot = slot.saturating_add(weight);
                }
            };
            add(text_normalized, 2, &mut vocabulary);
            if let Some(heading) = heading_texts.last().copied() {
                add(heading, 1, &mut vocabulary);
            }
            if let Some(header) = header_text {
                add(header, 1, &mut vocabulary);
            }
            features.length = weights.values().map(|weight| u32::from(*weight)).sum();
            features.terms = weights.into_iter().collect();
            features.tokens = estimated_tokens(&collapse_whitespace(&draft.text)) as u32;

            let is_running = running[ordinal];
            let index_on_page = if is_running {
                u16::MAX
            } else {
                let slot = page_index.entry(draft.page).or_insert(0);
                let index = *slot;
                *slot = slot.saturating_add(1);
                index
            };
            let strong_signature = text_normalized.contains("in witness whereof")
                || (draft.kind == UnitKind::Heading && tags & tag::SIGNATURE != 0)
                || text_normalized.contains("signature page");
            if strong_signature {
                signature_left = SIGNATURE_REACH;
            }
            let signed_field = draft.label.as_deref().is_some_and(|label| {
                matches!(
                    normalize(label).as_str(),
                    "by" | "name" | "title" | "signature" | "signed" | "date signed"
                )
            });
            let signature = signature_left > 0
                || features.cues.signature_cues >= 2
                || signed_field
                || tags & tag::SIGNATURE != 0;
            signature_left = signature_left.saturating_sub(1);
            features.position = Position {
                first_page: draft.page == first_page,
                last_page: draft.page == last_page,
                opening: opening.contains(&ordinal),
                closing: closing.contains(&ordinal),
                page_index: index_on_page,
                letterhead: draft.page == first_page
                    && !is_running
                    && (index_on_page < 3 || draft.kind == UnitKind::RunningHeader),
                signature,
                title: titles.contains(&ordinal),
                first_paragraph: first_paragraph == Some(ordinal),
                early: early.contains(&ordinal),
            };

            units.push(EvidenceUnit {
                id: draft.id.clone(),
                ordinal: ordinal as u32,
                page: draft.page,
                block_id: draft.block_id.clone(),
                block: draft.block,
                kind: draft.kind,
                text: draft.text.clone(),
                normalized: text_normalized.clone(),
                label: draft.label.clone(),
                level: draft.level,
                section_path: section_paths[ordinal].clone(),
                table_header: draft.table_header,
                source: draft.source,
                confidence: draft.confidence,
                running: is_running,
                features,
            });
        }

        let mut document_frequency = vec![0_u32; vocabulary.len()];
        let mut indexed_units = 0_u32;
        let mut total_length = 0_u64;
        for unit in units.iter().filter(|unit| !unit.running) {
            indexed_units += 1;
            total_length += u64::from(unit.features.length);
            for (term, _) in &unit.features.terms {
                document_frequency[*term as usize] += 1;
            }
        }
        let by_id = units
            .iter()
            .map(|unit| (unit.id.clone(), unit.ordinal))
            .collect::<BTreeMap<_, _>>();
        let headings = units
            .iter()
            .filter(|unit| unit.kind == UnitKind::Heading)
            .map(|unit| unit.ordinal)
            .collect::<Vec<_>>();
        let sections = sections_of(&units);
        EvidenceIndex {
            units,
            by_id,
            sections,
            vocabulary,
            document_frequency,
            indexed_units,
            total_length,
            numeric_order,
            headings,
            page_count,
            options,
        }
    }
}

/// Which drafts are running: a repeat of a page header or footer the
/// worker marked, or - as distillation collapses them - a short line whose
/// shape, digits masked, recurs on at least half the pages (and three) of a
/// document of three pages or more. The first instance of each is kept.
fn running_units(drafts: &[Draft], page_count: usize) -> Vec<bool> {
    let shape = |draft: &Draft| digit_masked(draft.text.trim());
    let eligible = |draft: &Draft| {
        draft.text.chars().count() <= MAX_RUNNING_CHARACTERS
            && !matches!(draft.kind, UnitKind::TableRow | UnitKind::TableHeader)
    };
    let mut recurring: BTreeSet<String> = BTreeSet::new();
    if page_count >= 3 {
        let mut pages_by_shape: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for draft in drafts.iter().filter(|draft| eligible(draft)) {
            pages_by_shape
                .entry(shape(draft))
                .or_default()
                .insert(draft.page);
        }
        let threshold = (page_count / 2).max(3);
        recurring = pages_by_shape
            .into_iter()
            .filter(|(_, pages)| pages.len() >= threshold)
            .map(|(shape, _)| shape)
            .collect();
    }
    let mut seen: BTreeSet<String> = BTreeSet::new();
    drafts
        .iter()
        .map(|draft| {
            let marked = draft.marked_running;
            let repeats = eligible(draft) && recurring.contains(&shape(draft));
            if !(marked || repeats) {
                return false;
            }
            !seen.insert(shape(draft))
        })
        .collect()
}

/// Sections: the units under each top-level heading; before the first
/// heading, and wherever no heading covers a unit, one per page.
fn sections_of(units: &[EvidenceUnit]) -> Vec<Section> {
    #[derive(PartialEq)]
    enum Key {
        Heading(u32),
        Page(usize),
    }
    let mut sections: Vec<Section> = Vec::new();
    let mut current: Option<Key> = None;
    for unit in units {
        let key = match unit.section_path.first() {
            Some(top) => Key::Heading(*top),
            None if unit.kind == UnitKind::Heading => Key::Heading(unit.ordinal),
            None => Key::Page(unit.page),
        };
        if current.as_ref() == Some(&key) {
            if let Some(section) = sections.last_mut() {
                section.last_unit = unit.ordinal;
                section.last_page = unit.page;
                section.tags |= unit.features.section_tags;
            }
            continue;
        }
        sections.push(Section {
            heading: match key {
                Key::Heading(heading) => Some(heading),
                Key::Page(_) => None,
            },
            first_unit: unit.ordinal,
            last_unit: unit.ordinal,
            first_page: unit.page,
            last_page: unit.page,
            tags: unit.features.section_tags,
        });
        current = Some(key);
    }
    sections
}

fn is_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return false;
    }
    let inner = trimmed.trim_matches('|');
    let cells = inner.split('|').collect::<Vec<_>>();
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let core = cell.trim().trim_matches(':');
            core.len() >= 3 && core.chars().all(|character| character == '-')
        })
}

fn saturate(count: usize) -> u8 {
    u8::try_from(count).unwrap_or(u8::MAX)
}

/// Runs of whitespace as one space: how a unit reads as a prompt line.
pub fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether `word` stands in `haystack` as whole words.
fn contains_word(haystack: &str, word: &str) -> bool {
    rfind_word(haystack, word).is_some()
}

/// The section tags a (normalized) heading's words name.
fn section_tags(heading: &str) -> u16 {
    let words = heading
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    SECTION_TAG_WORDS
        .iter()
        .filter(|(cue, _)| {
            if cue.contains(' ') {
                heading.contains(*cue)
            } else {
                words.iter().any(|word| {
                    *word == *cue
                        || (word.starts_with(*cue) && word.len() <= cue.len() + 2 && cue.len() >= 4)
                })
            }
        })
        .fold(0, |tags, (_, bits)| tags | bits)
}

/// Every date a unit states, line by wrapped line, with the wording before
/// each read the way validation reads it.
fn date_mentions(
    text: &str,
    order: Option<NumericOrder>,
    headers: Option<&[String]>,
) -> Vec<DateMention> {
    let mut mentions = Vec::new();
    for line in wrapped_lines(text) {
        let normalized = normalize(&line);
        for (iso, at) in dates_stated_on(&normalized, order) {
            if mentions
                .iter()
                .any(|mention: &DateMention| mention.iso == iso && mention.at == at)
            {
                continue;
            }
            let window = window_before(&normalized, at);
            let numeric = normalized[at..]
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
                && !normalized[at..].chars().take(10).any(char::is_alphabetic)
                && !normalized[at..].starts_with(&iso[..4]);
            // The column a row's date stands in - the cell the pipes before
            // it put it in - and that column's header as its label.
            let column_label = headers.and_then(|headers| {
                let before = &normalized[..at];
                let pipes = before.matches('|').count();
                let column = if before.trim_start().starts_with('|') {
                    pipes.checked_sub(1)?
                } else {
                    pipes
                };
                let header = normalize(headers.get(column)?);
                (!header.is_empty()).then(|| format!("{header}: "))
            });
            let column = |read: fn(&str) -> bool| column_label.as_deref().is_some_and(read);
            mentions.push(DateMention {
                column_role: column_label.as_deref().and_then(role_from_wording),
                column_deadline: column(labels_a_deadline),
                column_issue: column(labels_the_issue_date),
                role: role_from_wording(&window),
                deadline: labels_a_deadline(&window),
                issue_label: labels_the_issue_date(&window),
                reference: reference_introduced(&normalized, at),
                defined_term: defines_a_dated_term(&window),
                numeric,
                at,
                iso,
            });
        }
    }
    mentions
}

/// The terms a unit defines: `"Products" means the microscopes ...`
/// defines `products`. Extracted text often runs a definitions article's
/// entries together, so every one in the unit is read, up to eight.
fn defined_terms(normalized: &str) -> Vec<String> {
    const MOST: usize = 8;
    let mut terms = Vec::new();
    for verb in ["\" means", "\" shall mean", "\" has the meaning"] {
        for (at, _) in normalized.match_indices(verb) {
            let Some(open) = normalized[..at].rfind('"') else {
                continue;
            };
            let term = normalized[open + 1..at].trim();
            if !term.is_empty()
                && term.split_whitespace().count() <= 5
                && !terms.iter().any(|known: &String| known == term)
            {
                terms.push(term.to_owned());
            }
            if terms.len() == MOST {
                return terms;
            }
        }
    }
    terms
}

/// Whether the words before a date define a dated term: `"Closing Date"
/// means`, `Effective Date shall mean`, `the "Maturity Date" is`.
fn defines_a_dated_term(window: &str) -> bool {
    let trimmed = window.trim_end();
    for verb in ["shall mean", "means", "shall be", "is"] {
        let Some(rest) = trimmed.strip_suffix(verb) else {
            continue;
        };
        let term = rest
            .trim_end()
            .trim_end_matches(['"', '\'', ')', ','])
            .trim_end();
        if term.ends_with("date") && (rest.contains('"') || verb != "is") {
            return true;
        }
    }
    false
}

fn cue_counts(text: &str, normalized: &str) -> CueCounts {
    let words = normalized
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    let type_nouns = words
        .iter()
        .filter(|word| TYPE_NOUNS.contains(*word))
        .count();
    let starts_with_part = NOT_A_TITLE.iter().any(|part| {
        normalized == *part
            || normalized
                .strip_prefix(*part)
                .is_some_and(|rest| rest.starts_with([' ', ':', '-', '.']))
    });
    let self_naming = words.windows(2).enumerate().any(|(index, pair)| {
        pair[0] == "this"
            && words[index + 1..]
                .iter()
                .take(5)
                .any(|word| TYPE_NOUNS.contains(word))
    });
    let defined_role = PARTY_LABELS.iter().any(|role| {
        role.len() >= 4
            && (normalized.contains(&format!("(\"{role}\")"))
                || normalized.contains(&format!("(the \"{role}\")"))
                || normalized.contains(&format!(", as {role}")))
    });
    CueCounts {
        type_cues: saturate(count_cues(text, TYPE_CUES)),
        type_nouns: saturate(type_nouns),
        party_cues: saturate(count_cues(text, PARTY_CUES)),
        date_role_cues: saturate(count_cues(text, DATE_ROLE_CUES)),
        subject_cues: saturate(count_cues(text, SUBJECT_CUES)),
        signature_cues: saturate(count_cues(text, SIGNATURE_CUES)),
        boilerplate_cues: saturate(count_cues(text, BOILERPLATE_CUES)),
        issuer_cues: saturate(count_cues(text, ISSUER_CUES)),
        customer_cues: saturate(count_cues(text, CUSTOMER_CUES)),
        not_a_title: starts_with_part,
        self_naming,
        defined_role,
    }
}

/// Amounts of money as written: a currency sign or code and the figure
/// after it.
fn money_amounts(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let characters = text.char_indices().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        let (start, character) = characters[index];
        let sign = matches!(character, '$' | '£' | '€');
        let code = !sign
            && ["USD", "EUR", "GBP", "CAD"].iter().any(|code| {
                text[start..].starts_with(*code)
                    && (start == 0
                        || !text[..start]
                            .chars()
                            .next_back()
                            .is_some_and(char::is_alphanumeric))
            });
        if !(sign || code) {
            index += 1;
            continue;
        }
        let mut cursor = index + if code { 3 } else { 1 };
        while cursor < characters.len() && characters[cursor].1 == ' ' {
            cursor += 1;
        }
        let digits_start = cursor;
        while cursor < characters.len()
            && (characters[cursor].1.is_ascii_digit()
                || (matches!(characters[cursor].1, ',' | '.')
                    && characters
                        .get(cursor + 1)
                        .is_some_and(|(_, next)| next.is_ascii_digit())))
        {
            cursor += 1;
        }
        if cursor > digits_start {
            let end = characters
                .get(cursor)
                .map_or(text.len(), |(offset, _)| *offset);
            let amount = text[start..end].trim().to_owned();
            if !found.contains(&amount) {
                found.push(amount);
            }
        }
        index = cursor.max(index + 1);
    }
    found
}

/// Words as written, with the punctuation that ends a name kept apart.
fn words_of(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

fn bare(word: &str) -> &str {
    word.trim_matches(|character: char| !character.is_alphanumeric() && character != '&')
}

fn is_capitalised(word: &str) -> bool {
    let core = bare(word);
    core == "&"
        || (core
            .chars()
            .next()
            .is_some_and(|character| character.is_uppercase() || character.is_ascii_digit())
            && core.chars().any(char::is_alphabetic))
}

/// A word folded the way the organisation lists are written: no case, no
/// trailing period, no quotes, brackets or commas around it.
fn folded_form(word: &str) -> String {
    word.trim_matches(|character: char| matches!(character, ',' | ';' | ':' | '(' | ')' | '"'))
        .trim_end_matches('.')
        .to_lowercase()
}

fn is_organisation_ending(word: &str) -> bool {
    ORGANISATION_ENDINGS.contains(&folded_form(word).as_str())
}

/// A company's legal form, which a comma may set off from its name.
fn is_legal_form(word: &str) -> bool {
    LEGAL_FORMS.contains(&folded_form(word).as_str())
}

/// An initial: "J." in "Ada J. Example".
fn is_initial(word: &str) -> bool {
    let core = bare(word);
    core.chars().count() == 1 && core.chars().all(char::is_uppercase)
}

/// Likely organisations: a run of capitalised words - "&", "and" and "of"
/// may join them - that ends on an organisation word ("LLC", "Bank",
/// "Cooperative") where the name ends: before punctuation, a lower-case
/// word, or the end of the text. A legal form after a comma joins it
/// ("Halden Bay National Bank, N.A.").
fn organisations(text: &str) -> Vec<String> {
    let words = words_of(text);
    let mut found: Vec<String> = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if !is_capitalised(word) || !is_organisation_ending(word) {
            continue;
        }
        let ends_name = word.ends_with([',', '.', ';', ':', ')', '"'])
            || words
                .get(index + 1)
                .is_none_or(|next| !is_capitalised(next) || bare(next).is_empty());
        if !ends_name {
            continue;
        }
        let mut start = index;
        let mut capitalised = 0;
        while start > 0 && index - start < 7 {
            let previous = words[start - 1];
            let clean = previous.chars().all(|character| {
                character.is_alphanumeric() || matches!(character, '&' | '-' | '\'' | '.' | ',')
            });
            // The one comma a name crosses is the one before its legal
            // form: "Quillon Ridge Bakery, Inc.".
            let crossing = start == index && is_legal_form(word);
            if !clean
                || (previous.ends_with([',', ';', ':']) && !crossing)
                || (previous.ends_with('.') && !crossing && !is_initial(previous))
            {
                break;
            }
            let core = bare(previous);
            if is_capitalised(previous) && !matches!(core, "The" | "THE" | "This" | "THIS") {
                capitalised += 1;
                start -= 1;
            } else if matches!(core, "&" | "and" | "of") && start >= 2 {
                start -= 1;
            } else {
                break;
            }
        }
        while start < index && matches!(bare(words[start]), "&" | "and" | "of") {
            start += 1;
            capitalised = capitalised.max(1);
        }
        if capitalised == 0 {
            continue;
        }
        let mut name = words[start..=index].join(" ");
        if word.ends_with(',')
            && let Some(form) = words.get(index + 1)
            && is_organisation_ending(form)
        {
            name.push(' ');
            name.push_str(form);
        }
        let name = normalize_loosely(&name);
        if !name.is_empty() && !found.contains(&name) {
            found.push(name);
        }
    }
    found
}

/// A run of two to four capitalised words with no figures, read from the
/// start of `text`: a person's name as a label's value or after a title.
fn name_at(words: &[&str]) -> Option<String> {
    let mut taken = Vec::new();
    for word in words.iter().take(4) {
        let core = bare(word);
        let initial = core.len() == 1 && core.chars().all(char::is_uppercase);
        if core.is_empty()
            || core.chars().any(|character| character.is_ascii_digit())
            || !(is_capitalised(word) || initial)
            || core == "&"
        {
            break;
        }
        taken.push(*word);
        if word.ends_with([',', ';', ':']) {
            break;
        }
    }
    (taken.len() >= 2 && !taken.iter().any(|word| is_organisation_ending(word)))
        .then(|| normalize_loosely(&taken.join(" ")))
}

/// Likely people: a name after a courtesy title, after "Dear", after a
/// party label (`Tenant: Paloma Iwasaki`, `By: /s/ Ada Example`), or a
/// signature line's `/s/ Name`.
fn people(text: &str, label: Option<&str>) -> Vec<String> {
    let words = words_of(text);
    let mut found: Vec<String> = Vec::new();
    let mut push = |name: Option<String>| {
        if let Some(name) = name
            && !found.contains(&name)
        {
            found.push(name);
        }
    };
    for (index, word) in words.iter().enumerate() {
        let lowered = bare(word).to_lowercase();
        if HONORIFICS.contains(&lowered.as_str()) || lowered == "dear" || word.contains("/s/") {
            let rest = &words[index + 1..];
            let name = name_at(rest).or_else(|| {
                // "Dr. Ada Example" has the title; "Ms. Vale" alone does
                // not make a full name.
                (HONORIFICS.contains(&lowered.as_str()) && !rest.is_empty())
                    .then(|| name_at(&[*word, rest[0]]))
                    .flatten()
            });
            push(name);
        }
    }
    if let Some(label) = label
        && PARTY_LABELS.contains(&label)
        && let Some(colon) = text.find(':')
    {
        let value = words_of(&text[colon + 1..]);
        let value = value
            .iter()
            .copied()
            .filter(|word| *word != "/s/")
            .collect::<Vec<_>>();
        push(name_at(&value));
    }
    found
}

/// How many runs of two or more capitalised words a unit has.
fn proper_name_runs(text: &str) -> usize {
    let mut runs = 0;
    let mut length = 0;
    for word in words_of(text) {
        if is_capitalised(word) && bare(word) != "&" {
            length += 1;
        } else if bare(word) != "&" {
            if length >= 2 {
                runs += 1;
            }
            length = 0;
        }
        if word.ends_with(['.', ',', ';', ':']) {
            if length >= 2 {
                runs += 1;
            }
            length = 0;
        }
    }
    if length >= 2 {
        runs += 1;
    }
    runs
}

/// Identifiers: tokens of letters and figures (`INV-20417`), and a figure
/// a number label introduces (`Invoice No. 1042`, `Policy Number: 88-1`).
fn identifiers(text: &str, label: Option<&str>) -> Vec<Identifier> {
    let mut found: Vec<Identifier> = Vec::new();
    let words = words_of(text);
    // A label that says its value is a number - "Invoice No.", "Policy
    // Number", "PO #" - and not a date.
    let numbered_label = |label: &str| {
        let words = label
            .split(|character: char| !character.is_alphanumeric() && character != '#')
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        (words.iter().any(|word| IDENTIFIER_LABELS.contains(word)) || label.ends_with('#'))
            && !words.contains(&"date")
    };
    for (index, word) in words.iter().enumerate() {
        let token = word.trim_matches(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '-' | '/' | '#')
        });
        let token = token.trim_start_matches('#');
        if token.is_empty() || !token.chars().any(|character| character.is_ascii_digit()) {
            continue;
        }
        let shaped = contains_identifier(token);
        // A figure right after "No.", "#", "Number" or "ID".
        let introduced = index > 0 && {
            let previous = words[index - 1].to_lowercase();
            let previous = previous.trim_end_matches(':');
            matches!(
                previous,
                "no." | "no" | "#" | "number" | "id" | "ref" | "ref."
            ) || previous.ends_with('#')
        };
        let labelled = label.is_some_and(numbered_label)
            && text
                .find(':')
                .is_some_and(|colon| text[colon..].contains(token));
        let is_date = token.matches(['-', '/', '.']).count() == 2
            && token.chars().all(|character| {
                character.is_ascii_digit() || matches!(character, '-' | '/' | '.')
            });
        let figures = token
            .chars()
            .filter(|character| character.is_ascii_digit())
            .count();
        if is_date || token.len() > 24 || (!shaped && figures < 3) {
            continue;
        }
        if shaped || introduced || labelled {
            let value = token.to_owned();
            if found.iter().any(|existing| existing.value == value) {
                continue;
            }
            found.push(Identifier {
                value,
                label: if labelled {
                    label.map(str::to_owned)
                } else if introduced {
                    Some(words[index - 1].trim_end_matches(':').to_lowercase())
                } else {
                    None
                },
            });
        }
    }
    found
}

/// A word as BM25 counts it: folded, with a plural's or a possessive's
/// ending taken off.
pub fn stem(word: &str) -> String {
    let word = word.strip_suffix("'s").unwrap_or(word);
    let count = word.chars().count();
    if count > 4 && word.ends_with("ies") {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if count >= 4
        && word.ends_with('s')
        && !word.ends_with("ss")
        && !word.ends_with("us")
        && !word.ends_with("is")
    {
        return word[..word.len() - 1].to_owned();
    }
    word.to_owned()
}

/// The BM25 terms of normalized text: words of two characters or more that
/// are not figures alone or stopwords, stemmed.
pub fn terms_of(normalized: &str) -> Vec<String> {
    normalized
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.chars().count() >= 2)
        .filter(|word| !word.chars().all(|character| character.is_numeric()))
        .filter(|word| !STOPWORDS.contains(word))
        .map(stem)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distill::source_from_text;
    use crate::domain::SourcePage;
    use crate::evidence::date_match_positions;

    fn page(number: usize, text: &str) -> SourcePage {
        SourcePage::new(number, text, PageOrigin::Native)
    }

    fn invoice() -> DocumentSource {
        DocumentSource::from_pages(vec![page(
            1,
            "HALVORSEN FIXTURE WORKS LLC\n500 Foundry Road\n\nINVOICE\n\nInvoice Number: INV-20417\nInvoice Date: 03/04/2026\nDue Date: 04/03/2026\nBill To: Quillon Ridge Bakery, Inc.\n\n| Item | Qty | Amount |\n| --- | --- | --- |\n| Display case | 2 | $4,000.00 |\n| Total | | $4,805.98 |\n\nThank you for your business. Payment is due within thirty days of the invoice date shown above, by check or wire.",
        )])
    }

    fn long_agreement() -> DocumentSource {
        let filler = "The parties shall perform their obligations in good faith and in \
                      accordance with the terms set forth in this section. ";
        let mut pages = Vec::new();
        for number in 1..=12 {
            let mut body = String::new();
            if number == 1 {
                body.push_str("CREDIT AGREEMENT\n\nThis Credit Agreement is dated as of the Closing Date by and between Marrowfield Packaging Holdings, Inc. (the \"Borrower\") and Halden Bay National Bank, N.A., as Lender.\n\n");
            }
            if number == 5 {
                body.push_str("ARTICLE I DEFINITIONS\n\n\"Closing Date\" means June 12, 2026.\n\n\"Maturity Date\" means June 12, 2031.\n\n");
            }
            body.push_str(&format!("Credit Agreement - Page {number}\n\n"));
            for _ in 0..8 {
                body.push_str(filler);
            }
            body.push_str("\n\n");
            if number == 12 {
                body.push_str("IN WITNESS WHEREOF, the parties have executed this Agreement.\n\nBy: /s/ Ada Example\nName: Ada Example\nTitle: Chief Financial Officer\n");
            }
            pages.push(page(number, &body));
        }
        DocumentSource::from_pages(pages)
    }

    #[test]
    fn units_are_blocks_rows_and_fields_with_unique_ids() {
        let index = EvidenceIndex::build(&invoice());
        let ids = index
            .units()
            .iter()
            .map(|unit| unit.id.as_str())
            .collect::<Vec<_>>();
        let unique = ids.iter().collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), ids.len(), "{ids:?}");
        let date = index.unit("p1.b4.f2").expect("the invoice date field");
        assert_eq!(date.kind, UnitKind::Field);
        assert_eq!(date.text, "Invoice Date: 03/04/2026");
        assert_eq!(date.label.as_deref(), Some("Invoice Date"));
        let header = index.unit("p1.b5.r1").expect("the header row");
        assert_eq!(header.kind, UnitKind::TableHeader);
        let total = index.unit("p1.b5.r3").expect("the total row");
        assert_eq!(total.kind, UnitKind::TableRow);
        assert_eq!(total.table_header, Some(header.ordinal));
        assert_eq!(total.features.money, vec!["$4,805.98".to_owned()]);
        assert!(total.features.money_labelled);
        assert_eq!(index.heading_context(total.ordinal), vec!["INVOICE"]);
        let number = index.unit("p1.b4.f1").unwrap();
        assert_eq!(number.features.identifiers[0].value, "INV-20417");
        let letterhead = &index.units()[0];
        assert!(letterhead.features.position.letterhead);
        assert_eq!(
            letterhead.features.organisations,
            vec!["halvorsen fixture works llc".to_owned()]
        );
        let bill_to = index.unit("p1.b4.f4").unwrap();
        assert_eq!(
            bill_to.features.organisations,
            vec!["quillon ridge bakery inc".to_owned()]
        );
    }

    #[test]
    fn every_unit_is_text_its_page_holds() {
        for source in [invoice(), long_agreement()] {
            let index = EvidenceIndex::build(&source);
            for unit in index.units() {
                let page = source
                    .pages
                    .iter()
                    .find(|page| page.page_number == unit.page)
                    .unwrap();
                assert!(
                    normalize(&page.text).contains(&unit.normalized),
                    "{}: {}",
                    unit.id,
                    unit.text
                );
                assert_eq!(unit.normalized, normalize(&unit.text));
            }
        }
    }

    #[test]
    fn the_index_is_deterministic() {
        let source = long_agreement();
        assert_eq!(EvidenceIndex::build(&source), EvidenceIndex::build(&source));
    }

    #[test]
    fn running_lines_are_kept_once_and_sections_follow_headings() {
        let index = EvidenceIndex::build(&long_agreement());
        let footers = index
            .units()
            .iter()
            .filter(|unit| unit.text.starts_with("Credit Agreement - Page"))
            .collect::<Vec<_>>();
        assert_eq!(footers.len(), 12);
        assert_eq!(footers.iter().filter(|unit| !unit.running).count(), 1);
        assert!(!footers[0].running);
        let definition = index
            .units()
            .iter()
            .find(|unit| unit.text.starts_with("\"Closing Date\" means"))
            .unwrap();
        assert_eq!(
            index.heading_context(definition.ordinal),
            vec!["ARTICLE I DEFINITIONS"]
        );
        assert!(definition.features.section_tags & tag::DEFINITIONS != 0);
        let mention = &definition.features.dates[0];
        assert_eq!(mention.iso, "2026-06-12");
        assert!(mention.defined_term);
        assert!(!mention.reference);
        let section = index.section_of(definition.ordinal).unwrap();
        assert_eq!(
            index.sections()[section].heading,
            index.headings().iter().copied().find(|heading| {
                index.units()[*heading as usize].text == "ARTICLE I DEFINITIONS"
            })
        );
        let signer = index
            .units()
            .iter()
            .find(|unit| unit.text.starts_with("By: /s/"))
            .unwrap();
        assert!(signer.features.position.signature);
        assert!(
            signer.features.people.contains(&"ada example".to_owned()),
            "{:?}",
            signer.features.people
        );
        let preamble = index
            .units()
            .iter()
            .find(|unit| unit.text.starts_with("This Credit Agreement"))
            .unwrap();
        assert!(preamble.features.cues.self_naming);
        assert!(preamble.features.cues.defined_role);
        assert!(
            preamble
                .features
                .organisations
                .contains(&"halden bay national bank na".to_owned()),
            "{:?}",
            preamble.features.organisations
        );
        assert!(
            preamble
                .features
                .organisations
                .contains(&"marrowfield packaging holdings inc".to_owned()),
            "{:?}",
            preamble.features.organisations
        );
    }

    #[test]
    fn a_long_paragraph_is_cut_into_sentence_chunks() {
        let sentence = "Each sentence of this paragraph says one more thing about the lease. ";
        let source = source_from_text(sentence.repeat(20));
        let index = EvidenceIndex::build_with(
            &source,
            IndexOptions {
                max_unit_characters: 250,
            },
        );
        assert!(index.units().len() > 1);
        assert!(index.units().iter().all(|unit| unit.id.contains(".s")));
        assert!(
            index
                .units()
                .iter()
                .all(|unit| unit.text.chars().count() <= 500)
        );
    }

    /// The index reads a date the way validation does: the role from the
    /// wording before it, a deadline label, an issue label, another
    /// document's date. These are the lines the corpus fixtures state their
    /// dates in, from the inference and validation tests.
    #[test]
    fn date_features_agree_with_inference_and_validation() {
        // text, date, role from wording, deadline, issue label, reference
        type Case = (
            &'static str,
            &'static str,
            Option<DateRole>,
            bool,
            bool,
            bool,
        );
        let cases: &[Case] = &[
            (
                "INVOICE\nInvoice Number: INV-7741\nInvoice Date: January 5, 2026\nPayment Due Date: February 4, 2026",
                "2026-01-05",
                Some(DateRole::Invoice),
                false,
                true,
                false,
            ),
            (
                "INVOICE\nInvoice Number: INV-7741\nInvoice Date: January 5, 2026\nPayment Due Date: February 4, 2026",
                "2026-02-04",
                None,
                true,
                false,
                false,
            ),
            (
                "NOTICE OF TERMINATION\nDate of this Notice: December 29, 2026\nYour employment will end effective January 31, 2027.",
                "2026-12-29",
                Some(DateRole::Notice),
                false,
                true,
                false,
            ),
            (
                "STATEMENT OF WORK\nIssued under the Master Services Agreement dated June 2, 2023\nThis Statement of Work is effective as of April 1, 2026.",
                "2023-06-02",
                None,
                false,
                true,
                true,
            ),
            (
                "STATEMENT OF WORK\nIssued under the Master Services Agreement dated June 2, 2023\nThis Statement of Work is effective as of April 1, 2026.",
                "2026-04-01",
                Some(DateRole::Effective),
                false,
                false,
                false,
            ),
            (
                "CERTIFICATE OF GOOD STANDING\nFiled on March 3, 2026 with the Secretary of State.",
                "2026-03-03",
                Some(DateRole::Filing),
                false,
                false,
                false,
            ),
            (
                "INVOICE\nInvoice Date: 04/30/2025    Due Date: 05/30/2025",
                "2025-05-30",
                None,
                true,
                false,
                false,
            ),
        ];
        for (text, date, role, deadline, issue, reference) in cases {
            let source = source_from_text(*text);
            let index = EvidenceIndex::build(&source);
            let mentions = index
                .units()
                .iter()
                .flat_map(|unit| &unit.features.dates)
                .filter(|mention| mention.iso == *date)
                .collect::<Vec<_>>();
            assert!(!mentions.is_empty(), "{date} in {text}");
            for mention in mentions {
                assert_eq!(mention.role, *role, "{date} in {text}");
                assert_eq!(mention.deadline, *deadline, "{date} in {text}");
                assert_eq!(mention.issue_label, *issue, "{date} in {text}");
                assert_eq!(mention.reference, *reference, "{date} in {text}");
            }
            // Every date the index finds is one the evidence check finds.
            for unit in index.units() {
                for mention in &unit.features.dates {
                    assert!(
                        !date_match_positions(&mention.iso, &unit.normalized).is_empty()
                            || wrapped_lines(&unit.text).iter().any(|line| {
                                !date_match_positions(&mention.iso, &normalize(line)).is_empty()
                            }),
                        "{} in {}",
                        mention.iso,
                        unit.text
                    );
                }
            }
        }
    }

    /// The validation tests' deadline and taint cases, read by the index
    /// and by validation's own helpers on the same lines.
    #[test]
    fn date_features_are_validations_own_readings() {
        let texts = [
            "INVOICE\nInvoice Date: April 30, 2025\nDue Date: May 30, 2025\nTotal: $1,248.00",
            "This First Amendment to Consulting Agreement (this \"Amendment\") is dated as of September 14, 2025, and amends the Consulting Agreement dated January 12, 2023.",
            "Pursuant to Section 9.2 of the Employment Agreement, your employment will terminate effective January 31, 2027.",
            "Issued under the Master Services Agreement, effective June 2, 2023, this order begins on July 1, 2023.",
        ];
        for text in texts {
            let index = EvidenceIndex::build(&source_from_text(text));
            for unit in index.units() {
                for line in wrapped_lines(&unit.text) {
                    let normalized = normalize(&line);
                    for (iso, at) in dates_stated_on(&normalized, index.numeric_order()) {
                        let mention = unit
                            .features
                            .dates
                            .iter()
                            .find(|mention| mention.iso == iso && mention.at == at)
                            .unwrap_or_else(|| panic!("{iso} at {at} in {line}"));
                        let window = window_before(&normalized, at);
                        assert_eq!(mention.role, role_from_wording(&window));
                        assert_eq!(mention.deadline, labels_a_deadline(&window));
                        assert_eq!(mention.reference, reference_introduced(&normalized, at));
                    }
                }
            }
        }
    }

    /// Geometry the worker sends - each line's box, the block's - says
    /// where text was, never what it says, so an index built without it is
    /// the same index.
    #[test]
    fn the_index_is_the_same_without_line_geometry() {
        use crate::structure::{LayoutLine, PageLayout, PageRoute, RouteSignals};
        let mut first = page(1, "INVOICE\n\nInvoice date: May 1, 2026\nTotal: $1,248.00");
        let mut layout = PageLayout {
            width: 6120,
            height: 7920,
            route: PageRoute::Layout,
            signals: RouteSignals::default(),
            blocks: Vec::new(),
        };
        let structured_page = structured(&DocumentSource::from_pages(vec![first.clone()]));
        layout.blocks = structured_page.pages[0].blocks.clone();
        for (index, block) in layout.blocks.iter_mut().enumerate() {
            let top = 400 + 200 * index as u32;
            block.bbox = Some([540, top, 3000, top + 150]);
            block.lines = block
                .text
                .lines()
                .enumerate()
                .map(|(line, text)| LayoutLine {
                    text: text.to_owned(),
                    bbox: Some([
                        540,
                        top + 40 * line as u32,
                        3000,
                        top + 40 * line as u32 + 30,
                    ]),
                    confidence: Some(90),
                })
                .collect();
            for field in &mut block.fields {
                field.key_bbox = Some([540, top, 900, top + 30]);
                field.value_bbox = Some([1000, top, 2000, top + 30]);
            }
        }
        first.layout = Some(layout.clone());
        let full = EvidenceIndex::build(&DocumentSource::from_pages(vec![first.clone()]));
        for block in &mut layout.blocks {
            block.lines.clear();
            block.bbox = None;
            for field in &mut block.fields {
                field.key_bbox = None;
                field.value_bbox = None;
            }
        }
        first.layout = Some(layout);
        let stripped = EvidenceIndex::build(&DocumentSource::from_pages(vec![first]));
        assert_eq!(full, stripped);
    }

    /// A row of dates under a header row: the header names each column's
    /// date, which nothing on the row's own line does. A date alone on its
    /// line is a dateline; a definitions entry names what it defines.
    #[test]
    fn a_tables_header_labels_the_dates_in_its_columns() {
        let index = EvidenceIndex::build(&source_from_text(
            "HALVORSEN FIXTURE WORKS LLC\n\nMarch 4, 2026\n\n| Invoice Date | PO Date | Due Date |\n| --- | --- | --- |\n| 03/04/2026 | 02/11/2026 | 04/23/2026 |\n\n\"Services\" means the installation of display cases. \"Site\" means the store.",
        ));
        let row = index
            .units()
            .iter()
            .find(|unit| unit.kind == UnitKind::TableRow)
            .unwrap();
        let mention = |iso: &str| {
            row.features
                .dates
                .iter()
                .find(|mention| mention.iso == iso)
                .unwrap_or_else(|| panic!("{iso} in {:?}", row.features.dates))
        };
        assert!(mention("2026-03-04").column_issue);
        assert_eq!(mention("2026-03-04").column_role, Some(DateRole::Invoice));
        assert!(mention("2026-04-23").column_deadline);
        assert!(!mention("2026-02-11").column_deadline);
        // What validation reads off the row's own line is unchanged.
        assert!(!mention("2026-04-23").deadline);
        let dateline = index.unit("p1.b2").unwrap();
        assert!(dateline.features.dateline);
        assert!(!row.features.dateline);
        let definitions = index
            .units()
            .iter()
            .find(|unit| unit.text.contains("\"Services\" means"))
            .unwrap();
        assert_eq!(definitions.features.defines, vec!["services", "site"]);
    }

    #[test]
    fn terms_are_folded_and_stemmed() {
        assert_eq!(
            terms_of(&normalize(
                "The Tenant's Premises, 2 PROPERTIES and 2026 fees"
            )),
            vec!["tenant", "premise", "property", "fee"]
        );
        let index = EvidenceIndex::build(&invoice());
        let term = index.term("Invoices").expect("invoice is a term");
        assert!(index.document_frequency(term) >= 2);
    }
}
