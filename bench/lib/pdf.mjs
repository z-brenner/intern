/// A deterministic PDF writer for the page model in layout.mjs.
///
/// Text uses the standard fonts (Helvetica, Helvetica-Bold, Times-Roman,
/// Courier) with WinAnsiEncoding, so nothing is embedded and every byte is a
/// function of the page model. Content streams are Flate-compressed at a
/// fixed level, objects are numbered in a fixed order, and the trailer
/// carries a fixed document ID, so the same pages always make the same file.
import { deflateSync } from 'node:zlib';
import { FACES } from './fonts.mjs';

const FONT_KEYS = { sans: 'F1', 'sans-bold': 'F2', serif: 'F3', mono: 'F4' };

/// WinAnsiEncoding bytes for the non-ASCII characters the atlas carries.
const WIN_ANSI = new Map([
  ['\u2013', 0x96], ['\u2014', 0x97], ['\u2018', 0x91], ['\u2019', 0x92], ['\u201c', 0x93], ['\u201d', 0x94],
  ['\u2022', 0x95], ['\u00e9', 0xe9], ['\u00a9', 0xa9], ['\u00b0', 0xb0], ['\u00bd', 0xbd], ['\u00a7', 0xa7],
]);

/// Formats a coordinate: at most two decimals, no exponent, no "-0".
export function num(value) {
  const rounded = Math.round(value * 100) / 100;
  return Object.is(rounded, -0) ? '0' : String(rounded);
}

/// A PDF string literal for `text`, encoded in WinAnsi.
export function pdfString(text) {
  let out = '(';
  for (const character of text) {
    const code = character.codePointAt(0);
    if (character === '(' || character === ')' || character === '\\') out += `\\${character}`;
    else if (code >= 0x20 && code < 0x7f) out += character;
    else if (WIN_ANSI.has(character)) out += `\\${WIN_ANSI.get(character).toString(8).padStart(3, '0')}`;
    else throw new Error(`character ${JSON.stringify(character)} has no WinAnsi encoding`);
  }
  return `${out})`;
}

function grey(value) {
  return num(value);
}

/// The content stream for one page. PDF space has its origin bottom-left,
/// the page model top-left.
function contentStream(page, imageNames) {
  const height = page.height;
  const ops = [];
  let inText = false;
  let font = null;
  let fill = null;
  let render = 0;
  const endText = () => {
    if (inText) ops.push('ET');
    inText = false;
  };
  for (const item of page.items) {
    if (item.type === 'text') {
      if (!inText) {
        ops.push('BT');
        inText = true;
        font = null;
      }
      const key = `${FONT_KEYS[item.face]} ${num(item.size)}`;
      if (key !== font) {
        ops.push(`/${key} Tf`);
        font = key;
      }
      if (item.grey !== fill) {
        ops.push(`${grey(item.grey)} g`);
        fill = item.grey;
      }
      if (item.render !== render) {
        ops.push(`${item.render} Tr`);
        render = item.render;
      }
      ops.push(`1 0 0 1 ${num(item.x)} ${num(height - item.y)} Tm ${pdfString(item.text)} Tj`);
      continue;
    }
    endText();
    if (item.type === 'line') {
      ops.push(`${num(item.width)} w ${grey(item.grey)} G ${num(item.x1)} ${num(height - item.y1)} m ${num(item.x2)} ${num(height - item.y2)} l S`);
    } else if (item.type === 'rect') {
      const rect = `${num(item.x)} ${num(height - item.y - item.h)} ${num(item.w)} ${num(item.h)} re`;
      if (item.fill !== null && item.fill !== undefined) {
        ops.push(`${grey(item.fill)} g ${rect} f`);
        fill = item.fill;
      }
      if (item.stroke !== null && item.stroke !== undefined) ops.push(`${num(item.width)} w ${grey(item.stroke)} G ${rect} S`);
    } else if (item.type === 'image') {
      ops.push(`q ${num(item.w)} 0 0 ${num(item.h)} ${num(item.x)} ${num(height - item.y - item.h)} cm /${imageNames.get(item.image)} Do Q`);
    } else if (item.type === 'path') {
      const [first, ...rest] = item.points;
      ops.push(`q 1 J 1 j ${num(item.width)} w ${grey(item.grey)} G ${num(first[0])} ${num(height - first[1])} m ${rest.map(([x, y]) => `${num(x)} ${num(height - y)} l`).join(' ')} S Q`);
    }
  }
  endText();
  return ops.join('\n');
}

