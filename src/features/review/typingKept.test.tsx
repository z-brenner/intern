import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { clickCell, detailsToggle, openDetails, queueCell, selectRow } from '../queue/gridTesting';
import type { QueueEventSource } from '../../lib/tauriBridge';
import type { QueueItem } from '../../types';

/*
  A name or a description with the caret in it leaves the screen only by
  something the person did.

  The fields can go by themselves. The backend reads a document again with
  nobody asking - every untouched provisional reading at once, when the
  accurate model becomes usable, and typing is not touching - or gives it
  another proposal where it waits; another machine files it; its file is
  taken from the Queue folder; the panel closes as the window changes width.
  The caret went with the field, to nothing, and what the person typed next
  was review's keys. Typed after the Lease left the queue, " subject to
  both." and Ctrl+Enter filed the first document in the queue (the j selects
  it, the panel shows it, Ctrl+Enter approves it): one held for a spelling
  nobody had checked. After a re-reading, ArrowUp and Ctrl+Enter approved the
  document above; " Bank" and Alt+K kept another's original name; " taken",
  Delete, Enter removed one. What had been typed was thrown away as well.

  So what the field held stays on screen, read-only, with the caret in it:
  in the panel while the panel still shows the document, above the queue
  otherwise. No review key acts from a read-only field, the queue never takes
  the focus out of a field, and what was typed is dropped only by the person.
*/
describe('a name or a description being typed', () => {
  afterEach(() => vi.unstubAllGlobals());

  const LEASE = 'Lease Agreement - 123 Main St.pdf';
  const EMPLOYMENT = 'Employment Agreement - John Smith.pdf';
  const NDA = 'NDA - Acme Corp.docx';
  const SERVICE = 'Service Agreement - BlueSky LLC.pdf';
  const PROPOSED = '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc';
  const TYPED = `${PROPOSED} v2 signed`;
  const DESCRIBED = 'Countersigned copy received 3 May; clause 7 amended by hand';
  const KEPT = `${TYPED}\n\n${DESCRIBED}`;
  const REREAD = { status: 'review', proposedFilename: '2023-09-15 Lease between ABC Properties and TenantCo.pdf', description: 'A lease of 123 Main St.', proposalRevision: '1' } as const;
  const BEING_READ = { status: 'processing', stage: 'reading', proposedFilename: undefined, description: undefined, reason: undefined } as const;

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

  /**
   * A queue whose documents change with nobody asking, as the backend's do:
   * `byItself` changes one - or takes it out of the queue, `'gone'` - and
   * says the queue changed.
   */
  const start = async (options: Parameters<typeof createInMemoryBridge>[0] = {}) => {
    const base = createInMemoryBridge({ liveEvents: true, ...options }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    const changes = new Map<string, Partial<QueueItem> | 'gone'>();
    let queueChanged = () => {};
    // What the person decides is the queue's own from then on.
    const decided = <Call extends [string, ...unknown[]], Result>(command: (...call: Call) => Promise<Result>) => vi.fn(async (...call: Call) => { const result = await command(...call); changes.delete(call[0]); return result; });
    const bridge = {
      ...base,
      approve: decided(base.approve), keepOriginal: decided(base.keepOriginal), remove: decided(base.remove), reanalyze: decided(base.reanalyze), pauseQueue: vi.fn(base.pauseQueue),
      listItems: async () => (await base.listItems()).flatMap((item) => {
        const change = changes.get(item.id);
        return change === 'gone' ? [] : [change ? { ...item, ...change } as QueueItem : item];
      }),
      subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); },
    };
    const byItself = async (id: string, change: Partial<QueueItem> | 'gone') => {
      changes.set(id, change);
      await act(async () => { queueChanged(); await new Promise((resolve) => setTimeout(resolve, 0)); });
    };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    return { bridge, byItself, changes };
  };
  const panel = () => screen.queryByRole('complementary', { name: /Review item|Filed document/ }) ?? screen.queryByRole('dialog', { name: /Review item|Filed document/ });
  const filename = () => screen.getByLabelText<HTMLTextAreaElement>('Filename');
  const description = () => screen.getByLabelText<HTMLTextAreaElement>('Description');
  const kept = () => screen.queryByLabelText<HTMLTextAreaElement>('What you had typed');
  const allKept = () => screen.queryAllByLabelText<HTMLTextAreaElement>('What you had typed');
  const selectedRow = () => screen.queryByRole('row', { current: true });
  /** The Lease open in review, a name and a description typed and not filed, and the caret left in one of them. */
  const typeInTheLease = async (caretIn: 'Filename' | 'Description' | 'neither' = 'Description') => {
    await openDetails(LEASE);
    fireEvent.change(filename(), { target: { value: TYPED } });
    fireEvent.change(description(), { target: { value: DESCRIBED } });
    if (caretIn !== 'neither') screen.getByLabelText(caretIn).focus();
  };
  /**
   * A key pressed where the focus is, with what the browser does with it
   * when nothing handles it: Enter and the space bar press a focused button,
   * a character goes into a field that can be typed in, and Tab moves the
   * focus to the next control that takes it (Shift+Tab, the one before).
   */
  const press = (key: string, modifiers: { ctrlKey?: boolean; altKey?: boolean; shiftKey?: boolean } = {}) => {
    const at = (document.activeElement ?? document.body) as HTMLElement;
    const unhandled = fireEvent.keyDown(at, { key, ...modifiers });
    if (unhandled && !modifiers.ctrlKey && !modifiers.altKey) {
      if (key === 'Tab') {
        const stops = [...document.querySelectorAll<HTMLElement>('button:not(:disabled), textarea, input, [tabindex="0"]')].filter((stop) => !stop.closest('[inert]'));
        stops[stops.indexOf(at) + (modifiers.shiftKey ? -1 : 1)]?.focus();
      } else if (at instanceof HTMLButtonElement && (key === ' ' || key === 'Enter')) fireEvent.click(at);
      else if (at instanceof HTMLTextAreaElement && !at.readOnly && key.length === 1) fireEvent.change(at, { target: { value: `${at.value}${key}` } });
    }
    fireEvent.keyUp((document.activeElement ?? document.body) as HTMLElement, { key, ...modifiers });
  };
  const type = (text: string) => { for (const key of text) press(key); };
  /**
   * What a person in the middle of a name or a sentence goes on to press:
   * Tab, to go on to the description, and each route that decided a
   * document nobody had looked at.
   */
  const typeOn = () => {
    press('Tab');
    type(' subject to both.');
    press('Enter', { ctrlKey: true });
    press('ArrowUp');
    press('Enter', { ctrlKey: true });
    type(' as taken by Deutsche Bank');
    press('Enter');
    press('Enter');
    press('k', { altKey: true });
    press('Delete');
    press('Enter');
  };
  const nothingDecided = (bridge: Awaited<ReturnType<typeof start>>['bridge']) => {
    expect(bridge.approve).not.toHaveBeenCalled();
    expect(bridge.keepOriginal).not.toHaveBeenCalled();
    expect(bridge.remove).not.toHaveBeenCalled();
    expect(screen.queryByRole('group', { name: 'Confirm removal' })).not.toBeInTheDocument();
  };

  for (const narrow of [false, true]) {
    const where = narrow ? 'in the drawer' : 'beside the queue';

    it(`stays on screen with the caret when its document is read again by itself, ${where}`, async () => {
      windowOf(narrow);
      const { bridge, byItself } = await start();
      await typeInTheLease('Description');

      await byItself('lease', BEING_READ);
      await waitFor(() => expect(screen.queryByLabelText('Description')).not.toBeInTheDocument());
      // What was typed, the name and the description under it, where typing changes nothing.
      expect(kept()).toHaveFocus();
      expect(kept()).toHaveValue(KEPT);
      expect(kept()).toHaveAttribute('readonly');
      expect(panel()).toContainElement(kept());
      expect(panel()).toHaveTextContent(LEASE);

      typeOn();
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(kept()).toHaveFocus();
      expect(kept()).toHaveValue(KEPT);
      expect(panel()).toHaveTextContent(LEASE);
      expect(selectedRow()).toBe(queueCell(LEASE).closest('tr'));
      nothingDecided(bridge);

      // The new reading is shown as it is, beside what was typed; the caret is not in it, and nothing typed goes into it.
      await byItself('lease', REREAD);
      await waitFor(() => expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo'));
      expect(description()).toHaveValue('A lease of 123 Main St.');
      expect(kept()).toHaveFocus();
      expect(kept()).toHaveValue(KEPT);
      typeOn();
      expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo');
      expect(description()).toHaveValue('A lease of 123 Main St.');
      expect(selectedRow()).toBe(queueCell(LEASE).closest('tr'));
      nothingDecided(bridge);

      // It goes when the person says so, and the focus goes to the panel's heading, not into the name.
      fireEvent.click(screen.getByRole('button', { name: 'Discard what I had' }));
      await waitFor(() => expect(kept()).not.toBeInTheDocument());
      await waitFor(() => expect(screen.getByRole('heading', { name: 'Review item' })).toHaveFocus());
      nothingDecided(bridge);
    });

    it(`stays on screen with the caret when its document leaves the queue by itself, ${where}`, async () => {
      windowOf(narrow);
      const { bridge, byItself } = await start();
      await typeInTheLease('Description');

      await byItself('lease', 'gone');
      await waitFor(() => expect(panel()).not.toBeInTheDocument());
      // Above the queue, which is not under a drawer any more.
      const queue = screen.getByRole('region', { name: 'Queue items' });
      expect(queue).not.toHaveAttribute('inert');
      expect(queue).toContainElement(kept());
      expect(kept()).toHaveFocus();
      expect(kept()).toHaveValue(KEPT);
      expect(within(queue).getByRole('group', { name: `Kept from ${LEASE}` })).toHaveTextContent('left the queue while you were in its name or description');

      typeOn();
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(kept()).toHaveFocus();
      expect(selectedRow()).not.toBeInTheDocument();
      expect(panel()).not.toBeInTheDocument();
      nothingDecided(bridge);
      expect((await bridge.listItems()).map((item) => item.status)).toEqual(['ready', 'ready', 'processing', 'ready', 'waiting', 'waiting', 'waiting', 'completed']);

      // Done takes it away, and the focus goes to the button of the view shown, where Enter and the space bar change nothing.
      fireEvent.click(within(queue).getByRole('button', { name: 'Done' }));
      await waitFor(() => expect(kept()).not.toBeInTheDocument());
      await waitFor(() => expect(screen.getByRole('button', { name: /^Queue, \d+$/ })).toHaveFocus());
      nothingDecided(bridge);
    });
  }

  it('stays on screen with the caret when its document is given another proposal by itself, and the caret does not go into the new name', async () => {
    windowOf(false);
    const { bridge, byItself } = await start();
    await typeInTheLease('Filename');

    await byItself('lease', REREAD);
    await waitFor(() => expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo'));
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(KEPT);
    expect(filename()).not.toHaveFocus();

    // The rest of the name, and the Enter that was to file it.
    type(' Deutsche Bank');
    press('Enter');
    press('Enter', { ctrlKey: true });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo');
    expect(kept()).toHaveFocus();
    nothingDecided(bridge);
  });

  it('stays on screen with the caret when its document is filed from another machine', async () => {
    windowOf(false);
    const { bridge, byItself } = await start();
    await typeInTheLease('Description');

    // Filed elsewhere, under the proposal it had: completed, with no fields.
    await byItself('lease', { status: 'completed', filedName: `${PROPOSED}.pdf` });
    await waitFor(() => expect(screen.queryByLabelText('Description')).not.toBeInTheDocument());
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(KEPT);
    expect(panel()).toContainElement(kept());
    typeOn();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(kept()).toHaveFocus();
    nothingDecided(bridge);
  });

  it('stays on screen with the caret when nothing had been typed, and when the caret was in what is already kept', async () => {
    windowOf(false);
    const { bridge, byItself } = await start();
    await openDetails(LEASE);
    description().focus();

    await byItself('lease', BEING_READ);
    await waitFor(() => expect(screen.queryByLabelText('Description')).not.toBeInTheDocument());
    // The fields as they were: the proposal nobody had typed over.
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(`${PROPOSED}\n\nCommercial lease agreement between landlord and tenant for 123 Main St.`);
    typeOn();
    nothingDecided(bridge);

    // The document leaves the queue with the caret in what the panel keeps: it is kept above the queue instead.
    await byItself('lease', 'gone');
    await waitFor(() => expect(panel()).not.toBeInTheDocument());
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(`${PROPOSED}\n\nCommercial lease agreement between landlord and tenant for 123 Main St.`);
    expect(screen.getByRole('region', { name: 'Queue items' })).toContainElement(kept());
    typeOn();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(selectedRow()).not.toBeInTheDocument();
    nothingDecided(bridge);
  });

  it('stays on screen with the caret when the panel closes as the window changes width, and is in the fields when the panel is asked for again', async () => {
    const window_ = windowOf(false);
    const { bridge } = await start();
    await typeInTheLease('Description');

    // Narrower than 1100px, a drawer opens only when asked: the panel goes.
    window_.resize('narrow');
    expect(panel()).not.toBeInTheDocument();
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(KEPT);
    expect(screen.getByRole('group', { name: `Kept from ${LEASE}` })).toHaveTextContent('closed while you were in its name or description. What you typed is kept with the document');
    typeOn();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(kept()).toHaveFocus();
    expect(panel()).not.toBeInTheDocument();
    expect(selectedRow()).toBe(queueCell(LEASE).closest('tr'));
    nothingDecided(bridge);

    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    await waitFor(() => expect(kept()).not.toBeInTheDocument());
    fireEvent.click(detailsToggle());
    await screen.findByRole('dialog', { name: 'Review item' });
    expect(filename()).toHaveValue(TYPED);
    expect(description()).toHaveValue(DESCRIBED);

    // Wider again, the panel was left open there: it stays, as the panel beside the queue, and so does the caret.
    description().focus();
    window_.resize('wide');
    expect(panel()).toHaveAttribute('role', 'complementary');
    expect(description()).toHaveFocus();
    expect(kept()).not.toBeInTheDocument();
    nothingDecided(bridge);
  });

  it('stays on screen with the caret when a drawer closes as the window is made wider', async () => {
    const window_ = windowOf(true);
    const { bridge } = await start();
    await typeInTheLease('Filename');
    expect(panel()).toHaveAttribute('role', 'dialog');

    // Wide, the panel is open only if it was left open: here it was not.
    window_.resize('wide');
    expect(panel()).not.toBeInTheDocument();
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(KEPT);
    typeOn();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(kept()).toHaveFocus();
    expect(selectedRow()).toBe(queueCell(LEASE).closest('tr'));
    nothingDecided(bridge);
    // Narrow again, nothing opens by itself, and the caret stays where it is.
    window_.resize('narrow');
    expect(panel()).not.toBeInTheDocument();
    expect(kept()).toHaveFocus();
    nothingDecided(bridge);
  });

  // A decision of the person's own still running - here, pausing the queue -
  // is not what took the field: it is held all the same.
  it('stays on screen with the caret while another action of the person\'s own is still running', async () => {
    windowOf(false);
    const { bridge, byItself } = await start();
    let finish = () => {};
    bridge.pauseQueue.mockImplementation(() => new Promise<void>((resolve) => { finish = resolve; }));
    await typeInTheLease('Description');
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));
    await waitFor(() => expect(bridge.pauseQueue).toHaveBeenCalled());

    await byItself('lease', BEING_READ);
    await waitFor(() => expect(screen.queryByLabelText('Description')).not.toBeInTheDocument());
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(KEPT);
    await act(async () => { finish(); await new Promise((resolve) => setTimeout(resolve, 0)); });
    // The action finishing does not take the caret either.
    expect(kept()).toHaveFocus();
    typeOn();
    nothingDecided(bridge);
  });

  // Tab is how a person goes from the name on to the description. From what
  // is kept it went to the button under it, where a space typed next pressed
  // it - what was kept, thrown away - and J, K, Ctrl+Enter and Alt+K were
  // review's keys again: Ctrl+Enter there filed the reading that had just
  // replaced the one being corrected.
  it('holds the caret against Tab, and lets it go with Escape, to its own button', async () => {
    windowOf(true);
    const { bridge, byItself } = await start();
    await typeInTheLease('Filename');
    // Another proposal, by itself: the panel offers Approve for it.
    await byItself('lease', REREAD);
    await waitFor(() => expect(kept()).toHaveFocus());
    expect(screen.getByRole('button', { name: /Approve & rename/ })).toBeEnabled();

    press('Tab');
    expect(kept()).toHaveFocus();
    press('Tab', { shiftKey: true });
    expect(kept()).toHaveFocus();
    type('Signed by both, subject to clause 7.');
    press('Enter', { ctrlKey: true });
    press('k', { altKey: true });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(kept()).toHaveFocus();
    expect(kept()).toHaveValue(KEPT);
    expect(screen.getByRole('group', { name: 'Kept from before this document changed' })).toHaveTextContent('Tab stays here: press Escape, or click, to leave it.');
    nothingDecided(bridge);

    // Escape: to its button, and the drawer stays open.
    press('Escape');
    expect(screen.getByRole('button', { name: 'Discard what I had' })).toHaveFocus();
    expect(panel()).toBeInTheDocument();
    expect(kept()).toHaveValue(KEPT);
    // From a button the keyboard is the person's again: Escape closes the drawer, as from any other.
    press('Escape');
    expect(panel()).not.toBeInTheDocument();
    nothingDecided(bridge);
    // What was kept is still with the document.
    await openDetails(LEASE);
    expect(kept()).toHaveValue(KEPT);
  });

  // A decision the backend refuses is not what took the field either. The
  // reading is taken back just as Enter files the name: the approval is
  // refused, and the queue read for the refusal shows the document being
  // read. The person, who pressed Enter to go on to the next name, types on.
  it('stays on screen with the caret when the document is read again by itself as its name is filed, and the filing is refused', async () => {
    windowOf(false);
    const { bridge, changes } = await start();
    await typeInTheLease('Filename');
    bridge.approve.mockImplementation(async () => { changes.set('lease', BEING_READ); throw { code: 'STATE_CONFLICT', message: 'item is not awaiting a decision' }; });
    press('Enter');
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByLabelText('Filename')).not.toBeInTheDocument());
    await waitFor(() => expect(kept()).toHaveFocus());
    expect(kept()).toHaveValue(KEPT);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Pause queue' })).toBeEnabled());
    typeOn();
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(kept()).toHaveFocus();
    expect(selectedRow()).toBe(queueCell(LEASE).closest('tr'));
    expect(bridge.approve).toHaveBeenCalledTimes(1);
    expect(bridge.keepOriginal).not.toHaveBeenCalled();
    expect(bridge.remove).not.toHaveBeenCalled();
    expect(screen.queryByRole('group', { name: 'Confirm removal' })).not.toBeInTheDocument();
  });

  /*
    What the person does takes the fields as it always has: nothing is kept
    on screen, and the focus goes where that action sends it.
  */
  it('goes with the fields when the person decides, moves on, closes the panel or has the document read again', async () => {
    const window_ = windowOf(false);
    const { bridge } = await start({ analysisDelayMs: 20 });

    // Alt+N from the name: the next undecided document's name has the caret.
    await typeInTheLease('Filename');
    press('n', { altKey: true });
    await waitFor(() => expect(panel()).toHaveTextContent(EMPLOYMENT));
    await waitFor(() => expect(filename()).toHaveFocus());
    expect(kept()).not.toBeInTheDocument();

    // Another row, and Close review, pressed with the caret still in a field (a press that does not take the focus, as on a Mac).
    description().focus();
    selectRow(NDA);
    await waitFor(() => expect(panel()).toHaveTextContent(NDA));
    expect(kept()).not.toBeInTheDocument();
    description().focus();
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Close review' }));
    expect(panel()).not.toBeInTheDocument();
    expect(kept()).not.toBeInTheDocument();

    // Escape in the drawer, from the description.
    window_.resize('narrow');
    await openDetails(NDA);
    description().focus();
    press('Escape');
    expect(panel()).not.toBeInTheDocument();
    expect(kept()).not.toBeInTheDocument();
    window_.resize('wide');

    // The other view, with the caret in a field.
    await openDetails(NDA);
    description().focus();
    fireEvent.click(screen.getByRole('button', { name: /^Completed,/ }));
    expect(panel()).not.toBeInTheDocument();
    expect(kept()).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Queue,/ }));

    // Analyze again, pressed with the caret in the name: what was typed goes with the reading it was typed over.
    await openDetails(LEASE);
    expect(filename()).toHaveValue(TYPED);
    filename().focus();
    fireEvent.click(within(panel()!).getByRole('button', { name: 'More review actions' }));
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Analyze again' }));
    await waitFor(() => expect(bridge.reanalyze).toHaveBeenCalledWith('lease'));
    await waitFor(() => expect(queueCell(LEASE, 'status')).toHaveTextContent(/review/i), { timeout: 5000 });
    expect(kept()).not.toBeInTheDocument();
    await openDetails(LEASE);
    expect(filename()).toHaveValue(PROPOSED);
    expect(kept()).not.toBeInTheDocument();

    // Enter in the name files it, and review moves on with the caret in the next name.
    fireEvent.change(filename(), { target: { value: TYPED } });
    filename().focus();
    press('Enter');
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('lease', `${TYPED}.pdf`, expect.any(String)));
    await waitFor(() => expect(panel()).toHaveTextContent(EMPLOYMENT));
    await waitFor(() => expect(filename()).toHaveFocus());
    expect(kept()).not.toBeInTheDocument();
    // Ctrl+Enter from the description, and Alt+K from the name.
    description().focus();
    press('Enter', { ctrlKey: true });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(panel()).toHaveTextContent(NDA));
    await waitFor(() => expect(filename()).toHaveFocus());
    press('k', { altKey: true });
    await waitFor(() => expect(bridge.keepOriginal).toHaveBeenCalledWith('nda'));
    await waitFor(() => expect(panel()).toHaveTextContent(SERVICE));
    expect(kept()).not.toBeInTheDocument();
    // What was filed left nothing behind for the filed documents either.
    fireEvent.click(screen.getByRole('button', { name: /^Completed,/ }));
    await openDetails(LEASE);
    expect(kept()).not.toBeInTheDocument();
  });

  // The queue takes the focus back when what had it has gone - never out of
  // a field someone is typing in.
  it('is not interrupted by the queue taking the focus back: the filter keeps the caret when an action finishes', async () => {
    windowOf(false);
    const { bridge } = await start();
    let finish = () => {};
    bridge.reanalyze.mockImplementation(() => new Promise<void>((resolve) => { finish = resolve; }));
    await openDetails(LEASE);
    fireEvent.click(within(panel()!).getByRole('button', { name: 'More review actions' }));
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Analyze again' }));
    await waitFor(() => expect(bridge.reanalyze).toHaveBeenCalled());
    // The person goes on to the filter while the document is sent back.
    const filter = screen.getByRole('searchbox', { name: 'Filter queue' });
    filter.focus();
    fireEvent.change(filter, { target: { value: 'agree' } });
    await act(async () => { finish(); await new Promise((resolve) => setTimeout(resolve, 20)); });
    await waitFor(() => expect(panel()).not.toBeInTheDocument());
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(filter).toHaveFocus();
    expect(filter).toHaveValue('agree');
  });

  // The filter box is shown above a view of more than six rows. A view that
  // shrank to six by itself took the empty box from under the caret, and
  // the word typed for it was review's keys.
  it('has a field to go into: the filter box stays while the caret is in it, however few rows are left', async () => {
    windowOf(false);
    const { bridge, byItself } = await start();
    const filter = () => screen.queryByRole<HTMLInputElement>('searchbox', { name: 'Filter queue' });
    filter()!.focus();
    // The document Intern selects at launch.
    await waitFor(() => expect(selectedRow()).toHaveTextContent(LEASE));
    await byItself('minutes', 'gone');
    await byItself('invoice', 'gone');
    await waitFor(() => expect(screen.getAllByRole('row')).toHaveLength(7));
    expect(filter()).toHaveFocus();
    fireEvent.keyDown(filter()!, { key: 'j' });
    fireEvent.change(filter()!, { target: { value: 'j' } });
    fireEvent.keyDown(filter()!, { key: 'Enter' });
    expect(filter()).toHaveValue('j');
    expect(panel()).not.toBeInTheDocument();
    nothingDecided(bridge);
    // Emptied, the row selected before is the one selected still; left, the box goes: six rows need no filter.
    fireEvent.change(filter()!, { target: { value: '' } });
    await waitFor(() => expect(selectedRow()).toHaveTextContent(LEASE));
    fireEvent.blur(filter()!);
    await waitFor(() => expect(filter()).not.toBeInTheDocument());
  });

  it('is not left behind by an add that finishes while it is typed: the selection stays', async () => {
    windowOf(false);
    const base = createInMemoryBridge();
    let copied = () => {};
    const copyToInput = vi.fn(async (selection: Parameters<typeof base.copyToInput>[0]) => { await new Promise<void>((resolve) => { copied = resolve; }); return base.copyToInput(selection); });
    const bridge = { ...base, copyToInput, approve: vi.fn(base.approve) };
    render(<App bridge={bridge} selection={{ pickFiles: async () => [{ path: 'C:/Inbox/Addendum.pdf', displayName: 'Addendum.pdf' }], pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await openDetails(LEASE);
    fireEvent.click(screen.getByRole('button', { name: 'Add files' }));
    await waitFor(() => expect(copyToInput).toHaveBeenCalled());
    // Typing the Lease's description while the copy runs.
    fireEvent.change(description(), { target: { value: DESCRIBED } });
    description().focus();
    await act(async () => { copied(); await new Promise((resolve) => setTimeout(resolve, 20)); });
    expect(await screen.findByRole('row', { name: /Addendum.pdf/i })).toBeVisible();
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(panel()).toHaveTextContent(LEASE);
    expect(description()).toHaveFocus();
    expect(description()).toHaveValue(DESCRIBED);
    expect(kept()).not.toBeInTheDocument();
    expect(selectedRow()).toBe(queueCell(LEASE).closest('tr'));
  });

  /*
    What was typed is dropped only by the person. A reading nobody asked for
    used to throw it away, with no notice, wherever the caret was and whether
    or not the document was on screen.
  */
  it('is kept beside the document when it is read again by itself, wherever the caret was and whether or not it was on screen', async () => {
    windowOf(false);
    const { bridge, byItself } = await start();
    await typeInTheLease('neither');
    const cell = queueCell(LEASE);
    cell.focus();

    await byItself('lease', BEING_READ);
    await waitFor(() => expect(screen.queryByLabelText('Description')).not.toBeInTheDocument());
    // Kept, and the focus left where the person had it.
    expect(kept()).toHaveValue(KEPT);
    expect(cell).toHaveFocus();
    await byItself('lease', REREAD);
    await waitFor(() => expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo'));
    expect(kept()).toHaveValue(KEPT);
    expect(screen.getByRole('group', { name: 'Kept from before this document changed' })).toHaveTextContent('This document changed while you were working on its name and description.');

    // Typed over the new reading and left for another row; read again by itself while off screen.
    fireEvent.change(filename(), { target: { value: '2023-09-15 Lease of 123 Main St' } });
    selectRow(NDA);
    await waitFor(() => expect(panel()).toHaveTextContent(NDA));
    expect(kept()).not.toBeInTheDocument();
    await byItself('lease', { ...REREAD, description: 'A commercial lease of 123 Main St.' });
    selectRow(LEASE);
    await waitFor(() => expect(panel()).toHaveTextContent(LEASE));
    expect(description()).toHaveValue('A commercial lease of 123 Main St.');
    expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo');
    // Both are there, oldest first, and each goes by its own button.
    expect(allKept().map((field) => field.value)).toEqual([KEPT, '2023-09-15 Lease of 123 Main St\n\nA lease of 123 Main St.']);
    fireEvent.click(screen.getAllByRole('button', { name: 'Discard what I had' })[0]);
    await waitFor(() => expect(allKept().map((field) => field.value)).toEqual(['2023-09-15 Lease of 123 Main St\n\nA lease of 123 Main St.']));
    // A second click of a double-click, landing on the button that moved up, discards nothing.
    fireEvent.click(screen.getByRole('button', { name: 'Discard what I had' }), { detail: 2 });
    expect(allKept()).toHaveLength(1);

    // Filing the document is the person's decision, and takes what is left of it.
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(1));
    fireEvent.click(await screen.findByRole('button', { name: /^Completed,/ }));
    await openDetails(LEASE);
    expect(kept()).not.toBeInTheDocument();
  });

  // An approval the backend accepts can still send the document back to
  // review. What was typed for it is still the person's to file.
  it('is still there when an approval sends the document back to review', async () => {
    windowOf(false);
    const { bridge, changes } = await start();
    await typeInTheLease('Filename');
    // Accepted, and back in review with another reading: the file had changed since it was read.
    bridge.approve.mockImplementation(async () => { changes.set('lease', REREAD); });
    press('Enter');
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(filename()).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo'));
    await waitFor(() => expect(kept()).toHaveValue(`${TYPED}\n\n${DESCRIBED}`));
  });

  /*
    What is typed is kept through closing the panel, and nothing took it
    back: Escape used to, and a field shown again has no undo history. Twelve
    letters deleted from the Lease's name by mistake, Escape, Enter on the
    row, Enter: approve(lease, 2023-09-15 Lease Agreement between ABC
    Properties LLC and.pdf), while the row still read "... and TenantCo Inc".
  */
  it('can be taken back with "Back to ATLAS\'s proposal", which is there only while what is in the fields differs', async () => {
    windowOf(true);
    const { bridge } = await start();
    const back = () => screen.queryByRole('button', { name: 'Back to ATLAS\'s proposal' });
    const PROPOSED_DESCRIPTION = 'Commercial lease agreement between landlord and tenant for 123 Main St.';
    await openDetails(LEASE);
    expect(back()).not.toBeInTheDocument();

    // Typed, and typed back by hand: nothing differs, and there is nothing to take back.
    fireEvent.change(filename(), { target: { value: `${PROPOSED} v2` } });
    expect(back()).toBeInTheDocument();
    fireEvent.change(filename(), { target: { value: PROPOSED } });
    expect(back()).not.toBeInTheDocument();
    fireEvent.change(description(), { target: { value: DESCRIBED } });
    expect(back()).toBeInTheDocument();

    // The name spoiled, the drawer closed with Escape and opened again: it is as it was typed, as before.
    const spoiled = PROPOSED.slice(0, -12);
    fireEvent.change(filename(), { target: { value: spoiled } });
    filename().focus();
    press('Escape');
    expect(panel()).not.toBeInTheDocument();
    await openDetails(LEASE);
    expect(filename()).toHaveValue(spoiled);
    expect(description()).toHaveValue(DESCRIBED);

    // The second click of a double-click takes nothing back. A press puts both back as Intern proposed them,
    // in fields of their own - the old ones' undo steps go with them - and the caret goes to the name.
    const fields = [filename(), description()];
    description().focus();
    fireEvent.click(back()!, { detail: 2 });
    expect(filename()).toHaveValue(spoiled);
    // The press does not take the focus (as on a Mac), and the window draws in the same commit (here, for the filter).
    act(() => { fireEvent.focus(screen.getByRole('searchbox', { name: 'Filter queue' })); fireEvent.click(back()!); });
    expect(filename()).toHaveValue(PROPOSED);
    expect(description()).toHaveValue(PROPOSED_DESCRIPTION);
    expect(back()).not.toBeInTheDocument();
    expect(filename()).toHaveFocus();
    for (const field of fields) expect(field).not.toBeInTheDocument();
    // Pressed with the caret in a field, and the fields went: the person's own doing, so nothing is kept on screen for it.
    expect(kept()).not.toBeInTheDocument();
    nothingDecided(bridge);

    // What was typed is dropped for good: closed and opened again it is Intern's proposal, and Enter in the name files that.
    press('Escape');
    expect(panel()).not.toBeInTheDocument();
    await openDetails(LEASE);
    expect(filename()).toHaveValue(PROPOSED);
    expect(description()).toHaveValue(PROPOSED_DESCRIPTION);
    expect(back()).not.toBeInTheDocument();
    filename().focus();
    press('Enter');
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('lease', `${PROPOSED}.pdf`, PROPOSED_DESCRIPTION));
  });

  // Taken back - by the link, or letter by letter - a name is Intern's again
  // and nobody's draft: Rename all ready counts the document and renames it.
  it('is no longer typed once it is taken back: Rename all ready counts the document again and renames it as proposed', async () => {
    windowOf(false);
    const { bridge } = await start();
    const all = () => screen.getByRole('button', { name: 'Rename all ready' });
    const EMPLOYMENT_PROPOSED = '2024-04-12 Employment Agreement with John Smith';
    await openDetails(EMPLOYMENT);
    fireEvent.change(filename(), { target: { value: `${EMPLOYMENT_PROPOSED} (corrected)` } });
    await waitFor(() => expect(all()).toHaveTextContent('Rename all ready 2'));
    fireEvent.click(screen.getByRole('button', { name: 'Back to ATLAS\'s proposal' }));
    await waitFor(() => expect(all()).toHaveTextContent('Rename all ready 3'));
    expect(all()).not.toHaveAttribute('title');

    fireEvent.change(description(), { target: { value: 'Signed by both.' } });
    await waitFor(() => expect(all()).toHaveTextContent('Rename all ready 2'));
    fireEvent.change(description(), { target: { value: '' } });
    await waitFor(() => expect(all()).toHaveTextContent('Rename all ready 3'));
    // Left for another row and come back to: nothing typed is waiting there.
    selectRow(NDA);
    await waitFor(() => expect(panel()).toHaveTextContent(NDA));
    selectRow(EMPLOYMENT);
    await waitFor(() => expect(panel()).toHaveTextContent(EMPLOYMENT));
    expect(filename()).toHaveValue(EMPLOYMENT_PROPOSED);

    fireEvent.click(all());
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed 3 documents.'));
    expect(bridge.approve.mock.calls.map(([id, name]) => [id, name])).toContainEqual(['employment', `${EMPLOYMENT_PROPOSED}.pdf`]);
    expect(screen.getByRole('status', { name: 'Action status' })).not.toHaveTextContent('left for you');
  });

  /*
    Rename all ready files Intern's proposal for each ready document. For one
    whose name a person had typed and not filed it sent the proposal all the
    same - approve(employment, 2024-04-12 Employment Agreement with John
    Smith.pdf) - said "Renamed 3 documents", and the typed name was gone.
  */
  it('is not filed over by Rename all ready, which leaves the document for the person and says so', async () => {
    windowOf(false);
    const { bridge } = await start();
    await openDetails(EMPLOYMENT);
    fireEvent.change(filename(), { target: { value: '2024-04-12 Employment Agreement with John Smith (corrected)' } });
    // Left for the next row, as review does.
    selectRow(NDA);
    await waitFor(() => expect(panel()).toHaveTextContent(NDA));
    const all = screen.getByRole('button', { name: 'Rename all ready' });
    // Its count is of what it will rename; the documents that are ready are still three.
    await waitFor(() => expect(all).toHaveTextContent('Rename all ready 2'));
    expect(all).toHaveAttribute('title', 'Renames the 2 ready documents as ATLAS proposed. 1 with a name or a description you typed is left for you to file.');
    expect(screen.getByRole('button', { name: /3 ready/ })).toBeInTheDocument();

    fireEvent.click(all);
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed 2 documents. 1 was left for you: it has a name or a description you typed and did not file.'));
    expect(bridge.approve.mock.calls.map(([id, name]) => [id, name])).toEqual([
      ['nda', '2024-03-01 Non-Disclosure Agreement with Acme Corp.docx'],
      ['service', '2024-02-28 Service Agreement with BlueSky LLC.pdf'],
    ]);
    expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('1 was left for you');

    // It is still ready, with the name that was typed, and filing it files that name.
    expect((await bridge.listItems()).find((item) => item.id === 'employment')).toMatchObject({ status: 'ready', proposedFilename: '2024-04-12 Employment Agreement with John Smith.pdf' });
    await openDetails(EMPLOYMENT);
    expect(filename()).toHaveValue('2024-04-12 Employment Agreement with John Smith (corrected)');
    // Nothing is left for the button to rename, and pressed, it renames nothing and says why.
    expect(screen.getByRole('button', { name: 'Rename all ready' })).toHaveTextContent('Rename all ready 0');
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Nothing was renamed. 1 was left for you: it has a name or a description you typed and did not file.'));
    expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Nothing was renamed. 1 was left for you');
    expect(bridge.approve).toHaveBeenCalledTimes(2);
    fireEvent.click(screen.getByRole('button', { name: /Apply rename/ }));
    await waitFor(() => expect(bridge.approve).toHaveBeenLastCalledWith('employment', '2024-04-12 Employment Agreement with John Smith (corrected).pdf', ''));
  });

  it('is not filed over by Rename all ready when it was typed after the press, or set aside when the document changed', async () => {
    windowOf(false);
    const { bridge, byItself } = await start();
    // Typed for the Service Agreement, which is then read again by itself and is ready again.
    await openDetails(SERVICE);
    fireEvent.change(description(), { target: { value: 'Signed by both; the schedule is missing.' } });
    selectRow(LEASE);
    await byItself('service', { proposedFilename: '2024-02-28 Services Agreement with BlueSky.pdf', proposalRevision: '1' });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Rename all ready' })).toHaveTextContent('Rename all ready 2'));

    // The first rename is slow, and the NDA's name is typed while it runs.
    let finish = () => {};
    const approve = bridge.approve.getMockImplementation()!;
    bridge.approve.mockImplementationOnce(async (...call) => { await new Promise<void>((resolve) => { finish = resolve; }); return approve(...call); });
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(1));
    await openDetails(NDA);
    fireEvent.change(filename(), { target: { value: '2024-03-01 Non-Disclosure Agreement with Acme Corporation' } });
    await act(async () => { finish(); await new Promise((resolve) => setTimeout(resolve, 20)); });

    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed 1 document. 2 were left for you: they have a name or a description you typed and did not file.'));
    expect(bridge.approve.mock.calls.map(([id]) => id)).toEqual(['employment']);
    // The NDA is still on screen with what was typed, and the Service Agreement shows what was typed for it beside its new reading.
    expect(filename()).toHaveValue('2024-03-01 Non-Disclosure Agreement with Acme Corporation');
    selectRow(SERVICE);
    await waitFor(() => expect(panel()).toHaveTextContent(SERVICE));
    expect(kept()).toHaveValue('2024-02-28 Service Agreement with BlueSky LLC\n\nSigned by both; the schedule is missing.');
    expect(filename()).toHaveValue('2024-02-28 Services Agreement with BlueSky');
  });

  /*
    A drawer is asked for at the narrow width it opens at. Asked for once, it
    opened again by itself each time the window was next made narrow, over
    whatever row was selected by then, with the caret in that row's name -
    even after Close review at the wide width. The Enter meant for the row's
    cell, which opens details there, filed the document.
  */
  it('is not started by a drawer opening by itself: asked for once, the drawer does not open again when the window is next made narrow', async () => {
    const window_ = windowOf(true);
    const { bridge } = await start();
    await openDetails(LEASE);
    expect(panel()).toHaveAttribute('role', 'dialog');

    // Wider, then the panel opened and closed there, and another row chosen.
    window_.resize('wide');
    expect(panel()).not.toBeInTheDocument();
    fireEvent.click(detailsToggle());
    expect(panel()).toHaveAttribute('role', 'complementary');
    fireEvent.click(within(panel()!).getByRole('button', { name: 'Close review' }));
    const cell = queueCell(EMPLOYMENT);
    clickCell(cell);
    cell.focus();

    window_.resize('narrow');
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(panel()).not.toBeInTheDocument();
    expect(cell).toHaveFocus();
    expect(screen.getByRole('region', { name: 'Queue items' })).not.toHaveAttribute('inert');
    // Enter on the cell opens the document's details, as it is meant to, and files nothing.
    fireEvent.keyDown(cell, { key: 'Enter' });
    expect(await screen.findByRole('dialog', { name: 'Review item' })).toHaveTextContent(EMPLOYMENT);
    expect(bridge.approve).not.toHaveBeenCalled();
  });
});
