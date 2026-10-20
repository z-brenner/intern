//! From detected boxes and their readings to the finished page: gutter
//! cuts, line splits, list markers, reading order, blocks, markdown and
//! tables (spec §8; reference `OrtOCR.assemble`, `pipeline.py:775-913`, and
//! `layout_post.py`).
//!
//! As in [`super::order`], box arithmetic is `f32` where the reference's
//! is (its boxes are `float32` arrays, and a Python float meeting one is
//! cast to `float32`), so splits, merges and tables come out bit-identical,
//! not merely close. Python's string semantics are in [`pystr`].

pub mod pystr;

use std::collections::BTreeMap;

use super::order::{
    decode_blocks_order_f32, line_features_f32, mean_f32, median_f32, median_f64, order_xycut_f32,
    split_blocks,
};
use super::types::{
    BlockClass, DetBox, OrderOutputs, PageBlock, PageLine, PageReading, RecLine, Token, TokenKind,
};
use super::vocab::Vocab;

/// A detected line is cut at a gap between neighbouring characters (their
/// centres) wider than this many line heights (`SPLIT_GAP`,
/// `pipeline.py:50`).
pub const SPLIT_GAP: f64 = 3.0;
/// Inside a table a line is cut into cells at this smaller gap (`CELL_GAP`,
/// `pipeline.py:51`).
pub const CELL_GAP: f64 = 1.8;
/// A gap this many line heights wide is cut when other lines' edges line up
/// with it, a column gutter (`GUTTER_GAP`, `pipeline.py:52`).
pub const GUTTER_GAP: f64 = 1.6;
/// How many other lines' edges must line up with a gutter, within half a
/// line height (`GUTTER_EDGES`, `pipeline.py:53`).
pub const GUTTER_EDGES: usize = 3;
/// Form fill-in dots and rules, specks (`LEADER_CHARS`, `pipeline.py:916`).
const LEADER_CHARS: &str = ".·…‥_-–—=";
/// A lone short dash can be a table cell (`DASHES`).
const DASHES: &str = "-–—";
/// List bullets (`BULLETS`).
const BULLETS: &str = "•◦▪▫●○■□‣⁃∙*";
/// A list marker and its item's text are at most this many line heights
/// apart (`MARKER_GAP`).
const MARKER_GAP: f32 = 2.5;
/// The characters `layout_post._ITEM` takes for a bullet.
const ITEM_BULLETS: &str = "•-–▪◦*·→";

/// How the base recognizer's class ids read as text: [`Vocab::decode`] and
/// the class id of the space (`vocab.index.get(" ")`).
pub trait IdText {
    fn decode(&self, ids: &[u32]) -> String;
    fn space_id(&self) -> Option<u32>;
}

impl IdText for Vocab {
    fn decode(&self, ids: &[u32]) -> String {
        Vocab::decode(self, ids)
    }

    fn space_id(&self) -> Option<u32> {
        let index = self.chars.iter().position(|c| c == " ")?;
        u32::try_from(index + 1).ok()
    }
}

/// What may change how a page is written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssembleOptions {
    /// Keep page headers and footers in the markdown (`--keep-furniture`);
    /// they are always among the lines and blocks.
    pub keep_furniture: bool,
}

/// The finished page and, block by block, the table each block was written
/// as (`Some` exactly for the blocks the markdown renders as an HTML
/// table: Table blocks of more than one line that are not display math).
#[derive(Clone, Debug, PartialEq)]
pub struct Assembly {
    pub page: PageReading,
    pub tables: Vec<Option<TableGrid>>,
    /// Block by block, the cells the table was laid out from (`Some`
    /// exactly where `tables` is): each table line split at [`CELL_GAP`],
    /// in the block's line order.
    pub cells: Vec<Option<Vec<TableCell>>>,
}

/// A table cell as [`Reader::block_part`] finds it: the index of the line
/// it was cut from (before ordering), its levelled box, its text.
type RawCell = (usize, [f32; 4], String);

/// One cell of a table line: the index into [`PageReading::lines`] of the
/// line it was cut from, its box on the levelled page (unmirrored, rounded
/// to 0.1 like the lines'), and its text.
#[derive(Clone, Debug, PartialEq)]
pub struct TableCell {
    pub line: usize,
    pub bbox: [f64; 4],
    pub text: String,
}

/// A table as `layout_post.table_html` lays it out: optional title lines
/// above it, then rows of slots across `columns` columns. Every row's
/// colspans add up to `columns`; the first row is a header row when there is
/// more than one row.
#[derive(Clone, Debug, PartialEq)]
pub struct TableGrid {
    /// Lines that run across most of the table above its first row, top
    /// first; the markdown writes them as paragraphs before the table.
    pub titles: Vec<String>,
    pub columns: usize,
    pub rows: Vec<Vec<GridCell>>,
}

/// One slot of a table row: a cell spanning `colspan` columns, or an empty
/// one-column slot (`text` `None`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridCell {
    pub colspan: usize,
    pub text: Option<String>,
}

impl TableGrid {
    /// Whether the first row is a header row (`<th>`).
    pub fn has_header(&self) -> bool {
        self.rows.len() > 1
    }

