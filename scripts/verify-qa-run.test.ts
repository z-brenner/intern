import { createHash } from 'node:crypto';
import { execFile, execFileSync } from 'node:child_process';
import { appendFile, mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { describe, expect, it } from 'vitest';
import { releaseInputsDigest } from './hash-release-inputs.mjs';
import { qaRunProblems } from './verify-qa-run.mjs';

const exec = promisify(execFile);
const sha = (value: string) => createHash('sha256').update(value).digest('hex');
const capture = 'the capture a reviewer looked at';
const repository = 'z-brenner/intern';

/**
 * A release commit and the QA run before it, as the release sequence leaves
 * them: QA ran on one commit, a reviewer accepted its capture, and the
 * sign-off commit after it changed nothing outside docs/qa/. The QA run's
 * evidence is written by the real create-release-evidence.mjs.
 */
async function releaseAfterQa() {
  const root = await mkdtemp(join(tmpdir(), 'verify-qa-run-'));
  const git = (...args: string[]) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
  git('init', '-q', '-b', 'main');
  git('config', 'user.email', 'test@example.com');
  git('config', 'user.name', 'Test');
  await mkdir(join(root, 'docs', 'qa'), { recursive: true });
  await writeFile(join(root, 'source.rs'), 'fn main() {}\n');
  await writeFile(join(root, 'docs', 'qa', 'rendered-fidelity-signoff.json'), '{"status":"pending"}\n');
  git('add', '-A');
  git('commit', '-qm', 'the commit QA runs on');
  const qaCommit = git('rev-parse', 'HEAD');
  const digest = releaseInputsDigest(root);

  const runId = '4242';
  const attempt = '2';
  const artifact = await mkdtemp(join(tmpdir(), 'verify-qa-run-artifact-'));
  await mkdir(join(artifact, 'docs', 'qa', 'logs'), { recursive: true });
  const write = (path: string, contents: string) => writeFile(join(artifact, path), contents);
  await write('docs/qa/model-evaluation.json', JSON.stringify({
    schema_version: 3, pipeline: 'evidence', status: 'completed', commit: qaCommit,
    release_inputs_sha256: digest, runner: { ci_run_id: runId, ci_run_attempt: attempt },
    acceptance: { status: 'accepted', failures: [] },
  }));
  await write('docs/qa/latest-implementation.png', capture);
  await write('docs/qa/release-checklist.md', '# QA run checklist\n');
  await write('docs/qa/rendered-fidelity-signoff.json', JSON.stringify({ schema_version: 1, status: 'pending' }));
  await write('docs/qa/Intern-qa-installer.exe', 'installer bytes');
  await write('docs/qa/installed-core-smoke.json', JSON.stringify({
    schema_version: 1, status: 'accepted', commit: qaCommit, workflow: 'Whole-product QA evidence',
    run_id: runId, run_attempt: attempt, installer_sha256: sha('installer bytes'),
    checks: {
      app_launched: true, clean_shutdown: true, runtime_inventory_verified: true,
      installed_worker_core_path: true, uninstall_succeeded: true, user_data_retained: true,
    },
  }));
  for (const log of ['cargo-test', 'installer-smoke', 'update-smoke']) await write(`docs/qa/logs/${log}.log`, `${log} passed\n`);

  // The reviewer's sign-off, committed after QA, touching only docs/qa/.
  await writeFile(join(root, 'docs', 'qa', 'latest-implementation.png'), capture);
  await writeFile(join(root, 'docs', 'qa', 'rendered-fidelity-signoff.json'), JSON.stringify({
    schema_version: 1, status: 'accepted', release_inputs_sha256: digest,
    screenshot_path: 'docs/qa/latest-implementation.png', screenshot_sha256: sha(capture),
    reviewer: 'release reviewer', reviewed_at: '2026-11-17T12:00:00Z', notes: 'Reviewed.',
  }));
  git('add', '-A');
  git('commit', '-qm', 'record the sign-off');

  const run = {
    id: Number(runId), name: 'Whole-product QA evidence', path: '.github/workflows/qa.yml',
    event: 'workflow_dispatch', head_branch: 'main', status: 'completed', conclusion: 'success',
    head_sha: qaCommit, run_attempt: Number(attempt),
    repository: { full_name: repository }, head_repository: { full_name: repository },
  };
  const evidence = async (logs = ['cargo-test', 'installer-smoke', 'update-smoke']) => exec(process.execPath, [
    'scripts/create-release-evidence.mjs', `--root=${artifact}`, `--output=${join(artifact, 'docs/qa/release-evidence-manifest.json')}`,
    `--commit=${qaCommit}`, '--workflow=Whole-product QA evidence', `--run-id=${runId}`, `--run-attempt=${attempt}`,
    '--model-evaluation=docs/qa/model-evaluation.json', '--screenshot=docs/qa/latest-implementation.png',
    '--checklist=docs/qa/release-checklist.md', '--fidelity-signoff=docs/qa/rendered-fidelity-signoff.json',
    '--installed-core-smoke=docs/qa/installed-core-smoke.json', '--installer=docs/qa/Intern-qa-installer.exe',
    ...logs.map((log) => `--log=docs/qa/logs/${log}.log`),
  ]);
  await evidence();
  return { root, artifact, run, git, evidence, check: (overrides = {}) => qaRunProblems({ root, run, artifact, repository, attempt, ...overrides }) };
}

describe('the QA run a release ships on', () => {
  it('is accepted when it is a successful QA run on main with these release inputs and the reviewed capture', async () => {
    const release = await releaseAfterQa();
    expect(release.check()).toEqual([]);
  });

  it('must be a successful, dispatched QA run on main in this repository', async () => {
    const release = await releaseAfterQa();
    const variants: [Record<string, unknown>, string][] = [
      [{ conclusion: 'failure' }, 'not completed/success'],
      [{ status: 'in_progress', conclusion: null }, 'not completed/success'],
      [{ name: 'CI', path: '.github/workflows/ci.yml' }, 'not a "Whole-product QA evidence" run'],
      [{ event: 'pull_request' }, 'dispatched on main'],
      [{ head_branch: 'feature' }, 'dispatched on main'],
      [{ head_repository: { full_name: 'someone/fork' } }, 'not z-brenner/intern'],
    ];
    for (const [change, reason] of variants) {
      expect(release.check({ run: { ...release.run, ...change } }), reason).toEqual(expect.arrayContaining([expect.stringContaining(reason)]));
    }
    // Run again after the release checked it: the evidence may differ.
    expect(release.check({ attempt: '1' })).toEqual([expect.stringContaining('run again in between')]);
  });

  it('refuses a QA run on other release inputs, or on a commit this one does not descend from', async () => {
    const release = await releaseAfterQa();
    await appendFile(join(release.root, 'source.rs'), '// changed after QA\n');
    release.git('add', '-A');
    release.git('commit', '-qm', 'a change outside docs/qa after QA');
    const problems = release.check();
    expect(problems).toEqual(expect.arrayContaining([
      expect.stringContaining('Something outside docs/qa/ changed after that QA run'),
      expect.stringContaining('scored the corpus on release inputs'),
    ]));

    const other = await releaseAfterQa();
    expect(other.check({ run: { ...other.run, head_sha: 'f'.repeat(40) } }))
      .toEqual(expect.arrayContaining([expect.stringContaining('not this commit or one of its ancestors')]));
  });

  it('refuses evidence that does not validate, or that has no update over alpha.14', async () => {
    const release = await releaseAfterQa();
    await writeFile(join(release.artifact, 'docs/qa/Intern-qa-installer.exe'), 'tampered installer');
    expect(release.check()).toEqual([expect.stringContaining('evidence manifest was not accepted: Error: installer hash or size does not match its artifact')]);

    const withoutUpdate = await releaseAfterQa();
    await withoutUpdate.evidence(['cargo-test', 'installer-smoke']);
    expect(withoutUpdate.check()).toEqual([expect.stringContaining('did not smoke an update over v0.1.0-alpha.14')]);
  });

  it('must be the run whose capture the sign-off accepted', async () => {
    const release = await releaseAfterQa();
    await writeFile(join(release.artifact, 'docs/qa/latest-implementation.png'), 'another run\'s capture');
    await release.evidence();
    expect(release.check()).toEqual([expect.stringContaining('Name the QA run whose capture was reviewed')]);
  });
});
