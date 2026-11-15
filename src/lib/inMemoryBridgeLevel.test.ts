import { describe, expect, it, vi } from 'vitest';
import { createBalancedLevelBridge, createInMemoryBridge, createLearnedSpellingBridge, createReviewBatchBridge } from './inMemoryBridge';
import type { DesktopBridge } from './bridge';
import { humanizeReason } from './reasons';
import type { AppSettings, QueueItem, ReviewLevel } from '../types';

/*
  The demo bridge stores the review level and, when it changes, judges the
  documents that wait again from the readings they have, as the backend
  does: at Balanced a waiting row whose codes the level lets through is
  ready with those codes as notes, and back at Strict it waits as it did.
  At Strict the demo is what it always was.
*/

async function setLevel(bridge: DesktopBridge, reviewLevel: ReviewLevel) {
  await bridge.saveSettings({ ...await bridge.getSettings(), reviewLevel });
}

const byId = async (bridge: DesktopBridge, id: string) => (await bridge.listItems()).find((item) => item.id === id)!;

/** A row waiting in review with `codes`, named and dated, the model sure by its own figure. */
const waiting = (id: string, codes: string[], extra: Partial<QueueItem> = {}): QueueItem => ({
  id, originalFilename: `${id}.pdf`, status: 'review', proposedFilename: '2024-03-08 Letter to Fabrikam Inc.pdf', confidence: 0.97,
  description: 'Letter to Fabrikam Inc.', reason: humanizeReason(codes.join(', ')), reasonCodes: codes, ...extra,
});

describe('the review level in the demo bridge', () => {
  it('is strict to begin with, and is returned as it was saved', async () => {
    const bridge = createInMemoryBridge();
    expect((await bridge.getSettings()).reviewLevel).toBe('strict');
    await setLevel(bridge, 'balanced');
    expect((await bridge.getSettings()).reviewLevel).toBe('balanced');
    // Beside every other setting, none of which it touches.
    const settings = await bridge.getSettings();
    await bridge.saveSettings({ ...settings, automaticRename: true });
    expect(await bridge.getSettings()).toEqual({ ...settings, automaticRename: true, reviewLevel: 'balanced' });
    await setLevel(bridge, 'strict');
    expect((await bridge.getSettings()).reviewLevel).toBe('strict');
  });

  it('keeps settings saved with no level as they came, which is strict', async () => {
    const bridge = createInMemoryBridge();
    const { reviewLevel: _reviewLevel, ...older } = await bridge.getSettings();
    await bridge.saveSettings(older as AppSettings);
    expect('reviewLevel' in await bridge.getSettings()).toBe(false);
    expect((await byId(bridge, 'lease')).status).toBe('review');
  });

  it('starts at the level it is given', async () => {
    const bridge = createInMemoryBridge({ settings: { reviewLevel: 'balanced' } });
    expect((await bridge.getSettings()).reviewLevel).toBe('balanced');
    expect((await byId(bridge, 'lease')).status).toBe('ready');
  });
});

describe('the demo queue at Strict', () => {
  it('is the queue it always was, before the level is touched and after a save that leaves it strict', async () => {
    const bridge = createInMemoryBridge();
    const before = await bridge.listItems();
    expect(before.map((item) => `${item.id}:${item.status}`)).toEqual(['employment:ready', 'lease:review', 'nda:ready', 'financials:processing', 'service:ready', 'minutes:waiting', 'invoice:waiting', 'notes:waiting', 'completed:completed']);
    // The lease says why it waits in its own sentence, and carries no codes.
    expect(before[1]).toMatchObject({ reason: 'Lower confidence due to unclear document type keywords and multiple possible dates.' });
    expect(before[1]).not.toHaveProperty('reasonCodes');
    expect(before[1]).not.toHaveProperty('errorCode');
    await bridge.saveSettings({ ...await bridge.getSettings(), automaticRename: true });
    expect(await bridge.listItems()).toEqual(before);
    await setLevel(bridge, 'strict');
    expect(await bridge.listItems()).toEqual(before);
  });

  it('is the queue it was again after Balanced and back', async () => {
    for (const create of [createInMemoryBridge, createReviewBatchBridge, createLearnedSpellingBridge]) {
      const bridge = create();
      const before = await bridge.listItems();
      await setLevel(bridge, 'balanced');
      await setLevel(bridge, 'strict');
      expect(await bridge.listItems()).toEqual(before);
    }
  });
});

