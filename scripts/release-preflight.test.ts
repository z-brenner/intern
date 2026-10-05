import { createHash } from 'node:crypto';
import { execFile, execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { describe, expect, it } from 'vitest';
import { releaseInputsDigest } from './hash-release-inputs.mjs';
import { pendingSignoff, preflightProblems, setAsideStaleSignoff } from './release-preflight.mjs';

const exec = promisify(execFile);
const version = '0.1.0-alpha.11';
const screenshot = 'the capture a reviewer looked at';
const sha = (value: string) => createHash('sha256').update(value).digest('hex');

/**
 * A repository shaped like a release commit: the version stated in each place
 * the release reads it, release notes, a reviewed screenshot, and a sign-off
 * accepted for exactly this commit's release inputs. Each test then breaks one
 * thing, the way a real release has broken.
 */
async function releaseCommit(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), 'release-preflight-'));
  const git = (...args: string[]) => execFileSync('git', args, { cwd: root });
  git('init', '-q');
  git('config', 'user.email', 'test@example.com');
  git('config', 'user.name', 'Test');
  await mkdir(join(root, 'src-tauri'), { recursive: true });
  await mkdir(join(root, '.github', 'workflows'), { recursive: true });
  await mkdir(join(root, 'docs', 'releases'), { recursive: true });
  await mkdir(join(root, 'docs', 'qa'), { recursive: true });
  await writeFile(join(root, 'package.json'), `${JSON.stringify({ name: 'intern', version }, null, 2)}\n`);
  await writeFile(join(root, 'Cargo.toml'), `[workspace]\nmembers = ["src-tauri"]\n\n[workspace.package]\nversion = "${version}"\nedition = "2024"\n`);
  await writeFile(join(root, 'src-tauri', 'tauri.conf.json'), `${JSON.stringify({ productName: 'Intern', version }, null, 2)}\n`);
  await writeFile(join(root, '.github', 'workflows', 'release.yml'), `name: Release v${version}\n\non:\n  workflow_dispatch:\n`);
  await writeFile(join(root, 'docs', 'releases', `v${version}.md`), `# Intern v${version}\n\nThe installer is not Authenticode signed.\n`);
  await writeFile(join(root, 'docs', 'qa', 'latest-implementation.png'), screenshot);
  git('add', '-A');
  git('commit', '-qm', 'release commit');
  await writeSignoff(root, { release_inputs_sha256: releaseInputsDigest(root) });
  // docs/qa/ is outside the digest, so committing the sign-off cannot move it.
  git('add', '-A');
  git('commit', '-qm', 'record the sign-off');
  return root;
}

async function writeSignoff(root: string, overrides: Record<string, unknown> = {}) {
  await writeFile(join(root, 'docs', 'qa', 'rendered-fidelity-signoff.json'), `${JSON.stringify({
    schema_version: 1,
    status: 'accepted',
    release_inputs_sha256: 'set by the caller',
    screenshot_path: 'docs/qa/latest-implementation.png',
    screenshot_sha256: sha(screenshot),
    reviewer: 'release reviewer',
    reviewed_at: '2026-10-01T12:00:00Z',
    notes: 'Reviewed the capture; nothing clipped or illegible.',
    ...overrides,
  }, null, 2)}\n`);
}

async function commitChange(root: string, path: string, contents: string) {
  await writeFile(join(root, path), contents);
  execFileSync('git', ['add', '-A'], { cwd: root });
  execFileSync('git', ['commit', '-qm', `change ${path}`], { cwd: root });
}

async function runPreflight(root: string, ...args: string[]) {
  try {
    const { stdout } = await exec(process.execPath, ['scripts/release-preflight.mjs', `--root=${root}`, ...args]);
    return { code: 0, stdout };
  } catch (error) {
    const failed = error as { code: number; stdout: string };
    return { code: failed.code, stdout: failed.stdout };
  }
}

