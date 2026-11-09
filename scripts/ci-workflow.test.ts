import { execFile } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { promisify } from 'node:util';
import { describe, expect, it } from 'vitest';

const exec = promisify(execFile);

/** One job of a workflow, from its key to the next job's. */
function jobOf(workflow: string, id: string): string {
  const lines = workflow.split('\n');
  const start = lines.indexOf(`  ${id}:`);
  expect(start, `job ${id}`).toBeGreaterThanOrEqual(0);
  const end = lines.findIndex((line, index) => index > start && /^ {2}[\w-]+:\s*$/.test(line));
  return lines.slice(start, end === -1 ? undefined : end).join('\n');
}

/** A job's steps, each from its `- ` line to the next step's. */
function stepsOf(job: string): string[] {
  return job.split(/\n(?= {6}- )/).slice(1);
}

const ci = () => readFile('.github/workflows/ci.yml', 'utf8');

describe('the runtime-asset cache in CI', () => {
  // CI_RELEASE_DOCS-1. The asset fetch rewrites runtime-assets.json, so a
  // save step that recomputed hashFiles() saved every run under a key that no
  // restore ever asked for: ~470 MB uploaded per run, and a 24-minute Tesseract
  // build every time.
  it('cache_save_reuses_primary_key: every save reuses the key the restore step computed', async () => {
    const steps = stepsOf(jobOf(await ci(), 'windows-release-gates'));
    const saves = steps.filter((step) => step.includes('actions/cache/save@'));
    expect(saves).toHaveLength(2);
    for (const save of saves) {
      expect(save).not.toContain('hashFiles(');
      expect(save).toContain('steps.assets-cache.outputs.cache-primary-key');
    }

    const [complete, partial] = saves;
    expect(complete).toContain("if: steps.fetch-assets.outcome == 'success' && steps.assets-cache.outputs.cache-hit != 'true'");
    expect(complete).toMatch(/^ {10}key: \$\{\{ steps\.assets-cache\.outputs\.cache-primary-key \}\}$/m);
    // A partial fetch is progress for the next run, under a key of its own: saved
    // under the primary key it would be restored as a hit and never completed.
    expect(partial).toContain("if: always() && steps.fetch-assets.outcome == 'failure'");
    // Never on cancellation: a killed fetch can leave a truncated download that
    // every run restoring it would fail on.
    expect(partial).not.toContain('cancelled');
    expect(partial).toContain('key: ${{ steps.assets-cache.outputs.cache-primary-key }}-partial-${{ github.run_id }}-${{ github.run_attempt }}');

    const restore = steps.find((step) => step.includes('actions/cache/restore@'))!;
    expect(restore).toContain('id: assets-cache');
    const key = /^ {10}key: (.+)$/m.exec(restore)![1];
    expect(restore).toContain(`restore-keys: ${key}-partial-`);

    const fetch = steps.findIndex((step) => step.includes('id: fetch-assets'));
    expect(steps[fetch]).toContain('fetch-windows-assets.ps1');
    expect(steps.indexOf(restore)).toBeLessThan(fetch);
    for (const save of saves) expect(steps.indexOf(save)).toBeGreaterThan(fetch);
  });

  it('concurrency_group_present: one run per pull request, canceling only superseded pull request runs', async () => {
    const workflow = await ci();
    const block = [
      'concurrency:',
      '  group: ci-${{ github.event.pull_request.number || github.ref }}',
      "  cancel-in-progress: ${{ github.event_name == 'pull_request' }}",
    ].join('\n');
    expect(workflow).toContain(`\n${block}\n`);
    expect(workflow.indexOf(block)).toBeLessThan(workflow.indexOf('\njobs:\n'));
  });
});

