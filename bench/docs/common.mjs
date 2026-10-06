/// Pieces most document builders share: letterheads, signature blocks,
/// turning laid-out pages into a digital PDF, and packaging a builder's
/// result.
import { pageText, signatureStroke } from '../lib/layout.mjs';
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

/// Packages a builder's output. `text` is one string per page (the text a
/// reader recovers: native text for digital pages, ocr_truth for scanned
/// ones).
export function result({ files, text, ...fields }) {
  return { files, text, document: entry(fields) };
}
