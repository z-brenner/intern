import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { clickCell, detailsToggle, openDetails, queueCell, selectRow } from '../queue/gridTesting';
import type { QueueEventSource } from '../../lib/tauriBridge';
import type { QueueItem } from '../../types';
import { proposalOf, setAsideOutdatedDrafts, typedAndNotFiled } from './drafts';
import type { EarlierDrafts, ReviewDrafts } from './drafts';

/*
  What is typed for a name and a description and not filed is kept for the
  document and the proposal it was typed over, by the window and not by the
  panel. The panel kept it, and the panel leaves the screen without anyone
  deciding anything: a wide panel when the window is made narrower than
  1100px, a drawer when it is made wider, either by Close review, Escape or
  the Details toggle, and when the selection is cleared. Each time, both
  fields came back as Intern's proposal and what had been typed was gone,
  with nothing filed.

  Kept by the window, it outlives the panel, so it is shown only over the
  proposal it was typed over, and forgotten once its document is seen with
  another proposal or none: a document read again shows its new reading.
  The backend numbers every new reading's proposal from 1, so the revision
  alone does not tell one reading from the next.
*/
describe('what was typed in review and not filed', () => {
  afterEach(() => vi.unstubAllGlobals());

  const LEASE = 'Lease Agreement - 123 Main St.pdf';
  const PROPOSED = '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc';
  const TYPED = `${PROPOSED} v2 signed`;
  const DESCRIBED = 'Countersigned copy received 3 May; clause 7 amended by hand.';

  /** A window whose width can change under the app: at 1100px and below the review panel is a drawer. */
  const windowOf = (narrow: boolean) => {
    const state = { narrow };
    const listeners = new Set<() => void>();
    vi.stubGlobal('matchMedia', (query: string) => ({
      get matches() { return query === '(max-width: 1100px)' && state.narrow; },
      media: query,
      onchange: null,
      addEventListener: (_type: string, listener: () => void) => { listeners.add(listener); },
      removeEventListener: (_type: string, listener: () => void) => { listeners.delete(listener); },
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));
    return { resize: (to: 'narrow' | 'wide') => act(() => { state.narrow = to === 'narrow'; for (const listener of [...listeners]) listener(); }) };
  };
  const start = async (base = createInMemoryBridge()) => {
    const bridge = { ...base, approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal), remove: vi.fn(base.remove) };
    render(<App bridge={bridge} />);
    await waitFor(() => expect(queueCell(LEASE).closest('tr')).toHaveAttribute('aria-current', 'true'), { timeout: 5000 });
    return bridge;
  };
  const panel = () => screen.queryByRole('complementary', { name: 'Review item' }) ?? screen.queryByRole('dialog', { name: 'Review item' });
  const filename = () => screen.getByLabelText('Filename');
  const description = () => screen.getByLabelText('Description');
  const type = () => {
    fireEvent.change(filename(), { target: { value: TYPED } });
    fireEvent.change(description(), { target: { value: DESCRIBED } });
  };
  const expectTyped = () => {
    expect(filename()).toHaveValue(TYPED);
    expect(description()).toHaveValue(DESCRIBED);
  };

  it('is still there after Close review, the Details toggle and Escape in the drawer', async () => {
    const window_ = windowOf(false);
    const bridge = await start();
    await openDetails(LEASE);
    type();

    fireEvent.click(within(panel()!).getByRole('button', { name: 'Close review' }));
    expect(panel()).not.toBeInTheDocument();
    await openDetails();
    expectTyped();

    fireEvent.click(detailsToggle());
    expect(panel()).not.toBeInTheDocument();
    fireEvent.click(detailsToggle());
    expectTyped();
    fireEvent.click(detailsToggle());

    // The drawer: Escape closes it, and Enter on the row opens it at the name.
    window_.resize('narrow');
    fireEvent.click(detailsToggle());
    const drawer = await screen.findByRole('dialog', { name: 'Review item' });
    expectTyped();
    fireEvent.keyDown(drawer, { key: 'Escape' });
    expect(panel()).not.toBeInTheDocument();
    const cell = queueCell(LEASE);
    cell.focus();
    fireEvent.keyDown(cell, { key: 'Enter' });
    await screen.findByRole('dialog', { name: 'Review item' });
    expectTyped();

    // Nothing was filed by any of it, and what is filed is what was typed.
    expect(bridge.approve).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    expect(bridge.approve).toHaveBeenCalledTimes(1);
    expect(bridge.approve).toHaveBeenCalledWith('lease', `${TYPED}.pdf`, DESCRIBED);
  });

  it('is still there when a wide panel is closed by a narrower window, at either width', async () => {
    const window_ = windowOf(false);
    const bridge = await start();
    await openDetails(LEASE);
    type();

    // Narrow, a drawer opens only when asked: the panel is off screen.
    window_.resize('narrow');
    expect(panel()).not.toBeInTheDocument();
    window_.resize('wide');
    expect(panel()).toHaveAttribute('role', 'complementary');
    expectTyped();

    // And in the drawer, asked for at the narrow width.
    window_.resize('narrow');
    fireEvent.click(detailsToggle());
    expect(await screen.findByRole('dialog', { name: 'Review item' })).toBeInTheDocument();
    expectTyped();
    expect(bridge.approve).not.toHaveBeenCalled();
  });

  it('is still there when a drawer is closed by a wider window, at either width', async () => {
    const window_ = windowOf(true);
    const bridge = await start();
    await openDetails(LEASE);
    type();

    // Wide, the panel is open only if it was left open: here it was not.
    window_.resize('wide');
    expect(panel()).not.toBeInTheDocument();
    // Narrow again, a drawer opens only when asked for at that width: it
    // used to open by itself here, asked for before the window was widened.
    window_.resize('narrow');
    expect(panel()).not.toBeInTheDocument();
    fireEvent.click(detailsToggle());
    expect(await screen.findByRole('dialog', { name: 'Review item' })).toBeInTheDocument();
    expectTyped();

    window_.resize('wide');
    expect(panel()).not.toBeInTheDocument();
    fireEvent.click(detailsToggle());
    expect(panel()).toHaveAttribute('role', 'complementary');
    expectTyped();
    expect(bridge.approve).not.toHaveBeenCalled();
  });

  it('is still there when the selection is cleared and the document is opened again', async () => {
    windowOf(false);
    const bridge = await start();
    await openDetails(LEASE);
    type();

    // The other view clears the selection, and the panel goes with it.
    fireEvent.click(screen.getByRole('button', { name: /^Completed,/ }));
    expect(panel()).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Queue,/ }));
    await openDetails(LEASE);
    expectTyped();
    expect(bridge.approve).not.toHaveBeenCalled();
  });

  it('is each document\'s own, whichever was on screen when the panel closed', async () => {
    windowOf(false);
    await start();
    await openDetails(LEASE);
    type();
    // Another document, with a name of its own typed, is on screen when the panel closes.
    clickCell(queueCell('NDA - Acme Corp.docx'));
    await waitFor(() => expect(panel()).toHaveTextContent('NDA - Acme Corp.docx'));
    fireEvent.change(filename(), { target: { value: '2024-03-01 Non-Disclosure Agreement with Acme Corporation' } });
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Close review' }));

    await openDetails();
    expect(filename()).toHaveValue('2024-03-01 Non-Disclosure Agreement with Acme Corporation');
    selectRow(LEASE);
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectTyped();
    // A document nothing was typed for shows its proposal.
    selectRow('Service Agreement - BlueSky LLC.pdf');
    await waitFor(() => expect(panel()).toHaveTextContent('Service Agreement - BlueSky LLC.pdf'));
    expect(filename()).toHaveValue('2024-02-28 Service Agreement with BlueSky LLC');
  });

  // Analyze again: the document goes back to be read, and its new reading
  // is numbered from 1 as the one before was. Whatever it says - here, the
  // same as before - it is shown as it is, not under what was typed over
  // the reading before.
  it('is not shown over a new reading, even one that says the same and has the same revision', async () => {
    windowOf(false);
    const base = createInMemoryBridge({ liveEvents: true, analysisDelayMs: 20 });
    // As the backend numbers them: every reading's proposal starts at revision 1.
    const numberedFromOne = { ...base, listItems: async () => (await base.listItems()).map((item) => item.proposedFilename === undefined ? item : { ...item, proposalRevision: '1' }) };
    const bridge = await start(numberedFromOne);
    await openDetails(LEASE);
    type();

    fireEvent.click(within(panel()!).getByRole('button', { name: 'More review actions' }));
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Analyze again' }));
    await waitFor(() => expect(queueCell(LEASE, 'status')).toHaveTextContent(/Waiting/i));
    await waitFor(() => expect(queueCell(LEASE, 'status')).toHaveTextContent(/review/i), { timeout: 5000 });
    expect((await bridge.listItems()).find((item) => item.id === 'lease')).toMatchObject({ proposalRevision: '1', proposedFilename: `${PROPOSED}.pdf` });

    await openDetails(LEASE);
    expect(filename()).toHaveValue(PROPOSED);
    expect(description()).toHaveValue('Commercial lease agreement between landlord and tenant for 123 Main St.');
    expect(bridge.approve).not.toHaveBeenCalled();
  });

  // Read again by itself - the accurate model taking over a reading the
  // quick one made - with the panel open on the document all the while.
  it('is not shown over a reading made again while the panel stays open', async () => {
    windowOf(false);
    const base = createInMemoryBridge({ liveEvents: true, analysisDelayMs: 300 }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    // The backend says the queue changed when it sends a document back to be read.
    let queueChanged = () => {};
    const numberedFromOne = {
      ...base,
      listItems: async () => (await base.listItems()).map((item) => item.proposedFilename === undefined ? item : { ...item, proposalRevision: '1' }),
      subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); },
    };
    const bridge = await start(numberedFromOne);
    await openDetails(LEASE);
    type();

    await act(async () => { await base.reanalyze('lease'); queueChanged(); });
    await waitFor(() => expect(queueCell(LEASE, 'status')).toHaveTextContent(/Waiting/i));
    expect(panel()).toHaveTextContent(LEASE);
    await waitFor(() => expect(queueCell(LEASE, 'status')).toHaveTextContent(/review/i), { timeout: 5000 });
    await waitFor(() => expect(screen.queryByLabelText('Filename')).toBeInTheDocument());
    expect(filename()).toHaveValue(PROPOSED);
    expect(description()).toHaveValue('Commercial lease agreement between landlord and tenant for 123 Main St.');
    expect(bridge.approve).not.toHaveBeenCalled();
  });

  // The panel by itself: taken off screen and put back, it shows what was
  // kept for the document and the proposal on screen, and a new proposal
  // as it is.
  const calls = () => ({ onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(), onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn() });
  const lease: QueueItem = { id: 'lease', originalFilename: 'scan.pdf', status: 'review', proposedFilename: '2024-03-01 Lease.pdf', description: 'A lease.', confidence: 0.8, proposalRevision: '1' };
  const invoice: QueueItem = { id: 'invoice', originalFilename: 'scan 2.pdf', status: 'review', proposedFilename: '2024-04-01 Invoice.pdf', description: 'An invoice.', confidence: 0.8, proposalRevision: '1' };

  it('is kept for the document and the proposal it was typed over, not for a new proposal', () => {
    const drafts: ReviewDrafts = new Map();
    const handlers = calls();
    const first = render(<ReviewInspector item={lease} drawer={false} drafts={drafts} {...handlers} />);
    fireEvent.change(filename(), { target: { value: '2024-03-01 Lease of Unit 4' } });
    fireEvent.change(description(), { target: { value: 'A lease of Unit 4.' } });
    // Review moves on, and something is typed there too, before the panel goes.
    first.rerender(<ReviewInspector item={invoice} drawer={false} drafts={drafts} {...handlers} />);
    fireEvent.change(description(), { target: { value: 'An invoice for March.' } });
    first.unmount();

    const second = render(<ReviewInspector item={lease} drawer={true} drafts={drafts} {...handlers} />);
    expect(filename()).toHaveValue('2024-03-01 Lease of Unit 4');
    expect(description()).toHaveValue('A lease of Unit 4.');
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    expect(handlers.onApprove).toHaveBeenLastCalledWith('2024-03-01 Lease of Unit 4.pdf', 'A lease of Unit 4.');
    second.rerender(<ReviewInspector item={invoice} drawer={true} drafts={drafts} {...handlers} />);
    expect(filename()).toHaveValue('2024-04-01 Invoice');
    expect(description()).toHaveValue('An invoice for March.');
    second.unmount();

    // Read again while the panel was closed, it is a new proposal: shown as it is.
    render(<ReviewInspector item={{ ...lease, proposedFilename: '2024-03-02 Lease Agreement.pdf', description: 'A lease agreement.', proposalRevision: '2' }} drawer={false} drafts={drafts} {...handlers} />);
    expect(filename()).toHaveValue('2024-03-02 Lease Agreement');
    expect(description()).toHaveValue('A lease agreement.');
  });

  // A new reading has the revision the one before it had. Its name or its
  // sentence says it is another proposal, with the panel on screen or off.
  it('is not shown over another name or another sentence under the same revision', () => {
    const drafts: ReviewDrafts = new Map();
    const handlers = calls();
    const renamed = { ...lease, proposedFilename: '2024-03-02 Lease Agreement.pdf' };
    const redescribed = { ...lease, description: 'A lease of the ground floor.' };
    const view = render(<ReviewInspector item={lease} drawer={false} drafts={drafts} {...handlers} />);
    fireEvent.change(filename(), { target: { value: '2024-03-01 Lease of Unit 4' } });
    fireEvent.change(description(), { target: { value: 'A lease of Unit 4.' } });

    // On screen: the fields are the new proposal's, and so is what Approve sends.
    view.rerender(<ReviewInspector item={renamed} drawer={false} drafts={drafts} {...handlers} />);
    expect(filename()).toHaveValue('2024-03-02 Lease Agreement');
    expect(description()).toHaveValue('A lease.');
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    expect(handlers.onApprove).toHaveBeenLastCalledWith('2024-03-02 Lease Agreement.pdf', 'A lease.');
    view.unmount();

    // Off screen while it changed: the same.
    drafts.clear();
    const again = render(<ReviewInspector item={lease} drawer={false} drafts={drafts} {...handlers} />);
    fireEvent.change(description(), { target: { value: 'A lease of Unit 4.' } });
    again.unmount();
    expect(drafts.get('lease')).toMatchObject({ description: 'A lease of Unit 4.', edited: true });
    render(<ReviewInspector item={redescribed} drawer={false} drafts={drafts} {...handlers} />);
    expect(filename()).toHaveValue('2024-03-01 Lease');
    expect(description()).toHaveValue('A lease of the ground floor.');
  });

  // An edit and the item that replaces its own can reach the panel in one
  // render, before any commit has shown the edit: it is kept all the same.
  it('is kept when review moves on in the render that would have shown the edit', () => {
    const drafts: ReviewDrafts = new Map();
    const handlers = calls();
    function Review() {
      const [item, setItem] = useState(lease);
      return <>
        <button type="button" onClick={() => { fireEvent.change(filename(), { target: { value: '2024-03-01 Lease of Unit 4' } }); setItem(invoice); }}>Type and move on</button>
        <button type="button" onClick={() => setItem(lease)}>Back</button>
        <ReviewInspector item={item} drawer={false} drafts={drafts} {...handlers} />
      </>;
    }
    render(<Review />);
    fireEvent.click(screen.getByRole('button', { name: 'Type and move on' }));
    expect(filename()).toHaveValue('2024-04-01 Invoice');
    fireEvent.click(screen.getByRole('button', { name: 'Back' }));
    expect(filename()).toHaveValue('2024-03-01 Lease of Unit 4');
  });

  // Out of the fields, which show the document as it now is - and not thrown
  // away: it used to be forgotten here, and a reading nobody asked for threw
  // away a name and a sentence. It is set aside, for the panel to show.
  it('is out of the fields once its document is seen with another proposal or none, and not for a document missing from the list', () => {
    const typedOver = (item: QueueItem) => ({ item: item.id, for: proposalOf(item), stem: 'typed', description: 'typed', edited: true });
    const drafts: ReviewDrafts = new Map([[lease.id, typedOver(lease)], [invoice.id, typedOver(invoice)]]);
    const earlier: EarlierDrafts = new Map();

    // Nothing changed, and a list the invoice is not in: both are kept.
    expect(setAsideOutdatedDrafts(drafts, earlier, [lease, invoice])).toEqual([]);
    setAsideOutdatedDrafts(drafts, earlier, [lease]);
    setAsideOutdatedDrafts(drafts, earlier, []);
    expect([...drafts.keys()]).toEqual(['lease', 'invoice']);
    expect(earlier.size).toBe(0);

    // Sent back to be read: no proposal. Read again, the same proposal is another reading.
    expect(setAsideOutdatedDrafts(drafts, earlier, [{ id: 'lease', originalFilename: 'scan.pdf', status: 'waiting' }, invoice])).toEqual(['lease']);
    expect([...drafts.keys()]).toEqual(['invoice']);
    setAsideOutdatedDrafts(drafts, earlier, [lease, { ...invoice, proposalRevision: '2' }]);
    expect([...drafts.keys()]).toEqual([]);
    // Each is set aside whole, the name and the description under it, and counts as typed and not filed.
    expect(earlier.get('lease')).toMatchObject([{ text: 'typed\n\ntyped', edited: true }]);
    expect(earlier.get('invoice')).toMatchObject([{ text: 'typed\n\ntyped', edited: true }]);
    expect(typedAndNotFiled(drafts, earlier, 'lease')).toBe(true);
    expect(typedAndNotFiled(drafts, earlier, 'someone-else')).toBe(false);

    // A document that can no longer be filed under the proposal it has - filed from another machine - has gone from under what was typed too.
    drafts.set(lease.id, typedOver(lease));
    expect(setAsideOutdatedDrafts(drafts, earlier, [{ ...lease, status: 'completed' }])).toEqual(['lease']);
    expect(earlier.get('lease')).toHaveLength(2);
    expect(new Set(earlier.get('lease')!.map((kept) => kept.key)).size).toBe(2);
    // Not while a decision of the person's own is being carried out for it: that decision says what becomes of it.
    drafts.set(invoice.id, typedOver(invoice));
    expect(setAsideOutdatedDrafts(drafts, earlier, [{ ...invoice, status: 'completed' }], 'invoice')).toEqual([]);
    expect([...drafts.keys()]).toEqual(['invoice']);
    drafts.clear();

    // Another name or another sentence under the same revision is another proposal.
    expect(proposalOf({ ...lease, proposedFilename: '2024-03-02 Lease.pdf' })).not.toBe(proposalOf(lease));
    expect(proposalOf({ ...lease, description: 'Another sentence.' })).not.toBe(proposalOf(lease));
    expect(proposalOf({ ...lease, id: 'lease-2' })).not.toBe(proposalOf(lease));
    // What does not change the proposal does not change what was typed over it.
    expect(proposalOf({ ...lease, status: 'ready', confidence: 0.2, reason: 'Check the date.' })).toBe(proposalOf(lease));
  });

  it('is not shared between panels that were given nowhere to keep it', () => {
    const first = render(<ReviewInspector item={lease} drawer={false} {...calls()} />);
    fireEvent.change(filename(), { target: { value: '2024-03-01 Lease of Unit 4' } });
    first.unmount();
    render(<ReviewInspector item={lease} drawer={false} {...calls()} />);
    expect(filename()).toHaveValue('2024-03-01 Lease');
  });
});
