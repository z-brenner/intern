import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ANALYSIS_DETAILS_KEY, SELF_RATED_UNSURE } from '../../components/ReviewInspector';
import { openRow } from '../queue/gridTesting';
import { createInMemoryBridge, createReviewBatchBridge } from '../../lib/inMemoryBridge';
import { humanizeReason } from '../../lib/reasons';
import { TauriBridge } from '../../lib/tauriBridge';
import type { QueueItem } from '../../types';
import validateSource from '../../../crates/intern-engine/src/validate.rs?raw';

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
    expect(screen.getByText("Model's own confidence")).toBeVisible();
  });

  // It closed itself for every document, so a reviewer who checks each name
  // against its evidence opened it again for every one of them.
  it('leaves analysis details as the person left them, from one document to the next', async () => {
    render(<App bridge={createReviewBatchBridge()} />);
    const panel = await openRow(await screen.findByRole('row', { name: /Scan 0101.pdf/i }));
    const disclosure = () => within(panel).getByText('Analysis details').closest('details')!;
    expect(disclosure()).not.toHaveAttribute('open');

    fireEvent.click(within(panel).getByText('Analysis details'));
    expect(disclosure()).toHaveAttribute('open');
    expect(within(panel).getByText("Model's own confidence")).toBeVisible();

    // Approving moves on to the next document, and its details are open too.
    const filename = within(panel).getByLabelText('Filename');
    filename.focus();
    fireEvent.keyDown(filename, { key: 'Enter', ctrlKey: true });
    await waitFor(() => expect(panel).toHaveTextContent('Scan 0102.pdf'));
    expect(disclosure()).toHaveAttribute('open');
    expect(within(panel).getByText("Model's own confidence").closest('p')).toHaveTextContent('81%');

    // Selecting another row, and one with nothing to disclose in between, changes nothing.
    await openRow(screen.getByRole('row', { name: /Scan 0107.pdf/i }));
    expect(within(panel).queryByText('Analysis details')).not.toBeInTheDocument();
    await openRow(screen.getByRole('row', { name: /Scan 0104.pdf/i }));
    expect(disclosure()).toHaveAttribute('open');

    // Closed, it stays closed for the next one as well.
    fireEvent.click(within(panel).getByText('Analysis details'));
    expect(disclosure()).not.toHaveAttribute('open');
    await openRow(screen.getByRole('row', { name: /Scan 0106.pdf/i }));
    expect(disclosure()).not.toHaveAttribute('open');
    expect(within(panel).getByRole('button', { name: 'Approve & rename' })).toBeVisible();
  });

  it('remembers analysis details open or closed from one launch to the next, and starts closed', async () => {
    const launch = async () => {
      const view = render(<App bridge={createReviewBatchBridge()} />);
      const panel = await openRow(await screen.findByRole('row', { name: /Scan 0101.pdf/i }));
      return { view, summary: within(panel).getByText('Analysis details') };
    };
    const first = await launch();
    expect(first.summary.closest('details')).not.toHaveAttribute('open');
    expect(window.localStorage.getItem(ANALYSIS_DETAILS_KEY)).toBeNull();
    fireEvent.click(first.summary);
    expect(window.localStorage.getItem(ANALYSIS_DETAILS_KEY)).toBe('open');
    first.view.unmount();

    const second = await launch();
    expect(second.summary.closest('details')).toHaveAttribute('open');
    fireEvent.click(second.summary);
    second.view.unmount();

    const third = await launch();
    expect(third.summary.closest('details')).not.toHaveAttribute('open');
  });

  it('still opens analysis details when the choice cannot be stored or read', async () => {
    const setItem = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('Storage is disabled.'); });
    const getItem = vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('Storage is disabled.'); });
    try {
      render(<App bridge={createInMemoryBridge({ items: [ambiguous] })} />);
      const panel = await openRow(await screen.findByRole('row', { name: /Scan 0102.pdf/i }));
      const summary = within(panel).getByText('Analysis details');
      expect(summary.closest('details')).not.toHaveAttribute('open');
      fireEvent.click(summary);
      expect(summary.closest('details')).toHaveAttribute('open');
      expect(within(panel).getByRole('heading', { name: 'Evidence' })).toBeVisible();
    } finally {
      setItem.mockRestore();
      getItem.mockRestore();
    }
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
    expect(panel).toHaveTextContent("Model's own confidence");
  });

  // 173 of 204 real review items: the model rated itself at 100% and Intern
  // measured doubt in the words of the date or a party. The panel used to say
  // "The model reported low confidence in its own proposal" above a readout
  // of "Model's confidence 100%".
  it('does not let the unsure-reading reason and the model\'s own figure contradict each other', async () => {
    const unsure: QueueItem = { ...ambiguous, id: 'unsure', originalFilename: 'Scan 0103.pdf', confidence: 1, reason: humanizeReason('LOW_CONFIDENCE'), reasonCodes: ['LOW_CONFIDENCE'] };
    const dated: QueueItem = { ...ambiguous, id: 'dated', originalFilename: 'Scan 0104.pdf', confidence: 1, reason: humanizeReason('DATE_AMBIGUOUS'), reasonCodes: ['DATE_AMBIGUOUS'] };
    render(<App bridge={createInMemoryBridge({ items: [unsure, dated] })} />);
    await openRow(await screen.findByRole('row', { name: /Scan 0103.pdf/i }));
    const panel = await screen.findByRole('complementary', { name: 'Review item' });

    const check = within(panel).getByRole('heading', { name: 'What to check' }).closest('section')!;
    expect(check).toHaveTextContent('The model was unsure of its reading. Check the name against the document, starting with the date and the parties.');
    // Nothing says the model reported the doubt: it reported 100%.
    expect(panel).not.toHaveTextContent(/reported low confidence/i);

    fireEvent.click(within(panel).getByText('Analysis details'));
    const readout = within(panel).getByText("Model's own confidence").closest('p')!;
    expect(readout).toHaveTextContent("Model's own confidence100%");
    const note = within(panel).getByRole('note', { name: "About the model's own confidence" });
    expect(note).toHaveTextContent(/the model's rating of its whole answer.*can be flagged as unsure while this figure is high/);
    // Straight after the figure it explains.
    expect(readout.nextElementSibling).toBe(note);

    // Without that reason there is nothing to reconcile, and no note. (The
    // section is still open: it stays as it was left from one document to the next.)
    await openRow(screen.getByRole('row', { name: /Scan 0104.pdf/i }));
    expect(within(panel).getByText("Model's own confidence")).toBeVisible();
    expect(within(panel).queryByRole('note', { name: "About the model's own confidence" })).not.toBeInTheDocument();
  });

  // The note is left out where the model's own figure is itself the cause.
  // That line is the engine's, so the panel's copy of it is held to it.
  it('draws the line for the note where the engine sends a low self-rating to review', async () => {
    const declared = /pub const READY_CONFIDENCE: f32 = ([0-9.]+);/.exec(validateSource)?.[1];
    expect(declared, 'READY_CONFIDENCE is declared in intern-engine').toBeDefined();
    expect(SELF_RATED_UNSURE).toBe(Number(declared));

    const at = (id: string, name: string, confidence: number): QueueItem => ({ ...ambiguous, id, originalFilename: name, confidence, reason: humanizeReason('LOW_CONFIDENCE'), reasonCodes: ['LOW_CONFIDENCE'] });
    render(<App bridge={createInMemoryBridge({ items: [at('on', 'Scan 0201.pdf', SELF_RATED_UNSURE), at('under', 'Scan 0202.pdf', SELF_RATED_UNSURE - 0.01)] })} />);
    const panel = await openRow(await screen.findByRole('row', { name: /Scan 0201.pdf/i }));
    fireEvent.click(within(panel).getByText('Analysis details'));
    // On the line the engine did not flag the figure, so something else did.
    expect(within(panel).getByRole('note', { name: "About the model's own confidence" })).toBeInTheDocument();
    await openRow(screen.getByRole('row', { name: /Scan 0202.pdf/i }));
    expect(within(panel).getByText("Model's own confidence")).toBeVisible();
    expect(within(panel).queryByRole('note', { name: "About the model's own confidence" })).not.toBeInTheDocument();
  });

  it('adds no explanation when the model\'s own figure is low too: the two agree', async () => {
    const low: QueueItem = { ...ambiguous, id: 'low', originalFilename: 'Scan 0105.pdf', confidence: 0.41, reason: humanizeReason('LOW_CONFIDENCE'), reasonCodes: ['LOW_CONFIDENCE'] };
    render(<App bridge={createInMemoryBridge({ items: [low] })} />);
    await openRow(await screen.findByRole('row', { name: /Scan 0105.pdf/i }));
    const panel = await screen.findByRole('complementary', { name: 'Review item' });
    fireEvent.click(within(panel).getByText('Analysis details'));
    expect(within(panel).getByText("Model's own confidence").closest('p')).toHaveTextContent('41%');
    expect(within(panel).queryByRole('note', { name: "About the model's own confidence" })).not.toBeInTheDocument();
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