describe('release preflight', () => {
  it('matching_signoff_passes: a consistent release commit passes, in seconds and from the command line', async () => {
    const root = await releaseCommit();

    expect(preflightProblems(root)).toEqual([]);
    const result = await runPreflight(root, `--workflow=Release v${version}`);
    expect(result.code).toBe(0);
    expect(result.stdout).not.toContain('::error::');
  });

  // The failure CI_RELEASE_DOCS-3 describes: the last release's accepted
  // sign-off left in place while application code moved on.
  it('stale_signoff_fails: accepted for other release inputs is refused, naming both digests', async () => {
    const root = await releaseCommit();
    const reviewed = releaseInputsDigest(root);
    await commitChange(root, 'package.json', `${JSON.stringify({ name: 'intern', version, description: 'changed after review' }, null, 2)}\n`);

    const problems = preflightProblems(root);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain(`accepted for release inputs ${reviewed}`);
    expect(problems[0]).toContain(releaseInputsDigest(root));
    const result = await runPreflight(root);
    expect(result.code).toBe(1);
    expect(result.stdout).toMatch(/^::error::The rendered-fidelity sign-off was accepted for release inputs/m);
  });

  it('refuses a pending sign-off, and a reviewed screenshot that is not the one committed', async () => {
    const pending = await releaseCommit();
    await writeSignoff(pending, pendingSignoff('Awaiting review.'));
    expect(preflightProblems(pending)).toEqual([expect.stringContaining('is "pending", not accepted')]);

    const swapped = await releaseCommit();
    await writeFile(join(swapped, 'docs', 'qa', 'latest-implementation.png'), 'a capture nobody reviewed');
    expect(preflightProblems(swapped)).toEqual([expect.stringContaining(`but the sign-off accepted ${sha(screenshot)}`)]);

    const unsigned = await releaseCommit();
    await writeSignoff(unsigned, { release_inputs_sha256: releaseInputsDigest(unsigned), reviewer: ' ' });
    expect(preflightProblems(unsigned)).toEqual([expect.stringContaining('The accepted sign-off is incomplete')]);
  });

  it('missing_notes_fails: without the notes file, or notes that leave out what they must say', async () => {
    const missing = await releaseCommit();
    await rm(join(missing, 'docs', 'releases', `v${version}.md`));
    expect(preflightProblems(missing)).toEqual([`docs/releases/v${version}.md is missing; it is published as the release notes.`]);

    const quiet = await releaseCommit();
    await writeFile(join(quiet, 'docs', 'releases', `v${version}.md`), `# Intern v${version}\n\nSigned and sealed.\n`);
    expect(preflightProblems(quiet)).toEqual([expect.stringContaining('not Authenticode signed')]);

    const untitled = await releaseCommit();
    await writeFile(join(untitled, 'docs', 'releases', `v${version}.md`), 'The installer is not Authenticode signed.\n');
    expect(preflightProblems(untitled)).toEqual([expect.stringContaining(`"# Intern v${version}"`)]);
  });

  it('version_mismatch_fails: one file left behind by the bump, or a workflow named for another release', async () => {
    const cargo = await releaseCommit();
    await writeFile(join(cargo, 'Cargo.toml'), '[workspace]\nmembers = ["src-tauri"]\n\n[workspace.package]\nversion = "0.1.0-alpha.10"\n');
    expect(preflightProblems(cargo)).toEqual([
      `The release version is not stated the same everywhere: package.json ${version}, Cargo.toml 0.1.0-alpha.10, src-tauri/tauri.conf.json ${version}, workflow ${version}.`,
    ]);

    const tauri = await releaseCommit();
    await writeFile(join(tauri, 'src-tauri', 'tauri.conf.json'), JSON.stringify({ version: '0.1.0-alpha.10' }));
    expect(preflightProblems(tauri)).toEqual([expect.stringContaining('src-tauri/tauri.conf.json 0.1.0-alpha.10')]);

    // The name the run actually carries wins over the file's.
    const workflow = await releaseCommit();
    expect(preflightProblems(workflow, { workflow: 'Release v0.1.0-alpha.10' })).toEqual([expect.stringContaining('workflow 0.1.0-alpha.10')]);
    const result = await runPreflight(workflow, '--workflow=Release v0.1.0-alpha.10');
    expect(result.code).toBe(1);
  });

  it('names every problem at once, so one commit can fix them all', async () => {
    const root = await releaseCommit();
    await rm(join(root, 'docs', 'releases', `v${version}.md`));
    await writeFile(join(root, 'package.json'), JSON.stringify({ version: '0.1.0-alpha.12' }));
    await rm(join(root, 'docs', 'qa', 'latest-implementation.png'));

    const problems = preflightProblems(root);
    expect(problems).toHaveLength(3);
    expect(problems.join('\n')).toMatch(/not stated the same everywhere[\s\S]*v0\.1\.0-alpha\.12\.md is missing[\s\S]*latest-implementation\.png is missing/);
  });
});

describe('setting aside a stale sign-off for a QA run', () => {
  it('turns an accepted sign-off for other release inputs into the pending record, and says so in the summary', async () => {
    const root = await releaseCommit();
    const reviewed = releaseInputsDigest(root);
    await commitChange(root, 'package.json', `${JSON.stringify({ name: 'intern', version, description: 'next release' }, null, 2)}\n`);
    const summary = join(root, 'summary.md');

    const result = setAsideStaleSignoff(root, { summaryPath: summary });

    expect(result).toMatchObject({ setAside: true, reviewedFor: reviewed, digest: releaseInputsDigest(root) });
    const written = JSON.parse(await readFile(join(root, 'docs', 'qa', 'rendered-fidelity-signoff.json'), 'utf8'));
    expect(written).toEqual(pendingSignoff(written.notes));
    expect(written.notes).toContain(reviewed);
    expect(await readFile(summary, 'utf8')).toContain('Rendered-fidelity sign-off set aside');
    // Still refused for release, which is the point of keeping it pending.
    expect(preflightProblems(root)).toEqual([expect.stringContaining('is "pending", not accepted')]);
  });

  it('leaves a sign-off for this exact commit, and a pending one, as they are', async () => {
    const current = await releaseCommit();
    const before = await readFile(join(current, 'docs', 'qa', 'rendered-fidelity-signoff.json'), 'utf8');
    expect(setAsideStaleSignoff(current)).toMatchObject({ setAside: false, status: 'accepted' });
    expect(await readFile(join(current, 'docs', 'qa', 'rendered-fidelity-signoff.json'), 'utf8')).toBe(before);

    const pending = await releaseCommit();
    await writeSignoff(pending, pendingSignoff('Awaiting review.'));
    const pendingBefore = await readFile(join(pending, 'docs', 'qa', 'rendered-fidelity-signoff.json'), 'utf8');
    expect(setAsideStaleSignoff(pending)).toMatchObject({ setAside: false, status: 'pending' });
    expect(await readFile(join(pending, 'docs', 'qa', 'rendered-fidelity-signoff.json'), 'utf8')).toBe(pendingBefore);
  });

  it('runs from the command line the way qa.yml calls it', async () => {
    const root = await releaseCommit();
    await commitChange(root, 'package.json', `${JSON.stringify({ name: 'intern', version, description: 'next release' }, null, 2)}\n`);

    const result = await runPreflight(root, '--set-aside-stale-signoff');

    expect(result.code).toBe(0);
    expect(result.stdout).toMatch(/^::notice::/);
    expect(JSON.parse(await readFile(join(root, 'docs', 'qa', 'rendered-fidelity-signoff.json'), 'utf8')).status).toBe('pending');
  });
});