describe('the demo queue at Balanced', () => {
  it('shows the lease ready with a note, and every other row as it was', async () => {
    const bridge = createInMemoryBridge();
    const before = await bridge.listItems();
    await setLevel(bridge, 'balanced');
    const after = await bridge.listItems();
    expect(after.filter((item) => item.id !== 'lease')).toEqual(before.filter((item) => item.id !== 'lease'));
    const lease = after.find((item) => item.id === 'lease')!;
    expect(lease).toMatchObject({
      status: 'ready', proposedFilename: before[1].proposedFilename, description: before[1].description, confidence: 0.72,
      reasonCodes: ['TYPE_UNSUPPORTED', 'LOW_CONFIDENCE'], reason: humanizeReason('TYPE_UNSUPPORTED, LOW_CONFIDENCE'),
    });
    expect(lease).not.toHaveProperty('errorCode');
    expect(lease).not.toHaveProperty('approved');
    // Nothing is renamed by itself in the demo, whatever automatic renaming says.
    await bridge.saveSettings({ ...await bridge.getSettings(), automaticRename: true });
    expect((await byId(bridge, 'lease')).status).toBe('ready');
  });

  it('makes the scans whose only doubt is the model\'s ready, and leaves the ambiguous date waiting', async () => {
    const bridge = createReviewBatchBridge();
    await setLevel(bridge, 'balanced');
    const status = Object.fromEntries((await bridge.listItems()).map((item) => [item.id, item.status]));
    expect(status).toEqual({ 'scan-0101': 'ready', 'scan-0102': 'review', 'scan-0103': 'ready', 'scan-0104': 'ready', 'scan-0105': 'ready', 'scan-0106': 'ready', 'scan-0107': 'waiting' });
    expect(await byId(bridge, 'scan-0101')).toMatchObject({ reasonCodes: ['LOW_CONFIDENCE'], reason: humanizeReason('LOW_CONFIDENCE') });
    // A row ready at Strict is ready at Balanced, with nothing added to it.
    expect(await byId(bridge, 'scan-0103')).not.toHaveProperty('reasonCodes');
    // Ready, a row no longer offers other parties: those go with a review.
    expect(await byId(bridge, 'scan-0104')).not.toHaveProperty('partyChoices');
    await setLevel(bridge, 'strict');
    expect(await byId(bridge, 'scan-0104')).toMatchObject({ status: 'review', partyChoices: ['Tailspin Toys'] });
  });

  it('lets through only the codes the level names, each with the rest of its rule', async () => {
    const through: Array<[string, QueueItem]> = [
      ['a type Intern supplied', waiting('a', ['TYPE_INFERRED'])],
      ['a type the document does not confirm', waiting('b', ['TYPE_UNSUPPORTED'])],
      ['a general type', waiting('c', ['TYPE_MISSING'])],
      ['a description the document does not bear out', waiting('d', ['DESCRIPTION_UNSUPPORTED'])],
      ['a model unsure by what Intern measured', waiting('e', ['LOW_CONFIDENCE'], { confidence: 0.6 })],
      ['a stand-in kind with no type of the model\'s', waiting('f', ['TYPE_UNSUPPORTED', 'TYPE_INFERRED', 'TYPE_MISSING'])],
      ['all of them at once', waiting('g', ['TYPE_MISSING', 'TYPE_INFERRED', 'DESCRIPTION_UNSUPPORTED', 'LOW_CONFIDENCE'])],
      ['a code sent only as the error code', waiting('h', [], { errorCode: 'LOW_CONFIDENCE', reasonCodes: undefined, reason: humanizeReason('LOW_CONFIDENCE') })],
    ];
    const held: Array<[string, QueueItem]> = [
      ['a date reason', waiting('m', ['DATE_AMBIGUOUS'])],
      ['a date reason beside one it lets through', waiting('n', ['TYPE_INFERRED', 'DATE_UNSUPPORTED'])],
      ['a party reason', waiting('o', ['PARTY_UNSUPPORTED', 'LOW_CONFIDENCE'])],
      ['a description that is no sentence', waiting('p', ['DESCRIPTION_INVALID'])],
      ['text that may be misread', waiting('q', ['PARSER_WARNING'])],
      ['a model that asked for a check', waiting('r', ['MODEL_REQUESTED_REVIEW'])],
      ['the quick model\'s reading', waiting('s', ['TYPE_INFERRED', 'FAST_MODEL_READING'])],
      ['a code nobody has heard of', waiting('t', ['TYPE_INFERRED', 'SOMETHING_NEW'])],
      ['a low figure of the model\'s own', waiting('u', ['LOW_CONFIDENCE'], { confidence: 0.59 })],
      ['a model unsure with no figure of its own', waiting('v', ['LOW_CONFIDENCE'], { confidence: undefined })],
      ['a title standing in for an unconfirmed type', waiting('w', ['TYPE_UNSUPPORTED', 'TYPE_INFERRED'])],
      ['a name with no date', waiting('x', ['TYPE_INFERRED'], { proposedFilename: 'Letter to Fabrikam Inc.pdf' })],
      ['a name with no type', waiting('y', ['TYPE_MISSING'], { proposedFilename: '2024-03-08.pdf' })],
      ['a name that only says Document', waiting('z', ['TYPE_MISSING'], { proposedFilename: '2024-03-08 Document.pdf' })],
      ['a near-duplicate', waiting('aa', ['TYPE_INFERRED', 'NEAR_DUPLICATE'], { nearDuplicateOf: '2024-03-08 Letter.pdf' })],
      ['an unchecked learned spelling', waiting('ab', ['LEARNED_SPELLING_UNCONFIRMED'], { errorCode: 'LEARNED_SPELLING_UNCONFIRMED' })],
      ['a rename the person undid', waiting('ac', ['TYPE_INFERRED', 'UNDONE'])],
      ['a file that changed', waiting('ad', ['TYPE_INFERRED'], { errorCode: 'FILE_CHANGED', reasonCodes: undefined, reason: humanizeReason('FILE_CHANGED') })],
      ['a reason given only in words', waiting('ae', [], { reasonCodes: undefined, reason: 'Needs review because the invoice and due dates are both present.' })],
      ['a document Intern could not read', { id: 'af', originalFilename: 'NEEDS REVIEW - locked.pdf', status: 'review', errorCode: 'PASSWORD_PROTECTED', reason: humanizeReason('PASSWORD_PROTECTED') }],
      ['a duplicate', { id: 'ag', originalFilename: 'again.pdf', status: 'review', errorCode: 'DUPLICATE', reason: 'Duplicate of 2024-03-08 Letter.pdf' }],
      ['files left part-way through a rename', waiting('ah', ['TYPE_INFERRED'], { parked: true })],
    ];
    const bridge = createInMemoryBridge({ items: [...through, ...held].map(([, item]) => item) });
    await setLevel(bridge, 'balanced');
    const now = new Map((await bridge.listItems()).map((item) => [item.id, item]));
    for (const [what, item] of through) expect(now.get(item.id)?.status, what).toBe('ready');
    for (const [what, item] of held) {
      expect(now.get(item.id)?.status, what).toBe('review');
      // Held, it is the row it was.
      expect(now.get(item.id), what).toEqual((await createInMemoryBridge({ items: [item] }).listItems())[0]);
    }
    // A code sent only as the error code is kept as the note, and a ready row has no error code.
    expect(now.get('h')).toMatchObject({ reasonCodes: ['LOW_CONFIDENCE'] });
    expect(now.get('h')).not.toHaveProperty('errorCode');
  });

  it('leaves alone what is not waiting for a decision: read, being read, filed, failed, approved, a copy', async () => {
    const items: QueueItem[] = [
      { id: 'queued', originalFilename: 'queued.pdf', status: 'waiting' },
      { id: 'reading', originalFilename: 'reading.pdf', status: 'processing', stage: 'naming', proposedFilename: '2024-03-08 Letter.pdf', reasonCodes: ['TYPE_INFERRED'] },
      { ...waiting('filed', ['TYPE_INFERRED']), status: 'completed', filedName: '2024-03-08 Letter to Fabrikam Inc.pdf', undoable: true, keptOriginal: false },
      { ...waiting('kept', ['TYPE_INFERRED']), status: 'completed', keptOriginal: true, undoable: false },
      { ...waiting('failed', ['TYPE_INFERRED']), status: 'failed' },
      { ...waiting('approved', ['TYPE_INFERRED']), status: 'ready', approved: true },
      { id: 'copy', originalFilename: 'copy.pdf', status: 'ready', proposedFilename: 'COPY - 2024-03-08 Letter.pdf', approved: true, copyOf: '2024-03-08 Letter.pdf', reason: humanizeReason('EXACT_COPY'), reasonCodes: ['EXACT_COPY'] },
      { id: 'sure', originalFilename: 'sure.pdf', status: 'ready', proposedFilename: '2024-03-08 Invoice.pdf', confidence: 0.99 },
    ];
    const bridge = createInMemoryBridge({ items });
    const before = await bridge.listItems();
    await setLevel(bridge, 'balanced');
    expect(await bridge.listItems()).toEqual(before);
    await setLevel(bridge, 'strict');
    expect(await bridge.listItems()).toEqual(before);
  });

  it('sends a row that came ready with notes back to wait when the level becomes Strict', async () => {
    const noted: QueueItem = { ...waiting('noted', ['TYPE_INFERRED', 'LOW_CONFIDENCE']), status: 'ready' };
    const bridge = createInMemoryBridge({ items: [noted], settings: { reviewLevel: 'balanced' } });
    expect((await byId(bridge, 'noted')).status).toBe('ready');
    await setLevel(bridge, 'strict');
    expect(await byId(bridge, 'noted')).toEqual({ ...noted, status: 'review' });
    await setLevel(bridge, 'balanced');
    expect(await byId(bridge, 'noted')).toEqual(noted);
  });

  it('keeps a queue given at Strict as it was given, ready rows with codes included', async () => {
    const noted: QueueItem = { ...waiting('noted', ['TYPE_INFERRED']), status: 'ready' };
    expect(await createInMemoryBridge({ items: [noted] }).listItems()).toEqual([noted]);
  });
});

