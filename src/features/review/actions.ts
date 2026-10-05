import type { QueueItem } from '../../types';

/**
 * Review codes the backend's retry accepts (Pipeline::retry): a duplicate is
 * processed anyway, an unverified upload is checked again, and a renamed copy
 * whose original could not be deleted has the deletion tried again. Every
 * other review reason is refused with INVALID_TRANSITION, which is why Retry
 * is not on the menu for them - SOURCE_LOCKED included: a rename refused
 * because another program held the file is rolled back, and approving again
 * once it is closed is how it is retried.
 */
const RETRYABLE_REVIEW_CODES = new Set(['DUPLICATE', 'UPLOADER_UNVERIFIED', 'SOURCE_DELETE_FAILED']);

/** The two codes that always meant a rename's files need checking, before the backend said so itself. */
const PARKED_CODES = new Set(['SOURCE_DELETE_FAILED', 'RECONCILIATION_REQUIRED']);

/**
 * Whether a review item's files need checking before it can be decided. The
 * backend reports it (`parked`); a backend from before that field is read
 * from the codes that meant it.
 */
export function isParked(item: QueueItem): boolean {
  if (item.status !== 'review') return false;
  return item.parked ?? PARKED_CODES.has(item.errorCode ?? '');
}

/**
 * Whether a completed item kept the name it arrived with. The backend says so
 * itself once it reports filings (`keptOriginal`). Until then, a completed
 * item has a finished rename behind it exactly when it can be undone - the
 * same test the backend's own intake report makes - so one that cannot be
 * undone, and names no filing, was not renamed. Falling back to the proposal
 * instead said "Renamed to" a name the file never had: the backend keeps an
 * unapplied proposal on a kept original.
 */
export function keptOriginal(item: QueueItem): boolean {
  if (item.status !== 'completed') return false;
  return item.keptOriginal ?? (item.undoable !== true && item.filedName === undefined);
}

/** Whether the backend would accept Retry for this item. */
export function retryAccepted(item: QueueItem): boolean {
  if (item.status === 'failed') return true;
  return item.status === 'review' && (isParked(item) || RETRYABLE_REVIEW_CODES.has(item.errorCode ?? ''));
}

/**
 * Whether the backend would accept Re-analyze (Pipeline::reanalyze): any
 * ready or review item, whatever its code, except one whose files need
 * checking - an operation that never finished leaves what is on disk an open
 * question, and the backend refuses until it is checked again.
 */
export function reanalyzeAccepted(item: QueueItem): boolean {
  return (item.status === 'ready' || item.status === 'review') && !isParked(item);
}

/**
 * Ready and review items wait for a decision, unless their files need
 * checking first - or the item is ready because it was approved already, and
 * waits only for the queue to be free to file it. Counted as undecided, an
 * approval the backend deferred was left "to decide", and review came back
 * round to it after the next one.
 */
export function undecided(item: QueueItem): boolean {
  if (item.status === 'ready') return item.approved !== true;
  return item.status === 'review' && !isParked(item);
}

/** What the inspector offers for an item: exactly what the backend accepts for it, and nothing it refuses. */
export interface ItemActions {
  /** Approve & rename (review) or Apply rename (ready). Needs a proposal: the backend has nothing to approve without one. */
  approve: boolean;
  keep: boolean;
  /** Retry under the name that says what it does here, and whether it is the item's main action. */
  retry?: { label: string; primary: boolean };
  reanalyze: boolean;
  /** Remove, and whether the person must first say they resolved the files themselves. */
  remove?: { label: string; resolvedFiles: boolean };
  cancel: boolean;
  undo: boolean;
  /** Open, in the program the system uses for it: the source until it is filed, the filed copy after. */
  open: boolean;
  /** Show in folder, the same document selected where it is. */
  reveal: boolean;
}

export function itemActions(item: QueueItem): ItemActions {
  const none: ItemActions = { approve: false, keep: false, reanalyze: false, cancel: false, undo: false, open: false, reveal: false };
  switch (item.status) {
    case 'review': {
      // Parked: the files decide what happens next, not the name. Keeping,
      // re-analyzing and an unconfirmed remove are refused until they are
      // checked, and approving checks them first and may find the rename
      // finished already - so the check is the main action rather than an
      // entry in a menu.
      if (isParked(item)) return { ...none, retry: { label: 'Check again', primary: true }, remove: { label: 'Remove from queue', resolvedFiles: true }, open: true, reveal: true };
      const retry = item.errorCode === 'DUPLICATE' ? 'Process anyway' : item.errorCode === 'UPLOADER_UNVERIFIED' ? 'Check again' : undefined;
      return {
        ...none,
        approve: item.proposedFilename !== undefined,
        keep: true,
        ...(retry ? { retry: { label: retry, primary: false } } : {}),
        // Accepted for every review code, a duplicate's and an unverified
        // upload's too: read again from the start, a duplicate is processed
        // and an upload's uploader is checked again on the way in.
        reanalyze: reanalyzeAccepted(item),
        remove: { label: 'Remove from queue', resolvedFiles: false },
        // Held because nobody could vouch for who uploaded it: opening it
        // would start its program from Intern's window, so it can only be
        // found in its folder until the uploader checks out.
        open: item.errorCode !== 'UPLOADER_UNVERIFIED',
        reveal: true,
      };
    }
    case 'ready':
      return { ...none, approve: item.proposedFilename !== undefined, keep: true, reanalyze: reanalyzeAccepted(item), remove: { label: 'Remove from queue', resolvedFiles: false }, open: true, reveal: true };
    case 'waiting':
      return { ...none, remove: { label: 'Remove from queue', resolvedFiles: false } };
    case 'failed':
      return { ...none, retry: { label: 'Retry item', primary: false }, remove: { label: 'Remove item', resolvedFiles: false } };
    case 'processing':
      return { ...none, cancel: item.cancelable !== false };
    case 'completed':
      return { ...none, undo: item.undoable === true, open: true, reveal: true };
  }
  return none;
}

/**
 * Items still waiting on a person, in the order review works through them:
 * those needing review first, then those ready to apply, each in the order
 * the table shows them.
 */
export function undecidedOrder(rows: QueueItem[]): QueueItem[] {
  return [...rows.filter((item) => item.status === 'review'), ...rows.filter((item) => item.status === 'ready')].filter(undecided);
}

/**
 * The undecided item to go to after `currentId`: the next one after it in
 * `undecidedOrder`, wrapping round to those skipped earlier. `before` is the
 * order when the decision was made, so an item decided out of order still
 * hands on to the one that followed it; `rows` is the queue now.
 */
export function nextUndecided(before: QueueItem[], currentId: string, rows: QueueItem[]): QueueItem | undefined {
  const now = undecidedOrder(rows).filter((item) => item.id !== currentId);
  const index = before.findIndex((item) => item.id === currentId);
  const rotation = index < 0 ? before : [...before.slice(index + 1), ...before.slice(0, index)];
  for (const candidate of rotation) {
    const current = now.find((item) => item.id === candidate.id);
    if (current) return current;
  }
  return now[0];
}