    /// The table as the reference's markdown writes it: titles, then
    /// `<table>` with one `<tr>` a line.
    pub fn to_html(&self) -> String {
        let mut out = String::new();
        for title in &self.titles {
            out.push_str(title);
            out.push_str("\n\n");
        }
        let mut lines = vec!["<table>".to_string()];
        for (ri, row) in self.rows.iter().enumerate() {
            let tag = if ri == 0 && self.has_header() {
                "th"
            } else {
                "td"
            };
            let mut tr = String::from("<tr>");
            for cell in row {
                match &cell.text {
                    Some(text) => {
                        let attr = if cell.colspan > 1 {
                            format!(" colspan=\"{}\"", cell.colspan)
                        } else {
                            String::new()
                        };
                        tr.push_str(&format!("<{tag}{attr}>{}</{tag}>", escape_html(text)));
                    }
                    None => tr.push_str(&format!("<{tag}></{tag}>")),
                }
            }
            tr.push_str("</tr>");
            lines.push(tr);
        }
        lines.push("</table>".to_string());
        out.push_str(&lines.join("\n"));
        out
    }
}

/// `OrtOCR.assemble`: the page `width` x `height` (turned and levelled)
/// from its detected boxes and their readings, asking `order` for the
/// order network's outputs given `[n, 16]` features. `order` is not called
/// for fewer than two lines. Boxes are left on the levelled page; the
/// engine maps them back.
pub fn assemble(
    width: usize,
    height: usize,
    boxes: &[DetBox],
    lines: &[RecLine],
    vocab: &Vocab,
    order: &mut dyn FnMut(&[f32], usize) -> OrderOutputs,
) -> PageReading {
    assemble_with(
        width,
        height,
        boxes,
        lines,
        vocab,
        AssembleOptions::default(),
        order,
    )
    .page
}

/// [`assemble`] with options and the table grids, for any [`IdText`].
pub fn assemble_with<D: IdText + ?Sized>(
    width: usize,
    height: usize,
    boxes: &[DetBox],
    lines: &[RecLine],
    ids: &D,
    options: AssembleOptions,
    order: &mut dyn FnMut(&[f32], usize) -> OrderOutputs,
) -> Assembly {
    let width_f = width as f64;
    // A right-to-left page: more right-to-left letters than others.
    let letters = lines
        .iter()
        .flat_map(|l| l.text.chars())
        .filter(|&c| pystr::is_alpha(c));
    let (mut n_rtl, mut n_letters) = (0usize, 0usize);
    for c in letters {
        n_letters += 1;
        n_rtl += usize::from(pystr::is_rtl(c));
    }
    let reader = Reader {
        ids,
        space: ids.space_id(),
        page_rtl: n_rtl > n_letters - n_rtl,
    };

    // Split at wide gaps and at column gutters.
    let det: Vec<[f32; 4]> = boxes.iter().map(|b| [b.x0, b.y0, b.x1, b.y1]).collect();
    let cuts = reader.gutter_cuts(&det, lines);
    let mut split = Vec::new();
    for ((bx, line), at) in det.iter().zip(lines).zip(&cuts) {
        for (b, text, toks) in reader.split_line(*bx, &line.tokens, SPLIT_GAP, at) {
            split.push(Line {
                bx: b,
                text,
                conf: line.conf,
                toks,
            });
        }
    }
    let lines = reader.tidy_lines(split);
    let n = lines.len();
    let texts: Vec<&str> = lines.iter().map(|l| l.text.as_str()).collect();
    let geo: Vec<[f32; 4]> = lines.iter().map(|l| reader.geo(l.bx, width_f)).collect();

    // Reading order and layout.
    let layout = if n < 2 {
        None
    } else {
        let feats = line_features_f32(&geo, width_f, height as f64, Some(&texts));
        let outputs = order(&feats, n);
        assert!(
            outputs.n == n
                && outputs.a.len() == n * n
                && outputs.same.len() == n * n
                && outputs.cls.len() == n * 10,
            "order network outputs for {} lines, expected {n}",
            outputs.n
        );
        Some(outputs)
    };
    let order: Vec<usize> = match &layout {
        Some(outputs) => decode_blocks_order_f32(outputs, &geo),
        None => order_xycut_f32(&geo),
    };

    let page_lines: Vec<PageLine> = order
        .iter()
        .map(|&i| {
            let l = &lines[i];
            PageLine {
                bbox: l.bx.map(|v| round_py(f64::from(v), 1)),
                text: l.text.clone(),
                conf: round_py(f64::from(l.conf), 4),
            }
        })
        .collect();
    let text = page_lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut blocks = Vec::new();
    let mut tables = Vec::new();
    let mut cells = Vec::new();
    let markdown = match &layout {
        None => texts.join("\n"),
        Some(outputs) => {
            let position: BTreeMap<usize, usize> =
                order.iter().enumerate().map(|(p, &i)| (i, p)).collect();
            let mut parts: Vec<String> = Vec::new();
            for (class, members) in split_blocks(&order, outputs) {
                let union = union_box(members.iter().map(|&i| geo[i]));
                let mut bbox = union.map(|v| round_py(f64::from(v), 1));
                if reader.page_rtl {
                    // geo(np.array(bbox)): mirrored back through float32.
                    let m = [
                        (width_f - bbox[2]) as f32,
                        bbox[1] as f32,
                        (width_f - bbox[0]) as f32,
                        bbox[3] as f32,
                    ];
                    bbox = m.map(|v| round_py(f64::from(v), 1));
                }
                let block_texts: Vec<&str> = members.iter().map(|&i| texts[i]).collect();
                let table =
                    reader.block_part(&mut parts, class, &members, &lines, &geo, width_f, options);
                let (table, table_cells) = match table {
                    Some((grid, found)) => (Some(grid), Some(found)),
                    None => (None, None),
                };
                tables.push(table);
                cells.push(table_cells.map(|found: Vec<RawCell>| {
                    found
                        .into_iter()
                        .map(|(line, bx, text)| TableCell {
                            line: position[&line],
                            bbox: bx.map(|v| round_py(f64::from(v), 1)),
                            text,
                        })
                        .collect()
                }));
                blocks.push(PageBlock {
                    class,
                    bbox,
                    text: block_texts.join("\n"),
                    lines: members.iter().map(|i| position[i]).collect(),
                });
            }
            parts
                .into_iter()
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n")
        }
    };
    Assembly {
        page: PageReading {
            rotation: 0,
            skew: 0.0,
            lines: page_lines,
            blocks,
            markdown,
            text,
        },
        tables,
        cells,
    }
}

