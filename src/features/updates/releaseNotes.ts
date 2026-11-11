/** Keep past entries: people who skip an update should see its improvements too. */
export interface ReleaseNotes { version: string; changes: readonly string[] }

export const releaseNotes: readonly ReleaseNotes[] = [
  {
    version: '0.1.0-alpha.17',
    changes: [
      'The accurate model needs about 2.6 GB less memory and reads documents faster on most processors.',
      'The first document after Intern starts is read sooner: the model’s instructions are kept between starts.',
      'Fewer wrong names are filed without your check: a name is filed on its own only when the model is surer of it, so more documents wait for a quick look.',
      'Court filings, invoices, letters and emails are named right a little more often.',
      'Evidence in review now says when it is the document’s own lines, and which model read the document.',
      'The “model unsure” reason says what to check, and no longer contradicts the confidence shown beside it.',
      'Analysis details stay open or closed as you left them while you move through review.',
      'Alt+N skips to the next document to decide, a double-click on a row opens its details, and Escape closes the More menu.',
      'Long queues respond faster while Intern is reading and while you type in the filter.',
      'Once Intern has read a couple of documents, the progress bar moves while each one is read, and the time left is shown from the start of every run.',
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
