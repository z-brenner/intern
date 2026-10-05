import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createReviewBatchBridge } from '../../lib/inMemoryBridge';

/*
  FRONTEND_UX-10. With 200 items queued, a keyboard user reached Approve from
  a selected row after about 197 presses of Tab, and after each decision focus
  went back to the toolbar, so the next item had to be found again. The batch
  is seven scans: needing review 0101, 0102, 0104 and 0106; ready 0103 and
  0105; and 0107 waiting.
*/
describe('keyboard review', () => {
  const inspector = () => screen.getByRole('complementary', { name: 'Review item' });
  const rowButton = (name: string) => screen.getByRole('button', { name: `Select ${name}` });
  const press = (key: string, init: Partial<KeyboardEventInit> = {}) => fireEvent.keyDown(document.activeElement ?? document.body, { key, ...init });
  const start = async () => {
    const base = createReviewBatchBridge();
    const bridge = { ...base, approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal), remove: vi.fn(base.remove) };
    render(<App bridge={bridge} />);
    await waitFor(() => expect(inspector()).toHaveTextContent('Scan 0101.pdf'));
    return bridge;
  };

  it('jk_moves_selection', async () => {
    await start();

    // One row is in the tab order, the selected one, so Tab leaves the table
    // in one press instead of one per row.
    expect(rowButton('Scan 0101.pdf')).toHaveAttribute('tabindex', '0');
    expect(rowButton('Scan 0102.pdf')).toHaveAttribute('tabindex', '-1');

    press('j');
    await waitFor(() => expect(rowButton('Scan 0102.pdf')).toHaveFocus());
    expect(inspector()).toHaveTextContent('Scan 0102.pdf');
    expect(rowButton('Scan 0102.pdf')).toHaveAttribute('tabindex', '0');
    expect(rowButton('Scan 0101.pdf')).toHaveAttribute('tabindex', '-1');

    press('J');
    await waitFor(() => expect(rowButton('Scan 0103.pdf')).toHaveFocus());
    press('k');
    await waitFor(() => expect(rowButton('Scan 0102.pdf')).toHaveFocus());
    press('ArrowDown');
    await waitFor(() => expect(rowButton('Scan 0103.pdf')).toHaveFocus());
    press('ArrowUp');
    await waitFor(() => expect(rowButton('Scan 0102.pdf')).toHaveFocus());
    expect(inspector()).toHaveTextContent('Scan 0102.pdf');

    // In a field a letter is a letter.
    const filename = within(inspector()).getByLabelText('Filename');
    filename.focus();
    press('j');
    press('ArrowDown');
    expect(filename).toHaveFocus();
    expect(inspector()).toHaveTextContent('Scan 0102.pdf');
  });

  it('ctrl_enter_approves', async () => {
    const bridge = await start();

    press('Enter', { ctrlKey: true });

    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('scan-0101', '2024-02-01 Engagement Letter with Northwind Traders.pdf', 'Engagement letter between Northwind Traders and the firm for 2024 advisory work.'));
    // From inside the name too, where Enter alone already files it: once, not twice.
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveFocus());
    press('Enter', { ctrlKey: true });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
    expect(bridge.approve).toHaveBeenLastCalledWith('scan-0102', '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', expect.any(String));
    expect(screen.getByRole('button', { name: /Approve & rename/ })).toHaveAttribute('aria-keyshortcuts', 'Control+Enter');
  });

  // The panel stays mounted as review moves on, and the name being edited
  // used to be put back to the new item's in a passive effect - a task after
  // the commit that showed the new item. A key pressed in between, as a held
  // or quick second Ctrl+Enter or Enter is, approved the item now on screen
  // under the name and description of the one just decided. The observer
  // presses it at exactly that moment: the commit that brings the next item.
  it('a key pressed as the next item appears approves it under its own name', async () => {
    const bridge = await start();
    const pressAsItAppears = (name: string, key: KeyboardEventInit) => {
      const observer = new MutationObserver(() => {
        if (!inspector().textContent?.includes(name)) return;
        observer.disconnect();
        (document.activeElement ?? document.body).dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...key }));
      });
      observer.observe(document.body, { subtree: true, childList: true, characterData: true });
      return observer;
    };

    // From the keyboard shortcut, with focus where the seeded selection left it.
    const first = pressAsItAppears('Scan 0102.pdf', { key: 'Enter', ctrlKey: true });
    press('Enter', { ctrlKey: true });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
    first.disconnect();
    expect(bridge.approve).toHaveBeenNthCalledWith(1, 'scan-0101', '2024-02-01 Engagement Letter with Northwind Traders.pdf', 'Engagement letter between Northwind Traders and the firm for 2024 advisory work.');
    expect(bridge.approve).toHaveBeenNthCalledWith(2, 'scan-0102', '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', 'Invoice INV-3301 from Fabrikam Inc. for February consulting hours.');

    // From Enter in the name itself, which still has focus as the next item arrives.
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveValue('2024-03-30 Statement of Work with Litware Inc'));
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveFocus());
    const second = pressAsItAppears('Scan 0106.pdf', { key: 'Enter' });
    press('Enter');
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(4));
    second.disconnect();
    expect(bridge.approve).toHaveBeenNthCalledWith(3, 'scan-0104', '2024-03-30 Statement of Work with Litware Inc.pdf', 'Statement of work with Litware Inc. for the spring data migration.');
    expect(bridge.approve).toHaveBeenNthCalledWith(4, 'scan-0106', '2024-04-18 Lease Amendment for 500 Pine St.pdf', 'First amendment to the lease of 500 Pine St.');
  });

  // Review moves on as soon as the backend answers, which is quicker than a
  // double-click, and the next item's Approve is enabled in the same place.
  // The second click used to approve a document nobody had looked at.
  it('the second click of a double-click does not decide the next item', async () => {
    const bridge = await start();
    const approve = () => within(inspector()).getByRole('button', { name: /Approve & rename/ });

    fireEvent.click(approve(), { detail: 1 });
    await waitFor(() => expect(inspector()).toHaveTextContent('Scan 0102.pdf'));
    await waitFor(() => expect(approve()).toBeEnabled());
    fireEvent.click(approve(), { detail: 2 });
    fireEvent.click(within(inspector()).getByRole('button', { name: /Keep original/ }), { detail: 2 });

    expect(bridge.approve).toHaveBeenCalledTimes(1);
    expect(bridge.keepOriginal).not.toHaveBeenCalled();
    expect(inspector()).toHaveTextContent('Scan 0102.pdf');

    // A click of its own still decides.
    fireEvent.click(approve(), { detail: 1 });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
    expect(bridge.approve).toHaveBeenLastCalledWith('scan-0102', '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', expect.any(String));
  });

  // The same for a key held down: its repeats arrive once the caret is in
  // the next item's name, and each one decided another document.
  it('a held key decides one item, not the ones after it', async () => {
    const bridge = await start();
    const filename = () => within(inspector()).getByLabelText('Filename');
    press('Enter', { ctrlKey: true });
    await waitFor(() => expect(filename()).toHaveValue('2024-02-09 Invoice INV-3301 from Fabrikam Inc'));
    await waitFor(() => expect(filename()).toHaveFocus());

    // Swallowed rather than let through: a repeated Enter would put a space in the name.
    for (const key of [{ key: 'Enter', ctrlKey: true }, { key: 'Enter' }, { key: 'k', altKey: true }]) {
      expect(fireEvent.keyDown(filename(), { ...key, repeat: true }), JSON.stringify(key)).toBe(false);
    }
    expect(filename()).toHaveValue('2024-02-09 Invoice INV-3301 from Fabrikam Inc');
    // Enter held on a button presses it again with each repeat.
    const approve = within(inspector()).getByRole('button', { name: /Approve & rename/ });
    approve.focus();
    expect(fireEvent.keyDown(approve, { key: 'Enter', repeat: true })).toBe(false);
    rowButton('Scan 0102.pdf').focus();
    fireEvent.keyDown(rowButton('Scan 0102.pdf'), { key: 'Delete', repeat: true });
    expect(within(inspector()).queryByRole('group', { name: 'Confirm removal' })).not.toBeInTheDocument();
    expect(bridge.approve).toHaveBeenCalledTimes(1);
    expect(bridge.keepOriginal).not.toHaveBeenCalled();

    // A press of its own still decides.
    press('Enter', { ctrlKey: true });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
    expect(bridge.approve).toHaveBeenLastCalledWith('scan-0102', '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', expect.any(String));
  });

  it('approve_advances_to_next_undecided_and_announces', async () => {
    const bridge = await start();
    fireEvent.click(rowButton('Scan 0102.pdf'));
    await waitFor(() => expect(inspector()).toHaveTextContent('Scan 0102.pdf'));
    expect(inspector()).toHaveTextContent('2 of 6 to decide');

    fireEvent.click(within(inspector()).getByRole('button', { name: /Approve & rename/ }));

    // The next one still to decide after it - review first, then ready -
    // with the caret already in its name.
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveValue('2024-03-30 Statement of Work with Litware Inc'));
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveFocus());
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed Scan 0102.pdf. Next: Scan 0104.pdf.');
    expect(inspector()).toHaveTextContent('2 of 5 to decide');
    expect(bridge.approve).toHaveBeenCalledTimes(1);

    // Keep and remove are decisions too.
    press('k', { altKey: true });
    await waitFor(() => expect(bridge.keepOriginal).toHaveBeenCalledWith('scan-0104'));
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Kept Scan 0104.pdf under its own name. Next: Scan 0106.pdf.'));
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveFocus());

    // One keystroke is too easy to press by accident: Delete asks first.
    rowButton('Scan 0106.pdf').focus();
    press('Delete');
    const confirm = await within(inspector()).findByRole('group', { name: 'Confirm removal' });
    expect(bridge.remove).not.toHaveBeenCalled();
    fireEvent.click(within(confirm).getByRole('button', { name: 'Remove from queue' }));
    await waitFor(() => expect(bridge.remove).toHaveBeenCalledWith('scan-0106'));
    // The last needing review hands on to the first ready one.
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Removed Scan 0106.pdf from the queue. Next: Scan 0103.pdf.'));
  });

  // While a batch is still being read, the backend files an approved name
  // between documents and the approval comes back ready. It is decided all
  // the same: review counted it as still to decide, and came back round to it.
  it('an approval the queue files later is not left to decide again', async () => {
    const base = createReviewBatchBridge({ deferApprovalsWhileBusy: true, items: [
      { id: 'a', originalFilename: 'Scan 0301.pdf', status: 'review', proposedFilename: '2024-06-01 Letter from Northwind Traders.pdf', description: 'A letter.', reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE' },
      { id: 'b', originalFilename: 'Scan 0302.pdf', status: 'review', proposedFilename: '2024-06-02 Invoice from Fabrikam Inc.pdf', description: 'An invoice.', reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE' },
      { id: 'busy', originalFilename: 'Scan 0303.pdf', status: 'processing', stage: 'reading' },
    ] });
    const bridge = { ...base, approve: vi.fn(base.approve) };
    render(<App bridge={bridge} />);
    await waitFor(() => expect(inspector()).toHaveTextContent('Scan 0301.pdf'));
    expect(inspector()).toHaveTextContent('1 of 2 to decide');
    const status = screen.getByRole('status', { name: 'Action status' });

    press('Enter', { ctrlKey: true });

    await waitFor(() => expect(status).toHaveTextContent('Scan 0301.pdf will be renamed when the queue is free. Next: Scan 0302.pdf.'));
    expect(inspector()).toHaveTextContent('1 of 1 to decide');
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveFocus());

    press('Enter', { ctrlKey: true });

    // Nothing is left to decide, so review goes back to the queue rather
    // than round to the first approval.
    await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument());
    expect(status).toHaveTextContent('Scan 0302.pdf will be renamed when the queue is free.');
    expect(status).not.toHaveTextContent('Next');
    expect(bridge.approve).toHaveBeenCalledTimes(2);

    // Opened again, it says why it is still in the queue.
    fireEvent.click(rowButton('Scan 0301.pdf'));
    await waitFor(() => expect(inspector()).toHaveTextContent('Approved. It will be renamed when the queue is free.'));
    expect(inspector()).not.toHaveTextContent('to decide');
  });

  // The queue can start filing a deferred approval the moment it frees up,
  // and the read after the command then shows it processing: a rename under
  // way, not one that needs review again.
  it('an approval already being filed is not called sent back', async () => {
    const base = createReviewBatchBridge();
    let filing = false;
    const bridge = {
      ...base,
      approve: vi.fn(async () => { filing = true; }),
      listItems: async () => (await base.listItems()).map((item) => filing && item.id === 'scan-0101'
        ? { ...item, status: 'processing' as const, stage: 'filing' as const, cancelable: false, reason: undefined, errorCode: undefined }
        : item),
    };
    render(<App bridge={bridge} />);
    await waitFor(() => expect(inspector()).toHaveTextContent('Scan 0101.pdf'));

    press('Enter', { ctrlKey: true });

    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Scan 0101.pdf is being renamed. Next: Scan 0102.pdf.'));
    expect(inspector()).toHaveTextContent('Scan 0102.pdf');
  });

  it('goes back to the queue when nothing is left to decide', async () => {
    const base = createReviewBatchBridge({ items: [
      { id: 'only', originalFilename: 'Scan 0201.pdf', status: 'review', proposedFilename: '2024-05-01 Letter.pdf', reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE' },
      { id: 'later', originalFilename: 'Scan 0202.pdf', status: 'waiting' },
    ] });
    render(<App bridge={base} />);
    await waitFor(() => expect(inspector()).toHaveTextContent('Scan 0201.pdf'));
    expect(inspector()).not.toHaveTextContent('Next undecided');

    press('Enter', { ctrlKey: true });

    await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument());
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed Scan 0201.pdf.');
    // Not <body>: the row was on its way out and the toolbar still disabled
    // when the target used to be chosen.
    await waitFor(() => expect(screen.getByRole('region', { name: 'Queue items' })).toContainElement(document.activeElement as HTMLElement));
  });

  it('enter_and_f2_go_to_the_name_and_slash_to_the_filter', async () => {
    await start();
    const filename = () => within(inspector()).getByLabelText('Filename');

    // A row clicked in the wide window: focus goes to the panel's heading,
    // not a row count of Tab presses away from it.
    fireEvent.click(rowButton('Scan 0104.pdf'));
    await waitFor(() => expect(within(inspector()).getByRole('heading', { name: 'Review item' })).toHaveFocus());
    press('F2');
    expect(filename()).toHaveFocus();
    expect(filename()).toHaveValue('2024-03-30 Statement of Work with Litware Inc');

    rowButton('Scan 0102.pdf').focus();
    press('Enter');
    await waitFor(() => expect(filename()).toHaveFocus());
    expect(filename()).toHaveValue('2024-02-09 Invoice INV-3301 from Fabrikam Inc');

    rowButton('Scan 0102.pdf').focus();
    press('/');
    expect(screen.getByRole('searchbox', { name: 'Filter queue' })).toHaveFocus();
  });

  it('shows where the item stands and goes to the next undecided one', async () => {
    await start();
    expect(inspector()).toHaveTextContent('1 of 6 to decide');

    fireEvent.click(within(inspector()).getByRole('button', { name: 'Next undecided' }));

    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveValue('2024-02-09 Invoice INV-3301 from Fabrikam Inc'));
    await waitFor(() => expect(within(inspector()).getByLabelText('Filename')).toHaveFocus());
    expect(inspector()).toHaveTextContent('2 of 6 to decide');

    // A waiting item is not one to decide: the count stands, without a place in it.
    fireEvent.click(rowButton('Scan 0107.pdf'));
    await waitFor(() => expect(inspector()).toHaveTextContent('Scan 0107.pdf'));
    expect(inspector()).toHaveTextContent('6 to decide');
    expect(inspector()).not.toHaveTextContent('of 6 to decide');
  });

  it('does nothing while a dialog is open over the queue', async () => {
    const bridge = await start();
    fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    await screen.findByRole('dialog', { name: 'Settings' });

    fireEvent.keyDown(document.body, { key: 'j' });
    fireEvent.keyDown(document.body, { key: 'Enter', ctrlKey: true });

    expect(inspector()).toHaveTextContent('Scan 0101.pdf');
    expect(bridge.approve).not.toHaveBeenCalled();
  });
});
