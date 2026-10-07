/// Pieces most document builders share: letterheads, signature blocks,
/// turning laid-out pages into a digital PDF, and packaging a builder's
/// result.
import { Page, pageText, signatureStroke } from '../lib/layout.mjs';
import { buildPdf } from '../lib/pdf.mjs';
import { entry } from '../lib/gold.mjs';

/// A company letterhead across the top of a page: name, then address and
/// contact lines, then a rule. Returns the y below it.
export function letterhead(page, { name, lines = [], x = 72, y = 56, width = 468, align = 'left', size = 17, tagline = null }) {
  const draw = (text, top, options) => {
    if (align === 'center') page.textCenter(x + width / 2, top, text, options);
    else if (align === 'right') page.textRight(x + width, top, text, options);
    else page.text(x, top, text, options);
  };
  draw(name, y + size, { face: 'sans-bold', size });
  let top = y + size + 4;
  if (tagline) {
    top += 10;
    draw(tagline, top, { face: 'sans', size: 8.5, grey: 0.25 });
  }
  for (const line of lines) {
    top += 11;
    draw(line, top, { face: 'sans', size: 8.5, grey: 0.2 });
  }
  top += 8;
  page.line(x, top, x + width, top, { width: 1.2 });
  return top + 18;
}

/// Signature blocks side by side (or stacked when `stacked`). Each block:
/// `{ heading, entity, name, title, date, signer }`. Digital documents sign
/// with "/s/"; scanned ones (`rng` given) get a ruled signature line with a
/// pen stroke across it, as a wet-ink signature looks on a scan.
export function signatureBlocks(flow, blocks, { rng = null, stacked = false, size = 10, dateLabel = 'Date:' } = {}) {
  const lineHeight = size * 1.45;
  const columns = stacked ? 1 : blocks.length;
  const gap = 24;
  const width = (flow.width - gap * (columns - 1)) / columns;
  const rows = (block) => [
    block.heading ? { text: block.heading, face: 'sans-bold' } : null,
    block.entity ? { text: block.entity, face: 'sans-bold' } : null,
    { text: rng ? 'By:' : `By: /s/ ${block.signer ?? block.name}`, face: 'serif', signature: Boolean(rng) },
    { text: `Name: ${block.name}`, face: 'serif' },
    block.title ? { text: `Title: ${block.title}`, face: 'serif' } : null,
    block.date !== undefined ? { text: `${dateLabel} ${block.date}`, face: 'serif' } : null,
  ].filter(Boolean);
  const groups = stacked ? blocks.map((block) => [block]) : [blocks];
  for (const group of groups) {
    const laid = group.map(rows);
    const height = Math.max(...laid.map((lines) => lines.length)) * lineHeight + 22;
    flow.ensure(height + 10);
    for (let line = 0; line < Math.max(...laid.map((lines) => lines.length)); line += 1) {
      const signatureRow = laid.some((lines) => lines[line]?.signature);
      if (signatureRow) flow.y += 12;
      laid.forEach((lines, index) => {
        const row = lines[line];
        if (!row) return;
        const x = flow.left + index * (width + gap);
        const baseline = flow.y + size;
        flow.page.text(x, baseline, row.text, { face: row.face, size });
        if (row.signature && rng) {
          flow.page.line(x + 24, baseline + 2, x + Math.min(width, 230), baseline + 2, { width: 0.6 });
          flow.page.stroke(signatureStroke(rng, x + 34, baseline - 3, 120), { width: 1.1 });
        }
      });
      flow.y += lineHeight;
    }
    flow.y += 14;
  }
}

/// Digital PDF bytes and per-page text model for finished pages.
export function digitalPdf(pages, options = {}) {
  return { bytes: buildPdf(pages, options), text: pages.map((page) => pageText(page)) };
}

/// Text as the structure scorer compares it: whitespace collapsed, quotes
/// straight, dashes hyphens.
export function normaliseText(text) {
  return text
    .replace(/[\u2018\u2019]/g, "'")
    .replace(/[\u201c\u201d]/g, '"')
    .replace(/[\u2010-\u2014]/g, '-')
    .replace(/\s+/g, ' ')
    .trim();
}

const alphanumeric = (character) => character !== undefined && /[\p{L}\p{N}]/u.test(character);
const digit = (character) => character !== undefined && /[0-9]/.test(character);

/// Where `needle` first stands on its own in `haystack` at or after
/// `from`, as the structure scorer finds it (`find_bounded` in
/// crates/intern-bench/src/structure.rs): not continuing a word, nor a
/// number. -1 when it does not.
function boundedAt(haystack, needle, from = 0) {
  if (!needle) return -1;
  const first = needle[0];
  const last = needle[needle.length - 1];
  for (let start = from; start <= haystack.length;) {
    const at = haystack.indexOf(needle, start);
    if (at < 0) return -1;
    const end = at + needle.length;
    const before = haystack[at - 1];
    const after = haystack[end];
    const numberBefore = digit(before) || ((before === '.' || before === ',') && digit(haystack[at - 2]));
    const numberAfter = digit(after) || ((after === '.' || after === ',') && digit(haystack[end + 1]));
    const joinedBefore = (alphanumeric(first) && alphanumeric(before)) || (digit(first) && numberBefore);
    const joinedAfter = (alphanumeric(last) && alphanumeric(after)) || (digit(last) && numberAfter);
    if (!joinedBefore && !joinedAfter) return at;
    start = at + 1;
  }
  return -1;
}

