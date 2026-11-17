import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueItem } from '../../types';
import { openDetails, queueCell } from '../queue/gridTesting';
import { asTheWindowReads, sentProposal, sentRow } from '../queue/sentTesting';
import { levelChanged, levelChangedText } from './levelChanged';
import type { LevelChange } from './levelChanged';

/*
  A Settings save that changes the review level has the queue judge again
  the documents that wait. The window says what that did, from its own rows:
  the queue just before the save, and as the first read after it shows it.
*/

const row = (id: string, status: QueueItem['status'], more: Partial<QueueItem> = {}): QueueItem => ({ id, originalFilename: `${id}.pdf`, status, proposedFilename: `2024-05-01 Letter ${id}.pdf`, ...more });
/** What a reading Balanced lets through carries, at either level: notes on the type and on the model's doubt. */
const NOTES = { reasonCodes: ['TYPE_UNSUPPORTED', 'LOW_CONFIDENCE'], reason: 'The document does not confirm the type.' };
const nobody = { decidedHere: () => false, typed: () => false };

describe('what a change to Balanced did, counted', () => {
  // Each document waits in review before the save; after it, it is where the second column says.
  const cases: Array<[string, QueueItem, boolean]> = [
    ['ready with its notes', row('ready', 'ready', NOTES), true],
    ['being renamed, with its notes', row('renaming', 'processing', { ...NOTES, stage: 'filing', cancelable: false }), true],
    ['renamed by Intern, with its notes', row('renamed', 'completed', { ...NOTES, filedName: '2024-05-01 Letter renamed.pdf', undoable: true, keptOriginal: false }), true],
    ['completed by Intern under the name it had', row('named', 'completed', { reasonCodes: [...NOTES.reasonCodes, 'ALREADY_NAMED'], keptOriginal: true }), true],
    ['still in review', row('review', 'review', NOTES), false],
    ['approved by a person and waiting for the queue', row('approved', 'ready', { approved: true }), false],
    ['renamed after a person approved it: no reasons left', row('approved-renamed', 'completed', { filedName: '2024-05-01 Letter approved-renamed.pdf', undoable: true, keptOriginal: false }), false],
    ['kept under its own name by a person: the review\'s words, no ALREADY_NAMED', row('kept', 'completed', { ...NOTES, keptOriginal: true }), false],
    ['being read again', row('reading', 'processing', { stage: 'reading', proposedFilename: undefined }), false],
    ['waiting to be read again', row('waiting', 'waiting', { proposedFilename: undefined }), false],
    ['failed', row('failed', 'failed', { errorCode: 'MODEL_FAILED' }), false],
    ['a staged copy', row('copy', 'ready', { ...NOTES, approved: true, copyOf: '2024-05-01 Letter.pdf' }), false],
  ];
  it.each(cases)('%s: %s', (_what, after, counted) => {
    const before = [{ ...after, status: 'review' as const, stage: undefined, ...NOTES }];
    expect(levelChanged(before, [after], 'balanced', nobody)).toEqual({ to: 'balanced', count: counted ? 1 : 0, typed: 0 });
  });

  it('counts only what waited in review, and not a document gone from the queue', () => {
    const before = [row('a', 'review', NOTES), row('b', 'ready'), row('c', 'review', NOTES), row('d', 'review', NOTES)];
    const after = [row('a', 'ready', NOTES), row('b', 'ready'), row('c', 'ready', NOTES)];
    expect(levelChanged(before, after, 'balanced', nobody).count).toBe(2);
  });

  it('leaves out a document the window decided between the two reads', () => {
    const before = [row('a', 'review', NOTES), row('b', 'review', NOTES)];
    const after = [row('a', 'ready', NOTES), row('b', 'ready', NOTES)];
    expect(levelChanged(before, after, 'balanced', { ...nobody, decidedHere: (id) => id === 'b' }).count).toBe(1);
  });

  it('says how many of them have something typed and not filed', () => {
    const before = [row('a', 'review', NOTES), row('b', 'review', NOTES), row('c', 'review', NOTES)];
    const after = [row('a', 'ready', NOTES), row('b', 'ready', NOTES), row('c', 'review', NOTES)];
    expect(levelChanged(before, after, 'balanced', { ...nobody, typed: (id) => id !== 'a' })).toEqual({ to: 'balanced', count: 2, typed: 1 });
  });

  it('reads rows as the desktop backend sends them: codes as one reason, a rename under way as applying', async () => {
    const letter = sentProposal('2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', 'Letter.', 1);
    const before = await asTheWindowReads([
      sentRow(1, 'Scan 0601.pdf', 'needs_review', { ...letter, reason: 'TYPE_INFERRED, LOW_CONFIDENCE' }),
      sentRow(2, 'Scan 0602.pdf', 'needs_review', { ...letter, reason: 'DESCRIPTION_UNSUPPORTED' }),
      sentRow(3, 'Scan 0603.pdf', 'needs_review', { ...letter, reason: 'DATE_AMBIGUOUS' }),
      sentRow(4, 'Scan 0604.pdf', 'needs_review', { ...letter, reason: 'TYPE_INFERRED' }),
    ]);
    const after = await asTheWindowReads([
      sentRow(1, 'Scan 0601.pdf', 'ready', { ...letter, reason: 'TYPE_INFERRED, LOW_CONFIDENCE' }),
      sentRow(2, 'Scan 0602.pdf', 'applying', { ...letter, reason: 'DESCRIPTION_UNSUPPORTED' }),
      sentRow(3, 'Scan 0603.pdf', 'needs_review', { ...letter, reason: 'DATE_AMBIGUOUS' }),
      // Approved by a person meanwhile: renamed, and the reasons went with the approval.
      sentRow(4, 'Scan 0604.pdf', 'completed', { ...letter, filedName: letter.proposedFilename, filedPath: `C:\\Filed\\${letter.proposedFilename}`, undoable: true }),
    ]);
    expect(levelChanged(before, after, 'balanced', nobody).count).toBe(2);
  });
});

