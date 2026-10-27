import { folderName } from '../features/intake/folderNames';
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
  IN_OUTPUT_FOLDER: 'it is already in your output folder',
  COPY_FAILED: 'it could not be copied into your input folder',
  CLAIMED_ELSEWHERE: 'another computer is already working on it',
  INPUT_FOLDER_NOT_SET: 'no input folder is set up yet; choose one in Settings',
  INPUT_FOLDER_UNAVAILABLE: 'your input folder could not be opened',
  SYNC_CONFLICT_COPY: 'a OneDrive conflict copy; resolve the conflict first',
  CLAIM_IO_FAILED: 'Intern could not note in the shared folder that it is working on it',
  INTAKE_COORDINATION_UNAVAILABLE: 'the input folder’s shared bookkeeping could not be written',
  NOT_IN_INTAKE_FOLDER: 'it is not in the input folder Intern is watching',
};

/** At most this many skipped files are named; the rest are counted. */
const NAMED_LIMIT = 5;

const files = (count: number) => `${count} ${count === 1 ? 'file' : 'files'}`;

/**
 * One message for an add: what was copied into the input folder and queued,
 * what was there already, and each file left out with its reason - "Copied 24
 * files into Input. Skipped 1: notes.zip (not a supported format)." A report
 * without a copy count - setup's "Process them too", which queues what is
 * already in the input folder - says what it added instead.
 */
export function describeAddReport(report: AddReport): string {
  const parts: string[] = [];
  const into = report.inputFolder ? folderName(report.inputFolder) : 'your input folder';
  const held = report.held ?? 0;
  if (report.copied !== undefined) {
    if (report.copied > 0) parts.push(`Copied ${files(report.copied)} into ${into}.`);
    const inPlace = report.inPlace ?? 0;
    if (inPlace > 0) parts.push(`${inPlace} already in ${into} ${inPlace === 1 ? 'was' : 'were'} added to the queue.`);
    if (held > 0) {
      const one = held === 1;
      // The usual reason is the verified team Inbox waiting for OneDrive's
      // upload of the copy; any other is said without promising Microsoft.
      parts.push(report.heldReason === undefined || report.heldReason === 'AWAITING_MICROSOFT'
        ? `${one ? 'It joins' : 'They join'} the queue once OneDrive has uploaded ${one ? 'it' : 'them'} and Microsoft confirms ${one ? 'it is' : 'they are'} yours.`
        : `${held} ${one ? 'waits' : 'wait'} in ${into} until Intern can confirm who uploaded ${one ? 'it' : 'them'}.`);
    }
  } else if (report.added > 0) {
    parts.push(`Added ${report.added} ${report.added === 1 ? 'document' : 'documents'}.`);
  }
  if (report.alreadyQueued > 0) {
    parts.push(`${report.alreadyQueued} ${report.alreadyQueued === 1 ? 'was' : 'were'} already in the queue.`);
  }
  if (report.skipped.length > 0) {
    const named = report.skipped.slice(0, NAMED_LIMIT).map(({ name, code }) => `${name} (${SKIP_REASONS[code] ?? code})`);
    const more = report.skipped.length - named.length;
    parts.push(`Skipped ${report.skipped.length}: ${named.join(', ')}${more > 0 ? `, and ${more} more` : ''}.`);
  }
  if (parts.length > 0) return parts.join(' ');
  return report.copied !== undefined ? 'No documents were found to copy.' : 'No documents were found to add.';
}
