import { chmod, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { INVENTORY, stageInventory, verifyInventory, writeInventory } from './linux-runtime-inventory.mjs';

/**
 * Whether this file system keeps an executable bit. The inventory is written
 * and verified on Linux, where a server without the bit cannot start; the
 * Windows job runs these tests too, and NTFS has no such bit to record or to
 * lose, so there the inventory's other checks are what is tested.
 */
const POSIX = process.platform !== 'win32';

/** Prism's build, flat in runtime/, and upstream's, in runtime/llama-upstream/. */
const prism = { path: '.', repository: 'https://example.invalid/prism/llama.cpp', release: 'prism-test', archive: 'llama-prism-test-bin-ubuntu-x64.tar.gz', url: 'https://example.invalid/llama-prism-test-bin-ubuntu-x64.tar.gz', size: 1, sha256: '0'.repeat(64) };
const upstream = { path: 'llama-upstream', repository: 'https://example.invalid/ggml-org/llama.cpp', release: 'b-test', archive: 'llama-b-test-bin-ubuntu-x64.tar.gz', url: 'https://example.invalid/llama-b-test-bin-ubuntu-x64.tar.gz', size: 2, sha256: '1'.repeat(64) };
const llamaCpp = [prism, upstream];

/**
 * A staged package: each build's server and a library of the same name, a
 * model, and one license.
 */
async function staged() {
  const root = await mkdtemp(join(tmpdir(), 'intern-linux-runtime-'));
  await mkdir(join(root, 'runtime/ocr-models'), { recursive: true });
  await mkdir(join(root, 'runtime/llama-upstream'), { recursive: true });
  await mkdir(join(root, 'licenses'), { recursive: true });
  await writeFile(join(root, 'runtime/llama-server'), 'server');
  await chmod(join(root, 'runtime/llama-server'), 0o755);
  await writeFile(join(root, 'runtime/libllama.so.0'), 'library');
  await writeFile(join(root, 'runtime/llama-upstream/llama-server'), 'upstream server');
  await chmod(join(root, 'runtime/llama-upstream/llama-server'), 0o755);
  await writeFile(join(root, 'runtime/llama-upstream/libllama.so.0'), 'upstream library');
  await writeFile(join(root, 'runtime/ocr-models/text-detection.onnx'), 'model');
  await writeFile(join(root, 'licenses/llama.cpp-LICENSE.txt'), 'MIT');
  const packages = new Map([
    ['runtime/llama-server', [{ name: 'llama.cpp', version: 'prism-test' }]],
    ['runtime/libllama.so.0', [{ name: 'llama.cpp', version: 'prism-test' }]],
    ['runtime/llama-upstream/llama-server', [{ name: 'llama.cpp', version: 'b-test' }]],
    ['runtime/llama-upstream/libllama.so.0', [{ name: 'llama.cpp', version: 'b-test' }]],
    ['runtime/ocr-models/text-detection.onnx', [{ name: 'PaddlePaddle/PP-OCRv5_mobile_det_onnx', version: 'abc' }]],
  ]);
  return { root, packages };
}

describe('the Linux runtime inventory', () => {
  it('records every staged file with its owner, size, digest, and executable bit, and verifies a copy', async () => {
    const { root, packages } = await staged();
    const inventory = await writeInventory(root, { packages, llamaCpp });
    expect(inventory.bundled_files.map((file: { path: string }) => file.path)).toEqual([
      'runtime/libllama.so.0', 'runtime/llama-server',
      'runtime/llama-upstream/libllama.so.0', 'runtime/llama-upstream/llama-server',
      'runtime/ocr-models/text-detection.onnx',
    ]);
    expect(inventory.bundled_files.find((file: { path: string }) => file.path === 'runtime/llama-server'))
      .toMatchObject({ executable: POSIX, size: 6, packages: [{ name: 'llama.cpp', version: 'prism-test' }] });
    expect(inventory.bundled_files.find((file: { path: string }) => file.path === 'runtime/llama-upstream/llama-server'))
      .toMatchObject({ executable: POSIX, size: 15, packages: [{ name: 'llama.cpp', version: 'b-test' }] });
    expect(inventory.license_files).toEqual([expect.objectContaining({ path: 'licenses/llama.cpp-LICENSE.txt', size: 3 })]);

    const destination = join(await mkdtemp(join(tmpdir(), 'intern-linux-stage-')), 'stage');
    await expect(stageInventory(root, destination)).resolves.toEqual({ runtimeFiles: 5, licenseFiles: 1 });
    await expect(verifyInventory(destination)).resolves.toEqual({ runtimeFiles: 5, licenseFiles: 1 });
  });

  it('records each llama.cpp build with the folder under runtime/ that holds it', async () => {
    const { root, packages } = await staged();
    await writeInventory(root, { packages, llamaCpp });
    const recorded = JSON.parse(await readFile(join(root, INVENTORY), 'utf8')).llama_cpp;
    expect(recorded).toEqual([prism, upstream]);
    expect(recorded.map((build: { path: string }) => build.path)).toEqual(['.', 'llama-upstream']);
  });

  // The shape from before there were two builds: one object, for the build in
  // runtime/ itself.
  it('records a single build object as the build in runtime/ itself', async () => {
    const { root, packages } = await staged();
    for (const path of [...packages.keys()].filter((path) => path.startsWith('runtime/llama-upstream/'))) packages.delete(path);
    await rm(join(root, 'runtime/llama-upstream'), { recursive: true });
    const { path: _path, ...single } = prism;
    const inventory = await writeInventory(root, { packages, llamaCpp: single });
    expect(inventory.llama_cpp).toEqual([prism]);
  });

  it('refuses to record a file nobody owns, or an owner of a file that is not there', async () => {
    const { root, packages } = await staged();
    const unowned = new Map(packages);
    unowned.delete('runtime/libllama.so.0');
    await expect(writeInventory(root, { packages: unowned, llamaCpp })).rejects.toThrow('no package identity for staged runtime file runtime/libllama.so.0');
    const unownedUpstream = new Map(packages);
    unownedUpstream.delete('runtime/llama-upstream/libllama.so.0');
    await expect(writeInventory(root, { packages: unownedUpstream, llamaCpp })).rejects.toThrow('no package identity for staged runtime file runtime/llama-upstream/libllama.so.0');
    const phantom = new Map(packages).set('runtime/libggml.so.0', [{ name: 'llama.cpp', version: 'prism-test' }]);
    await expect(writeInventory(root, { packages: phantom, llamaCpp })).rejects.toThrow('runtime/libggml.so.0, which is not staged');
  });

  // The two builds' libraries share names, and each server loads the ones
  // beside it, so a file in the other build's folder is a server loading
  // the wrong build.
  it('refuses a llama.cpp file outside its own build\'s folder', async () => {
    const { root, packages } = await staged();
    const crossed = new Map(packages).set('runtime/llama-upstream/libllama.so.0', [{ name: 'llama.cpp', version: 'prism-test' }]);
    await expect(writeInventory(root, { packages: crossed, llamaCpp }))
      .rejects.toThrow('runtime/llama-upstream/libllama.so.0 belongs to llama.cpp prism-test, but its folder holds llama.cpp b-test');
    const stray = new Map(packages).set('runtime/ocr-models/text-detection.onnx', [{ name: 'llama.cpp', version: 'b-test' }]);
    await expect(writeInventory(root, { packages: stray, llamaCpp }))
      .rejects.toThrow('runtime/ocr-models/text-detection.onnx belongs to llama.cpp b-test, but no llama.cpp build is recorded in its folder');
  });

  it.each([
    ['a build with no server of its own', [prism, upstream, { ...upstream, release: 'b-other', path: 'llama-other' }], 'llama.cpp b-other has no llama-server of its own at runtime/llama-other/llama-server'],
    ['two builds in one folder', [prism, { ...upstream, path: '.' }], 'two llama.cpp builds are recorded in runtime/.'],
    ['a build with no folder', [prism, { ...upstream, path: undefined }], 'llama.cpp b-test names no folder under runtime/'],
    ['a build folder that leaves runtime/', [prism, { ...upstream, path: '../llama-upstream' }], 'unsafe llama.cpp b-test folder'],
    ['a build with no archive pin', [prism, { ...upstream, sha256: 'unpinned' }], 'llama.cpp b-test has an invalid archive SHA-256'],
    ['no build at all', [], 'no llama.cpp build is recorded'],
  ])('refuses %s', async (_label, builds, message) => {
    const { root, packages } = await staged();
    await expect(writeInventory(root, { packages, llamaCpp: builds })).rejects.toThrow(message);
  });

  it.each([
    ['a changed file', (root: string) => writeFile(join(root, 'runtime/libllama.so.0'), 'librarx'), /SHA-256 mismatch: runtime\/libllama\.so\.0/],
    ['a truncated file', (root: string) => writeFile(join(root, 'runtime/libllama.so.0'), 'lib'), /size mismatch: runtime\/libllama\.so\.0/],
    ['a changed upstream file', (root: string) => writeFile(join(root, 'runtime/llama-upstream/libllama.so.0'), 'upstream librarx'), /SHA-256 mismatch: runtime\/llama-upstream\/libllama\.so\.0/],
    ['a missing license', (root: string) => writeFile(join(root, 'licenses/llama.cpp-LICENSE.txt'), 'MIT!'), /licenses\/llama\.cpp-LICENSE\.txt/],
    ['an unlisted file', (root: string) => writeFile(join(root, 'runtime/libextra.so'), 'extra'), /not in the inventory: runtime\/libextra\.so/],
    ['an unlisted upstream file', (root: string) => writeFile(join(root, 'runtime/llama-upstream/libextra.so'), 'extra'), /not in the inventory: runtime\/llama-upstream\/libextra\.so/],
  ])('refuses %s', async (_label, tamper, message) => {
    const { root, packages } = await staged();
    await writeInventory(root, { packages, llamaCpp });
    await tamper(root);
    await expect(verifyInventory(root)).rejects.toThrow(message);
  });

  // A package step that drops the executable bit leaves a server nothing can
  // start. Only a file system with the bit can lose it.
  it.runIf(POSIX)('refuses a lost executable bit', async () => {
    const { root, packages } = await staged();
    await writeInventory(root, { packages, llamaCpp });
    await chmod(join(root, 'runtime/llama-upstream/llama-server'), 0o644);
    await expect(verifyInventory(root)).rejects.toThrow(/lost its executable bit: runtime\/llama-upstream\/llama-server/);
  });

  it('refuses a listed path that leaves the package', async () => {
    const { root, packages } = await staged();
    const inventory = await writeInventory(root, { packages, llamaCpp });
    inventory.bundled_files[0].path = 'runtime/../../etc/passwd';
    await writeFile(join(root, INVENTORY), JSON.stringify(inventory));
    await expect(verifyInventory(root)).rejects.toThrow('unsafe packaged path');
  });
});