/// One line on its way through assembly: its box (`float32`, levelled
/// page), text, the recognizer's confidence and its tokens.
#[derive(Clone, Debug)]
struct Line {
    bx: [f32; 4],
    text: String,
    conf: f32,
    toks: Vec<Token>,
}

/// The parts of `OrtOCR` that assembly reads: the id decoder, the space
/// token, and the page direction.
struct Reader<'a, D: ?Sized> {
    ids: &'a D,
    space: Option<u32>,
    page_rtl: bool,
}

impl<D: IdText + ?Sized> Reader<'_, D> {
    /// `_is_space(tok)`: a space character or the space class.
    fn is_space(&self, tok: &Token) -> bool {
        match &tok.kind {
            TokenKind::Char(s) => s == " ",
            TokenKind::Id(id) => Some(*id) == self.space,
        }
    }

    /// `geo(b)`: on a right-to-left page, the box mirrored left to right.
    fn geo(&self, b: [f32; 4], width: f64) -> [f32; 4] {
        if !self.page_rtl {
            return b;
        }
        let w = width as f32;
        [w - b[2], b[1], w - b[0], b[3]]
    }

    /// `_decode(toks)` (spec §8.2): class ids through the vocabulary;
    /// characters joined with runs of white space squeezed, composed (NFC)
    /// and, with right-to-left letters, put in logical order.
    fn decode(&self, toks: &[Token]) -> String {
        let Some(first) = toks.first() else {
            return String::new();
        };
        if let TokenKind::Char(_) = first.kind {
            let joined: String = toks
                .iter()
                .map(|t| match &t.kind {
                    TokenKind::Char(s) => s.clone(),
                    TokenKind::Id(id) => self.ids.decode(&[*id]),
                })
                .collect();
            let text = pystr::nfc(&pystr::squeeze_spaces(&joined));
            if pystr::has_rtl(&text) {
                return pystr::flip(&text, self.page_rtl.then_some(true));
            }
            return text;
        }
        let ids: Vec<u32> = toks
            .iter()
            .filter_map(|t| match t.kind {
                TokenKind::Id(id) => Some(id),
                TokenKind::Char(_) => None,
            })
            .collect();
        self.ids.decode(&ids)
    }

    /// `gutter_cuts(boxes, rec)` (`pipeline.py:563-582`): per line, the x
    /// positions of the gaps over [`GUTTER_GAP`] line heights where at least
    /// [`GUTTER_EDGES`] other lines end at the gap's left side or start at
    /// its right side.
    fn gutter_cuts(&self, boxes: &[[f32; 4]], lines: &[RecLine]) -> Vec<Vec<f64>> {
        boxes
            .iter()
            .zip(lines)
            .enumerate()
            .map(|(i, (b, line))| {
                let h = py_max_one(b[3] - b[1]);
                let xs: Vec<f64> = line
                    .tokens
                    .iter()
                    .filter(|t| !self.is_space(t))
                    .map(|t| t.x)
                    .collect();
                let mut mine = Vec::new();
                for pair in xs.windows(2) {
                    let (a, c) = (pair[0], pair[1]);
                    if c - a <= GUTTER_GAP * h {
                        continue;
                    }
                    // The gap's edges, and the tolerance, meet float32 boxes.
                    let gl = (a + 0.35 * h) as f32;
                    let gr = (c - 0.35 * h) as f32;
                    let tol = (0.5 * h) as f32;
                    let near = boxes
                        .iter()
                        .enumerate()
                        .filter(|&(j, o)| {
                            j != i && ((o[2] - gl).abs() < tol || (o[0] - gr).abs() < tol)
                        })
                        .count();
                    if near >= GUTTER_EDGES {
                        mine.push((a + c) / 2.0);
                    }
                }
                mine
            })
            .collect()
    }

    /// `split_line(box, toks, gap, at)` (`pipeline.py:602-636`): the line cut
    /// where two neighbouring characters are more than `gap` line heights
    /// apart or a cut in `at` falls between them. One piece comes back as
    /// the line itself; each of several gets its stripped text and a box
    /// around its characters.
    fn split_line(
        &self,
        bx: [f32; 4],
        toks: &[Token],
        gap: f64,
        at: &[f64],
    ) -> Vec<([f32; 4], String, Vec<Token>)> {
        let h = py_max_one(bx[3] - bx[1]);
        let mut at = at.to_vec();
        at.sort_by(f64::total_cmp);
        let mut segs: Vec<Vec<Token>> = vec![Vec::new()];
        let mut prev: Option<f64> = None;
        for tk in toks {
            let x = tk.x;
            if !self.is_space(tk) {
                if let Some(p) = prev
                    && (x - p > gap * h || at.iter().any(|&c| p < c && c < x))
                {
                    segs.push(Vec::new());
                }
                prev = Some(x);
            }
            segs.last_mut().expect("a segment").push(tk.clone());
        }
        if segs.len() == 1 {
            return vec![(bx, self.decode(toks), toks.to_vec())];
        }
        let mut out = Vec::new();
        for sg in segs {
            let text = pystr::strip(&self.decode(&sg)).to_string();
            let xs: Vec<f64> = sg
                .iter()
                .filter(|t| !self.is_space(t))
                .map(|t| t.x)
                .collect();
            if text.is_empty() || xs.is_empty() {
                continue;
            }
            let lo = xs.iter().copied().fold(f64::INFINITY, f64::min) - 0.35 * h;
            let hi = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 0.35 * h;
            // max(box[0], lo) compares in float32, then the box is cast.
            let (lo, hi) = (lo as f32, hi as f32);
            let x0 = if lo > bx[0] { lo } else { bx[0] };
            let x1 = if hi < bx[2] { hi } else { bx[2] };
            out.push(([x0, bx[1], x1, bx[3]], text, sg));
        }
        out
    }

    /// `tidy_lines(boxes, rec, rtl, space)` (`pipeline.py:928-992`): lines of
    /// leader characters dropped, lone list markers joined to their items.
    fn tidy_lines(&self, mut lines: Vec<Line>) -> Vec<Line> {
        let n = lines.len();
        let mut keep = vec![true; n];
        let h: Vec<f32> = lines.iter().map(|l| l.bx[3] - l.bx[1]).collect();
        let w: Vec<f32> = lines.iter().map(|l| l.bx[2] - l.bx[0]).collect();
        for (i, line) in lines.iter().enumerate() {
            let core: Vec<char> = line.text.chars().filter(|&c| c != ' ').collect();
            if !core.is_empty() && core.iter().all(|&c| LEADER_CHARS.contains(c)) {
                let wide = w[i] >= 3.0 * h[i];
                if core.len() >= 3 || wide || !core.iter().any(|&c| DASHES.contains(c)) {
                    keep[i] = false;
                }
            }
        }
        for i in 0..n {
            let t = pystr::strip(&lines[i].text).to_string();
            if !keep[i] || !(is_bullet(&t) || is_enumerator(&t)) {
                continue;
            }
            let mut best: Option<(usize, f32)> = None;
            for j in 0..n {
                if j == i || !keep[j] || pystr::strip(&lines[j].text).is_empty() {
                    continue;
                }
                let (bi, bj) = (lines[i].bx, lines[j].bx);
                let ov = py_min(bi[3], bj[3]) - py_max(bi[1], bj[1]);
                if ov < 0.5 * py_min(h[i], h[j]) {
                    continue;
                }
                let gap = if self.page_rtl {
                    bi[0] - bj[2]
                } else {
                    bj[0] - bi[2]
                };
                if -0.3 * h[i] <= gap
                    && gap <= MARKER_GAP * py_max(h[i], h[j])
                    && best.is_none_or(|(_, g)| gap < g)
                {
                    best = Some((j, gap));
                }
            }
            let Some((best, _)) = best else { continue };
            let marker_toks = lines[i].toks.clone();
            let marker_box = lines[i].bx;
            let target = &mut lines[best];
            let chars = matches!(
                target.toks.first(),
                Some(Token {
                    kind: TokenKind::Char(_),
                    ..
                })
            );
            let space = if chars {
                Some(TokenKind::Char(" ".to_string()))
            } else {
                self.space.map(TokenKind::Id)
            };
            if let (false, Some(first), false, Some(space)) = (
                self.page_rtl,
                marker_toks.first(),
                target.toks.is_empty(),
                space,
            ) && matches!(first.kind, TokenKind::Char(_)) == chars
            {
                let x = f64::from(marker_box[2] + target.bx[0]) / 2.0;
                let mut toks = marker_toks;
                // The joining space is not read, so it is certain.
                toks.push(Token {
                    kind: space,
                    x,
                    p: 1.0,
                });
                toks.append(&mut target.toks);
                target.toks = toks;
            }
            target.text = format!("{t} {}", pystr::strip(&target.text));
            target.bx = [
                py_min(marker_box[0], target.bx[0]),
                py_min(marker_box[1], target.bx[1]),
                py_max(marker_box[2], target.bx[2]),
                py_max(marker_box[3], target.bx[3]),
            ];
            keep[i] = false;
        }
        lines
            .into_iter()
            .zip(keep)
            .filter_map(|(l, k)| k.then_some(l))
            .collect()
    }

    /// `_block_part` (`layout_post.py:347-381`): one block's markdown
    /// appended to `parts`, nothing for page furniture; the table grid when
    /// the block is written as a table.
    #[allow(clippy::too_many_arguments)]
    fn block_part(
        &self,
        parts: &mut Vec<String>,
        class: BlockClass,
        members: &[usize],
        lines: &[Line],
        geo: &[[f32; 4]],
        width: f64,
        options: AssembleOptions,
    ) -> Option<(TableGrid, Vec<RawCell>)> {
        let texts: Vec<&str> = members.iter().map(|&i| lines[i].text.as_str()).collect();
        if matches!(class, BlockClass::PageHeader | BlockClass::PageFooter)
            && !options.keep_furniture
        {
            return None;
        }
        if texts.iter().all(|t| is_whole_math(pystr::strip(t))) {
            let display: Vec<String> = texts
                .iter()
                .map(|t| {
                    let chars: Vec<char> = pystr::strip(t).chars().collect();
                    let inner: String = chars[1..chars.len() - 1].iter().collect();
                    format!("$${inner}$$")
                })
                .collect();
            parts.push(display.join("\n"));
            return None;
        }
        match class {
            BlockClass::SectionHeader => parts.push(format!("## {}", join_lines(&texts))),
            BlockClass::Table if members.len() > 1 => {
                // Inside tables a line splits into cells at CELL_GAP.
                let (mut cell_boxes, mut cell_texts) = (Vec::new(), Vec::new());
                let mut found = Vec::new();
                for &i in members {
                    let cells = self.split_line(lines[i].bx, &lines[i].toks, CELL_GAP, &[]);
                    if cells.is_empty() {
                        cell_boxes.push(geo[i]);
                        cell_texts.push(lines[i].text.clone());
                        found.push((i, lines[i].bx, lines[i].text.clone()));
                    }
                    for (b, text, _) in cells {
                        cell_boxes.push(self.geo(b, width));
                        cell_texts.push(text.clone());
                        found.push((i, b, text));
                    }
                }
                let grid = table_grid(&cell_boxes, &cell_texts);
                parts.push(grid.to_html());
                return Some((grid, found));
            }
            BlockClass::ListGroup => {
                let mut items: Vec<Vec<&str>> = Vec::new();
                for &t in &texts {
                    match items.last_mut() {
                        Some(item) if !is_item_start(t) => item.push(t),
                        _ => items.push(vec![t]),
                    }
                }
                let items: Vec<String> = items.iter().map(|it| join_lines(it)).collect();
                parts.push(items.join("\n"));
            }
            _ => parts.push(join_lines(&texts)),
        }
        None
    }
}

