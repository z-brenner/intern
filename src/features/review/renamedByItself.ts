import type { QueueItem } from '../../types';
import { keptOriginal } from './actions';

/*
  Intern can rename by itself a document whose name or description a person
  typed in Details and did not file: automatic renaming after a reading it
  started by itself, and at Balanced after a level change. What was typed is
  then set aside beside the document, read-only (setAsideOutdatedDrafts),
  and nothing else said it happened. A document filed from somewhere else
  is the same to the window.

  So the window watches the documents that wait for a decision, and says
  so when one with something typed for it leaves review or ready to be
  renamed, or renamed, and the window did not decide it. It knows its own
  decisions (the document it is deciding for); anything else is not its own.
  A document read again by itself stays watched while it waits and is read:
  with automatic renaming on, the backend files a reading the moment it is
  ready, so a read of the queue seldom shows it ready in between.
*/

/** A document as the window last saw it waiting for a decision, kept so while it is read again. `renaming`: since seen being renamed. */
export interface Waiting { name: string; proposedFilename?: string; description?: string; approved: boolean; renaming: boolean }

/**
 * Whose name a document was renamed under, as far as the window can tell:
 * the proposal it last showed for it (`proposal`), or one it cannot place
 * (`elsewhere`).
 */
export type Whose = 'proposal' | 'elsewhere';

/** A document seen renamed, or being renamed, over what was typed for it. `done`: renamed, not only being renamed. */
export interface RenameSeen { id: string; name: string; whose: Whose; done: boolean }

interface Watch {
  /** A name or a description typed for the document and not filed. */
  typed(id: string): boolean;
  /** The document a decision of the window's own is being carried out for. */
  decidedHere(id: string): boolean;
}

/** Being renamed: the move a rename makes, which cannot be canceled. */
export const beingRenamed = (item: QueueItem) => item.status === 'processing' && item.stage === 'filing';

/**
 * The name the document is renamed under is the one the window last showed
 * proposed for it, nobody had approved another, and its sentence is the
 * same: renamed under Intern's proposal. Anything else it cannot place:
 * another name, or an approval the window did not make. The name filed can
 * still differ from the proposal by a number the folder needed, so the
 * proposal is what is compared, not the filed name.
 */
function whose(was: Waiting, is: QueueItem): Whose {
  return !was.approved && Boolean(was.proposedFilename) && is.proposedFilename === was.proposedFilename && (is.description ?? '') === (was.description ?? '') ? 'proposal' : 'elsewhere';
}

/**
 * Looks at the queue as it is now (`rows`) against the documents the window
 * last saw waiting (`watched`). Returns what to watch from here, the
 * renames over typed text seen in between (`seen`), and the documents seen
 * being renamed that came back without being renamed (`back`): sent back to
 * review, failed, or gone from the queue.
 *
 * Nothing is seen of a document the window is deciding for: whatever
 * becomes of it is the decision's to say. It stays watched as it was while
 * the decision is carried out, since the backend can refuse it because the
 * document is being renamed by itself at that moment; once the backend
 * accepts it, the window stops watching it (App). Nothing is seen either of
 * one with nothing typed for it at the moment it is renamed. A completion
 * that renamed nothing (the name was already the document's own) is not a
 * rename.
 *
 * One waiting to be read again, or being read, is still watched, with the
 * proposal the window last showed for it: the new one is not known until a
 * read shows it in review or ready. Renamed before that, under another name
 * or sentence, it is renamed under a name the window cannot place.
 */
export function watchRenames(watched: ReadonlyMap<string, Waiting>, rows: readonly QueueItem[], watch: Watch): { watched: Map<string, Waiting>; seen: RenameSeen[]; back: string[] } {
  const next = new Map<string, Waiting>();
  const seen: RenameSeen[] = [];
  const back: string[] = [];
  const listed = new Set<string>();
  for (const row of rows) {
    listed.add(row.id);
    const was = watched.get(row.id);
    if (watch.decidedHere(row.id)) {
      if (was?.renaming) back.push(row.id);
      if (was) next.set(row.id, { ...was, renaming: false });
      continue;
    }
    if (row.status === 'review' || row.status === 'ready') {
      if (was?.renaming) back.push(row.id);
      next.set(row.id, { name: row.originalFilename, proposedFilename: row.proposedFilename ?? undefined, description: row.description ?? undefined, approved: row.approved === true, renaming: false });
    } else if (beingRenamed(row) && was) {
      if (!was.renaming && watch.typed(row.id)) seen.push({ id: row.id, name: was.name, whose: whose(was, row), done: false });
      next.set(row.id, { ...was, renaming: true });
    } else if (row.status === 'completed' && !keptOriginal(row) && was) {
      if (watch.typed(row.id)) seen.push({ id: row.id, name: was.name, whose: whose(was, row), done: true });
    } else if (was && (row.status === 'waiting' || row.status === 'processing')) {
      // Read again by itself (being renamed is the branch above). A reading
      // the person asks for here is a decision of the window's, so it is not
      // watched until a read shows it waiting for a decision again; their
      // Read again drops what was typed as well.
      if (was.renaming) back.push(row.id);
      next.set(row.id, { ...was, renaming: false });
    } else if (was?.renaming) {
      back.push(row.id);
    }
  }
  for (const [id, was] of watched) if (!listed.has(id) && was.renaming) back.push(id);
  return { watched: next, seen, back };
}

