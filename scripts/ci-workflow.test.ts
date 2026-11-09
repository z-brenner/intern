import { readFile } from 'node:fs/promises';
import { describe, expect, it } from 'vitest';

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

describe('the server speed battery', () => {
  const battery = async () => stepsOf(jobOf(await readFile('.github/workflows/speed-battery.yml', 'utf8'), 'battery'));

  // ggml loads its CPU backends from the folder /proc/self/exe resolves to,
  // so a folder of symlinks to the release loads the release's own best
  // variant: an AMX or AVX-512 path no laptop has. Hard links are files of
  // their own, and a folder holding one variant can load only that one.
  it('pins one CPU backend with hard links, before any server starts', async () => {
    const steps = await battery();
    const pin = steps.findIndex((step) => step.includes('name: Pin the CPU backend'));
    expect(pin).toBeGreaterThan(steps.findIndex((step) => step.includes('name: Fetch llama-server')));
    const step = steps[pin];
    expect(step).toContain("if: inputs.cpu_variant != ''");
    expect(step).toContain('cp -a --link "$source/." "$pinned/"');
    expect(step).not.toMatch(/ln -s|--symbolic/);
    expect(step).toContain(`! -name "libggml-cpu-$CPU_VARIANT.so" -delete`);
    expect(step).toContain('echo "LLAMA_SERVER=$pinned/');
    for (const name of ['Measure the cold start', 'Run the battery']) {
      expect(steps.findIndex((s) => s.includes(`name: ${name}`)), name).toBeGreaterThan(pin);
    }
    // The backend the server loads is recorded, before any server starts,
    // and the summary names it, so a pin that did not take is visible.
    const record = steps.findIndex((s) => s.includes('name: Record the CPU backend'));
    expect(record).toBeGreaterThan(pin);
    expect(steps[record]).toContain('"$LLAMA_SERVER" -lv 4 --list-devices');
    expect(steps.find((s) => s.includes('name: Summarize'))).toContain('loaded CPU backend from');
  });

  // The battery warms every slot before it times anything, so the first
  // document of a server lifetime - the one a slot restore speeds up - is
  // timed only by the cold-start step, on fresh servers, before the battery.
  it('times the cold start on fresh servers before the warmed battery', async () => {
    const steps = await battery();
    const cold = steps.findIndex((step) => step.includes('name: Measure the cold start'));
    expect(cold).toBeGreaterThan(steps.findIndex((step) => step.includes('name: Fetch and verify the model')));
    expect(cold).toBeLessThan(steps.findIndex((step) => step.includes('name: Run the battery')));
    expect(steps[cold]).toContain('if: inputs.cold_start');
    expect(steps[cold]).toContain('scripts/bonsai-eval/cold-start.py');
    const upload = steps.find((step) => step.includes('actions/upload-artifact@'))!;
    expect(upload).toContain('out/cold-start/cold-start.json');
    const script = await readFile('scripts/bonsai-eval/cold-start.py', 'utf8');
    // The app's launch, with the slot directory and a key the slot calls carry.
    for (const flag of ['"--cache-ram", "0", "-ub", "256", "--prio", "-1", "--poll", "0", "--no-mmproj"', '"--slot-save-path"', '"--api-key"']) {
      expect(script).toContain(flag);
    }
    expect(script).toContain('add_header("Authorization", f"Bearer {key}")');
  });

  // The OCR comparison runs on a runner of its own, both workers built here
  // from source with the shipped features, alternated, every reading kept.
  it('times this commit\'s parser worker against the base\'s on one runner', async () => {
    const workflow = await readFile('.github/workflows/speed-battery.yml', 'utf8');
    const job = jobOf(workflow, 'ocr-pages');
    expect(job).toContain('if: inputs.ocr_pages');
    expect(job).toContain('fetch-depth: 0');
    const steps = stepsOf(job);
    const at = (name: string) => steps.findIndex((step) => step.includes(`name: ${name}`));
    const head = at("Build this commit's parser worker");
    const stage = at('Stage the runtime as the package lays it out');
    const base = at("Build the base's parser worker");
    const time = at('Time the two workers, alternated');
    expect(head).toBeGreaterThan(-1);
    expect(stage).toBeGreaterThan(head);
    expect(base).toBeGreaterThan(stage);
    expect(time).toBeGreaterThan(base);
    for (const index of [head, base]) {
      expect(steps[index]).toContain('cargo build --locked --release -p intern-worker --features windows-native');
    }
    expect(steps[base]).toContain('git worktree add --detach "$LAB/base"');
    expect(steps[base]).toContain('--target-dir "$GITHUB_WORKSPACE/target"');
    expect(steps[time]).toContain('INTERN_RUNTIME_DIR: ${{ runner.temp }}/ocr-pages/stage/runtime');
    expect(steps[time]).toContain('scripts/bonsai-eval/ocr-pages.py --base "$LAB/worker-base" --head "$LAB/worker-head"');
    expect(steps[time]).toContain('--rounds 3');
    expect(steps.find((step) => step.includes('actions/upload-artifact@'))).toContain('ocr-pages/out/');
    const script = await readFile('scripts/bonsai-eval/ocr-pages.py', 'utf8');
    expect(script).toContain('PROTOCOL_VERSION = 1');
    expect(script).toContain('VmHWM');
  });

  // The app's own launch and slot code against the real server: the test
  // the workflow names must exist, and a run that did not pass it fails.
  it('runs the app\'s live slot test on the pinned server, and fails when it does', async () => {
    const workflow = await readFile('.github/workflows/speed-battery.yml', 'utf8');
    expect(jobOf(workflow, 'battery')).toContain('if: inputs.battery');
    const job = jobOf(workflow, 'live-slots');
    expect(job).toContain('if: inputs.live_slots');
    const steps = stepsOf(job);
    const fetch = steps.find((step) => step.includes('name: Fetch and verify llama-server'))!;
    expect(fetch).toContain('echo "$LLAMA_UPSTREAM_SHA256  $LAB/llama/$archive" | sha256sum --check --strict');
    const run = steps.find((step) => step.includes('name: Run the live slot-restore test'))!;
    const test = /cargo test --locked -p intern-app --lib (\w+)/.exec(run)![1];
    expect(await readFile('src-tauri/src/commands.rs', 'utf8')).toContain(`fn ${test}`);
    expect(run).toContain('INTERN_LIVE_LLAMA_SERVER="$LLAMA_SERVER" INTERN_LIVE_MODEL="$MODEL"');
    expect(run).toContain("grep -q 'test result: ok. 1 passed'");
    expect(steps.find((step) => step.includes('actions/upload-artifact@'))).toContain('live-slots/out/');
  });
});