/// `join_lines(texts)` (spec §8.6, `layout_post.py:49-69`): the lines of one
/// paragraph, stripped, empty ones skipped; a word hyphenated at a line end
/// joined ("tumori-" + "genesis"), CJK lines joined without a space, others
/// with one.
pub fn join_lines(texts: &[&str]) -> String {
    let mut out: Vec<char> = Vec::new();
    for t in texts {
        let t: Vec<char> = pystr::strip(t).chars().collect();
        let Some(&t0) = t.first() else { continue };
        let n = out.len();
        if n > 1
            && out[n - 1] == '-'
            && pystr::is_alpha(out[n - 2])
            && pystr::is_alpha(t0)
            && pystr::is_lower(t0)
        {
            out.pop();
            out.extend(t);
        } else if n > 0 && is_cjk(out[n - 1]) && is_cjk(t0) {
            out.extend(t);
        } else {
            if n > 0 {
                out.push(' ');
            }
            out.extend(t);
        }
    }
    out.into_iter().collect()
}

/// `layout_post._cjk`: Han, kana, CJK punctuation and fullwidth forms,
/// written without spaces (Hangul is not).
fn is_cjk(c: char) -> bool {
    matches!(u32::from(c),
        0x3000..=0x30FF | 0x3400..=0x9FFF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF | 0x20000..=0x2FA1F)
}

