import { describe, expect, it, vi } from 'vitest';
import type { QueueItem } from '../../types';
import { keptText, proposalOf, typedAndNotFiled } from './drafts';
import { MAX_DESCRIPTION, MAX_NAME, MAX_STORED, TYPED_KEY, documentOf, keepTypedBetweenLaunches, readingOf, storedTyped, windowStorage } from './typedStore';

/*
  A name or a description typed in review and not filed was the window's
  alone: closing Intern, a crash, or the restart an update makes threw all of
  it away, with no notice. It is stored as it changes, in the window's own
  storage, and put back when the queue is first read at the next launch: in
  the fields over the reading it was typed over, set aside beside any other,
  dropped with a document that is no longer in the queue.

  Here the storage is one handed in, and a launch is a new keeper told the
  queue it starts with.
*/
describe('what was typed in review and not filed, through closing Intern', () => {
  const lease: QueueItem = { id: '7', originalFilename: 'scan.pdf', status: 'review', proposedFilename: '2024-03-01 Lease.pdf', description: 'A lease.', confidence: 0.8, proposalRevision: '1', readBy: { tier: 'accurate', modelId: 'accurate-a', displayName: 'Accurate' } };
  const invoice: QueueItem = { id: '8', originalFilename: 'scan 2.pdf', status: 'ready', proposedFilename: '2024-04-01 Invoice.pdf', description: 'An invoice.', confidence: 0.9, proposalRevision: '1' };
  const unreadable: QueueItem = { id: '9', originalFilename: 'fax.pdf', status: 'review', errorCode: 'EXTRACTION_FAILED', reason: 'Intern could not read it.' };
  /** What `documentOf` and `readingOf` give for the lease and for the unreadable document above. */
  const DIGESTS = ['c3bd92c21e2e0c25', 'ed2ee7de81d24cc9', '1a9c63b8880aef05', 'e06ae1af2c56fc8c'];
  const NAME = '2024-03-01 Lease of Unit 4';
  const SENTENCE = 'A lease of Unit 4, countersigned.';

  /** Storage as a window has it, holding what it is given. */
  const storageOf = (held: Record<string, string> = {}) => {
    const values = new Map(Object.entries(held));
    const storage = {
      get length() { return values.size; },
      key: (index: number) => [...values.keys()][index] ?? null,
      getItem: vi.fn((key: string): string | null => values.get(key) ?? null),
      setItem: vi.fn((key: string, value: string) => { values.set(key, String(value)); }),
      removeItem: vi.fn((key: string) => { values.delete(key); }),
      clear: () => values.clear(),
    };
    return storage as Storage & typeof storage;
  };
  /** Intern starting: a window of its own, told the queue once it has been read. */
  const launch = (storage: Storage | undefined, queue: readonly QueueItem[]) => {
    const kept = keepTypedBetweenLaunches(() => storage, () => {});
    kept.queueIs(queue, true);
    return kept;
  };
  const typedOver = (item: QueueItem, stem = NAME, description = SENTENCE) => ({ item: item.id, for: proposalOf(item), stem, description, edited: true });
  const stored = (storage: Storage) => storage.getItem(TYPED_KEY);
  /** What a launch with this queue finds typed for the lease: in the fields, and set aside. */
  const found = (storage: Storage, queue: readonly QueueItem[], id = lease.id) => {
    const kept = launch(storage, queue);
    return { fields: kept.drafts.get(id), aside: (kept.earlier.get(id) ?? []).map(({ text, edited }) => ({ text, edited })), kept };
  };

  it('is in the fields again when Intern starts with the document as it was typed over', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    first.drafts.set(lease.id, typedOver(lease));

    const { fields, aside, kept } = found(storage, [lease, invoice]);
    expect(fields).toEqual({ item: lease.id, for: proposalOf(lease), stem: NAME, description: SENTENCE, edited: true });
    expect(aside).toEqual([]);
    // It counts as typed and not filed, as one typed in this window does; the other document does not.
    expect(typedAndNotFiled(kept.drafts, kept.earlier, lease.id)).toBe(true);
    expect(typedAndNotFiled(kept.drafts, kept.earlier, invoice.id)).toBe(false);
    expect(kept.drafts.size).toBe(1);
    // And is still there at the launch after that, with nothing typed in between.
    expect(found(storage, [lease, invoice]).fields).toMatchObject({ stem: NAME, description: SENTENCE });
  });

  it('is stored in the change itself, so closing the window straight after a letter loses nothing', () => {
    const storage = storageOf();
    const first = launch(storage, [lease]);
    for (const stem of ['2', '20', '2024-03-01 Lease of Unit 4']) {
      first.drafts.set(lease.id, typedOver(lease, stem));
      // No timer has run and nothing was flushed: it is there already.
      expect(found(storage, [lease]).fields).toMatchObject({ stem });
    }
    // The same text set again writes nothing.
    const writes = storage.setItem.mock.calls.length;
    first.drafts.set(lease.id, typedOver(lease));
    expect(storage.setItem).toHaveBeenCalledTimes(writes);
  });

  it('is set aside, not in the fields, when the document starts with another reading', () => {
    const another: Array<[string, QueueItem]> = [
      ['another name', { ...lease, proposedFilename: '2024-03-02 Lease Agreement.pdf' }],
      ['another sentence', { ...lease, description: 'A lease of the ground floor.' }],
      ['another revision of the proposal', { ...lease, proposalRevision: '2' }],
      ['the same words from another model', { ...lease, readBy: { tier: 'fast', modelId: 'fast-b', displayName: 'Fast' } }],
      ['being read again', { id: lease.id, originalFilename: lease.originalFilename, status: 'processing', stage: 'reading' }],
      ['waiting to be read again', { id: lease.id, originalFilename: lease.originalFilename, status: 'waiting' }],
      ['filed from somewhere else', { ...lease, status: 'completed', filedName: lease.proposedFilename }],
    ];
    for (const [how, now] of another) {
      const storage = storageOf();
      launch(storage, [lease]).drafts.set(lease.id, typedOver(lease));
      const { fields, aside, kept } = found(storage, [now]);
      expect(fields, how).toBeUndefined();
      expect(aside, how).toEqual([{ text: `${NAME}\n\n${SENTENCE}`, edited: true }]);
      expect(typedAndNotFiled(kept.drafts, kept.earlier, lease.id), how).toBe(true);
      // Set aside is how it stays: put back under the first reading at a later launch, it is still not in the fields.
      expect(found(storage, [lease]), how).toMatchObject({ fields: undefined, aside: [{ text: `${NAME}\n\n${SENTENCE}` }] });
    }
  });

  it('is still in the fields when only what else the queue says of the reading has moved', () => {
    // Validated again after an update, a reading can change status, reasons and confidence and keep its name and sentence.
    for (const now of [{ ...lease, status: 'ready' as const }, { ...lease, confidence: 0.31 }, { ...lease, reason: 'Check the date.', reasonCodes: ['DATE_AMBIGUOUS'] }, { ...lease, readBy: { ...lease.readBy!, displayName: 'Accurate (new name)', provisional: true } }]) {
      const storage = storageOf();
      launch(storage, [lease]).drafts.set(lease.id, typedOver(lease));
      expect(found(storage, [now])).toMatchObject({ fields: { stem: NAME, description: SENTENCE, for: proposalOf(now) }, aside: [] });
    }
  });

  it('is the reading it was first typed over, not one made again since that says the same', () => {
    const storage = storageOf();
    const first = launch(storage, [lease]);
    first.drafts.set(lease.id, typedOver(lease, '2024-03-01 Lease of'));
    // Read again by another model to the same name and sentence, between two reads of the queue: the window cannot tell, and typing goes on.
    const again = { ...lease, readBy: { tier: 'fast' as const, modelId: 'fast-b', displayName: 'Fast' } };
    first.queueIs([again], true);
    first.drafts.set(lease.id, typedOver(lease));
    expect(found(storage, [again])).toMatchObject({ fields: undefined, aside: [{ text: `${NAME}\n\n${SENTENCE}` }] });
  });

  it('comes back set aside when the queue never showed the reading it was typed over', () => {
    const storage = storageOf();
    const first = launch(storage, [lease]);
    // Over a proposal this window was never told of: which reading that is cannot be said, so it is no reading's.
    const unseen = { ...lease, proposalRevision: '5' };
    first.drafts.set(lease.id, typedOver(unseen));
    expect(first.drafts.get(lease.id)).toMatchObject({ stem: NAME });
    for (const now of [lease, unseen]) expect(found(storage, [now])).toMatchObject({ fields: undefined, aside: [{ text: `${NAME}\n\n${SENTENCE}` }] });
    // The same after a draft over a reading it was told of: the next one's reading is not taken for the last one's.
    const again = storageOf();
    const second = launch(again, [lease]);
    second.drafts.set(lease.id, typedOver(lease));
    second.drafts.set(lease.id, typedOver(unseen));
    expect(found(again, [lease])).toMatchObject({ fields: undefined, aside: [{ text: `${NAME}\n\n${SENTENCE}` }] });
    // For a document the queue never listed there is nothing to tell it by at the next start: it stays in the window only.
    const nowhere = storageOf();
    const third = launch(nowhere, [lease]);
    expect(() => third.drafts.set('nobody', { ...typedOver(lease), item: 'nobody' })).not.toThrow();
    third.earlier.set('nobody', [keptText('typed', true)]);
    expect(third.drafts.get('nobody')).toMatchObject({ stem: NAME });
    expect(stored(nowhere)).toBeNull();
  });

  it('is told by the document as it was last listed, when it is not in the queue as its typing is stored', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    first.drafts.set(lease.id, typedOver(lease, 'one'));
    // A list the Lease is not in - one that could not be read whole is not a document gone - and typing goes on.
    first.queueIs([invoice], true);
    first.drafts.set(lease.id, typedOver(lease));
    first.earlier.set(lease.id, [keptText('before', true)]);
    expect(found(storage, [lease, invoice])).toMatchObject({ fields: { stem: NAME }, aside: [{ text: 'before' }] });
  });

  it('is dropped when its document is not in the queue as Intern starts, and goes from storage', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    first.drafts.set(lease.id, typedOver(lease));
    first.earlier.set(invoice.id, [keptText('2024-04-01 Invoice 17', true)]);

    const { kept } = found(storage, [invoice]);
    expect(kept.drafts.size).toBe(0);
    expect([...kept.earlier.keys()]).toEqual([invoice.id]);
    // Not kept in storage for a document that might come back: at a later launch with the lease, there is nothing for it.
    expect(stored(storage)).not.toContain(NAME);
    expect(found(storage, [lease, invoice])).toMatchObject({ fields: undefined, aside: [] });
    // An empty queue that was read drops the rest, and nothing is left under the key.
    launch(storage, []);
    expect(stored(storage)).toBeNull();
  });

  it('is not shown for another document that has its number', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, unreadable]);
    first.drafts.set(lease.id, typedOver(lease));
    first.earlier.set(lease.id, [keptText('2024-03-01 Lease, first try', true)]);
    first.drafts.set(unreadable.id, typedOver(unreadable, '2024-05-05 Fax from the bank', ''));

    // Another queue, whose numbers begin again: the same numbers, other files. Even the same proposal under another file's name.
    const others = [{ ...lease, originalFilename: 'other scan.pdf' }, { ...unreadable, originalFilename: 'letter.pdf' }];
    const { kept } = found(storage, others);
    expect(kept.drafts.size).toBe(0);
    expect(kept.earlier.size).toBe(0);
    expect(stored(storage)).toBeNull();
  });

  it('is the same document under the NEEDS REVIEW name Intern gives one it could not read', () => {
    for (const marked of ['NEEDS REVIEW - fax.pdf', 'NEEDS REVIEW - fax (2).pdf', 'needs review - fax.pdf']) {
      const storage = storageOf();
      launch(storage, [unreadable]).drafts.set(unreadable.id, typedOver(unreadable, '2024-05-05 Fax from the bank', ''));
      const now = { ...unreadable, originalFilename: marked };
      expect(found(storage, [now], unreadable.id), marked).toMatchObject({ fields: { stem: '2024-05-05 Fax from the bank', for: proposalOf(now) }, aside: [] });
    }
    // And back again, when the mark is taken off.
    const storage = storageOf();
    const marked = { ...unreadable, originalFilename: 'NEEDS REVIEW - fax.pdf' };
    launch(storage, [marked]).drafts.set(marked.id, typedOver(marked, '2024-05-05 Fax from the bank', ''));
    expect(found(storage, [unreadable], unreadable.id).fields).toMatchObject({ stem: '2024-05-05 Fax from the bank' });
  });

  it('comes back set aside when it was set aside, oldest first, ahead of what is set aside as Intern starts', () => {
    const storage = storageOf();
    const first = launch(storage, [lease]);
    first.earlier.set(lease.id, [keptText('first', true), keptText('second', true)]);
    first.drafts.set(lease.id, typedOver(lease));

    // The reading it starts with is the one typed over: the draft is in the fields, the rest beside it.
    expect(found(storage, [lease])).toMatchObject({ fields: { stem: NAME }, aside: [{ text: 'first', edited: true }, { text: 'second', edited: true }] });
    // Another reading: all three set aside, in the order they were typed.
    const { aside, kept } = found(storage, [{ ...lease, proposalRevision: '2' }]);
    expect(aside.map(({ text }) => text)).toEqual(['first', 'second', `${NAME}\n\n${SENTENCE}`]);
    expect(new Set(kept.earlier.get(lease.id)!.map((text) => text.key)).size).toBe(3);
  });

  it('is not put back, and storage is left as it is, until the queue has been read', () => {
    const storage = storageOf();
    launch(storage, [lease]).drafts.set(lease.id, typedOver(lease));
    const before = stored(storage);
    storage.getItem.mockClear();
    storage.setItem.mockClear();
    storage.removeItem.mockClear();

    // A window that has not read its queue yet: the list is empty, and that is not every document gone.
    const kept = keepTypedBetweenLaunches(() => storage, () => {});
    kept.queueIs([], false);
    kept.queueIs([], false);
    expect(kept.drafts.size).toBe(0);
    expect(storage.getItem).not.toHaveBeenCalled();
    expect(storage.setItem).not.toHaveBeenCalled();
    expect(storage.removeItem).not.toHaveBeenCalled();
    // A window closed before its queue could be read has dropped nothing.
    expect(stored(storage)).toBe(before);
    kept.queueIs([lease], true);
    expect(kept.drafts.get(lease.id)).toMatchObject({ stem: NAME, description: SENTENCE });
    // Once: a queue read again later puts nothing back that has since been dropped, and storage is not read for it again.
    kept.drafts.delete(lease.id);
    storage.getItem.mockClear();
    kept.queueIs([lease], true);
    kept.queueIs([], true);
    expect(kept.drafts.size).toBe(0);
    expect(storage.getItem).not.toHaveBeenCalled();
  });

  it('is put back whole before anything is written, never a part of it in between', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice, unreadable]);
    first.drafts.set(lease.id, typedOver(lease));
    first.drafts.set(invoice.id, typedOver(invoice, '2024-04-01 Invoice 17', ''));
    first.earlier.set(unreadable.id, [keptText('2024-05-05 Fax', true)]);
    storage.setItem.mockClear();
    storage.removeItem.mockClear();

    launch(storage, [lease, invoice, unreadable]);
    expect(storage.removeItem).not.toHaveBeenCalled();
    for (const [, value] of storage.setItem.mock.calls) {
      for (const text of [NAME, '2024-04-01 Invoice 17', '2024-05-05 Fax']) expect(value).toContain(text);
    }
  });

  it('keeps what is typed in a window over what was stored for the same document, and writes nothing before the queue is read', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    first.drafts.set(lease.id, typedOver(lease, 'stored'));
    first.drafts.set(invoice.id, typedOver(invoice, '2024-04-01 Invoice 17', ''));
    const before = stored(storage);
    const kept = keepTypedBetweenLaunches(() => storage, () => {});
    kept.queueIs([], false);
    kept.drafts.set(lease.id, typedOver(lease, 'typed since'));
    // Not written yet: what is stored for the other document would go with it.
    expect(stored(storage)).toBe(before);
    kept.queueIs([lease, invoice], true);
    expect(kept.drafts.get(lease.id)).toMatchObject({ stem: 'typed since' });
    expect(kept.drafts.get(invoice.id)).toMatchObject({ stem: '2024-04-01 Invoice 17' });
    expect(kept.earlier.size).toBe(0);
    expect(found(storage, [lease, invoice]).fields).toMatchObject({ stem: 'typed since' });
  });

  it('goes from storage at once with each way a person drops it', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    first.drafts.set(lease.id, typedOver(lease));
    first.drafts.set(invoice.id, typedOver(invoice, '2024-04-01 Invoice 17', 'For March.'));
    first.earlier.set(invoice.id, [keptText('older', true), keptText('old', true)]);

    // The document is decided, or read again at the person's word, or its typing taken back: its draft is deleted.
    first.drafts.delete(lease.id);
    expect(stored(storage)).not.toContain(NAME);
    expect(found(storage, [lease, invoice])).toMatchObject({ fields: undefined, aside: [] });
    // One set-aside text discarded: the others stay.
    first.earlier.set(invoice.id, first.earlier.get(invoice.id)!.slice(1));
    expect(found(storage, [lease, invoice], invoice.id)).toMatchObject({ fields: { stem: '2024-04-01 Invoice 17' }, aside: [{ text: 'old' }] });
    // The last of them, and the draft: nothing is left under the key.
    first.earlier.delete(invoice.id);
    expect(stored(storage)).not.toContain('old');
    first.drafts.delete(invoice.id);
    expect(stored(storage)).toBeNull();
    expect(storage.length).toBe(0);
    // Set aside by the window when its document changes: out of the fields and in what is set aside, in storage too.
    first.drafts.set(lease.id, typedOver(lease));
    first.drafts.delete(lease.id);
    first.earlier.set(lease.id, [keptText(`${NAME}\n\n${SENTENCE}`, true)]);
    expect(found(storage, [lease])).toMatchObject({ fields: undefined, aside: [{ text: `${NAME}\n\n${SENTENCE}` }] });
    first.earlier.clear();
    expect(stored(storage)).toBeNull();
  });

  it('is not stored when nobody typed it', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    // The proposal as it stood, kept only to hold the caret when its field went; and a draft that no longer differs.
    first.earlier.set(lease.id, [keptText('2024-03-01 Lease\n\nA lease.', false)]);
    first.drafts.set(invoice.id, { ...typedOver(invoice, '2024-04-01 Invoice', 'An invoice.'), edited: false });
    expect(stored(storage)).toBeNull();
    // Beside one that was typed, only the typed one is stored.
    first.earlier.set(lease.id, [keptText('2024-03-01 Lease\n\nA lease.', false), keptText('typed', true)]);
    expect(found(storage, [lease, invoice]).aside).toEqual([{ text: 'typed', edited: true }]);
  });

  it('holds what was typed and nothing Intern read', () => {
    const storage = storageOf();
    const first = launch(storage, [lease]);
    first.drafts.set(lease.id, typedOver(lease, 'typed name', 'typed sentence'));
    first.earlier.set(lease.id, [keptText('typed before', true)]);
    const value = stored(storage)!;
    expect(JSON.parse(value)).toEqual({ version: 1, documents: [{ id: lease.id, document: documentOf(lease), draft: { reading: readingOf(lease), stem: 'typed name', description: 'typed sentence' }, earlier: ['typed before'] }] });
    for (const read of ['scan.pdf', '2024-03-01 Lease', 'A lease.', 'accurate-a']) expect(value).not.toContain(read);
    expect(documentOf(lease)).toMatch(/^[0-9a-f]{16}$/);
    expect(readingOf(lease)).toMatch(/^[0-9a-f]{16}$/);
    // Two sums, each of them over the whole text: the halves differ from each other, and each differs for another text.
    const [one, other] = [documentOf(lease), documentOf({ ...lease, id: '8' })];
    expect(one.slice(0, 8)).not.toBe(one.slice(8));
    expect(one.slice(0, 8)).not.toBe(other.slice(0, 8));
    expect(one.slice(8)).not.toBe(other.slice(8));
    // The digits are what is stored, so they stay as they are from one release to the next:
    // changed, every name typed before an update would come back set aside, or be dropped.
    expect([documentOf(lease), readingOf(lease), documentOf(unreadable), readingOf(unreadable)]).toEqual(DIGESTS);
  });

  it('tells a document by its number and file name, and a reading by those, its proposal and its model', () => {
    const readings = [lease, { ...lease, id: '70' }, { ...lease, originalFilename: 'scan 3.pdf' }, { ...lease, proposalRevision: '2' }, { ...lease, proposedFilename: '2024-03-01 Lease .pdf' }, { ...lease, description: 'A lease' }, { ...lease, readBy: { ...lease.readBy!, modelId: 'accurate-b' } }, { ...lease, readBy: undefined }, { ...lease, proposedFilename: undefined, description: undefined }].map(readingOf);
    expect(new Set(readings).size).toBe(readings.length);
    expect(readingOf({ ...lease, status: 'ready', confidence: 0.1, reason: 'Check it.', evidence: { date: '1 March' } })).toBe(readingOf(lease));
    const documents = [lease, { ...lease, id: '70' }, { ...lease, originalFilename: 'scan 3.pdf' }, { ...lease, originalFilename: 'scan.docx' }].map(documentOf);
    expect(new Set(documents).size).toBe(documents.length);
    expect(documentOf({ ...lease, proposalRevision: '2', proposedFilename: 'Other.pdf', description: 'Other.', status: 'completed' })).toBe(documentOf(lease));
    // The mark Intern puts on a file it could not read, and the number that mark can take, are not another file.
    expect(documentOf({ ...lease, originalFilename: 'NEEDS REVIEW - scan (2).pdf' })).toBe(documentOf(lease));
    expect(documentOf({ ...lease, originalFilename: 'NEEDS REVIEW - scan' })).toBe(documentOf({ ...lease, originalFilename: 'scan (3)' }));
    expect(documentOf({ ...lease, originalFilename: 'scan (2) final.pdf' })).not.toBe(documentOf({ ...lease, originalFilename: 'scan final.pdf' }));
  });

  it('is ignored and removed when what is stored is not what Intern writes', () => {
    const entry = { id: lease.id, document: documentOf(lease), draft: { reading: readingOf(lease), stem: NAME, description: SENTENCE } };
    const foreign = [
      'not json', '', 'null', '7', '"text"', '[]', '{}',
      JSON.stringify({ version: 2, documents: [entry] }),
      JSON.stringify({ version: '1', documents: [entry] }),
      JSON.stringify({ version: 1, documents: { 0: entry } }),
      JSON.stringify({ version: 1 }),
      // Longer than Intern ever stores, though well formed.
      JSON.stringify({ version: 1, documents: [entry], padding: 'x'.repeat(MAX_STORED) }),
    ];
    for (const value of foreign) {
      const storage = storageOf({ [TYPED_KEY]: value, 'intern.detailsPanel': 'open' });
      expect(storedTyped(storage), value.slice(0, 40)).toEqual([]);
      expect(stored(storage), value.slice(0, 40)).toBeNull();
      // Another launch on it: nothing put back, nothing thrown, and the window's other values left alone.
      const again = storageOf({ [TYPED_KEY]: value, 'intern.detailsPanel': 'open' });
      const kept = launch(again, [lease]);
      expect(kept.drafts.size + kept.earlier.size, value.slice(0, 40)).toBe(0);
      expect(again.getItem('intern.detailsPanel')).toBe('open');
    }
    // The same entry as Intern writes it is read.
    expect(found(storageOf({ [TYPED_KEY]: JSON.stringify({ version: 1, documents: [entry] }) }), [lease]).fields).toMatchObject({ stem: NAME });
  });

  it('passes over an entry that is not as Intern writes it, and keeps the others', () => {
    const good = { id: invoice.id, document: documentOf(invoice), earlier: ['2024-04-01 Invoice 17'] };
    const draft = { reading: readingOf(lease), stem: NAME, description: SENTENCE };
    const bad: unknown[] = [
      null, 7, 'text', [],
      { document: documentOf(lease), draft },
      { id: 7, document: documentOf(lease), draft },
      { id: '', document: documentOf(lease), draft },
      { id: lease.id, draft },
      { id: lease.id, document: documentOf(lease) },
      { id: lease.id, document: documentOf(lease), draft: 'text' },
      { id: lease.id, document: documentOf(lease), draft: null },
      { id: lease.id, document: documentOf(lease), draft: { ...draft, reading: 7 } },
      { id: lease.id, document: documentOf(lease), draft: { ...draft, stem: undefined } },
      { id: lease.id, document: documentOf(lease), draft: { ...draft, description: ['a'] } },
      { id: lease.id, document: documentOf(lease), draft: { ...draft, stem: 'two\nlines' } },
      { id: lease.id, document: documentOf(lease), draft: { ...draft, stem: 'x'.repeat(MAX_NAME + 1) } },
      { id: lease.id, document: documentOf(lease), draft: { ...draft, description: 'x'.repeat(MAX_DESCRIPTION + 1) } },
      { id: lease.id, document: documentOf(lease), earlier: 'text' },
      { id: lease.id, document: documentOf(lease), earlier: [] },
      { id: lease.id, document: documentOf(lease), earlier: ['kept', 7] },
      { id: lease.id, document: documentOf(lease), earlier: [''] },
      { id: lease.id, document: documentOf(lease), earlier: ['x'.repeat(MAX_NAME + MAX_DESCRIPTION + 3)] },
    ];
    for (const entry of bad) {
      const storage = storageOf({ [TYPED_KEY]: JSON.stringify({ version: 1, documents: [entry, good] }) });
      const kept = launch(storage, [lease, invoice]);
      expect(kept.drafts.size, JSON.stringify(entry).slice(0, 60)).toBe(0);
      expect([...kept.earlier.keys()], JSON.stringify(entry).slice(0, 60)).toEqual([invoice.id]);
      // What was passed over is not left in storage either.
      expect(JSON.parse(stored(storage)!).documents).toEqual([good]);
      expect(storedTyped(storageOf({ [TYPED_KEY]: JSON.stringify({ version: 1, documents: [good, entry] }) })), JSON.stringify(entry).slice(0, 60)).toEqual([good]);
    }
    // Two entries for one document: the first is the document's.
    const twice = storageOf({ [TYPED_KEY]: JSON.stringify({ version: 1, documents: [{ id: lease.id, document: documentOf(lease), draft }, { id: lease.id, document: documentOf(lease), draft: { ...draft, stem: 'second' } }] }) });
    expect(storedTyped(twice).map((entry) => entry.draft?.stem)).toEqual([NAME]);
    expect(found(twice, [lease])).toMatchObject({ fields: { stem: NAME }, aside: [] });
  });

  it('is not stored past what is stored of a name and a description, and an older copy of it does not stay', () => {
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    first.drafts.set(invoice.id, typedOver(invoice, '2024-04-01 Invoice 17', ''));
    first.drafts.set(lease.id, typedOver(lease, 'x'.repeat(MAX_NAME), 'y'.repeat(MAX_DESCRIPTION)));
    expect(found(storage, [lease, invoice]).fields).toMatchObject({ stem: 'x'.repeat(MAX_NAME), description: 'y'.repeat(MAX_DESCRIPTION) });

    // One character more in either: in the window still, and not in storage - not even as it was a letter ago.
    for (const [stem, description] of [['x'.repeat(MAX_NAME + 1), SENTENCE], [NAME, 'y'.repeat(MAX_DESCRIPTION + 1)]]) {
      first.drafts.set(lease.id, typedOver(lease, 'short', 'short'));
      first.drafts.set(lease.id, typedOver(lease, stem, description));
      expect(first.drafts.get(lease.id)).toMatchObject({ stem, description });
      expect(stored(storage)).not.toContain('x'.repeat(MAX_NAME + 1));
      expect(stored(storage)).not.toContain('y'.repeat(MAX_DESCRIPTION + 1));
      const next = found(storage, [lease, invoice]);
      expect(next).toMatchObject({ fields: undefined, aside: [] });
      // The other document's is stored as before.
      expect(next.kept.drafts.get(invoice.id)).toMatchObject({ stem: '2024-04-01 Invoice 17' });
    }
    // A set-aside text longer than a name and a description together is not stored either.
    first.earlier.set(invoice.id, [keptText('z'.repeat(MAX_NAME + MAX_DESCRIPTION + 3), true), keptText('kept', true)]);
    expect(stored(storage)).not.toContain('zzz');
    expect(found(storage, [lease, invoice], invoice.id).aside).toEqual([{ text: 'kept', edited: true }]);
  });

  it('leaves out what was typed longest ago when the whole would be too long', () => {
    const storage = storageOf();
    const queue: QueueItem[] = Array.from({ length: 45 }, (_, index) => ({ ...lease, id: String(100 + index), originalFilename: `scan ${index}.pdf` }));
    const first = launch(storage, [invoice, ...queue]);
    // A few words typed first of all, for another document.
    first.drafts.set(invoice.id, typedOver(invoice, '2024-04-01 Invoice 17', ''));
    // Then each as long as is stored: forty-five of them are more than the whole value holds.
    for (const item of queue) first.drafts.set(item.id, typedOver(item, `${item.id} `.padEnd(MAX_NAME, 'n'), 'd'.repeat(MAX_DESCRIPTION)));
    expect(stored(storage)!.length).toBeLessThanOrEqual(MAX_STORED);
    const back = launch(storage, [invoice, ...queue]);
    // The few words still fit, behind long ones that did not.
    expect(back.drafts.get(invoice.id)).toMatchObject({ stem: '2024-04-01 Invoice 17' });
    first.drafts.delete(invoice.id);
    const kept = [...back.drafts.keys()].filter((id) => id !== invoice.id);
    expect(kept.length).toBeGreaterThan(20);
    expect(kept.length).toBeLessThan(queue.length);
    // The last typed are the ones stored, in the order they were typed.
    expect(kept).toEqual(queue.slice(-kept.length).map((item) => item.id));
    // Typing for the first again makes it the newest: it is stored, and the oldest of the rest gives way.
    first.drafts.set(queue[0].id, typedOver(queue[0], `${queue[0].id} again`.padEnd(MAX_NAME, 'n'), 'd'.repeat(MAX_DESCRIPTION)));
    const after = [...launch(storage, queue).drafts.keys()];
    expect(after).toEqual([...kept.slice(1), queue[0].id]);
    // A short text typed for a document that had given way still fits.
    first.drafts.set(queue[1].id, typedOver(queue[1], 'short', ''));
    expect(launch(storage, queue).drafts.get(queue[1].id)).toMatchObject({ stem: 'short' });
    // All of it is still in the window.
    expect(first.drafts.size).toBe(queue.length);
  });

  it('changes nothing in the window when storage cannot be had, read or written', () => {
    const refusing = () => { throw new DOMException('The operation is insecure.', 'SecurityError'); };
    const full = () => { throw new DOMException('The quota has been exceeded.', 'QuotaExceededError'); };
    const typeAndDrop = (kept: ReturnType<typeof launch>) => {
      kept.drafts.set(lease.id, typedOver(lease));
      kept.earlier.set(invoice.id, [keptText('kept', true)]);
      expect(kept.drafts.get(lease.id)).toMatchObject({ stem: NAME, description: SENTENCE, edited: true });
      expect(typedAndNotFiled(kept.drafts, kept.earlier, invoice.id)).toBe(true);
      kept.drafts.delete(lease.id);
      kept.earlier.delete(invoice.id);
      expect(kept.drafts.size + kept.earlier.size).toBe(0);
      kept.drafts.set(lease.id, typedOver(lease));
    };

    // No storage at all, and a window that refuses even to hand it over.
    typeAndDrop(launch(undefined, [lease, invoice]));
    const refused = keepTypedBetweenLaunches(refusing, () => {});
    expect(() => refused.queueIs([lease, invoice], true)).not.toThrow();
    typeAndDrop(refused);

    // Storage that cannot be read: nothing is put back, and typing is stored from then on.
    const unread = storageOf({ [TYPED_KEY]: 'whatever is there' });
    unread.getItem.mockImplementation(refusing);
    expect(storedTyped(unread)).toEqual([]);
    const blind = launch(unread, [lease, invoice]);
    expect(blind.drafts.size).toBe(0);
    typeAndDrop(blind);

    // Storage that cannot be written (full, or a private window): the window keeps it all the same.
    const storage = storageOf();
    const first = launch(storage, [lease, invoice]);
    first.drafts.set(lease.id, typedOver(lease, 'before it filled'));
    storage.setItem.mockImplementation(full);
    first.drafts.set(lease.id, typedOver(lease, 'before it filled, and after'));
    // The copy from before the failure is gone at once: it is not what the window holds any more.
    expect(stored(storage)).toBeNull();
    expect(first.drafts.get(lease.id)).toMatchObject({ stem: 'before it filled, and after' });
    typeAndDrop(first);
    // And the copy from before the failure is gone, not left to come back in place of what was typed since.
    expect(stored(storage)).toBeNull();
    expect(found(storage, [lease, invoice])).toMatchObject({ fields: undefined, aside: [] });
    // Nothing can be removed either: still nothing thrown.
    storage.removeItem.mockImplementation(refusing);
    expect(() => typeAndDrop(first)).not.toThrow();
    expect(() => storedTyped(storageOf({ [TYPED_KEY]: 'not json' }))).not.toThrow();
    const stuck = storageOf({ [TYPED_KEY]: 'not json' });
    stuck.removeItem.mockImplementation(refusing);
    expect(storedTyped(stuck)).toEqual([]);
  });

  it('is stored again once storage takes it', () => {
    const storage = storageOf();
    const first = launch(storage, [lease]);
    const write = storage.setItem.getMockImplementation()!;
    storage.setItem.mockImplementation(() => { throw new DOMException('The quota has been exceeded.', 'QuotaExceededError'); });
    first.drafts.set(lease.id, typedOver(lease, 'while it was full'));
    expect(stored(storage)).toBeNull();
    storage.setItem.mockImplementation(write);
    // The same text kept again - the panel keeps a draft each time it shows it - is written now: it never was.
    first.drafts.set(lease.id, typedOver(lease, 'while it was full'));
    expect(found(storage, [lease]).fields).toMatchObject({ stem: 'while it was full' });
    first.drafts.set(lease.id, typedOver(lease));
    expect(found(storage, [lease]).fields).toMatchObject({ stem: NAME });
  });

  it('takes the window\'s storage, or none when the window refuses to say', () => {
    expect(windowStorage()).toBe(window.localStorage);
    const refusal = vi.spyOn(window, 'localStorage', 'get').mockImplementation(() => { throw new DOMException('The operation is insecure.', 'SecurityError'); });
    expect(windowStorage()).toBeUndefined();
    refusal.mockRestore();
    expect(windowStorage()).toBe(window.localStorage);
  });

  it('says so when a document gains or loses a draft as it is put back', async () => {
    const storage = storageOf();
    launch(storage, [lease]).drafts.set(lease.id, typedOver(lease));
    const onChange = vi.fn();
    const kept = keepTypedBetweenLaunches(() => storage, onChange);
    kept.queueIs([lease], true);
    await Promise.resolve();
    expect(onChange).toHaveBeenCalledTimes(1);
  });
});
