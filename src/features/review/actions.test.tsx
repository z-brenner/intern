import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ReviewInspector } from '../../components/ReviewInspector';
import type { QueueItem } from '../../types';
import { nextUndecided, undecidedOrder } from './actions';

function handlers() {
  return {
    onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(),
    onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn(),
  };
}

/** A button's accessible name: its label, or its text without the hidden parts (icons, key hints). */
function nameOf(button: HTMLElement): string {
  return button.getAttribute('aria-label') ?? [...button.childNodes]
    .filter((node) => !(node instanceof Element && node.getAttribute('aria-hidden') === 'true'))
    .map((node) => node.textContent).join('').trim();
}

/** Every action the inspector offers for `item`, with its More menu opened. */
function actionSet(item: QueueItem): string[] {
  render(<ReviewInspector item={item} drawer={false} {...handlers()} />);
  const more = screen.queryByRole('button', { name: 'More review actions' });
  if (more) fireEvent.click(more);
  const names = screen.getAllByRole('button').map(nameOf).filter((name) => name !== 'Close review' && name !== 'Copy description');
  cleanup();
  return names;
}

const proposal = { proposedFilename: '2024-03-01 Lease Agreement.pdf', confidence: 0.8, description: 'A lease.' };
const review: QueueItem = { id: 'review', originalFilename: 'scan.pdf', status: 'review', ...proposal, reason: 'The model reported low confidence in its own proposal.', errorCode: 'LOW_CONFIDENCE' };

describe('the actions each status offers', () => {
  afterEach(() => cleanup());

  // FRONTEND_UX-3 and FRONTEND_UX-4: Retry on every review item, which the
  // backend refuses for all but three codes, and nothing at all to keep or
  // remove a ready item, or to drop one waiting item.
  it('action_set_per_status', () => {
    const open = ['Open', 'Show in folder'];
    expect(actionSet({ ...review, id: 'ready', status: 'ready', reason: undefined, errorCode: undefined }))
      .toEqual([...open, 'Apply rename', 'Keep original', 'More review actions', 'Analyze again', 'Remove from queue']);
    expect(actionSet({ id: 'waiting', originalFilename: 'scan.pdf', status: 'waiting' }))
      .toEqual(['Remove from queue']);
    expect(actionSet(review))
      .toEqual([...open, 'Approve & rename', 'Keep original', 'More review actions', 'Analyze again', 'Remove from queue']);
    // Its files need checking, so checking is the main action; keeping and
    // reading it again are refused until then. Approving is accepted - the
    // backend checks the files first - so the name can still be approved.
    expect(actionSet({ ...review, parked: true, errorCode: 'FILE_CHANGED' }))
      .toEqual([...open, 'Check again', 'Approve & rename', 'Remove from queue']);
    // A backend from before `parked` existed: the codes that always meant it.
    expect(actionSet({ ...review, errorCode: 'RECONCILIATION_REQUIRED' }))
      .toEqual([...open, 'Check again', 'Approve & rename', 'Remove from queue']);
    // No proposal, nothing to approve: only the check and the confirmed remove.
    expect(actionSet({ id: 'parked', originalFilename: 'scan.pdf', status: 'review', parked: true, errorCode: 'RECONCILIATION_REQUIRED' }))
      .toEqual([...open, 'Check again', 'Remove from queue']);
    // Flagged before analysis, so there is no proposal to approve. Reading
    // it from the start is accepted too (Pipeline::reanalyze takes any ready
    // or review item whose files are settled).
    expect(actionSet({ id: 'duplicate', originalFilename: 'copy.pdf', status: 'review', reason: 'Duplicate of 2024-03-01 Lease Agreement.pdf', errorCode: 'DUPLICATE' }))
      .toEqual([...open, 'Keep original', 'More review actions', 'Process anyway', 'Analyze again', 'Remove from queue']);
    // Held because nobody could vouch for its uploader: found in its folder,
    // but not opened in its program from Intern's window.
    expect(actionSet({ id: 'unverified', originalFilename: 'upload.pptm', status: 'review', errorCode: 'UPLOADER_UNVERIFIED' }))
      .toEqual(['Show in folder', 'Keep original', 'More review actions', 'Check again', 'Analyze again', 'Remove from queue']);
    // A rename another program refused by holding the file is rolled back
    // and retried by approving again once it is closed; Retry is refused.
    expect(actionSet({ ...review, errorCode: 'SOURCE_LOCKED', reason: 'The document is open in another program.' }))
      .toEqual([...open, 'Approve & rename', 'Keep original', 'More review actions', 'Analyze again', 'Remove from queue']);
    expect(actionSet({ id: 'failed', originalFilename: 'broken.pdf', status: 'failed', reason: 'Extraction failed.' }))
      .toEqual(['Retry item', 'Remove item']);
    expect(actionSet({ id: 'active', originalFilename: 'scan.pdf', status: 'processing', stage: 'reading' }))
      .toEqual(['Cancel processing']);
    expect(actionSet({ id: 'applying', originalFilename: 'scan.pdf', status: 'processing', stage: 'filing', cancelable: false }))
      .toEqual([]);
    expect(actionSet({ ...review, id: 'completed', status: 'completed', reason: undefined, errorCode: undefined, filedName: proposal.proposedFilename, undoable: true }))
      .toEqual([...open, 'Undo']);
    // Kept under its own name: nothing was renamed, so nothing can be undone.
    expect(actionSet({ ...review, id: 'kept', status: 'completed', reason: undefined, errorCode: undefined, keptOriginal: true, undoable: false }))
      .toEqual(open);
  });

  it('never offers Retry for an ordinary review reason', () => {
    for (const errorCode of [undefined, 'LOW_CONFIDENCE', 'DATE_UNSUPPORTED', 'TYPE_INFERRED', 'NEAR_DUPLICATE', 'UNDONE', 'FILE_CHANGED', 'SOURCE_LOCKED', 'DESTINATION_UNAVAILABLE']) {
      const offered = actionSet({ ...review, errorCode });
      expect(offered, String(errorCode)).toContain('Analyze again');
      expect(offered.filter((name) => /retry|again|anyway/i.test(name) && name !== 'Analyze again'), String(errorCode)).toEqual([]);
    }
  });

  it('wires each menu entry to the action it names', () => {
    const calls = handlers();
    render(<ReviewInspector item={review} drawer={false} {...calls} />);
    fireEvent.click(screen.getByRole('button', { name: 'More review actions' }));
    fireEvent.click(screen.getByRole('button', { name: 'Analyze again' }));
    expect(calls.onReanalyze).toHaveBeenCalledOnce();
    expect(calls.onRetry).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'More review actions' }));
    fireEvent.click(screen.getByRole('button', { name: 'Remove from queue' }));
    expect(calls.onRemove).toHaveBeenCalledWith(false);

    fireEvent.click(screen.getByRole('button', { name: 'Open' }));
    fireEvent.click(screen.getByRole('button', { name: 'Show in folder' }));
    expect(calls.onOpen).toHaveBeenCalledOnce();
    expect(calls.onReveal).toHaveBeenCalledOnce();
  });

  // Pipeline::approve checks a parked item's files before it approves, so
  // the name can be approved from the panel; checking stays the main action.
  it('leads a parked item with Check again, and still lets its name be approved', () => {
    const calls = handlers();
    render(<ReviewInspector item={{ ...review, parked: true, errorCode: 'FILE_CHANGED' }} drawer={false} {...calls} />);

    expect(screen.getByRole('button', { name: 'Check again' })).toHaveClass('primary');
    const approve = screen.getByRole('button', { name: /Approve & rename/ });
    expect(approve).not.toHaveClass('primary');
    fireEvent.change(screen.getByRole('textbox', { name: 'Filename' }), { target: { value: '2024-03-01 Lease' } });
    fireEvent.click(approve);
    expect(calls.onApprove).toHaveBeenCalledWith('2024-03-01 Lease.pdf', 'A lease.');
    expect(calls.onRetry).not.toHaveBeenCalled();
  });

  // QUEUE_CORE-8: a parked item refused every action. With WP-07 it can be
  // removed, once the person says they put the files right themselves.
  it('removes a parked item only after the person says they resolved its files', () => {
    const calls = handlers();
    render(<ReviewInspector item={{ ...review, parked: true }} drawer={false} {...calls} />);

    fireEvent.click(screen.getByRole('button', { name: 'Remove from queue' }));
    expect(calls.onRemove).not.toHaveBeenCalled();
    const question = screen.getByRole('group', { name: 'Confirm removal' });
    expect(question).toHaveTextContent('Remove it only once you have');
    const confirm = within(question).getByRole('button', { name: 'I have resolved the files myself' });
    expect(confirm).toHaveFocus();

    fireEvent.keyDown(confirm, { key: 'Escape' });
    expect(screen.queryByRole('group', { name: 'Confirm removal' })).not.toBeInTheDocument();
    expect(calls.onClose).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Remove from queue' }));
    fireEvent.click(screen.getByRole('button', { name: 'I have resolved the files myself' }));
    expect(calls.onRemove).toHaveBeenCalledWith(true);

    fireEvent.click(screen.getByRole('button', { name: 'Check again' }));
    expect(calls.onRetry).toHaveBeenCalledOnce();
  });
});

