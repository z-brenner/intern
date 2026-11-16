import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { Toast, TOAST_TIMEOUT_MS } from '../../components/Toast';
import { SETTLE_MS, takeOnlyAimedPresses } from '../../lib/aimedPress';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueEventSource } from '../../lib/tauriBridge';
import type { QueueItem } from '../../types';
import { openDetails, queueCell } from '../queue/gridTesting';
import { asTheWindowReads, desktopOver, sentProposal, sentRow } from '../queue/sentTesting';
import type { Sent } from '../queue/sentTesting';
import { keptText, typedAndNotFiled } from './drafts';
import type { EarlierDrafts, ReviewDrafts } from './drafts';
import { renamedByItselfText, stillSo, undoableRenames, watchRenames, withRenames } from './renamedByItself';
import type { RenameSaid } from './renamedByItself';

/*
  Intern can rename by itself a document whose name or description a person
  typed in Details and did not file. What was typed is set aside beside it,
  read-only. The window says so, with Undo, when the backend marks the
  rename as made with nobody's approval (renamedWithoutApproval), and never
  of a rename the person approved, however it was finished. The rows here
  are as the desktop backend sends them, read through TauriBridge
  (sentTesting): its statuses ("needs_review", "applying"), its reasons as
  one list, the mark on every row (false but on a rename by itself), and
  null where it has nothing.
*/

const LETTER = sentProposal('2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', 'Letter to Halvorsen Fixture Works LLC.', 1);
const INVOICE = sentProposal('2025-05-02 Invoice from Brightwater Supply Co.pdf', 'Invoice from Brightwater Supply Co.', 0.98);
const NOTICE = sentProposal('2025-05-03 Notice to Contoso Ltd.pdf', 'Notice to Contoso Ltd.', 0.9);
/**
 * Filed as the backend reports a filing: completed, with the receipt's name
 * and path, and undoable. `byItself`: renamed by Intern with nobody's
 * approval, which the backend marks (renamedWithoutApproval); a filing a
 * person approved is not marked, however it was finished.
 */
const filed = (name: unknown, byItself = false) => ({ status: 'completed', filedName: name, filedPath: `C:\\Filed\\${String(name)}`, keptOriginal: false, undoable: true, renamedWithoutApproval: byItself });
const filedByItself = (name: unknown) => filed(name, true);

/** One look at the queue, as the window takes one each time it is read: the renames over typed text seen happen since the last. */
function looker(typed: (id: string) => boolean) {
  let unfiled: ReadonlySet<string> = new Set();
  return async (rows: Sent[]) => {
    const look = watchRenames(unfiled, await asTheWindowReads(rows), typed);
    unfiled = look.unfiled;
    return look.seen;
  };
}

