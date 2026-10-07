//! Whole-document distillation.
//!
//! The model only ever sees a *digest* of the document. The digest is built by
//! reading every block on every page, scoring each one for how much it helps
//! answer "what is this document, when does it take effect, and who is it
//! between", and then keeping the highest-value blocks **in original document
//! order** until a character budget is spent.
//!
//! Three properties matter and are enforced by the tests:
//!
//! 1. Every page is considered. There is no head/tail window, so a fact buried
//!    on page 5 of an 8-page agreement is as reachable as one on page 1.
//! 2. Kept text is verbatim. Evidence the model quotes back can therefore be
//!    checked literally against the digest, which is what makes the
//!    anti-hallucination guarantee real.
//! 3. Order and page boundaries survive, and elisions are marked, so the model
//!    still sees relationships ("this date belongs to that recital") rather
//!    than a bag of disconnected facts.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cues::{
    BOILERPLATE_CUES, DATE_ROLE_CUES, PARTY_CUES, SIGNATURE_CUES, SUBJECT_CUES, TYPE_CUES,
};
use crate::domain::{DocumentSource, PageOrigin, ParserWarning};
use crate::text::{
    contains_identifier, contains_money, count_cues, date_signal_count, digit_masked,
    split_sentences,
};

/// Documents at or below this size are passed through untouched.
pub const PASSTHROUGH_CHARACTERS: usize = 12_000;
/// Upper bound on the distilled document handed to the model.
pub const DEFAULT_BUDGET_CHARACTERS: usize = 12_000;
/// Paragraphs longer than this are split so salient sentences can survive
/// independently of the boilerplate around them.
const MAX_BLOCK_CHARACTERS: usize = 700;
/// A running header or footer must be this short to be collapsed.
const MAX_RUNNING_LINE_CHARACTERS: usize = 120;

const GAP_MARKER: &str = "[...]";

/// The SECTIONS line lists at most this many headings...
const MAX_OUTLINE_HEADINGS: usize = 40;
/// ...in at most this many characters, the cut marked by [`OUTLINE_CUT`].
///
/// The outline is a map, not the territory: a contract's article headings fit
/// in a few hundred characters, and the corpus's longest is about 104. Every
/// heading in a 500-line all-caps scan used to be listed, which made the
/// outline most of the prompt and pushed it past the context window.
const MAX_OUTLINE_CHARACTERS: usize = 1_500;
const OUTLINE_CUT: &str = " | …";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlockKind {
    Heading,
    Table,
    Body,
}

#[derive(Clone, Debug)]
struct Block {
    page_number: usize,
    order: usize,
    kind: BlockKind,
    text: String,
    score: i32,
    mandatory: bool,
}

/// How aggressively to distill.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DigestBudget {
    pub passthrough_characters: usize,
    pub max_characters: usize,
}

impl Default for DigestBudget {
    fn default() -> Self {
        Self {
            passthrough_characters: PASSTHROUGH_CHARACTERS,
            max_characters: DEFAULT_BUDGET_CHARACTERS,
        }
    }
}

/// The distilled document handed to the model, plus what it cost to build.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentDigest {
    /// The exact text placed in the prompt.
    pub text: String,
    /// Verbatim kept blocks. Evidence must appear inside one of these.
    pub segments: Vec<String>,
    /// Every heading found anywhere in the document, in order.
    pub outline: Vec<String>,
    /// Every line that carries a date, in document order.
    ///
    /// A long agreement can mention a dozen dates. Listing each one next to the
    /// words around it turns "which of these defines the document" from a
    /// scanning problem into a reading problem, which is what a small model is
    /// actually good at.
    pub date_lines: Vec<String>,
    pub page_count: usize,
    pub source_characters: usize,
    pub digest_characters: usize,
    pub compressed: bool,
    pub image_included: bool,
    pub parser_warnings: Vec<ParserWarning>,
}

impl DocumentDigest {
    pub fn compression_ratio(&self) -> f32 {
        if self.source_characters == 0 {
            return 1.0;
        }
        self.digest_characters as f32 / self.source_characters as f32
    }
}

