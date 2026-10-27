// The inventory of the Linux package's runtime: every file it carries under
// runtime/ and licenses/, with where each came from, its size, its SHA-256,
// and whether it must be executable. scripts/fetch-linux-assets.sh writes it
// as the last step of staging; scripts/stage-linux-runtime.sh, the Linux
// smoke, and anyone holding an installed package check files against it.
//
// Paths are relative to the package's resource folder - src-tauri/resources
// before bundling, /usr/lib/Intern once installed - so the same inventory
// checks the staged tree, a copy of it, and the installed package alike.
//
//   node scripts/linux-runtime-inventory.mjs write ROOT PACKAGES_TSV LLAMA_CPP_JSON
//   node scripts/linux-runtime-inventory.mjs verify ROOT
//   node scripts/linux-runtime-inventory.mjs stage ROOT DESTINATION
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { chmod, copyFile, lstat, mkdir, readdir, readFile, stat, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, join, relative, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

export const INVENTORY = 'runtime/runtime-inventory.json';
const TARGET = 'x86_64-unknown-linux-gnu';

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

/** A path inside the package, refused if it could name anything outside it. */
function safeRelative(path, label) {
  assert(typeof path === 'string' && path.length > 0 && !isAbsolute(path), `unsafe ${label}: ${path}`);
  assert(!path.includes('\\') && !path.split('/').some((part) => part === '..' || part === '.' || part === ''), `unsafe ${label}: ${path}`);
  return path;
}

async function sha256(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

/** Every regular file under ROOT/DIRECTORY, as package paths, sorted. */
async function filesUnder(root, directory) {
  const found = [];
  async function walk(path) {
    for (const entry of await readdir(join(root, path), { withFileTypes: true })) {
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) await walk(child);
      else {
        assert(entry.isFile(), `runtime files must be regular files, not links or devices: ${child}`);
        found.push(child);
      }
    }
  }
  await walk(directory);
  return found.sort();
}

async function describe(root, path) {
  const metadata = await stat(join(root, path));
  return { size: metadata.size, sha256: await sha256(join(root, path)), executable: (metadata.mode & 0o111) !== 0 };
}

/**
 * Records what is staged under ROOT/runtime and ROOT/licenses. `packages`
 * maps each runtime file to the packages it belongs to; a file without one,
 * or one named that is not there, is refused rather than recorded.
 */
export async function writeInventory(root, { packages, llamaCpp }) {
  const runtime = (await filesUnder(root, 'runtime')).filter((path) => path !== INVENTORY);
  const licenses = await filesUnder(root, 'licenses');
  assert(runtime.length > 0, 'no runtime files are staged');
  assert(licenses.length > 0, 'no license files are staged');
  for (const path of packages.keys()) assert(runtime.includes(path), `a package names ${path}, which is not staged`);
  const bundled = [];
  for (const path of runtime) {
    const owners = packages.get(path);
    assert(owners?.length, `no package identity for staged runtime file ${path}`);
    bundled.push({ path, packages: owners, ...(await describe(root, path)) });
  }
  const licenseFiles = [];
  for (const path of licenses) {
    const { size, sha256: digest } = await describe(root, path);
    licenseFiles.push({ path, size, sha256: digest });
  }
  const inventory = {
    schema_version: 1,
    generated_by: 'scripts/fetch-linux-assets.sh',
    target: TARGET,
    llama_cpp: llamaCpp,
    bundled_files: bundled,
    license_files: licenseFiles,
  };
  await writeFile(join(root, INVENTORY), `${JSON.stringify(inventory, null, 2)}\n`);
  return inventory;
}

async function readInventory(root) {
  const inventory = JSON.parse(await readFile(join(root, INVENTORY), 'utf8'));
  assert(inventory.schema_version === 1, 'runtime inventory schema must be 1');
  assert(inventory.target === TARGET, `runtime inventory is for ${inventory.target}, not ${TARGET}`);
  assert(Array.isArray(inventory.bundled_files) && inventory.bundled_files.length > 0, 'runtime inventory lists no runtime files');
  assert(Array.isArray(inventory.license_files) && inventory.license_files.length > 0, 'runtime inventory lists no license files');
  return inventory;
}

/** Every listed file under ROOT, checked against the inventory there. */
export async function verifyInventory(root) {
  const inventory = await readInventory(root);
  const seen = new Set();
  const entries = [
    ...inventory.bundled_files.map((file) => ({ ...file, kind: 'runtime' })),
    ...inventory.license_files.map((file) => ({ ...file, kind: 'license' })),
  ];
  for (const file of entries) {
    const path = safeRelative(file.path, 'packaged path');
    assert(path.startsWith(file.kind === 'runtime' ? 'runtime/' : 'licenses/'), `${file.kind} file outside its folder: ${path}`);
    assert(path !== INVENTORY, 'the inventory cannot list itself');
    assert(!seen.has(path), `duplicate packaged path: ${path}`);
    seen.add(path);
    assert(Number.isSafeInteger(file.size) && file.size >= 0, `${path} has an invalid size`);
    assert(typeof file.sha256 === 'string' && /^[0-9a-f]{64}$/.test(file.sha256), `${path} has an invalid SHA-256 digest`);
    let metadata;
    try { metadata = await lstat(join(root, path)); } catch (error) {
      if (error?.code === 'ENOENT') throw new Error(`runtime file is missing: ${path}`);
      throw error;
    }
    assert(metadata.isFile() && !metadata.isSymbolicLink(), `runtime file is not a regular file: ${path}`);
    assert(metadata.size === file.size, `runtime file size mismatch: ${path}`);
    assert(await sha256(join(root, path)) === file.sha256, `runtime file SHA-256 mismatch: ${path}`);
    if (file.executable) assert((metadata.mode & 0o111) !== 0, `runtime file lost its executable bit: ${path}`);
  }
  // Anything else in runtime/ is something nobody verified - a file left from
  // another build, or one put there since.
  for (const path of await filesUnder(root, 'runtime')) {
    assert(path === INVENTORY || seen.has(path), `runtime file is not in the inventory: ${path}`);
  }
  return { runtimeFiles: inventory.bundled_files.length, licenseFiles: inventory.license_files.length };
}

/** Verifies ROOT, then copies everything listed, and the inventory, to DESTINATION. */
export async function stageInventory(root, destination) {
  const inventory = await readInventory(root);
  await verifyInventory(root);
  const files = [...inventory.bundled_files, ...inventory.license_files].map((file) => file.path);
  for (const path of [...files, INVENTORY]) {
    const target = join(destination, path);
    assert(!relative(destination, target).startsWith('..'), `packaged path escapes the stage: ${path}`);
    await mkdir(dirname(target), { recursive: true });
    await copyFile(join(root, path), target);
    await chmod(target, (await stat(join(root, path))).mode & 0o777);
  }
  return verifyInventory(destination);
}

async function readPackages(tsv) {
  const packages = new Map();
  for (const line of (await readFile(tsv, 'utf8')).split('\n').filter(Boolean)) {
    const [path, name, version] = line.split('\t');
    assert(path && name && version, `malformed package line: ${line}`);
    safeRelative(path, 'packaged path');
    const owners = packages.get(path) ?? [];
    if (!owners.some((owner) => owner.name === name && owner.version === version)) owners.push({ name, version });
    packages.set(path, owners);
  }
  return packages;
}

async function runCli() {
  const [command, ...args] = process.argv.slice(2);
  if (command === 'write' && args.length === 3) {
    const [root, tsv, llama] = args;
    const inventory = await writeInventory(resolve(root), {
      packages: await readPackages(tsv),
      llamaCpp: JSON.parse(await readFile(llama, 'utf8')),
    });
    process.stdout.write(`Recorded ${inventory.bundled_files.length} runtime files and ${inventory.license_files.length} license files in ${INVENTORY}.\n`);
  } else if (command === 'verify' && args.length === 1) {
    const result = await verifyInventory(resolve(args[0]));
    process.stdout.write(`Verified ${result.runtimeFiles} runtime files and ${result.licenseFiles} license files against ${INVENTORY}.\n`);
  } else if (command === 'stage' && args.length === 2) {
    const result = await stageInventory(resolve(args[0]), resolve(args[1]));
    process.stdout.write(`Staged and verified ${result.runtimeFiles} runtime files and ${result.licenseFiles} license files at ${resolve(args[1])}.\n`);
  } else {
    process.stderr.write('usage: linux-runtime-inventory.mjs write ROOT PACKAGES_TSV LLAMA_CPP_JSON | verify ROOT | stage ROOT DESTINATION\n');
    process.exit(2);
  }
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await runCli();
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exit(1);
  }
}