describe('a document ready with a note in the demo bridge', () => {
  it('is renamed with no note left when a person approves it, as the backend clears an approved record\'s reasons', async () => {
    const bridge = createInMemoryBridge();
    await setLevel(bridge, 'balanced');
    const lease = await byId(bridge, 'lease');
    await bridge.approve('lease', lease.proposedFilename!, lease.description ?? '');
    const filed = await byId(bridge, 'lease');
    expect(filed).toMatchObject({ status: 'completed', filedName: lease.proposedFilename, undoable: true });
    expect(filed).not.toHaveProperty('reasonCodes');
    expect(filed).not.toHaveProperty('reason');
    // Filed, it is not what a change of level touches.
    await setLevel(bridge, 'strict');
    expect(await byId(bridge, 'lease')).toEqual(filed);
  });

  // An undone rename waits for a decision rather than being renamed again
  // by the next pass. The lease's seeded reading must not make it ready
  // again behind that.
  it('waits at either level once its rename is undone', async () => {
    const bridge = createInMemoryBridge();
    await setLevel(bridge, 'balanced');
    const lease = await byId(bridge, 'lease');
    await bridge.approve('lease', lease.proposedFilename!, lease.description ?? '');
    await bridge.undo('lease');
    const undone = await byId(bridge, 'lease');
    expect(undone).toMatchObject({ status: 'review', reason: humanizeReason('UNDONE') });
    await setLevel(bridge, 'strict');
    expect(await byId(bridge, 'lease')).toEqual(undone);
    await setLevel(bridge, 'balanced');
    expect(await byId(bridge, 'lease')).toEqual(undone);
  });

  it('is kept under its own name, or removed, like any ready document', async () => {
    const bridge = createReviewBatchBridge();
    await setLevel(bridge, 'balanced');
    await bridge.keepOriginal('scan-0101');
    expect(await byId(bridge, 'scan-0101')).toMatchObject({ status: 'completed', keptOriginal: true, undoable: false });
    await bridge.remove('scan-0104');
    expect((await bridge.listItems()).some((item) => item.id === 'scan-0104')).toBe(false);
    // Decided, neither is put back to wait when the level becomes Strict.
    await setLevel(bridge, 'strict');
    expect((await byId(bridge, 'scan-0101')).status).toBe('completed');
    expect((await byId(bridge, 'scan-0106')).status).toBe('review');
  });

  it('is judged by the level again when it is read again', async () => {
    vi.useFakeTimers();
    try {
      const bridge = createInMemoryBridge({ analysisDelayMs: 10 });
      const strictLease = await byId(bridge, 'lease');
      await setLevel(bridge, 'balanced');
      await bridge.reanalyze('lease');
      expect((await byId(bridge, 'lease')).status).toBe('waiting');
      vi.advanceTimersByTime(10);
      expect(await byId(bridge, 'lease')).toMatchObject({ status: 'ready', reasonCodes: ['TYPE_UNSUPPORTED', 'LOW_CONFIDENCE'], proposalRevision: '1' });
      // And back at Strict it waits as it did, as the reading it now is.
      await setLevel(bridge, 'strict');
      expect(await byId(bridge, 'lease')).toEqual({ ...strictLease, proposalRevision: '1' });
    } finally { vi.useRealTimers(); }
  });

  it('comes back for review, as before, when it is read again at Strict', async () => {
    vi.useFakeTimers();
    try {
      const bridge = createInMemoryBridge({ analysisDelayMs: 10 });
      const lease = await byId(bridge, 'lease');
      await bridge.reanalyze('lease');
      vi.advanceTimersByTime(10);
      expect(await byId(bridge, 'lease')).toEqual({ ...lease, proposalRevision: '1' });
    } finally { vi.useRealTimers(); }
  });
});

