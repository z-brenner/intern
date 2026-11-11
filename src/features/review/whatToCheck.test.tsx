import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { openRow } from '../queue/gridTesting';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { humanizeReason } from '../../lib/reasons';
import { TauriBridge } from '../../lib/tauriBridge';
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
  readBy: { tier: 'accurate', modelId: 'qwen', displayName: 'Qwen3.5 9B' },
};

describe('what to check', () => {
  it('keeps optional analysis details collapsed while the decision stays available', async () => {
    render(<App bridge={createInMemoryBridge({ items: [ambiguous] })} />);
    await openRow(await screen.findByRole('row', { name: /Scan 0102.pdf/i }));
    const disclosure = screen.getByText('Analysis details').closest('details')!;
    expect(disclosure).not.toHaveAttribute('open');
    expect(screen.getByRole('button', { name: 'Approve & rename' })).toBeVisible();
    fireEvent.click(screen.getByText('Analysis details'));
    expect(disclosure).toHaveAttribute('open');
    expect(screen.getByText("Model's confidence")).toBeVisible();
  });

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
    expect(evidence).toHaveTextContent('Lines from the document behind the date and document type. The parties are the names Intern read, not lines from the document.');
    // The date and the type are lines the engine found in the document. The
    // Parties row is the proposal's party names, which no line of the
    // document need read like, so it is never set in quotation marks.
    expect([...evidence.querySelectorAll('q')].map((quote) => quote.textContent)).toEqual(['Invoice date: 02/09/2024', 'INVOICE INV-3301']);
    const parties = within(evidence).getByText('Parties').closest('.evidence-item')!;
    expect(parties.querySelector('q')).toBeNull();
    expect(parties.querySelector('.reading')).toHaveTextContent('Fabrikam Inc.');

    await openRow(screen.getByRole('row', { name: /Hosted.pdf/i }));
    evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(evidence).toHaveTextContent(/in its own words/);
    // A paraphrase is never set in quotation marks.
    expect(evidence.querySelectorAll('q')).toHaveLength(0);
    expect(evidence).toHaveTextContent('Invoice date: 02/09/2024');
  });

  it('labels a desktop reading from what the backend sends: who read it, its lines quoted, its parties named', async () => {
    // One item as queue_list serializes a local reading (queue_item_dto in commands.rs).
    const sent = [{
      id: 7, originalFilename: 'Scan 0201.pdf', status: 'needs_review',
      proposedFilename: '2025-05-01 Invoice from Halvorsen Fixture Works LLC.pdf', confidence: 1,
      description: 'Invoice from Halvorsen Fixture Works LLC to Quillon Ridge Bakery, Inc. for display shelving.',
      evidence: { date: 'Invoice Date: May 1, 2025', type: 'INVOICE', parties: 'Halvorsen Fixture Works LLC; Quillon Ridge Bakery, Inc.' },
      reason: 'PARTY_UNSUPPORTED, LOW_CONFIDENCE', proposalRevision: '1',
      readBy: { tier: 'fast', modelId: 'qwen3.5-4b-instruct-q4-k-m', displayName: 'Qwen3.5 4B', provisional: true },
    }];
    const items = await new TauriBridge({ invoke: async <T,>() => sent as T, listen: async () => () => {} }).listItems();
    render(<App bridge={createInMemoryBridge({ items })} />);
    await openRow(await screen.findByRole('row', { name: /Scan 0201.pdf/i }));
    fireEvent.click(screen.getByText('Analysis details'));

    expect(screen.getByLabelText('Read by')).toHaveTextContent('Read by Qwen3.5 4B. Provisional — will be re-read by the accurate model.');
    const evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(evidence).toHaveTextContent(/^EvidenceLines from the document behind the date and document type\. The parties are the names Intern read, not lines from the document\./);
    expect(evidence).not.toHaveTextContent(/in its own words/);
    expect([...evidence.querySelectorAll('q')].map((quote) => quote.textContent)).toEqual(['Invoice Date: May 1, 2025', 'INVOICE']);
    // The doubted party is marked, and is still not a quotation.
    const parties = within(evidence).getByText('Parties').closest('.evidence-item')!;
    expect(parties).toHaveClass('flagged');
    expect([...parties.querySelectorAll('.reading')].map((name) => name.textContent)).toEqual(['Halvorsen Fixture Works LLC', 'Quillon Ridge Bakery, Inc.']);
  });

  it('says only what the evidence it has is: names alone are not called lines from the document', async () => {
    const namesOnly: QueueItem = { ...ambiguous, id: 'names', originalFilename: 'Names.pdf', evidence: { parties: 'Fabrikam Inc.; Northwind Traders' } };
    const dateOnly: QueueItem = { ...ambiguous, id: 'date', originalFilename: 'Dated.pdf', evidence: { date: 'Invoice date: 02/09/2024' } };
    render(<App bridge={createInMemoryBridge({ items: [namesOnly, dateOnly] })} />);

    await openRow(await screen.findByRole('row', { name: /Names.pdf/i }));
    let evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(within(evidence).getByText(/The parties are/)).toHaveTextContent(/^The parties are the names Intern read\. They are not lines from the document\.$/);
    expect(evidence.querySelectorAll('q')).toHaveLength(0);
    expect([...evidence.querySelectorAll('.reading')].map((name) => name.textContent)).toEqual(['Fabrikam Inc.', 'Northwind Traders']);

    await openRow(screen.getByRole('row', { name: /Dated.pdf/i }));
    evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(within(evidence).getByText(/^Lines from the document/)).toHaveTextContent(/^Lines from the document behind the date\.$/);
    expect(evidence.querySelectorAll('q')).toHaveLength(1);
  });
});