/// Reads the whole document and returns the digest the model will see.
pub fn distill(source: &DocumentSource, budget: DigestBudget) -> DocumentDigest {
    let source_characters = source.character_count();
    let mut blocks = segment(source);
    collapse_running_lines(&mut blocks, source.pages.len());
    let outline = blocks
        .iter()
        .filter(|block| block.kind == BlockKind::Heading)
        .map(|block| normalize_heading(&block.text))
        .filter(|heading| !heading.is_empty())
        .collect::<Vec<_>>();

    let last_page = source.pages.last().map_or(0, |page| page.page_number);
    let block_count = blocks.len();
    for (index, block) in blocks.iter_mut().enumerate() {
        score_block(block, index, block_count, last_page);
    }

    let compressed = source_characters > budget.passthrough_characters;
    let kept = if compressed {
        select(&blocks, budget.max_characters)
    } else {
        (0..blocks.len()).collect()
    };

    let date_lines = date_lines(&blocks, &kept);
    let (text, segments) = emit(&blocks, &kept, &outline, &date_lines, compressed);
    DocumentDigest {
        date_lines,
        digest_characters: text.chars().count(),
        text,
        segments,
        outline,
        page_count: source.pages.len(),
        source_characters,
        compressed,
        image_included: source.page_image.is_some(),
        parser_warnings: source.parser_warnings.clone(),
    }
}

fn segment(source: &DocumentSource) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut order = 0;
    for page in &source.pages {
        let mut pending: Vec<&str> = Vec::new();
        let mut pending_table = false;
        for line in page.text.lines() {
            let trimmed = line.trim_end();
            if trimmed.trim().is_empty() {
                flush_pending(
                    &mut blocks,
                    &mut pending,
                    &mut pending_table,
                    &mut order,
                    page.page_number,
                );
                continue;
            }
            let table_line = trimmed.trim_start().starts_with('|');
            if table_line != pending_table && !pending.is_empty() {
                flush_pending(
                    &mut blocks,
                    &mut pending,
                    &mut pending_table,
                    &mut order,
                    page.page_number,
                );
            }
            pending_table = table_line;
            // A table row is never a heading, however capitalised: a ledger's
            // `| 4000 | SALES REVENUE | CR |` rows each became one, and a
            // 600-row sheet filled the outline with 600 of them.
            if !table_line && is_heading_line(trimmed) {
                flush_pending(
                    &mut blocks,
                    &mut pending,
                    &mut pending_table,
                    &mut order,
                    page.page_number,
                );
                blocks.push(Block {
                    page_number: page.page_number,
                    order,
                    kind: BlockKind::Heading,
                    text: trimmed.trim().to_owned(),
                    score: 0,
                    mandatory: false,
                });
                order += 1;
                continue;
            }
            pending.push(trimmed);
        }
        flush_pending(
            &mut blocks,
            &mut pending,
            &mut pending_table,
            &mut order,
            page.page_number,
        );
    }
    blocks
}

fn flush_pending(
    blocks: &mut Vec<Block>,
    pending: &mut Vec<&str>,
    pending_table: &mut bool,
    order: &mut usize,
    page_number: usize,
) {
    if pending.is_empty() {
        return;
    }
    let joined = pending.join("\n");
    pending.clear();
    let kind = if *pending_table {
        BlockKind::Table
    } else {
        BlockKind::Body
    };
    *pending_table = false;
    for chunk in split_sentences(&joined, MAX_BLOCK_CHARACTERS) {
        if chunk.trim().is_empty() {
            continue;
        }
        blocks.push(Block {
            page_number,
            order: *order,
            kind,
            text: chunk.trim_end().to_owned(),
            score: 0,
            mandatory: false,
        });
        *order += 1;
    }
}

pub(crate) fn is_heading_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 90 {
        return false;
    }
    if trimmed.starts_with('#') {
        return true;
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
    // A short line that is essentially all capitals reads as a heading in every
    // extracted format Intern supports, including OCR output.
    uppercase * 10 >= letter_count * 8
}

pub(crate) fn normalize_heading(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('#')
        .trim()
        .trim_end_matches(':')
        .to_owned()
}

/// Drops repeated running headers and footers after their first appearance.
fn collapse_running_lines(blocks: &mut Vec<Block>, page_count: usize) {
    if page_count < 3 {
        return;
    }
    let mut pages_by_shape: HashMap<String, Vec<usize>> = HashMap::new();
    for block in blocks.iter() {
        if block.text.chars().count() > MAX_RUNNING_LINE_CHARACTERS {
            continue;
        }
        pages_by_shape
            .entry(digit_masked(block.text.trim()))
            .or_default()
            .push(block.page_number);
    }
    let threshold = (page_count / 2).max(3);
    let running = pages_by_shape
        .into_iter()
        .filter(|(_, pages)| {
            let mut unique = pages.clone();
            unique.sort_unstable();
            unique.dedup();
            unique.len() >= threshold
        })
        .map(|(shape, _)| shape)
        .collect::<std::collections::HashSet<_>>();
    if running.is_empty() {
        return;
    }
    let mut seen = std::collections::HashSet::new();
    blocks.retain(|block| {
        if block.text.chars().count() > MAX_RUNNING_LINE_CHARACTERS {
            return true;
        }
        let shape = digit_masked(block.text.trim());
        if !running.contains(&shape) {
            return true;
        }
        seen.insert(shape)
    });
}

