/** Keep past entries: people who skip an update should see its improvements too. */
export interface ReleaseNotes { version: string; changes: readonly string[] }

export const releaseNotes: readonly ReleaseNotes[] = [
  {
    version: '0.1.0-alpha.20',
    // Shown whole in a dialog that must not scroll at the app's smallest
    // window, 800 by 600 (tests/e2e/updates.spec.ts): short lines, the rest
    // in docs/releases/v0.1.0-alpha.20.md.
    changes: [
      'Intern is now ATLAS: Analyze, Title, Label, Approve, Store.',
      'Names lead with the organization: 2025-03-26 Vistage Malaysia - Termination Letter for Johnny Appleseed.',
      'A file’s own name is kept when it already says the right thing.',
      'A missing date comes from the file name or the file’s last-edited date, with a note.',
      'Review reasons are a few words each. View document (Alt+V) shows the file inside ATLAS.',
      'Double-click a row to open it. Clear queue and Clear completed are in the ⋯ menu.',
    ],
  },
  {
    version: '0.1.0-alpha.19',
    // Shown whole in a dialog that must not scroll at the app's smallest
    // window, 800 by 600 (tests/e2e/updates.spec.ts): short lines, the rest
    // in docs/releases/v0.1.0-alpha.19.md. That holds for these lines alone.
    // Shown after alpha.18's eight to someone who skipped that release, the
    // dialog scrolls there (165 pixels over, measured), as it did for anyone
    // who skipped a release before this one.
    changes: [
      'A spreadsheet date worked out by a TODAY or NOW formula waits for your check.',
      'A click on a button that came under the pointer less than half a second ago does nothing: look, then click again.',
      'What you typed in review and did not file is kept when Intern closes. It can come back beside the document, read-only.',
    ],
  },
  {
    version: '0.1.0-alpha.18',
    // Shown whole in a dialog that must not scroll at the app's smallest
    // window, 800 by 600 (tests/e2e/updates.spec.ts): eight lines of two
    // lines each at most, the rest in docs/releases/v0.1.0-alpha.18.md.
    changes: [
      'Fewer wrong names are filed without your check: more kinds of doubtful name wait for a look.',
      'A name a learned spelling changed waits until you choose “Use on its own” in Settings.',
      'In review, another party the document names is one click from the name.',
      'While Intern is open, what you type in review stays with its document, even if it is read again.',
      'Scans whose own text layer reads badly are read again, so dates and names are found more often.',
      'The quicker model holds about 1.3 GB less memory and reads a little faster.',
      'After an update, Intern no longer reads its model files through again to check them.',
      'At sign-in a checked model waits to load until a document does, and starts as one arrives.',
    ],
  },
  {
    version: '0.1.0-alpha.17',
    // Shown whole in a dialog that must not scroll at the smallest window it
    // is tested in (a scrolling dialog takes a Tab stop of its own): short
    // lines, the rest in docs/releases/v0.1.0-alpha.17.md.
    changes: [
      'The accurate model reads faster on most processors and holds about 2.7 GB less memory.',
      'The first document after Intern starts is read sooner on most computers.',
      'Fewer wrong names are filed without your check, so more documents wait for a quick look.',
      'Court filings, invoices, letters and emails are named right a little more often.',
      'A file dropped into a folder on this computer’s own disk is noticed within seconds.',
      'Review says which model read a document, and its reasons say what to check.',
      'Alt+N skips to the next document to decide; a double-click opens a row’s details.',
      'Long queues respond faster, and the progress bar moves while each document is read.',
    ],
  },
  {
    version: '0.1.0-alpha.16',
    changes: [
      'Status now comes first in Queue and Completed, so you can copy filenames and descriptions together.',
      'Simpler screens and settings keep everyday tasks easier to find. Input and Output folders are now called Queue and Completed.',
      'Files changed while waiting in Queue are checked against the version Intern actually reads. Changes after reading still need another analysis.',
      'File-access problems now report an access error instead of saying the file changed.',
      'API-key setup has been removed. Intern analyzes documents locally.',
      'After an update, this summary shows what has changed.',
    ],
  },
];

/** Compare release versions, including numeric prereleases such as alpha.9 and alpha.10. */
export function compareVersions(left: string, right: string): number {
  const parse = (version: string) => version.replace(/^v/, '').split('+')[0].split(/-(.*)/s);
  const [leftCore, leftPre] = parse(left);
  const [rightCore, rightPre] = parse(right);
  const a = leftCore.split('.').map(Number);
  const b = rightCore.split('.').map(Number);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const difference = (a[i] ?? 0) - (b[i] ?? 0);
    if (difference) return difference;
  }
  if (leftPre === rightPre) return 0;
  if (!leftPre) return 1;
  if (!rightPre) return -1;
  const x = leftPre.split('.');
  const y = rightPre.split('.');
  for (let i = 0; i < Math.max(x.length, y.length); i++) {
    if (x[i] === undefined) return -1;
    if (y[i] === undefined) return 1;
    if (x[i] === y[i]) continue;
    const xn = /^\d+$/.test(x[i]);
    const yn = /^\d+$/.test(y[i]);
    if (xn && yn) return Number(x[i]) - Number(y[i]);
    if (xn !== yn) return xn ? -1 : 1;
    return x[i] < y[i] ? -1 : 1;
  }
  return 0;
}

export function changesSince(previous: string | undefined, current: string, notes: readonly ReleaseNotes[] = releaseNotes): string[] {
  return [...new Set(notes
    .filter((release) => compareVersions(release.version, current) <= 0
      && (previous ? compareVersions(release.version, previous) > 0 : compareVersions(release.version, current) === 0))
    .sort((a, b) => compareVersions(a.version, b.version))
    .flatMap((release) => release.changes))];
}
