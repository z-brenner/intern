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
import type { RenameSeen, Waiting } from './renamedByItself';

/*
  Intern can rename by itself a document whose name or description a person
  typed in Details and did not file, and a document can be filed from
  somewhere else. What was typed is set aside beside it, read-only. The
  window now says so, and offers Undo for a rename it saw complete. The rows
  here are as the desktop backend sends them, read through TauriBridge
  (sentTesting): its statuses ("needs_review", "applying"), its reasons as
  one list, and null where it has nothing.
*/

const LETTER = sentProposal('2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', 'Letter to Halvorsen Fixture Works LLC.', 1);
const INVOICE = sentProposal('2025-05-02 Invoice from Brightwater Supply Co.pdf', 'Invoice from Brightwater Supply Co.', 0.98);
const NOTICE = sentProposal('2025-05-03 Notice to Contoso Ltd.pdf', 'Notice to Contoso Ltd.', 0.9);
/** Filed as the backend reports a filing: completed, with the receipt's name and path, and undoable. */
const filed = (name: unknown) => ({ status: 'completed', filedName: name, filedPath: `C:\\Filed\\${String(name)}`, keptOriginal: false, undoable: true });

/** One look at the queue, as the window takes one each time it is read. */
function looker(typed: (id: string) => boolean) {
  let watched: ReadonlyMap<string, Waiting> = new Map();
  return async (rows: Sent[], deciding?: string) => {
    const look = watchRenames(watched, await asTheWindowReads(rows), { typed, decidedHere: (id) => id === deciding });
    watched = look.watched;
    return { seen: look.seen, back: look.back };
  };
}
const nothing = { seen: [], back: [] };

