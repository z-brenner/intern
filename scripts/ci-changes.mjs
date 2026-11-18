import { execFileSync } from 'node:child_process';
import { appendFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Which of CI's Windows jobs a pull request's changes need.
 *
 * The Windows jobs are the slow ones (about 17 minutes each, against 2 to 8
 * for the Ubuntu jobs), and a pull request that touches only documentation
 * used to hold a Windows runner for most of an hour: the rendered-fidelity
 * sign-off commit of every release, which changes nothing outside docs/qa/,
 * among them. This names what each kind of change can affect, and errs toward
 * running: a path it does not recognise needs everything.
 *
 * - Documentation: nothing built or tested on Windows reads it. The Ubuntu
 *   frontend job's Vitest run reads some of it (the README, docs/releasing.md,
 *   docs/qa/) and runs on every change.
 * - The React app and its browser tests: built into the installer, so the
 *   Windows package job still runs, but no Rust code reads them (the Rust
 *   tests read crates/, src-tauri/, fixtures/, bench/ and scripts/).
 *
 * Only pull requests are narrowed. A push to main, and a run started by hand,
 * run every job, so main's runs keep the caches that pull requests and QA
 * read filled.
 */

const DOCUMENTATION = [/^docs\//, /^site\//, /^[^/]+\.md$/, /^LICENSE$/];
const FRONTEND = [
  /^src\//,
  /^tests\/e2e\//,
  /^index\.html$/,
  /^(?:playwright|vite)\.config\.ts$/,
  /^vitest\.setup\.ts$/,
  /^tsconfig(?:\.node)?\.json$/,
];

const matchesAny = (patterns) => (path) => patterns.some((pattern) => pattern.test(path));
const isDocumentation = matchesAny(DOCUMENTATION);
const isFrontend = matchesAny(FRONTEND);

/** The Windows jobs these changed paths need. No paths at all needs everything. */
export function windowsJobsFor(paths) {
  const changed = paths.filter(Boolean);
  if (changed.length === 0) return { rust: true, package: true };
  return {
    rust: changed.some((path) => !isDocumentation(path) && !isFrontend(path)),
    package: changed.some((path) => !isDocumentation(path)),
  };
}

/**
 * A pull request's changes: the merge commit GitHub checks out against its
 * first parent, the base branch as it is now. Anything else runs everything.
 * `--no-renames` lists a renamed file under both names, so moving a Rust file
 * into docs/ still counts as a Rust change.
 */
export function changedPaths(event, root = '.') {
  if (event !== 'pull_request') return [];
  return execFileSync('git', ['diff', '--name-only', '--no-renames', '-z', 'HEAD^1', 'HEAD'], { cwd: resolve(root) })
    .toString('utf8')
    .split('\0')
    .filter(Boolean);
}

const invokedPath = process.argv[1] && resolve(process.argv[1]);
if (invokedPath === fileURLToPath(import.meta.url)) {
  const event = process.argv.find((argument) => argument.startsWith('--event='))?.slice('--event='.length) ?? '';
  const paths = changedPaths(event);
  const jobs = windowsJobsFor(paths);
  const lines = Object.entries(jobs).map(([job, needed]) => `${job}=${needed}`);
  process.stdout.write(event === 'pull_request'
    ? `${paths.length} changed path(s):\n${paths.map((path) => `  ${path}`).join('\n')}\n`
    : `A ${event || 'local'} run: every Windows job runs.\n`);
  process.stdout.write(`${lines.join('\n')}\n`);
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `${lines.join('\n')}\n`);
}
