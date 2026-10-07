import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { verifyRuntimeAssets } from './verify-assets.mjs';

describe('runtime asset verification', () => {
  it.each([
    ['changed URL', (manifest: any) => { manifest.downloads[0].url = 'https://example.invalid/llama.zip'; }],
    ['changed archive basename', (manifest: any) => { manifest.downloads[1].archive = 'other.tgz'; }],
    ['unsafe archive path', (manifest: any) => { manifest.downloads[2].archive = '../eng.traineddata'; }],
    ['extra download', (manifest: any) => { manifest.downloads.push({ ...manifest.downloads[0], id: 'extra' }); }],
    ['changed vcpkg repository', (manifest: any) => { manifest.vcpkg.repository = 'https://example.invalid/vcpkg.git'; }],
    // A model URL on a branch rather than a commit is not a pin: the same
    // bytes are not guaranteed tomorrow, and the size and hash would fail late.
    ['moving model revision', (manifest: any) => {
      const model = manifest.downloads.find((download: any) => download.id === 'ocr-text-detection');
      model.url = model.url.replace(/resolve\/[0-9a-f]{40}\//, 'resolve/main/');
    }],
    ['missing ONNX Runtime download', (manifest: any) => {
      manifest.downloads = manifest.downloads.filter((download: any) => download.id !== 'onnxruntime');
    }],
  ])('rejects a %s in the exact runtime acquisition contract', async (_label, mutate) => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-pins-'));
    const source = JSON.parse(await readFile(join(process.cwd(), 'src-tauri/resources/runtime-assets.json'), 'utf8'));
    mutate(source);
    const manifestPath = join(root, 'runtime-assets.json');
    await writeFile(manifestPath, JSON.stringify(source));

    await expect(verifyRuntimeAssets(manifestPath, { root, requireExpectedPins: true }))
      .rejects.toThrow(/download|URL|archive|vcpkg/i);
  });

  // The DLL is taken out of a 157 MB package of every platform's build; the
  // package hash says the archive is Microsoft's, and the DLL's own pin says
  // the one file that ships is the one that was measured.
  it.each([
    ['changed hash', (extract: any) => { extract.sha256 = '0'.repeat(64); }, true, /extracted file SHA-256 pin changed/],
    ['changed size', (extract: any) => { extract.size += 1; }, true, /extracted file size pin changed/],
    ['different file', (extract: any) => { extract.path = 'runtimes/win-arm64/native/onnxruntime.dll'; }, true, /extracted file pin changed/],
    // Checked whether or not the release pins are: a manifest that names a
    // file outside the package is malformed in any build.
    ['escaping path', (extract: any) => { extract.path = '../onnxruntime.dll'; }, false, /unsafe onnxruntime extracted file path/],
    ['malformed hash', (extract: any) => { extract.sha256 = 'not-a-digest'; }, false, /invalid SHA-256/],
  ])('rejects an ONNX Runtime DLL pin with a %s', async (_label, mutate, requireExpectedPins, message) => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-ort-'));
    const source = JSON.parse(await readFile(join(process.cwd(), 'src-tauri/resources/runtime-assets.json'), 'utf8'));
    mutate(source.downloads.find((download: any) => download.id === 'onnxruntime').extract);
    const manifestPath = join(root, 'runtime-assets.json');
    await writeFile(manifestPath, JSON.stringify(source));

    await expect(verifyRuntimeAssets(manifestPath, { root, requireExpectedPins })).rejects.toThrow(message);
  });

  it('does not treat an empty bundle inventory as a verified Windows runtime', async () => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-empty-'));
    const manifestPath = join(root, 'runtime-assets.json');
    await writeFile(manifestPath, JSON.stringify({ schema_version: 1, downloads: [], bundled_files: [], license_files: [] }));

    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .rejects.toThrow(/empty/i);
  });

  it('fails closed when a signed bundled asset is missing or tampered', async () => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-'));
    const bytes = Buffer.from('trusted runtime fixture');
    const manifest = {
      schema_version: 1,
      downloads: [],
      bundled_files: [{
        path: 'src-tauri/binaries/runtime.exe',
        install_path: 'runtime.exe',
        packages: [{ name: 'test-runtime', version: '1.0.0' }],
        size: bytes.length,
        sha256: createHash('sha256').update(bytes).digest('hex'),
      }],
      license_files: [{
        path: 'licenses/runtime.txt', install_path: 'licenses/runtime.txt', size: 7,
        sha256: createHash('sha256').update('license').digest('hex'),
      }],
    };
    const manifestPath = join(root, 'runtime-assets.json');
    await mkdir(join(root, 'licenses'));
    await writeFile(join(root, 'licenses/runtime.txt'), 'license');
    await writeFile(manifestPath, JSON.stringify(manifest));

    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .rejects.toThrow(/missing/i);
    await writeFile(join(root, 'runtime.exe'), 'tampered');
    manifest.bundled_files[0].path = 'runtime.exe';
    await writeFile(manifestPath, JSON.stringify(manifest));
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .rejects.toThrow(/size|SHA-256/i);
    await writeFile(join(root, 'runtime.exe'), bytes);
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .resolves.toMatchObject({ verifiedFiles: 1 });
  });

  it('rejects manifest paths that escape the verification root', async () => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-path-'));
    const manifestPath = join(root, 'runtime-assets.json');
    await writeFile(manifestPath, JSON.stringify({
      schema_version: 1,
      downloads: [],
      bundled_files: [{ path: '../escape.exe', install_path: 'escape.exe', packages: [{ name: 'test-runtime', version: '1.0.0' }], size: 0, sha256: '0'.repeat(64) }],
      license_files: [{ path: 'notice.txt', install_path: 'licenses/notice.txt', size: 0, sha256: createHash('sha256').update('').digest('hex') }],
    }));

    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .rejects.toThrow(/unsafe/i);
  });

  it('rejects unsafe or duplicate packaged paths and requires a license inventory', async () => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-install-path-'));
    const bytes = Buffer.from('runtime');
    const file = { path: 'runtime.exe', install_path: '../runtime.exe', packages: [{ name: 'test-runtime', version: '1.0.0' }], size: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') };
    await writeFile(join(root, 'runtime.exe'), bytes);
    await writeFile(join(root, 'notice.txt'), 'notice');
    const license = { path: 'notice.txt', install_path: 'licenses/notice.txt', size: 6, sha256: createHash('sha256').update('notice').digest('hex') };
    const manifestPath = join(root, 'runtime-assets.json');
    await writeFile(manifestPath, JSON.stringify({ schema_version: 1, downloads: [], bundled_files: [file] }));
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true })).rejects.toThrow(/license_files/i);

    await writeFile(manifestPath, JSON.stringify({ schema_version: 1, downloads: [], bundled_files: [file], license_files: [license] }));
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true })).rejects.toThrow(/packaged|install/i);

    file.install_path = 'runtime.exe';
    await writeFile(manifestPath, JSON.stringify({ schema_version: 1, downloads: [], bundled_files: [file, { ...file, path: 'copy.exe' }], license_files: [license] }));
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true })).rejects.toThrow(/duplicate packaged/i);
  });

  it('requires exact package owner and version metadata for each runtime file', async () => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-packages-'));
    await mkdir(join(root, 'licenses'));
    await writeFile(join(root, 'runtime.dll'), 'runtime');
    await writeFile(join(root, 'licenses/notice.txt'), 'notice');
    const runtime = { path: 'runtime.dll', install_path: 'runtime.dll', packages: [{ name: 'libpng', version: '' }], size: 7, sha256: createHash('sha256').update('runtime').digest('hex') };
    const license = { path: 'licenses/notice.txt', install_path: 'licenses/notice.txt', size: 6, sha256: createHash('sha256').update('notice').digest('hex') };
    const manifestPath = join(root, 'runtime-assets.json');
    await writeFile(manifestPath, JSON.stringify({ schema_version: 1, downloads: [], bundled_files: [runtime], license_files: [license] }));
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true })).rejects.toThrow(/package version/i);
  });

  it('pins the OCR runtime to Microsoft\'s NuGet package and the models to Hugging Face commits', async () => {
    const manifest = JSON.parse(await readFile(join(process.cwd(), 'src-tauri/resources/runtime-assets.json'), 'utf8'));
    const byId = new Map(manifest.downloads.map((download: any) => [download.id, download]));
    expect((byId.get('onnxruntime') as any).url)
      .toBe('https://api.nuget.org/v3-flatcontainer/microsoft.ml.onnxruntime/1.30.0/microsoft.ml.onnxruntime.1.30.0.nupkg');
    for (const id of ['ocr-text-detection', 'ocr-text-recognition', 'ocr-page-orientation']) {
      const model = byId.get(id) as any;
      expect(model.url).toMatch(new RegExp(`^https://huggingface\\.co/PaddlePaddle/[\\w.-]+/resolve/${model.version}/inference\\.onnx$`));
      expect(model.version).toMatch(/^[0-9a-f]{40}$/);
    }
    await expect(verifyRuntimeAssets(join(process.cwd(), 'src-tauri/resources/runtime-assets.json'), { requireExpectedPins: true }))
      .resolves.toMatchObject({ verifiedDownloads: 10 });
  });

  it('refuses a bundle that carries only part of the OCR runtime', async () => {
    const root = await mkdtemp(join(tmpdir(), 'intern-assets-ocr-'));
    await mkdir(join(root, 'licenses'));
    await writeFile(join(root, 'licenses/notice.txt'), 'notice');
    const license = { path: 'licenses/notice.txt', install_path: 'licenses/notice.txt', size: 6, sha256: createHash('sha256').update('notice').digest('hex') };
    const entry = (installPath: string) => {
      const bytes = Buffer.from(installPath);
      return {
        path: `staged/${installPath}`,
        install_path: installPath,
        packages: [{ name: 'test-runtime', version: '1.0.0' }],
        size: bytes.length,
        sha256: createHash('sha256').update(bytes).digest('hex'),
      };
    };
    const ocr = ['onnxruntime.dll', 'ocr-models/text-detection.onnx', 'ocr-models/text-recognition.onnx', 'ocr-models/page-orientation.onnx'];
    for (const installPath of ['pdfium.dll', ...ocr]) {
      await mkdir(join(root, 'staged', installPath, '..'), { recursive: true });
      await writeFile(join(root, 'staged', installPath), installPath);
    }
    const pinned = JSON.parse(await readFile(join(process.cwd(), 'src-tauri/resources/runtime-assets.json'), 'utf8'));
    const manifestPath = join(root, 'runtime-assets.json');
    const write = (files: string[]) => writeFile(manifestPath, JSON.stringify({
      ...pinned, bundled_files: files.map(entry), license_files: [license],
    }));

    // The runtime without the recognizer: the worker would load it and fall
    // back to Tesseract on every scan.
    await write(['pdfium.dll', 'onnxruntime.dll', 'ocr-models/text-detection.onnx', 'ocr-models/page-orientation.onnx']);
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .rejects.toThrow(/OCR runtime is incomplete: missing ocr-models\/text-recognition\.onnx/);

    await write(['pdfium.dll', ...ocr]);
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .resolves.toMatchObject({ verifiedFiles: 5 });

    // An inventory from before PP-OCR is still a valid inventory...
    await write(['pdfium.dll']);
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
      .resolves.toMatchObject({ verifiedFiles: 1 });
    // ...but not a release one, which is staged against pins that include it.
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true, requireExpectedPins: true }))
      .rejects.toThrow(/OCR runtime is incomplete: missing onnxruntime\.dll/);
    await write(['pdfium.dll', ...ocr]);
    await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true, requireExpectedPins: true }))
      .resolves.toMatchObject({ verifiedFiles: 5 });
  });
});
