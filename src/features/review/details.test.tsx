import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge, createReviewBatchBridge } from '../../lib/inMemoryBridge';
import type { QueueByItself } from '../../lib/inMemoryBridge';
import { clickCell, detailsToggle, queueCell } from '../queue/gridTesting';

/*
  The Details panel opens and closes only by its own buttons: the toolbar's
  Details toggle and the panel's close button (and Enter or F2 on the
  selected item, which are the toggle's keys). Selecting a row or a cell used
  to open it; now that only selects, and the panel, while open, follows the
  selection.
*/
describe('the Details panel', () => {
  const panel = () => screen.queryByRole('complementary', { name: 'Review item' });
  const start = async () => {
    const base = createReviewBatchBridge();
    const bridge = { ...base, approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal), remove: vi.fn(base.remove) };
    const view = render(<App bridge={bridge} />);
    await waitFor(() => expect(queueCell('Scan 0101.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    return { bridge, view };
  };

  it('starts closed, and opens and closes from its toggle and its own close button', async () => {
    await start();
    // Intern selected the first item to decide, and opened nothing.
    expect(panel()).not.toBeInTheDocument();
    const toggle = detailsToggle();
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(toggle).not.toHaveAttribute('aria-controls');

    fireEvent.click(toggle);
    expect(panel()).toHaveTextContent('Scan 0101.pdf');
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    expect(toggle).toHaveAttribute('aria-controls', panel()!.id);
    // Wide, the panel is beside the queue, not over it: nothing moves focus.
    expect(screen.getByRole('region', { name: 'Queue items' })).not.toHaveAttribute('inert');

    // While open it follows the selection, and focus stays in the table.
    clickCell(queueCell('Scan 0103.pdf', 'description'));
    expect(panel()).toHaveTextContent('Scan 0103.pdf');
    expect(queueCell('Scan 0103.pdf', 'description')).toHaveFocus();

    // Its close button closes it, and focus goes back to the cell being
    // worked on now - not the toggle that first opened the panel; the row
    // stays selected.
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Close review' }));
    expect(panel()).not.toBeInTheDocument();
    await waitFor(() => expect(queueCell('Scan 0103.pdf', 'description')).toHaveFocus());
    expect(queueCell('Scan 0103.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');

    // Closed from the toggle, focus stays on the toggle.
    fireEvent.click(toggle);
    expect(panel()).toHaveTextContent('Scan 0103.pdf');
    toggle.focus();
    fireEvent.click(toggle);
    expect(panel()).not.toBeInTheDocument();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(toggle).toHaveFocus();
  });

  it('is remembered open or closed from one launch to the next', async () => {
    const first = await start();
    fireEvent.click(detailsToggle());
    expect(panel()).toBeInTheDocument();
    first.view.unmount();

    const second = await start();
    expect(panel()).toHaveTextContent('Scan 0101.pdf');
    fireEvent.click(detailsToggle());
    second.view.unmount();

    await start();
    expect(panel()).not.toBeInTheDocument();
  });

  it('still opens when the preference cannot be stored', async () => {
    const setItem = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('Storage is disabled.'); });
    const getItem = vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('Storage is disabled.'); });
    try {
      await start();
      fireEvent.click(detailsToggle());
      expect(panel()).toBeInTheDocument();
    } finally {
      setItem.mockRestore();
      getItem.mockRestore();
    }
  });

  // The mouse had no way to open the panel from the row itself: click the
  // row, then find the Details button in the toolbar. A double-click is the
  // mouse's Enter.
  it('opens from a double-click on a row, at the name, and a single click still only selects', async () => {
    await start();
    const cell = queueCell('Scan 0102.pdf', 'proposed');
    // A double-click as the browser delivers it: two presses, the second
    // counted second, then dblclick. The second press opens it.
    clickCell(cell, { detail: 1 });
    expect(panel()).not.toBeInTheDocument();
    clickCell(cell, { detail: 2 });
    expect(panel()).toHaveTextContent('Scan 0102.pdf');
    // The dblclick that follows opens nothing more.
    fireEvent.doubleClick(cell, { button: 0, detail: 2 });

    expect(panel()).toHaveTextContent('Scan 0102.pdf');
    await waitFor(() => expect(within(panel()!).getByLabelText('Filename')).toHaveFocus());
    expect(within(panel()!).getByLabelText('Filename')).toHaveValue('2024-02-09 Invoice INV-3301 from Fabrikam Inc');
    expect(detailsToggle()).toHaveAttribute('aria-expanded', 'true');
    expect(queueCell('Scan 0102.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    // The button says how else it opens.
    expect(detailsToggle()).toHaveAttribute('title', expect.stringMatching(/Enter, F2 or a double-click/));

    // Open already, a double-click on another row's Status goes to that row's name.
    const status = queueCell('Scan 0104.pdf', 'status');
    clickCell(status);
    fireEvent.doubleClick(status, { button: 0, detail: 2 });
    await waitFor(() => expect(within(panel()!).getByLabelText('Filename')).toHaveValue('2024-03-30 Statement of Work with Litware Inc'));
    await waitFor(() => expect(within(panel()!).getByLabelText('Filename')).toHaveFocus());

    // A row with no name to edit opens at the panel's heading.
    const waiting = queueCell('Scan 0107.pdf');
    clickCell(waiting);
    fireEvent.doubleClick(waiting, { button: 0, detail: 2 });
    await waitFor(() => expect(within(panel()!).getByRole('heading', { name: 'Review item' })).toHaveFocus());
    expect(panel()).toHaveTextContent('Scan 0107.pdf');
  });

  // A double-click is two presses at one place, and the queue can move under
  // it: a document filed or read above, a toolbar or a progress strip that
  // grows by a line. The browser's dblclick goes where the rows are by then -
  // another row, or, when they moved under the second press, the table body.
  // The row the first press was on is the one opened.
  it('opens the row its first press was on when the rows move under the second', async () => {
    await start();
    clickCell(queueCell('Scan 0103.pdf', 'description'), { detail: 1 });
    // The rows moved: the second press lands on the row below.
    clickCell(queueCell('Scan 0104.pdf', 'description'), { detail: 2 });
    expect(panel()).toHaveTextContent('Scan 0103.pdf');
    expect(queueCell('Scan 0103.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    expect(queueCell('Scan 0104.pdf').closest('tr')).not.toHaveAttribute('aria-current');
    expect(document.querySelectorAll('.queue-grid [aria-selected="true"]')).toHaveLength(1);
    expect(queueCell('Scan 0103.pdf', 'description')).toHaveAttribute('aria-selected', 'true');
    // The dblclick, sent to the row that had come under the pointer, opens nothing else.
    fireEvent.doubleClick(queueCell('Scan 0104.pdf', 'description'), { button: 0, detail: 2 });
    expect(panel()).toHaveTextContent('Scan 0103.pdf');
  });

  it('opens from a dblclick that reached only the table body, for the row first pressed', async () => {
    await start();
    // Something that sends dblclick without counting its presses, the
    // release having landed on another row: only the body hears it.
    clickCell(queueCell('Scan 0105.pdf', 'check'), { detail: 1 });
    fireEvent.doubleClick(queueCell('Scan 0105.pdf').closest('tbody')!, { button: 0 });
    expect(panel()).toHaveTextContent('Scan 0105.pdf');
  });

  it('opens nothing when the row first pressed has left the queue before the second press', async () => {
    let queue: QueueByItself | undefined;
    render(<App bridge={createReviewBatchBridge({ liveEvents: true, byItself: (changes) => { queue = changes; } })} />);
    await waitFor(() => expect(queueCell('Scan 0101.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    clickCell(queueCell('Scan 0106.pdf'), { detail: 1 });
    // Filed from another machine between the presses: the second lands on the row that took its place.
    queue!.leave('scan-0106');
    await waitFor(() => expect(screen.queryByRole('gridcell', { name: 'Scan 0106.pdf' })).not.toBeInTheDocument());
    clickCell(queueCell('Scan 0107.pdf'), { detail: 2 });
    expect(panel()).not.toBeInTheDocument();
    // That press only selected where it landed.
    expect(queueCell('Scan 0107.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
  });

  it('opens nothing from a second press when the first was somewhere else', async () => {
    await start();
    clickCell(queueCell('Scan 0103.pdf'), { detail: 1 });
    fireEvent.mouseDown(document.body, { button: 0, detail: 1 });
    clickCell(queueCell('Scan 0105.pdf'), { detail: 2 });
    expect(panel()).not.toBeInTheDocument();
  });

  // With nothing selected the toggle cannot open anything, and says how a row's details are opened.
  it('says how to open a row from the Details toggle while nothing is selected', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    await screen.findByRole('grid', { name: 'Queue' });
    fireEvent.click(screen.getByRole('button', { name: /^Completed/ }));
    // Nothing the window selects by itself at launch lands on a row this view does not show.
    await waitFor(() => expect(detailsToggle()).toBeDisabled());
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(detailsToggle()).toBeDisabled();
    expect(detailsToggle()).toHaveAttribute('title', 'Double-click a row to see its details');
    const filed = queueCell('Completed lease.pdf', 'proposed');
    clickCell(filed, { detail: 1 });
    clickCell(filed, { detail: 2 });
    expect(panel()).toHaveTextContent('Completed lease.pdf');
  });

  // How to open a row is said below the table until a row has been opened from it once.
  it('says how to open a row until one has been opened that way', async () => {
    const first = await start();
    const hint = () => screen.queryByText('Double-click a row to see its details');
    expect(hint()).toBeInTheDocument();
    // Opening from the toggle teaches nothing about the rows.
    fireEvent.click(detailsToggle());
    expect(hint()).not.toBeInTheDocument();
    fireEvent.click(detailsToggle());
    expect(hint()).toBeInTheDocument();

    const cell = queueCell('Scan 0103.pdf');
    clickCell(cell, { detail: 1 });
    clickCell(cell, { detail: 2 });
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Close review' }));
    expect(hint()).not.toBeInTheDocument();
    first.view.unmount();

    await start();
    expect(hint()).not.toBeInTheDocument();
  });

  it('does not open from a double-click that was extending a selection, or one of another button', async () => {
    const { bridge } = await start();
    clickCell(queueCell('Scan 0101.pdf'));
    const cell = queueCell('Scan 0103.pdf', 'description');
    clickCell(cell, { shiftKey: true });
    fireEvent.doubleClick(cell, { button: 0, shiftKey: true });
    fireEvent.doubleClick(cell, { button: 0, ctrlKey: true });
    fireEvent.doubleClick(cell, { button: 1 });

    expect(panel()).not.toBeInTheDocument();
    expect(screen.getByRole('status', { name: 'Cell selection' })).toHaveTextContent('3 rows by 3 columns selected');
    expect(bridge.approve).not.toHaveBeenCalled();
  });

  it('opens from Enter on a cell, at the name', async () => {
    await start();
    const cell = queueCell('Scan 0102.pdf', 'check');
    clickCell(cell);
    fireEvent.keyDown(cell, { key: 'Enter' });

    expect(panel()).toHaveTextContent('Scan 0102.pdf');
    await waitFor(() => expect(within(panel()!).getByLabelText('Filename')).toHaveFocus());
    expect(detailsToggle()).toHaveAttribute('aria-expanded', 'true');
  });

  // The decision keys press the panel's buttons; with the panel closed there
  // are none to press, so a key cannot do what a click could not.
  it('decides nothing from the keyboard while closed', async () => {
    const { bridge } = await start();
    const cell = queueCell('Scan 0101.pdf');
    cell.focus();

    fireEvent.keyDown(cell, { key: 'Enter', ctrlKey: true });
    fireEvent.keyDown(cell, { key: 'k', altKey: true });
    fireEvent.keyDown(cell, { key: 'Delete' });

    expect(bridge.approve).not.toHaveBeenCalled();
    expect(bridge.keepOriginal).not.toHaveBeenCalled();
    expect(bridge.remove).not.toHaveBeenCalled();
    expect(panel()).not.toBeInTheDocument();
    expect(screen.queryByRole('group', { name: 'Confirm removal' })).not.toBeInTheDocument();

    // J still moves the selection with the panel closed.
    fireEvent.keyDown(cell, { key: 'j' });
    await waitFor(() => expect(queueCell('Scan 0102.pdf')).toHaveFocus());
    expect(panel()).not.toBeInTheDocument();
  });

  it('cannot be opened with nothing selected', async () => {
    render(<App bridge={createReviewBatchBridge({ items: [{ id: 'w', originalFilename: 'Scan 0900.pdf', status: 'waiting' }] })} />);
    await screen.findByRole('grid', { name: 'Queue' });
    expect(detailsToggle()).toBeDisabled();
    clickCell(queueCell('Scan 0900.pdf'));
    expect(detailsToggle()).toBeEnabled();
  });

  // The cell selection follows the item the panel describes wherever that
  // changes outside the table, so Enter and Ctrl+C act on the row on screen
  // in the panel, not one left behind.
  it('moves the active cell to the next item when a decision moves on', async () => {
    const { bridge } = await start();
    fireEvent.click(detailsToggle());
    clickCell(queueCell('Scan 0101.pdf', 'description'));
    clickCell(queueCell('Scan 0103.pdf', 'description'), { shiftKey: true });
    expect(document.querySelectorAll('.queue-grid [aria-selected="true"]')).toHaveLength(3);

    fireEvent.click(within(panel()!).getByRole('button', { name: /Approve & rename/ }), { detail: 1 });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('scan-0101', expect.any(String), expect.any(String)));
    await waitFor(() => expect(panel()).toHaveTextContent('Scan 0102.pdf'));
    // The cells chosen around the old row are gone; Tab now lands on the
    // new item's row, in the same column.
    expect(document.querySelectorAll('.queue-grid [aria-selected="true"]')).toHaveLength(0);
    expect(queueCell('Scan 0102.pdf', 'description')).toHaveAttribute('tabindex', '0');
    const data = new Map<string, string>();
    const event = new Event('copy', { bubbles: true, cancelable: true }) as ClipboardEvent;
    Object.defineProperty(event, 'clipboardData', { value: { setData: (type: string, value: string) => data.set(type, value) } });
    queueCell('Scan 0102.pdf', 'description').focus();
    document.activeElement!.dispatchEvent(event);
    expect(data.get('text/plain')).toBe('Invoice INV-3301 from Fabrikam Inc. for February consulting hours.');
  });

  it('moves the active cell to a document just added', async () => {
    const base = createReviewBatchBridge();
    const resolveDrop = vi.fn(async () => ({ files: [{ path: 'browser://Scan 0900.pdf', displayName: 'Scan 0900.pdf' }] }));
    render(<App bridge={base} selection={{ pickFiles: async () => [], pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop }} />);
    await waitFor(() => expect(queueCell('Scan 0101.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    clickCell(queueCell('Scan 0103.pdf', 'proposed'));

    fireEvent.drop(screen.getByRole('region', { name: /drag files/i }));
    await waitFor(() => expect(queueCell('Scan 0900.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    expect(queueCell('Scan 0900.pdf', 'proposed')).toHaveAttribute('tabindex', '0');
    expect(document.querySelectorAll('.queue-grid [aria-selected="true"]')).toHaveLength(0);
  });

  // Delete removes the one document the panel describes. With cells of
  // several rows selected it would read as removing all of them.
  it('ignores Delete while cells of several rows are selected', async () => {
    const { bridge } = await start();
    fireEvent.click(detailsToggle());
    clickCell(queueCell('Scan 0101.pdf'));
    clickCell(queueCell('Scan 0102.pdf'), { shiftKey: true });
    fireEvent.keyDown(document.activeElement!, { key: 'Delete' });
    expect(within(panel()!).queryByRole('group', { name: 'Confirm removal' })).not.toBeInTheDocument();
    expect(bridge.remove).not.toHaveBeenCalled();
    // Not silently: the key says why it did nothing.
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Select one row to remove it.');

    // One row's cells: Delete asks about that row's document, as ever.
    clickCell(queueCell('Scan 0102.pdf', 'proposed'), { shiftKey: false });
    clickCell(queueCell('Scan 0102.pdf', 'description'), { shiftKey: true });
    fireEvent.keyDown(document.activeElement!, { key: 'Delete' });
    expect(within(panel()!).getByRole('group', { name: 'Confirm removal' })).toHaveTextContent('Scan 0102.pdf');
  });

  // Narrow, the drawer is opened for the moment. Opening or closing it used
  // to overwrite whether the wide panel opens at the next launch.
  it('leaves the remembered wide preference alone in a narrow window', async () => {
    vi.stubGlobal('matchMedia', (query: string) => ({ matches: query === '(max-width: 1100px)', media: query, onchange: null, addEventListener: vi.fn(), removeEventListener: vi.fn(), addListener: vi.fn(), removeListener: vi.fn(), dispatchEvent: vi.fn() }));
    try {
      window.localStorage.setItem('intern.detailsPanel', 'closed');
      const first = await start();
      fireEvent.click(detailsToggle());
      expect(await screen.findByRole('dialog', { name: 'Review item' })).toBeInTheDocument();
      expect(window.localStorage.getItem('intern.detailsPanel')).toBe('closed');
      first.view.unmount();

      window.localStorage.setItem('intern.detailsPanel', 'open');
      await start();
      fireEvent.click(detailsToggle());
      fireEvent.keyDown(await screen.findByRole('dialog', { name: 'Review item' }), { key: 'Escape' });
      expect(screen.queryByRole('dialog', { name: 'Review item' })).not.toBeInTheDocument();
      expect(window.localStorage.getItem('intern.detailsPanel')).toBe('open');
    } finally {
      vi.unstubAllGlobals();
    }
  });
});
