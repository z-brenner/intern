import type { QueueItem } from '../../types';

/**
 * The name and description being edited for one document (`item`), and the
 * proposal they were typed over (`for`, see `proposalOf`).
 */
export interface ReviewDraft { item: string; for: string; stem: string; description: string; edited: boolean }

/**
 * What was typed in review and not filed: one draft a document, under the
 * document's id. It is the window's, not the review panel's. The panel
 * leaves the screen without anything being decided - a wide panel when the
 * window is made narrower than 1100px, a drawer when it is made wider,
 * either by Close review, Escape or the Details toggle, and when the
 * selection is cleared - and while the panel kept what was typed, each of
 * those threw it away: the fields came back as Intern's proposal.
 */
export type ReviewDrafts = Map<string, ReviewDraft>;

/**
 * The proposal a draft is typed over: the document, and its proposal's
 * revision, name and sentence. What was typed is shown only over the
 * proposal it was typed over, so a document read again shows its new
 * reading and not an edit of the one before.
 *
 * The revision alone does not say that. It moves when a waiting document's
 * name or sentence changes where it waits, but a new reading is numbered
 * from 1 again (`record_for` in pipeline.rs), as the reading before it
 * usually was. While the panel kept its drafts that seldom showed: Analyze
 * again closes the panel. Kept by the window under the document and the
 * revision only, a name typed over one reading would have come back in the
 * field over the next.
 */
export function proposalOf(item: QueueItem): string {
  return JSON.stringify([item.id, item.proposalRevision ?? '', item.proposedFilename ?? '', item.description ?? '']);
}

/**
 * Forgets what was typed over a proposal its document no longer has: the
 * document was read again, or its name or sentence changed where it waits,
 * or it was approved under another name. Even if the same proposal comes
 * back - a file read again as it was - it is a new reading, shown as it is.
 * A document that is not in the list is left alone: a list that could not
 * be read is not a document that changed.
 */
export function forgetOutdatedDrafts(drafts: ReviewDrafts, items: readonly QueueItem[]): void {
  if (drafts.size === 0) return;
  for (const item of items) {
    const typed = drafts.get(item.id);
    if (typed && typed.for !== proposalOf(item)) drafts.delete(item.id);
  }
}
