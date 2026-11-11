/** Keep past entries: people who skip an update should see its improvements too. */
export interface ReleaseNotes { version: string; changes: readonly string[] }

export const releaseNotes: readonly ReleaseNotes[] = [
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