/// Serialises pages to PDF bytes. Each page is `{ width, height, rotate,
/// items }` as built by layout.mjs; `storeTurned` writes a landscape page
/// as a portrait sheet with /Rotate 90 (see below).
export function buildPdf(pages, { title = null } = {}) {
  const objects = [];
  const reserve = () => {
    objects.push(null);
    return objects.length;
  };
  const set = (id, value) => {
    objects[id - 1] = Buffer.isBuffer(value) ? value : Buffer.from(value, 'latin1');
  };
  const stream = (dictionary, payload) => Buffer.concat([
    Buffer.from(`<< ${dictionary} /Length ${payload.length} >>\nstream\n`, 'latin1'),
    payload,
    Buffer.from('\nendstream', 'latin1'),
  ]);

  const catalogId = reserve();
  const pagesId = reserve();
  const infoId = title ? reserve() : null;
  const fontIds = {};
  for (const [face, key] of Object.entries(FONT_KEYS)) {
    fontIds[key] = reserve();
    set(fontIds[key], `<< /Type /Font /Subtype /Type1 /BaseFont /${FACES[face].pdf} /Encoding /WinAnsiEncoding >>`);
  }
  const fontResources = Object.entries(fontIds).map(([key, id]) => `/${key} ${id} 0 R`).join(' ');

  const pageIds = [];
  for (const page of pages) {
    const pageId = reserve();
    const contentId = reserve();
    pageIds.push(pageId);
    const imageNames = new Map();
    const imageRefs = [];
    for (const item of page.items) {
      if (item.type !== 'image' || imageNames.has(item.image)) continue;
      const name = `Im${imageNames.size}`;
      const imageId = reserve();
      imageNames.set(item.image, name);
      imageRefs.push(`/${name} ${imageId} 0 R`);
      const { width, height, bits, data } = item.image;
      set(imageId, stream(
        `/Type /XObject /Subtype /Image /Width ${width} /Height ${height} /ColorSpace /DeviceGray /BitsPerComponent ${bits} /Filter /FlateDecode`,
        deflateSync(data, { level: 9 }),
      ));
    }
    // A page stored turned: the model is drawn as the page is displayed
    // (landscape), the sheet is stored portrait with /Rotate 90, and the
    // content is turned a quarter counter-clockwise into it, so a viewer
    // that applies /Rotate shows it upright - the way a landscape page
    // printed from a portrait template is written.
    const turned = page.storeTurned === true;
    if (turned && page.rotate !== 90) throw new Error('a stored-turned page carries /Rotate 90');
    const content = turned ? `q 0 1 -1 0 ${num(page.height)} 0 cm\n${contentStream(page, imageNames)}\nQ` : contentStream(page, imageNames);
    set(contentId, stream('/Filter /FlateDecode', deflateSync(Buffer.from(content, 'latin1'), { level: 9 })));
    const resources = `<< /Font << ${fontResources} >>${imageRefs.length ? ` /XObject << ${imageRefs.join(' ')} >>` : ''} >>`;
    const [boxWidth, boxHeight] = turned ? [page.height, page.width] : [page.width, page.height];
    set(pageId, `<< /Type /Page /Parent ${pagesId} 0 R /MediaBox [0 0 ${num(boxWidth)} ${num(boxHeight)}]${page.rotate ? ` /Rotate ${page.rotate}` : ''} /Resources ${resources} /Contents ${contentId} 0 R >>`);
  }
  set(catalogId, `<< /Type /Catalog /Pages ${pagesId} 0 R >>`);
  set(pagesId, `<< /Type /Pages /Kids [${pageIds.map((id) => `${id} 0 R`).join(' ')}] /Count ${pageIds.length} >>`);
  if (infoId) set(infoId, `<< /Title ${pdfString(title)} /Producer (InternBench generator) /CreationDate (D:20260101000000Z) >>`);

  const chunks = [Buffer.from('%PDF-1.7\n%\xe2\xe3\xcf\xd3\n', 'latin1')];
  const offsets = [];
  let length = chunks[0].length;
  objects.forEach((object, index) => {
    if (!object) throw new Error(`PDF object ${index + 1} was reserved but never written`);
    offsets.push(length);
    const chunk = Buffer.concat([Buffer.from(`${index + 1} 0 obj\n`, 'latin1'), object, Buffer.from('\nendobj\n', 'latin1')]);
    chunks.push(chunk);
    length += chunk.length;
  });
  const xref = length;
  const entries = offsets.map((offset) => `${String(offset).padStart(10, '0')} 00000 n \n`).join('');
  chunks.push(Buffer.from(
    `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n${entries}trailer\n<< /Size ${objects.length + 1} /Root ${catalogId} 0 R${infoId ? ` /Info ${infoId} 0 R` : ''} /ID [<496e7465726e42656e63682d76312d30><496e7465726e42656e63682d76312d30>] >>\nstartxref\n${xref}\n%%EOF\n`,
    'latin1',
  ));
  return Buffer.concat(chunks);
}

/// Checks that every xref entry points at the object it names. Used by tests.
export function verifyXref(bytes) {
  const text = bytes.toString('latin1');
  const start = Number(/startxref\n(\d+)\n%%EOF\n$/.exec(text)?.[1]);
  if (!Number.isFinite(start) || !text.startsWith('xref\n', start)) return false;
  const header = /^xref\n0 (\d+)\n/.exec(text.slice(start));
  const count = Number(header[1]);
  let position = start + header[0].length + 20;
  for (let id = 1; id < count; id += 1) {
    const offset = Number(text.slice(position, position + 10));
    if (!text.startsWith(`${id} 0 obj\n`, offset)) return false;
    position += 20;
  }
  return true;
}
