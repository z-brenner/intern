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
    await waitFor(() => expect(queueCell('Scan 0101.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
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

    // Its close button closes it, back to the toggle; the row stays selected.
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Close review' }));
    expect(panel()).not.toBeInTheDocument();
    await waitFor(() => expect(toggle).toHaveFocus());
    expect(queueCell('Scan 0103.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');

    fireEvent.click(toggle);
    expect(panel()).toHaveTextContent('Scan 0103.pdf');
    fireEvent.click(toggle);
    expect(panel()).not.toBeInTheDocument();
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
    expect(panel()).not.toBeInTheDocument();

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
});
