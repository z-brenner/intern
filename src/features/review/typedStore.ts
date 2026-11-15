import type { QueueItem } from '../../types';
import { itemActions, withoutReviewPrefix } from './actions';
import { keptText, nameThenDescription, proposalOf, watchedDrafts } from './drafts';
import type { EarlierDrafts, ReviewDrafts } from './drafts';

/**
 * Where what was typed in review and not filed is kept from one launch of
 * Intern to the next: one value in the window's own storage, on this
 * computer, where the Details preference is (`intern.detailsPanel`). The
 * window alone kept it, and closing Intern, a crash, or the restart an
 * update makes threw away every name and description typed and not filed,
 * with no notice.
 *
 * The number is the version of what the value holds. A value under this key
 * that is not this version's, or cannot be read as it, is nobody's typing:
 * it is ignored and removed.
 */
export const TYPED_KEY = 'intern.typedInReview.v1';

/**
 * The mark that the window which stored the value above closed in the
 * ordinary way: written as the window goes (`pagehide`), with what was
 * typed as it stood then, and taken out again by the next window before it
 * reads or writes anything else. A window that finds what was typed and no
 * mark does not know how the one before it ended - killed, the power cut,
 * or one of the ways out that end the process with no word to the page -
 * and so does not know that the stored text is the last there was.
 */
export const CLOSED_KEY = 'intern.typedInReview.closed';

/**
 * How much is stored. A name that can be filed is at most 512 bytes
 * (MAX_FILENAME_BYTES) and a description is a sentence or two, so a name or
 * a description longer than these is not stored; nor is a document's typing
 * that would take the whole value past `MAX_STORED` characters, the
 * documents coming first whose typing was kept last (typed, or shown again
 * in the panel). What is not stored is as all of it was before: on screen
 * while Intern is open, and gone when it closes.
 */
export const MAX_NAME = 1_000;
export const MAX_DESCRIPTION = 5_000;
export const MAX_STORED = 200_000;
/** A name and the description under it, set aside as one text (`nameThenDescription`). */
const MAX_SET_ASIDE = MAX_NAME + MAX_DESCRIPTION + 2;

/** What is stored for one document: its number in the queue, and what was typed for it. */
interface StoredDocument {
  id: string;
  /** Which document the number meant (`documentOf`). */
  document: string;
  /** What was in the fields, and the reading it was typed over (`readingOf`). */
  draft?: { reading: string; stem: string; description: string };
  /** What had been set aside when the document changed, oldest first. */
  earlier?: string[];
}

/**
 * Sixteen hexadecimal digits worked out from a text: two 32-bit sums over
 * its characters, each mixed with the other at the end. The same text gives
 * the same digits, so the digits can stand for the text where it is only
 * ever compared; they are stored in place of a document's file name and of
 * Intern's proposal for it, so the stored value holds what the person typed
 * and nothing Intern read.
 */
function digest(text: string): string {
  let low = 0xdeadbeef;
  let high = 0x41c6ce57;
  for (let index = 0; index < text.length; index += 1) {
    const unit = text.charCodeAt(index);
    low = Math.imul(low ^ unit, 2654435761);
    high = Math.imul(high ^ unit, 1597334677);
  }
  low = Math.imul(low ^ (low >>> 16), 2246822507) ^ Math.imul(high ^ (high >>> 13), 3266489909);
  high = Math.imul(high ^ (high >>> 16), 2246822507) ^ Math.imul(low ^ (low >>> 13), 3266489909);
  return [high, low].map((half) => (half >>> 0).toString(16).padStart(8, '0')).join('');
}

/**
 * The file name a document came with: without the NEEDS REVIEW mark Intern
 * puts in front of one it could not read, and without the " (2)" that mark
 * takes when its name is already taken (`mark_for_review` in file_ops.rs).
 * Both can come while the document waits, and it is the same document.
 */
function ownName(name: string): string {
  return withoutReviewPrefix(name).replace(/ \(\d+\)(?=(?:\.[^.]*)?$)/, '');
}

/**
 * Which document a number in the queue means: the number, and the file's
 * own name. The desktop queue numbers its documents from a counter that
 * never gives a number twice (`queue_items.id`, AUTOINCREMENT in store.rs),
 * so within one queue the number alone says which document. The name is
 * for a queue that is not the one the text was typed in - its database
 * replaced while the window's storage stayed - where the same number is
 * another document's.
 */
export function documentOf(item: QueueItem): string {
  return digest(JSON.stringify([item.id, ownName(item.originalFilename)]));
}

