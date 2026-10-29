import { readFile } from 'node:fs/promises';
import { expect, it } from 'vitest';

// The characters scripts/generate-sbom.ps1 accepts in a runtime package's name
// and version. Anything else - a space above all - makes the SBOM tool
// reference that package's SBOM twice, which failed alpha.12's first release
// run ninety minutes in; this catches it on the pull request instead.
const SBOM_SAFE = /^[A-Za-z0-9._~/-]+$/;

it('gives every runtime package an identity the release SBOM can reference once', async () => {
  const sbom = await readFile('scripts/generate-sbom.ps1', 'utf8');
  expect(sbom).toContain("'^[A-Za-z0-9._~/-]+$'");

  const windows = await readFile('scripts/fetch-windows-assets.ps1', 'utf8');
  const names = [
    ...[...windows.matchAll(/(?:Copy-SidecarDll|Register-RuntimePackage)\s+\$\w+\s+"([^"]+)"/g)].map((m) => m[1]),
    ...[...windows.matchAll(/"ocr-[a-z-]+"\s*=\s*"([^"]+)"/g)].map((m) => m[1]),
  ];
  const linux = await readFile('scripts/fetch-linux-assets.sh', 'utf8');
  for (const m of linux.matchAll(/^owned_by\s+\S+\s+(?:"([^"]+)"|(\S+))\s+(\S+)/gm)) {
    names.push(m[1] ?? m[2]);
    if (!m[3].startsWith('"$') && !m[3].startsWith('$')) names.push(m[3]);
  }
  const manifest = JSON.parse(await readFile('src-tauri/resources/runtime-assets.json', 'utf8'));
  const versions = manifest.downloads.map((download: { version: string }) => download.version);

  // Guard against a pattern that silently stops matching. llvm-openmp owns the
  // OpenMP runtime that ships beside upstream llama.cpp.
  expect(names).toEqual(expect.arrayContaining(['onnxruntime', 'llama.cpp', 'llvm-openmp', 'PDFium', 'tessdata_fast']));
  expect(names.filter((name) => name.startsWith('PaddlePaddle/'))).toHaveLength(3);
  expect(versions.length).toBeGreaterThan(5);

  expect(names.filter((name) => !SBOM_SAFE.test(name))).toEqual([]);
  expect(versions.filter((version: string) => !SBOM_SAFE.test(version))).toEqual([]);
});
