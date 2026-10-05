import { readFile } from 'node:fs/promises';
import { expect, it } from 'vitest';

it('runs non-publishing browser, Windows, model, and installer QA and uploads evidence', async () => {
  const workflow = await readFile('.github/workflows/qa.yml', 'utf8');
  expect(workflow).toContain('workflow_dispatch:');
  expect(workflow).toContain('contents: read');
  expect(workflow).not.toContain('contents: write');
  expect(workflow).not.toContain('gh release create');
  expect(workflow).not.toContain('git push');
  expect(workflow).toContain('INTERN_QA_CAPTURE: "1"');
  expect(workflow).toContain('docs/qa/latest-implementation.png');
  expect(workflow).toContain('intern-evaluate');
  expect(workflow).toContain('run-model-evaluation.ps1');
  expect(workflow).toContain('smoke-installer.ps1');
  expect(workflow).toContain('validate-model-evaluation.mjs');
  expect(workflow).toContain('create-release-evidence.mjs');
  expect(workflow).toContain('validate-release-evidence.mjs');
  expect(workflow).toContain('docs/qa/release-evidence-manifest.json');
  expect(workflow).toContain('if: always()');
  expect(workflow).toContain('actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4');
});

it('records bounded Rust workspace test evidence without a PowerShell pipe', async () => {
  const workflow = await readFile('.github/workflows/qa.yml', 'utf8');
  const startMarker = '      - name: Run Rust workspace tests with evidence log';
  const endMarker = '      - name: Build parser worker and the corpus evaluator';
  const start = workflow.indexOf(startMarker);
  const end = workflow.indexOf(endMarker);
  expect(start).toBeGreaterThanOrEqual(0);
  expect(end).toBeGreaterThan(start);
  const rustTestStep = workflow.slice(start, end);

  expect(rustTestStep).toContain('timeout-minutes: 45');
  expect(rustTestStep).toContain('New-Item -ItemType Directory docs\\qa\\logs -Force | Out-Null');
  expect(rustTestStep).toContain('& cmd /c "cargo test --locked --workspace --all-targets > docs\\qa\\logs\\cargo-test.log 2>&1"');
  expect(rustTestStep).toContain('$TestExit = $LASTEXITCODE');
  expect(rustTestStep).toContain('Get-Content -LiteralPath docs\\qa\\logs\\cargo-test.log');
  expect(rustTestStep).toContain('if ($TestExit -ne 0) { throw "Rust workspace tests failed" }');
  expect(rustTestStep).not.toContain('| Tee-Object');
});

it('never publishes as a side effect of merging to main', async () => {
  const workflow = await readFile('.github/workflows/release.yml', 'utf8');
  // Releasing is dispatched deliberately; a merge must not tag or publish.
  expect(workflow).toContain('workflow_dispatch:');
  expect(workflow).not.toContain('branches: [main]');
  expect(workflow).toContain('group: intern-v0.1.0-alpha.11-release');
  expect(workflow).toContain('release_target:');
});

it('gates the exact main commit before creating its annotated tag and publishing', async () => {
  const workflow = await readFile('.github/workflows/release.yml', 'utf8');
  expect(workflow).toContain('cargo build --locked -p intern-engine --release --bin intern-evaluate');
  expect(workflow).toContain('run-model-evaluation.ps1');
  expect(workflow).toContain('-OutputPath release\\model-evaluation.json');
  expect(workflow).toContain('create-release-evidence.mjs');
  expect(workflow).toContain('validate-release-evidence.mjs');
  expect(workflow).toContain('git tag -a $Tag $env:GITHUB_SHA');
  expect(workflow).toContain('git push origin "refs/tags/$Tag"');
  expect(workflow.indexOf('validate-release-evidence.mjs')).toBeLessThan(workflow.indexOf('git tag -a $Tag $env:GITHUB_SHA'));
  expect(workflow.indexOf('validate-release-evidence.mjs')).toBeLessThan(workflow.indexOf('gh release create'));
  expect(workflow.indexOf('git tag -a $Tag $env:GITHUB_SHA')).toBeLessThan(workflow.indexOf('gh release create'));
  expect(workflow).toContain('gh release view $Tag --json isDraft');
  // The release ships the capture a reviewer inspected rather than taking a new
  // one: a sign-off cannot carry the digest of an image that does not exist yet,
  // and the image reviewed must be the image published. Fresh captures come from
  // the QA workflow, which still sets INTERN_QA_CAPTURE.
  expect(workflow).not.toContain('INTERN_QA_CAPTURE: "1"');
  expect(workflow).toContain('docs/qa/latest-implementation.png is missing');
  expect(workflow).toContain('release\\cargo-test.log');
  expect(workflow).toContain('--log=release/cargo-test.log');
  expect(workflow).toContain('Copy-Item docs\\qa\\latest-implementation.png $Release');
  expect(workflow).toContain('--screenshot=release/latest-implementation.png');
  expect(workflow).toContain('--fidelity-signoff=release/rendered-fidelity-signoff.json');
});