describe('a rename over what was typed, seen in the queue as it is read', () => {
  const typedFor = (...ids: string[]) => (id: string) => ids.includes(id);

  it('is seen once the document is renamed, not while it is being renamed, and once only', async () => {
    const look = looker(typedFor('31'));
    const waiting = sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...LETTER, reason: 'TYPE_INFERRED, LOW_CONFIDENCE' });
    expect(await look([waiting])).toEqual([]);
    // Judged again at Balanced: ready, with its reasons as notes. Not renamed yet.
    expect(await look([{ ...waiting, status: 'ready' }])).toEqual([]);
    expect(await look([{ ...waiting, status: 'applying' }])).toEqual([]);
    expect(await look([{ ...waiting, ...filedByItself(LETTER.proposedFilename) }])).toEqual([{ id: '31', name: 'Scan 0601.pdf' }]);
    // Once seen renamed, it is not looked for any more.
    expect(await look([{ ...waiting, ...filedByItself(LETTER.proposedFilename) }])).toEqual([]);
  });

  it('is seen when a ready document is renamed between two reads, under a name with a number the folder needed', async () => {
    const look = looker(typedFor('32'));
    const ready = sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE);
    await look([ready]);
    expect(await look([{ ...ready, ...filedByItself('2025-05-02 Invoice from Brightwater Supply Co (2).pdf') }])).toEqual([{ id: '32', name: 'Scan 0602.pdf' }]);
  });

  it('takes what was typed as the window does: in the fields, or set aside when the document changed, and not a proposal only shown', async () => {
    const drafts: ReviewDrafts = new Map();
    const earlier: EarlierDrafts = new Map();
    const look = looker((id) => typedAndNotFiled(drafts, earlier, id));
    const rows = [sentRow(31, 'Scan 0601.pdf', 'ready', LETTER), sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE), sentRow(33, 'Scan 0603.pdf', 'ready', NOTICE)];
    await look(rows);
    // Typed in the fields; set aside when it was read again; and shown in the panel but not changed.
    drafts.set('31', { item: '31', for: 'x', stem: '2025-05-01 Letter to Halvorsen about the fixtures', description: '', edited: true });
    earlier.set('32', [keptText('2025-05-02 Invoice 4471 from Brightwater', true)]);
    drafts.set('33', { item: '33', for: 'x', stem: '2025-05-03 Notice to Contoso Ltd', description: 'Notice to Contoso Ltd.', edited: false });
    earlier.set('33', [keptText('2025-05-03 Notice to Contoso Ltd', false)]);
    const seen = await look(rows.map((row) => ({ ...row, ...filedByItself(row.proposedFilename) })));
    expect(seen.map(({ id }) => id)).toEqual(['31', '32']);
  });

  it('is not seen for a document with nothing typed for it', async () => {
    const look = looker(typedFor());
    const ready = sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE);
    await look([ready]);
    expect(await look([{ ...ready, status: 'applying' }])).toEqual([]);
    expect(await look([{ ...ready, ...filedByItself(INVOICE.proposedFilename) }])).toEqual([]);
  });

  it('is seen only for a rename the backend marks, whatever name it was filed under; never for one a person approved, however it was finished', async () => {
    const look = looker(() => true);
    const mine = '2025-05-01 Letter to Halvorsen about the fixtures.pdf';
    const rows = [
      sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...LETTER, reason: 'DATE_AMBIGUOUS' }),
      sentRow(32, 'Scan 0602.pdf', 'ready', { ...INVOICE, approved: true }),
      sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...NOTICE, proposedFilename: mine, reason: 'SOURCE_DELETE_FAILED', errorCode: 'SOURCE_DELETE_FAILED', parked: true }),
      sentRow(34, 'Scan 0604.pdf', 'needs_review', { ...NOTICE, reason: 'DATE_AMBIGUOUS' }),
      sentRow(35, 'Scan 0605.pdf', 'ready', LETTER),
    ];
    await look(rows);
    const seen = await look([
      // Approved here, under the name typed, and filed.
      { ...rows[0], ...filed(mine), proposedFilename: mine, reason: null },
      // Approved while the queue was busy, and filed by the queue once it was free.
      { ...rows[1], ...filed(INVOICE.proposedFilename), approved: false },
      // An approval parked halfway, finished by Check again.
      { ...rows[2], ...filed(mine), reason: null, errorCode: null, parked: false },
      // Read again by itself and renamed under a reading no read of the queue showed.
      { ...rows[3], ...filedByItself('2025-05-03 Termination notice to Contoso Ltd.pdf'), proposedFilename: '2025-05-03 Termination notice to Contoso Ltd.pdf', description: 'A notice of termination.' },
      // Renamed by itself under the proposal shown.
      { ...rows[4], ...filedByItself(LETTER.proposedFilename) },
    ]);
    expect(seen.map(({ id }) => id)).toEqual(['34', '35']);
  });

  it('is not seen for a completion that renamed nothing, nor for a document read again; one read again, shown waiting with its new proposal, and then renamed is seen', async () => {
    const look = looker(() => true);
    const letter = sentRow(31, 'Scan 0601.pdf', 'ready', LETTER);
    const invoice = sentRow(32, 'Scan 0602.pdf', 'needs_review', { ...INVOICE, reason: 'LOW_CONFIDENCE' });
    await look([letter, invoice]);
    // The name was the document's own already: completed, nothing renamed, and nothing marked.
    expect(await look([{ ...letter, status: 'completed', keptOriginal: true, reason: 'ALREADY_NAMED' }, { ...invoice, status: 'analyzing', proposedFilename: null }])).toEqual([]);
    const reread = sentProposal('2025-05-02 Invoice 4471 from Brightwater Supply Co.pdf', 'Invoice 4471.', 0.99);
    expect(await look([{ ...letter, status: 'completed', keptOriginal: true }, { ...invoice, ...reread, status: 'ready', reason: null }])).toEqual([]);
    expect(await look([{ ...letter, status: 'completed', keptOriginal: true }, { ...invoice, ...reread, reason: null, ...filedByItself(reread.proposedFilename) }])).toEqual([{ id: '32', name: 'Scan 0602.pdf' }]);
  });

  it('is seen for a document read again by itself and renamed with no read showing it waiting between', async () => {
    const look = looker(() => true);
    const invoice = sentRow(32, 'Scan 0602.pdf', 'needs_review', { ...INVOICE, reason: 'FAST_MODEL_READING' });
    const notice = sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...NOTICE, reason: 'FAST_MODEL_READING' });
    await look([invoice, notice]);
    // Read again by itself (the accurate model became usable): one is being read, the other waits its turn.
    expect(await look([{ ...invoice, status: 'analyzing', proposedFilename: null, description: null }, { ...notice, status: 'queued' }])).toEqual([]);
    const reread = sentProposal('2025-05-02 Invoice 4471 from Brightwater Supply Co.pdf', 'Invoice 4471.', 0.99);
    // The reading is ready and filed at once, on the same pass: the next read sees it being renamed, then renamed.
    expect(await look([{ ...invoice, ...reread, status: 'applying', reason: null }, { ...notice, status: 'analyzing', proposedFilename: null }])).toEqual([]);
    expect(await look([{ ...invoice, ...reread, reason: null, ...filedByItself(reread.proposedFilename) }, { ...notice, status: 'analyzing', proposedFilename: null }])).toEqual([{ id: '32', name: 'Scan 0602.pdf' }]);
    // Read again to the very name and sentence shown before, and renamed between two reads of the queue.
    expect(await look([{ ...invoice, ...reread, reason: null, ...filedByItself(reread.proposedFilename) }, { ...notice, reason: null, ...filedByItself(NOTICE.proposedFilename) }])).toEqual([{ id: '33', name: 'Scan 0603.pdf' }]);
  });

  it('is not seen for a document being renamed that is read again instead, and is seen once it is then renamed', async () => {
    const look = looker(() => true);
    const ready = sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE);
    await look([ready]);
    expect(await look([{ ...ready, status: 'applying' }])).toEqual([]);
    expect(await look([{ ...ready, status: 'queued' }])).toEqual([]);
    expect(await look([{ ...ready, ...filedByItself('2025-05-02 Invoice 4471.pdf'), proposedFilename: '2025-05-02 Invoice 4471.pdf' }])).toEqual([{ id: '32', name: 'Scan 0602.pdf' }]);
  });

  it('is not seen for documents being renamed that come back without being renamed: to review, failed, or gone', async () => {
    const look = looker(() => true);
    const rows = [31, 32, 33].map((id) => sentRow(id, `Scan 06${id - 30}.pdf`, 'ready', LETTER));
    await look(rows);
    expect(await look(rows.map((row) => ({ ...row, status: 'applying' })))).toEqual([]);
    expect(await look([{ ...rows[0], status: 'needs_review', reason: 'FILE_CHANGED' }, { ...rows[1], status: 'failed', errorCode: 'IO_ERROR' }])).toEqual([]);
  });

  it('sees two renamed at once as two', async () => {
    const look = looker(() => true);
    const rows = [sentRow(31, 'Scan 0601.pdf', 'ready', LETTER), sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE)];
    await look(rows);
    expect((await look(rows.map((row) => ({ ...row, ...filedByItself(row.proposedFilename) })))).map(({ id }) => id)).toEqual(['31', '32']);
  });

  it('is not seen for a rename made before the queue was first read, nor again once an Undo the backend refused leaves it renamed; undone, and renamed by itself again, it is seen', async () => {
    const look = looker(() => true);
    const invoice = sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE);
    const renamed = { ...invoice, ...filedByItself(INVOICE.proposedFilename) };
    // Renamed while the window was closed: completed and marked at its first read.
    expect(await look([renamed])).toEqual([]);
    // An Undo carried out passes through being renamed, unmarked, and the backend refuses it.
    expect(await look([{ ...renamed, status: 'applying', renamedWithoutApproval: false }])).toEqual([]);
    expect(await look([renamed])).toEqual([]);
    // Undone: waiting again, under UNDONE. Renamed by itself once more, that is a rename of its own.
    expect(await look([{ ...invoice, status: 'needs_review', reason: 'UNDONE' }])).toEqual([]);
    expect(await look([renamed])).toEqual([{ id: '32', name: 'Scan 0602.pdf' }]);
  });
});

