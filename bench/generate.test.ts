// @vitest-environment node
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { inflateSync } from 'node:zlib';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import { PINNED_NODE, generateBench } from './generate.mjs';
import { verifyXref } from './lib/pdf.mjs';
import { inspectPng, png } from './lib/image.mjs';
import { crc32, unzip, zip } from './lib/zip.mjs';
import { Rng } from './lib/rng.mjs';
import { textWidth, unsupportedCharacters } from './lib/fonts.mjs';
import { wrap } from './lib/layout.mjs';
import { addDays, fromDays, isRealDate, toDays } from './lib/format.mjs';

const BENCH = dirname(fileURLToPath(import.meta.url));

// The category vocabulary from the InternBench spec. Every one must be
// carried by at least one document.
const REQUIRED_CATEGORIES = [
  'simple_digital', 'complex_pdf', 'multi_column', 'form', 'invoice', 'table', 'statement', 'purchase_order', 'contract',
  'amendment', 'sow', 'notice', 'hr', 'healthcare', 'financial', 'letter', 'presentation', 'spreadsheet', 'image_only_scan',
  'mixed_scan', 'rotated_scan', 'low_resolution_scan', 'noisy_scan', 'ocr_corrupted', 'pages_5', 'pages_10', 'pages_25',
  'pages_50', 'pages_100', 'middle_fact', 'competing_dates', 'referenced_agreement', 'irrelevant_names', 'date_in_table',
  'layout_parties', 'unusual', 'information_dense', 'email', 'docx', 'pptx', 'xlsx', 'csv', 'tiff', 'png',
];

const DOCUMENT_KEYS = ['id', 'file', 'title', 'kind', 'format', 'text_layer', 'pages', 'categories', 'notes', 'gold', 'ocr_truth'];
const GOLD_KEYS = [
  'document_type', 'acceptable_types', 'document_date', 'acceptable_dates', 'date_role', 'forbidden_dates', 'parties',
  'party_relation', 'acceptable_party_sets', 'party_roles', 'forbidden_parties', 'description_facts', 'description_forbidden',
  'subject_terms', 'expected_readiness', 'evidence',
];

type Evidence = { date_text: string[]; party_text: Record<string, string[]> };
type Gold = {
  document_type: string;
  acceptable_types: string[];
  document_date: string | null;
  acceptable_dates: string[];
  date_role: string | null;
  forbidden_dates: { date: string; why: string }[];
  parties: string[];
  party_relation: string;
  acceptable_party_sets: { parties: string[]; relation: string }[];
  party_roles: { name: string; role: string }[];
  forbidden_parties: { name: string; why: string }[];
  description_facts: string[][];
  description_forbidden: string[];
  subject_terms: string[];
  expected_readiness: string;
  evidence: Evidence;
};
type OcrTruth = { pages: { page: number; text: string }[]; dates: string[]; names: string[]; identifiers: string[] };
type BenchDocument = {
  id: string;
  file: string;
  format: string;
  text_layer: string;
  pages: number;
  categories: string[];
  notes: string;
  gold: Gold;
  ocr_truth: OcrTruth | null;
  clean_text?: string;
};
type Manifest = { files: { file: string; size: number; sha256: string }[] };
type Generated = { gold: { documents: BenchDocument[] }; manifest: Manifest; texts: Record<string, string[]> };

/// Whitespace (including the no-break spaces that keep dates on one line)
/// collapsed, and typographic quotes straightened, so a gold string matches
/// however the page wrapped or typeset it.
function normalise(text: string) {
  return text.replace(/[\u2018\u2019]/g, "'").replace(/[\u201c\u201d]/g, '"').replace(/\s+/g, ' ').trim();
}

/// Case- and punctuation-insensitive form, as the runner compares names.
function loose(text: string) {
  return normalise(text).toLowerCase().replace(/[^a-z0-9& ]+/g, ' ').replace(/\s+/g, ' ').trim();
}