fn score_block(block: &mut Block, index: usize, total: usize, last_page: usize) {
    let text = &block.text;
    let dates = date_signal_count(text);
    let date_roles = count_cues(text, DATE_ROLE_CUES);
    let parties = count_cues(text, PARTY_CUES);
    let types = count_cues(text, TYPE_CUES);
    let subjects = count_cues(text, SUBJECT_CUES);
    let signatures = count_cues(text, SIGNATURE_CUES);
    let boilerplate = count_cues(text, BOILERPLATE_CUES);

    let mut score = 0_i32;
    score += (dates as i32) * 30;
    score += (date_roles as i32) * 45;
    score += (parties as i32) * 22;
    score += (types as i32) * 26;
    score += (subjects as i32) * 24;
    score += (signatures as i32) * 14;
    if block.kind == BlockKind::Heading {
        score += 40;
    }
    if block.kind == BlockKind::Table {
        score += 10;
    }
    if contains_identifier(text) {
        score += 14;
    }
    if contains_money(text) {
        score += 8;
    }
    // The opening of a document names it; the closing signs it.
    let opening = index < 4;
    let closing = index + 3 >= total;
    if opening {
        score += 60;
    }
    if closing {
        score += 25;
    }
    if block.page_number == 1 {
        score += 12;
    }
    if block.page_number == last_page {
        score += 8;
    }
    if boilerplate > 0 && block.kind != BlockKind::Heading {
        score -= 70;
    }
    let length = text.chars().count() as i32;
    if length < 25 {
        score -= 8;
    }
    if length > 400 {
        score -= 6;
    }

    block.score = score;
    // Boilerplate is never mandatory, even at the top of a document: a contract
    // whose first clause is "Governing Law" must not spend its budget on it.
    let names_the_document = block.kind == BlockKind::Heading && (types > 0 || dates > 0);
    let dates_something = date_roles > 0 || (dates > 0 && block.page_number == 1);
    let names_the_parties = parties > 0 && types > 0;
    block.mandatory = boilerplate == 0
        && (opening
            || dates_something
            || names_the_parties
            || subjects > 0
            || (closing && signatures > 0)
            || names_the_document);
}

/// Chooses the block indices that fit inside the budget.
///
/// Mandatory blocks are taken first, in descending score so that a document
/// whose mandatory set alone exceeds the budget still keeps its best evidence.
/// Everything else fills the remainder by score. Ties break on document order,
/// so the same document always produces the same digest.
fn select(blocks: &[Block], max_characters: usize) -> Vec<usize> {
    let mut ranked = (0..blocks.len()).collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        let left_block = &blocks[*left];
        let right_block = &blocks[*right];
        right_block
            .mandatory
            .cmp(&left_block.mandatory)
            .then(right_block.score.cmp(&left_block.score))
            .then(left_block.order.cmp(&right_block.order))
    });

    let mut kept = Vec::new();
    let mut used = 0;
    // A contract repeats the same clause wording in a dozen places. Once one
    // instance is in the digest the others buy nothing, so they never compete
    // for budget with text that appears only once.
    //
    // Two keys. A block whose text is identical to one already kept (clause
    // number aside) is dropped whether or not it is mandatory: the corpus
    // statement of work names its parties and its type inside the very clauses
    // it repeats, which made every copy mandatory and spent a third of the
    // budget on four readings of the same assignment clause. A block that
    // merely *shapes* like one already kept - digits masked, so page and
    // clause numbers do not count - is dropped only when it is not mandatory,
    // because two mandatory blocks that differ only in their digits may differ
    // in exactly the date this digest exists to carry.
    let mut exact = std::collections::HashSet::new();
    let mut shapes = std::collections::HashSet::new();
    for index in ranked {
        let block = &blocks[index];
        let body = strip_enumerators(block.text.trim());
        let repeated = !exact.insert(collapse_whitespace(body));
        let (head, tail) = duplicate_shapes(block.kind, body);
        let shaped_like_kept =
            shapes.contains(&head) || tail.as_ref().is_some_and(|tail| shapes.contains(tail));
        shapes.insert(head);
        if let Some(tail) = tail {
            shapes.insert(tail);
        }
        if repeated || (shaped_like_kept && !block.mandatory) {
            continue;
        }
        let cost = block.text.chars().count() + 2;
        if used + cost > max_characters && !kept.is_empty() {
            continue;
        }
        used += cost;
        kept.push(index);
        if used >= max_characters {
            break;
        }
    }
    kept.sort_unstable();
    kept
}

