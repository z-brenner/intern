import { createHash } from 'node:crypto';
import { appendFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { releaseInputsDigest } from './hash-release-inputs.mjs';
import { acceptedFidelity } from './release-evidence-lib.mjs';

/**
 * Everything the release workflow's last step requires of a commit that can
 * be known before anything is built, checked in seconds instead.
 *
 * The final evidence validation in release.yml runs after about ninety minutes
 * of Windows build and real inference, and until now it was the first place a
 * stale rendered-fidelity sign-off, a missing release-notes file, or a version
 * that one file forgot to bump was noticed. None of those depend on the build.
 * This reads the committed tree directly and names every problem at once, so a
 * maintainer fixes them in one commit rather than one per two-hour run.
 *
 * With `--set-aside-stale-signoff` it is the QA workflow's counterpart: an
 * accepted sign-off reviewed against other release inputs is replaced, on that
 * runner only, by the pending record it stands for until someone reviews the
 * new capture. See docs/releasing.md.
 */

export const SIGNOFF_PATH = 'docs/qa/rendered-fidelity-signoff.json';
export const SCREENSHOT_PATH = 'docs/qa/latest-implementation.png';

/** The record a sign-off returns to between reviews, field for field as the alpha.10 prep commit wrote it. */
export function pendingSignoff(notes) {
  return {
    schema_version: 1,
    status: 'pending',
    release_inputs_sha256: 'pending',
    screenshot_path: SCREENSHOT_PATH,
    screenshot_sha256: 'pending',
    reviewer: 'pending',
    reviewed_at: 'pending',
    notes,
  };
}

function readJson(root, path) {
  return JSON.parse(readFileSync(join(root, path), 'utf8'));
}

function sha256(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

/** `[workspace.package] version`, which every crate inherits. */
function cargoWorkspaceVersion(root) {
  const manifest = readFileSync(join(root, 'Cargo.toml'), 'utf8');
  const section = /^\[workspace\.package\]\s*$([\s\S]*?)(?=^\[|(?![\s\S]))/m.exec(manifest)?.[1] ?? '';
  return /^version\s*=\s*"([^"]+)"/m.exec(section)?.[1];
}

/** The workflow's own name, `Release v<version>`, as release.yml declares it. */
function releaseWorkflowName(root) {
  const workflow = readFileSync(join(root, '.github/workflows/release.yml'), 'utf8');
  return /^name:\s*(.+?)\s*$/m.exec(workflow)?.[1]?.replace(/^(['"])(.*)\1$/, '$2');
}

const EVIDENCE_VALIDATOR_PATH = 'scripts/validate-release-evidence.mjs';

/**
 * The one workflow name the final evidence validation accepts. It states the
 * version as a literal in a pattern, `/^Release v(1\.2\.3)$/`, and only a
 * release run ever reaches that check, so a bump that missed it used to fail
 * after the whole Windows build and the corpus scoring.
 */
function evidenceValidatorVersion(root) {
  const path = join(root, EVIDENCE_VALIDATOR_PATH);
  if (!existsSync(path)) return undefined;
  return /\^Release v\((.+?)\)\$/.exec(readFileSync(path, 'utf8'))?.[1]?.replaceAll('\\.', '.');
}

/** The release version as each place that states it states it. */
export function statedVersions(root, workflow = releaseWorkflowName(root)) {
  return {
    'package.json': readJson(root, 'package.json').version,
    'Cargo.toml': cargoWorkspaceVersion(root),
    'src-tauri/tauri.conf.json': readJson(root, 'src-tauri/tauri.conf.json').version,
    workflow: /^Release v(.+)$/.exec(workflow ?? '')?.[1],
    [EVIDENCE_VALIDATOR_PATH]: evidenceValidatorVersion(root),
  };
}

function signoffProblems(root, digest) {
  let signoff;
  try {
    signoff = readJson(root, SIGNOFF_PATH);
  } catch {
    return [`${SIGNOFF_PATH} could not be read as JSON.`];
  }
  if (signoff?.status !== 'accepted') {
    return [`The rendered-fidelity sign-off is "${signoff?.status}", not accepted. Run the QA workflow on this commit and have a reviewer accept its capture (docs/releasing.md).`];
  }
  const problems = [];
  if (signoff.release_inputs_sha256 !== digest) {
    problems.push(`The rendered-fidelity sign-off was accepted for release inputs ${signoff.release_inputs_sha256}, but this commit's are ${digest}: something outside docs/qa/ changed after the review. Run the QA workflow on this commit and have its capture reviewed again.`);
  }
  const screenshot = join(root, SCREENSHOT_PATH);
  const screenshotDigest = existsSync(screenshot) ? sha256(screenshot) : undefined;
  if (!screenshotDigest) {
    problems.push(`${SCREENSHOT_PATH} is missing; the release ships the capture a reviewer accepted.`);
  } else if (signoff.screenshot_sha256 !== screenshotDigest) {
    problems.push(`${SCREENSHOT_PATH} has SHA-256 ${screenshotDigest}, but the sign-off accepted ${signoff.screenshot_sha256}. Commit the capture that was reviewed, or have this one reviewed.`);
  }
  // The final validation's own test, so nothing this passes can still be
  // refused there for a field the checks above do not name.
  if (!problems.length && !acceptedFidelity(signoff, { sha256: screenshotDigest }, { release_inputs_sha256: digest })) {
    problems.push('The accepted sign-off is incomplete: it needs schema_version 1, a screenshot_path, a reviewer, a reviewed_at date, and notes.');
  }
  return problems;
}

function notesProblems(root, version) {
  const path = `docs/releases/v${version}.md`;
  if (!existsSync(join(root, path))) return [`${path} is missing; it is published as the release notes.`];
  const notes = readFileSync(join(root, path), 'utf8');
  const problems = [];
  if (!notes.includes(`# Intern v${version}`)) problems.push(`${path} does not start its notes with "# Intern v${version}".`);
  if (!notes.includes('not Authenticode signed')) problems.push(`${path} does not say the installer is "not Authenticode signed", which every downloader needs to know before SmartScreen tells them.`);
  return problems;
}

/** Every reason this commit cannot be released, or none. */
export function preflightProblems(root = '.', { workflow } = {}) {
  const repository = resolve(root);
  const problems = [];
  const versions = statedVersions(repository, workflow);
  const distinct = new Set(Object.values(versions));
  if (distinct.size !== 1 || distinct.has(undefined)) {
    problems.push(`The release version is not stated the same everywhere: ${Object.entries(versions).map(([where, version]) => `${where} ${version ?? '(none)'}`).join(', ')}.`);
  }
  const version = versions['package.json'];
  if (version) problems.push(...notesProblems(repository, version));
  problems.push(...signoffProblems(repository, releaseInputsDigest(repository)));
  return problems;
}

/**
 * Replace an accepted sign-off whose release inputs are not this commit's with
 * the pending record, and say so. Anything else - pending, rejected, or
 * accepted for exactly this commit - is left as it is.
 */
export function setAsideStaleSignoff(root = '.', { summaryPath } = {}) {
  const repository = resolve(root);
  const digest = releaseInputsDigest(repository);
  const signoff = readJson(repository, SIGNOFF_PATH);
  if (signoff?.status !== 'accepted' || signoff.release_inputs_sha256 === digest) return { setAside: false, digest, status: signoff?.status };
  const reviewedFor = signoff.release_inputs_sha256;
  const notes = `Set aside by this QA run: the committed sign-off was accepted for release inputs ${reviewedFor}, and this commit's are ${digest}. It stays pending until a reviewer accepts this run's capture.`;
  writeFileSync(join(repository, SIGNOFF_PATH), `${JSON.stringify(pendingSignoff(notes), null, 2)}\n`);
  if (summaryPath) {
    appendFileSync(summaryPath, [
      '### Rendered-fidelity sign-off set aside',
      '',
      `The committed sign-off was accepted for release inputs \`${reviewedFor}\`, but this commit's are \`${digest}\`, so this run treats it as pending.`,
      'The committed file is unchanged. Review this run\'s capture and commit an accepted sign-off for this commit before releasing (docs/releasing.md).',
      '',
    ].join('\n'));
  }
  return { setAside: true, digest, reviewedFor };
}

const invokedPath = process.argv[1] && resolve(process.argv[1]);
if (invokedPath === fileURLToPath(import.meta.url)) {
  const argument = (name) => process.argv.find((value) => value.startsWith(`--${name}=`))?.slice(name.length + 3);
  const root = argument('root') ?? '.';
  if (process.argv.includes('--set-aside-stale-signoff')) {
    const result = setAsideStaleSignoff(root, { summaryPath: process.env.GITHUB_STEP_SUMMARY });
    process.stdout.write(result.setAside
      ? `::notice::The rendered-fidelity sign-off was accepted for release inputs ${result.reviewedFor}, not this commit's ${result.digest}; it is pending for this run.\n`
      : `The rendered-fidelity sign-off is left as committed (${result.status}); this commit's release inputs are ${result.digest}.\n`);
  } else {
    const problems = preflightProblems(root, { workflow: argument('workflow') });
    for (const problem of problems) process.stdout.write(`::error::${problem}\n`);
    if (problems.length) process.exit(1);
    process.stdout.write('Release preflight passed: the sign-off, the screenshot, the release notes, and every version agree with this commit.\n');
  }
}
