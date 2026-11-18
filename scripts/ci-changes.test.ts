import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, rename, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { changedPaths, windowsJobsFor } from './ci-changes.mjs';

describe('the Windows jobs a pull request needs', () => {
  it('skips both for documentation alone, such as a release sign-off', () => {
    // The files the alpha.19 sign-off commit changed.
    expect(windowsJobsFor([
      'docs/qa/fidelity-ledger.md',
      'docs/qa/latest-implementation.png',
      'docs/qa/release-checklist.md',
      'docs/qa/rendered-fidelity-signoff.json',
    ])).toEqual({ rust: false, package: false });
    expect(windowsJobsFor(['README.md', 'SECURITY.md', 'LICENSE', 'site/index.html', 'docs/releasing.md']))
      .toEqual({ rust: false, package: false });
  });

  it('packages but does not test Rust for the React app alone', () => {
    expect(windowsJobsFor(['src/App.tsx', 'tests/e2e/queue.spec.ts', 'index.html', 'vite.config.ts', 'tsconfig.json']))
      .toEqual({ rust: false, package: true });
  });

  it('runs everything for anything else, including files it has never seen', () => {
    for (const path of [
      'crates/intern-worker/src/main.rs',
      'src-tauri/src/lib.rs',
      'src-tauri/resources/runtime-assets.json',
      'Cargo.lock',
      'package.json',
      'package-lock.json',
      'fixtures/generate-fixtures.mjs',
      'bench/gold.json',
      'scripts/fetch-windows-assets.ps1',
      '.github/workflows/ci.yml',
      'integrations/power-automate/flow.json',
      'something-new/file.txt',
      // Markdown is documentation only at the root and under docs/: these
      // are bundled, read by tests, or both.
      'src-tauri/resources/THIRD_PARTY_NOTICES.md',
      'fixtures/generated/meeting-minutes.md',
    ]) {
      expect(windowsJobsFor(['docs/releasing.md', path]), path).toEqual({ rust: true, package: true });
    }
  });

  it('runs everything when it cannot tell what changed', () => {
    expect(windowsJobsFor([])).toEqual({ rust: true, package: true });
    expect(changedPaths('push')).toEqual([]);
    expect(changedPaths('workflow_dispatch')).toEqual([]);
  });

  it('reads a pull request as its merge commit against the base, both names of a rename included', async () => {
    const root = await mkdtemp(join(tmpdir(), 'ci-changes-'));
    const git = (...args: string[]) => execFileSync('git', args, { cwd: root });
    git('init', '-q', '-b', 'main');
    git('config', 'user.email', 'test@example.com');
    git('config', 'user.name', 'Test');
    await mkdir(join(root, 'crates'), { recursive: true });
    await mkdir(join(root, 'docs'), { recursive: true });
    await writeFile(join(root, 'crates', 'lib.rs'), 'fn main() {}\n');
    git('add', '-A');
    git('commit', '-qm', 'base');
    git('checkout', '-q', '-b', 'topic');
    await rename(join(root, 'crates', 'lib.rs'), join(root, 'docs', 'lib.rs'));
    git('add', '-A');
    git('commit', '-qm', 'move a Rust file into docs');
    git('checkout', '-q', 'main');
    git('merge', '-q', '--no-ff', '-m', 'the merge GitHub checks out', 'topic');
    const paths = changedPaths('pull_request', root);
    expect(paths.sort()).toEqual(['crates/lib.rs', 'docs/lib.rs']);
    expect(windowsJobsFor(paths)).toEqual({ rust: true, package: true });
  });
});