describe('what is said of the renames seen', () => {
  const said = (id: string): RenameSaid => ({ id, name: `${id}.pdf` });
  const item = (id: string, status: QueueItem['status'], more: Partial<QueueItem> = {}): QueueItem => ({ id, originalFilename: `${id}.pdf`, status, ...more });

  it('adds what is new, keeps what was said, and takes out what is gone', () => {
    expect(withRenames([], [said('a')], [])).toEqual([said('a')]);
    expect(withRenames([said('a'), said('b')], [said('a'), said('c')], [])).toEqual([said('a'), said('b'), said('c')]);
    expect(withRenames([said('a'), said('b')], [], ['a'])).toEqual([said('b')]);
    expect(withRenames([said('a')], [said('b')], ['b'])).toEqual([said('a')]);
  });

  it('is what was said, the very list, when nothing changes: a document gone was never said, or one is seen again', () => {
    const was = [said('a'), said('b')];
    expect(withRenames(was, [], ['x'])).toBe(was);
    expect(withRenames(was, [said('a')], [])).toBe(was);
    expect(withRenames(was, [said('c')], [])).not.toBe(was);
  });

  it('keeps only what is still so, and offers Undo only for a rename still completed that can still be undone', () => {
    const was = ['renamed', 'undone', 'undoing', 'unmarked', 'gone', 'not-undoable', 'typed-gone'].map(said);
    const rows = [
      item('renamed', 'completed', { undoable: true, renamedWithoutApproval: true }),
      item('undone', 'review'),
      // An Undo of it carried out.
      item('undoing', 'processing', { stage: 'filing', cancelable: false }),
      // Completed, and not by itself as far as the backend says.
      item('unmarked', 'completed', { undoable: true }),
      item('not-undoable', 'completed', { undoable: false, renamedWithoutApproval: true }),
      item('typed-gone', 'completed', { undoable: true, renamedWithoutApproval: true }),
    ];
    // Renamed, but what was typed for it is no longer kept: discarded.
    const still = stillSo(was, rows, (id) => id !== 'typed-gone');
    expect(still.map(({ id }) => id)).toEqual(['renamed', 'undoing', 'not-undoable']);
    // Being undone, it is not offered again.
    expect(undoableRenames(still, rows)).toEqual(['renamed']);
  });

  it('says which document, that nobody approved the name, and where what was typed is, for one', () => {
    expect(renamedByItselfText([said('Scan 0601')])).toBe("Scan 0601.pdf was renamed under Intern's proposal, without anyone approving it. What you typed for it was not filed, and is kept in its details.");
  });

  it('says how many for several', () => {
    expect(renamedByItselfText([said('a'), said('b')])).toBe("2 documents with a name or a description you typed were renamed under Intern's proposals, without anyone approving them. What you typed for each was not filed, and is kept in its details.");
    expect(renamedByItselfText([])).toBe('');
  });

  it('has no long dash, and no figure but counts of documents', () => {
    const all = [[said('a')], [said('a'), said('b'), said('c')]].map(renamedByItselfText);
    for (const words of all) {
      expect(words).not.toMatch(/[\u2012-\u2015]/);
      expect(words.replace(/^\d+ documents\b/g, '')).not.toMatch(/\d/);
    }
  });
});