/// The text a document carries: the native text model for digital pages,
/// plus `ocr_truth` for scanned ones.
function carried(document: BenchDocument, texts: Record<string, string[]>) {
  const pages = [...texts[document.id]];
  if (document.ocr_truth && document.text_layer !== 'scan') pages.push(...document.ocr_truth.pages.map((page) => page.text));
  return normalise(pages.join('\n'));
}

/// What a reader of the printed page sees. For an OCR-corrupted document
/// that is the clean text, not the corrupted layer.
function printed(document: BenchDocument, texts: Record<string, string[]>) {
  return document.clean_text !== undefined ? normalise(document.clean_text) : carried(document, texts);
}

async function inventory(root: string) {
  const names = (await readdir(root)).sort();
  return Promise.all(names.map(async (name) => ({ name, sha256: createHash('sha256').update(await readFile(join(root, name))).digest('hex') })));
}

/// Walks a little-endian TIFF and inflates each frame's strip, returning
/// width, height, and whether the pixel data has the size the tags claim.
function readTiff(bytes: Buffer) {
  expect(bytes.subarray(0, 4).toString('latin1')).toBe('II*\0');
  const frames: { width: number; height: number; complete: boolean }[] = [];
  let offset = bytes.readUInt32LE(4);
  while (offset) {
    const count = bytes.readUInt16LE(offset);
    const tags = new Map<number, number>();
    for (let index = 0; index < count; index += 1) {
      const at = offset + 2 + index * 12;
      const type = bytes.readUInt16LE(at + 2);
      tags.set(bytes.readUInt16LE(at), type === 3 ? bytes.readUInt16LE(at + 8) : bytes.readUInt32LE(at + 8));
    }
    const width = tags.get(256) ?? 0;
    const height = tags.get(257) ?? 0;
    const strip = bytes.subarray(tags.get(273), (tags.get(273) ?? 0) + (tags.get(279) ?? 0));
    const pixels = tags.get(259) === 8 ? inflateSync(strip) : strip;
    frames.push({ width, height, complete: pixels.length === width * height });
    offset = bytes.readUInt32LE(offset + 2 + count * 12);
  }
  return frames;
}

/// Lines that recur (digits aside) on at least half the pages - running
/// headers, footers, page numbers - are layout, not content.
function withoutRunningLines(pages: string[]) {
  const mask = (line: string) => line.trim().replace(/\d/g, '0');
  const counts = new Map<string, number>();
  for (const page of pages) for (const line of new Set(page.split('\n').map(mask).filter(Boolean))) counts.set(line, (counts.get(line) ?? 0) + 1);
  return pages.map((page) => page.split('\n').filter((line) => mask(line) && (counts.get(mask(line)) ?? 0) < pages.length / 2).map((line) => line.trim()).join('\n'));
}

