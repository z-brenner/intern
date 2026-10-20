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
      .resolves.toMatchObject({ verifiedDownloads: 18 });
  });

  // Indra's files come from one commit of its model repository - the one the
  // indra-ocr 1.1.2 package's v1.1.0 tag resolves to - and only the Latin
  // set: not the multilingual recognizers, and not model.safetensors, which
  // nothing reads.
  it('pins Indra OCR to one model commit and its code licence to one source commit', async () => {
    const manifest = JSON.parse(await readFile(join(process.cwd(), 'src-tauri/resources/runtime-assets.json'), 'utf8'));
    const indra = manifest.downloads.filter((download: any) => download.id.startsWith('indra-') && download.id !== 'indra-license');
    expect(indra.map((download: any) => download.url.split('/').pop()).sort())
      .toEqual(['LICENSE', 'det.onnx', 'lm.npz', 'meta.json', 'order.onnx', 'orient.onnx', 'rec.onnx']);
    for (const download of indra) {
      expect(download.version).toBe('a1283c4b42a273c09480ad50e71461daff50fcc7');
      expect(download.url).toMatch(/^https:\/\/huggingface\.co\/datalab-to\/indra-ocr\/resolve\/a1283c4b42a273c09480ad50e71461daff50fcc7\/[\w.]+$/);
    }
    const code = manifest.downloads.find((download: any) => download.id === 'indra-license');
    expect(code.url).toBe('https://raw.githubusercontent.com/datalab-to/indra/0e96b93c6810f17cf8352acbb4487cc9e45b66d5/LICENSE');
    expect(manifest.downloads.map((download: any) => download.url).join('\n')).not.toMatch(/safetensors|rec_cjk|rec_rtl|rec_world|script\.onnx|lm_cjk|lm_world/);
  });

  describe('a bundle with an OCR engine', () => {
    const ocr = ['onnxruntime.dll', 'ocr-models/text-detection.onnx', 'ocr-models/text-recognition.onnx', 'ocr-models/page-orientation.onnx'];
    const indra = ['indra-models/det.onnx', 'indra-models/orient.onnx', 'indra-models/rec.onnx', 'indra-models/order.onnx', 'indra-models/lm.npz', 'indra-models/meta.json'];
    const indraDownloads: Record<string, string> = {
      'indra-models/det.onnx': 'indra-detection',
      'indra-models/orient.onnx': 'indra-orientation',
      'indra-models/rec.onnx': 'indra-recognition',
      'indra-models/order.onnx': 'indra-order',
      'indra-models/lm.npz': 'indra-language-model',
      'indra-models/meta.json': 'indra-meta',
    };
    const indraLicenses = ['licenses/indra-ocr-models-LICENSE.txt', 'licenses/indra-LICENSE.txt'];
    const digest = (contents: string) => createHash('sha256').update(contents).digest('hex');

    /**
     * A root with every file staged as a stand-in whose contents are its own
     * install path, and a manifest whose pins are those stand-ins' digests,
     * as a real staging step's files are their pins.
     */
    async function stage() {
      const root = await mkdtemp(join(tmpdir(), 'intern-assets-ocr-'));
      for (const installPath of ['pdfium.dll', ...ocr, ...indra]) {
        await mkdir(join(root, 'staged', installPath, '..'), { recursive: true });
        await writeFile(join(root, 'staged', installPath), installPath);
      }
      await mkdir(join(root, 'licenses'));
      for (const license of ['licenses/notice.txt', ...indraLicenses]) await writeFile(join(root, license), license);
      const pinned = JSON.parse(await readFile(join(process.cwd(), 'src-tauri/resources/runtime-assets.json'), 'utf8'));
      const releasePins = structuredClone(pinned.downloads);
      const download = (id: string) => pinned.downloads.find((entry: any) => entry.id === id);
      download('onnxruntime').extract.sha256 = digest('onnxruntime.dll');
      download('ocr-text-detection').sha256 = digest('ocr-models/text-detection.onnx');
      download('ocr-text-recognition').sha256 = digest('ocr-models/text-recognition.onnx');
      download('ocr-page-orientation').sha256 = digest('ocr-models/page-orientation.onnx');
      for (const [path, id] of Object.entries(indraDownloads)) download(id).sha256 = digest(path);
      const manifestPath = join(root, 'runtime-assets.json');
      const entry = (installPath: string) => ({
        path: `staged/${installPath}`,
        install_path: installPath,
        packages: [{ name: 'test-runtime', version: '1.0.0' }],
        size: Buffer.byteLength(installPath),
        sha256: digest(installPath),
      });
      const licenseEntry = (path: string) => ({ path, install_path: path, size: Buffer.byteLength(path), sha256: digest(path) });
      const write = (files: string[], licenses = ['licenses/notice.txt', ...indraLicenses]) => writeFile(manifestPath, JSON.stringify({
        ...pinned, bundled_files: files.map(entry), license_files: licenses.map(licenseEntry),
      }));
      return { root, manifestPath, pinned, releasePins, download, write };
    }

    it('refuses a bundle that carries only part of the PP-OCR runtime', async () => {
      const { root, manifestPath, pinned, releasePins, download, write } = await stage();

      // The runtime without the recognizer: the worker would load it and fall
      // back to Tesseract on every scan.
      await write(['pdfium.dll', 'onnxruntime.dll', 'ocr-models/text-detection.onnx', 'ocr-models/page-orientation.onnx']);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .rejects.toThrow(/OCR runtime is incomplete: missing ocr-models\/text-recognition\.onnx/);

      await write(['pdfium.dll', ...ocr]);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .resolves.toMatchObject({ verifiedFiles: 5 });

      // The same files, but the DLL is not the one its download is pinned to:
      // a file can match its own inventory line and still not be what ships.
      const dllPin = download('onnxruntime').extract.sha256;
      download('onnxruntime').extract.sha256 = 'b'.repeat(64);
      await write(['pdfium.dll', ...ocr]);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .rejects.toThrow(/onnxruntime\.dll is not the pinned download/);
      download('onnxruntime').extract.sha256 = dllPin;

      // An inventory from before PP-OCR is still a valid inventory...
      pinned.downloads = releasePins;
      await write(['pdfium.dll']);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .resolves.toMatchObject({ verifiedFiles: 1 });
      // ...but not a release one, which is staged against pins that include it.
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true, requireExpectedPins: true }))
        .rejects.toThrow(/OCR runtime is incomplete: missing onnxruntime\.dll/);
      // And a release bundle's OCR files must be the pinned ones, not stand-ins.
      await write(['pdfium.dll', ...ocr, ...indra]);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true, requireExpectedPins: true }))
        .rejects.toThrow(/is not the pinned download/);
    });

    it('holds Indra OCR to all of its files, the shared runtime, and its licences', async () => {
      const { root, manifestPath, write } = await stage();

      await write(['pdfium.dll', ...ocr, ...indra]);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .resolves.toMatchObject({ verifiedFiles: 11, verifiedLicenses: 3 });

      // Without its language model the worker would report Indra incomplete
      // on every install and read with PP-OCR.
      await write(['pdfium.dll', ...ocr, ...indra.filter((path) => path !== 'indra-models/lm.npz')]);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .rejects.toThrow(/Indra OCR runtime is incomplete: missing indra-models\/lm\.npz/);

      // Indra runs on ONNX Runtime; its models alone read nothing.
      await write(['pdfium.dll', ...indra]);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .rejects.toThrow(/Indra OCR models need onnxruntime\.dll/);

      // The weights' licence says a copy goes with them.
      await write(['pdfium.dll', ...ocr, ...indra], ['licenses/notice.txt', 'licenses/indra-LICENSE.txt']);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .rejects.toThrow(/need their licences: missing licenses\/indra-ocr-models-LICENSE\.txt/);

      // A bundle with PP-OCR and not Indra is a valid inventory. (A release
      // one is staged against the real pins and must hold both engines; the
      // stand-ins here cannot be the real downloads, so that is left to the
      // release staging itself.)
      await write(['pdfium.dll', ...ocr], ['licenses/notice.txt']);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .resolves.toMatchObject({ verifiedFiles: 5 });
    });

    it('refuses a bundled Indra file that is not its pinned download', async () => {
      const { root, manifestPath, download, write } = await stage();
      download('indra-meta').sha256 = 'c'.repeat(64);
      await write(['pdfium.dll', ...ocr, ...indra]);
      await expect(verifyRuntimeAssets(manifestPath, { root, requireBundled: true }))
        .rejects.toThrow(/indra-models\/meta\.json is not the pinned download/);
    });
  });
});
