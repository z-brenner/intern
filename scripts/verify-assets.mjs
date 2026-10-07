import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { lstat, readFile } from 'node:fs/promises';
import { dirname, isAbsolute, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const REPOSITORY_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const DEFAULT_MANIFEST = resolve(REPOSITORY_ROOT, 'src-tauri/resources/runtime-assets.json');
const EXPECTED_DOWNLOADS = Object.freeze({
  'llama.cpp': { version: 'b10361', archive: 'llama-b10361-bin-win-cpu-x64.zip', url: 'https://github.com/ggml-org/llama.cpp/releases/download/b10361/llama-b10361-bin-win-cpu-x64.zip', size: 18_427_695, sha256: '36da9e9c1c094bf7842fab69e6cc0921125a67fa2611ba8f329a00804350302a' },
  'llama.cpp-license': { version: 'b10361', archive: 'llama.cpp-LICENSE.txt', url: 'https://raw.githubusercontent.com/ggml-org/llama.cpp/b10361/LICENSE', size: 1_078, sha256: '94f29bbed6a22c35b992c5c6ebf0e7c92f13b836b90f36f461c9cf2f0f1d010d' },
  pdfium: { version: 'chromium/7881', archive: 'pdfium-win-x64.tgz', url: 'https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7881/pdfium-win-x64.tgz', size: 3_733_154, sha256: '73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08' },
  'eng.traineddata': { version: '87416418657359cb625c412a48b6e1d6d41c29bd', archive: 'eng.traineddata', url: 'https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/87416418657359cb625c412a48b6e1d6d41c29bd/eng.traineddata', size: 4_113_088, sha256: '7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2' },
  'osd.traineddata': { version: '87416418657359cb625c412a48b6e1d6d41c29bd', archive: 'osd.traineddata', url: 'https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/87416418657359cb625c412a48b6e1d6d41c29bd/osd.traineddata', size: 10_562_727, sha256: '9cf5d576fcc47564f11265841e5ca839001e7e6f38ff7f7aacf46d15a96b00ff' },
  // The NuGet package is a zip of every platform's build; the one file taken
  // out of it is pinned too, so the DLL that ships is checked byte for byte
  // and not only the archive it came in.
  onnxruntime: { version: '1.30.0', archive: 'microsoft.ml.onnxruntime.1.30.0.nupkg', url: 'https://api.nuget.org/v3-flatcontainer/microsoft.ml.onnxruntime/1.30.0/microsoft.ml.onnxruntime.1.30.0.nupkg', size: 157_230_753, sha256: '30df119bd9164eaa187666d93147448fdd4a9a1a3caf83f1505daf7af2d1c5d2', extract: { path: 'runtimes/win-x64/native/onnxruntime.dll', size: 16_462_648, sha256: '7e39e2bdbba836d98071ef28620735ba36a47c554cf794585269aecc50fab0da' } },
  'ocr-text-detection': { version: 'e6f4fa85f00e168c862bc462aebca69eef9b3d3d', archive: 'text-detection.onnx', url: 'https://huggingface.co/PaddlePaddle/PP-OCRv5_mobile_det_onnx/resolve/e6f4fa85f00e168c862bc462aebca69eef9b3d3d/inference.onnx', size: 4_826_518, sha256: 'a431985659dc921974177a95adcfbb90fd9e51989a5e04d70d0b75f597b6e61d' },
  'ocr-text-recognition': { version: '3fafbc3b5dcf93dd72add9f48368be8a3a2cd33b', archive: 'text-recognition.onnx', url: 'https://huggingface.co/PaddlePaddle/en_PP-OCRv5_mobile_rec_onnx/resolve/3fafbc3b5dcf93dd72add9f48368be8a3a2cd33b/inference.onnx', size: 7_848_423, sha256: 'b5f833dfc5d0eb71da397b4efa06ebeee9b431b690a47d6af40d77d8eabc557f' },
  'ocr-page-orientation': { version: '7330ab7039123e46af2dc03154b9969aa412c61d', archive: 'page-orientation.onnx', url: 'https://huggingface.co/PaddlePaddle/PP-LCNet_x1_0_doc_ori_onnx/resolve/7330ab7039123e46af2dc03154b9969aa412c61d/inference.onnx', size: 6_788_069, sha256: 'af9a0a4f317ff0709ce752067807f819cb15d883f8ecad89f28df1c6ee2d9c92' },
  'paddleocr-license': { version: 'v3.7.0', archive: 'PaddleOCR-LICENSE.txt', url: 'https://raw.githubusercontent.com/PaddlePaddle/PaddleOCR/b03f46425e8ff4442b268ce449e3eef758146cd4/LICENSE', size: 11_376, sha256: '3840c5c0c61c294264d2dd77b8777be6ddd90121ef4e0e64abcd22edea581d6e' },
});

/**
 * Files the worker cannot run PP-OCR without. A bundle that has some of them
 * and not the rest makes the worker fall back to Tesseract on every install
 * while looking complete, so an inventory holds all of them or none - and the
 * release inventory, staged against the expected pins, holds all of them.
 */
const OCR_RUNTIME_INSTALL_PATHS = Object.freeze([
  'onnxruntime.dll',
  'ocr-models/text-detection.onnx',
  'ocr-models/text-recognition.onnx',
  'ocr-models/page-orientation.onnx',
]);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function safePath(root, path) {
  assert(typeof path === 'string' && path.length > 0 && !isAbsolute(path), `unsafe runtime asset path: ${path}`);
  assert(!path.includes('\\') && !path.includes(':') && !path.split('/').includes('..'), `unsafe runtime asset path: ${path}`);
  const absolute = resolve(root, path);
  const fromRoot = relative(root, absolute);
  assert(fromRoot !== '..' && !fromRoot.startsWith(`..${process.platform === 'win32' ? '\\' : '/'}`) && !isAbsolute(fromRoot), `unsafe runtime asset path: ${path}`);
  return absolute;
}

function safeInstallPath(path, label = 'packaged runtime path') {
  assert(typeof path === 'string' && path.length > 0 && !isAbsolute(path), `unsafe ${label}: ${path}`);
  assert(!path.includes('\\') && !path.includes(':') && !path.split('/').includes('..'), `unsafe ${label}: ${path}`);
  const normalized = relative('/intern-package', resolve('/intern-package', path)).replaceAll('\\', '/');
  assert(normalized === path && normalized !== '..' && !normalized.startsWith('../'), `unsafe ${label}: ${path}`);
  return path;
}

async function sha256(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

function validateDigest(value, label) {
  assert(typeof value === 'string' && /^[0-9a-f]{64}$/.test(value), `${label} has an invalid SHA-256 digest`);
}

function verifyPins(manifest) {
  const byId = new Map(manifest.downloads.map((download) => [download.id, download]));
  assert(manifest.downloads.length === Object.keys(EXPECTED_DOWNLOADS).length, 'runtime download set must contain exactly the expected pinned entries');
  for (const [id, expected] of Object.entries(EXPECTED_DOWNLOADS)) {
    const download = byId.get(id);
    assert(download, `runtime manifest is missing the ${id} pin`);
    assert(download.version === expected.version, `${id} version pin changed`);
    assert(download.archive === expected.archive && !/[\\/]/.test(download.archive), `${id} archive basename changed or is unsafe`);
    assert(download.url === expected.url, `${id} URL pin changed`);
    assert(download.size === expected.size, `${id} size pin changed`);
    assert(download.sha256 === expected.sha256, `${id} SHA-256 pin changed`);
    assert(download.extract?.path === expected.extract?.path, `${id} extracted file pin changed`);
    assert(download.extract?.size === expected.extract?.size, `${id} extracted file size pin changed`);
    assert(download.extract?.sha256 === expected.extract?.sha256, `${id} extracted file SHA-256 pin changed`);
  }
  assert(manifest.vcpkg?.repository === 'https://github.com/microsoft/vcpkg.git', 'vcpkg repository pin changed');
  assert(manifest.vcpkg?.baseline === '644588ca32576d86325fb3fe3b6020042bee61b8', 'vcpkg baseline pin changed');
  assert(manifest.vcpkg?.triplet === 'x64-windows', 'vcpkg triplet pin changed');
  assert(manifest.vcpkg?.packages?.tesseract === '5.5.2', 'Tesseract version pin changed');
}

export async function verifyRuntimeAssets(manifestPath = DEFAULT_MANIFEST, options = {}) {
  const root = resolve(options.root ?? REPOSITORY_ROOT);
  const requireBundled = options.requireBundled ?? false;
  const requireExpectedPins = options.requireExpectedPins ?? false;
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  assert(manifest.schema_version === 1, 'runtime asset manifest schema must be 1');
  assert(Array.isArray(manifest.downloads), 'runtime asset manifest downloads must be an array');
  assert(Array.isArray(manifest.bundled_files), 'runtime asset manifest bundled_files must be an array');
  assert(Array.isArray(manifest.license_files), 'runtime asset manifest license_files must be an array');
  if (requireBundled) assert(manifest.bundled_files.length > 0, 'bundled runtime inventory is empty');
  if (requireBundled) assert(manifest.license_files.length > 0, 'bundled license inventory is empty');
  if (requireExpectedPins) verifyPins(manifest);
  const seen = new Set();
  for (const download of manifest.downloads) {
    assert(!seen.has(download.id), `duplicate runtime download id: ${download.id}`);
    seen.add(download.id);
    assert(Number.isSafeInteger(download.size) && download.size >= 0, `${download.id} has an invalid size`);
    validateDigest(download.sha256, download.id);
    if (download.extract !== undefined) {
      safeInstallPath(download.extract?.path, `${download.id} extracted file path`);
      assert(Number.isSafeInteger(download.extract.size) && download.extract.size >= 0, `${download.id} extracted file has an invalid size`);
      validateDigest(download.extract.sha256, `${download.id} extracted file`);
    }
  }
  let verifiedFiles = 0;
  const seenPaths = new Set();
  const seenInstallPaths = new Set();
  for (const file of manifest.bundled_files) {
    assert(!seenPaths.has(file.path), `duplicate bundled runtime path: ${file.path}`);
    seenPaths.add(file.path);
    safeInstallPath(file.install_path, 'packaged runtime path');
    assert(Array.isArray(file.packages) && file.packages.length > 0, `bundled runtime package identity is missing: ${file.path}`);
    const packageNames = new Set();
    for (const runtimePackage of file.packages) {
      assert(typeof runtimePackage?.name === 'string' && runtimePackage.name.length > 0, `runtime package name is missing: ${file.path}`);
      assert(typeof runtimePackage?.version === 'string' && runtimePackage.version.length > 0, `runtime package version is missing: ${file.path}`);
      const identity = `${runtimePackage.name}@${runtimePackage.version}`;
      assert(!packageNames.has(identity), `duplicate runtime package identity ${identity}: ${file.path}`);
      packageNames.add(identity);
    }
    assert(!seenInstallPaths.has(file.install_path), `duplicate packaged runtime path: ${file.install_path}`);
    seenInstallPaths.add(file.install_path);
    const path = safePath(root, file.path);
    validateDigest(file.sha256, file.path);
    let metadata;
    try { metadata = await lstat(path); } catch (error) {
      if (error?.code === 'ENOENT' && !requireBundled) continue;
      if (error?.code === 'ENOENT') throw new Error(`signed runtime asset is missing: ${file.path}`);
      throw error;
    }
    assert(metadata.isFile() && !metadata.isSymbolicLink(), `runtime asset is not a regular file: ${file.path}`);
    assert(metadata.size === file.size, `runtime asset size mismatch: ${file.path}`);
    assert(await sha256(path) === file.sha256, `runtime asset SHA-256 mismatch: ${file.path}`);
    verifiedFiles += 1;
  }
  const stagedOcr = OCR_RUNTIME_INSTALL_PATHS.filter((path) => seenInstallPaths.has(path));
  const missingOcr = OCR_RUNTIME_INSTALL_PATHS.filter((path) => !seenInstallPaths.has(path));
  const ocrRequired = requireBundled && requireExpectedPins;
  assert(missingOcr.length === 0 || (stagedOcr.length === 0 && !ocrRequired),
    `bundled OCR runtime is incomplete: missing ${missingOcr.join(', ')}`);
  let verifiedLicenses = 0;
  const seenLicensePaths = new Set();
  for (const file of manifest.license_files) {
    assert(!seenPaths.has(file.path), `duplicate signed source path: ${file.path}`);
    seenPaths.add(file.path);
    safeInstallPath(file.install_path, 'packaged license path');
    assert(file.install_path.startsWith('licenses/'), `packaged license path must be below licenses/: ${file.install_path}`);
    assert(!seenLicensePaths.has(file.install_path), `duplicate packaged license path: ${file.install_path}`);
    seenLicensePaths.add(file.install_path);
    const path = safePath(root, file.path);
    validateDigest(file.sha256, file.path);
    let metadata;
    try { metadata = await lstat(path); } catch (error) {
      if (error?.code === 'ENOENT' && !requireBundled) continue;
      if (error?.code === 'ENOENT') throw new Error(`signed license file is missing: ${file.path}`);
      throw error;
    }
    assert(metadata.isFile() && !metadata.isSymbolicLink(), `license asset is not a regular file: ${file.path}`);
    assert(metadata.size === file.size, `license asset size mismatch: ${file.path}`);
    assert(await sha256(path) === file.sha256, `license asset SHA-256 mismatch: ${file.path}`);
    verifiedLicenses += 1;
  }
  return { verifiedDownloads: manifest.downloads.length, verifiedFiles, verifiedLicenses };
}

async function runCli() {
  const requireBundled = process.argv.includes('--require-bundled');
  const result = await verifyRuntimeAssets(DEFAULT_MANIFEST, { root: REPOSITORY_ROOT, requireBundled, requireExpectedPins: true });
  process.stdout.write(`Verified ${result.verifiedDownloads} pinned downloads, ${result.verifiedFiles} bundled runtime files, and ${result.verifiedLicenses} license files.\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) await runCli();
