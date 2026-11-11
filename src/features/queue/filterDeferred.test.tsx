import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { QueueItem } from '../../types';

/*
  Typing in the filter narrowed the table in the same pass that showed the
  character typed, so on a long queue each keystroke waited for the whole
  table: measured on a thousand rows, a third of a second before the
  character appeared. The box now updates in one pass and the table in a
  second, deferred one that the next keystroke interrupts. What is recorded
  here is each pass over the table and how many rows it was given.
*/
const passes = vi.hoisted(() => ({ rows: [] as number[] }));

vi.mock('../../components/QueueTable', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../components/QueueTable')>();
  return { ...original, QueueTable: (props: Parameters<typeof original.QueueTable>[0]) => { passes.rows.push(props.items.length); return original.QueueTable(props); } };
});

const { App } = await import('../../App');
const { createInMemoryBridge } = await import('../../lib/inMemoryBridge');
const { queueCell } = await import('./gridTesting');

const items: QueueItem[] = Array.from({ length: 40 }, (_, index) => ({
  id: `doc-${index}`, originalFilename: `Scan ${String(index).padStart(3, '0')}.pdf`, status: 'review',
  proposedFilename: `2024-01-${String((index % 28) + 1).padStart(2, '0')} Letter ${index}.pdf`, description: `Letter ${index}.`,
}));

describe('typing in the queue filter', () => {
  it('shows the text at once and narrows the table in a pass of its own', async () => {
    render(<App bridge={createInMemoryBridge({ items })} />);
    await waitFor(() => expect(queueCell('Scan 000.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    const filter = screen.getByRole('searchbox', { name: 'Filter queue' });

    passes.rows.length = 0;
    fireEvent.change(filter, { target: { value: 'scan 03' } });
    expect(filter).toHaveValue('scan 03');
    // The pass that put the text in the box left the table as it was; the
    // ten matching rows came in the pass after it.
    expect(passes.rows[0]).toBe(40);
    expect(passes.rows.at(-1)).toBe(10);
    expect(screen.getAllByRole('row')).toHaveLength(11);
    expect(screen.getByText('10 of 40 items')).toBeVisible();

    // Nothing matches: the message quotes the text the rows were narrowed by.
    passes.rows.length = 0;
    fireEvent.change(filter, { target: { value: '  no such scan ' } });
    expect(passes.rows[0]).toBe(10);
    expect(screen.getByText('No items match “no such scan”.')).toBeVisible();
    expect(screen.getByText('0 of 40 items')).toBeVisible();

    // Escape empties the box at once, and the whole queue follows.
    fireEvent.keyDown(filter, { key: 'Escape' });
    expect(filter).toHaveValue('');
    expect(screen.getAllByRole('row')).toHaveLength(41);
    expect(screen.getByText('40 items')).toBeVisible();
  });

  it('keeps the box on screen for as long as it has text in it, on a queue too short to need one', async () => {
    const base = createInMemoryBridge({ items: items.slice(0, 7) });
    let changed: (() => void) | undefined;
    const bridge = { ...base, subscribeQueue: async (listener: (event: { type: 'changed' }) => void) => { changed = () => listener({ type: 'changed' }); return () => {}; } };
    render(<App bridge={bridge} />);
    await waitFor(() => expect(queueCell('Scan 000.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    await waitFor(() => expect(changed).toBeTypeOf('function'));
    const filter = screen.getByRole('searchbox', { name: 'Filter queue' });
    fireEvent.change(filter, { target: { value: 'scan 005' } });
    expect(screen.getByText('1 of 7 items')).toBeVisible();

    // Two documents leave, and seven become five: too few for the box to be
    // offered, but it is in use, so it stays.
    await base.remove('doc-0');
    await base.remove('doc-1');
    act(() => changed!());
    await waitFor(() => expect(screen.getByText('1 of 5 items')).toBeVisible());
    expect(screen.getByRole('searchbox', { name: 'Filter queue' })).toBe(filter);
    expect(filter).toHaveValue('scan 005');

    // Emptied, it goes, as it did before.
    fireEvent.change(filter, { target: { value: '' } });
    expect(screen.queryByRole('searchbox', { name: 'Filter queue' })).not.toBeInTheDocument();
    expect(screen.getByText('5 items')).toBeVisible();
    expect(screen.getAllByRole('row')).toHaveLength(6);
  });
});
