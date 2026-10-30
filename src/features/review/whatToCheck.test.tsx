import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { openRow } from '../queue/gridTesting';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { humanizeReason } from '../../lib/reasons';
import type { QueueItem } from '../../types';

const ambiguous: QueueItem = {
  id: 'ambiguous',
  originalFilename: 'Scan 0102.pdf',
  status: 'review',
  proposedFilename: '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf',
  confidence: 0.81,
  description: 'Invoice INV-3301 from Fabrikam Inc. for February consulting hours.',
  evidence: { date: 'Invoice date: 02/09/2024', type: 'INVOICE INV-3301', parties: 'Fabrikam Inc.' },
  reason: humanizeReason('DATE_AMBIGUOUS, LOW_CONFIDENCE'),
  reasonCodes: ['DATE_AMBIGUOUS', 'LOW_CONFIDENCE'],
  readBy: { tier: 'accurate', modelId: 'qwen', displayName: 'Qwen3.5 2B' },
};

describe('what to check', () => {
  it('says what to check in the table, in words, and marks that part of the name', async () => {
    render(<App bridge={createInMemoryBridge({ items: [ambiguous] })} />);
    const row = await screen.findByRole('row', { name: /Scan 0102.pdf/i });
    // The model's own percentage is gone from the table; the failed check is there instead.
    expect(row).not.toHaveTextContent('81%');
    expect(within(row).getByText('Date could be day- or month-first')).toBeInTheDocument();
    expect(row).toHaveTextContent('+1 more');
    const mark = row.querySelector('mark.flagged');
    expect(mark).toHaveTextContent('2024-02-09');
    expect(mark).toHaveAttribute('title', 'Check the date');
  });

  it('puts the reason first in the panel and flags the date there and in the evidence', async () => {
    render(<App bridge={createInMemoryBridge({ items: [ambiguous] })} />);
    await openRow(await screen.findByRole('row', { name: /Scan 0102.pdf/i }));
    const panel = await screen.findByRole('complementary', { name: 'Review item' });

    const check = within(panel).getByRole('heading', { name: 'What to check' }).closest('section')!;
    expect(check).toHaveTextContent(/day-first or month-first/);
    // Before the name it is about, not after the evidence.
    const filename = within(panel).getByLabelText('Filename');
    expect(check.compareDocumentPosition(filename) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();

    expect(within(panel).getByRole('note', { name: 'Part of the name to check' })).toHaveTextContent('Check the date: 2024-02-09');
    // Editing the date away takes the flag with it.
    fireEvent.change(filename, { target: { value: 'Invoice INV-3301 from Fabrikam Inc' } });
    expect(within(panel).queryByRole('note', { name: 'Part of the name to check' })).not.toBeInTheDocument();

    const evidence = within(panel).getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(within(evidence).getByText('Date')).toHaveTextContent('Date (check this)');
    // The model's figure is still there, as a detail.
    expect(panel).toHaveTextContent("Model's confidence");
  });

  it('quotes the document only when the evidence is the document\'s own lines', async () => {
    const hosted: QueueItem = { ...ambiguous, id: 'hosted', originalFilename: 'Hosted.pdf', readBy: undefined };
    render(<App bridge={createInMemoryBridge({ items: [ambiguous, hosted] })} />);

    await openRow(await screen.findByRole('row', { name: /Scan 0102.pdf/i }));
    let evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(evidence).toHaveTextContent('Lines from the document behind each part of this name.');
    expect(evidence.querySelectorAll('q')).toHaveLength(3);

    await openRow(screen.getByRole('row', { name: /Hosted.pdf/i }));
    evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(evidence).toHaveTextContent(/in its own words/);
    // A paraphrase is never set in quotation marks.
    expect(evidence.querySelectorAll('q')).toHaveLength(0);
    expect(evidence).toHaveTextContent('Invoice date: 02/09/2024');
  });
});
