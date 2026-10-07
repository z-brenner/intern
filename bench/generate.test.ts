// @vitest-environment node
import { createHash } from 'node:crypto';
import { cp, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
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
import { result } from './docs/common.mjs';

const BENCH = dirname(fileURLToPath(import.meta.url));

// The category vocabulary from the InternBench spec. Every one must be
// carried by at least one document.
const REQUIRED_CATEGORIES = [
  'simple_digital', 'complex_pdf', 'multi_column', 'form', 'invoice', 'table', 'statement', 'purchase_order', 'contract',
  'amendment', 'sow', 'notice', 'hr', 'healthcare', 'financial', 'letter', 'presentation', 'spreadsheet', 'image_only_scan',
  'mixed_scan', 'rotated_scan', 'low_resolution_scan', 'noisy_scan', 'ocr_corrupted', 'pages_5', 'pages_10', 'pages_25',
  'pages_50', 'pages_100', 'middle_fact', 'competing_dates', 'referenced_agreement', 'irrelevant_names', 'date_in_table',
  'layout_parties', 'unusual', 'information_dense', 'email', 'docx', 'pptx', 'xlsx', 'csv', 'tiff', 'png',
  // Added with the structure measurements.
  'key_value', 'stream_order', 'rotated_page', 'image_region', 'ocr_critical_fields',
];

const ROUTES = ['fast', 'layout', 'ocr', 'ocr_regions'];

const DOCUMENT_KEYS = ['id', 'file', 'title', 'kind', 'format', 'text_layer', 'pages', 'categories', 'notes', 'gold', 'ocr_truth'];
const GOLD_KEYS = [
  'document_type', 'acceptable_types', 'document_date', 'acceptable_dates', 'date_role', 'forbidden_dates', 'parties',
  'party_relation', 'acceptable_party_sets', 'party_roles', 'forbidden_parties', 'description_facts', 'description_forbidden',
  'subject_terms', 'expected_readiness', 'evidence',
];

type Evidence = {
  date_text: string[];
  party_text: Record<string, string[]>;
  date_anchor?: string;
  type_text?: string[];
  identifier_text?: string[];
};
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
type Structure = {
  reading_order?: string[];
  tables?: { rows: string[][] }[];
  key_values?: { key: string; value: string }[];
  expected_routes?: Record<string, string>;
};
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
  structure?: Structure;
  recording?: string;
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

/// What the printed page shows of the part Intern reads. The worker reads
/// only the first frame of a TIFF, so a fact drawn only on a later frame is
/// one no description built from the reading could state.
function readable(document: BenchDocument, texts: Record<string, string[]>) {
  if (document.format === 'tiff') return normalise(texts[document.id][0]);
  return printed(document, texts);
}

/// Whether a page states a fact: its words in order, or - for an amount or
/// other number - the same value however the page groups it.
function states(page: string, fact: string) {
  if (loose(page).includes(loose(fact))) return true;
  const value = fact.replace(/[$,%\s]/g, '');
  if (!/^\d+(\.\d+)?$/.test(value)) return false;
  return (normalise(page).match(/\d[\d,]*(\.\d+)?/g) ?? []).some((number) => Number(number.replace(/,/g, '')) === Number(value));
}

/// Words that state nothing on their own.
const STOP_WORDS = new Set(['a', 'an', 'the', 'of', 'and', 'or', 'for', 'to', 'in', 'on', 'at', 'by', 'with', 'from', 'as', 'per', 'no', 'not', 'will', 'is', 'this', 'that']);
const NUMBER_WORDS = new Set([
  'zero', 'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight', 'nine', 'ten', 'eleven', 'twelve', 'thirteen', 'fourteen', 'fifteen',
  'sixteen', 'seventeen', 'eighteen', 'nineteen', 'twenty', 'thirty', 'forty', 'fifty', 'sixty', 'seventy', 'eighty', 'ninety', 'hundred',
  'first', 'second', 'third', 'fourth', 'fifth', 'sixth', 'seventh', 'eighth', 'ninth', 'tenth', 'eleventh', 'twelfth',
]);

/// Whether a form states a number: digits, or a number spelled out.
function statesNumber(form: string) {
  return /\d/.test(form) || form.toLowerCase().split(/[^a-z]+/).some((word) => NUMBER_WORDS.has(word));
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

  it('rebuilds only the --only documents and keeps the rest of the corpus', async () => {
    const corpus = await mkdtemp(join(tmpdir(), 'internbench-only-'));
    try {
      await cp(first, corpus, { recursive: true });
      const rebuilt = await generateBench(corpus, { only: ['invoice-date-in-table'] });
      expect(rebuilt.manifest.files.map((entry) => entry.file)).toEqual(['invoice-date-in-table.pdf']);
      // Every other document is still there, and the manifest on disk is
      // the full one again, not a list of the one document rebuilt.
      expect(await inventory(corpus)).toEqual(await inventory(first));
      expect(await readFile(join(corpus, 'manifest.json'), 'utf8')).toBe(await readFile(join(first, 'manifest.json'), 'utf8'));
    } finally {
      await rm(corpus, { recursive: true, force: true });
    }
  });

  it('marks the manifest of a corpus that holds only some documents as partial', async () => {
    const corpus = await mkdtemp(join(tmpdir(), 'internbench-partial-'));
    try {
      await generateBench(corpus, { only: ['invoice-date-in-table'] });
      const manifest = JSON.parse(await readFile(join(corpus, 'manifest.json'), 'utf8'));
      expect(manifest.partial).toBe(true);
      expect(manifest.files.map((entry: { file: string }) => entry.file)).toEqual(['invoice-date-in-table.pdf']);
      // A full build over it replaces the files it listed and drops the mark.
      await generateBench(corpus);
      expect(await inventory(corpus)).toEqual(await inventory(first));
    } finally {
      await rm(corpus, { recursive: true, force: true });
    }
  }, 60_000);

  it('refuses to write into a directory that is not an earlier output', async () => {
    const elsewhere = await mkdtemp(join(tmpdir(), 'internbench-elsewhere-'));
    try {
      await writeFile(join(elsewhere, 'notes.txt'), 'not a corpus');
      await expect(generateBench(elsewhere, { only: ['invoice-date-in-table'] })).rejects.toThrow(/holds no InternBench manifest/);
      await expect(generateBench(elsewhere)).rejects.toThrow(/holds no InternBench manifest/);
      expect(await readdir(elsewhere)).toEqual(['notes.txt']);
      await expect(generateBench(BENCH)).rejects.toThrow(/refusing to write the corpus/);
    } finally {
      await rm(elsewhere, { recursive: true, force: true });
    }
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
      const optional = keys.slice(DOCUMENT_KEYS.length);
      expect(optional, where).toEqual(['clean_text', 'structure', 'recording'].filter((key) => optional.includes(key)));
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
      // An OCR-corrupted document is read either way: from its text layer,
      // or again from the image as the printed text. Each evidence form is
      // carried by one of the two, and each reading carries a form of every
      // item, so either reading can be credited.
      const readings = [carried(document, generated.texts)];
      if (document.clean_text !== undefined) readings.push(printed(document, generated.texts));
      const carries = (form: string) => readings.some((text) => text.includes(normalise(form)));
      const { gold } = document;
      for (const form of gold.evidence.date_text) expect(carries(form), `${where}: date evidence "${form}"`).toBe(true);
      for (const [party, forms] of Object.entries(gold.evidence.party_text)) {
        for (const form of forms) expect(carries(form), `${where}: party evidence "${form}" for ${party}`).toBe(true);
      }
      for (const form of [...(gold.evidence.type_text ?? []), ...(gold.evidence.identifier_text ?? [])]) expect(carries(form), `${where}: evidence "${form}"`).toBe(true);
      // The words that define the date are printed on the same page as it.
      const anchor = gold.evidence.date_anchor;
      if (anchor) {
        const pages = generated.texts[document.id].map(normalise);
        expect(pages.some((page) => page.includes(normalise(anchor)) && gold.evidence.date_text.some((form) => page.includes(normalise(form)))), `${where}: date anchor "${anchor}" is not on a page with the date`).toBe(true);
      }
      readings.forEach((text, reading) => {
        if (gold.evidence.date_text.length > 0) expect(gold.evidence.date_text.some((form) => text.includes(normalise(form))), `${where}: reading ${reading} carries no date evidence`).toBe(true);
        for (const [party, forms] of Object.entries(gold.evidence.party_text)) {
          expect(forms.some((form) => text.includes(normalise(form))), `${where}: reading ${reading} carries no evidence for ${party}`).toBe(true);
        }
      });
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
      const read = loose(readable(document, generated.texts));
      const { gold } = document;
      const names = [...gold.parties, ...gold.acceptable_party_sets.flatMap((set) => set.parties), ...gold.party_roles.map((entry) => entry.name), ...gold.forbidden_parties.map((entry) => entry.name)];
      for (const name of names) expect(text.includes(loose(name)), `${where}: "${name}"`).toBe(true);
      // A description is built from what Intern reads, so what it should
      // state must be on a page Intern reads.
      for (const group of gold.description_facts) expect(read.includes(loose(group[0])), `${where}: fact "${group[0]}" is not on a page Intern reads`).toBe(true);
      for (const term of gold.subject_terms) expect(read.includes(loose(term)), `${where}: subject term "${term}"`).toBe(true);
    }
  });

  it('never forbids a fact the document states', () => {
    // description_forbidden holds what a careless reading would assert and
    // the document does not say: a description containing one is scored as
    // stating something false. A value the page prints (an invoice's real
    // subtotal) is true, so it may never be listed.
    const invoice = documents.find((document) => document.id === 'invoice-date-in-table');
    expect(invoice && states(printed(invoice, generated.texts), '$4,296.75'), 'the check finds the invoice subtotal it was written for').toBe(true);
    for (const document of documents) {
      const text = printed(document, generated.texts);
      for (const fact of document.gold.description_forbidden) {
        expect(states(text, fact), `${document.id}: description_forbidden "${fact}" is printed on the document`).toBe(false);
      }
    }
  });

  it('lists only specific, stated spellings of each description fact', () => {
    // A fact counts as covered when any of its forms is in the description
    // (a substring, ignoring case), so every form must say the fact itself:
    // not a stop word, not a word the document type already says, not
    // another of the document's subject terms (which any description of it
    // uses), and - for a fact that is a number - not the number's label
    // without the number ("Loan No" for a loan number, "vendor" for "32
    // vendors").
    for (const document of documents) {
      const where = document.id;
      const { gold } = document;
      const typeWords = new Set(loose(gold.document_type).split(' '));
      const subjects = new Set(gold.subject_terms.map(loose));
      for (const group of gold.description_facts) {
        const [primary, ...alternates] = group;
        const forms = group.map((form) => normalise(form).toLowerCase());
        expect(new Set(forms).size, `${where}: ${JSON.stringify(group)} repeats a form (matching ignores case)`).toBe(forms.length);
        for (const form of group) {
          const words = loose(form).split(' ').filter(Boolean);
          expect(words.every((word) => STOP_WORDS.has(word) || typeWords.has(word)), `${where}: "${form}" says no more than the type "${gold.document_type}"`).toBe(false);
        }
        for (const alternate of alternates) {
          // A shorter spelling of the primary ("Basalt Telemetry" for
          // "Basalt Telemetry Inc.") states it, whatever else it is.
          const shortening = loose(primary).includes(loose(alternate));
          if (!shortening) expect(subjects.has(loose(alternate)), `${where}: "${alternate}" is a subject term, not a statement of "${primary}"`).toBe(false);
          if (!statesNumber(primary)) continue;
          // An address may drop its house number ("Gristmill Lane"); any
          // other spelling of a number fact keeps the number.
          const street = shortening && loose(alternate).split(' ').length >= 2;
          expect(statesNumber(alternate) || street, `${where}: "${alternate}" drops the number "${primary}" states`).toBe(true);
        }
      }
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
    // A pages_N category is the bucket the page count falls in: 5, 10, 25,
    // 50 or 100 pages and up to the next.
    const bucket = (pages: number) => [100, 50, 25, 10, 5].find((floor) => pages >= floor);
    for (const document of long) {
      expect(document.categories, document.id).toContain(`pages_${bucket(document.pages)}`);
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

/// Every content stream of a PDF, inflated, in file order.
function contentStreams(bytes: Buffer) {
  const text = bytes.toString('latin1');
  const streams: string[] = [];
  const pattern = /<< ([^>]*?)\/Length (\d+) >>\nstream\n/g;
  let match;
  while ((match = pattern.exec(text))) {
    const start = match.index + match[0].length;
    const data = bytes.subarray(start, start + Number(match[2]));
    if (match[1].includes('/Subtype /Image')) continue;
    streams.push(inflateSync(data).toString('latin1'));
  }
  return streams;
}

/// The text-showing operators of a content stream, in stream order.
function shown(stream: string) {
  return stream.split('\n').filter((line) => line.endsWith(' Tj'));
}

function normalisedText(text: string) {
  return normalise(text).replace(/[‐-—]/g, '-');
}

/// The documents added with the structure gold. They were pending a
/// recording until the phase 2 live run recorded them.
const ADDED_FOR_STRUCTURE = [
  'newsletter-three-column', 'agreement-two-column-footnotes', 'meeting-notice-columns', 'meeting-notice-interleaved',
  'meeting-notice-reversed', 'rate-confirmation-rotated', 'inspection-log-ruled-2p', 'price-list-unruled',
  'invoice-label-above', 'invoice-right-aligned', 'invoice-boxed-grid', 'benefits-change-checkbox-form',
  'loss-notice-boxed-fields', 'scan-rotated-page-in-pdf', 'scan-cancellation-notice-150dpi', 'scan-remittance-advice-120dpi',
  'scan-mixed-middle-page', 'mixed-signature-region', 'scan-certificate-of-insurance', 'scan-bill-of-lading',
];

/// The long documents added for phase 3, pending a recording.
const ADDED_FOR_RETRIEVAL = [
  'msa-effective-date-in-definitions-12p', 'industrial-lease-dated-in-schedule-25p', 'term-loan-parties-apart-40p',
  'property-policy-declarations-mid-60p', 'watershed-monitoring-report-100p',
] as const;

describe('InternBench structure gold and the documents added for it', () => {
  const added = () => documents.filter((document) => ADDED_FOR_STRUCTURE.includes(document.id));

  it('adds twenty documents with full gold and a structure block, every one now recorded', () => {
    expect(added().length).toBe(20);
    // Only the documents added since the phase 2 run wait for a recording.
    expect(documents.filter((document) => document.recording === 'pending').map((document) => document.id)).toEqual([...ADDED_FOR_RETRIEVAL]);
    for (const document of added()) {
      expect(document.structure, document.id).toBeDefined();
      expect(document.gold.document_date, document.id).not.toBeNull();
      expect(document.gold.forbidden_dates.length, `${document.id}: traps`).toBeGreaterThan(0);
      expect(document.gold.parties.length, document.id).toBeGreaterThan(0);
      if (['scan', 'mixed'].includes(document.text_layer)) expect(document.ocr_truth, document.id).not.toBeNull();
    }
    // The recordings of record never hold a pending document.
    expect(documents.filter((document) => document.recording !== undefined && document.recording !== 'pending')).toEqual([]);
  });

  it('gives structure only in the documented shape, every string printed and every snippet once', () => {
    const withStructure = documents.filter((document) => document.structure);
    expect(withStructure.length).toBeGreaterThanOrEqual(28);
    for (const document of withStructure) {
      const where = document.id;
      const layout = document.structure!;
      expect(Object.keys(layout).every((key) => ['reading_order', 'tables', 'key_values', 'expected_routes'].includes(key)), where).toBe(true);
      for (const [page, route] of Object.entries(layout.expected_routes ?? {})) {
        expect(ROUTES, `${where}: page ${page}`).toContain(route);
        expect(Number(page) >= 1 && Number(page) <= document.pages, `${where}: page ${page}`).toBe(true);
      }
      if (layout.reading_order) expect(layout.reading_order.length, where).toBeGreaterThanOrEqual(2);
      const text = normalisedText([...generated.texts[document.id], ...(document.ocr_truth?.pages.map((page) => page.text) ?? [])].join('\n'));
      for (const pair of layout.key_values ?? []) {
        expect(text.includes(normalisedText(pair.key)), `${where}: label ${pair.key}`).toBe(true);
        expect(text.includes(normalisedText(pair.value)), `${where}: value ${pair.value}`).toBe(true);
      }
      for (const table of layout.tables ?? []) {
        expect(table.rows.length, where).toBeGreaterThanOrEqual(2);
        for (const cell of table.rows.flat().filter(Boolean)) {
          for (const word of normalisedText(cell).split(' ')) expect(text.includes(word), `${where}: cell ${cell}`).toBe(true);
        }
      }
    }
    // A scanned page is routed to OCR wherever the gold says anything.
    for (const document of withStructure.filter((entry) => entry.text_layer === 'scan')) {
      for (const route of Object.values(document.structure!.expected_routes ?? {})) expect(route, document.id).toBe('ocr');
    }
  });

  it('adds five long documents whose deciding evidence sits deep inside', () => {
    const pagesWith = (document: BenchDocument, needle: string) => generated.texts[document.id].flatMap((page, index) => (normalise(page).includes(normalise(needle)) ? [index + 1] : []));
    // Each document's page count, and the one page its date is printed on.
    const expected: Record<(typeof ADDED_FOR_RETRIEVAL)[number], [number, number]> = {
      'msa-effective-date-in-definitions-12p': [12, 9],
      'industrial-lease-dated-in-schedule-25p': [25, 23],
      'term-loan-parties-apart-40p': [40, 1],
      'property-policy-declarations-mid-60p': [60, 30],
      'watershed-monitoring-report-100p': [100, 88],
    };
    const phase3Roles = new Set(['client', 'contractor', 'employer', 'employee', 'buyer', 'seller', 'landlord', 'tenant', 'issuer', 'recipient', 'vendor', 'customer', 'borrower', 'lender', 'licensor', 'licensee', 'sender', 'addressee', 'other']);
    for (const [id, [pages, datePage]] of Object.entries(expected)) {
      const document = documents.find((entry) => entry.id === id)!;
      expect(document.pages, id).toBe(pages);
      expect(document.recording, id).toBe('pending');
      expect(document.structure, id).toBeDefined();
      expect(document.gold.forbidden_dates.length, `${id}: traps`).toBeGreaterThan(0);
      expect(document.gold.forbidden_parties.length, `${id}: names that are not parties`).toBeGreaterThan(0);
      expect(pagesWith(document, document.gold.evidence.date_text[0]), id).toEqual([datePage]);
      for (const party of document.gold.parties) {
        expect(document.gold.party_roles.some((entry) => entry.name === party && phase3Roles.has(entry.role)), `${id}: ${party} has a role from the phase 3 list`).toBe(true);
      }
    }
    // The loan's parties are printed thirty-nine pages apart, and nowhere between.
    const loan = documents.find((entry) => entry.id === 'term-loan-parties-apart-40p')!;
    for (const party of loan.gold.parties) expect(pagesWith(loan, party), party).toEqual([1, 40]);
  });

  it('refuses a labelled value printed under two occurrences of its label', () => {
    // A change order's DATE box and the architect's signature DATE box,
    // both dated the same day: the scorer could credit either.
    const text = ['CONTRACT DATE\n01/15/2025\nDATE\n05/07/2026\nARCHITECT\nDATE\n05/07/2026'];
    const structure = { key_values: [{ key: 'DATE', value: '05/07/2026' }] };
    expect(() => result({ id: 'twice', files: [], text, structure })).toThrow('the value "05/07/2026" is printed under 2 occurrences of the label "DATE"');
    const changeOrder = documents.find((document) => document.id === 'change-order-form')!;
    expect((changeOrder.structure!.key_values ?? []).map((pair) => pair.key)).not.toContain('DATE');
  });

  it('writes one meeting notice three ways: the same runs, in three content-stream orders', async () => {
    const variants = await Promise.all(['meeting-notice-columns', 'meeting-notice-interleaved', 'meeting-notice-reversed'].map(async (id) => shown(contentStreams(await readFile(join(first, `${id}.pdf`)))[0])));
    const [columns, rows, reverse] = variants;
    expect(columns.length).toBeGreaterThan(40);
    for (const variant of [rows, reverse]) {
      expect([...variant].sort()).toEqual([...columns].sort());
      expect(variant).not.toEqual(columns);
    }
    expect(reverse).toEqual([...columns].reverse());
    const gold = (id: string) => documents.find((document) => document.id === id)!;
    expect(gold('meeting-notice-reversed').gold).toEqual(gold('meeting-notice-columns').gold);
    expect(gold('meeting-notice-interleaved').structure).toEqual(gold('meeting-notice-columns').structure);
  });

  it('stores the rate confirmation\'s landscape page portrait, turned, with /Rotate 90', async () => {
    const bytes = await readFile(join(first, 'rate-confirmation-rotated.pdf'));
    const text = bytes.toString('latin1');
    expect(text).toContain('/MediaBox [0 0 612 792] /Rotate 90');
    expect(contentStreams(bytes)[0].startsWith('q 0 1 -1 0 612 0 cm\n')).toBe(true);
    expect(contentStreams(bytes)[1].startsWith('q 0 1 -1 0')).toBe(false);
  });

  it('prints the amendment\'s signature dates only inside the pasted scan, a sixth of the page or more', async () => {
    const bytes = await readFile(join(first, 'mixed-signature-region.pdf'));
    const streams = contentStreams(bytes);
    expect(streams.join('\n')).not.toContain('August 21, 2026');
    const placed = /q ([\d.]+) 0 0 ([\d.]+) [\d.]+ [\d.]+ cm \/Im0 Do Q/.exec(streams[1]);
    expect(placed).not.toBeNull();
    expect((Number(placed![1]) * Number(placed![2])) / (612 * 792)).toBeGreaterThanOrEqual(0.15);
    const document = documents.find((entry) => entry.id === 'mixed-signature-region')!;
    expect(document.ocr_truth!.dates).toContain('August 21, 2026');
    expect(document.structure!.expected_routes).toEqual({ 1: 'fast', 2: 'ocr_regions' });
  });

  it('scans the low-resolution documents at 150 and 120 DPI', async () => {
    const notice = (await readFile(join(first, 'scan-cancellation-notice-150dpi.pdf'))).toString('latin1');
    expect(notice).toContain('/Width 1275 /Height 1650');
    const receipt = inspectPng(await readFile(join(first, 'scan-remittance-advice-120dpi.png')));
    expect([receipt.width, receipt.height]).toEqual([1020, 1320]);
  });

  it('puts the mixed lease\'s scan between two pages of text, and the claim\'s sideways page in an upright PDF', async () => {
    const lease = contentStreams(await readFile(join(first, 'scan-mixed-middle-page.pdf')));
    expect(lease.map((stream) => shown(stream).length > 0)).toEqual([true, false, true]);
    const claim = (await readFile(join(first, 'scan-rotated-page-in-pdf.pdf'))).toString('latin1');
    expect(claim.match(/\/Width 2550 \/Height 3300/g)?.length).toBe(2);
    const truth = documents.find((entry) => entry.id === 'scan-rotated-page-in-pdf')!.ocr_truth!;
    expect(truth.pages[1].text.startsWith('SCHEDULE OF DAMAGED AND MISSING ITEMS')).toBe(true);
  });

  it('splits the inspection log\'s table across its two pages', () => {
    const document = documents.find((entry) => entry.id === 'inspection-log-ruled-2p')!;
    const rows = document.structure!.tables![0].rows;
    const [one, two] = generated.texts[document.id];
    expect(one).toContain(rows[1][0]);
    expect(two).toContain(rows[rows.length - 1][0]);
    expect(one).not.toContain(rows[rows.length - 1][0]);
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
