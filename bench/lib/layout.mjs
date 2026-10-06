/// The page model every InternBench document is drawn from.
///
/// A page is a list of drawing items in the order a PDF content stream would
/// paint them: text runs placed at absolute positions, rules, rectangles,
/// images, and pen strokes. The same page serialises to a digital PDF
/// (pdf.mjs) or rasterises to a scanned image (raster.mjs), so a scan and its
/// `ocr_truth` are two views of one model and cannot drift apart.
///
/// Coordinates are PDF points measured from the TOP-left corner, and a text
/// run's `y` is its baseline. Text runs are kept in stream order on purpose:
/// PDFium extracts text in roughly that order, so a two-column page written
/// column by column reads correctly while one written row by row does not -
/// which is exactly what the multi-column fixtures test.
import { assertSupported, textWidth } from './fonts.mjs';

export const PAGE_SIZES = {
  letter: { width: 612, height: 792 },
  a4: { width: 595.28, height: 841.89 },
};

const NBSP = '\u00a0';
const MONTH = '(?:January|February|March|April|May|June|July|August|September|October|November|December|Jan\.?|Feb\.?|Mar\.?|Apr\.?|Jun\.?|Jul\.?|Aug\.?|Sept?\.?|Oct\.?|Nov\.?|Dec\.?)';
/// Dates in words, "March 4, 2026" or "4th day of March, 2026", which the
/// wrapper never breaks: a reader that finds "March" at the end of one line
/// and "4, 2026" at the start of the next has not found the date.
const DATE_PATTERN = new RegExp(`\\b(?:${MONTH} \\d{1,2}(?:st|nd|rd|th)?,? \\d{4}|\\d{1,2}(?:st|nd|rd|th)? (?:day of )?${MONTH},? \\d{4})`, 'g');

export class Page {
  constructor({ width = 612, height = 792, rotate = 0 } = {}) {
    this.width = width;
    this.height = height;
    this.rotate = rotate;
    this.items = [];
  }

  /// Places a run with its left edge at `x`; returns its width.
  text(x, y, text, { face = 'sans', size = 10, render = 0, grey = 0 } = {}) {
    const clean = text.replaceAll(NBSP, ' ');
    assertSupported(clean);
    const width = textWidth(clean, face, size);
    // A run that leaves the page is clipped by every reader and printer; it
    // is always a layout mistake here, never something a document intends.
    if (x < -0.01 || x + width > this.width + 0.01) throw new Error(`text runs off the page: ${JSON.stringify(clean.slice(0, 60))}`);
    if (clean.length) this.items.push({ type: 'text', x, y, text: clean, face, size, render, grey, width });
    return width;
  }

  /// Wrapped text in a box `width` wide starting at baseline `y`; returns
  /// the baseline after the last line.
  textBlock(x, y, width, text, { face = 'sans', size = 9, leading = 1.25, grey = 0 } = {}) {
    let baseline = y;
    for (const line of wrap(text, { width, face, size })) {
      for (const run of line.runs) this.text(x + run.x, baseline, run.text, { face: run.face, size, grey });
      baseline += size * leading;
    }
    return baseline;
  }

  textRight(right, y, text, options = {}) {
    const width = textWidth(text.replaceAll(NBSP, ' '), options.face ?? 'sans', options.size ?? 10);
    return this.text(right - width, y, text, options);
  }

  textCenter(center, y, text, options = {}) {
    const width = textWidth(text.replaceAll(NBSP, ' '), options.face ?? 'sans', options.size ?? 10);
    return this.text(center - width / 2, y, text, options);
  }

  line(x1, y1, x2, y2, { width = 0.5, grey = 0 } = {}) {
    this.items.push({ type: 'line', x1, y1, x2, y2, width, grey });
  }

  /// `fill` and `stroke` are grey levels in [0, 1] (0 black), or null.
  rect(x, y, w, h, { fill = null, stroke = 0, width = 0.5 } = {}) {
    this.items.push({ type: 'rect', x, y, w, h, fill, stroke, width });
  }

  /// A raster image drawn into the box: `{ width, height, bits, data }`,
  /// grey, top row first, `bits` 8 or 1 (1 = white).
  image(x, y, w, h, image) {
    this.items.push({ type: 'image', x, y, w, h, image });
  }

  /// A pen stroke through points (a handwritten signature, a tick).
  stroke(points, { width = 1, grey = 0.1 } = {}) {
    this.items.push({ type: 'path', points, width, grey });
  }
}

