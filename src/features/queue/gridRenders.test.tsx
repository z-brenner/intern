import { fireEvent, render, screen, waitFor } from '@testing-library/react';
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
