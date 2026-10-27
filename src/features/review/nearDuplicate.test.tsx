import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { openRow } from '../queue/gridTesting';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueItem } from '../../types';

const selectRow = openRow;

const rescan: QueueItem = {
  id: 'rescan',
  originalFilename: 'Scan 0051.pdf',
  status: 'review',
  proposedFilename: '2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf',
  confidence: 0.94,
  description: 'Employment agreement between John Smith and Acme Corporation covering duties, salary, and term.',
  evidence: { date: 'signed April 12, 2024', type: 'EMPLOYMENT AGREEMENT', parties: 'John Smith; Acme Corporation' },
  // The desktop bridge turns the NEAR_DUPLICATE code into this sentence; the in-memory bridge passes reasons through as given.
  reason: 'This looks like a document that was filed already. Approve to file it as well, keep the original, or remove it.',
  nearDuplicateOf: '2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf (filed from Front desk)',
};

describe('a document waiting to be read again', () => {
  it('says why it waits, not that it is in review', async () => {
    const doubted: QueueItem = {
      id: 'doubted',
      originalFilename: 'Slow scan.pdf',
      status: 'waiting',
      reason: 'Waiting to be read again once Intern has shown it can read another document.',
    };
    render(<App bridge={createInMemoryBridge({ items: [doubted] })} />);
    await selectRow(await screen.findByRole('row', { name: /Slow scan.pdf/i }));

    const why = screen.getByRole('heading', { name: 'Why it is waiting' }).closest('section')!;
    expect(why).toHaveTextContent('Waiting to be read again once Intern has shown it can read another document.');
    expect(screen.queryByRole('heading', { name: 'Reason for review' })).not.toBeInTheDocument();
    // The grid says so too, with the reason on hover.
    const status = within(screen.getByRole('row', { name: /Slow scan.pdf/i })).getByText('Waiting to read again');
    expect(status.closest('[title]')).toHaveAttribute('title', 'Waiting to be read again once Intern has shown it can read another document.');
  });
});

describe('a copy staged while the queue is paused', () => {
  it('shows the copy name it is filed under and needs no one', async () => {
    const staged: QueueItem = {
      id: 'staged', originalFilename: 'lease scan.pdf', status: 'ready', proposedFilename: 'COPY - 2026-01-02 Lease.pdf',
      approved: true, copyOf: '2026-01-02 Lease.pdf', reason: 'Intern has filed this exact file before, so it filed it as a copy.',
    };
    render(<App bridge={createInMemoryBridge({ items: [staged] })} />);
    const row = await screen.findByRole('row', { name: /lease scan.pdf/i });
    expect(within(row).getByText('Ready to file as a copy')).toBeInTheDocument();
    expect(within(row).getByText('COPY - 2026-01-02 Lease.pdf')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /need review/i })).not.toBeInTheDocument();
  });
});

describe('a document that repeats a filed one', () => {
  it('says so in words and names the filing it repeats', async () => {
    render(<App bridge={createInMemoryBridge({ items: [rescan] })} />);
    await selectRow(await screen.findByRole('row', { name: /Scan 0051.pdf/i }));

    const reason = screen.getByRole('heading', { name: 'Reason for review' }).closest('section')!;
    expect(reason).toHaveTextContent('This looks like a document that was filed already. Approve to file it as well, keep the original, or remove it.');
    expect(within(reason).getByLabelText('Filed already as')).toHaveTextContent('2024-04-12 Employment Agreement between John Smith and Acme Corporation.pdf (filed from Front desk)');
    // The usual choices stay: it is a person's call, not Intern's.
    expect(screen.getByRole('button', { name: /Approve & rename/i })).toBeEnabled();
    expect(screen.getByRole('button', { name: /Keep original/i })).toBeEnabled();
  });
});