/*
  The window, with the backend's part played out by hand: a document typed
  for is renamed by itself (automatic renaming), and the window says so.
*/
describe('the window, when a document typed for is renamed by something other than the person', () => {
  afterEach(() => cleanup());
  const TYPED = '2025-05-01 Letter to Halvorsen about the fixtures';

  async function start(automaticRename = false) {
    const sent: Sent[] = [
      sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...LETTER, reason: 'TYPE_INFERRED, LOW_CONFIDENCE' }),
      sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE),
      sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...NOTICE, reason: 'DATE_AMBIGUOUS' }),
      sentRow(34, 'Scan 0604.pdf', 'ready', sentProposal('2025-05-04 Receipt from Fabrikam Inc.pdf', 'Receipt.', 0.99)),
    ];
    const change = (id: number | string, to: Record<string, unknown>) => { const index = sent.findIndex((row) => row.id === String(id)); sent[index] = { ...sent[index], ...to }; };
    const at = (id: number) => sent.find((row) => row.id === String(id))!;
    const desktop = desktopOver(() => sent);
    const base = createInMemoryBridge({ liveEvents: true, settings: { automaticRename } }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    let queueChanged = () => {};
    const bridge = {
      ...base,
      listItems: () => desktop.listItems(),
      // A person's approval: their name becomes the proposal, its reasons go, and it is filed.
      approve: vi.fn(async (id: string, name: string, description: string) => { change(id, { proposedFilename: name, description, reason: null, ...filed(name) }); }),
      // Kept under its own name: completed, nothing renamed, nothing to undo.
      keepOriginal: vi.fn(async (id: string) => { change(id, { status: 'completed', keptOriginal: true, undoable: false }); }),
      // As an undone rename is left: back under its own name, waiting, with UNDONE beside what it carried.
      undo: vi.fn(async (id: string) => { const row = at(Number(id)); change(id, { status: 'needs_review', reason: [row.reason, 'UNDONE'].filter(Boolean).join(', '), filedName: null, filedPath: null, undoable: false, renamedWithoutApproval: false }); }),
      // Check again on a parked document; each test says what it finds.
      retry: vi.fn(async (_id: string) => {}),
      subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); },
    };
    /** The backend does something with nobody asking, and says the queue changed. */
    const byItself = async (run: () => void) => { run(); await act(async () => { queueChanged(); await new Promise((resolve) => setTimeout(resolve, 0)); }); };
    const renaming = (id: number) => change(id, { status: 'applying' });
    const renamed = (id: number) => change(id, filedByItself(at(id).proposedFilename));
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    return { bridge, byItself, renaming, renamed, change };
  }
  const typeName = async (document: string, name = TYPED) => {
    const panel = await openDetails(document);
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: name } });
    return panel;
  };
  const toast = () => screen.queryByRole('group', { name: 'Renamed without you' });
  const said = () => screen.getByRole('status', { name: 'Action status' });
  /** What is said of one document renamed without the person. */
  const oneSaid = (name: string) => `${name} was renamed under Intern's proposal, without anyone approving it. What you typed for it was not filed, and is kept in its details.`;

  it('says nothing while the document is being renamed, then that it was renamed under Intern\'s proposal without anyone approving it, with Undo through the same path as a rename\'s Undo', async () => {
    const { bridge, byItself, renaming, renamed } = await start();
    const panel = await typeName('Scan 0601.pdf');
    fireEvent.change(within(panel).getByLabelText('Description'), { target: { value: 'Their reply to our notice.' } });

    await byItself(() => renaming(31));
    await waitFor(() => expect(queueCell('Scan 0601.pdf', 'status')).not.toHaveTextContent(/Needs review|Ready/));
    expect(toast()).toBeNull();
    expect(said()).not.toHaveTextContent(/renamed/);

    await byItself(() => renamed(31));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0601.pdf')));
    expect(said()).toHaveTextContent(oneSaid('Scan 0601.pdf'));
    // What was typed is where it always is after such a rename: set aside, read-only, in its details.
    const kept = within(panel).getByLabelText('What you had typed');
    expect(kept).toHaveValue(`${TYPED}\n\nTheir reply to our notice.`);
    expect(kept).toHaveAttribute('readonly');
    expect(bridge.approve).not.toHaveBeenCalled();
    // Where a rename's outcome is: above it, nothing else; and no ten seconds' life of its own.
    expect(document.querySelector('.toasts')?.firstElementChild).toBe(toast());

    fireEvent.click(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0601.pdf' }));
    await waitFor(() => expect(bridge.undo).toHaveBeenCalledWith('31'));
    // The same path as Undo in a rename's outcome: undone, said so, and the toast that offered it gone.
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Undid 1 rename.'));
    expect(toast()).toBeNull();
    await waitFor(() => expect(queueCell('Scan 0601.pdf', 'status')).toHaveTextContent('Needs review'));
    const again = await openDetails('Scan 0601.pdf');
    expect(within(again).getByLabelText('What you had typed')).toHaveValue(`${TYPED}\n\nTheir reply to our notice.`);
  });

  it('says nothing of a rename the window decided: the person approving what they typed', async () => {
    const { bridge } = await start();
    const panel = await typeName('Scan 0601.pdf');
    fireEvent.click(within(panel).getByRole('button', { name: /^Approve & rename/ }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('31', `${TYPED}.pdf`, expect.anything()));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Renamed 1 document.'));
    expect(toast()).toBeNull();
  });

  it('watches again a document whose approval the backend refused, and says so when it is then renamed by itself', async () => {
    const { bridge, byItself, renamed } = await start();
    bridge.approve.mockRejectedValueOnce({ code: 'STATE_CONFLICT', message: 'the queue is busy' });
    const panel = await typeName('Scan 0601.pdf');
    fireEvent.click(within(panel).getByRole('button', { name: /^Approve & rename/ }));
    await waitFor(() => expect(screen.getByRole('alert', { name: 'Action error' })).toBeInTheDocument());
    // Refused: the queue reads as it did, and what was typed is still in the fields.
    expect(within(panel).getByLabelText('Filename')).toHaveValue(TYPED);
    await byItself(() => renamed(31));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0601.pdf')));
  });

  // The person decides a ready document just as automatic renaming renames
  // it: the backend refuses the decision, since the document no longer
  // waits for one, and what was typed was not filed.
  const decidedAsRenamedByItself = async (decision: 'approve' | 'keepOriginal', button: string | RegExp) => {
    const { bridge, change } = await start(true);
    const panel = await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    bridge[decision].mockImplementationOnce(async () => {
      change(32, filedByItself(INVOICE.proposedFilename));
      throw { code: 'INVALID_TRANSITION', message: 'proposal is not reviewable' };
    });
    fireEvent.click(within(panel).getByRole('button', { name: button }));
    await waitFor(() => expect(screen.getByRole('alert', { name: 'Action error' })).toBeInTheDocument());
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf')));
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0602.pdf' })).toBeEnabled();
    expect(screen.getByLabelText('What you had typed')).toHaveValue('2025-05-02 Invoice 4471 from Brightwater\n\nInvoice from Brightwater Supply Co.');
  };

  it('says so of a document renamed by itself as the person approves it, the backend refusing the approval', async () => {
    await decidedAsRenamedByItself('approve', /^Apply rename/);
  });

  it('says so of a document renamed by itself as the person keeps it under its own name, the backend refusing that', async () => {
    await decidedAsRenamedByItself('keepOriginal', 'Keep original');
  });

  it('says nothing of a document with nothing typed for it, renamed by itself as before', async () => {
    const { byItself, renamed } = await start();
    await byItself(() => renamed(32));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument());
    expect(document.querySelector('.toasts')).toBeNull();
  });

  it('says it of two at once in one message, with one Undo for both; and a document typed for that was not renamed is left out', async () => {
    const { bridge, byItself, renamed } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    await typeName('Scan 0604.pdf', '2025-05-04 Receipt 88 from Fabrikam');
    await typeName('Scan 0603.pdf', '2025-05-03 Notice of termination to Contoso');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => { renamed(32); renamed(34); });
    await waitFor(() => expect(toast()).toHaveTextContent("2 documents with a name or a description you typed were renamed under Intern's proposals, without anyone approving them. What you typed for each was not filed, and is kept in its details."));
    expect(screen.getAllByRole('group', { name: 'Renamed without you' })).toHaveLength(1);
    fireEvent.click(within(toast()!).getByRole('button', { name: 'Undo the renames of these 2 documents' }));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Undid 2 renames.'));
    expect(bridge.undo.mock.calls.map(([id]) => id)).toEqual(['32', '34']);
  });

  it('adds a rename seen later to what it says, says nothing of a filing the backend does not mark, and leaves out what was undone since', async () => {
    const { byItself, renamed, change } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    await typeName('Scan 0603.pdf', '2025-05-03 Notice of termination to Contoso');
    await typeName('Scan 0601.pdf');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf')));
    const first = toast();
    // Filed under a name a person approved, not here (approved, so not marked): nothing more to say.
    await byItself(() => change(33, { proposedFilename: '2025-05-03 Termination notice to Contoso Ltd.pdf', ...filed('2025-05-03 Termination notice to Contoso Ltd.pdf') }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 2' })).toBeInTheDocument());
    expect(toast()).toBe(first);
    expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf'));
    // Another renamed by itself: added.
    await byItself(() => renamed(31));
    await waitFor(() => expect(toast()).toHaveTextContent("2 documents with a name or a description you typed were renamed under Intern's proposals, without anyone approving them."));
    expect(within(toast()!).getByRole('button', { name: 'Undo the renames of these 2 documents' })).toBeEnabled();
    // Undone from somewhere else: no longer said to be renamed, and not offered to Undo.
    await byItself(() => change(32, { status: 'needs_review', reason: 'UNDONE', filedName: null, filedPath: null, undoable: false, renamedWithoutApproval: false }));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0601.pdf')));
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0601.pdf' })).toBeEnabled();
    // Done takes it away.
    fireEvent.click(within(toast()!).getByRole('button', { name: 'Done' }));
    expect(toast()).toBeNull();
  });

  it('says nothing of a document being renamed that goes back to review instead', async () => {
    const { byItself, renaming, change } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renaming(32));
    await waitFor(() => expect(queueCell('Scan 0602.pdf', 'status')).not.toHaveTextContent('Ready'));
    expect(toast()).toBeNull();
    await byItself(() => change(32, { status: 'needs_review', reason: 'FILE_CHANGED' }));
    await waitFor(() => expect(queueCell('Scan 0602.pdf', 'status')).toHaveTextContent('Needs review'));
    expect(toast()).toBeNull();
  });

  it('says so of a document read again by itself and renamed at once, with no read showing it ready between, with Undo', async () => {
    const { bridge, byItself, change } = await start();
    await typeName('Scan 0601.pdf');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    // Read again by itself; the reading is ready and the backend files it on the same pass.
    await byItself(() => change(31, { status: 'analyzing', proposedFilename: null, description: null, reason: null }));
    expect(toast()).toBeNull();
    const reread = sentProposal('2025-05-01 Letter from Halvorsen Fixture Works LLC.pdf', 'Letter from Halvorsen.', 0.99);
    await byItself(() => change(31, { ...reread, status: 'applying', reason: null }));
    expect(toast()).toBeNull();
    await byItself(() => change(31, filedByItself(reread.proposedFilename)));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0601.pdf')));
    fireEvent.click(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0601.pdf' }));
    await waitFor(() => expect(bridge.undo).toHaveBeenCalledWith('31'));
  });

  it('says nothing of a document read again by itself that comes back to review, and says it once it is then renamed under the new proposal shown', async () => {
    const { byItself, change, renamed } = await start();
    await typeName('Scan 0601.pdf');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => change(31, { status: 'queued' }));
    await byItself(() => change(31, { status: 'analyzing', proposedFilename: null, description: null, reason: null }));
    const reread = sentProposal('2025-05-01 Letter from Halvorsen Fixture Works LLC.pdf', 'Letter from Halvorsen.', 0.99);
    await byItself(() => change(31, { ...reread, status: 'needs_review', reason: 'DATE_AMBIGUOUS' }));
    await waitFor(() => expect(queueCell('Scan 0601.pdf', 'status')).toHaveTextContent('Needs review'));
    expect(toast()).toBeNull();
    await byItself(() => renamed(31));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0601.pdf')));
  });

  it('never says a rename the window decided: one renamed by itself, undone from its details, then approved here', async () => {
    const { bridge, byItself, renamed } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf')));
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    const filedPanel = await openDetails('Scan 0602.pdf');
    fireEvent.click(within(filedPanel).getByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 0' })).toBeInTheDocument());
    expect(toast()).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: /^Queue, / }));
    const panel = await openDetails('Scan 0602.pdf');
    await waitFor(() => expect(queueCell('Scan 0602.pdf', 'status')).toHaveTextContent('Needs review'));
    // The person approves it here: their own rename, said in its own outcome.
    fireEvent.click(within(panel).getByRole('button', { name: /^Approve & rename/ }));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Renamed 1 document.'));
    expect(toast()).toBeNull();
    expect(screen.getAllByRole('button', { name: /^Undo/ }).map((button) => button.getAttribute('aria-label') ?? button.textContent)).toEqual(['Undo this rename']);
    expect(bridge.undo).toHaveBeenCalledTimes(1);
  });

  it('never says a rename the window decided: one renamed by itself, undone from somewhere else, then approved here', async () => {
    const { bridge, byItself, renamed, change } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toBeInTheDocument());
    await byItself(() => change(32, { status: 'needs_review', reason: 'UNDONE', filedName: null, filedPath: null, undoable: false, renamedWithoutApproval: false }));
    await waitFor(() => expect(toast()).toBeNull());
    const panel = await openDetails('Scan 0602.pdf');
    fireEvent.click(within(panel).getByRole('button', { name: /^Approve & rename/ }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Renamed 1 document.'));
    expect(toast()).toBeNull();
  });

  // The person's approval can stop halfway: the original cannot be removed
  // (open in a viewer, say), and the document is parked in review with what
  // was typed still kept. Check again then finishes their own rename. The
  // backend can say so as a refusal, or accept the approval and leave the
  // document in review.
  const renamedUndoneThenMineFinishedByCheckAgain = async (parked: 'refused' | 'accepted') => {
    const { bridge, byItself, renamed, change } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf')));
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    const filedPanel = await openDetails('Scan 0602.pdf');
    fireEvent.click(within(filedPanel).getByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 0' })).toBeInTheDocument());
    expect(toast()).toBeNull();

    fireEvent.click(screen.getByRole('button', { name: /^Queue, / }));
    const panel = await openDetails('Scan 0602.pdf');
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: '2025-05-02 Invoice 4471 from Brightwater' } });
    const mine = '2025-05-02 Invoice 4471 from Brightwater.pdf';
    bridge.approve.mockImplementationOnce(async () => {
      change(32, { status: 'needs_review', proposedFilename: mine, reason: 'SOURCE_DELETE_FAILED', errorCode: 'SOURCE_DELETE_FAILED', parked: true });
      if (parked === 'refused') throw { code: 'SOURCE_DELETE_FAILED', message: 'the original could not be removed' };
    });
    fireEvent.click(within(panel).getByRole('button', { name: /^Approve & rename/ }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('Check again'));

    bridge.retry.mockImplementationOnce(async () => { change(32, { ...filed(mine), reason: null, errorCode: null, parked: false }); });
    fireEvent.click(within(screen.getByRole('complementary', { name: 'Review item' })).getByRole('button', { name: /Check again/ }));
    await waitFor(() => expect(said()).toHaveTextContent('Checked Scan 0602.pdf: it is filed.'));
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
    // Their own rename, finished: not said to be a rename without them, under any name, and no Undo of it from such a message.
    expect(toast()).toBeNull();
    expect(bridge.retry).toHaveBeenCalledWith('32');
    expect(bridge.undo).toHaveBeenCalledTimes(1);
  };

  it('never says a rename the window decided: one renamed by itself, undone, then approved here, the approval refused halfway and finished by Check again', async () => {
    await renamedUndoneThenMineFinishedByCheckAgain('refused');
  });

  it('never says a rename the window decided: one renamed by itself, undone, then approved here, the approval left in review halfway and finished by Check again', async () => {
    await renamedUndoneThenMineFinishedByCheckAgain('accepted');
  });

  it('keeps saying a rename whose Undo from its details the backend refused: the document is still renamed', async () => {
    const { bridge, byItself, renamed } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf')));
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    const filedPanel = await openDetails('Scan 0602.pdf');
    bridge.undo.mockRejectedValueOnce({ code: 'STATE_CONFLICT', message: 'the queue is busy' });
    fireEvent.click(within(filedPanel).getByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(screen.getByRole('alert', { name: 'Action error' })).toBeInTheDocument());
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
    expect(bridge.undo).toHaveBeenCalledWith('32');
    expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument();
    expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf'));
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0602.pdf' })).toBeEnabled();
  });

  it('takes the message away once what was typed is discarded: it no longer is kept in the details', async () => {
    const { byItself, renamed } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    await typeName('Scan 0604.pdf', '2025-05-04 Receipt 88 from Fabrikam');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => { renamed(32); renamed(34); });
    await waitFor(() => expect(toast()).toHaveTextContent('2 documents with a name or a description you typed were renamed'));
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    const filedPanel = await openDetails('Scan 0602.pdf');
    fireEvent.click(within(filedPanel).getByRole('button', { name: 'Discard what I had' }));
    await waitFor(() => expect(within(filedPanel).queryByLabelText('What you had typed')).toBeNull());
    // Only the other is said now, and only it is offered to Undo.
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0604.pdf')));
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0604.pdf' })).toBeEnabled();
    const otherPanel = await openDetails('Scan 0604.pdf');
    fireEvent.click(within(otherPanel).getByRole('button', { name: 'Discard what I had' }));
    await waitFor(() => expect(toast()).toBeNull());
  });

  it('stands beside the outcome of the person\'s own action, each with its own Undo', async () => {
    const { bridge, byItself, renamed } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toBeInTheDocument());
    // Rename all ready renames the one ready document nobody typed for, and says so in its own toast.
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Renamed 1 document.'));
    expect(bridge.approve).toHaveBeenCalledTimes(1);
    expect(bridge.approve).toHaveBeenCalledWith('34', expect.anything(), expect.anything());
    expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf'));
    expect([...document.querySelector('.toasts')!.children].map((each) => each.getAttribute('aria-label'))).toEqual(['Renamed without you', 'Action result']);
    expect(within(screen.getByRole('group', { name: 'Action result' })).getByRole('button', { name: 'Undo this rename' })).toBeEnabled();
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0602.pdf' })).toBeEnabled();
  });

  /*
    The person's own rename, finished in a way the window did not see as its
    own decision. The backend files each under their approval, so it marks
    none of them (renamedWithoutApproval false); each was once said here as
    a rename without them, with an Undo of their own rename.
  */
  // Their approval parks (the PDF is open in a viewer). They correct the name
  // and approve again: the backend checks the files first, finds the first
  // rename landed, files it under their first name, and refuses the second
  // as ALREADY_FILED (Pipeline::approve, already_filed).
  for (const parkedAs of ['refused', 'accepted'] as const) {
    it(`never says the person's own rename that their second approval finished, refused as already filed (the first ${parkedAs} as parked)`, async () => {
      const { bridge, change } = await start();
      const panel = await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
      const first = '2025-05-02 Invoice 4471 from Brightwater.pdf';
      bridge.approve.mockImplementationOnce(async () => {
        change(32, { status: 'needs_review', proposedFilename: first, reason: 'SOURCE_DELETE_FAILED', errorCode: 'SOURCE_DELETE_FAILED', parked: true });
        if (parkedAs === 'refused') throw { code: 'SOURCE_DELETE_FAILED', message: 'the original could not be removed' };
      });
      fireEvent.click(within(panel).getByRole('button', { name: /^Apply rename/ }));
      await waitFor(() => expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('Check again'));
      const again = screen.getByRole('complementary', { name: 'Review item' });
      fireEvent.change(within(again).getByLabelText('Filename'), { target: { value: '2025-05-02 Invoice 4472 from Brightwater' } });
      bridge.approve.mockImplementationOnce(async () => {
        change(32, { ...filed(first), proposedFilename: first, reason: null, errorCode: null, parked: false });
        throw { code: 'ALREADY_FILED', message: `The earlier rename had already finished, so the document is filed as ${first} and your changes were not applied. Undo it to rename it again.` };
      });
      fireEvent.click(within(again).getByRole('button', { name: /^Approve & rename/ }));
      await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
      await waitFor(() => expect(screen.getByRole('alert', { name: 'Action error' })).toBeInTheDocument());
      await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument());
      await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
      expect(toast()).toBeNull();
      expect(bridge.undo).not.toHaveBeenCalled();
    });
  }

  it('never says the person\'s own rename that the backend reports failed though its reconciliation finished it under their name', async () => {
    const { bridge, change } = await start();
    const panel = await typeName('Scan 0601.pdf');
    bridge.approve.mockImplementationOnce(async (_id: string, name: string, description: string) => {
      change(31, { proposedFilename: name, description, reason: null, ...filed(name) });
      throw { code: 'APPLY_FAILED', message: 'the rename could not be confirmed' };
    });
    fireEvent.click(within(panel).getByRole('button', { name: /^Approve & rename/ }));
    await waitFor(() => expect(screen.getByRole('alert', { name: 'Action error' })).toBeInTheDocument());
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument());
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
    expect(toast()).toBeNull();
  });

  it('never says the person\'s own approval that the queue filed once it was free, though they typed again meanwhile', async () => {
    const { bridge, byItself, change } = await start();
    const panel = await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    const mine = '2025-05-02 Invoice 4471 from Brightwater.pdf';
    // Accepted and kept for the queue (APPLY_DEFERRED): ready, approved, under their name.
    bridge.approve.mockImplementationOnce(async (_id: string, name: string, description: string) => { change(32, { proposedFilename: name, description, reason: null, approved: true }); });
    fireEvent.click(within(panel).getByRole('button', { name: /^Apply rename/ }));
    await waitFor(() => expect(said()).toHaveTextContent('Scan 0602.pdf will be renamed when the queue is free.'));
    const again = await openDetails('Scan 0602.pdf');
    fireEvent.change(within(again).getByLabelText('Filename'), { target: { value: '2025-05-02 Invoice 4471 from Brightwater, corrected' } });
    await byItself(() => change(32, { ...filed(mine), approved: false }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument());
    expect(toast()).toBeNull();
  });

  it('keeps saying a rename while an Undo from its details is carried out and after the backend refuses it, and says it once', async () => {
    const { bridge, byItself, renamed, change } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf')));
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    const filedPanel = await openDetails('Scan 0602.pdf');
    let refuse: () => void = () => {};
    // The undo is journaled first: the row is applying, and marked no longer, while it runs; it then fails and is rolled back.
    bridge.undo.mockImplementationOnce(() => new Promise<void>((_resolve, reject) => {
      change(32, { status: 'applying', renamedWithoutApproval: false });
      refuse = () => { change(32, filedByItself(INVOICE.proposedFilename)); reject({ code: 'UNDO_FAILED', message: 'the file could not be moved back' }); };
    }));
    fireEvent.click(within(filedPanel).getByRole('button', { name: 'Undo' }));
    // Another document's progress has the queue read while the undo runs.
    await byItself(() => {});
    expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf'));
    await act(async () => { refuse(); await new Promise((resolve) => setTimeout(resolve, 0)); });
    await waitFor(() => expect(screen.getByRole('alert', { name: 'Action error' })).toBeInTheDocument());
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
    expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument();
    expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf'));
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0602.pdf' })).toBeEnabled();
    // Not said again: Action status carries the refusal's outcome, not the rename once more.
    expect(said()).not.toHaveTextContent('renamed under');
  });

  it('says a rename once: the same message, and nothing more announced, at later reads of the queue', async () => {
    const { byItself, renamed } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent(oneSaid('Scan 0602.pdf')));
    const first = toast();
    const announced = said().textContent;
    expect(announced).toBe(oneSaid('Scan 0602.pdf'));
    await byItself(() => {});
    await byItself(() => {});
    expect(toast()).toBe(first);
    expect(said().textContent).toBe(announced);
  });
});

