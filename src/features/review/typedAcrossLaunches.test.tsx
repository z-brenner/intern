import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { detailsToggle, moreAction, openDetails, queueCell, selectRow } from '../queue/gridTesting';
import type { QueueItem } from '../../types';
import { TYPED_KEY } from './typedStore';

/*
  What was typed for a name and a description in review, and not filed, was
  kept by the window and by nothing else. Closing Intern threw it away: a
  crash, the restart an update makes, or the window's own close button, with
  no notice - a morning's corrections for twenty documents, typed and not
  yet filed, gone with the installer's restart.

  It is stored as it changes, and put back when Intern starts and reads its
  queue: in the fields when the document has the reading it was typed over,
  set aside read-only beside another, dropped with a document that is no
  longer in the queue. In the fields only after an ordinary close: a window
  that went with no word to the page (killed, the power cut) leaves no mark
  in storage, and what the next one finds is all set aside. Here closing
  Intern is the window told it is going (`pagehide`) and taken down, a kill
  is the window taken down untold, and starting it is a new window on a
  queue of its own; what is stored is what jsdom's storage holds between
  the two.
*/
describe('what was typed in review and not filed, after ATLAS is closed and opened', () => {
  afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks(); });

  const LEASE = 'Lease Agreement - 123 Main St.pdf';
  const EMPLOYMENT = 'Employment Agreement - John Smith.pdf';
  const PROPOSED = '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc';
  const PROPOSED_DESCRIPTION = 'Commercial lease agreement between landlord and tenant for 123 Main St.';
  const TYPED = `${PROPOSED} v2 signed`;
  const DESCRIBED = 'Countersigned copy received 3 May; clause 7 amended by hand.';
  const REREAD = { status: 'review', proposedFilename: '2023-09-15 Lease between ABC Properties and TenantCo.pdf', description: 'A lease of 123 Main St.', proposalRevision: '1' } as const;

  /** A window of a fixed width: at 1100px and below the review panel is a drawer. */
  const windowOf = (narrow: boolean) => vi.stubGlobal('matchMedia', (query: string) => ({
    matches: query === '(max-width: 1100px)' && narrow,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
  /**
   * Intern starting: a window on a queue as the demo starts with it, or with
   * `changed` made of it while Intern was closed - a document's fields, or
   * `'gone'`. `close` takes the window down as an ordinary close does, the
   * page told first; `kill` takes it down untold.
   */
  const launch = async (changed: Record<string, Partial<QueueItem> | 'gone'> = {}, options: Parameters<typeof createInMemoryBridge>[0] = {}) => {
    const base = createInMemoryBridge(options);
    const meanwhile = new Map(Object.entries(changed));
    // What the person decides is the queue's own from then on.
    const decided = <Call extends [string, ...unknown[]], Result>(command: (...call: Call) => Promise<Result>) => vi.fn(async (...call: Call) => { const result = await command(...call); meanwhile.delete(call[0]); return result; });
    const bridge = {
      ...base,
      approve: decided(base.approve), keepOriginal: decided(base.keepOriginal), remove: decided(base.remove), reanalyze: decided(base.reanalyze),
      listItems: async () => (await base.listItems()).flatMap((item) => {
        const change = meanwhile.get(item.id);
        return change === 'gone' ? [] : [change ? { ...item, ...change } as QueueItem : item];
      }),
    };
    const view = render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' }, { timeout: 5000 });
    return { bridge, close: () => { window.dispatchEvent(new Event('pagehide')); view.unmount(); }, kill: () => view.unmount() };
  };
  const panel = () => screen.queryByRole('complementary', { name: 'Review item' }) ?? screen.queryByRole('dialog', { name: 'Review item' });
  const filename = () => screen.getByLabelText('Filename');
  const description = () => screen.getByLabelText('Description');
  const kept = () => screen.queryAllByLabelText('What you had typed');
  const type = () => {
    fireEvent.change(filename(), { target: { value: TYPED } });
    fireEvent.change(description(), { target: { value: DESCRIBED } });
  };
  const expectTyped = () => {
    expect(filename()).toHaveValue(TYPED);
    expect(description()).toHaveValue(DESCRIBED);
  };
  const expectProposed = () => {
    expect(filename()).toHaveValue(PROPOSED);
    expect(description()).toHaveValue(PROPOSED_DESCRIPTION);
  };
  const said = () => screen.getByRole('status', { name: 'Action status' });

  it('is in the fields again, with nothing given the focus for it', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.close();

    // The panel was left open, so it is open on the first document to review: the Lease, as it was typed.
    const second = await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectTyped();
    expect(kept()).toEqual([]);
    // Nothing has the focus: no key typed now goes into a name, or files one.
    expect(document.body).toHaveFocus();
    expect(screen.getByRole('button', { name: 'Back to ATLAS\'s proposal' })).toBeInTheDocument();
    // A second close and start with nothing typed in between: still there.
    second.close();
    const third = await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectTyped();
    expect(third.bridge.approve).not.toHaveBeenCalled();
    // Filed, it is what was typed that is filed.
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    await waitFor(() => expect(third.bridge.approve).toHaveBeenCalledWith('lease', `${TYPED}.pdf`, DESCRIBED));
  });

  it('is set aside, not in its fields, when the window before went without closing in the ordinary way', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.kill();

    // The Lease as it was typed over, and all the same its fields are Intern's proposal: what was typed is beside them, read-only.
    const second = await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectProposed();
    expect(kept()).toHaveLength(1);
    expect(kept()[0]).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
    expect(kept()[0]).toHaveAttribute('readonly');
    expect(document.body).toHaveFocus();
    expect(screen.queryByRole('button', { name: 'Back to ATLAS\'s proposal' })).not.toBeInTheDocument();
    // Closed in the ordinary way this time: set aside is how it stays.
    second.close();
    const third = await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectProposed();
    expect(kept()[0]).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
    // What is filed is the name on screen, not the one that was put back.
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    await waitFor(() => expect(third.bridge.approve).toHaveBeenCalledWith('lease', `${PROPOSED}.pdf`, PROPOSED_DESCRIPTION));
  });

  it('goes from storage with a document that Clear queue or Clear completed takes out of the queue, and with no other', async () => {
    windowOf(false);
    type Changing = { change(id: string, change: Partial<QueueItem>): void };
    const EMPLOYMENT_TYPED = '2024-01-15 Employment Agreement, signed copy';
    /** A window with something typed for the Employment Agreement and for the Lease, and the Lease then changed by itself. */
    const typedThenChanged = async (change: Partial<QueueItem>, bridgeOver: (base: ReturnType<typeof createInMemoryBridge>) => Partial<ReturnType<typeof createInMemoryBridge>> = () => ({})) => {
      window.localStorage.clear();
      let queue: Changing | undefined;
      const base = createInMemoryBridge({ liveEvents: true, byItself: (changing) => { queue = changing; } });
      const view = render(<App bridge={{ ...base, ...bridgeOver(base) }} />);
      await screen.findByRole('grid', { name: 'Queue' }, { timeout: 5000 });
      await openDetails(EMPLOYMENT);
      fireEvent.change(filename(), { target: { value: EMPLOYMENT_TYPED } });
      selectRow(LEASE);
      await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
      type();
      await act(async () => { queue!.change('lease', change); await new Promise((resolve) => setTimeout(resolve, 0)); });
      await waitFor(() => expect(window.localStorage.getItem(TYPED_KEY)).toContain('"earlier"'));
      expect(window.localStorage.getItem(TYPED_KEY)).toContain(TYPED);
      return view;
    };
    // Clear queue, here taking the Lease and keeping the rest - as it keeps
    // a document whose files need checking - so the Employment Agreement is
    // a document still in the queue.
    const clearQueue = async () => {
      fireEvent.click(moreAction('Clear queue…'));
      fireEvent.click(screen.getByRole('button', { name: 'Clear queue' }));
      await waitFor(() => expect(said()).toHaveTextContent(/^Cleared 1 document from the queue\./));
    };
    const keepingEmployment = (base: ReturnType<typeof createInMemoryBridge>) => async () => {
      await base.remove('lease');
      return { cleared: 1, unsettled: 0, busy: 0, stillMarked: 0 };
    };

    // The Lease went back to waiting by itself, with what was typed for it set aside; Clear queue takes it out of the queue.
    let view = await typedThenChanged({ status: 'waiting', proposedFilename: undefined, description: undefined }, (base) => ({ clearQueue: keepingEmployment(base) }));
    await clearQueue();
    expect(screen.queryByText(LEASE)).not.toBeInTheDocument();
    // What was typed for it is out of storage now, not at the next start; what was typed for a document still in the queue stays.
    expect(window.localStorage.getItem(TYPED_KEY)).not.toContain(TYPED);
    expect(window.localStorage.getItem(TYPED_KEY)).toContain(EMPLOYMENT_TYPED);
    view.unmount();

    // Filed from somewhere else, it is in the history; Clear completed takes it out.
    view = await typedThenChanged({ status: 'completed', filedName: `${PROPOSED}.pdf` });
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    fireEvent.click(moreAction('Clear completed…'));
    fireEvent.click(screen.getByRole('button', { name: 'Clear completed' }));
    await waitFor(() => expect(said()).toHaveTextContent(/^Cleared \d+ completed documents?\./));
    expect(window.localStorage.getItem(TYPED_KEY)).not.toContain(TYPED);
    expect(window.localStorage.getItem(TYPED_KEY)).toContain(EMPLOYMENT_TYPED);
    view.unmount();

    // The queue could not be read after the command: nothing says the Lease is gone, and what was typed for it stays where it was.
    let unreadable = false;
    view = await typedThenChanged({ status: 'waiting', proposedFilename: undefined, description: undefined }, (base) => ({
      clearQueue: async () => { const report = await keepingEmployment(base)(); unreadable = true; return report; },
      listItems: async () => { if (unreadable) throw new Error('The queue could not be read.'); return base.listItems(); },
    }));
    await clearQueue();
    expect(window.localStorage.getItem(TYPED_KEY)).toContain(TYPED);
    expect(window.localStorage.getItem(TYPED_KEY)).toContain(EMPLOYMENT_TYPED);
    view.unmount();
  });

  it('is in the document\'s drawer when asked for in a narrow window, which opens nothing as it starts', async () => {
    windowOf(true);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.close();

    await launch();
    await waitFor(() => expect(queueCell(LEASE).closest('tr')).toHaveAttribute('aria-current', 'true'));
    expect(panel()).not.toBeInTheDocument();
    expect(document.body).toHaveFocus();
    await openDetails(LEASE);
    expectTyped();
  });

  it('is set aside beside the new reading when the document was read again meanwhile', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.close();

    const second = await launch({ lease: REREAD });
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    // The fields are the document as it now is; what was typed is beside them, read-only, and has not taken the focus.
    expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo');
    expect(description()).toHaveValue('A lease of 123 Main St.');
    expect(kept()).toHaveLength(1);
    expect(kept()[0]).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
    expect(kept()[0]).toHaveAttribute('readonly');
    expect(document.body).toHaveFocus();
    second.close();
    // Set aside is how it stays, back under the first reading too, until it is discarded.
    const third = await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectProposed();
    expect(kept()[0]).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Discard what I had' }));
    expect(kept()).toEqual([]);
    expect(window.localStorage.getItem(TYPED_KEY)).toBeNull();
    third.close();
    await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expect(kept()).toEqual([]);
    expectProposed();
  });

  it('is not in what is filed for the new reading, and goes when that is filed', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.close();

    const second = await launch({ lease: REREAD });
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expect(kept()).toHaveLength(1);
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    await waitFor(() => expect(said()).toHaveTextContent(`Renamed ${LEASE}.`));
    expect(second.bridge.approve).toHaveBeenCalledWith('lease', '2023-09-15 Lease between ABC Properties and TenantCo.pdf', 'A lease of 123 Main St.');
    // The decision is what becomes of what was typed for the document: it is not kept for the next start.
    expect(window.localStorage.getItem(TYPED_KEY)).toBeNull();
    second.close();
    await launch();
    await openDetails(LEASE);
    expect(kept()).toEqual([]);
    expectProposed();
  });

  it('is set aside when the document is being read again as ATLAS starts', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.close();

    await launch({ lease: { status: 'processing', stage: 'reading', proposedFilename: undefined, description: undefined, reason: undefined } });
    await openDetails(LEASE);
    expect(screen.queryByLabelText('Filename')).not.toBeInTheDocument();
    expect(kept()).toHaveLength(1);
    expect(kept()[0]).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
    expect(kept()[0]).not.toHaveFocus();
  });

  it('is dropped with a document that is no longer in the queue when ATLAS starts', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.close();
    expect(window.localStorage.getItem(TYPED_KEY)).toContain(TYPED);

    const second = await launch({ lease: 'gone' });
    await waitFor(() => expect(screen.getByRole('grid', { name: 'Queue' })).toBeInTheDocument());
    expect(screen.queryByText(LEASE)).not.toBeInTheDocument();
    expect(kept()).toEqual([]);
    expect(screen.queryByRole('group', { name: /^Kept from/ })).not.toBeInTheDocument();
    // Out of storage as well: the document back at a later start has nothing typed for it.
    expect(window.localStorage.getItem(TYPED_KEY)).toBeNull();
    second.close();
    await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectProposed();
  });

  it('is not put back while the queue cannot be read, and is there once it can', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    first.close();
    const stored = window.localStorage.getItem(TYPED_KEY);

    // A start whose queue never arrives: nothing is dropped for the documents it could not list.
    const base = createInMemoryBridge();
    const unread = render(<App bridge={{ ...base, listItems: () => Promise.reject(new Error('The queue is not there yet.')) }} />);
    await screen.findByRole('alert', { name: 'Queue connection error' });
    expect(window.localStorage.getItem(TYPED_KEY)).toBe(stored);
    unread.unmount();
    expect(window.localStorage.getItem(TYPED_KEY)).toBe(stored);

    await launch();
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectTyped();
  });

  // Each way a person drops what they typed for a document drops it from storage: Intern closed and opened finds none of it.
  const drops: Array<[string, (bridge: Awaited<ReturnType<typeof launch>>['bridge']) => Promise<void>]> = [
    ['files the document', async (bridge) => {
      fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
      await waitFor(() => expect(said()).toHaveTextContent(`Renamed ${LEASE}.`));
      expect(bridge.approve).toHaveBeenCalledWith('lease', `${TYPED}.pdf`, DESCRIBED);
    }],
    ['keeps the original name', async (bridge) => {
      fireEvent.click(screen.getByRole('button', { name: /Keep original/ }));
      await waitFor(() => expect(said()).toHaveTextContent(`Kept ${LEASE} under its own name.`));
      expect(bridge.keepOriginal).toHaveBeenCalledWith('lease');
    }],
    ['removes the document', async (bridge) => {
      fireEvent.click(within(panel()!).getByRole('button', { name: 'More review actions' }));
      fireEvent.click(within(panel()!).getByRole('button', { name: /Remove from queue/ }));
      await waitFor(() => expect(said()).toHaveTextContent(`Removed ${LEASE} from the queue.`));
      expect(bridge.remove).toHaveBeenCalledWith('lease');
    }],
    ['has it analyzed again', async (bridge) => {
      fireEvent.click(within(panel()!).getByRole('button', { name: 'More review actions' }));
      fireEvent.click(within(panel()!).getByRole('button', { name: 'Analyze again' }));
      await waitFor(() => expect(said()).toHaveTextContent('Sent back to be analyzed again.'));
      expect(bridge.reanalyze).toHaveBeenCalledWith('lease');
    }],
    ['presses Back to ATLAS\'s proposal', async () => {
      fireEvent.click(screen.getByRole('button', { name: 'Back to ATLAS\'s proposal' }));
      expectProposed();
    }],
    ['types it back to the proposal', async () => {
      fireEvent.change(filename(), { target: { value: PROPOSED } });
      fireEvent.change(description(), { target: { value: PROPOSED_DESCRIPTION } });
    }],
  ];
  it.each(drops)('is gone when the person %s', async (_how, drop) => {
    windowOf(false);
    const first = await launch();
    await openDetails(LEASE);
    type();
    expect(window.localStorage.getItem(TYPED_KEY)).toContain(TYPED);
    await drop(first.bridge);
    // At once, not when the window closes.
    expect(window.localStorage.getItem(TYPED_KEY)).toBeNull();
    first.close();
    expect(window.localStorage.getItem(TYPED_KEY)).toBeNull();

    // The queue starts as the demo's does, the Lease in review as it was: nothing typed comes back for it.
    await launch();
    await openDetails(LEASE);
    expectProposed();
    expect(kept()).toEqual([]);
    expect(screen.queryByRole('button', { name: 'Back to ATLAS\'s proposal' })).not.toBeInTheDocument();
  });

  it('is left alone by Rename all ready as one typed in this window is: the count, the tooltip and what it says it did', async () => {
    windowOf(false);
    const all = () => screen.getByRole('button', { name: 'Rename all ready' });
    const CORRECTED = '2024-04-12 Employment Agreement with John Smith (corrected)';
    /** What the button shows, and what pressing it does and says. */
    const pressed = async (bridge: Awaited<ReturnType<typeof launch>>['bridge']) => {
      await waitFor(() => expect(all()).toHaveTextContent('Rename all ready 2'));
      const shown = { text: all().textContent, title: all().getAttribute('title'), ready: screen.getByRole('button', { name: /\d+ ready/ }).textContent };
      fireEvent.click(all());
      await waitFor(() => expect(said()).toHaveTextContent(/Renamed \d+ documents?\./));
      return { ...shown, said: said().textContent, toast: screen.getByRole('group', { name: 'Action result' }).textContent, approved: bridge.approve.mock.calls.map(([id, name]) => [id, name]) };
    };

    // Typed in this window, and the button pressed in it.
    const here = await launch();
    await openDetails(EMPLOYMENT);
    fireEvent.change(filename(), { target: { value: CORRECTED } });
    selectRow(LEASE);
    const inThisWindow = await pressed(here.bridge);
    here.close();
    window.localStorage.clear();

    // Typed, Intern closed and opened, and the button pressed with nothing else done.
    const first = await launch();
    await openDetails(EMPLOYMENT);
    fireEvent.change(filename(), { target: { value: CORRECTED } });
    first.close();
    const second = await launch();
    const afterClosing = await pressed(second.bridge);

    expect(afterClosing).toEqual(inThisWindow);
    expect(afterClosing).toMatchObject({
      text: 'Rename all ready 2',
      title: 'Renames the 2 ready documents as ATLAS proposed. 1 with a name or a description you typed is left for you to file.',
      ready: '3 ready',
      said: 'Renamed 2 documents. 1 was left for you: it has a name or a description you typed and did not file.',
      approved: [['nda', '2024-03-01 Non-Disclosure Agreement with Acme Corp.docx'], ['service', '2024-02-28 Service Agreement with BlueSky LLC.pdf']],
    });
    // The document left is still ready, with the name that was typed in its field.
    await openDetails(EMPLOYMENT);
    expect(filename()).toHaveValue(CORRECTED);
  });

  // Put back as the window first shows its queue, not a moment after: the button never offers to rename the document.
  it('is counted out of Rename all ready from the first time the button is shown', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(EMPLOYMENT);
    fireEvent.change(filename(), { target: { value: '2024-04-12 Employment Agreement with John Smith (corrected)' } });
    first.close();

    const button = () => document.querySelector<HTMLElement>('button[aria-label="Rename all ready"]');
    const shown: string[] = [];
    const watch = new MutationObserver((changes) => {
      // What the button said before a change to it, and what it says now.
      for (const change of changes) if (change.type === 'characterData' && change.oldValue !== null && button()?.contains(change.target)) shown.push(`Rename all ready ${change.oldValue}`);
      if (button()) shown.push(button()!.textContent ?? '');
    });
    watch.observe(document.body, { subtree: true, childList: true, characterData: true, characterDataOldValue: true });
    await launch();
    await waitFor(() => expect(button()).toHaveTextContent('Rename all ready 2'));
    await new Promise((resolve) => setTimeout(resolve, 50));
    watch.disconnect();
    expect(shown.length).toBeGreaterThan(0);
    expect([...new Set(shown)]).toEqual(['Rename all ready 2']);
  });

  it('is left alone by Rename all ready when it comes back set aside, too', async () => {
    windowOf(false);
    const first = await launch();
    await openDetails(EMPLOYMENT);
    fireEvent.change(description(), { target: { value: 'Signed by both.' } });
    first.close();

    // Read again while Intern was closed, and ready again under another name.
    const second = await launch({ employment: { proposedFilename: '2024-04-12 Employment Contract with John Smith.pdf', proposalRevision: '1' } });
    const all = () => screen.getByRole('button', { name: 'Rename all ready' });
    await waitFor(() => expect(all()).toHaveTextContent('Rename all ready 2'));
    fireEvent.click(all());
    await waitFor(() => expect(said()).toHaveTextContent('Renamed 2 documents. 1 was left for you: it has a name or a description you typed and did not file.'));
    expect(second.bridge.approve.mock.calls.map(([id]) => id)).toEqual(['nda', 'service']);
  });

  it('is kept by the window as before, and nothing is said, when storage refuses to be read or written', async () => {
    windowOf(false);
    const refuse = () => { throw new DOMException('The quota has been exceeded.', 'QuotaExceededError'); };
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(refuse);
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(refuse);
    vi.spyOn(Storage.prototype, 'removeItem').mockImplementation(refuse);
    const first = await launch();
    await openDetails(LEASE);
    type();
    // Through closing the panel and coming back from another document, as while Intern stays open.
    fireEvent.click(detailsToggle());
    expect(panel()).not.toBeInTheDocument();
    await openDetails(EMPLOYMENT);
    selectRow(LEASE);
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expectTyped();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    // Filed as typed.
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    await waitFor(() => expect(first.bridge.approve).toHaveBeenCalledWith('lease', `${TYPED}.pdf`, DESCRIBED));
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('is not put back from a value ATLAS did not write, which is removed', async () => {
    windowOf(false);
    for (const value of ['not json', '{"version":2,"documents":[]}', '[]', JSON.stringify({ version: 1, documents: [{ id: 'lease', document: 'someone-else', draft: { reading: 'x', stem: 'Not this one', description: '' }, earlier: ['Nor this'] }] })]) {
      window.localStorage.setItem(TYPED_KEY, value);
      const window_ = await launch();
      await openDetails(LEASE);
      expectProposed();
      expect(kept()).toEqual([]);
      expect(window.localStorage.getItem(TYPED_KEY)).toBeNull();
      window_.close();
    }
  });
});
