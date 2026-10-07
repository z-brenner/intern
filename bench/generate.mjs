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
///   --out DIR       write somewhere other than bench/generated. DIR must be
///                   missing, empty, or an earlier output of this generator
///                   (it holds an InternBench manifest.json); the files that
///                   manifest lists are replaced, and nothing else is removed.
///   --only id,id    build only these documents (compared against their committed
///                   entries). The other documents' files stay as they are, and
///                   the output's manifest.json is updated for the ones rebuilt;
///                   unless it then lists every document it says "partial": true.
///
/// Every document is a pure function of its builder: fixed seeds, fixed
/// timestamps, fixed zlib level, no locale, no clock.
import { createHash } from 'node:crypto';
import { mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { BUILDERS } from './docs/index.mjs';

const BENCH_DIRECTORY = dirname(fileURLToPath(import.meta.url));
export const PINNED_NODE = 'v24.15.0';

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

/// The manifest an earlier run left in `root`, or null when `root` is
/// missing or empty. Refuses a directory that holds anything else, so a
/// mistyped `--out` never overwrites or removes someone's files.
async function previousOutput(root) {
  if (root === BENCH_DIRECTORY) throw new Error(`refusing to write the corpus into ${root}, which holds the builders and the committed gold`);
  let entries;
  try {
    entries = await readdir(root);
  } catch (error) {
    if (error.code === 'ENOENT') return null;
    throw error;
  }
  if (!entries.length) return null;
  let manifest = null;
  try {
    manifest = JSON.parse(await readFile(join(root, 'manifest.json'), 'utf8'));
  } catch {
    // Not one of ours; refused below.
  }
  if (manifest?.suite !== 'internbench' || !Array.isArray(manifest.files)) {
    throw new Error(`refusing to write into ${root}: it is not empty and holds no InternBench manifest.json`);
  }
  return manifest;
}

/// Where a file sits in a full build: its builder's place in BUILDERS (a
/// file is named after its document id), or after every builder's.
function buildOrder(file) {
  const index = BUILDERS.findIndex(([id]) => file.slice(0, file.lastIndexOf('.')) === id);
  return index < 0 ? BUILDERS.length : index;
}

/// Builds the corpus into `outputDirectory`. Returns the gold, the
/// manifest of the documents built, and each document's text model (keyed
/// by id) for tests. A full build replaces the files an earlier build
/// listed; `only` rebuilds just those documents and leaves the rest.
export async function generateBench(outputDirectory, { only = null } = {}) {
  const root = resolve(outputDirectory);
  const selected = only ? BUILDERS.filter(([id]) => only.includes(id)) : BUILDERS;
  if (only) {
    const unknown = only.filter((id) => !BUILDERS.some(([known]) => known === id));
    if (unknown.length) throw new Error(`unknown document id(s): ${unknown.join(', ')}`);
  }
  const previous = await previousOutput(root);
  if (previous && !only) {
    for (const { file } of previous.files) {
      if (typeof file === 'string' && file === basename(file)) await rm(join(root, file), { force: true });
    }
    await rm(join(root, 'manifest.json'), { force: true });
  }
  await mkdir(root, { recursive: true });
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
  await writeFile(join(root, 'manifest.json'), `${JSON.stringify(only ? onDisk(previous, files) : manifest, null, 2)}\n`);
  return { gold, manifest, texts };
}

/// The manifest of what an `only` build leaves in the directory: the
/// earlier manifest's entries with the rebuilt files' replaced, in build
/// order. Unless it lists every document it is marked partial, so it is
/// never taken for a full corpus's manifest.
function onDisk(previous, rebuilt) {
  const entries = new Map((previous?.files ?? []).map((entry) => [entry.file, entry]));
  for (const entry of rebuilt) entries.set(entry.file, entry);
  const files = [...entries.values()].sort((a, b) => buildOrder(a.file) - buildOrder(b.file));
  const built = new Set(files.map((entry) => buildOrder(entry.file)));
  const complete = BUILDERS.every((_, index) => built.has(index));
  return { schema_version: 1, suite: 'internbench', ...(complete ? {} : { partial: true }), generator: { node: PINNED_NODE.slice(1) }, files };
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
