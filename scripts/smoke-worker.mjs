// Drives a package-shaped parser worker through the fixtures the release
// smoke reads: its hello, native PDF text, PDFium-rendered scans through
// OCR, a rotated low-resolution image, a PDF that mixes the two, a DOCX
// through AnyDoc, a JPEG, and the two documents it must refuse. The port of
// scripts/smoke-worker.ps1, which the Windows job runs, to a script the Linux
// smoke - or anyone with Node - can run.
//
//   node scripts/smoke-worker.mjs --worker PATH --runtime DIR [--fixtures DIR]
//
// DIR is what the worker reads PDFium, ONNX Runtime, the OCR models and
// tessdata from; it is passed as INTERN_RUNTIME_DIR. Run `npm run fixtures`
// first for the default fixtures.
import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const REPOSITORY_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const VERSION = JSON.parse(readFileSync(join(REPOSITORY_ROOT, 'src-tauri/tauri.conf.json'), 'utf8')).version;
const REQUEST_TIMEOUT_MS = 5 * 60 * 1000;

const { values } = parseArgs({
  options: {
    worker: { type: 'string' },
    runtime: { type: 'string' },
    fixtures: { type: 'string', default: join(REPOSITORY_ROOT, 'fixtures/generated') },
  },
});
if (!values.worker || !values.runtime) {
  process.stderr.write('usage: node scripts/smoke-worker.mjs --worker PATH --runtime DIR [--fixtures DIR]\n');
  process.exit(2);
}
const worker = resolve(values.worker);
const runtime = resolve(values.runtime);
const fixtures = resolve(values.fixtures);
const windows = process.platform === 'win32';

for (const required of [windows ? 'pdfium.dll' : 'libpdfium.so', 'tessdata/eng.traineddata', 'tessdata/osd.traineddata']) {
  if (!existsSync(join(runtime, required))) throw new Error(`Package-shaped runtime is missing ${required}`);
}

const child = spawn(worker, [], {
  cwd: runtime,
  env: { ...process.env, INTERN_RUNTIME_DIR: runtime },
  stdio: ['pipe', 'pipe', 'pipe'],
});
let stderr = '';
child.stderr.setEncoding('utf8');
child.stderr.on('data', (chunk) => { stderr += chunk; });
// After the process ends and its output closes, so stderr has all of it.
const exited = new Promise((resolveExit) => child.on('close', (code, signal) => resolveExit({ code, signal })));
const lines = createInterface({ input: child.stdout })[Symbol.asyncIterator]();

async function nextLine(requestId) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error(`Timed out waiting for worker request ${requestId}. stderr: ${stderr}`)), REQUEST_TIMEOUT_MS);
  });
  try {
    const { value, done } = await Promise.race([lines.next(), timeout]);
    if (done) throw new Error(`Worker closed stdout: ${stderr}`);
    return value;
  } finally {
    clearTimeout(timer);
  }
}

async function request(requestId, command) {
  child.stdin.write(`${JSON.stringify({ protocol_version: 1, request_id: requestId, command })}\n`);
  for (;;) {
    const line = await nextLine(requestId);
    if (!line.trim()) continue;
    const envelope = JSON.parse(line);
    if (envelope.protocol_version !== 1 || typeof envelope.event?.type !== 'string') throw new Error(`Worker emitted unsupported protocol: ${line}`);
    if (envelope.request_id !== requestId) throw new Error(`Worker interleaved an unexpected request: ${line}`);
    if (['hello', 'parsed', 'error'].includes(envelope.event.type)) return envelope.event;
  }
}

function stem(file) {
  return file.replace(/\.[^.]+$/, '');
}

