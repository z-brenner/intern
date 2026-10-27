import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createReviewBatchBridge } from '../../lib/inMemoryBridge';
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

  it('opens from Enter on a cell, at the name', async () => {
    await start();
    const cell = queueCell('Scan 0102.pdf', 'confidence');
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
