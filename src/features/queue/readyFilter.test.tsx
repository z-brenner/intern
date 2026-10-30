import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueItem } from '../../types';

describe('show only ready', () => {
  it('narrows the queue to what Rename all ready would rename, and back', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const toggle = await screen.findByRole('button', { name: '3 ready' });
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getAllByRole('row')).toHaveLength(9);

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    const rows = screen.getAllByRole('row').slice(1);
    expect(rows).toHaveLength(3);
    for (const row of rows) expect(row).toHaveTextContent('Ready');
    expect(screen.getByText('3 of 8 items')).toBeInTheDocument();

    fireEvent.click(toggle);
    expect(screen.getAllByRole('row')).toHaveLength(9);
  });
});

describe('show only ready, beside an exact copy', () => {
  const items: QueueItem[] = [
    { id: 'own', originalFilename: 'scan-1.pdf', status: 'ready', proposedFilename: '2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf' },
    { id: 'copy', originalFilename: 'scan-1 (copy).pdf', status: 'ready', proposedFilename: 'COPY 2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf', approved: true, copyOf: '2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf' },
    { id: 'later', originalFilename: 'later.pdf', status: 'waiting' },
  ];

  it('shows only what Rename all ready renames, and turns itself off when that is gone', async () => {
    render(<App bridge={createInMemoryBridge({ items })} />);
    const toggle = await screen.findByRole('button', { name: '1 ready' });
    fireEvent.click(toggle);
    const rows = screen.getAllByRole('row').slice(1);
    expect(rows).toHaveLength(1);
    expect(rows[0]).toHaveTextContent('scan-1.pdf');

    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    // The copy is still staged as ready, but nothing is left for the button,
    // so the filter lets go rather than hiding the rest behind a toggle
    // that is no longer there.
    await waitFor(() => expect(screen.queryByRole('button', { name: /\d+ ready/ })).not.toBeInTheDocument());
    expect(await screen.findByText('later.pdf')).toBeInTheDocument();
  });
});
