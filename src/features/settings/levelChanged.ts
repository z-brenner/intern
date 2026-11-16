import type { QueueItem, ReviewLevel } from '../../types';
import { itemNote } from '../queue/check';
import { keptOriginal } from '../review/actions';

/*
  A Settings save that changes the review level has the backend judge the
  documents that wait again, from what Intern has already read. Toward
  Balanced some of those in review become ready, and with automatic renaming
  on the scheduler can rename them by itself after the save (not while the
  queue is paused, while it reads a backlog, or while no model is ready to
  read). Toward Strict, a ready document nobody approved that Strict would
  hold goes back to review. The queue changed under the dialog, and nothing
  on screen said what the save had done.

  So the window says it, from its own rows: the queue as it was just before
  the save, and as the first read after it shows it. It says what it saw,
  and nothing of what the scheduler will do next.
*/

/** What a level change was seen to do. `typed`: how many of the documents counted have a name or a description typed and not filed. */
export interface LevelChange { to: ReviewLevel; count: number; typed: number }

/** How documents that are ready get renamed after the save: by Intern itself, not while the queue is paused, or with Rename all ready. */
export type ReadyRenamed = 'automatic' | 'paused' | 'off';

interface Seen {
  /** A document the window decided between the two reads: approved, kept, removed, read again, undone. */
  decidedHere(id: string): boolean;
  /** A name or a description typed for the document and not filed. */
  typed(id: string): boolean;
}

/**
 * Whether a row nobody has approved is where a level could have put it from
 * review: ready, being renamed, or renamed. A person's approval clears the
 * reasons on a proposal, so a row that still carries them was approved by
 * nobody, wherever that approval might have been made. A document a person
 * kept under its own name keeps the review's words but gets no
 * ALREADY_NAMED; one Intern completed by itself has it beside the rest.
 */
function readyByTheLevel(item: QueueItem): boolean {
  const codes = item.reasonCodes ?? [];
  if (!codes.length || item.copyOf !== undefined) return false;
  if (item.status === 'ready') return item.approved !== true;
  if (item.status === 'processing') return item.stage === 'filing';
  if (item.status !== 'completed') return false;
  return !keptOriginal(item) || (codes.includes('ALREADY_NAMED') && codes.some((code) => code !== 'ALREADY_NAMED'));
}

/** A ready row with a note that nobody approved: what Strict can send back to review. */
const readyWithANote = (item: QueueItem) => item.status === 'ready' && item.approved !== true && item.copyOf === undefined && itemNote(item) !== undefined;

/**
 * What a save that changed the level to `to` did to the documents that
 * waited, from the rows before it and the rows read after it.
 *
 * Toward Balanced: those that waited in review and are now ready, being
 * renamed or renamed, with no decision of the person's in between. Toward
 * Strict: those that were ready with a note, approved by nobody, and now
 * wait in review. A document the window decided meanwhile is neither, and
 * neither is one being read, one that failed, or one a person completed.
 */
export function levelChanged(before: readonly QueueItem[], after: readonly QueueItem[], to: ReviewLevel, seen: Seen): LevelChange {
  const now = new Map(after.map((item) => [item.id, item]));
  let count = 0;
  let typed = 0;
  for (const was of before) {
    const is = now.get(was.id);
    if (!is || seen.decidedHere(was.id)) continue;
    const moved = to === 'balanced'
      ? was.status === 'review' && readyByTheLevel(is)
      : readyWithANote(was) && is.status === 'review';
    if (!moved) continue;
    count += 1;
    if (seen.typed(was.id)) typed += 1;
  }
  return { to, count, typed };
}

/**
 * The words for a level change, in the voice of Rename all ready's outcome:
 * the level, then how many documents, then what renames them. Only counts
 * of documents; no figure of how often a level is right.
 */
export function levelChangedText({ to, count, typed }: LevelChange, renamed: ReadyRenamed): string {
  const one = count === 1;
  if (to === 'strict') {
    return [
      'The review level is now Strict.',
      count === 0 ? 'No ready document with a note went back to review.' : `${count} ready ${one ? 'document' : 'documents'} with a note ${one ? 'needs' : 'need'} review again.`,
    ].join(' ');
  }
  if (count === 0) return 'The review level is now Balanced. No document that needed review became ready.';
  const them = one ? 'it' : 'them';
  const how = renamed === 'automatic' ? `With automatic renaming on, Intern renames ${them} by itself.`
    : renamed === 'paused' ? 'The queue is paused, so Intern renames nothing by itself until it is resumed.'
      : byRenameAllReady(count, typed);
  return [
    'The review level is now Balanced.',
    `${count} ${one ? 'document' : 'documents'} that needed review became ready.`,
    how,
  ].join(' ');
}

/**
 * With automatic renaming off, Rename all ready renames them: all but one
 * with a name or a description typed and not filed, which it leaves for the
 * person, as its own outcome says.
 */
function byRenameAllReady(count: number, typed: number): string {
  if (typed === 0) return `Rename all ready renames ${count === 1 ? 'it' : 'them'}.`;
  if (typed === count) return count === 1
    ? 'It has a name or a description you typed, so Rename all ready leaves it for you to file.'
    : 'They have a name or a description you typed, so Rename all ready leaves them for you to file.';
  return `Rename all ready renames ${count - typed}. ${typed} with a name or a description you typed ${typed === 1 ? 'is' : 'are'} left for you to file.`;
}