// Write-Host writes to the information stream, not the pipeline, so
// `2>&1 | Tee-Object` produced no log file at all for the smoke scripts while
// their output still appeared in the step log - the step looked healthy and the
// release died three steps later on a missing artifact. Any pipeline that tees a
// script reporting via Write-Host has to redirect every stream.
it('captures smoke output that is written to the host, not just stdout', async () => {
  const [release, qa, installer, worker] = await Promise.all([
    readFile('.github/workflows/release.yml', 'utf8'),
    readFile('.github/workflows/qa.yml', 'utf8'),
    readFile('scripts/smoke-installer.ps1', 'utf8'),
    readFile('scripts/smoke-worker.ps1', 'utf8'),
  ]);

  // The premise: these scripts report through Write-Host. If that ever changes
  // to Write-Output the redirect is harmless, but this test explains itself.
  expect(installer).toContain('Write-Host');
  expect(worker).toContain('Write-Host');

  for (const [name, workflow] of [['release.yml', release], ['qa.yml', qa]] as const) {
    const teed = workflow.split('\n').filter((line) => line.includes('Tee-Object') && line.includes('smoke-installer.ps1'));
    expect(teed.length, `${name} should tee the installer smoke log`).toBe(1);
    expect(teed[0], `${name} must redirect all streams, not just stderr`).toContain('*>&1');
  }
  // And the log must be checked where it is written, not where it is consumed.
  expect(release).toContain('Installer smoke produced no log');
});

// Tauri v2 signs the NSIS installer itself and writes <installer>.exe.sig
// beside it. There is no separate .nsis.zip package - looking for one failed a
// release with "Updater artifact was not produced" on a build that had signed
// correctly and logged "Finished 1 updater signature at: ...x64-setup.exe.sig".
it('publishes an update manifest pointing at the signed installer', async () => {
  const workflow = await readFile('.github/workflows/release.yml', 'utf8');
  expect(workflow).toContain('$SignatureFile = "$($Installer.FullName).sig"');
  // The glob form, not the bare extension: the surrounding comment explains
  // what .nsis.zip was and why it is gone, so matching the bare string would
  // fail on the explanation itself.
  expect(workflow).not.toContain('*.nsis.zip');
  // The manifest must name the asset that is actually uploaded and carry the
  // signature inline, because the updater reads it from here rather than
  // fetching a .sig.
  expect(workflow).toContain('Join-Path $Release "latest.json"');
  expect(workflow).toContain('signature = $Signature');
  expect(workflow).toContain('releases/download/$env:RELEASE_TAG/$($Installer.Name)');
  // A build with no signing key must fail loudly rather than publish a release
  // whose update button can never work.
  expect(workflow).toContain('TAURI_SIGNING_PRIVATE_KEY is not set');
  expect(workflow).toContain('src-tauri/tauri.release.conf.json');

  // The endpoint compiled into the app is /releases/latest/download/latest.json,
  // and GitHub resolves /releases/latest to the newest NON-prerelease release.
  // Publishing as a prerelease made every installed copy's update check 404
  // against a manifest that existed one URL away. A shipped build cannot be
  // pointed somewhere else, so this has to stay right at publish time.
  const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
  expect(config.plugins.updater.endpoints[0]).toContain('/releases/latest/download/latest.json');
  // Scoped to the gh commands, not the whole file: the comment above explains
  // what --prerelease did, so a bare absence check would fail on the
  // explanation. That mistake has now been made twice in this file.
  const publishCommands = workflow.split('\n').filter((line) => /^\s*gh release (create|edit)\b/.test(line));
  expect(publishCommands.length).toBeGreaterThan(0);
  for (const command of publishCommands) {
    expect(command, 'a prerelease is unreachable via /releases/latest').not.toContain('--prerelease');
  }
  expect(workflow).toContain('gh release create $Tag $Assets --verify-tag --latest');
});