/*
  It comes by itself, and can come under a pointer at rest on its way to a
  row. Its Undo and its Done are controls that act: a press on one before it
  has been there for the settle time does nothing (aimedPress), as Undo in
  a rename's outcome always was guarded. The rule is applied to the window
  here with a clock the test sets; jsdom lays nothing out, so a control has
  arrived when it is first drawn.
*/
describe('its buttons under a pointer at rest', () => {
  let now = 0;
  let stop = () => {};
  afterEach(() => { stop(); cleanup(); vi.useRealTimers(); });
  const press = (control: Element) => {
    const at = { clientX: 10, clientY: 10, button: 0, buttons: 1, pointerId: 1, isPrimary: true, pointerType: 'mouse' };
    const down = fireEvent.pointerDown(control, at);
    if (down) fireEvent.mouseDown(control, { ...at, detail: 1 });
    fireEvent.pointerUp(control, { ...at, buttons: 0 });
    if (down) fireEvent.mouseUp(control, { ...at, buttons: 0, detail: 1 });
    fireEvent.click(control, { ...at, buttons: 0, detail: 1 });
    return down;
  };

  it('are marked as controls that act, and stay until Done is pressed', () => {
    vi.useFakeTimers();
    const onDismiss = vi.fn();
    render(<Toast byItself tone="success" label="Renamed without you" action={{ label: 'Undo', name: 'Undo the rename of Scan 0601.pdf', onClick: vi.fn() }} onDismiss={onDismiss}>Scan 0601.pdf was renamed.</Toast>);
    expect(screen.getByRole('button', { name: 'Undo the rename of Scan 0601.pdf' })).toHaveAttribute('data-acts');
    expect(screen.getByRole('button', { name: 'Done' })).toHaveAttribute('data-acts');
    expect(screen.queryByRole('button', { name: 'Dismiss' })).not.toBeInTheDocument();
    act(() => { vi.advanceTimersByTime(TOAST_TIMEOUT_MS * 30); });
    expect(onDismiss).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('take no press made before the settle time, in the window, and take one made after it', async () => {
    stop = takeOnlyAimedPresses(window, () => now);
    const sent = [sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE), sentRow(34, 'Scan 0604.pdf', 'ready', NOTICE)];
    const base = createInMemoryBridge({ liveEvents: true }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    let queueChanged = () => {};
    const undo = vi.fn(async (id: string) => { const index = sent.findIndex((row) => row.id === id); sent[index] = { ...sent[index], status: 'needs_review', reason: 'UNDONE', filedName: null, filedPath: null, undoable: false, renamedWithoutApproval: false }; });
    const bridge = { ...base, listItems: () => desktopOver(() => sent).listItems(), undo, subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); } };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    const panel = await openDetails('Scan 0602.pdf');
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: '2025-05-02 Invoice 4471 from Brightwater' } });
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));

    now = 10_000;
    sent[0] = { ...sent[0], ...filedByItself(INVOICE.proposedFilename) };
    await act(async () => { queueChanged(); await new Promise((resolve) => setTimeout(resolve, 0)); });
    const toast = await screen.findByRole('group', { name: 'Renamed without you' });
    const undoButton = within(toast).getByRole('button', { name: 'Undo the rename of Scan 0602.pdf' });
    const done = within(toast).getByRole('button', { name: 'Done' });
    now = 10_000 + SETTLE_MS - 200;
    expect(press(undoButton)).toBe(false);
    expect(press(done)).toBe(false);
    expect(undo).not.toHaveBeenCalled();
    expect(screen.getByRole('group', { name: 'Renamed without you' })).toBe(toast);

    now = 10_000 + SETTLE_MS + 400;
    expect(press(undoButton)).toBe(true);
    await waitFor(() => expect(undo).toHaveBeenCalledWith('32'));
  });
});