describe('a rename over what was typed, seen in the queue as it is read', () => {
  const typedFor = (...ids: string[]) => (id: string) => ids.includes(id);

  it('is seen as the document leaves review to be renamed, and again once it is renamed, under Intern\'s proposal', async () => {
    const look = looker(typedFor('31'));
    const waiting = sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...LETTER, reason: 'TYPE_INFERRED, LOW_CONFIDENCE' });
    expect(await look([waiting])).toEqual(nothing);
    // Judged again at Balanced: ready, with its reasons as notes. Not renamed yet.
    expect(await look([{ ...waiting, status: 'ready' }])).toEqual(nothing);
    expect(await look([{ ...waiting, status: 'applying' }])).toEqual({ seen: [{ id: '31', name: 'Scan 0601.pdf', whose: 'proposal', done: false }], back: [] });
    // Still being renamed: said once.
    expect(await look([{ ...waiting, status: 'applying' }])).toEqual(nothing);
    expect(await look([{ ...waiting, ...filed(LETTER.proposedFilename) }])).toEqual({ seen: [{ id: '31', name: 'Scan 0601.pdf', whose: 'proposal', done: true }], back: [] });
    // Once said renamed, it is not watched any more.
    expect(await look([{ ...waiting, ...filed(LETTER.proposedFilename) }])).toEqual(nothing);
  });

  it('is seen when a ready document is renamed between two reads, under a name with a number the folder needed', async () => {
    const look = looker(typedFor('32'));
    const ready = sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE);
    await look([ready]);
    expect(await look([{ ...ready, ...filed('2025-05-02 Invoice from Brightwater Supply Co (2).pdf') }])).toEqual({ seen: [{ id: '32', name: 'Scan 0602.pdf', whose: 'proposal', done: true }], back: [] });
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
    const { seen } = await look(rows.map((row) => ({ ...row, ...filed(row.proposedFilename) })));
    expect(seen.map(({ id }) => id)).toEqual(['31', '32']);
  });

  it('is not seen for a document with nothing typed for it', async () => {
    const look = looker(typedFor());
    const ready = sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE);
    await look([ready]);
    expect(await look([{ ...ready, status: 'applying' }])).toEqual(nothing);
    expect(await look([{ ...ready, ...filed(INVOICE.proposedFilename) }])).toEqual(nothing);
  });

  it('is not seen for the document the window is deciding for, and is again once a decision the backend refused leaves it waiting', async () => {
    const look = looker(typedFor('31', '33'));
    const letter = sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...LETTER, reason: 'DATE_AMBIGUOUS' });
    const notice = sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...NOTICE, reason: 'DATE_AMBIGUOUS' });
    await look([letter, notice]);
    // The person approves the letter here, with what they typed: being renamed, renamed. The window's own.
    expect(await look([{ ...letter, status: 'applying' }, notice], '31')).toEqual(nothing);
    expect(await look([{ ...letter, ...filed('2025-05-01 Letter to Halvorsen about the fixtures.pdf') }, notice], '31')).toEqual(nothing);
    expect(await look([{ ...letter, ...filed('2025-05-01 Letter to Halvorsen about the fixtures.pdf') }, notice])).toEqual(nothing);
    // An approval of the notice that the backend refused: it waits again, and is renamed later by something else.
    expect(await look([notice], '33')).toEqual(nothing);
    expect(await look([notice])).toEqual(nothing);
    expect((await look([{ ...notice, ...filed(NOTICE.proposedFilename) }])).seen).toEqual([{ id: '33', name: 'Scan 0603.pdf', whose: 'proposal', done: true }]);
  });

  it('says "from somewhere else" when the window cannot tell whose name it was', async () => {
    const look = looker(() => true);
    const rows = [
      sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...LETTER, reason: 'DATE_AMBIGUOUS' }),
      sentRow(32, 'Scan 0602.pdf', 'ready', { ...INVOICE, approved: true }),
      sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...NOTICE, reason: 'DATE_AMBIGUOUS' }),
      sentRow(34, 'NEEDS REVIEW - Lease scan.pdf', 'needs_review', { errorCode: 'PASSWORD_PROTECTED' }),
      sentRow(35, 'Scan 0605.pdf', 'ready', LETTER),
    ];
    await look(rows);
    const { seen } = await look([
      // Filed under another name than the one proposed.
      { ...rows[0], ...filed('2025-05-01 Letter from Halvorsen.pdf'), proposedFilename: '2025-05-01 Letter from Halvorsen.pdf' },
      // Approved by someone, not here: whatever name it has, it is not one the window saw proposed.
      { ...rows[1], ...filed(INVOICE.proposedFilename), approved: false },
      // The same name, another sentence.
      { ...rows[2], ...filed(NOTICE.proposedFilename), description: 'A notice of termination.' },
      // Intern had no name for it at all.
      { ...rows[3], ...filed('2026-01-02 Lease.pdf'), proposedFilename: '2026-01-02 Lease.pdf' },
      // And one under Intern's proposal, beside them.
      { ...rows[4], ...filed(LETTER.proposedFilename) },
    ]);
    expect(seen.map(({ id, whose }) => `${id} ${whose}`)).toEqual(['31 elsewhere', '32 elsewhere', '33 elsewhere', '34 elsewhere', '35 proposal']);
  });

  it('is not seen for a completion that renamed nothing, nor for a document read again; one read again, shown waiting with its new proposal, and then renamed is seen under that proposal', async () => {
    const look = looker(() => true);
    const letter = sentRow(31, 'Scan 0601.pdf', 'ready', LETTER);
    const invoice = sentRow(32, 'Scan 0602.pdf', 'needs_review', { ...INVOICE, reason: 'LOW_CONFIDENCE' });
    await look([letter, invoice]);
    // The name was the document's own already: completed, nothing renamed.
    expect(await look([{ ...letter, status: 'completed', keptOriginal: true, reason: 'ALREADY_NAMED' }, { ...invoice, status: 'analyzing', proposedFilename: null }])).toEqual(nothing);
    const reread = sentProposal('2025-05-02 Invoice 4471 from Brightwater Supply Co.pdf', 'Invoice 4471.', 0.99);
    expect(await look([{ ...letter, status: 'completed', keptOriginal: true }, { ...invoice, ...reread, status: 'ready', reason: null }])).toEqual(nothing);
    expect((await look([{ ...letter, status: 'completed', keptOriginal: true }, { ...invoice, ...reread, reason: null, ...filed(reread.proposedFilename) }])).seen).toEqual([{ id: '32', name: 'Scan 0602.pdf', whose: 'proposal', done: true }]);
  });

  it('is seen for a document read again by itself and renamed with no read showing it waiting between: from somewhere else under a proposal never shown, under Intern\'s proposal under the one shown before', async () => {
    const look = looker(() => true);
    const invoice = sentRow(32, 'Scan 0602.pdf', 'needs_review', { ...INVOICE, reason: 'FAST_MODEL_READING' });
    const notice = sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...NOTICE, reason: 'FAST_MODEL_READING' });
    await look([invoice, notice]);
    // Read again by itself (the accurate model became usable): one is being read, the other waits its turn.
    expect(await look([{ ...invoice, status: 'analyzing', proposedFilename: null, description: null }, { ...notice, status: 'queued' }])).toEqual(nothing);
    const reread = sentProposal('2025-05-02 Invoice 4471 from Brightwater Supply Co.pdf', 'Invoice 4471.', 0.99);
    // The reading is ready and filed at once, on the same pass: the next read sees it being renamed, then renamed.
    expect(await look([{ ...invoice, ...reread, status: 'applying', reason: null }, { ...notice, status: 'analyzing', proposedFilename: null }])).toEqual({ seen: [{ id: '32', name: 'Scan 0602.pdf', whose: 'elsewhere', done: false }], back: [] });
    expect(await look([{ ...invoice, ...reread, reason: null, ...filed(reread.proposedFilename) }, { ...notice, status: 'analyzing', proposedFilename: null }])).toEqual({ seen: [{ id: '32', name: 'Scan 0602.pdf', whose: 'elsewhere', done: true }], back: [] });
    // Read again to the very name and sentence shown before, and renamed between two reads of the queue.
    expect(await look([{ ...invoice, ...reread, reason: null, ...filed(reread.proposedFilename) }, { ...notice, reason: null, ...filed(NOTICE.proposedFilename) }])).toEqual({ seen: [{ id: '33', name: 'Scan 0603.pdf', whose: 'proposal', done: true }], back: [] });
  });

  it('takes back a document seen being renamed that is read again instead, and keeps watching it', async () => {
    const look = looker(() => true);
    const ready = sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE);
    await look([ready]);
    expect((await look([{ ...ready, status: 'applying' }])).seen).toHaveLength(1);
    expect(await look([{ ...ready, status: 'queued' }])).toEqual({ seen: [], back: ['32'] });
    expect((await look([{ ...ready, ...filed('2025-05-02 Invoice 4471.pdf'), proposedFilename: '2025-05-02 Invoice 4471.pdf' }])).seen).toEqual([{ id: '32', name: 'Scan 0602.pdf', whose: 'elsewhere', done: true }]);
  });

  it('says which documents seen being renamed came back without being renamed: to review, failed, or gone', async () => {
    const look = looker(() => true);
    const rows = [31, 32, 33].map((id) => sentRow(id, `Scan 06${id - 30}.pdf`, 'ready', LETTER));
    await look(rows);
    expect((await look(rows.map((row) => ({ ...row, status: 'applying' })))).seen).toHaveLength(3);
    expect(await look([{ ...rows[0], status: 'needs_review', reason: 'FILE_CHANGED' }, { ...rows[1], status: 'failed', errorCode: 'IO_ERROR' }])).toEqual({ seen: [], back: ['31', '32', '33'] });
  });

  it('sees two renamed at once as two', async () => {
    const look = looker(() => true);
    const rows = [sentRow(31, 'Scan 0601.pdf', 'ready', LETTER), sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE)];
    await look(rows);
    expect((await look(rows.map((row) => ({ ...row, ...filed(row.proposedFilename) })))).seen.map(({ id, done }) => `${id} ${done}`)).toEqual(['31 true', '32 true']);
  });
});