describe('what a change to Strict did, counted', () => {
  it('counts the ready documents with a note that nobody approved and that wait in review again', () => {
    const before = [
      row('noted', 'ready', NOTES),
      row('noted-too', 'ready', NOTES),
      row('plain', 'ready'),
      row('approved', 'ready', { ...NOTES, approved: true }),
      row('copy', 'ready', { ...NOTES, approved: true, copyOf: '2024-05-01 Letter.pdf' }),
      row('review', 'review', NOTES),
    ];
    const after = before.map((item) => ({ ...item, status: 'review' as const, approved: undefined }));
    expect(levelChanged(before, after, 'strict', nobody)).toEqual({ to: 'strict', count: 2, typed: 0 });
  });

  it('does not count one renamed, read again, failed or decided by the window meanwhile', () => {
    const before = ['renamed', 'reading', 'failed', 'decided', 'back'].map((id) => row(id, 'ready', NOTES));
    const after = [
      row('renamed', 'completed', { ...NOTES, filedName: '2024-05-01 Letter renamed.pdf', undoable: true }),
      row('reading', 'processing', { stage: 'reading' }),
      row('failed', 'failed', { errorCode: 'MODEL_FAILED' }),
      row('decided', 'review', NOTES),
      row('back', 'review', NOTES),
    ];
    expect(levelChanged(before, after, 'strict', { ...nobody, decidedHere: (id) => id === 'decided' }).count).toBe(1);
  });

  it('is none where nothing moved, either way', () => {
    const queue = [row('a', 'review', NOTES), row('b', 'ready'), row('c', 'completed', { undoable: true, filedName: 'c.pdf' })];
    expect(levelChanged(queue, queue, 'strict', nobody).count).toBe(0);
    expect(levelChanged(queue, queue, 'balanced', nobody).count).toBe(0);
    expect(levelChanged([], [], 'balanced', nobody).count).toBe(0);
  });
});