describe('the cargo caches in CI', () => {
  // Every release changes Cargo.lock. With the lock's hash as the whole key, the
  // release pull request and the merge after it each built everything cold:
  // twelve minutes of Windows tests, nine of bundling and seven of saving.
  it('restore the newest earlier entry when the lock changes, and key on cargo inputs alone', async () => {
    const workflow = await ci();
    const caches = ['corpus-replay', 'rust-ubuntu', 'linux-desktop', 'windows-release-gates'].flatMap((id) =>
      stepsOf(jobOf(workflow, id))
        .filter((step) => step.includes('actions/cache@') && /^ {12}target$/m.test(step))
        .map((step) => [id, step] as const),
    );
    expect(caches.map(([id]) => id)).toEqual(['corpus-replay', 'rust-ubuntu', 'linux-desktop', 'windows-release-gates']);
    for (const [id, step] of caches) {
      const key = /^ {10}key: (.+)$/m.exec(step)![1];
      const prefix = key.slice(0, key.indexOf('${{ hashFiles('));
      expect(prefix, id).toMatch(/^cargo-[\w-]*\$\{\{ runner\.os \}\}-$/);
      expect(step, id).toMatch(new RegExp(`^ {10}restore-keys: ${prefix.replace(/[$.{}]/g, '\\$&')}$`, 'm'));
      expect(key, id).not.toContain('runtime-assets.json');
    }
  });

  // With restore-keys, the entry saved after a lock change would carry the
  // earlier entry's builds of the workspace's own crates, which a release's
  // version bump leaves unused, and the next entry those and its own, release
  // after release, toward the repository's cache limit.
  it('drop the workspace builds an inexact restore brings back, before anything builds', async () => {
    const workflow = await ci();
    for (const id of ['corpus-replay', 'rust-ubuntu', 'linux-desktop', 'windows-release-gates']) {
      const steps = stepsOf(jobOf(workflow, id));
      const cache = steps.findIndex((step) => step.includes('actions/cache@') && /^ {12}target$/m.test(step));
      expect(steps[cache], id).toMatch(/^ {8}id: cargo-cache$/m);
      const prune = steps.findIndex((step) => /^ {8}run: bash scripts\/prune-cargo-workspace\.sh$/m.test(step));
      expect(prune, id).toBe(cache + 1);
      expect(steps[prune], id).toMatch(/^ {8}if: steps\.cargo-cache\.outputs\.cache-hit != 'true'$/m);
      expect(steps[prune], id).toMatch(/^ {8}shell: bash$/m);
      const code = (step: string) => step.split('\n').filter((line) => !/^\s*#/.test(line)).join('\n');
      const builds = steps.findIndex((step) => /\bcargo (build|run|test|clippy)\b|tauri build/.test(code(step)));
      expect(builds, id).toBeGreaterThan(prune);
    }
  });

  it.skipIf(process.platform === 'win32')('prune every build of a workspace package and keep third-party ones', async () => {
    const target = await mkdtemp(join(tmpdir(), 'prune-cargo-'));
    try {
      const files = {
        workspace: [
          'release/deps/libintern_engine-0123456789abcdef.rlib',
          'release/deps/libintern_engine-fedcba9876543210.rlib',
          'release/.fingerprint/intern-engine-0123456789abcdef/lib-intern_engine',
          'debug/build/intern-app-0123456789abcdef/output',
          'debug/deps/intern_bench-0123456789abcdef',
        ],
        thirdParty: ['debug/deps/libserde-0123456789abcdef.rlib', 'release/.fingerprint/serde-0123456789abcdef/lib-serde'],
      };
      for (const file of [...files.workspace, ...files.thirdParty]) {
        await mkdir(dirname(join(target, file)), { recursive: true });
        await writeFile(join(target, file), '');
      }
      await exec('bash', ['scripts/prune-cargo-workspace.sh'], { env: { ...process.env, CARGO_TARGET_DIR: target } });
      for (const file of files.workspace) expect(existsSync(join(target, file)), file).toBe(false);
      for (const file of files.thirdParty) expect(existsSync(join(target, file)), file).toBe(true);
    } finally {
      await rm(target, { recursive: true, force: true });
    }
  });

  // The asset cache holds a Tesseract built from source for 22 minutes; a key
  // that moved with the version threw it away on every release.
  it('key the Windows runtime assets on nothing a release bumps', async () => {
    const version = JSON.parse(await readFile('package.json', 'utf8')).version as string;
    const restore = stepsOf(jobOf(await ci(), 'windows-release-gates')).find((step) => step.includes('id: assets-cache'))!;
    const hashed = /^ {10}key: .*hashFiles\(([^)]*)\)/m.exec(restore)![1].split(',').map((file) => file.trim().replace(/'/g, ''));
    expect(hashed).toEqual(['src-tauri/resources/runtime-assets.json', 'scripts/fetch-windows-assets.ps1']);
    for (const file of hashed) expect(await readFile(file, 'utf8'), file).not.toContain(version);
  });
});

describe('the corpus replay job', () => {
  const sets = [
    ['real1', 'sample.json'],
    ['real2', 'sample-heldout.json'],
    ['real3', 'sample-heldout2.json'],
    ['real4', 'sample-heldout3.json'],
  ] as const;

  it('replays in release builds', async () => {
    const job = jobOf(await ci(), 'corpus-replay');
    const runs = job.split('\n').filter((line) => line.includes('cargo run'));
    expect(runs).toHaveLength(7);
    for (const line of runs) expect(line).toContain('cargo run --release --locked -p ');
    expect(job).toContain('key: cargo-replay-release-${{ runner.os }}-');
  });

  // The real sets are replayed from committed recordings against committed
  // baselines, with gold built from the committed samples: nothing fetched.
  it('holds each real-document set to its baseline, one step per set', async () => {
    const workflow = await ci();
    const steps = stepsOf(jobOf(workflow, 'corpus-replay'));
    const bench = await readFile('.github/workflows/candidate-bench.yml', 'utf8');
    for (const [set, sample] of sets) {
      // The same sample the live benchmark fetches for the set.
      expect(bench).toContain(`"${set}": "${sample}",`);
      const step = steps.find((candidate) => candidate.includes(`name: Replay the real-document set ${set} `));
      expect(step, set).toBeDefined();
      expect(step).toContain('if: always()');
      const recording = `bench/realdocs/recordings/${set}-qwen35-9b.json`;
      expect(step).toContain(`python3 -I scripts/realdocs-eval/fetch.py --gold-only \\\n            --recording ${recording} "$RUNNER_TEMP/realdocs/${set}" ${sample}`);
      expect(step).toContain(`run --corpus "$RUNNER_TEMP/realdocs/${set}/corpus" --gold "$RUNNER_TEMP/realdocs/${set}/gold.json"`);
      expect(step).toContain(`--replay ${recording} --baseline bench/realdocs/baseline-${set}.json`);
      expect(step).toContain(`--min-token-confidence 0.7 --output realdocs-${set}-report.json --markdown realdocs-${set}-report.md`);
      await readFile(recording);
      await readFile(`bench/realdocs/baseline-${set}.json`);
    }
    const upload = steps.find((step) => step.includes('actions/upload-artifact@'))!;
    expect(upload).toContain('realdocs-real*-report.json');
    expect(upload).toContain('realdocs-real*-report.md');
  });

  // A prompt change makes the recorded replies stale. That fails by design:
  // the change carries re-recorded recordings and baselines.
  it('never lets a stale reply through', async () => {
    const job = jobOf(await ci(), 'corpus-replay')
      .split('\n').filter((line) => !line.trimStart().startsWith('#')).join('\n');
    expect(job).not.toContain('--allow-stale');
  });
});

describe('the candidate benchmark', () => {
  const bench = () => readFile('.github/workflows/candidate-bench.yml', 'utf8');

  // Once a week the shipped accurate model runs live on every set, so a change
  // merged to main is measured within a week and the real sets' recordings are
  // made again before the last ones expire.
  it('runs the accurate model the app ships on every set once a week', async () => {
    const workflow = await bench();
    expect(workflow).toMatch(/^on:\n {2}schedule:\n {4}- cron: "\d+ \d+ \* \* \d"\n {2}workflow_dispatch:$/m);
    // A scheduled run has no inputs: no candidates means the manifest's model.
    expect(workflow).toMatch(/^ {6}candidates:\n(?: {8}.*\n)*? {8}required: false\n {8}default: ""$/m);
    const plan = jobOf(workflow, 'plan');
    expect(plan).toContain("SETS: ${{ inputs.sets || 'internbench,real1,real2,real3,real4' }}");
    expect(plan).toContain('sparse-checkout: src-tauri/resources/model-manifest.json');
    expect(plan).toContain('with open("src-tauri/resources/model-manifest.json", encoding="utf-8") as file:');
    expect(plan).toContain('if model["tier"] == "accurate"');
    // The file the engine embeds as its manifest, so the default is the model
    // crates/intern-engine/src/manifest.rs downloads, not a copy of its pins.
    const manifest = await readFile('crates/intern-engine/src/manifest.rs', 'utf8');
    expect(manifest).toContain('include_str!("../../../src-tauri/resources/model-manifest.json")');
    const pinned = JSON.parse(await readFile('src-tauri/resources/model-manifest.json', 'utf8'));
    const accurate = pinned.models.filter((model: { tier: string }) => model.tier === 'accurate');
    expect(accurate).toHaveLength(1);
    expect(accurate[0].model_id).toMatch(/^[a-z0-9._-]+$/);
    // Every step that reads the thread count has the dispatch default.
    for (const line of workflow.split('\n').filter((text) => text.includes('THREADS: ${{'))) {
      expect(line).toContain("THREADS: ${{ inputs.threads || '4' }}");
    }
  });

  it('keeps its recordings as long as a public repository can', async () => {
    const upload = stepsOf(jobOf(await bench(), 'bench')).find((step) => step.includes('actions/upload-artifact@'))!;
    expect(upload).toMatch(/^ {10}retention-days: 90$/m);
    expect(upload).toContain('/out/recording.json');
  });
});

describe('the Rust (Ubuntu) job', () => {
  it('ubuntu_rust_job_runs_fmt_clippy_test with the pinned toolchain and the desktop libraries', async () => {
    const job = jobOf(await ci(), 'rust-ubuntu');
    expect(job).toContain('name: Rust (Ubuntu)');
    expect(job).toContain('runs-on: ubuntu-latest');
    expect(job).toContain('rustup toolchain install 1.88.0 --profile minimal --component rustfmt,clippy');
    expect(job).toContain('rustup default 1.88.0');
    expect(job).toContain('sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev');
    expect(job).toContain('actions/cache@');
    expect(job).toMatch(/key: .*hashFiles\('Cargo\.lock'\)/);
    expect(job).toMatch(/^ {12}~\/\.cargo\/registry$/m);
    expect(job).toMatch(/^ {12}target$/m);
    // The build settings it was proven green with, which also keep the cached
    // target directory free of incremental state and debug info.
    for (const setting of ['CARGO_INCREMENTAL', 'CARGO_PROFILE_DEV_DEBUG', 'CARGO_PROFILE_TEST_DEBUG']) {
      expect(job).toMatch(new RegExp(`^ {6}${setting}: "0"$`, 'm'));
    }

    const runs = stepsOf(job).flatMap((step) => /run: (.+)$/m.exec(step)?.[1] ?? []);
    const fmt = runs.indexOf('cargo fmt --all -- --check');
    const clippy = runs.indexOf('cargo clippy --locked --workspace --all-targets -- -D warnings');
    const test = runs.indexOf('cargo test --locked --workspace --all-targets');
    expect(fmt).toBeGreaterThanOrEqual(0);
    expect(clippy).toBeGreaterThan(fmt);
    expect(test).toBeGreaterThan(clippy);
  });

  // Tauri's build script refuses to compile the desktop crate unless every
  // bundled sidecar and resource exists, so the Linux job has to stage exactly
  // what the Windows job stages, under its own target triple.
  it('stages every sidecar the bundle names, and the same resource placeholders as the Windows job', async () => {
    const workflow = await ci();
    const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
    const linux = stepsOf(jobOf(workflow, 'rust-ubuntu')).find((step) => step.includes('compile-placeholder'))!;
    const windows = stepsOf(jobOf(workflow, 'windows-release-gates')).find((step) => step.includes('compile-placeholder'))!;

    expect(linux).toContain('cargo build --locked -p intern-worker');
    expect(linux).toContain('cp target/debug/intern-worker src-tauri/binaries/intern-worker-x86_64-unknown-linux-gnu');
    for (const sidecar of config.bundle.externalBin as string[]) {
      expect(linux).toContain(`src-tauri/${sidecar}-x86_64-unknown-linux-gnu`);
      expect(windows).toContain(`src-tauri\\${sidecar.replace('/', '\\')}-x86_64-pc-windows-msvc.exe`);
    }
    const windowsPlaceholders = [...windows.matchAll(/New-Item -ItemType File (src-tauri\\resources\\\S+compile-placeholder\.\w+)/g)]
      .map((match) => match[1].replaceAll('\\', '/'));
    const linuxPlaceholders = [...linux.matchAll(/touch (src-tauri\/resources\/\S+compile-placeholder\.\w+)/g)].map((match) => match[1]);
    expect(windowsPlaceholders).toHaveLength(6);
    // Upstream llama.cpp's folder, a directory resource of its own.
    expect(windowsPlaceholders).toContain('src-tauri/resources/native/llama-upstream/compile-placeholder.dll');
    expect(linuxPlaceholders.sort()).toEqual(windowsPlaceholders.sort());
  });

  // A Linux build bundles one runtime folder in place of the sidecars and the
  // Windows DLLs (tauri.linux.conf.json), and its build script refuses to
  // compile without that folder too.
  it('stages the runtime folder the Linux bundle names', async () => {
    const linuxConfig = JSON.parse(await readFile('src-tauri/tauri.linux.conf.json', 'utf8'));
    expect(linuxConfig.bundle.externalBin).toEqual([]);
    expect(linuxConfig.bundle.resources['resources/runtime/']).toBe('runtime/');
    const linux = stepsOf(jobOf(await ci(), 'rust-ubuntu')).find((step) => step.includes('compile-placeholder'))!;
    expect(linux).toMatch(/^ {10}mkdir -p .* src-tauri\/resources\/runtime$/m);
    expect(linux).toMatch(/^ {10}touch src-tauri\/resources\/runtime\/compile-placeholder$/m);
  });
});

describe('the Windows builds', () => {
  // The release and QA builds stage the same placeholders as CI's Windows job
  // and take every one of them away again before the real assets are fetched:
  // one left behind would be bundled into the installer, and one missing would
  // stop the desktop crate compiling.
  it('creates and removes the same placeholders in every Windows build', async () => {
    const created = (step: string) => [...step.matchAll(/New-Item -ItemType File (src-tauri\\resources\\\S+compile-placeholder\.\w+)/g)].map((match) => match[1]).sort();
    const removed = (step: string) => [...step.matchAll(/Remove-Item (src-tauri\\resources\\\S+compile-placeholder\.\w+)/g)].map((match) => match[1]).sort();
    const jobs = [
      ['ci.yml', jobOf(await ci(), 'windows-release-gates')],
      ['release.yml', jobOf(await readFile('.github/workflows/release.yml', 'utf8'), 'release')],
      ['qa.yml', jobOf(await readFile('.github/workflows/qa.yml', 'utf8'), 'windows-whole-product-qa')],
    ] as const;
    const reference = created(stepsOf(jobs[0][1]).find((step) => step.includes('New-Item -ItemType File src-tauri\\resources\\'))!);
    expect(reference).toHaveLength(6);
    for (const [name, job] of jobs) {
      const steps = stepsOf(job);
      const stage = steps.findIndex((step) => step.includes('New-Item -ItemType File src-tauri\\resources\\'));
      // The invocation, not the cache steps whose keys hash the script.
      const fetch = steps.findIndex((step) => step.includes('./scripts/fetch-windows-assets.ps1'));
      expect(stage, name).toBeGreaterThanOrEqual(0);
      expect(fetch, name).toBeGreaterThan(stage);
      expect(created(steps[stage]), name).toEqual(reference);
      expect(removed(steps[fetch]), name).toEqual(reference);
      expect(steps[fetch].indexOf('Remove-Item'), name).toBeLessThan(steps[fetch].indexOf('./scripts/fetch-windows-assets.ps1'));
    }
  });
});

describe('the Linux desktop job', () => {
  it('builds the package from the pinned runtime and smoke-tests it installed, on Ubuntu 24.04', async () => {
    const job = jobOf(await ci(), 'linux-desktop');
    expect(job).toContain('runs-on: ubuntu-24.04');
    expect(job).toContain('rustup default 1.88.0');
    for (const library of ['libwebkit2gtk-4.1-dev', 'libayatana-appindicator3-dev', 'librsvg2-dev', 'xvfb', 'tesseract-ocr']) {
      expect(job).toContain(library);
    }
    // The runtime's downloads are cached, keyed by the scripts and manifest that pin them.
    expect(job).toContain("hashFiles('scripts/fetch-linux-assets.sh', 'scripts/fetch-ocr-runtime.sh', 'src-tauri/resources/runtime-assets.json')");

    const order = [
      'cargo build --locked -p intern-worker --release --features windows-native',
      'scripts/fetch-linux-assets.sh "${{ runner.temp }}/intern-linux-assets-cache"',
      'run: scripts/smoke-linux.sh\n',
      'npm run tauri build -- --bundles deb -- --locked',
      'sudo apt-get install -y ./target/release/bundle/deb/*.deb',
      'scripts/smoke-linux.sh --runtime /usr/lib/Intern --app /usr/bin/intern',
    ].map((command) => job.indexOf(command));
    for (const position of order) expect(position).toBeGreaterThan(0);
    expect([...order].sort((a, b) => a - b)).toEqual(order);
  });
});

describe('the Rust lock and format check', () => {
  // CI_RELEASE_DOCS-10: it used to regenerate the lock and format the tree,
  // and upload both, so it could never fail on either.
  it('fails on lock drift and unformatted code rather than regenerating them', async () => {
    // Comments stripped: the header explains what the old job ran, and a
    // bare absence check would fail on the explanation.
    const workflow = (await readFile('.github/workflows/lockfile.yml', 'utf8'))
      .split('\n').filter((line) => !line.trimStart().startsWith('#')).join('\n');
    expect(workflow).toMatch(/^name: Rust lock and format check$/m);
    expect(workflow).toContain('cargo metadata --locked --format-version 1 > /dev/null');
    expect(workflow).toContain('cargo fmt --all -- --check');
    expect(workflow).not.toContain('generate-lockfile');
    expect(workflow).not.toContain('upload-artifact');
    expect(workflow).not.toContain('codex/intern-v1');
    expect(workflow).not.toMatch(/^\s*run: cargo fmt --all\s*$/m);
  });
});

describe('the Pages deployment', () => {
  it('says how to enable Pages when it cannot deploy', async () => {
    const workflow = await readFile('.github/workflows/pages.yml', 'utf8');
    const remedy = stepsOf(jobOf(workflow, 'deploy')).find((step) => step.includes('if: failure()'));
    expect(remedy).toBeDefined();
    expect(remedy).toContain('Enable Pages: Settings → Pages → Source: GitHub Actions');
    expect(remedy).toContain('>> "$GITHUB_STEP_SUMMARY"');
    // A run the environment's rules refuse never reaches a step, so advice
    // for that case here could never be shown.
    expect(remedy).not.toMatch(/tag/i);
  });

  // The release publishes with the workflow's own token, and GitHub starts no
  // run for an event that token causes. A `release: published` trigger on
  // pages.yml therefore never fired for a release made by release.yml; only
  // a dispatch does.
  it('is started by the release once it has published, from main', async () => {
    const pages = await readFile('.github/workflows/pages.yml', 'utf8');
    expect(pages).toMatch(/^on:\n[\s\S]*^ {2}workflow_dispatch:$/m);
    expect(pages).not.toMatch(/^ {2}release:$/m);

    const release = await readFile('.github/workflows/release.yml', 'utf8');
    const job = jobOf(release, 'pages');
    expect(job).toContain('needs: release');
    expect(job).toMatch(/^ {4}permissions:\n {6}actions: write$/m);
    expect(job).toContain('GH_TOKEN: ${{ github.token }}');
    expect(job).toContain('gh workflow run pages.yml --repo "$GITHUB_REPOSITORY" --ref main');
    // The release is already out; a refused dispatch must not read as a
    // failed release that someone dispatches again.
    expect(job).toMatch(/if ! gh workflow run pages\.yml[^\n]*; then\n\s+echo '::warning::/);
    expect(release.indexOf('  pages:\n')).toBeGreaterThan(release.indexOf('gh release create $Tag'));
  });
});
