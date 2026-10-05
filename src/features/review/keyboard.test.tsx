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
