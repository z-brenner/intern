import { reviewLevelOf } from '../../lib/reviewLevel';
import type { AppSettings, QueueItem } from '../../types';

/*
  With automatic renaming on, the backend renames every ready document nobody
  has approved, under Intern's name for it. It does not know what a person
  typed in review and did not file: that is the window's. Rename all ready
  leaves such a document alone and says so; automatic renaming cannot.

  Two saves of the settings can therefore rename a document under the very
  name its person was correcting: automatic renaming switched on while a
  ready document has something typed for it, and the level changed from
  Strict to Balanced with automatic renaming on, which judges the documents
  waiting in review again and makes some of them ready. Settings says so
  before either is saved (`typedWarning`), from the counts here.
*/

/**
 * How many documents have a name or a description a person typed and did not
 * file, by where they wait: ready, or in review.
 */
export interface TypedWaiting { ready: number; review: number }

/** Nothing typed anywhere, and what Settings is given where there is no queue yet. */
export const NOTHING_TYPED: TypedWaiting = { ready: 0, review: 0 };

/**
 * Counts the documents with something typed and not filed (`typed`, by a
 * document's id) that automatic renaming could rename: those that are ready,
 * and those waiting in review.
 *
 * Only a document Intern has a name for. One it could not read has none, so
 * nothing renames it by itself at either level, whatever was typed for it. A
 * ready document a person approved is theirs already, and a staged copy is
 * named after its original. A document being read, renamed, failed or
 * completed is not waiting for a name.
 *
 * No name is a name left out, and also one that is null: the desktop backend
 * always sends the key (QueueItemDto has no skip on it), and the bridge
 * hands a null on as it came, whatever `QueueItem` says of its type.
 */
export function typedWaiting(items: readonly QueueItem[], typed: (id: string) => boolean): TypedWaiting {
  const counts = { ready: 0, review: 0 };
  for (const item of items) {
    if (!item.proposedFilename || !typed(item.id)) continue;
    if (item.status === 'review') counts.review += 1;
    else if (item.status === 'ready' && item.approved !== true && item.copyOf === undefined) counts.ready += 1;
  }
  return counts;
}

/**
 * How many documents with something typed and not filed Intern can rename
 * under the names it proposed once `draft` is saved over `loaded`. None
 * unless the draft has automatic renaming on. Then:
 *
 * - the ready ones, when automatic renaming was off: it is this save that
 *   lets Intern rename them;
 * - the ones in review, when the level goes from Strict to Balanced: this
 *   save has them judged again, and those the level lets through are ready.
 *   Which ones is the engine's rule and is not worked out here, so this is
 *   the most there can be, and the warning says "can".
 *
 * A save that changes neither renames nothing it would not have renamed
 * already, and Balanced back to Strict makes nothing ready.
 */
export function renamedOverTyped(loaded: AppSettings, draft: AppSettings, typed: TypedWaiting): number {
  if (!draft.automaticRename) return 0;
  const ready = loaded.automaticRename ? 0 : typed.ready;
  const review = reviewLevelOf(loaded) === 'strict' && reviewLevelOf(draft) === 'balanced' ? typed.review : 0;
  return ready + review;
}

/**
 * The warning for `count` such documents, none for none. It says only what
 * was seen to happen: a document renamed by Intern while it had something
 * typed for it keeps that text beside it, read-only, in its details
 * (`setAsideOutdatedDrafts`), and its name is Intern's. "The name it
 * proposed", not "its own name", which of a document means the name it
 * came with.
 */
export function typedWarning(count: number): string {
  if (count <= 0) return '';
  const one = count === 1;
  return [
    `${one ? '1 document has' : `${count} documents have`} a name or a description you typed and did not file.`,
    `When you save, Intern can rename ${one ? 'it under the name' : 'them under the names'} it proposed.`,
    `You can file ${one ? 'it' : 'them'} first, or take back what you typed.`,
    `What you typed is kept with ${one ? 'the' : 'each'} document, but is not filed.`,
  ].join(' ');
}