describe('the order review works through', () => {
  const item = (id: string, status: QueueItem['status'], extra: Partial<QueueItem> = {}): QueueItem => ({ id, originalFilename: `${id}.pdf`, status, ...extra });
  const rows = [
    item('a', 'ready'), item('b', 'review'), item('c', 'waiting'), item('d', 'review', { parked: true }),
    item('e', 'review'), item('f', 'ready'), item('g', 'completed'),
  ];

  it('takes review items first, then ready ones, each in table order, and skips what cannot be decided', () => {
    expect(undecidedOrder(rows).map(({ id }) => id)).toEqual(['b', 'e', 'a', 'f']);
    // Ready because it was approved already, and waiting only for the queue.
    expect(undecidedOrder([...rows, item('h', 'ready', { approved: true })]).map(({ id }) => id)).toEqual(['b', 'e', 'a', 'f']);
  });

  it('goes on from the item just decided, and comes back round to any skipped', () => {
    const before = undecidedOrder(rows);
    const after = (decided: string) => rows.map((row) => row.id === decided ? { ...row, status: 'completed' as const } : row);
    expect(nextUndecided(before, 'b', after('b'))?.id).toBe('e');
    expect(nextUndecided(before, 'e', after('e'))?.id).toBe('a');
    expect(nextUndecided(before, 'f', after('f'))?.id).toBe('b');
    expect(nextUndecided([item('b', 'review')], 'b', [item('b', 'completed')])).toBeUndefined();
  });
});
