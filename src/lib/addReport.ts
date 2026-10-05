import type { AddReport } from '../types';

/**
 * Why a file was left out of an add, short enough to sit in parentheses after
 * its name. The add used to refuse every file for the sake of one; now it
 * queues the rest and names the one, so the reason has to be readable at a
 * glance. A code with no entry is shown as it is, like an unmapped reason.
 */
const SKIP_REASONS: Record<string, string> = {
  UNSUPPORTED_FORMAT: 'not a supported format',
  EMPTY_FILE: 'the file is empty',
  TEMPORARY_FILE: 'a temporary or hidden file',
  FILE_MISSING: 'not found',
  FOLDER_UNAVAILABLE: 'the folder could not be read',
  PATH_UNTRUSTED: 'a shortcut or link, which Intern does not follow',
  SOURCE_LOCKED: 'another program has it open',
  IO_ERROR: 'it could not be read',
  FILE_CHANGED: 'it changed while it was being added',
  UPLOADER_OTHER: 'someone else uploaded it',
  UPLOADER_UNVERIFIED: 'its uploader could not be confirmed',
};

/** At most this many skipped files are named; the rest are counted. */
const NAMED_LIMIT = 5;

/**
 * One message for an add: what was queued, what was there already, and each
 * file left out with its reason - "Added 24 documents. Skipped 1: notes.zip
 * (not a supported format)."
 */
export function describeAddReport(report: AddReport): string {
  const parts: string[] = [];
  if (report.added > 0) parts.push(`Added ${report.added} ${report.added === 1 ? 'document' : 'documents'}.`);
  if (report.alreadyQueued > 0) {
    parts.push(`${report.alreadyQueued} ${report.alreadyQueued === 1 ? 'was' : 'were'} already in the queue.`);
  }
  if (report.skipped.length > 0) {
    const named = report.skipped.slice(0, NAMED_LIMIT).map(({ name, code }) => `${name} (${SKIP_REASONS[code] ?? code})`);
    const more = report.skipped.length - named.length;
    parts.push(`Skipped ${report.skipped.length}: ${named.join(', ')}${more > 0 ? `, and ${more} more` : ''}.`);
  }
  return parts.length > 0 ? parts.join(' ') : 'No documents were found to add.';
}
