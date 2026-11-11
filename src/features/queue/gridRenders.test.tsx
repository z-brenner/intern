import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { QueueItem } from '../../types';

/*
  A drag changes the selection at every cell the pointer crosses. Held as
  the app's own state, each step re-rendered the whole window - the review
  panel included - and every cell of the queue: on a thousand rows, long
  tasks of up to 94ms while dragging. The selection lives beside the table
  now, and each row is given only its own part of it.
*/
const renders = vi.hoisted(() => ({ status: 0, inspector: 0 }));

vi.mock('../../components/StatusCell', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../components/StatusCell')>();
  return { StatusCell: (props: Parameters<typeof original.StatusCell>[0]) => { renders.status += 1; return original.StatusCell(props); } };
});
vi.mock('../../components/ReviewInspector', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../components/ReviewInspector')>();
  return { ...original, ReviewInspector: (props: Parameters<typeof original.ReviewInspector>[0]) => { renders.inspector += 1; return original.ReviewInspector(props); } };
});

const { App } = await import('../../App');
const { createInMemoryBridge } = await import('../../lib/inMemoryBridge');
const { openDetails, queueCell } = await import('./gridTesting');

/*
  Every queue event makes the window read the whole queue again, and each
  read is all new objects. The rows are memoized on their item, so every
  event rendered every row - measured on a thousand rows, a long task of
  70-80 ms each time one document changed status, landing while someone
  types a name. A read now keeps the items that read the same (useQueue's
  shareUnchanged), so an event renders the rows it changed.
*/
describe('a queue event', () => {
  const letters = (): QueueItem[] => Array.from({ length: 40 }, (_, index) => ({
    id: `doc-${index}`, originalFilename: `Scan ${String(index).padStart(3, '0')}.pdf`, status: 'review',
    proposedFilename: `2024-01-${String((index % 28) + 1).padStart(2, '0')} Letter ${index}.pdf`, confidence: 0.7, description: `Letter ${index}.`,
    reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE', evidence: { date: 'January 2, 2024', parties: 'Northwind Traders' },
  }));

  it('renders the row whose document changed, and nothing when none did', async () => {
    let queue = letters();
    let listener: ((event: { type: 'changed' }) => void) | undefined;
    let lists = 0;
    const bridge = {
      ...createInMemoryBridge({ items: [] }),
      // New objects on every read, as the desktop bridge returns them.
      listItems: async () => { lists += 1; return queue.map((item) => structuredClone(item)); },
      subscribeQueue: async (next: (event: { type: 'changed' }) => void) => { listener = next; return () => { listener = undefined; }; },
    };
    render(<App bridge={bridge} />);
    await waitFor(() => expect(queueCell('Scan 000.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    // The launch reads twice (once more after subscribing); let both land.
    await waitFor(() => expect(listener).toBeTypeOf('function'));
    await waitFor(() => expect(lists).toBeGreaterThanOrEqual(2));
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });

    // An event with nothing behind it: the queue is read, and nothing renders.
    renders.status = 0;
    let before = lists;
    await act(async () => { listener!({ type: 'changed' }); await new Promise((resolve) => setTimeout(resolve, 0)); });
    expect(lists).toBe(before + 1);
    expect(renders.status).toBe(0);

    // One document's description changes: its row renders, the other 39 do not.
    queue = queue.map((item) => item.id === 'doc-17' ? { ...item, description: 'A letter about the lease.' } : item);
    before = lists;
    await act(async () => { listener!({ type: 'changed' }); await new Promise((resolve) => setTimeout(resolve, 0)); });
    expect(lists).toBe(before + 1);
    await waitFor(() => expect(queueCell('Scan 017.pdf', 'description')).toHaveTextContent('A letter about the lease.'));
    expect(renders.status).toBe(1);

    // And one leaving, as every approval makes one leave: the rows after it
    // move up a place and are still the same rows. A status cell used to be
    // named by its row's place, so all nine after it rendered again - on a
    // thousand rows, every row below each document approved.
    renders.status = 0;
    queue = queue.filter((item) => item.id !== 'doc-30');
    await act(async () => { listener!({ type: 'changed' }); await new Promise((resolve) => setTimeout(resolve, 0)); });
    await waitFor(() => expect(screen.queryByRole('row', { name: /Scan 030\.pdf/ })).not.toBeInTheDocument());
    expect(screen.getAllByRole('row')).toHaveLength(40);
    expect(renders.status).toBe(0);
  });
});

describe('dragging across the queue', () => {
  it('renders only the rows whose cells change, and not the rest of the window', async () => {
    const items: QueueItem[] = Array.from({ length: 40 }, (_, index) => ({
      id: `doc-${index}`, originalFilename: `Scan ${String(index).padStart(3, '0')}.pdf`, status: 'review',
      proposedFilename: `2024-01-${String((index % 28) + 1).padStart(2, '0')} Letter ${index}.pdf`, confidence: 0.7, description: `Letter ${index}.`,
      reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE',
    }));
    render(<App bridge={createInMemoryBridge({ items })} />);
    await waitFor(() => expect(queueCell('Scan 000.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    await openDetails();

    fireEvent.mouseDown(queueCell('Scan 010.pdf'), { button: 0, buttons: 1 });
    await waitFor(() => expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('Scan 010.pdf'));
    renders.status = 0;
    renders.inspector = 0;

    // Six steps down and across, each growing the rectangle by a row.
    for (let row = 11; row <= 16; row += 1) {
      fireEvent.mouseEnter(queueCell(`Scan ${String(row).padStart(3, '0')}.pdf`, 'description'), { buttons: 1 });
    }
    fireEvent.mouseUp(document, { button: 0 });

    expect(screen.getAllByRole('gridcell').filter((cell) => cell.getAttribute('aria-selected') === 'true')).toHaveLength(7 * 3);
    // Each step re-renders the row it reaches and the row whose bottom edge
    // it moves off (and, the first time, the anchor's): about two rows a
    // step, where every one of the forty was rendered before.
    expect(renders.status).toBeLessThanOrEqual(6 * 2 + 1);
    expect(renders.inspector).toBe(0);
  });
});