/**
 * Which reading of a document something was typed over, to be told again
 * after Intern has been closed: what the window goes by while it is open -
 * the document, and its proposal's revision, name and sentence
 * (`proposalOf`) - and with them the document's own file name and the model
 * that read it. A reading made again by another model is another reading
 * whatever it says; and a document Intern could not read has no proposal,
 * so its name is all that tells it from another under the same number.
 *
 * Not what else a listing says of the document - its status, the reasons it
 * waits for, the model's confidence: the backend validates a waiting
 * reading again after an update, and when only those move, what was typed
 * over the name and the sentence still stands (`revalidate_item` in
 * pipeline.rs keeps the revision for exactly that).
 */
export function readingOf(item: QueueItem): string {
  return digest(JSON.stringify([proposalOf(item), ownName(item.originalFilename), item.readBy?.modelId ?? '']));
}

/** The window's storage, or none where it cannot be had: a locked-down profile refuses even the question. */
export function windowStorage(): Storage | undefined {
  try { return window.localStorage; } catch { return undefined; }
}

function storedDocument(value: unknown): StoredDocument | undefined {
  if (typeof value !== 'object' || value === null) return undefined;
  const { id, document, draft, earlier } = value as Record<string, unknown>;
  if (typeof id !== 'string' || id === '' || typeof document !== 'string') return undefined;
  const stored: StoredDocument = { id, document };
  if (draft !== undefined) {
    if (typeof draft !== 'object' || draft === null) return undefined;
    const { reading, stem, description } = draft as Record<string, unknown>;
    if (typeof reading !== 'string' || typeof stem !== 'string' || typeof description !== 'string') return undefined;
    // A name has no line break (the field takes none), and neither is longer than what is stored.
    if (stem.length > MAX_NAME || description.length > MAX_DESCRIPTION || /[\r\n]/.test(stem)) return undefined;
    stored.draft = { reading, stem, description };
  }
  if (earlier !== undefined) {
    if (!Array.isArray(earlier) || !earlier.every((text) => typeof text === 'string' && text !== '' && text.length <= MAX_SET_ASIDE)) return undefined;
    if (earlier.length > 0) stored.earlier = earlier as string[];
  }
  return stored.draft || stored.earlier ? stored : undefined;
}

/**
 * What is stored, read without ever throwing. Storage that cannot be read
 * holds nothing. A value that is not what Intern writes - not JSON, another
 * version, longer than Intern stores, or of another shape - is ignored and
 * removed; within one that is, a document's entry that is not as Intern
 * writes it is passed over, and so is a second entry for one document.
 */
export function storedTyped(storage: Storage | undefined): StoredDocument[] {
  let raw: string | null;
  try { raw = storage?.getItem(TYPED_KEY) ?? null; } catch { return []; }
  if (raw === null) return [];
  let value: unknown;
  try { value = raw.length > MAX_STORED ? undefined : JSON.parse(raw); } catch { value = undefined; }
  const documents = typeof value === 'object' && value !== null && (value as { version?: unknown }).version === 1 ? (value as { documents?: unknown }).documents : undefined;
  if (!Array.isArray(documents)) {
    try { storage?.removeItem(TYPED_KEY); } catch { /* It could not be removed; it is still not read. */ }
    return [];
  }
  const seen = new Set<string>();
  return documents.flatMap((entry) => {
    const stored = storedDocument(entry);
    if (!stored || seen.has(stored.id)) return [];
    seen.add(stored.id);
    return [stored];
  });
}

/** `map`, saying which key changed whenever one is set or taken out. */
function told<Value>(map: Map<string, Value>, changed: (id?: string) => void): Map<string, Value> {
  const set = map.set.bind(map);
  const remove = map.delete.bind(map);
  const clear = map.clear.bind(map);
  map.set = (id, value) => { set(id, value); changed(id); return map; };
  map.delete = (id) => { const had = remove(id); if (had) changed(id); return had; };
  map.clear = () => { clear(); changed(); };
  return map;
}

/** What was typed in review and not filed, kept through closing Intern. */
export interface TypedBetweenLaunches {
  /** What is in the fields (`ReviewDrafts`). */
  drafts: ReviewDrafts;
  /** What was set aside when its document changed (`EarlierDrafts`). */
  earlier: EarlierDrafts;
  /**
   * The queue, each time the window is about to show it: `read` once it has
   * been read from the backend, not before. What was stored is put back at
   * the first queue that was read.
   */
  queueIs(items: readonly QueueItem[], read: boolean): void;
  /** The window is going in the ordinary way (`pagehide`): what is stored is marked as left by an ordinary close. */
  closing(): void;
}