/// `t in BULLETS`: the whole text is one bullet character.
fn is_bullet(t: &str) -> bool {
    let mut chars = t.chars();
    matches!((chars.next(), chars.next()), (Some(c), None) if BULLETS.contains(c))
}

/// `ENUMERATOR.fullmatch(t)`: `(\d{1,3}|[A-Za-z])[.)]` or
/// `\((\d{1,3}|[A-Za-z]|[ivxlcIVXLC]{1,5})\)`; `\d` is any Unicode decimal
/// digit, as in Python's `re`.
fn is_enumerator(t: &str) -> bool {
    let c: Vec<char> = t.chars().collect();
    let digits = |s: &[char]| (1..=3).contains(&s.len()) && s.iter().all(|&d| pystr::is_decimal(d));
    let letter = |s: &[char]| s.len() == 1 && s[0].is_ascii_alphabetic();
    let roman =
        |s: &[char]| (1..=5).contains(&s.len()) && s.iter().all(|&r| "ivxlcIVXLC".contains(r));
    if c.len() >= 2 && matches!(c[c.len() - 1], '.' | ')') {
        let body = &c[..c.len() - 1];
        if digits(body) || letter(body) {
            return true;
        }
    }
    if c.len() >= 3 && c[0] == '(' && c[c.len() - 1] == ')' {
        let body = &c[1..c.len() - 1];
        return digits(body) || letter(body) || roman(body);
    }
    false
}

/// `_ITEM.match(t)`: `^\s*([•\-–▪◦*·→]|\(?\d{1,3}[.)]|\(?[a-zA-Z][.)])(\s|$)`,
/// a line that starts a list item.
fn is_item_start(t: &str) -> bool {
    let c: Vec<char> = t.chars().skip_while(|&c| pystr::is_space(c)).collect();
    // The marker's length, if one starts the text.
    let marker = || -> Option<usize> {
        let first = *c.first()?;
        if ITEM_BULLETS.contains(first) {
            return Some(1);
        }
        let start = usize::from(first == '(');
        // \(?\d{1,3}[.)]: the digits are greedy; backtracking cannot help,
        // since a shorter run is followed by a digit.
        let digits = c[start..]
            .iter()
            .take(3)
            .take_while(|&&d| pystr::is_decimal(d))
            .count();
        if digits > 0 && matches!(c.get(start + digits), Some('.' | ')')) {
            return Some(start + digits + 1);
        }
        if c.get(start).is_some_and(char::is_ascii_alphabetic)
            && matches!(c.get(start + 1), Some('.' | ')'))
        {
            return Some(start + 2);
        }
        None
    };
    // `(\s|$)`: `$` also matches before a final newline, which is white
    // space anyway.
    marker().is_some_and(|m| c.get(m).is_none_or(|&next| pystr::is_space(next)))
}

