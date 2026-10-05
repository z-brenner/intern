import { readFile, readdir } from 'node:fs/promises';
import { expect, it } from 'vitest';

const version = '0.1.0-alpha.11';
const tag = `v${version}`;

it('keeps every current alpha.3 release surface synchronized without rewriting historical alpha.2 records', async () => {
  const [packageJson, packageLock, workspace, tauri, workerProtocol, smoke, sbom, assets, notices, readme, checklist, release, ci, notes, evidenceValidator] = await Promise.all([
    readFile('package.json', 'utf8').then(JSON.parse),
    readFile('package-lock.json', 'utf8').then(JSON.parse),
    readFile('Cargo.toml', 'utf8'),
    readFile('src-tauri/tauri.conf.json', 'utf8').then(JSON.parse),
    readFile('crates/intern-worker/tests/protocol.rs', 'utf8'),
    readFile('scripts/smoke-worker.ps1', 'utf8'),
    readFile('scripts/generate-sbom.ps1', 'utf8'),
    readFile('scripts/fetch-windows-assets.ps1', 'utf8'),
    readFile('src-tauri/resources/THIRD_PARTY_NOTICES.md', 'utf8'),
    readFile('README.md', 'utf8'),
    readFile('docs/qa/release-checklist.md', 'utf8'),
    readFile('.github/workflows/release.yml', 'utf8'),
    readFile('.github/workflows/ci.yml', 'utf8'),
    readFile(`docs/releases/${tag}.md`, 'utf8'),
    readFile('scripts/validate-release-evidence.mjs', 'utf8'),
  ]);

  expect(packageJson.version).toBe(version);
  expect(packageLock.version).toBe(version);
  expect(packageLock.packages[''].version).toBe(version);
  expect(workspace).toContain(`version = "${version}"`);
  expect(tauri.version).toBe(version);
  expect(workerProtocol).toContain(`worker_version":"${version}`);
  expect(smoke).toContain(`worker_version -ne "${version}"`);
  expect(sbom).toContain(`-Version "${version}"`);
  expect(sbom).toContain(`Intern-v${version}.spdx.json`);
  expect(assets).toContain(`version = "${version}"`);
  expect(notices).toContain(`Intern ${version}`);
  expect(readme).toContain(`Intern_${version}_x64-setup.exe`);
  expect(checklist).toContain(`Intern v${version} release checklist`);
  expect(checklist).toContain('pending/blocked');
  expect(notes).toContain(`# Intern ${tag}`);
  expect(notes).toContain('not Authenticode signed');
  expect(release).toContain(`name: Release ${tag}`);
  expect(release).toContain(`RELEASE_TAG: ${tag}`);
  expect(release).toContain(`docs/releases/${tag}.md`);
  expect(release).toContain(`--title 'Intern ${tag}'`);
  expect(release).toContain(`group: intern-${tag}-release`);
  expect(ci).toContain(`intern-${tag}-windows-`);
  // The final evidence validation accepts exactly one workflow name, written
  // as a regular expression. Only a release run reaches it, so a bump that
  // missed it used to fail after the whole Windows build.
  expect(evidenceValidator).toContain(`/^Release v(${version.replaceAll('.', '\\.')})$/`);
  expect(evidenceValidator).toContain(`must be exactly Release ${tag}`);
});

/**
 * Every workflow in the repository, by file name. Read from the directory and
 * looked up by name rather than by position in a hand-kept list: removing or
 * adding one workflow used to shift which text the gate-order checks below
 * were reading, and a new workflow escaped the pinning check entirely.
 */
async function workflowsByName() {
  const names = (await readdir('.github/workflows')).filter((name) => /\.ya?ml$/.test(name)).sort();
  return new Map(await Promise.all(names.map(async (name) => [name, await readFile(`.github/workflows/${name}`, 'utf8')] as const)));
}

it('locates workflows by name, so the checks read the file they mean', async () => {
  const workflows = await workflowsByName();
  for (const name of ['ci.yml', 'lockfile.yml', 'pages.yml', 'qa.yml', 'release.yml']) expect(workflows.has(name), name).toBe(true);
  expect(workflows.get('release.yml')).toMatch(/^name: Release v/m);
  expect(workflows.get('qa.yml')).toMatch(/^name: Whole-product QA evidence$/m);
});

it('pins every release-critical action to an immutable commit and preserves the release gate order', async () => {
  const workflows = await workflowsByName();
  for (const [name, workflow] of workflows) {
    const uses = workflow.split('\n').filter((line) => line.includes('uses: actions/'));
    expect(uses.length, name).toBeGreaterThan(0);
    for (const line of uses) expect(line, name).toMatch(/actions\/[\w/-]+@[a-f0-9]{40}\s+# v\d+/);
  }
  const release = workflows.get('release.yml')!;
  const gateMarkers = [
    'cargo run --locked -p intern-release-verifier --',
    'SHA256SUMS.txt',
    'validate-release-evidence.mjs',
    'attest-build-provenance@',
    'git tag -a $Tag $env:GITHUB_SHA',
    'gh release create',
  ];
  const gateIndexes = gateMarkers.map((marker) => release.indexOf(marker));
  for (const index of gateIndexes) expect(index).toBeGreaterThanOrEqual(0);
  for (let index = 0; index < gateIndexes.length - 1; index += 1) {
    expect(gateIndexes[index]).toBeLessThan(gateIndexes[index + 1]);
  }
});