/**
 * The window's drafts and set-aside texts, stored as they change and put
 * back when Intern next starts.
 *
 * Stored at each change, in the change itself: a letter typed is in the
 * window's storage before the next can be, so nothing waits to be written
 * when the window closes, and a document that is decided, read again at
 * the person's word, or whose typing is discarded or taken back, is out of
 * storage as it is out of the window. Only what a person typed is stored:
 * a text kept only to hold the caret (`edited` false) is Intern's proposal,
 * and is not.
 *
 * When the window's storage reaches the disk is the browser engine's own
 * business, and a page cannot hurry it. Measured in Chromium, typing a few
 * words into a name: the window closed in the ordinary way straight after
 * the last letter, all of it was there at the next start; every process of
 * the browser killed 2 s after the last letter, none of it was, and 8 s
 * after, all of it; a letter every half second for 40 s and then killed,
 * the disk had the first 5 s of them. A name taken back with Back to
 * Intern's proposal just before such a kill was in storage again at the
 * next start. So after a kill or a power cut what comes back can be half
 * a minute old or more: its last letters missing, something since dropped
 * back. The desktop window's engine (WebView2) has not been run for this.
 *
 * So what comes back goes into the fields only after an ordinary close.
 * As the window goes (`closing`, on `pagehide`) what is stored is marked
 * (`CLOSED_KEY`); the next window takes the mark out before anything else
 * and, where it found none, sets aside everything it puts back: read-only
 * beside the document, where no key and no press files it. A kill and a
 * power cut leave no mark. Measured in Chromium with a profile on disk,
 * one run each: the window closed straight after the last letter, the name
 * was in its field at the next start; typed, left 8 s, taken back and
 * killed at once - the case above - it was set aside, the field Intern's
 * proposal; the same typed, left 8 s and filed, or just left 8 s, before
 * the kill. The whole browser closed by the test driver, not the window by
 * itself, left no mark (nothing a page wrote on `pagehide`, `unload` or
 * `visibilitychange` was there afterwards, 2 of 2), and the name was set
 * aside.
 *
 * The desktop window has not been run. Two of its ways out leave no mark:
 * Quit Intern in the notification area and the restart after an update end
 * the process with no word to the page - and where Intern stays in the
 * background, closing the window only hides it, so Quit is the one way
 * out. Whether its window sends `pagehide` when it does close in the
 * ordinary way has not been observed; where it does not, what was typed
 * always comes back set aside, to be copied into its field by hand.
 *
 * One case is left in which an older copy comes back in the fields. A
 * window that started after an ordinary close and is killed before its
 * first change to the storage has reached the disk - the mark's removal is
 * that first change - leaves the disk as the window before it did, mark
 * and all, whatever was taken back in those seconds. Measured as above: a
 * name taken back 1.7, 3.1 and 5.0 s after such a start, and the browser
 * killed at once, was in its field at the start after; taken back 9.0 and
 * 21 s after, it was set aside.
 *
 * Put back once, when the queue has first been read - never before, when an
 * empty list would say every document had gone, and nothing is written
 * until then, so a queue that cannot be read leaves what is stored, and its
 * mark, for the next launch. Each document's typing goes back to where the
 * window would have it now. The document is in the queue under the reading
 * it was typed over, can be filed, and the window before closed in the
 * ordinary way: in the fields. It is there with another reading
 * (`readingOf`), or cannot be filed as it stands, or there is no mark: set
 * aside, after whatever was set aside for it before, read-only beside the
 * document as it now is. The document is not in the queue, or its number is
 * another document's (`documentOf`): dropped. Nothing takes the focus for
 * any of it.
 *
 * Storage that cannot be read or written changes nothing in the window: it
 * keeps what is typed while it is open, as it did before. A write that
 * fails takes the stored value out instead, so that what a person has since
 * dropped cannot come back from an older copy.
 */