describe('the words for a level change', () => {
  const balanced = (count: number, typed = 0): LevelChange => ({ to: 'balanced', count, typed });
  it('toward Balanced: how many became ready, and what renames them', () => {
    expect(levelChangedText(balanced(1), 'automatic')).toBe('The review level is now Balanced. 1 document that needed review became ready. With automatic renaming on, Intern renames it by itself.');
    expect(levelChangedText(balanced(3), 'automatic')).toBe('The review level is now Balanced. 3 documents that needed review became ready. With automatic renaming on, Intern renames them by itself.');
    expect(levelChangedText(balanced(2), 'paused')).toBe('The review level is now Balanced. 2 documents that needed review became ready. The queue is paused, so Intern renames nothing by itself until it is resumed.');
    expect(levelChangedText(balanced(1), 'off')).toBe('The review level is now Balanced. 1 document that needed review became ready. Rename all ready renames it.');
    expect(levelChangedText(balanced(4), 'off')).toBe('The review level is now Balanced. 4 documents that needed review became ready. Rename all ready renames them.');
  });

  it('toward Balanced with automatic renaming off: Rename all ready leaves what was typed for, as its own outcome says', () => {
    expect(levelChangedText(balanced(1, 1), 'off')).toBe('The review level is now Balanced. 1 document that needed review became ready. It has a name or a description you typed, so Rename all ready leaves it for you to file.');
    expect(levelChangedText(balanced(2, 2), 'off')).toBe('The review level is now Balanced. 2 documents that needed review became ready. They have a name or a description you typed, so Rename all ready leaves them for you to file.');
    expect(levelChangedText(balanced(3, 1), 'off')).toBe('The review level is now Balanced. 3 documents that needed review became ready. Rename all ready renames 2. 1 with a name or a description you typed is left for you to file.');
    expect(levelChangedText(balanced(5, 2), 'off')).toBe('The review level is now Balanced. 5 documents that needed review became ready. Rename all ready renames 3. 2 with a name or a description you typed are left for you to file.');
  });

  it('toward Strict: how many need review again', () => {
    expect(levelChangedText({ to: 'strict', count: 1, typed: 0 }, 'off')).toBe('The review level is now Strict. 1 ready document with a note needs review again.');
    expect(levelChangedText({ to: 'strict', count: 2, typed: 1 }, 'automatic')).toBe('The review level is now Strict. 2 ready documents with a note need review again.');
  });

  it('none either way: the level is on, and nothing that waited changed', () => {
    for (const renamed of ['automatic', 'paused', 'off'] as const) {
      expect(levelChangedText(balanced(0), renamed)).toBe('The review level is now Balanced. No document that needed review became ready.');
      expect(levelChangedText({ to: 'strict', count: 0, typed: 0 }, renamed)).toBe('The review level is now Strict. No ready document with a note went back to review.');
    }
  });

  it('has no long dash, promises nothing, and has no figure but counts of documents', () => {
    const all = [0, 1, 2].flatMap((count) => (['automatic', 'paused', 'off'] as const).flatMap((renamed) => [
      levelChangedText(balanced(count), renamed), levelChangedText(balanced(count, count), renamed), levelChangedText({ to: 'strict', count, typed: 0 }, renamed),
    ]));
    for (const words of all) {
      expect(words).not.toMatch(/[\u2012-\u2015]/);
      expect(words).not.toMatch(/\bwill\b|%/);
      // Every number is a count of documents, or how many Rename all ready renames.
      expect(words.replace(/\b\d+ (ready )?documents?\b|\b\d+ with a name\b|renames \d+\./g, '')).not.toMatch(/\d/);
    }
  });
});

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  const dialog = await screen.findByRole('dialog', { name: 'Settings' });
  fireEvent.click(within(dialog).getByText('How Intern names files', { selector: 'summary' }));
  return dialog;
}
const level = (dialog: HTMLElement) => within(dialog).getByRole('group', { name: 'Which documents are ready' });
const strictChoice = (dialog: HTMLElement) => within(level(dialog)).getByRole('radio', { name: /^Strict \(default\)/ });
const balancedChoice = (dialog: HTMLElement) => within(level(dialog)).getByRole('radio', { name: /^Balanced/ });
const save = async (choose: (dialog: HTMLElement) => void) => {
  const dialog = await openSettings();
  choose(dialog);
  fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
  await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
  await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
};
const said = () => screen.getByRole('status', { name: 'Action status' });
const result = () => screen.queryByRole('group', { name: 'Action result' });
const LEASE = 'Lease Agreement - 123 Main St.pdf';