/// `_WHOLE_MATH.match(t)`: `^\$[^$]+\$$`, a line that is one formula.
fn is_whole_math(t: &str) -> bool {
    let c: Vec<char> = t.chars().collect();
    c.len() >= 3 && c[0] == '$' && c[c.len() - 1] == '$' && !c[1..c.len() - 1].contains(&'$')
}

/// `html.escape(text, quote=False)`.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The table `layout_post.table_html(boxes, texts)` writes, as a grid
/// (spec §8.7): rows by vertical centre, wrapped cells' rows merged, column
/// bands from the fullest rows, each line in the bands it covers by more
/// than half. `boxes` are the cells' `[x0, y0, x1, y1]` (in the page's
/// reading direction: mirrored on a right-to-left page), `texts` theirs.
pub fn table_grid(boxes: &[[f32; 4]], texts: &[String]) -> TableGrid {
    let mut boxes = boxes.to_vec();
    let mut texts = texts.to_vec();
    let mut titles = Vec::new();
    // A first row that is one line across most of the table is a title.
    let rows = loop {
        let rows = table_rows(&boxes);
        if boxes.is_empty() {
            break rows;
        }
        let right = boxes.iter().map(|b| b[2]).fold(f32::NEG_INFINITY, f32::max);
        let left = boxes.iter().map(|b| b[0]).fold(f32::INFINITY, f32::min);
        let span_x = right - left;
        let widest = rows.iter().map(Vec::len).max().unwrap_or(0);
        if rows.len() > 2 && rows[0].len() == 1 && widest > 1 {
            let t = rows[0][0];
            if boxes[t][2] - boxes[t][0] > 0.6 * span_x {
                titles.push(texts.remove(t));
                boxes.remove(t);
                continue;
            }
        }
        break rows;
    };
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    let full: Vec<&Vec<usize>> = rows.iter().filter(|r| r.len() == width).collect();
    let rows = merge_wrapped(rows.clone(), &boxes);
    // Column bands from the rows with the most cells: those rows rarely span.
    let bands: Vec<[f32; 2]> = (0..width)
        .map(|c| {
            let x0: Vec<f32> = full.iter().map(|r| boxes[r[c]][0]).collect();
            let x1: Vec<f32> = full.iter().map(|r| boxes[r[c]][2]).collect();
            [median_f32(&x0), median_f32(&x1)]
        })
        .collect();
    // Each band widened to the midpoints between neighbours.
    let mut edges = vec![f64::NEG_INFINITY];
    for c in 0..width.saturating_sub(1) {
        edges.push(f64::from((bands[c][1] + bands[c + 1][0]) / 2.0));
    }
    edges.push(f64::INFINITY);
    let mut grid_rows = Vec::with_capacity(rows.len());
    for r in &rows {
        let mut members = r.clone();
        members.sort_by(|&a, &b| {
            boxes[a][1]
                .total_cmp(&boxes[b][1])
                .then(boxes[a][0].total_cmp(&boxes[b][0]))
        });
        let mut cells: BTreeMap<usize, (usize, String)> = BTreeMap::new();
        for k in members {
            let (x0, x1) = (boxes[k][0], boxes[k][2]);
            let first = search_right(&edges, f64::from(x0)) as isize - 1;
            let last = search_right(&edges, f64::from(x1)) as isize - 1;
            let lo = first.max(0) as usize;
            let hi = last.min(width as isize - 1);
            let mut span: Vec<usize> = Vec::new();
            if hi >= lo as isize {
                for (c, &[b0, b1]) in bands.iter().enumerate().take(hi as usize + 1).skip(lo) {
                    if py_min(x1, b1) - py_max(x0, b0) > 0.5 * (b1 - b0) {
                        span.push(c);
                    }
                }
            }
            if span.is_empty() {
                span.push((first.max(0) as usize).min(width - 1));
            }
            let c0 = span[0];
            let entry = match cells.get(&c0) {
                None => (span.len(), texts[k].clone()),
                Some((n, prev)) => ((*n).max(span.len()), format!("{prev} {}", texts[k])),
            };
            cells.insert(c0, entry);
        }
        let mut slots = Vec::new();
        let mut c = 0;
        while c < width {
            match cells.get(&c) {
                Some((n, text)) => {
                    slots.push(GridCell {
                        colspan: *n,
                        text: Some(text.clone()),
                    });
                    c += n;
                }
                None => {
                    slots.push(GridCell {
                        colspan: 1,
                        text: None,
                    });
                    c += 1;
                }
            }
        }
        grid_rows.push(slots);
    }
    TableGrid {
        titles,
        columns: width,
        rows: grid_rows,
    }
}

/// `table_html(boxes, texts)` (spec §8.7): [`table_grid`] written as the
/// reference writes it.
pub fn table_html(boxes: &[[f32; 4]], texts: &[String]) -> String {
    table_grid(boxes, texts).to_html()
}

/// `_rows(boxes)`: cell lines grouped into rows by vertical centre, top to
/// bottom, each row left to right.
fn table_rows(boxes: &[[f32; 4]]) -> Vec<Vec<usize>> {
    let cy: Vec<f32> = boxes.iter().map(|b| (b[1] + b[3]) / 2.0).collect();
    let hs: Vec<f32> = boxes.iter().map(|b| b[3] - b[1]).collect();
    let tol = 0.5 * median_f32(&hs);
    let mut by_cy: Vec<usize> = (0..boxes.len()).collect();
    by_cy.sort_by(|&a, &b| cy[a].total_cmp(&cy[b]));
    let mut rows: Vec<Vec<usize>> = Vec::new();
    for i in by_cy {
        match rows.last_mut() {
            Some(row) => {
                let centres: Vec<f32> = row.iter().map(|&k| cy[k]).collect();
                if (cy[i] - mean_f32(&centres)).abs() < tol {
                    row.push(i);
                } else {
                    rows.push(vec![i]);
                }
            }
            None => rows.push(vec![i]),
        }
    }
    for row in &mut rows {
        row.sort_by(|&a, &b| boxes[a][0].total_cmp(&boxes[b][0]));
    }
    rows
}