export function keepTypedBetweenLaunches(storage: () => Storage | undefined, onChange: () => void): TypedBetweenLaunches {
  let items: readonly QueueItem[] = [];
  let restored = false;
  let restoring = false;
  // What storage holds, as far as this window knows: a value, `null` for none, `undefined` for not known.
  let written: string | null | undefined;
  // Whether this window has marked what is stored as left by an ordinary close (`closing`).
  let marked = false;
  // The reading each draft was typed over, found while the queue showed it, and the document each number means.
  const readings = new Map<string, { for: string; reading: string }>();
  const documents = new Map<string, string>();
  // When each document's typing last changed, for storing the newest first.
  const changedAt = new Map<string, number>();
  let changes = 0;

  const entries = (): StoredDocument[] => {
    const listed = new Map(items.map((item) => [item.id, item]));
    const stored = new Map<string, StoredDocument>();
    const entry = (id: string): StoredDocument | undefined => {
      const item = listed.get(id);
      if (item) documents.set(id, documentOf(item));
      const document = documents.get(id);
      if (document === undefined) return undefined;
      if (!stored.has(id)) stored.set(id, { id, document });
      return stored.get(id);
    };
    for (const [id, typed] of drafts) {
      // The reading as it was when the draft was first seen over it: one
      // made again that says the same is not the one typed over.
      const item = readings.get(id)?.for === typed.for ? undefined : listed.get(id);
      if (item && proposalOf(item) === typed.for) readings.set(id, { for: typed.for, reading: readingOf(item) });
      if (!typed.edited || typed.stem.length > MAX_NAME || typed.description.length > MAX_DESCRIPTION) continue;
      const over = readings.get(id);
      const kept = entry(id);
      // Typed over a reading the queue never showed this window: stored as
      // typed over none, which no reading is, so it comes back set aside.
      if (kept) kept.draft = { reading: over?.for === typed.for ? over.reading : '', stem: typed.stem, description: typed.description };
    }
    for (const [id, kept] of earlier) {
      const texts = kept.filter((text) => text.edited && text.text !== '' && text.text.length <= MAX_SET_ASIDE).map((text) => text.text);
      const under = texts.length > 0 ? entry(id) : undefined;
      if (under) under.earlier = texts;
    }
    return [...stored.values()];
  };

  /** The value to store, or `null` for none: the newest typing first into what room there is, written oldest first. */
  const value = (): string | null => {
    const newestFirst = entries().sort((left, right) => (changedAt.get(right.id) ?? 0) - (changedAt.get(left.id) ?? 0));
    const fitting: string[] = [];
    let length = '{"version":1,"documents":[]}'.length;
    for (const stored of newestFirst) {
      const text = JSON.stringify(stored);
      if (length + text.length + 1 > MAX_STORED) continue;
      length += text.length + 1;
      fitting.push(text);
    }
    return fitting.length > 0 ? `{"version":1,"documents":[${fitting.reverse().join(',')}]}` : null;
  };

  const save = () => {
    if (!restored || restoring) return;
    const next = value();
    if (next === written) return;
    let store: Storage | undefined;
    try {
      store = storage();
      // The window is still here after all (a page kept and shown again): what it stores from now on is not what an ordinary close left.
      if (marked) { store?.removeItem(CLOSED_KEY); marked = false; }
      if (next === null) store?.removeItem(TYPED_KEY); else store?.setItem(TYPED_KEY, next);
      written = next;
    } catch {
      // Not stored: no room, or storage refuses. An older value left there
      // would bring back, at the next launch, what has since been dropped.
      try { store?.removeItem(TYPED_KEY); written = null; } catch { written = undefined; }
    }
  };
  const changed = (id?: string) => {
    if (id !== undefined) changedAt.set(id, changes += 1);
    save();
  };
  const drafts = told(watchedDrafts(onChange), (id) => {
    // A draft that is gone takes with it the reading it was typed over: what
    // is typed for the document next is typed over the reading it has then.
    if (id === undefined) readings.clear(); else if (!drafts.has(id)) readings.delete(id);
    changed(id);
  });
  const earlier: EarlierDrafts = told(new Map(), changed);

  /** Whether the window before this one marked what it stored as it closed. The mark is taken out: it was that window's. */
  const closedOrdinarily = (): boolean => {
    try {
      const store = storage();
      const mark = store?.getItem(CLOSED_KEY) ?? null;
      if (mark !== null) store?.removeItem(CLOSED_KEY);
      return mark !== null;
    } catch {
      // A mark that cannot be read, or cannot be taken out, is no mark.
      return false;
    }
  };

  const restore = () => {
    restored = true;
    restoring = true;
    try {
      const ordinary = closedOrdinarily();
      const listed = new Map(items.map((item) => [item.id, item]));
      for (const stored of storedTyped(storage())) {
        const item = listed.get(stored.id);
        // Gone from the queue, or another document under the number; or typed for in this window already.
        if (!item || documentOf(item) !== stored.document || drafts.has(stored.id) || earlier.has(stored.id)) continue;
        const aside = (stored.earlier ?? []).map((text) => keptText(text, true));
        if (stored.draft) {
          const { reading, stem, description } = stored.draft;
          if (ordinary && reading === readingOf(item) && itemActions(item).approve) drafts.set(item.id, { item: item.id, for: proposalOf(item), stem, description, edited: true });
          else aside.push(keptText(nameThenDescription(stem, description), true));
        }
        if (aside.length > 0) earlier.set(item.id, aside);
      }
    } catch {
      // Storage that cannot be had holds nothing to put back.
    } finally {
      restoring = false;
    }
    // Storage now holds what was put back and nothing else.
    written = undefined;
    save();
  };

  return {
    drafts,
    earlier,
    queueIs(list, read) {
      items = list;
      if (read && !restored) restore();
    },
    closing() {
      // What storage would not take as it was typed is tried once more.
      save();
      // Only what this window wrote whole is marked. Nothing where nothing
      // is stored; and nothing before the queue was read, when this window
      // has written nothing and taken no mark out: what is there then is as
      // the window before left it, with its mark or without.
      if (typeof written !== 'string') return;
      try { storage()?.setItem(CLOSED_KEY, '1'); marked = true; } catch { /* Not marked: the next window sets aside what it finds. */ }
    },
  };
}
