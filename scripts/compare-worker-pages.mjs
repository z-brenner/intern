import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { pathToFileURL } from 'node:url';

/**
 * Compares what two builds of the parser worker read from the same files.
 *
 * A page the newer worker read on the fast route is promised to be
 * byte-for-byte the page the older worker read: the same text, from the
 * same source, with the same OCR confidence and page-image flag. Existing
 * prompts and recordings rest on that, so any difference there fails the
 * comparison. Pages on the other routes - rebuilt from geometry, read by
 * OCR - are expected to differ and are only counted. Documents that one
 * worker read and the other refused, or that came back with a different
 * number of pages, warnings, or truncation, fail it too.
 *
 *   node scripts/compare-worker-pages.mjs --before old/intern-worker \
 *     --after new/intern-worker [--runtime DIR] FILE...
 *
 * Each worker is started once and given the files in turn, the way the app
 * uses it. `INTERN_RUNTIME_DIR` (or `--runtime`) must hold PDFium, and
 * Tesseract for scans.
 */
export function compareDocuments(name, before, after) {
  const problems = [];
  const routes = {};
  let identical = 0;
  if (before.type !== after.type) {
    problems.push(`${name}: ${before.type} before, ${after.type} after`);
    return { problems, routes, identical };
  }
  if (before.type !== 'parsed') return { problems, routes, identical };
  const old = before.document;
  const now = after.document;
  if (old.pages.length !== now.pages.length) {
    problems.push(`${name}: ${old.pages.length} pages before, ${now.pages.length} after`);
    return { problems, routes, identical };
  }
  for (const [index, page] of now.pages.entries()) {
    const route = page.layout?.route ?? 'fast';
    routes[route] = (routes[route] ?? 0) + 1;
    if (route !== 'fast') continue;
    const was = old.pages[index];
    for (const key of ['text', 'source', 'ocr_confidence', 'vision_escalated']) {
      if (was[key] !== page[key]) {
        problems.push(`${name} page ${page.page_number}: fast-route ${key} changed`);
      }
    }
    if (was.text === page.text) identical += 1;
  }
  if (JSON.stringify(old.warnings) !== JSON.stringify(now.warnings)) {
    problems.push(`${name}: warnings ${JSON.stringify(old.warnings)} before, ${JSON.stringify(now.warnings)} after`);
  }
  if (old.truncated !== now.truncated) problems.push(`${name}: truncation changed`);
  if ((old.optional_image === null) !== (now.optional_image === null)) {
    problems.push(`${name}: the page image came and went`);
  }
  return { problems, routes, identical };
}

/** Reads every file with one worker process; resolves to {file: event}. */
export async function readWithWorker(worker, files, environment = process.env) {
  const child = spawn(worker, [], { env: environment, stdio: ['pipe', 'pipe', 'ignore'] });
  const lines = createInterface({ input: child.stdout });
  const pending = new Map();
  lines.on('line', (line) => {
    const response = JSON.parse(line);
    const event = response.event;
    if (event.type !== 'parsed' && event.type !== 'error') return;
    pending.get(response.request_id)?.(event);
  });
  const results = {};
  for (const [index, file] of files.entries()) {
    const id = `r${index}`;
    const event = await new Promise((done) => {
      pending.set(id, done);
      child.stdin.write(`${JSON.stringify({ protocol_version: 1, request_id: id, command: { type: 'parse', path: resolve(file) } })}\n`);
    });
    results[file] = event;
  }
  child.stdin.end(`${JSON.stringify({ protocol_version: 1, request_id: 'shutdown', command: { type: 'shutdown' } })}\n`);
  await new Promise((done) => child.on('close', done));
  return results;
}

function argumentsOf(argv) {
  const options = { files: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--before' || value === '--after' || value === '--runtime') {
      options[value.slice(2)] = argv[index + 1];
      index += 1;
    } else {
      options.files.push(value);
    }
  }
  if (!options.before || !options.after || !options.files.length) {
    throw new Error('usage: compare-worker-pages.mjs --before WORKER --after WORKER [--runtime DIR] FILE...');
  }
  return options;
}

async function main() {
  const options = argumentsOf(process.argv.slice(2));
  const environment = { ...process.env };
  if (options.runtime) environment.INTERN_RUNTIME_DIR = options.runtime;
  const before = await readWithWorker(options.before, options.files, environment);
  const after = await readWithWorker(options.after, options.files, environment);
  const routes = {};
  let identical = 0;
  const problems = [];
  for (const file of options.files) {
    const result = compareDocuments(file, before[file], after[file]);
    problems.push(...result.problems);
    identical += result.identical;
    for (const [route, count] of Object.entries(result.routes)) routes[route] = (routes[route] ?? 0) + count;
  }
  console.log(`pages by route after: ${JSON.stringify(routes)}`);
  console.log(`fast-route pages identical to before: ${identical}`);
  for (const problem of problems) console.log(`  ${problem}`);
  if (problems.length) process.exitCode = 1;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 2;
  });
}