/// `_merge_wrapped(rows, boxes)`: the line rows of wrapped cells joined into
/// one table row (a gap under half the median row gap and 0.6 line heights,
/// and fewer lines than the row above).
fn merge_wrapped(rows: Vec<Vec<usize>>, boxes: &[[f32; 4]]) -> Vec<Vec<usize>> {
    if rows.len() < 3 {
        return rows;
    }
    let hs: Vec<f32> = boxes.iter().map(|b| b[3] - b[1]).collect();
    let h = f64::from(median_f32(&hs));
    let gaps: Vec<f64> = rows
        .windows(2)
        .map(|pair| {
            let top = pair[1]
                .iter()
                .map(|&k| boxes[k][1])
                .fold(f32::INFINITY, f32::min);
            let bottom = pair[0]
                .iter()
                .map(|&k| boxes[k][3])
                .fold(f32::NEG_INFINITY, f32::max);
            f64::from(top - bottom)
        })
        .collect();
    let med = median_f64(&gaps);
    let mut out = vec![rows[0].clone()];
    let mut prev = &rows[0];
    for (g, r) in gaps.iter().zip(&rows[1..]) {
        if *g < 0.5 * med && *g < 0.6 * h && r.len() < prev.len() {
            out.last_mut().expect("a row").extend(r.iter().copied());
        } else {
            out.push(r.clone());
        }
        prev = r;
    }
    out
}

