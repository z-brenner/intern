import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { appendFileSync, existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { releaseInputsDigest } from './hash-release-inputs.mjs';
import { SCREENSHOT_PATH, SIGNOFF_PATH } from './release-preflight.mjs';
import { QA_WORKFLOW } from './release-evidence-lib.mjs';

/**
 * Whether the QA run a release names is one it may ship on.
 *
 * The release workflow used to run every gate again and score the whole
 * corpus with the real model again (about forty minutes of the alpha.19
 * release's ninety-four) on release inputs the preflight had already required
 * to be byte for byte the ones QA ran on and the reviewer accepted. It now
 * names one "Whole-product QA evidence" run and ships on that run's evidence,
 * and this is what makes that safe. The run must be:
 *
 * - a successful, completed, dispatched run of .github/workflows/qa.yml on
 *   main in this repository. qa.yml is itself a release input, so the run
 *   took every step this commit's qa.yml has, unconditionally: the gates, the
 *   Windows browser tests, the workspace tests, the native and PP-OCR tests,
 *   the installer smoke, the update over alpha.14, and the scoring.
 * - on a commit that is this commit or an ancestor of it, with this commit's
 *   release-inputs digest (computed from git here, not read from the
 *   evidence): what QA tested is what this commit builds.
 * - backed by its evidence manifest, which validates against that run, with
 *   an accepted model evaluation scored on these release inputs, an accepted
 *   installed-core smoke, and the update smoke's log bound in.
 * - the run whose capture the committed sign-off accepted: the capture in its
 *   evidence is the committed one, byte for byte.
 *
 * Anything else is refused with every reason at once.
 */

const QA_PATH = '.github/workflows/qa.yml';
const EVIDENCE = 'docs/qa/release-evidence-manifest.json';
const MODEL_EVALUATION = 'docs/qa/model-evaluation.json';
const UPDATE_SMOKE_LOG = 'docs/qa/logs/update-smoke.log';

const sha256 = (path) => createHash('sha256').update(readFileSync(path)).digest('hex');
const readJson = (path) => JSON.parse(readFileSync(path, 'utf8'));
const tryReadJson = (path) => {
  try {
    return readJson(path);
  } catch {
    return undefined;
  }
};

/** Every reason this QA run cannot be shipped on, or none. */
export function qaRunProblems({ root = '.', run, artifact, repository, attempt }) {
  const repositoryRoot = resolve(root);
  const evidenceRoot = resolve(artifact);
  const problems = [];
  const id = String(run?.id ?? '');

  if (run?.name !== QA_WORKFLOW || run?.path !== QA_PATH) problems.push(`Run ${id} is "${run?.name}" (${run?.path}), not a "${QA_WORKFLOW}" run of ${QA_PATH}.`);
  if (run?.event !== 'workflow_dispatch' || run?.head_branch !== 'main') problems.push(`Run ${id} was started by ${run?.event} on ${run?.head_branch}; a release ships only on a QA run dispatched on main.`);
  if (run?.status !== 'completed' || run?.conclusion !== 'success') problems.push(`Run ${id} is ${run?.status}/${run?.conclusion}, not completed/success.`);
  if (repository && (run?.repository?.full_name !== repository || run?.head_repository?.full_name !== repository)) problems.push(`Run ${id} belongs to ${run?.repository?.full_name} (from ${run?.head_repository?.full_name}), not ${repository}.`);
  if (attempt !== undefined && String(run?.run_attempt) !== String(attempt)) problems.push(`Run ${id} is at attempt ${run?.run_attempt}, not attempt ${attempt}, which this release checked: it was run again in between.`);
  const commit = run?.head_sha;
  if (!/^[a-f0-9]{40}$/.test(commit ?? '')) {
    problems.push(`Run ${id} names no commit.`);
    return problems;
  }

  const digest = releaseInputsDigest(repositoryRoot);
  let qaDigest;
  try {
    execFileSync('git', ['merge-base', '--is-ancestor', commit, 'HEAD'], { cwd: repositoryRoot, stdio: 'ignore' });
    qaDigest = releaseInputsDigest(repositoryRoot, commit);
  } catch {
    problems.push(`Run ${id} ran on ${commit}, which is not this commit or one of its ancestors.`);
  }
  if (qaDigest && qaDigest !== digest) {
    problems.push(`Run ${id} ran on ${commit}, whose release inputs are ${qaDigest}; this commit's are ${digest}. Something outside docs/qa/ changed after that QA run: run QA on this commit and have its capture reviewed.`);
  }

  const manifestPath = join(evidenceRoot, EVIDENCE);
  if (!existsSync(manifestPath)) {
    problems.push(`Run ${id}'s evidence has no ${EVIDENCE}.`);
    return problems;
  }
  try {
    execFileSync(process.execPath, [
      join(dirname(fileURLToPath(import.meta.url)), 'validate-release-evidence.mjs'),
      manifestPath,
      '--allow-pending',
      `--root=${evidenceRoot}`,
      `--commit=${commit}`,
      `--workflow=${QA_WORKFLOW}`,
      `--run-id=${id}`,
      `--run-attempt=${run.run_attempt}`,
    ], { stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (error) {
    const reason = String(error.stderr ?? '').split(/\r?\n/).find((line) => line.startsWith('Error: ')) ?? error.message;
    problems.push(`Run ${id}'s evidence manifest was not accepted: ${reason}`);
  }
  const manifest = tryReadJson(manifestPath);
  if (!manifest?.artifacts?.logs?.some((log) => log.path === UPDATE_SMOKE_LOG)) {
    problems.push(`Run ${id}'s evidence has no ${UPDATE_SMOKE_LOG}: it did not smoke an update over v0.1.0-alpha.14.`);
  }
  const model = tryReadJson(join(evidenceRoot, MODEL_EVALUATION));
  if (model?.release_inputs_sha256 !== digest) {
    problems.push(`Run ${id} scored the corpus on release inputs ${model?.release_inputs_sha256}, not this commit's ${digest}.`);
  }

  // The reviewer accepted one run's capture; the release ships on that run.
  const signoff = tryReadJson(join(repositoryRoot, SIGNOFF_PATH));
  const capture = join(evidenceRoot, SCREENSHOT_PATH);
  const captureDigest = existsSync(capture) ? sha256(capture) : undefined;
  if (!captureDigest || captureDigest !== signoff?.screenshot_sha256) {
    problems.push(`Run ${id}'s capture has SHA-256 ${captureDigest}, but the committed sign-off accepted ${signoff?.screenshot_sha256}. Name the QA run whose capture was reviewed.`);
  }
  return problems;
}

const invokedPath = process.argv[1] && resolve(process.argv[1]);
if (invokedPath === fileURLToPath(import.meta.url)) {
  const argument = (name) => process.argv.find((value) => value.startsWith(`--${name}=`))?.slice(name.length + 3);
  const run = readJson(argument('run'));
  const problems = qaRunProblems({
    root: argument('root') ?? '.',
    run,
    artifact: argument('artifact'),
    repository: argument('repository'),
    attempt: argument('expect-attempt'),
  });
  for (const problem of problems) process.stdout.write(`::error::${problem}\n`);
  if (problems.length) process.exit(1);
  process.stdout.write(`This release ships on "${QA_WORKFLOW}" run ${run.id} attempt ${run.run_attempt} at ${run.head_sha}: same release inputs, evidence accepted, capture the one the sign-off accepted.\n`);
  if (process.env.GITHUB_OUTPUT) {
    appendFileSync(process.env.GITHUB_OUTPUT, `commit=${run.head_sha}\nattempt=${run.run_attempt}\n`);
  }
}