/// The text a reader recovers from a page, in stream order: runs on one
/// baseline joined (with a space where there is a visible gap), baselines
/// separated by newlines. This is the "generated text model" gold strings are
/// checked against; it mirrors how PDFium reports a page's text closely
/// enough that a string present here is present in the worker's output.
export function pageText(page, { visibleOnly = false } = {}) {
  const lines = [];
  let current = null;
  for (const item of page.items) {
    if (item.type !== 'text') continue;
    if (visibleOnly && item.render === 3) continue;
    const sameLine = current && Math.abs(item.y - current.y) < Math.max(item.size, current.size) * 0.35;
    if (sameLine) {
      const gap = item.x - current.end;
      current.text += gap > item.size * 0.12 || gap < -item.size ? ` ${item.text}` : item.text;
      current.end = item.x + item.width;
    } else {
      if (current) lines.push(current.text);
      current = { y: item.y, size: item.size, text: item.text, end: item.x + item.width };
    }
  }
  if (current) lines.push(current.text);
  return lines.map((line) => line.replace(/ +$/, '')).join('\n');
}

/// The text a scanner would capture: visible runs in visual order, top to
/// bottom and left to right within a baseline. This is what `ocr_truth`
/// records for a scanned page.
export function visualText(page) {
  const runs = page.items.filter((item) => item.type === 'text' && item.render !== 3 && item.text.trim());
  const sorted = [...runs].sort((a, b) => (Math.abs(a.y - b.y) < 0.01 ? a.x - b.x : a.y - b.y));
  const lines = [];
  let current = null;
  for (const run of sorted) {
    if (current && Math.abs(run.y - current.y) < Math.max(run.size, current.size) * 0.35) {
      current.parts.push(run.text.trim());
    } else {
      if (current) lines.push(current.parts.join(' '));
      current = { y: run.y, size: run.size, parts: [run.text.trim()] };
    }
  }
  if (current) lines.push(current.parts.join(' '));
  return lines.map((line) => line.replace(/\s+/g, ' ').trim()).join('\n');
}

