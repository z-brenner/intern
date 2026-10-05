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

  it('concurrency_group_present: one run per pull request, cancelling only superseded pull request runs', async () => {
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
    expect(windowsPlaceholders).toHaveLength(4);
    expect(linuxPlaceholders.sort()).toEqual(windowsPlaceholders.sort());
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
  it('redeploys on a published release and says how to enable Pages when it cannot deploy', async () => {
    const workflow = await readFile('.github/workflows/pages.yml', 'utf8');
    expect(workflow).toMatch(/^ {2}release:\n {4}types: \[published\]$/m);
    const remedy = stepsOf(jobOf(workflow, 'deploy')).find((step) => step.includes('if: failure()'));
    expect(remedy).toBeDefined();
    expect(remedy).toContain('Enable Pages: Settings → Pages → Source: GitHub Actions');
    expect(remedy).toContain('>> "$GITHUB_STEP_SUMMARY"');
  });
});
