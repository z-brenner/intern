import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge, createReviewBatchBridge } from '../../lib/inMemoryBridge';
import type { QueueItem } from '../../types';
import { clickCell, detailsToggle, openDetails, queueCell } from './gridTesting';

/*
  The queue table behaves like a spreadsheet's cells: click, Shift+click,
  drag, Ctrl+A and the arrow keys select; Ctrl+C and the Copy button copy the
  selection as tab-separated text and as an HTML table, so it pastes into
  Excel or Google Sheets one value to a cell. Status is never selected or
  copied. None of it opens the Details panel.
*/

const tricky: QueueItem[] = [
  { id: 'one', originalFilename: 'Scan 1.pdf', status: 'review', proposedFilename: '2024-01-02 Lease.pdf', description: 'Says "hi"\tthen\nbreaks', confidence: 0.874, reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE' },
  { id: 'two', originalFilename: 'Scan 2 東京.pdf', status: 'ready', proposedFilename: '2024-02-03 Invoice — Café.pdf', description: 'Plain.', confidence: 0.95 },
  { id: 'three', originalFilename: 'Scan 3.pdf', status: 'waiting' },
];

/** A copy event as the browser raises for Ctrl+C, with a clipboard that records what was put on it. */
function copyFrom(target: Element) {
  const data = new Map<string, string>();
  const event = new Event('copy', { bubbles: true, cancelable: true }) as ClipboardEvent;
  Object.defineProperty(event, 'clipboardData', { value: { setData: (type: string, value: string) => data.set(type, value), getData: (type: string) => data.get(type) ?? '' } });
  target.dispatchEvent(event);
  return { data, event };
}

const selectedCells = () => screen.getAllByRole('gridcell').filter((cell) => cell.getAttribute('aria-selected') === 'true').map((cell) => `${cell.closest('tr')?.querySelector('[data-column="original"]')?.textContent}:${cell.dataset.column}`);

async function start(items?: QueueItem[]) {
  render(<App bridge={items ? createInMemoryBridge({ items }) : createReviewBatchBridge()} />);
  const grid = await screen.findByRole('grid', { name: 'Queue' }, { timeout: 5000 });
  // Intern selects the first item needing review once the queue is read.
  if (!items || items.some((item) => item.status === 'review')) await waitFor(() => expect(grid.querySelector('tr[aria-current]')).not.toBeNull(), { timeout: 5000 });
  return grid;
}

describe('queue cells', () => {
  afterEach(() => { vi.unstubAllGlobals(); });

  it('selects a cell, a rectangle with Shift, and never a Status cell', async () => {
    const grid = await start();
    expect(grid).toHaveAttribute('aria-multiselectable', 'true');
    expect(screen.getByRole('button', { name: 'Copy table' })).toBeEnabled();

    clickCell(queueCell('Scan 0102.pdf'));
    expect(selectedCells()).toEqual(['Scan 0102.pdf:original']);
    expect(queueCell('Scan 0102.pdf')).toHaveFocus();
    // Its row is the selected item; the Details panel stays closed.
    expect(queueCell('Scan 0102.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Copy selection' })).toBeInTheDocument();

    clickCell(queueCell('Scan 0104.pdf', 'proposed'), { shiftKey: true });
    expect(selectedCells()).toEqual([
      'Scan 0102.pdf:original', 'Scan 0102.pdf:proposed',
      'Scan 0103.pdf:original', 'Scan 0103.pdf:proposed',
      'Scan 0104.pdf:original', 'Scan 0104.pdf:proposed',
    ]);
    // Status is not selectable at all, so it says nothing about selection.
    expect(queueCell('Scan 0103.pdf', 'status')).not.toHaveAttribute('aria-selected');
    expect(queueCell('Scan 0103.pdf', 'status')).not.toHaveAttribute('tabindex');
    // The keyboard steps over it, so the filename cell carries it for a screen reader.
    expect(queueCell('Scan 0103.pdf')).toHaveAccessibleDescription('Ready');
    expect(queueCell('Scan 0102.pdf')).toHaveAccessibleDescription('Needs review');
    // Extending moved neither the selected item nor anything else.
    expect(queueCell('Scan 0102.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    expect(screen.getByRole('status', { name: 'Cell selection' })).toHaveTextContent('3 rows by 2 columns selected');

    // A click on Status selects that row's item, and no cells.
    clickCell(queueCell('Scan 0105.pdf', 'status'));
    expect(selectedCells()).toEqual([]);
    expect(queueCell('Scan 0105.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    expect(screen.getByRole('button', { name: 'Copy table' })).toBeInTheDocument();
  });

  // Intern selects the first item to decide once the queue is read. A click
  // landing in the moment after that, before React ran its effects, used to
  // be undone by the active cell catching up with the earlier selection.
  it('keeps a click made as the launch selection appears', async () => {
    render(<App bridge={createInMemoryBridge({ items: tricky })} />);
    await new Promise<void>((resolve) => {
      const observer = new MutationObserver(() => {
        if (!document.querySelector('.queue-grid tr[aria-current]')) return;
        observer.disconnect();
        clickCell(queueCell('Scan 2 東京.pdf', 'proposed'));
        resolve();
      });
      observer.observe(document.body, { subtree: true, childList: true, attributes: true });
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(selectedCells()).toEqual(['Scan 2 東京.pdf:proposed']);
    expect(queueCell('Scan 2 東京.pdf', 'proposed')).toHaveAttribute('tabindex', '0');
  });

  it('selects by dragging, and a column from its heading', async () => {
    await start();
    fireEvent.mouseDown(queueCell('Scan 0101.pdf', 'description'), { button: 0, buttons: 1 });
    fireEvent.mouseEnter(queueCell('Scan 0102.pdf', 'status'), { buttons: 1 });
    fireEvent.mouseEnter(queueCell('Scan 0103.pdf', 'check'), { buttons: 1 });
    fireEvent.mouseUp(document, { button: 0 });
    // A move after the button is up selects nothing more.
    fireEvent.mouseEnter(queueCell('Scan 0104.pdf', 'check'), { buttons: 0 });
    expect(selectedCells()).toEqual([
      'Scan 0101.pdf:description', 'Scan 0101.pdf:check',
      'Scan 0102.pdf:description', 'Scan 0102.pdf:check',
      'Scan 0103.pdf:description', 'Scan 0103.pdf:check',
    ]);

    fireEvent.mouseDown(screen.getByRole('columnheader', { name: 'Proposed filename' }), { button: 0 });
    expect(selectedCells()).toHaveLength(7);
    expect(selectedCells().every((cell) => cell.endsWith(':proposed'))).toBe(true);
    // Status's heading selects nothing.
    fireEvent.mouseDown(screen.getByRole('columnheader', { name: 'Status' }), { button: 0 });
    expect(selectedCells()).toHaveLength(7);
  });

  it('moves and extends with the keyboard, stepping over Status, and Escape clears', async () => {
    await start();
    const cell = queueCell('Scan 0101.pdf');
    // One Tab stop: the selected item's row, in the first column.
    expect(cell).toHaveAttribute('tabindex', '0');
    expect(screen.getAllByRole('gridcell').filter((each) => each.getAttribute('tabindex') === '0')).toHaveLength(1);
    cell.focus();

    fireEvent.keyDown(cell, { key: 'ArrowRight' });
    expect(queueCell('Scan 0101.pdf', 'proposed')).toHaveFocus();
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown' });
    expect(queueCell('Scan 0102.pdf', 'proposed')).toHaveFocus();
    // Up and down move the selected item with the active cell.
    expect(queueCell('Scan 0102.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    expect(selectedCells()).toEqual(['Scan 0102.pdf:proposed']);

    fireEvent.keyDown(document.activeElement!, { key: 'ArrowRight', shiftKey: true });
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown', shiftKey: true });
    expect(queueCell('Scan 0103.pdf', 'description')).toHaveFocus();
    expect(selectedCells()).toEqual(['Scan 0102.pdf:proposed', 'Scan 0102.pdf:description', 'Scan 0103.pdf:proposed', 'Scan 0103.pdf:description']);

    fireEvent.keyDown(document.activeElement!, { key: 'ArrowLeft', shiftKey: true });
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowLeft', shiftKey: true });
    // Over Status to the first column, never onto it.
    expect(queueCell('Scan 0103.pdf', 'original')).toHaveFocus();
    expect(selectedCells()).toEqual(['Scan 0102.pdf:original', 'Scan 0102.pdf:proposed', 'Scan 0103.pdf:original', 'Scan 0103.pdf:proposed']);

    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(selectedCells()).toEqual([]);
    // Focus stays where it was, and the arrows carry on from there.
    expect(queueCell('Scan 0103.pdf')).toHaveFocus();
    expect(queueCell('Scan 0103.pdf')).toHaveAttribute('tabindex', '0');

    fireEvent.keyDown(document.activeElement!, { key: 'End', ctrlKey: true });
    expect(queueCell('Scan 0107.pdf', 'check')).toHaveFocus();
    fireEvent.keyDown(document.activeElement!, { key: 'Home' });
    expect(queueCell('Scan 0107.pdf')).toHaveFocus();
  });

  it('selects a column with Ctrl+Space and a row with Shift+Space', async () => {
    await start();
    const cell = queueCell('Scan 0103.pdf', 'description');
    cell.focus();
    fireEvent.keyDown(cell, { key: ' ', ctrlKey: true });
    expect(selectedCells()).toHaveLength(7);
    expect(selectedCells().every((each) => each.endsWith(':description'))).toBe(true);
    fireEvent.keyDown(document.activeElement!, { key: ' ', shiftKey: true });
    expect(selectedCells()).toEqual(['Scan 0101.pdf:original', 'Scan 0101.pdf:proposed', 'Scan 0101.pdf:description', 'Scan 0101.pdf:check']);
  });

  // The active cell is the end of the selection that moves, and keyboard
  // focus and Tab are always on it. Ctrl+A and Shift+Space used to move that
  // end without moving focus, so Tab jumped and the next Shift+arrow grew
  // the selection from somewhere else.
  it('keeps focus on the active end of the selection after Ctrl+A and Shift+Space', async () => {
    await start();
    const start0101 = queueCell('Scan 0101.pdf');
    start0101.focus();
    fireEvent.keyDown(start0101, { key: 'a', ctrlKey: true });
    expect(selectedCells()).toHaveLength(28);
    // Focus is on the far corner, the one Tab returns to.
    expect(queueCell('Scan 0107.pdf', 'check')).toHaveFocus();
    expect(queueCell('Scan 0107.pdf', 'check')).toHaveAttribute('tabindex', '0');
    expect(screen.getAllByRole('gridcell').filter((cell) => cell.getAttribute('tabindex') === '0')).toHaveLength(1);
    // As in a spreadsheet, Shift+Up then gives back the last row.
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowUp', shiftKey: true });
    expect(selectedCells()).toHaveLength(24);
    expect(selectedCells().some((cell) => cell.startsWith('Scan 0107.pdf'))).toBe(false);

    const cell = queueCell('Scan 0102.pdf', 'proposed');
    clickCell(cell);
    fireEvent.keyDown(cell, { key: ' ', shiftKey: true });
    expect(queueCell('Scan 0102.pdf', 'check')).toHaveFocus();
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown', shiftKey: true });
    expect(selectedCells()).toEqual([
      'Scan 0102.pdf:original', 'Scan 0102.pdf:proposed', 'Scan 0102.pdf:description', 'Scan 0102.pdf:check',
      'Scan 0103.pdf:original', 'Scan 0103.pdf:proposed', 'Scan 0103.pdf:description', 'Scan 0103.pdf:check',
    ]);
    // The active cell of a selection of several is marked as a spreadsheet marks it.
    expect(queueCell('Scan 0103.pdf', 'check')).toHaveClass('cell-active');
    expect(document.querySelectorAll('.cell-active')).toHaveLength(1);
    // A single selected cell needs no marker of its own.
    clickCell(queueCell('Scan 0104.pdf'));
    expect(document.querySelectorAll('.cell-active')).toHaveLength(0);
  });

  it('copies the selection on Ctrl+C as tab-separated text and an HTML table, without Status', async () => {
    await start(tricky);
    clickCell(queueCell('Scan 1.pdf'));
    clickCell(queueCell('Scan 2 東京.pdf', 'description'), { shiftKey: true });

    const { data, event } = copyFrom(document.activeElement!);

    expect(event.defaultPrevented).toBe(true);
    expect(data.get('text/plain')).toBe([
      'Scan 1.pdf\t2024-01-02 Lease.pdf\t"Says ""hi""\tthen\nbreaks"',
      'Scan 2 東京.pdf\t2024-02-03 Invoice — Café.pdf\tPlain.',
    ].join('\r\n'));
    const table = new DOMParser().parseFromString(data.get('text/html')!, 'text/html');
    expect([...table.querySelectorAll('tr')].map((row) => [...row.children].map((cell) => cell.textContent))).toEqual([
      ['Scan 1.pdf', '2024-01-02 Lease.pdf', 'Says "hi"\tthenbreaks'],
      ['Scan 2 東京.pdf', '2024-02-03 Invoice — Café.pdf', 'Plain.'],
    ]);
    expect(table.querySelector('th')).toBeNull();
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Copied 2 rows, 3 columns.'));
  });

  it('selects every copyable cell with Ctrl+A, and copies the check as shown', async () => {
    await start(tricky);
    const cell = queueCell('Scan 1.pdf');
    cell.focus();
    fireEvent.keyDown(cell, { key: 'a', ctrlKey: true });
    expect(selectedCells()).toHaveLength(12);

    const { data } = copyFrom(document.activeElement!);
    expect(data.get('text/plain')!.split('\r\n')).toEqual([
      'Scan 1.pdf\t2024-01-02 Lease.pdf\t"Says ""hi""\tthen\nbreaks"\tModel unsure of its reading',
      'Scan 2 東京.pdf\t2024-02-03 Invoice — Café.pdf\tPlain.\tAll checks passed',
      'Scan 3.pdf\t\t\t',
    ]);
  });

  // A spreadsheet's Ctrl+C with nothing more selected copies the cell it is on.
  it('copies the active cell on Ctrl+C when no cells are selected', async () => {
    await start(tricky);
    clickCell(queueCell('Scan 1.pdf'));
    clickCell(queueCell('Scan 2 東京.pdf', 'proposed'), { shiftKey: true });
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(selectedCells()).toEqual([]);

    const { data } = copyFrom(document.activeElement!);
    expect(data.get('text/plain')).toBe('2024-02-03 Invoice — Café.pdf');
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Copied 1 cell.'));
    // The button, labeled for it, still copies the whole table.
    expect(screen.getByRole('button', { name: 'Copy table' })).toBeInTheDocument();
  });

  // Copying is the table's only while a cell has focus. A text field reached
  // after working in the table copies its own text, as ever.
  it('leaves copying in a text field to the browser, even after the table had cells selected', async () => {
    const items = [...tricky, ...Array.from({ length: 5 }, (_, index): QueueItem => ({ id: `w${index}`, originalFilename: `Wait ${index}.pdf`, status: 'waiting' }))];
    await start(items);
    clickCell(queueCell('Scan 1.pdf'));
    clickCell(queueCell('Scan 2 東京.pdf', 'description'), { shiftKey: true });
    expect(selectedCells()).toHaveLength(6);

    const fields = async () => {
      const filter = screen.getByRole('searchbox', { name: 'Filter queue' });
      fireEvent.change(filter, { target: { value: 'Scan' } });
      const panel = await openDetails();
      return [filter, within(panel).getByLabelText('Filename'), within(panel).getByLabelText('Description')];
    };
    for (const field of await fields()) {
      field.focus();
      expect(field).toHaveFocus();
      const { data, event } = copyFrom(field);
      expect(event.defaultPrevented, field.getAttribute('aria-label') ?? field.tagName).toBe(false);
      expect(data.size).toBe(0);
    }
    const { event } = copyFrom(screen.getByRole('button', { name: /^Copy/ }));
    expect(event.defaultPrevented).toBe(false);
  });

  it('copies the whole table with headings from the Copy button when nothing is selected', async () => {
    const write = vi.fn(async (_items: unknown[]) => {});
    class FakeClipboardItem { constructor(public parts: Record<string, Blob>) {} }
    vi.stubGlobal('ClipboardItem', FakeClipboardItem);
    vi.stubGlobal('navigator', { ...navigator, clipboard: { write, writeText: vi.fn() } });
    await start(tricky);

    fireEvent.click(screen.getByRole('button', { name: 'Copy table' }));

    await waitFor(() => expect(write).toHaveBeenCalledOnce());
    const item = write.mock.calls[0][0][0] as FakeClipboardItem;
    expect(await item.parts['text/plain'].text()).toBe([
      'Original filename\tProposed filename\tDescription\tCheck',
      'Scan 1.pdf\t2024-01-02 Lease.pdf\t"Says ""hi""\tthen\nbreaks"\tModel unsure of its reading',
      'Scan 2 東京.pdf\t2024-02-03 Invoice — Café.pdf\tPlain.\tAll checks passed',
      'Scan 3.pdf\t\t\t',
    ].join('\r\n'));
    expect(await item.parts['text/html'].text()).toContain('<thead><tr><th>Original filename</th><th>Proposed filename</th><th>Description</th><th>Check</th></tr></thead>');
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Copied the table, 3 rows.'));
  });

  it('copies the selection from the Copy button through the copy command first', async () => {
    // The desktop webview's route: the document is asked to copy, and the
    // copy event that raises is filled with both formats.
    let copied: Map<string, string> | undefined;
    const execCommand = vi.fn((command: string) => { if (command === 'copy') copied = copyFrom(document.activeElement ?? document.body).data; return true; });
    Object.defineProperty(document, 'execCommand', { value: execCommand, configurable: true });
    try {
      await start(tricky);
      clickCell(queueCell('Scan 2 東京.pdf', 'check'));
      const button = screen.getByRole('button', { name: 'Copy selection' });
      button.focus();
      fireEvent.click(button);

      expect(execCommand).toHaveBeenCalledWith('copy');
      expect(copied?.get('text/plain')).toBe('All checks passed');
      expect(copied?.get('text/html')).toBe('<meta charset="utf-8"><table><tbody><tr><td style="mso-number-format:\'\\@\'" data-sheets-value="{&quot;1&quot;:2,&quot;2&quot;:&quot;All checks passed&quot;}">All checks passed</td></tr></tbody></table>');
      await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Copied 1 cell.'));
    } finally {
      delete (document as { execCommand?: unknown }).execCommand;
    }
  });

  // With every row it covered filed, there is no table left to drop the
  // selection, and the disabled button went on saying "Copy selection".
  it('forgets a selection once every row it covered has left the view', async () => {
    await start([tricky[1], { ...tricky[1], id: 'two-b', originalFilename: 'Scan 2b.pdf', proposedFilename: '2024-02-04 Receipt.pdf' }]);
    clickCell(queueCell('Scan 2 東京.pdf'));
    clickCell(queueCell('Scan 2b.pdf', 'description'), { shiftKey: true });
    expect(screen.getByRole('button', { name: 'Copy selection' })).toBeEnabled();

    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    await waitFor(() => expect(screen.queryByRole('grid', { name: 'Queue' })).not.toBeInTheDocument());
    expect(screen.getByRole('button', { name: 'Copy table' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Copy selection' })).not.toBeInTheDocument();

    // A selection does not come back with the next row; in Completed the filed rows are unselected.
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    await screen.findByRole('grid', { name: 'Queue' });
    expect(selectedCells()).toEqual([]);
    expect(screen.getByRole('button', { name: 'Copy table' })).toBeEnabled();
  });

  it('says so when the clipboard cannot be written', async () => {
    vi.stubGlobal('navigator', { ...navigator, clipboard: undefined });
    await start(tricky);
    fireEvent.click(screen.getByRole('button', { name: 'Copy table' }));
    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('Intern could not copy to the clipboard.');
  });

  it('counts what needs review where the Needs Review page used to, and steps through them from the count', async () => {
    await start();
    expect(screen.queryByRole('button', { name: /Needs Review,/ })).not.toBeInTheDocument();
    const count = screen.getByRole('button', { name: '4 need review' });
    count.focus();

    fireEvent.click(count);
    // After the selected Scan 0101, the next one needing review; selecting,
    // not opening, and the count keeps focus so the next press goes on.
    await waitFor(() => expect(queueCell('Scan 0102.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    expect(count).toHaveFocus();
    expect(queueCell('Scan 0102.pdf')).toHaveAttribute('tabindex', '0');
    expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument();
    fireEvent.click(count);
    await waitFor(() => expect(queueCell('Scan 0104.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    fireEvent.click(count);
    fireEvent.click(count);
    // Round again to the first.
    await waitFor(() => expect(queueCell('Scan 0101.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    expect(count).toHaveFocus();
  });

  it('counts the review items the filter leaves, the ones it steps through', async () => {
    await start();
    fireEvent.change(screen.getByRole('searchbox', { name: 'Filter queue' }), { target: { value: 'Fabrikam' } });
    const count = screen.getByRole('button', { name: '1 needs review' });
    fireEvent.click(count);
    await waitFor(() => expect(queueCell('Scan 0102.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    fireEvent.change(screen.getByRole('searchbox', { name: 'Filter queue' }), { target: { value: 'Contoso' } });
    expect(screen.queryByRole('button', { name: /need(s)? review/ })).not.toBeInTheDocument();
  });

  it('shows no count when nothing needs review, or in Completed', async () => {
    await start([tricky[1], tricky[2]]);
    expect(screen.queryByRole('button', { name: /need(s)? review/ })).not.toBeInTheDocument();
    cleanup();

    // Review items in the queue, and one filed document: Completed holds
    // nothing to review, so the count stays with the queue.
    await start([...tricky, { id: 'done', originalFilename: 'Done.pdf', status: 'completed', proposedFilename: '2024-01-01 Done.pdf', filedName: '2024-01-01 Done.pdf', undoable: true }]);
    expect(screen.getByRole('button', { name: '1 needs review' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    await screen.findByRole('gridcell', { name: 'Done.pdf' });
    expect(screen.queryByRole('button', { name: /need(s)? review/ })).not.toBeInTheDocument();
    expect(within(screen.getByRole('region', { name: 'Queue items' })).getByRole('button', { name: 'Copy table' })).toBeEnabled();
    expect(detailsToggle()).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: /^Queue, / }));
    expect(screen.getByRole('button', { name: '1 needs review' })).toBeInTheDocument();
  });
});