describe('the Balanced demo queue', () => {
  it('starts at Balanced with the lease ready with a note and a scan renamed with one', async () => {
    const bridge = createBalancedLevelBridge();
    expect((await bridge.getSettings()).reviewLevel).toBe('balanced');
    expect(await byId(bridge, 'lease')).toMatchObject({ status: 'ready', reasonCodes: ['TYPE_UNSUPPORTED', 'LOW_CONFIDENCE'] });
    const renamed = await byId(bridge, 'renamed-noted');
    expect(renamed).toMatchObject({ status: 'completed', keptOriginal: false, undoable: true, reasonCodes: ['TYPE_INFERRED', 'TYPE_MISSING'], reason: humanizeReason('TYPE_INFERRED, TYPE_MISSING') });
    expect(renamed.filedName).toBe(renamed.proposedFilename);
    // Its rename is in the history, newest, for the mark there.
    expect((await bridge.historyList())[0]).toMatchObject({ queueItemId: 'renamed-noted', direction: 'apply', stage: 'complete' });
    // Chosen Strict, the lease waits again; the filed scan is not touched.
    await setLevel(bridge, 'strict');
    expect(await byId(bridge, 'lease')).toMatchObject({ status: 'review', reason: 'Lower confidence due to unclear document type keywords and multiple possible dates.' });
    expect(await byId(bridge, 'renamed-noted')).toEqual(renamed);
  });

  it('puts an undone rename back to wait with its codes and the undo, which holds it at either level', async () => {
    const bridge = createBalancedLevelBridge();
    await bridge.undo('renamed-noted');
    const undone = await byId(bridge, 'renamed-noted');
    expect(undone).toMatchObject({ status: 'review', undoable: false, reasonCodes: ['TYPE_INFERRED', 'TYPE_MISSING', 'UNDONE'], reason: humanizeReason('TYPE_INFERRED, TYPE_MISSING, UNDONE') });
    expect(undone).not.toHaveProperty('filedName');
    await setLevel(bridge, 'strict');
    await setLevel(bridge, 'balanced');
    expect(await byId(bridge, 'renamed-noted')).toEqual(undone);
  });
});
