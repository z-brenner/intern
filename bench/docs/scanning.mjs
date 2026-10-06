/// Turning laid-out pages into scans: rasterise each page model, apply the
/// degradations a document category calls for, and package the result as a
/// PDF of page images, together with the `ocr_truth` the scorer compares OCR
/// output against.
import { pdfImage, rasterize, threshold } from '../lib/raster.mjs';
import { buildPdf } from '../lib/pdf.mjs';
import { Page, visualText } from '../lib/layout.mjs';

/// Every character on a page that will be scanned must be drawable from the
/// glyph atlas, which has no fixed-pitch face.
export function assertScannable(page) {
  for (const item of page.items) {
    if (item.type === 'text' && item.face === 'mono') throw new Error('scanned pages cannot use the fixed-pitch face');
    if (item.type === 'image') throw new Error('scanned pages are drawn from text and strokes only');
  }
}

/// Rasterises pages and applies `degrade(image, index)` to each. Returns
/// grey images (0..255).
export function scanImages(pages, { dpi, weight = 1, degrade = (image) => image }) {
  return pages.map((page, index) => {
    assertScannable(page);
    return degrade(rasterize(page, dpi, { weight }), index);
  });
}

/// A PDF page that is nothing but the given scan image, the size of the
/// sheet it was scanned from.
export function scanPage(image, { width = 612, height = 792, bits = 1 } = {}) {
  const page = new Page({ width, height });
  page.image(0, 0, width, height, pdfImage(image, bits));
  return page;
}

/// A PDF made of page images. `bits` 1 for bilevel office scans, 8 for grey.
export function scanPdf(images, { width = 612, height = 792, bits = 1, prefix = [] }) {
  const pages = [...prefix, ...images.map((image) => scanPage(bits === 1 ? threshold(image) : image, { width, height, bits }))];
  return buildPdf(pages);
}

/// `ocr_truth` for scanned pages: `scanned` is a list of `{ number, page }`
/// (1-based page numbers in the delivered file). Every listed date, name,
/// and identifier must actually be drawn.
export function truthFor(scanned, { dates = [], names = [], identifiers = [] }) {
  const pages = scanned.map(({ number, page }) => ({ page: number, text: visualText(page) }));
  const all = pages.map((entry) => entry.text).join('\n');
  for (const value of [...dates, ...names, ...identifiers]) {
    if (!all.includes(value)) throw new Error(`ocr_truth lists ${JSON.stringify(value)} but no scanned page draws it`);
  }
  return { pages, dates, names, identifiers };
}
