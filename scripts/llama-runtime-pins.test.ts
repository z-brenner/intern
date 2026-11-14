import { readFile } from 'node:fs/promises';
import { expect, it } from 'vitest';

// The Linux package's llama.cpp is pinned in scripts/fetch-linux-assets.sh and
// the Windows package's in runtime-assets.json, each to its own platform's
// archive, and a stored reply's runtime identity names that platform's archive
// (tiers::pinned_runtime_build, which a Rust test holds to the Linux script).
// The architecture and the speed reports speak of one llama.cpp release, b11946
// today, in both packages, so the two pins must name the same release: one
// moved without the other would ship two builds under one release's claims.
it('pins the same llama.cpp release for the Linux and the Windows package', async () => {
  const [linux, assets] = await Promise.all([
    readFile('scripts/fetch-linux-assets.sh', 'utf8'),
    readFile('src-tauri/resources/runtime-assets.json', 'utf8').then(JSON.parse),
  ]);
  const release = linux.match(/^LLAMA_UPSTREAM_RELEASE="([^"]+)"$/m)?.[1];
  const sha256 = linux.match(/^LLAMA_UPSTREAM_SHA256="([0-9a-f]{64})"$/m)?.[1];
  const windows = (assets.downloads as { id: string; version: string; sha256: string; archive: string }[])
    .find((download) => download.id === 'llama.cpp-upstream');

  expect(release).toMatch(/^b\d+$/);
  expect(sha256).toBeDefined();
  expect(windows).toBeDefined();
  expect(windows?.version).toBe(release);
  expect(windows?.archive).toContain(`llama-${release}-bin-win-`);
  // Each pin is its own platform's archive, not a copy of the other's.
  expect(windows?.sha256).not.toBe(sha256);
});