describe('what is said of the renames seen', () => {
  const seen = (id: string, done = true, whose: RenameSeen['whose'] = 'proposal'): RenameSeen => ({ id, name: `${id}.pdf`, whose, done });
  const item = (id: string, status: QueueItem['status'], more: Partial<QueueItem> = {}): QueueItem => ({ id, originalFilename: `${id}.pdf`, status, ...more });

  it('adds what is new, puts renamed in place of being renamed, and takes out what came back', () => {
    expect(withRenames([], [seen('a', false)], [])).toEqual([seen('a', false)]);
    expect(withRenames([seen('a', false), seen('b')], [seen('a'), seen('c')], [])).toEqual([seen('a'), seen('b'), seen('c')]);
    expect(withRenames([seen('a', false), seen('b')], [], ['a'])).toEqual([seen('b')]);
  });

  it('is what was said, the very list, when nothing changes: a document that comes back was never said to be renamed, or one is seen again as it was', () => {
    const said = [seen('a'), seen('b', false)];
    expect(withRenames(said, [], ['x'])).toBe(said);
    expect(withRenames(said, [seen('a')], [])).toBe(said);
    expect(withRenames(said, [seen('b')], [])).not.toBe(said);
  });

  it('keeps only what is still so, and offers Undo only for a rename seen complete that can still be undone', () => {
    const said = [seen('renamed'), seen('undone'), seen('renaming', false), seen('since-renamed', false), seen('gone'), seen('not-undoable'), seen('typed-gone')];
    const rows = [
      item('renamed', 'completed', { undoable: true }),
      item('undone', 'review'),
      item('renaming', 'processing', { stage: 'filing', cancelable: false }),
      item('since-renamed', 'completed', { undoable: true }),
      item('not-undoable', 'completed', { undoable: false }),
      item('typed-gone', 'completed', { undoable: true }),
    ];
    // Renamed, but what was typed for it is no longer kept: discarded, or filed by a decision here.
    const still = stillSo(said, rows, (id) => id !== 'typed-gone');
    expect(still.map(({ id }) => id)).toEqual(['renamed', 'renaming', 'since-renamed', 'not-undoable']);
    // Seen only being renamed, it is not offered: the window did not see that rename complete.
    expect(undoableRenames(still, rows)).toEqual(['renamed']);
  });

  it('says which document, whose name, and where what was typed is, for one', () => {
    expect(renamedByItselfText([seen('Scan 0601')])).toBe("Scan 0601.pdf was renamed under Intern's proposal. What you typed for it was not filed, and is kept in its details.");
    expect(renamedByItselfText([seen('Scan 0601', true, 'elsewhere')])).toBe('Scan 0601.pdf was renamed from somewhere else. What you typed for it was not filed, and is kept in its details.');
    expect(renamedByItselfText([seen('Scan 0601', false)])).toBe("Scan 0601.pdf is being renamed under Intern's proposal. What you typed for it is not filed, and is kept in its details.");
  });

  it('says how many, and whose names, for several', () => {
    expect(renamedByItselfText([seen('a'), seen('b')])).toBe("2 documents with a name or a description you typed were renamed under Intern's proposals. What you typed for each was not filed, and is kept in its details.");
    expect(renamedByItselfText([seen('a', true, 'elsewhere'), seen('b', true, 'elsewhere')])).toBe('2 documents with a name or a description you typed were renamed from somewhere else. What you typed for each was not filed, and is kept in its details.');
    expect(renamedByItselfText([seen('a'), seen('b', true, 'elsewhere'), seen('c', true, 'elsewhere')])).toBe("3 documents with a name or a description you typed were renamed, 1 under Intern's proposal and 2 from somewhere else. What you typed for each was not filed, and is kept in its details.");
    expect(renamedByItselfText([seen('a', false), seen('b', false)])).toBe("2 documents with a name or a description you typed are being renamed under Intern's proposals. What you typed for each is not filed, and is kept in its details.");
    expect(renamedByItselfText([seen('a'), seen('b', false)])).toBe("2 documents with a name or a description you typed were renamed or are being renamed under Intern's proposals. What you typed for each is not filed, and is kept in its details.");
    expect(renamedByItselfText([])).toBe('');
  });

  it('has no long dash, and no figure but counts of documents', () => {
    const all = [[seen('a')], [seen('a', false, 'elsewhere')], [seen('a'), seen('b', false, 'elsewhere'), seen('c')]].map(renamedByItselfText);
    for (const words of all) {
      expect(words).not.toMatch(/[\u2012-\u2015]/);
      expect(words.replace(/^\d+ documents\b|\b\d+ (under|from)\b/g, '')).not.toMatch(/\d/);
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

  async function start() {
    const sent: Sent[] = [
      sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...LETTER, reason: 'TYPE_INFERRED, LOW_CONFIDENCE' }),
      sentRow(32, 'Scan 0602.pdf', 'ready', INVOICE),
      sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...NOTICE, reason: 'DATE_AMBIGUOUS' }),
      sentRow(34, 'Scan 0604.pdf', 'ready', sentProposal('2025-05-04 Receipt from Fabrikam Inc.pdf', 'Receipt.', 0.99)),
    ];
    const change = (id: number | string, to: Record<string, unknown>) => { const index = sent.findIndex((row) => row.id === String(id)); sent[index] = { ...sent[index], ...to }; };
    const at = (id: number) => sent.find((row) => row.id === String(id))!;
    const desktop = desktopOver(() => sent);
    const base = createInMemoryBridge({ liveEvents: true }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    let queueChanged = () => {};
    const bridge = {
      ...base,
      listItems: () => desktop.listItems(),
      // A person's approval: their name becomes the proposal, its reasons go, and it is filed.
      approve: vi.fn(async (id: string, name: string, description: string) => { change(id, { proposedFilename: name, description, reason: null, ...filed(name) }); }),
      // As an undone rename is left: back under its own name, waiting, with UNDONE beside what it carried.
      undo: vi.fn(async (id: string) => { const row = at(Number(id)); change(id, { status: 'needs_review', reason: [row.reason, 'UNDONE'].filter(Boolean).join(', '), filedName: null, filedPath: null, undoable: false }); }),
      subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); },
    };
    /** The backend does something with nobody asking, and says the queue changed. */
    const byItself = async (run: () => void) => { run(); await act(async () => { queueChanged(); await new Promise((resolve) => setTimeout(resolve, 0)); }); };
    const renaming = (id: number) => change(id, { status: 'applying' });
    const renamed = (id: number) => change(id, filed(at(id).proposedFilename));
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

  it('says the document is being renamed, then that it was renamed under Intern\'s proposal, with Undo through the same path as a rename\'s Undo', async () => {
    const { bridge, byItself, renaming, renamed } = await start();
    const panel = await typeName('Scan 0601.pdf');
    fireEvent.change(within(panel).getByLabelText('Description'), { target: { value: 'Their reply to our notice.' } });

    await byItself(() => renaming(31));
    await waitFor(() => expect(toast()).toHaveTextContent("Scan 0601.pdf is being renamed under Intern's proposal. What you typed for it is not filed, and is kept in its details."));
    expect(said()).toHaveTextContent("Scan 0601.pdf is being renamed under Intern's proposal.");
    // Not yet renamed: no Undo.
    expect(within(toast()!).queryByRole('button', { name: /^Undo/ })).not.toBeInTheDocument();

    await byItself(() => renamed(31));
    await waitFor(() => expect(toast()).toHaveTextContent("Scan 0601.pdf was renamed under Intern's proposal. What you typed for it was not filed, and is kept in its details."));
    expect(said()).toHaveTextContent("Scan 0601.pdf was renamed under Intern's proposal.");
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
    await waitFor(() => expect(toast()).toHaveTextContent("Scan 0601.pdf was renamed under Intern's proposal."));
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
    await waitFor(() => expect(toast()).toHaveTextContent("2 documents with a name or a description you typed were renamed under Intern's proposals. What you typed for each was not filed, and is kept in its details."));
    expect(screen.getAllByRole('group', { name: 'Renamed without you' })).toHaveLength(1);
    fireEvent.click(within(toast()!).getByRole('button', { name: 'Undo the renames of these 2 documents' }));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Undid 2 renames.'));
    expect(bridge.undo.mock.calls.map(([id]) => id)).toEqual(['32', '34']);
  });

  it('adds a rename seen later to what it says, says "from somewhere else" for a name it cannot place, and leaves out what was undone since', async () => {
    const { byItself, renamed, change } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    await typeName('Scan 0603.pdf', '2025-05-03 Notice of termination to Contoso');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent("Scan 0602.pdf was renamed under Intern's proposal."));
    // Filed from somewhere else, under a name the window never saw proposed.
    await byItself(() => change(33, { proposedFilename: '2025-05-03 Termination notice to Contoso Ltd.pdf', ...filed('2025-05-03 Termination notice to Contoso Ltd.pdf') }));
    await waitFor(() => expect(toast()).toHaveTextContent("2 documents with a name or a description you typed were renamed, 1 under Intern's proposal and 1 from somewhere else."));
    expect(within(toast()!).getByRole('button', { name: 'Undo the renames of these 2 documents' })).toBeEnabled();
    // Undone from somewhere else: no longer said to be renamed, and not offered to Undo.
    await byItself(() => change(32, { status: 'needs_review', reason: 'UNDONE', filedName: null, filedPath: null, undoable: false }));
    await waitFor(() => expect(toast()).toHaveTextContent('Scan 0603.pdf was renamed from somewhere else.'));
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0603.pdf' })).toBeEnabled();
    // Done takes it away.
    fireEvent.click(within(toast()!).getByRole('button', { name: 'Done' }));
    expect(toast()).toBeNull();
  });

  it('takes back what it said of a document seen being renamed that went back to review', async () => {
    const { byItself, renaming, change } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renaming(32));
    await waitFor(() => expect(toast()).toHaveTextContent('Scan 0602.pdf is being renamed'));
    await byItself(() => change(32, { status: 'needs_review', reason: 'FILE_CHANGED' }));
    await waitFor(() => expect(toast()).toBeNull());
  });

  it('says so of a document read again by itself and renamed at once, with no read showing it ready between: from somewhere else, and Undo once renamed', async () => {
    const { bridge, byItself, change } = await start();
    await typeName('Scan 0601.pdf');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    // Read again by itself; the reading is ready and the backend files it on the same pass.
    await byItself(() => change(31, { status: 'analyzing', proposedFilename: null, description: null, reason: null }));
    expect(toast()).toBeNull();
    const reread = sentProposal('2025-05-01 Letter from Halvorsen Fixture Works LLC.pdf', 'Letter from Halvorsen.', 0.99);
    await byItself(() => change(31, { ...reread, status: 'applying', reason: null }));
    await waitFor(() => expect(toast()).toHaveTextContent('Scan 0601.pdf is being renamed from somewhere else. What you typed for it is not filed, and is kept in its details.'));
    expect(within(toast()!).queryByRole('button', { name: /^Undo/ })).not.toBeInTheDocument();
    await byItself(() => change(31, filed(reread.proposedFilename)));
    await waitFor(() => expect(toast()).toHaveTextContent('Scan 0601.pdf was renamed from somewhere else. What you typed for it was not filed, and is kept in its details.'));
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
    await waitFor(() => expect(toast()).toHaveTextContent("Scan 0601.pdf was renamed under Intern's proposal."));
  });

  it('never says a rename the window decided: one renamed by itself, undone from its details, then approved here', async () => {
    const { bridge, byItself, renamed } = await start();
    await typeName('Scan 0602.pdf', '2025-05-02 Invoice 4471 from Brightwater');
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await byItself(() => renamed(32));
    await waitFor(() => expect(toast()).toHaveTextContent("Scan 0602.pdf was renamed under Intern's proposal."));
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
    await byItself(() => change(32, { status: 'needs_review', reason: 'UNDONE', filedName: null, filedPath: null, undoable: false }));
    await waitFor(() => expect(toast()).toBeNull());
    const panel = await openDetails('Scan 0602.pdf');
    fireEvent.click(within(panel).getByRole('button', { name: /^Approve & rename/ }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Renamed 1 document.'));
    expect(toast()).toBeNull();
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
    await waitFor(() => expect(toast()).toHaveTextContent("Scan 0604.pdf was renamed under Intern's proposal. What you typed for it was not filed, and is kept in its details."));
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
    expect(toast()).toHaveTextContent("Scan 0602.pdf was renamed under Intern's proposal.");
    expect([...document.querySelector('.toasts')!.children].map((each) => each.getAttribute('aria-label'))).toEqual(['Renamed without you', 'Action result']);
    expect(within(screen.getByRole('group', { name: 'Action result' })).getByRole('button', { name: 'Undo this rename' })).toBeEnabled();
    expect(within(toast()!).getByRole('button', { name: 'Undo the rename of Scan 0602.pdf' })).toBeEnabled();
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
    const undo = vi.fn(async (id: string) => { const index = sent.findIndex((row) => row.id === id); sent[index] = { ...sent[index], status: 'needs_review', reason: 'UNDONE', filedName: null, filedPath: null, undoable: false }; });
    const bridge = { ...base, listItems: () => desktopOver(() => sent).listItems(), undo, subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); } };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    const panel = await openDetails('Scan 0602.pdf');
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: '2025-05-02 Invoice 4471 from Brightwater' } });
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));

    now = 10_000;
    sent[0] = { ...sent[0], ...filed(INVOICE.proposedFilename) };
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
