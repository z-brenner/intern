import { readFile, readdir, writeFile, mkdir } from 'node:fs/promises';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Builds the Power Automate package that fills a SharePoint library's
 * Description column from Intern's description records.
 *
 * The flow's source is committed as readable JSON under
 * `integrations/power-automate/description-flow/package/`, laid out exactly as
 * Power Automate's "Export > Package (.zip)" lays out a flow: `manifest.json`
 * at the root, and `Microsoft.Flow/flows/<id>/` holding `definition.json`,
 * `apisMap.json`, and `connectionsMap.json`. The zip a person imports with
 * "Import > Import Package (Legacy)" is that tree, nothing more.
 *
 * The zip is committed under `site/downloads/` so the Pages deployment serves
 * it next to the guide that explains it, and so it is in every commit's tree
 * the release digest covers, rather than being produced by a release step that
 * would need its own evidence. `scripts/build-description-flow.test.ts` checks
 * that the committed zip is byte for byte what this script builds from the
 * committed source, so the two cannot drift: change the JSON, run
 * `npm run flow:build`, commit both.
 *
 * Node has no zip writer (`node:zlib` compresses streams; it does not make
 * archives), and a dependency for one small archive would be out of
 * proportion. So this writes the simplest valid zip there is: every entry
 * stored, not compressed, with a fixed timestamp and a fixed entry order, so
 * the same source always makes the same bytes. Five small JSON files gain
 * nothing worth having from compression.
 */

const here = dirname(fileURLToPath(import.meta.url));
export const repositoryRoot = resolve(here, '..');
export const packageSource = join(repositoryRoot, 'integrations', 'power-automate', 'description-flow', 'package');
export const packageOutput = join(repositoryRoot, 'site', 'downloads', 'intern-description-flow.zip');

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  return table;
})();

/** The CRC-32 (IEEE 802.3, as zip uses it) of a buffer. */
export function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

// 2026-01-01 00:00:00 in MS-DOS form. Any fixed value would do; a real date
// keeps archive tools from showing 1980.
const DOS_TIME = 0;
const DOS_DATE = ((2026 - 1980) << 9) | (1 << 5) | 1;

/**
 * A zip of `entries` ({ name, data }), each stored without compression, in
 * the order given. Names are `/`-separated and ASCII; no directory entries are
 * written, because none are needed to recreate the tree.
 */
export function storedZip(entries) {
  const locals = [];
  const centrals = [];
  let offset = 0;
  for (const { name, data } of entries) {
    const nameBytes = Buffer.from(name, 'utf8');
    const crc = crc32(data);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0); // local file header signature
    local.writeUInt16LE(20, 4); // version needed to extract (2.0)
    local.writeUInt16LE(0, 6); // flags
    local.writeUInt16LE(0, 8); // method: stored
    local.writeUInt16LE(DOS_TIME, 10);
    local.writeUInt16LE(DOS_DATE, 12);
    local.writeUInt32LE(crc, 14);
    local.writeUInt32LE(data.length, 18); // compressed size
    local.writeUInt32LE(data.length, 22); // uncompressed size
    local.writeUInt16LE(nameBytes.length, 26);
    local.writeUInt16LE(0, 28); // extra field length
    locals.push(local, nameBytes, data);

    const central = Buffer.alloc(46);
    central.writeUInt32LE(0x02014b50, 0); // central directory header signature
    central.writeUInt16LE(20, 4); // version made by (2.0, MS-DOS attributes)
    central.writeUInt16LE(20, 6); // version needed to extract
    central.writeUInt16LE(0, 8); // flags
    central.writeUInt16LE(0, 10); // method: stored
    central.writeUInt16LE(DOS_TIME, 12);
    central.writeUInt16LE(DOS_DATE, 14);
    central.writeUInt32LE(crc, 16);
    central.writeUInt32LE(data.length, 20);
    central.writeUInt32LE(data.length, 24);
    central.writeUInt16LE(nameBytes.length, 28);
    central.writeUInt16LE(0, 30); // extra field length
    central.writeUInt16LE(0, 32); // comment length
    central.writeUInt16LE(0, 34); // disk number start
    central.writeUInt16LE(0, 36); // internal attributes
    central.writeUInt32LE(0, 38); // external attributes
    central.writeUInt32LE(offset, 42); // offset of the local header
    centrals.push(central, nameBytes);

    offset += local.length + nameBytes.length + data.length;
  }
  const directory = Buffer.concat(centrals);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0); // end of central directory signature
  end.writeUInt16LE(0, 4); // this disk
  end.writeUInt16LE(0, 6); // disk with the central directory
  end.writeUInt16LE(entries.length, 8);
  end.writeUInt16LE(entries.length, 10);
  end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  end.writeUInt16LE(0, 20); // comment length
  return Buffer.concat([...locals, directory, end]);
}

async function filesUnder(directory) {
  const found = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) found.push(...await filesUnder(path));
    else if (entry.isFile()) found.push(path);
  }
  return found;
}

/**
 * The package's entries, read from the committed source: each JSON file
 * parsed (so a typo fails the build rather than the import) and written back
 * compact, the way Power Automate's own exports are. `manifest.json` comes
 * first, then the rest in a fixed order.
 */
export async function descriptionFlowEntries(source = packageSource) {
  const paths = await filesUnder(source);
  const entries = await Promise.all(paths.map(async (path) => {
    const name = relative(source, path).split(sep).join('/');
    let parsed;
    try {
      parsed = JSON.parse(await readFile(path, 'utf8'));
    } catch (error) {
      throw new Error(`${name} is not valid JSON: ${error instanceof Error ? error.message : String(error)}`);
    }
    return { name, data: Buffer.from(JSON.stringify(parsed), 'utf8') };
  }));
  const rank = (name) => (name === 'manifest.json' ? 0 : 1);
  return entries.sort((a, b) => rank(a.name) - rank(b.name) || (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
}

export async function buildDescriptionFlow(source = packageSource) {
  return storedZip(await descriptionFlowEntries(source));
}

const invokedPath = process.argv[1] && resolve(process.argv[1]);
if (invokedPath === fileURLToPath(import.meta.url)) {
  const built = await buildDescriptionFlow();
  const shown = relative(repositoryRoot, packageOutput).split(sep).join('/');
  if (process.argv.includes('--check')) {
    const committed = await readFile(packageOutput).catch(() => Buffer.alloc(0));
    if (!committed.equals(built)) {
      process.stderr.write(`${shown} is out of date with its source; run npm run flow:build and commit the result.\n`);
      process.exit(1);
    }
    process.stdout.write(`${shown} is up to date.\n`);
  } else {
    await mkdir(dirname(packageOutput), { recursive: true });
    await writeFile(packageOutput, built);
    process.stdout.write(`Wrote ${shown} (${built.length} bytes).\n`);
  }
}