/// Splits rich content into words. Content is a string or a list of
/// segments `{ text, face }`. A word is a run of non-space characters in one
/// face; `gap` says whether a space separated it from the word before (a bold
/// label running straight into regular text has none). Phrases in `keep` have
/// their spaces made non-breaking so they wrap as one word.
/// Straight quotes as a word processor prints them: apostrophes and closing
/// quotes curl right, opening quotes curl left. Scanned pages use these, so
/// that what OCR reads back (it reports curly quotes for printed ones) is
/// what was drawn.
export function curlyQuotes(text) {
  return text
    .replace(/(^|[\s(\[])"/g, '$1\u201c')
    .replace(/"/g, '\u201d')
    .replace(/(^|[\s(\[])'/g, '$1\u2018')
    .replace(/'/g, '\u2019');
}

function tokenize(content, defaultFace, keep, curly = false) {
  const segments = (typeof content === 'string' ? [{ text: content }] : content).map((segment) => (curly ? { ...segment, text: curlyQuotes(segment.text) } : segment));
  const tokens = [];
  let pendingSpace = false;
  for (const segment of segments) {
    const face = segment.face ?? defaultFace;
    let text = segment.text.replace(DATE_PATTERN, (match) => match.replaceAll(' ', NBSP));
    for (const phrase of keep) {
      if (phrase.includes(' ') && text.includes(phrase)) text = text.split(phrase).join(phrase.replaceAll(' ', NBSP));
    }
    for (const part of text.split(/( +)/)) {
      if (!part) continue;
      if (part[0] === ' ') {
        pendingSpace = true;
        continue;
      }
      tokens.push({ text: part, face, gap: tokens.length > 0 && pendingSpace });
      pendingSpace = false;
    }
  }
  return tokens;
}

/// Greedy word wrap with real advance widths. Returns lines, each a list of
/// runs `{ text, face, x }` relative to the line start, plus the line width.
/// A word that is joined to its predecessor never starts a line.
export function wrap(content, { width, face = 'serif', size = 10, firstIndent = 0, indent = 0, keep = [], curly = false }) {
  const lines = [];
  let current = { runs: [], width: 0, indent: firstIndent };
  for (const token of tokenize(content, face, keep, curly)) {
    const tokenWidth = textWidth(token.text, token.face, size);
    let space = token.gap && current.runs.length ? textWidth(' ', token.face, size) : 0;
    if (current.runs.length && token.gap && current.width + space + tokenWidth > width - current.indent + 0.01) {
      lines.push(current);
      current = { runs: [], width: 0, indent };
      space = 0;
    }
    const last = current.runs[current.runs.length - 1];
    if (last && last.face === token.face) last.text += (space ? ' ' : '') + token.text;
    else current.runs.push({ text: token.text, face: token.face, x: current.width + space });
    current.width += space + tokenWidth;
  }
  if (current.runs.length) lines.push(current);
  return lines;
}

/// A document laid out top to bottom across as many pages as it needs, with
/// running headers and footers, optional multi-column frames, and tables
/// that repeat their header row on a new page.
export class Flow {
  constructor({
    size = 'letter',
    margins = {},
    face = 'serif',
    fontSize = 10.5,
    leading = 1.3,
    header = null,
    footer = null,
    keep = [],
    curly = false,
  } = {}) {
    const dimensions = PAGE_SIZES[size] ?? size;
    this.pageWidth = dimensions.width;
    this.pageHeight = dimensions.height;
    this.margins = { top: 72, right: 72, bottom: 72, left: 72, ...margins };
    this.face = face;
    this.fontSize = fontSize;
    this.leading = leading;
    this.header = header;
    this.footer = footer;
    this.keep = [...keep];
    this.curly = curly;
    this.pages = [];
    this.columns = null;
    this.newPage();
  }

  get left() { return this.frame.x; }
  get width() { return this.frame.width; }
  get bottom() { return this.pageHeight - this.margins.bottom - (this.page.reserved ?? 0); }
  get pageNumber() { return this.pages.length; }

  /// Phrases the wrapper must never break across lines (dates, party names,
  /// identifiers): a gold string split over two lines is not "in" the text
  /// any reader recovers.
  keepTogether(...phrases) {
    for (const phrase of phrases.flat()) if (phrase && !this.keep.includes(phrase)) this.keep.push(phrase);
  }

  newPage() {
    this.page = new Page({ width: this.pageWidth, height: this.pageHeight });
    this.page.reserved = 0;
    this.pages.push(this.page);
    this.y = this.margins.top;
    if (this.header) this.header(this.page, { number: this.pages.length, flow: this });
    if (this.columns) {
      this.columns.index = 0;
      this.columns.top = this.y;
    }
    this.frame = this.columns ? this.columnFrame(0) : { x: this.margins.left, width: this.pageWidth - this.margins.left - this.margins.right };
  }

  columnFrame(index) {
    const { count, gap } = this.columns;
    const total = this.pageWidth - this.margins.left - this.margins.right;
    const width = (total - gap * (count - 1)) / count;
    return { x: this.margins.left + index * (width + gap), width };
  }

  /// Moves to the next column, or the next page after the last column.
  nextFrame() {
    if (this.columns && this.columns.index < this.columns.count - 1) {
      this.columns.bottoms[this.columns.index] = this.y;
      this.columns.index += 1;
      this.frame = this.columnFrame(this.columns.index);
      this.y = this.columns.top;
    } else {
      this.newPage();
    }
  }

  ensure(height) {
    if (this.y + height > this.bottom) this.nextFrame();
  }

  space(points) {
    this.y += points;
  }

  pageBreak() {
    this.newPage();
  }

  startColumns(count, gap = 18) {
    this.columns = { count, gap, index: 0, top: this.y, bottoms: [] };
    this.frame = this.columnFrame(0);
  }

  endColumns() {
    const bottoms = [...this.columns.bottoms, this.y];
    this.y = Math.max(...bottoms);
    this.columns = null;
    this.frame = { x: this.margins.left, width: this.pageWidth - this.margins.left - this.margins.right };
  }

  /// A wrapped paragraph. `content` is a string or rich segments.
  paragraph(content, {
    face = this.face,
    size = this.fontSize,
    leading = this.leading,
    indent = 0,
    firstIndent = indent,
    align = 'left',
    before = 0,
    after = size * 0.6,
    grey = 0,
    keepWithNext = 0,
    keep = [],
  } = {}) {
    const lineHeight = size * leading;
    const lines = wrap(content, { width: this.width - indent, face, size, firstIndent: firstIndent - indent, indent: 0, keep: [...this.keep, ...keep], curly: this.curly });
    this.y += before;
    // Orphan control: never leave a single line of a paragraph at a page foot.
    this.ensure(Math.min(lines.length, 2) * lineHeight + keepWithNext);
    for (const line of lines) {
      this.ensure(lineHeight);
      const baseline = this.y + size;
      let start = this.left + indent + line.indent;
      if (align === 'center') start = this.left + indent + (this.width - indent - line.width) / 2;
      if (align === 'right') start = this.left + this.width - line.width;
      for (const run of line.runs) this.page.text(start + run.x, baseline, run.text, { face: run.face, size, grey });
      this.y += lineHeight;
    }
    this.y += after;
    return lines.length;
  }

  heading(text, { level = 1, face = 'sans-bold', size, before, after, align = 'left' } = {}) {
    const sizes = { 1: 14, 2: 11.5, 3: 10.5 };
    const fontSize = size ?? sizes[level] ?? 10.5;
    const spaceBefore = before ?? (level === 1 ? 10 : 8);
    this.y += spaceBefore;
    this.ensure(fontSize * 1.3 + this.fontSize * this.leading * 3);
    this.paragraph(text, { face, size: fontSize, leading: 1.25, after: after ?? fontSize * 0.45, align });
  }

  /// A horizontal rule across the frame.
  rule({ width = 0.6, grey = 0, before = 2, after = 6 } = {}) {
    this.y += before;
    this.ensure(width + after);
    this.page.line(this.left, this.y, this.left + this.width, this.y, { width, grey });
    this.y += after;
  }

  /// A table. Columns: `{ header, width, align }`, widths in points or as
  /// fractions of the frame. Cells are strings or `{ text, face, align }`
  /// and wrap within their column. Text is emitted row by row and, inside a
  /// row, visual line by visual line, the way a well-behaved PDF writer
  /// orders it.
  table(columns, rows, {
    size = 9,
    face = 'sans',
    headerFace = 'sans-bold',
    leading = 1.25,
    padding = 3,
    border = 'grid',
    headerFill = 0.88,
    zebra = null,
    repeatHeader = true,
    after = 8,
    header = true,
    keep = [],
  } = {}) {
    const total = this.width;
    const fixed = columns.reduce((sum, column) => sum + (column.width > 1 ? column.width : 0), 0);
    const fractions = columns.reduce((sum, column) => sum + (column.width <= 1 ? column.width ?? 0 : 0), 0);
    const widths = columns.map((column) => (column.width > 1 ? column.width : ((column.width ?? 0) / (fractions || 1)) * (total - fixed)));
    const keepAll = [...this.keep, ...keep];
    const layoutRow = (cells, rowFace) => cells.map((cell, index) => {
      const spec = typeof cell === 'object' && cell !== null ? cell : { text: String(cell ?? '') };
      const cellFace = spec.face ?? rowFace;
      const lines = spec.text === '' ? [] : wrap(spec.text, { width: widths[index] - padding * 2, face: cellFace, size, keep: keepAll, curly: this.curly });
      return { lines, align: spec.align ?? columns[index].align ?? 'left', fill: spec.fill ?? null };
    });
    const lineHeight = size * leading;
    const rowHeight = (cells) => Math.max(1, ...cells.map((cell) => cell.lines.length)) * lineHeight + padding * 2;
    const drawRow = (cells, { fill = null } = {}) => {
      const height = rowHeight(cells);
      const top = this.y;
      let x = this.left;
      if (fill !== null) this.page.rect(this.left, top, total, height, { fill, stroke: null });
      cells.forEach((cell, index) => {
        if (cell.fill !== null) this.page.rect(x, top, widths[index], height, { fill: cell.fill, stroke: null });
        x += widths[index];
      });
      const maxLines = Math.max(0, ...cells.map((cell) => cell.lines.length));
      for (let lineIndex = 0; lineIndex < maxLines; lineIndex += 1) {
        let cellX = this.left;
        cells.forEach((cell, index) => {
          const line = cell.lines[lineIndex];
          if (line) {
            const inner = widths[index] - padding * 2;
            let start = cellX + padding;
            if (cell.align === 'right') start = cellX + padding + inner - line.width;
            if (cell.align === 'center') start = cellX + padding + (inner - line.width) / 2;
            const baseline = top + padding + lineIndex * lineHeight + size * 0.95;
            for (const run of line.runs) this.page.text(start + run.x, baseline, run.text, { face: run.face, size });
          }
          cellX += widths[index];
        });
      }
      if (border === 'grid') {
        this.page.rect(this.left, top, total, height, { fill: null, stroke: 0, width: 0.5 });
        let lineX = this.left;
        for (let index = 0; index < widths.length - 1; index += 1) {
          lineX += widths[index];
          this.page.line(lineX, top, lineX, top + height, { width: 0.5 });
        }
      } else if (border === 'rules') {
        this.page.line(this.left, top + height, this.left + total, top + height, { width: 0.4, grey: 0.35 });
      }
      this.y += height;
    };
    const headerCells = header ? layoutRow(columns.map((column) => column.header ?? ''), headerFace) : null;
    const drawHeader = () => {
      if (!headerCells) return;
      drawRow(headerCells, { fill: headerFill });
      if (border === 'rules') this.page.line(this.left, this.y, this.left + total, this.y, { width: 0.8 });
    };
    const laidRows = rows.map((row) => layoutRow(row, face));
    this.ensure((headerCells ? rowHeight(headerCells) : 0) + (laidRows[0] ? rowHeight(laidRows[0]) : 0));
    drawHeader();
    laidRows.forEach((cells, index) => {
      if (this.y + rowHeight(cells) > this.bottom) {
        this.nextFrame();
        if (repeatHeader) drawHeader();
      }
      drawRow(cells, { fill: zebra !== null && index % 2 === 1 ? zebra : null });
    });
    this.y += after;
  }

  /// Label/value pairs on lines: "Label: value", label in bold.
  fields(pairs, { size = this.fontSize, labelFace = 'sans-bold', face = this.face, labelWidth = null, after = 6, gap = 2 } = {}) {
    const lineHeight = size * this.leading;
    for (const [label, value] of pairs) {
      this.ensure(lineHeight);
      const baseline = this.y + size;
      if (labelWidth) {
        this.page.text(this.left, baseline, label, { face: labelFace, size });
        const lines = wrap(value, { width: this.width - labelWidth, face, size, keep: this.keep, curly: this.curly });
        lines.forEach((line, index) => {
          for (const run of line.runs) this.page.text(this.left + labelWidth + run.x, baseline + index * lineHeight, run.text, { face: run.face, size });
        });
        this.y += Math.max(1, lines.length) * lineHeight + gap;
      } else {
        this.paragraph([{ text: `${label} `, face: labelFace }, { text: value, face }], { size, after: gap });
      }
    }
    this.y += after;
  }

  /// A bordered box around content drawn by `draw` (a sidebar, a callout).
  /// The box must fit where it starts; `estimate` reserves room for it.
  box(draw, { estimate = 100, padding = 8, fill = 0.94, stroke = 0.3, after = 10 } = {}) {
    this.ensure(estimate);
    const page = this.page;
    const insertAt = page.items.length;
    const top = this.y;
    const saved = this.frame;
    this.frame = { x: saved.x + padding, width: saved.width - padding * 2 };
    this.y += padding;
    draw(this);
    this.y += padding - 4;
    this.frame = saved;
    if (this.page !== page) throw new Error('a box ran over a page break; raise its estimate');
    page.items.splice(insertAt, 0, { type: 'rect', x: saved.x, y: top, w: saved.width, h: this.y - top, fill, stroke, width: 0.6 });
    this.y += after;
  }

  /// A footnote at the foot of the current page. Space is reserved as notes
  /// are added, so body text never runs into them.
  footnote(marker, text, { size = 7.5 } = {}) {
    const page = this.page;
    const width = this.pageWidth - this.margins.left - this.margins.right;
    const lines = wrap(`${marker} ${text}`, { width, face: 'serif', size, keep: this.keep });
    page.footnotes = page.footnotes ?? [];
    page.footnotes.push({ lines, size });
    page.reserved += lines.length * size * 1.25 + (page.footnotes.length === 1 ? 10 : 2);
  }

  /// Finishes every page: footnotes and running footers, which need the
  /// final page count.
  finish() {
    const total = this.pages.length;
    this.pages.forEach((page, index) => {
      if (page.footnotes?.length) {
        let y = this.pageHeight - this.margins.bottom - page.reserved + 6;
        page.line(this.margins.left, y, this.margins.left + 120, y, { width: 0.4 });
        y += 4;
        for (const note of page.footnotes) {
          for (const line of note.lines) {
            y += note.size * 1.25;
            for (const run of line.runs) page.text(this.margins.left + run.x, y, run.text, { face: run.face, size: note.size });
          }
          y += 2;
        }
      }
      if (this.footer) this.footer(page, { number: index + 1, total, flow: this });
      delete page.reserved;
      delete page.footnotes;
    });
    return this.pages;
  }
}

/// A pen stroke that reads as a handwritten signature: a seeded run of
/// loops and slants over a baseline, `width` points wide.
export function signatureStroke(rng, x, baseline, width) {
  const points = [];
  const steps = 26 + rng.int(0, 10);
  let pen = x;
  const step = width / steps;
  for (let index = 0; index <= steps; index += 1) {
    const lift = index % 3 === 0 ? rng.int(4, 12) : rng.int(-3, 6);
    points.push([pen, baseline - lift]);
    pen += step * (0.6 + rng.float() * 0.8);
  }
  points.push([pen + 6, baseline - rng.int(10, 16)]);
  return points;
}