describe('the window after a save that changed the level, in the demo', () => {
  it('says the lease became ready at Balanced, and needs review again at Strict, where the outcome of an action is said, with no button', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    await screen.findByRole('grid', { name: 'Queue' });

    await save((dialog) => fireEvent.click(balancedChoice(dialog)));
    const toBalanced = 'The review level is now Balanced. 1 document that needed review became ready. Rename all ready renames it.';
    await waitFor(() => expect(result()).toHaveTextContent(toBalanced));
    expect(said()).toHaveTextContent(toBalanced);
    expect(within(result()!).queryByRole('button', { name: /Undo/ })).not.toBeInTheDocument();
    expect(queueCell(LEASE, 'status')).toHaveTextContent('Ready with a note');

    await save((dialog) => fireEvent.click(strictChoice(dialog)));
    const toStrict = 'The review level is now Strict. 1 ready document with a note needs review again.';
    await waitFor(() => expect(result()).toHaveTextContent(toStrict));
    expect(said()).toHaveTextContent(toStrict);
    expect(queueCell(LEASE, 'status')).toHaveTextContent('Needs review');
  });

  it('says nothing of a save that leaves the level as it was, at Strict with automatic renaming off', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    await screen.findByRole('grid', { name: 'Queue' });
    const status = said().textContent;
    await save((dialog) => fireEvent.change(within(dialog).getByLabelText("This machine's name"), { target: { value: 'Reception' } }));
    expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ reviewLevel: 'strict', automaticRename: false, machineLabel: 'Reception' }));
    expect(document.querySelector('.toasts')).toBeNull();
    expect(said().textContent).toBe(status);
    expect(queueCell(LEASE, 'status')).toHaveTextContent('Needs review');
  });

  it('says that Intern renames them by itself with automatic renaming on, and that it does not while the queue is paused', async () => {
    render(<App bridge={createInMemoryBridge({ settings: { automaticRename: true } })} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await save((dialog) => fireEvent.click(balancedChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('The review level is now Balanced. 1 document that needed review became ready. With automatic renaming on, Intern renames it by itself.'));

    await save((dialog) => fireEvent.click(strictChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('1 ready document with a note needs review again.'));
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));
    await screen.findByRole('button', { name: 'Resume queue' });
    await save((dialog) => fireEvent.click(balancedChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('The review level is now Balanced. 1 document that needed review became ready. The queue is paused, so Intern renames nothing by itself until it is resumed.'));
  });

  it('says Rename all ready leaves the lease when a name was typed for it, and keeps what was typed', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    await screen.findByRole('grid', { name: 'Queue' });
    const panel = await openDetails(LEASE);
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: '2023-09-15 Lease of 123 Main St' } });
    await save((dialog) => fireEvent.click(balancedChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('1 document that needed review became ready. It has a name or a description you typed, so Rename all ready leaves it for you to file.'));
    expect(within(panel).getByLabelText('Filename')).toHaveValue('2023-09-15 Lease of 123 Main St');
  });

  it('says none became ready where none could, and none went back where none was ready with a note', async () => {
    // The lease with no codes behind its words: nothing a level can judge.
    render(<App bridge={createInMemoryBridge({ readings: {} })} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await save((dialog) => fireEvent.click(balancedChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('The review level is now Balanced. No document that needed review became ready.'));
    await save((dialog) => fireEvent.click(strictChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('The review level is now Strict. No ready document with a note went back to review.'));
  });

  it('does not count a document decided somewhere else during the save, and says nothing when the queue cannot be read after it', async () => {
    const base = createInMemoryBridge();
    let failRead = false;
    const bridge = {
      ...base,
      // As the save is judged, someone approves the lease elsewhere: renamed, with its reasons cleared.
      saveSettings: vi.fn(async (next: Parameters<typeof base.saveSettings>[0]) => { await base.saveSettings(next); if (next.reviewLevel === 'balanced') await base.approve('lease', '2023-09-15 Lease Agreement.pdf', ''); }),
      listItems: vi.fn(async () => { if (failRead) throw new Error('the queue could not be read'); return base.listItems(); }),
    };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await save((dialog) => fireEvent.click(balancedChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('The review level is now Balanced. No document that needed review became ready.'));
    fireEvent.click(within(result()!).getByRole('button', { name: 'Dismiss' }));

    failRead = true;
    await save((dialog) => fireEvent.click(strictChoice(dialog)));
    await waitFor(() => expect(screen.getByRole('alert', { name: 'Queue connection error' })).toBeInTheDocument());
    expect(result()).toBeNull();
  });

  it('counts with automatic renaming on what the backend renamed by itself before the first read, as it sends the rows', async () => {
    const letter = sentProposal('2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', 'Letter.', 1);
    const sent = [
      sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...letter, reason: 'TYPE_INFERRED, LOW_CONFIDENCE' }),
      sentRow(32, 'Scan 0602.pdf', 'needs_review', { ...letter, reason: 'DESCRIPTION_UNSUPPORTED' }),
      sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...letter, reason: 'DATE_AMBIGUOUS' }),
    ];
    const base = createInMemoryBridge({ settings: { automaticRename: true } });
    const bridge = {
      ...base,
      listItems: () => asTheWindowReads(sent),
      saveSettings: vi.fn(async (next: Parameters<typeof base.saveSettings>[0]) => {
        await base.saveSettings(next);
        if (next.reviewLevel !== 'balanced') return;
        // Judged again, and the scheduler already at work: one renamed, one being renamed.
        sent[0] = { ...sent[0], status: 'completed', filedName: letter.proposedFilename, filedPath: `C:\\Filed\\${letter.proposedFilename}`, undoable: true };
        sent[1] = { ...sent[1], status: 'applying' };
      }),
    };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await save((dialog) => fireEvent.click(balancedChoice(dialog)));
    await waitFor(() => expect(result()).toHaveTextContent('The review level is now Balanced. 2 documents that needed review became ready. With automatic renaming on, Intern renames them by itself.'));
  });
});
