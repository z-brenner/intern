import { humanizeReason } from './reasons';

/**
 * Plain sentences for the codes a review action can be refused with. The
 * backend's messages are written for its logs - "filename must be one
 * nonblank path component" - and a code with no entry here still shows its
 * message, as before.
 */
const ACTION_ERRORS: Record<string, string> = {
  NAME_INVALID: 'That filename cannot be used. Keep the file\'s extension, and leave out / \\ < > : " | ? * and invisible characters.',
  ITEM_NOT_FOUND: 'That document is no longer in the queue.',
  PATH_UNAVAILABLE: 'The document is not where Intern last saw it. It may have been moved, renamed, or deleted outside Intern.',
  UNSUPPORTED_FORMAT: 'Intern opens only the document formats it reads.',
  INVALID_TRANSITION: 'That action does not apply to this document in its current state.',
  // Open, read, approve is the review the panel invites, and on Windows the
  // viewer still holds the file it opened: the rename, or an undo, then
  // fails with the system's own "used by another process". The backend rolls
  // a refused rename back and leaves the document in review under this same
  // code, so the refusal and the item's reason are one sentence.
  SOURCE_LOCKED: humanizeReason('SOURCE_LOCKED'),
  // Keep, cancel or remove of an item whose files were left part-way by a
  // rename - one the panel showed before it was parked. Checking them is the
  // way on, and the panel now offers it.
  RECONCILIATION_REQUIRED: 'A rename of this document stopped part-way, so its files need checking first. Use Check again.',
};

/**
 * What the refused command was, where one code means different things to
 * different commands and the sentence for one would mislead about another.
 */
export interface RefusedAction {
  /** Approve & rename, Apply rename, or Apply all ready. */
  approve?: boolean;
  /** The command looked at a stopped rename's files first: Check again, or approving a parked item. */
  checkedFiles?: boolean;
}

/**
 * An approval refused because the file changed since it was read. The
 * backend says "Re-analyze it", and the panel calls that Analyze again. Only
 * for an approval: an undo refused with the same code is about a filed copy
 * that was moved, which the backend names by its path.
 */
const APPROVAL_FILE_CHANGED = 'The file changed after it was analyzed, so this name may no longer fit it. Use Analyze again, under More review actions, to read it as it is now.';

/**
 * Checking the files again was itself what failed: they are still not where
 * either a finished rename or one that never happened would leave them. The
 * usual sentence would send the person back to Check again, which has just
 * said the same; sorting the files out by hand, and saying so, is the way out.
 */
const STILL_UNSETTLED = 'Intern checked the files again and still cannot tell which one is the document. Sort them out yourself, then use Remove from queue and choose “I have resolved the files myself”.';

function refusalCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}

/** What to tell a person about a command that failed, whatever shape the failure arrived in. */
export function describeActionError(error: unknown, refused: RefusedAction = {}): string {
  const code = refusalCode(error);
  if (code === 'RECONCILIATION_REQUIRED' && refused.checkedFiles) return STILL_UNSETTLED;
  if (code === 'FILE_CHANGED' && refused.approve) return APPROVAL_FILE_CHANGED;
  if (code && ACTION_ERRORS[code]) return ACTION_ERRORS[code];
  if (typeof error === 'string' && error.trim()) return error.trim();
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  return 'The operation could not be completed.';
}