it('attests the installer, and only after its evidence has been accepted', async () => {
  const workflow = await readFile('.github/workflows/release.yml', 'utf8');
  // Provenance is keyless, so there is no certificate or secret to store, and the
  // permissions have to be granted explicitly for the OIDC token to exist.
  expect(workflow).toContain('id-token: write');
  expect(workflow).toContain('attestations: write');
  expect(workflow).toContain('actions/attest-build-provenance@96b4a1ef7235a096b17240c259729fdd70c83d45 # v2');
  expect(workflow).toContain('subject-path: release/*_x64-setup.exe');
  // Ordering is the substance of this test. Attesting before validation would
  // publish a signed claim about a build that failed its own gates, and
  // attesting after publication would leave the downloadable file unattested.
  expect(workflow.indexOf('validate-release-evidence.mjs')).toBeLessThan(
    workflow.indexOf('actions/attest-build-provenance@96b4a1ef7235a096b17240c259729fdd70c83d45'),
  );
  expect(workflow.indexOf('actions/attest-build-provenance@96b4a1ef7235a096b17240c259729fdd70c83d45')).toBeLessThan(
    workflow.indexOf('gh release create'),
  );
});

/** One job of a workflow, from its key to the next job's. */
function jobOf(workflow: string, id: string): string {
  const lines = workflow.split('\n');
  const start = lines.indexOf(`  ${id}:`);
  expect(start, `job ${id}`).toBeGreaterThanOrEqual(0);
  const end = lines.findIndex((line, index) => index > start && /^ {2}[\w-]+:\s*$/.test(line));
  return lines.slice(start, end === -1 ? undefined : end).join('\n');
}

/** A workflow's steps, each from its `- ` line to the next step's. */
function stepsOf(text: string): string[] {
  return text.split(/\n(?= {6}- )/).slice(1);
}

// CI_RELEASE_DOCS-6. Dispatched after a run that tagged its commit and then
// failed to publish, the old "no-op" branch exited 0, skipped the release job,
// and showed a green run with nothing published.
it('tag_mismatch_is_fatal: a tag at another commit fails the run and says how to recover', async () => {
  const target = jobOf(await readFile('.github/workflows/release.yml', 'utf8'), 'release_target');
  const mismatch = target.indexOf('!= "$GITHUB_SHA"');
  expect(mismatch).toBeGreaterThan(0);
  const after = target.slice(mismatch);
  expect(after).toContain('echo "::error::$tag already points at $(git rev-parse "refs/tags/$tag^{commit}"), not $GITHUB_SHA. Delete the tag or bump the version."');
  expect(after).toMatch(/::error::[^\n]*\n\s*exit 1\n/);
  expect(after).not.toContain('exit 0');
  expect(target).not.toContain('eligible=false');
});

// CI_RELEASE_DOCS-3. Everything the final evidence validation needs from the
// commit itself is checked before the ninety-minute Windows job starts.
it('preflights the sign-off, notes, and versions before the release job can start', async () => {
  const release = await readFile('.github/workflows/release.yml', 'utf8');
  const target = jobOf(release, 'release_target');
  expect(target).toContain('runs-on: ubuntu-latest');
  expect(target).toContain('run: node scripts/release-preflight.mjs --root=. --workflow="$GITHUB_WORKFLOW"');
  expect(jobOf(release, 'release')).toContain('needs: release_target');
  // The lead's call: no new code inspects whether a secret exists. The signing
  // key is checked where it is used, in the build step, as it always was.
  expect(target).not.toContain('secrets.');
});

it('sets a stale sign-off aside on the QA runner before anything else runs', async () => {
  const steps = stepsOf(jobOf(await readFile('.github/workflows/qa.yml', 'utf8'), 'windows-whole-product-qa'));
  expect(steps[0]).toContain('actions/checkout@');
  expect(steps[1]).toContain('node scripts/hash-release-inputs.mjs');
  expect(steps[1]).toContain('node scripts/release-preflight.mjs --set-aside-stale-signoff --root=.');
});

// CI_RELEASE_DOCS-8. Release bounds its scoring step at an hour because a wedged
// llama-server once held a job open; QA had no bound and no log.
it('qa_scoring_has_timeout_and_log', async () => {
  const steps = stepsOf(jobOf(await readFile('.github/workflows/qa.yml', 'utf8'), 'windows-whole-product-qa'));
  const scoring = steps.find((step) => step.includes('run-model-evaluation.ps1'))!;
  expect(scoring).toMatch(/^ {8}timeout-minutes: 60$/m);
  expect(scoring).toContain('-OutputPath docs\\qa\\model-evaluation.json 2>&1 | Tee-Object -FilePath docs\\qa\\logs\\model-evaluation.log');
  expect(scoring).toContain('New-Item -ItemType Directory docs\\qa\\logs -Force');
});
