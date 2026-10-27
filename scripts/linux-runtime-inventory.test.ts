import { chmod, mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { INVENTORY, stageInventory, verifyInventory, writeInventory } from './linux-runtime-inventory.mjs';

const llamaCpp = { repository: 'https://example.invalid/llama.cpp', release: 'test', archive: 'llama-test-bin-ubuntu-x64.tar.gz', url: 'https://example.invalid/llama-test-bin-ubuntu-x64.tar.gz', size: 1, sha256: '0'.repeat(64) };

/** A staged package: a server, a library, a model, and one license. */
async function staged() {
  const root = await mkdtemp(join(tmpdir(), 'intern-linux-runtime-'));
  await mkdir(join(root, 'runtime/ocr-models'), { recursive: true });
  await mkdir(join(root, 'licenses'), { recursive: true });
  await writeFile(join(root, 'runtime/llama-server'), 'server');
  await chmod(join(root, 'runtime/llama-server'), 0o755);
  await writeFile(join(root, 'runtime/libllama.so.0'), 'library');
  await writeFile(join(root, 'runtime/ocr-models/text-detection.onnx'), 'model');
  await writeFile(join(root, 'licenses/llama.cpp-LICENSE.txt'), 'MIT');
  const packages = new Map([
    ['runtime/llama-server', [{ name: 'llama.cpp', version: 'test' }]],
    ['runtime/libllama.so.0', [{ name: 'llama.cpp', version: 'test' }]],
    ['runtime/ocr-models/text-detection.onnx', [{ name: 'PaddlePaddle/PP-OCRv5_mobile_det_onnx', version: 'abc' }]],
  ]);
  return { root, packages };
}

describe('the Linux runtime inventory', () => {
  it('records every staged file with its owner, size, digest, and executable bit, and verifies a copy', async () => {
    const { root, packages } = await staged();
    const inventory = await writeInventory(root, { packages, llamaCpp });
    expect(inventory.bundled_files.map((file: { path: string }) => file.path)).toEqual([
      'runtime/libllama.so.0', 'runtime/llama-server', 'runtime/ocr-models/text-detection.onnx',
    ]);
    expect(inventory.bundled_files.find((file: { path: string }) => file.path === 'runtime/llama-server'))
      .toMatchObject({ executable: true, size: 6, packages: [{ name: 'llama.cpp', version: 'test' }] });
    expect(inventory.license_files).toEqual([expect.objectContaining({ path: 'licenses/llama.cpp-LICENSE.txt', size: 3 })]);
    expect(JSON.parse(await readFile(join(root, INVENTORY), 'utf8')).llama_cpp).toEqual(llamaCpp);

    const destination = join(await mkdtemp(join(tmpdir(), 'intern-linux-stage-')), 'stage');
    await expect(stageInventory(root, destination)).resolves.toEqual({ runtimeFiles: 3, licenseFiles: 1 });
    await expect(verifyInventory(destination)).resolves.toEqual({ runtimeFiles: 3, licenseFiles: 1 });
  });

  it('refuses to record a file nobody owns, or an owner of a file that is not there', async () => {
    const { root, packages } = await staged();
    const unowned = new Map(packages);
    unowned.delete('runtime/libllama.so.0');
    await expect(writeInventory(root, { packages: unowned, llamaCpp })).rejects.toThrow('no package identity for staged runtime file runtime/libllama.so.0');
    const phantom = new Map(packages).set('runtime/libggml.so.0', [{ name: 'llama.cpp', version: 'test' }]);
    await expect(writeInventory(root, { packages: phantom, llamaCpp })).rejects.toThrow('runtime/libggml.so.0, which is not staged');
  });

  it.each([
    ['a changed file', (root: string) => writeFile(join(root, 'runtime/libllama.so.0'), 'librarx'), /SHA-256 mismatch: runtime\/libllama\.so\.0/],
    ['a truncated file', (root: string) => writeFile(join(root, 'runtime/libllama.so.0'), 'lib'), /size mismatch: runtime\/libllama\.so\.0/],
    ['a missing license', (root: string) => writeFile(join(root, 'licenses/llama.cpp-LICENSE.txt'), 'MIT!'), /licenses\/llama\.cpp-LICENSE\.txt/],
    // A package step that drops the executable bit leaves a server nothing can start.
    ['a lost executable bit', (root: string) => chmod(join(root, 'runtime/llama-server'), 0o644), /lost its executable bit: runtime\/llama-server/],
    ['an unlisted file', (root: string) => writeFile(join(root, 'runtime/libextra.so'), 'extra'), /not in the inventory: runtime\/libextra\.so/],
  ])('refuses %s', async (_label, tamper, message) => {
    const { root, packages } = await staged();
    await writeInventory(root, { packages, llamaCpp });
    await tamper(root);
    await expect(verifyInventory(root)).rejects.toThrow(message);
  });

  it('refuses a listed path that leaves the package', async () => {
    const { root, packages } = await staged();
    const inventory = await writeInventory(root, { packages, llamaCpp });
    inventory.bundled_files[0].path = 'runtime/../../etc/passwd';
    await writeFile(join(root, INVENTORY), JSON.stringify(inventory));
    await expect(verifyInventory(root)).rejects.toThrow('unsafe packaged path');
  });
});