/// The keys under which a block counts as a repeat of one already kept.
///
/// The head is the first 80 characters with digits masked and any leading
/// clause number stripped, so `25. Assignment.` and `41. Assignment.` collapse
/// onto each other and onto an unnumbered copy. Long body blocks also key on
/// their last 80 characters: the statement of work in the corpus repeated its
/// assignment and force-majeure clauses with some copies starting mid-clause,
/// and those copies still end on the same words. Tables and headings never
/// key on their tail, because a totals row or a short title legitimately ends
/// many different blocks the same way.
fn duplicate_shapes(kind: BlockKind, body: &str) -> (String, Option<String>) {
    const WINDOW: usize = 80;
    const TAIL_MINIMUM: usize = 3 * WINDOW;
    let masked = digit_masked(body);
    let head = masked.chars().take(WINDOW).collect::<String>();
    let length = masked.chars().count();
    let tail = (kind == BlockKind::Body && length >= TAIL_MINIMUM)
        .then(|| masked.chars().skip(length - WINDOW).collect::<String>());
    (head, tail)
}

/// The keys selection deduplicates a block's text under: the exact text
/// (clause number stripped, whitespace collapsed) and its digit-masked
/// shapes ([`duplicate_shapes`]), the tail only for body text. The evidence
/// retriever drops repeats by the same keys the digest does.
pub(crate) fn duplicate_keys(text: &str, body: bool) -> (String, String, Option<String>) {
    let stripped = strip_enumerators(text.trim());
    let kind = if body {
        BlockKind::Body
    } else {
        BlockKind::Table
    };
    let (head, tail) = duplicate_shapes(kind, stripped);
    (collapse_whitespace(stripped), head, tail)
}

