import type { QueueItem } from '../../types';
import { itemActions } from './actions';

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
 * A name and a description as a person had them, kept on screen where typing
 * changes nothing: the name, and the description under it. `edited` says
 * whether anything in it was typed; `key` tells one from another.
 */
export interface KeptText { key: number; text: string; edited: boolean }

/**
 * What was typed over a proposal its document no longer has, by the
 * document's id, oldest first. It is not in the fields - they show the
 * document as it now is - and it is not thrown away either: only the person
 * drops what they typed, by filing it, by deciding the document some other
 * way, or by discarding it.
 */
export type EarlierDrafts = Map<string, KeptText[]>;

let keptTexts = 0;

/** A text to keep, told from every other by its key. */
export function keptText(text: string, edited: boolean): KeptText {
  return { key: ++keptTexts, text, edited };
}

/** A name and the description under it, as one text. */
export function nameThenDescription(name: string, description: string): string {
  return description ? `${name}\n\n${description}` : name;
}

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
 * Sets aside what was typed over a proposal its document no longer has, or
 * can no longer be filed under: the document was read again, its name or
 * sentence changed where it waits, or it was filed from somewhere else.
 * Even if the same proposal comes back - a file read again as it was - it is
 * a new reading, shown as it is. What was typed moves from the fields'
 * drafts to `earlier`, where the panel shows it beside the document as it
 * now is. It used to be forgotten here, and a re-reading nobody asked for
 * threw away a name and a sentence mid-word, with no notice.
 *
 * Not for `deciding`, the document a decision of the person's own is being
 * carried out for: what that leaves of its draft is the decision's to say.
 * A document that is not in the list is left alone: a list that could not
 * be read is not a document that changed. Returns the documents whose
 * drafts were set aside.
 */
export function setAsideOutdatedDrafts(drafts: ReviewDrafts, earlier: EarlierDrafts, items: readonly QueueItem[], deciding?: string): string[] {
  const moved: string[] = [];
  if (drafts.size === 0) return moved;
  for (const item of items) {
    const typed = drafts.get(item.id);
    if (!typed || item.id === deciding) continue;
    if (typed.for === proposalOf(item) && itemActions(item).approve) continue;
    drafts.delete(item.id);
    earlier.set(item.id, [...(earlier.get(item.id) ?? []), keptText(nameThenDescription(typed.stem, typed.description), typed.edited)]);
    moved.push(item.id);
  }
  return moved;
}

/**
 * Whether a name or a description was typed for a document and not filed:
 * in its fields now, or set aside when the document changed. Rename all
 * ready leaves such a document for the person, since it files Intern's
 * proposal and what was typed would be gone without a word.
 */
export function typedAndNotFiled(drafts: ReviewDrafts, earlier: EarlierDrafts, id: string): boolean {
  return Boolean(drafts.get(id)?.edited) || Boolean(earlier.get(id)?.some((kept) => kept.edited));
}

/**
 * Drafts that say when a document gains or loses one, for whatever counts
 * them. Said a moment later, not as it happens: the panel keeps a draft
 * while it renders, and the window cannot be told to redraw from there.
 */
export function watchedDrafts(onChange: () => void): ReviewDrafts {
  const drafts: ReviewDrafts = new Map();
  const set = drafts.set.bind(drafts);
  const remove = drafts.delete.bind(drafts);
  const later = () => queueMicrotask(onChange);
  drafts.set = (id, draft) => { const had = drafts.get(id)?.edited === draft.edited; set(id, draft); if (!had) later(); return drafts; };
  drafts.delete = (id) => { const had = remove(id); if (had) later(); return had; };
  return drafts;
}