/// How many occurrences of a pair's label carry its value in the text, by
/// the scorer's line rules: the value after the label on its line and
/// before the next label there, or on the next line under a label that
/// stands alone on its line.
function labelsCarrying(lines, labels, key, value) {
  const separators = (part) => /^[\s:|.-]*$/.test(part);
  let carriers = 0;
  lines.forEach((line, index) => {
    for (let at = boundedAt(line, key); at >= 0; at = boundedAt(line, key, at + key.length)) {
      const rest = line.slice(at + key.length);
      const start = boundedAt(rest, value);
      const after = start >= 0 && !labels.some((label) => {
        const labelAt = boundedAt(rest, label);
        return labelAt >= 0 && labelAt + label.length <= start;
      });
      const below = separators(line.slice(0, at)) && separators(rest) && index + 1 < lines.length && boundedAt(lines[index + 1], value) >= 0;
      if (after || below) carriers += 1;
    }
  });
  return carriers;
}

/// Every string a structure block names must be printed on the document -
/// each reading-order snippet exactly once, so its position is unambiguous,
/// and each labelled value under one occurrence of its label, so the
/// scorer cannot credit it to another.
function checkStructure(id, layout, text) {
  const printed = normaliseText(text.join('\n'));
  const occurrences = (value) => printed.split(normaliseText(value)).length - 1;
  for (const snippet of layout.reading_order ?? []) {
    const count = occurrences(snippet);
    if (count !== 1) throw new Error(`${id}: reading-order snippet ${JSON.stringify(snippet)} is printed ${count} times`);
  }
  for (const pair of layout.key_values ?? []) {
    for (const value of [pair.key, pair.value]) {
      if (!occurrences(value)) throw new Error(`${id}: structure names ${JSON.stringify(value)}, which is not printed`);
    }
  }
  const lines = text.join('\n').split('\n').map(normaliseText).filter(Boolean);
  const labels = (layout.key_values ?? []).map((pair) => normaliseText(pair.key)).filter(Boolean);
  for (const pair of layout.key_values ?? []) {
    const carriers = labelsCarrying(lines, labels, normaliseText(pair.key), normaliseText(pair.value));
    if (carriers > 1) throw new Error(`${id}: the value ${JSON.stringify(pair.value)} is printed under ${carriers} occurrences of the label ${JSON.stringify(pair.key)}, so the scorer could credit it to the wrong one`);
  }
  // A table cell that wraps is printed line by line between its
  // neighbours' lines: its words are on the page in order, not together.
  const words = printed.split(' ');
  const inOrder = (value) => {
    let at = 0;
    for (const word of normaliseText(value).split(' ')) {
      at = words.indexOf(word, at);
      if (at < 0) return false;
      at += 1;
    }
    return true;
  };
  for (const cell of (layout.tables ?? []).flatMap((table) => table.rows.flat()).filter(Boolean)) {
    if (!inOrder(cell)) throw new Error(`${id}: structure names the cell ${JSON.stringify(cell)}, which is not printed`);
  }
}

/// Packages a builder's output. `text` is one string per page (the text a
/// reader recovers: native text for digital pages, the ocr_truth text for
/// scanned ones, a mixed page's native text and its scanned region's).
///
/// `structureText`, when given, is the pages' text in reading order, for a
/// document whose stream order is deliberately not.
export function result({ files, text, structureText = null, ...fields }) {
  if (fields.structure) checkStructure(fields.id, fields.structure, structureText ?? text);
  return { files, text, document: entry(fields) };
}

/// Reading-order snippets for a page laid out in reading order: the last
/// two words of a line and the first two of the line after it, at `count`
/// evenly spaced line breaks (lengthened until each is printed once). A
/// snippet that spans a line break is found in a reading only if nothing
/// was read between those two lines - which is what column order, stream
/// order and OCR line order get wrong.
export function readingSnippets(lines, count, { skip = 0 } = {}) {
  const usable = lines.map((line) => line.trim()).filter(Boolean);
  const text = normaliseText(usable.join('\n'));
  const once = (snippet) => text.split(snippet).length === 2;
  const breaks = usable.length - 1 - skip;
  const snippets = [];
  for (let index = 0; index < count; index += 1) {
    const at = skip + Math.floor(((index + 0.5) * breaks) / count);
    const before = normaliseText(usable[at]).split(' ');
    const after = normaliseText(usable[at + 1]).split(' ');
    for (let words = 2; words <= 4; words += 1) {
      const snippet = [...before.slice(-words), ...after.slice(0, words)].join(' ');
      if (once(snippet) && !snippets.includes(snippet)) {
        snippets.push(snippet);
        break;
      }
    }
  }
  if (snippets.length < 2) throw new Error('too few distinct reading-order snippets');
  return snippets;
}

/// The same page with its text written in another content-stream order:
/// `columns` keeps the order it was laid out in, `rows` writes it line by
/// line across the page (top to bottom, left to right), `reverse` writes it
/// last run first. Rules, boxes and images are painted first in every
/// variant, so the variants are the same page to the eye.
export function restream(page, order) {
  const drawing = page.items.filter((item) => item.type !== 'text');
  const runs = page.items.filter((item) => item.type === 'text');
  let ordered = runs;
  if (order === 'rows') ordered = [...runs].sort((a, b) => (Math.abs(a.y - b.y) < 0.01 ? a.x - b.x : a.y - b.y));
  else if (order === 'reverse') ordered = [...runs].reverse();
  else if (order !== 'columns') throw new Error(`unknown stream order ${order}`);
  const copy = new Page({ width: page.width, height: page.height, rotate: page.rotate });
  copy.items = [...drawing, ...ordered];
  return copy;
}
