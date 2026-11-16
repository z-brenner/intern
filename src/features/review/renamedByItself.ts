import type { QueueItem } from '../../types';

/*
  Intern can rename by itself a document whose name or description a person
  typed in Details and did not file: automatic renaming after a reading, at
  Balanced after a level change, and after validating again. What was typed
  is then set aside beside the document, read-only (setAsideOutdatedDrafts).

  The window says so, and only when the backend says the document was
  renamed with nobody's approval (renamedWithoutApproval). Every approval
  the backend carries out marks the document's record approved, and filing
  leaves the record as it was, so a rename the person approved is not
  marked once their approval is done, however it was finished: at once, by
  the queue once it was free, by Check again, by a reconciliation after an
  error, or by the check an approval makes first, of a rename Intern had
  started by itself that the approval names exactly. That last one is
  marked while the approval is carried out, until the backend records it:
  a queue read made then finds it so. So the window judges no document
  while a decision of the person's on it is carried out, and judges it
  once the decision ends. The window used to tell whose rename it was from
  the rows going by, and told some of the person's own renames as made
  without them, with an Undo that would have undone them.

  It says a rename when it sees it happen: a document it saw not yet filed
  that is then completed and marked. Nothing is said of a document already
  renamed when the window first read the queue.
*/

/** A document said to be renamed without the person, by the name it arrived with. */
export interface RenameSaid { id: string; name: string }

/** Being renamed: the move a rename or an undo makes, which cannot be canceled. */
export const beingRenamed = (item: QueueItem) => item.status === 'processing' && item.stage === 'filing';

/** Renamed by Intern with nobody's approval, as the backend says. */
export const renamedWithoutApproval = (item: QueueItem) => item.status === 'completed' && item.renamedWithoutApproval === true;

/**
 * Looks at the queue as it is now (`rows`) against the documents the window
 * last saw not yet filed (`unfiled`). Returns those to look for from here,
 * and the documents renamed without the person in between with something
 * typed for them and not filed (`typed`).
 *
 * A document being renamed is left as it was: an undo carried out passes
 * through it from completed, and an undo the backend refuses comes back
 * completed and marked, which is no new rename. So is the document a
 * decision of the person's is being carried out for (`deciding`), whatever
 * it shows: their approval of exactly what Intern was filing by itself,
 * finished as it is approved, shows completed and marked until the backend
 * records the approval. Once the decision ends it is judged as it then is:
 * an approval carried out has forgotten what was typed for it, and one the
 * backend refused has not. A document seen renamed is no longer looked
 * for, so it is said once; one that comes back waiting is looked for again.
 */
export function watchRenames(unfiled: ReadonlySet<string>, rows: readonly QueueItem[], typed: (id: string) => boolean, deciding?: string): { unfiled: Set<string>; seen: RenameSaid[] } {
  const next = new Set<string>();
  const seen: RenameSaid[] = [];
  for (const row of rows) {
    if (row.id === deciding) {
      if (unfiled.has(row.id)) next.add(row.id);
    } else if (row.status === 'completed') {
      if (unfiled.has(row.id) && renamedWithoutApproval(row) && typed(row.id)) seen.push({ id: row.id, name: row.originalFilename });
    } else if (!beingRenamed(row) || unfiled.has(row.id)) {
      next.add(row.id);
    }
  }
  return { unfiled: next, seen };
}

/**
 * What is said with those just seen added, and those no longer so (`gone`)
 * taken out. The very list it was given when that changes nothing.
 */
export function withRenames(said: readonly RenameSaid[], seen: readonly RenameSaid[], gone: readonly string[]): readonly RenameSaid[] {
  const out = new Set(gone);
  const known = new Set(said.map(({ id }) => id));
  const kept = said.filter(({ id }) => !out.has(id));
  const added = seen.filter(({ id }) => !known.has(id) && !out.has(id));
  return kept.length === said.length && added.length === 0 ? said : [...kept, ...added];
}

/**
 * What is still so of what was said, by the queue as it is now and what is
 * kept of what was typed (`typed`): the document still completed and marked,
 * or being renamed (an Undo of it carried out), with what was typed for it
 * still kept in its details. The window takes out for good what is no
 * longer so by the queue alone (App), so a document undone and then filed
 * by the person's own approval is not said again.
 */
export function stillSo(said: readonly RenameSaid[], rows: readonly QueueItem[], typed: (id: string) => boolean): RenameSaid[] {
  const now = new Map(rows.map((row) => [row.id, row]));
  return said.filter(({ id }) => {
    const row = now.get(id);
    return row !== undefined && (renamedWithoutApproval(row) || beingRenamed(row)) && typed(id);
  });
}

/** The renames Undo puts back: those still completed that can still be undone. */
export function undoableRenames(said: readonly RenameSaid[], rows: readonly QueueItem[]): string[] {
  const now = new Map(rows.map((row) => [row.id, row]));
  return said.filter(({ id }) => now.get(id)?.status === 'completed' && now.get(id)?.undoable === true).map(({ id }) => id);
}

/**
 * The words, in the voice of Rename all ready's outcome: which document, or
 * how many, that nobody approved the name, and where what was typed is now.
 * "Renamed", as the window says it elsewhere.
 */
export function renamedByItselfText(renames: readonly RenameSaid[]): string {
  if (renames.length === 0) return '';
  if (renames.length === 1) return `${renames[0].name} was renamed under Intern's proposal, without anyone approving it. What you typed for it was not filed, and is kept in its details.`;
  return `${renames.length} documents with a name or a description you typed were renamed under Intern's proposals, without anyone approving them. What you typed for each was not filed, and is kept in its details.`;
}