async function assertParsed(file, facts, sources, minimumOcrConfidence = 0) {
  const path = join(fixtures, file);
  if (!existsSync(path)) throw new Error(`Fixture is missing: ${file}`);
  const result = await request(`parse-${stem(file)}`, { type: 'parse', path });
  if (result.type !== 'parsed') throw new Error(`Expected ${file} to parse, got ${result.type}: ${JSON.stringify(result)}`);
  const pages = result.document.pages;
  const text = pages.map((page) => page.text).join('\n').toLowerCase();
  for (const fact of facts) {
    if (!text.includes(fact.toLowerCase())) throw new Error(`${file} is missing extracted fact '${fact}': ${text}`);
  }
  const actualSources = new Set(pages.map((page) => page.source));
  for (const source of sources) {
    if (!actualSources.has(source)) throw new Error(`${file} did not exercise ${source} routing; got ${[...actualSources].join(', ')}`);
  }
  // A page read in the wrong orientation still "parses": same word count,
  // plausible shape, useless text. Only confidence tells that from a real
  // reading, so a fixture that exists to prove orientation asserts a floor.
  if (minimumOcrConfidence > 0) {
    const confidences = pages.map((page) => page.ocr_confidence).filter((value) => value !== null && value !== undefined);
    if (!confidences.length) throw new Error(`${file} reported no OCR confidence to check`);
    const best = Math.max(...confidences);
    if (best < minimumOcrConfidence) throw new Error(`${file} read at OCR confidence ${best}, below the ${minimumOcrConfidence} floor: ${text}`);
  }
}

async function assertRejected(file, code = 'PARSE_FAILED') {
  const result = await request(`reject-${stem(file)}`, { type: 'parse', path: join(fixtures, file) });
  if (result.type !== 'error' || result.code !== code) throw new Error(`Expected ${file} to fail with ${code}: ${JSON.stringify(result)}`);
}

try {
  const hello = await request('hello', { type: 'hello' });
  if (hello.type !== 'hello' || hello.worker_version !== VERSION) {
    throw new Error(`Worker hello did not report the release protocol: ${JSON.stringify(hello)}`);
  }
  // The same fixtures and the same facts as scripts/smoke-worker.ps1, which
  // says why the scans assert the words that survive OCR of the clean-room
  // bitmap font rather than its digits.
  await assertParsed('employment-agreement.pdf', ['Mira Vale', 'February 14, 2025'], ['native']);
  await assertParsed('scanned-lease.pdf', ['lease agreement', 'september', 'juniper loop'], ['ocr']);
  await assertParsed('rotated-low-resolution-scan.png', ['delivery receipt', 'june 12', 'violet cartography studio'], ['ocr'], 60);
  await assertParsed('mixed-signature.pdf', ['Aurora Catalog Project', 'January 8, 2025'], ['native', 'ocr']);
  await assertParsed('nda.docx', ['Project Marigold', 'March 3, 2025'], ['any_doc']);
  await assertParsed('document-image.jpg', ['Packing Slip', 'PS-311'], ['ocr']);
  await assertRejected('encrypted.pdf', 'PASSWORD_PROTECTED');
  await assertRejected('malformed.pdf');

  child.stdin.end(`${JSON.stringify({ protocol_version: 1, request_id: 'shutdown', command: { type: 'shutdown' } })}\n`);
  const ended = await Promise.race([exited, new Promise((resolveLate) => setTimeout(resolveLate, 10_000).unref())]);
  if (!ended) throw new Error('Worker did not exit after shutdown');
  if (ended.code !== 0) throw new Error(`Worker exited with ${ended.code ?? ended.signal}: ${stderr}`);
  // A runtime that carries ONNX Runtime and the OCR models reads scans with
  // PP-OCR. If they do not load, the worker says so on stderr and reads with
  // Tesseract instead - which passes every assertion above while shipping
  // the engine the package was not built for.
  const usesPpOcr = existsSync(join(runtime, windows ? 'onnxruntime.dll' : 'libonnxruntime.so'));
  if (usesPpOcr && stderr.includes('OCR_ENGINE_FALLBACK')) {
    throw new Error(`The runtime carries PP-OCR, but the worker fell back to Tesseract: ${stderr}`);
  }
  process.stdout.write(`Package-shaped worker hello, native PDF, PDFium+${usesPpOcr ? 'PP-OCR' : 'Tesseract'} OCR, DOCX/image, and invalid-fixture smoke passed.\n`);
} catch (error) {
  if (child.exitCode === null) child.kill('SIGKILL');
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
}