/// Lowercased with runs of whitespace collapsed, so a clause re-flowed onto
/// different line breaks still reads as the same text.
fn collapse_whitespace(value: &str) -> String {
    value
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Drops a leading clause label - `25.`, `4.2`, `(a)`, `(iv)`, `Section 4.`,
/// `ARTICLE IV` - so the same clause under different numbers reads the same.
fn strip_enumerators(text: &str) -> &str {
    let mut rest = text.trim_start();
    let mut after_word = false;
    for _ in 0..3 {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let token = &rest[..end];
        if token.is_empty() || token.chars().count() > 12 {
            break;
        }
        let lowered = token.to_ascii_lowercase();
        let word = matches!(
            lowered.trim_end_matches('.'),
            "section" | "article" | "clause" | "part" | "§" | "item" | "paragraph"
        );
        let core =
            token.trim_matches(|character: char| matches!(character, '(' | ')' | '.' | ':' | '-'));
        let punctuated = token.ends_with('.') || token.ends_with(')') || token.ends_with(':');
        let numeric = !core.is_empty()
            && core
                .chars()
                .all(|character| character.is_ascii_digit() || character == '.')
            && core.chars().any(|character| character.is_ascii_digit());
        let roman = !core.is_empty()
            && core.len() <= 5
            && core
                .chars()
                .all(|character| matches!(character, 'i' | 'v' | 'x' | 'I' | 'V' | 'X'));
        let letter = core.len() == 1
            && core
                .chars()
                .all(|character| character.is_ascii_alphabetic());
        let label = numeric || ((roman || letter) && (punctuated || after_word));
        if !(word || label) {
            break;
        }
        after_word = word;
        rest = rest[end..].trim_start();
        if label {
            after_word = false;
        }
    }
    rest
}

/// Collects every kept line that carries a date, trimmed and deduplicated.
fn date_lines(blocks: &[Block], kept: &[usize]) -> Vec<String> {
    const MAX_LINES: usize = 14;
    const MAX_LINE_CHARACTERS: usize = 150;
    let mut lines: Vec<String> = Vec::new();
    for index in kept {
        // The line is the unit. On an invoice, an order form, or a letter every
        // date sits on its own labelled line, and running them together is
        // exactly what destroys the distinction this index exists to make. Only
        // a line too long to read on its own is split further, which is what
        // hard-wrapped contract prose needs.
        for line in blocks[*index].text.lines() {
            let candidates = if line.chars().count() > MAX_LINE_CHARACTERS {
                split_sentences(line, MAX_LINE_CHARACTERS)
            } else {
                vec![line.to_owned()]
            };
            for candidate in candidates {
                let trimmed = candidate.trim();
                if trimmed.is_empty() || date_signal_count(trimmed) == 0 {
                    continue;
                }
                let shortened = trimmed
                    .chars()
                    .take(MAX_LINE_CHARACTERS)
                    .collect::<String>();
                if !lines.iter().any(|existing| existing == &shortened) {
                    lines.push(shortened);
                }
                if lines.len() >= MAX_LINES {
                    return lines;
                }
            }
        }
    }
    lines
}

fn emit(
    blocks: &[Block],
    kept: &[usize],
    outline: &[String],
    date_lines: &[String],
    compressed: bool,
) -> (String, Vec<String>) {
    let mut text = String::new();
    let mut segments = Vec::with_capacity(kept.len());
    if compressed && !outline.is_empty() {
        text.push_str("SECTIONS: ");
        text.push_str(&capped_outline(outline));
        text.push_str("\n\n");
    }
    if date_lines.len() > 1 {
        text.push_str("EVERY DATE IN THIS DOCUMENT, WITH ITS OWN WORDS:\n");
        for line in date_lines {
            text.push_str("- ");
            text.push_str(line);
            text.push('\n');
        }
        text.push('\n');
    }
    let mut previous_index: Option<usize> = None;
    let mut previous_page = 0;
    for index in kept {
        let block = &blocks[*index];
        if block.page_number != previous_page {
            if previous_page != 0 {
                text.push('\n');
            }
            text.push_str(&format!("[Page {}]\n", block.page_number));
            previous_page = block.page_number;
        } else if previous_index.is_some_and(|previous| previous + 1 != *index) {
            text.push_str(GAP_MARKER);
            text.push('\n');
        }
        text.push_str(&block.text);
        text.push_str("\n\n");
        segments.push(block.text.clone());
        previous_index = Some(*index);
    }
    (text.trim_end().to_owned(), segments)
}

/// The outline as the SECTIONS line shows it: the headings in order, joined,
/// up to [`MAX_OUTLINE_HEADINGS`] and [`MAX_OUTLINE_CHARACTERS`] including the
/// cut marker, so a document of a few dozen headings reads exactly as before.
fn capped_outline(outline: &[String]) -> String {
    const SEPARATOR: &str = " | ";
    let full = outline.join(SEPARATOR);
    if outline.len() <= MAX_OUTLINE_HEADINGS && full.chars().count() <= MAX_OUTLINE_CHARACTERS {
        return full;
    }
    let room = MAX_OUTLINE_CHARACTERS - OUTLINE_CUT.chars().count();
    let mut capped = String::new();
    let mut used = 0;
    for (index, heading) in outline.iter().take(MAX_OUTLINE_HEADINGS).enumerate() {
        let cost = heading.chars().count() + if index == 0 { 0 } else { SEPARATOR.len() };
        if used + cost > room {
            break;
        }
        if index > 0 {
            capped.push_str(SEPARATOR);
        }
        capped.push_str(heading);
        used += cost;
    }
    capped.push_str(OUTLINE_CUT);
    capped
}

/// Convenience constructor for callers that only have flat text.
pub fn source_from_text(text: impl Into<String>) -> DocumentSource {
    DocumentSource::from_pages(vec![crate::domain::SourcePage::new(
        1,
        text,
        PageOrigin::PlainText,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SourcePage;

    fn page(number: usize, text: &str) -> SourcePage {
        SourcePage::new(number, text, PageOrigin::Native)
    }

    #[test]
    fn small_documents_are_passed_through_verbatim() {
        let source = DocumentSource::from_pages(vec![page(
            1,
            "NOTICE OF TERMINATION\n\nDated March 3, 2026.",
        )]);
        let digest = distill(&source, DigestBudget::default());
        assert!(!digest.compressed);
        assert!(digest.text.contains("NOTICE OF TERMINATION"));
        assert!(digest.text.contains("Dated March 3, 2026."));
    }

    #[test]
    fn a_fact_buried_in_the_middle_of_a_long_document_survives() {
        let filler = "The parties shall perform their obligations in good faith and in \
                      accordance with the terms set forth in this section. ";
        let mut pages = Vec::new();
        for number in 1..=8 {
            let mut body = String::new();
            if number == 1 {
                body.push_str("MASTER SERVICES AGREEMENT\n\n");
            }
            if number == 5 {
                body.push_str(
                    "5.1 Effective Date. This Statement of Work is effective as of April 1, 2026.\n\n",
                );
            }
            for _ in 0..40 {
                body.push_str(filler);
            }
            pages.push(page(number, &body));
        }
        let source = DocumentSource::from_pages(pages);
        let digest = distill(&source, DigestBudget::default());

        assert!(digest.compressed);
        assert!(digest.digest_characters < digest.source_characters);
        assert!(
            digest.text.contains("effective as of April 1, 2026"),
            "middle-of-document effective date was dropped: {}",
            digest.text
        );
    }

    #[test]
    fn distillation_never_invents_text() {
        let source = DocumentSource::from_pages(vec![page(
            1,
            "INVOICE\n\nInvoice Date: April 30, 2026\nPayment Due: May 30, 2026\n",
        )]);
        let digest = distill(&source, DigestBudget::default());
        for segment in &digest.segments {
            assert!(source.pages[0].text.contains(segment.trim()), "{segment}");
        }
    }

    #[test]
    fn boilerplate_is_demoted_below_identifying_content() {
        let mut body = String::from("CONSULTING AGREEMENT\n\n");
        body.push_str("This Consulting Agreement is made as of September 14, 2025 by and between Contoso Worldwide, Inc. and Jane Smith.\n\n");
        for index in 0..60 {
            body.push_str(&format!(
                "{index}. Governing Law. This Agreement shall be governed by and construed in \
                 accordance with the laws of the State of Delaware without regard to its conflict \
                 of laws principles, and each party irrevocably submits to the exclusive \
                 jurisdiction of the courts located therein.\n\n"
            ));
        }
        let source = DocumentSource::from_pages(vec![page(1, &body)]);
        let digest = distill(
            &source,
            DigestBudget {
                passthrough_characters: 500,
                max_characters: 1_500,
            },
        );
        assert!(digest.text.contains("September 14, 2025"));
        assert!(digest.text.contains("Contoso Worldwide, Inc."));
        let governing = digest.text.matches("Governing Law").count();
        assert!(
            governing <= 2,
            "boilerplate dominated the digest: {governing}"
        );
    }

    #[test]
    fn running_footers_collapse_to_one_instance() {
        let pages = (1..=6)
            .map(|number| {
                page(
                    number,
                    &format!(
                        "Confidential - Do Not Distribute\n\nSection {number} body text that differs per page.\n\nPage {number} of 6"
                    ),
                )
            })
            .collect();
        let source = DocumentSource::from_pages(pages);
        let digest = distill(&source, DigestBudget::default());
        assert_eq!(
            digest
                .text
                .matches("Confidential - Do Not Distribute")
                .count(),
            1
        );
    }

    #[test]
    fn a_form_keeps_each_labelled_date_on_its_own_line() {
        let source = DocumentSource::from_pages(vec![page(
            1,
            "INVOICE\nAcme Corporation, 500 Foundry Road\nInvoice Number: INV-7741\nInvoice Date: January 5, 2026\nPayment Due Date: February 4, 2026\nBill To: Contoso Worldwide, Inc.\n",
        )]);
        let digest = distill(&source, DigestBudget::default());
        assert!(
            digest
                .date_lines
                .iter()
                .any(|line| line == "Invoice Date: January 5, 2026"),
            "{:?}",
            digest.date_lines
        );
        assert!(
            digest
                .date_lines
                .iter()
                .any(|line| line == "Payment Due Date: February 4, 2026"),
            "{:?}",
            digest.date_lines
        );
        // The two dates must never share an entry, and a party must never be
        // swept into one: telling them apart is the whole point of the index.
        for line in &digest.date_lines {
            assert!(!line.contains("Bill To"), "{line}");
            assert!(
                !(line.contains("January 5") && line.contains("February 4")),
                "{line}"
            );
        }
    }

    #[test]
    fn wrapped_contract_prose_still_keeps_its_date_with_its_meaning() {
        let source = DocumentSource::from_pages(vec![page(
            1,
            "4.1 Effective Date.\nThis Statement of Work is effective as of April 1, 2026 and continues\nthrough March 31, 2027 unless terminated earlier.\n",
        )]);
        let digest = distill(&source, DigestBudget::default());
        assert!(
            digest
                .date_lines
                .iter()
                .any(|line| line.contains("effective as of April 1, 2026")),
            "{:?}",
            digest.date_lines
        );
    }

    /// The corpus statement of work repeats its assignment clause several
    /// times: numbered, renumbered, and as a copy that starts mid-clause. The
    /// first two collapsed already; the third cost budget for nothing.
    #[test]
    fn a_clause_repeated_under_new_numbers_or_from_mid_clause_is_kept_once() {
        let clause = "The Contractor may not assign this engagement or any of its rights or \
                      obligations under it without prior written consent, and any purported \
                      assignment without that consent is void and of no effect. Any permitted \
                      assignee must first agree in writing to be bound by every term of this \
                      engagement, must confirm that agreement to each other signatory, and must \
                      do so before the assignment takes effect.";
        let partial = &clause[clause.find("without prior written").unwrap()..];
        assert!(
            partial.chars().count() >= 240,
            "{}",
            partial.chars().count()
        );
        let digest = distill(
            &repeated_clause_document(&[
                format!("25. Assignment. {clause}"),
                format!("41. Assignment. {clause}"),
                clause.to_owned(),
                partial.to_owned(),
            ]),
            DigestBudget::default(),
        );
        assert!(digest.compressed);
        assert_eq!(
            digest
                .text
                .matches("before the assignment takes effect")
                .count(),
            1,
            "{}",
            digest.text
        );
        assert!(digest.text.contains("effective as of April 1, 2026"));
    }

    /// A clause that names the parties and the type is mandatory, and the
    /// corpus statement of work repeats such clauses verbatim under new
    /// numbers. Identical text is kept once even when mandatory; text that
    /// differs in its digits is not the same text, because the digits may be
    /// the date.
    #[test]
    fn identical_mandatory_clauses_collapse_but_digit_differences_do_not() {
        let clause = "Acme Corporation and Contoso Worldwide, Inc. agree that this Statement of \
                      Work governs the assignment of every deliverable listed in it.";
        let digest = distill(
            &repeated_clause_document(&[
                format!("12. Scope. {clause}"),
                format!("30. Scope. {clause}"),
                format!("31. Scope. {clause}"),
            ]),
            DigestBudget::default(),
        );
        assert_eq!(
            digest.text.matches("governs the assignment").count(),
            1,
            "{}",
            digest.text
        );

        let digest = distill(
            &repeated_clause_document(&[
                "This Statement of Work between Acme Corporation and Contoso Worldwide, Inc. commences on April 1, 2026.".to_owned(),
                "This Statement of Work between Acme Corporation and Contoso Worldwide, Inc. commences on April 5, 2026.".to_owned(),
            ]),
            DigestBudget::default(),
        );
        assert!(digest.text.contains("April 1, 2026"), "{}", digest.text);
        assert!(digest.text.contains("April 5, 2026"), "{}", digest.text);
    }

    /// A long document whose first page names the agreement and whose later
    /// pages each carry one of `clauses` between runs of filler prose.
    fn repeated_clause_document(clauses: &[String]) -> DocumentSource {
        let filler = "The parties shall perform their obligations in good faith and in \
                      accordance with the terms set forth in this section. ";
        let mut pages = Vec::new();
        let mut body = String::from(
            "STATEMENT OF WORK\n\nThis Statement of Work is effective as of April 1, 2026 by and between Acme Corporation and Contoso Worldwide, Inc.\n\n",
        );
        for _ in 0..40 {
            body.push_str(filler);
        }
        pages.push(page(1, &body));
        for (offset, clause) in clauses.iter().enumerate() {
            let mut body = String::new();
            for _ in 0..20 {
                body.push_str(filler);
            }
            body.push_str("\n\n");
            body.push_str(clause);
            body.push_str("\n\n");
            for _ in 0..20 {
                body.push_str(filler);
            }
            pages.push(page(offset + 2, &body));
        }
        let source = DocumentSource::from_pages(pages);
        assert!(source.character_count() > PASSTHROUGH_CHARACTERS);
        source
    }

    #[test]
    fn clause_labels_are_stripped_from_the_duplicate_key_only() {
        assert_eq!(
            strip_enumerators("25. Assignment. The Contractor"),
            "Assignment. The Contractor"
        );
        assert_eq!(
            strip_enumerators("4.2 Fees. The Company"),
            "Fees. The Company"
        );
        assert_eq!(strip_enumerators("(a) the first item"), "the first item");
        assert_eq!(strip_enumerators("(iv) the fourth item"), "the fourth item");
        assert_eq!(strip_enumerators("Section 4. Term."), "Term.");
        assert_eq!(strip_enumerators("ARTICLE IV DELIVERABLES"), "DELIVERABLES");
        assert_eq!(strip_enumerators("A. Definitions"), "Definitions");
        assert_eq!(
            strip_enumerators("A party may terminate"),
            "A party may terminate",
            "a bare article is prose, not a label"
        );
        assert_eq!(
            strip_enumerators("I agree to the terms"),
            "I agree to the terms"
        );
        assert_eq!(strip_enumerators("Invoice 2026-001"), "Invoice 2026-001");
    }

    /// The SECTIONS line of a digest, without its label.
    fn sections(digest: &DocumentDigest) -> &str {
        digest
            .text
            .strip_prefix("SECTIONS: ")
            .and_then(|rest| rest.split("\n\n").next())
            .expect("a compressed digest with headings starts with its outline")
    }

    /// A general-ledger export renders as a markdown table of all-caps rows.
    /// Each row used to read as a heading, so the outline listed all 600 of
    /// them - 32,000 characters in a prompt meant to hold 12,000.
    #[test]
    fn table_rows_are_never_headings() {
        let mut sheet = String::from(
            "GENERAL LEDGER\n\n| ACCOUNT | DESCRIPTION | SIDE |\n| --- | --- | --- |\n",
        );
        for row in 0..600 {
            sheet.push_str(&format!(
                "| {} | SALES REVENUE REGION {} | CR |\n",
                4000 + row,
                row % 7
            ));
        }
        let source = DocumentSource::from_pages(vec![page(1, &sheet)]);
        assert!(source.character_count() > PASSTHROUGH_CHARACTERS);
        let digest = distill(&source, DigestBudget::default());

        assert!(digest.compressed);
        assert_eq!(digest.outline, vec!["GENERAL LEDGER".to_owned()]);
        let outline = sections(&digest);
        assert!(!outline.contains('|'), "{outline}");
        assert!(outline.chars().count() <= 1_500);
        // The rows are still there to be read, as a table.
        assert!(
            digest
                .text
                .contains("| 4000 | SALES REVENUE REGION 0 | CR |")
        );
    }

    /// An all-caps scan makes every line a heading. However many there are,
    /// the outline stays a short map, and says that it was cut.
    #[test]
    fn sections_outline_is_capped() {
        let mut scan = String::new();
        for line in 0..500 {
            scan.push_str(&format!(
                "SCHEDULE {line} OF THE ASSET PURCHASE AGREEMENT\n"
            ));
            scan.push_str("The assets listed in this schedule transfer at closing.\n\n");
        }
        let source = DocumentSource::from_pages(vec![page(1, &scan)]);
        let digest = distill(&source, DigestBudget::default());

        assert_eq!(
            digest.outline.len(),
            500,
            "the digest still knows every heading"
        );
        let outline = sections(&digest);
        assert!(
            outline.chars().count() <= 1_500,
            "{}",
            outline.chars().count()
        );
        assert!(outline.ends_with(" | …"), "{outline}");
        let listed = outline.trim_end_matches(" | …").split(" | ").count();
        assert!((1..=40).contains(&listed), "{listed}");
        assert!(outline.starts_with("SCHEDULE 0 OF THE ASSET PURCHASE AGREEMENT | SCHEDULE 1 "));

        // Long headings run into the character cap before the heading cap.
        let wide = (0..60)
            .map(|index| format!("{index:02} {}", "W".repeat(80)))
            .collect::<Vec<_>>();
        let capped = capped_outline(&wide);
        assert!(
            capped.chars().count() <= 1_500,
            "{}",
            capped.chars().count()
        );
        assert!(capped.ends_with(" | …"));
        assert!(capped.trim_end_matches(" | …").split(" | ").count() < 40);
    }

    /// The corpus's outlines are about a hundred characters, and a prompt that
    /// changes by a byte is a stale recording.
    #[test]
    fn a_short_outline_is_emitted_unchanged() {
        let outline = [
            "MASTER SERVICES AGREEMENT",
            "1. DEFINITIONS",
            "2. SERVICES",
            "3. FEES",
        ]
        .map(str::to_owned);
        assert_eq!(
            capped_outline(&outline),
            "MASTER SERVICES AGREEMENT | 1. DEFINITIONS | 2. SERVICES | 3. FEES"
        );
        let forty = (0..40)
            .map(|index| format!("ARTICLE {index}"))
            .collect::<Vec<_>>();
        assert_eq!(capped_outline(&forty), forty.join(" | "));
    }

    #[test]
    fn the_digest_is_deterministic() {
        let source = DocumentSource::from_pages(
            (1..=6)
                .map(|number| {
                    page(
                        number,
                        &format!("Section {number}\n\nBody {number} text. ").repeat(40),
                    )
                })
                .collect(),
        );
        let budget = DigestBudget::default();
        assert_eq!(distill(&source, budget), distill(&source, budget));
    }
}
