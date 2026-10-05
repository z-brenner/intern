import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueItem } from '../../types';

const selectRow = (row: HTMLElement) => fireEvent.click(within(row).getByRole('button', { name: /select/i }));

const byCounterparty: QueueItem = {
  id: 'sow',
  originalFilename: 'SOW final v3.pdf',
  status: 'ready',
  proposedFilename: '2026-04-01 Statement of Work with Ridgeline Cartography LLC.pdf',
  confidence: 0.93,
  description: 'Statement of work between Ridgeline Cartography LLC and Contoso Worldwide, Inc. for the 2026 member-map engagement.',
  evidence: { date: 'effective as of April 1, 2026', type: 'STATEMENT OF WORK', parties: 'Ridgeline Cartography LLC; Contoso Worldwide, Inc.' },
  omittedParties: ['Contoso Worldwide, Inc.'],
};

/*
  The evidence names two parties and the proposed name only one. Without a
  word about why, that reads as Intern losing a party - the one fact a name
  must never quietly drop.
*/
describe('a name filed by the other side', () => {
  it('says which party was left out as the organisation, while the evidence keeps it', async () => {
    render(<App bridge={createInMemoryBridge({ items: [byCounterparty] })} />);
    selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));

    const note = screen.getByRole('note', { name: 'Filed by the other side' });
    expect(note).toHaveTextContent('Filed by the other side: Contoso Worldwide, Inc. is your organisation, so it is left out of the name. Change this under Settings.');
    expect(within(note).getByText('Contoso Worldwide, Inc.').tagName).toBe('Q');
    expect(screen.getByLabelText('Filename')).toHaveValue('2026-04-01 Statement of Work with Ridgeline Cartography LLC.pdf');
    const evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(within(evidence).getByText('Contoso Worldwide, Inc.')).toBeVisible();
    expect(within(evidence).getByText('Ridgeline Cartography LLC')).toBeVisible();
  });

  it('names every spelling it left out', async () => {
    const twice = { ...byCounterparty, omittedParties: ['Contoso Worldwide, Inc.', 'Contoso UK', 'CONTOSO'] };
    render(<App bridge={createInMemoryBridge({ items: [twice] })} />);
    selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));

    expect(screen.getByRole('note', { name: 'Filed by the other side' })).toHaveTextContent('Filed by the other side: Contoso Worldwide, Inc., Contoso UK and CONTOSO are your organisation, so they are left out of the name. Change this under Settings.');
  });

  it('says nothing when the name carries every party', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    expect(screen.queryByRole('note', { name: 'Filed by the other side' })).not.toBeInTheDocument();
  });
});
