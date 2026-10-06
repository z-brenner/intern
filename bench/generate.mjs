/// InternBench corpus generator.
///
/// `node bench/generate.mjs` writes every document into bench/generated/
/// with a manifest of sizes and digests, then compares the gold and the
/// manifest it just produced with the reviewed bench/gold.json and
/// bench/manifest.json, failing on any difference. The builders in
/// bench/docs/ are the source of truth; the committed files record what was
/// reviewed. Flags:
///
///   --update-gold   rewrite bench/gold.json and bench/manifest.json (pinned Node only)
///   --out DIR       write somewhere other than bench/generated
///   --only id,id    build only these documents (compared against their committed entries)
///
/// Every document is a pure function of its builder: fixed seeds, fixed
/// timestamps, fixed zlib level, no locale, no clock.
import { createHash } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { BUILDERS } from './docs/index.mjs';

const BENCH_DIRECTORY = dirname(fileURLToPath(import.meta.url));
export const PINNED_NODE = 'v24.15.0';

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

/// Builds the corpus into `outputDirectory`. Returns the gold, the
/// manifest, and each document's text model (keyed by id) for tests.
export async function generateBench(outputDirectory, { only = null } = {}) {
  const root = resolve(outputDirectory);
  await rm(root, { recursive: true, force: true });
  await mkdir(root, { recursive: true });
  const selected = only ? BUILDERS.filter(([id]) => only.includes(id)) : BUILDERS;
  if (only) {
    const unknown = only.filter((id) => !BUILDERS.some(([known]) => known === id));
    if (unknown.length) throw new Error(`unknown document id(s): ${unknown.join(', ')}`);
  }
  const documents = [];
  const files = [];
  const texts = {};
  const seen = new Set();
  for (const [id, build] of selected) {
    const built = build();
    if (built.document.id !== id) throw new Error(`builder ${id} produced ${built.document.id}`);
    if (seen.has(id)) throw new Error(`duplicate document id ${id}`);
    seen.add(id);
    for (const file of built.files) {
      if (file.name !== built.document.file) throw new Error(`${id}: file ${file.name} is not the gold file ${built.document.file}`);
      await writeFile(join(root, file.name), file.bytes);
      files.push({ file: file.name, size: file.bytes.length, sha256: sha256(file.bytes) });
    }
    documents.push(built.document);
    texts[id] = built.text;
  }
  const gold = { schema_version: 1, suite: 'internbench', notice: 'All names, organizations, addresses, identifiers, and events are fictional.', documents };
  const manifest = { schema_version: 1, suite: 'internbench', generator: { node: PINNED_NODE.slice(1) }, files };
  await writeFile(join(root, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
  return { gold, manifest, texts };
}

function differences(expected, actual, key) {
  const before = new Map(expected.map((item) => [item[key], JSON.stringify(item)]));
  const after = new Map(actual.map((item) => [item[key], JSON.stringify(item)]));
  const changed = [];
  for (const [name, value] of after) if (before.get(name) !== value) changed.push(before.has(name) ? `changed: ${name}` : `added: ${name}`);
  for (const name of before.keys()) if (!after.has(name)) changed.push(`missing: ${name}`);
  return changed;
}

async function runCli() {
  const args = process.argv.slice(2);
  const value = (flag) => {
    const index = args.indexOf(flag);
    return index >= 0 ? args[index + 1] : null;
  };
  const updateGold = args.includes('--update-gold');
  const only = value('--only')?.split(',').map((id) => id.trim()).filter(Boolean) ?? null;
  const output = value('--out') ?? join(BENCH_DIRECTORY, 'generated');
  if (updateGold && only) throw new Error('--update-gold rewrites the whole corpus; drop --only');
  const started = process.hrtime.bigint();
  const { gold, manifest } = await generateBench(output, { only });
  const seconds = Number(process.hrtime.bigint() - started) / 1e9;
  const bytes = manifest.files.reduce((sum, file) => sum + file.size, 0);
  process.stdout.write(`Generated ${gold.documents.length} InternBench documents (${(bytes / 1048576).toFixed(1)} MiB) in ${resolve(output)} in ${seconds.toFixed(1)} s\n`);

  const goldPath = join(BENCH_DIRECTORY, 'gold.json');
  const manifestPath = join(BENCH_DIRECTORY, 'manifest.json');
  if (updateGold) {
    if (process.version !== PINNED_NODE) throw new Error(`gold updates require pinned Node ${PINNED_NODE}, got ${process.version}`);
    await writeFile(goldPath, `${JSON.stringify(gold, null, 2)}\n`);
    await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
    process.stdout.write('Rewrote bench/gold.json and bench/manifest.json\n');
    return;
  }
  let committedGold;
  let committedManifest;
  try {
    committedGold = JSON.parse(await readFile(goldPath, 'utf8'));
    committedManifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  } catch (error) {
    process.stderr.write(`Cannot read the committed gold or manifest (${error.message}). Run with --update-gold on Node ${PINNED_NODE} to create them.\n`);
    process.exitCode = 1;
    return;
  }
  const subset = (list, key, ids) => (ids ? list.filter((item) => ids.has(item[key])) : list);
  const ids = only ? new Set(only) : null;
  const fileNames = only ? new Set(gold.documents.map((document) => document.file)) : null;
  const goldChanges = differences(subset(committedGold.documents, 'id', ids), gold.documents, 'id');
  const headerChanged = JSON.stringify({ ...committedGold, documents: null }) !== JSON.stringify({ ...gold, documents: null });
  const fileChanges = differences(subset(committedManifest.files, 'file', fileNames), manifest.files, 'file');
  if (goldChanges.length || headerChanged || fileChanges.length) {
    if (headerChanged) process.stderr.write('bench/gold.json header differs from the generator\n');
    for (const change of goldChanges) process.stderr.write(`gold ${change}\n`);
    for (const change of fileChanges) process.stderr.write(`manifest ${change}\n`);
    process.stderr.write(`Generated output does not match the committed gold/manifest. Review the change and run with --update-gold on Node ${PINNED_NODE}.\n`);
    process.exitCode = 1;
    return;
  }
  process.stdout.write('Gold and manifest match the committed bench/gold.json and bench/manifest.json\n');
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await runCli();