/// `np.searchsorted(edges, x, side="right")`: NumPy's binary search, which
/// is the number of edges `<= x` when they are sorted.
fn search_right(edges: &[f64], x: f64) -> usize {
    let (mut lo, mut hi) = (0, edges.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if edges[mid] <= x {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// The box around `boxes` (`b[:, 0].min()`, ...).
fn union_box(boxes: impl Iterator<Item = [f32; 4]>) -> [f32; 4] {
    boxes.fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |u, b| {
            [
                u[0].min(b[0]),
                u[1].min(b[1]),
                u[2].max(b[2]),
                u[3].max(b[3]),
            ]
        },
    )
}

/// `max(1.0, float(d))`: a line height of at least one pixel.
fn py_max_one(d: f32) -> f64 {
    let d = f64::from(d);
    if d > 1.0 { d } else { 1.0 }
}

/// Python's `min(a, b)`: `a` unless `b < a`.
fn py_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}

/// Python's `max(a, b)`: `a` unless `b > a`.
fn py_max(a: f32, b: f32) -> f32 {
    if b > a { b } else { a }
}

/// Python's `round(x, digits)`: the decimal nearest the exact binary value,
/// ties to even, read back as the nearest double. The exact expansion is
/// written out and rounded by hand, so the result does not depend on how
/// the formatter breaks ties.
pub fn round_py(x: f64, digits: usize) -> f64 {
    if !x.is_finite() || x == 0.0 {
        return x;
    }
    // Every finite double's decimal expansion ends within 1074 places.
    let exact = format!("{:.1074}", x.abs());
    let (int_part, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut kept: Vec<u8> = int_part.bytes().chain(frac.bytes().take(digits)).collect();
    let rest = &frac.as_bytes()[digits.min(frac.len())..];
    let round_up = match rest.first() {
        Some(b'5') => {
            rest[1..].iter().any(|&d| d != b'0')
                || kept.last().is_some_and(|&d| (d - b'0') % 2 == 1)
        }
        Some(&d) => d > b'5',
        None => false,
    };
    if round_up {
        let mut k = kept.len();
        loop {
            if k == 0 {
                kept.insert(0, b'1');
                break;
            }
            k -= 1;
            if kept[k] == b'9' {
                kept[k] = b'0';
            } else {
                kept[k] += 1;
                break;
            }
        }
    }
    let split = kept.len() - digits;
    let text = format!(
        "{}{}.{}",
        if x < 0.0 { "-" } else { "" },
        std::str::from_utf8(&kept[..split]).unwrap_or("0"),
        std::str::from_utf8(&kept[split..]).unwrap_or("0"),
    );
    text.parse().unwrap_or(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_like_python() {
        assert_eq!(round_py(61.25, 1), 61.2);
        assert_eq!(round_py(61.35, 1), 61.4); // 61.35 is above the tie in binary
        assert_eq!(round_py(0.125, 2), 0.12);
        assert_eq!(round_py(0.375, 2), 0.38);
        assert_eq!(round_py(9.96, 1), 10.0);
        assert_eq!(round_py(-2.25, 1), -2.2);
        assert_eq!(round_py(f64::from(0.955_399_3_f32), 4), 0.9554);
        assert_eq!(round_py(99.95, 1), 100.0);
    }

    #[test]
    fn joins_paragraph_lines() {
        assert_eq!(join_lines(&["tumori-", "genesis is"]), "tumorigenesis is");
        assert_eq!(join_lines(&["A-", "B"]), "A- B");
        assert_eq!(join_lines(&[" one ", "", "two"]), "one two");
        assert_eq!(join_lines(&["漢字", "かな"]), "漢字かな");
        assert_eq!(join_lines(&["x-", "y"]), "xy");
        assert_eq!(join_lines(&["-", "y"]), "- y");
    }

    #[test]
    fn recognizes_list_markers() {
        for t in ["1.", "12)", "a.", "B)", "(3)", "(b)", "(iv)", "(XII)", "٣."] {
            assert!(is_enumerator(t), "{t}");
        }
        for t in ["1234.", "ab.", "()", "(abcdef)", "1", "(iiiiii)"] {
            assert!(!is_enumerator(t), "{t}");
        }
        for t in ["• item", "  - x", "1. a", "(2) b", "a) c", "3.", "→"] {
            assert!(is_item_start(t), "{t}");
        }
        for t in ["1.5 m", "ab. c", "word", "(ab) x", "1234. x"] {
            assert!(!is_item_start(t), "{t}");
        }
    }

    #[test]
    fn writes_tables() {
        // Two columns, a header row and a wrapped second cell.
        let boxes = [
            [0.0, 0.0, 40.0, 10.0],
            [100.0, 0.0, 140.0, 10.0],
            [0.0, 30.0, 40.0, 40.0],
            [100.0, 30.0, 150.0, 40.0],
            [100.0, 41.0, 150.0, 51.0],
            [0.0, 70.0, 40.0, 80.0],
            [100.0, 70.0, 140.0, 80.0],
        ];
        let texts: Vec<String> = ["Name", "Value", "a", "b", "b2", "c<", "d&"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let grid = table_grid(&boxes, &texts);
        assert_eq!(grid.columns, 2);
        assert_eq!(
            grid.to_html(),
            "<table>\n<tr><th>Name</th><th>Value</th></tr>\n<tr><td>a</td><td>b b2</td></tr>\n\
             <tr><td>c&lt;</td><td>d&amp;</td></tr>\n</table>"
        );
    }

    /// A table block's lines come back cut into the cells its grid was
    /// laid out from, each naming the page line it was cut from.
    #[test]
    fn table_lines_are_cut_into_their_cells() {
        use crate::indra::types::{DetBox, Token, TokenKind};
        let vocab = Vocab::new([" ", "a", "b", "c", "d"].map(String::from).to_vec());
        let token = |id: u32, x: f64| Token {
            kind: TokenKind::Id(id),
            x,
            p: 0.9,
        };
        // Lines 10 high: a 25-pixel gap is past CELL_GAP (18) and short of
        // SPLIT_GAP (30), so each line stays whole and is cut only as a
        // table's.
        let lines: Vec<RecLine> = (0..2)
            .map(|_| RecLine {
                text: "abcd".into(),
                conf: 0.95,
                tokens: vec![
                    token(2, 5.0),
                    token(3, 10.0),
                    token(4, 35.0),
                    token(5, 40.0),
                ],
            })
            .collect();
        let boxes = [0.0_f32, 20.0].map(|y| DetBox {
            x0: 0.0,
            y0: y,
            x1: 50.0,
            y1: y + 10.0,
            score: 0.9,
        });
        let mut order = |_: &[f32], n: usize| {
            let mut cls = vec![0.0; n * 10];
            for line in 0..n {
                cls[line * 10 + 5] = 10.0; // Table
            }
            OrderOutputs {
                n,
                a: vec![0.0, 5.0, -5.0, 0.0],
                cls,
                same: vec![0.0, 5.0, 5.0, 0.0],
            }
        };
        let assembly = assemble_with(
            100,
            100,
            &boxes,
            &lines,
            &vocab,
            AssembleOptions::default(),
            &mut order,
        );
        assert_eq!(assembly.page.blocks.len(), 1);
        assert_eq!(assembly.page.blocks[0].class, BlockClass::Table);
        assert!(assembly.tables[0].is_some());
        let cells = assembly.cells[0].as_ref().expect("the table's cells");
        let found: Vec<(usize, &str)> = cells
            .iter()
            .map(|cell| (cell.line, cell.text.as_str()))
            .collect();
        assert_eq!(found, [(0, "ab"), (0, "cd"), (1, "ab"), (1, "cd")]);
        // Each cell's box is around its characters, inside its line's.
        assert!(cells[0].bbox[2] < cells[1].bbox[0]);
        assert_eq!(cells[1].bbox[1], assembly.page.lines[0].bbox[1]);
        assert_eq!(cells[2].bbox[1], assembly.page.lines[1].bbox[1]);
    }

    #[test]
    fn flips_right_to_left_text() {
        // Expected values from the reference `rtl.flip` (Python 3.12).
        let cases = [
            ("ab \u{5d0}\u{5d1}", None, "ab \u{5d1}\u{5d0}"),
            ("ab \u{5d0}\u{5d1}", Some(true), "\u{5d1}\u{5d0} ab"),
            ("\u{5d0}\u{5d1} ab", None, "\u{5d1}\u{5d0} ab"),
            ("\u{5d0}\u{5d1} ab", Some(true), "ab \u{5d1}\u{5d0}"),
            (
                "(\u{5d0}) 12 \u{5d1}\u{5b8}\u{5d2}",
                None,
                "\u{5d2}\u{5d1}\u{5b8} 12 (\u{5d0})",
            ),
            ("x \u{627}\u{644} (y) 3", None, "x \u{644}\u{627} (y) 3"),
            (
                "x \u{627}\u{644} (y) 3",
                Some(true),
                "y) 3) \u{644}\u{627} x",
            ),
        ];
        for (text, base, want) in cases {
            assert_eq!(pystr::flip(text, base), want, "{text:?} {base:?}");
        }
        assert_eq!(pystr::nfc("\u{1100}\u{1161}\u{11a8}"), "\u{ac01}");
    }
}
