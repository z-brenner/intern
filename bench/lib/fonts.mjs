/// Font metrics shared by the PDF writer and the page rasteriser.
///
/// The glyph atlas (bench/assets/raster-fonts.json) is metric-compatible with
/// the PDF standard fonts: its sans faces have Helvetica's advance widths and
/// its serif face has Times-Roman's. Laying a page out once with these
/// advances therefore places text identically in a digital PDF and on a
/// scanned page drawn from the same page model. Courier is fixed-pitch and
/// exists only in digital documents; nothing scanned may use it.
import { readFileSync } from 'node:fs';
import { inflateSync } from 'node:zlib';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ATLAS_PATH = join(dirname(fileURLToPath(import.meta.url)), '..', 'assets', 'raster-fonts.json');

export const ATLAS = JSON.parse(readFileSync(ATLAS_PATH, 'utf8'));

/// Layout face -> atlas face and PDF base font.
export const FACES = {
  sans: { atlas: 'sans', pdf: 'Helvetica' },
  'sans-bold': { atlas: 'sans-bold', pdf: 'Helvetica-Bold' },
  serif: { atlas: 'serif', pdf: 'Times-Roman' },
  mono: { atlas: null, pdf: 'Courier' },
};

/// Every character a document may contain: the atlas's repertoire, which is
/// also a subset of WinAnsiEncoding, so PDF text needs no embedded font.
export const CHARSET = new Set(Object.keys(ATLAS.faces.sans.glyphs));

export function unsupportedCharacters(text) {
  return [...new Set([...text].filter((character) => !CHARSET.has(character)))];
}

export function assertSupported(text) {
  const missing = unsupportedCharacters(text);
  if (missing.length) {
    throw new Error(`characters outside the glyph atlas: ${missing.map((c) => JSON.stringify(c)).join(' ')} in ${JSON.stringify(text.slice(0, 80))}`);
  }
}

const ADVANCES = new Map();
for (const [face, entry] of Object.entries(FACES)) {
  if (!entry.atlas) continue;
  const atlasFace = ATLAS.faces[entry.atlas];
  const table = new Map();
  for (const [character, glyph] of Object.entries(atlasFace.glyphs)) table.set(character, glyph.advance / atlasFace.units_per_em);
  ADVANCES.set(face, table);
}

/// Advance of one character in ems.
export function advance(character, face) {
  if (face === 'mono') return 0.6;
  const table = ADVANCES.get(face);
  if (!table) throw new Error(`unknown face ${face}`);
  const value = table.get(character === '\u00a0' ? ' ' : character);
  if (value === undefined) throw new Error(`no glyph for ${JSON.stringify(character)} in ${face}`);
  return value;
}

/// Width of a string in points.
export function textWidth(text, face, size) {
  let total = 0;
  for (const character of text) total += advance(character, face);
  return total * size;
}

const DECODED = new Map();

/// A glyph's coverage bitmap (0..15 per pixel, `cell` x `height`), decoded
/// on first use.
export function glyphBitmap(face, character) {
  const atlasFace = FACES[face]?.atlas;
  if (!atlasFace) throw new Error(`face ${face} cannot be rasterised`);
  const key = `${atlasFace}\u0000${character}`;
  let decoded = DECODED.get(key);
  if (!decoded) {
    const glyph = ATLAS.faces[atlasFace].glyphs[character];
    if (!glyph) throw new Error(`no glyph for ${JSON.stringify(character)} in ${atlasFace}`);
    const packed = inflateSync(Buffer.from(glyph.bitmap, 'base64'));
    const width = glyph.cell;
    const pixels = new Uint8Array(width * ATLAS.height);
    for (let index = 0; index < pixels.length; index += 1) {
      const byte = packed[index >> 1];
      pixels[index] = index & 1 ? byte & 15 : byte >> 4;
    }
    decoded = { width, height: ATLAS.height, pixels, advance: glyph.advance / ATLAS.faces[atlasFace].units_per_em };
    DECODED.set(key, decoded);
  }
  return decoded;
}
