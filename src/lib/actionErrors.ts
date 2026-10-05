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
  // fails with the system's own "used by another process".
  SOURCE_LOCKED: 'The document is open in another program, or a sync client is still writing it. Close it, then try again.',
  RECONCILIATION_REQUIRED: 'A rename of this document stopped part-way, so its files need checking first. Use Check again.',
};

/** What to tell a person about a command that failed, whatever shape the failure arrived in. */
export function describeActionError(error: unknown): string {
  const code = typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
  if (code && ACTION_ERRORS[code]) return ACTION_ERRORS[code];
  if (typeof error === 'string' && error.trim()) return error.trim();
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  return 'The operation could not be completed.';
}