function fiveGramRatio(pages: string[]) {
  const words = pages.join(' ').toLowerCase().split(/[^a-z0-9$%.,'-]+/).filter(Boolean);
  const grams: string[] = [];
  for (let index = 0; index + 5 <= words.length; index += 1) grams.push(words.slice(index, index + 5).join(' '));
  return new Set(grams).size / grams.length;
}

let first: string;
let second: string;
let generated: Generated;
let again: Generated;
let documents: BenchDocument[];

beforeAll(async () => {
  first = await mkdtemp(join(tmpdir(), 'internbench-a-'));
  second = await mkdtemp(join(tmpdir(), 'internbench-b-'));
  generated = await generateBench(first);
  again = await generateBench(second);
  documents = generated.gold.documents;
}, 120_000);

afterAll(async () => {
  await Promise.all([first, second].filter(Boolean).map((directory) => rm(directory, { recursive: true, force: true })));
});

describe('InternBench corpus generator', () => {
  it('writes byte-identical files on every run', async () => {
    expect(again.manifest).toEqual(generated.manifest);
    expect(await inventory(second)).toEqual(await inventory(first));
  });

  it('produces exactly the reviewed gold', async () => {
    const committed = JSON.parse(await readFile(join(BENCH, 'gold.json'), 'utf8'));
    expect(JSON.parse(JSON.stringify(generated.gold))).toEqual(committed);
  });

  it.runIf(process.version === PINNED_NODE)('produces exactly the reviewed bytes on the pinned Node', async () => {
    const committed = JSON.parse(await readFile(join(BENCH, 'manifest.json'), 'utf8'));
    expect(generated.manifest).toEqual(committed);
  });

  it('stays within the size budget and lists every file once', async () => {
    const total = generated.manifest.files.reduce((sum, file) => sum + file.size, 0);
    expect(total).toBeLessThan(60 * 1024 * 1024);
    expect(generated.manifest.files.map((file) => file.file).sort()).toEqual(documents.map((document) => document.file).sort());
    expect((await readdir(first)).sort()).toEqual([...documents.map((document) => document.file), 'manifest.json'].sort());
  });

  it('gives every document a well-formed gold entry', () => {
    expect(documents.length).toBeGreaterThanOrEqual(50);
    expect(new Set(documents.map((document) => document.id)).size).toBe(documents.length);
    for (const document of documents) {
      const where = document.id;
      const keys = Object.keys(document);
      expect(keys.slice(0, DOCUMENT_KEYS.length), where).toEqual(DOCUMENT_KEYS);
      expect(keys.slice(DOCUMENT_KEYS.length).every((key) => key === 'clean_text'), where).toBe(true);
      expect(Object.keys(document.gold), where).toEqual(GOLD_KEYS);
      expect(document.file, where).toBe(`${document.id}.${document.format}`);
      expect(document.notes.length, where).toBeGreaterThan(40);
      const { gold } = document;
      const dates = [gold.document_date, ...gold.acceptable_dates, ...gold.forbidden_dates.map((entry) => entry.date)].filter((date): date is string => date !== null);
      for (const date of dates) expect(isRealDate(date), `${where}: ${date}`).toBe(true);
      expect(gold.document_date === null, where).toBe(gold.date_role === null);
      const allowed = [gold.document_date, ...gold.acceptable_dates];
      for (const entry of gold.forbidden_dates) {
        expect(allowed, `${where}: forbidden date ${entry.date} is also accepted`).not.toContain(entry.date);
        expect(entry.why.length, where).toBeGreaterThan(3);
      }
      const forbiddenNames = gold.forbidden_parties.map((entry) => entry.name);
      for (const party of gold.parties) {
        expect(forbiddenNames, `${where}: ${party}`).not.toContain(party);
        expect(gold.party_roles.some((entry) => entry.name === party), `${where}: no role for ${party}`).toBe(true);
        expect(gold.evidence.party_text[party]?.length, `${where}: no party evidence for ${party}`).toBeGreaterThan(0);
      }
      if (gold.document_date) expect(gold.evidence.date_text.length, where).toBeGreaterThan(0);
      for (const set of gold.acceptable_party_sets) for (const party of set.parties) expect(forbiddenNames, `${where}: ${party}`).not.toContain(party);
      expect(['ready', 'needs_review', 'either'], where).toContain(gold.expected_readiness);
      if (['scan', 'mixed'].includes(document.text_layer)) expect(document.ocr_truth, where).not.toBeNull();
      if (document.ocr_truth) {
        expect(document.ocr_truth.pages.length, where).toBeGreaterThan(0);
        for (const page of document.ocr_truth.pages) expect(page.page >= 1 && page.page <= document.pages, where).toBe(true);
      }
    }
  });

  it('only cites evidence the document actually carries', () => {
    for (const document of documents) {
      const where = document.id;
      const text = carried(document, generated.texts);
      const { gold } = document;
      for (const form of gold.evidence.date_text) expect(text.includes(normalise(form)), `${where}: date evidence "${form}"`).toBe(true);
      for (const [party, forms] of Object.entries(gold.evidence.party_text)) {
        for (const form of forms) expect(text.includes(normalise(form)), `${where}: party evidence "${form}" for ${party}`).toBe(true);
      }
      if (document.ocr_truth) {
        const truth = normalise(document.ocr_truth.pages.map((page) => page.text).join('\n'));
        for (const value of [...document.ocr_truth.dates, ...document.ocr_truth.names, ...document.ocr_truth.identifiers]) {
          expect(truth.includes(normalise(value)), `${where}: ocr_truth lists "${value}"`).toBe(true);
        }
      }
    }
  });

  it('only names parties, traps, and facts that appear on the page', () => {
    for (const document of documents) {
      const where = document.id;
      const text = loose(printed(document, generated.texts));
      const { gold } = document;
      const names = [...gold.parties, ...gold.acceptable_party_sets.flatMap((set) => set.parties), ...gold.party_roles.map((entry) => entry.name), ...gold.forbidden_parties.map((entry) => entry.name)];
      for (const name of names) expect(text.includes(loose(name)), `${where}: "${name}"`).toBe(true);
      for (const group of gold.description_facts) expect(group.some((fact) => text.includes(loose(fact))), `${where}: none of ${JSON.stringify(group)}`).toBe(true);
      for (const term of gold.subject_terms) expect(text.includes(loose(term)), `${where}: subject term "${term}"`).toBe(true);
    }
  });

  it('covers every required category', () => {
    const covered = new Set(documents.flatMap((document) => document.categories));
    expect(REQUIRED_CATEGORIES.filter((category) => !covered.has(category))).toEqual([]);
    expect([...covered].filter((category) => !REQUIRED_CATEGORIES.includes(category))).toEqual([]);
  });

  it('makes long documents genuinely information-dense', () => {
    const long = documents.filter((document) => document.categories.some((category) => /^pages_\d+$/.test(category)));
    expect(long.map((document) => document.pages).sort((a, b) => a - b)).toEqual(expect.arrayContaining([5, 10, 25, 50, 100]));
    for (const document of long) {
      expect(document.categories, document.id).toContain(`pages_${document.pages}`);
      const pages = generated.texts[document.id];
      expect(pages.length, document.id).toBe(document.pages);
      const content = withoutRunningLines(pages);
      // No page is another with different numbers on it ...
      const masked = content.map((page) => page.replace(/\d/g, '0'));
      expect(new Set(masked).size / masked.length, `${document.id}: distinct pages`).toBeGreaterThanOrEqual(0.95);
      // ... the prose and tables do not repeat themselves ...
      expect(fiveGramRatio(content), `${document.id}: distinct 5-grams`).toBeGreaterThanOrEqual(0.85);
      // ... and a typical page is substantially full.
      const sizes = content.map((page) => page.length).sort((a, b) => a - b);
      expect(sizes[Math.floor(sizes.length / 2)], `${document.id}: median page length`).toBeGreaterThanOrEqual(900);
    }
  });

  it('draws scanned text only from glyphs the atlas has', () => {
    for (const document of documents.filter((entry) => entry.ocr_truth)) {
      for (const page of document.ocr_truth!.pages) expect(unsupportedCharacters(page.text.replaceAll('\n', ' ')), document.id).toEqual([]);
    }
  });

  it('writes structurally valid PDFs, PNGs, TIFFs, and Office packages', async () => {
    for (const document of documents) {
      const bytes = await readFile(join(first, document.file));
      if (document.format === 'pdf') {
        expect(verifyXref(bytes), document.id).toBe(true);
        expect(bytes.toString('latin1').match(/\/Type \/Page\b/g)?.length, document.id).toBe(document.pages);
      } else if (document.format === 'png') {
        expect(inspectPng(bytes).valid, document.id).toBe(true);
      } else if (document.format === 'tiff') {
        const frames = readTiff(bytes);
        expect(frames.length, document.id).toBe(document.pages);
        for (const frame of frames) expect(frame.complete, document.id).toBe(true);
      } else if (['docx', 'pptx', 'xlsx'].includes(document.format)) {
        const entries = await unzip(bytes);
        expect(entries.has('[Content_Types].xml'), document.id).toBe(true);
        for (const [name, entry] of entries) expect(entry.valid, `${document.id}: ${name}`).toBe(true);
      }
    }
  });
});

describe('InternBench generator libraries', () => {
  it('PDF xref offsets point at their objects', async () => {
    const bytes = await readFile(join(first, documents.find((document) => document.format === 'pdf')!.file));
    expect(verifyXref(bytes)).toBe(true);
    const shifted = Buffer.concat([Buffer.from('%junk\n'), bytes]);
    expect(verifyXref(shifted)).toBe(false);
  });

  it('PNG chunks carry correct CRCs', () => {
    const image = { width: 3, height: 2, pixels: new Uint8Array([0, 128, 255, 255, 128, 0]) };
    const bytes = png(image, { bits: 8, dpi: 300 });
    expect(inspectPng(bytes)).toMatchObject({ valid: true, width: 3, height: 2, bits: 8 });
    const damaged = Buffer.from(bytes);
    damaged[damaged.length - 20] ^= 0xff;
    expect(inspectPng(damaged).valid).toBe(false);
  });

  it('zip entries round-trip with correct CRCs', async () => {
    expect(crc32(Buffer.from('123456789'))).toBe(0xcbf43926);
    const bytes = zip([['a.txt', 'hello hello hello'], ['dir/b.xml', '<b/>']]);
    const entries = await unzip(bytes);
    expect(entries.get('a.txt')).toMatchObject({ valid: true });
    expect(Buffer.from(entries.get('a.txt')!.content).toString()).toBe('hello hello hello');
    expect(Buffer.from(entries.get('dir/b.xml')!.content).toString()).toBe('<b/>');
  });

  it('seeded random streams are reproducible and independent', () => {
    const draw = (rng: Rng) => Array.from({ length: 8 }, () => rng.uint32());
    expect(draw(Rng.from('invoice'))).toEqual(draw(Rng.from('invoice')));
    expect(draw(Rng.from('invoice'))).not.toEqual(draw(Rng.from('invoices')));
    const parent = Rng.from('lease');
    expect(draw(parent.fork('signatures'))).toEqual(draw(Rng.from('lease').fork('signatures')));
    const rng = Rng.from('range');
    for (let index = 0; index < 500; index += 1) {
      const value = rng.int(3, 9);
      expect(value >= 3 && value <= 9 && Number.isInteger(value)).toBe(true);
    }
  });

  it('wraps text within the requested width without losing words', () => {
    const rng = Rng.from('wrap-test');
    const words = ['the', 'Tenant', 'shall', 'pay', 'Base Rent', 'monthly', 'in advance', 'without', 'deduction', 'or', 'offset', 'on the first day', 'of each calendar month'];
    for (let trial = 0; trial < 40; trial += 1) {
      const text = Array.from({ length: rng.int(5, 60) }, () => rng.pick(words)).join(' ') + ' due March 4, 2026.';
      const width = rng.int(120, 460);
      const size = rng.pick([9, 10, 11.5, 12]);
      const lines = wrap(text, { width, face: 'serif', size });
      for (const line of lines) {
        const content = line.runs.map((run: { text: string }) => run.text).join('');
        if (content.includes(' ')) expect(textWidth(content, 'serif', size)).toBeLessThanOrEqual(width + 0.01);
      }
      const rejoined = lines.map((line: { runs: { text: string }[] }) => line.runs.map((run) => run.text).join('')).join(' ');
      expect(rejoined.replace(/\u00a0/g, ' ')).toBe(text);
      expect(lines.some((line: { runs: { text: string }[] }) => line.runs.some((run) => run.text.includes('March\u00a04,\u00a02026')))).toBe(true);
    }
  });

  it('keeps calendar arithmetic exact', () => {
    for (let day = toDays('1999-12-25'); day < toDays('2041-01-07'); day += 1) expect(toDays(fromDays(day))).toBe(day);
    expect(isRealDate('2028-02-29')).toBe(true);
    expect(isRealDate('2026-02-29')).toBe(false);
    expect(isRealDate('2026-13-01')).toBe(false);
    expect(addDays('2026-02-27', 2)).toBe('2026-03-01');
  });
});