/**
 * What is said of the renames seen so far, with those just seen: a document
 * seen again replaces what was said of it (being renamed, then renamed), and
 * one that came back without being renamed is taken out. The very list it
 * was given when that changes nothing.
 */
export function withRenames(said: readonly RenameSeen[], seen: readonly RenameSeen[], back: readonly string[]): readonly RenameSeen[] {
  const gone = new Set(back);
  const fresh = new Map(seen.map((rename) => [rename.id, rename]));
  const kept = said.filter((rename) => !gone.has(rename.id)).map((rename) => fresh.get(rename.id) ?? rename);
  const known = new Set(kept.map((rename) => rename.id));
  const next = [...kept, ...seen.filter((rename) => !known.has(rename.id) && !gone.has(rename.id))];
  // Nothing new to say (a document came back that was never said to be renamed): what was said, as it was.
  const same = (one: RenameSeen, other: RenameSeen) => one.id === other.id && one.name === other.name && one.whose === other.whose && one.done === other.done;
  return next.length === said.length && next.every((rename, index) => same(rename, said[index])) ? said : next;
}

/**
 * What is still so of what was said, by the queue as it is now and what is
 * kept of what was typed (`typed`): a document renamed, and still
 * completed; one being renamed, and still being renamed or since renamed;
 * each with what was typed for it still kept in its details. One undone
 * since, cleared from the history, or gone is no longer said to be renamed.
 * Nor is one whose typed text is no longer kept: the person discarded it,
 * or a decision of theirs here filed it or gave it up.
 *
 * This only hides. A document undone and then renamed by the person's own
 * approval can be completed again with what was typed still kept (an
 * approval that stopped halfway and was finished by Check again), and would
 * show here again. So the window takes out for good a rename the queue no
 * longer shows so by its status (App), before it is said again.
 */
export function stillSo(said: readonly RenameSeen[], rows: readonly QueueItem[], typed: (id: string) => boolean): RenameSeen[] {
  const now = new Map(rows.map((row) => [row.id, row]));
  return said.filter(({ id, done }) => {
    const row = now.get(id);
    return row !== undefined && (row.status === 'completed' || (!done && beingRenamed(row))) && typed(id);
  });
}

/** The renames Undo puts back: those the window saw complete, and that can still be undone. */
export function undoableRenames(said: readonly RenameSeen[], rows: readonly QueueItem[]): string[] {
  const now = new Map(rows.map((row) => [row.id, row]));
  return said.filter(({ id, done }) => done && now.get(id)?.status === 'completed' && now.get(id)?.undoable === true).map(({ id }) => id);
}

/**
 * The words, in the voice of Rename all ready's outcome: which document, or
 * how many, whose name each was renamed under, and where what was typed is
 * now. "Renamed", as the window says it elsewhere.
 */
export function renamedByItselfText(renames: readonly RenameSeen[]): string {
  if (renames.length === 0) return '';
  const allDone = renames.every((rename) => rename.done);
  const kept = `What you typed for ${renames.length === 1 ? 'it' : 'each'} ${allDone ? 'was' : 'is'} not filed, and is kept in its details.`;
  if (renames.length === 1) {
    const [{ name, whose: by }] = renames;
    return `${name} ${allDone ? 'was' : 'is being'} renamed ${by === 'proposal' ? "under Intern's proposal" : 'from somewhere else'}. ${kept}`;
  }
  const what = allDone ? 'were renamed' : renames.some((rename) => rename.done) ? 'were renamed or are being renamed' : 'are being renamed';
  const ours = renames.filter((rename) => rename.whose === 'proposal').length;
  const elsewhere = renames.length - ours;
  const by = elsewhere === 0 ? " under Intern's proposals" : ours === 0 ? ' from somewhere else' : `, ${ours} under Intern's ${ours === 1 ? 'proposal' : 'proposals'} and ${elsewhere} from somewhere else`;
  return `${renames.length} documents with a name or a description you typed ${what}${by}. ${kept}`;
}
